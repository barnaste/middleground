use crate::dto::*;
use crate::auth_manager::AuthManager;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::ErrorResponse,
};
use supabase_auth::models::{AuthClient, Session};

// -----------------
//      HELPERS
// -----------------

// TODO: if you plan on keeping this function, make it Session -> AuthSession

/// Convert a Session to a returnable AuthResponse.
/// This will consume the Session.
fn session_to_auth_response(session: Session) -> AuthResponse {
    AuthResponse {
        access_token: session.access_token,
        refresh_token: session.refresh_token,
        expires_at: session.expires_at,
    }
}

// -----------------
//     ENDPOINTS
// -----------------

// TODO:
//  1. send_otp
//  2. verify_otp
//  3. logout
//  4. refresh_token

pub async fn send_otp<A: AuthManager>(
    State(state): State<A>,
    Json(payload): Json<SendOtpRequest>,
) -> Result<Json<MessageResponse>, (StatusCode, Json<ErrorResponse>)> {
    todo!()
}
