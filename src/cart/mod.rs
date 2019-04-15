use crate::prelude::{
    CartGroupFuture, CartItem, CartItemProduct, CartItemPromotion, Database, ErrorVariant,
    Optimizer, ProductAmount, ProductAmountGroupFuture,
};
use futures::prelude::*;
use std::fmt;

pub mod fut;
pub mod item;
pub mod optimizer;
pub mod optimizer_candidate;

#[derive(Clone)]
pub struct Cart {
    database: Database,
    items: Vec<Box<dyn CartItem>>,
}

impl Cart {
    pub fn new(database: Database) -> Self {
        let items = vec![];
        Cart { database, items }
    }

    pub fn get_items(&self) -> &Vec<Box<dyn CartItem>> {
        &self.items
    }

    pub fn get_total_price(&self) -> f64 {
        self.get_items().iter().map(|i| i.get_total()).sum()
    }

    pub fn get_products(&self) -> Vec<ProductAmount> {
        let items: Vec<Box<dyn CartItem>> = self
            .get_items()
            .iter()
            .filter(|item| item.is_product())
            .map(|item| item.clone())
            .collect();

        let mut products: Vec<ProductAmount> = vec![];
        items.iter().for_each(|item| {
            item.get_products()
                .iter()
                .for_each(|&p| products.push(p.clone()));
        });

        products
    }

    pub fn remove_all_products(&mut self) {
        let items: Vec<Box<dyn CartItem>> = self
            .get_items()
            .iter()
            .filter(|item| !item.is_product())
            .map(|item| item.clone())
            .collect();

        self.items = items;
    }

    pub fn push_product(&mut self, code: &String, amount: f64) -> Result<(), ErrorVariant> {
        let product = self.database.fetch_product(code)?;
        let cart_item_product = CartItemProduct::new(product.clone(), amount);
        self.items.push(Box::new(cart_item_product));
        Ok(())
    }

    pub fn push_product_amount(&mut self, product_amount: ProductAmount) {
        let product = product_amount.get_product().clone();
        let amount = *product_amount.get_amount();
        let cart_item_product = CartItemProduct::new(product, amount);
        self.items.push(Box::new(cart_item_product));
    }

    pub fn push_promotion(&mut self, code: &String, amount: f64) -> Result<(), ErrorVariant> {
        let promotion = self.database.fetch_promotion(code)?;
        let cart_item_promotion = CartItemPromotion::new(promotion.clone(), amount);
        self.items.push(Box::new(cart_item_promotion));
        Ok(())
    }

    pub fn consume_available_products_for_promotion(
        &mut self,
        promotion_code: &String,
    ) -> Result<(), ErrorVariant> {
        let promotion = self.database.fetch_promotion(promotion_code)?;
        let products = self.get_products();
        let products = ProductAmountGroupFuture::new(products).wait()?;
        let products = promotion.consume_items(products)?;
        self.remove_all_products();
        for p in products {
            self.push_product_amount(p);
        }
        Ok(())
    }

    pub fn get_flat_quantities_future(&self) -> CartGroupFuture {
        CartGroupFuture::new(&self)
    }

    /// Optimize the cart items composition with [Optimizer](crate::cart::optimizer::Optimizer)
    pub fn optimize_promotions(&mut self) -> Result<&Cart, ErrorVariant> {
        let products = self.get_flat_quantities_future().wait()?;
        let mut optimizer = Optimizer::new(products, self.database.clone());
        let (products, promotions) = optimizer.get_optimal_products_promotions()?;
        self.remove_all_products();
        products
            .iter()
            .for_each(|p| self.push_product_amount(p.clone()));
        for p in promotions {
            self.push_promotion(p.get_code(), 1.0)?;
        }
        Ok(self)
    }

    pub fn reset(&mut self) -> Result<(), ErrorVariant> {
        self.items = vec![];
        Ok(())
    }
}

impl fmt::Display for Cart {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let items_fmt = self
            .get_items()
            .iter()
            .fold(String::from(""), |s, i| format!("{}\n{}", s, i));

        write!(
            f,
            r#"Items: {}
Total: {}"#,
            items_fmt,
            self.get_total_price()
        )
    }
}
