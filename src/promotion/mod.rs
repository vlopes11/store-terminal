use crate::prelude::{
    CartItem, CartItemVariant, ErrorVariant, ProductAmount, ProductAmountGroupFuture,
    TerminalEntityInterface, WithNewPricing,
};
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Promotion {
    code: String,
    products: Vec<ProductAmount>,
    price: f64,
}

impl Promotion {
    pub fn new(
        code: String,
        products: Vec<ProductAmount>,
        price: f64,
    ) -> Result<Self, ErrorVariant> {
        let products = ProductAmountGroupFuture::new(products).wait()?;
        let promotion = Promotion {
            code,
            products,
            price,
        };
        Ok(promotion)
    }

    pub fn get_code(&self) -> &String {
        &self.code
    }

    pub fn get_products(&self) -> Vec<&ProductAmount> {
        self.products.iter().collect()
    }

    pub fn get_price(&self) -> &f64 {
        &self.price
    }

    /// Check if the current promotion is contained by a set of [ProductAmount](crate::prelude::ProductAmount)
    ///
    /// Will assume the argument is optimized by [CartGroupFuture](crate::prelude::CartGroupFuture)
    ///
    /// # Example
    ///
    /// ```
    /// use store_terminal::prelude::*;
    ///
    /// let mut database = Database::new();
    ///
    /// database.append(Product::new("A".to_string(), 100.0)).unwrap();
    /// database.append(Product::new("B".to_string(), 100.0)).unwrap();
    /// database.append(Product::new("C".to_string(), 100.0)).unwrap();
    ///
    /// let products = vec![
    ///     database.code_to_product_amount("A".to_string(), 1.0).unwrap(),
    ///     database.code_to_product_amount("A".to_string(), 1.0).unwrap(),
    ///     database.code_to_product_amount("A".to_string(), 1.0).unwrap(),
    ///     database.code_to_product_amount("B".to_string(), 1.0).unwrap(),
    /// ];
    /// let promotion = Promotion::new("P1".to_string(), products, 1.0).unwrap();
    /// database.append(promotion).unwrap();
    ///
    /// let test_amount = vec![
    ///     database.fetch_product(&"A".to_string()).unwrap().generate_amount(2.0),
    ///     database.fetch_product(&"B".to_string()).unwrap().generate_amount(2.0),
    /// ];
    /// let mut assert_array = vec![];
    /// for t in &test_amount {
    ///     assert_array.push(t);
    /// }
    /// assert!(! database.fetch_promotion(&"P1".to_string()).unwrap().is_contained_by(&assert_array));
    ///
    /// let test_amount = vec![
    ///     database.fetch_product(&"A".to_string()).unwrap().generate_amount(3.0),
    ///     database.fetch_product(&"B".to_string()).unwrap().generate_amount(2.0),
    /// ];
    /// let mut assert_array = vec![];
    /// for t in &test_amount {
    ///     assert_array.push(t);
    /// }
    /// assert!(database.fetch_promotion(&"P1".to_string()).unwrap().is_contained_by(&assert_array));
    ///
    /// let test_amount = vec![
    ///     database.fetch_product(&"A".to_string()).unwrap().generate_amount(4.0),
    ///     database.fetch_product(&"B".to_string()).unwrap().generate_amount(2.0),
    /// ];
    /// let mut assert_array = vec![];
    /// for t in &test_amount {
    ///     assert_array.push(t);
    /// }
    /// assert!(database.fetch_promotion(&"P1".to_string()).unwrap().is_contained_by(&assert_array));
    /// ```
    pub fn is_contained_by(&self, products: &Vec<&ProductAmount>) -> bool {
        self.get_products()
            .iter()
            .fold(true, |is_contained, product| {
                if !is_contained {
                    return false;
                }

                for arg_prod in products {
                    if product.get_code() == arg_prod.get_code() {
                        return product.get_amount() <= arg_prod.get_amount();
                    }
                }

                false
            })
    }

    pub fn consume_items(
        &self,
        products: Vec<ProductAmount>,
    ) -> Result<Vec<ProductAmount>, ErrorVariant> {
        let mut products = products.clone();

        for p in &self.products {
            let index = ProductAmount::get_index_of_product(&products, p.get_code())?;
            products[index].dec_amount(*p.get_amount())?;
        }

        Ok(products
            .iter()
            .filter(|p| p.get_amount() > &0.0)
            .map(|p| p.clone())
            .collect())
    }
}

impl PartialEq for Promotion {
    fn eq(&self, other: &Promotion) -> bool {
        self.get_code() == other.get_code()
    }
}

impl Eq for Promotion {}

#[derive(Debug, Clone)]
pub struct CartItemPromotion {
    id: Uuid,
    promotion: Promotion,
    amount: f64,
}

impl CartItemPromotion {
    pub fn new(promotion: Promotion, amount: f64) -> Self {
        let id = Uuid::new_v4();

        CartItemPromotion {
            id,
            promotion,
            amount,
        }
    }
}

impl CartItem for CartItemPromotion {
    fn get_id(&self) -> &Uuid {
        &self.id
    }

    fn get_products(&self) -> Vec<&ProductAmount> {
        self.promotion.get_products()
    }

    fn get_amount(&self) -> f64 {
        self.amount
    }

    fn get_price(&self) -> f64 {
        *self.promotion.get_price()
    }

    fn get_variant<'a>(&self) -> CartItemVariant {
        CartItemVariant::Promotion(&self)
    }
}

impl fmt::Display for CartItemPromotion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl WithNewPricing for Promotion {
    fn with_new_pricing(&self, price: f64) -> Result<Self, ErrorVariant> {
        let code = self.get_code().clone();
        let products = self.get_products().iter().map(|&p| p.clone()).collect();
        let promotion = Promotion::new(code, products, price)?;
        Ok(promotion)
    }
}

impl TerminalEntityInterface for Promotion {
    fn get_syntax_example() -> &'static str {
        r#"{"code":"PA","products":[{"product":{"code":"A","price":2.0},"amount":4.0}],"price":7.0}"#
    }

    fn from_json(json: String) -> Result<Self, ErrorVariant> {
        serde_json::from_str::<Promotion>(json.as_str()).map_err(|_| ErrorVariant::JsonParseError)
    }

    fn to_json(&self) -> Result<String, ErrorVariant> {
        serde_json::to_string(&self).map_err(|_| ErrorVariant::JsonParseError)
    }
}
