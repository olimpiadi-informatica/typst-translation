use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    /// This is the password hash
    pub password: String,
    pub login_epoch: i64,
    pub automatic_translation_budget: i64,
    pub tokens_used: i64, // Added
    pub name: String,     // Added
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginParams {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WhoAmIResponse {
    RegularUser(User),
    AdminUser,
    StaffUser,
    Nobody,
}
