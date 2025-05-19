// TODO: register(), verify(), login(), logout()
// TODO: use an in-memory database for now, we'll switch to sqlx later

// NOTE: USEFUL EXTRACTORS
//       Path, Query<HashMap<String, String>>, OriginalUri, Json (consumes body), HeaderMap (all headers)

use axum::{
    extract::{Json, rejection::JsonRejection},
    http::StatusCode,
};
use serde_json::Value;

pub async fn register(payload: Result<Json<Value>, JsonRejection>) -> StatusCode {
    // here, Value should have the two entries email: and passcode:
    // passcode is expected to already be hashed by the frontend
    match payload {
        Ok(Json(payload)) => {
            // valid JSON payload -- main service logic goes here!
            StatusCode::CREATED
        }
        Err(JsonRejection::MissingJsonContentType(_)) => {
            // request didn't have `Content-Type: application/json` header
            StatusCode::BAD_REQUEST
        }
        Err(JsonRejection::JsonDataError(_)) => {
            // couldn't deserialize body into target type
            StatusCode::BAD_REQUEST
        }
        Err(JsonRejection::JsonSyntaxError(_)) => {
            // syntax error in the body
            StatusCode::BAD_REQUEST
        }
        Err(JsonRejection::BytesRejection(_)) => {
            // failed to extract the request body
            StatusCode::BAD_REQUEST
        }
        Err(_) => {
            // `JsonRejection` is non-exhaustive, so must include a catch-all
            StatusCode::BAD_REQUEST
        }
    }
}
