mod config;

use anyhow::Result;

use crate::config::Config;

fn main() -> Result<()> {
    // dotenvy is best-effort in dev; in prod systemd provides the env.
    let _ = dotenvy::dotenv();
    let cfg = Config::from_env()?;
    println!(
        "hibi-nuku {} — would listen on {}:{} (db={})",
        env!("CARGO_PKG_VERSION"),
        cfg.host,
        cfg.port,
        cfg.db_path.display()
    );
    Ok(())
}
