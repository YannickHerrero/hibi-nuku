//! Longest-match dictionary segmentation pass over a sentence.
//!
//! Combines lindera tokenization with JMDict lookups. For each
//! starting character position, we try the longest substring first,
//! deinflect each candidate, and accept the first that matches JMDict
//! with compatible rule tags. Failed positions emit lindera's own
//! token at that offset (or a single-char fallback if even that's
//! absent).

use std::collections::HashMap;

use crate::tokenize::deinflect;
use crate::tokenize::jmdict_index::JmdictIndex;
use crate::tokenize::lindera_wrap::{LinderaToken, coarse_pos};

use super::Token;

/// Segment `text` against `dict`, using `lindera_tokens` (already
/// produced by [`crate::tokenize::lindera_wrap::tokenize`]) as the
/// per-position fallback when no dictionary match is found.
pub fn segment(text: &str, lindera_tokens: &[LinderaToken], dict: &JmdictIndex) -> Vec<Token> {
    // Build an index from char-offset → lindera token starting there.
    let lindera_by_start: HashMap<usize, &LinderaToken> =
        lindera_tokens.iter().map(|t| (t.char_start, t)).collect();

    // Pre-compute character byte positions so substrings are quick.
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let n_chars = chars.len();

    let mut out: Vec<Token> = Vec::new();
    let mut pos: usize = 0;

    while pos < n_chars {
        let mut accepted = None;

        // Longest substring first, down to length 1.
        let max_len = (n_chars - pos).min(MAX_SPAN);
        for len in (1..=max_len).rev() {
            let end = pos + len;
            let byte_start = chars[pos].0;
            let byte_end = if end < n_chars {
                chars[end].0
            } else {
                text.len()
            };
            let candidate = &text[byte_start..byte_end];

            if let Some(hit) = try_dict(candidate, dict) {
                let lemma = hit.lemma.clone();
                let reading = hit.reading.clone();
                let pos_tag = hit.pos.clone();
                accepted = Some(Token {
                    span: (pos, end),
                    surface: candidate.to_string(),
                    lemma,
                    reading,
                    pos: pos_tag,
                    dict_seq: Some(hit.seq),
                });
                break;
            }
        }

        if let Some(tok) = accepted {
            let next = tok.span.1;
            out.push(tok);
            pos = next;
            continue;
        }

        // Fallback: use the lindera token starting at this position.
        if let Some(lt) = lindera_by_start.get(&pos) {
            let span = (lt.char_start, lt.char_end);
            out.push(Token {
                span,
                surface: lt.surface.clone(),
                lemma: lt.lemma.clone(),
                reading: lt.reading.clone(),
                pos: coarse_pos(&lt.pos1, &lt.pos2).to_string(),
                dict_seq: None,
            });
            pos = span.1;
            continue;
        }

        // Last-resort fallback: single character.
        let (byte_start, ch) = chars[pos];
        let byte_end = byte_start + ch.len_utf8();
        let surface = text[byte_start..byte_end].to_string();
        out.push(Token {
            span: (pos, pos + 1),
            surface: surface.clone(),
            lemma: surface.clone(),
            reading: String::new(),
            pos: "other".to_string(),
            dict_seq: None,
        });
        pos += 1;
    }

    out
}

/// Heuristic cap so we don't try absurd lookups on long lines.
const MAX_SPAN: usize = 16;

struct DictHit {
    seq: i64,
    lemma: String,
    reading: String,
    pos: String,
}

fn try_dict(candidate: &str, dict: &JmdictIndex) -> Option<DictHit> {
    for cand in deinflect::deinflect(candidate) {
        let entries = dict.lookup(&cand.stem);
        if entries.is_empty() {
            continue;
        }
        for entry in entries {
            // If the deinflection produced rule tags, require overlap
            // with the entry's rules. Identity deinflection (no tags)
            // matches anything.
            if !cand.rules_out.is_empty() {
                let overlap = entry.rules.iter().any(|r| cand.rules_out.contains(r.as_str()));
                if !overlap {
                    continue;
                }
            }
            let lemma = entry
                .kanji
                .first()
                .cloned()
                .or_else(|| entry.readings.first().cloned())
                .unwrap_or_else(|| cand.stem.clone());
            let reading = entry.readings.first().cloned().unwrap_or_default();
            let pos = entry
                .pos_tags
                .first()
                .cloned()
                .unwrap_or_else(|| "other".to_string());
            return Some(DictHit {
                seq: entry.seq,
                lemma,
                reading,
                pos,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenize::jmdict_index::JmdictEntry;

    fn make_dict() -> JmdictIndex {
        let mut d = JmdictIndex::new();
        d.insert(JmdictEntry {
            seq: 1,
            kanji: vec!["食べる".into()],
            readings: vec!["たべる".into()],
            glosses: vec!["to eat".into()],
            pos_tags: vec!["v1".into()],
            rules: vec!["v1".into()],
            priority: 0,
        });
        d.insert(JmdictEntry {
            seq: 2,
            kanji: vec!["今日".into()],
            readings: vec!["きょう".into()],
            glosses: vec!["today".into()],
            pos_tags: vec!["n".into()],
            rules: vec![],
            priority: 0,
        });
        d
    }

    #[test]
    fn glues_taberu_into_one_span() {
        let dict = make_dict();
        let lindera = crate::tokenize::lindera_wrap::tokenize("食べた").unwrap();
        let toks = segment("食べた", &lindera, &dict);
        assert!(toks.iter().any(|t| t.dict_seq == Some(1)));
    }

    #[test]
    fn unknown_words_fall_back_to_lindera() {
        let dict = make_dict();
        let lindera = crate::tokenize::lindera_wrap::tokenize("ねこ").unwrap();
        let toks = segment("ねこ", &lindera, &dict);
        assert!(!toks.is_empty());
    }
}
