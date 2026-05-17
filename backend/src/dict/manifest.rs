//! Compute the dict manifest from files on disk (sha256 + size + mtime).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Clone)]
pub struct Manifest {
    pub jmdict: Option<Bundle>,
    pub wk: Option<Bundle>,
    pub frequency: Option<Bundle>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    pub url: String,
    pub size: u64,
    pub sha256: String,
    pub version: String,
}

pub fn compute(data_dir: &Path) -> Result<Manifest> {
    Ok(Manifest {
        jmdict: bundle_for(data_dir, "jmdict.json.gz", "/api/dict/jmdict")?,
        wk: bundle_for(data_dir, "wk.json.gz", "/api/dict/wk")?,
        frequency: bundle_for(data_dir, "frequency.json.gz", "/api/dict/frequency")?,
    })
}

fn bundle_for(data_dir: &Path, file: &str, url: &str) -> Result<Option<Bundle>> {
    let path = data_dir.join(file);
    if !path.exists() {
        return Ok(None);
    }
    let meta = std::fs::metadata(&path).with_context(|| format!("stat {}", path.display()))?;
    let size = meta.len();
    let mtime: DateTime<Utc> = meta.modified()?.into();
    let version = mtime.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut hasher = Sha256::new();
    let mut file = std::fs::File::open(&path).with_context(|| format!("open {}", path.display()))?;
    std::io::copy(&mut file, &mut hasher)?;
    let sha256 = hex::encode(hasher.finalize());
    Ok(Some(Bundle {
        url: url.to_string(),
        size,
        sha256,
        version,
    }))
}

pub fn bundle_path(data_dir: &Path, name: &str) -> Option<PathBuf> {
    let p = data_dir.join(name);
    if p.exists() { Some(p) } else { None }
}
