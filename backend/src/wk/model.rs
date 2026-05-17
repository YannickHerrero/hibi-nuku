//! Subset of the WaniKani subject schema we actually consume.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PageResponse<T> {
    pub data: Vec<DataItem<T>>,
    pub pages: Pages,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pages {
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DataItem<T> {
    pub id: i64,
    pub object: String,
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SubjectData {
    pub characters: Option<String>,
    pub level: i64,
    pub meanings: Vec<Meaning>,
    #[serde(default)]
    pub readings: Vec<Reading>,
    /// For vocabulary, the kanji it's composed of as `subject_id` ints.
    #[serde(default)]
    pub component_subject_ids: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Meaning {
    pub meaning: String,
    pub primary: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Reading {
    pub reading: String,
    #[serde(default)]
    pub primary: bool,
    #[serde(default, rename = "type")]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserResponse {
    pub data: UserData,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserData {
    pub level: i64,
    pub username: String,
}
