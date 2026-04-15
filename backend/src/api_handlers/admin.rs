use axum::Json;
use axum::extract::State;
use common::admin::{
    AddUserLanguageRequest, AdminUserOverview, AdminUserOverviewResponse, SetAllBudgetsRequest,
    SetBudgetRequest, UpdatePasswordsCsvRequest,
};
use common::error::Error;
use common::language::Language;

use crate::AppState;
use crate::auth::AuthAdmin;
use crate::db_ops::{language_db, user_db};

pub async fn get_users_overview(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
) -> Result<Json<AdminUserOverviewResponse>, Error> {
    let pool = app_state.db();
    let users = user_db::get_all(pool).await?;
    let all_languages = language_db::get_all(pool).await?;

    let mut overview = Vec::new();
    for user in users {
        let user_languages = all_languages
            .iter()
            .filter(|l| l.user_id == user.id)
            .cloned()
            .collect();
        overview.push(AdminUserOverview {
            user,
            languages: user_languages,
        });
    }

    Ok(Json(overview))
}

pub async fn set_user_budget(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<SetBudgetRequest>,
) -> Result<Json<()>, Error> {
    user_db::set_automatic_translation_budget(app_state.db(), payload.user_id, payload.new_budget)
        .await?;
    Ok(Json(()))
}

pub async fn set_all_users_budget(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<SetAllBudgetsRequest>,
) -> Result<Json<()>, Error> {
    user_db::set_all_automatic_translation_budgets(app_state.db(), payload.new_budget).await?;
    Ok(Json(()))
}

pub async fn add_user_language(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<AddUserLanguageRequest>,
) -> Result<Json<()>, Error> {
    let mut tx = app_state.db().begin().await?;

    let mut language = Language {
        id: 0,
        code: payload.language_code,
        user_id: payload.user_id,
        public: false,
    };
    language_db::insert(&mut language, &mut *tx).await?;

    sqlx::query!(
        "INSERT INTO translations (task_id, language_id) SELECT id, ? FROM tasks",
        language.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(()))
}

pub async fn update_passwords_csv(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<UpdatePasswordsCsvRequest>,
) -> Result<Json<()>, Error> {
    let mut reader = csv::Reader::from_reader(payload.csv_content.as_bytes());
    let mut tx = app_state.db().begin().await?;

    for result in reader.records() {
        let record: csv::StringRecord =
            result.map_err(|e| Error::InvalidInput(format!("CSV error: {e}")))?;
        // Expected format: username,password
        let username = record
            .get(0)
            .ok_or_else(|| Error::InvalidInput("Missing username".to_string()))?;
        let password = record
            .get(1)
            .ok_or_else(|| Error::InvalidInput("Missing password".to_string()))?;

        let existing_user = user_db::get_user_by_username(&mut *tx, username).await?;

        if let Some(mut user) = existing_user {
            user.password = password.to_string();
            user.login_epoch += 1;
            user_db::update(&user, &mut *tx).await?;
        } else {
            return Err(Error::InvalidInput(format!("User not found: {username}")));
        }
    }

    tx.commit().await?;
    Ok(Json(()))
}
