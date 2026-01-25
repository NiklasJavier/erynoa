# 🔍 Harmonisierung & Redundanz-Analyse

## Übersicht

Analyse der Redundanzen und Harmonisierungspotenziale im Projekt.

---

## 🔴 Gefundene Redundanzen

### 1. Fallback-Konfiguration in `frontend/src/lib/config.ts`

**Problem:**
- Hardcoded Fallback-Werte (localhost:8080, localhost:3000, etc.)
- Diese Werte sind bereits in `backend/config/base.toml` definiert
- Inkonsistenz-Risiko bei Änderungen

**Aktuell:**
```typescript
// frontend/src/lib/config.ts
return {
  environment: "local",
  version: "0.1.0",
  auth: {
    issuer: "http://localhost:8080",  // ❌ Redundant
    clientId: "godstack-frontend",    // ❌ Redundant
  },
  urls: {
    frontend: "http://localhost:5173", // ❌ Redundant
    api: "http://localhost:3000",      // ❌ Redundant
  },
  // ...
};
```

**Lösung:**
- Fallback-Werte aus zentraler Konfiguration laden
- Oder: Fallback komplett entfernen (Backend sollte immer erreichbar sein)

---

### 2. API Version String mehrfach hardcodiert

**Problem:**
- `/api/v1` ist an mehreren Stellen hardcodiert
- Änderung der API-Version erfordert mehrere Änderungen

**Gefunden in:**
- `frontend/src/lib/api-config.ts`: `getApiVersion()` → `/api/v1`
- `frontend/src/api/rest/endpoints.ts`: `/api/v1/health`, `/api/v1/users`, etc.
- `backend/src/api/routes.rs`: `/api/v1` Route-Prefix
- `scripts/dev-check.sh`: `/api/v1/health`, `/api/v1/info`

**Lösung:**
- Zentrale Konstante für API-Version
- Frontend: Export aus `api-config.ts`
- Backend: Konstante in `routes.rs` oder Config

---

### 3. Legacy ApiError Interface

**Problem:**
- `frontend/src/api/types/index.ts` definiert noch `ApiError` (Legacy)
- `frontend/src/api/types/errors.ts` definiert `ApiErrorResponse` (Neu)
- Beide existieren parallel

**Aktuell:**
```typescript
// frontend/src/api/types/index.ts
export interface ApiError {  // ❌ Legacy
  status: number;
  message: string;
  code?: string;
}

// frontend/src/api/types/errors.ts
export interface ApiErrorResponse {  // ✅ Neu
  error: ErrorDetails;
}
```

**Lösung:**
- Legacy `ApiError` entfernen
- Alle Verwendungen auf `ApiErrorResponse` migrieren
- Oder: `ApiError` als Type-Alias für Kompatibilität

---

### 4. Docker Compose Environment Variables

**Problem:**
- Viele Werte sind sowohl in `base.toml` als auch in `docker-compose.yml` definiert
- Inkonsistenz-Risiko

**Beispiel:**
```yaml
# docker-compose.yml
- APP_APPLICATION__FRONTEND_URL=http://localhost:5173
- APP_APPLICATION__API_URL=http://localhost:3000
- APP_DATABASE__USERNAME=godstack
- APP_DATABASE__PASSWORD=godstack
```

```toml
# base.toml
[application]
frontend_url = "http://localhost:5173"
api_url = "http://localhost:3000"

[database]
username = "godstack"
password = "godstack"
```

**Lösung:**
- Docker Compose sollte nur Docker-spezifische Overrides enthalten (z.B. `host=db` statt `host=localhost`)
- Standard-Werte aus `base.toml` verwenden

---

### 5. Service-Namen und Ports mehrfach definiert

**Problem:**
- Service-Namen (`db`, `cache`, `zitadel`, `minio`) an mehreren Stellen
- Ports (3000, 5173, 8080, 9000) mehrfach hardcodiert

**Gefunden in:**
- `docker-compose.yml`: Service-Definitionen
- `base.toml`: Connection-Strings
- `scripts/dev-check.sh`: Health-Check-URLs
- `justfile`: Service-Namen
- Dokumentation: Mehrere Stellen

**Lösung:**
- Zentrale Service-Konfiguration
- Oder: Dokumentation mit Referenzen

---

### 6. Version-Informationen mehrfach definiert

**Problem:**
- Version in `Cargo.toml` (Backend)
- Version in `package.json` (Frontend)
- Version in Fallback-Config (`0.1.0`)
- Version in Dokumentation

**Lösung:**
- Single source of truth für Version
- Backend: `CARGO_PKG_VERSION`
- Frontend: Aus Backend `/api/v1/info` laden

---

## ✅ Bereits Harmonisiert

### 1. Error-Handling ✅
- Frontend `ApiErrorResponse` harmonisiert mit Backend `ErrorResponse`
- Error-Codes konsistent

### 2. API-URL Konfiguration ✅
- `frontend/src/lib/api-config.ts` als Single source of truth
- Alle Clients verwenden `getApiBaseUrl()`

### 3. Service-Verbindungen ✅
- Zentrale Konfiguration in `base.toml`
- Connection-String-Helper in Rust

---

## 🎯 Empfohlene Verbesserungen

### Priorität 1 (Hoch)
1. **API Version String zentralisieren**
   - Impact: Hoch (viele Stellen betroffen)
   - Aufwand: Niedrig
   - Nutzen: Einfache API-Version-Änderung

2. **Legacy ApiError entfernen**
   - Impact: Mittel (nur Frontend)
   - Aufwand: Niedrig
   - Nutzen: Klarere Code-Struktur

### Priorität 2 (Mittel)
3. **Fallback-Konfiguration optimieren**
   - Impact: Niedrig (nur bei Backend-Ausfall)
   - Aufwand: Mittel
   - Nutzen: Weniger Redundanz

4. **Docker Compose Environment Variables reduzieren**
   - Impact: Niedrig (nur Docker-Setup)
   - Aufwand: Mittel
   - Nutzen: Weniger Redundanz, einfachere Wartung

### Priorität 3 (Niedrig)
5. **Service-Namen zentralisieren**
   - Impact: Sehr niedrig
   - Aufwand: Hoch
   - Nutzen: Minimal (nur Dokumentation)

6. **Version-Informationen zentralisieren**
   - Impact: Sehr niedrig
   - Aufwand: Niedrig
   - Nutzen: Minimal

---

## 📋 Nächste Schritte

1. ✅ Analyse abgeschlossen
2. ⏳ API Version String zentralisieren
3. ⏳ Legacy ApiError entfernen
4. ⏳ Fallback-Konfiguration optimieren
5. ⏳ Docker Compose Environment Variables reduzieren

---

## 📚 Referenzen

- [Connections Harmonized](./CONNECTIONS_HARMONIZED.md)
- [API Restructure Complete](../changelog/restructure-2024.md)
- [Frontend API Restructure](../changelog/FRONTEND_API_RESTRUCTURE_COMPLETE.md)
