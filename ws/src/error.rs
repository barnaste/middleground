use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WsError {
    #[error("Unauthorized")]
    Unauthorized,

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
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
