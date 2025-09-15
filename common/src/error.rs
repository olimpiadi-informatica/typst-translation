use thiserror::Error;

#[cfg(feature = "server-side")]
use axum::response::IntoResponse;
#[cfg(feature = "server-side")]
use axum::http::StatusCode;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    DbError(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    MigrationError(#[from] sqlx::migrate::MigrateError),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Login required")]
    LoginRequired,
    #[error("Login invalid")]
    LoginInvalidated,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not found")]
    NotFound,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Unauthorized change")]
    UnauthorizedChange,
    #[error("Internal server error")]
    InternalServerError,
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Url parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::Other(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::Other(value.to_string())
    }
}

#[cfg(feature = "server-side")]
impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Error::LoginRequired => StatusCode::UNAUTHORIZED,
            Error::LoginInvalidated => StatusCode::UNAUTHORIZED,
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Error::UnauthorizedChange => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}