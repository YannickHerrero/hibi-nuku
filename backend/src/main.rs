use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;

use hibi_nuku::api::{self, AppState};
use hibi_nuku::config::Config;
use hibi_nuku::jmdict;
use hibi_nuku::tokenize::jmdict_index::JmdictIndex;
use hibi_nuku::{db, logging};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    logging::init();

    let cfg = Config::from_env()?;
    let addr = format!("{}:{}", cfg.host, cfg.port);

    let pool = db::connect(&cfg.db_path).await?;
    db::migrate(&pool).await?;

    let jmdict_bundle = cfg.data_dir.join("jmdict.json.gz");
    let jmdict_index = if jmdict_bundle.exists() {
        let idx = jmdict::loader::load(&jmdict_bundle)?;
        tracing::info!(entries = idx.len(), path = %jmdict_bundle.display(), "loaded JMDict");
        idx
    } else {
        tracing::warn!(path = %jmdict_bundle.display(), "JMDict bundle missing — tokenizer will fall back to lindera only");
        JmdictIndex::new()
    };

    let state = AppState {
        config: Arc::new(cfg),
        db: pool,
        jmdict: Arc::new(jmdict_index),
    };
    let app = api::router(state);

    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("bind {addr}"))?;
    tracing::info!(addr = %addr, version = env!("CARGO_PKG_VERSION"), "hibi-nuku listening");
    axum::serve(listener, app).await?;
    Ok(())
}
