# Godstack Monorepo

Full-Stack Application mit Rust Backend und SolidJS Frontend.

## Struktur

```
├── backend/           # Rust API Server
│   ├── src/           # Rust Source Code
│   ├── config/        # Konfigurationsdateien
│   ├── migrations/    # SQL Migrations
│   └── tests/         # Integration Tests
├── frontend/          # SolidJS Frontend
│   ├── src/           # TypeScript Source
│   │   ├── api/       # API Client
│   │   ├── components/# UI Komponenten
│   │   ├── lib/       # Utilities & Auth
│   │   └── pages/     # Seiten
│   └── dist/          # Production Build
├── scripts/           # Utility Scripts
├── docker/            # Docker Configs für Host
└── .devcontainer/     # DevContainer Setup
```

## Tech Stack

### Backend
- **Runtime**: Rust + Tokio + Jemalloc
- **Framework**: Axum (HTTP/2)
- **Database**: OrioleDB (PostgreSQL)
- **Cache**: DragonflyDB
- **Auth**: ZITADEL (JWT/OIDC)
- **Build**: Nix + Crane

### Frontend
- **Framework**: SolidJS (fine-grained Reactivity)
- **Routing**: @solidjs/router
- **State**: TanStack Solid Query
- **Styling**: Tailwind CSS
- **UI**: Ark UI (headless components)
- **Auth**: oidc-client-ts (PKCE flow)
- **Build**: Vite 5

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

## Befehle

### Entwicklung (Full-Stack)

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | **Startet Backend + Frontend zusammen** |
| `just bootstrap` | Setup + ZITADEL Init + Full-Stack |

### Backend

| Befehl | Beschreibung |
|--------|--------------|
| `just dev-backend` | Nur Backend mit Hot-Reload |
| `just run` | Backend einmal ausführen |
| `just check` | Cargo check |
| `just test` | Tests ausführen |
| `just lint` | Clippy Linting |
| `just fmt` | Code formatieren |

### Frontend

| Befehl | Beschreibung |
|--------|--------------|
| `just frontend-dev` | Frontend + Backend (mit Abhängigkeit) |
| `just frontend-only` | Nur Frontend (ohne Backend) |
| `just frontend-build` | Frontend Build |
| `just frontend-install` | Dependencies installieren |

### Datenbank

| Befehl | Beschreibung |
|--------|--------------|
| `just db-migrate` | Migrations ausführen |
| `just db-new <name>` | Neue Migration erstellen |
| `just db-reset` | Datenbank zurücksetzen |

### Services

| Befehl | Beschreibung |
|--------|--------------|
| `just services` | Alle Services starten |
| `just services-down` | Services stoppen |
| `just services-logs` | Logs anzeigen |

### ZITADEL

| Befehl | Beschreibung |
|--------|--------------|
| `just zitadel` | ZITADEL Console öffnen |
| `just zitadel-setup` | ZITADEL automatisch einrichten |
| `just zitadel-reset` | ZITADEL zurücksetzen |

### Build

```bash
# Standard Build
just build

# Static musl Binary
just build-static

# Docker Image
just docker-load
```

## Konfiguration

Umgebungsvariablen in `.env`:

```bash
APP_ENVIRONMENT=local
APP_DATABASE__HOST=localhost
APP_DATABASE__PORT=5432
APP_CACHE__URL=redis://localhost:6379
APP_AUTH__ISSUER=http://localhost:8080
```

## API Endpoints

| Endpoint | Beschreibung |
|----------|--------------|
| `GET /api/v1/health` | Health Check |
| `GET /api/v1/ready` | Readiness Check (DB, Cache, Auth) |
| `GET /api/v1/info` | API Info |
| `GET /api/v1/users` | User List (Auth required) |

## Lizenz

MIT
