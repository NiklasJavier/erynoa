# God-Stack Backend - Justfile

set dotenv-load

default:
    @just --list

# ─────────────────────────────────────────────────────
# Development
# ─────────────────────────────────────────────────────

# Dev server with hot reload
dev:
    cargo watch -x run -w src -w config

# Run once
run:
    cargo run

# Quick check
check:
    cargo check

# ─────────────────────────────────────────────────────
# Build (Nix)
# ─────────────────────────────────────────────────────

# Build with Nix (default)
build:
    nix build

# Build static musl binary
build-static:
    nix build .#static

# Build Docker image via Nix
build-docker:
    nix build .#docker
    @echo "Image: ./result (load with: docker load < result)"

# Build and load Docker image
docker-load:
    nix build .#docker
    docker load < result

# ─────────────────────────────────────────────────────
# Test & Quality
# ─────────────────────────────────────────────────────

# Run tests
test:
    cargo test

# Clippy lint
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# All checks
ci: fmt lint test

# Nix checks (clippy + fmt + build)
ci-nix:
    nix flake check

# ─────────────────────────────────────────────────────
# Database
# ─────────────────────────────────────────────────────

# Run migrations
db-migrate:
    sqlx migrate run

# Create migration
db-new name:
    sqlx migrate add {{name}}

# Prepare for offline
db-prepare:
    cargo sqlx prepare

# Reset database
db-reset:
    sqlx database drop -y || true
    sqlx database create
    sqlx migrate run

# ─────────────────────────────────────────────────────
# Infrastructure (Docker)
# ─────────────────────────────────────────────────────

# Start infrastructure (DB + Cache)
infra:
    docker compose -f docker/docker-compose.yml up -d

# Start with ZITADEL
infra-auth:
    docker compose -f docker/docker-compose.yml --profile auth up -d

# Stop infrastructure
infra-down:
    docker compose -f docker/docker-compose.yml --profile auth down

# View logs
logs service="":
    docker compose -f docker/docker-compose.yml logs -f {{service}}

# ─────────────────────────────────────────────────────
# Full Stack
# ─────────────────────────────────────────────────────

# Start infra + run dev server
start: infra
    @sleep 2
    just dev

# Clean all
clean:
    cargo clean
    rm -f result
    docker compose -f docker/docker-compose.yml --profile auth down -v
