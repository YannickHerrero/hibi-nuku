mod api;
mod config;
mod error;
mod logging;

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
    let state = AppState {
        config: Arc::new(cfg),
    };
    let app = api::router(state);

    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("bind {addr}"))?;
    tracing::info!(addr = %addr, version = env!("CARGO_PKG_VERSION"), "hibi-nuku listening");
    axum::serve(listener, app).await?;
    Ok(())
}
