use chrono::{DateTime, Utc};
use db::queries::messages as query;
use redis::{AsyncCommands, Client as RedisClient};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::WsResult;

// Incoming message type from client
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum IncomingMessage {
    Send { payload: SendPayload },
    Edit { payload: EditPayload },
    Delete { payload: DeletePayload },
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendPayload {
    pub content: String,
    pub quoted_id: Option<Uuid>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EditPayload {
    pub message_id: Uuid,
    pub content: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DeletePayload {
    pub message_id: Uuid,
}

#[derive(Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum OutgoingMessage {
    Send {
        message_id: Uuid,
        sender_id: Uuid,
        content: String,
        quoted_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
    },
    Edit {
        message_id: Uuid,
        sender_id: Uuid,
        content: String,
        timestamp: DateTime<Utc>,
    },
    Delete {
        message_id: Uuid,
        sender_id: Uuid,
        timestamp: DateTime<Utc>,
    },
}

impl IncomingMessage {
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

async fn handle_send(
    user_id: Uuid,
    conversation_id: Uuid,
    payload: SendPayload,
    db: &PgPool,
) -> WsResult<OutgoingMessage> {
    let now = Utc::now();

    let message_id = query::create_message(
        db,
        query::CreateMessageParams {
            conversation_id,
            sender_id: user_id,
            quoted_id: payload.quoted_id,
            created_at: now,
            content: payload.content.clone(),
        },
    )
    .await?;

    Ok(OutgoingMessage::Send {
        message_id,
        sender_id: user_id,
        content: payload.content,
        quoted_id: payload.quoted_id,
        timestamp: now,
    })
}

async fn handle_edit(
    user_id: Uuid,
    conversation_id: Uuid,
    payload: EditPayload,
    db: &PgPool,
) -> WsResult<OutgoingMessage> {
    let now = Utc::now();

    query::edit_message(
        db,
        query::EditMessageParams {
            conversation_id,
            sender_id: user_id,
            message_id: payload.message_id,
            created_at: now,
            content: payload.content.clone(),
        },
    )
    .await?;

    Ok(OutgoingMessage::Edit {
        message_id: payload.message_id,
        sender_id: user_id,
        content: payload.content,
        timestamp: now,
    })
}

async fn handle_delete(
    user_id: Uuid,
    conversation_id: Uuid,
    payload: DeletePayload,
    db: &PgPool,
) -> WsResult<OutgoingMessage> {
    let now = Utc::now();

    query::soft_delete_message(
        db,
        query::DeleteMessageParams {
            conversation_id,
            sender_id: user_id,
            message_id: payload.message_id,
        },
    )
    .await?;

    Ok(OutgoingMessage::Delete {
        message_id: payload.message_id,
        sender_id: user_id,
        timestamp: now,
    })
}

pub async fn publish_msg(
    conversation_id: Uuid,
    message: OutgoingMessage,
    redis: &RedisClient,
) -> WsResult<()> {
    let mut conn = redis
        .get_multiplexed_async_connection()
        .await
        .inspect_err(|e| tracing::error!("Failed to create Redis connection: {}", e))?;
    let channel = format!("conversation:{}", conversation_id);
    let payload = serde_json::to_string(&message)?;

    conn.publish::<_, _, ()>(channel, payload).await?;

    Ok(())
}
