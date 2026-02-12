//! # ws - WebSocket Service for Real-Time Messaging
//!
//! A WebSocket service built on axum and redis pub/sub for real-time messaging in conversation
//! channels. This crate provides complete WebSocket infrastructure, with:
//! - Full-duplex communication
//! - Database-first message persistence
//! - Redis pub/sub broadcasting
//! - Type-safe message and error handling
//!
//! ## Architecture
//!
//! ```text
//! Client ↔ WS Handler ↔ Redis Pub/Sub ↔ WS Handler ↔ Other Client
//!              ↓
//!          PestgreSQL
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ws::router;
//! use shared::AppState;
//! use axum::Router;
//!
//! #[tokio::main]
//! async fn main() {
//!     let state = AppState {
//!         db_pool: db::create_pool().await.unwrap(),
//!         redis: redis::Client::open("redis://localhost:6379").unwrap(),
//!     };
//!
//!     let app = Router::new()
//!         .merge(router(state))
//!         .layer(/* insert your user-ID-injecting middleware here... */);
//!
//!     // Start server...
//! }
//! ```
//!
//! ## Authentication
//!
//! Requires authentication middleware to inject `user_id` into request extension. You may use the
//! `auth` crate's middleware for this purpose.
//!
//! ## Redis Protocol
//!
//! Uses RESP3 for push-based pub/sub notifications, providing lower latency than RESP2's polling
//! model.

use axum::{Router, routing::any};
use shared::AppState;

mod error;
mod handler;
mod messages;
mod session;

pub use error::{WsError, WsResult};

/// Create the WebSocket router.
///
/// Returns an Axum router with a single route at `/ws` that handles WebSocket upgrade requests.
///
/// # Requirements
///
/// - `conversation_id` query parameter (UUID)
/// - `user_id` in request extensions (from auth middleware)
///
/// **Must** be wrapped with middleware that injects the user's UUID into request extensions. This
/// is satisfied by the `auth` crate middleware.
///
/// # Example
///
/// ```rust,no_run
/// use ws::router as ws_router;
/// use axum::middleware;
///
/// use shared::Appstate;
/// let state = AppState {
///     db_pool: todo!(),
///     redis: todo!(),
/// };
/// let authenticator = todo!();
/// let auth_function = todo!();
///
/// let app = Router::new()
///     .merge(ws_router(state))
///     .route_layer(middleware::from_fn_with_state(
///         authenticator,
///         auth_function
///     ));
/// ```
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/ws", any(handler::ws_handler))
        .with_state(state)
}
