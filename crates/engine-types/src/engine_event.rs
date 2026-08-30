//! Engine events representing state transitions in the matching engine.
//!
//! This module provides the event types that the matching engine emits as a result
//! of processing [`EngineCommand`]s. Events represent state changes such as order
//! acceptance, rejection, cancellation, replacement, and trades.
//!
//! # Event Variants
//!
//! - [`EngineEvent::Accepted`]: An order has been accepted into the order book.
//! - [`EngineEvent::Rejected`]: A command was rejected (e.g., invalid quantity, unknown instrument).
//! - [`EngineEvent::Replaced`]: An order has been replaced with modified price/quantity.
//! - [`EngineEvent::Canceled`]: An order has been canceled before execution.
//! - [`EngineEvent::Trade`]: A trade occurred between a maker and taker order.
//!
//! # Event Lifecycle
//!
//! Events are appended by the [`MatchingEngine`] to a caller-owned buffer:
//!
//! ```
//! use engine_types::{AccountId, ClientOrderId, EngineCommand, EngineEvent, InstrumentId, MatchingEngine, OrderType, Price, Quantity, Side};
//!
//! struct MyEngine;
//!
//! impl MatchingEngine for MyEngine {
//!     fn process(&mut self, _command: EngineCommand, _out: &mut Vec<EngineEvent>) {
//!         // TODO: implement actual matching logic
//!     }
//! }
//!
//! let mut engine = MyEngine;
//! let command = EngineCommand::New {
//!     account_id: AccountId::new(1),
//!     client_order_id: ClientOrderId::new(7),
//!     instrument_id: InstrumentId::new(2),
//!     side: Side::Buy,
//!     order_type: OrderType::Limit,
//!     price: Price::new(100),
//!     quantity: Quantity::new(10),
//! };
//!
//! let mut events = Vec::new();
//! engine.process(command, &mut events);
//! // `events` now contains the result of processing `command`
//! ```
//!
//! # Copy Semantics
//!
//! All event variants implement [`Copy`] because they contain only integer types that are `Copy`.
//! This allows efficient event propagation without ownership transfer.

use crate::identity::{AccountId, ClientOrderId, InstrumentId, OrderId};
use crate::price::Price;
use crate::quantity::Quantity;

/// Reason why an engine event was rejected.
///
/// This is a closed enum representing all possible rejection reasons.
/// Use this with [`EngineEvent::Rejected`] to communicate why a command failed.
///
/// # Wire Encoding
///
/// In the protocol, rejection reasons are encoded as `u8`:
///
/// - `0` → `InvalidQuantity`
/// - `1` → `UnknownInstrument`
/// - `2` → `OrderNotFound`
/// - `3` → `InvalidPrice`
/// - `255` → `Other`
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EngineEventRejectReason {
    /// The quantity in the command was invalid (e.g., zero, negative, or exceeds limits).
    InvalidQuantity,
    /// The instrument specified in the command is unknown to this engine.
    UnknownInstrument,
    /// The order ID referenced in a cancel or replace command does not exist.
    OrderNotFound,
    /// The price in the command is invalid (e.g., negative or exceeds tick limits).
    InvalidPrice,
    /// A rejection reason not covered by the above variants.
    Other,
}

