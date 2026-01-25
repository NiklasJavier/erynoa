# 📋 Strukturelle Verbesserungsvorschläge

## 🎯 Übersicht

Dieses Dokument listet geplante strukturelle Verbesserungen für das Godstack-Projekt auf, um die Übersichtlichkeit, Wartbarkeit und Skalierbarkeit zu erhöhen.

---

## 🔴 Priorität 1: Kritische Verbesserungen

### 1. Backend API Struktur - Feature-basierte Organisation

**Problem:**
- Alle Handler sind flach in `api/handlers/` organisiert
- Keine klare Trennung nach Domänen/Features
- Schwer skalierbar bei wachsendem Code

**Lösung:**
```
backend/src/api/
├── mod.rs
├── routes.rs
├── middleware/          # Neu: Middleware-Layer
│   ├── mod.rs
│   ├── auth.rs         # Auth-Middleware
│   ├── logging.rs      # Request-Logging
│   └── error_handler.rs # Error-Handling
├── v1/                  # API Versionierung
│   ├── mod.rs
│   ├── health/          # Health-Check Domain
│   │   ├── mod.rs
│   │   ├── handler.rs
│   │   └── routes.rs
│   ├── users/           # User Domain
│   │   ├── mod.rs
│   │   ├── handler.rs
│   │   ├── routes.rs
│   │   └── models.rs    # Request/Response Types
│   ├── storage/         # Storage Domain
│   │   ├── mod.rs
│   │   ├── handler.rs
│   │   ├── routes.rs
│   │   └── models.rs
│   └── info/            # Info Domain
│       ├── mod.rs
│       ├── handler.rs
│       └── routes.rs
└── shared/              # Shared API Utilities
    ├── mod.rs
    ├── pagination.rs    # Pagination Helpers
    └── validation.rs    # Request Validation
```

**Vorteile:**
- Klare Feature-Trennung
- Einfacheres Testing pro Domain
- Bessere Skalierbarkeit
- Versionierung vorbereitet

---

### 2. Storage Handler Integration

**Problem:**
- `storage.rs` Handler existiert, aber ist nicht in `routes.rs` registriert
- Storage-Routen fehlen komplett

**Lösung:**
- Storage-Handler in `api/handlers/mod.rs` aufnehmen
- Routen in `routes.rs` hinzufügen:
  ```rust
  .route("/storage/upload", post(handlers::storage::upload))
  .route("/storage/objects", get(handlers::storage::list_objects))
  .route("/storage/objects/:key", get(handlers::storage::get_object))
  .route("/storage/objects/:key", delete(handlers::storage::delete_object))
  .route("/storage/presigned", post(handlers::storage::presigned_url))
  ```

---

### 3. Frontend API Client Konsolidierung

**Problem:**
- Zwei parallele API-Client-Implementierungen:
  - `api/client.ts` (REST)
  - `api/connect.ts` (Connect-RPC)
- Unklare Verwendung, welche wann genutzt wird

**Lösung Option A (Empfohlen):**
```
frontend/src/api/
├── index.ts              # Hauptexport
├── rest/                 # REST Client
│   ├── client.ts
│   └── endpoints.ts
├── connect/              # Connect-RPC Client
│   ├── transport.ts
│   └── services.ts
└── types/                # Shared Types
    ├── index.ts
    ├── user.ts
    ├── storage.ts
    └── common.ts
```

**Lösung Option B:**
- Connect-RPC als primärer Client (bessere Performance)
- REST nur für Legacy/Storage
- Klare Dokumentation wann was verwendet wird

---

## 🟡 Priorität 2: Wichtige Verbesserungen

### 4. Frontend Type-Definitionen zentralisieren

**Problem:**
- Types sind über verschiedene Dateien verstreut
- Keine zentrale Quelle der Wahrheit
- Duplikation zwischen API-Responses und Components

**Lösung:**
```
frontend/src/
├── types/                # Neu: Zentrale Types
│   ├── index.ts         # Re-exports
│   ├── api/             # API Response Types
│   │   ├── user.ts
│   │   ├── storage.ts
│   │   └── common.ts
│   ├── domain/          # Domain Models
│   │   ├── user.ts
│   │   └── file.ts
│   └── ui/              # UI-spezifische Types
│       └── theme.ts
```

---

### 5. Backend Middleware-Struktur

**Problem:**
- Middleware-Logik ist in `routes.rs` verstreut
- Keine wiederverwendbaren Middleware-Komponenten
- CORS-Konfiguration könnte ausgelagert werden

**Lösung:**
```
backend/src/api/
├── middleware/
│   ├── mod.rs
│   ├── auth.rs          # JWT-Validation
│   ├── cors.rs          # CORS-Konfiguration
│   ├── logging.rs       # Request/Response Logging
│   ├── rate_limit.rs    # Rate Limiting (zukünftig)
│   └── error_handler.rs # Global Error Handler
```

---

### 6. Validierungsschicht

**Problem:**
- Keine zentrale Validierung
- Validierung in Handlern verstreut
- Keine wiederverwendbaren Validatoren

