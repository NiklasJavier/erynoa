# 🧠 State.rs Kerngedanken: Synthetische Pluto-Integration

> **Teil von:** Projekt Pluto
> **Quelle:** `backend/src/core/state.rs` (21.495 Zeilen, 823 KB)
> **Status:** Tiefenanalyse abgeschlossen

---

## 1. Die 9 Design-Prinzipien

Aus den Zeilen 1-11 der state.rs – das Fundament für Pluto:

```text
1. Hierarchische Komposition    → State-Layer bauen aufeinander auf
2. Thread-Safety               → Atomic Counters + RwLock-Strukturen
3. Dependency Injection        → Jeder Layer kennt Abhängigkeiten
4. Event-Driven Updates        → Observer-Pattern für Propagation
5. Snapshot-Isolation          → Konsistente Reads ohne Locking
6. Per-Realm Isolation         → TrustVector, Rules, Metrics pro Realm
7. Event-Inversion             → P2P/Core Entkopplung via Queues
8. Circuit Breaker             → Automatische Degradation
9. CQRS Light                  → Broadcast-Channels für State-Deltas
```

### Implikation für Pluto

| Prinzip | Pluto-Umsetzung |
|---------|-----------------|
| Hierarchische Komposition | `nervous_system/layers/` mit Core → Execution → Peer → Engine |
| Thread-Safety | Beibehalten: `AtomicU64`, `DashMap`, `RwLock` |
| Dependency Injection | `SynapseHub` injiziert State in alle Module |
| Event-Driven | `StateEvent` bleibt zentral, SynapseHub dispatcht |
| Snapshot-Isolation | `UnifiedState.snapshot()` ohne Locks |
| Per-Realm Isolation | `RealmState` mit eigenen TrustVectors |
| Event-Inversion | `EventBus` bleibt für P2P/Core Entkopplung |
| Circuit Breaker | `CircuitBreaker` in `nervous_system/protection/` |
| CQRS | `StateBroadcaster` für Subscriber-Updates |

---

## 2. Kern-Strukturen aus state.rs

### 2.1 EventBus (Zeilen 245-400)

```rust
pub struct EventBus {
    // Ingress: P2P → Core (empfangene Events)
    pub ingress_tx: mpsc::Sender<NetworkEvent>,
    pub ingress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,

    // Egress: Core → P2P (zu sendende Events)
    pub egress_tx: mpsc::Sender<NetworkEvent>,
    pub egress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,

    // Priority Queue für Consensus/Trust-Critical
    pub priority_ingress_tx: mpsc::Sender<NetworkEvent>,

    // Metriken
    pub ingress_count: AtomicU64,
    pub egress_count: AtomicU64,
    pub dropped_count: AtomicU64,
}
```

**Pluto-Integration:**
```text
nervous_system/
├── event_bus/
│   ├── mod.rs          # EventBus unverändert
│   ├── channels.rs     # Ingress/Egress/Priority
│   └── metrics.rs      # EventBusSnapshot
```

### 2.2 StateDelta + StateBroadcaster (Zeilen 400-527)

```rust
pub struct StateDelta {
    pub sequence: u64,
    pub component: StateComponent,
    pub delta_type: DeltaType,
    pub data: Vec<u8>,
    pub timestamp_ms: u64,
    pub realm_id: Option<String>,
}

pub struct StateBroadcaster {
    sender: broadcast::Sender<StateDelta>,
    sequence: AtomicU64,
    pub deltas_sent: AtomicU64,
    pub subscriber_count: AtomicU64,
}
```

**Pluto-Integration:**
```text
nervous_system/
├── broadcast/
│   ├── mod.rs           # StateBroadcaster
│   ├── delta.rs         # StateDelta, DeltaType
│   └── subscribers.rs   # Subscriber-Management
```

### 2.3 CircuitBreaker (Zeilen 603-749)

