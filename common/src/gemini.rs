use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum GeminiModel {
    Gemini25Pro,
    Gemini25Flash,
}

impl GeminiModel {
    pub fn api_name(&self) -> &'static str {
        match self {
            GeminiModel::Gemini25Pro => "gemini-2.5-pro",
            GeminiModel::Gemini25Flash => "gemini-2.5-flash",
        }
    }

    // cost in 1/10^9 dollars per token.
    pub fn token_input_cost(&self) -> i64 {
        match self {
            GeminiModel::Gemini25Pro => 1250,
            GeminiModel::Gemini25Flash => 300,
        }
    }

    // cost in 1/10^9 dollars per token.
    pub fn token_output_cost(&self) -> i64 {
        match self {
            GeminiModel::Gemini25Pro => 10000,
            GeminiModel::Gemini25Flash => 2500,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GeminiRequest {
    pub task_id: i64,
    pub prompt: String,
    pub model: GeminiModel,
}
