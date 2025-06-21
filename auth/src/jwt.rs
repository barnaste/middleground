use axum::http::HeaderMap;
use jsonwebtoken::{DecodingKey, Validation, decode, errors::Error as JwtError};
use serde::{Deserialize, Serialize};

// JWT access token base claims, used in JWT creation/decryption
#[derive(Deserialize, Serialize)]
pub struct Claims {
    pub sub: String, // subject, namely UUID of the user
    pub exp: usize,  // expiration time, as UTC timestamp
}

/// Extract JWT from Authorization header.
/// Header is expected to be in standard form:
///     Authorization: Bearer <token>
pub fn extract_jwt_from_headers(headers: &HeaderMap) -> Result<String, String> {
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

/// Verify that the JWT is not expired and has not
/// been tampered with.
pub fn validate_jwt_hmac(token: &str, secret: &str) -> Result<Claims, JwtError> {
    let key = DecodingKey::from_secret(secret.as_ref());
    // decode will result in an error if the token or signature is invalid,
    // the token has invalid base64, or validation of a reserved claim fails
    let token = decode::<Claims>(token, &key, &Validation::default())?;
    Ok(token.claims)
}

// -----------------
//       TESTS
// -----------------

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
            &Claims {
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

        let tampered_token = format!(
            "{}.{}.{}",
            parts[0],
            BASE64_STANDARD_NO_PAD.encode(payload),
            parts[2]
        );

        let result = validate_jwt_hmac(&tampered_token, secret);
        assert!(result.is_err());
    }
}
