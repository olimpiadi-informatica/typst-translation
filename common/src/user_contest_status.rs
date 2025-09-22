use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserContestStatus {
    pub id: i64,
    pub user_id: i64,
    pub contest_id: i64,
    pub finalized_translations: bool,
    pub skip_envelope_verification: bool,
    pub envelope_received_at: Option<NaiveDateTime>,
}
