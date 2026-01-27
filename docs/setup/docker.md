# 🐳 Docker Development Setup

## Übersicht

Das Projekt verwendet einen **Container-in-Container** Ansatz:
- Der DevContainer enthält Docker + Docker Compose
- Alle Services laufen als Container innerhalb des DevContainers
- Hot-Reloading funktioniert durch Volume-Mounts

```
┌── DevContainer ─────────────────────────────────────────┐
│                                                          │
│  ┌── Docker Compose ──────────────────────────────────┐ │
│  │  console (5173)  backend (3000)  db (5432)        │ │
│  │  minio (9000/9001)  cache (6379)  zitadel (8080)   │ │
│  └────────────────────────────────────────────────────┘ │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

## Features

- ✅ **Console Hot-Reload**: Vite HMR (<100ms Updates)
- ✅ **Backend Hot-Reload**: cargo-watch (5-15s Rebuild)
- ✅ **Isolierte Services**: Alle Abhängigkeiten containerisiert
- ✅ **Volume Mounts**: Code-Änderungen sofort sichtbar
- ✅ **Health Checks**: Automatische Abhängigkeitsprüfung

## Quick Start

```bash
just dev
```

Siehe [dev_setup.md](dev_setup.md) für ausführliche Dokumentation.

## Services

| Service | Port | Container Name | Beschreibung |
|---------|------|----------------|--------------|
| Console | 3001/console | console | SvelteKit + Vite (via Caddy Proxy) |
| Platform | 3001/platform | platform | SvelteKit + Vite (via Caddy Proxy) |
| Docs | 3001/docs | docs | SvelteKit + Vite (via Caddy Proxy) |
| Proxy | 3001 | caddy | Caddy Reverse Proxy |
| Backend | 3000 | backend | Rust + Axum |
| Database | 5432 | db | PostgreSQL (OrioleDB) |
| Cache | 6379 | cache | DragonflyDB (Redis) |
| Storage | 9000/9001 | minio | MinIO (S3) |
| Auth | 8080 | zitadel | ZITADEL (OIDC) |

## Befehle

| Befehl | Beschreibung |
|--------|--------------|
| `just dev` | Startet alles mit sichtbaren Logs |
| `just status` | Zeigt Service-Status |
| `just check` | Health Check aller Services |
| `just stop` | Stoppt alle Container |
| `just logs [service]` | Container-Logs (alle oder spezifischer Service) |
| `just shell [service]` | Shell in Container (backend, console, platform, docs) |
| `just restart` | Schneller Neustart aller Dev-Services |

## Hot-Reloading

### Console
- **Tool**: Vite mit HMR
- **Command**: `npm run dev -- --host 0.0.0.0`
- **Mount**: `/workspace/console → /app`
- **Speed**: <100ms

### Backend
- **Tool**: cargo-watch
- **Command**: `cargo watch -w src -w Cargo.toml -w config -w ../backend/proto -x run`
- **Mount**: `/workspace/backend → /app`, `/workspace/backend/proto → /workspace/backend/proto`
- **Speed**: 5-15 Sekunden

## Troubleshooting

### Container starten nicht
```bash
just reset
just dev
```

### Port belegt
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

### Container Shell
```bash
just shell backend
cargo check  # Debug Kompilierung
```

## Dateien

| Datei | Zweck |
|-------|-------|
| `infra/docker/docker-compose.yml` | Service-Definitionen |
| `infra/docker/Dockerfile.backend` | Backend Container |
| `infra/docker/Dockerfile.console` | Console Container |
| `infra/scripts/setup-*.sh` | Init-Skripte |
