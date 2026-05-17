//! Thin WaniKani API client (paginated subjects + /user).

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde::de::DeserializeOwned;
use tracing::debug;

use super::model::{DataItem, PageResponse, SubjectData, UserResponse};

const BASE: &str = "https://api.wanikani.com/v2";

pub struct WaniKani {
    http: Client,
    api_key: String,
}

impl WaniKani {
    pub fn new(api_key: String) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("reqwest"),
            api_key,
        }
    }

    pub async fn user(&self) -> Result<UserResponse> {
        self.get(&format!("{BASE}/user")).await
    }

    /// Fetch all subjects of `types` (e.g. "kanji" or "vocabulary"),
    /// following `pages.next_url` until exhausted.
    pub async fn all_subjects(&self, types: &str) -> Result<Vec<DataItem<SubjectData>>> {
        let mut url = format!("{BASE}/subjects?types={types}");
        let mut acc: Vec<DataItem<SubjectData>> = Vec::new();
        loop {
            debug!(url = %url, "WK fetch");
            let page: PageResponse<SubjectData> = self.get(&url).await?;
            acc.extend(page.data);
            match page.pages.next_url {
                Some(n) => url = n,
                None => break,
            }
        }
        Ok(acc)
    }

    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self
            .http
            .get(url)
            .bearer_auth(&self.api_key)
            .header("Wanikani-Revision", "20170710")
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        let status = resp.status();
        let text = resp.text().await.context("body")?;
        if !status.is_success() {
            return Err(anyhow!("WK {}: {}", status, truncate(&text, 200)));
        }
        serde_json::from_str(&text)
            .with_context(|| format!("decode {url}: {}", truncate(&text, 200)))
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() > n {
        format!("{}…", &s[..n])
    } else {
        s.to_string()
    }
}
