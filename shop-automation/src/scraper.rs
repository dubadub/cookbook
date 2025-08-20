use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptions};
use std::path::Path;
use std::fs;
use std::time::Duration;
use crate::models::{ProductOption, ShoppingData};

const SUPERVALU_BASE_URL: &str = "https://shop.supervalu.ie";

pub async fn scrape_product(product_name: &str, db_path: &str, visible: bool) -> Result<()> {
    // Check if shopping.yml already exists for this product
    let clean_name = product_name
        .to_lowercase()
        .replace(" ", "_")
        .replace("/", "_")
        .replace("\\", "_");
    
    let yaml_path = Path::new(db_path).join(&clean_name).join("shopping.yml");
    
    if yaml_path.exists() {
        println!("⏭ Skipping {} - shopping.yml already exists", product_name);
        return Ok(());
    }
    
    // Launch browser with appropriate options
    let launch_options = LaunchOptions {
        headless: !visible,
        sandbox: false,
        enable_gpu: false,
        enable_logging: visible,
        idle_browser_timeout: Duration::from_secs(120),
        window_size: Some((1920, 1080)),
        ..Default::default()
    };
    
    let browser = Browser::new(launch_options)
        .context("Failed to launch Chrome browser. Please ensure Google Chrome or Chromium is installed. On macOS, you can install it via: brew install --cask google-chrome")?;
    
    let tab = browser.new_tab()
        .context("Failed to create new tab")?;
    
    // Navigate to search page
    let search_url = format!(
        "{}/sm/delivery/rsid/404/results?q={}",
        SUPERVALU_BASE_URL,
        urlencoding::encode(product_name)
    );
    
    println!("🔗 Navigating to: {}", search_url);
    tab.navigate_to(&search_url)
        .context("Failed to navigate to search page")?;
    
    // Wait for page to load
    println!("⏳ Waiting for page to load...");
    std::thread::sleep(Duration::from_secs(3));
    
    // Handle cookie consent popup
    println!("🍪 Handling cookie consent...");
    let cookie_js = r#"
        (() => {
            // Try to find and click accept cookies button
            const acceptButtons = [
                'button[id*="accept"]',
                'button[class*="accept"]',
                'button[aria-label*="Accept"]',
                '#onetrust-accept-btn-handler',
                '.onetrust-close-btn-handler',
                '[aria-label="Accept cookies"]'
            ];
            
            for (const selector of acceptButtons) {
                try {
                    const btn = document.querySelector(selector);
                    if (btn && (btn.textContent.toLowerCase().includes('accept') || 
                               btn.getAttribute('aria-label')?.toLowerCase().includes('accept'))) {
                        btn.click();
                        console.log('Clicked accept cookies button');
                        return true;
                    }
                } catch (e) {}
            }
            
            // Also try to find buttons by text content
            const allButtons = document.querySelectorAll('button');
            for (const btn of allButtons) {
                if (btn.textContent.toLowerCase().includes('accept all') || 
                    btn.textContent.toLowerCase().includes('accept cookies')) {
                    btn.click();
                    console.log('Clicked accept cookies button by text');
                    return true;
                }
            }
            
            return false;
        })()
    "#;
    
    let _ = tab.evaluate(cookie_js, false);
    std::thread::sleep(Duration::from_secs(2));
    
    // Wait for products to load - using the actual selector from the HTML
    println!("🔍 Waiting for products to load...");
    let _ = tab.wait_for_element_with_custom_timeout("article[data-testid*='ProductCardWrapper']", Duration::from_secs(10));
    
    // Additional wait for dynamic content
    std::thread::sleep(Duration::from_secs(2));
    
    // Extract product information using SuperValu's actual selectors
    let products = extract_products_supervalu(&tab)?;
    
    // Keep browser open for inspection in visible mode
    if visible && products.is_empty() {
        println!("🔍 No products found. Browser will stay open for 15 seconds for inspection...");
        std::thread::sleep(Duration::from_secs(15));
    }
    
    // Save to YAML file
    if !products.is_empty() {
        save_to_yaml(product_name, products, db_path)?;
    } else {
        // Print in red using ANSI escape codes
        println!("\x1b[31m⚠ No products found for: {}\x1b[0m", product_name);
    }
    
    Ok(())
}

