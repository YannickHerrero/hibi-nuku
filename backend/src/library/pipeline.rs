//! End-to-end async import pipeline.
//!
//! Invoked from POST /api/library/import after the initial probe.
//! Drives the video through extracting → parsing → tokenizing →
//! translating, writing progress to `videos.status` at each step so
//! the frontend can poll. Any failure sets `status = error` and
//! `error_message` to the human-readable cause.

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use sqlx::SqlitePool;
use tracing::{error, info};

use crate::library::lines;
use crate::library::model::VideoStatus;
use crate::library::repo;
use crate::library::tracks;
use crate::subtitle;

/// Run the import pipeline for an already-persisted video row.
///
/// Errors are caught and persisted as `status = error`; only an
/// infrastructure failure (DB writes themselves failing) escapes.
pub async fn run(pool: Arc<SqlitePool>, video_id: i64) {
    if let Err(e) = drive(pool.clone(), video_id).await {
        error!(video_id, error = %e, "import pipeline failed");
        let _ = repo::set_status(&pool, video_id, VideoStatus::Error, Some(&e.to_string())).await;
    }
}

async fn drive(pool: Arc<SqlitePool>, video_id: i64) -> Result<()> {
    let video = repo::get(&pool, video_id)
        .await?
        .ok_or_else(|| anyhow!("video {video_id} vanished"))?;

    let Some(sub_idx) = video.jp_subtitle_idx else {
        return Err(anyhow!("no Japanese subtitle track on this file"));
    };
    let format = video
        .subtitle_format
        .as_deref()
        .ok_or_else(|| anyhow!("no subtitle format recorded"))?;

    if tracks::is_image_subtitle(format) {
        return Err(anyhow!(
            "image-based subtitles ({format}) not supported"
        ));
    }

    // 1. extracting + 2. parsing
    repo::set_status(&pool, video_id, VideoStatus::Extracting, None).await?;
    let path = Path::new(&video.path);
    let parsed = subtitle::extract::extract_and_parse(path, sub_idx as usize, format)
        .await
        .context("subtitle extraction")?;

    if parsed.is_empty() {
        return Err(anyhow!("subtitle track has 0 dialogue lines"));
    }

    repo::set_status(&pool, video_id, VideoStatus::Parsing, None).await?;
    lines::replace_for_video(&pool, video_id, &parsed)
        .await
        .context("persist parsed lines")?;

    info!(
        video_id,
        n_lines = parsed.len(),
        "subtitle parse complete"
    );

    // 3. tokenizing — Phase 4
    // 4. translating — Phase 5
    // Until those land, jump straight to ready.
    repo::set_status(&pool, video_id, VideoStatus::Ready, None).await?;
    Ok(())
}
