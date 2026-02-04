# Core-Modul – Vollständiger Refactoring-Plan

Detaillierter Plan zur Restrukturierung des `backend/src/core/`-Moduls. Basiert auf der Analyse von `state.rs` (STATE-RS-REFERENCE.md) als Referenzarchitektur.

---

## 1. Analyse der aktuellen Situation

### 1.1 Dateien und Größen

| Datei | Zeilen | Beschreibung | Status |
|-------|--------|--------------|--------|
| `state.rs` | ~20.800 | Monolithisches State-Management | ⚠️ Kritisch groß |
| `state_integration.rs` | ~6.400 | Observer-Pattern-Integration | ⚠️ Groß |
| `state_coordination.rs` | ~680 | Transaktionen, Health, Invarianten | ✅ OK |
| `event_engine.rs` | ~715 | DAG-Management (Κ9-Κ12) | ✅ OK |
| `trust_engine.rs` | ~740 | Trust-Berechnung (Κ2-Κ5) | ✅ OK |
| `engine.rs` | ~300 | Unified Engine Layer | ✅ OK |
| `consensus.rs` | ~200 | Konsensus (Κ18) | ✅ OK |
| `surprisal.rs` | ~150 | Surprisal-Berechnung | ✅ OK |
| `world_formula.rs` | ~200 | Weltformel (Κ15b-d) | ✅ OK |
| `identity_types.rs` | ~400 | Identity Types & Traits | ✅ OK |
| `mod.rs` | ~290 | Re-exports | ⚠️ Zu viele Exports |

### 1.2 Kernprobleme

1. **`state.rs` ist ein Monolith (20.800 Zeilen)**
   - 15+ Sub-State-Typen in einer Datei
   - 50+ StateEvent-Varianten
   - Alle Snapshot-Typen
   - Event-Sourcing, Merkle-Tracker, Multi-Gas
   - StateView, StateHandle, ECLVMStateContext
   - Schwer zu navigieren, testen, maintainen

2. **Enge Kopplung zwischen State-Layern**
   - `UnifiedState` kennt alle Sub-States direkt
   - `apply_state_event()` ist ein riesiger Match-Block
   - Änderungen an einem State propagieren durch gesamte Datei

3. **Observer-Pattern-Komplexität**
   - `state_integration.rs` mit 6.400 Zeilen
   - 25+ Observer-Traits
   - `StateIntegrator` als God-Object

4. **Redundanz zwischen Engines und State**
   - `TrustEngine` vs `TrustState`
   - `EventEngine` vs `EventState`
   - Unklare Verantwortungstrennung

5. **ECLVM-Integration unklar**
   - `ECLVMStateContext` vs `ErynoaHost`
   - `StateView` vs Storage
   - Siehe ECL-ECLVM-REFACTORING-PLAN.md

---

## 2. Zielarchitektur

### 2.1 Verzeichnisstruktur (nach Refactoring)

