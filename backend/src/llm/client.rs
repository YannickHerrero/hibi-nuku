//! Minimal OpenRouter chat-completions client returning a JSON object.

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Single OpenAI-style message.
#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    response_format: ResponseFormat,
    temperature: f32,
}

#[derive(Debug, Clone, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChoiceContent>,
}

#[derive(Debug, Deserialize)]
struct ChoiceContent {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

pub struct OpenRouter {
    http: Client,
    api_key: String,
    referrer: String,
}

impl OpenRouter {
    pub fn new(api_key: String) -> Self {
        Self {
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("reqwest client"),
            api_key,
            referrer: "https://hibi-nuku.local".into(),
        }
    }

    /// Send `messages` and parse the assistant content as JSON.
    pub async fn complete_json(&self, model: &str, messages: &[ChatMessage]) -> Result<Value> {
        let body = ChatRequest {
            model,
            messages,
            response_format: ResponseFormat { kind: "json_object" },
            temperature: 0.2,
        };

        let resp = self
            .http
            .post(ENDPOINT)
            .bearer_auth(&self.api_key)
            .header("HTTP-Referer", &self.referrer)
            .header("X-Title", "Hibi Nuku")
            .json(&body)
            .send()
            .await
            .context("openrouter request")?;

        let status = resp.status();
        let text = resp.text().await.context("openrouter body")?;
        if !status.is_success() {
            return Err(anyhow!("openrouter {}: {}", status, truncate(&text, 500)));
        }

        let parsed: ChatResponse = serde_json::from_str(&text)
            .with_context(|| format!("decode openrouter response: {}", truncate(&text, 500)))?;
        let choice = parsed
            .choices
            .first()
            .ok_or_else(|| anyhow!("no choices in OpenRouter response"))?;
        let stripped = strip_code_fence(&choice.message.content);
        let value: Value = serde_json::from_str(stripped)
            .with_context(|| format!("decode model JSON: {}", truncate(stripped, 500)))?;
        Ok(value)
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() > n {
        format!("{}…", &s[..n])
    } else {
        s.to_string()
    }
}

/// Some models wrap JSON in ```json ... ``` fences even with JSON
/// mode requested. Strip them defensively.
fn strip_code_fence(s: &str) -> &str {
    let t = s.trim();
    let body = t
        .strip_prefix("```json")
        .or_else(|| t.strip_prefix("```JSON"))
        .or_else(|| t.strip_prefix("```"))
        .unwrap_or(t);
    body.strip_suffix("```").unwrap_or(body).trim()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_fence_with_lang() {
        assert_eq!(strip_code_fence("```json\n{\"a\":1}\n```"), "{\"a\":1}");
    }

    #[test]
    fn strips_bare_fence() {
        assert_eq!(strip_code_fence("```\n{\"a\":1}\n```"), "{\"a\":1}");
    }

    #[test]
    fn leaves_unwrapped_alone() {
        assert_eq!(strip_code_fence("{\"a\":1}"), "{\"a\":1}");
    }
}
