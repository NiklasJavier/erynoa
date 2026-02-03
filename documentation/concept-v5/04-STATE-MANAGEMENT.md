# State Management System

> **Version:** V5.0 – Konsolidiert
> **Axiom-Basis:** Κ2-Κ24
> **Status:** Implementiert

---

## Überblick

Das State Management System ist das zentrale Nervensystem von Erynoa. Es verwaltet den gesamten Systemzustand hierarchisch, thread-safe und mit vollständiger Kausalitäts-Verfolgung.

### Kernprinzipien

| Prinzip                        | Beschreibung                                       |
| ------------------------------ | -------------------------------------------------- |
| **Hierarchische Komposition**  | State-Layer bauen aufeinander auf                  |
| **Thread-Safety**              | Atomare Counter, RwLock für komplexe Strukturen    |
| **Dependency Injection**       | Jeder Layer kennt seine Abhängigkeiten             |
| **Event-Driven Updates**       | Änderungen propagieren durch Observer-Pattern      |
| **Snapshot-Isolation**         | Konsistente Reads ohne globales Locking            |
| **Per-Realm Isolation**        | Jedes Realm hat eigenen State                      |
| **Deep Relationship Tracking** | StateGraph-Kanten sind aktiv, nicht nur deklarativ |

---

## I. Architektur

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              UNIFIED STATE                                       │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                          CoreState (Κ2-Κ18)                              │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │   │
│  │  │ TrustState   │──│ EventState   │──│ FormulaState │──│ Consensus  │  │   │
│  │  │  (Κ2-Κ5)     │  │  (Κ9-Κ12)    │  │  (Κ15b-d)    │  │   (Κ18)    │  │   │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └─────┬──────┘  │   │
│  │         └─────────────────┴─────────────────┴────────────────┘         │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                        ExecutionState (IPS ℳ)                         │     │
│  │  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐        │     │
│  │  │   GasState     │───│   ManaState    │───│ ExecutionsState│        │     │
│  │  └────────────────┘   └────────────────┘   └────────────────┘        │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                       ProtectionState (Κ19-Κ21)                        │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
│  │  │ AnomalyState │  │DiversityState│  │QuadraticState│  │AntiCalc  │  │     │
│  │  │              │──│    (Κ20)     │──│    (Κ21)     │──│  (Κ19)   │  │     │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────┘  │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                         PeerState (Κ22-Κ24)                            │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
│  │  │   Gateway    │  │ SagaComposer │  │ IntentParser │  │  Realm   │  │     │
│  │  │   (Κ23)      │──│  (Κ22/Κ24)   │──│              │──│  State   │  │     │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────┘  │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                           P2P Layer                                    │     │
│  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐          │     │
│  │  │ Swarm  │  │ Gossip │  │  Relay │  │  NAT   │  │Privacy │          │     │
│  │  └────────┘  └────────┘  └────────┘  └────────┘  └────────┘          │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## II. StateGraph – Beziehungs-Modell

Der StateGraph definiert die kausalen Beziehungen zwischen allen State-Komponenten. Diese Beziehungen sind **aktiv** – Änderungen propagieren entlang der Kanten.

### Beziehungstypen

| Relation        | Symbol | Semantik                  |
| --------------- | ------ | ------------------------- |
| `DependsOn`     | A ← B  | A hängt kausal von B ab   |
| `Triggers`      | A → B  | A triggert Updates in B   |
| `Bidirectional` | A ↔ B  | Gegenseitige Abhängigkeit |
| `Aggregates`    | A ⊃ B  | A aggregiert Daten aus B  |
| `Validates`     | A ✓ B  | A validiert B             |

### State-Komponenten

