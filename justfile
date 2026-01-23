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

# Start infrastructure (DB + Cache + ZITADEL) - DevContainer
services:
    docker compose -f .devcontainer/services.yml up -d

# Stop services - DevContainer
services-down:
    docker compose -f .devcontainer/services.yml down

# View service logs - DevContainer
services-logs service="":
    docker compose -f .devcontainer/services.yml logs -f {{service}}

# Restart services - DevContainer
services-restart:
    docker compose -f .devcontainer/services.yml restart

# Service status - DevContainer
services-ps:
    docker compose -f .devcontainer/services.yml ps

# Start infrastructure (DB + Cache) - Host
infra:
    docker compose -f docker/docker-compose.yml up -d

# Start with ZITADEL - Host
infra-auth:
    docker compose -f docker/docker-compose.yml --profile auth up -d

# Stop infrastructure - Host
infra-down:
    docker compose -f docker/docker-compose.yml --profile auth down

# View logs - Host
logs service="":
    docker compose -f docker/docker-compose.yml logs -f {{service}}

# ─────────────────────────────────────────────────────
# Full Stack
# ─────────────────────────────────────────────────────

# Start services + run dev server (DevContainer)
start: services
    @sleep 5
    just dev

# Clean all
clean:
    cargo clean
    rm -f result
    docker compose -f .devcontainer/services.yml down -v 2>/dev/null || true
    docker compose -f docker/docker-compose.yml --profile auth down -v 2>/dev/null || true
