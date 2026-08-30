use crate::engine::Engine;

pub struct GloomioMatchingEngineBuilder;

impl GloomioMatchingEngineBuilder {
    pub fn build() -> Engine {
        Engine::default_engine()
    }
}
