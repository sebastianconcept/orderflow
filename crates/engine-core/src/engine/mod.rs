// Engine core – defines the MatchingEngine API
pub mod types;

use engine_types::{EngineCommand, EngineEvent};

/// Default implementation that does nothing – placeholder.
///
/// This is a test double for scenarios where an engine instance is needed
/// but actual matching logic is not required.
pub struct DummyEngine;

impl DummyEngine {
    /// Create a new DummyEngine instance.
    pub fn new() -> Self {
        DummyEngine
    }
}

impl Default for DummyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MatchingEngine for DummyEngine {
    fn process(&mut self, _command: EngineCommand, out: &mut Vec<EngineEvent>) {
        // This is a test double that doesn't actually process commands.
        // In production, use a real engine implementation.
    }
}