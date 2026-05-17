#![allow(dead_code)]

use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use tokio::process::Command;

/// Probe a media file with ffprobe, returning the parsed stream summary.
///
/// We only consume the JSON shape we actually use; ffprobe emits a lot
/// of detail we ignore.
pub async fn probe(path: &Path) -> Result<Probe> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_format",
            "-show_streams",
        ])
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("spawn ffprobe")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffprobe failed: {}", stderr.trim()));
    }

    let raw: RawProbe = serde_json::from_slice(&output.stdout).context("parse ffprobe JSON")?;
    Ok(raw.into())
}

#[derive(Debug, Clone)]
pub struct Probe {
    pub duration_ms: i64,
    pub streams: Vec<Stream>,
}

#[derive(Debug, Clone)]
pub struct Stream {
    pub index: usize,
    pub codec_type: StreamKind,
    pub codec_name: String,
    pub language: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamKind {
    Video,
    Audio,
    Subtitle,
    Other,
}

impl Probe {
    /// Streams of the given kind, in declaration order.
    pub fn streams_of(&self, kind: StreamKind) -> impl Iterator<Item = &Stream> {
        self.streams.iter().filter(move |s| s.codec_type == kind)
    }
}

// --- Raw ffprobe shape ----------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawProbe {
    format: Option<RawFormat>,
    streams: Vec<RawStream>,
}

#[derive(Debug, Deserialize)]
struct RawFormat {
    duration: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawStream {
    index: usize,
    codec_type: Option<String>,
    codec_name: Option<String>,
    #[serde(default)]
    tags: RawTags,
}

#[derive(Debug, Default, Deserialize)]
struct RawTags {
    language: Option<String>,
    title: Option<String>,
}

impl From<RawProbe> for Probe {
    fn from(raw: RawProbe) -> Self {
        let duration_ms = raw
            .format
            .and_then(|f| f.duration)
            .and_then(|s| s.parse::<f64>().ok())
            .map(|secs| (secs * 1000.0) as i64)
            .unwrap_or(0);

        let streams = raw
            .streams
            .into_iter()
            .map(|s| Stream {
                index: s.index,
                codec_type: match s.codec_type.as_deref() {
                    Some("video") => StreamKind::Video,
                    Some("audio") => StreamKind::Audio,
                    Some("subtitle") => StreamKind::Subtitle,
                    _ => StreamKind::Other,
                },
                codec_name: s.codec_name.unwrap_or_default(),
                language: s.tags.language,
                title: s.tags.title,
            })
            .collect();

        Probe {
            duration_ms,
            streams,
        }
    }
}
