# Hibi Nuku — ops / deploy.
#
# Day-to-day on the VPS:
#   make update      # one command: pull, rebuild, restart, tail logs
#
# First install: see `make install` (or docs/deploy.md for the
# longer story).

REPO        := /home/ilios/dev/hibi-nuku
BIN         := /usr/local/bin/nuku
UNIT        := /etc/systemd/system/nuku.service
ENV_FILE    := /etc/nuku.env
LIB_DIR     := /srv/hibi-nuku/library

# Use bash for richer shell features in recipes.
SHELL       := /bin/bash
.SHELLFLAGS := -eu -o pipefail -c

.DEFAULT_GOAL := help

.PHONY: help
help: ## Show this list
	@awk 'BEGIN{FS=":.*##"; printf "Targets:\n"} \
	     /^[a-zA-Z0-9_-]+:.*##/ {printf "  %-12s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ---------------- the only command you'll usually need ----------------

.PHONY: update
update: pull build install-bin restart ## git pull → build → swap binary → restart
	@echo
	@echo "✓ deployed. Tail logs:  make logs"

# ---------------- composable pieces ----------------

.PHONY: pull
pull: ## git pull (fast-forward only)
	cd $(REPO) && git pull --ff-only

.PHONY: build
build: build-frontend build-backend ## Build frontend dist + release binary

.PHONY: build-frontend
build-frontend: ## pnpm install + pnpm build (frontend only)
	cd $(REPO)/frontend && pnpm install --frozen-lockfile && pnpm build

.PHONY: build-backend
build-backend: ## cargo build --release (backend only)
	cd $(REPO)/backend && cargo build --release

.PHONY: install-bin
install-bin: ## Copy the freshly built binary into /usr/local/bin
	sudo install -m 755 $(REPO)/backend/target/release/nuku $(BIN)

# ---------------- systemd ----------------

.PHONY: restart
restart: ## systemctl restart nuku
	sudo systemctl restart nuku

.PHONY: stop
stop: ## systemctl stop nuku
	sudo systemctl stop nuku

.PHONY: start
start: ## systemctl start nuku
	sudo systemctl start nuku

.PHONY: status
status: ## systemctl status nuku
	systemctl status nuku --no-pager

.PHONY: logs
logs: ## Tail live logs (Ctrl-C to leave)
	journalctl -u nuku -f -n 50

# ---------------- first install (idempotent) ----------------

.PHONY: install
install: build install-bin install-env install-unit install-dirs enable ## First-time install — safe to re-run
	@echo
	@echo "✓ installed and started. Check it:"
	@echo "    curl -sS http://localhost:8787/api/health"

.PHONY: install-env
install-env: ## Copy repo .env → /etc/nuku.env (root:ilios 0640)
	@if [ ! -f $(REPO)/.env ]; then \
	  echo "✗ $(REPO)/.env not found. Copy .env.example and fill it in first."; \
	  exit 1; \
	fi
	sudo install -o root -g $$(id -gn) -m 0640 $(REPO)/.env $(ENV_FILE)
	@echo "  → $(ENV_FILE) installed. Edit later with:  sudo \$$EDITOR $(ENV_FILE)"

.PHONY: install-unit
install-unit: ## Install /etc/systemd/system/nuku.service + daemon-reload
	sudo install -m 644 $(REPO)/docs/nuku.service $(UNIT)
	sudo systemctl daemon-reload

.PHONY: install-dirs
install-dirs: ## Create + chown library + data dirs
	sudo mkdir -p $(LIB_DIR)
	sudo chown -R $$(id -un):$$(id -gn) $(LIB_DIR)
	mkdir -p $(REPO)/backend/data

.PHONY: enable
enable: ## systemctl enable --now nuku
	sudo systemctl enable --now nuku

# ---------------- destructive: full uninstall ----------------

.PHONY: uninstall
uninstall: ## Stop, disable, remove binary/unit/env. Leaves data + repo.
	sudo systemctl disable --now nuku || true
	sudo rm -f $(UNIT) $(BIN) $(ENV_FILE)
	sudo systemctl daemon-reload
	@echo "✓ uninstalled. Repo + $(LIB_DIR) + backend/data left intact."
