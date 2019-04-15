use crate::prelude::{
    CartItem, CartItemVariant, ErrorVariant, ProductAmount, TerminalEntityInterface, WithNewPricing,
};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use uuid::Uuid;

pub mod extra;
pub mod fut;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    code: String,
    price: f64,
}

impl Product {
    /// Instantiate a new product
    ///
    /// # Example
    ///
    /// ```
    /// use store_terminal::prelude::*;
    ///
    /// let p1 = Product::new("Foo".to_string(), 15.0);
    /// let p2 = Product::new("Bar".to_string(), 20.0);
    /// let p3 = Product::new("Foo".to_string(), 15.0);
    ///
    /// assert!(p1 != p2);
    /// assert!(p1 == p3);
    /// ```
    pub fn new(code: String, price: f64) -> Self {
        Product { code, price }
    }

    pub fn get_code(&self) -> &String {
        &self.code
    }

    pub fn get_price(&self) -> &f64 {
        &self.price
    }

    pub fn generate_amount(&self, amount: f64) -> ProductAmount {
        ProductAmount::new(self.clone(), amount)
    }
}

impl Ord for Product {
    fn cmp(&self, other: &Product) -> Ordering {
        self.code.cmp(&other.code)
    }
}

impl PartialOrd for Product {
    fn partial_cmp(&self, other: &Product) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Product {
    fn eq(&self, other: &Product) -> bool {
        self.code == other.code
    }
}

impl Eq for Product {}

#[derive(Debug, Clone)]
pub struct CartItemProduct {
    id: Uuid,
    product_amount: ProductAmount,
}

impl CartItemProduct {
    pub fn new(product: Product, amount: f64) -> Self {
        let product_amount = ProductAmount::new(product, amount);
        let id = Uuid::new_v4();

        CartItemProduct { id, product_amount }
    }
}

impl fmt::Display for CartItemProduct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl CartItem for CartItemProduct {
    fn get_id(&self) -> &Uuid {
        &self.id
    }

    fn get_products(&self) -> Vec<&ProductAmount> {
        vec![&self.product_amount]
    }

    fn get_amount(&self) -> f64 {
        *self.product_amount.get_amount()
    }

    fn get_variant<'a>(&self) -> CartItemVariant {
        CartItemVariant::Product(&self)
    }
}

impl WithNewPricing for Product {
    fn with_new_pricing(&self, price: f64) -> Result<Self, ErrorVariant> {
        let code = self.get_code().clone();
        let product = Product::new(code, price);
        Ok(product)
    }
}

impl TerminalEntityInterface for Product {
    fn get_syntax_example() -> &'static str {
        r#"{code: "A", price: 15.3}"#
    }

    fn from_json(json: String) -> Result<Self, ErrorVariant> {
        serde_json::from_str::<Product>(json.as_str()).map_err(|_| ErrorVariant::JsonParseError)
    }

    fn to_json(&self) -> Result<String, ErrorVariant> {
        serde_json::to_string(&self).map_err(|_| ErrorVariant::JsonParseError)
    }
}
