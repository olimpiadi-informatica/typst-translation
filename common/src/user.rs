use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub login_epoch: i64,
    pub automatic_translation_budget: i64,
    pub tokens_used: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtUser {
    pub user: Option<User>,
    pub is_admin: bool,
    pub is_staff: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

pub type WhoAmIResponse = Option<ExtUser>;
