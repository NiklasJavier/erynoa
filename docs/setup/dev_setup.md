# 🚀 Development Setup - Container-in-Container Entwicklung

## Architektur

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Dev Container (VS Code)                          │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │              Docker Compose Services                          │    │
│  ├─────────────────────────────────────────────────────────────┤    │
│  │                                                               │    │
│  │  Console (Container)     Backend (Container)                 │    │
│  │  ─────────────────────    ────────────────────                │    │
│  │  Port: 5173               Port: 3000                          │    │
│  │  Vite HMR ✓               cargo watch ✓                       │    │
│  │  Hot-reload on save       Hot-reload on save                  │    │
│  │                                                               │    │
│  │  ─────────────────────────────────────────────────────────   │    │
│  │                    Hintergrund-Services                       │    │
│  │  ─────────────────────────────────────────────────────────   │    │
│  │  PostgreSQL (db)      :5432   │  MinIO (minio)    :9000/9001 │    │
│  │  DragonflyDB (cache)  :6379   │  ZITADEL (zitadel):8080      │    │
│  │                                                               │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

```bash
just dev
```

Das ist alles! Dieser Befehl:
1. Startet Hintergrund-Services (DB, Cache, MinIO, ZITADEL)
2. Wartet auf Health-Checks
3. Führt Init-Skripte aus (nur beim ersten Mal)
4. Startet Console + Backend mit Hot-Reload und sichtbaren Logs

**URLs:**
| Service | URL | Beschreibung |
|---------|-----|--------------|
| Console | http://localhost:3001/console | SvelteKit App (via Caddy Proxy) |
| Platform | http://localhost:3001/platform | SvelteKit App (via Caddy Proxy) |
| Docs | http://localhost:3001/docs | SvelteKit App (via Caddy Proxy) |
| Backend | http://localhost:3000 | Rust API |
| ZITADEL | http://localhost:8080 | Auth Console |
| MinIO | http://localhost:9001 | Storage Console |

**Test Login:**
- User: `testuser` / Password: `Test123!`
- Admin: `zitadel-admin` / Password: `Password1!`

## 📁 Projektstruktur

```
/workspace
├── backend/                 # Rust API Server
│   ├── src/                 # Source Code
│   ├── config/              # Konfigurationsdateien
│   │   ├── base.toml        # Standard-Konfig
│   │   ├── local.toml       # Local Overrides (auto-generated)
│   │   └── production.toml  # Production Overrides
│   ├── migrations/          # SQL Migrations
│   └── tests/               # Integration Tests
│
├── frontend/               # Frontend Monorepo (pnpm Workspace)
│   ├── console/            # SvelteKit Console
│   ├── platform/           # SvelteKit Platform
│   └── docs/               # SvelteKit Docs
│   ├── src/
│   │   ├── api/             # API Client (Connect-RPC)
│   │   ├── components/      # UI Komponenten
│   │   ├── lib/             # Auth, Config, Utils
│   │   └── pages/           # Seiten
│   └── dist/                # Production Build
│
├── infra/                   # Infrastructure & Deployment
│   ├── docker/              # Docker Compose & Dockerfiles
│   │   ├── docker-compose.yml
│   │   ├── Dockerfile.backend
│   │   ├── Dockerfile.console
│   │   ├── Dockerfile.platform
│   │   └── Dockerfile.docs
│   ├── proxy/               # Reverse Proxy
│   │   └── Caddyfile
│   ├── auth/                # Authentication
│   │   └── zitadel/         # ZITADEL Init-Config
│   │       └── init-steps.yaml
│   └── static/              # Static Files
│       └── landing.html
│
├── backend/
│   └── proto/               # Protobuf Definitionen
├── docs/                    # Dokumentation
├── .data/                   # Lokale Daten (gitignored)
└── justfile                 # Task Runner
```

## 🔧 Wichtige Befehle

### Entwicklung

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | **Startet alles** - Console + Platform + Docs + Backend + Services |
| `just dev [frontend]` | Startet spezifisches Frontend (console, platform, docs, all) |
| `just status` | Zeigt Status aller Services |
| `just check` | Health Check aller Services |
| `just init` | Initialisierung ohne Dev-Server (erstellt auch `.env` aus `.env.example`) |
| `just init-env` | Erstellt `.env` aus `.env.example` (für Neuaufstellung) |
| `just restart` | Schneller Neustart aller Dev-Services |
| `just stop` | Stoppt alle Container |

