use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WsError {
    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for WsError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
        };

        (status, self.to_string()).into_response()
    }
}