```rust
pub enum StateComponent {
    // Core Layer (Κ2-Κ18)
    Trust,           // Trust-Management (Κ2-Κ5)
    Event,           // Event-DAG (Κ9-Κ12)
    WorldFormula,    // 𝔼 Berechnung (Κ15b-d)
    Consensus,       // BFT-Konsens (Κ18)

    // Execution Layer (IPS ℳ)
    Gas,             // Compute-Ressource
    Mana,            // Bandwidth-Ressource
    Execution,       // Execution-Tracking

    // ECLVM Layer
    ECLVM,           // Virtual Machine
    ECLPolicy,       // Policy Engine
    ECLBlueprint,    // Blueprint Management

    // Protection Layer (Κ19-Κ21)
    Anomaly,         // Anomalie-Erkennung
    Diversity,       // Diversity-Monitor (Κ20)
    Quadratic,       // Quadratic Governance (Κ21)
    AntiCalcification, // Anti-Calc (Κ19)
    Calibration,     // Parameter-Kalibrierung

    // Storage Layer
    KvStore, EventStore, Archive, Blueprint,

    // Peer Layer (Κ22-Κ24)
    Gateway,         // Crossing-Gateway (Κ23)
    SagaComposer,    // Saga-Orchestrierung (Κ22/Κ24)
    IntentParser,    // Intent-Parsing
    Realm,           // Realm-Isolation

    // P2P Layer
    Swarm, Gossip, Kademlia, Relay, NatTraversal, Privacy,
}
```

### Beziehungs-Graph (50+ Kanten)

```
                    ┌─────────────────────────────────────────────────────┐
                    │                  CORE LAYER                         │
                    │                                                     │
                    │   Trust ←→ Event ←── WorldFormula ──→ Consensus    │
                    │     │        │              │              │        │
                    │     │        ▼              │              ▼        │
                    │     │    [validates]        │       [validates]     │
                    └─────┼────────┼──────────────┼──────────────┼────────┘
                          │        │              │              │
          ┌───────────────┼────────┼──────────────┼──────────────┼───────────────┐
          │               ▼        ▼              ▼              ▼               │
          │           ┌──────────────────────────────────────────────┐          │
          │           │              EXECUTION LAYER                  │          │
          │           │   Gas ←─[DependsOn]─ Trust                   │          │
          │           │   Mana ←─[DependsOn]─ Trust                  │          │
          │           │   Execution ─[Aggregates]→ Gas, Mana         │          │
          │           │   Execution ─[Triggers]──→ Event             │          │
          │           └──────────────────────────────────────────────┘          │
          │                            │                                         │
          │           ┌────────────────┼────────────────────────────────────┐   │
          │           │         PROTECTION LAYER                            │   │
          │           │   Anomaly ─[Validates]→ Event, Trust               │   │
          │           │   Diversity ─[Validates]→ Trust, Consensus         │   │
          │           │   Quadratic ─[DependsOn]→ Trust                    │   │
          │           │   AntiCalcification ─[Validates/Triggers]→ Trust   │   │
          │           └────────────────────────────────────────────────────┘   │
          │                            │                                         │
          │           ┌────────────────┼────────────────────────────────────┐   │
          │           │           PEER LAYER                                │   │
          │           │   Gateway ─[Validates/DependsOn]→ Trust            │   │
          │           │   Gateway ─[Triggers]─→ Event                      │   │
          │           │   SagaComposer ─[DependsOn]→ Trust, ECLVM          │   │
          │           │   SagaComposer ─[Triggers]→ Execution              │   │
          │           │   Realm ─[DependsOn/Triggers]→ Trust               │   │
          │           │   Realm ─[Validates]→ Event                        │   │
          │           └────────────────────────────────────────────────────┘   │
          └─────────────────────────────────────────────────────────────────────┘
```

### Query-Methoden

```rust
impl StateGraph {
    // Direkte Beziehungen
    fn dependents(&self, component: StateComponent) -> Vec<StateComponent>;
    fn triggered_by(&self, component: StateComponent) -> Vec<StateComponent>;
    fn aggregated_by(&self, component: StateComponent) -> Vec<StateComponent>;
    fn validated_by(&self, component: StateComponent) -> Vec<StateComponent>;
    fn validators_of(&self, component: StateComponent) -> Vec<StateComponent>;
    fn bidirectional_with(&self, component: StateComponent) -> Vec<StateComponent>;
    fn dependencies_of(&self, component: StateComponent) -> Vec<StateComponent>;

    // Transitive Operationen
    fn transitive_dependencies(&self, component: StateComponent) -> HashSet<StateComponent>;
    fn transitive_triggers(&self, component: StateComponent) -> HashSet<StateComponent>;
    fn validation_chain(&self, component: StateComponent) -> Vec<StateComponent>;

    // Metriken
    fn criticality_score(&self, component: StateComponent) -> usize;
}
```

