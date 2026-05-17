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
    let header_tok = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_owned());

    let query_tok = req
        .uri()
        .query()
        .and_then(|q| {
            q.split('&').find_map(|kv| {
                let (k, v) = kv.split_once('=')?;
                if k == "token" {
                    Some(
                        percent_encoding::percent_decode_str(v)
                            .decode_utf8_lossy()
                            .into_owned(),
                    )
                } else {
                    None
                }
            })
        });

    let presented = header_tok.or(query_tok).ok_or(AppError::Unauthorised)?;

    let expected = state.config.token.as_bytes();
    if expected.ct_eq(presented.as_bytes()).into() {
        Ok(next.run(req).await)
    } else {
        Err(AppError::Unauthorised)
    }
}
