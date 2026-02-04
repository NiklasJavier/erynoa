# 🏗️ Ziel-Architektur: Neue Verzeichnisstruktur

> **Teil von:** Projekt Pluto
> **Version:** 1.0.0

---

## 1. Neue Verzeichnisstruktur

```
backend/src/
│
├── lib.rs                          # Crate-Root (nur Re-exports)
├── main.rs                         # Entrypoint
│
├── 🧠 nervous_system/              # ZENTRALES STATE-MANAGEMENT
│   ├── mod.rs                      # Öffentliche API
│   ├── unified_state.rs            # UnifiedState (~2.000 Zeilen)
│   │
│   ├── event_sourcing/             # Event-Sourcing Subsystem
│   │   ├── mod.rs
│   │   ├── state_event.rs          # StateEvent enum (42 Varianten)
│   │   ├── wrapped_event.rs        # WrappedStateEvent + Kausalität
│   │   ├── event_log.rs            # StateEventLog (Ring-Buffer)
│   │   └── replay.rs               # Recovery + Checkpoint-Replay
│   │
│   ├── merkle/                     # Merkle-Verifizierung
│   │   ├── mod.rs
│   │   ├── tracker.rs              # MerkleStateTracker
│   │   ├── delta.rs                # MerkleDelta
│   │   └── proofs.rs               # State-Proofs für Light-Clients
│   │
│   ├── components/                 # StateComponent-States
│   │   ├── mod.rs
│   │   ├── core.rs                 # TrustState, EventState, FormulaState
│   │   ├── execution.rs            # ExecutionState, GasState
│   │   ├── protection.rs           # AnomalyState, DiversityState
│   │   ├── peer.rs                 # RealmState, GatewayState
│   │   ├── p2p.rs                  # SwarmState, GossipState
│   │   ├── identity.rs             # IdentityState
│   │   └── eclvm.rs                # ECLVMState
│   │
│   ├── graph/                      # StateGraph
│   │   ├── mod.rs
│   │   ├── components.rs           # StateComponent enum (37 Varianten)
│   │   ├── relations.rs            # StateRelation enum
│   │   └── analysis.rs             # Dependency/Trigger-Analyse
│   │
│   └── infrastructure/             # Infrastruktur
│       ├── mod.rs
│       ├── event_bus.rs            # EventBus (Ingress/Egress)
│       ├── broadcaster.rs          # StateBroadcaster (CQRS)
│       ├── circuit_breaker.rs      # CircuitBreaker
│       ├── multi_gas.rs            # MultiGas (L1-L4)
│       └── storage_handle.rs       # StorageHandle
│
├── 🔌 synapses/                    # OBSERVER-HUB & ADAPTER
│   ├── mod.rs
│   ├── traits.rs                   # Alle Observer-Traits
│   ├── hub.rs                      # SynapseHub (Event-Dispatch)
│   ├── integrator.rs               # StateIntegrator (Facade)
│   └── adapters/                   # Engine-Adapter
│       ├── mod.rs
│       ├── trust.rs
│       ├── event.rs
│       ├── consensus.rs
│       ├── formula.rs
│       ├── eclvm.rs
│       ├── realm.rs
│       └── p2p.rs
│
├── 🆔 identity/                    # IDENTITY-LAYER (Κ6-Κ8)
│   ├── mod.rs
│   ├── types.rs                    # DID, DIDDocument, Delegation
│   ├── state.rs                    # IdentityState
│   ├── resolver.rs                 # IdentityResolver
│   ├── key_store.rs                # SecureKeyStore
│   ├── passkey.rs                  # PasskeyManager
│   └── wallet.rs                   # WalletAddress + CAIP-10
│
├── ⚙️ engines/                     # CORE-ENGINES (Κ2-Κ18)
│   ├── mod.rs
│   ├── trust.rs                    # TrustEngine (Κ2-Κ5)
│   ├── event.rs                    # EventEngine (Κ9-Κ12)
│   ├── formula.rs                  # WorldFormulaEngine (Κ15)
│   ├── consensus.rs                # ConsensusEngine (Κ18)
│   └── surprisal.rs                # SurprisalCalculator (Κ15a)
│
├── 💰 execution/                   # EXECUTION-LAYER
│   ├── mod.rs
│   ├── context.rs                  # ExecutionContext
│   ├── gas/
│   │   ├── mod.rs
│   │   ├── metering.rs
│   │   └── pricing.rs              # Congestion Pricing
│   ├── mana/
│   │   ├── mod.rs
│   │   └── regeneration.rs
│   └── tracked.rs                  # TrackedValue (IPS)
│
├── 🌐 realm/                       # REALM-LAYER (Κ22-Κ24)
│   ├── mod.rs
│   ├── state.rs                    # RealmSpecificState
│   ├── sharding/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   └── lazy_state.rs           # LazyShardedRealmState
│   ├── quota/
│   │   ├── mod.rs
│   │   └── enforcer.rs             # RealmQuota
│   ├── gateway/                    # Κ23
│   │   ├── mod.rs
│   │   ├── guard.rs                # GatewayGuard
│   │   └── policy.rs
│   └── saga/                       # Κ24
│       ├── mod.rs
│       ├── composer.rs             # SagaComposer
│       └── compensation.rs
│
├── 🛡️ protection/                  # PROTECTION-LAYER (Κ19-Κ21)
│   ├── mod.rs
│   ├── anomaly/
│   │   ├── mod.rs
│   │   └── detector.rs
│   ├── diversity/
│   │   ├── mod.rs
│   │   └── gini.rs                 # Κ19
│   ├── quadratic/
│   │   ├── mod.rs
│   │   └── voting.rs               # Κ20
│   ├── anti_calcification/
│   │   ├── mod.rs
│   │   └── decay.rs                # Κ21
│   └── calibration/
│       ├── mod.rs
│       └── adaptive.rs
│
├── 🔗 p2p/                         # P2P-LAYER (konsolidiert)
│   ├── mod.rs
│   ├── swarm/
│   │   ├── mod.rs
│   │   └── manager.rs
│   ├── gossip/
│   │   ├── mod.rs
│   │   └── handler.rs
│   ├── dht/
│   │   ├── mod.rs
│   │   └── resolver.rs
│   ├── relay/
│   │   └── mod.rs
│   ├── privacy/
│   │   ├── mod.rs
│   │   └── onion.rs
│   └── trust_gate/
│       └── mod.rs
│
├── 🎛️ eclvm/                       # ECLVM + WASM
│   ├── mod.rs
│   ├── ast.rs                      # Unverändert
│   ├── parser.rs                   # Unverändert
│   ├── compiler.rs                 # Erweitert: + WASM-Backend
│   ├── bytecode.rs                 # Legacy (Hybrid-Mode)
│   ├── runtime/                    # Legacy Runtime
│   │   ├── vm.rs
│   │   ├── host.rs
│   │   └── runner.rs              # Dual-Mode (Legacy/WASM)
│   │
│   └── 🆕 wasm/                    # NEU: WASM-Subsystem (Phase 5)
│       ├── mod.rs
│       ├── engine.rs              # WasmPolicyEngine
│       ├── codegen/               # ECL → WASM Compiler
│       │   ├── mod.rs
│       │   ├── compiler.rs
│       │   └── opcodes.rs
│       ├── host/                  # Host-Functions
│       │   ├── mod.rs
│       │   ├── trust.rs
│       │   ├── identity.rs
│       │   ├── state.rs
│       │   ├── budget.rs
│       │   └── bridge.rs          # WasmStateBridge
│       ├── runtime/               # Wasmtime Integration
│       │   ├── mod.rs
│       │   ├── store.rs
│       │   └── fuel.rs
│       └── types/                 # WIT-Typen
│           └── mod.rs
│
├── 📦 storage/                     # STORAGE-LAYER
│   ├── mod.rs
│   ├── decentralized.rs            # DecentralizedStorage
│   ├── kv/
│   │   └── mod.rs
│   ├── event_store/
│   │   └── mod.rs
│   ├── identity_store/
│   │   └── mod.rs
│   ├── trust_store/
│   │   └── mod.rs
│   ├── content_store/
│   │   └── mod.rs
│   ├── archive/
│   │   └── mod.rs
│   ├── realm/
│   │   └── mod.rs
│   ├── blueprint/
│   │   ├── mod.rs
│   │   └── marketplace.rs
│   └── metrics/
│       └── mod.rs
│
├── 🌐 api/                         # API-LAYER
│   └── ... (wie bisher)
│
├── 📊 domain/                      # DOMAIN-TYPEN
│   ├── mod.rs
│   └── unified/
│       ├── mod.rs
│       ├── primitives.rs
│       ├── cost.rs
│       ├── trust.rs
│       ├── identity.rs
│       ├── event.rs
│       ├── realm.rs
│       ├── saga.rs
│       ├── component.rs            # StateComponent, StateRelation
│       └── error.rs                # Unified Error Types
│
├── ⚙️ config/                      # KONFIGURATION
│   └── ... (wie bisher)
│
└── 🔧 core/                        # LEGACY (nur Re-Exports)
    └── mod.rs                      # Deprecated Re-Exports
```

