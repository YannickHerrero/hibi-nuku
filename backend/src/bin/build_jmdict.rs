//! CLI: parse JMDict_e XML → gzipped JSON bundle.
//!
//! Usage:
//!     cargo run --release --bin build-jmdict -- \
//!         --input data-sources/JMdict_e \
//!         --output backend/data/jmdict.json.gz

use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use flate2::Compression;
use flate2::write::GzEncoder;
use sha2::{Digest, Sha256};

use hibi_nuku::jmdict::bundle::Bundle;
use hibi_nuku::jmdict::parser;

#[derive(Parser, Debug)]
struct Args {
    /// JMdict_e XML file (.xml or .xml.gz).
    #[arg(short, long)]
    input: PathBuf,
    /// Output bundle path (will be gzipped).
    #[arg(short, long)]
    output: PathBuf,
    /// Override the version string (defaults to today's date YYYY-MM-DD).
    #[arg(long)]
    version: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let version = args
        .version
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

    eprintln!("Reading {}", args.input.display());
    let xml = parser::read_xml_to_string(&args.input)?;

    eprintln!("Parsing XML…");
    let entries = parser::parse(&xml)?;
    eprintln!("  {} entries", entries.len());

    let bundle = Bundle { version, entries };

    eprintln!("Encoding JSON…");
    let json = serde_json::to_vec(&bundle).context("encode JSON")?;

    eprintln!("Writing {}", args.output.display());
    if let Some(parent) = args.output.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let out = std::fs::File::create(&args.output)?;
    let mut enc = GzEncoder::new(out, Compression::best());
    enc.write_all(&json)?;
    enc.finish()?;

    let meta = std::fs::metadata(&args.output)?;

    let mut hasher = Sha256::new();
    let mut f = std::fs::File::open(&args.output)?;
    std::io::copy(&mut f, &mut hasher)?;
    let digest = hex::encode(hasher.finalize());

    eprintln!(
        "Wrote {} bytes (sha256={})",
        meta.len(),
        &digest[..12]
    );
    Ok(())
}
