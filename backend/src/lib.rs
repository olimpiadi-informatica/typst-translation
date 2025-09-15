use axum::Router;
use axum::routing::post;
use common::error::Error;
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::services::{ServeDir, ServeFile};

pub mod auth;
pub mod db_ops;
pub mod file_storage;
pub mod logging;

pub use logging::init_logging;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

impl AppState {
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let db = SqlitePoolOptions::new().connect(database_url).await?;

        sqlx::migrate!().run(&db).await?;

        Ok(Self { db })
    }

    pub fn db(&self) -> &SqlitePool {
        &self.db
    }

    pub async fn serve(self, listen_address: std::net::SocketAddr) -> Result<(), Error> {
        let app = Router::new()
            .route("/api/login", post(auth::login))
            .fallback_service(
                ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
            )
            .with_state(self);

        let listener = tokio::net::TcpListener::bind(listen_address).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
