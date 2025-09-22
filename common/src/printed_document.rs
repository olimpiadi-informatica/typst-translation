use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrintedDocument {
    pub id: i64,
    pub contestant_id: i64,
    pub statement_version_id: i64,
    pub language_id: Option<i64>, // NULL for original English statement
    pub printed_at: NaiveDateTime,
}
