//! Tokio matching engine.
//!
//! When a caller constructs an Engine that implements MatchingEngine, it uses
//! this module so the engine is bound to EngineConfig. process leaves the
//! event buffer unchanged.

pub mod builder;
pub mod config;

pub use builder::TokioMatchingEngineBuilder;
pub use config::EngineConfig;
use engine_types::{EngineEvent, MatchingEngine, SequencedCommand};

/// Engine is the Tokio matching engine.
/// When a caller processes a SequencedCommand on a Tokio runtime, it uses this
/// type. process leaves the event buffer unchanged.
pub struct Engine {
    #[allow(dead_code)]
    config: EngineConfig,
}

impl Engine {
    /// Answers an Engine from EngineConfig.
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Answers an Engine with default EngineConfig.
    pub fn default_engine() -> Self {
        Self {
            config: EngineConfig::default(),
        }
    }

    /// Answers an Engine with test EngineConfig.
    #[cfg(test)]
    pub fn for_test() -> Self {
        Self {
            config: EngineConfig::for_test(),
        }
    }
}

impl MatchingEngine for Engine {
    fn process(&mut self, _command: SequencedCommand, _out: &mut Vec<EngineEvent>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_types::{
        AccountId, ClientOrderId, CommandSequence, EngineCommand, EngineEvent, InstrumentId, Price,
        Quantity, Side,
    };

    fn sample_new_limit_command() -> SequencedCommand {
        SequencedCommand::new(
            CommandSequence::new(0),
            EngineCommand::NewLimit {
                account_id: AccountId::new(1),
                client_order_id: ClientOrderId::new(7),
                instrument_id: InstrumentId::new(2),
                side: Side::Buy,
                price: Price::new(100),
                quantity: Quantity::new(10),
            },
        )
    }

    #[test]
    fn process_appends_no_events_in_this_slice() {
        // Given a Tokio engine and an empty event buffer
        let mut engine = Engine::for_test();
        let command = sample_new_limit_command();
        let mut out: Vec<EngineEvent> = Vec::new();

        // When process runs
        engine.process(command, &mut out);

        // Then no events are appended
        assert!(out.is_empty());
    }

    #[test]
    fn process_does_not_clear_caller_buffer() {
        // Given a buffer that already holds one event
        let mut engine = Engine::for_test();
        let mut out = vec![EngineEvent::Canceled {
            event_sequence: engine_types::EventSequence::new(1),
            command_sequence: CommandSequence::new(0),
            timestamp_nanos: engine_types::TimestampNanos::new(1),
            order_id: engine_types::OrderId::new(1),
        }];

        // When process runs
        engine.process(sample_new_limit_command(), &mut out);

        // Then the existing event remains
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn engine_builder_builds_engine() {
        // Given the Tokio matching engine builder
        // When we build
        let engine = TokioMatchingEngineBuilder::build();

        // Then process appends no events
        let mut out: Vec<EngineEvent> = Vec::new();
        let mut engine = engine;
        engine.process(sample_new_limit_command(), &mut out);
        assert!(out.is_empty());
    }
}
