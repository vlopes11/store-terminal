use crate::prelude::{Cart, Database, DatabaseAppend, Product, Promotion};
use std::sync::{Arc, Mutex};

pub mod cart;
pub mod database;
pub mod prelude;
pub mod product;
pub mod promotion;

#[derive(Debug)]
pub enum ErrorVariant {
    ArcUnlockError,
    ProductNotFound,
    PromotionNotFound,
    NotEnoughItems,
    JsonParseError,
}

pub trait WithNewPricing: Sized {
    fn with_new_pricing(&self, price: f64) -> Result<Self, ErrorVariant>;
}

pub trait TerminalEntityInterface: Sized {
    fn get_syntax_example() -> &'static str;
    fn from_json(json: String) -> Result<Self, ErrorVariant>;
    fn to_json(&self) -> Result<String, ErrorVariant>;
}

pub struct Terminal {
    database: Database,
    cart: Arc<Mutex<Cart>>,
}

impl Terminal {
    pub fn new() -> Result<Self, ErrorVariant> {
        let database = Database::new();
        let cart = Arc::new(Mutex::new(Cart::new(database.clone())));

        let terminal = Terminal { cart, database };

        Ok(terminal)
    }

    /// Scanner interface
    ///
    /// # Example
    ///
    /// ```
    /// use store_terminal::prelude::*;
    ///
    /// let terminal = Terminal::new().unwrap();
    /// terminal.init().unwrap();
    ///
    /// terminal.scan("ABCDABAA".to_string()).unwrap();
    /// terminal.scan("CCCCCCC".to_string()).unwrap();
    ///
    /// assert_eq!(terminal.get_cart().unwrap().get_total_price(), 39.65);
    /// ```
    pub fn scan(&self, codes: String) -> Result<(), ErrorVariant> {
        let mut codes = codes;
        while let Some(c) = codes.pop() {
            print!("Scanning code {}...", c);
            {
                self.cart
                    .lock()
                    .map_err(|_| ErrorVariant::ArcUnlockError)
                    .and_then(|mut cart| Ok(cart.push_product(&c.to_string(), 1.0)))??;
            }
            println!("product inserted!")
        }
        Ok(())
    }

    pub fn init(&self) -> Result<(), ErrorVariant> {
        self.database.reset()?;
        {
            self.cart
                .lock()
                .map_err(|_| ErrorVariant::ArcUnlockError)
                .and_then(|mut cart| Ok(cart.reset()))??;
        }

        self.database.append(Product::new("A".to_string(), 2.0))?;
        self.database.append(Product::new("B".to_string(), 12.0))?;
        self.database.append(Product::new("C".to_string(), 1.25))?;
        self.database.append(Product::new("D".to_string(), 0.15))?;

        let products = vec![self.database.code_to_product_amount("A".to_string(), 4.0)?];
        self.database
            .append(Promotion::new("PA".to_string(), products, 7.0)?)?;

        let products = vec![self.database.code_to_product_amount("C".to_string(), 6.0)?];
        self.database
            .append(Promotion::new("PC".to_string(), products, 6.0)?)?;

        Ok(())
    }

    pub fn set_pricing<T: WithNewPricing>(&self, entity: T, price: f64) -> Result<(), ErrorVariant>
    where
        Database: DatabaseAppend<T>,
    {
        let entity = entity.with_new_pricing(price)?;
        self.database.append(entity)?;
        Ok(())
    }

    pub fn get_cart(&self) -> Result<Cart, ErrorVariant> {
        let cart = {
            self.cart
                .lock()
                .map_err(|_| ErrorVariant::ArcUnlockError)
                .and_then(|mut cart| Ok(cart.optimize_promotions()?.clone()))?
        };
        Ok(cart)
    }

    pub fn reset_cart(&self) -> Result<(), ErrorVariant> {
        {
            self.cart
                .lock()
                .map_err(|_| ErrorVariant::ArcUnlockError)
                .and_then(|mut c| Ok(c.reset()))??;
        }
        Ok(())
    }

    pub fn get_db(&self) -> Result<&Database, ErrorVariant> {
        Ok(&self.database)
    }
}
