//! Engine shared types
//!
//! This module provides core domain types and traits for the matching engine.
//! For order-specific types, see the [`order`] module.

use crate::engine_command::EngineCommand;
use crate::engine_event::EngineEvent;
use crate::identity::OrderId;
use crate::quantity::Quantity;

/// Execution result from a matched order.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Execution {
    pub order_id: OrderId,
    pub filled_quantity: Quantity,
}

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
/// struct MyEngine;
///
/// impl MatchingEngine for MyEngine {
///     fn process(&mut self, command: EngineCommand, out: &mut Vec<EngineEvent>) {
///         // Process the command and append events to `out`
///         // out.push(EngineEvent::Accepted { ... });
///         todo!()
///     }
/// }
///
/// let mut engine = MyEngine;
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
