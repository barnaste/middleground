// TODO: register(), verify(), login(), logout()
// TODO: use an in-memory database for now, we'll switch to sqlx later

// NOTE: USEFUL EXTRACTORS
//       Path, Query<HashMap<String, String>>, OriginalUri, Json (consumes body), HeaderMap (all headers)

use axum::{
    extract::{Json, rejection::JsonRejection},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::Value;
use validator::ValidateEmail;

// note that passcode is expected to already be hashed by the frontend
pub async fn register(
    payload: Result<Json<Value>, JsonRejection>,
) -> Result<StatusCode, impl IntoResponse> {
    // if this fails, the JSON data given does not contain an object
    let Json(payload) = payload.map_err(|_| StatusCode::BAD_REQUEST)?;

    // extract JSON object from the payload
    let map = payload.as_object().ok_or(StatusCode::BAD_REQUEST)?;

    // extract email and passcode
    let email = map
        .get("email")
        .and_then(|s| s.as_str())
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;
    let passcode = map
        .get("passcode")
        .and_then(|s| s.as_str())
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;
    
    // ensure email is of correct format
    if !email.validate_email() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // check to see if information we received is correct
    println!("Email: {}, Passcode: {}", email, passcode);

    Ok(StatusCode::CREATED)
}
