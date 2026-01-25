# 🐳 Docker Development Setup

Dieses Setup ermöglicht es, Frontend und Backend in Docker-Containern mit **Hot-Reloading** auszuführen.

Die Docker-Konfiguration befindet sich im `infra/` Ordner (logisch zusammen mit anderen Infrastruktur-Komponenten).

## Features

- ✅ **Frontend Hot-Reload**: Änderungen in Solid.js werden sofort im Browser aktualisiert
- ✅ **Backend Hot-Reload**: Änderungen in Rust werden automatisch neu kompiliert und gestartet
- ✅ **Isolierte Services**: PostgreSQL (OrioleDB), DragonflyDB (Cache), MinIO (S3) laufen alle im Docker
- ✅ **Volume Mounts**: Code-Änderungen werden sofort in Container reflektiert
- ✅ **Netzwerk**: Alle Services sind automatisch verbunden

## Quick Start

### Mit `just` Kommandos:

```bash
# Starte alles mit Hot-Reloading
just docker-dev

# Logs anschauen
just docker-logs

# Backend-Shell (zum Debuggen)
just docker-backend-shell

# Frontend-Shell
just docker-frontend-shell

# Stoppt alles
just docker-stop
```

### Mit Docker-Compose direkt:

```bash
# Starte alles (muss aus infra/ aufgerufen werden)
cd infra
docker-compose up --build

# In separatem Terminal: Logs
docker-compose logs -f

# Shell in Backend
docker-compose exec backend sh

# Shell in Frontend
docker-compose exec frontend sh

# Stoppt alles
docker-compose down
```

## Ports

| Service | Port | URL |
|---------|------|-----|
| Frontend (Vite) | 5173 | http://localhost:5173 |
| Backend (API) | 3000 | http://localhost:3000 |
| PostgreSQL | 5432 | `postgres://godstack:godstack@localhost:5432/godstack` |
| DragonflyDB (Cache) | 6379 | `redis://localhost:6379` |
| MinIO (S3) | 9000/9001 | http://localhost:9001 (Admin) |
| ZITADEL (Auth) | 8080 | http://localhost:8080 (mit `--profile auth`) |

## Umgebungsvariablen

Die folgenden Umgebungsvariablen sind in `docker-compose.yml` definiert:

### Backend
- `RUST_LOG=debug`
- `DATABASE_URL=postgresql://godstack:godstack@postgres:5432/godstack`
- `REDIS_URL=redis://cache:6379`
- `S3_ENDPOINT=http://s3:9000`
- `AUTH_ISSUER=http://zitadel:8080`
- `FRONTEND_URL=http://localhost:5173`
- `API_URL=http://localhost:3000`

### Frontend
- `VITE_API_URL=http://localhost:3000`

## How Hot-Reloading Works

### Frontend (Vite)
- Vite läuft mit `npm run dev -- --host 0.0.0.0`
- Dateien im `/workspace/frontend` Volume werden überwacht
- Browser wird automatisch aktualisiert via WebSocket (HMR)
- Konfiguriert in `frontend/vite.config.ts`

### Backend (Cargo-Watch)
- Cargo-Watch läuft als Hauptprozess: `cargo watch -x run --bin godstack-api`
- Dateien im `/workspace/backend` Volume werden überwacht
- Server wird automatisch neu gestartet bei Code-Änderungen
- Schneller als kompletter Neu-Build dank Cargo-Caching

## Troubleshooting

### Port bereits belegt
```bash
# Nutze andere Ports in docker-compose.yml
# Oder kill die Prozesse:
lsof -ti :5173 | xargs kill -9  # Frontend
lsof -ti :3000 | xargs kill -9  # Backend
```

### Container bauen sich nicht neu
```bash
# Erzwinge Rebuild
docker-compose build --no-cache
docker-compose up --build
```

### Volume-Mount funktioniert nicht
```bash
# Überprüfe ob Volumes korrekt mounted sind
docker inspect <container-name> | grep -A 20 Mounts

# Oder nutze bind mount direkt:
docker-compose exec backend ls -la /workspace/backend/src
```

### Logs überprüfen
```bash
# Alle Logs
docker-compose logs -f

# Nur Backend
docker-compose logs -f backend

# Nur Frontend
docker-compose logs -f frontend
```

## Dateien

- `infra/docker-compose.yml` - Main Docker-Konfiguration mit allen Services
- `infra/Dockerfile.frontend` - Frontend Container (Node.js + Vite)
- `infra/Dockerfile.backend` - Backend Container (Rust + Cargo-Watch)
- `infra/start.sh` - Hilfsskript zum Starten aller Services
- `justfile` - Convenience Commands (just docker-dev, etc.)

## Weitere Info

- [Docker-Compose Dokumentation](https://docs.docker.com/compose/)
- [Vite HMR Dokumentation](https://vitejs.dev/config/server-options.html#server-hmr)
- [Cargo-Watch](https://github.com/watchexec/cargo-watch)
