//! Thin Hibi API client.

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use serde_json::Value;

use super::model::{CardRequest, CreatedCard, KnownWordsResp, SessionRequest, UploadKey, WordStatusReq};

pub struct Hibi {
    base: String,
    http: Client,
    api_key: String,
}

impl Hibi {
    pub fn new(base: String, api_key: String) -> Self {
        Self {
            base,
            http: Client::builder()
                .timeout(std::time::Duration::from_secs(60))
                .build()
                .expect("reqwest"),
            api_key,
        }
    }

    pub async fn create_card(&self, req: &CardRequest) -> Result<CreatedCard> {
        self.post_json("/v1/cards", req).await
    }

    pub async fn upload_audio(&self, bytes: Vec<u8>, filename: &str, mime: &str) -> Result<UploadKey> {
        self.upload("/v1/uploads/audio", bytes, filename, mime).await
    }

    pub async fn upload_image(&self, bytes: Vec<u8>, filename: &str, mime: &str) -> Result<UploadKey> {
        self.upload("/v1/uploads/image", bytes, filename, mime).await
    }

    pub async fn known_words(&self) -> Result<KnownWordsResp> {
        self.get("/v1/known-words").await
    }

    pub async fn put_word_status(&self, req: &WordStatusReq) -> Result<Value> {
        self.put_json("/v1/word-status", req).await
    }

    pub async fn create_session(&self, req: &SessionRequest) -> Result<Value> {
        self.post_json("/v1/sessions", req).await
    }

    async fn upload(
        &self,
        path: &str,
        bytes: Vec<u8>,
        filename: &str,
        mime: &str,
    ) -> Result<UploadKey> {
        let url = format!("{}{}", self.base, path);
        let part = Part::bytes(bytes)
            .file_name(filename.to_string())
            .mime_str(mime)?;
        let form = Form::new().part("file", part);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .with_context(|| format!("upload {url}"))?;
        Self::decode(resp).await
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        Self::decode(resp).await
    }

    async fn post_json<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?;
        Self::decode(resp).await
    }

    async fn put_json<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .http
            .put(&url)
            .bearer_auth(&self.api_key)
            .json(body)
            .send()
            .await
            .with_context(|| format!("PUT {url}"))?;
        Self::decode(resp).await
    }

    async fn decode<T: DeserializeOwned>(resp: reqwest::Response) -> Result<T> {
        let status = resp.status();
        let text = resp.text().await.context("body")?;
        if !status.is_success() {
            return Err(anyhow!("hibi {}: {}", status, truncate(&text, 500)));
        }
        serde_json::from_str(&text).with_context(|| format!("decode: {}", truncate(&text, 500)))
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() > n {
        format!("{}…", &s[..n])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hibi::model::{FuriganaPair, KanjiEntry};

    #[test]
    fn card_request_serialises_to_expected_shape() {
        let req = CardRequest {
            sentence: "食べた".into(),
            focus_word: "食べる".into(),
            focus_word_reading: "たべる".into(),
            furigana: vec![FuriganaPair {
                base: "食".into(),
                reading: "た".into(),
            }],
            english: "ate".into(),
            glosses: vec!["to eat".into()],
            grammar_note: Some("past tense of ichidan verb".into()),
            kanji_list: vec![KanjiEntry {
                kanji: "食".into(),
                meaning: "Eat".into(),
                wanikani_level: Some(9),
            }],
            image_key: Some("k1".into()),
            audio_key: Some("k2".into()),
            source: "Frieren S01E03".into(),
            tags: vec!["anime".into(), "frieren".into()],
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["focusWord"], "食べる");
        assert_eq!(json["focusWordReading"], "たべる");
        assert_eq!(json["grammarNote"], "past tense of ichidan verb");
        assert_eq!(json["furigana"][0]["base"], "食");
        assert_eq!(json["kanjiList"][0]["wanikaniLevel"], 9);
        assert_eq!(json["imageKey"], "k1");
    }

    #[test]
    fn word_status_clear_serialises_as_null() {
        let req = WordStatusReq {
            lemma: "食べる".into(),
            reading: "たべる".into(),
            status: None,
        };
        let v = serde_json::to_value(&req).unwrap();
        assert!(v["status"].is_null());
    }
}
