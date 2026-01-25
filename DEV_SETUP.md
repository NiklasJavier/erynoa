# 🚀 Development Setup - Frontend + Backend + Services

## Architektur

```
┌─────────────────────────────────────────────────────────────────┐
│                    Development Environment                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Frontend (Docker)          Backend (Local)        Services       │
│  ─────────────────          ───────────────        ─────────     │
│  Port: 5173 🟢              Port: 3000 🟢          DB: 5432 🟢   │
│  Vite HMR enabled           cargo watch            Cache: 6379   │
│  Hot-reload on save         Hot-reload on save     MinIO: 9000   │
│                                                     ZITADEL: 8080 │
│  Stack: Solid.js            Stack: Rust/Axum                    │
│  Build: npm                 Build: cargo                         │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## 🚀 Quick Start

### Option 1: Full Development (Recommended)
```bash
just dev
```
Dies startet:
- Frontend in Docker (:5173)
- Backend lokal (:3000) mit `cargo watch`
- Services (DB, Cache, MinIO, ZITADEL)

### Option 2: Nur Services + Frontend
```bash
just docker-dev
```
Dann in separatem Terminal:
```bash
just dev-backend
```

### Option 3: Nur Backend lokal
```bash
just dev-backend
```

## 📁 Struktur

### Docker Services (`/workspace/infra/docker-compose.yml`)
```yaml
services:
  frontend:      # Solid.js + Vite (:5173)
  db:            # PostgreSQL OrioleDB (:5432)
  cache:         # DragonflyDB (:6379)
  minio:         # S3-compatible (:9000-9001)
  zitadel:       # Auth (:8080)
```

**Wichtig:** Backend ist **NICHT** in Docker - läuft lokal für schnellere Entwicklung!

### Backend lokal (`/workspace/backend/`)
```bash
cargo watch -x run -w src -w config
```
Automatisches Neukompilieren bei Dateiänderungen in `src/` oder `config/`

## 🔧 Environment Variables

Backend nutzt diese für lokale Entwicklung:
```env
DATABASE_URL=postgresql://godstack:godstack@localhost:5432/godstack
REDIS_URL=redis://localhost:6379
S3_ENDPOINT=http://localhost:9000
RUST_LOG=debug
FRONTEND_URL=http://localhost:5173
API_URL=http://localhost:3000
```

Diese sind in der Dockerfile und im `.env` konfiguriert.

## 🔥 Hot-Reload

### Frontend
- **Tool:** Vite HMR (Hot Module Replacement)
- **Trigger:** Jede Änderung in `/workspace/frontend/src/`
- **Auswirkung:** Browser aktualisiert Module ohne Reload
- **Sichtbar:** Browser zeigt "HMR ready" in DevTools

### Backend
- **Tool:** `cargo watch`
- **Trigger:** Jede Änderung in `/workspace/backend/src/`
- **Auswirkung:** Server wird neu kompiliert und gestartet
- **Sichtbar:** Terminal zeigt Build-Meldungen

## 📝 Beispiel - Code ändern

### Frontend ändern
1. Öffne `/workspace/frontend/src/pages/Home.tsx`
2. Ändere einen Text z.B. "Dashboard" → "Dashboard 2"
3. Speichern (Ctrl+S)
4. Browser aktualisiert automatisch ohne Reload ✨

### Backend ändern
1. Öffne `/workspace/backend/src/api/handlers/status.rs`
2. Ändere die Response z.B. Feldname oder Wert
3. Speichern (Ctrl+S)
4. Terminal zeigt: `[Running 'cargo run']`
5. Server wird neu gestartet, kein neuer Build nötig!

## 🐛 Troubleshooting

### Frontend lädt nicht
```bash
curl http://localhost:5173
# Sollte HTML zurückgeben
```

### Backend stellt keine Verbindung zur DB her
```bash
# Prüfe DB-Health:
curl http://localhost:5432  # oder
docker-compose -f infra/docker-compose.yml ps | grep db
```

### Backend kompiliert nicht
```bash
cd /workspace/backend
cargo check  # Schnellere Syntax-Prüfung
cargo build --verbose  # Mit Details
```

### Ports bereits in Verwendung
```bash
lsof -i :5173   # Welcher Prozess nutzt Port?
kill -9 <PID>   # Wenn nötig
```

## 📊 Status Commands

```bash
# Alle Docker-Services prüfen
docker-compose -f infra/docker-compose.yml ps

# Backend Log live ansehen
tail -f /tmp/backend.log

# Frontend ist unter http://localhost:5173 erreichbar
# Backend ist unter http://localhost:3000 erreichbar

# Status-Endpoint testen
curl http://localhost:3000/api/v1/status | jq .
```

## 🎯 Nächste Schritte

1. ✅ `just dev` ausführen
2. ✅ Frontend öffnen: http://localhost:5173
3. ✅ Eine Datei ändern → Hot-Reload testen
4. ✅ Backend-API testen: `curl http://localhost:3000/api/v1/status`

## 📚 Weitere Infos

- Frontend Code: `/workspace/frontend/`
- Backend Code: `/workspace/backend/`
- Docker Config: `/workspace/infra/docker-compose.yml`
- Just Commands: `/workspace/justfile`
