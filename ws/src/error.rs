use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

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
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}

pub type WsResult<T> = Result<T, WsError>;