```rust
pub struct CircuitBreaker {
    mode: AtomicU8,                    // SystemMode
    critical_window: RwLock<Vec<u64>>, // Anomalien pro Minute
    pub degraded_threshold: AtomicU64,
    pub emergency_threshold: AtomicU64,
    pub gini_threshold: RwLock<f64>,   // Anti-Calcification Κ19
}
```

**Pluto-Integration:**
```text
nervous_system/
├── protection/
│   ├── circuit_breaker.rs   # CircuitBreaker
│   ├── system_mode.rs       # SystemMode enum
│   └── thresholds.rs        # Konfigurierbare Schwellwerte
```

### 2.4 StateEvent (Zeilen 800-1768)

**42 Event-Varianten**, gruppiert in 8 Kategorien:

| Kategorie | Events | Zeilen |
|-----------|--------|--------|
| Core State | TrustUpdate, EventProcessed, FormulaComputed, ConsensusRound | 812-879 |
| Execution + ECLVM | ExecutionStarted/Completed, PolicyEvaluated, BlueprintAction, SagaProgress | 881-958 |
| Protection | AnomalyDetected, DiversityMetricUpdate, CalibrationApplied, SystemModeChanged | 960-1011 |
| Peer + Realm | RealmLifecycle, MembershipChange, CrossingEvaluated | 1013-1060 |
| P2P Network | NetworkMetricUpdate, PeerConnectionChange, TrustUpdated, PeerBanned | 1062-1133 |
| Privacy Layer | CircuitCreated/Closed, MessageSent, CoverTraffic, MixingPool, RelaySelection | 1134-1219 |
| Recovery + Reorg | CheckpointCreated, RecoveryCompleted, ReorgDetected | 1221-1256 |
| Identity (Κ6-Κ8) | IdentityBootstrapped, SubDIDDerived, Delegation, Credential, KeyRotated | 1330-1495 |

**Pluto-Integration:**
```text
nervous_system/
├── events/
│   ├── mod.rs              # StateEvent enum
│   ├── core.rs             # Core State Events
│   ├── execution.rs        # Execution + ECLVM
│   ├── protection.rs       # Protection Events
│   ├── realm.rs            # Peer + Realm Events
│   ├── network.rs          # P2P + Privacy Events
│   ├── recovery.rs         # Recovery Events
│   └── identity.rs         # Identity Events (Κ6-Κ8)
```

### 2.5 StateEventEmitter Trait (Zeilen 1770-1906)

```rust
pub trait StateEventEmitter: Send + Sync {
    fn emit(&self, event: StateEvent);
    fn emit_batch(&self, events: Vec<StateEvent>);
    fn is_active(&self) -> bool;
}

// Implementierungen:
// - NoOpEmitter (Tests)
// - ChannelEmitter (mpsc::UnboundedSender)
// - CallbackEmitter (Callbacks)
```

**Pluto-Integration:**
```text
nervous_system/
├── emitters/
│   ├── mod.rs              # StateEventEmitter trait
│   ├── noop.rs             # NoOpEmitter
│   ├── channel.rs          # ChannelEmitter
│   └── callback.rs         # CallbackEmitter
```

### 2.6 WrappedStateEvent + StateEventLog (Zeilen 1907-2200)

```rust
pub struct WrappedStateEvent {
    pub id: String,              // Blake3-Hash
    pub timestamp_ms: u128,
    pub parent_ids: Vec<String>, // Kausalität
    pub component: StateComponent,
    pub sequence: u64,
    pub event: StateEvent,
    pub signature: Option<Vec<u8>>,
}

pub struct StateEventLog {
    sequence: AtomicU64,
    buffer: RwLock<Vec<WrappedStateEvent>>,  // Ring-Buffer 10.000
    last_checkpoint_id: RwLock<Option<String>>,
    checkpoint_interval: u64,                 // 5.000 Events
}
```

