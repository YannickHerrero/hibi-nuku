# Hibi Nuku

> 日々抜く — Japanese sentence mining web app for the Hibi ecosystem.

Personal-use web app for mining Japanese sentences from video (primarily anime).
Imports a file, extracts the JP subtitle track, tokenizes and LLM-translates it once,
then offers a Yomitan-style popup and one-click card creation into the Hibi SRS.

**Status**: in development.
**License**: private — not licensed for redistribution.
**Audience**: solo developer (single user behind Tailscale).

See [`hibi-nuku-spec.md`](./hibi-nuku-spec.md) for the full specification.

## Quick start (dev)

Prerequisites: `cargo`, `pnpm`, `just`, `ffmpeg`, `ffprobe`, `sqlite3`.

```sh
cp .env.example .env
# fill in NUKU_TOKEN, HIBI_API_KEY, OPENROUTER_API_KEY, WANIKANI_API_KEY

just dev          # runs backend (:8787) and frontend (:5173) in parallel
just check        # cargo check + pnpm typecheck
just build        # production build of both
```

## Layout

```
backend/   axum server + ffmpeg shell + sqlite + LLM client
frontend/  Vite + React + TS PWA
data-sources/  Raw inputs for dict bundle build (JMdict, JPDB freq)
docs/      Architecture, deployment notes
```
