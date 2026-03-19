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
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
    #[error("Database error")]
    Db(#[from] sqlx::Error),
    #[error("Redis error")]
    Redis(#[from] redis::RedisError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized    => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden       => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::NotFound        => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::BadRequest(m)   => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Conflict(m)     => (StatusCode::CONFLICT, m.clone()),
            AppError::Internal(_)     => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
            AppError::Db(_)           => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            AppError::Redis(_)        => (StatusCode::INTERNAL_SERVER_ERROR, "Cache error".to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
