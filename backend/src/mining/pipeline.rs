//! Mining orchestration: extract assets → upload → assemble card →
//! POST to Hibi.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tracing::info;

use crate::config::Config;
use crate::hibi::client::Hibi;
use crate::hibi::model::CardRequest;
use crate::library::{lines, repo};
use crate::mining::{assets, furigana, kanji};
use crate::tokenize::Token;
use crate::tokenize::jmdict_index::JmdictIndex;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MineRequest {
    pub video_id: i64,
    pub line_id: i64,
    pub focus_word: String,
    pub focus_word_reading: String,
    #[serde(default = "default_pad")]
    pub pad_before_ms: i64,
    #[serde(default = "default_pad")]
    pub pad_after_ms: i64,
    #[serde(default)]
    pub user_english_override: Option<String>,
    #[serde(default)]
    pub user_grammar_override: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_pad() -> i64 {
    500
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MineResponse {
    pub card_id: String,
    pub audio_key: String,
    pub image_key: String,
}

pub async fn run(
    pool: Arc<SqlitePool>,
    config: Arc<Config>,
    jmdict: Arc<JmdictIndex>,
    req: MineRequest,
) -> Result<MineResponse> {
    let video = repo::get(&pool, req.video_id)
        .await?
        .ok_or_else(|| anyhow!("video {} not found", req.video_id))?;
    let line = lines::list_for_video(&pool, req.video_id)
        .await?
        .into_iter()
        .find(|l| l.id == req.line_id)
        .ok_or_else(|| anyhow!("line {} not in video {}", req.line_id, req.video_id))?;

    let path: PathBuf = video.path.into();
    if !path.exists() {
        return Err(anyhow!("video file missing on disk: {}", path.display()));
    }

    let start_ms = (line.start_ms - req.pad_before_ms).max(0);
    let end_ms = line.end_ms + req.pad_after_ms;
    let mid_ms = (line.start_ms + line.end_ms) / 2;

    // Temp file paths.
    let tmp_dir = std::env::temp_dir();
    let audio_tmp = tmp_dir.join(format!("nuku-mine-{}-{}.m4a", req.video_id, req.line_id));
    let image_tmp = tmp_dir.join(format!("nuku-mine-{}-{}.webp", req.video_id, req.line_id));

    let (audio_res, image_res) = tokio::join!(
        assets::extract_audio(&path, video.jp_audio_idx, start_ms, end_ms, &audio_tmp),
        assets::extract_screenshot(&path, mid_ms, &image_tmp),
    );
    audio_res.context("extract audio")?;
    image_res.context("extract screenshot")?;

    let hibi = Hibi::new(
        config.hibi_base.clone(),
        config.hibi_api_key.clone(),
    );

    let audio_bytes = tokio::fs::read(&audio_tmp).await?;
    let image_bytes = tokio::fs::read(&image_tmp).await?;

    let (audio_up, image_up) = tokio::join!(
        hibi.upload_audio(audio_bytes, "clip.m4a", "audio/mp4"),
        hibi.upload_image(image_bytes, "shot.webp", "image/webp"),
    );
    let audio_key = audio_up.context("upload audio")?.key;
    let image_key = image_up.context("upload image")?.key;

    let furigana = furigana::build(&req.focus_word, &req.focus_word_reading);
    let kanji_list = kanji::enrich_for_text(&pool, &line.raw_text).await?;

    // Pull top-3 glosses for focus word from JMDict via the token's
    // dict_seq if available, otherwise by surface lookup as a fallback.
    let glosses = focus_glosses(&line.tokens_json, &req.focus_word, jmdict.as_ref());

    let english = req
        .user_english_override
        .clone()
        .or_else(|| line.translation.clone())
        .unwrap_or_default();
    let grammar_note = req
        .user_grammar_override
        .clone()
        .or_else(|| line.grammar_note.clone());

    let card = CardRequest {
        sentence: line.raw_text.clone(),
        focus_word: req.focus_word.clone(),
        focus_word_reading: req.focus_word_reading.clone(),
        furigana,
        english,
        glosses,
        grammar_note,
        kanji_list,
        image_key: Some(image_key.clone()),
        audio_key: Some(audio_key.clone()),
        source: video.source_tag.clone(),
        tags: req.tags.clone(),
    };

    let created = hibi.create_card(&card).await.context("hibi create card")?;
    info!(
        card_id = %created.id,
        video_id = req.video_id,
        line_id = req.line_id,
        "mined card"
    );

    // Best-effort cleanup; failure is just a stale temp file.
    let _ = tokio::fs::remove_file(&audio_tmp).await;
    let _ = tokio::fs::remove_file(&image_tmp).await;

    Ok(MineResponse {
        card_id: created.id,
        audio_key,
        image_key,
    })
}

fn focus_glosses(tokens_json: &Option<String>, focus: &str, jmdict: &JmdictIndex) -> Vec<String> {
    if let Some(json) = tokens_json {
        if let Ok(tokens) = serde_json::from_str::<Vec<Token>>(json) {
            if let Some(seq) = tokens
                .iter()
                .find(|t| t.lemma == focus)
                .and_then(|t| t.dict_seq)
            {
                if let Some(entry) = jmdict.entry(seq) {
                    return entry.glosses.iter().take(3).cloned().collect();
                }
            }
        }
    }
    if let Some(entry) = jmdict.lookup(focus).into_iter().next() {
        return entry.glosses.iter().take(3).cloned().collect();
    }
    Vec::new()
}
