use serde::{Deserialize, Serialize};

use crate::contestant::Contestant;
use crate::language::Language;
use crate::task::Task;
use crate::user::User;
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContestWithAll {
    pub contest: Contest,
    pub user_contest_status: Vec<UserContestStatus>,
    pub tasks: Vec<Task>,
    pub printed_contestants: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct All {
    pub contests: Vec<ContestWithAll>,
    pub contestants: Vec<Contestant>,
    pub languages: Vec<Language>,
    pub users: Vec<User>,
}
