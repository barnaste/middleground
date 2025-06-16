// NOTE: USEFUL EXTRACTORS
//       Path, Query<HashMap<String, String>>

use crate::models;
use axum::{
    extract::{Json, rejection::JsonRejection},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use jsonwebtoken::{DecodingKey, Validation, decode, errors::Error as JwtError};
use supabase_auth::models::{AuthClient, Session};

// TODO:
//  1. send_otp
//  2. verify_otp
//  3. logout
//  4. refresh_token

/// Extract JWT from Authorization header.
/// Header is expected to be in standard form:
///     Authorization: Bearer <token>
fn extract_jwt_from_headers(headers: &HeaderMap) -> Result<String, String> {
    let auth_header = Some(
        headers
            .get("authorization")
            .ok_or("Missing authorization header")?,
    );

    let jwt = auth_header
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or("Invalid authorization header")?
        .to_string();

    Ok(jwt)
}

/// Convert a Session to a returnable AuthResponse.
/// This will consume the Session.
fn session_to_auth_response(session: Session) -> models::AuthResponse {
    models::AuthResponse {
        access_token: session.access_token,
        refresh_token: session.refresh_token,
        expires_at: session.expires_at,
    }
}

/// Verify that the JWT is not expired and has not
/// been tampered with.
fn validate_jwt_hmac(token: &str, secret: &str) -> Result<models::Claims, JwtError> {
    let key = DecodingKey::from_secret(secret.as_ref());
    // decode will result in an error if the token or signature is invalid,
    // the token has invalid base64, or validation of a reserved claim fails
    let token = decode::<models::Claims>(&token, &key, &Validation::default())?;
    Ok(token.claims)
}

#[cfg(test)]
mod tests {
    use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    /// Create an HS256 encrypted demo JWT using the given secret.
    /// Use only for testing. The JWT has claims:
    ///     sub: "user"
    ///     exp: `offset` seconds after present
    fn create_secure_jwt(secret: &str, offset: i32) -> Result<String, JwtError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;

        encode(
            &Header::new(Algorithm::HS256),
            &models::Claims {
                sub: "user".to_string(),
                exp: (now + offset) as usize,
            },
            &EncodingKey::from_secret(secret.as_ref()),
        )
    }

    #[test]
    // test that valid JWTs are recognized by validate_jwt_hmac
    fn test_jwt_valid() {
        let secret = "secret";
        let token = create_secure_jwt(secret, 3600).unwrap();

        let result = validate_jwt_hmac(&token, secret);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().sub, "user");
    }

    #[test]
    // test that expired JWTs are recognized by validate_jwt_hmac
    fn test_expired_jwt_invalid() {
        let secret = "secret";
        let token = create_secure_jwt(secret, -3600).unwrap();

        let result = validate_jwt_hmac(&token, secret);
        assert!(result.is_err());
    }

    #[test]
    // test that tampered with JWTs are recognized by validate_jwt_hmac
    fn test_tampered_jwt_invalid() {
        let secret = "secret";
        let token = create_secure_jwt(secret, 3600).unwrap();

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3); // the JWT should have three components

        // decode the body and convert to a JSON Value
        let mut payload = BASE64_STANDARD_NO_PAD.decode(parts[1]).unwrap();
        let mut payload_json: serde_json::Value = serde_json::from_slice(&payload).unwrap();

        // modify a field within the Value and convert back to token
        payload_json["sub"] = serde_json::Value::String("admin".to_string());
        payload = serde_json::to_vec(&payload_json).unwrap();

        let tampered_token = format!("{}.{}.{}",
            parts[0],
            BASE64_STANDARD_NO_PAD.encode(payload),
            parts[2]
        );

        let result = validate_jwt_hmac(&tampered_token, secret);
        assert!(result.is_err());
    }
}

//
// // note that passcode is expected to already be hashed by the frontend (maybe use argon2?)
// pub async fn register(
//     session: Session,
//     payload: Result<Json<Value>, JsonRejection>,
// ) -> Result<StatusCode, impl IntoResponse> {
//     // if this fails, the JSON data given does not contain an object
//     let Json(payload) = payload.map_err(|_| StatusCode::BAD_REQUEST)?;
//
//     // extract JSON object from the payload
//     let map = payload.as_object().ok_or(StatusCode::BAD_REQUEST)?;
//
//     // extract email and passcode
//     let email = map
//         .get(EMAIL_SESSION_KEY)
//         .and_then(|s| s.as_str())
//         .ok_or_else(|| StatusCode::BAD_REQUEST)?;
//     let passcode = map
//         .get(PASSCODE_SESSION_KEY)
//         .and_then(|s| s.as_str())
//         .ok_or_else(|| StatusCode::BAD_REQUEST)?;
//
//     // ensure email is of correct format
//     // if !email.validate_email() {
//     //     return Err(StatusCode::BAD_REQUEST);
//     // }
//
//     // check that email is not already in use; return 409 Conflict if it does
//
//     // insert new user record (later: need database)
//     let user_id = 142857; // temporary -- should be safely generated
//
//     // initialize default user profile (later: need to design preferences interface)
//     // generate verification code
//     let verification_code = 123456; // temporary -- should be safely generated
//
//     // store user_id and authentication details in session
//     session
//         .insert("user_id", user_id)
//         .await
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//     session
//         .insert("auth_status", AuthStatus::Pending(verification_code))
//         .await
//         .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
//
//     // request email verification via notifs (later: need notifications interface)
//
//     Ok(StatusCode::CREATED)
// }
