//! Load a built JMDict bundle into the in-memory index used by the
//! tokenizer.

use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;

use super::bundle::Bundle;
use crate::tokenize::jmdict_index::{JmdictEntry, JmdictIndex};

pub fn load_bundle(path: &Path) -> Result<Bundle> {
    let file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut json = String::new();
    GzDecoder::new(file)
        .read_to_string(&mut json)
        .context("gunzip bundle")?;
    let bundle: Bundle = serde_json::from_str(&json).context("decode bundle JSON")?;
    Ok(bundle)
}

pub fn build_index(bundle: Bundle) -> JmdictIndex {
    let mut idx = JmdictIndex::new();
    for entry in bundle.entries {
        let pos_tags: Vec<String> = entry
            .senses
            .iter()
            .flat_map(|s| s.pos.iter().cloned())
            .collect();
        let glosses: Vec<String> = entry
            .senses
            .iter()
            .flat_map(|s| s.glosses.iter().cloned())
            .collect();
        idx.insert(JmdictEntry {
            seq: entry.seq,
            kanji: entry.kanji,
            readings: entry.readings,
            glosses,
            pos_tags,
            rules: entry.rules,
        });
    }
    idx
}

/// Convenience: load + index.
pub fn load(path: &Path) -> Result<JmdictIndex> {
    let bundle = load_bundle(path)?;
    Ok(build_index(bundle))
}
