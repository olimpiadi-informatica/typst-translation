use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Language {
    pub id: i64,
    pub code: String,
    pub user_id: i64,
}
