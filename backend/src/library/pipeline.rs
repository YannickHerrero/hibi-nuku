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

use crate::config::Config;
use crate::library::lines;
use crate::subtitle::{ass, srt};
use crate::library::model::VideoStatus;
use crate::library::repo;
use crate::library::tracks;
use crate::llm::client::OpenRouter;
use crate::llm::translate;
use crate::media::{remux, thumb};
use crate::subtitle;
use crate::tokenize::jmdict_index::JmdictIndex;
use crate::tokenize::{lindera_wrap, segment};

async fn parse_sidecar(path_s: &str) -> Result<Vec<crate::subtitle::Line>> {
    let path = Path::new(path_s);
    let body = tokio::fs::read_to_string(path)
        .await
        .with_context(|| format!("read sidecar {path_s}"))?;
    let lower = path_s.to_ascii_lowercase();
    if lower.ends_with(".ass") || lower.ends_with(".ssa") {
        ass::parse(&body)
    } else {
        srt::parse(&body)
    }
}

/// Run the import pipeline for an already-persisted video row.
///
/// Errors are caught and persisted as `status = error`; only an
/// infrastructure failure (DB writes themselves failing) escapes.
pub async fn run(
    pool: Arc<SqlitePool>,
    jmdict: Arc<JmdictIndex>,
    config: Arc<Config>,
    video_id: i64,
) {
    if let Err(e) = drive(pool.clone(), jmdict, config, video_id).await {
        // {:#} joins anyhow's context chain so we see the real cause,
        // not just the topmost `.context("...")` wrapper.
        let full = format!("{e:#}");
        error!(video_id, error = %full, "import pipeline failed");
        let _ = repo::set_status(&pool, video_id, VideoStatus::Error, Some(&full)).await;
    }
}

async fn drive(
    pool: Arc<SqlitePool>,
    jmdict: Arc<JmdictIndex>,
    config: Arc<Config>,
    video_id: i64,
) -> Result<()> {
    let video = repo::get(&pool, video_id)
        .await?
        .ok_or_else(|| anyhow!("video {video_id} vanished"))?;

    let sidecar = repo::get_subtitle_sidecar(&pool, video_id).await?;

    // If a sidecar file is set, it wins. Otherwise we need an embedded
    // subtitle track index + format.
    let (sub_idx, format) = if sidecar.is_some() {
        (None::<i64>, video.subtitle_format.clone().unwrap_or_default())
    } else {
        let idx = video
            .jp_subtitle_idx
            .ok_or_else(|| anyhow!("no Japanese subtitle track or sidecar"))?;
        let fmt = video
            .subtitle_format
            .as_deref()
            .ok_or_else(|| anyhow!("no subtitle format recorded"))?
            .to_string();
        (Some(idx), fmt)
    };

    if !format.is_empty() && tracks::is_image_subtitle(&format) {
        return Err(anyhow!(
            "image-based subtitles ({format}) not supported"
        ));
    }

    // 1. extracting + 2. parsing
    repo::set_status(&pool, video_id, VideoStatus::Extracting, None).await?;
    let path = Path::new(&video.path);

    // Best-effort thumbnail at 30% of the duration; failure is non-fatal.
    let thumb_dir = config.data_dir.join("thumbs");
    if let Err(e) = tokio::fs::create_dir_all(&thumb_dir).await {
        tracing::warn!(error = %e, "thumb dir create");
    }
    let thumb_path = thumb_dir.join(format!("{video_id}.webp"));
    let at_sec = (video.duration_ms as f64 / 1000.0) * 0.30;
    if let Err(e) = thumb::extract(path, at_sec, &thumb_path).await {
        tracing::warn!(error = %e, "thumbnail extract failed; continuing");
    } else if let Err(e) =
        repo::set_thumbnail_path(&pool, video_id, &thumb_path.to_string_lossy()).await
    {
        tracing::warn!(error = %e, "thumbnail path persist failed");
    }

    let parsed = if let Some(sidecar_path) = sidecar.as_deref() {
        info!(video_id, sidecar = sidecar_path, "using sidecar subtitle");
        parse_sidecar(sidecar_path).await.context("parse sidecar subtitle")?
    } else {
        let idx = sub_idx.expect("sub_idx set when no sidecar");
        subtitle::extract::extract_and_parse(path, idx as usize, &format)
            .await
            .context("subtitle extraction")?
    };

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

    // 3. tokenizing
    repo::set_status(&pool, video_id, VideoStatus::Tokenizing, None).await?;
    let persisted = lines::list_for_video(&pool, video_id)
        .await
        .context("re-read persisted lines")?;
    for line in &persisted {
        let lindera = match lindera_wrap::tokenize(&line.raw_text) {
            Ok(v) => v,
            Err(e) => {
                error!(line_id = line.id, error = %e, "lindera failed; skipping line");
                continue;
            }
        };
        let tokens = segment::segment(&line.raw_text, &lindera, jmdict.as_ref());
        let json = serde_json::to_string(&tokens).context("serialise tokens")?;
        lines::set_tokens(&pool, line.id, &json).await?;
    }

    info!(video_id, "tokenization complete");

    // 4. translating
    repo::set_status(&pool, video_id, VideoStatus::Translating, None).await?;
    let client = OpenRouter::new(config.openrouter_api_key.clone());
    let n = translate::translate_lines(pool.clone(), &client, &config.llm_model, video_id)
        .await
        .context("translate lines")?;
    info!(video_id, n_translated = n, "translation complete");

    // 5. remuxing — one-shot ffmpeg pass into a faststart MP4 sibling.
    // Lets the stream endpoint serve a real file with native HTTP
    // Range support, so scrubbing + subtitle nav are buffer-free.
    // Best-effort: failure logs a warning and leaves the row Ready,
    // falling back to the on-the-fly ffmpeg pipe at request time.
    repo::set_status(&pool, video_id, VideoStatus::Remuxing, None).await?;
    let audio_codec = probe_audio_codec(path, video.jp_audio_idx).await;
    match remux::remux(path, video.jp_audio_idx, &audio_codec).await {
        Ok(out) => {
            repo::set_remuxed_path(&pool, video_id, &out.to_string_lossy()).await?;
            info!(video_id, path = %out.display(), "remux complete");
        }
        Err(e) => {
            tracing::warn!(video_id, error = %format!("{e:#}"), "remux failed; falling back to live pipe");
        }
    }

    repo::set_status(&pool, video_id, VideoStatus::Ready, None).await?;
    Ok(())
}

async fn probe_audio_codec(path: &Path, audio_idx: Option<i64>) -> String {
    let probe = match crate::media::probe(path).await {
        Ok(p) => p,
        Err(_) => return String::new(),
    };
    probe
        .streams
        .iter()
        .find(|s| match audio_idx {
            Some(idx) => s.index as i64 == idx,
            None => matches!(s.codec_type, crate::media::StreamKind::Audio),
        })
        .map(|s| s.codec_name.clone())
        .unwrap_or_default()
}
