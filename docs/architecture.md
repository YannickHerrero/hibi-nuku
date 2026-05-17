# Architecture

The authoritative reference is [`/hibi-nuku-spec.md`](../hibi-nuku-spec.md).

This file holds quick notes that aren't worth amending the spec for:

- Backend binary name: `nuku`. Other binaries (`build-jmdict`, `build-wk-bundle`, `build-frequency`, `wk-import`) live in the same crate as additional `[[bin]]` targets.
- Backend dev port: `8787` (env `NUKU_PORT`).
- Frontend dev port: `5173` (Vite default). Vite proxies `/api` to backend.
- Production: backend serves `frontend/dist` from the same port as `/api`. Single binary, single port.
- Env: `dotenvy` reads `.env` in dev. In prod use a systemd `EnvironmentFile` (sample lands in `docs/deploy.md` once the VPS is provisioned).
