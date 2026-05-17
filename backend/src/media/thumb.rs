//! Generate a single screenshot at a percentage through the video.

use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result, anyhow};
use tokio::process::Command;

pub async fn extract(media: &Path, at_sec: f64, out: &Path) -> Result<()> {
    let output = Command::new("ffmpeg")
        .args(["-y", "-loglevel", "error"])
        .arg("-ss")
        .arg(format!("{at_sec}"))
        .arg("-i")
        .arg(media)
        .args(["-frames:v", "1", "-q:v", "4"])
        .arg(out)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("spawn ffmpeg")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("thumbnail failed: {}", stderr.trim()));
    }
    Ok(())
}
