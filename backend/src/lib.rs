use std::sync::Arc;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use color_eyre::eyre::Result;
use common::typst_packages::TypstPackagePayload;
use dashmap::DashMap;
use logging::trace_requests;
use reqwest::Client;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::services::{ServeDir, ServeFile};

pub mod api_handlers;
pub mod auth;
pub mod config;
pub mod db_ops;
pub mod file_storage;
mod gemini;
pub mod logging;

pub use logging::init_logging;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    db: SqlitePool,
    config: AppConfig,
    reqwest: Client,
    typst_packages: Arc<DashMap<TypstPackagePayload, Vec<u8>>>,
}

impl AppState {
    pub async fn new(config: AppConfig, db: SqlitePool) -> Result<Self> {
        Ok(Self {
            db,
            config,
            reqwest: Client::new(),
            typst_packages: Arc::new(DashMap::new()),
        })
    }

    pub fn db(&self) -> &SqlitePool {
        &self.db
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub async fn serve(self, listen_address: std::net::SocketAddr) -> Result<()> {
        let app = self.app();

        let listener = tokio::net::TcpListener::bind(&listen_address)
            .await
            .unwrap();
        tracing::info!("listening on http://{}", &listen_address);
        axum::serve(listener, app).await?;

        Ok(())
    }

    pub fn app(self) -> Router {
        Router::new()
            .route("/api/login", post(auth::login))
            .route("/api/admin/login", post(auth::admin_login))
            .route("/api/staff/login", post(auth::staff_login))
            .route("/api/whoami", post(auth::whoami))
            .route("/api/logout", post(auth::logout))
            .route(
                "/api/user/contestants_with_languages",
                post(api_handlers::languages::get_user_contestants),
            )
            .route(
                "/api/user/translation_languages",
                post(api_handlers::languages::get_user_translation_languages),
            )
            .route(
                "/api/user/available_languages",
                post(api_handlers::languages::get_available_languages),
            )
            .route(
                "/api/user/assign_language_to_contestant",
                post(api_handlers::languages::assign_language_to_contestant),
            )
            .route(
                "/api/user/toggle_language_public_status",
                post(api_handlers::languages::toggle_language_public_status),
            )
            .route(
                "/api/user/all_languages",
                get(api_handlers::languages::get_all_languages),
            )
            .route(
                "/api/user/translation_status",
                get(api_handlers::user::get_user_translation_status),
            )
            .route(
                "/api/user/finalize_translation",
                post(api_handlers::user::finalize_translation),
            )
            .route(
                "/api/user/set_translation_session_token",
                post(api_handlers::user::set_translation_session_token),
            )
            .route("/api/get_ai_translation", post(gemini::get_ai_translation))
            .route(
                "/api/tasks/{task_id}/statement_versions/latest",
                get(api_handlers::statement_versions::get_latest_statement_version),
            )
            .route(
                "/api/statement_version/{version_id}",
                get(api_handlers::statement_versions::get_statement_version),
            )
            .route(
                "/api/tasks/{task_id}/statement_versions/all",
                get(api_handlers::statement_versions::get_all_statement_versions),
            )
            .route(
                "/api/update_translation",
                post(api_handlers::task::update_translation),
            )
            .route(
                "/api/tasks/{task_id}",
                get(api_handlers::task::get_task_by_id),
            )
            .route("/api/tasks/import", post(api_handlers::task::import_task))
            .route(
                "/api/languages/{language_id}",
                get(api_handlers::languages::get_language_by_id),
            )
            .route(
                "/api/typst_packages",
                post(api_handlers::typst_packages::get_typst_package),
            )
            .route(
                "/api/contests/get_all",
                get(api_handlers::contest::get_all_contests),
            )
            .route(
                "/api/admin/users/overview",
                get(api_handlers::admin::get_users_overview),
            )
            .route(
                "/api/admin/users/set_budget",
                post(api_handlers::admin::set_user_budget),
            )
            .route(
                "/api/admin/users/set_all_budgets",
                post(api_handlers::admin::set_all_users_budget),
            )
            .route(
                "/api/admin/users/add_language",
                post(api_handlers::admin::add_user_language),
            )
            .route(
                "/api/admin/users/impersonate",
                post(api_handlers::admin::impersonate_user),
            )
            .route(
                "/api/admin/users/update_passwords",
                post(api_handlers::admin::update_passwords_jsonl),
            )
            .route(
                "/api/admin/users/import",
                post(api_handlers::admin::import_users),
            )
            .route(
                "/api/admin/export/translations/{contest_id}",
                get(api_handlers::admin::export_translations),
            )
            .route(
                "/api/admin/update_task_files",
                post(api_handlers::admin::update_task_files),
            )
            .route(
                "/api/admin/create_contest",
                post(api_handlers::admin::create_contest),
            )
            .route(
                "/api/admin/contest/contestant/print_status",
                post(api_handlers::admin::update_contestant_print_status),
            )
            .route(
                "/api/admin/contestant/update",
                post(api_handlers::admin::update_contestant),
            )
            .route("/api/all", get(api_handlers::contest::all))
            .route("/files/{hash}/{filename}", get(file_storage::get_file))
            .fallback_service(
                ServeDir::new("dist")
                    .precompressed_br()
                    .not_found_service(ServeFile::new("dist/index.html")),
            )
            .layer(axum::middleware::from_fn(trace_requests))
            .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
            .with_state(self)
    }
}

pub async fn init() -> Result<SqlitePool> {
    color_eyre::install()?;

    // ignore missing .env files.
    let _ = dotenvy::dotenv();

    init_logging();
    let database_url = std::env::var("DATABASE_URL").unwrap_or("./db.sqlite".to_string());

    let db = SqlitePoolOptions::new().connect(&database_url).await?;

    sqlx::migrate!().run(&db).await?;

    Ok(db)
}
