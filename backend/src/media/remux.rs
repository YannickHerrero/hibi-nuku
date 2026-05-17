//! One-shot remux: read source file, write a faststart MP4 sibling
//! with H.264 video stream-copied and audio re-encoded to AAC if it
//! isn't already MP4-compatible. Runs once during the import
//! pipeline; the resulting file is then served with native HTTP
//! Range support.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result, anyhow};
use tokio::process::Command;

use crate::media::stream::{AudioPlan, audio_plan};

/// Returns the path the remuxed MP4 will live at — same dir as the
/// source, with `.remuxed.mp4` appended to the stem.
pub fn remuxed_path_for(source: &Path) -> PathBuf {
    let stem = source.file_stem().and_then(|s| s.to_str()).unwrap_or("video");
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{stem}.remuxed.mp4"))
}

/// Remux the source file. Returns the output path on success.
/// Idempotent: if the output already exists and is newer than the
/// source, returns immediately.
pub async fn remux(
    source: &Path,
    audio_idx: Option<i64>,
    audio_codec: &str,
) -> Result<PathBuf> {
    let out = remuxed_path_for(source);

    // Skip if output is already up-to-date.
    if let (Ok(src_meta), Ok(out_meta)) = (std::fs::metadata(source), std::fs::metadata(&out)) {
        if let (Ok(src_mtime), Ok(out_mtime)) = (src_meta.modified(), out_meta.modified()) {
            if out_mtime >= src_mtime && out_meta.len() > 0 {
                return Ok(out);
            }
        }
    }

    let plan = audio_plan(audio_codec);
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-y", "-loglevel", "error"]);
    cmd.arg("-i").arg(source);

    // Map video stream 0; map the chosen JP audio if known, else first audio.
    cmd.arg("-map").arg("0:v:0");
    if let Some(idx) = audio_idx {
        cmd.arg("-map").arg(format!("0:{idx}"));
    } else {
        cmd.arg("-map").arg("0:a:0?");
    }

    cmd.arg("-c:v").arg("copy");
    match plan {
        AudioPlan::Copy => {
            cmd.arg("-c:a").arg("copy");
        }
        AudioPlan::Aac => {
            cmd.arg("-c:a").arg("aac").arg("-b:a").arg("160k");
        }
    }

    // `+faststart` moves the moov atom to the head so byte-range
    // playback can begin before the whole file is fetched.
    cmd.arg("-movflags").arg("+faststart");
    cmd.arg("-f").arg("mp4");
    cmd.arg(&out);

    let output = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("spawn ffmpeg remux")?;

    if !output.status.success() {
        let _ = std::fs::remove_file(&out);
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffmpeg remux failed: {}", stderr.trim()));
    }
    Ok(out)
}
