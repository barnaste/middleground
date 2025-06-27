//! Error types for the auth crate.

use thiserror::Error;

/// Unified error type for authentication operations.
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingAuthHeader,

    #[error("Invalid authorization header format")]
    InvalidAuthHeader,

    #[error("Invalid JWT token: {0}")]
    InvalidToken(String),
}
