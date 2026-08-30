// Input gateway – placeholder

use engine_types::Order;

/// Input gateway for receiving orders into the matching engine.
///
/// This component is responsible for ingesting incoming order commands
/// and preparing them for processing by the matching engine.
pub struct InputGateway;

impl InputGateway {
    /// Create a new InputGateway instance.
    pub fn new() -> Self {
        Self
    }

    /// Attempt to receive an order from the input source.
    ///
    /// This is a placeholder implementation that always returns None.
    /// Actual order reception logic will be implemented in future stories.
    ///
    /// # Returns
    ///
    /// Always returns None as a placeholder for future implementation.
    pub fn receive(&self) -> Option<Order> {
        None
    }
}

impl Default for InputGateway {
    /// Create a default (new) InputGateway instance.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::InputGateway;
    use engine_types::{Order, OrderType, Price, Quantity, Side};

    #[test]
    fn gateway_new_constructs() {
        // Given: no existing state
        // When: we create a new InputGateway
        let gateway = InputGateway::new();

        // Then: the gateway is created successfully
        assert!(matches!(gateway, InputGateway));
    }

    #[test]
    fn receive_returns_none_placeholder() {
        // Given: a new InputGateway
        let gateway = InputGateway::new();

        // When: we call receive()
        let result = gateway.receive();

        // Then: it returns None (placeholder behavior)
        assert!(result.is_none());
    }

    #[test]
    fn lib_compiles_with_engine_types_order() {
        // Given: an Order constructed from engine-types
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

        // When: we use the order in a gateway context
        let gateway = InputGateway::new();

        // Then: it compiles and we can attempt to receive
        let _ = gateway.receive();

        // Verify the order has correct integer fields (no f64)
        assert_eq!(order.price.inner(), 100);
        assert_eq!(order.quantity.inner(), 10);
        assert!(order.is_limit());
    }
}
