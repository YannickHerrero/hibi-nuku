mod api;
mod config;
mod db;
mod error;
mod library;
mod logging;
mod media;
mod subtitle;

use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;

use crate::api::AppState;
use crate::config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    logging::init();

    let cfg = Config::from_env()?;
    let addr = format!("{}:{}", cfg.host, cfg.port);

    let pool = db::connect(&cfg.db_path).await?;
    db::migrate(&pool).await?;

    let state = AppState {
        config: Arc::new(cfg),
        db: pool,
    };
    let app = api::router(state);

    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("bind {addr}"))?;
    tracing::info!(addr = %addr, version = env!("CARGO_PKG_VERSION"), "hibi-nuku listening");
    axum::serve(listener, app).await?;
    Ok(())
}
