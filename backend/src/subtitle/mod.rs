#![allow(dead_code)]

pub mod ass;
pub mod extract;
pub mod srt;

/// One subtitle line as parsed from SRT/ASS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub idx: usize,
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}
