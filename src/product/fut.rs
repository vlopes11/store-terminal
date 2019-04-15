use crate::prelude::{ErrorVariant, ProductAmount};
use futures::prelude::*;

pub struct ProductAmountGroupFuture {
    queue: Vec<ProductAmount>,
}

impl ProductAmountGroupFuture {
    /// Group different instances of ProductAmount into an optimal size vec
    ///
    /// # Example
    ///
    /// ```
    /// use store_terminal::prelude::*;
    /// use futures::prelude::*;
    ///
    /// let mut v = vec![];
    ///
    /// v.push(ProductAmount::new(Product::new("Foo".to_string(), 1.0), 15.0));
    /// v.push(ProductAmount::new(Product::new("Bar".to_string(), 1.0), 35.0));
    /// v.push(ProductAmount::new(Product::new("Foo".to_string(), 1.0), 4.0));
    /// v.push(ProductAmount::new(Product::new("Foo".to_string(), 1.0), 12.0));
    ///
    /// let mut v_min = vec![];
    ///
    /// v_min.push(ProductAmount::new(Product::new("Foo".to_string(), 1.0), 31.0));
    /// v_min.push(ProductAmount::new(Product::new("Bar".to_string(), 1.0), 35.0));
    ///
    /// let result = ProductAmountGroupFuture::new(v).wait().unwrap();
    ///
    /// assert_eq!(result, v_min);
    /// ```
    pub fn new(queue: Vec<ProductAmount>) -> Self {
        ProductAmountGroupFuture { queue }
    }
}

impl Future for ProductAmountGroupFuture {
    type Item = Vec<ProductAmount>;
    type Error = ErrorVariant;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        let mut result: Self::Item = vec![];

        while let Some(item) = self.queue.pop() {
            let index =
                result
                    .iter()
                    .enumerate()
                    .fold(None, |index, (current_index, current_item)| match index {
                        Some(_) => index,
                        None if current_item == &item => Some(current_index),
                        _ => None,
                    });

            match index {
                Some(i) => result[i].inc_amount(*item.get_amount()),
                None => result.push(item),
            }
        }

        Ok(Async::Ready(result))
    }
}
