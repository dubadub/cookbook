# SuperValu Shopping Automation

This tool provides automated shopping cart management for SuperValu Ireland, including product scraping and automated cart filling from shopping lists.

## Features

### 1. Product Scraping (`scrape` command)
- Scrapes product information from SuperValu website
- Extracts: name, URL, price, price per unit, and quantity/weight
- Saves data in YAML format for recipe database
- Skips products that already have data
- Shows missing products in red for easy identification

### 2. Login (`login` command)
- Logs into SuperValu account
- Saves session cookies for reuse
- Cookies stored locally for future shopping sessions

### 3. Shopping Automation (`shop` command)
- Uses saved cookies from login (no need to login each time)
- Pauses for delivery slot selection (press Enter when ready)
- Adds items from shopping list to cart
- Supports fallback products if primary is unavailable
- Shows cart summary with subtotal
- Waits for you to press Enter after checkout (in visible mode)

## Installation

1. Install Chrome/Chromium browser:
```bash
brew install --cask google-chrome
```

2. Build the project:
```bash
cargo build --release
```

3. Set up credentials (for shopping automation):
```bash
cp .env.example .env
# Edit .env and add your SuperValu credentials
```

## Usage

### Scraping Products

```bash
# Scrape products (headless mode)
echo -e "onions\ntomatoes\npotatoes" | cargo run -- scrape

# Scrape in visible mode (for debugging)
echo "apples" | cargo run -- scrape --visible

# Use custom database path
echo "carrots" | cargo run -- scrape --db-path /path/to/db
```

### Login (One-time setup)

```bash
# Manual login (RECOMMENDED) - Opens browser, you login yourself, then press Enter to save cookies
cargo run -- login --manual

# Automatic login (requires .env with credentials)
cargo run -- login

# Automatic login in visible mode to see the process
cargo run -- login --visible
```

### Automated Shopping

1. Generate shopping list using CookCLI:
```bash
cook report -t Reports/shopping-list.yaml.jinja -d ./config/db Recipe.cook > shopping-list.yaml
```

2. Run shopping automation:
```bash
# Add items to cart from file (uses saved cookies from login)
cargo run -- shop shopping-list.yaml --visible

# Or pipe from command output (use '-' for stdin)
cook report -t Reports/shopping-list.yaml.jinja -d ./config/db Recipe.cook | cargo run -- shop - --visible

# Force fresh login (if cookies expired)
cargo run -- shop shopping-list.yaml --force-login --visible
```

### Shopping List Format

The shopping list should be in YAML format:
```yaml
items:
  - name: eggs
    amount: 3
    link: https://shop.supervalu.ie/sm/delivery/rsid/404/product/...
    backup_link: https://shop.supervalu.ie/sm/delivery/rsid/404/product/...
  - name: milk
    amount: 250 ml
    link: https://shop.supervalu.ie/sm/delivery/rsid/404/product/...
    backup_link: https://shop.supervalu.ie/sm/delivery/rsid/404/product/...
```

## Environment Variables

Create a `.env` file with:
```
SUPERVALU_EMAIL=your_email@example.com
SUPERVALU_PASSWORD=your_password
```

## Output

### Scraping Output
Creates files at: `../config/db/[product_name]/shopping.yml`

Example:
```yaml
supervalu:
  opt_1:
    name: SuperValu Tomatoes
    url: https://shop.supervalu.ie/...
    price: €1.29
    price_per_unit: €0.22 each
    quantity: 6 Piece
```

### Shopping Workflow
1. Browser opens with your logged-in session
2. **Pause #1**: Select your delivery slot, then press Enter
3. Items are automatically added to cart
4. Cart summary is displayed
5. **Pause #2**: Complete checkout, then press Enter to close browser

### Shopping Output
- Shows progress for each item
- Indicates successful additions with ✅
- Shows failures/out of stock with ⚠️ or ❌
- Skips items with empty links (shown in yellow)
- Displays cart summary with total items and subtotal
- Browser stays open until you press Enter (complete checkout at your own pace)

## Troubleshooting

- **"Failed to launch Chrome browser"**: Install Chrome or Chromium
- **Red "No products found"**: Product doesn't exist or search term needs adjustment
- **Login fails**: Use `--manual` flag to login yourself, or check credentials in .env file
- **Items not adding to cart**: Product might be out of stock or page structure changed
- **Cookies expired**: Run `cargo run -- login --manual` to refresh session

## Notes

- The scraper handles cookie consent popups automatically
- Login cookies are saved locally and reused automatically
- Cookies are stored in your system's local data directory
- Shopping automation waits between items to avoid rate limiting
- In visible mode, browser stays open until you press Enter (no timeout)
- Complete checkout at your own pace - no rush!
- All prices and availability are subject to SuperValu's current stock

## Cookie Storage

Cookies are saved to:
- **macOS**: `~/Library/Application Support/shop-automation/supervalu_cookies.json`
- **Linux**: `~/.local/share/shop-automation/supervalu_cookies.json`
- **Windows**: `%APPDATA%\shop-automation\supervalu_cookies.json`