// Gloomio matching engine

pub mod builder;
pub mod config;

pub use builder::GloomioMatchingEngineBuilder;
use config::Config;
use engine_types::{Order, Execution, MatchingEngine};

pub struct Engine {
    config: Config,
}
impl Engine {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self {
            config: Config::default(),
        }
    }

    #[cfg(test)]
    pub fn for_test() -> Self {
        Self {
            config: Config::for_test(),
        }
    }
}

impl MatchingEngine for Engine {
    fn process(&mut self, _order: Order) -> Vec<Execution> {
        // TODO: real implementation
        todo!("Implement gloomio matching engine logic");
    }
}
