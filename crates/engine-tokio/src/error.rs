use displaydoc::Display;
use thiserror::Error;

/// Error type for the tokio engine.
#[derive(Debug, Display, Error)]
pub enum Error {
    /// Engine configuration error: {0}
    Config(&'static str),
    /// Engine startup error: {0}
    Startup(&'static str),
    /// Engine runtime error: {0}
    Runtime(&'static str),
}
