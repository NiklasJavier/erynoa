# 🔧 Service-Konfiguration

**Letzte Aktualisierung**: 2026-01-27

## Zentrale Service-Definitionen

Dieses Dokument definiert die zentralen Service-Namen, Ports und URLs für das gesamte Projekt.

---

## 📡 Services

### Console (via Caddy Proxy)
- **Service Name:** `console`
- **Proxy Port:** `3001`
- **URL:** `http://localhost:3001/console` (via Caddy Proxy)
- **Direct Port:** `5173` (nur intern im Container)
- **Container Name:** `erynoa-services-console-1`

### Platform (via Caddy Proxy)
- **Service Name:** `platform`
- **Proxy Port:** `3001`
- **URL:** `http://localhost:3001/platform` (via Caddy Proxy)
- **Direct Port:** `5174` (nur intern im Container)
- **Container Name:** `erynoa-services-platform-1`

### Docs (via Caddy Proxy)
- **Service Name:** `docs`
- **Proxy Port:** `3001`
- **URL:** `http://localhost:3001/docs` (via Caddy Proxy)
- **Direct Port:** `5175` (nur intern im Container)
- **Container Name:** `erynoa-services-docs-1`

### Caddy Reverse Proxy
- **Service Name:** `caddy`
- **Port:** `3001`
- **URL:** `http://localhost:3001` (routet zu Console/Platform/Docs)
- **Container Name:** `erynoa-services-caddy-1`

### Backend API
- **Service Name:** `backend`
- **Port:** `3000`
- **URL:** `http://localhost:3000`
- **Container Name:** `erynoa-services-backend-1`

### Database (PostgreSQL/OrioleDB)
- **Service Name:** `db`
- **Port:** `5432`
- **URL:** `postgresql://localhost:5432`
- **Docker Internal:** `postgresql://db:5432`
- **Container Name:** `erynoa-services-db-1`

### Cache (DragonflyDB/Redis)
- **Service Name:** `cache`
- **Port:** `6379`
- **URL:** `redis://localhost:6379`
- **Docker Internal:** `redis://cache:6379`
- **Container Name:** `erynoa-services-cache-1`

### Storage (MinIO S3)
- **Service Name:** `minio`
- **API Port:** `9000`
- **Console Port:** `9001`
- **API URL:** `http://localhost:9000`
- **Console URL:** `http://localhost:9001`
- **Docker Internal:** `http://minio:9000`
- **Container Name:** `erynoa-services-minio-1`

### Authentication (ZITADEL)
- **Service Name:** `zitadel`
- **Port:** `8080`
- **URL:** `http://localhost:8080`
- **Docker Internal:** `http://zitadel:8080`
- **Container Name:** `erynoa-services-zitadel-1`

### ZITADEL Database
- **Service Name:** `zitadel-db`
- **Port:** `5433` (external, mapped from 5432)
- **Docker Internal:** `postgresql://zitadel-db:5432`
- **Container Name:** `erynoa-services-zitadel-db-1`

---

## 🔗 Connection Strings

### Development (Local)
```bash
# Database
postgresql://erynoa:erynoa@localhost:5432/erynoa

# Cache
redis://localhost:6379

# Storage
http://localhost:9000

# Auth
http://localhost:8080
```

### Docker (Internal)
```bash
# Database
postgresql://erynoa:erynoa@db:5432/erynoa

# Cache
redis://cache:6379

# Storage
http://minio:9000

# Auth
http://zitadel:8080
```

---

## 📝 Verwendung

Diese Konfiguration sollte als Referenz verwendet werden für:
- Docker Compose Service-Definitionen
- Backend-Konfiguration (`base.toml`)
- Console-Konfiguration
- Dokumentation
- Scripts

---

## ⚠️ Wichtig

- **Service-Namen** müssen in `docker-compose.yml` und Backend-Config übereinstimmen
- **Ports** müssen in `docker-compose.yml` und Dokumentation übereinstimmen
- **URLs** müssen in Console- und Backend-Config übereinstimmen
