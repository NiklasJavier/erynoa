# God-Stack Backend

High-Performance Rust Backend Template.

## Tech Stack

- **Runtime**: Rust + Tokio + Jemalloc
- **Framework**: Axum (HTTP/2)
- **Database**: OrioleDB (PostgreSQL)
- **Cache**: DragonflyDB
- **Auth**: ZITADEL (JWT)
- **TLS**: rustls (no OpenSSL)
- **Build**: Nix + Crane

## Quick Start

### Option 1: DevContainer (empfohlen)

1. VS Code öffnen
2. `Cmd+Shift+P` → "Dev Containers: Reopen in Container"
3. Warten bis Container bereit ist
4. `just dev` ausführen

Alle Services (DB, Cache, ZITADEL) laufen automatisch im DevContainer.

### Option 2: Lokal mit Nix

```bash
# Enter dev shell
nix develop

# Start infrastructure (DB + Cache)
just infra

# Run dev server
just dev
```

## Build

Alle Builds laufen über Nix - kein Docker Build notwendig.

```bash
# Standard Build
nix build
# oder: just build

# Static musl Binary (single file, ~15MB)
nix build .#static
# oder: just build-static

# Docker Image via Nix (minimal scratch image)
nix build .#docker
docker load < result
# oder: just docker-load
```

## Structure

```
src/
├── main.rs          # Entry point
├── lib.rs           # Module exports
├── server.rs        # Server & AppState
├── api/             # REST handlers
│   ├── routes.rs
│   ├── middleware.rs
│   └── handlers/
├── auth/            # JWT validation
├── db/              # SQLx pool
├── cache/           # Redis pool
├── config/          # Settings
├── error.rs         # Error types
└── telemetry.rs     # Logging

config/              # TOML configs
docker/              # Dockerfile + Compose
migrations/          # SQL migrations
```

## Commands

```bash
# Development
just dev          # Dev server mit hot reload
just run          # Run once
just check        # cargo check

# Build (via Nix)
just build        # Standard build
just build-static # Static musl binary
just build-docker # Docker image via Nix
just docker-load  # Build + load into Docker

# Test & Quality
just test         # Run tests
just lint         # Clippy
just fmt          # Format code
just ci           # fmt + lint + test
just ci-nix       # nix flake check

# Database
just db-migrate   # Run migrations
just db-reset     # Reset DB
just db-prepare   # SQLx offline mode

# Infrastructure
just infra        # Start DB + Cache
just infra-auth   # Start with ZITADEL
just infra-down   # Stop all
just logs         # View logs

# Full Stack
just start        # infra + dev server
just clean        # Clean all
```

## API Endpoints

```
GET  /api/v1/health  # Liveness probe
GET  /api/v1/ready   # Readiness probe
GET  /api/v1/info    # Public config

# Protected (JWT required)
GET  /api/v1/me      # Current user
GET  /api/v1/users   # List users (admin)
GET  /api/v1/users/:id
```

## Environment

Copy `.env.example` to `.env` or use `APP_*` env vars:

```bash
APP_DATABASE__HOST=localhost
APP_CACHE__URL=redis://localhost:6379
APP_AUTH__ISSUER=http://localhost:8080
```

## License

MIT
