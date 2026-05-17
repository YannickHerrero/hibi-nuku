//! GET /api/settings — read-only snapshot of non-secret config.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::get;
use serde::Serialize;
use sqlx::Row;

use crate::dict::manifest::{Manifest, compute};
use crate::error::{AppError, AppResult};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/settings", get(settings))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    llm_model: String,
    wk_user_level: Option<i64>,
    wk_username: Option<String>,
    library_dir: String,
    dict: Manifest,
}

async fn settings(State(state): State<AppState>) -> AppResult<Json<Settings>> {
    let row =
        sqlx::query("SELECT key, value FROM wk_meta WHERE key IN ('user_level', 'username')")
            .fetch_all(&state.db)
            .await
            .map_err(|e| AppError::Other(e.into()))?;
    let mut wk_user_level: Option<i64> = None;
    let mut wk_username: Option<String> = None;
    for r in row {
        let k: String = r.try_get("key").unwrap_or_default();
        let v: String = r.try_get("value").unwrap_or_default();
        match k.as_str() {
            "user_level" => wk_user_level = v.parse().ok(),
            "username" => wk_username = Some(v),
            _ => {}
        }
    }
    let dict = compute(&state.config.data_dir).map_err(AppError::Other)?;
    Ok(Json(Settings {
        llm_model: state.config.llm_model.clone(),
        wk_user_level,
        wk_username,
        library_dir: state.config.library_dir.to_string_lossy().into_owned(),
        dict,
    }))
}
