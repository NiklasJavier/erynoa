# 📊 IST-Analyse: Backend Code-Struktur

> **Teil von:** Projekt Pluto
> **Analysiert:** 2026-02-04

---

## 1. Core-Layer Analyse

### 1.1 state.rs (21.495 Zeilen, 823 KB)

**Struktur:**
```
Zeilen 1-800     → Infrastruktur (NetworkEvent, EventBus, StateBroadcaster, CircuitBreaker)
Zeilen 800-1900  → StateEvent enum (42 Varianten)
Zeilen 1900-2500 → Event-Sourcing (WrappedStateEvent, StateEventLog)
Zeilen 2500-3000 → Merkle-Tracking (MerkleStateTracker, MerkleDelta)
Zeilen 3000-4100 → IdentityState, StateGraph
Zeilen 4100-6000 → Core-States (TrustState, EventState, FormulaState)
Zeilen 6000-8000 → Protection-States (AnomalyState, DiversityState)
Zeilen 8000-10000 → Peer-States (RealmState, GatewayState)
Zeilen 10000-12000 → Engine-States (ECLVMState, GovernanceState)
Zeilen 12000-21495 → UnifiedState + Tests
```

**Probleme:**
- Monolithisch: Eine Datei enthält alles
- Schwer testbar: Tests inline statt modular
- Merge-Konflikte häufig bei paralleler Entwicklung

### 1.2 Engine-Dateien

| Datei | Zeilen | Axiome | Abhängigkeiten |
|-------|--------|--------|----------------|
| `engine.rs` | 719 | - | ExecutionContext, alle Engines |
| `trust_engine.rs` | 737 | Κ2-Κ5 | TrustVector6D, ExecutionContext |
| `event_engine.rs` | 715 | Κ9-Κ12 | Event, EventId, FinalityLevel |
| `world_formula.rs` | 727 | Κ15b-d | WorldFormulaContribution, Surprisal |
| `consensus.rs` | 753 | Κ18 | FinalityLevel, WitnessAttestation |
| `state_integration.rs` | 6.427 | - | Alle Observer-Traits |

**Gut:**
- Jede Engine implementiert `*_with_ctx()` Methoden
- Gas-Accounting integriert
- Klare Axiom-Referenzen

**Problem:**
- `engine.rs` wrappet alle anderen → Redundanz
- Observer-Traits in 6.400 Zeilen verstreut

---

## 2. Peer-Layer Analyse

### 2.1 Struktur

```
peer/
├── mod.rs           (37 Zeilen)
├── gateway.rs       (20 KB) - Κ23 Realm-Crossing
├── intent_parser.rs (11 KB) - Κ22 Intent→Saga
├── saga_composer.rs (21 KB) - Κ22,Κ24 Orchestrierung
└── p2p/             (38 Dateien, ~400 KB)
    ├── behaviour/
    ├── gossip/
    ├── kademlia/
    ├── relay/
    ├── privacy/
    └── trust_gate/
```

**Problem:**
- 38 Dateien in `p2p/` ohne klare Hierarchie
- Viele kleine Dateien mit ähnlicher Funktionalität
- Keine einheitlichen Traits

---

## 3. Storage-Layer Analyse

### 3.1 Struktur (local/)

| Datei | Größe | Funktion |
|-------|-------|----------|
| `mod.rs` | 35 KB | DecentralizedStorage, Snapshots |
| `realm_storage.rs` | 106 KB | Schema-Evolution, Stores |
| `blueprint_marketplace.rs` | 71 KB | Blueprint-Management |
| `identity_store.rs` | 29 KB | DID-Persistenz |
| `content_store.rs` | 20 KB | CAS (BLAKE3) |
| `event_store.rs` | 17 KB | Event-Persistenz |
| `trust_store.rs` | 19 KB | Trust-Persistenz |
| `kv_store.rs` | 12 KB | Key-Value Basis |
| `archive.rs` | 19 KB | Cold Storage |
| `metrics.rs` | 29 KB | Store-Metriken |

**Gut:**
- Einheitliches Metriken-Framework
- Health-Score pro Store
- StorageState-Integration vorhanden

**Problem:**
- `realm_storage.rs` mit 106 KB zu groß
- `blueprint_marketplace.rs` sollte eigenes Modul sein

---

## 4. ECLVM-Layer Analyse

### 4.1 Struktur

```
eclvm/
├── mod.rs              (5 KB)
├── ast.rs              (21 KB) - AST-Definitionen
├── parser.rs           (30 KB) - ECL-Parser
├── bytecode.rs         (26 KB) - Bytecode-Format
├── compiler.rs         (13 KB) - AST→Bytecode
├── optimizer.rs        (21 KB) - Bytecode-Optimierung
├── erynoa_host.rs      (34 KB) - Host-Functions
├── bridge.rs           (25 KB) - State-Bridge
├── mana.rs             (16 KB) - Mana-Management
├── stdlib.rs           (15 KB) - Standardbibliothek
├── cli.rs              (19 KB) - CLI-Tools
├── entrypoints.rs      (12 KB) - Entry-Points
├── programmable_gateway.rs (25 KB) - Gateway-Integration
└── runtime/            (6 Dateien)
```

**Bewertung:** ✅ Gut strukturiert, keine größeren Änderungen nötig

---

## 5. Protection-Layer Analyse

```
protection/
├── mod.rs                  (2 KB)
├── adaptive_calibration.rs (22 KB) - Parameter-Tuning
├── anomaly.rs              (16 KB) - Anomalie-Detection
├── anti_calcification.rs   (11 KB) - Κ21 Decay
├── diversity.rs            (10 KB) - Κ19 Gini
└── quadratic.rs            (15 KB) - Κ20 Voting
```

**Bewertung:** ✅ Gut strukturiert, nur Trait-Integration fehlt

---

## 6. Identifizierte Redundanzen

| Redundanz | Orte | Lösung |
|-----------|------|--------|
| Snapshot-Pattern | state.rs, local/*.rs | Gemeinsamer Trait |
| Error-Typen | 8 verschiedene | Unified Error Hierarchy |
| Health-Score | state.rs, local/mod.rs | Gemeinsamer Trait |
| Observer-Traits | 30+ in state_integration.rs | Konsolidieren |
| Config-Structs | Verstreut | Zentrales config/ |

---

## 7. Abhängigkeitsanalyse

### Kritische Pfade

```
Identity → Trust → Event → Consensus → Finality
    ↓         ↓       ↓
   Realm → Gateway → Saga
    ↓
  ECLVM → Policy → Crossing
```

### Zirkuläre Risiken

| Von | Nach | Status |
|-----|------|--------|
| state.rs | domain/ | OK (nur Typen) |
| engines | state.rs | OK (über Observer) |
| local/ | state.rs | OK (update_storage_state) |
| peer/ | state.rs | ⚠️ Teilweise direkt |

---

## 8. Empfohlene Priorisierung

### Kritisch (Woche 1-2)
1. Unified Traits erstellen
2. Error-Hierarchie vereinheitlichen
3. state.rs aufsplitten beginnen

### Hoch (Woche 3-5)
4. StateEvent extrahieren
5. Merkle-Tracking extrahieren
6. Observer-Hub erstellen

### Mittel (Woche 6-10)
7. P2P konsolidieren
8. Storage refactoren
9. Engine-Wrapper entfernen

### Niedrig (Woche 11-12)
10. Performance-Tuning
11. Dokumentation
12. API-Stabilisierung
