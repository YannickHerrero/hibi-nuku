# Hibi Nuku

> 日々抜く — "daily extraction". A sentence-mining web app for Japanese learners, watching anime / shows.

Upload a video → server probes it, extracts the Japanese subtitle track, tokenises with [lindera](https://github.com/lindera/lindera) + UniDic, batches subtitles through an LLM for natural translations + grammar notes → you watch in the browser with a Yomitan-style hover popup over every token, and mine sentences into the [Hibi](https://hibi-api.vercel.app) SRS with one click (audio clip + screenshot + furigana + WK kanji breakdown attached automatically).

**Status**: usable. Hosted on a single VPS, accessed over Tailscale.
**License**: private. Not licensed for redistribution.
**Audience**: one person. Not designed for multi-tenancy or public exposure.

Full spec (the source of truth for design decisions): [`hibi-nuku-spec.md`](./hibi-nuku-spec.md).

---

## What it does

1. **Library**: drop in a video from your browser. The server probes audio + subtitle tracks via `ffprobe`, asks which Japanese tracks to use, and accepts an optional sidecar `.srt`/`.ass`/`.vtt` if no embedded JP subs work.
2. **Pipeline** (async, status visible in the library UI):
   - `probing` → `extracting` → `parsing` → `tokenizing` → `translating` → `ready`
   - Tokenisation: lindera (UniDic) + a longest-match JMDict pass with Yomitan-style deinflection.
   - Translation: batched (50 lines, 2-line context on each side) through OpenRouter (default `anthropic/claude-sonnet-4.6`). Responses are cached in SQLite, so reprocessing is free.
3. **Player**: HTML `<video>` with a custom subtitle overlay. Tokens are coloured by Hibi known-words status (new / learning / known / ignored). Keyboard shortcuts: `Space`, `A`/`D` (prev/next line), `Z` (loop current line).
4. **Popup**: hover or tap a token — instant local lookups against IndexedDB (JMDict, WaniKani, JPDB frequency). Shows reading, top-3 glosses, frequency rank, kanji breakdown with WK level + sibling vocab.
5. **Mining**: click `Mine` → edit fields if you want → server extracts audio clip + screenshot, uploads them to Hibi, builds the full card payload (furigana, kanji list with WK levels, source tag) and POSTs.

---

## Stack

**Backend** — Rust 2024 edition, single crate with multiple `[[bin]]` targets.

| Crate | Why |
|---|---|
| `axum` + `tower-http` | HTTP server |
| `sqlx` (SQLite, WAL) | Persistence |
| `lindera` w/ `embed-unidic` | JP morphological analysis (UniDic shipped in-binary) |
| `quick-xml` | JMDict XML parsing |
| `reqwest` | OpenRouter, WaniKani, Hibi API |
| `tokio::process::Command` | `ffmpeg` / `ffprobe` shell-outs |

**Frontend** — Vite + React 19 + TypeScript (strict).

| Library | Why |
|---|---|
| TanStack Router | Type-safe file-based routing |
| TanStack Query | Server state |
| Tailwind v4 | Styles + design tokens |
| `idb` | IndexedDB wrapper for dict hydration |
| `vite-plugin-pwa` | Service worker + manifest |

**Design system** — five theme variants (`paper`, `stone`, `sage`, `clay`, `ink`) toggled via `html[data-theme]`, adapted from the Hibi monorepo's Torakaa-based DS.

---

## Layout

```
backend/
  migrations/       SQLite migrations (0001-0005)
  src/
    api/            axum routers (health, library, upload, videos, dict, mine, settings, hibi_proxy, auth)
    config.rs       env loader with path candidate resolution
    db.rs           sqlx pool + migration runner
    error.rs        AppError → JSON response
    library/        videos repo, pipeline (probing → ready), subtitle line repo, JP track selector
    media/          ffprobe wrapper, ffmpeg stream + thumbnail
    subtitle/       SRT + ASS parsers, ffmpeg extractor
    tokenize/       lindera wrapper, JMDict index, deinflect rules, segmenter
    jmdict/         XML parser + bundle (serde) + loader
    llm/            OpenRouter client, prompt builder, batch translator, cache
    wk/             WaniKani API client, importer, bundle builder
    dict/           Frequency parser, manifest builder
    hibi/           Hibi API client (cards, uploads, sessions, known-words)
    mining/         furigana alignment, kanji enrichment, asset extractors, orchestrator
    bin/            build-jmdict, build-wk-bundle, build-frequency, wk-import CLIs
frontend/
  src/
    api/            typed client + XHR uploader
    dict/           IndexedDB schema, hydration, deinflect, lookups
    routes/         TanStack Router file-based routes
    player/         SubtitleOverlay, Controls
    popup/          DictionaryPopup
    library/        ImportModal (stepped upload flow)
    mining/         MiningModal
    lib/            token + theme storage
    styles/         tokens.css (5 themes) + tailwind entry
data-sources/       Raw inputs (JMdict_e XML, JPDB Yomitan-format freq)
docs/               architecture notes
```

---

## Setup

### Prerequisites

- `cargo` (Rust 1.94+)
- `pnpm` 10
- `just` (optional but recommended)
- `ffmpeg` + `ffprobe` on PATH
- `sqlite3` (for cache cleanup when iterating on the LLM)

### Configuration

```sh
cp .env.example .env
```

Fill in:

| Var | Purpose |
|---|---|
| `NUKU_TOKEN` | Static bearer the frontend uses (single-user defense-in-depth behind Tailscale). Generate: `openssl rand -hex 32` |
| `HIBI_API_KEY` | From your Hibi account, including the `hibi_` prefix |
| `OPENROUTER_API_KEY` | <https://openrouter.ai/> |
| `WANIKANI_API_KEY` | <https://www.wanikani.com/settings/personal_access_tokens> |
| `NUKU_LIBRARY_DIR` | Absolute path where uploaded media lives |
| `NUKU_DATA_DIR` | Absolute path where dict bundles + SQLite live |
| `NUKU_DB_PATH` | Absolute path to the SQLite file |

`NUKU_LLM_MODEL` defaults to `anthropic/claude-sonnet-4.5`; override if you want a cheaper / faster one.

### One-time: build the dict bundles

```sh
just build-jmdict       # → $NUKU_DATA_DIR/jmdict.json.gz  (~8 MB, ~5 min release build)
just import-wk          # populate WK tables from the API   (~1 min)
just build-wk-bundle    # → $NUKU_DATA_DIR/wk.json.gz
just build-frequency    # → $NUKU_DATA_DIR/frequency.json.gz
```

(Re-run `import-wk` + `build-wk-bundle` whenever you level up on WaniKani.)

### Dev (hot reload)

```sh
just dev   # backend on :8787, frontend on :5173, both die on Ctrl-C
```

Browse to `http://localhost:5173` (or `http://<tailnet-host>:5173`). First visit: `/setup` asks for `NUKU_TOKEN`, then hydrates the dict bundles into IndexedDB (one-time, ~30s on a fast device).

### Prod-style (single binary, no Vite)

```sh
just build       # builds frontend dist + cargo release binary
cd backend && cargo run --release --bin nuku
```

The backend serves the built SPA from `frontend/dist` on the same port as `/api`. Visit `http://<tailnet-host>:8787`.

---

## Justfile recipes

| Recipe | What |
|---|---|
| `just dev` | Run backend + Vite together |
| `just dev-backend` / `just dev-frontend` | Run only one |
| `just check` | `cargo check --all-targets` + `pnpm typecheck` |
| `just build` | Release build of both |
| `just test` | All tests (frontend has none yet) |
| `just build-jmdict` / `just build-wk-bundle` / `just build-frequency` | Build dict bundles |
| `just import-wk` | Refresh WK cache from API |

---

## API surface

All under `/api`, all (except `/api/health`) gated by `Authorization: Bearer $NUKU_TOKEN` or `?token=...` query param.

Key endpoints (full list in `backend/src/api/`):

| Endpoint | Use |
|---|---|
| `GET /api/health` | Readiness check (public) |
| `POST /api/library/upload-media` | Multipart video upload, returns probe results |
| `POST /api/library/upload-subtitle` | Multipart sidecar `.srt`/`.ass`/`.vtt` upload |
| `POST /api/library/create` | Create video row + kick pipeline |
| `GET  /api/library` | List videos |
| `GET  /api/videos/{id}` | Video details |
| `GET  /api/videos/{id}/stream` | Range-streamed remuxed MP4 (`?from=<sec>` for seek) |
| `GET  /api/videos/{id}/subtitles` | Parsed subtitle lines with tokens + translations |
| `GET  /api/videos/{id}/thumbnail` | Thumbnail webp |
| `POST/GET /api/videos/{id}/progress` | Watch position (multi-device resume) |
| `POST /api/mine` | Mine a sentence — extracts assets, uploads to Hibi, creates card |
| `GET  /api/known-words` | Proxied Hibi known-words (60s cache) |
| `PUT  /api/word-status` | Proxied Hibi word-status update |
| `POST /api/sessions` | Proxied Hibi session logger |
| `GET  /api/dict/manifest` + `GET /api/dict/{jmdict,wk,frequency}` | Dict bundle serving for client hydration |
| `GET  /api/settings` | Non-secret config snapshot for the Settings page |

---

## Out of scope (v1)

Per spec §17–18: multi-user, public hosting, in-app review UI, AnkiConnect export, pitch accent dictionary, PGS subtitle OCR, multiple frequency lists.

---

## Useful refs

- [Hibi OpenAPI](https://hibi-api.vercel.app/openapi.json)
- [WaniKani API](https://docs.api.wanikani.com/)
- [JMdict project](http://www.edrdg.org/jmdict/edict_doc.html)
- [Lindera](https://github.com/lindera/lindera)
- [Yomitan deinflect rules](https://github.com/yomidevs/yomitan/blob/master/ext/data/deinflect.json)
