//! Engine shared types

use crate::identity::{AccountId, ClientOrderId, InstrumentId, OrderId};

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

/// Price in ticks (i64).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Price(i64);

impl Price {
    /// Create a new Price from ticks.
    pub fn new(ticks: i64) -> Self {
        Price(ticks)
    }

    /// Get the inner ticks value.
    pub fn inner(&self) -> i64 {
        self.0
    }
}

/// Quantity in lots (u128).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Quantity(u128);

impl Quantity {
    /// Create a new Quantity from lots.
    pub fn new(lots: u128) -> Self {
        Quantity(lots)
    }

    /// Get the inner lots value.
    pub fn inner(&self) -> u128 {
        self.0
    }
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
