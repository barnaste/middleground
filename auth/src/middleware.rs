// TODO:
//  1. require_auth_standard
//  2. require_auth_strict

// use axum::{extract::Request, http::StatusCode, middleware::Next, response::IntoResponse};
// use serde::{Deserialize, Serialize};
// use tower_sessions::{Expiry, Session, SessionManagerLayer, SessionStore, cookie::time::Duration};
//
// // no authentication : a user account is not attached to the client
// // pending authentication : an unverified user account is attached to the client
// // authentication : a verified user account is attached to the client
// #[derive(Serialize, Deserialize)]
// pub enum AuthStatus {
//     Unauthenticated,
//     Pending(i32),
//     Authenticated,
// }
//
// pub fn create_session_layer<S: SessionStore + Clone>(store: S) -> SessionManagerLayer<S> {
//     // session ID cookies will expire after one week
//     let expiry = Expiry::OnInactivity(Duration::weeks(1));
//
//     SessionManagerLayer::new(store.clone())
//         .with_name("sessionId")
//         .with_expiry(expiry)
//         .with_always_save(true)
//         .with_http_only(true)
//         .with_secure(false) // TODO: set to true once https is enabled
// }
//
// pub async fn require_pending(
//     session: Session,
//     req: Request,
//     next: Next,
// ) -> Result<impl IntoResponse, StatusCode> {
//     match session.get::<AuthStatus>("auth_status").await {
//         Ok(Some(AuthStatus::Pending(_))) => Ok(next.run(req).await),
//         // if user is recognized but has wrong auth status, status is FORBIDDEN
//         Ok(Some(_)) => Err(StatusCode::FORBIDDEN),
//         _ => Err(StatusCode::UNAUTHORIZED),
//     }
// }
//
// pub async fn require_auth(
//     session: Session,
//     req: Request,
//     next: Next,
// ) -> Result<impl IntoResponse, StatusCode> {
//     match session.get::<AuthStatus>("auth_status").await {
//         Ok(Some(AuthStatus::Authenticated)) => Ok(next.run(req).await),
//         // if user is recognized but has wrong auth status, status is FORBIDDEN
//         Ok(Some(_)) => Err(StatusCode::FORBIDDEN),
//         _ => Err(StatusCode::UNAUTHORIZED),
//     }
// }
