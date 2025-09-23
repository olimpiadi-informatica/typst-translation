use serde::{Deserialize, Serialize};

use crate::translation::Translation;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    pub id: i64,
    pub contest_id: i64,
    pub name: String,
    pub translations: Vec<Translation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskDb {
    pub id: i64,
    pub contest_id: i64,
    pub name: String,
}
