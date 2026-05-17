//! Normalised JPDB-style frequency bundle and parser for the
//! Yomitan v3 `term_meta_bank_*.json` format.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub version: String,
    pub name: String,
    /// term → integer rank (lower = more frequent).
    pub freq: std::collections::HashMap<String, i64>,
    /// term → first known reading (optional metadata).
    pub readings: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct IndexJson {
    pub revision: String,
    pub title: String,
}

/// Parse a Yomitan term_meta_bank file body. Entries are arrays of
/// `[term, "freq", payload]` where payload is either an int, a string,
/// `{value:..., displayValue:...}`, or `{reading, frequency:{value}}`.
pub fn parse_term_meta_bank(json: &str) -> anyhow::Result<Vec<(String, Option<String>, i64)>> {
    let arr: Vec<Value> = serde_json::from_str(json)?;
    let mut out = Vec::with_capacity(arr.len());
    for row in arr {
        let Some(slice) = row.as_array() else {
            continue;
        };
        if slice.len() < 3 {
            continue;
        }
        let term = slice[0].as_str().unwrap_or("").to_string();
        if term.is_empty() {
            continue;
        }
        let kind = slice[1].as_str().unwrap_or("");
        if kind != "freq" {
            continue;
        }
        let (reading, value) = extract_freq(&slice[2]);
        if let Some(v) = value {
            out.push((term, reading, v));
        }
    }
    Ok(out)
}

fn extract_freq(v: &Value) -> (Option<String>, Option<i64>) {
    if let Some(n) = v.as_i64() {
        return (None, Some(n));
    }
    if let Some(s) = v.as_str() {
        return (None, s.parse().ok());
    }
    if let Some(obj) = v.as_object() {
        if let Some(reading) = obj.get("reading").and_then(|x| x.as_str()) {
            if let Some(freq_obj) = obj.get("frequency").and_then(|x| x.as_object()) {
                let value = freq_obj.get("value").and_then(|x| x.as_i64());
                return (Some(reading.to_string()), value);
            }
            if let Some(n) = obj.get("frequency").and_then(|x| x.as_i64()) {
                return (Some(reading.to_string()), Some(n));
            }
        }
        if let Some(n) = obj.get("value").and_then(|x| x.as_i64()) {
            return (None, Some(n));
        }
    }
    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bare_int_payload() {
        let json = r#"[["の","freq",{"value":1,"displayValue":"1"}]]"#;
        let v = parse_term_meta_bank(json).unwrap();
        assert_eq!(v, vec![("の".to_string(), None, 1)]);
    }

    #[test]
    fn parses_payload_with_reading() {
        let json = r#"[["思う","freq",{"reading":"おもう","frequency":{"value":20,"displayValue":"20"}}]]"#;
        let v = parse_term_meta_bank(json).unwrap();
        assert_eq!(v, vec![("思う".to_string(), Some("おもう".into()), 20)]);
    }

    #[test]
    fn skips_non_freq_entries() {
        let json = r#"[["X","other",{"value":99}]]"#;
        let v = parse_term_meta_bank(json).unwrap();
        assert!(v.is_empty());
    }
}
