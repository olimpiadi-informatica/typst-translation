use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Translation {
    pub id: i64,
    pub task_id: i64,
    pub language_id: i64,
    pub content_hash: Option<String>,
    pub last_updated_at: NaiveDateTime,
    pub session_token: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpdateTranslationRequest {
    pub task_id: i64,
    pub language_id: i64,
    pub content: Vec<u8>,
    pub session_token: String,
}
