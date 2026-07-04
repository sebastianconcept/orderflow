// Scheduler placeholder

use engine_types::Order;

pub struct Scheduler;

impl Scheduler {
    pub fn new() -> Self {
        Self
    }

    pub fn schedule(&self, order: Order) -> Order {
        order
    }
}
