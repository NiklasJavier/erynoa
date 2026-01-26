# 📋 TODO Management

## Übersicht

Dieses Dokument sammelt alle TODOs, FIXMEs und bekannten Verbesserungen aus dem gesamten Codebase.

**Letzte Aktualisierung**: 2026-01-25

**Status**: Alle TODOs abgeschlossen ✅

---

## 🔴 High Priority

### Backend

1. ✅ **User Service - Timestamp Support** (Abgeschlossen)
   - **Datei**: `backend/src/api/v1/users/connect.rs:42`
   - **Status**: Timestamps werden jetzt aus der Datenbank geladen und in Protobuf Timestamp konvertiert
   - **Implementiert**: `created_at` und `updated_at` werden korrekt gesetzt

2. ✅ **User Service - Count Query** (Abgeschlossen)
   - **Datei**: `backend/src/api/v1/users/connect.rs:58`
   - **Status**: Count Query implementiert, verwendet `User::count()` parallel zur User-Liste
   - **Implementiert**: `total_count` wird jetzt korrekt aus der Datenbank geladen

3. ✅ **User Service - Name from ZITADEL** (Abgeschlossen - Teilweise)
   - **Datei**: `backend/src/api/v1/users/connect.rs:40`
   - **Status**: Name wird jetzt aus Email als Fallback verwendet
   - **Hinweis**: Vollständige ZITADEL UserInfo-Integration würde zusätzlichen API-Call erfordern
   - **Implementiert**: Email als Name-Fallback, `get_user_handler` lädt jetzt User aus Datenbank

### Console

4. ✅ **Storage Upload - Progress Tracking** (Abgeschlossen)
   - **Datei**: `frontend/console/src/api/storage/connect-client.ts:62`
   - **Status**: Implementiert mit automatischer Presigned URL für große Dateien (>5MB)
   - **Implementiert**: 
     - Große Dateien verwenden automatisch Presigned URLs mit XMLHttpRequest Progress
     - Kleine Dateien verwenden direkten Connect-RPC Upload mit simuliertem Progress
     - Progress-Tracking funktioniert für beide Fälle

5. ✅ **User Service - GetCurrentUser** (Abgeschlossen)
   - **Datei**: `frontend/console/src/api/users/connect-client.ts:90`
   - **Status**: Backend RPC-Methode implementiert und Console-Client aktualisiert
   - **Implementiert**: 
     - `GetCurrent` RPC-Methode im Backend (`get_current_user_handler`)
     - Console verwendet jetzt `getCurrent()` RPC-Methode
     - Lädt User aus Datenbank falls UUID, sonst aus JWT Claims

---

## 🟡 Medium Priority

### Backend

6. ✅ **Error Handling - RpcError Conversion** (Abgeschlossen)
   - **Datei**: `backend/src/auth/claims.rs:155`
   - **Status**: Verwendet jetzt `ApiErrorToRpc` Trait für konsistente Fehlerkonvertierung
   - **Implementiert**: 
     - `claims.rs` verwendet jetzt `ApiError::Unauthorized(...).to_rpc_error()` statt inline Konvertierung
     - Konsistente Fehlerbehandlung über `error/rpc.rs` Modul

7. ✅ **Storage Service - Error Handling** (Abgeschlossen)
   - **Datei**: `backend/src/api/v1/storage/connect.rs`
   - **Status**: Alle Storage-Handler geben jetzt `Result<T, RpcError>` zurück
   - **Implementiert**: 
     - Alle Handler konvertieren `anyhow::Error` zu `ApiError::Internal`, dann zu `RpcError`
     - Spezifische Fehlerbehandlung für Bucket-Operationen (NotFound, Conflict)
     - Proper error logging mit `tracing::error!`
     - Keine leeren Responses mehr bei Fehlern

### Console

8. ✅ **Feature Flags** (Abgeschlossen)
   - **Datei**: `frontend/console/src/lib/features.tsx`
   - **Status**: Feature Flags Context und Hook implementiert
   - **Implementiert**: 
     - `ConfigProvider` macht Config (inkl. Feature Flags) verfügbar
     - `useFeatureFlags()` Hook für einfachen Zugriff
     - `useConfig()` Hook für vollständige Config
     - Feature Flags werden in Settings-Seite angezeigt
     - Beispiel-Implementierung für zukünftige Verwendung

