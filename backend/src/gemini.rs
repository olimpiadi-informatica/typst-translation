#![allow(unused)]

use axum::extract::State;
use common::error::Error;
use common::gemini::GeminiRequest;
use serde::Serialize;

use crate::AppState;
use crate::auth::AuthUser;
use crate::db_ops::task_db::get_task_by_id;

#[derive(Serialize)]
struct IOTextPart {
    text: String,
}

#[derive(Serialize)]
struct IOParts {
    parts: Vec<IOTextPart>,
}

#[derive(Serialize)]
struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    thinking_budget: i64,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i64,
    #[serde(rename = "thinkingConfig")]
    thinking_config: i64,
}

#[derive(Serialize)]
struct Query {
    system_instruction: IOParts,
    contents: IOParts,
    generation_config: GenerationConfig,
}

async fn get_ai_translation(
    State(app_state): State<AppState>,
    current_user: AuthUser,
    params: GeminiRequest,
) -> Result<String, Error> {
    let task = get_task_by_id(app_state.db(), params.task_id).await?;

    Err(Error::Other("Not implemented".to_string()))
}
