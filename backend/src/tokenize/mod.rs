#![allow(dead_code)]

pub mod deinflect;
pub mod jmdict_index;
pub mod lindera_wrap;
pub mod segment;

use serde::{Deserialize, Serialize};

/// Final token shape persisted to `subtitle_lines.tokens_json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// Character offsets into the original line, `[start, end)`.
    pub span: (usize, usize),
    /// Surface form as it appears in the line.
    pub surface: String,
    /// Dictionary form (lemma) — what JMdict and the user "know".
    pub lemma: String,
    /// Kana reading (katakana from UniDic; we keep as-is).
    pub reading: String,
    /// Coarse POS tag (`noun`, `verb`, `adj`, `aux`, `particle`,
    /// `punctuation`, `other`).
    pub pos: String,
    /// JMdict ent_seq if we successfully matched into the dictionary,
    /// otherwise `None`.
    pub dict_seq: Option<i64>,
}