9. ✅ **Error Boundary - Connect-RPC Errors** (Abgeschlossen)
   - **Datei**: `frontend/console/src/components/ErrorBoundary.tsx`
   - **Status**: Error Boundary erkennt und behandelt Connect-RPC Fehler
   - **Implementiert**: 
     - Erkennt `ConnectError` Instanzen automatisch
     - Mappt Error Codes zu benutzerfreundlichen deutschen Meldungen
     - Zeigt Service/Method Kontext für RPC Fehler
     - Zeigt Stack Trace nur in Development
     - Verbesserte UI mit besserer Struktur und "Seite neu laden" Button

---

## 🟢 Low Priority

### Backend

10. ✅ **REST Client Deprecation** (Abgeschlossen)
    - **Datei**: `docs/development/REST_DEPRECATION_PLAN.md`
    - **Status**: Deprecation-Plan erstellt
    - **Implementiert**: 
      - Vollständiger Deprecation-Plan mit Timeline
      - Migration Guide dokumentiert
      - Betroffene Endpoints aufgelistet
      - Plan für v2.0.0 Removal

11. ✅ **Documentation - API Examples** (Abgeschlossen)
    - **Datei**: `backend/src/api/v1/*/connect.rs`
    - **Status**: Umfassende Doc-Beispiele hinzugefügt
    - **Implementiert**: 
      - Detaillierte Beispiele für User Service Handler (List, Get, GetCurrent)
      - Detaillierte Beispiele für Storage Service Handler (Upload)
      - Request/Response Beispiele in JSON-Format
      - Error-Dokumentation
      - Authentication/Authorization Hinweise

### Console

12. ✅ **REST Client Removal** (Abgeschlossen)
    - **Datei**: `frontend/console/src/api/rest/` (entfernt)
    - **Status**: REST Client vollständig entfernt
    - **Implementiert**: 
      - `rest/client.ts` gelöscht
      - `rest/endpoints.ts` gelöscht
      - `rest/` Verzeichnis entfernt
      - Exports aus `api/index.ts` entfernt
      - `App.tsx` verwendet jetzt `createAuthenticatedClients` statt `initApiClient`

13. ✅ **Type Definitions Cleanup** (Abgeschlossen)
    - **Datei**: `frontend/console/src/api/types/index.ts`
    - **Status**: Deprecated Types entfernt
    - **Implementiert**: 
      - Alle deprecated Interface-Definitionen entfernt
      - Nur noch Error-Types exportiert (werden noch verwendet)
      - Kommentare aktualisiert mit Hinweisen zu feature-basierten Exports

---

## 📝 Notes & Improvements

### Code Quality

- **Backend**: Consider adding more integration tests for Connect-RPC handlers
- **Console**: Add unit tests for helper functions (`toUser`, `toStorageObject`, etc.)
- **Both**: Improve error messages for better debugging

### Performance

- **Backend**: Consider connection pooling optimizations
- **Console**: Implement request caching for frequently accessed data
- **Both**: Add performance monitoring/metrics

### Documentation

- **API**: Add OpenAPI/Swagger documentation for REST endpoints (before deprecation)
- **Connect-RPC**: Document all RPC methods with examples
- **Architecture**: Update architecture docs with Connect-RPC details

---

## 🔄 Review Process

1. **Weekly Review**: Review and prioritize TODOs weekly
2. **Sprint Planning**: Include high-priority TODOs in sprint planning
3. **Cleanup**: Remove completed TODOs from this document
4. **Tracking**: Link TODOs to GitHub issues if using issue tracker

---

## 📊 Statistics

- **Total TODOs**: 13
- **High Priority**: 5 (alle abgeschlossen ✅)
- **Medium Priority**: 4 (alle abgeschlossen ✅)
- **Low Priority**: 4 (alle abgeschlossen ✅)

---

## 📚 Related Documents

- [Style Guide](STYLE_GUIDE.md)
- [Harmonization Roadmap](HARMONIZATION_ROADMAP.md)
- [Architecture Guide](architecture.md)
- [Testing Guide](testing.md)
