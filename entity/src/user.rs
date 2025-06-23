use chrono::{DateTime, Utc};
use uuid::Uuid;

pub enum AccountStatus {
    Active,
    Suspended(DateTime<Utc>),
    Banned,
    Unverified,
    Deactivated,
}

pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub status: AccountStatus,
    pub created_at: DateTime<Utc>,
    pub last_login: DateTime<Utc>,
    pub rating: f32,
}
