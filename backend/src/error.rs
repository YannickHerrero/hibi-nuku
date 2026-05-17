#![allow(dead_code)]

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Single application error type that converts into a JSON response.
///
/// Specific failure modes carry HTTP status; `Other` wraps arbitrary
/// `anyhow::Error` for one-off internal failures.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("unauthorised")]
    Unauthorised,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorised => StatusCode::UNAUTHORIZED,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let msg = self.to_string();
        if status.is_server_error() {
            tracing::error!(error = %msg, "internal error");
        } else if matches!(status, StatusCode::BAD_REQUEST | StatusCode::CONFLICT) {
            // Visible at default info level so misuse surfaces in
            // dev / prod logs without needing a debug filter.
            tracing::warn!(error = %msg, status = %status, "client error");
        } else {
            tracing::debug!(error = %msg, status = %status, "client error");
        }
        (status, Json(ErrorBody { error: &msg })).into_response()
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;
