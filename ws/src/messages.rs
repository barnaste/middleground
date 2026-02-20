//! Message types and processing logic.
//!
//! Defines message types for WebSocket communication, and implements processing logic. Messages
//! are validated, persisted to database, and then broadcast via Redis.

use chrono::{DateTime, Utc};
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use db::queries::messages as msg_query;

use crate::WsResult;

// ========================== Requests ==========================

/// Messages received from WebSocket clients.
/// Implemented as a tagged union with camelCase serialization.
///
/// # JSON format
///
/// ```json
/// {
///     "type": "send",
///     "payload": { "content": "Hello!", "quotedId": null }
/// }
/// ```
///
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum IncomingMessage {
    /// Create a new message
    Send { payload: SendPayload },

    /// Edit an existing message
    Edit { payload: EditPayload },

    /// Delete a message (soft delete)
    Delete { payload: DeletePayload },
}

/// Payload for sending a new message.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPayload {
    pub content: String,
    pub quoted_id: Option<Uuid>,
}

/// Payload for editing a message.
///
/// Users can only edit their own messages (enforced by database).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditPayload {
    pub message_id: Uuid,
    pub content: String,
}

/// Payload for deleting a message.
///
/// Performs a soft delete.
/// Users can only delete their own messages (enforced by database).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePayload {
    pub message_id: Uuid,
}

// ========================== Responses ==========================

/// Messages broadcast to WebSocket clients.
///
/// All subscribed clients receive these via Redis pub/sub.
#[derive(Debug, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum OutgoingMessage {
    /// Broadcast newly created message.
    Send {
        message_id: Uuid,
        sender_id: Uuid,
        content: String,
        quoted_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
    },

    /// Broadcast message edit.
    Edit {
        message_id: Uuid,
        sender_id: Uuid,
        content: String,
        timestamp: DateTime<Utc>,
    },

    /// Broadcast message deletion.
    Delete {
        message_id: Uuid,
        sender_id: Uuid,
        timestamp: DateTime<Utc>,
    },
}

// ========================== Message Processing ==========================

impl IncomingMessage {
    /// Process incoming message from client.
    ///
    /// Validates, persists to database, constructs outgoing message for broadcast.
    /// Note that messages are persisted before broadcasting. If the database write fails, the
    /// message is never broadcast.
    ///
    /// # Arguments
    /// * `user_id` - UUID of sender
    /// * `conversation_id` - UUID of conversation
    /// * `db` - Database connection pool
    ///
    /// # Returns
    /// * `Ok(OutgoingMessage)` - Ready for broadcast
    /// * `Err(WsError)` - Processing failed
    pub async fn handle(
        self,
        user_id: Uuid,
        conversation_id: Uuid,
        db: &PgPool,
    ) -> WsResult<OutgoingMessage> {
        match self {
            Self::Send { payload } => handle_send(user_id, conversation_id, payload, db).await,
            Self::Edit { payload } => handle_edit(user_id, conversation_id, payload, db).await,
            Self::Delete { payload } => handle_delete(user_id, conversation_id, payload, db).await,
        }
    }
}

// ========================== Handlers ==========================

/// Handle new message send.
async fn handle_send(
    user_id: Uuid,
    conversation_id: Uuid,
    payload: SendPayload,
    db: &PgPool,
) -> WsResult<OutgoingMessage> {
    let now = Utc::now();

    tracing::debug!(
        user_id = %user_id,
        conversation_id = %conversation_id,
        quoted_id = ?payload.quoted_id,
        "Processing send message"
    );

    let message_id = msg_query::create_message(
        db,
        msg_query::CreateMessageParams {
            conversation_id,
            sender_id: user_id,
            quoted_id: payload.quoted_id,
            created_at: now,
            content: payload.content.clone(),
        },
    )
    .await?;

    tracing::info!(
        user_id = %user_id,
        conversation_id = %conversation_id,
        message_id = %message_id,
        "Message created"
    );

    Ok(OutgoingMessage::Send {
        message_id,
        sender_id: user_id,
        content: payload.content,
        quoted_id: payload.quoted_id,
        timestamp: now,
    })
}

/// Handle message edit.
async fn handle_edit(
    user_id: Uuid,
    conversation_id: Uuid,
    payload: EditPayload,
    db: &PgPool,
) -> WsResult<OutgoingMessage> {
    let now = Utc::now();

    tracing::debug!(
        user_id = %user_id,
        conversation_id = %conversation_id,
        message_id = %payload.message_id,
        "Processing edit message"
    );

    msg_query::edit_message(
        db,
        msg_query::EditMessageParams {
            conversation_id,
            sender_id: user_id,
            message_id: payload.message_id,
            created_at: now,
            content: payload.content.clone(),
        },
    )
    .await?;

    tracing::info!(
        user_id = %user_id,
        conversation_id = %conversation_id,
        message_id = %payload.message_id,
        "Message edited"
    );

    Ok(OutgoingMessage::Edit {
        message_id: payload.message_id,
        sender_id: user_id,
        content: payload.content,
        timestamp: now,
    })
}

/// Handle message deletion.
async fn handle_delete(
    user_id: Uuid,
    conversation_id: Uuid,
    payload: DeletePayload,
    db: &PgPool,
) -> WsResult<OutgoingMessage> {
    let now = Utc::now();

    tracing::debug!(
        user_id = %user_id,
        conversation_id = %conversation_id,
        message_id = %payload.message_id,
        "Processing delete message"
    );

    msg_query::soft_delete_message(
        db,
        msg_query::DeleteMessageParams {
            conversation_id,
            sender_id: user_id,
            message_id: payload.message_id,
        },
    )
    .await?;

    tracing::info!(
        user_id = %user_id,
        conversation_id = %conversation_id,
        message_id = %payload.message_id,
        "Message deleted"
    );

    Ok(OutgoingMessage::Delete {
        message_id: payload.message_id,
        sender_id: user_id,
        timestamp: now,
    })
}

// ========================== Broadcasting ==========================

/// Publish message to conversation's Redis channel `conversation:{conversation_id}`.
/// Serializes to JSON and publishes. All subscribed clients receive.
///
/// # Arguments
/// * `conversation_id` - UUID of conversation
/// * `message` - Outgoing message to broadcast
/// * `redis` - Redis client
///
/// # Returns
/// * `Ok(())` - Publishes successfully
/// * `Err(WsError)` - Redis or JSON error
pub async fn publish_msg(
    conversation_id: Uuid,
    message: OutgoingMessage,
    redis: &RedisClient,
) -> WsResult<()> {
    let mut conn = redis
        .get_multiplexed_async_connection()
        .await
        .inspect_err(|e| {
            tracing::error!(
                error = %e,
                conversation_id = %conversation_id,
                "Failed to create Redis connection"
            );
        })?;

    let channel = format!("conversation:{}", conversation_id);
    let payload = serde_json::to_string(&message).inspect_err(|e| {
        tracing::error!(
            error = %e,
            conversation_id = %conversation_id,
            "Failed to serialize message"
        );
    })?;

    conn.publish::<_, _, ()>(channel.clone(), payload)
        .await
        .inspect_err(|e| {
            tracing::error!(
                error = %e,
                channel = %channel,
                "Failed to publish to Redis"
            );
        })?;

    tracing::debug!(
        conversation_id = %conversation_id,
        message_type = ?std::mem::discriminant(&message),
        "Message published"
    );

    Ok(())
}
