//! Engine shared types

use crate::identity::{AccountId, ClientOrderId, InstrumentId, OrderId};
use crate::price::Price;
use crate::quantity::Quantity;

/// Side of an order (buy or sell).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Side {
    /// Buy side
    Buy,
    /// Sell side
    Sell,
}

/// Type of order (limit or market).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OrderType {
    /// Limit order with a specific price
    Limit,
    /// Market order executed at best available price
    Market,
}

/// Order representation with integer fields.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Order {
    pub id: OrderId,
    pub client_order_id: ClientOrderId,
    pub instrument_id: InstrumentId,
    pub account_id: AccountId,
    pub side: Side,
    pub order_type: OrderType,
    pub price: Price,
    pub quantity: Quantity,
}

/// Execution result from a matched order.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Execution {
    pub order_id: OrderId,
    pub filled_quantity: Quantity,
}

/// Trait that all matching engine implementations must provide.
pub trait MatchingEngine {
    /// Process an incoming order and return any executions produced.
    fn process(&mut self, order: Order) -> Vec<Execution>;
}
