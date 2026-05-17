//! Minimal ASS/SSA parser. We only need the Dialogue events; the rest
//! (script info, styles, fonts) is skipped. Inline override blocks
//! (`{\an8}`, `{\fad(...)}`, etc.) and `\N` line breaks are stripped.

use anyhow::{Result, anyhow};

use super::Line;

pub fn parse(input: &str) -> Result<Vec<Line>> {
    let s = input.strip_prefix('\u{feff}').unwrap_or(input);
    let s = s.replace("\r\n", "\n").replace('\r', "\n");

    let mut idx_counter: usize = 0;
    let mut lines = Vec::new();

    // Find [Events] section; everything before is ignored.
    let events_pos = s
        .find("[Events]")
        .or_else(|| s.find("[events]"))
        .ok_or_else(|| anyhow!("no [Events] section"))?;
    let events = &s[events_pos..];

    // Find Format: line to discover column layout.
    let format_idx = events
        .lines()
        .position(|l| l.trim_start().starts_with("Format:"))
        .ok_or_else(|| anyhow!("no Format: line in [Events]"))?;
    let format_line = events.lines().nth(format_idx).unwrap();
    let columns: Vec<&str> = format_line
        .trim_start()
        .trim_start_matches("Format:")
        .split(',')
        .map(|s| s.trim())
        .collect();
    let start_col = columns
        .iter()
        .position(|c| c.eq_ignore_ascii_case("Start"))
        .ok_or_else(|| anyhow!("Format: missing Start column"))?;
    let end_col = columns
        .iter()
        .position(|c| c.eq_ignore_ascii_case("End"))
        .ok_or_else(|| anyhow!("Format: missing End column"))?;
    let text_col = columns
        .iter()
        .position(|c| c.eq_ignore_ascii_case("Text"))
        .ok_or_else(|| anyhow!("Format: missing Text column"))?;
    // Text is always the last column (per ASS spec it absorbs further commas).
    let n_fields = columns.len();

    for raw in events.lines().skip(format_idx + 1) {
        let raw = raw.trim_end();
        if !raw.starts_with("Dialogue:") {
            continue;
        }
        let payload = raw.trim_start().trim_start_matches("Dialogue:").trim();
        // splitn so the Text column keeps any commas it contains.
        let parts: Vec<&str> = payload.splitn(n_fields, ',').collect();
        if parts.len() < n_fields {
            continue;
        }
        let start = parse_ts(parts[start_col].trim())?;
        let end = parse_ts(parts[end_col].trim())?;
        let text = strip_tags(parts[text_col]);
        if text.is_empty() {
            continue;
        }
        lines.push(Line {
            idx: idx_counter,
            start_ms: start,
            end_ms: end,
            text,
        });
        idx_counter += 1;
    }

    // ASS dialogue rows aren't guaranteed to be time-ordered; sort by start.
    lines.sort_by_key(|l| l.start_ms);
    for (i, l) in lines.iter_mut().enumerate() {
        l.idx = i;
    }

    Ok(lines)
}

/// Strip inline override blocks `{...}` and convert `\N`/`\n` to '\n'.
/// Drawing commands (`{\p1}...{\p0}`) are removed wholesale.
fn strip_tags(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut drawing_mode = false;

    while let Some(c) = chars.next() {
        if c == '{' {
            // Look ahead for \p<digit> in this block to toggle drawing mode.
            let block_str: String = chars.by_ref().take_while(|c| *c != '}').collect();
            if let Some(pos) = block_str.find("\\p") {
                let rest = &block_str[pos + 2..];
                let digit = rest
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>();
                if let Ok(n) = digit.parse::<u32>() {
                    drawing_mode = n > 0;
                }
            }
            continue;
        }
        if drawing_mode {
            continue;
        }
        if c == '\\' {
            match chars.peek() {
                Some('N') | Some('n') => {
                    chars.next();
                    out.push('\n');
                    continue;
                }
                Some('h') => {
                    chars.next();
                    out.push(' ');
                    continue;
                }
                _ => {}
            }
        }
        out.push(c);
    }

    out.trim().to_string()
}

/// ASS timestamps: `H:MM:SS.cc` (centiseconds).
fn parse_ts(ts: &str) -> Result<i64> {
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() != 3 {
        return Err(anyhow!("invalid ASS timestamp `{ts}`"));
    }
    let h: i64 = parts[0].parse()?;
    let m: i64 = parts[1].parse()?;
    let (s, cs) = parts[2]
        .split_once('.')
        .ok_or_else(|| anyhow!("missing centiseconds in `{ts}`"))?;
    let s: i64 = s.parse()?;
    let cs: i64 = cs.parse()?;
    Ok(((h * 3600 + m * 60 + s) * 1000) + cs * 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIN: &str = "[Script Info]\nTitle: x\n\n[V4+ Styles]\nFormat: Name\nStyle: Default\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n";

    #[test]
    fn parses_basic_dialogue() {
        let mut s = MIN.to_string();
        s.push_str("Dialogue: 0,0:00:01.50,0:00:02.75,Default,,0,0,0,,Hello there\n");
        let lines = parse(&s).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].start_ms, 1500);
        assert_eq!(lines[0].end_ms, 2750);
        assert_eq!(lines[0].text, "Hello there");
    }

    #[test]
    fn strips_override_blocks_and_n() {
        let mut s = MIN.to_string();
        s.push_str("Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,{\\an8\\fad(300,300)}line\\Nbreak\n");
        let lines = parse(&s).unwrap();
        assert_eq!(lines[0].text, "line\nbreak");
    }

    #[test]
    fn drops_drawing_commands() {
        let mut s = MIN.to_string();
        s.push_str("Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,{\\p1}m 0 0 l 100 100{\\p0}only this\n");
        let lines = parse(&s).unwrap();
        assert_eq!(lines[0].text, "only this");
    }

    #[test]
    fn ignores_comments_and_non_dialogue() {
        let mut s = MIN.to_string();
        s.push_str("Comment: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,boring\n");
        s.push_str("Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,kept\n");
        let lines = parse(&s).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "kept");
    }

    #[test]
    fn sorts_by_start_time() {
        let mut s = MIN.to_string();
        s.push_str("Dialogue: 0,0:00:05.00,0:00:06.00,Default,,0,0,0,,B\n");
        s.push_str("Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,A\n");
        let lines = parse(&s).unwrap();
        assert_eq!(lines[0].text, "A");
        assert_eq!(lines[1].text, "B");
        assert_eq!(lines[0].idx, 0);
        assert_eq!(lines[1].idx, 1);
    }

    #[test]
    fn preserves_commas_in_text() {
        let mut s = MIN.to_string();
        s.push_str("Dialogue: 0,0:00:01.00,0:00:02.00,Default,,0,0,0,,well, hello, friend\n");
        let lines = parse(&s).unwrap();
        assert_eq!(lines[0].text, "well, hello, friend");
    }
}
