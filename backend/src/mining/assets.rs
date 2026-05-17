//! Per-line audio + still-frame extraction via ffmpeg.

use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result, anyhow};
use tokio::process::Command;

/// Extract `[start_ms, end_ms]` from `media` as AAC m4a, audio-only,
/// 128 kbps. Returns the temp file path; caller is responsible for
/// reading + deleting it.
pub async fn extract_audio(
    media: &Path,
    audio_idx: Option<i64>,
    start_ms: i64,
    end_ms: i64,
    out: &Path,
) -> Result<()> {
    let start = (start_ms as f64) / 1000.0;
    let end = (end_ms as f64) / 1000.0;
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-y", "-loglevel", "error"]);
    cmd.arg("-ss").arg(format!("{start}"));
    cmd.arg("-to").arg(format!("{end}"));
    cmd.arg("-i").arg(media);
    if let Some(idx) = audio_idx {
        cmd.arg("-map").arg(format!("0:{idx}"));
    } else {
        cmd.arg("-map").arg("0:a:0?");
    }
    cmd.arg("-vn");
    cmd.arg("-c:a").arg("aac").arg("-b:a").arg("128k");
    cmd.arg("-movflags").arg("+faststart");
    cmd.arg(out);

    let output = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("spawn ffmpeg audio")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffmpeg audio failed: {}", stderr.trim()));
    }
    Ok(())
}

/// Snap one video frame at the midpoint of the line.
pub async fn extract_screenshot(media: &Path, mid_ms: i64, out: &Path) -> Result<()> {
    crate::media::thumb::extract(media, (mid_ms as f64) / 1000.0, out).await
}
