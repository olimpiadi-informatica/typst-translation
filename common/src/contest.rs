use serde::{Deserialize, Serialize};

use crate::task::Task;
use crate::user_contest_status::UserContestStatus;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contest {
    pub id: i64,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContestWithTasksAndStatus {
    pub contest: Contest,
    pub user_contest_status: UserContestStatus,
    pub tasks: Vec<Task>,
}
