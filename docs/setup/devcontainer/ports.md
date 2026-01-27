# Port-Weiterleitungen DevContainer

## Übersicht aller freigegeben Ports

### Development Services
| Port | Service | Beschreibung | Auto-Forward |
|------|---------|-------------|--------------|
| **3000** | Backend API | Rust Axum Server | Notify |
| **3001** | Caddy Proxy | Reverse Proxy für alle Frontends | Browser öffnen |
| **5173** | Console Frontend | SvelteKit Dev Server (Console) | Ignorieren |
| **5174** | Platform Frontend | SvelteKit Dev Server (Platform) | Ignorieren |
| **5175** | Docs Frontend | SvelteKit Dev Server (Docs) | Ignorieren |

### Datenbanken & Cache
| Port | Service | Beschreibung | Auto-Forward |
|------|---------|-------------|--------------|
| **5432** | PostgreSQL (App) | OrioleDB für Anwendung | Ignorieren |
| **5433** | PostgreSQL (ZITADEL) | Separate DB für Auth (nur mit `--profile auth`) | Ignorieren |
| **6379** | Redis/DragonflyDB | Cache & Session Storage | Ignorieren |

### S3 Object Storage
| Port | Service | Beschreibung | Auto-Forward |
|------|---------|-------------|--------------|
| **9000** | MinIO S3 API | S3-kompatible Object Storage API | Ignorieren |
| **9001** | MinIO Console | Web UI für S3 Management | Notify |

### Authentication
| Port | Service | Beschreibung | Auto-Forward |
|------|---------|-------------|--------------|
| **8080** | ZITADEL | Identity Provider (nur mit `--profile auth`) | Notify |

## Docker Compose Befehle

### Basis-Services starten (DB, Cache, MinIO)
```bash
docker compose -f /workspace/infra/docker/docker-compose.yml up -d
```

### Mit ZITADEL Auth starten
```bash
docker compose -f /workspace/infra/docker/docker-compose.yml --profile auth up -d
```

### Services stoppen
```bash
docker compose -f /workspace/infra/docker/docker-compose.yml down
```

### Logs anschauen
```bash
logs  # Alias aus .devcontainer/setup-and-init.sh
# oder direkt:
docker compose -f /workspace/infra/docker/docker-compose.yml logs -f
```

## Verbindungsdetails

### Datenbank
- **Host:** localhost:5432
- **User:** erynoa
- **Password:** erynoa
- **Database:** erynoa
- **Connection String:** `postgres://erynoa:erynoa@localhost:5432/erynoa`

### Cache (Redis-kompatibel)
- **Host:** localhost:6379
- **Connection String:** `redis://localhost:6379`

### MinIO S3
- **API:** http://localhost:9000
- **Console:** http://localhost:9001
- **Access Key:** erynoa
- **Secret Key:** erynoa123

### ZITADEL (nur mit auth-profile)
- **URL:** http://localhost:8080
- **Issuer:** http://localhost:8080

## VS Code Extensions für Datenbank & Cache

Die DevContainer-Konfiguration installiert automatisch Extensions für direkte Datenbank- und Cache-Verbindungen:

### PostgreSQL Extension
- **Extension**: `ms-ossdata.vscode-postgresql` (Microsoft)
- **Verwendung**: Datenbank-Explorer, SQL Editor, Schema-Visualisierung
- **Verbindungen**: Automatisch konfiguriert für beide Datenbanken (App & ZITADEL)

### Redis/Dragonfly Extension
- **Extension**: `Redis.redis-for-vscode`
- **Verwendung**: Key-Explorer, Key-Editor, TTL-Verwaltung
- **Verbindung**: Automatisch konfiguriert für Dragonfly Cache

**Details**: Siehe [database_connection.md](./database_connection.md) für vollständige Anleitung.
