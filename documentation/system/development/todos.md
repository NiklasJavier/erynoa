# Erynoa – TODO Management

> **Dokumenttyp:** Tracking
> **Bereich:** Entwicklung
> **Status:** ✅ Alle Prioritäten abgeschlossen
> **Lesezeit:** ca. 5 Minuten

---

## Übersicht

Zentrale Sammlung aller **TODOs**, **FIXMEs** und geplanten Verbesserungen.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📋 TODO STATUS                                                            │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   🔴 High       🟡 Medium      🟢 Low          📊 Total             │  │
│   │   ─────────     ──────────     ─────────      ─────────             │  │
│   │   5/5 ✅        4/4 ✅         4/4 ✅         13/13 ✅              │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   🎉 Alle TODOs aus dem initialen Backlog wurden abgeschlossen!            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🔴 High Priority

<details>
<summary><strong>✅ Backend – User Service (3 Tasks)</strong></summary>

| #   | Task              | Datei                 | Status |
| :-- | :---------------- | :-------------------- | :----: |
| 1   | Timestamp Support | `users/connect.rs:42` |   ✅   |
| 2   | Count Query       | `users/connect.rs:58` |   ✅   |
| 3   | Name from ZITADEL | `users/connect.rs:40` |   ✅   |

**Details:**

- Timestamps werden aus DB geladen → Protobuf Timestamp
- `User::count()` parallel zur User-Liste
- Email als Name-Fallback implementiert

</details>

<details>
<summary><strong>✅ Console – User & Storage (2 Tasks)</strong></summary>

| #   | Task                    | Datei                          | Status |
| :-- | :---------------------- | :----------------------------- | :----: |
| 4   | Storage Upload Progress | `storage/connect-client.ts:62` |   ✅   |
| 5   | GetCurrentUser          | `users/connect-client.ts:90`   |   ✅   |

**Details:**

- Presigned URLs für große Dateien (>5MB)
- `GetCurrent` RPC-Methode im Backend

</details>

---

## 🟡 Medium Priority

<details>
<summary><strong>✅ Backend – Error Handling (2 Tasks)</strong></summary>

| #   | Task                   | Datei                | Status |
| :-- | :--------------------- | :------------------- | :----: |
| 6   | RpcError Conversion    | `auth/claims.rs:155` |   ✅   |
| 7   | Storage Error Handling | `storage/connect.rs` |   ✅   |

**Details:**

- `ApiErrorToRpc` Trait für konsistente Konvertierung
- `Result<T, RpcError>` für alle Storage-Handler

</details>

<details>
<summary><strong>✅ Console – UX (2 Tasks)</strong></summary>

| #   | Task           | Datei                          | Status |
| :-- | :------------- | :----------------------------- | :----: |
| 8   | Feature Flags  | `lib/features.tsx`             |   ✅   |
| 9   | Error Boundary | `components/ErrorBoundary.tsx` |   ✅   |

**Details:**

- `ConfigProvider` + `useFeatureFlags()` Hook
- Connect-RPC Error Handling mit deutschen Meldungen

</details>

---

## 🟢 Low Priority

<details>
<summary><strong>✅ Documentation & Cleanup (4 Tasks)</strong></summary>

| #   | Task                     | Status |
| :-- | :----------------------- | :----: |
| 10  | REST Deprecation Plan    |   ✅   |
| 11  | API Examples             |   ✅   |
| 12  | REST Client Removal      |   ✅   |
| 13  | Type Definitions Cleanup |   ✅   |

**Details:**

- Deprecation-Plan mit Timeline erstellt
- Doc-Beispiele für User & Storage Service
- `rest/` Verzeichnis vollständig entfernt
- Nur noch Error-Types exportiert

</details>

---

## 📝 Backlog – Zukünftige Verbesserungen

### Code Quality

| Bereich | Verbesserung                           | Priorität |
| :------ | :------------------------------------- | :-------: |
| Backend | Mehr Integration Tests für Connect-RPC |    🟡     |
| Console | Unit Tests für Helper Functions        |    🟡     |
| Beide   | Bessere Error Messages                 |    🟢     |

### Performance

| Bereich | Verbesserung                   | Priorität |
| :------ | :----------------------------- | :-------: |
| Backend | Connection Pooling Optimierung |    🟢     |
| Console | Request Caching                |    🟢     |
| Beide   | Performance Monitoring/Metrics |    🟢     |

### Documentation

| Bereich      | Verbesserung                       | Priorität |
| :----------- | :--------------------------------- | :-------: |
| API          | OpenAPI für REST (vor Deprecation) |    🟢     |
| Connect-RPC  | Alle Methoden dokumentieren        |    🟡     |
| Architecture | Connect-RPC Details ergänzen       |    🟢     |

---

## 🔄 Review-Prozess

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📅 TODO LIFECYCLE                                                         │
│                                                                             │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐            │
│   │  Create  │ ─▶ │ Prioritize│ ─▶ │  Work    │ ─▶ │ Complete │            │
│   │  TODO    │    │  Weekly   │    │  Sprint  │    │  Review  │            │
│   └──────────┘    └──────────┘    └──────────┘    └──────────┘            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

| Schritt             | Beschreibung                      |
| :------------------ | :-------------------------------- |
| **Weekly Review**   | TODOs priorisieren                |
| **Sprint Planning** | High Priority in Sprint aufnehmen |
| **Completion**      | Dokumentation aktualisieren       |
| **Cleanup**         | Erledigte TODOs archivieren       |

---

## 📋 TODO Template

```markdown
### [Bereich] – [Kurzbeschreibung]

| Feld          | Wert                                    |
| :------------ | :-------------------------------------- |
| **Datei**     | `path/to/file.rs:line`                  |
| **Priorität** | 🔴 High / 🟡 Medium / 🟢 Low            |
| **Status**    | ⬜ Offen / 🔄 In Progress / ✅ Erledigt |

**Beschreibung:**
Was soll erreicht werden?

**Implementierung:**
Wie wurde es gelöst? (nach Abschluss)
```

---

## 📊 Statistiken

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📊 COMPLETION OVERVIEW                                                    │
│                                                                             │
│   Total:        ████████████████████████████████████████  13/13 (100%)     │
│   High:         ████████████████████████████████████████   5/5  (100%)     │
│   Medium:       ████████████████████████████████████████   4/4  (100%)     │
│   Low:          ████████████████████████████████████████   4/4  (100%)     │
│                                                                             │
│   Backlog:      9 Items (nicht priorisiert)                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 📚 Weiterführende Dokumente

| Dokument                                     | Beschreibung       |
| :------------------------------------------- | :----------------- |
| [Style Guide](style-guide.md)                | Code-Standards     |
| [Testing](testing.md)                        | Test-Strategien    |
| [Architecture](../reference/architecture.md) | System-Architektur |

---

<div align="center">

```
┌───────────────────────────────────────────────┐
│                                               │
│   📋 Plan   →   🔨 Build   →   ✅ Ship       │
│   TODOs        Implement      Deliver        │
│                                               │
└───────────────────────────────────────────────┘
```

**🎉 Initialer Backlog vollständig abgearbeitet!**

</div>
