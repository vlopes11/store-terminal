use crate::prelude::{
    Cart, CartItem, CartItemProduct, ErrorVariant, ProductAmount, ProductAmountGroupFuture,
};
use futures::prelude::*;
use futures::try_ready;

pub struct CartGroupFuture {
    queue: Vec<Box<dyn CartItem>>,
    result: Vec<ProductAmount>,
}

impl CartGroupFuture {
    /// Group all items of a cart into an optimal size vec of ProductAmount
    ///
    /// # Example
    ///
    /// ```
    /// use store_terminal::prelude::*;
    /// use futures::prelude::*;
    ///
    /// let mut database = Database::new();
    ///
    /// database.append(Product::new("Foo".to_string(), 1.0));
    /// database.append(Product::new("Bar".to_string(), 2.0));
    ///
    /// let mut cart = Cart::new(database);
    /// cart.push_product(&"Foo".to_string(), 15.0).unwrap();
    /// cart.push_product(&"Bar".to_string(), 35.0).unwrap();
    /// cart.push_product(&"Foo".to_string(), 4.0).unwrap();
    /// cart.push_product(&"Foo".to_string(), 12.0).unwrap();
    ///
    /// let mut v_min = vec![];
    /// v_min.push(ProductAmount::new(Product::new("Foo".to_string(), 1.0), 31.0));
    /// v_min.push(ProductAmount::new(Product::new("Bar".to_string(), 1.0), 35.0));
    ///
    /// let result = CartGroupFuture::new(&cart).wait().unwrap();
    ///
    /// assert_eq!(result, v_min);
    /// ```
    pub fn new(cart: &Cart) -> Self {
        let result = vec![];
        let queue = cart.get_items().clone();
        CartGroupFuture { queue, result }
    }
}

impl Future for CartGroupFuture {
    type Item = Vec<ProductAmount>;
    type Error = ErrorVariant;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while !self.queue.is_empty() {
            let amounts = self.queue[self.queue.len() - 1]
                .get_products()
                .iter()
                .map(|&p| p.clone())
                .collect();

            self.result.append(&mut try_ready!(
                ProductAmountGroupFuture::new(amounts).poll()
            ));

            self.queue.pop();
        }

        Ok(Async::Ready(try_ready!(ProductAmountGroupFuture::new(
            (*self.result).to_vec()
        )
        .poll())))
    }
}

pub struct CartOptimizeFuture {
    result: Vec<Box<dyn CartItem>>,
}

impl CartOptimizeFuture {
    pub fn new(base: Vec<ProductAmount>) -> Self {
        let result = base
            .iter()
            .map(|p| {
                let cart_item: Box<dyn CartItem> = Box::new(CartItemProduct::new(
                    p.get_product().clone(),
                    *p.get_amount(),
                ));
                cart_item
            })
            .collect();
        CartOptimizeFuture { result }
    }
}

impl Future for CartOptimizeFuture {
    type Item = Vec<Box<dyn CartItem>>;
    type Error = ErrorVariant;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        Ok(Async::Ready((*self.result).to_vec()))
    }
}
