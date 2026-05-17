use serde::{Deserialize, Serialize};

/// Per-line LLM output as defined in spec §9.2.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineTranslation {
    pub idx: i64,
    pub english: String,
    #[serde(default)]
    pub grammar_note: String,
    #[serde(default)]
    pub tone_tags: Vec<String>,
}

/// Wrapper shape the model returns: `{ "lines": [...] }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationBatch {
    pub lines: Vec<LineTranslation>,
}
