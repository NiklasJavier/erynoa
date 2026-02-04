# State.rs Referenz-Dokumentation

## Überblick

`state.rs` ist das zentrale State-Management-Modul des Erynoa-Backends. Es definiert eine **hierarchische, thread-safe State-Architektur** für alle Module des Systems.

---

## 1. Architektur-Übersicht

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              UNIFIED STATE                                       │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                          CoreState (Κ2-Κ18)                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │   │
│  │  │ TrustState   │──│ EventState   │──│ FormulaState │──│ Consensus  │  │   │
│  │  │  (Κ2-Κ5)     │  │  (Κ9-Κ12)    │  │  (Κ15b-d)    │  │   (Κ18)    │  │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘  │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                        ExecutionState (IPS ℳ)                         │     │
│  │  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐        │     │
│  │  │  GasTracker    │───│  ManaTracker   │───│  EventEmitter  │        │     │
│  │  └────────────────┘   └────────────────┘   └────────────────┘        │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                       ProtectionState (Κ19-Κ21)                        │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
│  │  │  Anomaly     │──│  Diversity   │──│  Quadratic   │──│AntiCalc  │  │     │
│  │  │  Detection   │  │  Monitor     │  │  Governance  │  │  (Κ19)   │  │     │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────┘  │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                        StorageState (Local)                           │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
│  │  │  KV Store    │──│  Event Store │──│   Archive    │──│Blueprint │  │     │
│  │  │              │  │   (DAG)      │  │  (ψ_archive) │  │Marketpl. │  │     │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────┘  │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                         PeerState (Κ22-Κ24)                            │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
│  │  │   Gateway    │──│ SagaComposer │──│ IntentParser │──│ Realm    │  │     │
│  │  │   (Κ23)      │  │  (Κ22/Κ24)   │  │              │  │  State   │  │     │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────┘  │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Design-Prinzipien

### 2.1 Kern-Prinzipien

| Prinzip | Beschreibung |
|---------|--------------|
| **Hierarchische Komposition** | State-Layer bauen aufeinander auf |
| **Thread-Safety** | Alle Counter atomar, komplexe Strukturen unter `RwLock` |
| **Dependency Injection** | Jeder Layer kennt seine Abhängigkeiten |
| **Event-Driven Updates** | Änderungen propagieren durch Observer-Pattern |
| **Snapshot-Isolation** | Konsistente Reads ohne Locking |
| **Per-Realm Isolation** | Jedes Realm hat eigenen TrustVector, Rules und Metrics |
| **Event-Inversion** | P2P/Core Entkopplung durch Ingress/Egress-Queues |
| **Circuit Breaker** | Automatische Degradation bei kritischen Anomalien |
| **CQRS light** | Broadcast-Channels für State-Deltas an Subscriber |

### 2.2 Thread-Safety Strategie

- **`AtomicU64`, `AtomicBool`**: Für einfache Counter und Flags
- **`RwLock<T>`**: Für komplexe Strukturen mit mehr Reads als Writes
- **`DashMap<K, V>`**: Für sharded, concurrent HashMaps
- **`tokio::sync::RwLock`**: Für async-kompatible Locks

---

## 3. Hauptkomponenten

### 3.1 UnifiedState (Wurzel)

```rust
pub struct UnifiedState {
    pub started_at: Instant,

    // Layer in hierarchischer Reihenfolge:
    pub identity: IdentityState,      // Κ6-Κ8 DID Management
    pub core: CoreState,               // Κ2-Κ18 Trust/Event/Formula
    pub execution: ExecutionState,     // IPS ℳ Gas/Mana
    pub eclvm: ECLVMState,            // ECLVM Policies/Blueprints
    pub protection: ProtectionState,   // Κ19-Κ21 Anomaly/Diversity
    pub storage: StorageState,         // KV/Event/Archive
    pub peer: PeerState,               // Κ22-Κ24 Gateway/Saga
    pub p2p: P2PState,                 // Network Layer

    // Engines
    pub ui: UIState,
    pub api: APIState,
    pub governance: GovernanceState,
    pub controller: ControllerState,
    // ...
}
```

### 3.2 IdentityState (Κ6-Κ8)

Verwaltet dezentrale Identitäten:

- **Root-DID**: Haupt-Identität des Nodes
- **Sub-DIDs**: Device, Agent, Realm-spezifische Ableitungen
- **Delegationen**: Κ8 Trust-Decay bei Delegation
- **Realm-Memberships**: Zugehörigkeit zu Realms
- **Wallet-Adressen**: Multi-Chain Adressen

