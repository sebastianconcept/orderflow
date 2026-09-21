//! State changes that a matching engine emits.
//!
//! When a caller observes matching, it uses this module so accept, reject, replace,
//! cancel, and trade are distinct events. Each event carries EventSequence and the
//! producing CommandSequence.

use crate::identity::{
    AccountId, ClientOrderId, CommandSequence, EventSequence, InstrumentId, OrderId, TimestampNanos,
};
use crate::price::Price;
use crate::quantity::Quantity;

/// EngineEventRejectReason is why a command did not change order state.
/// When a Rejected event names the failure, it uses this type so the reason is a
/// closed set.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EngineEventRejectReason {
    /// Quantity in the command was invalid.
    InvalidQuantity,
    /// Instrument in the command is unknown.
    UnknownInstrument,
    /// Order in a cancel or replace command does not exist.
    OrderNotFound,
    /// Price in the command was invalid.
    InvalidPrice,
    /// Rejection reason outside the named set.
    Other,
}

/// EngineEvent is a state change produced while processing a command.
/// When a caller records matching, it uses this type so each variant carries
/// EventSequence and the producing CommandSequence.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EngineEvent {
    /// Accept of a NewLimit or NewMarket into the book.
    /// The event correlates ClientOrderId with the engine OrderId.
    Accepted {
        event_sequence: EventSequence,
        command_sequence: CommandSequence,
        timestamp_nanos: TimestampNanos,
        order_id: OrderId,
        client_order_id: ClientOrderId,
        account_id: AccountId,
    },

    /// Failure of a command to change order state.
    /// The event correlates the request through ClientOrderId.
    Rejected {
        event_sequence: EventSequence,
        command_sequence: CommandSequence,
        timestamp_nanos: TimestampNanos,
        client_order_id: ClientOrderId,
        reason: EngineEventRejectReason,
    },

    /// Amend of price or quantity on a resting order.
    /// The event tracks the replace through its fresh ClientOrderId.
    Replaced {
        event_sequence: EventSequence,
        command_sequence: CommandSequence,
        timestamp_nanos: TimestampNanos,
        order_id: OrderId,
        client_order_id: ClientOrderId,
    },

    /// Removal of a resting order before a fill.
    /// The event names the engine OrderId that left the book.
    Canceled {
        event_sequence: EventSequence,
        command_sequence: CommandSequence,
        timestamp_nanos: TimestampNanos,
        order_id: OrderId,
    },

    /// Match between a maker order and a taker order.
    /// The event names both order identities, the instrument, and the fill.
    Trade {
        event_sequence: EventSequence,
        command_sequence: CommandSequence,
        timestamp_nanos: TimestampNanos,
        maker_order_id: OrderId,
        taker_order_id: OrderId,
        instrument_id: InstrumentId,
        price: Price,
        quantity: Quantity,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_copy<T: Copy + PartialEq + std::fmt::Debug>(value: T) {
        let copy = value;
        assert_eq!(value, copy);
    }

    fn sample_accepted() -> EngineEvent {
        EngineEvent::Accepted {
            event_sequence: EventSequence::new(1),
            command_sequence: CommandSequence::new(10),
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        }
    }

    #[test]
    fn engine_event_variants_implement_copy() {
        // Given each EngineEvent variant
        let accepted = sample_accepted();
        let rejected = EngineEvent::Rejected {
            event_sequence: EventSequence::new(2),
            command_sequence: CommandSequence::new(11),
            timestamp_nanos: TimestampNanos::new(2000),
            client_order_id: ClientOrderId::new(7),
            reason: EngineEventRejectReason::InvalidQuantity,
        };
        let replaced = EngineEvent::Replaced {
            event_sequence: EventSequence::new(3),
            command_sequence: CommandSequence::new(12),
            timestamp_nanos: TimestampNanos::new(3000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
        };
        let canceled = EngineEvent::Canceled {
            event_sequence: EventSequence::new(4),
            command_sequence: CommandSequence::new(13),
            timestamp_nanos: TimestampNanos::new(4000),
            order_id: OrderId::new(42),
        };
        let trade = EngineEvent::Trade {
            event_sequence: EventSequence::new(5),
            command_sequence: CommandSequence::new(14),
            timestamp_nanos: TimestampNanos::new(5000),
            maker_order_id: OrderId::new(10),
            taker_order_id: OrderId::new(20),
            instrument_id: InstrumentId::new(2),
            price: Price::new(100),
            quantity: Quantity::new(5),
        };

        // When we copy each event
        // Then the original and the copy compare equal
        assert_copy(accepted);
        assert_copy(rejected);
        assert_copy(replaced);
        assert_copy(canceled);
        assert_copy(trade);
    }

    #[test]
    fn engine_event_reject_reason_variants_are_distinct() {
        // Given every rejection reason
        let reasons = [
            EngineEventRejectReason::InvalidQuantity,
            EngineEventRejectReason::UnknownInstrument,
            EngineEventRejectReason::OrderNotFound,
            EngineEventRejectReason::InvalidPrice,
            EngineEventRejectReason::Other,
        ];

        // When we copy each reason
        // Then each reason equals its copy and InvalidQuantity is not Other
        for reason in reasons {
            assert_copy(reason);
        }
        assert_ne!(
            EngineEventRejectReason::InvalidQuantity,
            EngineEventRejectReason::Other
        );
    }

    #[test]
    fn engine_event_equality() {
        // Given two Accepted events with the same fields
        let event_one = sample_accepted();
        let event_two = sample_accepted();
        let different_sequence = EngineEvent::Accepted {
            event_sequence: EventSequence::new(2),
            command_sequence: CommandSequence::new(10),
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        };

        // When we compare them
        // Then equal events match and a different event sequence does not
        assert_eq!(event_one, event_two);
        assert_ne!(event_one, different_sequence);
    }

    #[test]
    fn accepted_carries_event_sequence_and_command_sequence() {
        // Given an Accepted event with specific clocks and identities
        let accepted_event = sample_accepted();

        // When we read the event
        // Then both clocks and both order identifiers are present
        let EngineEvent::Accepted {
            event_sequence,
            command_sequence,
            timestamp_nanos,
            order_id,
            client_order_id,
            account_id,
        } = accepted_event
        else {
            panic!("expected Accepted");
        };
        assert_eq!(event_sequence.inner(), 1);
        assert_eq!(command_sequence.inner(), 10);
        assert_eq!(timestamp_nanos.inner(), 1000);
        assert_eq!(order_id.inner(), 42);
        assert_eq!(client_order_id.inner(), 7);
        assert_eq!(account_id.inner(), 1);
        assert_copy(accepted_event);
    }

    #[test]
    fn event_sequence_can_differ_from_command_sequence() {
        // Given an Accepted event whose event and command clocks differ
        let accepted = sample_accepted();

        // When we read the clocks
        let EngineEvent::Accepted {
            event_sequence,
            command_sequence,
            ..
        } = accepted
        else {
            panic!("expected Accepted");
        };

        // Then the EventSequence is independent of the producing CommandSequence
        assert_ne!(event_sequence.inner(), command_sequence.inner());
    }

    #[test]
    fn trade_includes_maker_order_id_and_taker_order_id() {
        // Given a Trade event with maker and taker identities
        let trade_event = EngineEvent::Trade {
            event_sequence: EventSequence::new(5),
            command_sequence: CommandSequence::new(14),
            timestamp_nanos: TimestampNanos::new(5000),
            maker_order_id: OrderId::new(10),
            taker_order_id: OrderId::new(20),
            instrument_id: InstrumentId::new(2),
            price: Price::new(100),
            quantity: Quantity::new(5),
        };

        // When we read the event
        // Then both maker and taker order identifiers are present
        let EngineEvent::Trade {
            maker_order_id,
            taker_order_id,
            instrument_id,
            ..
        } = trade_event
        else {
            panic!("expected Trade");
        };
        assert_eq!(maker_order_id.inner(), 10);
        assert_eq!(taker_order_id.inner(), 20);
        assert_eq!(instrument_id.inner(), 2);
        assert_copy(trade_event);
    }
}
