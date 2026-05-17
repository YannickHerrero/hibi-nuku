//! WaniKani DB import. Populates wk_kanji / wk_vocab /
//! wk_kanji_to_vocab and caches the user's level in wk_meta.

use std::collections::HashMap;

use anyhow::{Context, Result};
use serde_json::json;
use sqlx::SqlitePool;
use tracing::info;

use super::client::WaniKani;
use super::model::{DataItem, SubjectData};

pub async fn run(pool: &SqlitePool, api_key: String) -> Result<Summary> {
    let client = WaniKani::new(api_key);

    info!("fetching /user");
    let user = client.user().await?;
    info!(level = user.data.level, user = %user.data.username, "WK user");
    sqlx::query("INSERT OR REPLACE INTO wk_meta (key, value) VALUES (?, ?)")
        .bind("user_level")
        .bind(user.data.level.to_string())
        .execute(pool)
        .await
        .context("persist user_level")?;
    sqlx::query("INSERT OR REPLACE INTO wk_meta (key, value) VALUES (?, ?)")
        .bind("username")
        .bind(&user.data.username)
        .execute(pool)
        .await
        .context("persist username")?;

    info!("fetching kanji subjects");
    let kanji = client.all_subjects("kanji").await?;
    info!(count = kanji.len(), "kanji fetched");
    let vocab = client.all_subjects("vocabulary").await?;
    info!(count = vocab.len(), "vocab fetched");

    let id_to_kanji: HashMap<i64, String> = kanji
        .iter()
        .filter_map(|d| d.data.characters.as_ref().map(|c| (d.id, c.clone())))
        .collect();

    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM wk_kanji").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM wk_vocab").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM wk_kanji_to_vocab")
        .execute(&mut *tx)
        .await?;

    for entry in &kanji {
        insert_kanji(&mut tx, entry).await?;
    }
    for entry in &vocab {
        insert_vocab(&mut tx, entry, &id_to_kanji).await?;
    }

    tx.commit().await?;

    Ok(Summary {
        kanji: kanji.len(),
        vocab: vocab.len(),
        user_level: user.data.level,
    })
}

async fn insert_kanji(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entry: &DataItem<SubjectData>,
) -> Result<()> {
    let Some(chars) = entry.data.characters.as_deref() else {
        return Ok(());
    };
    let primary_meaning = entry
        .data
        .meanings
        .iter()
        .find(|m| m.primary)
        .or_else(|| entry.data.meanings.first())
        .map(|m| m.meaning.clone())
        .unwrap_or_default();
    let meanings: Vec<String> = entry.data.meanings.iter().map(|m| m.meaning.clone()).collect();
    let primary_reading = entry
        .data
        .readings
        .iter()
        .find(|r| r.primary)
        .or_else(|| entry.data.readings.first())
        .map(|r| r.reading.clone())
        .unwrap_or_default();
    let reading_type = entry
        .data
        .readings
        .iter()
        .find(|r| r.primary)
        .and_then(|r| r.kind.clone())
        .or_else(|| entry.data.readings.first().and_then(|r| r.kind.clone()))
        .unwrap_or_else(|| "onyomi".into());
    let readings_json = serde_json::to_string(&entry.data.readings)?;
    let raw_json = serde_json::to_string(&json!({
        "id": entry.id,
        "level": entry.data.level,
        "meanings": entry.data.meanings.iter().map(|m| &m.meaning).collect::<Vec<_>>(),
    }))?;

    sqlx::query(
        "INSERT OR REPLACE INTO wk_kanji
         (characters, level, primary_meaning, meanings_json,
          primary_reading, reading_type, readings_json, raw_json)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(chars)
    .bind(entry.data.level)
    .bind(&primary_meaning)
    .bind(serde_json::to_string(&meanings)?)
    .bind(&primary_reading)
    .bind(&reading_type)
    .bind(&readings_json)
    .bind(&raw_json)
    .execute(&mut **tx)
    .await
    .context("insert kanji")?;
    Ok(())
}

async fn insert_vocab(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    entry: &DataItem<SubjectData>,
    id_to_kanji: &HashMap<i64, String>,
) -> Result<()> {
    let Some(chars) = entry.data.characters.as_deref() else {
        return Ok(());
    };
    let meanings: Vec<String> = entry.data.meanings.iter().map(|m| m.meaning.clone()).collect();
    let readings: Vec<String> = entry.data.readings.iter().map(|r| r.reading.clone()).collect();
    let kanji_chars: Vec<String> = entry
        .data
        .component_subject_ids
        .iter()
        .filter_map(|id| id_to_kanji.get(id).cloned())
        .collect();

    sqlx::query(
        "INSERT OR REPLACE INTO wk_vocab
         (characters, level, meanings_json, readings_json, kanji_chars_json)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(chars)
    .bind(entry.data.level)
    .bind(serde_json::to_string(&meanings)?)
    .bind(serde_json::to_string(&readings)?)
    .bind(serde_json::to_string(&kanji_chars)?)
    .execute(&mut **tx)
    .await
    .context("insert vocab")?;

    for k in &kanji_chars {
        sqlx::query(
            "INSERT OR REPLACE INTO wk_kanji_to_vocab (kanji, vocab) VALUES (?, ?)",
        )
        .bind(k)
        .bind(chars)
        .execute(&mut **tx)
        .await
        .context("insert kanji_to_vocab")?;
    }
    Ok(())
}

pub struct Summary {
    pub kanji: usize,
    pub vocab: usize,
    pub user_level: i64,
}
