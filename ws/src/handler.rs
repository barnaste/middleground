use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::Response,
};
use serde::Deserialize;
use shared::AppState;
use uuid::Uuid;

use crate::error;

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
) -> Result<Response, error::WsError> {
    todo!()
}
