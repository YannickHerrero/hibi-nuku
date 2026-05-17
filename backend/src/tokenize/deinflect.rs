//! Yomitan-style suffix-rule deinflector ported to Rust.
//!
//! The full rule table is large; we embed a subset tuned to the
//! morphology our subtitles actually need. Each rule defines a
//! transformation `suffix_in → suffix_out` and the JMDict POS-rule
//! tags it produces, plus the tags it requires from previous
//! deinflections (chained).

use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct Rule {
    pub suffix_in: &'static str,
    pub suffix_out: &'static str,
    /// JMDict POS rule tags this rule produces (`v1`, `v5k`, `adj-i`, …).
    pub rules_out: &'static [&'static str],
    /// Tags that must be present in the input's rule set for this rule
    /// to apply. Empty means: applicable from any starting form.
    pub rules_in: &'static [&'static str],
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub stem: String,
    pub rules_out: HashSet<&'static str>,
}

/// Core inflection table — pragmatic subset of Yomitan's `deinflect.json`.
/// Order matters: longer suffixes first so we don't shortcut.
pub static RULES: &[Rule] = &[
    // -- copula / polite endings --
    Rule { suffix_in: "ました",   suffix_out: "ます",   rules_out: &["v5"], rules_in: &[] },
    Rule { suffix_in: "ません",   suffix_out: "ます",   rules_out: &["v5"], rules_in: &[] },
    Rule { suffix_in: "まして",   suffix_out: "ます",   rules_out: &["v5"], rules_in: &[] },
    Rule { suffix_in: "ますか",   suffix_out: "ます",   rules_out: &["v5"], rules_in: &[] },
    Rule { suffix_in: "ます",     suffix_out: "る",     rules_out: &["v1"], rules_in: &[] },

    // -- te / ta forms — ichidan --
    Rule { suffix_in: "て",       suffix_out: "る",     rules_out: &["v1"], rules_in: &[] },
    Rule { suffix_in: "た",       suffix_out: "る",     rules_out: &["v1"], rules_in: &[] },
    Rule { suffix_in: "ない",     suffix_out: "る",     rules_out: &["v1"], rules_in: &[] },
    Rule { suffix_in: "なかった", suffix_out: "る",     rules_out: &["v1"], rules_in: &[] },
    Rule { suffix_in: "ます",     suffix_out: "る",     rules_out: &["v1"], rules_in: &[] },

    // -- godan -ku --
    Rule { suffix_in: "いた",     suffix_out: "く",     rules_out: &["v5k"], rules_in: &[] },
    Rule { suffix_in: "いて",     suffix_out: "く",     rules_out: &["v5k"], rules_in: &[] },
    Rule { suffix_in: "かない",   suffix_out: "く",     rules_out: &["v5k"], rules_in: &[] },
    Rule { suffix_in: "きます",   suffix_out: "く",     rules_out: &["v5k"], rules_in: &[] },
    Rule { suffix_in: "かなかった", suffix_out: "く",   rules_out: &["v5k"], rules_in: &[] },

    // -- godan -gu --
    Rule { suffix_in: "いだ",     suffix_out: "ぐ",     rules_out: &["v5g"], rules_in: &[] },
    Rule { suffix_in: "いで",     suffix_out: "ぐ",     rules_out: &["v5g"], rules_in: &[] },
    Rule { suffix_in: "がない",   suffix_out: "ぐ",     rules_out: &["v5g"], rules_in: &[] },
    Rule { suffix_in: "ぎます",   suffix_out: "ぐ",     rules_out: &["v5g"], rules_in: &[] },

    // -- godan -su --
    Rule { suffix_in: "した",     suffix_out: "す",     rules_out: &["v5s"], rules_in: &[] },
    Rule { suffix_in: "して",     suffix_out: "す",     rules_out: &["v5s"], rules_in: &[] },
    Rule { suffix_in: "さない",   suffix_out: "す",     rules_out: &["v5s"], rules_in: &[] },
    Rule { suffix_in: "します",   suffix_out: "す",     rules_out: &["v5s"], rules_in: &[] },

    // -- godan -tsu / -ru / -u (regular t-base) --
    Rule { suffix_in: "った",     suffix_out: "つ",     rules_out: &["v5t"], rules_in: &[] },
    Rule { suffix_in: "って",     suffix_out: "つ",     rules_out: &["v5t"], rules_in: &[] },
    Rule { suffix_in: "たない",   suffix_out: "つ",     rules_out: &["v5t"], rules_in: &[] },
    Rule { suffix_in: "ちます",   suffix_out: "つ",     rules_out: &["v5t"], rules_in: &[] },

    Rule { suffix_in: "った",     suffix_out: "る",     rules_out: &["v5r"], rules_in: &[] },
    Rule { suffix_in: "って",     suffix_out: "る",     rules_out: &["v5r"], rules_in: &[] },
    Rule { suffix_in: "らない",   suffix_out: "る",     rules_out: &["v5r"], rules_in: &[] },
    Rule { suffix_in: "ります",   suffix_out: "る",     rules_out: &["v5r"], rules_in: &[] },

    Rule { suffix_in: "った",     suffix_out: "う",     rules_out: &["v5u"], rules_in: &[] },
    Rule { suffix_in: "って",     suffix_out: "う",     rules_out: &["v5u"], rules_in: &[] },
    Rule { suffix_in: "わない",   suffix_out: "う",     rules_out: &["v5u"], rules_in: &[] },
    Rule { suffix_in: "います",   suffix_out: "う",     rules_out: &["v5u"], rules_in: &[] },

    // -- godan -bu / -mu / -nu --
    Rule { suffix_in: "んだ",     suffix_out: "ぶ",     rules_out: &["v5b"], rules_in: &[] },
    Rule { suffix_in: "んで",     suffix_out: "ぶ",     rules_out: &["v5b"], rules_in: &[] },
    Rule { suffix_in: "ばない",   suffix_out: "ぶ",     rules_out: &["v5b"], rules_in: &[] },
    Rule { suffix_in: "びます",   suffix_out: "ぶ",     rules_out: &["v5b"], rules_in: &[] },

    Rule { suffix_in: "んだ",     suffix_out: "む",     rules_out: &["v5m"], rules_in: &[] },
    Rule { suffix_in: "んで",     suffix_out: "む",     rules_out: &["v5m"], rules_in: &[] },
    Rule { suffix_in: "まない",   suffix_out: "む",     rules_out: &["v5m"], rules_in: &[] },
    Rule { suffix_in: "みます",   suffix_out: "む",     rules_out: &["v5m"], rules_in: &[] },

    Rule { suffix_in: "んだ",     suffix_out: "ぬ",     rules_out: &["v5n"], rules_in: &[] },
    Rule { suffix_in: "んで",     suffix_out: "ぬ",     rules_out: &["v5n"], rules_in: &[] },
    Rule { suffix_in: "なない",   suffix_out: "ぬ",     rules_out: &["v5n"], rules_in: &[] },
    Rule { suffix_in: "にます",   suffix_out: "ぬ",     rules_out: &["v5n"], rules_in: &[] },

    // -- suru and kuru irregulars --
    Rule { suffix_in: "しました", suffix_out: "する",   rules_out: &["vs-i"], rules_in: &[] },
    Rule { suffix_in: "して",     suffix_out: "する",   rules_out: &["vs-i"], rules_in: &[] },
    Rule { suffix_in: "した",     suffix_out: "する",   rules_out: &["vs-i"], rules_in: &[] },
    Rule { suffix_in: "しない",   suffix_out: "する",   rules_out: &["vs-i"], rules_in: &[] },
    Rule { suffix_in: "します",   suffix_out: "する",   rules_out: &["vs-i"], rules_in: &[] },

    Rule { suffix_in: "来た",     suffix_out: "来る",   rules_out: &["vk"], rules_in: &[] },
    Rule { suffix_in: "来て",     suffix_out: "来る",   rules_out: &["vk"], rules_in: &[] },
    Rule { suffix_in: "きた",     suffix_out: "くる",   rules_out: &["vk"], rules_in: &[] },
    Rule { suffix_in: "きて",     suffix_out: "くる",   rules_out: &["vk"], rules_in: &[] },

    // -- i-adjectives --
    Rule { suffix_in: "かった",   suffix_out: "い",     rules_out: &["adj-i"], rules_in: &[] },
    Rule { suffix_in: "くない",   suffix_out: "い",     rules_out: &["adj-i"], rules_in: &[] },
    Rule { suffix_in: "くて",     suffix_out: "い",     rules_out: &["adj-i"], rules_in: &[] },
];