**Pluto-Integration:**
```text
nervous_system/
├── log/
│   ├── mod.rs              # StateEventLog
│   ├── wrapped.rs          # WrappedStateEvent
│   ├── buffer.rs           # Ring-Buffer
│   └── checkpoint.rs       # Checkpoint-Management
```

### 2.7 MerkleStateTracker (Zeilen 2203-2420)

```rust
pub struct MerkleStateTracker {
    root_hash: RwLock<MerkleHash>,
    component_hashes: RwLock<HashMap<StateComponent, MerkleHash>>,
    delta_history: RwLock<Vec<MerkleDelta>>,
}

pub struct MerkleDelta {
    pub old_root: MerkleHash,
    pub new_root: MerkleHash,
    pub component: StateComponent,
    pub proof_path: Vec<MerkleHash>,
    pub data: Vec<u8>,
}
```

**Pluto-Integration:**
```text
nervous_system/
├── merkle/
│   ├── mod.rs              # MerkleStateTracker
│   ├── hash.rs             # MerkleHash, MerkleNode
│   ├── delta.rs            # MerkleDelta
│   └── proof.rs            # Proof-Verifikation
```

---

## 3. StateGraph: Das Beziehungsnetz

### 3.1 Die 110+ Relationen (Zeilen 4080-4450)

Der `StateGraph::erynoa_graph()` definiert **110+ Kanten** zwischen **40 StateComponents**:

```rust
pub struct StateGraph {
    pub edges: Vec<(StateComponent, StateRelation, StateComponent)>,
}

pub enum StateRelation {
    DependsOn,    // A hängt von B ab (B vor A initialisiert)
    Triggers,     // A löst Updates in B aus
    Aggregates,   // A enthält/aggregiert B
    Validates,    // A validiert B
    Bidirectional,// Bidirektionale Abhängigkeit
}
```

### 3.2 Ausgewählte Schlüssel-Relationen

```text
IDENTITY-LAYER (Κ6-Κ8)
├── Trust ──DependsOn──▶ Identity        # Trust basiert auf Identity
├── Identity ──Triggers──▶ Trust         # Neue IDs erhalten Initial-Trust
├── Event ──DependsOn──▶ Identity        # Events müssen signiert sein
├── Realm ──DependsOn──▶ Identity        # Membership basiert auf Identity
├── Gateway ──Validates──▶ Identity      # Crossing erfordert Verifikation
└── Swarm ──Validates──▶ Identity        # Peer-Authentifizierung

CORE-LAYER (Κ2-Κ18)
├── Trust ──Triggers──▶ Event            # Trust-Updates erzeugen Events
├── Event ──Triggers──▶ Trust            # Events beeinflussen Trust
├── Trust ──DependsOn──▶ WorldFormula    # Trust fließt in 𝔼
├── WorldFormula ──Triggers──▶ Consensus # 𝔼 beeinflusst Konsens
└── Consensus ──Validates──▶ Event       # Konsens validiert Events

EXECUTION-LAYER
├── Gas ──DependsOn──▶ Trust             # Gas-Budget basiert auf Trust
├── Mana ──DependsOn──▶ Trust            # Mana basiert auf Trust
├── Execution ──Aggregates──▶ Gas        # Execution trackt Gas
├── Execution ──Aggregates──▶ Mana       # Execution trackt Mana
└── Execution ──Triggers──▶ Event        # Execution emittiert Events

ECLVM-LAYER
├── ECLVM ──DependsOn──▶ Gas             # Compute verbraucht Gas
├── ECLVM ──DependsOn──▶ Mana            # Events verbrauchen Mana
├── ECLVM ──Triggers──▶ Event            # ECL-Ausführung emittiert Events
├── ECLPolicy ──Validates──▶ Gateway     # Policies validieren Crossings
└── ECLPolicy ──Validates──▶ Realm       # Policies definieren Realm-Regeln

REALM-LAYER (Κ22-Κ24)
├── Realm ──Aggregates──▶ Gateway        # Realm trackt Crossings
├── Realm ──Triggers──▶ SagaComposer     # Cross-Realm-Sagas
├── Gateway ──DependsOn──▶ ECLPolicy     # Gateway führt Policies aus
└── SagaComposer ──DependsOn──▶ ECLVM    # Sagas via ECLVM orchestriert

P2P-LAYER
├── Swarm ──Triggers──▶ Event            # Swarm propagiert Events
├── Gossip ──DependsOn──▶ Trust          # Gossip-Scoring nutzt Trust
├── Privacy ──DependsOn──▶ Trust         # Privacy-Level basiert auf Trust
└── Privacy ──Validates──▶ Gossip        # Privacy validiert Routing
```

