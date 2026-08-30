// Output sink – placeholder implementation

use engine_types::EngineEvent;

pub struct OutputSink;

impl OutputSink {
    pub fn new() -> Self {
        Self
    }

    /// Publish an engine event.
    ///
    /// This is a placeholder that accepts any EngineEvent variant without processing.
    /// Real output implementations will handle events based on their variant.
    ///
    /// # Arguments
    ///
    /// * `_event` - The engine event to publish (Accepted, Rejected, Replaced, Canceled, or Trade)
    pub fn publish(&self, _event: &EngineEvent) {
        // TODO: real output implementation
    }
}

impl Default for OutputSink {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_types::{
        AccountId, ClientOrderId, EngineEvent, EngineEventRejectReason, InstrumentId, OrderId,
        Price, Quantity, Sequence, TimestampNanos,
    };

    /// Test that OutputSink can be constructed with the new method.
    #[test]
    fn sink_new_constructs() {
        // When: creating a new OutputSink
        let _sink = OutputSink::new();

        // Then: it should be successfully created (no panic)
    }

    /// Test that publish accepts a Trade event without panicking.
    #[test]
    fn publish_accepts_trade_event_without_panic() {
        // Given: a Trade event with all required fields
        let trade_event = EngineEvent::Trade {
            sequence: Sequence::new(1),
            timestamp_nanos: TimestampNanos::new(1000),
            maker_order_id: OrderId::new(10),
            taker_order_id: OrderId::new(20),
            instrument_id: InstrumentId::new(2),
            price: Price::new(100),
            quantity: Quantity::new(5),
        };

        let sink = OutputSink::new();

        // When: publishing the Trade event
        // Then: it should not panic
        sink.publish(&trade_event);
    }

    /// Test that publish is a placeholder no-op that accepts any event.
    #[test]
    fn publish_is_placeholder_noop() {
        // Given: various event types
        let events = vec![
            EngineEvent::Accepted {
                sequence: Sequence::new(1),
                timestamp_nanos: TimestampNanos::new(1000),
                order_id: OrderId::new(42),
                client_order_id: ClientOrderId::new(7),
                account_id: AccountId::new(1),
            },
            EngineEvent::Rejected {
                sequence: Sequence::new(2),
                timestamp_nanos: TimestampNanos::new(2000),
                client_order_id: ClientOrderId::new(7),
                reason: EngineEventRejectReason::InvalidQuantity,
            },
            EngineEvent::Replaced {
                sequence: Sequence::new(3),
                timestamp_nanos: TimestampNanos::new(3000),
                order_id: OrderId::new(42),
                client_order_id: ClientOrderId::new(99),
            },
            EngineEvent::Canceled {
                sequence: Sequence::new(4),
                timestamp_nanos: TimestampNanos::new(4000),
                order_id: OrderId::new(42),
            },
            EngineEvent::Trade {
                sequence: Sequence::new(5),
                timestamp_nanos: TimestampNanos::new(5000),
                maker_order_id: OrderId::new(10),
                taker_order_id: OrderId::new(20),
                instrument_id: InstrumentId::new(2),
                price: Price::new(100),
                quantity: Quantity::new(5),
            },
        ];

        let sink = OutputSink::new();

        // When: publishing each event type
        for event in events {
            // Then: it should not panic (no-op placeholder)
            sink.publish(&event);
        }
    }

    /// Test that OutputSink implements Default.
    #[test]
    fn sink_default_works() {
        // When: creating an OutputSink using Default
        let _sink = OutputSink;

        // Then: it should be successfully created (no panic)
    }
}
