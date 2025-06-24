use axum::{Router, routing::post};

mod dto;
mod handlers;
mod jwt;

pub mod models;
pub mod middleware;

/// Set up the authentication router using the given authenticator.
pub fn router<A>(authenticator: A) -> Router
where
    A: models::Authenticator,
{
    Router::new()
        .route("/send-otp", post(handlers::send_otp::<A>))
        .route("/verify-otp", post(handlers::verify_otp::<A>))
        .route("/logout", post(handlers::logout::<A>))
        .route("/refresh", post(handlers::refresh_token::<A>))
        .with_state(authenticator)
}
