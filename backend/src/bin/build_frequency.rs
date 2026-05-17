//! CLI: convert a JPDB Yomitan-format frequency dictionary into our
//! normalized JSON bundle.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use flate2::Compression;
use flate2::write::GzEncoder;

use hibi_nuku::config;
use hibi_nuku::dict::frequency::{Bundle, IndexJson, parse_term_meta_bank};

#[derive(Parser, Debug)]
struct Args {
    /// Directory containing index.json + term_meta_bank_*.json.
    #[arg(short, long)]
    input: PathBuf,
    /// Output gzipped JSON bundle. Defaults to
    /// `$NUKU_DATA_DIR/frequency.json.gz`.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    let args = Args::parse();
    let index: IndexJson = serde_json::from_slice(
        &fs::read(args.input.join("index.json"))
            .with_context(|| format!("read {}/index.json", args.input.display()))?,
    )?;
    eprintln!("Source: {} ({})", index.title, index.revision);

    let mut freq: HashMap<String, i64> = HashMap::new();
    let mut readings: HashMap<String, String> = HashMap::new();
    for entry in fs::read_dir(&args.input)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_s = name.to_string_lossy();
        if !name_s.starts_with("term_meta_bank_") || !name_s.ends_with(".json") {
            continue;
        }
        eprintln!("  parsing {name_s}");
        let body = fs::read_to_string(entry.path())?;
        for (term, reading, rank) in parse_term_meta_bank(&body)? {
            freq.entry(term.clone()).or_insert(rank);
            if let Some(r) = reading {
                readings.entry(term).or_insert(r);
            }
        }
    }
    eprintln!("Total {} terms", freq.len());

    let bundle = Bundle {
        version: index.revision,
        name: index.title,
        freq,
        readings,
    };

    let output = args
        .output
        .unwrap_or_else(|| config::data_dir().join("frequency.json.gz"));
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let json = serde_json::to_vec(&bundle)?;
    let out = std::fs::File::create(&output)?;
    let mut enc = GzEncoder::new(out, Compression::best());
    enc.write_all(&json)?;
    enc.finish()?;

    let meta = std::fs::metadata(&output)?;
    eprintln!("Wrote {} ({} bytes)", output.display(), meta.len());
    Ok(())
}
