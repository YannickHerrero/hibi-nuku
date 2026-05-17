//! CLI: refresh the local WaniKani cache from the API.

use anyhow::Result;

use hibi_nuku::config::Config;
use hibi_nuku::db;
use hibi_nuku::wk::import;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    hibi_nuku::logging::init();

    let cfg = Config::from_env()?;
    let pool = db::connect(&cfg.db_path).await?;
    db::migrate(&pool).await?;

    let summary = import::run(&pool, cfg.wanikani_api_key.clone()).await?;
    println!(
        "wk-import: user_level={} kanji={} vocab={}",
        summary.user_level, summary.kanji, summary.vocab
    );
    Ok(())
}
