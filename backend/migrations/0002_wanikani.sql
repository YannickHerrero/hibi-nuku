-- WaniKani cache. Populated by `wk-import` CLI, exported via build-wk-bundle.

CREATE TABLE wk_kanji (
  characters       TEXT PRIMARY KEY,
  level            INTEGER NOT NULL,
  primary_meaning  TEXT NOT NULL,
  meanings_json    TEXT NOT NULL,
  primary_reading  TEXT NOT NULL,
  reading_type     TEXT NOT NULL,        -- onyomi|kunyomi|nanori
  readings_json    TEXT NOT NULL,
  raw_json         TEXT NOT NULL
);

CREATE TABLE wk_vocab (
  characters         TEXT PRIMARY KEY,
  level              INTEGER NOT NULL,
  meanings_json      TEXT NOT NULL,
  readings_json      TEXT NOT NULL,
  kanji_chars_json   TEXT NOT NULL
);

CREATE TABLE wk_kanji_to_vocab (
  kanji TEXT NOT NULL,
  vocab TEXT NOT NULL,
  PRIMARY KEY (kanji, vocab)
);

CREATE INDEX idx_wk_kanji_to_vocab_kanji ON wk_kanji_to_vocab(kanji);

CREATE TABLE wk_meta (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
