use super::Engine;

/// Builder for the Tokio matching engine.
///
/// This builder constructs a [`Engine`] instance with default configuration.
pub struct TokioMatchingEngineBuilder;

impl TokioMatchingEngineBuilder {
    /// Build a new tokio engine with default configuration.
    pub fn build() -> Engine {
        Engine::default_engine()
    }
}
