use axum::Json;
use axum::extract::State;
use common::error::Error;
use common::user_contest_status::SetTranslationSessionTokenRequest;

use crate::AppState;
use crate::auth::AuthUser;
use crate::db_ops::{contest_db, translation_db, user_db};

pub async fn get_user_translation_status(
    State(app_state): State<AppState>,
    current_user: AuthUser,
) -> Result<Json<Vec<common::contest::ContestWithTasksAndStatus>>, Error> {
    let user = current_user.as_user().ok_or(Error::Forbidden)?;
    let statuses_with_tasks =
        contest_db::get_user_contest_statuses_and_tasks(app_state.db(), user.id).await?;

    Ok(Json(statuses_with_tasks))
}

pub async fn skip_envelope_verification(
    State(app_state): State<AppState>,
    current_user: AuthUser,
    Json(payload): Json<common::user_contest_status::SkipEnvelopeVerificationRequest>,
) -> Result<Json<()>, Error> {
    let user = current_user.as_user().ok_or(Error::Forbidden)?;
    user_db::set_skip_envelope_verification(
        app_state.db(),
        user.id,
        payload.contest_id,
        payload.skip,
    )
    .await?;
    Ok(Json(()))
}

pub async fn finalize_translation(
    State(app_state): State<AppState>,
    current_user: AuthUser,
    Json(contest_id): Json<i64>,
) -> Result<Json<()>, Error> {
    let user = current_user.as_user().ok_or(Error::Forbidden)?;
    translation_db::finalize_translation(app_state.db(), user.id, contest_id).await?;
    Ok(Json(()))
}

pub async fn set_translation_session_token(
    State(app_state): State<AppState>,
    current_user: AuthUser,
    Json(payload): Json<SetTranslationSessionTokenRequest>,
) -> Result<Json<()>, Error> {
    let user = current_user.as_user().ok_or(Error::Forbidden)?;
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
