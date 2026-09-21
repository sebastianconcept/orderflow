//! Gloomio-backed matching engine.
//!
//! When a caller constructs an Engine, it uses this crate so the engine
//! implements MatchingEngine on Gloomio. Main types: [Engine],
//! [GloomioMatchingEngineBuilder].

pub mod engine;
pub mod error;

pub use engine::config::EngineConfig;
pub use engine::{Engine, GloomioMatchingEngineBuilder};
pub use error::EngineError;