```text
backend/src/core/
├── mod.rs                      # Minimale Re-exports
├── state/
│   ├── mod.rs                  # UnifiedState + create_unified_state()
│   ├── unified.rs              # UnifiedState, UnifiedSnapshot
│   ├── core_state.rs           # CoreState (Trust, Event, Formula, Consensus)
│   ├── execution_state.rs      # ExecutionState (Gas, Mana, Executions)
│   ├── eclvm_state.rs          # ECLVMState (Policies, Blueprints, Sagas)
│   ├── protection_state.rs     # ProtectionState (Anomaly, Diversity, etc.)
│   ├── storage_state.rs        # StorageState (Metriken)
│   ├── peer_state.rs           # PeerState (Gateway, Saga, Intent, Realm)
│   ├── p2p_state.rs            # P2PState (Swarm, Gossip, Kademlia, etc.)
│   ├── engine_state.rs         # UI, API, Governance, Controller, etc.
│   ├── identity_state.rs       # IdentityState (DIDs, Delegations, etc.)
│   ├── graph.rs                # StateGraph, StateComponent, StateRelation
│   ├── events.rs               # StateEvent, WrappedStateEvent
│   ├── event_log.rs            # StateEventLog
│   ├── merkle.rs               # MerkleStateTracker, MerkleDelta
│   ├── circuit_breaker.rs      # CircuitBreaker, SystemMode
│   ├── event_bus.rs            # EventBus, NetworkEvent
│   ├── broadcaster.rs          # StateBroadcaster, StateDelta
│   ├── storage_handle.rs       # StorageHandle, StorageBackend
│   ├── multi_gas.rs            # MultiGas, GasLayer
│   ├── view.rs                 # StateView (Read-Only)
│   ├── handle.rs               # StateHandle (Write Access)
│   └── context.rs              # ECLVMStateContext, TransactionGuard
├── integration/
│   ├── mod.rs                  # Re-exports
│   ├── traits.rs               # Alle Observer-Traits
│   ├── integrator.rs           # StateIntegrator
│   ├── core_observers.rs       # Trust/Event/Formula/Consensus-Observer
│   ├── execution_observers.rs  # Execution/Gas/Mana-Observer
│   ├── eclvm_observers.rs      # ECLVM-Observer, Adapter
│   ├── peer_observers.rs       # Gateway/Saga/Intent/Realm-Observer
│   ├── p2p_observers.rs        # Swarm/Gossip/Kademlia-Observer
│   ├── engine_observers.rs     # UI/API/Governance-Observer
│   └── protection_observers.rs # Anomaly/Diversity-Observer
├── coordination/
│   ├── mod.rs                  # Re-exports
│   ├── coordinator.rs          # StateCoordinator
│   ├── invariants.rs           # Invariant, InvariantResult
│   ├── health.rs               # HealthReport, HealthStatus
│   └── transaction.rs          # StateTransaction, TransactionError
├── engines/
│   ├── mod.rs
│   ├── trust.rs                # TrustEngine (existiert)
│   ├── event.rs                # EventEngine (existiert)
│   ├── consensus.rs            # ConsensusEngine (existiert)
│   ├── surprisal.rs            # SurprisalCalculator (existiert)
│   ├── world_formula.rs        # WorldFormulaEngine (existiert)
│   └── unified.rs              # engine.rs (existiert)
└── identity/
    ├── mod.rs
    └── types.rs                # identity_types.rs (existiert)
```

### 2.2 Modul-Verantwortlichkeiten

| Modul | Verantwortung |
|-------|---------------|
| `state/` | Alle State-Typen, Snapshots, Event-Sourcing |
| `integration/` | Observer-Traits, StateIntegrator |
| `coordination/` | Transaktionen, Health, Invarianten |
| `engines/` | Bestehende Engines (Trust, Event, etc.) |
| `identity/` | Identity-Types und Traits |

---

## 3. Refactoring-Phasen

### Phase 1: State-Modul aufteilen (Priorität: Hoch)

**Ziel:** `state.rs` in ~15 kleinere Dateien aufteilen ohne Breaking Changes.

#### 1.1 Infrastructure-Types extrahieren

**Dateien erstellen:**
- `state/circuit_breaker.rs` – SystemMode, CircuitBreaker, CircuitBreakerSnapshot
- `state/event_bus.rs` – EventPriority, NetworkEvent, EventBus, EventBusSnapshot
- `state/broadcaster.rs` – StateDelta, DeltaType, StateBroadcaster, BroadcasterSnapshot
- `state/storage_handle.rs` – StorageHandle, StorageBackend, StorageMetrics
- `state/merkle.rs` – MerkleHash, MerkleNode, Hashable, MerkleDelta, MerkleStateTracker
- `state/multi_gas.rs` – GasLayer, MultiGas, MultiGasSnapshot

