# Hibi Nuku — Project Specification

> Sentence mining web app for Japanese learners. Imports video files (anime/JP content), pre-processes subtitles with tokenization + LLM translation, then offers a Yomitan-style popup dictionary and one-click card creation into the Hibi SRS ecosystem.
>
> **Status**: Greenfield. This document is the source of truth for v1.
> **Audience**: Solo developer (the author) + Claude Code for implementation.

---

## 1. Project Overview

**Hibi Nuku** is a personal-use web application for Japanese sentence mining from video content (primarily anime, MKV format). It is part of the Hibi ecosystem and integrates with the existing **Hibi API** (a personal Anki-equivalent SRS at `https://hibi-api.vercel.app`).

The app pipeline:

1. User imports a video file (path on the VPS, or upload).
2. Server probes the file, extracts the Japanese subtitle track, tokenizes it, and uses an LLM to produce contextual translations + grammar notes.
3. User watches the video in the browser. Subtitles are rendered as a custom DOM overlay synced to playback, with words color-coded by known/learning/ignored status from Hibi.
4. User hovers/taps a word → instant local popup with definitions, frequency, WaniKani-style kanji breakdown, grammar note, full sentence with translation.
5. User clicks "mine" → backend extracts an audio clip + screenshot for the subtitle line (with 500ms padding), uploads them to Hibi, assembles the card payload, and creates the card via the Hibi API.

**Name**: *Hibi Nuku* (日々抜く) — "daily extraction" / "pulling out (sentences) daily". Sits inside the broader Hibi family.

---

## 2. Goals & Non-Goals

### Goals (v1)

- Personal use only, single user.
- Hosted on the user's VPS (`ilios`), accessible **only via Tailscale**. Not internet-facing.
- Multi-device: import on one device, watch and mine from any other (laptop with Hyprland, iPad, Vision Pro, Android phone, etc.). Server is the canonical source of truth.
- Sub-10ms popup dictionary lookups (local data on client).
- Card creation integrated tightly with Hibi API (audio, image, kanji metadata, furigana, glosses).
- Installable as a PWA on mobile/tablet/Vision Pro.

### Non-Goals (v1)

- Multi-user / authentication beyond a single hardcoded API key.
- Public hosting.
- Persistent video library (user explicitly cleans up watched files; storage is transient).
- Anki AnkiConnect support (Hibi is the only mining target).
- Mobile native app (PWA only).
- Background download / torrenting / Jellyfin-style discovery.

---

## 3. Deployment Context

- **VPS**: Debian on OVH, hostname `ilios`, Tailscale IP `100.89.150.38`.
- **Access**: Tailnet only. No public DNS, no public ports.
- **System dependencies on host**: `ffmpeg` and `ffprobe` (latest stable) — the Rust backend shells out to them rather than linking ffmpeg libraries.
- **Storage**: video files live in a configurable directory (e.g. `/srv/hibi-nuku/library/`). Files are transient; user adds and removes them manually or via the app.
- **External API keys** (in backend `.env`):
  - `HIBI_API_KEY` — Hibi SRS, format `hibi_<key>`
  - `OPENROUTER_API_KEY` — for translation/grammar LLM
  - `WANIKANI_API_KEY` — for WK data import
- **Devices**: Arch Linux ARM laptop, Samsung Galaxy Z Fold 7 (Android), iPad, Apple Vision Pro. All on the tailnet.

---

## 4. Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│ Client (React + Vite PWA)                                   │
│   - Library view                                            │
│   - Video player + custom subtitle overlay                  │
│   - Popup dictionary (lookups against IndexedDB)            │
│   - Mining confirm UI                                       │
│   - Card review (offline) — minimal in v1                   │
└───────────────┬─────────────────────────────────────────────┘
                │ HTTPS over Tailscale
┌───────────────▼─────────────────────────────────────────────┐
│ Backend (Rust, axum)                                        │
│  - Library management (scan, probe, list)                   │
│  - Video streaming (HTTP range requests, direct play)       │
│  - Subtitle pipeline (extract → tokenize → LLM → cache)     │
│  - Card mining (audio clip + screenshot + Hibi upload)      │
│  - Hibi API proxy (cards, uploads, known-words, sessions)   │
│  - Static dictionary bundle serving (JMDict, WK, freq)      │
│  - SQLite for all persistence                               │
└───────────────┬─────────────────────────────────────────────┘
                │
        ┌───────┴────────┬─────────────┬──────────────┐
        ▼                ▼             ▼              ▼
   ┌─────────┐    ┌────────────┐  ┌──────────┐  ┌──────────┐
   │ ffmpeg  │    │ Hibi API   │  │OpenRouter│  │WaniKani  │
   │ (shell) │    │ (tailnet→  │  │   API    │  │  API     │
   │         │    │  internet) │  │          │  │          │
   └─────────┘    └────────────┘  └──────────┘  └──────────┘
