use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Language {
    pub id: i64,
    pub code: String,
    pub user_id: i64,
    pub public: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignLanguagePayload {
    pub contestant_id: i64,
    pub language_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToggleLanguagePublicStatusPayload {
    pub language_id: i64,
    pub public: bool,
}
