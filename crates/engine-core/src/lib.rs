//! Binary entry that constructs a Tokio matching engine.
//!
//! When a process starts, it uses this crate so [run] answers an Engine.
//! Main types: [Engine], [TokioMatchingEngineBuilder].

pub use engine_tokio::{Engine, TokioMatchingEngineBuilder};

/// Answers a constructed Tokio matching engine for this binary.
pub fn run() -> Engine {
    TokioMatchingEngineBuilder::build()
}

#[cfg(test)]
mod tests {
    use super::run;
    use engine_types::{
        AccountId, ClientOrderId, CommandSequence, EngineCommand, EngineEvent, InstrumentId,
        MatchingEngine, Price, Quantity, SequencedCommand, Side,
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
    fn run_constructs_tokio_engine() {
        // Given the engine-core entry
        // When run constructs an engine
        let mut engine = run();
        let mut out: Vec<EngineEvent> = Vec::new();

        // Then process appends no events in this slice
        engine.process(sample_new_limit_command(), &mut out);
        assert!(out.is_empty());
    }
}
