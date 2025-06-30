//! Error types for the auth crate.

use thiserror::Error;

/// Unified error type for authentication operations.
///
/// This enum covers all possible error scenarios that can occur when
/// during authentication, providing detailed context for debugging
/// and error handling.
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Missing authorization header")]
    MissingAuthHeader,

    #[error("Invalid authorization header format")]
    InvalidAuthHeader,

    #[error("Invalid JWT token: {0}")]
    InvalidToken(String),
}
