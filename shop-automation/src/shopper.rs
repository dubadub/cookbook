use anyhow::{Context, Result, bail};
use headless_chrome::{Browser, LaunchOptions, Tab, protocol::cdp::Network};
use std::time::Duration;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::io;
use crate::models::{ShoppingList, ShoppingItem};
use serde::{Serialize, Deserialize};

const SUPERVALU_BASE_URL: &str = "https://shop.supervalu.ie";

#[derive(Debug, Serialize, Deserialize)]
struct Cookie {
    name: String,
    value: String,
    domain: String,
    path: String,
    expires: Option<f64>,
    size: u32,
    http_only: bool,
    secure: bool,
    session: bool,
    same_site: Option<String>,
}

fn get_cookie_file_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("shop-automation");
    fs::create_dir_all(&path).ok();
    path.push("supervalu_cookies.json");
    path
}

pub fn manual_login_and_save_cookies() -> Result<()> {
    // Launch browser in visible mode
    let launch_options = LaunchOptions {
        headless: false,
        sandbox: false,
        enable_gpu: false,
        enable_logging: false,
        idle_browser_timeout: Duration::from_secs(600),
        window_size: Some((1920, 1080)),
        ..Default::default()
    };
    
    let browser = Browser::new(launch_options)
        .context("Failed to launch Chrome browser")?;
    
    let tab = browser.new_tab()
        .context("Failed to create new tab")?;
    
    // Navigate to SuperValu
    println!("🌐 Opening SuperValu website...");
    tab.navigate_to(SUPERVALU_BASE_URL)?;
    std::thread::sleep(Duration::from_secs(3));
    
    // Handle cookies consent
    handle_cookies(&tab)?;
    
    println!("\n📝 Please login to SuperValu manually in the browser window.");
    println!("   When you're done logging in, press Enter here to save cookies...");
    
    // Wait for user to press Enter
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    // Check if logged in
    if !verify_logged_in(&tab)? {
        println!("⚠️  You don't appear to be logged in. Let me save cookies anyway...");
    }
    
    // Get all cookies
    let cookies = tab.get_cookies()?;
    
    // Convert to our Cookie struct and save
    let cookie_data: Vec<Cookie> = cookies.into_iter().map(|c| Cookie {
        name: c.name,
        value: c.value,
        domain: c.domain,
        path: c.path,
        expires: Some(c.expires),
        size: c.size,
        http_only: c.http_only,
        secure: c.secure,
        session: c.session,
        same_site: c.same_site.map(|s| format!("{:?}", s)),
    }).collect();
    
    // Save cookies to file
    let cookie_path = get_cookie_file_path();
    let json = serde_json::to_string_pretty(&cookie_data)?;
    fs::write(&cookie_path, json)?;
    
    println!("✅ Cookies saved to: {:?}", cookie_path);
    println!("   You can now use the 'shop' command.");
    
    Ok(())
}

pub async fn login_and_save_cookies(visible: bool) -> Result<()> {
    // Launch browser
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
        .context("Failed to launch Chrome browser")?;
    
    let tab = browser.new_tab()
        .context("Failed to create new tab")?;
    
    // Navigate to SuperValu
    tab.navigate_to(SUPERVALU_BASE_URL)?;
    std::thread::sleep(Duration::from_secs(3));
    
    // Handle cookies consent
    handle_cookies(&tab)?;
    
    // Perform login
    login_to_supervalu(&tab)?;
    
    // Get all cookies
    let cookies = tab.get_cookies()?;
    
    // Convert to our Cookie struct and save
    let cookie_data: Vec<Cookie> = cookies.into_iter().map(|c| Cookie {
        name: c.name,
        value: c.value,
        domain: c.domain,
        path: c.path,
        expires: Some(c.expires),
        size: c.size,
        http_only: c.http_only,
        secure: c.secure,
        session: c.session,
        same_site: c.same_site.map(|s| format!("{:?}", s)),
    }).collect();
    
    // Save cookies to file
    let cookie_path = get_cookie_file_path();
    let json = serde_json::to_string_pretty(&cookie_data)?;
    fs::write(&cookie_path, json)?;
    
    println!("✅ Login successful! Cookies saved to: {:?}", cookie_path);
    println!("   You can now use the 'shop' command without logging in each time.");
    
    // Keep browser open briefly in visible mode
    if visible {
        println!("\n   Browser will close in 5 seconds...");
        std::thread::sleep(Duration::from_secs(5));
    }
    
    Ok(())
}

