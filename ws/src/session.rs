//! WebSocket session management.
//!
//! Manages individual WebSocket sessions, coordinating message flow between client, database, and
//! Redis pub/sub. Each session runs two concurrent tasks.

use axum::extract::ws::{Message, WebSocket};
use futures::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use shared::AppState;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    WsResult,
    messages::{IncomingMessage, publish_msg},
};

/// Handle WebSocket session for a user in a conversation.
///
/// Orchestrates the entire session lifecycle:
/// 1. Splits socket into read/write blocks
/// 2. Creates async Redis connection with push notifications
/// 3. Subscribes to conversation's Redis channel `conversation:{conversation_id}`
/// 4. Spawns concurrent tasks:
///     - Reader: Client -> Database -> Redis
///     - Writer: Redis -> Client
///
/// On disconnect, the read task completes and the write task is aborted. Resources are
/// automatically cleaned up.
///
/// # Arguments
/// * `socket` - Upgraded WebSocket connection
/// * `state` - Application state (DB pool, Redis client)
/// * `user_id` - UUID of authenticated user
/// * `conversation_id` - UUID of conversation
///
/// # Returns
/// * `Ok(())` - Session completed normally
/// * `Err(WsError)` - Session failed
pub async fn handle_socket(
    socket: WebSocket,
    state: AppState,
    user_id: Uuid,
    conversation_id: Uuid,
) -> WsResult<()> {
    tracing::debug!(
        user_id = %user_id,
            conversation_id = %conversation_id,
            "Starting WebSocket connection"
    );

    let (sender, receiver) = socket.split();

    // set up a receiver rx that takes in all updates in the redis channel corresponding to the
    // conversation the user is connecting to
    let (tx, rx) = mpsc::unbounded_channel();
    let config = redis::AsyncConnectionConfig::new().set_push_sender(tx);

    // create async redis connection
    let mut conn = state
        .redis
        .clone()
        .get_multiplexed_async_connection_with_config(&config)
        .await
        .inspect_err(|e| {
            tracing::error!(
                error = %e,
                user_id = %user_id,
                conversation_id = %conversation_id,
                "Failed to create Redis connection"
            );
        })?;

    // subscribe to conversation channel
    let channel_name = format!("conversation:{}", conversation_id);
    conn.subscribe(&channel_name).await.inspect_err(|e| {
        tracing::error!(
            error = %e,
            user_id = %user_id,
            conversation_id = %conversation_id,
            "Failed to subscribe to Redis"
        );
    })?;

    tracing::info!(
        user_id = %user_id,
        conversation_id = %conversation_id,
        channel = %channel_name,
        "Subscribed to conversation"
    );

    let write_task = tokio::spawn(socket_write(sender, rx, user_id, conversation_id));
    let read_result = socket_read(receiver, state.clone(), user_id, conversation_id).await;

    match &read_result {
        Ok(()) => {
            tracing::info!(
                user_id = %user_id,
                conversation_id = %conversation_id,
                "Client disconnected normally"
            );
        }
        Err(e) => {
            tracing::error!(
                error = %e,
                user_id = %user_id,
                conversation_id = %conversation_id,
                "WebSocket read task failed"
            );
        }
    }

    // once the read task has terminated, abort the write task
    write_task.abort();

    tracing::debug!(
        user_id = %user_id,
        conversation_id = %conversation_id,
        "Session terminated"
    );

    Ok(())
}

