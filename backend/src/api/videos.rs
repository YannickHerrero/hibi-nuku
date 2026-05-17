//! Routes for streaming media, serving extracted subtitles, thumbnails,
//! and per-video playback progress.

use std::path::PathBuf;

use anyhow::Context;
use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::routing::get;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tokio_util::io::ReaderStream;

use crate::error::{AppError, AppResult};
use crate::library::lines;
use crate::library::repo;
use crate::media::stream as mstream;
use crate::media::stream::AudioPlan;

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/videos/{id}/stream", get(stream))
        .route("/videos/{id}/subtitles", get(subtitles))
        .route("/videos/{id}/thumbnail", get(thumbnail))
        .route("/videos/{id}/progress", get(get_progress).post(post_progress))
}

#[derive(Deserialize)]
struct StreamQuery {
    /// Seconds to seek to before streaming.
    #[serde(default)]
    from: Option<f64>,
}

async fn stream(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(q): Query<StreamQuery>,
) -> AppResult<Response> {
    let video = repo::get(&state.db, id)
        .await
        .map_err(AppError::Other)?
        .ok_or(AppError::NotFound)?;

    let path: PathBuf = video.path.into();
    if !path.exists() {
        return Err(AppError::BadRequest(format!(
            "file missing on disk: {}",
            path.display()
        )));
    }

    let audio_idx = video.jp_audio_idx;
    // Heuristic: use ffprobe codec name from the jp track if known.
    let plan = pick_plan(&path, audio_idx).await;

    let mut s = mstream::spawn(&path, audio_idx, plan, q.from).map_err(AppError::Other)?;
    let stdout = s
        .stdout()
        .ok_or_else(|| AppError::Other(anyhow::anyhow!("ffmpeg stdout missing")))?;
    let body = Body::from_stream(ReaderStream::new(stdout));

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static("video/mp4"))
        .header(header::CACHE_CONTROL, HeaderValue::from_static("no-store"))
        .body(body)
        .map_err(|e| AppError::Other(anyhow::anyhow!("response build: {e}")))?;

    Ok(resp)
}

async fn pick_plan(path: &std::path::Path, audio_idx: Option<i64>) -> AudioPlan {
    let probe = match crate::media::probe(path).await {
        Ok(p) => p,
        Err(_) => return AudioPlan::Aac,
    };
    let codec = probe
        .streams
        .iter()
        .find(|s| match audio_idx {
            Some(idx) => s.index as i64 == idx,
            None => matches!(s.codec_type, crate::media::StreamKind::Audio),
        })
        .map(|s| s.codec_name.as_str())
        .unwrap_or("");
    mstream::audio_plan(codec)
}

async fn subtitles(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<Vec<lines::SubtitleLine>>> {
    let lines = lines::list_for_video(&state.db, id)
        .await
        .map_err(AppError::Other)?;
    Ok(Json(lines))
}

async fn thumbnail(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Response> {
    let row = sqlx::query("SELECT thumbnail_path FROM videos WHERE id = ?")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Other(e.into()))?
        .ok_or(AppError::NotFound)?;
    let path: Option<String> = row.try_get("thumbnail_path").ok();
    let Some(path) = path else {
        return Err(AppError::NotFound);
    };
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| AppError::Other(anyhow::anyhow!("read thumb {path}: {e}")))?;

    let ct = if path.ends_with(".webp") {
        "image/webp"
    } else if path.ends_with(".png") {
        "image/png"
    } else {
        "image/jpeg"
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, HeaderValue::from_static(ct))
        .header(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=86400"),
        )
        .body(Body::from(bytes))
        .unwrap())
}

#[derive(Deserialize)]
struct ProgressBody {
    position_ms: i64,
    #[serde(default)]
    device: Option<String>,
}

#[derive(Serialize)]
struct ProgressResp {
    position_ms: i64,
    last_watched_at: Option<String>,
    last_device: Option<String>,
}

async fn post_progress(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ProgressBody>,
) -> AppResult<Json<ProgressResp>> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO video_progress (video_id, position_ms, last_watched_at, last_device)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(video_id) DO UPDATE SET
            position_ms = excluded.position_ms,
            last_watched_at = excluded.last_watched_at,
            last_device = excluded.last_device",
    )
    .bind(id)
    .bind(body.position_ms)
    .bind(&now)
    .bind(body.device.as_deref())
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Other(e.into()))?;

    Ok(Json(ProgressResp {
        position_ms: body.position_ms,
        last_watched_at: Some(now),
        last_device: body.device,
    }))
}

async fn get_progress(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<ProgressResp>> {
    let row = sqlx::query(
        "SELECT position_ms, last_watched_at, last_device FROM video_progress WHERE video_id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Other(e.into()))?;

    let Some(row) = row else {
        return Ok(Json(ProgressResp {
            position_ms: 0,
            last_watched_at: None,
            last_device: None,
        }));
    };
    Ok(Json(ProgressResp {
        position_ms: row
            .try_get("position_ms")
            .context("position_ms")
            .map_err(AppError::Other)?,
        last_watched_at: row.try_get("last_watched_at").ok(),
        last_device: row.try_get("last_device").ok(),
    }))
}
