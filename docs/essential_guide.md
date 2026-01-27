# 📚 Essential Guide - Alles was du brauchst

**Letzte Aktualisierung**: 2026-01-27 (20:57)

Diese Datei konsolidiert alle wichtigen Informationen aus den verschiedenen Dokumenten.

---

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

**Voraussetzungen:**
- Nix installiert (siehe [Setup Guide](setup/setup.md))
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

**Test Login:**
- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

---

## 📋 Offene TODOs

**Status**: Alle High-Priority TODOs abgeschlossen ✅

**Vollständige Liste**: [docs/development/todos.md](development/todos.md)

### 🔄 In Arbeit
- Frontend Tests implementieren
- Performance Monitoring
- Erweiterte Error-Tracking

### 📅 Geplant
- REST Endpoints deprecaten
- Documentation - API Examples
- Type Definitions Cleanup

---

## 🏗️ Architektur

### Projektstruktur

```
/workspace
├── backend/              # Rust API (Axum + Connect-RPC)
│   ├── src/api/v1/       # Feature-basierte API
│   │   ├── health/       # Health Checks
│   │   ├── info/         # Info & Status
│   │   ├── users/        # User Management
│   │   └── storage/      # Storage Operations
│   ├── config/           # Konfiguration (TOML)
│   └── proto/            # Protobuf Definitionen
├── frontend/            # Frontend Monorepo (pnpm Workspace)
│   ├── console/         # SvelteKit Console
│   ├── platform/        # SvelteKit Platform
│   └── docs/            # SvelteKit Docs
│   ├── src/api/          # API Clients (Connect-RPC)
│   ├── src/components/   # UI Komponenten
│   └── src/lib/          # Auth, Config, Utils
├── infra/                # Infrastructure & Deployment
│   ├── docker/          # Docker Compose & Dockerfiles
│   ├── proxy/           # Reverse Proxy (Caddyfile)
│   ├── auth/            # Authentication (ZITADEL)
│   └── static/          # Static Files (landing.html)
└── docs/                 # Dokumentation
```

### Tech Stack

| Komponente | Technologie |
|------------|-------------|
| Backend | Rust, Axum, Tokio, SQLx |
| Console | SvelteKit, Tailwind |
| API | Connect-RPC/gRPC-Web (Protobuf) |
| Auth | ZITADEL (OIDC/JWT) |
| Database | PostgreSQL (OrioleDB) |
| Cache | DragonflyDB (Redis) |
| Storage | MinIO (S3) |

---

## 🔧 Wichtige Befehle

### Entwicklung
| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | Startet alles (Console + Platform + Docs + Backend + Services) |
| `just dev [frontend]` | Startet spezifisches Frontend (console, platform, docs, all) |
| `just status` | Zeigt Status aller Services |
| `just check` | Health Check aller Services |
| `just restart` | Schneller Neustart aller Dev-Services |

### 📦 Nix Installation

**macOS:**
```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

**Ubuntu/Debian:**
```bash
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

Terminal neu starten, dann verifizieren:
```bash
nix --version
```

**Was Nix automatisch bereitstellt:**
- ✅ Rust Toolchain (inkl. rust-analyzer, clippy)
- ✅ Node.js & pnpm
- ✅ buf (Protobuf)
- ✅ just (Task Runner)
- ✅ sqlx CLI
- ✅ Alle Build-Tools

**Vorteile:**
- ⚡ **Schnell**: Keine manuelle Tool-Installation nötig
- 🔒 **Reproduzierbar**: Gleiche Tools für alle Entwickler
- 🧹 **Sauber**: Keine System-Installationen (außer Nix selbst)

