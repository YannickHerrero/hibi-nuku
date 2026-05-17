//! Minimal SRT parser. Tolerates BOM, CRLF, missing/extra blank lines,
//! and the "WEBVTT" header (treats `.vtt` as SRT-like for now).

use anyhow::{Result, anyhow};

use super::Line;

pub fn parse(input: &str) -> Result<Vec<Line>> {
    let s = input.strip_prefix('\u{feff}').unwrap_or(input);

    let mut lines: Vec<Line> = Vec::new();
    let mut idx_counter: usize = 0;

    // Normalise to LF for splitting.
    let normalised = s.replace("\r\n", "\n").replace('\r', "\n");

    // WEBVTT header: skip everything up to the first blank line
    // (header block contains "WEBVTT" then optional metadata).
    let body = if normalised.trim_start().starts_with("WEBVTT") {
        normalised
            .split_once("\n\n")
            .map(|(_, rest)| rest.to_string())
            .unwrap_or_default()
    } else {
        normalised
    };

    for block in body.split("\n\n") {
        let block = block.trim_matches('\n');
        if block.is_empty() {
            continue;
        }

        let mut iter = block.split('\n');
        // First line may be a numeric counter or directly the timestamp.
        let first = iter.next().ok_or_else(|| anyhow!("empty block"))?;
        let timing = if first.contains("-->") {
            first
        } else {
            iter.next()
                .ok_or_else(|| anyhow!("expected timestamp line"))?
        };

        let (start_ms, end_ms) = parse_timing(timing)?;
        let text = iter.collect::<Vec<_>>().join("\n");
        let text = text.trim().to_string();

        lines.push(Line {
            idx: idx_counter,
            start_ms,
            end_ms,
            text,
        });
        idx_counter += 1;
    }

    Ok(lines)
}

fn parse_timing(line: &str) -> Result<(i64, i64)> {
    let line = line.trim();
    let (lhs, rhs) = line
        .split_once("-->")
        .ok_or_else(|| anyhow!("missing '-->' in `{line}`"))?;
    // VTT cues sometimes have trailing settings after the end time
    // (e.g. "00:01:23.456 --> 00:01:25.789 align:start position:50%").
    let rhs_ts = rhs.split_whitespace().next().unwrap_or("");
    Ok((parse_ts(lhs.trim())?, parse_ts(rhs_ts)?))
}

/// Accept `HH:MM:SS,mmm` (SRT) or `HH:MM:SS.mmm` (VTT). Hours optional.
fn parse_ts(ts: &str) -> Result<i64> {
    let ts = ts.trim().replace(',', ".");
    let mut parts: Vec<&str> = ts.split(':').collect();
    while parts.len() < 3 {
        parts.insert(0, "0");
    }
    if parts.len() != 3 {
        return Err(anyhow!("invalid timestamp `{ts}`"));
    }
    let h: i64 = parts[0].parse().map_err(|e| anyhow!("hours: {e}"))?;
    let m: i64 = parts[1].parse().map_err(|e| anyhow!("minutes: {e}"))?;
    let secs_part = parts[2];

    let (s, ms) = match secs_part.split_once('.') {
        Some((s, ms)) => {
            let s: i64 = s.parse().map_err(|e| anyhow!("seconds: {e}"))?;
            // ms may be 1-3 digits; pad/truncate to 3.
            let ms_str = if ms.len() >= 3 { &ms[..3] } else { ms };
            let mut ms_v: i64 = ms_str.parse().unwrap_or(0);
            if ms.len() == 1 {
                ms_v *= 100;
            } else if ms.len() == 2 {
                ms_v *= 10;
            }
            (s, ms_v)
        }
        None => (
            secs_part.parse().map_err(|e| anyhow!("seconds: {e}"))?,
            0,
        ),
    };
    Ok(((h * 3600 + m * 60 + s) * 1000) + ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_two_cues() {
        let input = "1\n00:00:01,000 --> 00:00:02,500\nHello\n\n2\n00:00:03,000 --> 00:00:04,000\nWorld";
        let lines = parse(input).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].start_ms, 1000);
        assert_eq!(lines[0].end_ms, 2500);
        assert_eq!(lines[0].text, "Hello");
        assert_eq!(lines[1].text, "World");
    }

    #[test]
    fn handles_bom_and_crlf() {
        let input = "\u{feff}1\r\n00:00:01,000 --> 00:00:02,000\r\n日本語\r\n\r\n";
        let lines = parse(input).unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "日本語");
    }

    #[test]
    fn multi_line_text() {
        let input = "1\n00:00:01,000 --> 00:00:02,000\nline one\nline two";
        let lines = parse(input).unwrap();
        assert_eq!(lines[0].text, "line one\nline two");
    }

    #[test]
    fn accepts_vtt_dot_separator_and_header() {
        let input = "WEBVTT\n\n00:00:01.500 --> 00:00:02.750\nhi\n\n";
        let lines = parse(input).unwrap();
        assert_eq!(lines[0].start_ms, 1500);
        assert_eq!(lines[0].end_ms, 2750);
        assert_eq!(lines[0].text, "hi");
    }

    #[test]
    fn extra_blank_lines_between_cues() {
        let input = "1\n00:00:01,000 --> 00:00:02,000\nA\n\n\n\n2\n00:00:03,000 --> 00:00:04,000\nB";
        let lines = parse(input).unwrap();
        assert_eq!(lines.len(), 2);
    }
}