```

### Key architectural decisions

1. **Server-canonical processing**: tokenization, deinflection seeding, LLM translation, kanji enrichment all happen once on the server per imported file. Clients consume pre-computed JSON. Means import-once-watch-anywhere works.
2. **Client-local dictionary data**: JMDict, WK, frequency lists are shipped as static bundles from the server, hydrated into IndexedDB on first run. Popup lookups never round-trip.
3. **Direct play, no transcoding by default**: probe each file, prefer `ffmpeg -c copy` remux on the fly into fragmented MP4 served via range requests. Transcode only specific incompatible audio tracks (AC3/DTS/TrueHD) to AAC.
4. **No web video transcoding fallbacks**: if a track is truly incompatible (e.g. PGS image subtitles only, no JP text track), report it and skip. We're not solving every codec edge case.

---

## 5. Tech Stack

### Backend (Rust)

- **`axum`** — HTTP server.
- **`tokio`** — async runtime.
- **`sqlx`** — SQLite with compile-time-checked queries. Migrations via `sqlx migrate`.
- **`reqwest`** — HTTP client for Hibi, OpenRouter, WaniKani.
- **`serde` / `serde_json`** — serialization.
- **`tracing` / `tracing-subscriber`** — structured logging.
- **`lindera`** (with `unidic` feature) — Japanese morphological analyzer. UniDic for better contemporary/colloquial speech handling.
- **`anyhow` / `thiserror`** — error handling.
- **`tower-http`** — middleware (CORS, compression, tracing, static files).
- **`tokio::process::Command`** — shell out to ffmpeg/ffprobe.
- **`clap`** — CLI subcommands (server, wk-import, etc.).

### Frontend

- **Vite + React + TypeScript** (strict mode).
- **TailwindCSS** — styling.
- **TanStack Query** — server state.
- **TanStack Router** or **React Router** — routing (developer's choice).
- **`idb`** — IndexedDB wrapper.
- **`zustand`** — light client state (player state, popup state).
- **`vite-plugin-pwa`** — service worker + manifest.
- **`wanakana`** — kana/romaji conversion (popup helpers, search).

### Build / Dev

- **`cargo`** for backend.
- **`pnpm`** for frontend.
- Single monorepo, no workspace tooling (Cargo + pnpm side by side is fine).
- **`just`** (Justfile) for common dev tasks (`just dev`, `just import-wk`, `just build`).

---

## 6. Repository Structure

```
hibi-nuku/
├── README.md
├── Justfile
├── .gitignore
├── .env.example
├── docs/
│   └── architecture.md      # link back to this spec
├── backend/
│   ├── Cargo.toml
│   ├── migrations/
│   │   └── 0001_init.sql
│   ├── data/                # bundled dict assets (gitignored, generated)
│   │   ├── jmdict.json.gz
│   │   ├── wk.json.gz
│   │   └── frequency.json.gz
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs
│   │   ├── error.rs
│   │   ├── db.rs
│   │   ├── library/
│   │   ├── media/           # streaming, ffmpeg helpers
│   │   ├── subtitle/        # extract, parse SRT/ASS
│   │   ├── tokenize/        # lindera + longest-match + deinflect seeding
│   │   ├── llm/             # OpenRouter client, prompts
│   │   ├── wk/              # WaniKani import + lookups
│   │   ├── jmdict/          # JMDict parser (build-time)
│   │   ├── hibi/            # Hibi API client
│   │   ├── mining/          # card pipeline
│   │   └── api/             # axum routers
│   └── tools/
│       ├── build-jmdict.rs  # CLI: build jmdict bundle from XML
│       ├── build-frequency.rs
│       └── wk-import.rs     # CLI: refresh WK cache
└── frontend/
    ├── package.json
    ├── vite.config.ts
    ├── tsconfig.json
    ├── tailwind.config.ts
    ├── index.html
    ├── public/
    │   └── manifest.webmanifest
    └── src/
        ├── main.tsx
        ├── App.tsx
        ├── api/             # typed client for backend
        ├── routes/          # library, player, settings
        ├── components/
        ├── dict/            # IndexedDB hydration + lookup
        ├── deinflect/       # yomitan deinflection rules (ported)
        ├── player/          # video element, subtitle overlay
        ├── popup/           # popup dictionary UI
        ├── mining/          # card confirm modal
        ├── pwa/             # service worker hooks
        └── lib/             # utils
```

---

## 7. Database Schema (SQLite)

```sql
-- Library
CREATE TABLE videos (
  id              INTEGER PRIMARY KEY,
  path            TEXT NOT NULL UNIQUE,         -- absolute path on server
  title           TEXT NOT NULL,                -- derived from filename, editable
  source_tag      TEXT NOT NULL,                -- e.g. "Frieren S01E03" — passed to Hibi card.source
  duration_ms     INTEGER NOT NULL,
  jp_audio_idx    INTEGER,                      -- ffprobe stream index
  jp_subtitle_idx INTEGER,
  subtitle_format TEXT,                         -- "ass" | "srt" | "vtt" | "pgs" (pgs => error)
  status          TEXT NOT NULL,                -- importing|probing|extracting|tokenizing|translating|ready|error
  error_message   TEXT,
  imported_at     TEXT NOT NULL,
  updated_at      TEXT NOT NULL
);

