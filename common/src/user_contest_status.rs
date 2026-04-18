use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserContestStatus {
    pub id: i64,
    pub user_id: i64,
    pub contest_id: i64,
    pub finalized_translations: bool,
    pub finalized_at: Option<NaiveDateTime>,
    pub skip_envelope_verification: bool,
    pub envelope_received_at: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkipEnvelopeVerificationRequest {
    pub contest_id: i64,
    pub skip: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetTranslationSessionTokenRequest {
    pub task_id: i64,
    pub language_id: i64,
    pub session_token: String,
}
