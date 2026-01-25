# Godstack Monorepo

Full-Stack Application mit Rust Backend und SolidJS Frontend.

## 🚀 Quick Start

```bash
just dev
```

Startet alles:
- **Frontend** auf http://localhost:5173 (Vite HMR)
- **Backend** auf http://localhost:3000 (cargo-watch)
- **ZITADEL** auf http://localhost:8080 (Auth)
- **MinIO** auf http://localhost:9001 (S3 Storage)
- PostgreSQL, DragonflyDB im Hintergrund

**Test Login:**
- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

## 📁 Projektstruktur

```
├── backend/           # Rust API Server (Axum + Connect-RPC)
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
| **API** | Connect-RPC/gRPC-Web (Protobuf) |
| **Auth** | ZITADEL (OIDC/JWT) |
| **Database** | PostgreSQL (OrioleDB) |
| **Cache** | DragonflyDB (Redis) |
| **Storage** | MinIO (S3) |
| **Build** | Nix + Crane, Vite |

## 🔧 Wichtige Befehle

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | **Startet alles** (Frontend + Backend + Services) |
| `just status` | Zeigt Status aller Services |
| `just dev-check` | Health Check aller Services |
| `just reset` | Alles löschen und neu starten |
| `just docker-stop` | Stoppt alle Container |
| `just lint` | Backend Clippy |
| `just fmt` | Backend Format |
| `just test` | Backend Tests |

Alle Befehle: `just --list`

## 📖 Dokumentation

### 📚 Hauptdokumentation

- **[ESSENTIAL_GUIDE.md](docs/ESSENTIAL_GUIDE.md)** - **Alles was du brauchst** (konsolidierte Dokumentation)
  - Quick Start, TODOs, Architektur, Befehle, Konfiguration, Troubleshooting

### 🚀 Setup & Entwicklung

- [DEV_SETUP.md](DEV_SETUP.md) - Development Environment Setup
- [DOCKER.md](DOCKER.md) - Docker & DevContainer Setup
- [SETUP.md](SETUP.md) - macOS Setup Guide

### 📋 Development Guides

- [docs/development/TODOS.md](docs/development/TODOS.md) - Offene Aufgaben & Prioritäten
- [docs/development/STYLE_GUIDE.md](docs/development/STYLE_GUIDE.md) - Code Standards & Naming Conventions
- [docs/development/architecture.md](docs/development/architecture.md) - System-Architektur
- [docs/development/testing.md](docs/development/testing.md) - Testing Guide
- [docs/development/connections.md](docs/development/connections.md) - Service-Verbindungen
- [docs/development/SERVICE_CONFIG.md](docs/development/SERVICE_CONFIG.md) - Service-Konfiguration

### 🔐 Guides

- [docs/guides/ZITADEL_SETUP.md](docs/guides/ZITADEL_SETUP.md) - ZITADEL Konfiguration

## 🧪 Testing

### Backend Tests
```bash
cd backend && cargo test
```

### CI/CD
GitHub Actions Workflows für:
- Backend: Format, Clippy, Tests, Build
- Frontend: TypeScript Check, Build
- Protobuf: Lint, Format

## 📊 Projekt-Status

### ✅ Abgeschlossen
- Connect-RPC vollständig implementiert
- Health Checks verbessert
- GitHub Workflows erstellt
- Dokumentation konsolidiert

### 🔄 In Arbeit
- Frontend Tests implementieren
- High-Priority TODOs (siehe [TODOS.md](docs/development/TODOS.md))

## 🤝 Beitragen

1. Prüfe [TODOS.md](docs/development/TODOS.md) für bekannte Aufgaben
2. Folge [Style Guide](docs/development/STYLE_GUIDE.md) für Code Standards
3. Verwende [Testing Guide](docs/development/testing.md) für Tests
4. Dokumentiere neue Features

## 📞 Support

Bei Fragen oder Problemen:
1. Prüfe [ESSENTIAL_GUIDE.md](docs/ESSENTIAL_GUIDE.md) - Troubleshooting Sektion
2. Schaue [TODOS.md](docs/development/TODOS.md) für bekannte Issues
3. Prüfe [Connections Guide](docs/reference/connections.md) für Service-Probleme

## Lizenz

MIT
