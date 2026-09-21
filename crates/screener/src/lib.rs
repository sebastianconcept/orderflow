//! Filter of orders before matching.
//!
//! When a pipeline stage decides whether an Order may continue, it uses this
//! module. allow answers true.

use engine_types::Order;

/// Screener is the filter of orders before matching.
/// When a pipeline stage decides whether an Order may continue, it uses this
/// type. allow answers true.
pub struct Screener;

impl Screener {
    /// Answers a Screener.
    pub fn new() -> Self {
        Self
    }

    /// Answers whether this order may continue.
    pub fn allow(&self, _order: &Order) -> bool {
        true
    }
}

impl Default for Screener {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_types::{
        AccountId, ClientOrderId, InstrumentId, OrderId, OrderType, Price, Quantity, Side,
    };

    fn sample_order() -> engine_types::Order {
        engine_types::Order::new(
            OrderId::new(1),
            ClientOrderId::new(7),
            InstrumentId::new(2),
            AccountId::new(10),
            Side::Buy,
            OrderType::Limit,
            Price::new(100),
            Quantity::new(10),
        )
    }

    #[test]
    fn allow_returns_true_in_this_slice() {
        // Given a Screener and an order
        let screener = Screener::new();
        let order = sample_order();

        // When we check whether the order is allowed
        let allowed = screener.allow(&order);

        // Then the order is allowed
        assert!(allowed);
    }

    #[test]
    fn order_used_in_allow_remains_usable() {
        // Given an Order and a Screener
        let original_order = sample_order();
        let screener = Screener::new();

        // When we pass the order to allow
        let allowed = screener.allow(&original_order);

        // Then the original order is still usable
        assert!(allowed);
        assert!(original_order.is_limit());
        assert_eq!(original_order.quantity, Quantity::new(10));
    }
}
