//! Thin wrapper over lindera UniDic tokenizer.
//!
//! Exposes a `tokenize` returning per-morpheme `LinderaToken`s with
//! the few fields we actually consume from UniDic's verbose feature
//! array.

use std::sync::OnceLock;

use anyhow::{Context, Result};
use lindera::dictionary::DictionaryKind;
use lindera::dictionary::load_embedded_dictionary;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer;

/// Per-morpheme output of lindera on UniDic.
#[derive(Debug, Clone)]
pub struct LinderaToken {
    pub surface: String,
    /// Byte offset of the surface in the input.
    pub byte_start: usize,
    pub byte_end: usize,
    /// Character offset of the surface in the input.
    pub char_start: usize,
    pub char_end: usize,
    pub lemma: String,
    pub reading: String,
    /// Major POS class (`名詞`/`動詞`/`形容詞`/…).
    pub pos1: String,
    pub pos2: String,
}

fn tokenizer() -> &'static Tokenizer {
    static T: OnceLock<Tokenizer> = OnceLock::new();
    T.get_or_init(|| {
        let dict = load_embedded_dictionary(DictionaryKind::UniDic)
            .expect("embedded UniDic dictionary");
        let seg = Segmenter::new(Mode::Normal, dict, None);
        Tokenizer::new(seg)
    })
}

/// Tokenise `text` and return our compact `LinderaToken` list.
pub fn tokenize(text: &str) -> Result<Vec<LinderaToken>> {
    #[allow(unused_mut)]
    let mut tok = tokenizer().clone();
    let raw = tok.tokenize(text).context("lindera tokenize")?;

    let mut out = Vec::with_capacity(raw.len());
    for mut t in raw {
        let surface = t.surface.to_string();
        let byte_start = t.byte_start;
        let byte_end = t.byte_end;
        let details = t.details();

        // UniDic feature columns (per documentation):
        //  0 pos1 / 1 pos2 / 2 pos3 / 3 pos4 / 4 cType / 5 cForm /
        //  6 lForm / 7 lemma / 8 orth / 9 pron / 10 orthBase / 11 pronBase /
        //  12 goshu / 13 iType / 14 iForm / 15 fType / 16 fForm / 17 kana /
        //  18 kanaBase / 19 form / 20 formBase / 21 iConType / 22 fConType /
        //  23 aType / 24 aConType / 25 aModType / 26 lid / 27 lemma_id
        let pos1 = details.first().map(|s| s.to_string()).unwrap_or_default();
        let pos2 = details.get(1).map(|s| s.to_string()).unwrap_or_default();
        let lemma = details
            .get(7)
            .or_else(|| details.get(10))
            .map(|s| s.to_string())
            .unwrap_or_else(|| surface.clone());
        let reading = details
            .get(17)
            .or_else(|| details.get(9))
            .map(|s| s.to_string())
            .unwrap_or_default();

        let char_start = text[..byte_start].chars().count();
        let char_end = char_start + surface.chars().count();

        out.push(LinderaToken {
            surface,
            byte_start,
            byte_end,
            char_start,
            char_end,
            lemma,
            reading,
            pos1,
            pos2,
        });
    }

    Ok(out)
}

/// Coarse POS bucket used downstream.
pub fn coarse_pos(pos1: &str, pos2: &str) -> &'static str {
    match (pos1, pos2) {
        ("名詞", _) => "noun",
        ("動詞", _) => "verb",
        ("形容詞", _) => "adj",
        ("形状詞", _) => "adj",
        ("助動詞", _) => "aux",
        ("助詞", _) => "particle",
        ("補助記号", _) | ("記号", _) => "punctuation",
        ("接続詞", _) => "conj",
        ("感動詞", _) => "interj",
        ("代名詞", _) => "pron",
        ("副詞", _) => "adv",
        ("連体詞", _) => "adn",
        ("接頭辞", _) => "prefix",
        ("接尾辞", _) => "suffix",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_taberu_past() {
        let toks = tokenize("食べた").unwrap();
        assert!(!toks.is_empty(), "expected at least one token");
        let surfaces: Vec<_> = toks.iter().map(|t| t.surface.as_str()).collect();
        assert!(surfaces.contains(&"食べ"));
        let taberu = toks.iter().find(|t| t.surface == "食べ").unwrap();
        assert_eq!(taberu.lemma, "食べる");
    }

    #[test]
    fn handles_mixed_sentence() {
        let toks = tokenize("今日は学校に行きます。").unwrap();
        assert!(toks.len() > 3);
        let kyou = toks.iter().find(|t| t.surface == "今日");
        assert!(kyou.is_some(), "today missing: {toks:?}");
    }
}
