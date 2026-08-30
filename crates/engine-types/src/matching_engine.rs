//! Matching engine trait and implementation types.
//!
//! This module provides the [`MatchingEngine`] trait that all matching engine
//! implementations must satisfy. The trait signature uses a caller-owned buffer
//! to append events, avoiding unnecessary allocation for each command processed.
//!
//! # Trait Signature
//!
//! ```ignore
//! pub trait MatchingEngine {
//!     fn process(&mut self, command: EngineCommand, out: &mut Vec<EngineEvent>);
//! }
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use engine_types::{EngineCommand, EngineEvent, MatchingEngine};
//!
//! struct MyMatchingEngine {
//!     // engine state
//! }
//!
//! impl MatchingEngine for MyMatchingEngine {
//!     fn process(&mut self, command: EngineCommand, out: &mut Vec<EngineEvent>) {
//!         // Process the command and append events to `out`
//!         match command {
//!             EngineCommand::New { .. } => {
//!                 // Generate Accepted event
//!                 out.push(EngineEvent::Accepted {
//!                     sequence: Sequence::new(1),
//!                     timestamp_nanos: TimestampNanos::new(1000),
//!                     order_id: OrderId::new(42),
//!                     client_order_id: ClientOrderId::new(7),
//!                     account_id: AccountId::new(1),
//!                 });
//!             }
//!             _ => {
//!                 // Handle other command types
//!             }
//!         }
//!     }
//! }
//!
//! let mut engine = MyMatchingEngine::new();
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
//! let mut events: Vec<EngineEvent> = Vec::new();
//! engine.process(command, &mut events);
//! assert_eq!(events.len(), 1); // Event was appended
//! ```
//!
//! # Buffer Behavior
//!
//! The `process` method **appends** to the caller-owned buffer. It does not clear
//! or replace existing events. To process a single command and get only its events,
//! clear the buffer before calling:
//!
//! ```ignore
//! let mut events = Vec::new();
//!
//! engine.process(command1, &mut events); // events.len() == 1
//! events.clear(); // Clear before next command
//! engine.process(command2, &mut events); // events.len() == 1 (only command2's events)
//! ```

use crate::engine_command::EngineCommand;
use crate::engine_event::EngineEvent;

/// Trait that all matching engine implementations must provide.
///
/// The matching engine processes [`EngineCommand`]s and appends resulting
/// [`EngineEvent`]s to a caller-owned buffer. This signature supports both
/// live traffic and replay scenarios where commands are injected into the engine.
///
/// # Usage
///
/// ```ignore
/// use engine_types::{EngineCommand, EngineEvent, MatchingEngine};
///
/// struct MyMatchingEngine;
///
/// impl MatchingEngine for MyMatchingEngine {
///     fn process(&mut self, command: EngineCommand, out: &mut Vec<EngineEvent>) {
///         // Process the command and append events to `out`
///         // out.push(EngineEvent::Accepted { ... });
///     }
/// }
///
/// let mut engine = MyMatchingEngine;
/// let command = EngineCommand::New {
///     account_id: AccountId::new(1),
///     client_order_id: ClientOrderId::new(7),
///     instrument_id: InstrumentId::new(2),
///     side: Side::Buy,
///     order_type: OrderType::Limit,
///     price: Price::new(100),
///     quantity: Quantity::new(10),
/// };
///
/// let mut events: Vec<EngineEvent> = Vec::new();
/// engine.process(command, &mut events);
/// // `events` now contains the result of processing `command`
/// ```
///
/// # Notes
///
/// The trait signature uses `&mut Vec<EngineEvent>` to allow the engine to append
/// events without ownership transfer. Callers should [`Vec::clear()`] the buffer
/// before each command if they want only that command's events.
pub trait MatchingEngine {
    /// Process an engine command and append resulting events to the output buffer.
    ///
    /// # Arguments
    ///
    /// * `command` - The command to process (New, CancelByOrder, CancelByClient, or Replace)
    /// * `out` - Caller-owned buffer to append events to
    ///
    /// # Behavior
    ///
    /// This method appends [`EngineEvent`]s to `out` based on the command type and
    /// engine state. It does not return a `Vec` to avoid unnecessary allocation.
    fn process(&mut self, command: EngineCommand, out: &mut Vec<EngineEvent>);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that process appends to an empty caller buffer.
    #[test]
    fn matching_engine_process_appends_to_caller_buffer() {
        // Given: an empty buffer and a New command
        let mut engine = DummyMatchingEngine;
        let mut out: Vec<EngineEvent> = Vec::new();
        let command = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };

        // When: we process the command
        engine.process(command, &mut out);

        // Then: buffer length grows by one (Accepted event)
        assert_eq!(
            out.len(),
            1,
            "Expected 1 event to be appended to empty buffer"
        );
    }

    /// Test that process appends to a pre-filled caller buffer without clearing it.
    #[test]
    fn matching_engine_process_does_not_clear_caller_buffer() {
        // Given: a pre-filled buffer and a New command
        let mut engine = DummyMatchingEngine;
        let mut out: Vec<EngineEvent> = vec![EngineEvent::Accepted {
            sequence: Sequence::new(1),
            timestamp_nanos: TimestampNanos::new(1000),
            order_id: OrderId::new(42),
            client_order_id: ClientOrderId::new(7),
            account_id: AccountId::new(1),
        }];
        assert_eq!(out.len(), 1, "Buffer should start with 1 event");

        let command = EngineCommand::New {
            account_id: AccountId::new(2),
            client_order_id: ClientOrderId::new(8),
            instrument_id: InstrumentId::new(2),
            side: Side::Sell,
            order_type: OrderType::Market,
            price: Price::new(0),
            quantity: Quantity::new(5),
        };

        // When: we process another command
        engine.process(command, &mut out);

        // Then: buffer has both events (appended, not replaced)
        assert_eq!(out.len(), 2, "Expected 1 more event to be appended");
    }

    /// Test that process does not have an old signature returning Vec.
    #[test]
    fn matching_engine_process_signature_takes_mutable_buffer_not_returning_vec() {
        // This test ensures the trait signature is correct:
        // fn process(&mut self, command: EngineCommand, out: &mut Vec<EngineEvent>)
        // NOT fn process(&mut self, command: EngineCommand) -> Vec<EngineEvent>

        let mut engine = DummyMatchingEngine;
        let command = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };

        let mut out: Vec<EngineEvent> = Vec::new();

        // When: we call process with mutable buffer
        engine.process(command, &mut out);

        // Then: the method returns () (unit), not Vec
        // This is verified at compile time by the trait signature
    }

    /// Dummy implementation for testing that doesn't actually process commands.
    ///
    /// This test double simply appends one Accepted event to demonstrate that
    /// process appends to the caller-owned buffer.
    struct DummyMatchingEngine;

    impl MatchingEngine for DummyMatchingEngine {
        fn process(&mut self, _command: EngineCommand, out: &mut Vec<EngineEvent>) {
            // Append a single Accepted event to demonstrate append behavior
            out.push(EngineEvent::Accepted {
                sequence: Sequence::new(1),
                timestamp_nanos: TimestampNanos::new(1000),
                order_id: OrderId::new(42),
                client_order_id: ClientOrderId::new(7),
                account_id: AccountId::new(1),
            });
        }
    }

    use crate::engine_event::{EngineEvent, Sequence, TimestampNanos};
    use crate::identity::{AccountId, ClientOrderId, InstrumentId, OrderId};
    use crate::order::{OrderType, Side};
    use crate::price::Price;
    use crate::quantity::Quantity;
}
