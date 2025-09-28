use axum::Json;
use axum::extract::{Path, State};
use common::error::Error;
use common::statement_version::StatementVersion;

use crate::AppState;
use crate::auth::{AuthAny, AuthUser};
use crate::db_ops::statement_version_db;

pub async fn get_latest_statement_version(
    Path(task_id): Path<i64>,
    _user: AuthAny,
    State(app_state): State<AppState>,
) -> Result<Json<StatementVersion>, Error> {
    Ok(Json(
        statement_version_db::get_latest_statement_version_by_task_id(app_state.db(), task_id)
            .await?,
    ))
}

pub async fn get_statement_version(
    Path(id): Path<i64>,
    _user: AuthUser,
    State(app_state): State<AppState>,
) -> Result<Json<StatementVersion>, Error> {
    Ok(Json(
        statement_version_db::get_by_id(app_state.db(), id).await?,
    ))
}

pub async fn get_all_statement_versions(
    Path(task_id): Path<i64>,
    _user: AuthUser,
    State(app_state): State<AppState>,
) -> Result<Json<Vec<StatementVersion>>, Error> {
    Ok(Json(
        statement_version_db::get_all_statement_versions_by_task_id(app_state.db(), task_id)
            .await?,
    ))
}
