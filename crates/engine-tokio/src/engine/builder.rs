//! Construction of a Tokio Engine.
//!
//! When a caller obtains an Engine with default EngineConfig, it uses this
//! module.

use super::Engine;

/// TokioMatchingEngineBuilder is the construction of a Tokio Engine.
/// When a process needs a Tokio Engine, it uses this type so the Engine has
/// default EngineConfig.
pub struct TokioMatchingEngineBuilder;

impl TokioMatchingEngineBuilder {
    /// Answers an Engine with default EngineConfig.
    pub fn build() -> Engine {
        Engine::default_engine()
    }
}
