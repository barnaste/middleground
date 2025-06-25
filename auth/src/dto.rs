//! Data Transfer Objects for API requests and responses.

use crate::models::AuthSession;
use serde::{Deserialize, Serialize};

// -----------------
//     REQUESTS
// -----------------

/// Request to send OTP to a user's contact (e.g. email).
#[derive(Deserialize)]
pub struct SendOtpRequest {
    pub contact: String,
}

/// Request to verify OTP and authenticate user.
#[derive(Deserialize)]
pub struct VerifyOtpRequest {
    pub contact: String,
    pub token: String,
}

// -----------------
//     RESPONSES
// -----------------

/// Authentication response containing tokens and expiration.
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

/// Generic success message response.
#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

/// Error response for failed operations.
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
