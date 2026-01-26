# Port-Weiterleitungen DevContainer

## Übersicht aller freigegeben Ports

### Development Services
| Port | Service | Beschreibung | Auto-Forward |
|------|---------|-------------|--------------|
| **3000** | Backend API | Rust Axum Server | Notify |
| **5173** | Control (Vite) | SvelteKit Dev Server | Browser öffnen |

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
docker compose -f /workspace/infra/docker-compose.yml up -d
```

### Mit ZITADEL Auth starten
```bash
docker compose -f /workspace/infra/docker-compose.yml --profile auth up -d
```

### Services stoppen
```bash
docker compose -f /workspace/infra/docker-compose.yml down
```

### Logs anschauen
```bash
logs  # Alias aus .devcontainer/setup.sh
# oder direkt:
docker compose -f /workspace/infra/docker-compose.yml logs -f
```

## Verbindungsdetails

### Datenbank
- **Host:** localhost:5432
- **User:** godstack
- **Password:** godstack
- **Database:** godstack
- **Connection String:** `postgres://godstack:godstack@localhost:5432/godstack`

### Cache (Redis-kompatibel)
- **Host:** localhost:6379
- **Connection String:** `redis://localhost:6379`

### MinIO S3
- **API:** http://localhost:9000
- **Console:** http://localhost:9001
- **Access Key:** godstack
- **Secret Key:** godstack123

### ZITADEL (nur mit auth-profile)
- **URL:** http://localhost:8080
- **Issuer:** http://localhost:8080
