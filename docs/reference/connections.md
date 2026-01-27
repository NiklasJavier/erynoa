# 🔗 Verbindungen & Harmonisierung

## Übersicht

Dokumentation der harmonisierten Verbindungen zwischen Console, Backend und Services.

---

## 🌐 API-Verbindungen

### Zentrale API-Konfiguration

**Console:** `frontend/console/src/lib/api-config.ts`
- Single source of truth für API-URLs
- Harmonisiert mit Backend-Konfiguration
- Unterstützt Environment-Variablen

**Backend:** `backend/config/base.toml`
- Zentrale Konfiguration für alle Services
- Environment-spezifische Overrides möglich

### API-URL Harmonisierung

**Console:**
```typescript
import { getApiBaseUrl, getApiUrl } from "@/lib/api-config";

const baseUrl = getApiBaseUrl(); // http://localhost:3000
const fullUrl = getApiUrl();     // http://localhost:3000/api/v1
```

**Backend:**
```toml
[application]
api_url = "http://localhost:3000"
console_url = "http://localhost:3001/console"
```

**Environment Variables:**
- `VITE_API_URL` (Console)
- `APP_APPLICATION__API_URL` (Backend)

---

## 🔴 Error-Handling Harmonisierung

### Backend Error Format

```rust
// backend/src/error.rs
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

pub struct ErrorDetails {
    pub code: &'static str,      // z.B. "UNAUTHORIZED"
    pub message: String,
    pub details: Option<Value>,
}
```

### Console Error Format

```typescript
// frontend/console/src/api/types/errors.ts
export interface ApiErrorResponse {
  error: ErrorDetails;
}

export interface ErrorDetails {
  code: ErrorCode;  // Harmonisiert mit Backend
  message: string;
  details?: unknown;
}
```

### Error Codes (Harmonisiert)

| Code | HTTP Status | Beschreibung |
|------|-------------|--------------|
| `UNAUTHORIZED` | 401 | Nicht authentifiziert |
| `FORBIDDEN` | 403 | Keine Berechtigung |
| `INVALID_TOKEN` | 401 | Ungültiges Token |
| `VALIDATION_ERROR` | 400 | Validierungsfehler |
| `BAD_REQUEST` | 400 | Ungültige Anfrage |
| `NOT_FOUND` | 404 | Ressource nicht gefunden |
| `CONFLICT` | 409 | Konflikt (z.B. Duplikat) |
| `DATABASE_ERROR` | 500 | Datenbankfehler |
| `CACHE_ERROR` | 500 | Cache-Fehler |
| `INTERNAL_ERROR` | 500 | Interner Fehler |
| `SERVICE_UNAVAILABLE` | 503 | Service nicht verfügbar |

---

## 🔌 Service-Verbindungen

### Database (PostgreSQL)

**Backend Konfiguration:**
```toml
[database]
host = "localhost"      # "db" im Docker
port = 5432
username = "erynoa"
password = "erynoa"
database = "erynoa"
```

**Connection String:**
```rust
postgres://erynoa:erynoa@localhost:5432/erynoa
```

### Cache (DragonflyDB/Redis)

**Backend Konfiguration:**
```toml
[cache]
url = "redis://localhost:6379"  # "redis://cache:6379" im Docker
pool_size = 10
default_ttl = 3600
```

### Storage (MinIO/S3)

**Backend Konfiguration:**
```toml
[storage]
endpoint = "http://localhost:9000"  # "http://minio:9000" im Docker
region = "us-east-1"
access_key_id = "erynoa"
secret_access_key = "erynoa123"
default_bucket = "erynoa"
```

### Authentication (ZITADEL)

**Backend Konfiguration:**
```toml
[auth]
issuer = "http://localhost:8080"      # Externe URL
internal_issuer = "http://zitadel:8080"  # Interne URL (Docker)
client_id = "erynoa-backend"
console_client_id = "erynoa-console"
```

**Console Konfiguration:**
```typescript
auth: {
  issuer: "http://localhost:8080",
  clientId: "erynoa-console",
}
```

---

## 📡 API-Clients Harmonisierung

### REST Client

**Console:** `frontend/console/src/api/rest/client.ts`
- Verwendet zentrale API-Konfiguration
- Harmonisiertes Error-Handling
- Automatische Token-Injection

### Connect-RPC Client

**Console:** `frontend/console/src/api/connect/transport.ts`
- Verwendet zentrale API-Konfiguration
- Binary Protobuf für Performance
- Automatische Token-Injection

### Storage Client

**Console:** `frontend/console/src/api/storage/client.ts`
- S3-kompatibler Client
- Harmonisiert mit Backend Storage

---

## 🔄 Response-Formate

### Erfolgreiche Responses

**Standard Response:**
```json
{
  "data": { ... }
}
```

**List Response:**
```json
{
  "items": [...],
  "count": 10,
  "page": 1,
  "page_size": 20
}
```

### Error Responses (Harmonisiert)

```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Unauthorized: Missing authentication",
    "details": null
  }
}
```

---

## 🎯 Best Practices

### 1. API-URLs
- ✅ Verwende `getApiBaseUrl()` aus `api-config.ts`
- ✅ Keine hardcodierten URLs
- ✅ Environment-Variablen für Konfiguration

### 2. Error-Handling
- ✅ Verwende `ApiErrorResponse` Format
- ✅ Prüfe `error.code` für spezifische Fehlerbehandlung
- ✅ Nutze `isErrorCode()` Helper

### 3. Service-Verbindungen
- ✅ Zentrale Konfiguration in `config/base.toml`
- ✅ Environment-spezifische Overrides
- ✅ Docker Service-Namen für interne Kommunikation

### 4. Typen-Synchronisation
- ✅ Console-Typen entsprechen Backend-Strukturen
- ✅ Shared Types in `frontend/console/src/api/types/`
- ✅ Konsistente Namenskonventionen

---

## 📚 Weitere Informationen

- [Architecture](architecture.md) - System-Architektur
- [API Design](api-design.md) - API-Struktur
- [Testing](testing.md) - Test-Guide
