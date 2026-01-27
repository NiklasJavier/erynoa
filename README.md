# Erynoa Monorepo

Full-Stack Application mit Rust Backend und SvelteKit Frontends.

## 🚀 Quick Start

**3 Schritte zum laufenden Projekt:**

```bash
# 1. Repository klonen
git clone git@github.com:NiklasJavier/erynoa.git
cd erynoa

# 2. Nix Dev-Shell betreten (lädt alle Tools automatisch)
nix develop

# 3. Projekt starten
just dev
```

**4. Warte 2 Minuten** ⏳

Die Services starten und ZITADEL wird automatisch konfiguriert. Nach ca. 2 Minuten kannst du im Browser öffnen:

```
http://localhost:3001
```

**Voraussetzungen:** (siehe [Setup Guide](docs/setup/setup.md))
- Nix installiert
- Docker Desktop installiert und gestartet

**Fertig!** 🎉

Das startet alles:
- **Proxy** auf http://localhost:3001 (Caddy Reverse Proxy)
  - **Console** auf http://localhost:3001/console
  - **Platform** auf http://localhost:3001/platform
  - **Docs** auf http://localhost:3001/docs
  - **Backend API** auf http://localhost:3001/api
- **Backend** direkt auf http://localhost:3000 (für Tests)
- **ZITADEL** auf http://localhost:8080 (Auth)
- **MinIO** auf http://localhost:9001 (S3 Storage Console)
- PostgreSQL (OrioleDB), DragonflyDB (Redis-kompatibel) im Hintergrund

**Test Login:**
- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

## 📁 Projektstruktur

```
├── backend/           # Rust API Server (Axum + Connect-RPC)
│   ├── src/           # Source Code
│   ├── config/        # Konfiguration (TOML)
│   ├── migrations/    # SQL Migrations
│   └── proto/         # Protobuf Definitionen
├── frontend/          # Frontend Monorepo (pnpm Workspace)
│   ├── console/       # Console (SvelteKit)
│   ├── platform/      # Platform (SvelteKit)
│   └── docs/          # Docs (SvelteKit)
├── infra/             # Infrastructure & Deployment
│   ├── docker/        # Docker Compose & Dockerfiles
│   │   ├── docker-compose.yml
│   │   └── Dockerfile.*
│   ├── proxy/         # Reverse Proxy
│   │   └── Caddyfile
│   ├── auth/          # Authentication
│   │   └── zitadel/   # ZITADEL Init
│   └── static/        # Static Files
│       └── landing.html
├── docs/              # Dokumentation
├── buf.gen.yaml       # Protobuf Code-Generierung (TypeScript)
├── buf.yaml           # Protobuf Module-Konfiguration
└── justfile           # Task Runner
```

## 🛠️ Tech Stack

| Komponente | Technologie |
|------------|-------------|
| **Backend** | Rust, Axum, Tokio, SQLx |
| **Console** | SvelteKit, Tailwind |
| **Platform** | SvelteKit, Tailwind |
| **Docs** | SvelteKit, Tailwind |
| **API** | Connect-RPC/gRPC-Web (Protobuf) |
| **Auth** | ZITADEL (OIDC/JWT) |
| **Database** | PostgreSQL (OrioleDB) |
| **Cache** | DragonflyDB (Redis) |
| **Storage** | MinIO (S3) |
| **Build** | Nix + Crane, Vite |

## 🔧 Wichtige Befehle

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | **Startet alles** (Console + Platform + Docs + Backend + Services) |
| `just dev [frontend]` | Startet spezifisches Frontend (console, platform, docs, all) |
| `just status` | Zeigt Status aller Services |
| `just check` | Health Check aller Services |
| `just stop` | Stoppt alle Container |
| `just logs [service]` | Logs anzeigen (alle oder spezifischer Service) |
| `just shell [service]` | Shell in Container (backend, console, platform, docs) |
| `just restart` | Schneller Neustart aller Dev-Services |
| `just reset` | Alles löschen und neu starten |
| `just init` | Initialisierung ohne Dev-Server |
| `just init-env` | Erstellt `.env` aus `.env.example` |
| `just lint` | Backend Clippy |
| `just fmt` | Backend Format |
| `just test` | Backend Tests (mit cargo-nextest) |
| `just proto-gen` | Protobuf Types generieren |

Alle Befehle: `just --list`

## 📖 Dokumentation

