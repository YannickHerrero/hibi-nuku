# Hibi Nuku

> 日々抜く — "daily extraction". A sentence-mining web app for Japanese learners, watching anime / shows.

Upload a video → server probes it, extracts the Japanese subtitle track (or accepts a sidecar `.srt`/`.ass`/`.vtt`), tokenises with [lindera](https://github.com/lindera/lindera) + UniDic, batches subtitles through an LLM for natural translations + grammar notes → you watch in the browser with a click-to-open Yomitan-style popup over every token, and mine sentences into the [Hibi](https://hibi-api.vercel.app) SRS with one click (audio clip + screenshot + furigana + WK kanji breakdown attached automatically).

**Status**: usable. Hosted on a single VPS, accessed over Tailscale.
**License**: private. Not licensed for redistribution.
**Audience**: one person. Not designed for multi-tenancy or public exposure.

Full spec (source of truth for design decisions): [`hibi-nuku-spec.md`](./hibi-nuku-spec.md).
Deploy guide (systemd, one-command install/update): [`docs/deploy.md`](./docs/deploy.md).

---

## What it does

1. **Library**: browser-driven upload. Pick a video → the server saves it under `$NUKU_LIBRARY_DIR/uploads/<ts>-<name>` and runs `ffprobe`. You then pick the JP audio + subtitle tracks from a dropdown (image-based subs like PGS are flagged unsupported). If no embedded JP sub fits, upload a sidecar `.srt`/`.ass`/`.vtt` in the same modal. Upload progress shows MB/s + ETA.
2. **Pipeline** (async, status visible in the library UI):
   - `probing` → `extracting` → `parsing` → `tokenizing` → `translating` → `ready`
   - Tokenisation: lindera (UniDic) + a longest-match JMDict pass with Yomitan-style deinflection.
   - Translation: batched (50 lines, 2-line context each side) through OpenRouter (default `anthropic/claude-sonnet-4.6`). Responses cached in SQLite — reprocessing is free.
3. **Player**: HTML `<video>` capped at `calc(100vh - 175px)` so the subtitle strip + transport controls always stay onscreen. JP subtitle line + a blurred English translation render below the video (hover/tap the translation to reveal). Tokens are underlined by Hibi word status: `new` → solid faint, `learning` → thick yellow, `known` → plain ink, `ignored` → muted no-decoration.
   - **Shortcuts**: `Space` play/pause · `←` / `→` (or `A`/`D`) prev/next subtitle line · `R` (or `Z`) replay current line · `S` pause + close popup.
4. **Popup**: click/tap a token. Sub-10ms IndexedDB lookups against JMDict, WK, JPDB frequency. Shows reading, top-3 glosses, frequency rank, all senses, kanji breakdown with per-kanji WK level + sibling vocab, tone tags from the LLM. Status badge cycles new → learning → known → ignored on click.
5. **Mining**: click `Mine` → edit any field → server extracts audio clip + screenshot in parallel, uploads to Hibi, builds the card payload (furigana, kanji list with WK levels, source tag, tags) and POSTs `/v1/cards`.
6. **Debug**: `/debug` page exposes a Hibi connectivity pinger (per-endpoint status + latency + body snippet), DB stats, IndexedDB inspector + wipe, local JMDict probe, LLM cache wipe, and a service-worker status readout.

---

## Stack

**Backend** — Rust 2024 edition, single crate, multiple `[[bin]]` targets.

| Crate | Why |
|---|---|
| `axum` (+ `multipart`) + `tower-http` | HTTP server, large multipart uploads, static SPA serving |
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
| XHR | Upload-progress visibility (`fetch` can't report it) |

**Design system** — five theme variants (`paper`, `stone`, `sage`, `clay`, `ink`) toggled via `html[data-theme]`, adapted from the Hibi monorepo's Torakaa-based DS.

---

## Layout

```
backend/
  migrations/       SQLite migrations (0001-0005)
  src/
    api/            axum routers — health, library, upload, videos, dict,
                    mine, settings, hibi_proxy, debug, auth
    config.rs       env loader; resolves data_dir from candidates
    db.rs           sqlx pool + migration runner
    error.rs        AppError → JSON (warn/error log level by class)
    library/        videos repo, pipeline (probing → ready), subtitle line
                    repo, JP track selector, sidecar handling
    media/          ffprobe wrapper, ffmpeg stream + thumbnail
    subtitle/       SRT + ASS parsers, ffmpeg extractor
    tokenize/       lindera wrapper, JMDict index, deinflect, segmenter
    jmdict/         XML parser, bundle (serde), loader
    llm/            OpenRouter client (strips ```json fences), prompt
                    builder, batch translator, SHA256 cache
    wk/             WaniKani API client, importer, bundle builder
    dict/           Frequency parser, manifest builder
    hibi/           Hibi API client (cards, uploads, sessions, statuses)
    mining/         furigana alignment, kanji enrichment, asset extractors,
                    orchestrator
    bin/            build-jmdict, build-wk-bundle, build-frequency,
                    wk-import CLIs
frontend/
  src/
    api/            typed client + http + XHR uploader (progress events)
    dict/           IndexedDB schema, hydration, deinflect, lookups
    routes/         TanStack Router file-based routes incl. /debug
    player/         SubtitleOverlay (status underlines + blurred EN),
                    Controls
    popup/          Popup (lookups, breakdown, status cycling, tone tags)
    library/        ImportModal (stepped upload + track picker)
    mining/         MiningModal
    components/     InstallPrompt, ErrorBoundary
    lib/            token + theme storage
    styles/         tokens.css (5 themes) + tailwind entry
data-sources/       Raw inputs (JMdict_e XML, JPDB Yomitan-format freq)
docs/
  deploy.md         systemd install + update runbook
  nuku.service      the systemd unit template
  architecture.md   short architecture pointers (spec stays authoritative)
Makefile            ops: `make install`, `make update`, `make logs`, …
Justfile            dev: `just dev`, `just check`, dict bundle builds
```

---

## Setup

### Prerequisites

- `cargo` (Rust 1.94+)
- `pnpm` 10
- `ffmpeg` + `ffprobe` on PATH
- `make` (ops) + `just` (dev, optional)
- `sqlite3` (manual cache poking)

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

`NUKU_LLM_MODEL` defaults to `anthropic/claude-sonnet-4.5`; override if you want a cheaper / faster one (e.g. `anthropic/claude-haiku-4.5`, `google/gemini-2.0-flash-001`).

### One-time: build the dict bundles

```sh
just build-jmdict       # → $NUKU_DATA_DIR/jmdict.json.gz   (~8 MB, ~5 min release build)
just import-wk          # populate WK tables from the API   (~1 min)
just build-wk-bundle    # → $NUKU_DATA_DIR/wk.json.gz
just build-frequency    # → $NUKU_DATA_DIR/frequency.json.gz
```

Re-run `import-wk` + `build-wk-bundle` whenever you level up on WaniKani.

### Dev (hot reload)

```sh
just dev   # backend on :8787, frontend on :5173, both die on Ctrl-C
```

Browse to `http://localhost:5173` (or `http://<tailnet-host>:5173`). First visit → `/setup` asks for `NUKU_TOKEN`, then hydrates the dict bundles into IndexedDB (one-time, ~30s on a fast device).

### Prod-style (single binary, no Vite) — manual

```sh
just build       # builds frontend dist + cargo release binary
./backend/target/release/nuku
```

The backend serves the built SPA from `frontend/dist` on the same port as `/api`. Visit `http://<tailnet-host>:8787`.

### Prod (systemd, recommended for the VPS)

```sh
make install     # first time only (idempotent)
make update      # every time after `git pull` — pull, rebuild, restart
make logs        # tail logs   (journalctl wrapper)
make status      # health snapshot
make help        # full target list
```

`make install` will copy the binary to `/usr/local/bin/nuku`, copy `.env` → `/etc/nuku.env`, install the systemd unit from `docs/nuku.service`, create + chown `/srv/hibi-nuku/library`, and `systemctl enable --now nuku`.

Full deploy notes (gotchas, ReadWritePaths, uninstall): [`docs/deploy.md`](./docs/deploy.md).

---

## Dev recipes (`Justfile`)

| Recipe | What |
|---|---|
| `just dev` | Run backend + Vite together |
| `just dev-backend` / `just dev-frontend` | Run only one |
| `just check` | `cargo check --all-targets` + `pnpm typecheck` |
| `just build` | Release build of both |
| `just test` | All tests |
| `just build-jmdict` / `just build-wk-bundle` / `just build-frequency` | Build dict bundles |
| `just import-wk` | Refresh WK cache from API |

## Ops recipes (`Makefile`)

| Target | What |
|---|---|
| `make install` | First-time setup (idempotent) |
| `make update` | Pull, build, install binary, restart |
| `make restart` / `stop` / `start` / `status` | systemctl wrappers |
| `make logs` | `journalctl -u nuku -f` |
| `make uninstall` | Remove binary + unit + env; leaves data + repo |

---

## API surface

All routes under `/api`, all (except `/api/health`) gated by `Authorization: Bearer $NUKU_TOKEN` *or* `?token=…` query param (the query form is needed for `<img src>` / `<video src>` which can't carry headers).

| Endpoint | Use |
|---|---|
| `GET /api/health` | Readiness check (public) |
| `POST /api/library/upload-media` | Multipart video upload → saves to disk, returns probe results |
| `POST /api/library/upload-subtitle` | Multipart sidecar `.srt`/`.ass`/`.vtt` upload |
| `POST /api/library/create` | Persist a video row + kick the pipeline |
| `GET  /api/library` | List videos |
| `GET  /api/videos/{id}` | Video details |
| `PATCH /api/videos/{id}` | Edit title / source_tag |
| `DELETE /api/videos/{id}` | Remove from library (file on disk untouched) |
| `POST /api/videos/{id}/reprocess` | Re-run the pipeline (uses LLM cache) |
| `GET  /api/videos/{id}/stream` | Remuxed MP4 (`?from=<sec>` for seek-by-reload) |
| `GET  /api/videos/{id}/subtitles` | Parsed lines with tokens + translations |
| `GET  /api/videos/{id}/thumbnail` | Thumbnail webp |
| `GET/POST /api/videos/{id}/progress` | Watch position (multi-device resume) |
| `POST /api/mine` | Extract assets → upload to Hibi → create card |
| `GET  /api/known-words` | Hibi known-words proxy (60s cache; soft-fails to empty on upstream 5xx) |
| `PUT  /api/word-status` | Hibi word-status passthrough |
| `POST /api/sessions` | Hibi session logger passthrough |
| `GET  /api/dict/manifest`, `GET /api/dict/{jmdict,wk,frequency}` | Dict bundle serving for client hydration |
| `GET  /api/settings` | Non-secret config snapshot for the Settings page |
| `GET  /api/debug/hibi-status` | Per-endpoint Hibi pinger |
| `GET  /api/debug/db-stats` | Row counts + DB file size |
| `DELETE /api/debug/llm-cache?model=…` | Wipe LLM cache (by model or whole table) |

---

## Out of scope (v1)

Per spec §17–18: multi-user, public hosting, in-app review UI, AnkiConnect export, pitch accent dictionary, PGS subtitle OCR, multiple frequency lists, byte-range scrubbing on the remuxed stream (seek reloads the stream from a new offset instead).

---

## Useful refs

- [Hibi OpenAPI](https://hibi-api.vercel.app/openapi.json)
- [WaniKani API](https://docs.api.wanikani.com/)
- [JMdict project](http://www.edrdg.org/jmdict/edict_doc.html)
- [Lindera](https://github.com/lindera/lindera)
- [Yomitan deinflect rules](https://github.com/yomidevs/yomitan/blob/master/ext/data/deinflect.json)
