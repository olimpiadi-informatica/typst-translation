use axum::Json;
use axum::extract::State;
use common::contestant::Contestant;
use common::error::Error;
use common::language::{AssignLanguagePayload, Language, ToggleLanguagePublicStatusPayload};

use crate::AppState;
use crate::auth::AuthUser;
use crate::db_ops::{DatabaseOps, contestant_db, language_db};

pub async fn get_user_contestants(
    State(app_state): State<AppState>,
    current_user: AuthUser,
) -> Result<Json<Vec<Contestant>>, Error> {
    let user = current_user.as_user().ok_or(Error::Forbidden)?;
    let pool = app_state.db();
    Ok(Json(
        contestant_db::get_contestants_by_user_id(pool, user.id).await?,
    ))
}

pub async fn get_user_translation_languages(
    State(app_state): State<AppState>,
    current_user: AuthUser,
) -> Result<Json<Vec<Language>>, Error> {
    let user = current_user.as_user().ok_or(Error::Forbidden)?;
    let pool = app_state.db();

    let languages = language_db::get_user_owned_languages(pool, user.id).await?;

    Ok(Json(languages))
}

pub async fn get_available_languages(
    State(app_state): State<AppState>,
    current_user: AuthUser,
) -> Result<Json<Vec<Language>>, Error> {
    let user = current_user.as_user().ok_or(Error::Forbidden)?;
    let pool = app_state.db();

    let languages = language_db::get_available_languages(pool, user.id).await?;

    Ok(Json(languages))
}

pub async fn assign_language_to_contestant(
    State(app_state): State<AppState>,
    current_user: AuthUser,
    Json(payload): Json<AssignLanguagePayload>,
) -> Result<(), Error> {
    let user = current_user.as_user().ok_or(Error::Forbidden)?;
    let pool = app_state.db();

    contestant_db::assign_language_to_contestant(
        pool,
        payload.contestant_id,
        payload.language_id,
        user.id,
    )
    .await?;

    Ok(())
}

pub async fn toggle_language_public_status(
    State(app_state): State<AppState>,
    current_user: AuthUser,
    Json(payload): Json<ToggleLanguagePublicStatusPayload>,
) -> Result<(), Error> {
    let user = current_user.as_user().ok_or(Error::Forbidden)?;
    let pool = app_state.db();

    // First, check if the user owns the language
    let language = Language::get_by_id(pool, payload.language_id)
        .await?
        .ok_or(Error::NotFound)?;
    if language.user_id != user.id {
        return Err(Error::Forbidden);
    }

    language_db::toggle_language_public_status(pool, payload.language_id, payload.public).await?;

    Ok(())
}
