use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Context, Result, bail};

/// All runtime configuration is materialised once on startup.
///
/// Required values fail-fast with a descriptive error so a misconfigured
/// server never boots into a half-working state.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub db_path: PathBuf,
    pub library_dir: PathBuf,
    pub token: String,
    pub hibi_base: String,
    pub hibi_api_key: String,
    pub openrouter_api_key: String,
    pub llm_model: String,
    pub wanikani_api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            host: env_or("NUKU_HOST", "0.0.0.0"),
            port: env_parse("NUKU_PORT", 8787)?,
            db_path: env_path("NUKU_DB_PATH", "backend/data/dev.db"),
            library_dir: env_path("NUKU_LIBRARY_DIR", "/srv/hibi-nuku/library"),
            token: require_env("NUKU_TOKEN")?,
            hibi_base: env_or("HIBI_API_BASE", "https://hibi-api.vercel.app"),
            hibi_api_key: require_env("HIBI_API_KEY")?,
            openrouter_api_key: require_env("OPENROUTER_API_KEY")?,
            llm_model: env_or("NUKU_LLM_MODEL", "anthropic/claude-sonnet-4.5"),
            wanikani_api_key: require_env("WANIKANI_API_KEY")?,
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_path(key: &str, default: &str) -> PathBuf {
    PathBuf::from(env_or(key, default))
}

fn env_parse<T: FromStr>(key: &str, default: T) -> Result<T>
where
    <T as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    match std::env::var(key) {
        Ok(v) => v
            .parse::<T>()
            .with_context(|| format!("invalid value for {key}: {v}")),
        Err(_) => Ok(default),
    }
}

fn require_env(key: &str) -> Result<String> {
    match std::env::var(key) {
        Ok(v) if !v.trim().is_empty() => Ok(v),
        _ => bail!("missing required env var: {key}"),
    }
}
