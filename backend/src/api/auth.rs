use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;
use subtle::ConstantTimeEq;

use crate::error::AppError;

use super::AppState;

/// Bearer-token gate for /api/* (except /api/health).
///
/// The bearer is a static value loaded from `NUKU_TOKEN`. Single user
/// behind Tailscale; this is defense-in-depth, not real auth.
pub async fn require_bearer(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::Unauthorised)?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or(AppError::Unauthorised)?;

    let expected = state.config.token.as_bytes();
    if expected.ct_eq(token.as_bytes()).into() {
        Ok(next.run(req).await)
    } else {
        Err(AppError::Unauthorised)
    }
}
