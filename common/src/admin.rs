use serde::{Deserialize, Serialize};

use crate::language::Language;
use crate::user::User;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AdminUserOverview {
    pub user: User,
    pub languages: Vec<Language>,
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
pub struct UpdatePasswordsCsvRequest {
    pub csv_content: String,
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
