# Integration: Core ↔ Domain ↔ Local im SystemState

Prüfung, ob **core**, **domain** und **local** nahtlos im SystemState (UnifiedState) integriert sind.

---

## Kurzfassung

| Integration        | Status     | Beschreibung |
|--------------------|------------|--------------|
| **core ↔ domain**  | ✅ Nahtlos | state.rs nutzt durchgängig domain-Typen (DID, TrustVector6D, UniversalId, …). |
| **local ↔ domain** | ✅ Nahtlos | local-Stores nutzen domain-Typen (DID, EventId, RealmId, TrustVector6D, …). |
| **core ↔ local**  | ⚠️ Getrennt | UnifiedState referenziert **kein** local::DecentralizedStorage. Anbindung nur auf AppState-Ebene. |

---

## 1. Core ↔ Domain ✅

- **state.rs** importiert und verwendet:
  - `crate::domain::unified::primitives::UniversalId`
  - `crate::domain::MemberRole`
  - `crate::domain::unified::identity::{DID, DIDDocument, DIDNamespace, Delegation, Capability}`
  - `crate::domain::unified::TrustVector6D`
  - `crate::domain::unified::primitives::TemporalCoord`
- IdentityState, RealmSpecificState, RealmSnapshot, SwarmState etc. bauen auf diesen Typen auf.
- **Fazit:** Domain ist die gemeinsame Typenbasis für Core-State; Integration ist nahtlos.

---

## 2. Local ↔ Domain ✅

- **local/** nutzt durchgängig domain-Typen:
  - `trust_store.rs`: TrustVector6D, DID
  - `event_store.rs`: Event, EventId, FinalityState, UniversalId, DID
  - `identity_store.rs`: DIDNamespace, DID
  - `realm_storage.rs`: RealmId, DID
  - `content_store.rs`, `blueprint_marketplace.rs`: DID
  - `mod.rs`: RealmId (realm_store_count)
- **Fazit:** Local ist vollständig auf dem Domain-Modell aufgebaut; Integration ist nahtlos.

---

## 3. Core ↔ Local im SystemState ⚠️

### Was UnifiedState enthält

- **storage: StorageState** – nur **in-memory Metriken** (kv_keys, kv_bytes, event_store_count, total_bytes, …). Keine Referenz auf echten Persistenz-Storage.
- **storage_handle: StorageHandle** – abstraktes **StorageBackend** (RocksDB, IPFS, Cloud, Memory). Kein Typ aus `local`.
- **Kein** Feld vom Typ `DecentralizedStorage`, `RealmStorage`, `IdentityStore` o.ä.

### Wo Local tatsächlich angebunden ist

- **AppState** (server.rs) hält beides nebeneinander:
  - `unified_state: SharedUnifiedState` (core)
  - `storage: DecentralizedStorage` (local)
- **API / ECLVM** nutzen Persistenz über `state.storage` (DecentralizedStorage), z.B.:
  - `state.storage.identities.store_passkey_credential`
  - `state.storage.ping()`
  - ECLVM ErynoaHost: `self.storage.trust`, `self.storage.identities`, `self.storage.realm`
- **Core-intern** gibt es eine **eigene** Persistenz-Abstraktion:
  - **RealmStorageLoader** (Trait in state.rs)
  - **ProductionStorageService** mit eigenem Backend (InMemoryStorage, optional Fjall-Pfad), **nicht** local::DecentralizedStorage
  - **LazyShardedRealmState** mit optionalem `storage_loader: Arc<dyn RealmStorageLoader>` – wird nirgends mit DecentralizedStorage gefüllt

### Konsequenz

- **UnifiedState** = reiner Core-State (inkl. Domain-Typen) + in-memory Storage-Metriken + abstraktes StorageHandle.
- **Persistenz** (local) ist **nicht** Teil des SystemState, sondern wird auf App-Ebene daneben gehalten und von API/ECLVM direkt genutzt.
- Es existiert **kein** Adapter, der `RealmStorageLoader` mit `local::DecentralizedStorage` (oder RealmStorage) implementiert; zwei getrennte „Storage-Welten“.

---

## 4. Empfehlung für nahtlose Core–Local-Integration

Wenn „nahtlos“ bedeuten soll, dass der **SystemState** (UnifiedState) auch den **einen** Persistenz-Storage kennt:

1. **Option A – Adapter im Core**
   - In **local** (oder einem kleinen Brücken-Modul) ein Typ `DecentralizedRealmStorageLoader` implementieren, der `RealmStorageLoader` implementiert und intern `DecentralizedStorage` / `RealmStorage` nutzt.
   - Beim Start (z.B. in server.rs) diesen Loader erstellen und an alle Stellen injizieren, die heute schon `RealmStorageLoader` erwarten (z.B. `LazyShardedRealmState::with_storage(...)`), und dafür **eine** DecentralizedStorage-Instanz verwenden.

2. **Option B – UnifiedState kennt Persistenz**
   - UnifiedState (oder ein zentraler „SystemState“-Container) um ein optionales `persistence: Option<Arc<DecentralizedStorage>>` (oder ein schlankes Trait-Handle) erweitern.
   - AppState baut UnifiedState mit dieser Referenz auf; API/ECLVM/Realm-Loader lesen Persistenz dann aus dem State statt aus einem separaten `state.storage`.

3. **Option C – So lassen**
   - Bewusst **orthogonal** lassen: UnifiedState = flüchtiger + Metriken-State, Persistenz = AppState.storage. Dann ist die aktuelle Trennung gewollt; „nahtlos“ bezieht sich nur auf Core↔Domain und Local↔Domain.

---

## 5. Abhängigkeitsgrafik (vereinfacht)

```text
                    ┌─────────────────────────────────────────────────────────┐
                    │                    domain (Typen)                       │
                    │  UniversalId, DID, TrustVector6D, EventId, RealmId, …   │
                    └───────────────────────┬─────────────────────────────────┘
                                            │
              ┌─────────────────────────────┼─────────────────────────────┐
              │                             │                             │
              ▼                             ▼                             ▼
    ┌─────────────────┐           ┌─────────────────┐           ┌─────────────────┐
    │      core       │           │      local       │           │  api / eclvm    │
    │  state.rs      │           │ Decentralized   │           │  (AppState)     │
    │  UnifiedState  │           │ Storage         │           │                 │
    │  StorageState  │           │ IdentityStore   │           │  unified_state  │
    │  (nur Metriken)│           │ EventStore      │           │  + storage      │
    │  StorageHandle │           │ TrustStore      │           │                 │
    │  (abstrakt)    │           │ RealmStorage    │           └────────┬────────┘
    └────────┬───────┘           └─────────────────┘                    │
             │                            │                             │
             │  RealmStorageLoader        │                             │
             │  (ProductionStorageService │   Keine direkte Verbindung   │
             │   mit eigenem Backend)     │   zwischen core und local    │
             └────────────────────────────┴─────────────────────────────┘
```

---

**Stand:** Prüfung basierend auf state.rs, local/mod.rs, server.rs, api/eclvm-Nutzung von `state.storage`.
