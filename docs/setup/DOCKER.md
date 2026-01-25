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
│  │  frontend (5173)  backend (3000)  db (5432)        │ │
│  │  minio (9000/9001)  cache (6379)  zitadel (8080)   │ │
│  └────────────────────────────────────────────────────┘ │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

## Features

- ✅ **Frontend Hot-Reload**: Vite HMR (<100ms Updates)
- ✅ **Backend Hot-Reload**: cargo-watch (5-15s Rebuild)
- ✅ **Isolierte Services**: Alle Abhängigkeiten containerisiert
- ✅ **Volume Mounts**: Code-Änderungen sofort sichtbar
- ✅ **Health Checks**: Automatische Abhängigkeitsprüfung

## Quick Start

```bash
just dev
```

Siehe [DEV_SETUP.md](DEV_SETUP.md) für ausführliche Dokumentation.

## Services

| Service | Port | Container Name | Beschreibung |
|---------|------|----------------|--------------|
| Frontend | 5173 | frontend | SolidJS + Vite |
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
| `just docker-stop` | Stoppt alle Container |
| `just docker-logs` | Alle Container-Logs |
| `just docker-backend-shell` | Shell im Backend-Container |

## Hot-Reloading

### Frontend
- **Tool**: Vite mit HMR
- **Command**: `npm run dev -- --host 0.0.0.0`
- **Mount**: `/workspace/frontend → /app`
- **Speed**: <100ms

### Backend
- **Tool**: cargo-watch
- **Command**: `cargo watch -w src -w Cargo.toml -w config -w ../proto -x run`
- **Mount**: `/workspace/backend → /app`, `/workspace/proto → /proto`
- **Speed**: 5-15 Sekunden

## Troubleshooting

### Container starten nicht
```bash
just reset
just dev
```

### Port belegt
```bash
just docker-stop
lsof -i :3000  # Check welcher Prozess
```

### Logs prüfen
```bash
just docker-logs          # Alle
just docker-logs-backend  # Nur Backend
just docker-logs-frontend # Nur Frontend
```

### Container Shell
```bash
just docker-backend-shell
cargo check  # Debug Kompilierung
```

## Dateien

| Datei | Zweck |
|-------|-------|
| `infra/docker-compose.yml` | Service-Definitionen |
| `infra/Dockerfile.backend` | Backend Container |
| `infra/Dockerfile.frontend` | Frontend Container |
| `infra/scripts/setup-*.sh` | Init-Skripte |
