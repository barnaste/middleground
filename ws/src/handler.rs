use axum::{
    extract::{Extension, Query, State, WebSocketUpgrade},
    response::Response,
};
use serde::Deserialize;
use shared::AppState;
use uuid::Uuid;

use crate::error::WsError;
use crate::session::handle_socket;

#[derive(Deserialize)]
pub struct WsQuery {
    pub conversation_id: Uuid,
}

/// WebSocket upgrade handler
///
/// Expects the ID of the conversation to connect to as a query parameter,
/// and the sender's user ID from middleware, as a request extension.
// TODO: fetch user id
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Response, WsError> {
    // Verify user has access to this conversation before establishing WebSocket -- fail fast
    // TODO: complete this verification

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user_id, query.conversation_id)))
}