/// Return all deinflection candidates for `surface`, including the
/// identity candidate (the surface itself with no rule requirements).
/// Single-step; chaining is rare in our domain and the cost-to-value
/// of full Yomitan-style chaining isn't worth it in v1.
pub fn deinflect(surface: &str) -> Vec<Candidate> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let identity = Candidate {
        stem: surface.to_string(),
        rules_out: HashSet::new(),
    };
    seen.insert(identity.stem.clone());
    out.push(identity);

    for rule in RULES {
        if let Some(prefix) = surface.strip_suffix(rule.suffix_in) {
            let stem = format!("{prefix}{}", rule.suffix_out);
            if seen.insert(stem.clone()) {
                let mut rules_out = HashSet::new();
                for tag in rule.rules_out {
                    rules_out.insert(*tag);
                }
                out.push(Candidate { stem, rules_out });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_stem(cands: &[Candidate], stem: &str) -> bool {
        cands.iter().any(|c| c.stem == stem)
    }

    #[test]
    fn taberu_past() {
        let cands = deinflect("食べた");
        assert!(has_stem(&cands, "食べる"), "got: {cands:?}");
    }

    #[test]
    fn ikimasu_polite() {
        let cands = deinflect("行きます");
        assert!(has_stem(&cands, "行く"), "got: {cands:?}");
    }

    #[test]
    fn wakaranai_negative() {
        let cands = deinflect("分からない");
        assert!(has_stem(&cands, "分かる"), "got: {cands:?}");
    }

    #[test]
    fn omoshirokatta() {
        let cands = deinflect("面白かった");
        assert!(has_stem(&cands, "面白い"), "got: {cands:?}");
    }

    #[test]
    fn identity_always_present() {
        let cands = deinflect("猫");
        assert!(has_stem(&cands, "猫"));
    }
}
