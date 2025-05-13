use chrono::{DateTime, Utc};
use uuid::Uuid;

pub enum ConversationRequestStatus {
    Pending,
    Expired,
    Matched,
}

pub struct ConversationRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub prompt: String,
    pub request_time: DateTime<Utc>,
    pub status: ConversationRequestStatus,
    pub match_id: Option<Uuid>,
}

pub enum ConversationEndReason {
    Completed,
    UserLeft,
    UserReported,
    Inactive,
}

pub struct Conversation {
    pub id: Uuid,
    pub topic: String,
    pub created_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub end_reason: Option<ConversationEndReason>,
    pub participant_a: Uuid,
    pub participant_b: Uuid,
}

pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub sent_at: DateTime<Utc>,
}
