use axum::Router;
use axum::routing::post;
use color_eyre::eyre::Result;
use common::error::Error;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::services::{ServeDir, ServeFile};

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
    pub async fn new(config: AppConfig, database_url: &str) -> Result<Self> {
        let db = SqlitePoolOptions::new().connect(database_url).await?;

        sqlx::migrate!().run(&db).await?;

        Ok(Self { db, config })
    }

    pub fn db(&self) -> &SqlitePool {
        &self.db
    }

    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    pub async fn serve(self, listen_address: std::net::SocketAddr) -> Result<(), Error> {
        let app = self.app();

        let listener = tokio::net::TcpListener::bind(listen_address).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    pub fn app(self) -> Router {
        Router::new()
            .route("/api/login", post(auth::login))
            .route("/api/admin/login", post(auth::admin_login))
            .route("/api/staff/login", post(auth::staff_login))
            .route("/api/whoami", post(auth::whoami))
            .fallback_service(
                ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
            )
            .with_state(self)
    }
}
