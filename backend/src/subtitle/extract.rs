//! Extract a subtitle track from a media file via ffmpeg.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{Context, Result, anyhow};
use tempfile::NamedTempFile;
use tokio::process::Command;

use super::Line;
use super::{ass, srt};

/// Subtitle stream codec_name as reported by ffprobe, mapped to the
/// container we'll ask ffmpeg to remux into.
pub fn ext_for_format(format: &str) -> &'static str {
    match format {
        "ass" | "ssa" => "ass",
        "subrip" | "srt" => "srt",
        "webvtt" | "vtt" => "vtt",
        _ => "srt", // fallback; ffmpeg can usually convert
    }
}

/// Extract subtitle stream `stream_idx` (overall stream index, ffmpeg's
/// `0:<idx>`) from `media` into a temp file and parse it.
pub async fn extract_and_parse(
    media: &Path,
    stream_idx: usize,
    format: &str,
) -> Result<Vec<Line>> {
    let ext = ext_for_format(format);
    let temp = NamedTempFile::new().context("temp file")?;
    let out_path: PathBuf = temp.path().with_extension(ext);

    let status = Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error"])
        .arg("-i")
        .arg(media)
        .args(["-map", &format!("0:{stream_idx}")])
        .args(["-c:s", codec_for_ext(ext)])
        .arg(&out_path)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("spawn ffmpeg")?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr);
        return Err(anyhow!("ffmpeg extract failed: {}", stderr.trim()));
    }

    let body = tokio::fs::read_to_string(&out_path)
        .await
        .with_context(|| format!("read {}", out_path.display()))?;
    let _ = tokio::fs::remove_file(&out_path).await;

    match ext {
        "ass" => ass::parse(&body),
        _ => srt::parse(&body),
    }
}

fn codec_for_ext(ext: &str) -> &'static str {
    match ext {
        "ass" => "ass",
        "vtt" => "webvtt",
        _ => "srt",
    }
}
