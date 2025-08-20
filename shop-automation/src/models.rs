use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProductOption {
    pub name: String,
    pub url: String,
    pub price: String,
    pub price_per_unit: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShoppingData {
    pub supervalu: HashMap<String, ProductOption>,
}

impl ShoppingData {
    pub fn new() -> Self {
        Self {
            supervalu: HashMap::new(),
        }
    }

    pub fn add_option(&mut self, index: usize, option: ProductOption) {
        let key = format!("opt_{}", index);
        self.supervalu.insert(key, option);
    }
}

// Shopping list item from YAML input
#[derive(Debug, Deserialize)]
pub struct ShoppingItem {
    pub name: String,
    pub amount: Option<String>,
    pub link: String,
    pub backup_link: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShoppingList {
    pub items: Vec<ShoppingItem>,
}