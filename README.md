# Erynoa Monorepo

Full-Stack Application mit Rust Backend und SvelteKit Frontends.

## 🚀 Quick Start

```bash
just dev
```

Startet alles:
- **Proxy** auf http://localhost:3001 (Caddy Reverse Proxy für Frontends)
  - **Console** auf http://localhost:3001/console
  - **Platform** auf http://localhost:3001/platform
  - **Docs** auf http://localhost:3001/docs
- **Backend** direkt auf http://localhost:3000 (Rust API, nicht über Proxy)
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
├── frontend/          # Frontend Applications
│   ├── console/       # Console (SvelteKit)
│   ├── platform/      # Platform (SvelteKit)
│   └── docs/          # Docs (SvelteKit)
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

- **[docs/README.md](docs/README.md)** - **Dokumentations-Übersicht** mit Quick Start
- **[ESSENTIAL_GUIDE.md](docs/ESSENTIAL_GUIDE.md)** - Konsolidierter Guide mit allen wichtigen Informationen

### 🚀 Guides (Schritt-für-Schritt Anleitungen)

- [Getting Started](docs/guides/getting-started.md) - Erste Schritte mit dem Projekt
- [Setup](docs/guides/setup.md) - Entwicklungsumgebung einrichten
- [ZITADEL Setup](docs/guides/zitadel.md) - Authentifizierung konfigurieren

### 📗 Reference (Technische Referenz)

- [Architecture](docs/reference/architecture.md) - Systemarchitektur und Design-Entscheidungen
- [Configuration](docs/reference/config.md) - Service-Konfiguration und Verbindungen
- [Connections](docs/reference/connections.md) - API-Verbindungen und Harmonisierung

### 📙 Development (Development-spezifisch)

- [Style Guide](docs/development/style-guide.md) - Code-Stil und Best Practices
- [Testing](docs/development/testing.md) - Test-Strategien und -Tools
- [TODOs](docs/development/todos.md) - Offene Aufgaben und Prioritäten
- [REST Deprecation Plan](docs/development/REST_DEPRECATION_PLAN.md) - Plan zur REST-API Entfernung
- [Structure Improvements](docs/development/STRUCTURE_IMPROVEMENTS.md) - Strukturverbesserungen

## 🧪 Testing

### Backend Tests
```bash
cd backend && cargo test
```

### CI/CD
GitHub Actions Workflows für:
- Backend: Format, Clippy, Tests, Build
- Console/Platform/Docs: TypeScript Check, Build
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
