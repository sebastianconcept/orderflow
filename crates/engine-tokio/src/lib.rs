//! Tokio-backed matching engine.
//!
//! When a process constructs an Engine, it uses this crate so the engine
//! implements MatchingEngine on Tokio. Main types: [Engine],
//! [TokioMatchingEngineBuilder].

pub mod engine;
pub mod error;

pub use engine::config::EngineConfig;
pub use engine::{Engine, TokioMatchingEngineBuilder};
pub use error::EngineError;