/// Process incoming messages from client.
///
/// Runs in loop, reading from WebSocket until disconnect or error.
///
/// For each message:
/// 1. Parse JSON to IncomingMessage
/// 2. Process (validate, write to DB)
/// 3. Publish to Redis
/// 4. Continue
///
/// Individual message errors are logged but do not terminate the session. This allows the
/// connection to survive transient failures.
///
/// # Termination
///
/// The loop ends when:
/// - Client sends Close frame
/// - WebSocket protocol error
/// - Stream ends unexpectedly
async fn socket_read(
    mut receiver: SplitStream<WebSocket>,
    state: AppState,
    user_id: Uuid,
    conversation_id: Uuid,
) -> WsResult<()> {
    while let Some(msg) = receiver.next().await {
        let msg = msg.inspect_err(|e| {
            tracing::error!(
                error = %e,
                user_id = %user_id,
                conversation_id = %conversation_id,
                "WebSocket protocol error"
            );
        })?;

        match msg {
            Message::Text(text) => {
                // parse incoming message
                let incoming: IncomingMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        tracing::warn!(
                            error = %e,
                            user_id = %user_id,
                            conversation_id = %conversation_id,
                            text = %text,
                            "Failed to parse message"
                        );
                        continue;
                    }
                };

                tracing::debug!(
                    user_id = %user_id,
                    conversation_id = %conversation_id,
                    message_type = ?std::mem::discriminant(&incoming),
                    "Received from client"
                );

                // process message
                let outgoing = match incoming
                    .handle(user_id, conversation_id, &state.db_pool)
                    .await
                {
                    Ok(out) => out,
                    Err(e) => {
                        tracing::error!(
                            error = %e,
                            user_id = %user_id,
                            conversation_id = %conversation_id,
                            "Failed to process message"
                        );
                        continue;
                    }
                };

                // publish to Redis
                publish_msg(conversation_id, outgoing, &state.redis)
                    .await
                    .inspect_err(|e| {
                        tracing::error!(
                            error = %e,
                            user_id = %user_id,
                            conversation_id = %conversation_id,
                            "Failed to publish to Redis"
                        );
                    })
                    .ok();
            }

            Message::Close(close) => {
                if let Some(frame) = close {
                    tracing::info!(
                        user_id = %user_id,
                        conversation_id = %conversation_id,
                        code = frame.code,
                        reason = %frame.reason,
                        "Client sent close frame"
                    );
                } else {
                    tracing::info!(
                        user_id = %user_id,
                        conversation_id = %conversation_id,
                        "Client sent close frame"
                    );
                }
                break;
            }

            Message::Ping(_) | Message::Pong(_) => {
                // automatically handled by library
                tracing::trace!(
                    user_id = %user_id,
                    conversation_id = %conversation_id,
                    "Received ping/pong"
                );
            }

            Message::Binary(_) => {
                tracing::warn!(
                    user_id = %user_id,
                    conversation_id = %conversation_id,
                    "Received unsupported binary message"
                );
            }
        }
    }

    Ok(())
}

/// Forward messages from Redis to client.
///
/// Runs in loop, reading push notifications from Redis subscription and forwarding to WebSocket
/// client.
///
/// For each notification:
/// 1. Extract message payload
/// 2. Forward JSON to client (already serialized)
///
/// Individual message errors are logged but do not terminate the session. This allows the
/// connection to survive transient failures.
///
/// # Termination
///
/// The loop ends when:
/// - Channel receiver closes (i.e. the read task completes)
/// - Task aborted by session handler
async fn socket_write(
    mut sender: SplitSink<WebSocket, Message>,
    mut rx: mpsc::UnboundedReceiver<redis::PushInfo>,
    user_id: Uuid,
    conversation_id: Uuid,
) {
    while let Some(push_info) = rx.recv().await {
        // we only handle push information that encodes a message;
        // that data should already be serialized, to we just forward it
        let redis_msg = match redis::Msg::from_push_info(push_info) {
            Some(msg) => msg,
            None => {
                tracing::trace!(
                    user_id = %user_id,
                    conversation_id = %conversation_id,
                    "Received non-message push info"
                );
                continue;
            }
        };

        // extract payload
        let payload: redis::RedisResult<String> = redis_msg.get_payload();

        match payload {
            Ok(msg_json) => {
                if let Err(e) = sender.send(Message::Text(msg_json.into())).await {
                    tracing::error!(
                        error = %e,
                        user_id = %user_id,
                        conversation_id = %conversation_id,
                        "Failed to send to client"
                    );
                    continue;
                }

                tracing::trace!(
                    user_id = %user_id,
                    conversation_id = %conversation_id,
                    "Forwarded to client"
                );
            }
            Err(e) => {
                tracing::error!(
                    error = %e,
                    user_id = %user_id,
                    conversation_id = %conversation_id,
                    "Failed to extract payload"
                );
            }
        };
    }

    tracing::debug!(
        user_id = %user_id,
        conversation_id = %conversation_id,
        "Write task terminated"
    );
}
