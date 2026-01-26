# 🔧 Service-Konfiguration

## Zentrale Service-Definitionen

Dieses Dokument definiert die zentralen Service-Namen, Ports und URLs für das gesamte Projekt.

---

## 📡 Services

### Console
- **Service Name:** `console`
- **Port:** `5173`
- **URL:** `http://localhost:5173`
- **Container Name:** `godstack-services-console-1`

### Backend API
- **Service Name:** `backend`
- **Port:** `3000`
- **URL:** `http://localhost:3000`
- **Container Name:** `godstack-services-backend-1`

### Database (PostgreSQL/OrioleDB)
- **Service Name:** `db`
- **Port:** `5432`
- **URL:** `postgresql://localhost:5432`
- **Docker Internal:** `postgresql://db:5432`
- **Container Name:** `godstack-services-db-1`

### Cache (DragonflyDB/Redis)
- **Service Name:** `cache`
- **Port:** `6379`
- **URL:** `redis://localhost:6379`
- **Docker Internal:** `redis://cache:6379`
- **Container Name:** `godstack-services-cache-1`

### Storage (MinIO S3)
- **Service Name:** `minio`
- **API Port:** `9000`
- **Console Port:** `9001`
- **API URL:** `http://localhost:9000`
- **Console URL:** `http://localhost:9001`
- **Docker Internal:** `http://minio:9000`
- **Container Name:** `godstack-services-minio-1`

### Authentication (ZITADEL)
- **Service Name:** `zitadel`
- **Port:** `8080`
- **URL:** `http://localhost:8080`
- **Docker Internal:** `http://zitadel:8080`
- **Container Name:** `godstack-services-zitadel-1`

### ZITADEL Database
- **Service Name:** `zitadel-db`
- **Port:** `5433` (external, mapped from 5432)
- **Docker Internal:** `postgresql://zitadel-db:5432`
- **Container Name:** `godstack-services-zitadel-db-1`

---

## 🔗 Connection Strings

### Development (Local)
```bash
# Database
postgresql://godstack:godstack@localhost:5432/godstack

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
postgresql://godstack:godstack@db:5432/godstack

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
