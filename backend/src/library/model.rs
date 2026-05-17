use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Pipeline status — written progressively as the import task advances.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoStatus {
    Probing,
    Extracting,
    Parsing,
    Tokenizing,
    Translating,
    Remuxing,
    Ready,
    Error,
}

impl VideoStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            VideoStatus::Probing => "probing",
            VideoStatus::Extracting => "extracting",
            VideoStatus::Parsing => "parsing",
            VideoStatus::Tokenizing => "tokenizing",
            VideoStatus::Translating => "translating",
            VideoStatus::Remuxing => "remuxing",
            VideoStatus::Ready => "ready",
            VideoStatus::Error => "error",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "probing" => VideoStatus::Probing,
            "extracting" => VideoStatus::Extracting,
            "parsing" => VideoStatus::Parsing,
            "tokenizing" => VideoStatus::Tokenizing,
            "translating" => VideoStatus::Translating,
            "remuxing" => VideoStatus::Remuxing,
            "ready" => VideoStatus::Ready,
            "error" => VideoStatus::Error,
            _ => return None,
        })
    }
}

impl fmt::Display for VideoStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub id: i64,
    pub path: String,
    pub title: String,
    pub source_tag: String,
    pub duration_ms: i64,
    pub jp_audio_idx: Option<i64>,
    pub jp_subtitle_idx: Option<i64>,
    pub subtitle_format: Option<String>,
    pub status: VideoStatus,
    pub error_message: Option<String>,
    pub imported_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
