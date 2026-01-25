// TODO: for all crates, add detailed documentation comments wherever relevant

use axum::{Router, routing::any};
use shared::AppState;

mod error;
mod handler;
mod messages;
mod session;

/// Create the WebSocket router.
/// Expects authentication so that the client's UUID is sent as a request extension.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/ws", any(handler::ws_handler))
        .with_state(state)
}
