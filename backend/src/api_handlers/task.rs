use axum::Json;
use axum::extract::{Path, State};
use common::error::Error;
use common::task::Task;

use crate::AppState;
use crate::db_ops::{task_db, translation_db};

pub async fn get_task_by_id(
    Path(task_id): Path<i64>,
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
