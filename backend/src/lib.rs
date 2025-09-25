use axum::Router;
use axum::routing::{get, post};
use color_eyre::eyre::Result;
use logging::trace_requests;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::services::{ServeDir, ServeFile};

pub mod api_handlers;
pub mod auth;
pub mod config;
pub mod db_ops;
pub mod file_storage;
pub mod logging;

pub use logging::init_logging;

use crate::config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    db: SqlitePool,
    config: AppConfig,
}

impl AppState {
    pub async fn new(config: AppConfig, db: SqlitePool) -> Result<Self> {
        Ok(Self { db, config })
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
                "/api/user/skip_envelope_verification",
                post(api_handlers::user::skip_envelope_verification),
            )
            .route(
                "/api/user/finalize_translation",
                post(api_handlers::user::finalize_translation),
            )
            .route(
                "/api/user/set_translation_session_token",
                post(api_handlers::user::set_translation_session_token),
            )
            .route(
                "/api/tasks/{task_id}/statement_versions/latest",
                get(api_handlers::statement_versions::get_latest_statement_version),
            )
            .route("/files/{hash}/{filename}", get(file_storage::get_file))
            .fallback_service(
                ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
            )
            .layer(axum::middleware::from_fn(trace_requests))
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
