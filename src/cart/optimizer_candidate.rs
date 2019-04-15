use crate::prelude::{ErrorVariant, ProductAmount, Promotion};

#[derive(Debug, Clone)]
pub struct OptimizerCandidate {
    price: f64,
    promotions: Vec<Promotion>,
    products: Vec<ProductAmount>,
}

impl OptimizerCandidate {
    pub fn new(promotions: Vec<Promotion>, products: Vec<ProductAmount>) -> Self {
        let mut optimizer_candidate = OptimizerCandidate {
            price: 0.0,
            promotions,
            products,
        };
        optimizer_candidate.set_price();
        optimizer_candidate
    }

    pub fn get_price(&self) -> &f64 {
        &self.price
    }

    pub fn get_promotions(&self) -> &Vec<Promotion> {
        &self.promotions
    }

    pub fn get_products(&self) -> &Vec<ProductAmount> {
        &self.products
    }

    fn set_price(&mut self) {
        let price = self
            .get_promotions()
            .iter()
            .map(|p| p.get_price())
            .sum::<f64>()
            + self
                .get_products()
                .iter()
                .map(|p| p.get_total_price())
                .sum::<f64>();
        self.price = price;
    }

    pub fn simulate_promotion(&self, promotion: Promotion) -> Result<Self, ErrorVariant> {
        let products = self.get_products().clone();
        let mut promotions = self.get_promotions().clone();

        let products = promotion.consume_items(products)?;
        promotions.push(promotion);

        Ok(OptimizerCandidate::new(promotions, products))
    }
}
