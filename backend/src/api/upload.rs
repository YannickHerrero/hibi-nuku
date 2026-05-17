//! Browser-driven import: upload-media → probe, upload-subtitle,
//! create. Replaces the older path-based POST /api/library/import for
//! the UI flow.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Multipart, State};
use axum::routing::post;
use chrono::Utc;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, AppResult};
use crate::library::model::VideoStatus;
use crate::library::{pipeline, repo, tracks};
use crate::media;

use super::AppState;

const MAX_VIDEO_BYTES: usize = 10 * 1024 * 1024 * 1024; // 10 GB
const MAX_SUBTITLE_BYTES: usize = 8 * 1024 * 1024; // 8 MB

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/library/upload-media",
            post(upload_media).layer(DefaultBodyLimit::max(MAX_VIDEO_BYTES)),
        )
        .route(
            "/library/upload-subtitle",
            post(upload_subtitle).layer(DefaultBodyLimit::max(MAX_SUBTITLE_BYTES)),
        )
        .route("/library/create", post(create))
}

// ----------------- upload-media -----------------

#[derive(Serialize)]
struct UploadMediaResp {
    path: String,
    probe: ProbeJson,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProbeJson {
    duration_ms: i64,
    audio: Vec<TrackJson>,
    subtitle: Vec<TrackJson>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrackJson {
    index: usize,
    codec_name: String,
    language: Option<String>,
    title: Option<String>,
    is_image: bool,
}

async fn upload_media(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<UploadMediaResp>> {
    let uploads_dir = upload_dir(&state).await?;
    let mut saved_path: Option<PathBuf> = None;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }
        let original = field.file_name().unwrap_or("upload.bin").to_string();
        let safe = sanitize_filename(&original);
        let ts = Utc::now().format("%Y%m%d-%H%M%S");
        let path = uploads_dir.join(format!("{ts}-{safe}"));
        let mut file = fs::File::create(&path)
            .await
            .map_err(|e| AppError::Other(anyhow::anyhow!("create {}: {e}", path.display())))?;
        while let Some(chunk) = field.next().await {
            let chunk: Bytes = chunk.map_err(|e| AppError::BadRequest(format!("chunk: {e}")))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| AppError::Other(anyhow::anyhow!("write: {e}")))?;
        }
        file.flush().await.ok();
        saved_path = Some(path);
        break;
    }

    let path = saved_path.ok_or_else(|| AppError::BadRequest("missing `file` field".into()))?;

    let probe = media::probe(&path)
        .await
        .map_err(|e| AppError::BadRequest(format!("ffprobe: {e}")))?;

    let audio = probe
        .streams_of(media::StreamKind::Audio)
        .map(track_json)
        .collect();
    let subtitle = probe
        .streams_of(media::StreamKind::Subtitle)
        .map(track_json)
        .collect();

    Ok(Json(UploadMediaResp {
        path: path.to_string_lossy().into_owned(),
        probe: ProbeJson {
            duration_ms: probe.duration_ms,
            audio,
            subtitle,
        },
    }))
}

fn track_json(s: &media::Stream) -> TrackJson {
    TrackJson {
        index: s.index,
        codec_name: s.codec_name.clone(),
        language: s.language.clone(),
        title: s.title.clone(),
        is_image: tracks::is_image_subtitle(&s.codec_name),
    }
}

// ----------------- upload-subtitle -----------------

#[derive(Serialize)]
struct UploadSubtitleResp {
    path: String,
    format: String,
}

