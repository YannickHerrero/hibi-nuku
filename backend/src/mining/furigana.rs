//! Align a kanji-bearing lemma against its kana reading and emit a
//! `[{base, reading}]` array suitable for the Hibi card payload.
//!
//! Strategy: split the word into runs of (kanji-only) vs (kana-only).
//! Compute kana boundaries against the reading by matching the kana
//! runs from both ends; everything in between belongs to the kanji
//! run. Non-kanji-bearing words emit a single pair with empty reading.

use crate::hibi::model::FuriganaPair;

/// Build furigana pairs for `lemma` aligned to `reading` (hiragana or
/// katakana; we lower-case katakana → hiragana before matching).
pub fn build(lemma: &str, reading: &str) -> Vec<FuriganaPair> {
    if !has_kanji(lemma) {
        return vec![FuriganaPair {
            base: lemma.to_string(),
            reading: String::new(),
        }];
    }
    let reading_h = to_hiragana(reading);

    // Split the lemma into runs.
    let runs = split_runs(lemma);
    if runs.is_empty() {
        return vec![FuriganaPair {
            base: lemma.to_string(),
            reading: reading_h,
        }];
    }

    // Single kanji run with no kana on either side: simple case.
    if runs.len() == 1 && runs[0].kind == Kind::Kanji {
        return vec![FuriganaPair {
            base: lemma.to_string(),
            reading: reading_h,
        }];
    }

    // Walk runs; for each kana run, consume matching kana from the
    // reading. For each kanji run, take everything until the next kana
    // run matches.
    let reading_chars: Vec<char> = reading_h.chars().collect();
    let mut r = 0usize;
    let mut out: Vec<FuriganaPair> = Vec::with_capacity(runs.len());

    for (i, run) in runs.iter().enumerate() {
        match run.kind {
            Kind::Kana => {
                let want_h = to_hiragana(&run.text);
                let want_chars: Vec<char> = want_h.chars().collect();
                if reading_chars[r..].starts_with(&want_chars) {
                    r += want_chars.len();
                } else {
                    // Fallback: dump remaining reading on this run with
                    // empty reading; preserve text.
                }
                out.push(FuriganaPair {
                    base: run.text.clone(),
                    reading: String::new(),
                });
            }
            Kind::Kanji => {
                // Look ahead for the next kana run, then find it in the
                // remaining reading to bound this kanji run's reading.
                let next_kana_h: Option<String> = runs
                    .iter()
                    .skip(i + 1)
                    .find(|r| r.kind == Kind::Kana)
                    .map(|r| to_hiragana(&r.text));
                let end = match next_kana_h {
                    Some(want) => {
                        let want_chars: Vec<char> = want.chars().collect();
                        let mut found = None;
                        for j in r..=reading_chars.len().saturating_sub(want_chars.len()) {
                            if reading_chars[j..].starts_with(&want_chars) {
                                found = Some(j);
                                break;
                            }
                        }
                        found.unwrap_or(reading_chars.len())
                    }
                    None => reading_chars.len(),
                };
                let segment: String = reading_chars[r..end].iter().collect();
                r = end;
                out.push(FuriganaPair {
                    base: run.text.clone(),
                    reading: segment,
                });
            }
        }
    }
    out
}

#[derive(Debug, PartialEq, Eq)]
enum Kind {
    Kanji,
    Kana,
}

struct Run {
    text: String,
    kind: Kind,
}

fn split_runs(s: &str) -> Vec<Run> {
    let mut runs = Vec::new();
    let mut cur = String::new();
    let mut cur_kind: Option<Kind> = None;
    for ch in s.chars() {
        let kind = if is_kanji(ch) { Kind::Kanji } else { Kind::Kana };
        match &cur_kind {
            Some(k) if *k == kind => cur.push(ch),
            _ => {
                if !cur.is_empty() {
                    runs.push(Run {
                        text: std::mem::take(&mut cur),
                        kind: cur_kind.take().unwrap(),
                    });
                }
                cur.push(ch);
                cur_kind = Some(kind);
            }
        }
    }
    if !cur.is_empty() {
        runs.push(Run {
            text: cur,
            kind: cur_kind.unwrap(),
        });
    }
    runs
}

fn is_kanji(c: char) -> bool {
    matches!(c as u32,
        0x4E00..=0x9FFF
        | 0x3400..=0x4DBF
        | 0x20000..=0x2A6DF
    )
}

fn has_kanji(s: &str) -> bool {
    s.chars().any(is_kanji)
}

fn to_hiragana(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        let code = c as u32;
        if (0x30A1..=0x30F6).contains(&code) {
            out.push(char::from_u32(code - 0x60).unwrap_or(c));
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taberu_splits_kanji_kana() {
        let pairs = build("食べる", "たべる");
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].base, "食");
        assert_eq!(pairs[0].reading, "た");
        assert_eq!(pairs[1].base, "べる");
        assert_eq!(pairs[1].reading, "");
    }

    #[test]
    fn all_kanji_word_keeps_full_reading() {
        let pairs = build("学校", "がっこう");
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].base, "学校");
        assert_eq!(pairs[0].reading, "がっこう");
    }

    #[test]
    fn pure_kana_word_has_empty_reading_field() {
        let pairs = build("ねこ", "ねこ");
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].reading, "");
    }

    #[test]
    fn katakana_reading_normalised_to_hiragana_match() {
        let pairs = build("行く", "イク");
        assert_eq!(pairs[0].reading, "い");
    }

    #[test]
    fn kana_prefix_then_kanji() {
        let pairs = build("お茶", "おちゃ");
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0].base, "お");
        assert_eq!(pairs[1].base, "茶");
        assert_eq!(pairs[1].reading, "ちゃ");
    }
}
