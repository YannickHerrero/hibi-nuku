//! JMDict parser and bundle format.
//!
//! The build-jmdict binary parses the raw `JMdict_e` XML once and
//! emits a compact gzipped JSON bundle. The server loads that bundle
//! at boot into `tokenize::jmdict_index::JmdictIndex`.

pub mod bundle;
pub mod loader;
pub mod parser;
