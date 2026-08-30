pub mod builder;
pub mod config;

pub use builder::*;
pub use config::Config;
use engine_types::{Execution, MatchingEngine, Order};

/// Default implementation that does nothing – placeholder.
#[allow(dead_code)]
pub struct Engine {
    config: Config,
}

impl Engine {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn default_engine() -> Self {
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
        todo!("Implement tokio matching engine logic");
    }
}
