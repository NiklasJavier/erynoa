# 📚 Essential Guide - Alles was du brauchst

**Letzte Aktualisierung**: 2026-01-25

Diese Datei konsolidiert alle wichtigen Informationen aus den verschiedenen Dokumenten.

---

## 🚀 Quick Start

```bash
just dev
```

Startet alles:
- Console: http://localhost:5173
- Backend: http://localhost:3000
- ZITADEL: http://localhost:8080
- MinIO: http://localhost:9001

**Test Login:**
- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

---

## 📋 Offene TODOs (Priorität)

### 🔴 High Priority

1. **Storage Upload - Progress Tracking**
   - Datei: `frontend/console/src/api/storage/connect-client.ts:62`
   - Schätzung: 4-6 Stunden
   - Connect-RPC unterstützt kein Upload-Progress nativ, evtl. Presigned URLs für große Dateien

2. **User Service - GetCurrentUser**
   - Datei: `frontend/console/src/api/users/connect-client.ts:90`
   - Schätzung: 3-4 Stunden
   - Backend RPC-Methode oder Token-Parsing implementieren

### 🟡 Medium Priority

3. **Error Handling - RpcError Conversion** (Backend)
   - Datei: `backend/src/auth/claims.rs:155`
   - Schätzung: 2-3 Stunden

4. **Storage Service - Error Handling** (Backend)
   - Datei: `backend/src/api/v1/storage/connect.rs`
   - Schätzung: 3-4 Stunden

5. **Feature Flags** (Console)
   - Datei: `frontend/console/src/lib/config.ts`
   - Schätzung: 2-3 Stunden

6. **Error Boundary - Connect-RPC Errors** (Console)
   - Datei: `frontend/console/src/components/ErrorBoundary.tsx`
   - Schätzung: 2-3 Stunden

### 🟢 Low Priority

- REST Client Deprecation (Planung)
- Documentation - API Examples
- REST Client Removal (Console)
- Type Definitions Cleanup

**Vollständige Liste**: `docs/development/TODOS.md`

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
│   └── config/           # Konfiguration (TOML)
├── console/              # SvelteKit Console
│   ├── src/api/          # API Clients (Connect-RPC)
│   ├── src/components/   # UI Komponenten
│   └── src/lib/          # Auth, Config, Utils
├── proto/                # Protobuf Definitionen
├── infra/                # Docker & Deployment
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

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | Startet alles (Console + Backend + Services) |
| `just status` | Zeigt Status aller Services |
| `just dev-check` | Health Check aller Services |
| `just docker-stop` | Stoppt alle Container |
| `just reset` | Alles löschen und neu starten |
| `just lint` | Backend Clippy |
| `just fmt` | Backend Format |
| `just test` | Backend Tests |
| `just proto-gen` | Protobuf Types generieren |

Alle Befehle: `just --list`

---

## 🔗 Service-Konfiguration

### Service URLs (Development)

| Service | Port | URL |
|---------|------|-----|
| Console | 5173 | http://localhost:5173 |
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

**Vollständiger Style Guide**: `docs/development/STYLE_GUIDE.md`

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

**Vollständiger Testing Guide**: `docs/development/testing.md`

---

## 🔐 ZITADEL Setup

### Quick Setup

1. ZITADEL Console öffnen: http://localhost:8080/ui/console
2. Erstanmeldung: `zitadel-admin@zitadel.localhost` / `Password1!`
3. Projekt erstellen: `erynoa`
4. API Application erstellen: `erynoa-api`
5. Test-User erstellen: `testuser` / `Test123!`

**Vollständiger Guide**: `docs/guides/ZITADEL_SETUP.md`

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

**Vollständiger Guide**: `docs/development/CONNECT_RPC_GUIDE.md`

---

## ⚙️ Konfiguration

### Backend

**Datei**: `backend/config/base.toml`

```toml
[application]
api_url = "http://localhost:3000"
console_url = "http://localhost:5173"

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
just docker-backend-shell
cargo check  # Zeigt Fehler
```

### Port bereits belegt
```bash
just docker-stop
lsof -i :3000  # Check welcher Prozess
```

### Logs prüfen
```bash
just docker-logs          # Alle
just docker-logs-backend  # Nur Backend
just docker-logs-console # Nur Console
```

---

## 📊 Projekt-Status

### ✅ Abgeschlossen

- ✅ Phase 1: Quick Wins (Error-Interceptor, Logging, Style Guide)
- ✅ Phase 2: Strukturelle Verbesserungen (Feature-basierte API, Protobuf-Types)
- ✅ Phase 3: Langfristige Verbesserungen (Test-Struktur, TODO-Management)
- ✅ Connect-RPC vollständig implementiert
- ✅ Health Checks verbessert
- ✅ GitHub Workflows erstellt

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

- **README.md** - Projekt-Übersicht
- **docs/setup/DEV_SETUP.md** - Entwicklungsumgebung
- **docs/setup/DOCKER.md** - Docker-spezifische Infos
- **docs/development/TODOS.md** - Offene Aufgaben
- **docs/development/STYLE_GUIDE.md** - Code Standards
- **docs/development/architecture.md** - System-Architektur
- **docs/development/testing.md** - Testing Guide
- **docs/guides/ZITADEL_SETUP.md** - ZITADEL Konfiguration

### Historische Dokumente (Referenz)

- `docs/development/HARMONIZATION_ROADMAP.md` - Harmonisierung (abgeschlossen)
- `docs/development/PHASE_3_COMPLETE.md` - Phase 3 Status
- `docs/changelog/*.md` - Changelog Einträge

---

## 🔄 Workflow

### Neue Features entwickeln

1. Prüfe [TODOS.md](development/TODOS.md) für bekannte Aufgaben
2. Folge [Style Guide](development/STYLE_GUIDE.md) für Naming
3. Verwende [Testing Structure](development/testing.md) für Tests
4. Dokumentiere in [TODOS.md](development/TODOS.md) wenn nötig

### Bug-Fixes

1. Prüfe Troubleshooting Guides
2. Schaue [Connections Guide](development/connections.md) für Service-Probleme
3. Prüfe [TODO Management](development/TODOS.md) für bekannte Issues

---

**Letzte Aktualisierung**: 2026-01-25
