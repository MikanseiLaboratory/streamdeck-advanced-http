//! HTTP request execution for the Advanced HTTP Stream Deck plugin.
//!
//! This crate owns request validation and execution only. Stream Deck settings
//! live in the plugin crate so the engine can be tested without a deck.

mod engine;
mod method;
mod spec;

pub use engine::{HttpEngine, Outcome};
pub use method::HttpMethod;
pub use spec::{Auth, RequestSpec};

/// Errors produced while building or sending a request.
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    /// The method is empty or is not a valid HTTP token.
    #[error("invalid HTTP method: {0:?}")]
    Method(String),
    /// The URL could not be parsed.
    #[error("invalid URL: {0}")]
    Url(String),
    /// The Authorization header value contains invalid bytes.
    #[error("invalid Authorization header")]
    Authorization,
    /// The request could not be sent, timed out, or the response could not be read.
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}
