# Architecture

The current high-level overview lives in the [README](../README.md). The original design rationale (why these libraries, what we're deliberately not building) lives in [`spec.md`](./spec.md) — see §21 for the deltas between the original spec and what's actually shipped.

This file holds quick operational notes:

- Backend binary name: `nuku`. Other binaries (`build-jmdict`, `build-wk-bundle`, `build-frequency`, `wk-import`) live in the same crate as additional `[[bin]]` targets.
- Backend dev port: `8787` (env `NUKU_PORT`).
- Frontend dev port: `5173` (Vite default). Vite proxies `/api` to the backend; uploads bypass the proxy by going through the backend port directly is recommended for files >1 GB.
- Production: backend serves `frontend/dist` from the same port as `/api`. Single binary, single port. systemd unit at `docs/nuku.service`; install via `make install`.
- Env: `dotenvy` reads `.env` in dev. In prod systemd reads `/etc/nuku.env` (see [deploy.md](./deploy.md)).
- Path resolution: `NUKU_DATA_DIR` (absolute) controls where dict bundles + SQLite live. The `[[bin]]` CLIs respect it via `config::data_dir()` — never hard-code paths in those.