CREATE TABLE subtitle_lines (
  id            INTEGER PRIMARY KEY,
  video_id      INTEGER NOT NULL REFERENCES videos(id) ON DELETE CASCADE,
  idx           INTEGER NOT NULL,               -- 0-based line index
  start_ms      INTEGER NOT NULL,
  end_ms        INTEGER NOT NULL,
  raw_text      TEXT NOT NULL,
  tokens_json   TEXT,                           -- JSON: [{span:[s,e], surface, lemma, reading, pos, dict_seq}]
  translation   TEXT,
  grammar_note  TEXT,
  tone_tags     TEXT,                           -- JSON array
  UNIQUE(video_id, idx)
);

-- Playback progress (per device optional, keep simple for v1)
CREATE TABLE video_progress (
  video_id        INTEGER PRIMARY KEY REFERENCES videos(id) ON DELETE CASCADE,
  position_ms     INTEGER NOT NULL,
  last_watched_at TEXT NOT NULL,
  last_device     TEXT
);

-- WaniKani cache (refreshed via CLI)
CREATE TABLE wk_kanji (
  characters       TEXT PRIMARY KEY,
  level            INTEGER NOT NULL,
  primary_meaning  TEXT NOT NULL,
  meanings_json    TEXT NOT NULL,
  primary_reading  TEXT NOT NULL,
  reading_type     TEXT NOT NULL,               -- onyomi|kunyomi|nanori
  readings_json    TEXT NOT NULL,
  raw_json         TEXT NOT NULL
);

CREATE TABLE wk_vocab (
  characters         TEXT PRIMARY KEY,
  level              INTEGER NOT NULL,
  meanings_json      TEXT NOT NULL,
  readings_json      TEXT NOT NULL,
  kanji_chars_json   TEXT NOT NULL              -- ["食","事"]
);

CREATE TABLE wk_kanji_to_vocab (
  kanji TEXT NOT NULL,
  vocab TEXT NOT NULL,
  PRIMARY KEY (kanji, vocab)
);
CREATE INDEX idx_wk_kanji_to_vocab_kanji ON wk_kanji_to_vocab(kanji);

-- LLM response cache (key = hash of prompt + model)
CREATE TABLE llm_cache (
  prompt_hash   TEXT PRIMARY KEY,
  model         TEXT NOT NULL,
  response_json TEXT NOT NULL,
  created_at    TEXT NOT NULL
);

-- Dictionary bundle versioning (server-side)
CREATE TABLE dict_versions (
  name    TEXT PRIMARY KEY,    -- "jmdict" | "wk" | "frequency"
  version TEXT NOT NULL,       -- semver or date string
  path    TEXT NOT NULL,       -- relative path to gzipped bundle
  size    INTEGER NOT NULL,
  sha256  TEXT NOT NULL,
  built_at TEXT NOT NULL
);
```

---

## 8. Backend HTTP API

All routes are under `/api/` and require a single header `Authorization: Bearer <NUKU_TOKEN>` (a static token from env, since this is single-user behind Tailscale — auth is defense-in-depth, not real auth).

### Library

- `POST   /api/library/import` — body `{ path: string }`. Server resolves, probes, starts async pipeline. Returns `{ videoId, status }`.
- `GET    /api/library` — list videos with status + progress.
- `GET    /api/videos/:id` — full video metadata + current status.
- `DELETE /api/videos/:id` — remove from library (does NOT delete the file on disk).
- `POST   /api/videos/:id/reprocess` — re-run tokenization + LLM (uses llm_cache so it's cheap if nothing changed).

### Streaming

- `GET /api/videos/:id/stream` — supports HTTP `Range` requests. Returns remuxed fragmented MP4 with JP audio track selected, video stream copied, audio transcoded to AAC if necessary. Internally spawns `ffmpeg` and pipes stdout.
- `GET /api/videos/:id/subtitles` — JSON array of pre-processed subtitle lines (full payload, cached client-side).

### Mining

- `POST /api/mine` — body:
  ```json
  {
    "videoId": 1,
    "lineId": 42,
    "focusWord": "食べる",
    "focusWordReading": "たべる",
    "padBeforeMs": 500,
    "padAfterMs": 500,
    "userEnglishOverride": null,
    "userGrammarOverride": null,
    "tags": ["anime", "frieren"]
  }
  ```
  Server: extracts audio clip + frame, uploads to Hibi, assembles card payload with kanji enrichment, POSTs to Hibi. Returns `{ cardId, audioUrl, imageUrl }`.

### Hibi proxy

- `GET  /api/known-words` — proxies Hibi `/v1/known-words` (with caching, e.g. 60s).
- `PUT  /api/word-status` — proxies Hibi `/v1/word-status`.
- `POST /api/sessions` — proxies Hibi `/v1/sessions`. Used to log immersion sessions.

### Dictionary bundles

- `GET /api/dict/manifest` — `{ jmdict: {version, sha256, size}, wk: {...}, frequency: {...} }`. Client compares vs. IndexedDB-stored version.
- `GET /api/dict/jmdict` — gzipped JSON bundle.
- `GET /api/dict/wk` — gzipped JSON bundle.
- `GET /api/dict/frequency` — gzipped JSON bundle (JPDB anime list).

### Health / admin

- `GET /api/health` — `{ ok: true, version }`.

---

## 9. External Integrations

### 9.1 Hibi API

OpenAPI spec at `https://hibi-api.vercel.app/openapi.json`. Auth: `Authorization: Bearer hibi_<key>`.