```rust
pub struct IdentityState {
    pub root_did: RwLock<Option<DID>>,
    pub root_document: RwLock<Option<DIDDocument>>,
    pub mode: AtomicU8,  // Interactive, AgentManaged, Ephemeral, Test
    pub sub_dids: RwLock<HashMap<String, Vec<DID>>>,
    pub delegations: RwLock<HashMap<UniversalId, Delegation>>,
    pub realm_memberships: RwLock<HashMap<UniversalId, RealmMembership>>,
    pub wallets: RwLock<HashMap<String, Vec<WalletAddress>>>,
    // Metrics...
}
```

### 3.3 CoreState (Κ2-Κ18)

Kernlogik des Systems:

- **TrustState**: 6D Trust-Vektoren (Κ2-Κ5)
- **EventState**: Kausaler Event-DAG (Κ9-Κ12)
- **FormulaState**: Weltformel-Berechnung (Κ15)
- **ConsensusState**: Distributed Consensus (Κ18)

### 3.4 ExecutionState (IPS ℳ)

Kosten-Tracking und Ressourcen-Management mit drei Sub-States:

```rust
pub struct ExecutionState {
    /// Gas-Verbrauch und Limits
    pub gas: GasState,
    /// Mana-Verbrauch und Regeneration
    pub mana: ManaState,
    /// Execution-Contexts und Metriken
    pub executions: ExecutionsState,
}

pub struct GasState {
    pub consumed: AtomicU64,
    pub refunded: AtomicU64,
    pub limit_hits: AtomicU64,
    pub aggregations: AtomicU64,
}

pub struct ManaState {
    pub consumed: AtomicU64,
    pub regenerated: AtomicU64,
    pub limit_hits: AtomicU64,
    pub current_balance: AtomicU64,
}

pub struct ExecutionsState {
    pub active_contexts: AtomicUsize,
    pub total: AtomicU64,
    pub successful: AtomicU64,
    pub failed: AtomicU64,
    pub events_emitted: AtomicU64,
    pub avg_execution_time_sum: AtomicU64,
    pub current_epoch: AtomicU64,
    pub current_lamport: AtomicU64,
    pub saga_triggered: AtomicU64,
}
```

**Wichtige Methoden:**

```rust
impl ExecutionState {
    /// Legacy-Kompatibilität: Start Execution
    pub fn start(&self);

    /// Legacy-Kompatibilität: Complete Execution
    pub fn complete(&self, success: bool, gas: u64, mana: u64, events: u64, duration_ms: u64);

    /// Durchschnittliche Ausführungszeit
    pub fn avg_execution_time(&self) -> f64;

    /// Erfolgsrate
    pub fn success_rate(&self) -> f64;
}
```

### 3.5 ECLVMState (Policy-Engine)

ECLVM-spezifische Metriken und Per-Realm-Tracking:

```rust
pub struct ECLVMState {
    // === Policy Engine ===
    pub policies_compiled: AtomicU64,
    pub policies_cached: AtomicUsize,
    pub policies_executed: AtomicU64,
    pub policies_passed: AtomicU64,
    pub policies_denied: AtomicU64,
    pub policy_runtime_errors: AtomicU64,
    pub policies_by_type: RwLock<HashMap<String, u64>>,

    // === Blueprint Engine ===
    pub blueprints_published: AtomicU64,
    pub blueprints_deployed: AtomicU64,
    pub blueprints_instantiated: AtomicU64,

    // === Saga/Intent ===
    pub intents_processed: AtomicU64,
    pub saga_steps_executed: AtomicU64,
    pub cross_realm_steps: AtomicU64,

    // === Resource Tracking ===
    pub total_gas_consumed: AtomicU64,
    pub total_mana_consumed: AtomicU64,
    pub out_of_gas_aborts: AtomicU64,

    // === Per-Realm ECL State ===
    pub realm_ecl: RwLock<HashMap<String, RealmECLState>>,

    // === Crossing-Policy (Κ23) ===
    pub crossing_evaluations: AtomicU64,
    pub crossings_allowed: AtomicU64,
    pub crossings_denied: AtomicU64,
    pub avg_evaluation_time_us: RwLock<f64>,
}
```

**Per-Realm ECL State:**

