use axum::Json;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum_extra::extract::CookieJar;
use common::admin::{
    AddUserLanguageRequest, AdminUserOverview, AdminUserOverviewResponse, CreateContestRequest,
    ImpersonateUserRequest, ImportUsersRequest, SetAllBudgetsRequest, SetBudgetRequest,
    UpdateContestantPrintStatusRequest, UpdateContestantRequest, UpdatePasswordsJsonlRequest,
    UpdateTaskFilesRequest,
};
use common::error::Error;
use common::language::Language;
use common::statement_version::StatementVersion;
use serde::Deserialize;

use crate::AppState;
use crate::auth::{AuthAdmin, AuthAny, Claims, add_cookie};
use crate::db_ops::{contestant_db, language_db, statement_version_db, user_db};
use crate::file_storage::{path_of_file, save_file};

pub async fn get_users_overview(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
) -> Result<Json<AdminUserOverviewResponse>, Error> {
    let pool = app_state.db();
    let users = user_db::get_all(pool).await?;
    let all_languages = language_db::get_all(pool).await?;
    let all_contestants = contestant_db::get_all(pool).await?;

    let mut overview = Vec::new();
    for user in users {
        let user_languages = all_languages
            .iter()
            .filter(|l| l.user_id == user.id)
            .cloned()
            .collect();
        let user_contestants = all_contestants
            .iter()
            .filter(|c| c.user_id == user.id)
            .cloned()
            .collect();
        overview.push(AdminUserOverview {
            user,
            languages: user_languages,
            contestants: user_contestants,
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

pub async fn impersonate_user(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    current_user: Option<AuthAny>,
    cookies: CookieJar,
    Json(payload): Json<ImpersonateUserRequest>,
) -> Result<CookieJar, Error> {
    let user = user_db::get_by_id(app_state.db(), payload.user_id)
        .await?
        .ok_or(Error::NotFound)?;

    Ok(add_cookie(
        cookies,
        Claims {
            user_id: Some(user.id),
            login_epoch: user.login_epoch,
            admin: current_user.as_ref().is_some_and(|x| x.is_admin),
            staff: current_user.as_ref().is_some_and(|x| x.is_staff),
            exp: 0,
        },
        &app_state,
    ))
}

#[derive(Debug, Deserialize)]
struct PasswordUpdateRow {
    username: String,
    password: String,
}

pub async fn update_passwords_jsonl(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<UpdatePasswordsJsonlRequest>,
) -> Result<Json<()>, Error> {
    let mut tx = app_state.db().begin().await?;

    for line in payload.jsonl_content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let row: PasswordUpdateRow = serde_json::from_str(line)
            .map_err(|e| Error::InvalidInput(format!("JSON error: {e}")))?;

        let existing_user = user_db::get_user_by_username(&mut *tx, &row.username).await?;

        if let Some(mut user) = existing_user {
            user.password = row.password;
            user.login_epoch += 1;
            user_db::update(&user, &mut *tx).await?;
        } else {
            return Err(Error::InvalidInput(format!(
                "User not found: {}",
                row.username
            )));
        }
    }

    tx.commit().await?;
    Ok(Json(()))
}

#[derive(Debug, Deserialize)]
struct ImportUserContestant {
    name: String,
    code: String,
    #[serde(default)]
    online_bit: bool,
}

#[derive(Debug, Deserialize)]
struct ImportUserRow {
    contestants: Vec<ImportUserContestant>,
    username: String,
    languages: Vec<String>,
}

pub async fn import_users(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<ImportUsersRequest>,
) -> Result<Json<()>, Error> {
    let mut tx = app_state.db().begin().await?;

    for line in payload.jsonl_content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let user_row: ImportUserRow = serde_json::from_str(line)
            .map_err(|e| Error::InvalidInput(format!("JSON error: {e}")))?;

        let password = uuid::Uuid::new_v4().to_string();
        let user_id = sqlx::query!(
            "INSERT INTO users(username, password, login_epoch) VALUES (?, ?, 0) RETURNING id;",
            user_row.username,
            password
        )
        .fetch_one(&mut *tx)
        .await?
        .id;

        sqlx::query!(
            "INSERT INTO user_contest_status (user_id, contest_id) SELECT ?, id FROM contests",
            user_id
        )
        .execute(&mut *tx)
        .await?;

        for lang in user_row.languages {
            let lang_id = sqlx::query!(
                "INSERT INTO languages(code, user_id) VALUES (?, ?) RETURNING id;",
                lang,
                user_id
            )
            .fetch_one(&mut *tx)
            .await?
            .id;

            sqlx::query!(
                "INSERT INTO translations (task_id, language_id) SELECT id, ? FROM tasks",
                lang_id
            )
            .execute(&mut *tx)
            .await?;
        }

        for contestant in user_row.contestants {
            let _contestant_id = sqlx::query!(
                "INSERT INTO contestants(code, name, online_bit, user_id) VALUES (?, ?, ?, ?) RETURNING id;",
                contestant.code,
                contestant.name,
                contestant.online_bit,
                user_id
            )
            .fetch_one(&mut *tx)
            .await?
            .id;
        }
    }

    tx.commit().await?;
    Ok(Json(()))
}

pub async fn export_translations(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Path(contest_id): Path<i64>,
) -> Result<impl IntoResponse, Error> {
    let pool = app_state.db();

    let translations = sqlx::query!(
        r#"
        SELECT languages.code, content_hash AS "content_hash!", tasks.name
        FROM translations
          JOIN languages ON translations.language_id = languages.id
          JOIN tasks ON translations.task_id = tasks.id
        WHERE tasks.contest_id = ? AND content_hash IS NOT NULL
        "#,
        contest_id
    )
    .fetch_all(pool)
    .await?;

    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));
        for translation in translations {
            let in_path = path_of_file(&translation.content_hash)?;
            let content = std::fs::read(in_path)?;
            let out_path = format!("{}/{}.typ", translation.name, translation.code);
            zip.start_file::<_, ()>(out_path, zip::write::FileOptions::default())?;
            std::io::Write::write_all(&mut zip, &content)?;
        }
        zip.finish()?;
    }

    Ok((
        [
            (axum::http::header::CONTENT_TYPE, "application/zip"),
            (
                axum::http::header::CONTENT_DISPOSITION,
                "attachment; filename=\"translations.zip\"",
            ),
        ],
        buf,
    ))
}

