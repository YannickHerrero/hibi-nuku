//! Persistence helpers for `subtitle_lines`.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleLine {
    pub id: i64,
    pub video_id: i64,
    pub idx: i64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub raw_text: String,
    pub tokens_json: Option<String>,
    pub translation: Option<String>,
    pub grammar_note: Option<String>,
    pub tone_tags: Option<String>,
}

pub async fn replace_for_video(
    pool: &SqlitePool,
    video_id: i64,
    lines: &[crate::subtitle::Line],
) -> Result<()> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM subtitle_lines WHERE video_id = ?")
        .bind(video_id)
        .execute(&mut *tx)
        .await
        .context("delete existing lines")?;

    for line in lines {
        sqlx::query(
            "INSERT INTO subtitle_lines (video_id, idx, start_ms, end_ms, raw_text)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(video_id)
        .bind(line.idx as i64)
        .bind(line.start_ms)
        .bind(line.end_ms)
        .bind(&line.text)
        .execute(&mut *tx)
        .await
        .context("insert subtitle line")?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn list_for_video(pool: &SqlitePool, video_id: i64) -> Result<Vec<SubtitleLine>> {
    let rows = sqlx::query(
        "SELECT id, video_id, idx, start_ms, end_ms, raw_text,
                tokens_json, translation, grammar_note, tone_tags
         FROM subtitle_lines WHERE video_id = ? ORDER BY idx",
    )
    .bind(video_id)
    .fetch_all(pool)
    .await
    .context("list subtitle lines")?;
    Ok(rows
        .into_iter()
        .map(|r| SubtitleLine {
            id: r.get("id"),
            video_id: r.get("video_id"),
            idx: r.get("idx"),
            start_ms: r.get("start_ms"),
            end_ms: r.get("end_ms"),
            raw_text: r.get("raw_text"),
            tokens_json: r.get("tokens_json"),
            translation: r.get("translation"),
            grammar_note: r.get("grammar_note"),
            tone_tags: r.get("tone_tags"),
        })
        .collect())
}

pub async fn set_tokens(
    pool: &SqlitePool,
    line_id: i64,
    tokens_json: &str,
) -> Result<()> {
    sqlx::query("UPDATE subtitle_lines SET tokens_json = ? WHERE id = ?")
        .bind(tokens_json)
        .bind(line_id)
        .execute(pool)
        .await
        .context("set tokens_json")?;
    Ok(())
}

pub async fn set_translation(
    pool: &SqlitePool,
    line_id: i64,
    translation: &str,
    grammar_note: &str,
    tone_tags_json: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE subtitle_lines
         SET translation = ?, grammar_note = ?, tone_tags = ?
         WHERE id = ?",
    )
    .bind(translation)
    .bind(grammar_note)
    .bind(tone_tags_json)
    .bind(line_id)
    .execute(pool)
    .await
    .context("set translation")?;
    Ok(())
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TimestampOnly {
    pub id: i64,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LineSnapshot {
    pub id: i64,
    pub idx: i64,
    pub start_ms: i64,
    pub end_ms: i64,
    pub raw_text: String,
    pub imported_at: DateTime<Utc>,
}
