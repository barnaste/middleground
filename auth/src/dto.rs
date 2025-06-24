/// Data Transfer Objects
use crate::models::AuthSession;
use serde::{Deserialize, Serialize};

// TODO: consider what really deserves to be public...

// -----------------
//     REQUESTS
// -----------------

#[derive(Deserialize)]
pub struct SendOtpRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub token: String,
}

// -----------------
//     RESPONSES
// -----------------

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: u64,
}

impl<S: AuthSession> From<S> for AuthResponse {
    fn from(value: S) -> Self {
        AuthResponse {
            access_token: value.access_token().to_string(),
            refresh_token: value.refresh_token().to_string(),
            expires_at: value.expires_at(),
        }
    }
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
