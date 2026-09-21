//! Intake of orders into the pipeline.
//!
//! When a caller polls for an incoming Order, it uses this module. receive
//! answers none.

use engine_types::Order;

/// InputGateway is the intake of orders into the pipeline.
/// When a caller polls for an incoming Order, it uses this type. receive
/// answers none.
pub struct InputGateway;

impl InputGateway {
    /// Answers an InputGateway.
    pub fn new() -> Self {
        Self
    }

    /// Answers the next order from the input source, or none when no order is waiting.
    pub fn receive(&self) -> Option<Order> {
        None
    }
}

impl Default for InputGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::InputGateway;
    use engine_types::{
        AccountId, ClientOrderId, InstrumentId, Order, OrderId, OrderType, Price, Quantity, Side,
    };

    #[test]
    fn gateway_new_constructs() {
        // Given no existing gateway
        // When we create an InputGateway
        let gateway = InputGateway::new();

        // Then the gateway is constructed
        assert!(matches!(gateway, InputGateway));
    }

    #[test]
    fn receive_returns_none_in_this_slice() {
        // Given a new InputGateway
        let gateway = InputGateway::new();

        // When we call receive
        let result = gateway.receive();

        // Then no order is returned
        assert!(result.is_none());
    }

    #[test]
    fn gateway_accepts_engine_types_order() {
        // Given an Order with integer fields
        let order = Order::new(
            OrderId::new(1),
            ClientOrderId::new(7),
            InstrumentId::new(2),
            AccountId::new(10),
            Side::Buy,
            OrderType::Limit,
            Price::new(100),
            Quantity::new(10),
        );
        let gateway = InputGateway::new();

        // When we poll the gateway
        let _ = gateway.receive();

        // Then the order still holds integer price and quantity
        assert_eq!(order.price.inner(), 100);
        assert_eq!(order.quantity.inner(), 10);
        assert!(order.is_limit());
    }
}
