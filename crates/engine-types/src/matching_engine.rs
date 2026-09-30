//! Process that turns a sequenced command into events.
//!
//! When a caller processes a command, it uses this module so resulting events append
//! into a buffer the caller owns. Events already in the buffer stay.

use crate::engine_event::EngineEvent;
use crate::sequenced_command::SequencedCommand;

/// MatchingEngine is the process that turns a sequenced command into events.
/// When a caller processes a command, it uses this trait so resulting events append
/// into a buffer the caller owns. Events already in the buffer stay.
pub trait MatchingEngine {
    /// Leaves resulting events appended in `out` for one sequenced command.
    /// Events already in the buffer stay.
    fn process(&mut self, command: SequencedCommand, out: &mut Vec<EngineEvent>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine_command::EngineCommand;
    use crate::engine_event::EngineEvent;
    use crate::identity::{
        AccountId, ClientOrderId, CommandSequence, EventSequence, InstrumentId, OrderId,
        TimestampNanos,
    };
    use crate::order::Side;
    use crate::price::Price;
    use crate::quantity::Quantity;
    use crate::sequenced_command::SequencedCommand;

    /// AppendOnlyMatchingEngine is a test matcher that appends one Accepted event.
    struct AppendOnlyMatchingEngine;

    impl MatchingEngine for AppendOnlyMatchingEngine {
        fn process(&mut self, command: SequencedCommand, out: &mut Vec<EngineEvent>) {
            out.push(EngineEvent::Accepted {
                event_sequence: EventSequence::new(1),
                command_sequence: command.command_sequence(),
                timestamp_nanos: TimestampNanos::new(1000),
                order_id: OrderId::new(42),
                client_order_id: ClientOrderId::new(7),
                account_id: AccountId::new(1),
            });
        }
    }

    fn sample_new_limit_command() -> SequencedCommand {
        SequencedCommand::new(
            CommandSequence::new(0),
            EngineCommand::NewLimit {
                account_id: AccountId::new(1),
                client_order_id: ClientOrderId::new(7),
                instrument_id: InstrumentId::new(2),
                side: Side::Buy,
                price: Price::new(100),
                quantity: Quantity::new(10),
            },
        )
    }

    #[test]
    fn matching_engine_process_appends_to_caller_buffer() {
        // Given an empty buffer and a sequenced NewLimit command
        let mut engine = AppendOnlyMatchingEngine;
        let mut out: Vec<EngineEvent> = Vec::new();
        let command = sample_new_limit_command();

        // When we process the command
        engine.process(command, &mut out);

        // Then the buffer holds one Accepted event
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn matching_engine_process_does_not_clear_caller_buffer() {
        // Given a buffer that already holds one Accepted event
        let mut engine = AppendOnlyMatchingEngine;
        let mut out: Vec<EngineEvent> = vec![EngineEvent::Accepted {
            event_sequence: EventSequence::new(1),
            command_sequence: CommandSequence::new(0),
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        }];
        let command = SequencedCommand::new(
            CommandSequence::new(1),
            EngineCommand::NewMarket {
                account_id: AccountId::new(2),
                client_order_id: ClientOrderId::new(8),
                instrument_id: InstrumentId::new(2),
                side: Side::Sell,
                quantity: Quantity::new(5),
            },
        );

        // When we process another command
        engine.process(command, &mut out);

        // Then the buffer holds both events
        assert_eq!(out.len(), 2);
    }
}
