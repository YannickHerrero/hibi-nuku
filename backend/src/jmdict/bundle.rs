//! On-disk JSON shape of a built JMDict bundle.

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub version: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub seq: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kanji: Vec<String>,
    pub readings: Vec<String>,
    pub senses: Vec<Sense>,
    /// Deinflection rule tags (`v1`, `v5k`, `adj-i`, …) — flattened
    /// from sense POS tags for fast filtering.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sense {
    /// Part-of-speech entity codes from JMDict (e.g. `["v1", "vt"]`).
    pub pos: Vec<String>,
    pub glosses: Vec<String>,
}
