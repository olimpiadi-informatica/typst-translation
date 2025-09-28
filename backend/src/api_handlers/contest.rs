use axum::Json;
use axum::extract::State;
use common::contest::Contest;
use common::error::Error;

use crate::AppState;
use crate::auth::AuthAny;
use crate::db_ops::contest_db;

pub async fn get_all_contests(
    State(app_state): State<AppState>,
    _user: AuthAny,
) -> Result<Json<Vec<Contest>>, Error> {
    let pool = app_state.db();
    let contests = contest_db::get_all(pool).await?;
    Ok(Json(contests))
}