### 3.3 Graph-Traversal-Methoden

```rust
impl StateGraph {
    fn dependents(&self, comp) -> Vec<StateComponent>;      // Wer hängt von mir ab?
    fn triggered_by(&self, comp) -> Vec<StateComponent>;    // Wen trigger ich?
    fn aggregated_by(&self, comp) -> Vec<StateComponent>;   // Was aggregiere ich?
    fn validators_of(&self, comp) -> Vec<StateComponent>;   // Wer validiert mich?
    fn transitive_dependencies(&self, comp) -> HashSet;     // Alle rekursiven Deps
    fn transitive_triggers(&self, comp) -> HashSet;         // Alle transitiven Trigger
    fn validation_chain(&self, comp) -> Vec;                // Validierungs-Kette
    fn criticality_score(&self, comp) -> usize;             // Wie kritisch bin ich?
}
```

**Pluto-Integration:**
```text
nervous_system/
├── graph/
│   ├── mod.rs              # StateGraph
│   ├── relations.rs        # StateRelation enum
│   ├── traversal.rs        # Traversal-Methoden
│   └── erynoa_graph.rs     # erynoa_graph() Definition
```

---

## 4. Sub-States im Detail

### 4.1 TrustState (Zeilen 4454-4790)

```rust
pub struct TrustState {
    // Atomic Counters
    pub entities_count: AtomicUsize,
    pub relationships_count: AtomicUsize,
    pub updates_total: AtomicU64,
    pub positive_updates: AtomicU64,
    pub negative_updates: AtomicU64,
    pub violations_count: AtomicU64,

    // Complex State
    pub avg_trust: RwLock<f64>,
    pub trust_distribution: RwLock<TrustDistribution>,

    // Identity-Integration (Phase 7)
    pub trust_by_id: RwLock<HashMap<UniversalId, TrustEntry>>,

    // Relationship-Tracking
    pub triggered_events: AtomicU64,
    pub event_triggered_updates: AtomicU64,
    pub realm_triggered_updates: AtomicU64,
}
```

**Schlüssel-Features:**
- `asymmetry_ratio()`: Κ4 erfordert ~2:1 Verhältnis neg:pos
- `TrustEntry` mit globalem + per-Realm Trust
- `apply_global_decay()` für Κ8 Trust-Decay

### 4.2 IdentityState (Zeilen 3800-4072)

```rust
pub struct IdentityState {
    // Bootstrap-Status
    pub bootstrap_completed: AtomicBool,
    pub root_did: RwLock<Option<UniversalId>>,
    pub mode: RwLock<IdentityMode>,

    // DID-Tracking
    pub sub_dids: DashMap<UniversalId, SubDIDInfo>,
    pub addresses: DashMap<String, WalletInfo>,

    // Delegation-Tracking (Κ8)
    pub delegations: DashMap<UniversalId, DelegationInfo>,
    pub active_delegations: AtomicU64,
    pub revoked_delegations: AtomicU64,

    // Credential-Tracking
    pub credentials_issued: AtomicU64,
    pub credentials_verified: AtomicU64,
}
```

