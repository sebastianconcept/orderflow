//! Shared domain types for Price ticks, Quantity lots, commands, and events.
//!
//! When a matching engine or the journal codec names a domain value, it uses this
//! crate so both share one set of types. Main types: [Price], [Quantity],
//! [EngineCommand], [EngineEvent], [SequencedCommand], [MatchingEngine].

pub mod engine_command;
pub mod engine_event;
pub mod identity;
pub mod instrument_spec;
pub mod matching_engine;
pub mod order;
pub mod price;
pub mod quantity;
pub mod sequenced_command;
pub mod sequencer;

pub use engine_command::EngineCommand;
pub use engine_event::{EngineEvent, EngineEventRejectReason};
pub use identity::{
    AccountId, ClientOrderId, CommandSequence, EventSequence, InstrumentId, JournalSequence,
    OrderId, SessionId, TimestampNanos,
};
pub use instrument_spec::{InstrumentSpec, PriceParseError, QuantityParseError};
pub use matching_engine::MatchingEngine;
pub use order::{Order, OrderType, Side};
pub use price::Price;
pub use quantity::Quantity;
pub use sequenced_command::SequencedCommand;
pub use sequencer::{CommandSequencer, EventSequencer};