```rust
pub struct RealmECLState {
    pub policies_executed: AtomicU64,
    pub policies_passed: AtomicU64,
    pub policies_denied: AtomicU64,
    pub gas_consumed: AtomicU64,
    pub mana_consumed: AtomicU64,
    pub crossing_policies: AtomicU64,
    pub membership_policies: AtomicU64,
    pub active_policies: RwLock<Vec<String>>,
    pub instantiated_blueprints: AtomicU64,
}
```

### 3.6 ECLVMStateContext (Orchestration)

Orchestriert State-Zugriff für ECLVM-Ausführung:

```rust
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │                     ECLVMStateContext                           │
/// │  ┌───────────────────────────────────────────────────────────┐  │
/// │  │                      StateView                             │  │
/// │  │  get_trust() | get_realm() | get_identity() ...           │  │
/// │  └───────────────────────────────────────────────────────────┘  │
/// │  ┌───────────────────────────────────────────────────────────┐  │
/// │  │                     StateHandle                            │  │
/// │  │  update_trust() | store_put() | emit_event() ...          │  │
/// │  └───────────────────────────────────────────────────────────┘  │
/// │  ┌───────────────────────────────────────────────────────────┐  │
/// │  │                     ECLVMBudget                            │  │
/// │  │  consume_gas() | consume_mana() | check_timeout()         │  │
/// │  └───────────────────────────────────────────────────────────┘  │
/// └─────────────────────────────────────────────────────────────────┘
/// ```
pub struct ECLVMStateContext {
    pub view: StateView,
    pub budget: Arc<ECLVMBudget>,
    caller_did: String,
    realm_id: String,
    state: Arc<UnifiedState>,
    created_at: Instant,
    execution_id: String,
}
```

**Wichtige Methoden:**

```rust
impl ECLVMStateContext {
    /// Read Operations (Gas-metered via Budget)
    pub fn get_trust(&self, entity_id: &str) -> Option<f64>;
    pub fn get_realm(&self, realm_id: &str) -> Option<RealmViewData>;
    pub fn get_identity(&self, did: &str) -> Option<IdentityViewData>;

    /// Write Operations (erstellt StateHandle)
    pub fn create_write_handle(&self) -> StateHandle<'_>;
    pub fn begin_transaction(&self) -> TransactionGuard<'_>;

    /// Budget-Interaktion
    pub fn gas_remaining(&self) -> u64;
    pub fn mana_remaining(&self) -> u64;

    /// Finalization
    pub fn finalize(self) -> ECLVMExecutionSummary;
}
```

### 3.7 ECLVMBudget (Ressourcen-Tracking)

Überwacht Gas/Mana/Timeout während ECLVM-Ausführung:

```rust
pub struct ECLVMBudget {
    pub limits: ECLVMBudgetLimits,
    gas_used: AtomicU64,
    mana_used: AtomicU64,
    started_at: Instant,
    exhausted: AtomicBool,
    exhaustion_reason: RwLock<Option<BudgetExhaustionReason>>,
}

pub struct ECLVMBudgetLimits {
    pub gas_limit: u64,       // Default: 1_000_000
    pub mana_limit: u64,      // Default: 10_000
    pub max_stack_depth: u32, // Default: 1024
    pub timeout_ms: u64,      // Default: 5_000
}

pub enum BudgetExhaustionReason {
    OutOfGas,
    OutOfMana,
    Timeout,
    StackOverflow,
}
```

**Trust-basierte Skalierung:**

```rust
impl ECLVMBudgetLimits {
    /// Skaliert Limits basierend auf Trust-Score
    pub fn with_trust_factor(mut self, trust: f64) -> Self {
        let factor = trust.clamp(0.0, 1.0);
        self.gas_limit = (self.gas_limit as f64 * (0.5 + factor * 0.5)) as u64;
        self.mana_limit = (self.mana_limit as f64 * (0.5 + factor * 0.5)) as u64;
        self
    }
}
```

### 3.8 StateView (Read-Only Snapshot)

Read-only Snapshot für Policy-Evaluation:

```rust
pub struct StateView {
    pub snapshot_time: u128,
    pub caller_did: Option<String>,
    pub current_realm: Option<String>,

    // Caches (lazy populated)
    trust_cache: Arc<RwLock<HashMap<String, f64>>>,
    realm_cache: Arc<RwLock<HashMap<String, RealmViewData>>>,
    identity_cache: Arc<RwLock<HashMap<String, IdentityViewData>>>,
}

