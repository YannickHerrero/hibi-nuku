//! Schema for Hibi API endpoints we consume.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FuriganaPair {
    pub base: String,
    pub reading: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanjiEntry {
    pub kanji: String,
    pub meaning: String,
    #[serde(rename = "wanikaniLevel")]
    pub wanikani_level: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CardRequest {
    pub sentence: String,
    pub focus_word: String,
    pub focus_word_reading: String,
    pub furigana: Vec<FuriganaPair>,
    pub english: String,
    pub glosses: Vec<String>,
    /// Per the OpenAPI: nullable string. Send None ↔ JSON null.
    pub grammar_note: Option<String>,
    pub kanji_list: Vec<KanjiEntry>,
    pub image_key: Option<String>,
    pub audio_key: Option<String>,
    pub source: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreatedCard {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UploadKey {
    pub key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KnownWord {
    pub lemma: String,
    pub reading: String,
    /// "learning" | "known" | "ignored"
    pub status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KnownWordsResp {
    pub items: Vec<KnownWord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordStatusReq {
    pub lemma: String,
    pub reading: String,
    /// `Some("learning"|"known"|"ignored")` or `None` to clear.
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WordStatus {
    pub lemma: String,
    pub reading: String,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequest {
    pub kind: String,
    pub source: String,
    pub started_at: String,
    pub ended_at: String,
    pub duration_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}
