use crate::prelude::{Database, ErrorVariant, OptimizerCandidate, ProductAmount, Promotion};

#[derive(Debug, Clone)]
pub struct Optimizer {
    available_items: Vec<ProductAmount>,
    maximum_price: f64,
    depleted_options: Vec<Vec<Promotion>>,
    database: Database,
    candidate: OptimizerCandidate,
}

impl Optimizer {
    pub fn new(available_items: Vec<ProductAmount>, database: Database) -> Self {
        let maximum_price = available_items.iter().map(|i| i.get_total_price()).sum();
        let depleted_options = vec![];
        let candidate = OptimizerCandidate::new(vec![], available_items.clone());
        Optimizer {
            available_items,
            maximum_price,
            depleted_options,
            database,
            candidate,
        }
    }

    /// Return a tuple with the optimal combination for products x promotions
    ///
    /// # Example
    ///
    /// ```
    /// use store_terminal::prelude::*;
    ///
    /// let database = Database::new();
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
    /// let mut cart = Cart::new(database.clone());
    /// cart.push_product(&"A".to_string(), 1.0).unwrap();
    /// cart.push_product(&"B".to_string(), 1.0).unwrap();
    /// cart.push_product(&"C".to_string(), 1.0).unwrap();
    /// cart.push_product(&"D".to_string(), 1.0).unwrap();
    /// cart.push_product(&"A".to_string(), 1.0).unwrap();
    /// cart.push_product(&"B".to_string(), 1.0).unwrap();
    /// cart.push_product(&"A".to_string(), 1.0).unwrap();
    /// cart.push_product(&"A".to_string(), 1.0).unwrap();
    /// cart.optimize_promotions().unwrap();
    /// assert_eq!(cart.get_total_price(), 32.4);
    ///
    /// let mut cart = Cart::new(database.clone());
    /// cart.push_product(&"C".to_string(), 7.0).unwrap();
    /// cart.optimize_promotions().unwrap();
    /// assert_eq!(cart.get_total_price(), 7.25);
    ///
    /// let mut cart = Cart::new(database.clone());
    /// cart.push_product(&"A".to_string(), 1.0).unwrap();
    /// cart.push_product(&"B".to_string(), 1.0).unwrap();
    /// cart.push_product(&"C".to_string(), 1.0).unwrap();
    /// cart.push_product(&"D".to_string(), 1.0).unwrap();
    /// cart.optimize_promotions().unwrap();
    /// assert_eq!(cart.get_total_price(), 15.4);
    /// ```
    pub fn get_optimal_products_promotions(
        &mut self,
    ) -> Result<(Vec<ProductAmount>, Vec<Promotion>), ErrorVariant> {
        let possible_promotions = self.database.fetch_possible_promotions_with_maximum_price(
            &self.candidate.get_products().iter().collect(),
            self.candidate.get_price().clone(),
        )?;

        if possible_promotions.is_empty() {
            let products = self.candidate.get_products().clone();
            let promotions = self.candidate.get_promotions().clone();
            return Ok((products, promotions));
        }

        // TODO - Very simple A* algorithm; improve to cover all possible permutations
        for prom in possible_promotions {
            match self.candidate.simulate_promotion(prom) {
                Ok(c) => {
                    if c.get_price() < self.candidate.get_price() {
                        self.candidate = c
                    }
                }
                _ => (),
            }
        }

        self.get_optimal_products_promotions()
    }
}