pub async fn update_task_files(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<UpdateTaskFilesRequest>,
) -> Result<Json<()>, Error> {
    let pool = app_state.db();

    let mut latest_version =
        statement_version_db::get_latest_statement_version_by_task_id(pool, payload.task_id)
            .await?;

    for (file_path, content) in payload.files {
        let content_hash = save_file(&content).await?;
        latest_version
            .content_manifest
            .insert(file_path, content_hash);
    }

    let mut new_version = StatementVersion {
        id: 0,
        task_id: payload.task_id,
        content_manifest: latest_version.content_manifest,
        is_live: true,
        created_at: chrono::Utc::now().naive_utc(),
    };

    statement_version_db::insert(pool, &mut new_version).await?;

    Ok(Json(()))
}

pub async fn create_contest(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<CreateContestRequest>,
) -> Result<Json<()>, Error> {
    let pool = app_state.db();
    let mut tx = pool.begin().await?;

    let contest_id = sqlx::query!(
        "INSERT INTO contests (name) VALUES (?) RETURNING id",
        payload.name
    )
    .fetch_one(&mut *tx)
    .await?
    .id;

    let users = user_db::get_all(&mut *tx).await?;
    for user in users {
        sqlx::query!(
            "INSERT INTO user_contest_status (user_id, contest_id) VALUES (?, ?)",
            user.id,
            contest_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(()))
}

pub async fn update_contestant_print_status(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<UpdateContestantPrintStatusRequest>,
) -> Result<Json<()>, Error> {
    let pool = app_state.db();
    if payload.printed {
        sqlx::query!(
            "INSERT OR IGNORE INTO contestant_print_status (contestant_id, contest_id) VALUES (?, ?)",
            payload.contestant_id,
            payload.contest_id
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            "DELETE FROM contestant_print_status WHERE contestant_id = ? AND contest_id = ?",
            payload.contestant_id,
            payload.contest_id
        )
        .execute(pool)
        .await?;
    }
    Ok(Json(()))
}

pub async fn update_contestant(
    State(app_state): State<AppState>,
    _admin: AuthAdmin,
    Json(payload): Json<UpdateContestantRequest>,
) -> Result<Json<()>, Error> {
    contestant_db::update_contestant(
        app_state.db(),
        payload.id,
        payload.code,
        payload.name,
        payload.online_bit,
    )
    .await?;
    Ok(Json(()))
}
