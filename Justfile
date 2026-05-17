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
build-jmdict:
    cd backend && cargo run --release --bin build-jmdict -- \
        --input ../data-sources/JMdict_e \
        --output data/jmdict.json.gz

build-wk-bundle:
    cd backend && cargo run --release --bin build-wk-bundle -- \
        --output data/wk.json.gz

build-frequency:
    cd backend && cargo run --release --bin build-frequency -- \
        --input ../data-sources/jpdb_v2.2_frequency \
        --output data/frequency.json.gz

import-wk:
    cd backend && cargo run --release --bin wk-import