Endpoints we use:

- `POST /v1/uploads/audio` — multipart, m4a/mp3/ogg/aac, max 5MB. Returns `{ key }`.
- `POST /v1/uploads/image` — multipart, jpeg/png/webp/gif, max 10MB. Returns `{ key }`.
- `POST /v1/cards` — card creation. Required fields: `sentence`, `focusWord`, `focusWordReading`, `furigana` (array of `{base, reading}`), `english`, `glosses`, `grammarNote`, `kanjiList` (array of `{kanji, meaning, wanikaniLevel}`), `imageKey`, `audioKey`, `source`, `tags`.
- `GET  /v1/known-words` — merged manual + SRS word statuses. Use for color overlay.
- `PUT  /v1/word-status` — `{ lemma, reading, status: "learning"|"known"|"ignored"|null }`.
- `POST /v1/sessions` — `{ kind: "video", source: "hibi-nuku", startedAt, endedAt, durationMs, metadata: { videoId, title } }`.

### 9.2 OpenRouter (LLM)

- Model: **default to Claude Sonnet via OpenRouter**, configurable via env (`NUKU_LLM_MODEL`). Cheaper alternative: `google/gemini-2.0-flash-001` or similar.
- Use **JSON mode / structured outputs**.
- Batched per subtitle file, chunks of ~50 lines with 2-line context before/after.
- Cache by `sha256(model + prompt_version + chunk_input)` in `llm_cache` table.

**Prompt shape** (system):

> You are translating Japanese anime/show subtitles for a language learner. For each target line, produce:
> - `english`: natural English translation
> - `grammar_note`: ONE short sentence (max 100 chars) explaining the key grammar point in the line. Empty string if nothing notable.
> - `tone_tags`: array from {`keigo`, `casual_female`, `casual_male`, `rough_male`, `polite`, `kansai_ben`, `tohoku_ben`, `archaic`, `child_speech`} or empty.
>
> Use the context lines for pronoun/reference resolution but only output entries for target lines.
>
> Output strict JSON: `{ "lines": [{ "idx": <int>, "english": "...", "grammar_note": "...", "tone_tags": [...] }] }`.

### 9.3 WaniKani

- Base: `https://api.wanikani.com/v2/`.
- Auth: `Authorization: Bearer <WANIKANI_API_KEY>`.
- Endpoints:
  - `GET /subjects?types=kanji` — paginate via `next_url`.
  - `GET /subjects?types=vocabulary` — same.
- Respect `Last-Modified` / `If-Modified-Since` for incremental refresh.
- Rate limit: 60 req/min.
- Imported once via `wk-import` CLI subcommand. Stored in `wk_kanji`, `wk_vocab`, `wk_kanji_to_vocab`.
- Bundled into `/api/dict/wk` bundle for client side (so popup can show kanji breakdown + related vocab without server round trips).

---

## 10. Data Pipelines

### 10.1 Subtitle import pipeline

Async background task triggered by `POST /api/library/import`. Status writes to `videos.status` at each step so the frontend can poll.

```
1. probing
   - ffprobe -print_format json -show_streams <file>
   - Identify duration, JP audio track (language tag = "jpn" or "ja"), JP subtitle track
   - If subtitle format is "pgs" or "hdmv_pgs_subtitle" → status=error, error_message="image-based subtitles not supported"
2. extracting
   - ffmpeg -i <file> -map 0:s:<idx> -c:s srt <tempfile>.srt
   - Or for ASS: -c:s ass
3. parsing
   - Parse SRT/ASS into [{startMs, endMs, rawText}] (strip ASS styling tags)
4. tokenizing
   - For each line: lindera (UniDic) → tokens with POS, lemma, reading
   - Run longest-match dictionary segmentation pass against JMDict to glue tokens into word-level spans
   - Persist tokens_json to subtitle_lines
5. translating
   - Chunk lines into batches of ~50 with 2-line context window
   - For each chunk: check llm_cache → hit returns cached, miss calls OpenRouter
   - Persist translation, grammar_note, tone_tags per line
6. ready
   - Update status=ready, updated_at=now
```

