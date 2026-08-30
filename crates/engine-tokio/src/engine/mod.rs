pub mod builder;
pub mod config;

pub use builder::*;
pub use config::Config;
use engine_types::{EngineCommand, EngineEvent, MatchingEngine};

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
    fn process(&mut self, _command: EngineCommand, _out: &mut Vec<EngineEvent>) {
        todo!("Implement tokio matching engine logic");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_types::{
        AccountId, ClientOrderId, EngineCommand, EngineEvent, InstrumentId, OrderType, Price,
        Quantity, Side,
    };

    /// Test that Engine implements MatchingEngine with the correct process signature.
    #[test]
    fn engine_implements_matching_engine_process_signature() {
        // Given: a tokio engine instance
        let mut engine = Engine::for_test();

        // When/Then: process can be called with EngineCommand and mutable Vec<EngineEvent>
        let command = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };
        let mut out: Vec<EngineEvent> = Vec::new();

        // This call compiles and uses the correct signature:
        // fn process(&mut self, command: EngineCommand, out: &mut Vec<EngineEvent>)
        // It panics because of todo!() inside, but we verify compilation
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine.process(command, &mut out);
        }));

        // Verify the signature: process returns () and panics (as expected)
        assert!(result.is_err(), "Expected process to panic due to todo!()");
    }

    /// Test that calling process panics (todo path) and tests can catch it.
    #[test]
    #[should_panic(expected = "Implement tokio matching engine logic")]
    fn engine_process_todo_panics_in_test_that_catches_should_panic() {
        // Given: a tokio engine
        let mut engine = Engine::for_test();

        // When: process is called (which has todo!() in body)
        let command = EngineCommand::New {
            account_id: AccountId::new(1),
            client_order_id: ClientOrderId::new(7),
            instrument_id: InstrumentId::new(2),
            side: Side::Buy,
            order_type: OrderType::Limit,
            price: Price::new(100),
            quantity: Quantity::new(10),
        };
        let mut out: Vec<EngineEvent> = Vec::new();

        // Then: process panics with the todo message
        engine.process(command, &mut out);
    }

    /// Test that TokioMatchingEngineBuilder still builds an Engine.
    #[test]
    fn engine_builder_still_builds_engine() {
        // Given: the builder
        let engine = TokioMatchingEngineBuilder::build();

        // When/Then: builder succeeds and returns Engine
        assert!(
            std::mem::size_of_val(&engine) > 0,
            "Engine should be constructible"
        );
    }
}
