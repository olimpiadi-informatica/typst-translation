use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize, PartialEq)]
pub enum Error {
    #[cfg(feature = "client-side")]
    #[error("Error communicating with server: {0}")]
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
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Translation budget exhausted")]
    TranslationBudgetExhausted,
    #[error("Internal server error: {0}")]
    InternalServerError(String),
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
impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::Json;
        use axum::http::StatusCode;
        let status = match self {
            Error::LoginRequired => StatusCode::UNAUTHORIZED,
            Error::LoginInvalidated => StatusCode::UNAUTHORIZED,
            Error::Forbidden => StatusCode::FORBIDDEN,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

#[cfg(feature = "server-side")]
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        tracing::warn!(error = ?err, "SQLx error");
        Error::InternalServerError(format!("{err}"))
    }
}

#[cfg(feature = "server-side")]
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        tracing::warn!(error = ?err, "Reqwest error");
        Error::InternalServerError(format!("{err}"))
    }
}

#[cfg(feature = "server-side")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        tracing::warn!(error = ?err, "I/O error");
        Error::InternalServerError(format!("{err}"))
    }
}

#[cfg(feature = "client-side")]
impl From<gloo_net::Error> for Error {
    fn from(err: gloo_net::Error) -> Self {
        Error::NetworkError(format!("{err}"))
    }
}

#[cfg(feature = "client-side")]
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::NetworkError(format!("{err}"))
    }
}
