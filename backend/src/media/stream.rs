//! ffmpeg-based streaming remux.
//!
//! We don't implement byte-range scrubbing over the ffmpeg pipe in v1.
//! The client reloads `?from=<seconds>` when the user seeks, which is
//! a one-RTT round-trip and works on every browser without needing
//! Content-Length tricks.

use std::path::Path;
use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::io::AsyncRead;
use tokio::process::{Child, Command};

pub struct Stream {
    pub child: Child,
}

impl Stream {
    pub fn stdout(&mut self) -> Option<impl AsyncRead + Unpin + Send + use<>> {
        self.child.stdout.take()
    }
}

/// Spawn an ffmpeg child that remuxes `path` to fragmented MP4 on
/// stdout, optionally starting at `start_sec`. JP audio track is
/// selected via `audio_idx`; video is copied; incompatible audio
/// codecs are transcoded to AAC 128k.
pub fn spawn(
    path: &Path,
    audio_idx: Option<i64>,
    audio_codec: AudioPlan,
    start_sec: Option<f64>,
) -> Result<Stream> {
    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-loglevel").arg("error");
    cmd.arg("-y");

    if let Some(s) = start_sec {
        cmd.arg("-ss").arg(format!("{s}"));
    }

    cmd.arg("-i").arg(path);
    cmd.arg("-map").arg("0:v:0");
    if let Some(idx) = audio_idx {
        cmd.arg("-map").arg(format!("0:{idx}"));
    } else {
        cmd.arg("-map").arg("0:a:0?");
    }

    cmd.arg("-c:v").arg("copy");
    match audio_codec {
        AudioPlan::Copy => {
            cmd.arg("-c:a").arg("copy");
        }
        AudioPlan::Aac => {
            cmd.arg("-c:a").arg("aac").arg("-b:a").arg("160k");
        }
    }

    cmd.arg("-movflags")
        .arg("frag_keyframe+empty_moov+default_base_moof");
    cmd.arg("-f").arg("mp4");
    cmd.arg("pipe:1");

    cmd.stdout(Stdio::piped()).stderr(Stdio::null());

    let child = cmd.spawn().context("spawn ffmpeg")?;
    Ok(Stream { child })
}

/// Whether the source's audio is already compatible with MP4
/// (so we can `-c:a copy`) or needs transcoding to AAC.
#[derive(Debug, Clone, Copy)]
pub enum AudioPlan {
    Copy,
    Aac,
}

pub fn audio_plan(codec_name: &str) -> AudioPlan {
    match codec_name {
        "aac" | "mp3" | "alac" => AudioPlan::Copy,
        // AC3/EAC3/DTS/TrueHD/FLAC/Opus aren't broadly playable in
        // browsers inside an MP4 container — transcode.
        _ => AudioPlan::Aac,
    }
}
