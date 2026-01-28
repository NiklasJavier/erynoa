# 🏗️ Struktur-Verbesserungen

**Status**: Planungsphase  
**Erstellt**: 2026-01-25

## Übersicht

Dieses Dokument beschreibt konkrete Verbesserungen zur Vereinfachung und Übersichtlichkeit der Projektstruktur.

---

## 🎯 Hauptziele

1. **Redundanz eliminieren** - Deprecated Code entfernen
2. **Konsolidierung** - Ähnliche Dateien zusammenführen
3. **Klarheit** - Eindeutige Verantwortlichkeiten
4. **Wartbarkeit** - Einfacher zu verstehen und zu erweitern

---

## 📋 Verbesserungsvorschläge

### 1. Backend: REST-Code entfernen (High Priority)

**Problem**: REST-Endpoints sind deprecated, aber Code existiert noch parallel zu Connect-RPC.

**Aktuell**:
```
backend/src/api/v1/users/
├── handler.rs      # REST handlers (deprecated)
├── connect.rs      # Connect-RPC handlers (primary)
├── models.rs       # Request/Response types
├── routes.rs       # REST routes (deprecated)
└── mod.rs
```

**Implementiert**:
```
backend/src/api/v1/users/
├── handlers.rs     # Connect-RPC handlers (umbenannt von connect.rs)
└── mod.rs          # Module exports
```

**Hinweis**: `types.rs` wurde nicht benötigt, da alle Types aus Protobuf-Definitionen kommen.

**Aktionen**:
- [x] `handler.rs` entfernen (REST handlers) ✅
- [x] `routes.rs` entfernen (REST routes) ✅
- [x] `connect.rs` → `handlers.rs` umbenennen ✅
- [x] `models.rs` → `types.rs` umbenennen (später entfernt, da nicht mehr benötigt) ✅
- [x] REST-Route-Registrierung aus `api/routes.rs` entfernen ✅
- [x] `types.rs` Dateien entfernen (REST-spezifische Types nicht mehr benötigt) ✅

**Vorteile**:
- ✅ 50% weniger Dateien pro Feature
- ✅ Klarere Struktur (nur Connect-RPC)
- ✅ Weniger Wartungsaufwand

---

### 2. Console: Config-Dateien konsolidieren (Medium Priority)

**Problem**: Drei separate Config-Dateien mit Überschneidungen.

**Aktuell**:
```
frontend/console/src/lib/
├── config.ts           # fetchConfig(), AppConfig
├── config-schema.ts    # Zod schema, Config type
└── config-defaults.ts  # DEFAULT_CONFIG
```

**Vorgeschlagen**:
```
frontend/console/src/lib/
└── config.ts           # Alles in einer Datei
    ├── Schema (Zod)
    ├── Types
    ├── Defaults
    └── fetchConfig()
```

**Aktionen**:
- [x] `config-schema.ts` in `config.ts` integrieren ✅
- [x] `config-defaults.ts` in `config.ts` integrieren ✅
- [x] Exports anpassen ✅
- [x] Alte Dateien löschen ✅

**Vorteile**:
- ✅ Alles an einem Ort
- ✅ Einfacher zu finden
- ✅ Weniger Imports

---

### 3. Scripts konsolidieren (Low Priority)

**Problem**: Scripts sind auf zwei Orte verteilt.

**Aktuell**:
```
/workspace/
├── scripts/
│   └── dev-check.sh
└── infra/
    └── scripts/
        ├── dev/
        ├── setup/
        └── test/
```

**Vorgeschlagen**:
```
/workspace/
└── scripts/
    ├── dev/
    │   └── dev-check.sh
    ├── setup/
    │   ├── setup-minio.sh
    │   └── setup-zitadel.sh
    └── test/
        ├── runtime-test.sh
        └── test-all.sh
```

**Aktionen**:
- [x] `infra/scripts/` nach `scripts/` verschieben ✅
- [x] `scripts/dev-check.sh` nach `scripts/dev/` verschieben ✅
- [x] Pfade in `justfile` anpassen ✅
- [x] Alte `infra/scripts/` Verzeichnis entfernen ✅

