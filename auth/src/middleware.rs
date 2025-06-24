use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;

use crate::models::Authenticator;
use crate::{dto, jwt};

// -----------------
//      HELPERS
// -----------------

fn verify_auth_standard(token: String) -> Result<uuid::Uuid, String> {
    match jwt::validate_jwt_hmac(token.as_str(), "temp") {
        Ok(c) => uuid::Uuid::parse_str(c.sub.as_str())
            .map_err(|e| format!("Token contains invalid uuid: {}", e)),
        Err(e) => Err(format!("Token not valid: {}", e)),
    }
}

async fn verify_auth_strict<A: Authenticator>(
    authenticator: A,
    token: String,
) -> Result<uuid::Uuid, String> {
    authenticator
        .verify_token(token.as_str())
        .await
        .map_err(|e| format!("Token not valid: {}", e))
}

/// Generic function that extracts an access token from a request's headers,
/// and verifies that it is valid using custom logic specified in verify_fn.
async fn extract_and_verify<E, F>(
    request: &Request,
    verify_fn: F,
) -> Result<uuid::Uuid, (StatusCode, Json<dto::ErrorResponse>)> 
where
    F: AsyncFnOnce(String) -> Result<uuid::Uuid, E>,
    E: ToString,
{
    let token = jwt::extract_jwt_from_headers(request.headers())
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(dto::ErrorResponse { error: e })))?;

    let result = verify_fn(token).await
            .map_err(|e| (StatusCode::UNAUTHORIZED, Json(dto::ErrorResponse { error: e.to_string() })))?;

    Ok(result)
}

// -----------------
//     MIDDLEWARE
// -----------------

// note:
// 2. handle the same AuthClient between router and middleware -- only auth_strict will need the
//    authenticator, as auth_standard does not talk to the database

async fn auth_standard(
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<dto::ErrorResponse>)> {
    let id = extract_and_verify(&request, |s: String| async move {
        verify_auth_standard(s)
    }).await?;

    request.extensions_mut().insert(id);
    Ok(next.run(request).await)
}

async fn auth_strict<A: Authenticator>(
    State(state): State<A>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<dto::ErrorResponse>)> {
    let id = extract_and_verify(&request, |s: String| async move {
        verify_auth_strict(state, s).await
    }).await?;

    request.extensions_mut().insert(id);
    Ok(next.run(request).await)
}
