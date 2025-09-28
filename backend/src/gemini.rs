use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use common::error::Error;
use common::gemini::GeminiRequest;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::AppState;
use crate::auth::AuthUser;
use crate::db_ops::statement_version_db;
use crate::db_ops::task_db::get_task_by_id;
use crate::db_ops::user_db::get_by_id;
use crate::file_storage::path_of_file;

#[derive(Debug, Serialize, Deserialize)]
struct IOTextPart {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IOParts {
    parts: Vec<IOTextPart>,
    role: Option<String>,
}

#[derive(Debug, Serialize)]
struct ThinkingConfig {
    #[serde(rename = "thinkingBudget")]
    thinking_budget: i64,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i64,
    #[serde(rename = "thinkingConfig")]
    thinking_config: ThinkingConfig,
}

#[derive(Debug, Serialize)]
struct Query {
    system_instruction: IOParts,
    contents: IOParts,
    generation_config: GenerationConfig,
}

#[derive(Deserialize, Debug)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: i64,
    #[serde(rename = "totalTokenCount")]
    total_token_count: i64,
    #[allow(unused)]
    #[serde(flatten)]
    other_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: IOParts,
    #[allow(unused)]
    #[serde(flatten)]
    other_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Response {
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: UsageMetadata,
    #[serde(flatten)]
    other_fields: HashMap<String, serde_json::Value>,
}

pub async fn get_ai_translation(
    State(app_state): State<AppState>,
    user: AuthUser,
    Json(params): Json<GeminiRequest>,
) -> Result<Json<String>, Error> {
    let task = get_task_by_id(app_state.db(), params.task_id).await?;

    if user.automatic_translation_budget == 0 {
        return Err(Error::TranslationBudgetExhausted);
    }

    let statement = statement_version_db::get_latest_statement_version_by_task_id(
        app_state.db(),
        params.task_id,
    )
    .await?;

    let statement_path = format!("{}/statement/statement.typ", task.name);
    let statement_hash = statement
        .content_manifest
        .0
        .get(&statement_path)
        .ok_or_else(|| Error::InternalServerError("statement file not found".to_string()))?;

    let statement = tokio::fs::read_to_string(path_of_file(statement_hash)?).await?;

    let req = Query {
        system_instruction: IOParts {
            parts: vec![IOTextPart {
                text: params.prompt,
            }],
            role: None,
        },
        generation_config: GenerationConfig {
            max_output_tokens: 100_000,
            thinking_config: ThinkingConfig {
                thinking_budget: 20_000,
            },
        },
        contents: IOParts {
            parts: vec![IOTextPart { text: statement }],
            role: None,
        },
    };

    let request = app_state
        .reqwest
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            params.model.api_name()
        ))
        .header("x-goog-api-key", &app_state.config.gemini_api_key)
        .json(&req);

    info!("sending translation request");

    let response = request.send().await?;
    response.error_for_status_ref()?;
    let response: Response = response.json().await?;

    let text = response.candidates[0].content.parts[0].text.clone();

    info!(
        response_other_fields = ?response.other_fields,
        usage = ?response.usage_metadata,
        response_len = text.len()
    );

    let usage = response.usage_metadata;

    let output_tokens = (usage.total_token_count - usage.prompt_token_count).max(0);

    let budget_cost = usage.prompt_token_count * params.model.token_input_cost()
        + output_tokens * params.model.token_output_cost();

    let dollar_cost = budget_cost as f64 / 1e9;

    info!(budget_cost, dollar_cost);

    let mut tx = loop {
        let tx = app_state.db().try_begin_with("BEGIN IMMEDIATE").await?;
        if let Some(tx) = tx {
            break tx;
        }
    };

    let user = get_by_id(&mut *tx, user.id).await?.unwrap();

    let new_budget = (user.automatic_translation_budget - budget_cost).max(0);
    let new_usage = user.tokens_used + budget_cost;

    sqlx::query!(
        "UPDATE users SET automatic_translation_budget = ?, tokens_used = ? WHERE id = ?",
        new_budget,
        new_usage,
        user.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(text))
}