Failure handling: any step fails → status=error, error_message set, full traceback logged.

### 10.2 Card mining pipeline

Triggered by `POST /api/mine`. Synchronous (user is waiting).

```
1. Load video + subtitle line from DB
2. Compute audio range: [line.start_ms - padBeforeMs, line.end_ms + padAfterMs]
3. Spawn ffmpeg in parallel:
   a. Audio:    ffmpeg -ss <start_sec> -to <end_sec> -i <file> -map 0:a:<jp_idx>
                       -c:a aac -b:a 128k -movflags +faststart tempfile.m4a
   b. Screenshot: ffmpeg -ss <mid_sec> -i <file> -frames:v 1 -q:v 2 tempfile.webp
4. POST multipart audio   → Hibi /v1/uploads/audio  → audioKey
5. POST multipart image   → Hibi /v1/uploads/image  → imageKey
6. Assemble card payload:
   - sentence = line.raw_text
   - focusWord, focusWordReading = from request
   - furigana = from tokens_json (extract per-kanji-base readings for the focus word)
   - english = userEnglishOverride || line.translation
   - glosses = JMDict glosses for focusWord (top 3)
   - grammarNote = userGrammarOverride || line.grammar_note
   - kanjiList = for each kanji in line.raw_text: { kanji, meaning, wanikaniLevel } from wk_kanji (omit non-kanji, omit kanji not in WK)
   - source = video.source_tag
   - tags = request.tags
7. POST /v1/cards to Hibi  → cardId
8. Cleanup tempfiles
9. Return { cardId, audioUrl, imageUrl } to frontend
```

---

## 11. Dictionary Data

### 11.1 JMDict

- Source: JMdict_e (English) XML from EDRDG (`http://www.edrdg.org/jmdict/edict_doc.html`).
- Build script: `cargo run --bin build-jmdict -- --input JMdict_e.xml --output backend/data/jmdict.json.gz`.
- Output format (gzipped JSON):
  ```json
  {
    "version": "2026-04-01",
    "entries": [
      {
        "seq": 1234567,
        "kanji": ["食べる"],
        "readings": ["たべる"],
        "senses": [
          {
            "pos": ["v1", "vt"],
            "glosses": ["to eat"]
          }
        ],
        "rules": ["v1"]
      }
    ]
  }
  ```
- Client hydrates into IndexedDB:
  - Object store `entries` keyed by `seq`.
  - Object store `index` keyed by every kanji + reading form, value `[seq, ...]`.

### 11.2 WaniKani bundle

- Built by `cargo run --bin build-wk-bundle` from the populated `wk_kanji` / `wk_vocab` / `wk_kanji_to_vocab` tables (populated by `wk-import`).
- Output format:
  ```json
  {
    "version": "2026-04-01",
    "kanji": { "食": { "level": 9, "meaning": "Eat", "reading": "ショク", "reading_type": "onyomi" } },
    "vocab_by_kanji": { "食": ["食べる", "食事", "食堂", ...] },
    "vocab": { "食べる": { "level": 9, "meanings": ["to eat"], "readings": ["たべる"] } }
  }
  ```

### 11.3 Frequency list

- Source: **JPDB anime/manga frequency list** (community-distributed in Yomitan format, search "jpdb anime frequency yomitan").
- Build script normalizes to:
  ```json
  {
    "version": "...",
    "name": "JPDB Anime",
    "freq": { "食べる": 1234, ... }
  }
  ```
- Rank is the numeric value; lower = more frequent.

### 11.4 Hydration on client

On app load:

1. Check IndexedDB for stored dict versions.
2. Fetch `/api/dict/manifest`.
3. For each bundle where stored version != server version (or missing): download gzipped bundle with progress UI, decompress, write to IndexedDB in transaction batches of 500-1000 entries.
4. Show "Ready" once all three are loaded.

This happens on first run (~30s on a fast device) and only re-runs when a bundle version changes.

---

## 12. Tokenization & Deinflection

### 12.1 Server-side tokenization

Pre-pass on each subtitle line at import time:

1. Run line through **lindera (UniDic)** → tokens with `surface`, `pos`, `lemma`, `reading`.
2. **Longest-match dictionary pass**:
   ```
   pos = 0
   while pos < line.length:
     best_match = None
     for end in range(line.length, pos, -1):
       candidate = line[pos:end]
       for deinflected, rules_out in deinflect(candidate):
         entries = jmdict_lookup(deinflected)
         valid = entries where entry.rules ∩ rules_out != ∅
         if valid:
           best_match = (pos, end, deinflected, valid[0].seq)
           break
       if best_match: break
     if best_match:
       emit span
       pos = best_match.end
     else:
       emit single-char span (fallback)
       pos += 1
   ```
