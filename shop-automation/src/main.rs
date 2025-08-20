use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, BufRead, Read};

mod scraper;
mod models;
mod shopper;

#[derive(Parser)]
#[command(name = "shop-automation")]
#[command(about = "SuperValu product scraper and database manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scrape product information from SuperValu
    Scrape {
        /// Base path for the database (default: ../config/db)
        #[arg(long, default_value = "../config/db")]
        db_path: String,
        
        /// Run in visible mode (show browser window)
        #[arg(long, short = 'v')]
        visible: bool,
    },
    
    /// Login to SuperValu and save session cookies
    Login {
        /// Run in visible mode (show browser window)
        #[arg(long, short = 'v')]
        visible: bool,
        
        /// Manual login - browser stays open for you to login yourself
        #[arg(long, short = 'm')]
        manual: bool,
    },
    
    /// Shop for items from a YAML shopping list
    Shop {
        /// Path to shopping list YAML file (use '-' for stdin)
        shopping_list: String,
        
        /// Run in visible mode (show browser window)
        #[arg(long, short = 'v')]
        visible: bool,
        
        /// Force fresh login even if cookies exist
        #[arg(long)]
        force_login: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scrape { db_path, visible } => {
            let stdin = io::stdin();
            let products: Vec<String> = stdin
                .lock()
                .lines()
                .filter_map(|line| line.ok())
                .filter(|line| !line.trim().is_empty())
                .collect();

            if products.is_empty() {
                eprintln!("No products provided. Please provide product names via stdin, one per line.");
                return Ok(());
            }

            println!("Starting to scrape {} products from SuperValu...", products.len());
            if visible {
                println!("Running in visible mode - browser windows will be shown");
            }
            
            for product in products {
                println!("Scraping: {}", product);
                match scraper::scrape_product(&product, &db_path, visible).await {
                    Ok(_) => println!("✓ Successfully scraped {}", product),
                    Err(e) => eprintln!("\x1b[31m✗ Failed to scrape {}: {}\x1b[0m", product, e),
                }
            }
        }
        Commands::Login { visible, manual } => {
            // Load environment variables
            dotenv::dotenv().ok();
            
            if manual {
                println!("🔐 Opening SuperValu for manual login...");
                shopper::manual_login_and_save_cookies()?;
            } else {
                println!("🔐 Logging in to SuperValu...");
                shopper::login_and_save_cookies(visible).await?;
            }
        }
        Commands::Shop { shopping_list: shopping_list_path, visible, force_login } => {
            // Load environment variables
            dotenv::dotenv().ok();
            
            // Read shopping list
            let input = if shopping_list_path == "-" {
                // Read from stdin
                let mut input = String::new();
                io::stdin().read_to_string(&mut input)?;
                input
            } else {
                // Read from file
                std::fs::read_to_string(&shopping_list_path)
                    .context(format!("Failed to read shopping list from {}", shopping_list_path))?
            };
            
            // Parse YAML
            let shopping_list: models::ShoppingList = serde_yaml::from_str(&input)
                .context("Failed to parse shopping list YAML")?;
            
            println!("🛒 Starting shopping automation with {} items", shopping_list.items.len());
            
            // Run shopping automation
            shopper::shop_items(shopping_list, visible, force_login).await?;
        }
    }

    Ok(())
}

use anyhow::Context;