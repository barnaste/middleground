//! JWT utilities for token extraction and validation.

use axum::http::HeaderMap;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};

use crate::error::AuthError;

/// JWT claims structure for access tokens.
#[derive(Deserialize, Serialize)]
pub struct Claims {
    /// Subject (user UUID)
    pub sub: String,
    /// Expiration time as Unix epoch timestamp
    pub exp: usize,
}

/// Extract JWT from Authorization header.
///
/// Expects the header to be in the format: `Authorization: Bearer <token>`
pub fn extract_jwt_from_headers(headers: &HeaderMap) -> Result<String, AuthError> {
    let auth_header = headers
        .get("authorization")
        .ok_or(AuthError::MissingAuthHeader)?;

    let auth_value = auth_header
        .to_str()
        .map_err(|_| AuthError::InvalidAuthHeader)?;

    let token = auth_value
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidAuthHeader)?;

    Ok(token.to_string())
}

/// Verify JWT using HMAC signature verification.
///
/// Verifies the token signature and expiration time.
pub fn validate_jwt_hmac(token: &str, secret: &str) -> Result<Claims, AuthError> {
    let key = DecodingKey::from_secret(secret.as_ref());

    // decode will result in an error if the token or signature is invalid,
    // the token has invalid base64, or validation of a reserved claim fails
    let token = decode::<Claims>(token, &key, &Validation::default())
        .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
    Ok(token.claims)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use base64::{Engine, prelude::BASE64_STANDARD_NO_PAD};
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode, errors::Error as JwtError};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    /// Create an HS256 encrypted test JWT using the given secret, with specified
    /// expiration offset `exp` in seconds, and UUID `sub` set to "user."
    fn create_test_jwt(secret: &str, offset: i32) -> Result<String, JwtError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;

        encode(
            &Header::new(Algorithm::HS256),
            &Claims {
                sub: "user".to_string(),
                exp: (now + offset) as usize,
            },
            &EncodingKey::from_secret(secret.as_ref()),
        )
    }

    #[test]
    fn test_extract_jwt_from_headers_success() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer test-token").unwrap(),
        );

        let result = extract_jwt_from_headers(&headers);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test-token");
    }

    #[test]
    fn test_extract_jwt_from_headers_missing() {
        let headers = HeaderMap::new();

        let result = extract_jwt_from_headers(&headers);
        assert!(matches!(result, Err(AuthError::MissingAuthHeader)));
    }

    #[test]
    fn test_extract_jwt_from_headers_invalid_format() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_str("Token test-token").unwrap(),
        );

        let result = extract_jwt_from_headers(&headers);
        assert!(matches!(result, Err(AuthError::InvalidAuthHeader)));
    }

    #[test]
    fn test_validate_jwt_hmac_valid() {
        let secret = "test-secret";
        let token = create_test_jwt(secret, 3600).unwrap();

        let result = validate_jwt_hmac(&token, secret);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().sub, "user");
    }

    #[test]
    fn test_validate_jwt_hmac_expired() {
        let secret = "test-secret";
        let token = create_test_jwt(secret, -3600).unwrap();

        let result = validate_jwt_hmac(&token, secret);
        assert!(matches!(result, Err(AuthError::InvalidToken(_))));
    }

    #[test]
    fn test_tampered_jwt_invalid() {
        let secret = "test-secret";
        let token = create_test_jwt(secret, 3600).unwrap();

        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 3); // the JWT should have three components

        // decode the body and convert to a JSON Value
        let mut payload = BASE64_STANDARD_NO_PAD.decode(parts[1]).unwrap();
        let mut payload_json: serde_json::Value = serde_json::from_slice(&payload).unwrap();

        // tamper with the payload and convert back to token
        payload_json["sub"] = serde_json::Value::String("admin".to_string());
        payload = serde_json::to_vec(&payload_json).unwrap();

        let tampered_token = format!(
            "{}.{}.{}",
            parts[0],
            BASE64_STANDARD_NO_PAD.encode(payload),
            parts[2]
        );

        let result = validate_jwt_hmac(&tampered_token, secret);
        assert!(matches!(result, Err(AuthError::InvalidToken(_))));
    }

    #[test]
    fn test_validate_jwt_hmac_wrong_secret() {
        let token = create_test_jwt("correct-secret", 3600).unwrap();
        let result = validate_jwt_hmac(&token, "wrong-secret");
        assert!(matches!(result, Err(AuthError::InvalidToken(_))));
    }
}