---

## III. State-Layer

### 3.1 Core State (Κ2-Κ18)

#### TrustState (Κ2-Κ5)

```rust
pub struct TrustState {
    // Atomic Counters
    pub entities: AtomicUsize,
    pub relationships: AtomicUsize,
    pub updates_total: AtomicU64,
    pub positive_updates: AtomicU64,
    pub negative_updates: AtomicU64,  // Κ4: Asymmetrie
    pub violations: AtomicU64,

    // Complex State
    pub avg_trust: RwLock<f64>,
    pub trust_distribution: RwLock<TrustDistribution>,

    // Relationship-Tracking
    pub triggered_events: AtomicU64,
    pub event_triggered_updates: AtomicU64,
    pub realm_triggered_updates: AtomicU64,
}

pub struct TrustDistribution {
    pub histogram: [u64; 10],  // [0-0.1, ..., 0.9-1.0]
    pub gini: f64,
    pub entropy: f64,
}
```

**Asymmetrie-Ratio (Κ4):**
$$\text{asymmetry\_ratio} = \frac{\text{negative\_updates}}{\text{positive\_updates}} \approx 2.0$$

#### EventState (Κ9-Κ12)

```rust
pub struct EventState {
    // DAG Metrics
    pub total: AtomicU64,
    pub genesis: AtomicU64,
    pub finalized: AtomicU64,
    pub witnessed: AtomicU64,
    pub validation_errors: AtomicU64,
    pub cycles_detected: AtomicU64,
    pub max_depth: AtomicU64,
    pub avg_parents: RwLock<f64>,
    pub finality_latency_ms: RwLock<Vec<u64>>,

    // Trigger-Tracking (alle → Event)
    pub trust_triggered: AtomicU64,
    pub consensus_validated: AtomicU64,
    pub execution_triggered: AtomicU64,
    pub gateway_triggered: AtomicU64,
    pub realm_triggered: AtomicU64,
    pub eclvm_triggered: AtomicU64,
}
```

#### FormulaState (Κ15b-d)

```rust
pub struct FormulaState {
    pub current_e: RwLock<f64>,
    pub computations: AtomicU64,
    pub contributors: AtomicUsize,
    pub human_verified: AtomicUsize,

    // Komponenten
    pub avg_activity: RwLock<f64>,
    pub avg_trust_norm: RwLock<f64>,
    pub human_factor: RwLock<f64>,

    // Trend
    pub e_history: RwLock<Vec<(u64, f64)>>,
}
```

#### ConsensusState (Κ18)

```rust
pub struct ConsensusState {
    pub epoch: AtomicU64,
    pub validators: AtomicUsize,
    pub successful_rounds: AtomicU64,
    pub failed_rounds: AtomicU64,
    pub avg_round_time_ms: RwLock<f64>,
    pub byzantine_detected: AtomicU64,
    pub events_validated: AtomicU64,
}
```

---

### 3.2 Execution State (IPS ℳ)

#### GasState

```rust
pub struct GasState {
    pub consumed: AtomicU64,
    pub refunded: AtomicU64,
    pub out_of_gas: AtomicU64,
    pub current_price: RwLock<f64>,
    pub max_per_block: AtomicU64,
    pub calibration_adjustments: AtomicU64,
    pub trust_dependency_updates: AtomicU64,
}
```

#### ManaState

```rust
pub struct ManaState {
    pub consumed: AtomicU64,
    pub regenerated: AtomicU64,
    pub rate_limited: AtomicU64,
    pub regen_rate: RwLock<f64>,
    pub max_per_entity: AtomicU64,
    pub calibration_adjustments: AtomicU64,
    pub trust_dependency_updates: AtomicU64,
}
```

