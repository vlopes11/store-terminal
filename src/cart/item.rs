use crate::prelude::{CartItemProduct, CartItemPromotion, ProductAmount};
use std::fmt;
use uuid::Uuid;

pub trait CloneIntoDynBox {
    fn clone_into_dyn_box<'a>(&self) -> Box<dyn 'a + CartItem>
    where
        Self: 'a;
}

impl<T: Clone + CartItem> CloneIntoDynBox for T {
    fn clone_into_dyn_box<'a>(&self) -> Box<dyn 'a + CartItem>
    where
        Self: 'a,
    {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn '_ + CartItem> {
    fn clone(&self) -> Self {
        self.clone_into_dyn_box()
    }
}

pub enum CartItemVariant<'a> {
    Product(&'a CartItemProduct),
    Promotion(&'a CartItemPromotion),
}

pub trait CartItem: CloneIntoDynBox + fmt::Display {
    fn get_id(&self) -> &Uuid;
    fn get_products(&self) -> Vec<&ProductAmount>;
    fn get_amount(&self) -> f64;
    fn get_variant<'a>(&self) -> CartItemVariant;

    fn is_product(&self) -> bool {
        match self.get_variant() {
            CartItemVariant::Product(_) => true,
            _ => false,
        }
    }

    fn get_price(&self) -> f64 {
        self.get_products()
            .iter()
            .fold(0.0, |price, p| price + p.get_price())
    }

    fn get_total(&self) -> f64 {
        self.get_amount() * self.get_price()
    }

    fn get_total_discount(&self) -> f64 {
        self.get_price() * self.get_amount() - self.get_total()
    }
}
