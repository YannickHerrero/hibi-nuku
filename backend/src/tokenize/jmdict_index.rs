//! Placeholder — loaded with real data once JMDict bundle is built
//! in the next commits.

use std::collections::HashMap;

/// In-memory JMDict index used by the longest-match segmenter and the
/// mining card-payload assembly.
#[derive(Debug, Default)]
pub struct JmdictIndex {
    /// surface (kanji or reading) → list of `ent_seq` ints, in JMDict
    /// declaration order so popular entries come first.
    by_surface: HashMap<String, Vec<i64>>,
    /// `ent_seq` → entry metadata.
    entries: HashMap<i64, JmdictEntry>,
}

#[derive(Debug, Clone)]
pub struct JmdictEntry {
    pub seq: i64,
    pub kanji: Vec<String>,
    pub readings: Vec<String>,
    pub glosses: Vec<String>,
    pub pos_tags: Vec<String>,
    /// Deinflection rule tags (`v1`, `v5k`, `adj-i`, …) — used by
    /// the deinflector to filter compatible matches.
    pub rules: Vec<String>,
}

impl JmdictIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, entry: JmdictEntry) {
        for k in &entry.kanji {
            self.by_surface.entry(k.clone()).or_default().push(entry.seq);
        }
        for r in &entry.readings {
            self.by_surface.entry(r.clone()).or_default().push(entry.seq);
        }
        self.entries.insert(entry.seq, entry);
    }

    pub fn lookup(&self, surface: &str) -> Vec<&JmdictEntry> {
        self.by_surface
            .get(surface)
            .map(|seqs| seqs.iter().filter_map(|s| self.entries.get(s)).collect())
            .unwrap_or_default()
    }

    pub fn entry(&self, seq: i64) -> Option<&JmdictEntry> {
        self.entries.get(&seq)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
