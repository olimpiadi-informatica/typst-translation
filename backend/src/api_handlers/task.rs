use axum::Json;
use axum::extract::{Path, State};
use common::error::Error;
use common::task::Task;
use common::translation::{Translation, UpdateTranslationRequest};

use crate::AppState;
use crate::auth::AuthUser;
use crate::db_ops::{task_db, translation_db};
use crate::file_storage::save_file;

pub async fn get_task_by_id(
    Path(task_id): Path<i64>,
    _user: AuthUser,
    State(app_state): State<AppState>,
) -> Result<Json<Task>, Error> {
    let task = task_db::get_task_by_id(app_state.db(), task_id).await?;
    Ok(Json(Task {
        id: task.id,
        contest_id: task.contest_id,
        name: task.name,
        translations: translation_db::get_translations_by_task_id(app_state.db(), task_id).await?,
    }))
}

pub async fn update_translation(
    State(app_state): State<AppState>,
    _user: AuthUser,
    Json(req): Json<UpdateTranslationRequest>,
) -> Result<Json<()>, Error> {
    let content_hash = save_file(&req.content).await?;

    let mut tx = app_state.db().begin().await?;

    let translation = sqlx::query_as!(
        Translation,
        "SELECT * FROM translations WHERE task_id = ? AND language_id = ?",
        req.task_id,
        req.language_id
    )
    .fetch_one(&mut *tx)
    .await?;

    if translation.session_token != Some(req.session_token) {
        return Err(Error::InvalidInput(
            "Tried to update a translation without holding the lock".to_string(),
        ));
    }

    sqlx::query!(
        "UPDATE translations SET content_hash = ? WHERE id = ?",
        content_hash,
        translation.id
    )
    .execute(&mut *tx)
    .await?;

    Ok(Json(()))
}