**Vorteile**:
- ✅ Alle Scripts an einem Ort
- ✅ Konsistente Struktur
- ✅ Einfacher zu finden

---

### 4. Backend: Error-Module vereinfachen (Medium Priority)

**Problem**: Error-Handling ist auf zwei Dateien verteilt.

**Aktuell**:
```
backend/src/
├── error.rs          # ApiError, IntoResponse
└── error/
    └── rpc.rs        # ApiErrorToRpc trait
```

**Vorgeschlagen**:
```
backend/src/
└── error.rs          # Alles in einer Datei
    ├── ApiError
    ├── IntoResponse
    └── RpcError conversion (mod rpc)
```

**Aktionen**:
- [x] `error/rpc.rs` in `error.rs` integriert (als `mod rpc`) ✅
- [x] Alle Imports angepasst ✅
- [x] Alte `error/rpc.rs` Datei entfernt ✅

**Vorteile**:
- ✅ Weniger Dateien
- ✅ Alles an einem Ort
- ✅ Einfacher zu navigieren

---

### 5. Console: API-Types vereinfachen (Low Priority)

**Problem**: `api/types/` und `api/*/types.ts` haben Überschneidungen.

**Aktuell**:
```
frontend/console/src/api/
├── types/
│   ├── errors.ts     # Error types (wird verwendet)
│   └── index.ts      # Deprecated types (nur errors.ts exportiert)
└── */types.ts        # Feature-spezifische Types
```

**Vorgeschlagen**:
```
frontend/console/src/api/
├── errors.ts         # Error types (direkt in api/)
└── */types.ts        # Feature-spezifische Types
```

**Aktionen**:
- [ ] `api/types/errors.ts` → `api/errors.ts` verschieben
- [ ] `api/types/index.ts` entfernen (nur noch errors.ts exportiert)
- [ ] Imports anpassen

**Vorteile**:
- ✅ Weniger Verschachtelung
- ✅ Klarere Struktur
- ✅ Einfacher zu finden

---

### 6. Docs: Bessere Organisation (Low Priority)

**Problem**: Viele Docs, teilweise redundant.

**Aktuell**:
```
docs/
├── essential_guide.md
├── development/
│   ├── architecture.md
│   ├── style_guide.md
│   ├── testing.md
│   ├── todos.md
│   └── rest_deprecation_plan.md
├── guides/
│   └── zitadel.md
├── reference/
│   ├── connections.md
│   └── service_config.md
└── setup/
    ├── dev_setup.md
    ├── docker.md
    └── setup.md
```

**Vorgeschlagen** (optional):
```
docs/
├── readme.md              # Übersicht + Quick Start
├── guides/                # Schritt-für-Schritt Guides
│   ├── getting-started.md
│   ├── setup.md
│   └── zitadel.md
├── reference/             # Referenz-Dokumentation
│   ├── architecture.md
│   ├── api.md
│   └── config.md
└── development/          # Development-spezifisch
    ├── style-guide.md
    ├── testing.md
    └── todos.md
```

**Aktionen**:
- [ ] Optional: Docs umorganisieren
- [ ] readme.md in docs/ erstellen
- [ ] Links aktualisieren

**Vorteile**:
- ✅ Klarere Kategorisierung
- ✅ Einfacher zu navigieren
- ✅ Bessere Struktur

---

## 📊 Priorisierung

### Phase 1: High Impact, Low Risk (Sofort umsetzbar)
1. ✅ **Backend: REST-Code entfernen** - **ABGESCHLOSSEN** ✅
   - Alle REST-Handler entfernt
   - Alle REST-Routen entfernt
   - Dateien umbenannt: `connect.rs` → `handlers.rs`
   - REST-spezifische Types entfernt
   - `api/routes.rs` bereinigt
   - **Ergebnis**: 16 Dateien weniger, klarere Struktur

2. ✅ **Console: Config konsolidieren** - **ABGESCHLOSSEN** ✅
   - `config-schema.ts` in `config.ts` integriert
   - `config-defaults.ts` in `config.ts` integriert
   - Alle Exports konsolidiert
   - Alte Dateien entfernt
   - **Ergebnis**: 2 Dateien weniger, alles an einem Ort

