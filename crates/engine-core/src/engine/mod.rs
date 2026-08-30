// Engine core – defines the MatchingEngine API
pub mod types;

use engine_types::{EngineCommand, EngineEvent, Execution};

/// Default implementation that does nothing – placeholder.
pub struct DummyEngine;
impl MatchingEngine for DummyEngine {
    fn process(&mut self, _command: EngineCommand, _out: &mut Vec<EngineEvent>) {
        vec![]
    }
}