#### ExecutionsState

```rust
pub struct ExecutionsState {
    pub active_contexts: AtomicUsize,
    pub total: AtomicU64,
    pub successful: AtomicU64,
    pub failed: AtomicU64,
    pub events_emitted: AtomicU64,
    pub execution_times_ms: RwLock<Vec<u64>>,
    pub saga_triggered: AtomicU64,
    pub gas_aggregations: AtomicU64,
    pub mana_aggregations: AtomicU64,
}
```

---

### 3.3 Protection State (Κ19-Κ21)

#### AnomalyState

```rust
pub struct AnomalyState {
    pub total: AtomicU64,
    pub critical: AtomicU64,
    pub high: AtomicU64,
    pub medium: AtomicU64,
    pub low: AtomicU64,
    pub false_positives: AtomicU64,
    pub events_validated: AtomicU64,
    pub trust_patterns_checked: AtomicU64,
}
```

#### DiversityState (Κ20)

```rust
pub struct DiversityState {
    pub dimensions: AtomicUsize,
    pub monoculture_warnings: AtomicU64,
    pub entropy_values: RwLock<HashMap<String, f64>>,
    pub min_entropy: RwLock<f64>,
    pub trust_distribution_checks: AtomicU64,
    pub validator_mix_checks: AtomicU64,
}
```

**Shannon-Entropie:**
$$H = -\sum_{i} p_i \log_2(p_i)$$

#### QuadraticState (Κ21)

```rust
pub struct QuadraticState {
    pub active_votes: AtomicUsize,
    pub completed_votes: AtomicU64,
    pub total_participants: AtomicU64,
    pub quadratic_reductions: AtomicU64,
    pub trust_dependency_updates: AtomicU64,
}
```

#### AntiCalcificationState (Κ19)

```rust
pub struct AntiCalcificationState {
    pub power_concentration: RwLock<f64>,
    pub gini_coefficient: RwLock<f64>,
    pub interventions: AtomicU64,
    pub watched_entities: AtomicUsize,
    pub threshold_violations: AtomicU64,
    pub trust_limits_checked: AtomicU64,
    pub power_checks: AtomicU64,
}
```

#### Health-Score

```rust
fn health_score(&self) -> f64 {
    let mut score = 100.0;
    score -= (critical_anomalies * 20) as f64;
    score -= (high_anomalies * 10) as f64;
    score -= (monoculture_warnings * 5) as f64;
    score -= (anti_calc_violations * 10) as f64;
    score.max(0.0).min(100.0)
}
```

---

### 3.4 Peer State (Κ22-Κ24)

#### GatewayState (Κ23)

```rust
pub struct GatewayState {
    pub crossings_total: AtomicU64,
    pub crossings_allowed: AtomicU64,
    pub crossings_denied: AtomicU64,
    pub trust_violations: AtomicU64,
    pub credential_violations: AtomicU64,
    pub rule_violations: AtomicU64,
    pub avg_crossing_trust: RwLock<f64>,
    pub dampening_applied: AtomicU64,
    pub registered_realms: AtomicUsize,
}
```

#### SagaComposerState (Κ22, Κ24)

```rust
pub struct SagaComposerState {
    pub sagas_composed: AtomicU64,
    pub successful_compositions: AtomicU64,
    pub failed_compositions: AtomicU64,
    pub avg_steps_per_saga: RwLock<f64>,
    pub compensations_executed: AtomicU64,
    pub compensations_successful: AtomicU64,
    pub budget_violations: AtomicU64,
    pub cross_realm_sagas: AtomicU64,
    pub goals_by_type: RwLock<HashMap<String, u64>>,
}
```

#### RealmState

