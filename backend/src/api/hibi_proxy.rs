//! Thin proxy for Hibi endpoints we forward (known-words, word-status,
//! sessions). known-words is cached server-side for 60s to avoid
//! hammering Hibi during a watch.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::{get, post, put};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::hibi::client::Hibi;
use crate::hibi::model::{KnownWord, KnownWordsResp, SessionRequest, WordStatusReq};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/known-words", get(known_words))
        .route("/word-status", put(word_status))
        .route("/sessions", post(sessions))
}

/// Process-wide 60-second cache of the Hibi known-words list.
#[derive(Default)]
pub struct KnownCache {
    inner: Mutex<Option<CachedKnown>>,
}

struct CachedKnown {
    fetched_at: Instant,
    items: Vec<KnownWord>,
}

impl KnownCache {
    pub fn new() -> Self {
        Self::default()
    }
}

async fn known_words(State(state): State<AppState>) -> AppResult<Json<KnownWordsResp>> {
    const TTL: Duration = Duration::from_secs(60);
    {
        let guard = state.known_cache.inner.lock().await;
        if let Some(c) = &*guard {
            if c.fetched_at.elapsed() < TTL {
                return Ok(Json(KnownWordsResp { items: c.items.clone() }));
            }
        }
    }
    let hibi = Hibi::new(state.config.hibi_base.clone(), state.config.hibi_api_key.clone());
    let resp = hibi.known_words().await.map_err(AppError::Other)?;
    let mut guard = state.known_cache.inner.lock().await;
    *guard = Some(CachedKnown {
        fetched_at: Instant::now(),
        items: resp.items.clone(),
    });
    Ok(Json(resp))
}

async fn word_status(
    State(state): State<AppState>,
    Json(req): Json<WordStatusReq>,
) -> AppResult<Json<Value>> {
    let hibi = Hibi::new(state.config.hibi_base.clone(), state.config.hibi_api_key.clone());
    let v = hibi.put_word_status(&req).await.map_err(AppError::Other)?;
    // Invalidate the known-words cache so the next poll reflects the change.
    let mut guard = state.known_cache.inner.lock().await;
    *guard = None;
    Ok(Json(v))
}

async fn sessions(
    State(state): State<AppState>,
    Json(req): Json<SessionRequest>,
) -> AppResult<Json<Value>> {
    let hibi = Hibi::new(state.config.hibi_base.clone(), state.config.hibi_api_key.clone());
    let v = hibi.create_session(&req).await.map_err(AppError::Other)?;
    Ok(Json(v))
}

// Keep this type alias to avoid using Arc directly in AppState; the
// state struct lives in api/mod.rs.
pub type SharedKnownCache = Arc<KnownCache>;
