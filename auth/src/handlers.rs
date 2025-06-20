use crate::dto::*;
use crate::managers::AuthManager;

use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::ErrorResponse,
};
use supabase_auth::models::{AuthClient, Session};

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
