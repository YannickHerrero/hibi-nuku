//! Batched subtitle translation: chunk lines, check cache, call LLM,
//! persist results.

use std::sync::Arc;

use anyhow::{Context, Result};
use serde_json::Value;
use sqlx::SqlitePool;
use tracing::{info, warn};

use crate::library::lines::{self, SubtitleLine};
use crate::llm::cache;
use crate::llm::client::{ChatMessage, OpenRouter};
use crate::llm::prompt::{self, PROMPT_VERSION};
use crate::llm::schema::{LineTranslation, TranslationBatch};

const BATCH_SIZE: usize = 50;
const CONTEXT: usize = 2;

pub async fn translate_lines(
    pool: Arc<SqlitePool>,
    client: &OpenRouter,
    model: &str,
    video_id: i64,
) -> Result<usize> {
    let all = lines::list_for_video(&pool, video_id)
        .await
        .context("load lines for translation")?;
    if all.is_empty() {
        return Ok(0);
    }

    let mut translated = 0usize;

    for chunk_start in (0..all.len()).step_by(BATCH_SIZE) {
        let chunk_end = (chunk_start + BATCH_SIZE).min(all.len());
        let before: Vec<&SubtitleLine> =
            all[chunk_start.saturating_sub(CONTEXT)..chunk_start].iter().collect();
        let targets: Vec<&SubtitleLine> = all[chunk_start..chunk_end].iter().collect();
        let after: Vec<&SubtitleLine> = all[chunk_end..(chunk_end + CONTEXT).min(all.len())]
            .iter()
            .collect();

        let user = prompt::user_message(&before, &targets, &after);
        let cache_key = cache::hash_key(model, PROMPT_VERSION, &user);

        let json_text = if let Some(hit) = cache::get(&pool, &cache_key).await? {
            info!(video_id, chunk_start, "llm_cache hit");
            hit
        } else {
            info!(video_id, chunk_start, "llm_cache miss; calling OpenRouter");
            let messages = vec![
                ChatMessage {
                    role: "system",
                    content: prompt::SYSTEM.to_string(),
                },
                ChatMessage {
                    role: "user",
                    content: user.clone(),
                },
            ];
            let resp = client.complete_json(model, &messages).await?;
            let resp_str = serde_json::to_string(&resp)?;
            cache::put(&pool, &cache_key, model, &resp_str).await?;
            resp_str
        };

        let batch = parse_response(&json_text).with_context(|| {
            format!("decode LLM response for chunk starting at idx {chunk_start}")
        })?;

        let target_by_idx: std::collections::HashMap<i64, &SubtitleLine> =
            targets.iter().map(|l| (l.idx, *l)).collect();
        for tr in batch.lines {
            let Some(line) = target_by_idx.get(&tr.idx) else {
                warn!(unknown_idx = tr.idx, "model returned unknown idx");
                continue;
            };
            let tone_json = serde_json::to_string(&tr.tone_tags)?;
            lines::set_translation(
                &pool,
                line.id,
                &tr.english,
                &tr.grammar_note,
                &tone_json,
            )
            .await?;
            translated += 1;
        }
    }

    Ok(translated)
}

fn parse_response(json_text: &str) -> Result<TranslationBatch> {
    let v: Value = serde_json::from_str(json_text)?;
    // Tolerate models that wrap output in extra structure: look for a
    // top-level "lines" key, otherwise interpret as the batch directly.
    let lines = if let Some(arr) = v.get("lines").and_then(|x| x.as_array()) {
        arr.clone()
    } else if let Some(arr) = v.as_array() {
        arr.clone()
    } else {
        return Err(anyhow::anyhow!("response has no `lines` array"));
    };
    let mut out = Vec::with_capacity(lines.len());
    for entry in lines {
        let tr: LineTranslation = serde_json::from_value(entry)?;
        out.push(tr);
    }
    Ok(TranslationBatch { lines: out })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wrapped_response() {
        let json = r#"{"lines":[{"idx":3,"english":"hi","grammar_note":"","tone_tags":[]}]}"#;
        let b = parse_response(json).unwrap();
        assert_eq!(b.lines.len(), 1);
        assert_eq!(b.lines[0].idx, 3);
    }

    #[test]
    fn parses_bare_array() {
        let json = r#"[{"idx":1,"english":"x","grammar_note":"","tone_tags":["polite"]}]"#;
        let b = parse_response(json).unwrap();
        assert_eq!(b.lines[0].tone_tags, vec!["polite".to_string()]);
    }

    #[test]
    fn hash_keys_are_stable_and_distinct() {
        let a = cache::hash_key("m1", "v1", "p");
        let b = cache::hash_key("m1", "v1", "p");
        let c = cache::hash_key("m1", "v2", "p");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
