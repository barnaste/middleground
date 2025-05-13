use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

pub struct Source {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publication: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub content_summary: Option<String>,
    pub credibility: f32,
}
