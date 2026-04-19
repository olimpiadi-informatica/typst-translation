use axum::Json;
use axum::extract::State;
use common::contest::ContestWithTasksAndStatus;
use common::error::Error;
use common::user_contest_status::SetTranslationSessionTokenRequest;

use crate::AppState;
use crate::auth::AuthUser;
use crate::db_ops::{contest_db, translation_db};

pub async fn get_user_translation_status(
    State(app_state): State<AppState>,
    user: AuthUser,
) -> Result<Json<Vec<ContestWithTasksAndStatus>>, Error> {
    let statuses_with_tasks =
        contest_db::get_user_contest_statuses_and_tasks(app_state.db(), user.id).await?;

    Ok(Json(statuses_with_tasks))
}

pub async fn finalize_translation(
    State(app_state): State<AppState>,
    user: AuthUser,
    Json(contest_id): Json<i64>,
) -> Result<Json<()>, Error> {
    translation_db::finalize_translation(app_state.db(), user.id, contest_id).await?;
    Ok(Json(()))
}

pub async fn set_translation_session_token(
    State(app_state): State<AppState>,
    user: AuthUser,
    Json(payload): Json<SetTranslationSessionTokenRequest>,
) -> Result<Json<()>, Error> {
    translation_db::set_translation_session_token(
        app_state.db(),
        user.id,
        payload.task_id,
        payload.language_id,
        payload.session_token,
    )
    .await?;
    Ok(Json(()))
}
