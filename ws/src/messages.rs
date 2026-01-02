// TODO: handle message type logic (send, reply, delete, edit)

use chrono::{DateTime, Utc};
use redis::Client as RedisClient;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::WsError;

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
    ) -> Result<(), WsError> {
        match self {
            Self::Send { payload } => {
                handle_send(user_id, conversation_id, payload, db).await
            }
            Self::Edit { payload } => {
                handle_edit(user_id, conversation_id, payload, db).await
            }
            Self::Delete { payload } => {
                handle_delete(user_id, conversation_id, payload, db).await
            }
        }
    }
}

async fn handle_send(
    user_id: Uuid,
    conversation_id: Uuid,
    payload: SendPayload,
    db: &PgPool,
) -> Result<(), WsError> {
    todo!()
}

async fn handle_edit(
    user_id: Uuid,
    conversation_id: Uuid,
    payload: EditPayload,
    db: &PgPool,
) -> Result<(), WsError> {
    todo!()
}

async fn handle_delete(
    user_id: Uuid,
    conversation_id: Uuid,
    payload: DeletePayload,
    db: &PgPool,
) -> Result<(), WsError> {
    todo!()
}

async fn publish_msg(
    conversation_id: Uuid,
    message: OutgoingMessage,
    redis: &RedisClient,
) {
    todo!()
}
