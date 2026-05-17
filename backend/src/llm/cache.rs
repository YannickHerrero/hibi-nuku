//! Persistent LLM response cache keyed by hash(model + prompt + payload).

use anyhow::{Context, Result};
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;

pub fn hash_key(model: &str, prompt_version: &str, payload: &str) -> String {
    let mut h = Sha256::new();
    h.update(model.as_bytes());
    h.update(b"|");
    h.update(prompt_version.as_bytes());
    h.update(b"|");
    h.update(payload.as_bytes());
    hex::encode(h.finalize())
}

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let row: Option<(String,)> = sqlx::query_as("SELECT response_json FROM llm_cache WHERE prompt_hash = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .context("llm_cache get")?;
    Ok(row.map(|(s,)| s))
}

pub async fn put(pool: &SqlitePool, key: &str, model: &str, response_json: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT OR REPLACE INTO llm_cache (prompt_hash, model, response_json, created_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(key)
    .bind(model)
    .bind(response_json)
    .bind(now)
    .execute(pool)
    .await
    .context("llm_cache put")?;
    Ok(())
}
