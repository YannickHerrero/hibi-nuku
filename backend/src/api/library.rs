use std::path::PathBuf;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::library::model::{Video, VideoStatus};
use crate::library::repo;
use crate::library::tracks;
use crate::media;

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/library/import", post(import))
        .route("/library", get(list))
        .route("/videos/{id}", get(by_id).patch(patch_video).delete(remove))
        .route("/videos/{id}/reprocess", post(reprocess))
}

// duplicate handler is fine; the .route().patch() above is just for clarity
async fn patch_video(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PatchVideoReq>,
) -> AppResult<Json<Video>> {
    patch_inner(&state, id, body).await
}

#[derive(Deserialize)]
struct ImportReq {
    path: String,
    #[serde(default)]
    source_tag: Option<String>,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Serialize)]
struct ImportResp {
    video_id: i64,
    status: VideoStatus,
}

async fn import(
    State(state): State<AppState>,
    Json(req): Json<ImportReq>,
) -> AppResult<Json<ImportResp>> {
    let raw = PathBuf::from(&req.path);
    let abs = raw.canonicalize().map_err(|e| {
        AppError::BadRequest(format!("path not found: {}: {e}", raw.display()))
    })?;

    let library_dir = state
        .config
        .library_dir
        .canonicalize()
        .unwrap_or_else(|_| state.config.library_dir.clone());
    if !abs.starts_with(&library_dir) {
        return Err(AppError::BadRequest(format!(
            "path must live under library dir {}",
            library_dir.display()
        )));
    }

    let probe = media::probe(&abs)
        .await
        .map_err(|e| AppError::BadRequest(format!("ffprobe failed: {e}")))?;
    let sel = tracks::select_japanese(&probe.streams);

    if let Some(fmt) = sel.subtitle_format.as_deref() {
        if tracks::is_image_subtitle(fmt) {
            return Err(AppError::BadRequest(format!(
                "image-based subtitle ({fmt}) not supported"
            )));
        }
    }

    let stem = abs
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled")
        .to_string();
    let title = req.title.unwrap_or_else(|| stem.clone());
    let source_tag = req.source_tag.unwrap_or(stem);

    let new = repo::NewVideo {
        path: abs.to_string_lossy().into_owned(),
        title,
        source_tag,
        duration_ms: probe.duration_ms,
        jp_audio_idx: sel.audio_idx.map(|x| x as i64),
        jp_subtitle_idx: sel.subtitle_idx.map(|x| x as i64),
        subtitle_format: sel.subtitle_format.clone(),
        status: VideoStatus::Probing,
    };

    let id = repo::insert(&state.db, new)
        .await
        .map_err(|e| match e.downcast_ref::<sqlx::Error>() {
            Some(sqlx::Error::Database(db)) if db.is_unique_violation() => {
                AppError::Conflict("video already imported".into())
            }
            _ => AppError::Other(e),
        })?;

    // Phase 3+: actual pipeline kicks off here. For now mark the
    // video ready-with-no-content so the row is consumable.
    repo::set_status(&state.db, id, VideoStatus::Ready, None)
        .await
        .map_err(AppError::Other)?;

    Ok(Json(ImportResp {
        video_id: id,
        status: VideoStatus::Ready,
    }))
}

async fn list(State(state): State<AppState>) -> AppResult<Json<Vec<Video>>> {
    let videos = repo::list(&state.db).await.map_err(AppError::Other)?;
    Ok(Json(videos))
}

async fn by_id(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<Video>> {
    let video = repo::get(&state.db, id)
        .await
        .map_err(AppError::Other)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(video))
}

async fn remove(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<RemoveResp>> {
    let deleted = repo::delete(&state.db, id).await.map_err(AppError::Other)?;
    if !deleted {
        return Err(AppError::NotFound);
    }
    Ok(Json(RemoveResp { deleted: true }))
}

#[derive(Serialize)]
struct RemoveResp {
    deleted: bool,
}

#[derive(Deserialize)]
struct PatchVideoReq {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    source_tag: Option<String>,
}

async fn patch_inner(
    state: &AppState,
    id: i64,
    req: PatchVideoReq,
) -> AppResult<Json<Video>> {
    if req.title.is_none() && req.source_tag.is_none() {
        return Err(AppError::BadRequest(
            "specify at least one of title, source_tag".into(),
        ));
    }
    let updated = repo::update_metadata(
        &state.db,
        id,
        req.title.as_deref(),
        req.source_tag.as_deref(),
    )
    .await
    .map_err(AppError::Other)?;
    if !updated {
        return Err(AppError::NotFound);
    }
    let video = repo::get(&state.db, id)
        .await
        .map_err(AppError::Other)?
        .ok_or(AppError::NotFound)?;
    Ok(Json(video))
}

#[derive(Serialize)]
struct ReprocessResp {
    video_id: i64,
    status: VideoStatus,
}

async fn reprocess(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> AppResult<Json<ReprocessResp>> {
    let video = repo::get(&state.db, id)
        .await
        .map_err(AppError::Other)?
        .ok_or(AppError::NotFound)?;
    // Phase 3+: re-run the pipeline. For now this just resets status.
    repo::set_status(&state.db, video.id, VideoStatus::Probing, None)
        .await
        .map_err(AppError::Other)?;
    Ok(Json(ReprocessResp {
        video_id: video.id,
        status: VideoStatus::Probing,
    }))
}
