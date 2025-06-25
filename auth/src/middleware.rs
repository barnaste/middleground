//! Authentication middleware for protecting routes.

use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

use crate::models::Authenticator;
use crate::{dto, jwt};

/// Standard authentication middleware that validates JWT tokens locally.
///
/// This middleware validates JWT tokens using HMAC signature verification
/// but does not check against the authentication backend. Use this for
/// most authentication needs where performance is important.
///
/// The validated user UUID is inserted into request extensions and can be
/// accessed in handlers using `axum::Extension`.
///
/// # Example
/// ```rust
/// use axum::{Router, routing::get, middleware};
/// use auth::middleware::auth_standard;
///
/// let app = Router::new()
///     .route("/protected", get(protected_handler))
///     .route_layer(middleware::from_fn(auth_standard));
/// ```
async fn auth_standard(
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<dto::ErrorResponse>)> {
    let token = jwt::extract_jwt_from_headers(request.headers()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(dto::ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let claims = jwt::validate_jwt_hmac(&token, "temp").map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(dto::ErrorResponse {
                error: e.to_string(),
            }),
        )
    })?;

    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(dto::ErrorResponse {
                error: format!("Invalid user ID in token: {}", e),
            }),
        )
    })?;

    request.extensions_mut().insert(user_id);
    Ok(next.run(request).await)
}

/// Strict authentication middleware that validates tokens against the backend.
///
/// This middleware validates JWT tokens and also checks with the authentication
/// backend to ensure the session the token refers to is still valid. Use this for
/// endpoints that require the highest level of security, though it comes with a
/// performance cost.
///
/// The validated user UUID is inserted into request extensions and can be
/// accessed in handlers using `axum::Extension`.
///
/// # Example
/// ```rust
/// use axum::{Router, routing::get, middleware};
/// use auth::{middleware::auth_strict, models::SbAuthenticator};
///
/// let authenticator = SbAuthenticator::default();
/// let app = Router::new()
///     .route("/admin", get(admin_handler))
///     .route_layer(middleware::from_fn_with_state(
///         authenticator.clone(),
///         auth_strict
///     ))
///     .with_state(authenticator);
/// ```
async fn auth_strict<A: Authenticator>(
    State(authenticator): State<A>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<dto::ErrorResponse>)> {
    let token = jwt::extract_jwt_from_headers(request.headers()).map_err(|e| {
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
                error: format!("Token verification failed: {}", e),
            }),
        )
    })?;

    request.extensions_mut().insert(user_id);
    Ok(next.run(request).await)
}

// TODO: introduce tests using tokio::test
