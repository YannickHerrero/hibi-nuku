//! /api/dict/{manifest,jmdict,wk,frequency} — manifest and gzipped
//! bundle serving.

use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::get;

use crate::dict::manifest::{Manifest, compute};
use crate::error::{AppError, AppResult};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/dict/manifest", get(manifest))
        .route("/dict/{name}", get(bundle))
}

async fn manifest(State(state): State<AppState>) -> AppResult<Json<Manifest>> {
    let m = compute(&state.config.data_dir).map_err(AppError::Other)?;
    Ok(Json(m))
}

async fn bundle(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> AppResult<Response> {
    let file = match name.as_str() {
        "jmdict" => "jmdict.json.gz",
        "wk" => "wk.json.gz",
        "frequency" => "frequency.json.gz",
        _ => return Err(AppError::NotFound),
    };
    let path = state.config.data_dir.join(file);
    if !path.exists() {
        return Err(AppError::NotFound);
    }
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| AppError::Other(anyhow::anyhow!("read {}: {e}", path.display())))?;
    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("application/json"))
        .header(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"))
        .header(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=3600"),
        )
        .body(Body::from(bytes))
        .unwrap();
    Ok(resp)
}
