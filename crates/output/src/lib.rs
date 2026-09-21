//! Publish of engine events out of the pipeline.
//!
//! When a caller hands an EngineEvent to an output stage, it uses this module.
//! publish leaves the event unpublished.

use engine_types::EngineEvent;

/// OutputSink is the publish of engine events out of the pipeline.
/// When a caller hands an EngineEvent to an output stage, it uses this type.
/// publish leaves the event unpublished.
pub struct OutputSink;

impl OutputSink {
    /// Answers an OutputSink.
    pub fn new() -> Self {
        Self
    }

    /// Leaves the engine event unpublished.
    pub fn publish(&self, _event: &EngineEvent) {}
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
        AccountId, ClientOrderId, CommandSequence, EngineEvent, EngineEventRejectReason,
        EventSequence, InstrumentId, OrderId, Price, Quantity, TimestampNanos,
    };

    fn sample_trade() -> EngineEvent {
        EngineEvent::Trade {
            event_sequence: EventSequence::new(1),
            command_sequence: CommandSequence::new(0),
            timestamp_nanos: TimestampNanos::new(1000),
            maker_order_id: OrderId::new(10),
            taker_order_id: OrderId::new(20),
            instrument_id: InstrumentId::new(2),
            price: Price::new(100),
            quantity: Quantity::new(5),
        }
    }

    #[test]
    fn publish_accepts_each_event_variant() {
        // Given each EngineEvent variant
        let events = [
            EngineEvent::Accepted {
                event_sequence: EventSequence::new(1),
                command_sequence: CommandSequence::new(0),
                timestamp_nanos: TimestampNanos::new(1000),
                order_id: OrderId::new(42),
                client_order_id: ClientOrderId::new(7),
                account_id: AccountId::new(1),
            },
            EngineEvent::Rejected {
                event_sequence: EventSequence::new(2),
                command_sequence: CommandSequence::new(1),
                timestamp_nanos: TimestampNanos::new(2000),
                client_order_id: ClientOrderId::new(7),
                reason: EngineEventRejectReason::InvalidQuantity,
            },
            EngineEvent::Replaced {
                event_sequence: EventSequence::new(3),
                command_sequence: CommandSequence::new(2),
                timestamp_nanos: TimestampNanos::new(3000),
                order_id: OrderId::new(42),
                client_order_id: ClientOrderId::new(99),
            },
            EngineEvent::Canceled {
                event_sequence: EventSequence::new(4),
                command_sequence: CommandSequence::new(3),
                timestamp_nanos: TimestampNanos::new(4000),
                order_id: OrderId::new(42),
            },
            sample_trade(),
        ];
        let sink = OutputSink::new();

        // When we publish each event
        // Then the sink accepts the event
        for event in events {
            sink.publish(&event);
        }
    }
}
