-- LLM response cache and dict bundle version metadata.

CREATE TABLE llm_cache (
  prompt_hash   TEXT PRIMARY KEY,
  model         TEXT NOT NULL,
  response_json TEXT NOT NULL,
  created_at    TEXT NOT NULL
);

CREATE TABLE dict_versions (
  name     TEXT PRIMARY KEY,        -- 'jmdict' | 'wk' | 'frequency'
  version  TEXT NOT NULL,
  path     TEXT NOT NULL,
  size     INTEGER NOT NULL,
  sha256   TEXT NOT NULL,
  built_at TEXT NOT NULL
);
