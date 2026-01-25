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
│  │  Frontend (Container)     Backend (Container)                 │    │
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
4. Startet Frontend + Backend mit Hot-Reload und sichtbaren Logs

**URLs:**
| Service | URL | Beschreibung |
|---------|-----|--------------|
| Frontend | http://localhost:5173 | SolidJS App |
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
├── frontend/                # SolidJS Frontend
│   ├── src/
│   │   ├── api/             # API Client (Connect-RPC)
│   │   ├── components/      # UI Komponenten
│   │   ├── lib/             # Auth, Config, Utils
│   │   └── pages/           # Seiten
│   └── dist/                # Production Build
│
├── infra/                   # Infrastructure & Deployment
│   ├── docker-compose.yml   # Service-Definitionen
│   ├── Dockerfile.backend   # Backend Container
│   ├── Dockerfile.frontend  # Frontend Container
│   ├── Caddyfile            # Reverse Proxy Config
│   ├── scripts/             # Setup-Skripte
│   │   ├── setup-zitadel.sh # ZITADEL Initialisierung
│   │   └── setup-minio.sh   # MinIO Buckets
│   └── zitadel/             # ZITADEL Init-Config
│
├── proto/                   # Protobuf Definitionen
├── docs/                    # Dokumentation
├── .data/                   # Lokale Daten (gitignored)
└── justfile                 # Task Runner
```

## 🔧 Wichtige Befehle

### Entwicklung

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | **Startet alles** - Frontend + Backend + Services |
| `just status` | Zeigt Status aller Services |
| `just restart-dev` | Schneller Neustart von Frontend + Backend |
| `just docker-stop` | Stoppt alle Container |

### Einzelne Services

| Befehl | Beschreibung |
|--------|--------------|
| `just dev-backend` | Nur Backend (Services müssen laufen) |
| `just dev-frontend` | Nur Frontend (Services müssen laufen) |
| `just services` | Nur Hintergrund-Services starten |

### Setup & Reset

| Befehl | Beschreibung |
|--------|--------------|
| `just init` | Initialisierung ohne Dev-Server |
| `just zitadel-setup` | ZITADEL neu konfigurieren |
| `just minio-setup` | MinIO Buckets erstellen |
| `just reset` | **Alles löschen** und neu starten |

### Logs & Debug

| Befehl | Beschreibung |
|--------|--------------|
| `just docker-logs` | Alle Container-Logs |
| `just docker-logs-backend` | Nur Backend-Logs |
| `just docker-backend-shell` | Shell im Backend-Container |

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
- **Watched**: `src/`, `Cargo.toml`, `config/`, `proto/`
- **Rebuild-Zeit**: ~5-15 Sekunden

### Frontend (Vite)
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
just docker-backend-shell
cargo check  # Zeigt Fehler
```

### Port bereits belegt
```bash
just docker-stop
lsof -i :3000  # oder :5173, :8080
```
