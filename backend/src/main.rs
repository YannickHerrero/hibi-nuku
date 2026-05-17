mod config;
mod logging;

use anyhow::Result;

use crate::config::Config;

fn main() -> Result<()> {
    let _ = dotenvy::dotenv();
    logging::init();
    let cfg = Config::from_env()?;
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        host = %cfg.host,
        port = cfg.port,
        db = %cfg.db_path.display(),
        "hibi-nuku boot"
    );
    Ok(())
}
