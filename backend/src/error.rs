use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Not found")]
    NotFound,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized  => (StatusCode::UNAUTHORIZED,            self.to_string()),
            AppError::Forbidden     => (StatusCode::FORBIDDEN,               self.to_string()),
            AppError::NotFound      => (StatusCode::NOT_FOUND,               self.to_string()),
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST,             m.clone()),
            AppError::Conflict(m)   => (StatusCode::CONFLICT,                m.clone()),
            AppError::Internal(e)   => (StatusCode::INTERNAL_SERVER_ERROR,   format!("Internal: {e:#}")),
            AppError::Db(e)         => (StatusCode::INTERNAL_SERVER_ERROR,   format!("Database error: {e}")),
            AppError::Redis(e)      => (StatusCode::INTERNAL_SERVER_ERROR,   format!("Cache error: {e}")),
        };
        eprintln!("[ERROR] {status}: {message}");
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