impl StateView {
    /// Aus UnifiedSnapshot befüllen (Phase 2.3 – State-backed ECL)
    pub fn refresh_from_snapshot(&mut self, snapshot: &UnifiedSnapshot);

    /// Trust-Abfrage
    pub fn get_trust(&self, entity_id: &str) -> Option<f64>;

    /// Realm-Abfrage
    pub fn get_realm(&self, realm_id: &str) -> Option<RealmViewData>;
}
```

### 3.9 StateHandle (Realm-scoped Write)

Schreibzugriff mit Transaction-Semantik:

```rust
pub struct StateHandle<'a> {
    state: &'a UnifiedState,
    caller_did: String,
    realm_id: String,
    budget: Arc<ECLVMBudget>,
    dirty_keys: RwLock<HashSet<String>>,
    events_emitted: AtomicU64,
    valid: AtomicBool,
}

impl<'a> StateHandle<'a> {
    /// Key-Value Store (nur UnifiedState, nicht Storage!)
    pub fn store_put(&self, key: &str, value: Vec<u8>) -> bool;
    pub fn store_get(&self, key: &str) -> Option<Vec<u8>>;

    /// Event emittieren
    pub fn emit_event(&self, event: &StateEvent);

    /// Transaction abschließen
    pub fn commit(self) -> CommitResult;
    pub fn rollback(self) -> RollbackResult;
}
```

### 3.10 ProtectionState (Κ19-Κ21)

Systemschutz und Anomalie-Erkennung:

- **AnomalyDetector**: Erkennt ungewöhnliche Muster
- **DiversityMonitor**: Überwacht Gini-Koeffizient
- **QuadraticGovernance**: Κ21 Quadratisches Voting
- **AntiCalcification**: Κ19 Verhindert Stagnation

---

## 4. Execution-Architektur Detailansicht

### 4.1 Beziehungen zwischen Execution-Komponenten

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           EXECUTION LAYER DETAIL                                 │
│                                                                                  │
│   ┌─────────────────────┐                                                       │
│   │   PolicyRunContext  │◄──────────────────────────────────────────┐           │
│   │   (Runner Input)    │                                           │           │
│   │   - caller_did      │                                           │           │
│   │   - realm_id        │                                           │           │
│   │   - gas_limit       │                                           │           │
│   └─────────┬───────────┘                                           │           │
│             │                                                       │           │
│             ▼                                                       │           │
│   ┌─────────────────────┐     ┌─────────────────────┐              │           │
│   │      ECLVM          │────▶│    GasMeter         │              │           │
│   │   (vm.rs)           │     │   (gas.rs)          │              │           │
│   │   - stack           │     │   - consumed        │              │           │
│   │   - ip              │     │   - limit           │              │           │
│   │   - host            │     └─────────────────────┘              │           │
│   └─────────┬───────────┘                                           │           │
│             │                                                       │           │
│             ▼                                                       │           │
│   ┌─────────────────────┐     ┌─────────────────────┐              │           │
│   │  ExecutionResult    │────▶│PolicyExecutionObserver│            │           │
│   │   - value           │     │   on_policy_executed │            │           │
│   │   - gas_used        │     │   on_crossing_eval   │            │           │
│   │   - duration_us     │     └──────────┬──────────┘              │           │
│   └─────────────────────┘                │                         │           │
│                                          │                         │           │
│                                          ▼                         │           │
│   ┌──────────────────────────────────────────────────────────────┐ │           │
│   │                     StateIntegrator                          │ │           │
│   │   - updates ECLVMState (policies_executed, gas, mana, ...)   │ │           │
│   │   - updates ExecutionState (active_contexts, success_rate)    │◄┘           │
│   │   - emits StateEvent::PolicyEvaluated                        │             │
│   └──────────────────────────────────────────────────────────────┘             │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 State vs. Storage Unterscheidung

| Aspekt | StateHandle (state.rs) | ErynoaHost (Storage) |
|--------|------------------------|----------------------|
| **Ziel** | UnifiedState (in-memory) | DecentralizedStorage (persistent) |
| **Schreibsemantik** | Ephemer, commit() → log_and_apply | Persistent, sofort in DB |
| **Verwendung** | Tests, State-only ECL | Produktion |
| **Realm-Daten** | Nicht persistiert | Persistiert in RealmStorage |
| **Trust-Updates** | Nur in-memory State | Identity/Trust-Storage |

### 4.3 ECL Policy-Typen (ECLPolicyType)

```rust
pub enum ECLPolicyType {
    Crossing,     // Κ23 Gateway-Policies
    Membership,   // Realm-Beitritt
    Transaction,  // Aktions-Validierung
    Governance,   // Abstimmungsregeln
    Privacy,      // Daten-Sichtbarkeit
    Custom,       // Benutzerdefiniert (inkl. API, UI, Controller)
}
```

**Mapping für Engines:**

| Engine | Policy-Typ | Beschreibung |
|--------|------------|--------------|
| Gateway | Crossing | Entry/Exit-Policies für Realms |
| API | Custom (→ erweiterbar zu `Api`) | Route-Handler |
| UI | Custom (→ erweiterbar zu `Ui`) | Sichtbarkeits-Gates |
| DataLogic | Custom (→ erweiterbar zu `DataLogic`) | Filter/Aggregation |
| Governance | Governance | Vote/Proposal-Entscheidung |
| Controller | Custom (→ erweiterbar zu `Controller`) | AuthZ-Checks |

---

## 5. Event-Bus (P2P/Core Entkopplung)

### 5.1 Architektur

```text
P2P Layer ──▶ [Ingress Queue] ──▶ Core Processor Task
                                        │
                                        ▼
                                  CoreState Updates
                                        │
                                        ▼
