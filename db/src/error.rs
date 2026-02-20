//! Error types for the db crate.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, DbError>;

/// Unified error type for database operations.
///
/// This enum covers all possible error scenarios that can occur when
/// interacting with the database, providing detailed context for debugging
/// and error handling.
#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(sqlx::Error),

    #[error("Query execution error: {0}")]
    Query(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),
}