### 4.3 Alle 40 StateComponents

| Layer | Components |
|-------|------------|
| **Core** | Trust, Event, WorldFormula, Consensus |
| **Execution** | Gas, Mana, Execution |
| **Engine** | ECLVM, ECLPolicy, ECLBlueprint |
| **Protection** | Anomaly, Diversity, Quadratic, AntiCalcification, Calibration |
| **Peer** | Realm, Gateway, SagaComposer, IntentParser, Room, Partition |
| **P2P** | Swarm, Gossip, Kademlia, Relay, NatTraversal, Privacy |
| **Storage** | EventStore, Archive, KvStore, Blueprint |
| **Identity** | Identity, Credential, KeyManagement |
| **Controller** | Controller |
| **UI/API** | UI, DataLogic, API, Governance, BlueprintComposer |

---

## 5. Pluto-Synthese: Nervensystem-Mapping

### 5.1 Verzeichnis-Struktur

```text
backend/src/
├── nervous_system/                 # NEUER ORDNER
│   ├── mod.rs                      # Öffentliche API
│   │
│   ├── state/                      # UnifiedState (aufgeteilt)
│   │   ├── mod.rs                  # UnifiedState Facade
│   │   ├── core.rs                 # TrustState, EventState, FormulaState
│   │   ├── execution.rs            # GasState, ManaState, ExecutionState
│   │   ├── protection.rs           # AnomalyState, DiversityState, etc.
│   │   ├── peer.rs                 # RealmState, GatewayState, SagaState
│   │   ├── network.rs              # SwarmState, GossipState, PrivacyState
│   │   ├── storage.rs              # EventStoreState, ArchiveState
│   │   ├── identity.rs             # IdentityState, CredentialState
│   │   └── engine.rs               # UIState, DataLogicState, etc.
│   │
│   ├── events/                     # StateEvent (aufgeteilt)
│   │   ├── mod.rs                  # StateEvent enum
│   │   ├── core.rs                 # TrustUpdate, EventProcessed, etc.
│   │   ├── execution.rs            # ExecutionStarted, PolicyEvaluated
│   │   ├── protection.rs           # AnomalyDetected, CalibrationApplied
│   │   ├── realm.rs                # RealmLifecycle, MembershipChange
│   │   ├── network.rs              # PeerConnectionChange, PrivacyCircuit
│   │   ├── identity.rs             # IdentityBootstrapped, Delegation
│   │   └── traits.rs               # StateEvent-Methoden
│   │
│   ├── graph/                      # StateGraph
│   │   ├── mod.rs                  # StateGraph struct
│   │   ├── component.rs            # StateComponent enum (von domain)
│   │   ├── relation.rs             # StateRelation enum
│   │   ├── erynoa_graph.rs         # Die 110+ Kanten
│   │   └── traversal.rs            # Traversal-Algorithmen
│   │
│   ├── synapse/                    # SynapseHub (NEU)
│   │   ├── mod.rs                  # SynapseHub
│   │   ├── hub.rs                  # Dispatch-Logik
│   │   ├── observer.rs             # StateObserver trait
│   │   └── adapters.rs             # Engine-Adapter
│   │
│   ├── emitters/                   # StateEventEmitter
│   │   ├── mod.rs                  # Trait
│   │   ├── noop.rs
│   │   ├── channel.rs
│   │   └── callback.rs
│   │
│   ├── log/                        # StateEventLog
│   │   ├── mod.rs
│   │   ├── wrapped.rs              # WrappedStateEvent
│   │   ├── buffer.rs               # Ring-Buffer
│   │   └── checkpoint.rs
│   │
│   ├── merkle/                     # MerkleStateTracker
│   │   ├── mod.rs
│   │   ├── hash.rs
│   │   ├── delta.rs
│   │   └── proof.rs
│   │
│   ├── bus/                        # EventBus (P2P/Core Entkopplung)
│   │   ├── mod.rs
│   │   ├── channels.rs
│   │   └── metrics.rs
│   │
│   ├── broadcast/                  # StateBroadcaster (CQRS)
│   │   ├── mod.rs
│   │   ├── delta.rs
│   │   └── subscribers.rs
│   │
│   └── protection/                 # CircuitBreaker
│       ├── mod.rs
│       ├── circuit_breaker.rs
│       ├── system_mode.rs
│       └── thresholds.rs
```

