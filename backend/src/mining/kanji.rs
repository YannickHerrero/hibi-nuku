//! Enrich a sentence's kanji set using the WK cache.

use std::collections::HashSet;

use anyhow::Result;
use sqlx::{Row, SqlitePool};

use crate::hibi::model::KanjiEntry;

fn is_kanji(c: char) -> bool {
    matches!(c as u32,
        0x4E00..=0x9FFF
        | 0x3400..=0x4DBF
        | 0x20000..=0x2A6DF
    )
}

/// Pull WK metadata for every kanji that appears in `text`. Kanji
/// not in WK are emitted with `wanikaniLevel: None` and empty
/// meaning so the card-side renderer can still show them.
pub async fn enrich_for_text(pool: &SqlitePool, text: &str) -> Result<Vec<KanjiEntry>> {
    let mut seen = HashSet::new();
    let mut ordered: Vec<char> = Vec::new();
    for c in text.chars() {
        if is_kanji(c) && seen.insert(c) {
            ordered.push(c);
        }
    }
    let mut out = Vec::with_capacity(ordered.len());
    for c in ordered {
        let row = sqlx::query("SELECT primary_meaning, level FROM wk_kanji WHERE characters = ?")
            .bind(c.to_string())
            .fetch_optional(pool)
            .await?;
        if let Some(r) = row {
            out.push(KanjiEntry {
                kanji: c.to_string(),
                meaning: r.try_get::<String, _>("primary_meaning").unwrap_or_default(),
                wanikani_level: r.try_get("level").ok(),
            });
        } else {
            out.push(KanjiEntry {
                kanji: c.to_string(),
                meaning: String::new(),
                wanikani_level: None,
            });
        }
    }
    Ok(out)
}
