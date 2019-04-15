use crate::prelude::{ErrorVariant, Product, ProductAmount, Promotion};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Database {
    hm_product: Arc<Mutex<HashMap<String, Product>>>,
    hm_promotion: Arc<Mutex<HashMap<String, Promotion>>>,
}

impl Database {
    /// Data storage
    ///
    /// # Example
    ///
    /// ```
    /// use store_terminal::prelude::*;
    ///
    /// let mut database = Database::new();
    ///
    /// database.append(Product::new("Foo".to_string(), 1.0)).unwrap();
    /// database.append(Product::new("Bar".to_string(), 2.0)).unwrap();
    ///
    /// let promotion_code = String::from("Some Promotion");
    ///
    /// let products = vec![
    ///     database.code_to_product_amount("Foo".to_string(), 2.0).unwrap(),
    ///     database.code_to_product_amount("Bar".to_string(), 1.0).unwrap(),
    /// ];
    /// let promotion = Promotion::new("Some Promotion".to_string(), products, 5.0).unwrap();
    /// database.append(promotion).unwrap();
    ///
    /// let promotion = database.fetch_promotion(&promotion_code).unwrap();
    /// assert_eq!(promotion.get_price(), &5.0);
    ///
    /// let mut v_base = vec![];
    /// v_base.push(ProductAmount::new(Product::new("Bar".to_string(), 2.0), 1.0));
    /// v_base.push(ProductAmount::new(Product::new("Foo".to_string(), 1.0), 2.0));
    ///
    /// promotion
    ///     .get_products()
    ///     .iter()
    ///     .enumerate()
    ///     .for_each(|(key, &prod)| assert_eq!(prod, &v_base[key]));
    /// ```
    pub fn new() -> Self {
        let hm_product = Arc::new(Mutex::new(HashMap::new()));
        let hm_promotion = Arc::new(Mutex::new(HashMap::new()));

        Database {
            hm_product,
            hm_promotion,
        }
    }

    pub fn code_to_product_amount(
        &self,
        code: String,
        amount: f64,
    ) -> Result<ProductAmount, ErrorVariant> {
        let product = self.fetch_product(&code)?;
        let product_amount = ProductAmount::new(product, amount);
        Ok(product_amount)
    }

    pub fn fetch_promotion(&self, code: &String) -> Result<Promotion, ErrorVariant> {
        let promotion = {
            self.hm_promotion
                .lock()
                .map_err(|_| ErrorVariant::ArcUnlockError)?
                .get(code)
                .map(|p| Ok(p))
                .unwrap_or(Err(ErrorVariant::PromotionNotFound))?
                .clone()
        };

        Ok(promotion)
    }

    pub fn fetch_product(&self, code: &String) -> Result<Product, ErrorVariant> {
        let product = {
            self.hm_product
                .lock()
                .map_err(|_| ErrorVariant::ArcUnlockError)?
                .get(code)
                .map(|p| Ok(p))
                .unwrap_or(Err(ErrorVariant::ProductNotFound))?
                .clone()
        };

        Ok(product)
    }

    pub fn fetch_products(&self, products_code: Vec<String>) -> Result<Vec<Product>, ErrorVariant> {
        let mut products = vec![];

        for c in products_code {
            products.push(self.fetch_product(&c)?);
        }

        Ok(products)
    }

    pub fn fetch_possible_promotions(
        &self,
        products: &Vec<&ProductAmount>,
    ) -> Result<Vec<Promotion>, ErrorVariant> {
        self.fetch_possible_promotions_with_maximum_price(products, std::f64::INFINITY)
    }