---

## 2. Modul-Verantwortlichkeiten

| Modul | Verantwortung | Größe (Ziel) |
|-------|---------------|--------------|
| `nervous_system/` | Zentraler State, Event-Sourcing, Merkle | ~8.000 Zeilen |
| `synapses/` | Observer-Hub, Adapter | ~2.000 Zeilen |
| `identity/` | DID-Management, Keys, Credentials | ~1.500 Zeilen |
| `engines/` | Trust, Event, Formula, Consensus | ~3.000 Zeilen |
| `execution/` | Gas, Mana, ExecutionContext | ~1.000 Zeilen |
| `realm/` | Realm-State, Gateway, Saga | ~2.500 Zeilen |
| `protection/` | Anomaly, Diversity, Calibration | ~2.000 Zeilen |
| `p2p/` | libp2p Integration | ~3.000 Zeilen |
| `storage/` | Fjall-Stores, Archive | ~4.000 Zeilen |
| `domain/` | Typen, Traits, Errors | ~2.000 Zeilen |

---

## 3. Trait-Hierarchie

```rust
// domain/unified/traits.rs

/// Basis-Trait für alle State-Layer
pub trait StateLayer: Send + Sync + 'static {
    type Snapshot: Clone + Serialize + DeserializeOwned;

    fn snapshot(&self) -> Self::Snapshot;
    fn health_score(&self) -> f64;
    fn apply_event(&self, event: &WrappedStateEvent);
    fn component(&self) -> StateComponent;
}

/// Observer-Trait für Event-Dispatch
pub trait StateObserver: Send + Sync + 'static {
    fn on_event(&self, event: &WrappedStateEvent);
    fn target_component(&self) -> StateComponent;
    fn priority(&self) -> ObserverPriority;
}

/// Metriken-Trait
pub trait Metered {
    fn metrics(&self) -> &StoreMetrics;
    fn health_score(&self) -> f64;
}

/// Resettable für Tests
pub trait Resettable {
    fn reset(&self);
}
```

---

## 4. Migrationspfad

### Phase 1: Strukturen erstellen
```bash
# Neue Verzeichnisse
mkdir -p src/{nervous_system,synapses,identity,engines,realm,storage}
mkdir -p src/nervous_system/{event_sourcing,merkle,components,graph,infrastructure}
mkdir -p src/synapses/adapters
```

### Phase 2: Code extrahieren
```
state.rs → nervous_system/unified_state.rs
state.rs (Zeilen 800-1900) → event_sourcing/state_event.rs
state.rs (Zeilen 2500-3000) → merkle/tracker.rs
state_integration.rs → synapses/
```

### Phase 3: Re-Exports
```rust
// core/mod.rs
#[deprecated(since = "0.5.0", note = "Use nervous_system instead")]
pub use crate::nervous_system::*;
```