P2P Layer ◀── [Egress Queue] ◀── Outbound Events
```

### 5.2 NetworkEvent

```rust
pub struct NetworkEvent {
    pub id: u64,
    pub event_type: String,
    pub payload: Vec<u8>,
    pub priority: EventPriority,  // Critical, High, Normal, Low
    pub peer_id: Option<String>,
    pub realm_id: Option<String>,
    pub timestamp_ms: u64,

    // Identity-Integration (Phase 7)
    pub peer_universal_id: Option<UniversalId>,
    pub signature: Option<[u8; 64]>,
}
```

### 5.3 EventBus Features

- **Bounded Queues**: 10.000 Events (verhindert Memory-Exhaustion)
- **Priority Queues**: Critical Events werden bevorzugt
- **Backpressure**: P2P blockiert bei voller Queue
- **Async Processing**: Non-blocking Event-Verarbeitung

---

## 6. State-Events (Event-Sourcing)

### 6.1 StateEvent Enum

Alle State-Änderungen werden als typisierte Events geloggt:

```rust
pub enum StateEvent {
    // Core Events
    TrustUpdate { entity_id, delta, reason, ... },
    EventProcessed { event_id, depth, parents_count, ... },
    FormulaComputed { old_e, new_e, human_factor, ... },
    ConsensusRoundCompleted { epoch, success, ... },

    // Execution Events
    ExecutionStarted { context_id, gas_budget, mana_budget, ... },
    ExecutionCompleted { context_id, success, gas_consumed, ... },
    PolicyEvaluated { policy_id, passed, ... },

    // Protection Events
    AnomalyDetected { severity, description, ... },
    SystemModeChanged { old_mode, new_mode, ... },

    // Identity Events (Κ6-Κ8)
    IdentityBootstrapped { root_did, namespace, mode, ... },
    SubDIDDerived { root_did, sub_did, derivation_path, ... },
    DelegationCreated { delegator, delegate, trust_factor, ... },