async fn upload_subtitle(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> AppResult<Json<UploadSubtitleResp>> {
    let uploads_dir = upload_dir(&state).await?;
    let mut saved: Option<(PathBuf, String)> = None;

    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name != "file" {
            continue;
        }
        let original = field.file_name().unwrap_or("subs.srt").to_string();
        let safe = sanitize_filename(&original);
        let format = subtitle_format(&safe)
            .ok_or_else(|| AppError::BadRequest("unsupported subtitle extension".into()))?;
        let ts = Utc::now().format("%Y%m%d-%H%M%S");
        let path = uploads_dir.join(format!("{ts}-{safe}"));
        let mut file = fs::File::create(&path)
            .await
            .map_err(|e| AppError::Other(anyhow::anyhow!("create {}: {e}", path.display())))?;
        while let Some(chunk) = field.next().await {
            let chunk: Bytes = chunk.map_err(|e| AppError::BadRequest(format!("chunk: {e}")))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| AppError::Other(anyhow::anyhow!("write: {e}")))?;
        }
        file.flush().await.ok();
        saved = Some((path, format.into()));
        break;
    }

    let (path, format) =
        saved.ok_or_else(|| AppError::BadRequest("missing `file` field".into()))?;
    Ok(Json(UploadSubtitleResp {
        path: path.to_string_lossy().into_owned(),
        format,
    }))
}

fn subtitle_format(name: &str) -> Option<&'static str> {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".srt") {
        Some("srt")
    } else if lower.ends_with(".ass") || lower.ends_with(".ssa") {
        Some("ass")
    } else if lower.ends_with(".vtt") {
        Some("vtt")
    } else {
        None
    }
}

// ----------------- create -----------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReq {
    pub path: String,
    pub title: Option<String>,
    pub source_tag: Option<String>,
    pub jp_audio_idx: Option<i64>,
    /// Either: embedded subtitle stream index (with format reported by
    /// probe) — or — sidecar path + format.
    pub jp_subtitle_idx: Option<i64>,
    pub subtitle_format: Option<String>,
    pub subtitle_sidecar_path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateResp {
    pub video_id: i64,
    pub status: VideoStatus,
}

async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateReq>,
) -> AppResult<Json<CreateResp>> {
    let abs = PathBuf::from(&req.path);
    if !abs.exists() {
        return Err(AppError::BadRequest(format!(
            "media path missing on disk: {}",
            abs.display()
        )));
    }

    // Re-probe so we have an authoritative duration (the upload-media
    // response is unsigned).
    let probe = media::probe(&abs)
        .await
        .map_err(|e| AppError::BadRequest(format!("ffprobe: {e}")))?;

    let stem = abs
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();
    let title = req.title.unwrap_or_else(|| stem.clone());
    let source_tag = req.source_tag.unwrap_or(stem);

    let subtitle_format = req.subtitle_format.clone();

    let new = repo::NewVideo {
        path: abs.to_string_lossy().into_owned(),
        title,
        source_tag,
        duration_ms: probe.duration_ms,
        jp_audio_idx: req.jp_audio_idx,
        jp_subtitle_idx: req.jp_subtitle_idx,
        subtitle_format,
        status: VideoStatus::Probing,
    };

    let id = repo::insert(&state.db, new)
        .await
        .map_err(|e| match e.downcast_ref::<sqlx::Error>() {
            Some(sqlx::Error::Database(db)) if db.is_unique_violation() => {
                AppError::Conflict("media already imported".into())
            }
            _ => AppError::Other(e),
        })?;

    if let Some(sc) = req.subtitle_sidecar_path.as_deref() {
        repo::set_subtitle_sidecar(&state.db, id, sc)
            .await
            .map_err(AppError::Other)?;
    }

    let pool = Arc::new(state.db.clone());
    tokio::spawn(pipeline::run(
        pool,
        state.jmdict.clone(),
        state.config.clone(),
        id,
    ));

    Ok(Json(CreateResp {
        video_id: id,
        status: VideoStatus::Probing,
    }))
}

// ----------------- helpers -----------------

async fn upload_dir(state: &AppState) -> AppResult<PathBuf> {
    let dir = state.config.library_dir.join("uploads");
    fs::create_dir_all(&dir)
        .await
        .map_err(|e| AppError::Other(anyhow::anyhow!("mkdir {}: {e}", dir.display())))?;
    Ok(dir)
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = Path::new(name)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "upload.bin".into())
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "upload.bin".into()
    } else {
        cleaned
    }
}