    /// Return all possible promotions for a given set of products
    ///
    /// # Example
    ///
    /// ```
    /// use store_terminal::prelude::*;
    /// use std::iter;
    ///
    /// let mut database = Database::new();
    ///
    /// database.append(Product::new("A".to_string(), 2.0)).unwrap();
    /// database.append(Product::new("B".to_string(), 12.0)).unwrap();
    /// database.append(Product::new("C".to_string(), 1.25)).unwrap();
    /// database.append(Product::new("D".to_string(), 0.15)).unwrap();
    ///
    /// let products = vec![database.code_to_product_amount("A".to_string(), 4.0).unwrap()];
    /// let promotion = Promotion::new("PA".to_string(), products, 7.0).unwrap();
    /// database.append(promotion).unwrap();
    ///
    /// let products = vec![database.code_to_product_amount("C".to_string(), 6.0).unwrap()];
    /// let promotion = Promotion::new("PC".to_string(), products, 6.0).unwrap();
    /// database.append(promotion).unwrap();
    ///
    /// let mut products = vec![];
    /// products.push(
    ///     database
    ///         .fetch_product(&"A".to_string())
    ///         .unwrap()
    ///         .generate_amount(9.0),
    /// );
    /// products.push(
    ///     database
    ///         .fetch_product(&"C".to_string())
    ///         .unwrap()
    ///         .generate_amount(9.0),
    /// );
    /// let param: Vec<&ProductAmount> = products.iter().collect();
    /// let mut possible = database
    ///     .fetch_possible_promotions_with_maximum_price(&param, 6.5)
    ///     .unwrap();
    /// let expect = database.fetch_promotion(&"PC".to_string()).unwrap();
    /// assert_eq!(possible.pop().unwrap(), expect);
    /// ```
    pub fn fetch_possible_promotions_with_maximum_price(
        &self,
        products: &Vec<&ProductAmount>,
        maximum_price: f64,
    ) -> Result<Vec<Promotion>, ErrorVariant> {
        Ok(self
            .hm_promotion
            .lock()
            .map_err(|_| ErrorVariant::ArcUnlockError)?
            .values()
            .filter(|promotion| {
                promotion.get_price() < &maximum_price && promotion.is_contained_by(products)
            })
            .map(|p| p.clone())
            .collect())
    }

    pub fn reset(&self) -> Result<(), ErrorVariant> {
        {
            self.hm_product
                .lock()
                .map_err(|_| ErrorVariant::ArcUnlockError)
                .and_then(|mut hm_product| Ok(hm_product.clear()))?;
        }
        {
            self.hm_promotion
                .lock()
                .map_err(|_| ErrorVariant::ArcUnlockError)
                .and_then(|mut hm_promotion| Ok(hm_promotion.clear()))?;
        }
        Ok(())
    }
}

pub trait DatabaseAppend<T> {
    fn append(&self, entity: T) -> Result<(), ErrorVariant>;
}

impl DatabaseAppend<Product> for Database {
    fn append(&self, entity: Product) -> Result<(), ErrorVariant> {
        let code = entity.get_code().clone();

        {
            self.hm_product
                .lock()
                .map_err(|_| ErrorVariant::ArcUnlockError)
                .and_then(|mut hm_product| Ok(hm_product.insert(code, entity)))?;
        }

        Ok(())
    }
}

impl DatabaseAppend<Promotion> for Database {
    fn append(&self, entity: Promotion) -> Result<(), ErrorVariant> {
        let code = entity.get_code().clone();

        {
            self.hm_promotion
                .lock()
                .map_err(|_| ErrorVariant::ArcUnlockError)
                .and_then(|mut hm_promotion| Ok(hm_promotion.insert(code, entity)))?;
        }

        Ok(())
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let buffer = {
            self.hm_promotion
                .lock()
                .map_err(|_| fmt::Error)?
                .values()
                .fold(String::from(""), |b, p| format!("{}\n{:?}", b, p))
        };
        let buffer = {
            self.hm_product
                .lock()
                .map_err(|_| fmt::Error)?
                .values()
                .fold(buffer, |b, p| format!("{}\n{:?}", b, p))
        };
        write!(f, "{}", buffer)
    }
}