pub async fn shop_items(shopping_list: ShoppingList, visible: bool, force_login: bool) -> Result<()> {
    // Launch browser
    let launch_options = LaunchOptions {
        headless: !visible,
        sandbox: false,
        enable_gpu: false,
        enable_logging: visible,
        idle_browser_timeout: Duration::from_secs(300),
        window_size: Some((1920, 1080)),
        ..Default::default()
    };
    
    let browser = Browser::new(launch_options)
        .context("Failed to launch Chrome browser")?;
    
    let tab = browser.new_tab()
        .context("Failed to create new tab")?;
    
    // Navigate to SuperValu
    println!("🌐 Navigating to SuperValu...");
    tab.navigate_to(SUPERVALU_BASE_URL)
        .context("Failed to navigate to SuperValu")?;
    
    std::thread::sleep(Duration::from_secs(3));
    
    // Handle cookie consent
    handle_cookies(&tab)?;
    
    // Load cookies or login
    if force_login {
        println!("🔐 Forcing fresh login...");
        login_to_supervalu(&tab)?;
        save_current_cookies(&tab)?;
    } else if !load_and_set_cookies(&tab)? {
        println!("🔐 No valid cookies found, logging in...");
        login_to_supervalu(&tab)?;
        save_current_cookies(&tab)?;
    } else {
        println!("🍪 Using saved cookies...");
        // Verify we're logged in
        if !verify_logged_in(&tab)? {
            println!("⚠️  Saved cookies expired, logging in again...");
            login_to_supervalu(&tab)?;
            save_current_cookies(&tab)?;
        } else {
            println!("✅ Successfully restored session");
        }
    }
    
    // Pause for delivery slot selection
    if visible {
        println!("\n📅 Please select your delivery slot in the browser.");
        println!("   Once you've selected a delivery slot, press Enter here to continue...");
        
        #[cfg(unix)]
        {
            use std::fs::File;
            use std::io::BufReader;
            use std::io::BufRead;
            
            let tty = File::open("/dev/tty")?;
            let mut reader = BufReader::new(tty);
            let mut input = String::new();
            reader.read_line(&mut input)?;
        }
        
        #[cfg(not(unix))]
        {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
        }
        
        println!("✅ Starting to add items to cart...\n");
    }
    
    // Add items to cart
    let mut added_count = 0;
    let mut failed_items: Vec<String> = Vec::new();
    
    for (index, item) in shopping_list.items.iter().enumerate() {
        println!("\n📦 [{}/{}] Processing: {}", index + 1, shopping_list.items.len(), item.name);
        if let Some(amount) = &item.amount {
            println!("   Amount needed: {}", amount);
        }
        
        // Check if link is empty
        if item.link.is_empty() {
            println!("   \x1b[33m⏭️  Skipping - no link provided\x1b[0m");
            failed_items.push(format!("{} (no link)", item.name));
            continue;
        }
        
        match add_item_to_cart(&tab, item) {
            Ok(true) => {
                added_count += 1;
                println!("   ✅ Added to cart");
            }
            Ok(false) => {
                println!("   ⚠️  Item might be out of stock");
                failed_items.push(item.name.clone());
            }
            Err(e) => {
                println!("   ❌ Failed: {}", e);
                failed_items.push(item.name.clone());
            }
        }
        
        // Small delay between items
        std::thread::sleep(Duration::from_secs(2));
    }
    
    // Show cart summary
    let failed_refs: Vec<&str> = failed_items.iter().map(|s| s.as_str()).collect();
    show_cart_summary(&tab, added_count, &failed_refs)?;
    
    // Keep browser open for manual checkout
    if visible {
        println!("\n🛒 Browser is ready for checkout.");
        println!("   Review your cart and complete your purchase.");
        println!("   Press Enter here when you're done to close the browser...");
        
        // Reopen stdin from /dev/tty to read user input
        // This is needed because stdin might have been consumed by the shopping list
        #[cfg(unix)]
        {
            use std::fs::File;
            use std::io::BufReader;
            use std::io::BufRead;
            
            let tty = File::open("/dev/tty")?;
            let mut reader = BufReader::new(tty);
            let mut input = String::new();
            reader.read_line(&mut input)?;
        }
        
        #[cfg(not(unix))]
        {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
        }
        
        println!("✅ Shopping session complete!");
    }
    
    Ok(())
}