**Schritte:**
1. Typen in neue Dateien verschieben
2. In `state/mod.rs` re-exportieren
3. In altem `state.rs` durch `use crate::core::state::*` ersetzen
4. Tests ausführen

**Aufwand:** ~2h pro Datei, gesamt ~12h

#### 1.2 Event-System extrahieren

**Dateien erstellen:**
- `state/events.rs` – TrustReason, AnomalySeverity, BlueprintActionType, RealmAction, MembershipAction, NetworkMetric, ECLPolicyType, StateEvent, StateEventEmitter, WrappedStateEvent
- `state/event_log.rs` – StateEventLog, EventLogSnapshot

**Schritte:**
1. Alle Event-Typen und Traits verschieben
2. Hilfsmethoden (`estimated_size_bytes()`, `is_critical()`, etc.) mitnehmen
3. In `state/mod.rs` re-exportieren

**Aufwand:** ~4h

#### 1.3 StateGraph extrahieren

**Datei erstellen:**
- `state/graph.rs` – StateComponent, StateRelation, StateGraph

**Schritte:**
1. Enum und Struct verschieben
2. `erynoa_graph()` und Query-Methoden mitnehmen

**Aufwand:** ~2h

#### 1.4 Sub-States extrahieren

**Dateien erstellen:**
- `state/core_state.rs` – TrustState, TrustSnapshot, TrustDistribution, EventState, EventSnapshot, FormulaState, FormulaSnapshot, ConsensusState, ConsensusSnapshot, CoreState, CoreSnapshot
- `state/execution_state.rs` – GasState, GasSnapshot, ManaState, ManaSnapshot, ExecutionsState, ExecutionsSnapshot, ExecutionState, ExecutionSnapshot
- `state/eclvm_state.rs` – ECLVMState, ECLVMSnapshot, BlueprintStatus, alle ECLVM-Zähler
- `state/protection_state.rs` – AnomalyState, DiversityState, QuadraticState, AntiCalcificationState, CalibrationState, ProtectionState, ProtectionSnapshot
- `state/storage_state.rs` – StorageState, StorageSnapshot
- `state/peer_state.rs` – GatewayState, GatewaySnapshot, SagaComposerState, IntentParserState, RealmState, RealmSpecificState, RealmQuota
- `state/p2p_state.rs` – SwarmState, GossipState, KademliaState, RelayState, PrivacyState, P2PState, P2PSnapshot
- `state/engine_state.rs` – UIState, APIState, GovernanceState, ControllerState, DataLogicState, BlueprintComposerState + alle Snapshots
- `state/identity_state.rs` – IdentityState, IdentitySnapshot (aus state.rs)

**Schritte (pro Datei):**
1. State-Struct und Snapshot-Struct verschieben
2. Alle `impl`-Blöcke mitnehmen
3. In `state/mod.rs` re-exportieren
4. Imports in anderen Dateien anpassen

**Aufwand:** ~3h pro Datei, gesamt ~27h

#### 1.5 View/Handle/Context extrahieren

**Dateien erstellen:**
- `state/view.rs` – StateView, IdentityViewData, RealmViewData, TrustViewData
- `state/handle.rs` – StateHandle, MutationResult, CommitResult, RollbackResult
- `state/context.rs` – ECLVMStateContext, ECLVMBudget, ECLVMBudgetLimits, ECLVMBudgetSnapshot, BudgetExhaustionReason, ECLVMExecutionSummary, TransactionGuard

**Aufwand:** ~4h

#### 1.6 UnifiedState in unified.rs

**Datei erstellen:**
- `state/unified.rs` – UnifiedState, UnifiedSnapshot, SharedUnifiedState, create_unified_state(), alle UnifiedState-Methoden

**Hinweis:** Dies ist die Hauptdatei, die alle anderen Sub-States zusammenführt.

