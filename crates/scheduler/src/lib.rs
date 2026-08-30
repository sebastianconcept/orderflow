//! Scheduler module for order scheduling.
//!
//! This module provides the [`Scheduler`] struct which handles order
//! scheduling functionality. It accepts orders and returns them unchanged,
//! serving as a pass-through for testing and future scheduling logic.

use engine_types::Order;

/// Scheduler for orders.
///
/// The scheduler accepts an order and returns it unchanged. This provides
/// a placeholder for future scheduling logic while ensuring the engine types
/// work correctly with integer-based fields.
#[derive(Debug, Clone)]
pub struct Scheduler;

impl Scheduler {
    /// Creates a new scheduler instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use scheduler::Scheduler;
    /// let _ = Scheduler::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Schedules an order, returning it unchanged.
    ///
    /// This method serves as a pass-through for the order. Future
    /// implementations may modify or route orders based on scheduling
    /// rules.
    ///
    /// # Arguments
    ///
    /// * `order` - The order to schedule
    ///
    /// # Examples
    ///
    /// ```
    /// use engine_types::{Order, OrderType, Side, Price, Quantity};
    /// use scheduler::Scheduler;
    ///
    /// let scheduler = Scheduler::new();
    /// let order = Order::new(
    ///     1,
    ///     7,
    ///     2,
    ///     10,
    ///     Side::Buy,
    ///     OrderType::Limit,
    ///     Price::new(100),
    ///     Quantity::new(10),
    /// );
    /// let scheduled = scheduler.schedule(order);
    /// assert_eq!(scheduled.id.inner(), 1);
    /// ```
    pub fn schedule(&self, order: Order) -> Order {
        order
    }
}

impl Default for Scheduler {
    /// Creates a default scheduler instance.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_types::{OrderType, Price, Quantity, Side};

    #[test]
    fn schedule_returns_order_unchanged() {
        // Given: an order with integer fields
        let original = Order::new(
            1,
            7,
            2,
            10,
            Side::Buy,
            OrderType::Limit,
            Price::new(100),
            Quantity::new(10),
        );

        // When: we schedule the order
        let scheduler = Scheduler::new();
        let result = scheduler.schedule(original);

        // Then: the order is returned unchanged
        assert_eq!(result.id.inner(), 1);
        assert_eq!(result.client_order_id.inner(), 7);
        assert_eq!(result.instrument_id.inner(), 2);
        assert_eq!(result.account_id.inner(), 10);
        assert_eq!(result.side, Side::Buy);
        assert_eq!(result.order_type, OrderType::Limit);
        assert_eq!(result.price.inner(), 100);
        assert_eq!(result.quantity.inner(), 10);
    }

    #[test]
    fn new_constructs_order_with_integer_quantity() {
        // Given: no order yet
        // When: we construct an order with integer quantity
        let order = Order::new(
            1,
            7,
            2,
            10,
            Side::Buy,
            OrderType::Limit,
            Price::new(100),
            Quantity::new(10),
        );

        // Then: the order is constructed correctly with integer quantity
        assert_eq!(order.quantity.inner(), 10);
        assert_eq!(order.price.inner(), 100);
        assert_eq!(order.id.inner(), 1);
        assert_eq!(order.client_order_id.inner(), 7);
    }

    #[test]
    fn accepts_engine_types_order() {
        // Given: an order from engine_types
        let order = Order::new(
            123,
            456,
            789,
            101112,
            Side::Sell,
            OrderType::Market,
            Price::new(0),
            Quantity::new(100),
        );

        // When: we schedule it
        let scheduler = Scheduler::new();
        let result = scheduler.schedule(order);

        // Then: the scheduler accepts engine_types Order
        assert_eq!(result.id.inner(), 123);
        assert_eq!(result.side, Side::Sell);
        assert_eq!(result.order_type, OrderType::Market);
        assert_eq!(result.quantity.inner(), 100);
    }

    #[test]
    fn scheduler_is_clone() {
        // Given: a scheduler instance
        let scheduler = Scheduler::new();

        // When: we clone it
        let cloned_scheduler = scheduler.clone();

        // Then: both can be used
        assert_eq!(
            format!("{:?}", scheduler),
            format!("{:?}", cloned_scheduler)
        );
    }
}
