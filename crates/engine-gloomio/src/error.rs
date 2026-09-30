//! Failure of the Gloomio matching engine.
//!
//! When a caller distinguishes configuration, startup, and runtime failure, it
//! uses this module so each phase stays a distinct variant.

use displaydoc::Display;
use thiserror::Error;

/// EngineError is the failure of the Gloomio matching engine.
/// Configuration, startup, and runtime failures stay distinct.
#[derive(Debug, Display, Error)]
pub enum EngineError {
    /// Engine configuration error: {0}
    Config(&'static str),
    /// Engine startup error: {0}
    Startup(&'static str),
    /// Engine runtime error: {0}
    Runtime(&'static str),
}