### 5.2 Migration-Schritte

| Phase | Woche | Aktion |
|-------|-------|--------|
| 2.1 | 3 | `nervous_system/` Ordner + `mod.rs` erstellen |
| 2.2 | 3 | `events/` aus StateEvent extrahieren |
| 2.3 | 3 | `graph/` aus StateGraph extrahieren |
| 2.4 | 4 | `state/core.rs` (TrustState, EventState) |
| 2.5 | 4 | `state/execution.rs` (GasState, ManaState) |
| 2.6 | 4 | `state/protection.rs` (AnomalyState, etc.) |
| 2.7 | 5 | `state/peer.rs` + `state/network.rs` |
| 2.8 | 5 | `state/identity.rs` + `state/engine.rs` |
| 3.1 | 6 | `synapse/hub.rs` implementieren |
| 3.2 | 6 | Adapter für alle Engines |
| 3.3 | 7 | Integration-Tests |

---

## 6. Invarianten-Checkliste (Κ1-Κ24)

Diese Invarianten aus domain/unified müssen bei der Migration gewahrt bleiben:

| Invariante | Beschreibung | state.rs Implementierung |
|------------|--------------|-------------------------|
| **Κ2** | Trust-Wertebereich [0, 1] | `TrustEntry.global_trust.clamp(0.0, 1.0)` |
| **Κ4** | Asymmetrische Updates | `TrustState.asymmetry_ratio()` |
| **Κ6** | DID-Format did:erynoa:* | `IdentityState.root_did` |
| **Κ8** | Delegation Trust-Decay | `TrustEntry.decay_factor` |
| **Κ9** | Event-Kausalität | `WrappedStateEvent.parent_ids` |
| **Κ11** | Monotone Gas-Exhaustion | `ExecutionState.gas_consumed` |
| **Κ13** | Positive Mana-Regeneration | ManaState |
| **Κ19** | Gini-Threshold | `CircuitBreaker.gini_threshold` |
| **Κ22** | Realm-Rule-Inheritance | `RealmState.parent_realm` |
| **Κ23** | Realm-Crossing-Policy | `GatewayState` + ECLPolicy |
| **Κ24** | Saga-Atomicity | `SagaState.compensation_triggered` |

---

## 7. Kritikalitäts-Matrix

Basierend auf `StateGraph.criticality_score()`:

| Component | Dependents | Triggers | Score | Priorität |
|-----------|------------|----------|-------|-----------|
| **Identity** | 18 | 6 | 24 | 🔴 P0 |
| **Trust** | 15 | 5 | 20 | 🔴 P0 |
| **Event** | 10 | 6 | 16 | 🔴 P0 |
| **ECLVM** | 8 | 4 | 12 | 🟡 P1 |
| **Gateway** | 6 | 3 | 9 | 🟡 P1 |
| **Realm** | 5 | 4 | 9 | 🟡 P1 |
| **Gas** | 8 | 0 | 8 | 🟢 P2 |
| **Swarm** | 4 | 3 | 7 | 🟢 P2 |
| **Privacy** | 2 | 1 | 3 | 🟢 P2 |

---

## 8. Nächste Schritte

1. **Phasenplan aktualisieren** mit diesen Details
2. **Ziel-Architektur erweitern** um `nervous_system/` Struktur
3. **Migration-Scripts anpassen** für state.rs Aufspaltung
4. **Unit-Tests** für StateGraph-Traversals erstellen
