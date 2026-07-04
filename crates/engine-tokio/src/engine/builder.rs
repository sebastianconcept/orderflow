use super::Engine;

pub struct TokioMatchingEngineBuilder;

impl TokioMatchingEngineBuilder {
    pub fn build() -> Engine {
        Engine::default()
    }
}