### Einzelne Services

| Befehl | Beschreibung |
|--------|--------------|
| `just dev-backend` | Nur Backend (Services müssen laufen) |
| `just dev-console` | Nur Console (Services müssen laufen) |
| `just dev-platform` | Nur Platform (Services müssen laufen) |
| `just dev-docs` | Nur Docs (Services müssen laufen) |
| `just services` | Nur Hintergrund-Services starten |

### Setup & Reset

| Befehl | Beschreibung |
|--------|--------------|
| `just init` | Initialisierung ohne Dev-Server (erstellt auch `.env` aus `.env.example`) |
| `just init-env` | Erstellt `.env` aus `.env.example` (für Neuaufstellung) |
| `just zitadel-setup` | ZITADEL neu konfigurieren |
| `just minio-setup` | MinIO Buckets erstellen |
| `just reset` | **Alles löschen** und neu starten |

### Logs & Debug

| Befehl | Beschreibung |
|--------|--------------|
| `just logs` | Alle Container-Logs |
| `just logs [service]` | Logs für spezifischen Service (backend, console, platform, docs, proxy) |
| `just shell [service]` | Shell in Container (backend, console, platform, docs) |

## ⚙️ Konfiguration

### Konfigurationspriorität (höchste zuerst):
1. **Umgebungsvariablen** (`APP_DATABASE__HOST=db`)
2. **local.toml** (auto-generated, gitignored)
3. **base.toml** (Standard-Werte)

### Docker-Compose Umgebungsvariablen

Die wichtigsten Overrides in `docker-compose.yml`:
```yaml
environment:
  # Database → Docker Service Name
  - APP_DATABASE__HOST=db
  # Cache → Docker Service Name  
  - APP_CACHE__URL=redis://cache:6379
  # Auth → Externe + Interne URL
  - APP_AUTH__ISSUER=http://localhost:8080
  - APP_AUTH__INTERNAL_ISSUER=http://zitadel:8080
  # Storage → Docker Service Name
  - APP_STORAGE__ENDPOINT=http://minio:9000
```

## 🔄 Hot-Reloading

### Backend (Rust)
- **Tool**: `cargo-watch`
- **Watched**: `src/`, `Cargo.toml`, `config/`, `backend/proto/`
- **Rebuild-Zeit**: ~5-15 Sekunden

### Console (Vite)
- **Tool**: Vite HMR
- **Watched**: Alle Dateien in `src/`
- **Rebuild-Zeit**: <100ms

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
lsof -i :3000  # oder :3001, :8080
```

---

## 📚 Weitere DevContainer-Dokumentation

Für detaillierte Informationen zu spezifischen DevContainer-Themen:

### 🔗 Schnellzugriff

- **[Database & Cache Verbindungen](devcontainer/database_connection.md)** - VS Code Extensions für PostgreSQL und Redis/Dragonfly
- **[Git-Konfiguration](devcontainer/git_setup.md)** - 1:1 Übernahme der Host-Git-Einstellungen
- **[Port-Forwarding](devcontainer/ports.md)** - Übersicht aller weitergeleiteten Ports

### 📖 Details

#### Database & Cache Verbindungen
Die VS Code IDE im DevContainer kann direkt mit der Datenbank und dem Cache verbinden:
- **PostgreSQL Extension**: Datenbank-Explorer, SQL Query Editor, Schema-Visualisierung
- **Redis/Dragonfly Extension**: Key-Explorer, Key-Editor, TTL-Verwaltung

Siehe: [database_connection.md](devcontainer/database_connection.md)

#### Git-Konfiguration
Der DevContainer übernimmt automatisch alle Git-Einstellungen vom Host:
- User-Konfiguration (name, email)
- Signing-Konfiguration (SSH oder GPG)
- SSH-Keys und Signing-Keys

Siehe: [git_setup.md](devcontainer/git_setup.md)

#### Port-Forwarding
Übersicht aller weitergeleiteten Ports und Auto-Forward-Einstellungen:
- Development Services (Backend, Frontends, Proxy)
- Datenbanken & Cache (PostgreSQL, Redis/Dragonfly)
- S3 Object Storage (MinIO)
- Authentication (ZITADEL)

Siehe: [ports.md](devcontainer/ports.md)