/// Engine event representing a state transition in the matching process.
///
/// Events are emitted by the [`MatchingEngine`] when processing [`EngineCommand`]s.
/// Each event contains a sequence number and timestamp for ordering and auditing purposes.
///
/// # Event Types
///
/// - [`EngineEvent::Accepted`]: Order accepted into the order book.
///   Contains: `sequence`, `timestamp_nanos`, `order_id`, `client_order_id`, `account_id`
/// - [`EngineEvent::Rejected`]: Command rejected due to validation failure.
///   Contains: `sequence`, `timestamp_nanos`, `client_order_id`, `reason`
/// - [`EngineEvent::Replaced`]: Order replaced with new price/quantity.
///   Contains: `sequence`, `timestamp_nanos`, `order_id`, `client_order_id`
/// - [`EngineEvent::Canceled`]: Order canceled before execution.
///   Contains: `sequence`, `timestamp_nanos`, `order_id`
/// - [`EngineEvent::Trade`]: Matched execution between maker and taker.
///   Contains: `sequence`, `timestamp_nanos`, `maker_order_id`, `taker_order_id`, `instrument_id`, `price`, `quantity`
///
/// # Usage
///
/// Events are appended to a caller-owned buffer by the [`MatchingEngine::process`] method:
///
/// ```
/// use engine_types::{EngineCommand, EngineEvent};
///
/// let mut events: Vec<EngineEvent> = Vec::new();
/// // engine.process(command, &mut events) appends to `events`
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    /// An order has been accepted into the matching engine's order book.
    ///
    /// # Fields
    ///
    /// * `sequence` - Monotonically increasing sequence number for this session.
    /// * `timestamp_nanos` - Timestamp when the order was accepted (nanoseconds).
    /// * `order_id` - Engine-assigned unique identifier for the order.
    /// * `client_order_id` - Client-assigned identifier from the original request.
    /// * `account_id` - Identifier for the trading account that placed the order.
    ///
    /// # Notes
    ///
    /// This event is emitted when a [`EngineCommand::New`] command is successfully processed.
    Accepted {
        sequence: Sequence,
        timestamp_nanos: TimestampNanos,
        order_id: OrderId,
        client_order_id: ClientOrderId,
        account_id: AccountId,
    },

    /// A command was rejected due to validation or business rule failure.
    ///
    /// # Fields
    ///
    /// * `sequence` - Monotonically increasing sequence number for this session.
    /// * `timestamp_nanos` - Timestamp when the rejection occurred (nanoseconds).
    /// * `client_order_id` - Client-assigned identifier from the original request.
    ///   For commands without a client order ID (e.g., `CancelByOrder`), this is `ClientOrderId(0)`.
    /// * `reason` - The specific reason for rejection.
    ///
    /// # Notes
    ///
    /// This event is emitted when a command fails validation or business logic checks.
    /// The `client_order_id` field allows the caller to correlate rejection with request,
    /// even if the command type (e.g., `CancelByOrder`) doesn't normally carry a client ID.
    Rejected {
        sequence: Sequence,
        timestamp_nanos: TimestampNanos,
        client_order_id: ClientOrderId,
        reason: EngineEventRejectReason,
    },

    /// An order has been replaced with modified price and/or quantity.
    ///
    /// # Fields
    ///
    /// * `sequence` - Monotonically increasing sequence number for this session.
    /// * `timestamp_nanos` - Timestamp when the replacement occurred (nanoseconds).
    /// * `order_id` - Engine-assigned identifier for the original order.
    /// * `client_order_id` - Client-assigned identifier from the REPLACE request
    ///   (distinct from the original order's `client_order_id`).
    ///
    /// # Notes
    ///
    /// The `client_order_id` here is from the [`EngineCommand::Replace`] command,
    /// not from the original [`EngineCommand::New`]. This allows tracking each
    /// replace request independently.
    Replaced {
        sequence: Sequence,
        timestamp_nanos: TimestampNanos,
        order_id: OrderId,
        client_order_id: ClientOrderId,
    },

    /// An order has been canceled before it could be executed.
    ///
    /// # Fields
    ///
    /// * `sequence` - Monotonically increasing sequence number for this session.
    /// * `timestamp_nanos` - Timestamp when the cancellation occurred (nanoseconds).
    /// * `order_id` - Engine-assigned identifier for the canceled order.
    ///
    /// # Notes
    ///
    /// This event is emitted when either [`EngineCommand::CancelByOrder`] or
    /// [`EngineCommand::CancelByClient`] is successfully processed.
    Canceled {
        sequence: Sequence,
        timestamp_nanos: TimestampNanos,
        order_id: OrderId,
    },

    /// A trade occurred between a maker and taker order.
    ///
    /// # Fields
    ///
    /// * `sequence` - Monotonically increasing sequence number for this session.
    /// * `timestamp_nanos` - Timestamp when the trade occurred (nanoseconds).
    /// * `maker_order_id` - Engine-assigned identifier for the resting order (maker).
    /// * `taker_order_id` - Engine-assigned identifier for the new order (taker).
    /// * `instrument_id` - Identifier for the tradable instrument.
    /// * `price` - Trade price in ticks.
    /// * `quantity` - Traded quantity in lots.
    ///
    /// # Notes
    ///
    /// This event is emitted when an incoming order matches with one or more resting orders.
    Trade {
        sequence: Sequence,
        timestamp_nanos: TimestampNanos,
        maker_order_id: OrderId,
        taker_order_id: OrderId,
        instrument_id: InstrumentId,
        price: Price,
        quantity: Quantity,
    },
}

/// Sequence number - monotonically increasing counter within a session.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Sequence(u64);

impl Sequence {
    /// Create a new Sequence from a raw u64 value.
    pub fn new(id: u64) -> Self {
        Sequence(id)
    }

    /// Get the inner u64 value.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

/// Timestamp in nanoseconds.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TimestampNanos(u64);

impl TimestampNanos {
    /// Create a new TimestampNanos from a raw u64 value.
    pub fn new(nanos: u64) -> Self {
        TimestampNanos(nanos)
    }

