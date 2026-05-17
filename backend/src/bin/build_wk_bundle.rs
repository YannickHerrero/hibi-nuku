//! CLI: build the gzipped JSON WK bundle from the cached tables.

use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use flate2::Compression;
use flate2::write::GzEncoder;

use hibi_nuku::config::Config;
use hibi_nuku::db;
use hibi_nuku::wk::bundle;

#[derive(Parser, Debug)]
struct Args {
    /// Output path. Defaults to `$NUKU_DATA_DIR/wk.json.gz` if set,
    /// otherwise `backend/data/wk.json.gz` relative to CWD.
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    hibi_nuku::logging::init();
    let args = Args::parse();

    let cfg = Config::from_env()?;
    let pool = db::connect(&cfg.db_path).await?;
    db::migrate(&pool).await?;

    eprintln!("Reading WK tables…");
    let bundle = bundle::build(&pool).await?;
    eprintln!(
        "  user_level={:?} kanji={} vocab={}",
        bundle.user_level,
        bundle.kanji.len(),
        bundle.vocab.len()
    );

    let output = args
        .output
        .unwrap_or_else(|| cfg.data_dir.join("wk.json.gz"));
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
