# State Management System

> **Version:** 4.0
> **Status:** Implementiert
> **Module:** `src/core/state.rs`, `src/core/state_integration.rs`

## Inhaltsverzeichnis

1. [Überblick](#überblick)
2. [Architektur](#architektur)
3. [StateGraph - Beziehungs-Modell](#stategraph---beziehungs-modell)
4. [State-Layer](#state-layer)
   - [Core State (Κ2-Κ18)](#core-state-κ2-κ18)
   - [Execution State (IPS ℳ)](#execution-state-ips-ℳ)
   - [ECLVM State](#eclvm-state)
   - [Protection State (Κ19-Κ21)](#protection-state-κ19-κ21)
   - [Storage State](#storage-state)
   - [Peer State (Κ22-Κ24)](#peer-state-κ22-κ24)
   - [Realm State](#realm-state)
   - [P2P State](#p2p-state)
5. [Observer-Pattern & Integration](#observer-pattern--integration)
6. [Propagation-System](#propagation-system)
7. [Thread-Safety & Concurrency](#thread-safety--concurrency)
8. [Snapshot-Isolation](#snapshot-isolation)
9. [Mathematische Grundlagen](#mathematische-grundlagen)

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

## Architektur

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
│  │         │                 │                 │                │         │   │
│  │         └─────────────────┴─────────────────┴────────────────┘         │   │
│  │                                    │                                    │   │
│  │                         Trust-Event-Kausalität                          │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                        ExecutionState (IPS ℳ)                         │     │
│  │  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐        │     │
│  │  │   GasState     │───│   ManaState    │───│ ExecutionsState│        │     │
│  │  └────────┬───────┘   └────────┬───────┘   └────────┬───────┘        │     │
│  │           │                    │                    │                 │     │
│  │           └────────────────────┴────────────────────┘                 │     │
│  │                               Cost Aggregation                        │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                       ProtectionState (Κ19-Κ21)                        │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
│  │  │ AnomalyState │  │DiversityState│  │QuadraticState│  │AntiCalc  │  │     │
│  │  │              │──│    (Κ20)     │──│    (Κ21)     │──│  (Κ19)   │  │     │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────┬─────┘  │     │
│  │         │                 │                 │               │         │     │
│  │         └─────────────────┴─────────────────┴───────────────┘         │     │
│  │                         Protection Signals                            │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                      │                                          │
│  ┌───────────────────────────────────┼───────────────────────────────────┐     │
│  │                         PeerState (Κ22-Κ24)                            │     │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
│  │  │   Gateway    │  │ SagaComposer │  │ IntentParser │  │  Realm   │  │     │
│  │  │   (Κ23)      │──│  (Κ22/Κ24)   │──│              │──│  State   │  │     │
│  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────┬─────┘  │     │
│  │         │                 │                 │               │         │     │
│  │         │           ┌─────┴─────┐           │               │         │     │
│  │         │           │ Per-Realm │           │               │         │     │
│  │         │           │ Isolation │           │               │         │     │
│  │         └───────────┤ TrustVec  ├───────────┘               │         │     │
│  │                     │ Rules     │                           │         │     │
│  │                     │ Identity  │                           │         │     │
│  │                     │ Metrics   │                           │         │     │
│  │                     └───────────┘                           │         │     │
│  │                     Cross-Realm Orchestration                         │     │
│  └───────────────────────────────────────────────────────────────────────┘     │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## StateGraph - Beziehungs-Modell

Der StateGraph definiert die kausalen Beziehungen zwischen allen State-Komponenten. Diese Beziehungen sind **aktiv** - Änderungen propagieren entlang der Kanten.

### Beziehungstypen (StateRelation)

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

### Vollständiger Beziehungs-Graph

```
                    ┌─────────────────────────────────────────────────────┐
                    │                  CORE LAYER                         │
                    │                                                     │
                    │   Trust ←→ Event ←── WorldFormula ──→ Consensus    │
                    │     │        │              │              │        │
                    │     │        ▼              │              ▼        │
                    │     │    [validates]        │       [validates]     │
                    │     │        │              │              │        │
                    └─────┼────────┼──────────────┼──────────────┼────────┘
                          │        │              │              │
          ┌───────────────┼────────┼──────────────┼──────────────┼───────────────┐
          │               ▼        ▼              ▼              ▼               │
          │           ┌──────────────────────────────────────────────┐          │
          │           │              EXECUTION LAYER                  │          │
          │           │                                               │          │
          │           │   Gas ←─[DependsOn]─ Trust                   │          │
          │           │   Mana ←─[DependsOn]─ Trust                  │          │
          │           │   Execution ─[Aggregates]→ Gas, Mana         │          │
          │           │   Execution ─[Triggers]──→ Event             │          │
          │           │                                               │          │
          │           └──────────────────────────────────────────────┘          │
          │                            │                                         │
          │           ┌────────────────┼────────────────────────────────────┐   │
          │           │                ▼                                    │   │
          │           │           ECLVM LAYER                               │   │
          │           │                                                     │   │
          │           │   ECLVM ─[DependsOn]→ Gas, Mana, Trust             │   │
          │           │   ECLVM ─[Triggers]─→ Event                        │   │
          │           │   ECLVM ─[Aggregates]→ Execution                   │   │
          │           │                                                     │   │
          │           │   ECLPolicy ─[Validates]→ Gateway, Realm           │   │
          │           │   ECLPolicy ─[DependsOn]→ ECLVM                    │   │
          │           │   ECLPolicy ─[Triggers]─→ Event                    │   │
          │           │                                                     │   │
          │           │   ECLBlueprint ─[DependsOn]→ ECLVM                 │   │
          │           │   ECLBlueprint ─[Aggregates]→ Blueprint            │   │
          │           │                                                     │   │
          │           └────────────────────────────────────────────────────┘   │
          │                            │                                         │
          │           ┌────────────────┼────────────────────────────────────┐   │
          │           │                ▼                                    │   │
          │           │         PROTECTION LAYER                            │   │
          │           │                                                     │   │
          │           │   Anomaly ─[Validates]→ Event, Trust               │   │
          │           │   Diversity ─[Validates]→ Trust, Consensus         │   │
          │           │   Quadratic ─[DependsOn]→ Trust                    │   │
          │           │   AntiCalcification ─[Validates/Triggers]→ Trust   │   │
          │           │   Calibration ─[Triggers]→ Gas, Mana               │   │
          │           │                                                     │   │
          │           └────────────────────────────────────────────────────┘   │
          │                            │                                         │
          │           ┌────────────────┼────────────────────────────────────┐   │
          │           │                ▼                                    │   │
          │           │           PEER LAYER                                │   │
          │           │                                                     │   │
          │           │   Gateway ─[Validates/DependsOn]→ Trust            │   │
          │           │   Gateway ─[Triggers]─→ Event                      │   │
          │           │   Gateway ─[DependsOn]→ Realm, ECLPolicy           │   │
          │           │                                                     │   │
          │           │   SagaComposer ─[DependsOn]→ Trust, ECLVM          │   │
          │           │   SagaComposer ─[Triggers]→ Execution              │   │
          │           │   SagaComposer ─[Aggregates]→ IntentParser         │   │
          │           │                                                     │   │
          │           │   Realm ─[DependsOn/Triggers]→ Trust               │   │
          │           │   Realm ─[Aggregates/DependsOn]→ Gateway           │   │
          │           │   Realm ─[Triggers]→ SagaComposer, Event           │   │
          │           │   Realm ─[Validates]→ Event                        │   │
          │           │   Realm ─[DependsOn/Aggregates]→ ECLPolicy         │   │
          │           │                                                     │   │
          │           └────────────────────────────────────────────────────┘   │
          │                            │                                         │
          │           ┌────────────────┼────────────────────────────────────┐   │
          │           │                ▼                                    │   │
          │           │           P2P LAYER                                 │   │
          │           │                                                     │   │
          │           │   Swarm ─[Triggers]→ Event                         │   │
          │           │   Gossip ─[DependsOn]→ Trust                       │   │
          │           │   Gossip ─[Triggers]→ Event                        │   │
          │           │   Relay ─[DependsOn]→ Trust                        │   │
          │           │   Privacy ─[DependsOn]→ Trust                      │   │
          │           │   Privacy ─[Validates]→ Gossip                     │   │
          │           │                                                     │   │
          │           └────────────────────────────────────────────────────┘   │
          └─────────────────────────────────────────────────────────────────────┘
```

### StateGraph Query-Methoden

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

    // Prüfungen
    fn has_relation(&self, from: StateComponent, relation: StateRelation, to: StateComponent) -> bool;
    fn all_relations(&self, component: StateComponent) -> Vec<(StateComponent, StateRelation, StateComponent)>;

    // Transitive Operationen
    fn transitive_dependencies(&self, component: StateComponent) -> HashSet<StateComponent>;
    fn transitive_triggers(&self, component: StateComponent) -> HashSet<StateComponent>;
    fn validation_chain(&self, component: StateComponent) -> Vec<StateComponent>;

    // Metriken
    fn criticality_score(&self, component: StateComponent) -> usize;
}
```

---

## State-Layer

### Core State (Κ2-Κ18)

#### TrustState

Verwaltet das Trust-Modell gemäß Axiomen Κ2-Κ5.

```rust
pub struct TrustState {
    // Atomic Counters
    pub entities: AtomicUsize,           // Registrierte Entitäten
    pub relationships: AtomicUsize,       // Trust-Beziehungen
    pub updates_total: AtomicU64,         // Gesamt-Updates
    pub positive_updates: AtomicU64,      // Positive Updates
    pub negative_updates: AtomicU64,      // Negative Updates (Κ4: Asymmetrie)
    pub violations: AtomicU64,            // Erkannte Verletzungen

    // Complex State
    pub avg_trust: RwLock<f64>,           // Durchschnittlicher Trust
    pub trust_distribution: RwLock<TrustDistribution>,

    // Relationship-Tracking
    pub triggered_events: AtomicU64,      // Trust → Event
    pub event_triggered_updates: AtomicU64, // Event → Trust
    pub realm_triggered_updates: AtomicU64, // Realm → Trust
}
```

**Trust-Verteilung:**

```rust
pub struct TrustDistribution {
    pub histogram: [u64; 10],  // Buckets: [0-0.1, 0.1-0.2, ..., 0.9-1.0]
    pub gini: f64,             // Gini-Koeffizient
    pub entropy: f64,          // Shannon-Entropie
}
```

**Asymmetrie-Ratio (Κ4):**
$$\text{asymmetry\_ratio} = \frac{\text{negative\_updates}}{\text{positive\_updates}} \approx 2.0$$

#### EventState

Verwaltet den Event-DAG gemäß Axiomen Κ9-Κ12.

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

    // Relationship-Tracking (alle → Event Triggers)
    pub trust_triggered: AtomicU64,
    pub consensus_validated: AtomicU64,
    pub execution_triggered: AtomicU64,
    pub gateway_triggered: AtomicU64,
    pub realm_triggered: AtomicU64,
    pub eclvm_triggered: AtomicU64,
    pub policy_triggered: AtomicU64,
    pub blueprint_triggered: AtomicU64,
    pub swarm_triggered: AtomicU64,
    pub gossip_triggered: AtomicU64,
}
```

#### FormulaState (Κ15b-d)

Verwaltet die World Formula 𝔼.

```rust
pub struct FormulaState {
    pub current_e: RwLock<f64>,           // Aktueller 𝔼-Wert
    pub computations: AtomicU64,
    pub contributors: AtomicUsize,
    pub human_verified: AtomicUsize,

    // 𝔼-Komponenten
    pub avg_activity: RwLock<f64>,        // Durchschnittliche Aktivität
    pub avg_trust_norm: RwLock<f64>,      // Durchschnittliche Trust-Norm
    pub human_factor: RwLock<f64>,        // Human-Faktor H(i)

    // Trend-Analyse
    pub e_history: RwLock<Vec<(u64, f64)>>, // (timestamp_ms, value)
}
```

**World Formula:**
$$\mathbb{E} = \sum_{i \in \mathcal{I}} w_i \cdot \sigma(\alpha \cdot A(i)) \cdot \|T(i)\| \cdot H(i)$$

#### ConsensusState (Κ18)

```rust
pub struct ConsensusState {
    pub epoch: AtomicU64,
    pub validators: AtomicUsize,
    pub successful_rounds: AtomicU64,
    pub failed_rounds: AtomicU64,
    pub avg_round_time_ms: RwLock<f64>,
    pub byzantine_detected: AtomicU64,
    pub leader_changes: AtomicU64,
    pub events_validated: AtomicU64,      // Consensus ✓ Event
}
```

---

### Execution State (IPS ℳ)

Das Execution State ist in drei Sub-States aufgeteilt für tiefe Relationship-Integration:

#### GasState

```rust
pub struct GasState {
    pub consumed: AtomicU64,
    pub refunded: AtomicU64,
    pub out_of_gas: AtomicU64,
    pub current_price: RwLock<f64>,
    pub max_per_block: AtomicU64,

    // Relationships
    pub calibration_adjustments: AtomicU64,  // Calibration → Gas
    pub trust_dependency_updates: AtomicU64, // Gas ← Trust
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

    // Relationships
    pub calibration_adjustments: AtomicU64,  // Calibration → Mana
    pub trust_dependency_updates: AtomicU64, // Mana ← Trust
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
    pub current_epoch: AtomicU64,
    pub current_lamport: AtomicU64,

    // Relationships
    pub saga_triggered: AtomicU64,      // SagaComposer → Execution
    pub gas_aggregations: AtomicU64,    // Execution ⊃ Gas
    pub mana_aggregations: AtomicU64,   // Execution ⊃ Mana
}
```

---

### ECLVM State

Die Erynoa Core Language Virtual Machine für Policy- und Blueprint-Ausführung.

#### Policy-Typen

```rust
pub enum ECLPolicyType {
    Crossing,     // Gateway-Policies (Κ23)
    Membership,   // Realm-Beitritt
    Transaction,  // Aktions-Regeln
    Governance,   // Abstimmungs-Regeln
    Privacy,      // Daten-Sichtbarkeit
    Custom,       // Benutzerdefiniert
}
```

#### ECLVMState

```rust
pub struct ECLVMState {
    // Policy Engine
    pub policies_compiled: AtomicU64,
    pub policies_cached: AtomicUsize,
    pub policies_executed: AtomicU64,
    pub policies_passed: AtomicU64,
    pub policies_denied: AtomicU64,

    // Blueprint Engine
    pub blueprints_published: AtomicU64,
    pub blueprints_deployed: AtomicU64,
    pub blueprints_instantiated: AtomicU64,
    pub blueprints_verified: AtomicU64,

    // Saga/Intent Orchestrierung
    pub intents_processed: AtomicU64,
    pub saga_steps_executed: AtomicU64,
    pub cross_realm_steps: AtomicU64,
    pub compensations_triggered: AtomicU64,

    // Resource Tracking
    pub total_gas_consumed: AtomicU64,
    pub total_mana_consumed: AtomicU64,
    pub out_of_gas_aborts: AtomicU64,
    pub mana_rate_limited: AtomicU64,

    // Per-Realm ECL State
    pub realm_ecl: RwLock<HashMap<String, RealmECLState>>,

    // Crossing-Policy (Κ23)
    pub crossing_evaluations: AtomicU64,
    pub crossings_allowed: AtomicU64,
    pub crossings_denied: AtomicU64,
}
```

#### RealmECLState (Per-Realm)

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

---

### Protection State (Κ19-Κ21)

#### AnomalyState

```rust
pub struct AnomalyState {
    pub total: AtomicU64,
    pub critical: AtomicU64,
    pub high: AtomicU64,
    pub medium: AtomicU64,
    pub low: AtomicU64,
    pub false_positives: AtomicU64,

    // Relationships
    pub events_validated: AtomicU64,        // Anomaly ✓ Event
    pub trust_patterns_checked: AtomicU64,  // Anomaly ✓ Trust
}
```

#### DiversityState (Κ20)

```rust
pub struct DiversityState {
    pub dimensions: AtomicUsize,
    pub monoculture_warnings: AtomicU64,
    pub entropy_values: RwLock<HashMap<String, f64>>,
    pub min_entropy: RwLock<f64>,

    // Relationships
    pub trust_distribution_checks: AtomicU64,  // Diversity ✓ Trust
    pub validator_mix_checks: AtomicU64,       // Diversity ✓ Consensus
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

    // Relationships
    pub trust_dependency_updates: AtomicU64,  // Quadratic ← Trust
}
```

**Quadratic Voting:**
$$\text{cost}(v) = v^2$$

#### AntiCalcificationState (Κ19)

```rust
pub struct AntiCalcificationState {
    pub power_concentration: RwLock<f64>,
    pub gini_coefficient: RwLock<f64>,
    pub interventions: AtomicU64,
    pub watched_entities: AtomicUsize,
    pub threshold_violations: AtomicU64,

    // Relationships
    pub trust_limits_checked: AtomicU64,  // AntiCalcification → Trust
    pub power_checks: AtomicU64,          // AntiCalcification ✓ Trust
}
```

**Power-Cap:**
$$P_{\text{capped}}(i) = \min(P(i), P_{\text{max}} \cdot (1 - \text{gini}))$$

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

### Peer State (Κ22-Κ24)

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

---

### Realm State

Per-Realm Isolation für Κ22-Κ24:

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
```

```rust
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

### P2P State

```rust
pub struct SwarmState { ... }
pub struct GossipState { ... }
pub struct RelayState { ... }
pub struct NatTraversalState { ... }
pub struct PrivacyState { ... }

pub struct P2PState {
    pub swarm: SwarmState,
    pub gossip: GossipState,
    pub relay: RelayState,
    pub nat: NatTraversalState,
    pub privacy: PrivacyState,
}
```

---

## Observer-Pattern & Integration

### Observer Traits

Jede Domäne hat einen spezifischen Observer-Trait:

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

pub trait ExecutionObserver: Send + Sync { ... }
pub trait ProtectionObserver: Send + Sync { ... }
pub trait FormulaObserver: Send + Sync { ... }
pub trait ConsensusObserver: Send + Sync { ... }
pub trait GatewayObserver: Send + Sync { ... }
pub trait SagaObserver: Send + Sync { ... }
pub trait RealmObserver: Send + Sync { ... }
pub trait ECLVMObserver: Send + Sync { ... }
pub trait SwarmObserver: Send + Sync { ... }
pub trait GossipObserver: Send + Sync { ... }
// ...
```

### StateIntegrator

Der `StateIntegrator` implementiert alle Observer-Traits und verbindet sie mit `UnifiedState`:

```rust
pub struct StateIntegrator {
    state: SharedUnifiedState,
    graph: StateGraph,
}

impl StateIntegrator {
    pub fn new(state: SharedUnifiedState) -> Self {
        Self {
            state,
            graph: StateGraph::erynoa_graph(),
        }
    }
}

// Beispiel-Implementation
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

## Propagation-System

Das Propagation-System ist das Herzstück der tiefen Relationship-Integration. Es propagiert State-Änderungen entlang der StateGraph-Kanten.

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
            (Gateway, Event) => {
                self.state.core.events.gateway_triggered.fetch_add(1, Ordering::Relaxed);
            }
            (Realm, Event) => {
                self.state.core.events.realm_triggered.fetch_add(1, Ordering::Relaxed);
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
            (Diversity, Trust) => {
                self.state.protection.diversity.trust_distribution_checks.fetch_add(1, Ordering::Relaxed);
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
            // ... weitere Aggregations-Beziehungen
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
            // ... weitere Dependency-Beziehungen
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

## Thread-Safety & Concurrency

### Atomare Counter

Alle einfachen numerischen Werte sind `AtomicU64` oder `AtomicUsize`:

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

**Lock-Granularität:**

- Feingranulare Locks (per Sub-State)
- Kurze Lock-Zeiten
- Read-Heavy Workloads bevorzugt

### SharedUnifiedState

```rust
pub type SharedUnifiedState = Arc<UnifiedState>;
```

Arc ermöglicht thread-safe Sharing ohne Mutex.

---

## Snapshot-Isolation

Snapshots ermöglichen konsistente Reads ohne globales Locking:

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

## Mathematische Grundlagen

### Trust-Asymmetrie (Κ4)

$$\Delta T^{-} = k \cdot \Delta T^{+}, \quad k \approx 2$$

### World Formula (Κ15b-d)

$$\mathbb{E} = \sum_{i \in \mathcal{I}} w_i \cdot \sigma(\alpha \cdot A(i)) \cdot \|T(i)\| \cdot H(i)$$

Wobei:

- $w_i$ = Gewicht der Identität
- $\sigma$ = Sigmoid-Funktion
- $A(i)$ = Aktivitäts-Score
- $\|T(i)\|$ = Trust-Norm
- $H(i)$ = Human-Faktor

### Shannon-Entropie (Κ20)

$$H = -\sum_{i=1}^{n} p_i \log_2(p_i)$$

### Gini-Koeffizient (Κ19)

$$G = \frac{\sum_{i=1}^{n} \sum_{j=1}^{n} |x_i - x_j|}{2n^2 \bar{x}}$$

### Quadratic Voting (Κ21)

$$\text{cost}(v) = v^2$$
$$\text{max\_votes}(c) = \lfloor \sqrt{c} \rfloor$$

### Trust-Dampening (Κ23)

$$T_{\text{dampened}} = T_{\text{original}} \cdot D_{\text{realm}}$$

Wobei $D_{\text{realm}}$ die Realm-spezifische Dämpfungsmatrix ist.

---

## Zusammenfassung

Das State Management System bietet:

1. **Hierarchische Struktur**: 7 Layer mit klaren Verantwortlichkeiten
2. **Tiefe Beziehungen**: 50+ StateGraph-Kanten mit aktiver Propagation
3. **Thread-Safety**: Atomare Counter + feingranulare RwLocks
4. **Vollständiges Tracking**: Alle Kausalitäten werden erfasst
5. **Snapshot-Isolation**: Konsistente Reads ohne Locking
6. **Observer-Pattern**: Lose Kopplung zwischen Engines und State
7. **Mathematische Fundierung**: Basierend auf Axiomen Κ2-Κ24

Das System ermöglicht:

- Echtzeit-Monitoring aller Systemzustände
- Debugging durch vollständige Kausalitäts-Ketten
- Performance-Optimierung durch parallele Reads
- Erweiterbarkeit durch modulare Sub-States
