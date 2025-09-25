use axum::Json;
use axum::extract::{Path, State};
use common::error::Error;

use crate::AppState;
use crate::auth::AuthUser;
use crate::db_ops::statement_version_db;

pub async fn get_latest_statement_version(
    Path(task_id): Path<i64>,
    _user: AuthUser,
    State(app_state): State<AppState>,
) -> Result<Json<common::statement_version::StatementVersion>, Error> {
    let statement_version =
        statement_version_db::get_latest_statement_version_by_task_id(app_state.db(), task_id)
            .await?;

    match statement_version {
        Some(sv) => Ok(Json(sv)),
        None => Err(Error::NotFound),
    }
}
