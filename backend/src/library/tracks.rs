//! Selecting Japanese audio + subtitle tracks from an ffprobe result.
//!
//! Heuristics, in order:
//! 1. Stream with language tag `jpn` / `ja`.
//! 2. If multiple JP candidates, prefer the one whose title doesn't look
//!    like commentary (signs/songs/forced).
//! 3. Otherwise, no track selected — caller decides how to fall back.

use crate::media::{Stream, StreamKind};

#[derive(Debug)]
pub struct TrackSelection {
    pub audio_idx: Option<usize>,
    pub subtitle_idx: Option<usize>,
    pub subtitle_format: Option<String>,
}

pub fn select_japanese(streams: &[Stream]) -> TrackSelection {
    let audio_idx = pick_jp(streams, StreamKind::Audio).map(|s| s.index);
    let subtitle = pick_jp(streams, StreamKind::Subtitle);

    TrackSelection {
        audio_idx,
        subtitle_idx: subtitle.map(|s| s.index),
        subtitle_format: subtitle.map(|s| s.codec_name.clone()),
    }
}

fn pick_jp(streams: &[Stream], kind: StreamKind) -> Option<&Stream> {
    let candidates: Vec<&Stream> = streams
        .iter()
        .filter(|s| s.codec_type == kind && is_japanese(s))
        .collect();

    if candidates.is_empty() {
        return None;
    }

    // Prefer one that doesn't look like signs/songs/forced commentary.
    candidates
        .iter()
        .find(|s| !looks_like_alt(s))
        .copied()
        .or_else(|| candidates.first().copied())
}

fn is_japanese(s: &Stream) -> bool {
    matches!(s.language.as_deref(), Some("jpn") | Some("ja"))
}

fn looks_like_alt(s: &Stream) -> bool {
    let Some(title) = s.title.as_deref() else {
        return false;
    };
    let t = title.to_ascii_lowercase();
    ["sign", "song", "forced", "commentary", "karaoke"]
        .iter()
        .any(|k| t.contains(k))
}

/// `true` if the chosen subtitle format is image-based (we can't OCR
/// these in v1).
pub fn is_image_subtitle(format: &str) -> bool {
    matches!(
        format,
        "pgs" | "hdmv_pgs_subtitle" | "dvd_subtitle" | "dvb_subtitle" | "xsub"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(idx: usize, kind: StreamKind, lang: Option<&str>, title: Option<&str>) -> Stream {
        Stream {
            index: idx,
            codec_type: kind,
            codec_name: "ass".into(),
            language: lang.map(str::to_string),
            title: title.map(str::to_string),
        }
    }

    #[test]
    fn picks_first_jp_subtitle() {
        let streams = vec![
            s(0, StreamKind::Subtitle, Some("eng"), None),
            s(1, StreamKind::Subtitle, Some("jpn"), None),
            s(2, StreamKind::Subtitle, Some("jpn"), Some("signs/songs")),
        ];
        let sel = select_japanese(&streams);
        assert_eq!(sel.subtitle_idx, Some(1));
    }

    #[test]
    fn skips_signs_when_dialogue_available() {
        let streams = vec![
            s(0, StreamKind::Subtitle, Some("jpn"), Some("Signs & Songs")),
            s(1, StreamKind::Subtitle, Some("jpn"), Some("Dialogue")),
        ];
        let sel = select_japanese(&streams);
        assert_eq!(sel.subtitle_idx, Some(1));
    }

    #[test]
    fn falls_back_when_only_signs_present() {
        let streams = vec![s(
            0,
            StreamKind::Subtitle,
            Some("jpn"),
            Some("Signs & Songs"),
        )];
        let sel = select_japanese(&streams);
        assert_eq!(sel.subtitle_idx, Some(0));
    }

    #[test]
    fn flags_pgs_as_image() {
        assert!(is_image_subtitle("hdmv_pgs_subtitle"));
        assert!(!is_image_subtitle("ass"));
    }
}