3. Persist as `tokens_json` array of `{span: [start, end], surface, lemma, reading, pos, dict_seq}` per subtitle line.

### 12.2 Client-side deinflection (popup)

For arbitrary user selection (user might select across token boundaries, or be in a position where pre-tokenization is wrong):

- Port **Yomitan's `deinflect.json`** rules to a TypeScript module.
- Algorithm: given input string, apply suffix rules recursively to produce candidate `(stem, rules_out)` pairs.
- Query IndexedDB JMDict index for each candidate; filter by rule compatibility.
- Rank by: longest stem first, then by frequency (lower rank = more common = higher), then by JMDict commonness flags.

Yomitan deinflection rules are MIT-licensed and well-documented. Reference: `https://github.com/yomidevs/yomitan/blob/master/ext/data/deinflect.json`.

---

## 13. Frontend

### 13.1 Routes

- `/` — library list
- `/import` — import a video by path or upload (could be a modal on `/`)
- `/watch/:videoId` — player
- `/settings` — API keys (display only, set server-side), dict versions, WK level, theme
- `/setup` — first-run wizard if dict bundles missing

### 13.2 Library view

- Grid or list of videos with thumbnail (screenshot at 30% playthrough, generated on import).
- Status badge: importing / ready / error.
- Progress bar on watched videos.
- "Mine count" per video (optional, query Hibi by source tag).
- Click → `/watch/:id`.

### 13.3 Player & subtitle overlay

- HTML `<video>` element with `src` pointing to `/api/videos/:id/stream`.
- Custom controls: play/pause, scrub, volume, playback rate (0.5x to 2x with 0.05 steps — useful for shadowing), subtitle visibility toggle, mining shortcut.
- **Subtitle overlay**: a positioned `<div>` over the video, rendered by syncing to `video.currentTime` (via `requestAnimationFrame` loop or `timeupdate` events + interpolation).
- Each subtitle line renders its tokens as `<span>` elements with classes by word status (known / learning / new / ignored).
- Hover (desktop) or tap (touch): show popup positioned relative to the span.
- Keyboard shortcuts: `A` / `D` previous/next subtitle line and seek to its start, `S` pause-and-mine current focused word, `Space` play/pause, `Z` repeat current line (loop start..end).

### 13.4 Popup dictionary UI

Layout (top to bottom):

- **Header**: focus word (large, with furigana) + kana reading + audio play button (TTS or focus audio if mined).
- **Status badge**: known / learning / new / ignored — clickable to cycle.
- **Glosses**: top 3 from JMDict, each prefixed with POS tag.
- **Frequency**: e.g. "JPDB Anime: #1,234".

Expandable sections (collapsed by default, remember last state):

- **Sentence context**: full sentence with translation + grammar note. Each token clickable for cross-lookup.
- **Kanji breakdown**: for each kanji in the focus word: WK level badge, primary meaning, primary reading. Below: "Vocab using these kanji" — up to 8 entries from `vocab_by_kanji`, sorted by closeness to user's WK level (config).
- **All meanings**: additional senses from JMDict if more than top 3.

Bottom action bar:

- "Mine" button (primary) → opens confirm modal.
- "Mark as known" / "Mark as learning" / "Ignore".

### 13.5 Mining confirm modal

Pre-filled from popup state and subtitle line:

