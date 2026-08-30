//! Screener module for filtering orders.
//!
//! This module provides the [`Screener`] struct which implements order
//! screening logic for the matching engine pipeline.

use engine_types::Order;

/// Screener filters orders before they enter the matching engine.
///
/// Currently a placeholder that accepts all orders. Future iterations will
/// implement actual screening logic based on account, instrument, or other
/// criteria.
pub struct Screener;

impl Screener {
    pub fn new() -> Self {
        Self
    }

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

    #[test]
    fn allow_returns_true_for_placeholder() {
        // Given: a Screener instance
        let screener = Screener::new();

        // When: we check if an order is allowed
        let order = Order::new(
            1,
            7,
            2,
            10,
            engine_types::Side::Buy,
            engine_types::OrderType::Limit,
            engine_types::Price::new(100),
            engine_types::Quantity::new(10),
        );

        // Then: the screener allows all orders (placeholder behavior)
        assert!(screener.allow(&order));
    }

    #[test]
    fn new_constructs() {
        // Given: an order with integer fields
        let expected_id = 42u64;
        let expected_client_order_id = 100u64;
        let expected_instrument_id = 5u64;
        let expected_account_id = 200u64;
        let expected_side = engine_types::Side::Sell;
        let expected_order_type = engine_types::OrderType::Market;
        let expected_price = engine_types::Price::new(0); // unused for market orders
        let expected_quantity = engine_types::Quantity::new(15);

        // When: we construct the order
        let order = Order::new(
            expected_id,
            expected_client_order_id,
            expected_instrument_id,
            expected_account_id,
            expected_side,
            expected_order_type,
            expected_price,
            expected_quantity,
        );

        // Then: all fields are set correctly
        assert_eq!(order.id.inner(), expected_id);
        assert_eq!(order.client_order_id.inner(), expected_client_order_id);
        assert_eq!(order.instrument_id.inner(), expected_instrument_id);
        assert_eq!(order.account_id.inner(), expected_account_id);
        assert_eq!(order.side, expected_side);
        assert_eq!(order.order_type, expected_order_type);
        assert_eq!(order.price, expected_price);
        assert_eq!(order.quantity, expected_quantity);

        // Verify market order flags
        assert!(!order.is_limit());
        assert!(order.is_market());
    }

    #[test]
    fn order_used_in_allow_is_copy() {
        // Given: an Order instance
        let original_order = Order::new(
            1,
            7,
            2,
            10,
            engine_types::Side::Buy,
            engine_types::OrderType::Limit,
            engine_types::Price::new(100),
            engine_types::Quantity::new(10),
        );

        // Given: a Screener
        let screener = Screener::new();

        // When: we pass the order to allow (copy semantics)
        let allowed = screener.allow(&original_order);

        // Then: the original order is still usable after being passed by reference
        assert!(allowed);
        assert!(original_order.is_limit());

        // And: the order can be used again (Copy trait)
        let allowed_again = screener.allow(&original_order);
        assert!(allowed_again);

        // Verify the order fields are unchanged
        assert_eq!(original_order.side, engine_types::Side::Buy);
        assert_eq!(original_order.quantity, engine_types::Quantity::new(10));
    }
}
