# Godstack Monorepo

Full-Stack Application mit Rust Backend und SolidJS Frontend.

## 🚀 Quick Start (DevContainer)

```bash
just dev
```

Das ist alles! Dieser Befehl startet:
- **Frontend** auf http://localhost:5173 (Vite HMR)
- **Backend** auf http://localhost:3000 (cargo-watch)
- **ZITADEL** auf http://localhost:8080 (Auth)
- **MinIO** auf http://localhost:9001 (S3 Storage)
- PostgreSQL, DragonflyDB im Hintergrund

📖 Ausführliche Dokumentation: [DEV_SETUP.md](DEV_SETUP.md)

## 📁 Projektstruktur

```
├── backend/           # Rust API Server (Axum)
│   ├── src/           # Source Code
│   ├── config/        # Konfiguration (TOML)
│   └── migrations/    # SQL Migrations
├── frontend/          # SolidJS Frontend
│   └── src/           # TypeScript + Components
├── infra/             # Infrastructure & Deployment
│   ├── docker-compose.yml
│   ├── Dockerfile.*   # Container Builds
│   ├── Caddyfile      # Reverse Proxy Config
│   ├── scripts/       # Setup-Skripte
│   └── zitadel/       # ZITADEL Init
├── proto/             # Protobuf Definitionen
├── docs/              # Dokumentation
└── justfile           # Task Runner
```

## 🛠️ Tech Stack

| Komponente | Technologie |
|------------|-------------|
| **Backend** | Rust, Axum, Tokio, SQLx |
| **Frontend** | SolidJS, TanStack Query, Tailwind |
| **Auth** | ZITADEL (OIDC/JWT) |
| **Database** | OrioleDB (PostgreSQL) |
| **Cache** | DragonflyDB (Redis) |
| **Storage** | MinIO (S3) |
| **Build** | Nix + Crane, Vite |

## 🔧 Wichtige Befehle

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | **Startet alles** (Frontend + Backend + Services) |
| `just status` | Zeigt Status aller Services |
| `just reset` | Alles löschen und neu starten |
| `just docker-stop` | Stoppt alle Container |

Alle Befehle: `just --list`

## 🔐 Test-Login

| Account | Benutzer | Passwort |
|---------|----------|----------|
| User | `testuser` | `Test123!` |
| Admin | `zitadel-admin` | `Password1!` |

## 📖 Weitere Dokumentation

- [DEV_SETUP.md](DEV_SETUP.md) - Ausführliche Entwickler-Doku
- [DOCKER.md](DOCKER.md) - Docker-spezifische Infos
- [docs/ZITADEL_SETUP.md](docs/ZITADEL_SETUP.md) - ZITADEL Konfiguration

## Lizenz

MIT