fn load_and_set_cookies(tab: &Tab) -> Result<bool> {
    let cookie_path = get_cookie_file_path();
    
    if !cookie_path.exists() {
        return Ok(false);
    }
    
    let json = fs::read_to_string(&cookie_path)?;
    let cookies: Vec<Cookie> = serde_json::from_str(&json)?;
    
    // Set each cookie
    for cookie in cookies {
        let js = format!(
            r#"document.cookie = "{}={}; domain={}; path={}; {}{}";"#,
            cookie.name,
            cookie.value,
            cookie.domain,
            cookie.path,
            if cookie.secure { "secure; " } else { "" },
            if let Some(ref same_site) = cookie.same_site {
                format!("SameSite={};", same_site)
            } else {
                String::new()
            }
        );
        tab.evaluate(&js, false)?;
    }
    
    // Refresh page to apply cookies
    tab.reload(false, None)?;
    std::thread::sleep(Duration::from_secs(3));
    
    Ok(true)
}

fn save_current_cookies(tab: &Tab) -> Result<()> {
    let cookies = tab.get_cookies()?;
    
    let cookie_data: Vec<Cookie> = cookies.into_iter().map(|c| Cookie {
        name: c.name,
        value: c.value,
        domain: c.domain,
        path: c.path,
        expires: Some(c.expires),
        size: c.size,
        http_only: c.http_only,
        secure: c.secure,
        session: c.session,
        same_site: c.same_site.map(|s| format!("{:?}", s)),
    }).collect();
    
    let cookie_path = get_cookie_file_path();
    let json = serde_json::to_string_pretty(&cookie_data)?;
    fs::write(&cookie_path, json)?;
    
    Ok(())
}

fn verify_logged_in(tab: &Tab) -> Result<bool> {
    let check_login = r#"
        (() => {
            // Check for indicators that we're logged in
            const logoutBtn = document.querySelector('button[aria-label*="Log out"], a[href*="logout"]');
            const userMenu = document.querySelector('[class*="user"], [class*="account"], [aria-label*="Account"]');
            const signInBtn = document.querySelector('button[aria-label*="Sign in"], a[href*="login"]');
            
            // If we see a sign in button, we're definitely not logged in
            if (signInBtn) return false;
            
            // If we see logout or user menu, we're logged in
            return !!(logoutBtn || userMenu);
        })()
    "#;
    
    let result = tab.evaluate(check_login, false)?;
    Ok(matches!(result.value, Some(serde_json::Value::Bool(true))))
}

fn handle_cookies(tab: &Tab) -> Result<()> {
    println!("🍪 Handling cookie consent...");
    let cookie_js = r#"
        (() => {
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
                        return true;
                    }
                } catch (e) {}
            }
            
            const allButtons = document.querySelectorAll('button');
            for (const btn of allButtons) {
                if (btn.textContent.toLowerCase().includes('accept all') || 
                    btn.textContent.toLowerCase().includes('accept cookies')) {
                    btn.click();
                    return true;
                }
            }
            
            return false;
        })()
    "#;
    
    let _ = tab.evaluate(cookie_js, false);
    std::thread::sleep(Duration::from_secs(2));
    Ok(())
}

