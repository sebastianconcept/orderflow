//! Routing of orders toward the matching engine.
//!
//! When a pipeline stage receives an Order, it uses this module. schedule
//! answers the same Order.

use engine_types::Order;

/// Scheduler is the routing of orders toward the matching engine.
/// When a pipeline stage receives an Order, it uses this type. schedule
/// answers the same Order.
#[derive(Debug, Clone)]
pub struct Scheduler;

impl Scheduler {
    /// Answers a Scheduler.
    pub fn new() -> Self {
        Self
    }

    /// Answers the order after scheduling.
    pub fn schedule(&self, order: Order) -> Order {
        order
    }
}

impl Default for Scheduler {
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

    fn sample_order() -> Order {
        Order::new(
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
    fn schedule_returns_order_unchanged() {
        // Given an order with integer fields
        let original = sample_order();

        // When we schedule the order
        let scheduler = Scheduler::new();
        let result = scheduler.schedule(original);

        // Then the order is returned unchanged
        assert_eq!(result, original);
        assert_eq!(result.id.inner(), 1);
        assert_eq!(result.quantity.inner(), 10);
    }

    #[test]
    fn accepts_engine_types_order() {
        // Given an order from engine_types
        let order = Order::new(
            OrderId::new(123),
            ClientOrderId::new(456),
            InstrumentId::new(789),
            AccountId::new(101112),
            Side::Sell,
            OrderType::Market,
            Price::new(0),
            Quantity::new(100),
        );

        // When we schedule it
        let scheduler = Scheduler::new();
        let result = scheduler.schedule(order);

        // Then the scheduler returns the same order
        assert_eq!(result, order);
        assert_eq!(result.side, Side::Sell);
        assert_eq!(result.order_type, OrderType::Market);
    }
}