Siehe [Setup Guide](setup/setup.md#-schnelles-setup-mit-nix-empfohlen-für-erfahrene-entwickler) für Details.

### Container Management
| Befehl | Beschreibung |
|--------|--------------|
| `just stop` | Stoppt alle Container |
| `just logs [service]` | Logs anzeigen (alle oder spezifischer Service) |
| `just shell [service]` | Shell in Container (backend, console, platform, docs) |
| `just build` | Baue alle Docker Images |

### Code Quality
| Befehl | Beschreibung |
|--------|--------------|
| `just lint` | Backend Clippy |
| `just fmt` | Backend Format |
| `just test` | Backend Tests (mit cargo-nextest) |
| `just proto-gen` | Protobuf Types generieren |
| `just frontend-lint` | Frontend Lint (Biome) |
| `just frontend-check` | Frontend TypeScript Check |

### Setup & Reset
| Befehl | Beschreibung |
|--------|--------------|
| `just init` | Initialisierung ohne Dev-Server |
| `just reset` | Alles löschen und neu starten |
| `just services` | Nur Hintergrund-Services starten |

Alle Befehle: `just --list`

---

## 🔗 Service-Konfiguration

### Service URLs (Development)

| Service | Port | URL |
|---------|------|-----|
| Console | 3001/console | http://localhost:3001/console (via Caddy Proxy) |
| Platform | 3001/platform | http://localhost:3001/platform (via Caddy Proxy) |
| Docs | 3001/docs | http://localhost:3001/docs (via Caddy Proxy) |
| Proxy | 3001 | http://localhost:3001 (Caddy Reverse Proxy) |
| Backend | 3000 | http://localhost:3000 |
| Database | 5432 | postgresql://localhost:5432 |
| Cache | 6379 | redis://localhost:6379 |
| MinIO API | 9000 | http://localhost:9000 |
| MinIO Console | 9001 | http://localhost:9001 |
| ZITADEL | 8080 | http://localhost:8080 |

### Docker Service Names (Internal)

| Service | Docker Name |
|---------|-------------|
| Database | `db` |
| Cache | `cache` |
| Storage | `minio` |
| Auth | `zitadel` |

**Connection Strings im Docker:**
- Database: `postgresql://erynoa:erynoa@db:5432/erynoa`
- Cache: `redis://cache:6379`
- Storage: `http://minio:9000`
- Auth: `http://zitadel:8080`

---

## 📝 Code Standards

### Naming Conventions

**Backend (Rust):**
- Functions: `snake_case` (z.B. `create_user`)
- Structs/Enums: `PascalCase` (z.B. `UserResponse`)
- Modules: `snake_case` (z.B. `user_handler`)
- Constants: `SCREAMING_SNAKE_CASE` (z.B. `API_VERSION`)

**Console (TypeScript):**
- Functions: `camelCase` (z.B. `createUser`)
- Classes/Interfaces: `PascalCase` (z.B. `UserResponse`)
- Files: `kebab-case.ts` oder `PascalCase.tsx` (Components)
- Constants: `SCREAMING_SNAKE_CASE` (z.B. `API_VERSION`)

### File Organization

**Backend API:**
```
api/v1/{feature}/
├── handler.rs      # REST handlers
├── connect.rs      # Connect-RPC handlers
├── models.rs       # Request/Response types
├── routes.rs       # Route definitions
└── mod.rs          # Module exports
```

**Console API:**
```
api/{feature}/
├── connect-client.ts  # Connect-RPC client
├── types.ts          # Type definitions (from proto)
└── index.ts          # Public API
```

**Vollständiger Style Guide**: [docs/development/style-guide.md](development/style-guide.md)

---

## 🧪 Testing

### Backend Tests

```bash
cd backend && cargo test
```

**Location**: `backend/tests/api.rs`
- Integration Tests für alle Endpoints
- TestApp Helper für Server-Setup
- 13+ Tests (Health, Info, Users, Storage, Routes, CORS)

### Console Tests

**Status**: Vorbereitet für zukünftige Implementierung
**Empfohlene Struktur**: `frontend/console/src/**/__tests__/`

**Vollständiger Testing Guide**: [docs/development/testing.md](development/testing.md)

---

## 🔐 ZITADEL Setup

### Quick Setup

1. ZITADEL Console öffnen: http://localhost:8080/ui/console
2. Erstanmeldung: `zitadel-admin@zitadel.localhost` / `Password1!`
3. Projekt erstellen: `erynoa`
4. API Application erstellen: `erynoa-api`
5. Test-User erstellen: `testuser` / `Test123!`

**Vollständiger Guide**: [docs/guides/zitadel.md](guides/zitadel.md)

---

## 🔌 Connect-RPC

### Status

✅ **Vollständig implementiert und aktiv**
- Backend: Connect-RPC Handler für alle Services
- Console: Connect-RPC Clients für alle Services
- Protobuf: Alle Services definiert
- Authentication: JWT Token Injection

### Protobuf Services

- `HealthService` - Health Checks
- `InfoService` - Info & Status
- `UserService` - User Management
- `StorageService` - Storage Operations

**Vollständiger Guide**: Siehe [docs/reference/architecture.md](reference/architecture.md) für Connect-RPC Details

---

## ⚙️ Konfiguration

### Backend

**Datei**: `backend/config/base.toml`

```toml
[application]
api_url = "http://localhost:3000"
console_url = "http://localhost:3001/console"

[database]
host = "db"  # "localhost" außerhalb Docker
port = 5432
username = "erynoa"
password = "erynoa"
database = "erynoa"

[cache]
url = "redis://cache:6379"  # "redis://localhost:6379" außerhalb Docker

[storage]
endpoint = "http://minio:9000"  # "http://localhost:9000" außerhalb Docker
region = "us-east-1"
access_key_id = "erynoa"
secret_access_key = "erynoa123"
default_bucket = "erynoa"

[auth]
issuer = "http://localhost:8080"
internal_issuer = "http://zitadel:8080"
client_id = "erynoa-backend"
console_client_id = "erynoa-console"
```

**Konfigurationspriorität:**
1. Umgebungsvariablen (`APP_DATABASE__HOST=db`)
2. `local.toml` (auto-generated, gitignored)
3. `base.toml` (Standard-Werte)

### Console

**Datei**: `frontend/console/src/lib/config.ts`

Konfiguration wird vom Backend `/api/v1/info` Endpoint geladen.

---

## 🐛 Troubleshooting

### Services starten nicht
```bash
just reset
just dev
```

### ZITADEL Client-ID ungültig
```bash
just zitadel-reset
```

### Backend kompiliert nicht
```bash
just shell backend
cargo check  # Zeigt Fehler
```

### Port bereits belegt
```bash
just stop
lsof -i :3000  # Check welcher Prozess
```

### Logs prüfen
```bash
just logs              # Alle
just logs backend      # Nur Backend
just logs console      # Nur Console
just logs platform     # Nur Platform
just logs docs         # Nur Docs
```

---

## 📊 Projekt-Status

### ✅ Abgeschlossen

- ✅ Phase 1: Quick Wins (Error-Interceptor, Logging, Style Guide)
- ✅ Phase 2: Strukturelle Verbesserungen (Feature-basierte API, Protobuf-Types)
- ✅ Phase 3: Langfristige Verbesserungen (Test-Struktur, TODO-Management)
- ✅ Connect-RPC vollständig implementiert
- ✅ Monorepo mit pnpm Workspace & Turborepo
- ✅ Svelte 5 Migration (Runes: $state, $derived, $effect)
- ✅ Health Checks verbessert
- ✅ GitHub Workflows optimiert (Turborepo, cargo-nextest, pnpm)
- ✅ Justfile optimiert (neue Befehle, bessere Performance)
- ✅ VS Code Extensions optimiert (22 Extensions)
- ✅ DevContainer optimiert (ein Terminal, bessere Konfiguration)

### 🔄 In Arbeit

- Console Tests implementieren
- High-Priority TODOs (siehe oben)

### 📅 Geplant

- REST Endpoints deprecaten
- Performance Monitoring
- Erweiterte Error-Tracking

---

## 📚 Weitere Dokumentation

### Wichtigste Dokumente

- **[readme.md](../readme.md)** - Projekt-Übersicht
- **[Dev Setup](setup/dev_setup.md)** - Entwicklungsumgebung
- **[Docker Setup](setup/docker.md)** - Docker-spezifische Infos
- **[todos](development/todos.md)** - Offene Aufgaben
- **[Style Guide](development/style-guide.md)** - Code Standards
- **[Architecture](reference/architecture.md)** - System-Architektur
- **[Testing](development/testing.md)** - Testing Guide
- **[ZITADEL Setup](guides/zitadel.md)** - ZITADEL Konfiguration

### Historische Dokumente (Referenz)

- `docs/development/HARMONIZATION_ROADMAP.md` - Harmonisierung (abgeschlossen)
- `docs/development/PHASE_3_COMPLETE.md` - Phase 3 Status
- `docs/changelog/*.md` - Changelog Einträge

---

## 🔄 Workflow

### Neue Features entwickeln

1. Prüfe [todos](development/todos.md) für bekannte Aufgaben
2. Folge [Style Guide](development/style-guide.md) für Naming
3. Verwende [Testing Guide](development/testing.md) für Tests
4. Dokumentiere in [TODOs](development/TODOS.md) wenn nötig

### Bug-Fixes

1. Prüfe Troubleshooting Guides
2. Schaue [Connections Guide](reference/connections.md) für Service-Probleme
3. Prüfe [TODOs](development/TODOS.md) für bekannte Issues

---

**Letzte Aktualisierung**: 2026-01-27
