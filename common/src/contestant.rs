use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contestant {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub online_bit: bool,
    pub user_id: i64,
    pub language_id: Option<i64>, // NULL if no specific translation needed
}
