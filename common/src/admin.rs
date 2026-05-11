use serde::{Deserialize, Serialize};

use crate::contestant::Contestant;
use crate::language::Language;
use crate::user::User;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdminUserOverview {
    pub user: User,
    pub languages: Vec<Language>,
    pub contestants: Vec<Contestant>,
}

pub type AdminUserOverviewResponse = Vec<AdminUserOverview>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetBudgetRequest {
    pub user_id: i64,
    pub new_budget: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetAllBudgetsRequest {
    pub new_budget: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddUserLanguageRequest {
    pub user_id: i64,
    pub language_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpersonateUserRequest {
    pub user_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePasswordsJsonlRequest {
    pub jsonl_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportUsersRequest {
    pub jsonl_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskFilesRequest {
    pub task_id: i64,
    pub files: Vec<(String, Vec<u8>)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContestRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContestantPrintStatusRequest {
    pub contest_id: i64,
    pub contestant_id: i64,
    pub printed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContestantRequest {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub online_bit: bool,
}
