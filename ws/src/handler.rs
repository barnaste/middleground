//! WebSocket connection handler.
//!
//! Provides the entry point for WebSocket connections, validates user access, and upgrades HTTP
//! connections to WebSocket protocol.

use axum::{
    extract::{Extension, Query, State, WebSocketUpgrade},
    response::Response,
};
use db::queries::conversations as query;
use serde::Deserialize;
use shared::AppState;
use uuid::Uuid;

use crate::error::WsError;
use crate::session::handle_socket;

/// Query parameters for WebSocket connection.
#[derive(Deserialize)]
pub struct WsQuery {
    pub conversation_id: Uuid,
}

/// WebSocket upgrade handler.
///
/// Entry point for WebSocket connections. Performs:
/// 1. Extracts conversation ID from query parameters
/// 2. Extracts user ID from request extensions
/// 3. Validates user has access to conversation
/// 4. Upgrades connection to WebSocket on success
/// 5. Hands off to session handler
///
/// # Arguments
/// * `ws` - WebSocket upgrade extractor
/// * `query` - Query parameters with conversation_id
/// * `state` - Application state (DB pool, Redis client)
/// * `user_id` - User UUID from auth middleware
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<AppState>,
    Extension(user_id): Extension<Uuid>,
) -> Result<Response, WsError> {
    // Verify user has access to this conversation before establishing WebSocket (fail fast)
    let has_access = query::user_has_access(&state.db_pool, query.conversation_id, user_id)
        .await
        .inspect_err(|e| {
            tracing::error!(
                error = %e,
                user_id = %user_id,
                conversation_id = %query.conversation_id,
                "Database error during access check"
            )
        })?;

    if !has_access {
        tracing::warn!(
            user_id = %user_id,
            conversation_id = %query.conversation_id,
            "Unauthorized access attempt"
        );
        return Err(WsError::Unauthorized);
    }

    // access granted; upgrade to websocket
    tracing::info!(
        user_id = %user_id,
        conversation_id = %query.conversation_id,
        "WebSocket connection established"
    );

    Ok(ws.on_upgrade(async move |socket| {
        if let Err(e) = handle_socket(socket, state, user_id, query.conversation_id).await {
            tracing::error!(
                error = %e,
                user_id = %user_id,
                conversation_id = %query.conversation_id,
                "WebSocket session error"
            );
        }
    }))
}