### Phase 2: Medium Impact (Nach Phase 1)
3. ✅ **Backend: Error-Module vereinfachen** - **ABGESCHLOSSEN** ✅
   - `error/rpc.rs` in `error.rs` integriert (als `mod rpc`)
   - Alle Imports angepasst
   - Alte Datei entfernt
   - **Ergebnis**: 1 Datei weniger, alles an einem Ort

4. ✅ **Scripts konsolidieren** - **ABGESCHLOSSEN** ✅
   - `infra/scripts/` nach `scripts/` verschoben
   - `dev-check.sh` nach `scripts/dev/` verschoben
   - Alle Pfade in `justfile` angepasst
   - Alte Verzeichnisse entfernt
   - **Ergebnis**: Konsistente Struktur, alle Scripts an einem Ort

### Phase 3: Low Priority (Optional)
5. ✅ **Console: API-Types vereinfachen** - **ABGESCHLOSSEN** ✅
   - `api/types/errors.ts` → `api/errors.ts` verschoben
   - `api/types/index.ts` entfernt
   - `api/types/` Verzeichnis entfernt
   - **Ergebnis**: Weniger Verschachtelung, klarere Struktur

6. ✅ **Docs reorganisieren** - **ABGESCHLOSSEN** ✅
   - `docs/readme.md` erstellt (Übersicht + Quick Start)
   - Guides konsolidiert: `getting-started.md`, `setup.md`, `zitadel.md`
   - Reference organisiert: `architecture.md`, `config.md`, `connections.md`
   - Development-Dokumentation: `style-guide.md`, `testing.md`, `todos.md`
   - **Ergebnis**: Klarere Kategorisierung, einfacher zu navigieren

---

## 🚀 Umsetzungsplan

### Schritt 1: Backend REST-Code entfernen
```bash
# Pro Feature (users, storage, health, info):
1. handler.rs löschen
2. routes.rs löschen
3. connect.rs → handlers.rs umbenennen
4. models.rs → types.rs umbenennen
5. mod.rs anpassen
6. api/routes.rs anpassen (REST-Routen entfernen)
```

### Schritt 2: Console Config konsolidieren
```bash
1. config-schema.ts Inhalt nach config.ts kopieren
2. config-defaults.ts Inhalt nach config.ts kopieren
3. Imports anpassen
4. Alte Dateien löschen
```

### Schritt 3: Scripts konsolidieren
```bash
1. mkdir -p scripts/{dev,setup,test}
2. mv infra/scripts/* scripts/
3. mv scripts/dev-check.sh scripts/dev/
4. justfile Pfade anpassen
```

---

## 📈 Erwartete Verbesserungen

### Dateien reduziert
- Backend: ~16 Dateien weniger (4 Features × 4 Dateien)
- Console: 2 Dateien weniger (Config)
- Scripts: Struktur konsolidiert

### Wartbarkeit
- ✅ Klarere Struktur
- ✅ Weniger Redundanz
- ✅ Einfacher zu verstehen
- ✅ Schneller zu navigieren

### Onboarding
- ✅ Neue Entwickler finden sich schneller zurecht
- ✅ Weniger Verwirrung durch deprecated Code
- ✅ Klarere Verantwortlichkeiten

---

## ⚠️ Risiken & Überlegungen

### Breaking Changes
- **REST-Code entfernen**: Keine Breaking Changes, da REST deprecated ist
- **Config konsolidieren**: Keine Breaking Changes, nur interne Reorganisation
- **Scripts verschieben**: `justfile` muss angepasst werden

### Migration
- Alle Änderungen sind intern
- Keine API-Änderungen
- Keine Breaking Changes für Nutzer

---

## 📝 Nächste Schritte

1. **Review**: Dieses Dokument reviewen
2. **Priorisierung**: Entscheiden welche Phasen umgesetzt werden
3. **Umsetzung**: Schritt für Schritt implementieren
4. **Testing**: Sicherstellen dass alles noch funktioniert
5. **Dokumentation**: README und Docs aktualisieren

---

**Letzte Aktualisierung**: 2026-01-27
