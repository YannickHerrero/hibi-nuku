//! /api/debug — diagnostics for the /debug frontend page.
//!
//! All read endpoints are safe; destructive ones (cache wipes) live
//! behind explicit DELETE methods.

use std::time::Instant;

use axum::Json;
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::{delete, get};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{AppError, AppResult};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/debug/hibi-status", get(hibi_status))
        .route("/debug/db-stats", get(db_stats))
        .route("/debug/llm-cache", delete(wipe_llm_cache))
}

// ---------------- Hibi pinger ----------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HibiProbe {
    method: &'static str,
    path: &'static str,
    /// `None` if the request itself errored (network / timeout).
    status: Option<u16>,
    latency_ms: u128,
    /// First 400 chars of the response body (when we have one).
    body_snippet: Option<String>,
    ok: bool,
}

#[derive(Serialize)]
struct HibiStatusResp {
    base: String,
    probes: Vec<HibiProbe>,
}

async fn hibi_status(State(state): State<AppState>) -> AppResult<Json<HibiStatusResp>> {
    let base = state.config.hibi_base.clone();
    let key = state.config.hibi_api_key.clone();

    // All probes use Bearer API-key auth. /v1/account/* needs a
    // session cookie instead, so we don't probe it here.
    let probes = vec![
        probe(&base, &key, "GET", "/v1/known-words").await,
        probe(&base, &key, "GET", "/v1/cards?limit=1").await,
        probe(&base, &key, "GET", "/v1/reviews/due?limit=1").await,
        probe(&base, &key, "GET", "/v1/stats/heatmap").await,
    ];

    Ok(Json(HibiStatusResp { base, probes }))
}

async fn probe(base: &str, key: &str, method: &'static str, path: &'static str) -> HibiProbe {
    let url = format!("{base}{path}");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("reqwest");
    let started = Instant::now();
    let req = match method {
        "GET" => client.get(&url),
        _ => unreachable!("only GET probes for now"),
    };
    let result = req.bearer_auth(key).send().await;
    let latency_ms = started.elapsed().as_millis();
    match result {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let ok = resp.status().is_success();
            let body = resp.text().await.unwrap_or_default();
            HibiProbe {
                method,
                path,
                status: Some(status),
                latency_ms,
                body_snippet: Some(truncate(&body, 400)),
                ok,
            }
        }
        Err(e) => HibiProbe {
            method,
            path,
            status: None,
            latency_ms,
            body_snippet: Some(format!("network: {e}")),
            ok: false,
        },
    }
}

// ---------------- DB stats ----------------

#[derive(Serialize)]
struct DbStats {
    db_path: String,
    db_file_bytes: u64,
    tables: serde_json::Value,
}

async fn db_stats(State(state): State<AppState>) -> AppResult<Json<DbStats>> {
    let db_path = state.config.db_path.to_string_lossy().into_owned();
    let db_file_bytes = std::fs::metadata(&state.config.db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let tables = [
        "videos",
        "subtitle_lines",
        "video_progress",
        "wk_kanji",
        "wk_vocab",
        "wk_kanji_to_vocab",
        "wk_meta",
        "llm_cache",
        "dict_versions",
    ];
    let mut counts = serde_json::Map::new();
    for t in tables {
        let n: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {t}"))
            .fetch_one(&state.db)
            .await
            .unwrap_or(-1);
        counts.insert(t.into(), json!(n));
    }

    Ok(Json(DbStats {
        db_path,
        db_file_bytes,
        tables: serde_json::Value::Object(counts),
    }))
}

// ---------------- llm cache wipe ----------------

#[derive(Deserialize)]
struct WipeQuery {
    /// Optional model filter; when omitted, wipes the whole table.
    model: Option<String>,
}

#[derive(Serialize)]
struct WipeResp {
    deleted: u64,
}

async fn wipe_llm_cache(
    State(state): State<AppState>,
    Query(q): Query<WipeQuery>,
) -> AppResult<Json<WipeResp>> {
    let res = if let Some(m) = q.model.as_deref() {
        sqlx::query("DELETE FROM llm_cache WHERE model = ?")
            .bind(m)
            .execute(&state.db)
            .await
    } else {
        sqlx::query("DELETE FROM llm_cache")
            .execute(&state.db)
            .await
    };
    let res = res.map_err(|e| AppError::Other(e.into()))?;
    Ok(Json(WipeResp {
        deleted: res.rows_affected(),
    }))
}

// ---------------- helpers ----------------

fn truncate(s: &str, n: usize) -> String {
    if s.len() > n {
        format!("{}…", &s[..n])
    } else {
        s.to_string()
    }
}