- Sentence (editable)
- Focus word (editable, in case user wants to change form)
- Reading (editable)
- English translation (editable, defaults to line's LLM translation)
- Grammar note (editable, defaults to LLM grammar note)
- Tags input
- Audio preview player (extracted on backend in parallel with modal opening so it's ready by the time user confirms)
- Image preview
- "Create card" button → calls `/api/mine` → toast on success.

### 13.6 PWA

- Manifest: name "Hibi Nuku", icon set, theme color matching app, display "standalone".
- Service worker via `vite-plugin-pwa` in `injectManifest` mode:
  - Precache app shell.
  - Runtime cache for dict bundles (cache-first, never expire — versioning handled at app layer).
  - Network-first for API calls.
- Install prompt on first visit if criteria met.

---

## 14. Multi-device Sync

Server is the canonical source. Clients are stateless except for cached dict bundles + UI prefs.

- **Watch progress**: client posts `POST /api/videos/:id/progress { position_ms }` debounced every 5 seconds while playing. On video open, fetch current progress and offer "Resume from 12:34?".
- **Mining state**: cards live in Hibi. Frontend queries Hibi cards by `source` tag to show "X cards mined from this episode" badge in library.
- **Word status**: refreshed from `/api/known-words` every minute while in player, on every popup open, and after every mine. Optimistic local updates on `PUT /api/word-status`.
- **Settings**: stored in localStorage per device (theme, default playback rate, popup section state). No server sync.

---

## 15. Development Workflow

### 15.1 Commit conventions

**This is important.** Make many small, atomic, independent commits. The more the better. Each commit should:

- Do **one thing**.
- Leave the repo in a working state (compiles, tests pass).
- Have a clear conventional-commits message.
- Be reviewable in isolation.

Format: `<type>(<scope>): <subject>` where:

- `type` ∈ {`feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `build`, `style`}
- `scope` ∈ {`backend`, `frontend`, `db`, `subtitle`, `tokenize`, `llm`, `wk`, `mining`, `popup`, `player`, `library`, `pwa`, `dict`, `hibi`, `infra`}

Examples of good commit granularity:

- `chore(infra): initialize cargo workspace`
- `chore(infra): add Justfile with dev tasks`
- `feat(backend): add config loader from env`
- `feat(db): add sqlx setup with migrations`
- `feat(db): add videos table migration`
- `feat(library): add ffprobe wrapper`
- `feat(library): POST /api/library/import endpoint`
- `feat(subtitle): SRT parser`
- `feat(subtitle): ASS parser with style stripping`
- `feat(tokenize): integrate lindera with UniDic`
- `feat(tokenize): longest-match dictionary segmentation`
- `feat(llm): OpenRouter client`
- `feat(llm): batched translation prompt + JSON output`
- `feat(llm): llm_cache table integration`

Avoid:

- Mega-commits (`feat: implement backend`).
- Mixing unrelated changes.
- "WIP" commits left in history.

### 15.2 Branch strategy

- Work on `main` directly is fine for personal project.
- Use branches only if experimenting with something disruptive.

### 15.3 Tests

Pragmatic, not dogmatic. Where tests pay for themselves:

- Subtitle parsers (SRT, ASS) — fixtures with edge cases.
- Tokenization longest-match — fixtures with known difficult sentences.
- Deinflection rules — known inflected forms → expected lemma.
- Hibi API client — mock HTTP layer.

Skip:

- React component tests beyond critical logic.
- E2E for v1.

### 15.4 Dev environment

- Backend dev: `cargo watch -x run` or via `just dev-backend`.
- Frontend dev: `pnpm dev` or via `just dev-frontend`.
- Both at once: `just dev` (parallel).
- SQLite DB at `backend/data/dev.db` (gitignored).

---

## 16. Implementation Phases

Suggested order. Each phase is multiple commits. **Do not try to ship a phase in one commit.**

### Phase 0 — Project skeleton

- Initialize repo, README, .gitignore, .env.example.
- Cargo workspace skeleton (just an empty `backend/` for now).
- Vite React TS skeleton for `frontend/`.
- Justfile with `dev`, `build`, `test` placeholders.
- License (or note it's private).

### Phase 1 — Backend foundations

- axum hello-world + `/api/health`.
- Config loader (env → struct) with `dotenvy`.
- Structured logging via `tracing`.
- Error type + axum response impl.
- sqlx + SQLite setup, migrations dir, first migration (videos + subtitle_lines tables).
- Auth middleware (single bearer token).

### Phase 2 — Library management

- Video schema + sqlx repo.
- ffprobe wrapper (parse JSON output to struct).
- `POST /api/library/import` → probes + persists.
- `GET /api/library` and `GET /api/videos/:id`.
- `DELETE /api/videos/:id`.
- Status state machine.

### Phase 3 — Subtitle extraction & parsing

- ffmpeg wrapper for subtitle extraction.
- SRT parser.
- ASS parser (strip styling).
- Persist subtitle_lines (raw only, no tokens yet).
- Detect and reject PGS subtitles.

### Phase 4 — Tokenization

- Lindera integration with UniDic.
- JMDict XML parser (build-time CLI).
- JMDict bundle build script → `backend/data/jmdict.json.gz`.
- In-memory JMDict index loader at server boot.
- Deinflection rule loader (Yomitan rules ported to Rust).
- Longest-match segmentation pass over a line.
- Persist tokens_json to subtitle_lines.

### Phase 5 — LLM translation

- OpenRouter client.
- Prompt builder + JSON schema validator.
- Batched call over chunks of subtitle lines.
- llm_cache table + lookup-or-compute.
- Persist translations + grammar_note + tone_tags.

### Phase 6 — Streaming

- ffmpeg streaming wrapper (remux + AAC fallback).
- `GET /api/videos/:id/stream` with Range request support.
- `GET /api/videos/:id/subtitles` returns full JSON.

### Phase 7 — WaniKani

- `wk-import` CLI subcommand.
- WK API client with pagination.
- Populate wk_kanji, wk_vocab, wk_kanji_to_vocab.
- WK bundle build script → `backend/data/wk.json.gz`.

### Phase 8 — Frequency list

- Find / acquire JPDB frequency list.
- Build script → `backend/data/frequency.json.gz`.
- Dict manifest endpoint + static bundle serving.

### Phase 9 — Hibi integration & mining

- Hibi client (uploads, cards, known-words, sessions).
- `POST /api/mine` end-to-end:
  - ffmpeg audio extract.
  - ffmpeg frame extract.
  - Parallel uploads.
  - Card payload assembly (with kanji enrichment from wk_kanji).
  - Card POST.
- Proxy endpoints: `/api/known-words`, `/api/word-status`, `/api/sessions`.

### Phase 10 — Frontend foundations

- Tailwind setup.
- Router, base layout, settings page.
- Typed API client (generate or hand-write based on backend types).
- TanStack Query setup.
- PWA manifest + service worker scaffolding.

### Phase 11 — Dict hydration on client

- IndexedDB schema (entries, index, wk, frequency).
- Bundle fetch + decompress + populate IndexedDB.
- Progress UI.
- Version comparison + re-hydrate on change.

### Phase 12 — Library UI

- Library route with video grid/list.
- Import modal (path input).
- Polling on importing videos.
- Status display + error display.

### Phase 13 — Player

- Video element + custom controls.
- Subtitle overlay synced to currentTime.
- Watch progress sync (5s debounced).
- Keyboard shortcuts.

### Phase 14 — Popup dictionary

- Word selection / hover detection.
- Lookup logic (IndexedDB query + deinflection).
- Popup component with default + expandable sections.
- Word coloring via known-words.

### Phase 15 — Mining

- Mining confirm modal.
- Submit → `/api/mine`.
- Toast notifications.
- Cross-tab/device card sync (re-fetch known-words after mine).

### Phase 16 — Session tracking

- Start/end immersion session on video open/close.
- Post to `/api/sessions`.

### Phase 17 — PWA polish

- Install prompt.
- Service worker runtime caching for dict bundles.
- Offline fallback page.

### Phase 18 — Polish & QA

- Tone tag display in popup.
- Settings page (WK level, theme, default padding).
- Per-device "last played" hint.
- Error boundary UI.

---

## 17. Open Questions / Decisions to Make During Build

These are intentionally deferred; let the implementation surface the best answer:

1. **Subtitle overlay rendering**: position-absolute spans vs. canvas? Start with DOM spans for hover/tap interaction simplicity.
2. **Per-line audio caching**: should we pre-extract audio for every line on import (so mining is instant) or extract on-demand at mine time? On-demand is simpler and ffmpeg seeks are fast. Default: on-demand.
3. **Vision Pro specifics**: do we need a "theater mode" route with minimal UI for VisionOS? Defer until tested.
4. **Backup of mined progress**: hibi is the source of truth for cards; do we need any local backup of `subtitle_lines` data? Probably no — re-import would regenerate it.
5. **Multiple subtitle tracks**: a file might have JP + EN subs. We use JP only for parsing, but should the player let the user toggle to EN subs as a "training wheels" overlay? Nice-to-have, not v1.

---

## 18. Out of Scope (v1)

Explicitly NOT in v1, to keep scope contained:

- Multi-user / accounts.
- Public hosting / non-tailnet access.
- File upload through the browser (paths on the server only — much simpler, fine for personal use).
- Bulk import / folder watching.
- Pitch accent dictionary integration (NHK Yomitan-format dataset). Easy to add later.
- Image-based subtitle OCR (PGS).
- Anki AnkiConnect export.
- Card review UI inside Nuku (use Hibi's app for reviewing).
- Multiple frequency lists with toggling.
- Custom user dictionaries.
- Pronunciation TTS (audio comes from the show itself).

---

## 19. Quick Reference: External URLs

- Hibi OpenAPI: `https://hibi-api.vercel.app/openapi.json`
- Hibi base: `https://hibi-api.vercel.app/v1/`
- WaniKani API docs: `https://docs.api.wanikani.com/`
- OpenRouter docs: `https://openrouter.ai/docs`
- JMdict project: `http://www.edrdg.org/jmdict/edict_doc.html`
- Lindera: `https://github.com/lindera/lindera`
- Yomitan deinflect rules: `https://github.com/yomidevs/yomitan/blob/master/ext/data/deinflect.json`

---

## 20. Instructions to Claude Code

When asked to plan and build this project:

1. **Read this whole document first.**
2. Propose an implementation order that closely follows §16 phases, but feel free to suggest re-ordering with justification.
3. **Commit constantly.** After every meaningfully complete chunk of work — even small ones — make a commit with a conventional-commits message. Aim for many small commits over few large ones. If a task takes 3 logical steps, that's 3 commits, not 1.
4. Run `cargo check` / `pnpm typecheck` before committing backend / frontend changes to ensure the tree is green.
5. Don't write tests speculatively. Write them for: parsers (SRT/ASS), tokenization edge cases, deinflection correctness, Hibi client request shapes.
6. Ask before introducing major new dependencies not listed in §5.
7. If something is genuinely ambiguous, list it as a question and propose a default — don't block.
8. Prefer simplicity. This is a personal tool, not a SaaS product.
