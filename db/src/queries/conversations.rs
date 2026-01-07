// authorization, participant checks, conversation fetches, rating checks, etc
// => deals with the conversation and conversation participant tables

use sqlx::PgPool;
use uuid::Uuid;

use crate::Result;

/// Check if a user has access to a conversation
pub async fn user_has_access(pool: &PgPool, conversation_id: Uuid, user_id: Uuid) -> Result<bool> {
    sqlx::query_scalar(
        r#"SELECT EXISTS(
            SELECT 1 FROM conversation_participant 
            WHERE conversation_id = $1 AND user_id = $2
        )"#,
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}
