# Deploying Hibi Nuku as a systemd service

Single-binary, single-port, auto-restart on crash, auto-start on boot, logs in journald. No Docker, no Vite, no extra moving parts.

## Layout once installed

```
/usr/local/bin/nuku                  # the release binary
/etc/nuku.env                        # secrets + config (root:ilios 0640)
/etc/systemd/system/nuku.service     # the unit
/home/ilios/dev/hibi-nuku/           # source tree (WorkingDirectory)
  frontend/dist/                     # built SPA, served by the binary
  backend/data/                      # SQLite + dict bundles + thumbs
/srv/hibi-nuku/library/              # uploaded videos
```

## First install (run once on the VPS)

```sh
cd /home/ilios/dev/hibi-nuku

# 1. Build the frontend + backend once.
cd frontend && pnpm install && pnpm build && cd ..
cd backend  && cargo build --release && cd ..

# 2. Drop the release binary somewhere stable.
sudo install -m 755 backend/target/release/nuku /usr/local/bin/nuku

# 3. Move the env file out of the repo tree. Root owns it, your
#    user can read it.
sudo install -o root -g ilios -m 0640 .env /etc/nuku.env
# (Optional sanity check: `sudo cat /etc/nuku.env | head`.)

# 4. Install the unit.
sudo install -m 644 docs/nuku.service /etc/systemd/system/nuku.service

# 5. Make sure the library + data dirs exist and are writable by you.
sudo mkdir -p /srv/hibi-nuku/library
sudo chown -R ilios:ilios /srv/hibi-nuku
mkdir -p backend/data

# 6. Enable + start.
sudo systemctl daemon-reload
sudo systemctl enable --now nuku
```

Verify:

```sh
systemctl status nuku
journalctl -u nuku -n 50 --no-pager
curl -sS http://localhost:8787/api/health
```

You should see `{"ok":true,"version":"0.1.0"}`. Browse from your laptop at `http://ilios:8787` (or whatever your tailnet hostname is).

## Day-to-day

```sh
systemctl status nuku            # health + last few log lines
systemctl restart nuku           # graceful restart
systemctl stop nuku
systemctl start nuku
journalctl -u nuku -f            # live logs
```

## Updating to new code

After `git pull`:

```sh
cd /home/ilios/dev/hibi-nuku/frontend && pnpm install && pnpm build
cd /home/ilios/dev/hibi-nuku/backend  && cargo build --release
sudo install -m 755 backend/target/release/nuku /usr/local/bin/nuku
sudo systemctl restart nuku
```

The `just deploy` recipe (see Justfile) bundles those four lines.

## Updating env vars

```sh
sudo $EDITOR /etc/nuku.env
sudo systemctl restart nuku
```

The repo's `.env` is **not** read in service mode — only `/etc/nuku.env` is. Keep `.env` around for `cargo run` dev mode if you like.

## Common gotchas

- **`ProtectHome=read-only`** + repo lives under `/home`. The unit whitelists `backend/data/` for writes via `ReadWritePaths`. If you change `NUKU_DATA_DIR` or `NUKU_DB_PATH` to point outside that path, add it to `ReadWritePaths` and `daemon-reload + restart`.
- **`ProtectSystem=full`** + library outside `/home`. The unit whitelists `/srv/hibi-nuku/library`. Same caveat if you move it.
- **Port 8787** must not clash with anything else. `ss -lntp | grep 8787` to check.
- **Tailscale-only access**: the unit binds `NUKU_HOST=0.0.0.0` per env. You're relying on the VPS having no public DNS / closed firewall for ports 8787. If you ever open it to the internet, put it behind a reverse proxy with TLS first.

## Uninstall

```sh
sudo systemctl disable --now nuku
sudo rm /etc/systemd/system/nuku.service /usr/local/bin/nuku /etc/nuku.env
sudo systemctl daemon-reload
# Repo + data are untouched.
```
