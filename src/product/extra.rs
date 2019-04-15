use crate::prelude::{ErrorVariant, Product};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductAmount {
    product: Product,
    amount: f64,
}

impl ProductAmount {
    pub fn new(product: Product, amount: f64) -> Self {
        ProductAmount { product, amount }
    }

    pub fn get_product(&self) -> &Product {
        &self.product
    }

    pub fn get_code(&self) -> &String {
        self.product.get_code()
    }

    pub fn get_amount(&self) -> &f64 {
        &self.amount
    }

    pub fn inc_amount(&mut self, amount: f64) {
        self.amount += amount;
    }

    pub fn dec_amount(&mut self, amount: f64) -> Result<(), ErrorVariant> {
        if amount > self.amount {
            Err(ErrorVariant::NotEnoughItems)
        } else {
            self.amount -= amount;
            Ok(())
        }
    }

    pub fn get_price(&self) -> &f64 {
        self.get_product().get_price()
    }

    pub fn get_total_price(&self) -> f64 {
        self.get_price() * self.amount
    }

    pub fn get_index_of_product(
        products: &Vec<ProductAmount>,
        code: &String,
    ) -> Result<usize, ErrorVariant> {
        products
            .iter()
            .enumerate()
            .fold(None, |index, (pos, product)| {
                if index.is_some() {
                    index
                } else if product.get_code() == code {
                    Some(pos)
                } else {
                    None
                }
            })
            .map(|p| Ok(p))
            .unwrap_or(Err(ErrorVariant::ProductNotFound))
    }
}

impl Ord for ProductAmount {
    fn cmp(&self, other: &ProductAmount) -> Ordering {
        match &self.partial_cmp(other) {
            Some(c) => *c,
            None => Ordering::Equal,
        }
    }
}

impl PartialOrd for ProductAmount {
    fn partial_cmp(&self, other: &ProductAmount) -> Option<Ordering> {
        let cmp = self.get_product().cmp(other.get_product());
        match cmp {
            Ordering::Equal => self.get_amount().partial_cmp(other.get_amount()),
            _ => Some(cmp),
        }
    }
}

impl PartialEq for ProductAmount {
    fn eq(&self, other: &ProductAmount) -> bool {
        self.get_product().eq(other.get_product())
    }
}

impl Eq for ProductAmount {}
