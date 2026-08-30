// Sequencer placeholder

use engine_types::Order;

/// Sequencer placeholder for order sequencing.
///
/// The `Sequencer` is responsible for assigning sequence numbers to orders
/// before they are processed by the matching engine. This placeholder version
/// returns a fixed sequence number.
pub struct Sequencer;

impl Sequencer {
    /// Create a new Sequencer instance.
    pub fn new() -> Self {
        Self
    }

    /// Get the next sequence number.
    ///
    /// # Returns
    ///
    /// Returns 0 as a placeholder sequence number.
    pub fn next_sequence(&self) -> u64 {
        0
    }

    /// Process an order through the sequencer.
    ///
    /// This is a placeholder implementation that simply returns the order unchanged.
    /// In a full implementation, this would assign sequence numbers and prepare
    /// the order for matching engine processing.
    ///
    /// # Arguments
    ///
    /// * `order` - The order to process
    ///
    /// # Returns
    ///
    /// Returns the same order with no modifications (placeholder).
    pub fn process(&self, _order: Order) -> Order {
        // Placeholder: return a dummy order with zeroed fields
        // In a real implementation, this would assign sequence numbers and validate
        Order {
            id: engine_types::OrderId::new(0),
            client_order_id: engine_types::ClientOrderId::new(0),
            instrument_id: engine_types::InstrumentId::new(0),
            account_id: engine_types::AccountId::new(0),
            side: engine_types::Side::Buy,
            order_type: engine_types::OrderType::Limit,
            price: engine_types::Price::new(0),
            quantity: engine_types::Quantity::new(0),
        }
    }
}

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_placeholder_returns_zero() {
        // Given: a sequencer instance
        let sequencer = Sequencer::new();

        // When: we process a placeholder order
        let result_order = sequencer.process(Order::new(
            1,
            100,
            1,
            50,
            engine_types::Side::Buy,
            engine_types::OrderType::Limit,
            engine_types::Price::new(1000),
            engine_types::Quantity::new(10),
        ));

        // Then: the result is a placeholder order with zeroed fields
        assert_eq!(result_order.id.inner(), 0);
        assert_eq!(result_order.client_order_id.inner(), 0);
        assert_eq!(result_order.instrument_id.inner(), 0);
        assert_eq!(result_order.account_id.inner(), 0);
        assert_eq!(result_order.side, engine_types::Side::Buy);
        assert_eq!(result_order.order_type, engine_types::OrderType::Limit);
        assert_eq!(result_order.price.inner(), 0);
        assert_eq!(result_order.quantity.inner(), 0);
    }

    #[test]
    fn new_constructs() {
        // Given: all required fields for an Order
        let id = 1u64;
        let client_order_id = 100u64;
        let instrument_id = 1u64;
        let account_id = 50u64;
        let side = engine_types::Side::Buy;
        let order_type = engine_types::OrderType::Limit;
        let price = engine_types::Price::new(1000);
        let quantity = engine_types::Quantity::new(10);

        // When: we construct an Order with integer fields
        let order = Order::new(
            id,
            client_order_id,
            instrument_id,
            account_id,
            side,
            order_type,
            price,
            quantity,
        );

        // Then: all fields are set correctly
        assert_eq!(order.id.inner(), id);
        assert_eq!(order.client_order_id.inner(), client_order_id);
        assert_eq!(order.instrument_id.inner(), instrument_id);
        assert_eq!(order.account_id.inner(), account_id);
        assert_eq!(order.side, side);
        assert_eq!(order.order_type, order_type);
        assert_eq!(order.price, price);
        assert_eq!(order.quantity, quantity);
    }

    #[test]
    fn accepts_engine_types_order() {
        // Given: an Order constructed with engine_types integers
        let order = Order::new(
            1,
            100,
            1,
            50,
            engine_types::Side::Sell,
            engine_types::OrderType::Market,
            engine_types::Price::new(0),
            engine_types::Quantity::new(25),
        );

        // When: we process it through the sequencer
        let sequencer = Sequencer::new();
        let result_order = sequencer.process(order);

        // Then: the sequencer accepts and processes the order without errors
        assert_eq!(result_order.id.inner(), 0);
    }
}