**Schritte:**
1. UnifiedState-Struct verschieben
2. Alle `impl UnifiedState`-Blöcke verschieben
3. `apply_state_event()` bleibt hier (oder wird zu separatem Modul)
4. Re-exports in `state/mod.rs`

**Aufwand:** ~6h

#### 1.7 state/mod.rs erstellen

```rust
//! # State Module
//!
//! Hierarchisches, thread-safe State-Management für alle Erynoa-Module.

mod unified;
mod graph;
mod events;
mod event_log;
mod circuit_breaker;
mod event_bus;
mod broadcaster;
mod storage_handle;
mod merkle;
mod multi_gas;

mod core_state;
mod execution_state;
mod eclvm_state;
mod protection_state;
mod storage_state;
mod peer_state;
mod p2p_state;
mod engine_state;
mod identity_state;

mod view;
mod handle;
mod context;

// Re-exports (minimiert)
pub use unified::*;
pub use graph::*;
pub use events::*;
// ... alle anderen
```

**Aufwand:** ~2h

---

### Phase 2: Integration-Modul aufteilen (Priorität: Mittel)

**Ziel:** `state_integration.rs` in logische Einheiten aufteilen.

#### 2.1 Observer-Traits extrahieren

**Datei erstellen:**
- `integration/traits.rs` – Alle Observer-Traits (TrustObserver, EventObserver, ExecutionObserver, etc.)

**Aufwand:** ~2h

#### 2.2 Observer-Implementierungen gruppieren

**Dateien erstellen:**
- `integration/core_observers.rs` – TrustObserver, EventObserver, FormulaObserver, ConsensusObserver Impls
- `integration/execution_observers.rs` – ExecutionObserver Impls
- `integration/eclvm_observers.rs` – ECLVMObserver, ECLVMObserverAdapter
- `integration/peer_observers.rs` – GatewayObserver, SagaObserver, IntentObserver, RealmObserver
- `integration/p2p_observers.rs` – SwarmObserver, GossipObserver, KademliaObserver, RelayObserver, PrivacyObserver
- `integration/engine_observers.rs` – UIObserver, APIObserver, GovernanceObserver, ControllerObserver, DataLogicObserver, BlueprintComposerObserver
- `integration/protection_observers.rs` – ProtectionObserver, StorageObserver

**Aufwand:** ~3h pro Datei, gesamt ~18h

#### 2.3 StateIntegrator refaktorieren

**Datei:** `integration/integrator.rs`

**Änderungen:**
1. StateIntegrator bleibt zentral
2. Aber: Observer-Registrierung modularisieren
3. `CompositeObserver` vereinfachen

**Aufwand:** ~4h

---

### Phase 3: Coordination-Modul aufteilen (Priorität: Niedrig)

**Ziel:** `state_coordination.rs` besser strukturieren.

#### 3.1 Logische Trennung

**Dateien erstellen:**
- `coordination/invariants.rs` – Invariant, InvariantSeverity, InvariantResult
- `coordination/health.rs` – HealthReport, HealthStatus
- `coordination/transaction.rs` – StateTransaction, TransactionError
- `coordination/coordinator.rs` – StateCoordinator

**Aufwand:** ~4h gesamt

---

### Phase 4: Engines konsolidieren (Priorität: Mittel)

**Ziel:** Engines-Ordner erstellen, bestehende Engines verschieben.

#### 4.1 Ordner erstellen

```
core/engines/
├── mod.rs
├── trust.rs          # ← trust_engine.rs
├── event.rs          # ← event_engine.rs
├── consensus.rs      # ← consensus.rs
├── surprisal.rs      # ← surprisal.rs
├── world_formula.rs  # ← world_formula.rs
└── unified.rs        # ← engine.rs
```

#### 4.2 Engine-State-Brücke klären

**Problem:** TrustEngine und TrustState sind unabhängig.

**Lösung:**
- TrustEngine nutzt TrustState für Metriken
- Engine berechnet, State speichert Ergebnisse
- Observer-Pattern für Synchronisation