**Lösung:**
```
backend/src/
├── validation/          # Neu: Validierung
│   ├── mod.rs
│   ├── user.rs          # User-Validierung
│   ├── storage.rs       # Storage-Validierung
│   └── common.rs        # Gemeinsame Validatoren
```

Oder mit `validator` Crate:
- Request-Types mit `#[derive(Validate)]`
- Automatische Validierung in Middleware

---

### 7. Dokumentation konsolidieren

**Problem:**
- Mehrere README/SETUP-Dateien im Root:
  - `README.md`
  - `DEV_SETUP.md`
  - `SETUP.md`
  - `DOCKER.md`
- Unklar, welche für wen gedacht ist

**Lösung:**
```
docs/
├── README.md            # Haupt-README (kurz, verweist auf andere)
├── getting-started/    # Neu: Getting Started Guides
│   ├── quick-start.md
│   ├── local-setup.md
│   └── devcontainer.md
├── development/        # Neu: Development Docs
│   ├── architecture.md
│   ├── api-design.md
│   └── testing.md
├── deployment/         # Neu: Deployment Docs
│   ├── docker.md
│   ├── production.md
│   └── infrastructure.md
└── guides/             # Neu: How-To Guides
    ├── zitadel-setup.md
    └── troubleshooting.md
```

**Root-Level:**
- `README.md` - Kurze Übersicht, Quick Start
- `CONTRIBUTING.md` - Beitragsrichtlinien (neu)

---

## 🟢 Priorität 3: Nice-to-Have Verbesserungen

### 8. Test-Struktur verbessern

**Problem:**
- Tests nur in `backend/tests/`
- Keine Frontend-Tests sichtbar
- Keine Integration-Test-Struktur

**Lösung:**
```
backend/
├── tests/
│   ├── integration/    # Integration Tests
│   │   ├── api/
│   │   │   ├── health_test.rs
│   │   │   ├── users_test.rs
│   │   │   └── storage_test.rs
│   │   └── common.rs    # Test Utilities
│   └── unit/            # Unit Tests (in src/ mit #[cfg(test)])
│       └── ...

frontend/
├── src/
│   └── ... (Tests neben Code)
└── tests/               # Neu: E2E Tests
    ├── e2e/
    └── setup.ts
```

---

### 9. Shared Types zwischen Frontend/Backend

**Problem:**
- Types müssen manuell synchronisiert werden
- Risiko von Inkonsistenzen

**Lösung:**
- Protobuf für API-Types (bereits vorhanden)
- Code-Generation für Frontend-Types aus Proto
- Oder: Shared TypeScript-Types (wenn möglich)

---

### 10. Environment-Konfiguration

**Problem:**
- Config-Dateien im Backend
- Frontend-Config in Code
- Keine zentrale Config-Verwaltung

**Lösung:**
```
config/                  # Neu: Root-Level Config (optional)
├── environments/
│   ├── development.toml
│   ├── staging.toml
│   └── production.toml
└── schema/              # Config-Schema für Validierung
    └── config.schema.json
```

---

### 11. Scripts-Organisation

**Problem:**
- Scripts in `infra/scripts/`
- Keine Kategorisierung

**Lösung:**
```
scripts/                 # Neu: Root-Level Scripts
├── setup/              # Setup-Scripts
│   ├── minio.sh
│   └── zitadel.sh
├── dev/                # Development-Scripts
│   └── cleanup-ports.sh
└── deploy/             # Deployment-Scripts (zukünftig)
    └── ...
```

Oder in `infra/scripts/` belassen, aber besser strukturieren.

---

## 📊 Implementierungsreihenfolge

### Phase 1: Foundation (Priorität 1)
1. ✅ Bereits erledigt: Root-Level Cleanup
2. 🔄 Storage Handler Integration
3. 🔄 Backend API Feature-Struktur
4. 🔄 Frontend API Client Konsolidierung

### Phase 2: Structure (Priorität 2)
5. Frontend Types zentralisieren
6. Backend Middleware-Struktur
7. Validierungsschicht
8. Dokumentation konsolidieren

### Phase 3: Polish (Priorität 3)
9. Test-Struktur
10. Shared Types
11. Environment-Config
12. Scripts-Organisation

---

## 🎯 Erwartete Vorteile

### Übersichtlichkeit
- ✅ Klare Feature-Trennung
- ✅ Einfacheres Navigieren im Code
- ✅ Bessere Onboarding-Erfahrung

### Wartbarkeit
- ✅ Einfacheres Finden von Code
- ✅ Reduzierte Kopplung
- ✅ Bessere Testbarkeit

### Skalierbarkeit
- ✅ Einfaches Hinzufügen neuer Features
- ✅ Klare API-Versionierung
- ✅ Wiederverwendbare Komponenten

---

## 📝 Notizen

- Alle Änderungen sollten rückwärtskompatibel sein
- Schrittweise Migration empfohlen
- Tests sollten parallel migriert werden
- Dokumentation sollte bei jeder Änderung aktualisiert werden
