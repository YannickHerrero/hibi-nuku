# Hibi Nuku — dev tasks
#
# Install `just`:  cargo install just  (or pacman -S just)

set shell := ["bash", "-cu"]
set dotenv-load := true
set dotenv-filename := ".env"

# Default: list recipes
default:
    @just --list

# Run backend and frontend in parallel (Ctrl-C kills both).
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    just dev-backend &
    BACK=$!
    just dev-frontend &
    FRONT=$!
    trap "kill $BACK $FRONT 2>/dev/null || true" INT TERM EXIT
    wait

dev-backend:
    cd backend && cargo run --bin nuku

dev-frontend:
    cd frontend && pnpm dev

# Static checks — green tree before commit.
check:
    cd backend && cargo check --all-targets
    cd frontend && pnpm typecheck

# Production builds.
build:
    cd backend && cargo build --release
    cd frontend && pnpm build

# Tests.
test:
    cd backend && cargo test
    cd frontend && pnpm test

# Dict bundle builds.
#
# Outputs go to $NUKU_DATA_DIR (set in .env). The recipes run from
# repo root so relative paths used by the CLIs land in a single,
# predictable location.

build-jmdict:
    cargo run --manifest-path backend/Cargo.toml --release --bin build-jmdict -- \
        --input data-sources/JMdict_e \
        --output "${NUKU_DATA_DIR}/jmdict.json.gz"

build-wk-bundle:
    cargo run --manifest-path backend/Cargo.toml --release --bin build-wk-bundle -- \
        --output "${NUKU_DATA_DIR}/wk.json.gz"

build-frequency:
    cargo run --manifest-path backend/Cargo.toml --release --bin build-frequency -- \
        --input data-sources/jpdb_v2.2_frequency \
        --output "${NUKU_DATA_DIR}/frequency.json.gz"

import-wk:
    cargo run --manifest-path backend/Cargo.toml --release --bin wk-import
