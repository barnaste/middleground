// => deals with the message and message atom tables

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{DbError, Result};

#[derive(Debug)]
pub struct CreateMessageParams {
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub quoted_id: Option<Uuid>,
    pub content: String,
}

/// Create a new message, and return it's ID on success
pub async fn create_message(pool: &PgPool, params: CreateMessageParams) -> Result<Uuid> {
    // start a transaction, since we need to create both message and message atom
    let mut tx = pool.begin().await?;

    // create a new message
    let message_id: Uuid = sqlx::query_scalar!(
        r#"
        INSERT INTO message (conversation_id, sender_id, created_at, quoted_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
        params.conversation_id,
        params.sender_id,
        params.created_at,
        params.quoted_id,
    )
    .fetch_one(&mut *tx)
    .await?;

    // create a new message atom with the message's content
    sqlx::query!(
        r#"
        INSERT INTO message_atom (message_id, content, created_at)
        VALUES ($1, $2, $3)  
        "#,
        message_id,
        params.content,
        params.created_at,
    )
    .execute(&mut *tx)
    .await.inspect_err(|e| println!("{e}"))?;

    tx.commit().await?;
    Ok(message_id)
}

pub struct EditMessageParams {
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub message_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub content: String,
}

/// Edit a message with the given ID
pub async fn edit_message(pool: &PgPool, params: EditMessageParams) -> Result<()> {
    // start a transaction, as we must verify the user is the owner and then edit
    let mut tx = pool.begin().await?;

    let is_owner: bool = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
        SELECT 1 FROM message
        WHERE id = $1 AND conversation_id = $2 AND sender_id = $3
        )
        "#,
        params.message_id,
        params.conversation_id,
        params.sender_id,
    )
    .fetch_one(&mut *tx)
    .await?
    .ok_or::<DbError>(sqlx::Error::RowNotFound.into())?;

    if !is_owner {
        return Err(DbError::Query(sqlx::Error::RowNotFound));
    }

    // get the current max revision number
    let max_revision: i32 = sqlx::query_scalar(
        r#"
        SELECT MAX(revision_number) FROM message_atom
        WHERE message_id = $1
        "#,
    )
    .bind(params.message_id)
    .fetch_one(&mut *tx)
    .await?;

    // create a new message atom with an incremented revision number
    sqlx::query!(
        r#"
        INSERT INTO message_atom (message_id, content, revision_number, created_at)
        VALUES ($1, $2, $3, $4)
        "#,
        params.message_id,
        params.content,
        max_revision + 1,
        params.created_at,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

pub struct DeleteMessageParams {
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub message_id: Uuid,
}

pub async fn soft_delete_message(pool: &PgPool, params: DeleteMessageParams) -> Result<()> {
    let result = sqlx::query!(
        r#"
        UPDATE message
        SET is_deleted = true
        WHERE id = $1 AND conversation_id = $2 AND sender_id = $3
        "#,
        params.message_id,
        params.conversation_id,
        params.sender_id,
    )
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::Query(sqlx::Error::RowNotFound));
    }
    Ok(())
}
