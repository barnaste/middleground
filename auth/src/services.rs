// NOTE: USEFUL EXTRACTORS
//       Path, Query<HashMap<String, String>>

use crate::middleware::AuthStatus;
use axum::{
    extract::{Json, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::Value;
use tower_sessions::Session;
use validator::ValidateEmail;

pub const EMAIL_SESSION_KEY: &str = "email";
pub const PASSCODE_SESSION_KEY: &str = "passcode";

// note that passcode is expected to already be hashed by the frontend (maybe use argon2?)
pub async fn register(
    session: Session,
    payload: Result<Json<Value>, JsonRejection>,
) -> Result<StatusCode, impl IntoResponse> {
    // if this fails, the JSON data given does not contain an object
    let Json(payload) = payload.map_err(|_| StatusCode::BAD_REQUEST)?;

    // extract JSON object from the payload
    let map = payload.as_object().ok_or(StatusCode::BAD_REQUEST)?;

    // extract email and passcode
    let email = map
        .get(EMAIL_SESSION_KEY)
        .and_then(|s| s.as_str())
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;
    let passcode = map
        .get(PASSCODE_SESSION_KEY)
        .and_then(|s| s.as_str())
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;

    // ensure email is of correct format
    if !email.validate_email() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // check that email is not already in use; return 409 Conflict if it does

    // insert new user record (later: need database)
    let user_id = 142857; // temporary -- should be safely generated

    // initialize default user profile (later: need to design preferences interface)
    // generate verification code
    let verification_code = 123456; // temporary -- should be safely generated

    // store user_id and authentication details in session
    session
        .insert("user_id", user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    session
        .insert("auth_status", AuthStatus::Pending(verification_code))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // request email verification via notifs (later: need notifications interface)

    Ok(StatusCode::CREATED)
}
