// Engine core – defines the MatchingEngine API
pub mod types;

use protocol::{Execution, Order};

/// Default implementation that does nothing – placeholder.
pub struct DummyEngine;
impl MatchingEngine for DummyEngine {
    fn process(&mut self, _order: Order) -> Vec<Execution> {
        vec![]
    }
}