fn extract_products_supervalu(tab: &headless_chrome::Tab) -> Result<Vec<ProductOption>> {
    // SuperValu specific extraction using their actual HTML structure
    let js_code = r#"
        (() => {
            const products = [];
            
            // Find all product cards using SuperValu's actual selectors
            const productCards = document.querySelectorAll('article[data-testid*="ProductCardWrapper"]');
            
            console.log(`Found ${productCards.length} product cards`);
            
            // Extract data from first 3 products
            for (let i = 0; i < Math.min(productCards.length, 3); i++) {
                const card = productCards[i];
                
                // Extract product name from the title span
                const titleEl = card.querySelector('.ProductCardTitle--1ln1u3g, [data-testid*="ProductNameTestId"]');
                let fullName = '';
                if (titleEl) {
                    // Remove the "Open product description" text that's added for accessibility
                    fullName = titleEl.textContent.trim().replace('Open product description', '').trim();
                }
                
                // If no name found, try the aria label
                if (!fullName) {
                    const ariaTitle = card.querySelector('.AriaProductTitle--1axj7ma p');
                    if (ariaTitle) {
                        // Get the first part before the price
                        const text = ariaTitle.textContent;
                        const match = text.match(/^([^,€]+)/);
                        if (match) {
                            fullName = match[1].trim();
                        }
                    }
                }
                
                // Extract quantity/weight from the name
                let name = fullName;
                let quantity = null;
                
                // Common patterns for quantity/weight in SuperValu product names
                // Examples: "(1 kg)", "(250 g)", "(6 Piece)", "(2 Piece)", "(500 ml)"
                const quantityMatch = fullName.match(/\(([^)]+)\)$/);
                if (quantityMatch) {
                    quantity = quantityMatch[1];
                    // Remove the quantity from the name
                    name = fullName.replace(/\s*\([^)]+\)$/, '').trim();
                }
                
                // Extract URL from the hidden link
                const linkEl = card.querySelector('a.ProductCardHiddenLink--v3c62m, a[href*="/product/"]');
                let url = '';
                if (linkEl) {
                    url = linkEl.getAttribute('href');
                    if (url && !url.startsWith('http')) {
                        url = 'https://shop.supervalu.ie' + url;
                    }
                }
                
                // Extract price
                const priceEl = card.querySelector('.ProductCardPrice--1sznkcp, [data-testid="productCardPricing-div-testId"] span');
                let price = '';
                if (priceEl) {
                    price = priceEl.textContent.trim();
                }
                
                // Extract price per unit (often the same as price for weight-based items)
                const unitPriceEl = card.querySelector('.ProductCardPriceInfo--18y10ci');
                let unitPrice = '';
                if (unitPriceEl) {
                    unitPrice = unitPriceEl.textContent.trim();
                }
                
                // Only add if we have meaningful data
                if (name && (url || price)) {
                    products.push({
                        name: name,
                        url: url || '',
                        price: price || 'Price not available',
                        price_per_unit: unitPrice || price || '',
                        quantity: quantity
                    });
                    console.log(`Added product: ${name} (${quantity || 'no quantity'}) - ${price}`);
                }
            }
            
            return JSON.stringify({
                count: products.length,
                results: products
            });
        })()
    "#;
    
    let result = tab.evaluate(js_code, false)
        .context("Failed to extract products from page")?;
    
    if let Some(json_str) = result.value {
        let json_str = json_str.as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert result to string"))?;
        
        #[derive(serde::Deserialize)]
        struct ExtractResult {
            count: usize,
            results: Vec<ProductOption>,
        }
        
        let parsed: ExtractResult = serde_json::from_str(json_str)
            .context("Failed to parse product JSON")?;
        
        println!("📦 Found {} products", parsed.count);
        
        Ok(parsed.results)
    } else {
        Ok(Vec::new())
    }
}

fn save_to_yaml(product_name: &str, products: Vec<ProductOption>, db_path: &str) -> Result<()> {
    // Clean product name for directory
    let clean_name = product_name
        .to_lowercase()
        .replace(" ", "_")
        .replace("/", "_")
        .replace("\\", "_");
    
    // Create directory path
    let dir_path = Path::new(db_path).join(&clean_name);
    fs::create_dir_all(&dir_path)
        .context(format!("Failed to create directory: {:?}", dir_path))?;
    
    // Create shopping data
    let mut shopping_data = ShoppingData::new();
    for (i, product) in products.into_iter().enumerate() {
        shopping_data.add_option(i + 1, product);
    }
    
    // Write YAML file
    let yaml_path = dir_path.join("shopping.yml");
    let yaml_content = serde_yaml::to_string(&shopping_data)
        .context("Failed to serialize to YAML")?;
    
    fs::write(&yaml_path, yaml_content)
        .context(format!("Failed to write file: {:?}", yaml_path))?;
    
    println!("✓ Saved shopping data to: {:?}", yaml_path);
    
    Ok(())
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                ' ' => "%20".to_string(),
                '!' => "%21".to_string(),
                '"' => "%22".to_string(),
                '#' => "%23".to_string(),
                '$' => "%24".to_string(),
                '%' => "%25".to_string(),
                '&' => "%26".to_string(),
                '\'' => "%27".to_string(),
                '(' => "%28".to_string(),
                ')' => "%29".to_string(),
                '*' => "%2A".to_string(),
                '+' => "%2B".to_string(),
                ',' => "%2C".to_string(),
                '/' => "%2F".to_string(),
                ':' => "%3A".to_string(),
                ';' => "%3B".to_string(),
                '<' => "%3C".to_string(),
                '=' => "%3D".to_string(),
                '>' => "%3E".to_string(),
                '?' => "%3F".to_string(),
                '@' => "%40".to_string(),
                '[' => "%5B".to_string(),
                '\\' => "%5C".to_string(),
                ']' => "%5D".to_string(),
                '^' => "%5E".to_string(),
                '`' => "%60".to_string(),
                '{' => "%7B".to_string(),
                '|' => "%7C".to_string(),
                '}' => "%7D".to_string(),
                '~' => "%7E".to_string(),
                c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' => c.to_string(),
                _ => format!("%{:02X}", c as u8),
            })
            .collect()
    }
}