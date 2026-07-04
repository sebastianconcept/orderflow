//! Engine shared types

use rkyv::{Archive, Deserialize, Serialize};


/// Trait that all matching engine implementations must provide.
pub trait MatchingEngine {
    /// Process an incoming order and return any executions produced.
    fn process(&mut self, order: Order) -> Vec<Execution>;
}


#[derive(Archive, Deserialize, Serialize, Debug)]
pub struct OrderId(u64);

impl OrderId {
    pub fn new(id: u64) -> Self {
        OrderId(id)
    }
}

#[derive(Archive, Deserialize, Serialize, Debug)]
pub struct Order {
    pub id: OrderId,
    pub quantity: f64,
}

#[derive(Archive, Deserialize, Serialize, Debug)]
pub struct Execution {
    pub order_id: OrderId,
    pub filled_quantity: f64,
}