**Aufwand:** ~4h

---

### Phase 5: apply_state_event refaktorieren (Priorität: Hoch)

**Problem:** `apply_state_event()` ist ein riesiger Match-Block (~1000+ Zeilen).

**Lösung: Event-Handler-Pattern**

#### 5.1 Event-Handler-Trait

```rust
// state/event_handler.rs
pub trait StateEventHandler: Send + Sync {
    /// Welche Events kann dieser Handler verarbeiten?
    fn handles(&self, event: &StateEvent) -> bool;

    /// Event anwenden
    fn apply(&self, state: &UnifiedState, wrapped: &WrappedStateEvent);
}
```

#### 5.2 Handler pro Komponente

```rust
// state/handlers/core_handler.rs
pub struct CoreEventHandler;

impl StateEventHandler for CoreEventHandler {
    fn handles(&self, event: &StateEvent) -> bool {
        matches!(event,
            StateEvent::TrustUpdate { .. } |
            StateEvent::EventProcessed { .. } |
            StateEvent::FormulaComputed { .. } |
            StateEvent::ConsensusRoundCompleted { .. }
        )
    }

    fn apply(&self, state: &UnifiedState, wrapped: &WrappedStateEvent) {
        match &wrapped.event {
            StateEvent::TrustUpdate { entity_id, delta, reason, from_realm, .. } => {
                let positive = *delta >= 0.0;
                state.core.trust.update(positive, false);
                // ...
            }
            // ...
        }
    }
}
```

#### 5.3 Handler-Registry in UnifiedState

```rust
impl UnifiedState {
    fn register_handlers(&self) {
        self.handlers.push(Arc::new(CoreEventHandler));
        self.handlers.push(Arc::new(ExecutionEventHandler));
        self.handlers.push(Arc::new(ECLVMEventHandler));
        // ...
    }

    pub fn apply_state_event(&self, wrapped: &WrappedStateEvent) {
        for handler in &self.handlers {
            if handler.handles(&wrapped.event) {
                handler.apply(self, wrapped);
            }
        }
    }
}
```

**Aufwand:** ~8h

---

### Phase 6: Identity-State isolieren (Priorität: Mittel)

**Ziel:** Identity-State als eigenständiges Sub-Modul.

#### 6.1 Ordner erstellen

```
core/identity/
├── mod.rs
├── types.rs         # ← identity_types.rs
└── state.rs         # IdentityState aus state.rs extrahiert
```

#### 6.2 Identity-State-Brücke

- IdentityState nutzt identity_types.rs
- RealmMembership, WalletAddress, etc. zentral in identity/

**Aufwand:** ~4h

---

### Phase 7: mod.rs Re-exports minimieren (Priorität: Niedrig)

**Problem:** `mod.rs` exportiert ~200 Typen.

**Lösung:**
1. Gruppierte Re-exports über Sub-Module
2. Nutzer importieren spezifisch: `use crate::core::state::{UnifiedState, CoreState}`
3. Prelude-Pattern für häufige Typen

```rust
// core/mod.rs
pub mod state;
pub mod integration;
pub mod coordination;
pub mod engines;
pub mod identity;

// Prelude für häufige Typen
pub mod prelude {
    pub use super::state::{UnifiedState, SharedUnifiedState, StateEvent, UnifiedSnapshot};
    pub use super::integration::StateIntegrator;
    pub use super::coordination::StateCoordinator;
}
```

**Aufwand:** ~2h

---

## 4. Abhängigkeiten und Reihenfolge

```
Phase 1 (State aufteilen)        ← KRITISCH, sofort starten
    ↓
Phase 5 (apply_state_event)      ← Parallel zu Phase 2 möglich
    ↓
Phase 2 (Integration aufteilen)
    ↓
Phase 4 (Engines konsolidieren)
    ↓
Phase 3 (Coordination)           ← Niedrige Priorität
    ↓
Phase 6 (Identity isolieren)
    ↓
Phase 7 (mod.rs minimieren)
```

