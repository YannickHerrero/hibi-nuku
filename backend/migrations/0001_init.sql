-- Library: imported videos and per-line subtitle state.

CREATE TABLE videos (
  id              INTEGER PRIMARY KEY,
  path            TEXT NOT NULL UNIQUE,
  title           TEXT NOT NULL,
  source_tag      TEXT NOT NULL,
  duration_ms     INTEGER NOT NULL,
  jp_audio_idx    INTEGER,
  jp_subtitle_idx INTEGER,
  subtitle_format TEXT,                       -- 'ass' | 'srt' | 'vtt' | 'pgs'
  status          TEXT NOT NULL,              -- importing|probing|extracting|parsing|tokenizing|translating|ready|error
  error_message   TEXT,
  imported_at     TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);

CREATE INDEX idx_videos_status ON videos(status);

CREATE TABLE subtitle_lines (
  id            INTEGER PRIMARY KEY,
  video_id      INTEGER NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
  idx           INTEGER NOT NULL,
  start_ms      INTEGER NOT NULL,
  end_ms        INTEGER NOT NULL,
  raw_text      TEXT NOT NULL,
  tokens_json   TEXT,
  translation   TEXT,
  grammar_note  TEXT,
  tone_tags     TEXT,
  UNIQUE(video_id, idx)
);

CREATE INDEX idx_subtitle_lines_video ON subtitle_lines(video_id);

CREATE TABLE video_progress (
  video_id        INTEGER PRIMARY KEY REFERENCES videos(id) ON DELETE CASCADE,
  position_ms     INTEGER NOT NULL,
  last_watched_at TEXT NOT NULL,
  last_device     TEXT
);
