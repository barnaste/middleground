// TODO: for all crates, add detailed documentation comments wherever relevant

use axum::{Router, routing::any};
use shared::AppState;

mod error;
mod handler;
mod messages;
mod session;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/ws", any(handler::ws_handler))
        .with_state(state)
}
