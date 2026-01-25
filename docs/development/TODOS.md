# 📋 TODO Management

## Übersicht

Dieses Dokument sammelt alle TODOs, FIXMEs und bekannten Verbesserungen aus dem gesamten Codebase.

**Letzte Aktualisierung**: 2026-01-25

**Status**: Phase 1 Abschluss und High Priority TODOs abgeschlossen ✅

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

### Frontend

4. **Storage Upload - Progress Tracking**
   - **Datei**: `frontend/src/api/storage/connect-client.ts:62`
   - **TODO**: Add progress tracking for Connect-RPC uploads
   - **Kontext**: Connect-RPC doesn't natively support upload progress. Consider using presigned URLs for large files
   - **Schätzung**: 4-6 Stunden

5. **User Service - GetCurrentUser**
   - **Datei**: `frontend/src/api/users/connect-client.ts:90`
   - **TODO**: Implement GetCurrentUser RPC method
   - **Kontext**: Currently throws error, needs backend RPC method or token parsing
   - **Schätzung**: 3-4 Stunden

---

## 🟡 Medium Priority

### Backend

6. **Error Handling - RpcError Conversion**
   - **Datei**: `backend/src/auth/claims.rs:155`
   - **TODO**: Improve error conversion from ApiError to RpcError
   - **Kontext**: Currently basic conversion, could be more comprehensive
   - **Schätzung**: 2-3 Stunden

7. **Storage Service - Error Handling**
   - **Datei**: `backend/src/api/v1/storage/connect.rs`
   - **TODO**: Add proper error handling for storage operations
   - **Kontext**: Some operations return empty responses on error, should return proper RpcError
   - **Schätzung**: 3-4 Stunden

### Frontend

8. **Feature Flags**
   - **Datei**: `frontend/src/lib/config.ts`
   - **TODO**: Implement feature flags from config
   - **Kontext**: Config has `features` object, but not used in UI
   - **Schätzung**: 2-3 Stunden

9. **Error Boundary - Connect-RPC Errors**
   - **Datei**: `frontend/src/components/ErrorBoundary.tsx`
   - **TODO**: Improve error boundary to handle Connect-RPC errors
   - **Kontext**: Should display user-friendly messages for RPC errors
   - **Schätzung**: 2-3 Stunden

---

## 🟢 Low Priority

### Backend

10. **REST Client Deprecation**
    - **Datei**: `backend/src/api/v1/*/routes.rs`
    - **TODO**: Plan deprecation timeline for REST endpoints
    - **Kontext**: Connect-RPC is now primary, REST should be deprecated
    - **Schätzung**: Planning only

11. **Documentation - API Examples**
    - **Datei**: Various handler files
    - **TODO**: Add more comprehensive doc examples
    - **Kontext**: Some handlers lack usage examples
    - **Schätzung**: 4-6 Stunden

### Frontend

12. **REST Client Removal**
    - **Datei**: `frontend/src/api/rest/`
    - **TODO**: Remove deprecated REST client exports
    - **Kontext**: Marked as deprecated, should be removed in next major version
    - **Schätzung**: 1-2 Stunden

13. **Type Definitions Cleanup**
    - **Datei**: `frontend/src/api/types/index.ts`
    - **TODO**: Remove deprecated type definitions
    - **Kontext**: Legacy types marked as deprecated, should be removed
    - **Schätzung**: 1-2 Stunden

---

## 📝 Notes & Improvements

### Code Quality

- **Backend**: Consider adding more integration tests for Connect-RPC handlers
- **Frontend**: Add unit tests for helper functions (`toUser`, `toStorageObject`, etc.)
- **Both**: Improve error messages for better debugging

### Performance

- **Backend**: Consider connection pooling optimizations
- **Frontend**: Implement request caching for frequently accessed data
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
- **High Priority**: 5
- **Medium Priority**: 4
- **Low Priority**: 4

---

## 📚 Related Documents

- [Style Guide](STYLE_GUIDE.md)
- [Harmonization Roadmap](HARMONIZATION_ROADMAP.md)
- [Architecture Guide](architecture.md)
- [Testing Guide](testing.md)
