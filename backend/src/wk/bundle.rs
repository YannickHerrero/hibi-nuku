//! Build the client-side WK bundle from the populated tables.

use std::collections::HashMap;

use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub version: String,
    pub user_level: Option<i64>,
    pub kanji: HashMap<String, KanjiEntry>,
    pub vocab: HashMap<String, VocabEntry>,
    pub vocab_by_kanji: HashMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KanjiEntry {
    pub level: i64,
    pub meaning: String,
    pub reading: String,
    pub reading_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VocabEntry {
    pub level: i64,
    pub meanings: Vec<String>,
    pub readings: Vec<String>,
}

pub async fn build(pool: &SqlitePool) -> Result<Bundle> {
    let mut kanji = HashMap::new();
    let rows = sqlx::query(
        "SELECT characters, level, primary_meaning, primary_reading, reading_type FROM wk_kanji",
    )
    .fetch_all(pool)
    .await
    .context("scan wk_kanji")?;
    for r in rows {
        let chars: String = r.try_get("characters")?;
        kanji.insert(
            chars,
            KanjiEntry {
                level: r.try_get("level")?,
                meaning: r.try_get("primary_meaning")?,
                reading: r.try_get("primary_reading")?,
                reading_type: r.try_get("reading_type")?,
            },
        );
    }

    let mut vocab = HashMap::new();
    let rows = sqlx::query("SELECT characters, level, meanings_json, readings_json FROM wk_vocab")
        .fetch_all(pool)
        .await
        .context("scan wk_vocab")?;
    for r in rows {
        let chars: String = r.try_get("characters")?;
        let meanings_json: String = r.try_get("meanings_json")?;
        let readings_json: String = r.try_get("readings_json")?;
        vocab.insert(
            chars,
            VocabEntry {
                level: r.try_get("level")?,
                meanings: serde_json::from_str(&meanings_json)?,
                readings: serde_json::from_str(&readings_json)?,
            },
        );
    }

    let mut vocab_by_kanji: HashMap<String, Vec<String>> = HashMap::new();
    let rows = sqlx::query("SELECT kanji, vocab FROM wk_kanji_to_vocab")
        .fetch_all(pool)
        .await
        .context("scan wk_kanji_to_vocab")?;
    for r in rows {
        let kanji: String = r.try_get("kanji")?;
        let vocab: String = r.try_get("vocab")?;
        vocab_by_kanji.entry(kanji).or_default().push(vocab);
    }

    let user_level: Option<i64> =
        sqlx::query_scalar("SELECT value FROM wk_meta WHERE key = 'user_level'")
            .fetch_optional(pool)
            .await?
            .and_then(|v: String| v.parse().ok());

    Ok(Bundle {
        version: Utc::now().format("%Y-%m-%d").to_string(),
        user_level,
        kanji,
        vocab,
        vocab_by_kanji,
    })
}
