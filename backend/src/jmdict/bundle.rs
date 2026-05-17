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
    /// Sum of JMDict priority markers across all k_ele + r_ele —
    /// higher = more common. Used to rank competing entries when
    /// multiple share a surface form (e.g. 僕 ぼく vs 僕 しもべ).
    /// 0 = no priority info.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub priority: i32,
}

fn is_zero(n: &i32) -> bool {
    *n == 0
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Sense {
    /// Part-of-speech entity codes from JMDict (e.g. `["v1", "vt"]`).
    pub pos: Vec<String>,
    pub glosses: Vec<String>,
}
