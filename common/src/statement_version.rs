use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server-side")]
use sqlx::types::Json;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatementVersion {
    pub id: i64,
    pub task_id: i64,
    pub version_hash: String,
    #[cfg(feature = "server-side")]
    pub content_manifest: Json<HashMap<String, String>>,
    #[cfg(not(feature = "server-side"))]
    pub content_manifest: HashMap<String, String>,
    pub is_live: bool,
    pub created_at: NaiveDateTime,
}
