//! # Auth Crate
//!
//! A flexible authentication library for Axum web applications, providing JWT-based
//! authentication with with OTP (One-Time Password) support.
//!
//! ## Features
//!
//! - OTP-based authentication
//! - JWT token management (access & refresh tokens)
//! - Flexible authentication backends (Supabase included)
//! - Authentication middleware for route protection
//! - Type-safe error handling
//!
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use auth::{router, SbAuthenticator};
//! use axum::Router;
//!
//! #[tokio::main]
//! async fn main() {
//!     let authenticator = SbAuthenticator::default();
//!     let auth_router = router(authenticator);
//!
//!     let app = Router::new()
//!         .nest("/auth", auth_router);
//!
//!     // Start your server...
//! }
//! ```

use axum::{Router, routing::post};

mod dto;
mod handlers;
mod jwt;
mod error;

pub mod models;
pub mod middleware;

/// Creates an authentication router with the standard endpoints using the provided authenticator.
///
/// The router includes the following endpoints:
///  - `POST /send-otp` - send OTP to user via their contact information; defaults to email
///  - `POST /verify-otp` - verify OTP and retrieve access and refresh tokens
///  - `POST /logout` - invalidate associated session
///  - `POST /refresh` - refresh access token using refresh token
///
///  # Example
///
///  ```rust
///  use auth::{router, models::SbAuthenticator};
///
///  let authenticator = SbAuthenticator::default();
///  let auth_router = router(authenticator);
///  ```
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