    /// Get the inner u64 value.
    pub fn inner(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that EngineEvent variants implement Copy.
    #[test]
    fn engine_event_variants_implement_copy() {
        // Test Accepted is Copy
        let original_accepted = EngineEvent::Accepted {
            sequence: Sequence::new(1),
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        };

        let _copied_accepted = original_accepted;
        match original_accepted {
            EngineEvent::Accepted { sequence, .. } => {
                assert_eq!(sequence.inner(), 1);
            }
            _ => unreachable!(),
        }

        // Test Rejected is Copy
        let original_rejected = EngineEvent::Rejected {
            sequence: Sequence::new(2),
            timestamp_nanos: TimestampNanos::new(2000),
            client_order_id: ClientOrderId::new(7),
            reason: EngineEventRejectReason::InvalidQuantity,
        };

        let _copied_rejected = original_rejected;
        match original_rejected {
            EngineEvent::Rejected { reason, .. } => {
                assert_eq!(reason, EngineEventRejectReason::InvalidQuantity);
            }
            _ => unreachable!(),
        }

        // Test Replaced is Copy
        let original_replaced = EngineEvent::Replaced {
            sequence: Sequence::new(3),
            timestamp_nanos: TimestampNanos::new(3000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(99),
        };

        let _copied_replaced = original_replaced;
        match original_replaced {
            EngineEvent::Replaced {
                client_order_id, ..
            } => {
                assert_eq!(client_order_id.inner(), 99);
            }
            _ => unreachable!(),
        }

        // Test Canceled is Copy
        let original_canceled = EngineEvent::Canceled {
            sequence: Sequence::new(4),
            timestamp_nanos: TimestampNanos::new(4000),
            order_id: OrderId::new(42),
        };

        let _copied_canceled = original_canceled;
        match original_canceled {
            EngineEvent::Canceled { order_id, .. } => {
                assert_eq!(order_id.inner(), 42);
            }
            _ => unreachable!(),
        }

        // Test Trade is Copy
        let original_trade = EngineEvent::Trade {
            sequence: Sequence::new(5),
            timestamp_nanos: TimestampNanos::new(5000),
            maker_order_id: OrderId::new(10),
            taker_order_id: OrderId::new(20),
            instrument_id: InstrumentId::new(2),
            price: Price::new(100),
            quantity: Quantity::new(5),
        };

        let _copied_trade = original_trade;
        match original_trade {
            EngineEvent::Trade { price, .. } => {
                assert_eq!(price.inner(), 100);
            }
            _ => unreachable!(),
        }
    }

    /// Test EngineEventRejectReason variants.
    #[test]
    fn engine_event_reject_reason_variants() {
        // Test all rejection reasons
        let reasons = vec![
            EngineEventRejectReason::InvalidQuantity,
            EngineEventRejectReason::UnknownInstrument,
            EngineEventRejectReason::OrderNotFound,
            EngineEventRejectReason::InvalidPrice,
            EngineEventRejectReason::Other,
        ];

        for reason in reasons {
            // Test Copy
            let original = reason;
            let copy = original;
            assert_eq!(original, copy);

            // Test Debug
            let _debug_str = format!("{:?}", reason);
        }
    }

    /// Test EngineEvent equality.
    #[test]
    fn engine_event_equality() {
        // Test Accepted equality
        let event1 = EngineEvent::Accepted {
            sequence: Sequence::new(1),
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        };

        let event2 = EngineEvent::Accepted {
            sequence: Sequence::new(1),
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        };

        assert_eq!(event1, event2);

        // Test inequality
        let event3 = EngineEvent::Accepted {
            sequence: Sequence::new(2), // Different
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        };

        assert_ne!(event1, event3);

        // Test Rejected equality
        let reject1 = EngineEvent::Rejected {
            sequence: Sequence::new(1),
            timestamp_nanos: TimestampNanos::new(1000),
            client_order_id: ClientOrderId::new(7),
            reason: EngineEventRejectReason::InvalidQuantity,
        };

        let reject2 = EngineEvent::Rejected {
            sequence: Sequence::new(1),
            timestamp_nanos: TimestampNanos::new(1000),
            client_order_id: ClientOrderId::new(7),
            reason: EngineEventRejectReason::InvalidQuantity,
        };

        assert_eq!(reject1, reject2);

        // Test Trade equality
        let trade1 = EngineEvent::Trade {
            sequence: Sequence::new(5),
            timestamp_nanos: TimestampNanos::new(5000),
            maker_order_id: OrderId::new(10),
            taker_order_id: OrderId::new(20),
            instrument_id: InstrumentId::new(2),
            price: Price::new(100),
            quantity: Quantity::new(5),
        };

        let trade2 = EngineEvent::Trade {
            sequence: Sequence::new(5),
            timestamp_nanos: TimestampNanos::new(5000),
            maker_order_id: OrderId::new(10),
            taker_order_id: OrderId::new(20),
            instrument_id: InstrumentId::new(2),
            price: Price::new(100),
            quantity: Quantity::new(5),
        };

        assert_eq!(trade1, trade2);
    }

    /// Test Sequence and TimestampNanos types.
    #[test]
    fn sequence_and_timestamp_nanos_work() {
        let sequence = Sequence::new(42);
        assert_eq!(sequence.inner(), 42);

        let timestamp = TimestampNanos::new(1234567890);
        assert_eq!(timestamp.inner(), 1234567890);

        // Test Copy
        let copy_sequence = sequence;
        assert_eq!(sequence.inner(), 42);
        assert_eq!(copy_sequence.inner(), 42);

        let copy_timestamp = timestamp;
        assert_eq!(timestamp.inner(), 1234567890);
        assert_eq!(copy_timestamp.inner(), 1234567890);
    }
}