fn login_to_supervalu(tab: &Tab) -> Result<()> {
    // Get credentials from environment
    let email = env::var("SUPERVALU_EMAIL")
        .context("SUPERVALU_EMAIL not found in environment. Please set it in .env file")?;
    let password = env::var("SUPERVALU_PASSWORD")
        .context("SUPERVALU_PASSWORD not found in environment. Please set it in .env file")?;
    
    println!("🔐 Logging in to SuperValu...");
    
    // Navigate to login page
    tab.navigate_to(&format!("{}/login", SUPERVALU_BASE_URL))?;
    std::thread::sleep(Duration::from_secs(3));
    
    // Fill in login form
    let login_js = format!(r#"
        (() => {{
            // Find email input
            const emailInput = document.querySelector('input[type="email"], input[name="email"], input[id*="email"], input[placeholder*="email"]');
            if (emailInput) {{
                emailInput.value = '{}';
                emailInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                emailInput.dispatchEvent(new Event('change', {{ bubbles: true }}));
            }}
            
            // Find password input
            const passwordInput = document.querySelector('input[type="password"], input[name="password"], input[id*="password"]');
            if (passwordInput) {{
                passwordInput.value = '{}';
                passwordInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                passwordInput.dispatchEvent(new Event('change', {{ bubbles: true }}));
            }}
            
            // Find and click login button
            setTimeout(() => {{
                const loginButton = document.querySelector('button[type="submit"], button[class*="login"], button[aria-label*="Sign in"], button[aria-label*="Log in"]');
                if (loginButton) {{
                    loginButton.click();
                }}
            }}, 500);
            
            return true;
        }})()
    "#, email.replace("'", "\\'"), password.replace("'", "\\'"));
    
    tab.evaluate(&login_js, false)?;
    
    // Wait for login to complete
    std::thread::sleep(Duration::from_secs(5));
    
    // Check if login was successful
    if verify_logged_in(&tab)? {
        println!("✅ Successfully logged in");
    } else {
        bail!("Login failed. Please check your credentials.");
    }
    
    Ok(())
}

fn add_item_to_cart(tab: &Tab, item: &ShoppingItem) -> Result<bool> {
    // Check if primary link is valid
    if !item.link.is_empty() {
        if add_product_by_url(tab, &item.link)? {
            return Ok(true);
        }
    }
    
    // If primary failed and we have a backup, try it
    if let Some(backup_link) = &item.backup_link {
        if !backup_link.is_empty() {
            println!("   🔄 Primary product unavailable, trying backup...");
            if add_product_by_url(tab, backup_link)? {
                return Ok(true);
            }
        } else {
            println!("   ⚠️  Backup link is also empty");
        }
    }
    
    Ok(false)
}

fn add_product_by_url(tab: &Tab, url: &str) -> Result<bool> {
    // Validate URL
    if url.is_empty() {
        return Ok(false);
    }
    
    // Ensure URL is complete
    let full_url = if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else if url.starts_with("/") {
        format!("{}{}", SUPERVALU_BASE_URL, url)
    } else {
        // Invalid URL format
        println!("   ⚠️  Invalid URL format: {}", url);
        return Ok(false);
    };
    
    // Navigate to product page
    tab.navigate_to(&full_url)?;
    std::thread::sleep(Duration::from_secs(3));
    
    // Try to find and click "Add to Cart" button
    let add_to_cart_js = r#"
        (() => {
            // Look for add to cart button with various selectors
            const selectors = [
                'button[aria-label*="Add to Trolley"]',
                'button[aria-label*="Add to Cart"]',
                'button[data-testid*="addToCart"]',
                'button[class*="AddToCart"]',
                'button:has-text("Add to Trolley")',
                'button:has-text("Add to Cart")'
            ];
            
            for (const selector of selectors) {
                try {
                    const btn = document.querySelector(selector);
                    if (btn && !btn.disabled) {
                        // Check if it's not already in cart
                        const btnText = btn.textContent.toLowerCase();
                        if (btnText.includes('add')) {
                            btn.click();
                            return 'added';
                        } else if (btnText.includes('update') || btnText.includes('quantity')) {
                            return 'already_in_cart';
                        }
                    }
                } catch (e) {}
            }
            
            // Check if item is out of stock
            const outOfStock = document.querySelector('[class*="out-of-stock"], [class*="OutOfStock"], [aria-label*="Out of stock"]');
            if (outOfStock) {
                return 'out_of_stock';
            }
            
            return 'not_found';
        })()
    "#;
    
    let result = tab.evaluate(add_to_cart_js, false)?;
    
    if let Some(value) = result.value {
        match value.as_str() {
            Some("added") => {
                std::thread::sleep(Duration::from_secs(1));
                return Ok(true);
            }
            Some("already_in_cart") => {
                println!("   ℹ️  Item already in cart");
                return Ok(true);
            }
            Some("out_of_stock") => {
                println!("   ⚠️  Item is out of stock");
                return Ok(false);
            }
            _ => return Ok(false),
        }
    }
    
    Ok(false)
}

fn show_cart_summary(tab: &Tab, added_count: usize, failed_items: &[&str]) -> Result<()> {
    println!("\n{}", "=".repeat(60));
    println!("📊 SHOPPING SUMMARY");
    println!("{}", "=".repeat(60));
    
    // Navigate to cart page
    tab.navigate_to(&format!("{}/cart", SUPERVALU_BASE_URL))?;
    std::thread::sleep(Duration::from_secs(3));
    
    // Get cart details
    let cart_info_js = r#"
        (() => {
            const result = {
                itemCount: 0,
                subtotal: '',
                items: []
            };
            
            // Try to find item count
            const countEl = document.querySelector('[class*="cart-count"], [class*="CartCount"], [aria-label*="items in cart"]');
            if (countEl) {
                const match = countEl.textContent.match(/\d+/);
                if (match) result.itemCount = parseInt(match[0]);
            }
            
            // Try to find subtotal
            const subtotalEl = document.querySelector('[class*="subtotal"], [class*="Subtotal"], [class*="total-price"]');
            if (subtotalEl) {
                result.subtotal = subtotalEl.textContent.trim();
            }
            
            // Get cart items
            const cartItems = document.querySelectorAll('[class*="cart-item"], [class*="CartItem"], article[data-testid*="cart"]');
            cartItems.forEach(item => {
                const nameEl = item.querySelector('h3, h4, [class*="product-name"], [class*="ProductName"]');
                const priceEl = item.querySelector('[class*="price"], [class*="Price"]');
                const quantityEl = item.querySelector('input[type="number"], [class*="quantity"], select');
                
                if (nameEl) {
                    result.items.push({
                        name: nameEl.textContent.trim(),
                        price: priceEl ? priceEl.textContent.trim() : '',
                        quantity: quantityEl ? (quantityEl.value || quantityEl.textContent.trim()) : '1'
                    });
                }
            });
            
            return JSON.stringify(result);
        })()
    "#;
    
    let cart_result = tab.evaluate(cart_info_js, false)?;
    
    if let Some(value) = cart_result.value {
        if let Some(json_str) = value.as_str() {
        #[derive(serde::Deserialize)]
        struct CartInfo {
            #[serde(rename = "itemCount")]
            item_count: usize,
            subtotal: String,
            items: Vec<CartItem>,
        }
        
        #[derive(serde::Deserialize)]
        struct CartItem {
            name: String,
            price: String,
            quantity: String,
        }
        
        if let Ok(cart_info) = serde_json::from_str::<CartInfo>(json_str) {
            println!("\n✅ Successfully added: {} items", added_count);
            
            if !failed_items.is_empty() {
                // Separate items with no links from other failures
                let no_link_items: Vec<&&str> = failed_items.iter()
                    .filter(|item| item.contains("(no link)"))
                    .collect();
                let other_failed: Vec<&&str> = failed_items.iter()
                    .filter(|item| !item.contains("(no link)"))
                    .collect();
                
                if !no_link_items.is_empty() {
                    println!("\n\x1b[33m⏭️  Skipped {} items (no links provided):\x1b[0m", no_link_items.len());
                    for item in no_link_items {
                        let clean_name = item.replace(" (no link)", "");
                        println!("   - {}", clean_name);
                    }
                }
                
                if !other_failed.is_empty() {
                    println!("\n❌ Failed to add {} items:", other_failed.len());
                    for item in other_failed {
                        println!("   - {}", item);
                    }
                }
            }
            
            if !cart_info.items.is_empty() {
                println!("\n🛒 Cart Contents:");
                for item in &cart_info.items {
                    println!("   • {} (qty: {}) - {}", item.name, item.quantity, item.price);
                }
            }
            
            if !cart_info.subtotal.is_empty() {
                println!("\n💰 Subtotal: {}", cart_info.subtotal);
            }
        }
        }
    }
    
    println!("\n{}", "=".repeat(60));
    
    Ok(())
}