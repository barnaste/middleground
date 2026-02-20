//! Error types for the ws crate.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

/// Unified error type for WebSocket operations.
///
/// Covers all posible failure scenarios in WebSocket operations with proper HTTP status code
/// mapping.
#[derive(Error, Debug)]
pub enum WsError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Database error: {0}")]
    Database(#[from] db::error::DbError),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] axum::Error),
}

impl IntoResponse for WsError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Json(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WebSocket(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}

/// Convenience type alias for WebSocket operation results.
///
/// Shorthand for `Result<T, WsError>`.
pub type WsResult<T> = Result<T, WsError>;