Vollständige Dokumentation findest du im [`docs/`](docs/) Verzeichnis:

### 📚 Hauptdokumentation

- **[docs/readme.md](docs/readme.md)** - **Dokumentations-Übersicht** mit Quick Start
- **[docs/essential_guide.md](docs/essential_guide.md)** - Konsolidierter Guide mit allen wichtigen Informationen

### 🚀 Guides (Schritt-für-Schritt Anleitungen)

- [Getting Started](docs/guides/getting-started.md) - Erste Schritte mit dem Projekt
- [Setup](docs/setup/setup.md) - Entwicklungsumgebung einrichten (macOS)
- [Dev Setup](docs/setup/dev_setup.md) - Container-in-Container Entwicklung
- [Docker](docs/setup/docker.md) - Docker Development Setup
- [ZITADEL Setup](docs/guides/zitadel.md) - Authentifizierung konfigurieren

### 📗 Reference (Technische Referenz)

- [Architecture](docs/reference/architecture.md) - Systemarchitektur und Design-Entscheidungen
- [Configuration](docs/reference/config.md) - Service-Konfiguration und Verbindungen
- [Connections](docs/reference/connections.md) - API-Verbindungen und Harmonisierung

### 📙 Development (Development-spezifisch)

- [Style Guide](docs/development/style-guide.md) - Code-Stil und Best Practices
- [Testing](docs/development/testing.md) - Test-Strategien und -Tools
- [TODOs](docs/development/todos.md) - Offene Aufgaben und Prioritäten
- [REST Deprecation Plan](docs/development/rest_deprecation_plan.md) - Plan zur REST-API Entfernung
- [Structure Improvements](docs/archive/structure_improvements.md) - Strukturverbesserungen

## 🧪 Testing

### Backend Tests
```bash
cd backend && cargo test
```

### CI/CD
GitHub Actions Workflows (optimiert für Performance):
- **Branches**: `main`, `develop`, `new-console-svelte`
- Backend: Format, Clippy, Tests (cargo-nextest), Build
- Frontend: TypeScript Check, Lint (Biome), Build (Turborepo)
- Protobuf: Lint, Format (backend/proto/)
- Turborepo: Parallele Builds mit Caching
- pnpm: Optimiertes Dependency-Management

## 📊 Projekt-Status

### ✅ Abgeschlossen
- Connect-RPC vollständig implementiert
- Monorepo-Struktur mit pnpm Workspace & Turborepo
- SvelteKit Frontends (Console, Platform, Docs) mit Svelte 5
- DevContainer mit vollständiger Entwicklungsumgebung
- VS Code Extensions optimiert (22 Extensions)
- Health Checks und automatische Service-Initialisierung
- GitHub Workflows optimiert (Turborepo, cargo-nextest, pnpm, new-console-svelte Branch)
- Justfile optimiert (neue Befehle: stop, logs, shell, restart, init-env, test-ci, devcontainer-remove, docker-cleanup)
- Infra-Verzeichnis optimiert (nach Typ organisiert: docker/, proxy/, auth/, static/)
- Environment-Setup (.env.example → .env automatisch)
- Dokumentation konsolidiert und organisiert
- Protobuf nach backend/proto/ verschoben (2026-01-27)
- Folder Structure optimiert (Priorität 1 Inkonsistenzen behoben)
- ZITADEL automatisches Setup mit dynamischer App-ID-Generierung
- Caddy Reverse Proxy für alle Frontends (Port 3001)
- Host-basierte Services (Services laufen auf dem Host, nicht im DevContainer)
- Protobuf Code-Generierung für alle Frontends (buf.gen.yaml im Root)

### 🔄 In Arbeit
- Frontend Tests implementieren
- High-Priority TODOs (siehe [todos.md](docs/development/todos.md))

## 🤝 Beitragen

1. Prüfe [todos.md](docs/development/todos.md) für bekannte Aufgaben
2. Folge [Style Guide](docs/development/style-guide.md) für Code Standards
3. Verwende [Testing Guide](docs/development/testing.md) für Tests
4. Dokumentiere neue Features

## 📞 Support

Bei Fragen oder Problemen:
1. Prüfe [essential_guide.md](docs/essential_guide.md) - Troubleshooting Sektion
2. Schaue [todos.md](docs/development/todos.md) für bekannte Issues
3. Prüfe [Connections Guide](docs/reference/connections.md) für Service-Probleme

## Lizenz

MIT
