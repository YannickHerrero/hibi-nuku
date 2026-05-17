//! Prompt builder for batched subtitle translation.

use crate::library::lines::SubtitleLine;

/// Prompt version — changes invalidate `llm_cache`. Bump when the
/// system prompt or schema changes meaningfully.
pub const PROMPT_VERSION: &str = "2026-05-17.v1";

pub const SYSTEM: &str = "You are translating Japanese anime/show subtitles for a language learner. For each TARGET line, produce:\n- english: natural English translation\n- grammar_note: ONE short sentence (max 100 chars) explaining the key grammar point in the line. Empty string if nothing notable.\n- tone_tags: array from {keigo, casual_female, casual_male, rough_male, polite, kansai_ben, tohoku_ben, archaic, child_speech} or empty.\n\nUse the CONTEXT lines for pronoun/reference resolution but only output entries for TARGET lines.\n\nOutput strict JSON: { \"lines\": [{ \"idx\": <int>, \"english\": \"...\", \"grammar_note\": \"...\", \"tone_tags\": [...] }] }.";

/// Build the user message body for a chunk.
///
/// `before`/`after` are context lines (not translated); `targets` are
/// the ones the model must produce output for. `idx` field on each
/// target line matches `SubtitleLine.idx`.
pub fn user_message(
    before: &[&SubtitleLine],
    targets: &[&SubtitleLine],
    after: &[&SubtitleLine],
) -> String {
    let mut buf = String::new();
    if !before.is_empty() {
        buf.push_str("CONTEXT (before):\n");
        for l in before {
            buf.push_str(&format!("- {}\n", l.raw_text.replace('\n', " ")));
        }
        buf.push('\n');
    }
    buf.push_str("TARGETS:\n");
    for l in targets {
        buf.push_str(&format!(
            "[idx={}] {}\n",
            l.idx,
            l.raw_text.replace('\n', " ")
        ));
    }
    if !after.is_empty() {
        buf.push_str("\nCONTEXT (after):\n");
        for l in after {
            buf.push_str(&format!("- {}\n", l.raw_text.replace('\n', " ")));
        }
    }
    buf
}
