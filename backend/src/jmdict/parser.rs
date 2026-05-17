//! Streaming JMDict_e XML parser.
//!
//! The XML uses entity references (`&v1;` → `verb (ichidan)`) defined
//! in the DOCTYPE. We *don't* want those expanded; we keep the entity
//! name as the POS code (e.g. `v1`), since that's what JMDict
//! consumers like Yomitan use. To avoid entity-resolution errors with
//! quick-xml, we strip the DOCTYPE block before parsing.

use std::collections::HashSet;
use std::io::Read;

use anyhow::{Context, Result, anyhow};
use quick_xml::events::{BytesText, Event};
use quick_xml::reader::Reader;

use super::bundle::{Entry, Sense};

/// Parse the JMDict_e XML body into entries.
pub fn parse(xml: &str) -> Result<Vec<Entry>> {
    let body = strip_doctype(xml);
    let mut reader = Reader::from_str(body);
    let cfg = reader.config_mut();
    cfg.trim_text(true);
    cfg.expand_empty_elements = true;
    cfg.check_end_names = true;

    let mut entries = Vec::with_capacity(200_000);
    let mut buf = Vec::new();
    let mut state = State::default();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => {
                return Err(anyhow!("xml at pos {}: {e}", reader.buffer_position()));
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"entry" => state = State::default(),
                b"k_ele" => state.in_kanji = true,
                b"r_ele" => state.in_reading = true,
                b"sense" => {
                    state.in_sense = true;
                    state.current_sense_pos.clear();
                    state.current_sense_glosses.clear();
                }
                b"gloss" => state.collecting_gloss = true,
                b"keb" | b"reb" | b"pos" => state.collecting_text = true,
                _ => {}
            },
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"entry" => {
                    if let Some(seq) = state.seq.take() {
                        entries.push(Entry {
                            seq,
                            kanji: std::mem::take(&mut state.kanji),
                            readings: std::mem::take(&mut state.readings),
                            senses: std::mem::take(&mut state.senses),
                            rules: std::mem::take(&mut state.rules_seen)
                                .into_iter()
                                .collect(),
                        });
                    }
                }
                b"k_ele" => state.in_kanji = false,
                b"r_ele" => state.in_reading = false,
                b"sense" => {
                    state.senses.push(Sense {
                        pos: std::mem::take(&mut state.current_sense_pos),
                        glosses: std::mem::take(&mut state.current_sense_glosses),
                    });
                    state.in_sense = false;
                }
                b"gloss" => state.collecting_gloss = false,
                b"keb" | b"reb" | b"pos" => state.collecting_text = false,
                _ => {}
            },
            Ok(Event::Text(t)) => handle_text(&mut state, &t)?,
            Ok(Event::GeneralRef(reference)) => {
                let name = std::str::from_utf8(reference.as_ref())
                    .with_context(|| "non-utf8 entity reference")?
                    .to_string();
                if state.collecting_text && state.in_sense {
                    state.current_sense_pos.push(name.clone());
                    state.rules_seen.insert(name);
                }
            }
            Ok(_) => {}
        }
        buf.clear();
    }

    Ok(entries)
}

/// Read XML from gzip if filename ends with `.gz`, otherwise raw.
pub fn read_xml_to_string(path: &std::path::Path) -> Result<String> {
    let file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    if path.extension().is_some_and(|e| e == "gz") {
        let mut decoder = flate2::read::GzDecoder::new(file);
        let mut s = String::new();
        decoder.read_to_string(&mut s).context("gunzip JMdict")?;
        Ok(s)
    } else {
        let mut s = String::new();
        std::io::BufReader::new(file)
            .read_to_string(&mut s)
            .context("read JMdict")?;
        Ok(s)
    }
}

#[derive(Default)]
struct State {
    seq: Option<i64>,
    kanji: Vec<String>,
    readings: Vec<String>,
    senses: Vec<Sense>,
    rules_seen: HashSet<String>,

    in_kanji: bool,
    in_reading: bool,
    in_sense: bool,
    collecting_text: bool,
    collecting_gloss: bool,

    current_sense_pos: Vec<String>,
    current_sense_glosses: Vec<String>,
}

fn handle_text(state: &mut State, t: &BytesText<'_>) -> Result<()> {
    if !(state.collecting_text || state.collecting_gloss) && state.seq.is_some() {
        return Ok(());
    }
    let text = t.decode().with_context(|| "decode text")?.into_owned();

    // ent_seq is the first text inside <entry>; we don't get a Start
    // for it because we route via collecting_text on `keb`/`reb`/`pos`
    // — so handle it explicitly when we see digits-only text at the
    // top of an entry.
    if state.seq.is_none() {
        if let Ok(n) = text.trim().parse::<i64>() {
            state.seq = Some(n);
            return Ok(());
        }
    }

    if state.collecting_gloss {
        state.current_sense_glosses.push(text);
        return Ok(());
    }
    if state.collecting_text {
        if state.in_kanji {
            state.kanji.push(text);
        } else if state.in_reading {
            state.readings.push(text);
        }
        // POS body text never appears (those are entity refs); ignored.
    }
    Ok(())
}

fn strip_doctype(xml: &str) -> &str {
    if let Some(start) = xml.find("<!DOCTYPE") {
        if let Some(end) = xml[start..].find("]>") {
            let after = start + end + 2;
            return &xml[after..];
        }
        if let Some(end) = xml[start..].find('>') {
            return &xml[start + end + 1..];
        }
    }
    xml
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE JMdict [
<!ENTITY v1 "Ichidan verb">
<!ENTITY vt "transitive verb">
<!ENTITY n "noun">
]>
<JMdict>
  <entry>
    <ent_seq>1577985</ent_seq>
    <k_ele><keb>食べる</keb></k_ele>
    <r_ele><reb>たべる</reb></r_ele>
    <sense>
      <pos>&v1;</pos>
      <pos>&vt;</pos>
      <gloss>to eat</gloss>
      <gloss>to live on (e.g. a salary)</gloss>
    </sense>
  </entry>
  <entry>
    <ent_seq>1579110</ent_seq>
    <k_ele><keb>今日</keb></k_ele>
    <r_ele><reb>きょう</reb></r_ele>
    <sense>
      <pos>&n;</pos>
      <gloss>today</gloss>
    </sense>
  </entry>
</JMdict>
"#;

    #[test]
    fn parses_two_entries() {
        let entries = parse(SAMPLE).unwrap();
        assert_eq!(entries.len(), 2);
        let taberu = &entries[0];
        assert_eq!(taberu.seq, 1577985);
        assert_eq!(taberu.kanji, vec!["食べる".to_string()]);
        assert_eq!(taberu.readings, vec!["たべる".to_string()]);
        assert_eq!(taberu.senses.len(), 1);
        assert_eq!(taberu.senses[0].pos, vec!["v1", "vt"]);
        assert_eq!(taberu.senses[0].glosses.len(), 2);
        assert!(taberu.rules.contains(&"v1".to_string()));
    }

    #[test]
    fn second_entry_noun() {
        let entries = parse(SAMPLE).unwrap();
        assert_eq!(entries[1].seq, 1579110);
        assert_eq!(entries[1].senses[0].glosses, vec!["today".to_string()]);
    }
}
