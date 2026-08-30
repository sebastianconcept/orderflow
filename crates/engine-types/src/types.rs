//! Engine shared types
//!
//! This module provides core domain types and traits for the matching engine.
//! For order-specific types, see the [`order`] module.

use crate::identity::OrderId;
use crate::order::Order;
use crate::quantity::Quantity;

/// Execution result from a matched order.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Execution {
    pub order_id: OrderId,
    pub filled_quantity: Quantity,
}

/// Trait that all matching engine implementations must provide.
///
/// # Notes
///
/// This trait has been updated to use [`EngineCommand`] and append to a caller-owned buffer
/// of [`EngineEvent`]. The old signature is no longer present.
///
/// # Examples
///
/// ```ignore
/// use engine_types::{MatchingEngine, Order, Execution};
///
/// struct DummyEngine;
///
/// impl MatchingEngine for DummyEngine {
///     fn process(&mut self, order: Order) -> Vec<Execution> {
///         // TODO: implement actual matching logic
///         todo!()
///     }
/// }
///
/// // let mut engine = DummyEngine;
/// // let order = Order::new(0, 0, 0, 0, engine_types::Side::Buy, engine_types::OrderType::Limit, engine_types::Price::new(100), engine_types::Quantity::new(10));
/// // let _executions = engine.process(order);
/// ```
pub trait MatchingEngine {
    /// Process an order and return any executions produced.
    ///
    /// # Arguments
    ///
    /// * `order` - The order to process
    ///
    /// # Returns
    ///
    /// A vector of executions produced by processing the order.
    fn process(&mut self, order: Order) -> Vec<Execution>;
}