    // P2P Events
    PeerConnectionChange { peer_id, connected, ... },
    PrivacyCircuitCreated { circuit_id, hop_count, ... },
    // ...
}
```

### 6.2 StateEventLog

```rust
pub struct StateEventLog {
    sequence: AtomicU64,
    buffer: RwLock<Vec<WrappedStateEvent>>,
    buffer_capacity: usize,  // Default: 10.000
    checkpoint_interval: u64, // Default: 5.000 Events
    // Recovery support...
}
```

**Vorteile:**
- Crash-Recovery durch Replay
- Audits: "Was hat Trust geändert?"
- Time-Travel: State zu historischem Punkt rekonstruieren
- Skalierbarkeit: Events ~100-500 Bytes

---

## 7. Circuit Breaker Pattern

### 7.1 SystemMode

```rust
pub enum SystemMode {
    Normal,           // Volle Funktionalität
    Degraded,         // Eingeschränkt
    EmergencyShutdown // Node offline
}
```

### 7.2 CircuitBreaker

```rust
pub struct CircuitBreaker {
    mode: AtomicU8,
    critical_window: RwLock<Vec<u64>>,  // Events in letzter Minute
    degraded_threshold: AtomicU64,      // Default: 10
    emergency_threshold: AtomicU64,     // Default: 50
    gini_threshold: RwLock<f64>,        // Default: 0.8
}
```

---

## 8. Merkle-State-Tracking

### 8.1 MerkleStateTracker

```rust
pub struct MerkleStateTracker {
    root_hash: RwLock<MerkleHash>,
    component_hashes: RwLock<HashMap<StateComponent, MerkleHash>>,
    delta_history: RwLock<Vec<MerkleDelta>>,
}
```

**Ermöglicht:**
- Light-Clients: Nur Deltas synchronisieren
- State-Proofs: Kryptographische Verifizierung
- Effiziente Recovery: Snapshot + Deltas replayen

---

## 9. Multi-Layer Gas Metering

### 9.1 GasLayer

```rust
pub enum GasLayer {
    Network,  // L1: P2P-Bandbreite
    Compute,  // L2: CPU/Instructions
    Storage,  // L3: Persistence
    Realm,    // L4: Per-Realm Quotas
}
```

### 8.2 MultiGas

```rust
pub struct MultiGas {
    pub network: AtomicU64,
    pub compute: AtomicU64,
    pub storage: AtomicU64,
    pub realm: RwLock<HashMap<String, AtomicU64>>,
    pub prices: RwLock<HashMap<GasLayer, u64>>,  // Dynamic pricing
}
```

---

## 9. Realm-Isolation (Self-Healing)

### 10.1 RealmQuota

```rust
pub struct RealmQuota {
    pub queue_slots_limit: AtomicU64,
    pub storage_bytes_limit: AtomicU64,
    pub compute_gas_limit: AtomicU64,
    pub events_limit: AtomicU64,
    pub crossings_limit: AtomicU64,
    pub violations: AtomicU64,
    pub quarantined: AtomicU8,
}
```

Bei Überschreitung: Realm wird pausiert/blockiert (Circuit Breaker pro Realm).

---

## 11. Beziehung zu Domain-Modul

### 10.1 Import-Beziehungen

`state.rs` importiert aus `domain::unified`:

```rust
use crate::domain::unified::primitives::UniversalId;
pub use crate::domain::MemberRole;
```

### 11.2 Typ-Verwendung

| State.rs | Domain Typ |
|----------|------------|
| `IdentityState` | `DID`, `DIDDocument`, `Delegation`, `DIDNamespace` |
| `CoreState` | `TrustVector6D`, `TrustRecord`, `TemporalCoord` |
| `PeerState` | `Realm`, `RealmRules`, `Saga`, `Intent` |
| `ExecutionState` | `Cost`, `Budget` |
| `EventState` | `Event`, `EventPayload`, `FinalityState` |

### 11.3 Konsistenz-Anforderungen

- **`UniversalId`** wird durchgängig als Identifikator verwendet
- **`TemporalCoord`** für alle Zeitstempel
- **`Cost`** Algebra für alle Kosten-Berechnungen
- **Axiom-Prüfungen** in `InvariantChecker`

---

## 11. Snapshot-Pattern

Jede State-Komponente implementiert eine `snapshot()` Methode:

```rust
pub fn snapshot(&self) -> ComponentSnapshot {
    ComponentSnapshot {
        counter1: self.counter1.load(Ordering::Relaxed),
        counter2: self.counter2.load(Ordering::Relaxed),
        // Komplexe Daten werden cloned
        data: self.data.read().map(|d| d.clone()).unwrap_or_default(),
    }
}
```

**Vorteile:**
- Konsistente Reads ohne Long-Lived Locks
- Serialisierbar für Persistence/API
- Metriken-Export ohne Blocking

---

## 13. Zusammenfassung

`state.rs` ist das **Single Source of Truth** für Runtime-State in Erynoa:

1. **Hierarchische Layer**: Identity → Core → Execution → Protection → Storage → Peer
2. **Thread-Safe**: Atomics + RwLock + DashMap
3. **Event-Sourced**: Alle Änderungen als StateEvents
4. **Self-Healing**: Circuit Breaker + Realm Isolation
5. **Merkle-Backed**: Kryptographische State-Proofs
6. **Multi-Layer Gas**: Faire Kosten-Abrechnung

Das **domain-Modul** liefert die **Datenstrukturen**, `state.rs` verwaltet deren **Runtime-Zustand**.
