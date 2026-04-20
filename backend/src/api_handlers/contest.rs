use axum::Json;
use axum::extract::State;
use common::contest::{All, Contest};
use common::error::Error;

use crate::AppState;
use crate::auth::{AuthAny, AuthStaff};
use crate::db_ops::contest_db;

pub async fn get_all_contests(
    State(app_state): State<AppState>,
    _user: AuthAny,
) -> Result<Json<Vec<Contest>>, Error> {
    let pool = app_state.db();
    let contests = contest_db::get_all(pool).await?;
    Ok(Json(contests))
}

pub async fn all(State(app_state): State<AppState>, _: AuthStaff) -> Result<Json<All>, Error> {
    let pool = app_state.db();
    let contests = contest_db::get_all_contests_with_all(pool).await?;
    Ok(Json(contests))
}
