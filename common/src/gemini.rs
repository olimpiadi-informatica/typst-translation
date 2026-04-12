use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum GeminiModel {
    Gemini31Pro,
    Gemini31FlashLite,
}

impl GeminiModel {
    pub fn api_name(&self) -> &'static str {
        match self {
            GeminiModel::Gemini31Pro => "gemini-3.1-pro-preview",
            GeminiModel::Gemini31FlashLite => "gemini-3.1-flash-lite-preview",
        }
    }

    // cost in 1/10^9 dollars per token.
    pub fn token_input_cost(&self) -> i64 {
        match self {
            GeminiModel::Gemini31Pro => 2000,
            GeminiModel::Gemini31FlashLite => 250,
        }
    }

    // cost in 1/10^9 dollars per token.
    pub fn token_output_cost(&self) -> i64 {
        match self {
            GeminiModel::Gemini31Pro => 12000,
            GeminiModel::Gemini31FlashLite => 1500,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct GeminiRequest {
    pub task_id: i64,
    pub prompt: String,
    pub model: GeminiModel,
}
