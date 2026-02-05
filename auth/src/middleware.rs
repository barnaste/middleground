//! Authentication middleware for protecting routes.

use axum::Json;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use crate::dto;
use crate::error::AuthError;
use crate::models::Authenticator;

/// Extract JWT from Authorization header.
///
/// Expects the header to be in the format: `Authorization: Bearer <token>`
pub(crate) fn extract_jwt_from_headers(headers: &HeaderMap) -> Result<String, AuthError> {
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

/// Standard authentication middleware that validates JWT tokens locally.
///
/// This middleware validates JWT tokens using the authenticator's verification method.
/// For JWKS-based authenticators (e.g. SbAuthenticator), this uses RSA or EC
/// signature verification with public keys fetched from the JWKS endpoint.
///
/// For HMAC-based authenticators, this uses shared secret verification.
///
/// The validated user UUID is inserted into request extensions and can be accessed
/// in handlers using `axum::Extension`.
///
/// # Performance (JWKS-based)
///
/// - First request: Fetches JWKS (~50-100ms)
/// - Subsequent requests: Uses cached keys (~1-2ms)
/// - Key rotation: Automatic refresh when unknown kid is encountered
///
/// # Example
///
/// ```rust,ignore
/// use axum::{Router, routing::get, middleware};
/// use auth::{middleware::auth_standard, models::SbAuthenticator};
///
/// let authenticator = SbAuthenticator::default();
/// let app = Router::new()
///     .route("/protected", get(protected_handler))
///     .route_layer(middleware::from_fn_with_state(
///         authenticator.clone(),
///         auth_standard
///     ));
/// ```
pub async fn auth_standard<A: Authenticator>(
    State(authenticator): State<A>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<dto::ErrorResponse>)> {
    let token = extract_jwt_from_headers(request.headers()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(dto::ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let user_id = authenticator.verify_token(&token).await.map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(dto::ErrorResponse {
                error: format!("Token validation failed: {}", e),
            }),
        )
    })?;

    request.extensions_mut().insert(user_id);
    Ok(next.run(request).await)
}

/// Strict authentication middleware that validates tokens with additional verification.
///
/// This middleware validates JWT tokens and using the authenticator's verify_token_strict,
/// which may include additional checks beyond signature verification. For SbAuthenticator,
/// this means JWKS verification with checking against Supabase's API, ensuring both
/// cryptographic validity and session existence.
///
/// Use this for endpoints that require the highest level of security assurance, such as
/// administrative operations or sensitive data modifications.
///
/// The validated user UUID is inserted into request extensions and can be accessed
/// in handlers using `axum::Extension`.
///
/// # Performance Impact
///
/// This middleware may make additional verification calls depending on the authenticator
/// implementation, which can add latency. Use sparingly for critical operations only.
///
/// # Example
///
/// ```rust,ignore
/// use axum::{Router, routing::get, middleware};
/// use auth::{middleware::auth_strict, models::SbAuthenticator};
///
/// let authenticator = SbAuthenticator::default();
/// let app = Router::new()
///     .route("/admin", get(admin_handler))
///     .route_layer(middleware::from_fn_with_state(
///         authenticator.clone(),
///         auth_strict
///     ));
/// ```
pub async fn auth_strict<A: Authenticator>(
    State(authenticator): State<A>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<dto::ErrorResponse>)> {
    let token = extract_jwt_from_headers(request.headers()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(dto::ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let user_id = authenticator.verify_token_strict(&token).await.map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(dto::ErrorResponse {
                error: format!("Token validation failed: {}", e),
            }),
        )
    })?;

    request.extensions_mut().insert(user_id);
    Ok(next.run(request).await)
}