---

## 5. Migrations-Strategie

### 5.1 Backward-Compatibility

- **Alle bestehenden Exports beibehalten** in mod.rs
- Deprecated-Attribute für alte Pfade
- Schrittweise Migration der Imports im Codebase

```rust
// Temporär in state/mod.rs:
#[deprecated(note = "Use crate::core::state::core_state::TrustState instead")]
pub use core_state::TrustState;
```

### 5.2 Test-Strategie

1. **Vor jedem Schritt:** Alle Tests ausführen (`cargo test`)
2. **Nach jeder Datei-Extraktion:** Kompilieren + Tests
3. **Am Ende jeder Phase:** Integration-Tests
4. **Keine Funktionsänderungen** während Refactoring

### 5.3 Git-Strategie

- Feature-Branch: `refactor/core-module-split`
- Kleine, atomare Commits pro Datei-Extraktion
- Commit-Message-Format: `refactor(core): extract CircuitBreaker to state/circuit_breaker.rs`

---

## 6. Aufwandsschätzung

| Phase | Aufwand | Priorität |
|-------|---------|-----------|
| 1.1 Infrastructure-Types | ~12h | Hoch |
| 1.2 Event-System | ~4h | Hoch |
| 1.3 StateGraph | ~2h | Hoch |
| 1.4 Sub-States | ~27h | Hoch |
| 1.5 View/Handle/Context | ~4h | Hoch |
| 1.6 UnifiedState | ~6h | Hoch |
| 1.7 state/mod.rs | ~2h | Hoch |
| **Phase 1 Gesamt** | **~57h** | |
| Phase 2 (Integration) | ~24h | Mittel |
| Phase 3 (Coordination) | ~4h | Niedrig |
| Phase 4 (Engines) | ~4h | Mittel |
| Phase 5 (Event-Handler) | ~8h | Hoch |
| Phase 6 (Identity) | ~4h | Mittel |
| Phase 7 (mod.rs) | ~2h | Niedrig |
| **Gesamt** | **~103h** | |

---

## 7. Risiken und Mitigationen

| Risiko | Wahrscheinlichkeit | Impact | Mitigation |
|--------|-------------------|--------|------------|
| Breaking Changes in API | Mittel | Hoch | Re-exports beibehalten, Deprecation |
| Zirkuläre Abhängigkeiten | Mittel | Mittel | Sorgfältige Modul-Grenzen |
| Performance-Regression | Niedrig | Mittel | Benchmarks vor/nach |
| Merge-Konflikte | Hoch | Niedrig | Kleine Commits, häufig rebasen |

---

## 8. Abnahmekriterien

### Phase 1 abgeschlossen wenn:
- [ ] `state.rs` ist gelöscht oder leer (nur Re-exports)
- [ ] Alle State-Typen in eigenen Dateien unter `state/`
- [ ] Alle Tests grün
- [ ] Keine neuen Compiler-Warnungen

### Gesamt-Refactoring abgeschlossen wenn:
- [ ] Keine Datei im `core/`-Modul > 1.500 Zeilen
- [ ] Klare Modul-Grenzen (state, integration, coordination, engines, identity)
- [ ] Dokumentation aktualisiert
- [ ] Performance-Benchmarks unverändert (±5%)

---

## 9. Beziehung zu anderen Refactoring-Plänen

- **ECL-ECLVM-REFACTORING-PLAN.md**: Phase 4 dort baut auf Phase 1.5 hier auf (ECLVMStateContext/StateView)
- **CORE-DOMAIN-LOCAL-INTEGRATION.md**: Unberührt, betrifft local/-Modul

---

**Erstellt:** 2026-02-04
**Basiert auf:** STATE-RS-REFERENCE.md, state.rs Analyse
