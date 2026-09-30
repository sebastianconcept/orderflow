//! Construction of a Gloomio Engine.
//!
//! When a caller obtains an Engine with default EngineConfig, it uses this
//! module.

use crate::engine::Engine;

/// GloomioMatchingEngineBuilder is the construction of a Gloomio Engine.
/// When a process needs a Gloomio Engine, it uses this type so the Engine has
/// default EngineConfig.
pub struct GloomioMatchingEngineBuilder;

impl GloomioMatchingEngineBuilder {
    /// Answers an Engine with default EngineConfig.
    pub fn build() -> Engine {
        Engine::default_engine()
    }
}
