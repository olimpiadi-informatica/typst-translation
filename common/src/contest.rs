use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contest {
    pub id: i64,
    pub name: String,
}