```rust
pub struct PerRealmState {
    pub realm_id: String,
    pub min_trust: f32,
    pub governance_type: String,
    pub member_count: AtomicU64,
    pub crossings_in: AtomicU64,
    pub crossings_out: AtomicU64,
    pub active_rules: RwLock<Vec<String>>,
    pub trust_state: RwLock<HashMap<String, f64>>,
}

pub struct RealmState {
    pub total_realms: AtomicU64,
    pub root_realm_id: RwLock<Option<String>>,
    pub active_crossings: AtomicU64,
    pub crossing_failures: AtomicU64,
    pub total_cross_realm_sagas: AtomicU64,
    pub realms: RwLock<HashMap<String, PerRealmState>>,
}
```

---

## IV. Observer-Pattern & Integration

### Observer Traits

```rust
pub trait TrustObserver: Send + Sync {
    fn on_trust_update(&self, from: &EntityId, to: &EntityId, old: f64, new: f64, positive: bool);
    fn on_entity_registered(&self, entity: &EntityId);
    fn on_relationship_created(&self, from: &EntityId, to: &EntityId);
    fn on_violation_detected(&self, entity: &EntityId, violation_type: &str);
}

pub trait EventObserver: Send + Sync {
    fn on_event_added(&self, event_id: &EventId, is_genesis: bool, parents: usize, depth: u64);
    fn on_event_finalized(&self, event_id: &EventId, latency_ms: u64);
    fn on_event_witnessed(&self, event_id: &EventId, witness: &EntityId);
    fn on_cycle_detected(&self, event_id: &EventId);
    fn on_validation_error(&self, event_id: &EventId, error: &str);
}

pub trait ExecutionObserver: Send + Sync { /* ... */ }
pub trait ProtectionObserver: Send + Sync { /* ... */ }
pub trait FormulaObserver: Send + Sync { /* ... */ }
pub trait ConsensusObserver: Send + Sync { /* ... */ }
pub trait GatewayObserver: Send + Sync { /* ... */ }
pub trait SagaObserver: Send + Sync { /* ... */ }
pub trait RealmObserver: Send + Sync { /* ... */ }
```

### StateIntegrator

```rust
pub struct StateIntegrator {
    state: SharedUnifiedState,
    graph: StateGraph,
}

impl TrustObserver for StateIntegrator {
    fn on_trust_update(&self, from: &EntityId, to: &EntityId, old: f64, new: f64, positive: bool) {
        let state = self.state.core.trust;
        state.update(positive, false);
        state.update_triggered_event();

        // Propagiere Update durch StateGraph
        self.propagate_update(StateComponent::Trust, "trust_update");
    }
}
```

---

## V. Propagation-System

Das Propagation-System ist das Herzstück der tiefen Relationship-Integration.

### 4-Phasen-Architektur

```rust
fn propagate_update(&self, source: StateComponent, event_type: &str) {
    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 1: TRIGGER PROPAGATION (A → B)
    // ═══════════════════════════════════════════════════════════════════════
    for target in self.graph.triggered_by(source) {
        match (source, target) {
            (Trust, Event) => {
                self.state.core.events.trust_triggered.fetch_add(1, Ordering::Relaxed);
            }
            (Execution, Event) => {
                self.state.core.events.execution_triggered.fetch_add(1, Ordering::Relaxed);
            }
            // ... weitere Trigger-Beziehungen
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 2: VALIDATION PROPAGATION (A ✓ B)
    // ═══════════════════════════════════════════════════════════════════════
    for target in self.graph.validated_by(source) {
        match (source, target) {
            (Anomaly, Event) => {
                self.state.protection.anomaly.events_validated.fetch_add(1, Ordering::Relaxed);
            }
            (Anomaly, Trust) => {
                self.state.protection.anomaly.trust_patterns_checked.fetch_add(1, Ordering::Relaxed);
            }
            // ... weitere Validierungs-Beziehungen
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 3: AGGREGATION PROPAGATION (A ⊃ B)
    // ═══════════════════════════════════════════════════════════════════════
    for target in self.graph.aggregated_by(source) {
        match (source, target) {
            (Execution, Gas) => {
                self.state.execution.executions.gas_aggregations.fetch_add(1, Ordering::Relaxed);
            }
            (Execution, Mana) => {
                self.state.execution.executions.mana_aggregations.fetch_add(1, Ordering::Relaxed);
            }
            // ...
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PHASE 4: DEPENDENCY TRACKING (A ← B)
    // ═══════════════════════════════════════════════════════════════════════
    for dep in self.graph.dependencies_of(source) {
        match (source, dep) {
            (Gas, Trust) => {
                self.state.execution.gas.trust_dependency_updates.fetch_add(1, Ordering::Relaxed);
            }
            (Mana, Trust) => {
                self.state.execution.mana.trust_dependency_updates.fetch_add(1, Ordering::Relaxed);
            }
            // ...
        }
    }
}
```

