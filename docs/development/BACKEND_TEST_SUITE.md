# 🧪 Backend Test-Suite

## Übersicht

Umfassende Test-Suite für die neue feature-basierte API-Struktur.

## Test-Kategorien

### ✅ Health Check Tests (v1/health)
- `health_check_works()` - Verifiziert `/api/v1/health` Endpoint
- `readiness_check_works()` - Verifiziert `/api/v1/ready` Endpoint

### ✅ Info Tests (v1/info)
- `info_endpoint_works()` - Verifiziert `/api/v1/info` Endpoint
- `status_endpoint_works()` - Verifiziert `/api/v1/status` Endpoint

### ✅ User Tests (v1/users) - Protected Routes
- `users_endpoint_requires_auth()` - Verifiziert Auth-Anforderung
- `me_endpoint_requires_auth()` - Verifiziert Auth-Anforderung

### ✅ Storage Tests (v1/storage) - Protected Routes
- `storage_list_requires_auth()` - Verifiziert Auth-Anforderung
- `storage_upload_requires_auth()` - Verifiziert Auth-Anforderung
- `storage_buckets_requires_auth()` - Verifiziert Auth-Anforderung

### ✅ Route Structure Tests
- `all_public_routes_accessible()` - Alle public routes erreichbar
- `all_protected_routes_require_auth()` - Alle protected routes benötigen Auth
- `non_existent_routes_return_404()` - 404 für nicht-existierende Routes

### ✅ CORS Tests
- `cors_headers_present()` - CORS-Header vorhanden

## Test-Infrastruktur

### TestApp Helper
```rust
pub struct TestApp {
    pub address: String,
    pub client: reqwest::Client,
}
```

**Methoden:**
- `spawn()` - Startet Test-Server auf zufälligem Port
- `get(path)` - GET Request
- `post(path, body)` - POST Request
- `delete(path)` - DELETE Request

## Ausführung

```bash
# Alle Tests ausführen
cargo test

# Nur Integration Tests
cargo test --test api

# Mit Output
cargo test -- --nocapture

# Spezifischer Test
cargo test health_check_works
```

## Erwartete Ergebnisse

### ✅ Public Routes (sollten funktionieren)
- `/api/v1/health` → 200 OK
- `/api/v1/ready` → 200 OK oder 503 (wenn Services nicht laufen)
- `/api/v1/info` → 200 OK
- `/api/v1/status` → 200 OK

### 🔒 Protected Routes (sollten 401/403 zurückgeben)
- `/api/v1/users` → 401 Unauthorized
- `/api/v1/users/:id` → 401 Unauthorized
- `/api/v1/me` → 401 Unauthorized
- `/api/v1/storage/list` → 401 Unauthorized
- `/api/v1/storage/upload` → 401 Unauthorized
- `/api/v1/storage/buckets` → 401 Unauthorized

### ❌ Non-existent Routes
- `/api/v1/nonexistent` → 404 Not Found

## Test-Coverage

| Feature | Tests | Status |
|---------|-------|--------|
| Health | 2 | ✅ |
| Info | 2 | ✅ |
| Users | 2 | ✅ |
| Storage | 3 | ✅ |
| Route Structure | 3 | ✅ |
| CORS | 1 | ✅ |
| **Total** | **13** | ✅ |

## Nächste Schritte

### Erweiterte Tests (zukünftig)
- [ ] Authentifizierte Requests mit JWT Token
- [ ] User CRUD Operationen
- [ ] Storage Upload/Download Tests
- [ ] Error Handling Tests
- [ ] Performance Tests
- [ ] Load Tests

### Mock-Services
- [ ] Mock ZITADEL für Auth-Tests
- [ ] Mock MinIO für Storage-Tests
- [ ] Mock Database für DB-Tests

## Hinweise

- Tests benötigen keine laufenden Services (außer für Integration Tests)
- Test-Server startet auf Port 0 (zufälliger Port)
- Tests sind isoliert und können parallel ausgeführt werden
- CORS-Tests prüfen nur ob Headers vorhanden sind (nicht vollständige CORS-Validierung)
