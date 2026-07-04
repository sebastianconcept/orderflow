// Sequencer placeholder

use engine_types::Order;

pub struct Sequencer;

impl Sequencer {
    pub fn new() -> Self {
        Self
    }

    pub fn next_sequence(&self) -> u64 {
        0
    }

    pub fn process(&self, order: Order) -> Order {
        order
    }
}