### Propagations-Fluss Beispiel

```
User Action: Trust Update (Alice → Bob)
    │
    ▼
TrustObserver::on_trust_update()
    │
    ├─► TrustState.update()
    │
    └─► propagate_update(Trust, "trust_update")
            │
            ├─► PHASE 1: Triggers
            │       └─► EventState.trust_triggered++
            │
            ├─► PHASE 2: Validations
            │       ├─► AnomalyState.trust_patterns_checked++
            │       ├─► DiversityState.trust_distribution_checks++
            │       └─► AntiCalcificationState.power_checks++
            │
            ├─► PHASE 3: Aggregations
            │       (keine für Trust als Source)
            │
            └─► PHASE 4: Dependencies
                    ├─► GasState.trust_dependency_updates++
                    ├─► ManaState.trust_dependency_updates++
                    └─► QuadraticState.trust_dependency_updates++
```

---

## VI. Thread-Safety & Concurrency

### Atomare Counter

```rust
pub total: AtomicU64,
pub entities: AtomicUsize,
```

**Ordering:**

- `Ordering::Relaxed` für unabhängige Counter (Performance)
- `Ordering::SeqCst` nur bei Synchronisations-Bedarf

### RwLock für komplexe Strukturen

```rust
pub trust_distribution: RwLock<TrustDistribution>,
pub realms: RwLock<HashMap<String, PerRealmState>>,
```

### SharedUnifiedState

```rust
pub type SharedUnifiedState = Arc<UnifiedState>;
```

---

## VII. Snapshot-Isolation

```rust
impl UnifiedState {
    pub fn snapshot(&self) -> UnifiedStateSnapshot {
        UnifiedStateSnapshot {
            core: self.core.snapshot(),
            execution: self.execution.snapshot(),
            eclvm: self.eclvm.snapshot(),
            protection: self.protection.snapshot(),
            storage: self.storage.snapshot(),
            peer: self.peer.snapshot(),
            p2p: self.p2p.snapshot(),
            timestamp_ms: now(),
        }
    }
}
```

**Eigenschaften:**

- Punkt-in-Zeit Konsistenz
- Keine Locks während Read
- Serialisierbar (JSON/MessagePack)
- Ideal für Monitoring/Debugging

---

## VIII. Mathematische Grundlagen

### Trust-Asymmetrie (Κ4)

$$\Delta T^{-} = k \cdot \Delta T^{+}, \quad k \approx 2$$

### World Formula (Κ15b-d)

$$\mathbb{E} = \sum_{i \in \mathcal{I}} w_i \cdot \sigma(\alpha \cdot A(i)) \cdot \|T(i)\| \cdot H(i)$$

### Shannon-Entropie (Κ20)

$$H = -\sum_{i=1}^{n} p_i \log_2(p_i)$$

### Gini-Koeffizient (Κ19)

$$G = \frac{\sum_{i=1}^{n} \sum_{j=1}^{n} |x_i - x_j|}{2n^2 \bar{x}}$$

### Quadratic Voting (Κ21)

$$\text{cost}(v) = v^2, \quad \text{max\_votes}(c) = \lfloor \sqrt{c} \rfloor$$

---

_Weiter zu [05-IMPLEMENTATION-GUIDE.md](05-IMPLEMENTATION-GUIDE.md) für technische Details._
