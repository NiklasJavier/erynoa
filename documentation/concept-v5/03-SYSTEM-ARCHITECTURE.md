# System-Architektur

> **Version:** V5.3 – Vollständige Verbindungsmatrizen (Deep Code-Aligned)
> **Axiom-Basis:** Κ1-Κ28
> **Status:** Implementiert in `backend/src/`
> **Umfang:** ~3000 Zeilen, 11 Sektionen, 6 Layer-Verbindungsmatrizen

---

## Übersicht

Die Erynoa-Architektur ist direkt aus den 28 Kern-Axiomen abgeleitet. Jede Komponente hat eine klare Axiom-Zuordnung, und jede Schicht erfüllt spezifische Garantien. Die Komponenten sind durch ein **State-Graph-System** miteinander verbunden, das die kausalen Abhängigkeiten explizit modelliert.

### Architektur-Philosophie

1. **Hierarchische Komposition**: State-Layer bauen aufeinander auf
2. **Thread-Safety**: Atomare Counter + RwLock für komplexe Strukturen
3. **Dependency Injection**: Jeder Layer kennt seine Abhängigkeiten explizit
4. **Observer-Pattern**: Änderungen propagieren automatisch durch das System
5. **Snapshot-Isolation**: Konsistente Reads ohne globales Locking
6. **Per-Realm Isolation**: Jedes Realm hat eigenen State-Scope

### Modul-Übersicht (`backend/src/`)

```
src/
├── core/                   # Core Logic Layer (Κ2-Κ18)
│   ├── trust_engine.rs     # TrustEngine (755 Zeilen)
│   ├── event_engine.rs     # EventEngine (733 Zeilen)
│   ├── world_formula.rs    # WorldFormulaEngine (727 Zeilen)
│   ├── surprisal.rs        # SurprisalCalculator (334 Zeilen)
│   ├── consensus.rs        # ConsensusEngine
│   ├── state.rs            # UnifiedState + StateGraph (4389 Zeilen!)
│   ├── state_integration.rs # Observer Pattern
│   └── engine.rs           # ExecutionContext Wrapper
│
├── protection/             # Protection Layer (Κ19-Κ21, Κ26-Κ28)
│   ├── anti_calcification.rs   # AntiCalcification (Κ19)
│   ├── adaptive_calibration.rs # Dynamische Parameter (§IX)
│   ├── diversity.rs            # DiversityMonitor (Κ20)
│   ├── quadratic.rs            # QuadraticGovernance (Κ21)
│   └── anomaly.rs              # AnomalyDetector
│
├── peer/                   # Peer Layer (Κ22-Κ24)
│   ├── gateway.rs          # GatewayGuard (591 Zeilen)
│   ├── saga_composer.rs    # SagaComposer (640 Zeilen)
│   ├── intent_parser.rs    # IntentParser
│   └── p2p/                # libp2p Netzwerk
│
├── eclvm/                  # ECLVM - Configuration Language VM (Κ25)
│   ├── runtime/vm.rs       # Stack-basierte VM (1416 Zeilen)
│   ├── parser.rs, compiler.rs, bytecode.rs
│   ├── mana.rs             # ManaManager
│   └── programmable_gateway.rs # Policy-Engine
│
├── local/                  # Dezentraler Storage Layer
│   ├── kv_store.rs         # Generic KV (Fjall-basiert)
│   ├── event_store.rs      # Event-DAG Persistence
│   ├── realm_storage.rs    # Per-Realm Stores
│   └── archive.rs          # Cold Storage (ψ_archive)
│
└── execution/              # Execution Layer (IPS ℳ)
    ├── context.rs          # ExecutionContext
    └── tracked.rs          # Tracked Execution
```

### 6-Schichten-Modell (Aktualisiert)

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│   LAYER 1: PEER LAYER (Κ22-Κ24) – peer/                                        │
│   ═════════════════════════════════════                                         │
│   • IntentParser:     Strukturierte/natürlichsprachliche Intents               │
│   • SagaComposer:     Intent → atomare Saga-Schritte (Κ22)                     │
│   • GatewayGuard:     Realm-Crossing-Validierung + Store-Init (Κ23)            │
│   • Trust Dampening:  𝕎_target = M_ctx × 𝕎_source (Κ24)                        │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   LAYER 2: CORE LOGIC LAYER (Κ2-Κ18) – core/                                   │
│   ══════════════════════════════════════════                                    │
│   • TrustEngine:      6D-Vektor, Asymmetrie 2×, Kombination (Κ2-Κ5)           │
│   • EventEngine:      DAG-Management, Kausalität, Finalität (Κ9-Κ12)          │
│   • SurprisalCalculator: Count-Min Sketch, Trust-Dämpfung (Κ15a)              │
│   • WorldFormulaEngine: 𝔼-Berechnung, Inkrementell (Κ15b-d)                    │
│   • ConsensusEngine:  Gewichteter Partition-Konsens (Κ18)                      │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   LAYER 3: ECLVM LAYER (Κ25) – eclvm/                                          │
│   ═══════════════════════════════════                                           │
│   • ECLVM:            Stack-basierte VM mit Gas-Metering                       │
│   • PolicyEngine:     ECL-Policies für Realm-Regeln                            │
│   • Blueprints:       Wiederverwendbare Policy-Templates                       │
│   • ManaManager:      Bandwidth-Ressourcen, Tiers                              │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   LAYER 4: STORAGE LAYER (Κ1, Κ6-Κ8) – local/                                  │
│   ═══════════════════════════════════════════                                   │
│   • DecentralizedStorage: Single-Binary Fjall-basiert                          │
│   • EventStore:       Immutable DAG Persistence                                │
│   • IdentityStore:    DIDs, Keys (Ed25519)                                     │
│   • RealmStorage:     Per-Realm dynamische Stores                              │
│   • Archive:          Cold Storage (ψ_archive Morphismus)                      │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   LAYER 5: PROTECTION LAYER (Κ19-Κ21) – protection/                            │
│   ═════════════════════════════════════════════════                             │
│   • AntiCalcification:     Diminishing Returns, Power-Cap (Κ19)               │
│   • AdaptiveCalibration:   PID-Controller für Parameter (§IX)                 │
│   • DiversityMonitor:      Shannon-Entropie, Monokultur-Check (Κ20)           │
│   • QuadraticGovernance:   vote_cost(n) = n² (Κ21)                            │
│   • AnomalyDetector:       Pattern-Erkennung, Vigilance                        │
│                                                                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│   LAYER 6: P2P NETWORK LAYER – peer/p2p/                                       │
│   ══════════════════════════════════════                                        │
│   • SwarmManager:     libp2p Swarm                                             │
│   • GossipSub:        Event-Propagation                                        │
│   • Kademlia:         Peer-Discovery                                           │
│   • SyncProtocol:     State-Synchronisation                                    │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 0. State-Graph: Komponentenverbindungen (Κ1-Κ28)

Das Erynoa-System verwendet einen expliziten **StateGraph** (definiert in `core/state.rs`, 4389 Zeilen), der alle Abhängigkeiten und Trigger-Ketten zwischen Komponenten modelliert. Dies ermöglicht automatische Event-Propagation und konsistente State-Updates.

### 0.1 Beziehungstypen

```rust
// Aus: backend/src/core/state.rs

/// Beziehungstyp zwischen State-Komponenten
pub enum StateRelation {
    /// A hängt kausal von B ab (A ← B)
    DependsOn,
    /// A triggert Updates in B (A → B)
    Triggers,
    /// A und B sind bidirektional verbunden (A ↔ B)
    Bidirectional,
    /// A aggregiert Daten aus B (A ⊃ B)
    Aggregates,
    /// A validiert B (A ✓ B)
    Validates,
}
```

### 0.2 Vollständiger Abhängigkeitsgraph

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   STATE GRAPH (aus state.rs)                                │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                             │
│   ╔═══════════════════════════════════════════════════════════════════════════════════╗   │
│   ║                          CORE LAYER (Κ2-Κ18)                                      ║   │
│   ║                                                                                   ║   │
│   ║   ┌─────────────┐   Triggers    ┌─────────────┐   Triggers   ┌─────────────┐    ║   │
│   ║   │   TRUST     │◄─────────────►│    EVENT    │◄────────────►│  CONSENSUS  │    ║   │
│   ║   │   (Κ2-Κ5)   │──────────────►│   (Κ9-Κ12)  │              │    (Κ18)    │    ║   │
│   ║   └──────┬──────┘               └──────┬──────┘              └──────┬──────┘    ║   │
│   ║          │ DependsOn                   │ DependsOn                  │ Validates ║   │
│   ║          │                             │                            │           ║   │
│   ║          └───────────┬─────────────────┴───────────────┬────────────┘           ║   │
│   ║                      │                                 │                         ║   │
│   ║                      ▼                                 ▼                         ║   │
│   ║               ┌─────────────────────────────────────────────┐                   ║   │
│   ║               │            WORLD FORMULA (Κ15b-d)           │                   ║   │
│   ║               │   𝔼 = Σ 𝔸(s) · σ⃗(‖𝕎(s)‖ · ln|ℂ(s)| · 𝒮(s)) │                   ║   │
│   ║               └─────────────────────────────────────────────┘                   ║   │
│   ╚═══════════════════════════════════════════════════════════════════════════════════╝   │
│                               │                                                           │
│                               │ DependsOn                                                 │
│                               ▼                                                           │
│   ╔═══════════════════════════════════════════════════════════════════════════════════╗   │
│   ║                         EXECUTION LAYER (IPS ℳ)                                   ║   │
│   ║                                                                                   ║   │
│   ║   ┌─────────────┐  Aggregates  ┌─────────────┐  Aggregates  ┌─────────────┐     ║   │
│   ║   │     GAS     │◄─────────────│  EXECUTION  │─────────────►│    MANA     │     ║   │
│   ║   │  (Compute)  │              │   Context   │              │ (Bandwidth) │     ║   │
│   ║   └──────┬──────┘              └──────┬──────┘              └──────┬──────┘     ║   │
│   ║          │                            │                            │            ║   │
│   ║          │ DependsOn                  │ Triggers                   │ DependsOn  ║   │
│   ║          └────────────────────────────┼────────────────────────────┘            ║   │
│   ║                                       ▼                                          ║   │
│   ║                                     EVENT                                        ║   │
│   ╚═══════════════════════════════════════════════════════════════════════════════════╝   │
│                               │                                                           │
│                               │ DependsOn (Gas, Mana, Trust)                              │
│                               ▼                                                           │
│   ╔═══════════════════════════════════════════════════════════════════════════════════╗   │
│   ║                          ECLVM LAYER (Κ25)                                        ║   │
│   ║                                                                                   ║   │
│   ║   ┌─────────────┐  DependsOn   ┌─────────────┐   Validates  ┌─────────────┐     ║   │
│   ║   │    ECLVM    │─────────────►│  ECLPolicy  │─────────────►│   GATEWAY   │     ║   │
│   ║   │   Runtime   │              │   Engine    │              │    (Κ23)    │     ║   │
│   ║   └──────┬──────┘              └──────┬──────┘              └─────────────┘     ║   │
│   ║          │ DependsOn                  │ Validates                                ║   │
│   ║          ▼                            ▼                                          ║   │
│   ║   ┌─────────────┐              ┌─────────────┐                                   ║   │
│   ║   │ ECLBlueprint│──Aggregates──│  BLUEPRINT  │                                   ║   │
│   ║   │   Manager   │              │   Storage   │                                   ║   │
│   ║   └─────────────┘              └─────────────┘                                   ║   │
│   ╚═══════════════════════════════════════════════════════════════════════════════════╝   │
│                               │                                                           │
│                               │ Validates, DependsOn                                      │
│                               ▼                                                           │
│   ╔═══════════════════════════════════════════════════════════════════════════════════╗   │
│   ║                         PEER LAYER (Κ22-Κ24)                                      ║   │
│   ║                                                                                   ║   │
│   ║   ┌─────────────┐  Aggregates  ┌─────────────┐  DependsOn   ┌─────────────┐     ║   │
│   ║   │   INTENT    │─────────────►│    SAGA     │─────────────►│   GATEWAY   │     ║   │
│   ║   │   PARSER    │              │  COMPOSER   │              │   GUARD     │     ║   │
│   ║   └──────┬──────┘              └──────┬──────┘              └──────┬──────┘     ║   │
│   ║          │ DependsOn                  │ DependsOn                  │ DependsOn  ║   │
│   ║          ▼                            ▼                            ▼            ║   │
│   ║       ECLPolicy                     ECLVM                        REALM          ║   │
│   ║          │                            │                            │            ║   │
│   ║          └────────────────────────────┼────────────────────────────┘            ║   │
│   ║                                       ▼                                          ║   │
│   ║   ┌───────────────────────────────────────────────────────────────────────┐     ║   │
│   ║   │                     REALM STATE (Per-Realm Isolation)                  │     ║   │
│   ║   │  • TrustVector per Identity per Realm                                  │     ║   │
│   ║   │  • Rules (ECL-Policies)                                                │     ║   │
│   ║   │  • Membership + Crossing-Metriken                                      │     ║   │
│   ║   └───────────────────────────────────────────────────────────────────────┘     ║   │
│   ╚═══════════════════════════════════════════════════════════════════════════════════╝   │
│                               │                                                           │
│                               │ Validates, Triggers                                       │
│                               ▼                                                           │
│   ╔═══════════════════════════════════════════════════════════════════════════════════╗   │
│   ║                        PROTECTION LAYER (Κ19-Κ21)                                 ║   │
│   ║                                                                                   ║   │
│   ║   ┌─────────────┐  Validates   ┌─────────────┐  Validates   ┌─────────────┐     ║   │
│   ║   │   ANOMALY   │─────────────►│    TRUST    │◄─────────────│  DIVERSITY  │     ║   │
│   ║   │  DETECTION  │              │             │              │   MONITOR   │     ║   │
│   ║   └─────────────┘              └──────┬──────┘              └──────┬──────┘     ║   │
│   ║                                       │ Validates                  │ Validates  ║   │
│   ║   ┌─────────────┐  Triggers    ┌──────▼──────┐              ┌──────▼──────┐     ║   │
│   ║   │ CALIBRATION │─────────────►│  ANTI-CALC  │              │  QUADRATIC  │     ║   │
│   ║   │ (PID-Ctrl)  │              │    (Κ19)    │              │   GOV (Κ21) │     ║   │
│   ║   └──────┬──────┘              └──────┬──────┘              └──────┬──────┘     ║   │
│   ║          │ Triggers                   │ Triggers                   │ DependsOn  ║   │
│   ║          ▼                            ▼                            ▼            ║   │
│   ║        GAS/MANA                     TRUST                        TRUST          ║   │
│   ╚═══════════════════════════════════════════════════════════════════════════════════╝   │
│                               │                                                           │
│                               │ Aggregates, DependsOn                                     │
│                               ▼                                                           │
│   ╔═══════════════════════════════════════════════════════════════════════════════════╗   │
│   ║                         P2P NETWORK LAYER                                         ║   │
│   ║                                                                                   ║   │
│   ║   ┌─────────────┐  Triggers    ┌─────────────┐  Aggregates  ┌─────────────┐     ║   │
│   ║   │    SWARM    │─────────────►│   GOSSIP    │─────────────►│    EVENT    │     ║   │
│   ║   │   Manager   │              │    SUB      │              │ Propagation │     ║   │
│   ║   └──────┬──────┘              └──────┬──────┘              └─────────────┘     ║   │
│   ║          │                            │ DependsOn                               ║   │
│   ║          ▼                            ▼                                          ║   │
│   ║   ┌─────────────┐              ┌─────────────┐              ┌─────────────┐     ║   │
│   ║   │  KADEMLIA   │              │    TRUST    │◄─Validates───│   PRIVACY   │     ║   │
│   ║   │    DHT      │              │  (Scoring)  │              │   Layer     │     ║   │
│   ║   └─────────────┘              └─────────────┘              └─────────────┘     ║   │
│   ╚═══════════════════════════════════════════════════════════════════════════════════╝   │
│                                                                                             │
└─────────────────────────────────────────────────────────────────────────────────────────────┘
```

### 0.3 Observer-Pattern (Event-Propagation)

Die Verbindungen werden durch **Observer-Traits** realisiert (definiert in `core/state_integration.rs`, 2986 Zeilen):

```rust
// Aus: backend/src/core/state_integration.rs

/// Trust Engine Observer - Propagiert Trust-Änderungen
pub trait TrustObserver: Send + Sync {
    fn on_trust_update(&self, from: &EntityId, to: &EntityId, old: f64, new: f64, positive: bool);
    fn on_entity_registered(&self, entity: &EntityId);
    fn on_relationship_created(&self, from: &EntityId, to: &EntityId);
    fn on_violation_detected(&self, entity: &EntityId, violation_type: &str);
}

/// Event Engine Observer - Propagiert DAG-Änderungen
pub trait EventObserver: Send + Sync {
    fn on_event_added(&self, event_id: &EventId, is_genesis: bool, parents_count: usize, depth: u64);
    fn on_event_finalized(&self, event_id: &EventId, latency_ms: u64);
    fn on_event_witnessed(&self, event_id: &EventId, witness: &EntityId);
    fn on_cycle_detected(&self, event_id: &EventId);
}

/// Execution Observer - Trackt Gas/Mana/Events
pub trait ExecutionObserver: Send + Sync {
    fn on_execution_start(&self, context_id: u64);
    fn on_execution_complete(&self, context_id: u64, success: bool, gas: u64, mana: u64, events: u64, duration_ms: u64);
    fn on_gas_consumed(&self, amount: u64);
    fn on_out_of_gas(&self, required: u64, available: u64);
    fn on_mana_consumed(&self, amount: u64);
    fn on_rate_limited(&self, entity: &EntityId);
}

/// Protection Observer - Meldet Anomalien und Interventionen
pub trait ProtectionObserver: Send + Sync {
    fn on_anomaly_detected(&self, severity: &str, description: &str);
    fn on_entropy_update(&self, dimension: &str, value: f64);
    fn on_monoculture_warning(&self, dimension: &str, concentration: f64);
    fn on_intervention(&self, entity: &EntityId, reason: &str);
    fn on_calibration_update(&self, param: &str, old_value: f64, new_value: f64);
}

/// ECLVM Observer - Tracks Policy/Blueprint Execution
pub trait ECLVMObserver: Send + Sync {
    fn on_policy_compiled(&self, policy_id: &str, policy_type: &str, bytecode_size: usize);
    fn on_policy_executed(&self, policy_id: &str, policy_type: &str, passed: bool, gas: u64, mana: u64, realm: Option<&str>);
    fn on_blueprint_published(&self, blueprint_id: &str, version: &str, author: &EntityId);
    fn on_blueprint_deployed(&self, blueprint_id: &str, realm_id: &str);
    fn on_saga_step_executed(&self, saga_id: &str, step: usize, total: usize, success: bool, gas: u64, mana: u64, cross_realm: bool);
}

/// Gateway Observer (Κ23) - Cross-Realm Tracking
pub trait GatewayObserver: Send + Sync {
    fn on_crossing_allowed(&self, entity: &EntityId, from: &str, to: &str, trust: f64);
    fn on_crossing_denied(&self, entity: &EntityId, from: &str, to: &str, reason: &str);
    fn on_realm_registered(&self, realm_id: &str);
    fn on_trust_dampened(&self, entity: &EntityId, original: f64, dampened: f64);
}

/// Realm Observer (Κ22-Κ24) - Per-Realm Events
pub trait RealmObserver: Send + Sync {
    fn on_realm_registered(&self, realm_id: &str, min_trust: f32, governance_type: &str);
    fn on_crossing_succeeded(&self, from: &str, to: &str);
    fn on_identity_joined_realm(&self, identity_id: &str, realm_id: &str);
    fn on_realm_trust_updated(&self, realm_id: &str, new_trust: f64);
    fn on_rule_added_to_realm(&self, realm_id: &str, rule_id: &str);
}
```

### 0.4 StateIntegrator - Der Verbindungs-Hub

```rust
// Aus: backend/src/core/state_integration.rs

/// StateIntegrator verbindet alle Observer mit UnifiedState
pub struct StateIntegrator {
    state: SharedUnifiedState,  // Arc<RwLock<UnifiedState>>
}

impl StateIntegrator {
    /// Event-Flow: Engine → Observer → StateIntegrator → UnifiedState → Cross-Module Triggers
    ///
    /// Beispiel: Trust-Update
    /// 1. TrustEngine.process_event() wird aufgerufen
    /// 2. TrustObserver.on_trust_update() feuert
    /// 3. StateIntegrator aktualisiert UnifiedState.core.trust
    /// 4. StateGraph prüft Trigger-Ketten (Trust → Event, Trust → WorldFormula)
    /// 5. Abhängige Module werden benachrichtigt
}
```

### 0.5 Datenfluss-Beispiel: Intent → Saga → Event → Trust

```
┌──────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                          │
│   USER                                                                                   │
│     │                                                                                    │
│     │ "Transfer 100 tokens to did:ery:bob"                                              │
│     ▼                                                                                    │
│   ┌────────────────────┐                                                                │
│   │   IntentParser     │  on_intent_parsed(type="transfer", success=true)               │
│   │                    │                                                                │
│   │  parse(text)       │──────────────────┐                                             │
│   │  ↓                 │                  │                                             │
│   │  Goal::Transfer {  │                  ▼                                             │
│   │    to: bob,        │           ┌──────────────┐                                     │
│   │    amount: 100     │           │ IntentParser │ (State-Update)                      │
│   │  }                 │           │    State     │                                     │
│   └────────┬───────────┘           └──────────────┘                                     │
│            │                                                                            │
│            │ Aggregates (SagaComposer ⊃ IntentParser)                                  │
│            ▼                                                                            │
│   ┌────────────────────┐                                                                │
│   │   SagaComposer     │  on_saga_composed(saga_id, steps=3, goal="transfer")           │
│   │                    │                                                                │
│   │  compose(goal)     │──────────────────┐                                             │
│   │  ↓                 │                  │                                             │
│   │  Saga {            │                  ▼                                             │
│   │    steps: [        │           ┌──────────────┐                                     │
│   │      Lock(100),    │           │ SagaComposer │ (State-Update)                      │
│   │      Transfer(...),│           │    State     │                                     │
│   │      Unlock()      │           └──────────────┘                                     │
│   │    ]               │                                                                │
│   │  }                 │                                                                │
│   └────────┬───────────┘                                                                │
│            │                                                                            │
│            │ DependsOn (Saga → ECLVM für Execution)                                     │
│            ▼                                                                            │
│   ┌────────────────────┐                                                                │
│   │      ECLVM         │  on_saga_step_executed(step=0, gas=150, mana=10)               │
│   │                    │                                                                │
│   │  execute_step()    │──────────────────┐                                             │
│   │  ↓                 │                  │                                             │
│   │  consume_gas(150)  │                  ▼                                             │
│   │  consume_mana(10)  │           ┌──────────────┐                                     │
│   │  emit_event(...)   │           │   ECLVM      │ (State-Update)                      │
│   │                    │           │    State     │                                     │
│   └────────┬───────────┘           └──────────────┘                                     │
│            │                                                                            │
│            │ Triggers (ECLVM → Event)                                                   │
│            ▼                                                                            │
│   ┌────────────────────┐                                                                │
│   │   EventEngine      │  on_event_added(id=ev123, parents=1, depth=42)                 │
│   │                    │                                                                │
│   │  add_event(...)    │──────────────────┐                                             │
│   │  ↓                 │                  │                                             │
│   │  validate_dag()    │                  ▼                                             │
│   │  check_parents()   │           ┌──────────────┐                                     │
│   │  update_depth()    │           │    Event     │ (State-Update)                      │
│   │                    │           │    State     │                                     │
│   └────────┬───────────┘           └──────────────┘                                     │
│            │                                                                            │
│            │ Triggers (Event → Trust, bidirectional)                                    │
│            ▼                                                                            │
│   ┌────────────────────┐                                                                │
│   │   TrustEngine      │  on_trust_update(from=alice, to=bob, old=0.5, new=0.55)        │
│   │                    │                                                                │
│   │  process_event()   │──────────────────┐                                             │
│   │  ↓                 │                  │                                             │
│   │  update_trust(     │                  ▼                                             │
│   │    dimension=i,    │           ┌──────────────┐                                     │
│   │    delta=+0.1      │           │    Trust     │ (State-Update)                      │
│   │  )                 │           │    State     │                                     │
│   └────────┬───────────┘           └──────────────┘                                     │
│            │                                                                            │
│            │ DependsOn (Trust → WorldFormula)                                           │
│            ▼                                                                            │
│   ┌────────────────────┐                                                                │
│   │  WorldFormulaEngine│  on_formula_computed(𝔼=42.3, activity=15, trust_norm=0.72)     │
│   │                    │                                                                │
│   │  update_contrib()  │──────────────────┐                                             │
│   │  ↓                 │                  │                                             │
│   │  𝔼 += delta        │                  ▼                                             │
│   │  (inkrementell!)   │           ┌──────────────┐                                     │
│   │                    │           │   Formula    │ (State-Update)                      │
│   │                    │           │    State     │                                     │
│   └────────────────────┘           └──────────────┘                                     │
│                                                                                          │
└──────────────────────────────────────────────────────────────────────────────────────────┘
```

### 0.6 Gas-Kosten Matrix (aus `core/engine.rs`)

```rust
// Aus: backend/src/core/engine.rs

/// Gas-Kosten für Event-Operationen
pub mod event_gas {
    pub const VALIDATE: u64 = 200;           // Event-Validierung
    pub const ADD_TO_DAG: u64 = 300;         // Hinzufügen zum DAG
    pub const PARENT_LOOKUP: u64 = 50;       // Pro Parent
    pub const CYCLE_CHECK: u64 = 100;        // Zyklus-Detection
    pub const SIGNATURE_VERIFY: u64 = 500;   // Signatur-Verifikation
}

/// Gas-Kosten für Trust-Operationen
pub mod trust_gas {
    pub const LOOKUP: u64 = 25;              // Trust-Lookup
    pub const UPDATE: u64 = 50;              // Eine Dimension updaten
    pub const COMBINE: u64 = 30;             // Kombination (Κ5)
    pub const CHAIN_TRUST_BASE: u64 = 40;    // Ketten-Trust Basis
    pub const CHAIN_TRUST_PER_HOP: u64 = 20; // Pro Hop
    pub const HISTORY_ENTRY: u64 = 15;       // History-Eintrag
}

/// Gas-Kosten für Weltformel-Operationen
pub mod formula_gas {
    pub const CONTRIBUTION: u64 = 150;       // Contribution-Berechnung
    pub const SURPRISAL: u64 = 80;           // Surprisal-Berechnung
    pub const SIGMOID: u64 = 20;             // Sigmoid-Aktivierung
    pub const AGGREGATE_PER_SUBJECT: u64 = 10; // Pro Subject
    pub const GLOBAL_COMPUTE: u64 = 500;     // Globale Berechnung
}
```

### 0.7 Mana-Kosten Matrix (aus `execution/mod.rs`)

```rust
// Aus: backend/src/execution/mod.rs

/// Standard-Mana-Kosten für verschiedene Operationen
pub mod mana_costs {
    pub const STORAGE_WRITE: u64 = 10;       // Storage-Write
    pub const P2P_BROADCAST: u64 = 50;       // P2P-Broadcast
    pub const DHT_LOOKUP: u64 = 5;           // DHT-Lookup
    pub const STORAGE_PER_KB: u64 = 1;       // Pro KB Storage
    pub const P2P_PER_KB: u64 = 2;           // Pro KB P2P
}

/// Standard-Gas-Kosten
pub mod gas_costs {
    pub const EVENT_EMIT: u64 = 100;         // Event-Emission
    pub const STORAGE_READ: u64 = 50;        // Storage-Read
    pub const STORAGE_WRITE: u64 = 200;      // Storage-Write
    pub const P2P_MESSAGE: u64 = 150;        // P2P-Message
    pub const TRUST_LOOKUP: u64 = 25;        // Trust-Lookup
    pub const SIGNATURE_VERIFY: u64 = 500;   // Signatur-Verifikation
    pub const HASH_COMPUTE: u64 = 10;        // Hash-Berechnung
    pub const STORAGE_PER_BYTE: u64 = 1;     // Pro Byte Storage
    pub const P2P_PER_BYTE: u64 = 2;         // Pro Byte P2P
}
```

### 0.8 Layer-zu-Layer Verbindungsübersicht

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                  ║
║   VOLLSTÄNDIGE LAYER-ZU-LAYER VERBINDUNGSMATRIX                                                                 ║
║                                                                                                                  ║
║   ┌──────────────┬───────────────┬───────────────┬───────────────┬───────────────┬───────────────┐              ║
║   │              │ PEER          │ CORE          │ ECLVM         │ STORAGE       │ PROTECTION    │              ║
║   ├──────────────┼───────────────┼───────────────┼───────────────┼───────────────┼───────────────┤              ║
║   │ PEER         │ IntentParser  │ GatewayGuard  │ SagaComposer  │ RealmStorage  │               │              ║
║   │              │ →SagaComposer │ →TrustEngine  │ →ECLVM        │ →stores_init  │               │              ║
║   │              │               │ (Trust-Check) │ (Saga-Exec)   │               │               │              ║
║   ├──────────────┼───────────────┼───────────────┼───────────────┼───────────────┼───────────────┤              ║
║   │ CORE         │ TrustEngine   │ Event↔Trust   │ EventEngine   │ EventStore    │ TrustEngine   │              ║
║   │              │ →GatewayGuard │ →WorldFormula │ →ECLVM        │ →TrustStore   │ →AntiCalcif.  │              ║
║   │              │ (crossing)    │ →Consensus    │ (Events)      │ →Archive      │ →Diversity    │              ║
║   ├──────────────┼───────────────┼───────────────┼───────────────┼───────────────┼───────────────┤              ║
║   │ ECLVM        │ ECLPolicy     │ ECLVM         │ Gas+Mana      │ ContentStore  │ Calibration   │              ║
║   │              │ →GatewayGuard │ →EventEngine  │ Meters        │ →Bytecode     │ →Gas/Mana     │              ║
║   │              │ (Rules)       │ (emit_event)  │               │ →Blueprints   │ (Preise)      │              ║
║   ├──────────────┼───────────────┼───────────────┼───────────────┼───────────────┼───────────────┤              ║
║   │ STORAGE      │ RealmStorage  │ EventStore    │ ContentStore  │ Fjall KS      │ PowerHistory  │              ║
║   │              │ →GatewayGuard │ →EventEngine  │ →ECLVM        │ Partitions    │ →AntiCalcif.  │              ║
║   │              │ (stores)      │ (persist)     │ (load)        │               │               │              ║
║   ├──────────────┼───────────────┼───────────────┼───────────────┼───────────────┼───────────────┤              ║
║   │ PROTECTION   │               │ AntiCalcif.   │ Calibration   │ PowerHistory  │ Calibration   │              ║
║   │              │               │ →TrustEngine  │ →Gas/Mana     │ →Archive      │ →Diversity    │              ║
║   │              │               │ (power-caps)  │ (Preise)      │               │ →Quadratic    │              ║
║   └──────────────┴───────────────┴───────────────┴───────────────┴───────────────┴───────────────┘              ║
║                                                                                                                  ║
║   KRITISCHE PFADE (Latenz-sensitiv):                                                                            ║
║                                                                                                                  ║
║   1. User Intent → Event:  Peer → Core → Storage                                                                ║
║      IntentParser → SagaComposer → ECLVM → EventEngine → EventStore                                             ║
║      Latenz-Ziel: <100ms                                                                                         ║
║                                                                                                                  ║
║   2. Cross-Realm Crossing: Peer → Core → ECLVM → Storage                                                        ║
║      GatewayGuard → TrustEngine → ECLPolicy → RealmStorage                                                      ║
║      Latenz-Ziel: <200ms (inkl. Policy-Eval)                                                                     ║
║                                                                                                                  ║
║   3. Trust Update: Core → Core → Protection                                                                     ║
║      EventEngine → TrustEngine → WorldFormula → AntiCalcification                                               ║
║      Latenz-Ziel: <50ms                                                                                          ║
║                                                                                                                  ║
║   4. Consensus: Core → Protection → Core                                                                        ║
║      ConsensusEngine → DiversityMonitor → QuadraticGovernance                                                   ║
║      Latenz-Ziel: <500ms (Abstimmungen)                                                                          ║
║                                                                                                                  ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

## I. Peer Layer (Κ22-Κ24) – `peer/`

### 1.0 Peer Layer Interne Verbindungen

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                      ║
║   PEER LAYER – INTERNE VERBINDUNGS-MATRIX                                                           ║
║                                                                                                      ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                                             │   ║
║   │    IntentParser ════════════════════════► SagaComposer ═══════════════► ECLVM (Execution) │   ║
║   │         │         parse_transfer()            │        compose()            ▲              │   ║
║   │         │         parse_delegation()          │                             │              │   ║
║   │         │         parse_structured()          │                             │              │   ║
║   │         │                                     │                             │              │   ║
║   │         │                                     ▼                             │              │   ║
║   │         │                              ┌─────────────┐                      │              │   ║
║   │         │                              │   Saga      │                      │              │   ║
║   │         │                              │   Steps[]   │──────────────────────┘              │   ║
║   │         │                              │   - Lock    │                                     │   ║
║   │         │                              │   - Transfer│                                     │   ║
║   │         │                              │   - Mint    │                                     │   ║
║   │         │                              │   - Burn    │                                     │   ║
║   │         │                              │   - WaitFor │                                     │   ║
║   │         │                              └──────┬──────┘                                     │   ║
║   │         │                                     │                                            │   ║
║   │         │                                     │ on_crossing()                              │   ║
║   │         │                                     ▼                                            │   ║
║   │         │                              ┌─────────────┐        validate_crossing()         │   ║
║   │         └─────────────────────────────►│ GatewayGuard │◄──────────────────────────────────┤   ║
║   │                   parse_attest()       │             │                                     │   ║
║   │                                        │  Trust-Check │────────► TrustEngine (Core)        │   ║
║   │                                        │  Rule-Check  │────────► ECLPolicy (ECLVM)         │   ║
║   │                                        │  Cred-Check  │                                     │   ║
║   │                                        │  Dampening   │────────► TrustDampeningMatrix      │   ║
║   │                                        └──────┬───────┘                                    │   ║
║   │                                               │                                            │   ║
║   │                                               │ stores_to_initialize                       │   ║
║   │                                               ▼                                            │   ║
║   │                                        ┌──────────────┐                                    │   ║
║   │                                        │ RealmStorage │────────► Local Layer               │   ║
║   │                                        │ (Personal    │                                    │   ║
║   │                                        │  Stores)     │                                    │   ║
║   │                                        └──────────────┘                                    │   ║
║   │                                                                                            │   ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────┘   ║
║                                                                                                      ║
║   DATENFLUSS:                                                                                       ║
║                                                                                                      ║
║   User Input ─► IntentParser.parse_*() ─► Intent ─► SagaComposer.compose() ─► Saga               ║
║                         │                                      │                                    ║
║                         │ (wenn cross-realm)                   │ (mit Compensations Κ24)            ║
║                         ▼                                      ▼                                    ║
║                  GatewayGuard.validate()              ECLVMState.execute_saga()                    ║
║                         │                                      │                                    ║
║                         │ CrossingResult                       │ SagaExecution                      ║
║                         ▼                                      ▼                                    ║
║                  - allowed: bool                         - step_results[]                           ║
║                  - dampened_trust                        - compensations[]                          ║
║                  - violations[]                          - final_state                              ║
║                  - stores_to_initialize[]                                                           ║
║                                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 1.1 Intent Parser (`intent_parser.rs`)

Der Intent Parser interpretiert Nutzeranfragen und transformiert sie in strukturierte Intents.

```rust
// Aus: backend/src/peer/intent_parser.rs

/// Intent Parser (Κ22)
pub struct IntentParser {
    /// Pattern-Matcher für natürlichsprachliche Intents
    patterns: Vec<PatternMatcher>,
    /// Konfiguration
    config: IntentParserConfig,
}

#[derive(Debug, Clone)]
pub struct IntentParserConfig {
    /// Default Timeout in Stunden
    pub default_timeout_hours: u64,     // 24
    /// Default Realm
    pub default_realm: RealmId,         // ROOT_REALM_ID
    /// Maximale Constraints pro Intent
    pub max_constraints: usize,         // 10
}

/// Unterstützte Goal-Typen
pub enum Goal {
    Transfer { to: UniversalId, amount: u64, asset_type: String },
    Attest { subject: UniversalId, claim: String },
    Delegate { to: UniversalId, capabilities: Vec<String>, trust_factor: f64, ttl_seconds: u64 },
    Query { predicate: String },
    Create { entity_type: String, params: HashMap<String, Value> },
    Complex { description: String, sub_goals: Vec<Goal> },
}
```

### 1.2 Saga Composer (`saga_composer.rs`) – Κ22, Κ24

Der Saga Composer zerlegt komplexe Intents in atomare, kompensierbare Schritte.

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   SAGA COMPOSER – INTENT → SAGA (Κ22)                                         ║
║                                                                                ║
║   INPUT: Intent { goal: Transfer(Alice → Bob, 100 ERY) }                      ║
║                                                                                ║
║   OUTPUT: Saga [                                                              ║
║       Step 0: Lock(Alice, 100, "ERY")     | Compensation: Unlock(lock_id)     ║
║       Step 1: Transfer(Alice → Bob, 100)  | Dependencies: [0]                 ║
║   ]                                                                           ║
║                                                                                ║
║   AXIOM Κ24: fail(Sᵢ) → compensate(S₁..Sᵢ₋₁)                                 ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

```rust
// Aus: backend/src/peer/saga_composer.rs

/// Saga Composer (Κ22, Κ24)
pub struct SagaComposer {
    config: SagaComposerConfig,
}

#[derive(Debug, Clone)]
pub struct SagaComposerConfig {
    pub default_lock_duration: u64,  // 3600 (1 Stunde)
    pub auto_compensation: bool,     // true
    pub max_steps: usize,            // 20
}

/// Saga-Aktionen
pub enum SagaAction {
    Lock { owner: UniversalId, amount: u64, asset_type: String, lock_id: Option<UniversalId>, release_conditions: Vec<String> },
    Unlock { lock_id: UniversalId, to: Option<UniversalId> },
    Transfer { from: UniversalId, to: UniversalId, amount: u64, asset_type: String },
    Mint { to: UniversalId, amount: u64, asset_type: String, authorization: Option<String> },
    Burn { from: UniversalId, amount: u64, asset_type: String, authorization: Option<String> },
    WaitFor { timeout_lamport: u64, condition: String, timeout_seconds: u64 },
}

impl SagaComposer {
    /// Κ22: Komponiere Saga aus Intent
    pub fn compose(&self, intent: &Intent) -> CompositionResult<Saga> {
        let steps = match &intent.goal {
            Goal::Transfer { to, amount, asset_type } =>
                self.compose_transfer(intent.source_did(), to, *amount, asset_type)?,
            Goal::Attest { subject, claim } =>
                self.compose_attest(intent.source_did(), subject, claim)?,
            Goal::Delegate { to, capabilities, ttl_seconds, .. } =>
                self.compose_delegate(intent.source_did(), to, capabilities, *ttl_seconds)?,
            Goal::Query { predicate } =>
                self.compose_query(intent.source_did(), predicate)?,
            Goal::Create { entity_type, params } =>
                self.compose_create(intent.source_did(), entity_type, params)?,
            Goal::Complex { description, sub_goals } =>
                self.compose_complex(intent.source_did(), description, sub_goals)?,
        };

        self.validate_constraints(&steps, &intent.constraints)?;
        Ok(Saga::from_intent(intent, steps, 0))
    }
}
```

### 1.3 Gateway Guard (`gateway.rs`) – Κ23

Der Gateway Guard validiert Realm-Übergänge, erzwingt Regeln und initialisiert automatisch Personal-Stores.

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   GATEWAY GUARD – REALM-CROSSING + STORE-INITIALISIERUNG (Κ23)                ║
║                                                                                ║
║   guard(user, target_realm) = ∧ᵢ Predicateᵢ(user, target_realm.rules)         ║
║                                                                                ║
║   PRÄDIKATE:                                                                  ║
║   ┌─────────────────────┬──────────────────────────────────────────────────┐  ║
║   │ Prädikat            │ Beschreibung                                     │  ║
║   ├─────────────────────┼──────────────────────────────────────────────────┤  ║
║   │ trust_norm_check    │ ‖𝕎(u)‖ ≥ target.min_trust                       │  ║
║   │ credential_check    │ u.credentials ⊇ target.required_creds            │  ║
║   │ apply_dampening     │ 𝕎_target = M_ctx × 𝕎_source (Κ24)               │  ║
║   └─────────────────────┴──────────────────────────────────────────────────┘  ║
║                                                                                ║
║   BEI ERFOLG:                                                                 ║
║   • Personal-Stores werden automatisch initialisiert                         ║
║   • Optional: Initial-Setup-Policy (ECL) wird ausgeführt                     ║
║                                                                                ║
╚════════════════════════════════════════════════════════════════════════════════╝
```

```rust
// Aus: backend/src/peer/gateway.rs

/// Gateway Guard (Κ23)
pub struct GatewayGuard {
    realms: HashMap<RealmId, RealmEntry>,
    trust_vectors: HashMap<UniversalId, TrustVector6D>,
    credentials: HashMap<UniversalId, Vec<String>>,
    config: GatewayConfig,
}

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub default_min_trust: f64,      // 0.3
    pub apply_trust_dampening: bool, // true
    pub verbose: bool,               // false
}

/// Ergebnis einer Gateway-Prüfung
#[derive(Debug, Clone)]
pub struct CrossingResult {
    pub allowed: bool,
    pub from_realm: RealmId,
    pub to_realm: RealmId,
    pub did: DID,
    pub original_trust: TrustVector6D,
    pub dampened_trust: TrustVector6D,
    pub violations: Vec<String>,
    /// Store-Templates die initialisiert werden sollen
    pub stores_to_initialize: Vec<StoreTemplate>,
    /// ECL-Policy die ausgeführt werden soll
    pub setup_policy: Option<String>,
}

impl GatewayGuard {
    /// Κ23: Validiere Realm-Crossing mit Store-Initialisierung
    pub fn validate_crossing(
        &self,
        did: &DID,
        from_realm: &RealmId,
        to_realm: &RealmId,
    ) -> GatewayResult<CrossingResult> {
        let mut violations = Vec::new();
        let target = self.realms.get(to_realm)?;
        let trust = self.trust_vectors.get(&did.id)?;

        // 1. Trust-Check
        let trust_norm = trust.weighted_norm(&[1.0; 6]) as f64;
        if trust_norm < target.min_trust {
            violations.push(format!("Insufficient trust: {} < {}", trust_norm, target.min_trust));
        }

        // 2. Credentials-Check
        let did_credentials = self.credentials.get(&did.id).map(|c| c.as_slice()).unwrap_or(&[]);
        for required in &target.required_credentials {
            if !did_credentials.contains(required) {
                violations.push(format!("Missing credential: {}", required));
            }
        }

        // 3. Κ24: Trust-Dämpfung
        let dampened = if self.config.apply_trust_dampening {
            TrustDampeningMatrix::generic_crossing(0.7).apply(trust)
        } else {
            trust.clone()
        };

        let allowed = violations.is_empty();

        // 4. Store-Initialisierung vorbereiten
        let (stores_to_initialize, setup_policy) = if allowed {
            (target.personal_store_templates.clone(), target.initial_setup_policy.clone())
        } else {
            (Vec::new(), None)
        };

        Ok(CrossingResult { allowed, from_realm: from_realm.clone(), to_realm: to_realm.clone(),
            did: did.clone(), original_trust: trust.clone(), dampened_trust: dampened,
            violations, stores_to_initialize, setup_policy })
    }
}
```

### 1.4 Trust-Dämpfungs-Matrix (Κ24)

```rust
// Aus: backend/src/domain/unified/trust.rs

pub struct TrustDampeningMatrix {
    coefficients: [[f32; 6]; 6],
}

impl TrustDampeningMatrix {
    /// Κ24: Generic Crossing Dampening (‖M_ctx‖ ≤ 1)
    pub fn generic_crossing(factor: f32) -> Self {
        Self {
            coefficients: [
                [factor, 0.0, 0.0, 0.0, 0.0, 0.0],
                [0.0, factor, 0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, factor, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, factor, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0, factor, 0.0],
                [0.0, 0.0, 0.0, 0.0, 0.0, factor],
            ],
        }
    }

    /// Anwenden: 𝕎_target = M_ctx × 𝕎_source
    pub fn apply(&self, source: &TrustVector6D) -> TrustVector6D {
        let s = [source.r, source.i, source.c, source.p, source.v, source.omega];
        let mut result = [0.0f32; 6];
        for i in 0..6 {
            result[i] = (0..6).map(|j| self.coefficients[i][j] * s[j]).sum();
        }
        TrustVector6D { r: result[0], i: result[1], c: result[2],
            p: result[3], v: result[4], omega: result[5] }
    }
}
```

---

## II. Core Logic Layer (Κ2-Κ18) – `core/`

### 2.0 Core Layer Interne Verbindungen

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                      ║
║   CORE LAYER – INTERNE VERBINDUNGS-MATRIX                                                           ║
║                                                                                                      ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                                             │   ║
║   │                           ┌────────────────────────────────────────────────────────────┐   │   ║
║   │                           │                TrustEngine (Κ2-Κ5)                         │   │   ║
║   │                           │                                                            │   │   ║
║   │    ┌──────────────────────┤  trust_vectors: HashMap<UniversalId, TrustVector6D>       │   │   ║
║   │    │                      │  relationships: HashMap<UniversalId, HashMap<...>>        │   │   ║
║   │    │                      │                                                            │   │   ║
║   │    │                      │  ┌─────────────────────────────────────────────────────┐   │   │   ║
║   │    │                      │  │ initialize_trust() ─────────────────────────────────│───│───┼──►║
║   │    │                      │  │ process_event()    ◄──────────────────────┐         │   │   │   ║
║   │    │                      │  │ combine_trust()    │ Κ5                   │         │   │   │   ║
║   │    │                      │  │ chain_trust()      │ Τ1: trust × ... × t  │         │   │   │   ║
║   │    │                      │  │ contextual_norm()  │                      │         │   │   │   ║
║   │    │  TrustObserver       │  └──────────┬────────────────────────────────┘         │   │   │   ║
║   │    │  on_trust_update()   │             │                                           │   │   │   ║
║   │    │                      └─────────────┼───────────────────────────────────────────┘   │   ║
║   │    │                                    │                                               │   ║
║   │    │                                    │ Triggers (Event ◄──► Trust, bidirektional)   │   ║
║   │    │                                    ▼                                               │   ║
║   │    │                      ┌────────────────────────────────────────────────────────────┐│   ║
║   │    │                      │                EventEngine (Κ9-Κ12)                        ││   ║
║   │    │                      │                                                            ││   ║
║   │    │                      │  events: HashMap<EventId, Event>                          ││   ║
║   │    │                      │  children_index: HashMap<EventId, HashSet<EventId>>       ││   ║
║   │    │                      │  genesis_events: HashSet<EventId>                         ││   ║
║   │    │  EventObserver       │                                                            ││   ║
║   │    │  on_event_added()    │  ┌─────────────────────────────────────────────────────┐   ││   ║
║   │    │  on_cycle_detected() │  │ validate_event()   │ Gas: 200 + 50/parent + 100 cyc│   ││   ║
║   │    │                      │  │ add_event()        │                                │   ││   ║
║   │    └──────────────────────│  │ find_witnesses()   │ min: 3, threshold: 0.5        │   ││   ║
║   │                           │  │ check_dag_props()  │ Κ10: no cycles (BFS)          │   ││   ║
║   │                           │  │ compute_depth()    │ Κ11: depth = max(p) + 1       │   ││   ║
║   │                           │  └───────────┬────────────────────────────────────────┘   ││   ║
║   │                           └──────────────┼────────────────────────────────────────────┘│   ║
║   │                                          │                                             │   ║
║   │                                          │ DependsOn (Event, Trust → WorldFormula)    │   ║
║   │                                          ▼                                             │   ║
║   │                           ┌────────────────────────────────────────────────────────────┐│   ║
║   │                           │              WorldFormulaEngine (Κ15-Κ17)                  ││   ║
║   │                           │                                                            ││   ║
║   │                           │  contributions: HashMap<UniversalId, Contribution>        ││   ║
║   │                           │  cached_total_e: f64  (O(1) access!)                      ││   ║
║   │                           │                                                            ││   ║
║   │  FormulaObserver          │  ┌─────────────────────────────────────────────────────┐   ││   ║
║   │  on_formula_computed()    │  │ 𝔼 = Σ 𝔸(s) · σ⃗(‖𝕎(s)‖ · ln|ℂ(s)| · 𝒮(s)) · Ĥ · w   │   ││   ║
║   │                           │  │                                                     │   ││   ║
║   │                           │  │ update_contribution()  │ Gas: 150 contrib           │   ││   ║
║   │                           │  │ get_cached_global()    │ O(1) amortisiert           │   ││   ║
║   │                           │  │                        │                            │   ││   ║
║   │                           │  │ ┌───────────────────────────────────────────────┐   │   ││   ║
║   │                           │  │ │ SurprisalCalculator (Count-Min Sketch 1024×5) │   │   ││   ║
║   │                           │  │ │ 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)  │ Gas: 80            │   │   ││   ║
║   │                           │  │ └───────────────────────────────────────────────┘   │   ││   ║
║   │                           │  └─────────────────────────────────────────────────────┘   ││   ║
║   │                           └────────────────────────────────────────────────────────────┘│   ║
║   │                                          │                                             │   ║
║   │                                          │ Validates (Consensus ✓ Event)              │   ║
║   │                                          ▼                                             │   ║
║   │                           ┌────────────────────────────────────────────────────────────┐│   ║
║   │                           │               ConsensusEngine (Κ18)                        ││   ║
║   │                           │                                                            ││   ║
║   │                           │  threshold: f64 = 2/3                                     ││   ║
║   │                           │                                                            ││   ║
║   │                           │  ┌─────────────────────────────────────────────────────┐   ││   ║
║   │                           │  │ Ψ(Σ)(φ) = Σ 𝕎(s)·[s ⊢ φ] / Σ 𝕎(s)                 │   ││   ║
║   │                           │  │                                                     │   ││   ║
║   │                           │  │ compute_consensus()  │ trust_weighted voting       │   ││   ║
║   │                           │  │ verify_partition()   │ θ = 2/3 threshold           │   ││   ║
║   │                           │  └─────────────────────────────────────────────────────┘   ││   ║
║   │                           └────────────────────────────────────────────────────────────┘│   ║
║   │                                                                                         │   ║
║   └─────────────────────────────────────────────────────────────────────────────────────────┘   ║
║                                                                                                      ║
║   DATEN-ABHÄNGIGKEITEN (aus StateGraph):                                                            ║
║                                                                                                      ║
║   Trust ◄──Triggers──► Event        (bidirektional: Events ändern Trust, Trust validiert Events)    ║
║   Trust ──DependsOn──► WorldFormula (𝕎 fließt in 𝔼-Berechnung)                                      ║
║   Event ──DependsOn──► WorldFormula (Activity, CausalHistory fließen in 𝔼)                          ║
║   Event ──Validates──► Consensus    (Events brauchen Consensus-Witness)                             ║
║   Consensus ──Validates──► Event    (Consensus basiert auf Event-DAG)                               ║
║                                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 2.1 TrustEngine (`trust_engine.rs`) – Κ2-Κ5

Die Trust Engine implementiert das 6-dimensionale Vertrauensmodell mit asymmetrischer Evolution.

```rust
// Aus: backend/src/core/trust_engine.rs (755 Zeilen)

/// Trust Engine - berechnet und aktualisiert Trust-Vektoren (Κ2-Κ5)
pub struct TrustEngine {
    /// Trust-Vektoren pro UniversalId (DID.id)
    trust_vectors: HashMap<UniversalId, TrustVector6D>,
    /// Trust-Beziehungen (from → to → context → trust)
    relationships: HashMap<UniversalId, HashMap<UniversalId, HashMap<ContextType, f64>>>,
    /// Konfiguration
    config: TrustEngineConfig,
}

/// Konfiguration (aus Simulation optimiert)
#[derive(Debug, Clone)]
pub struct TrustEngineConfig {
    pub default_trust: f64,          // Κ2: 0.5
    pub positive_rate: f64,          // 0.1
    pub negative_rate: f64,          // Κ4: 0.2 (2× positive)
    pub interaction_threshold: f64,  // 0.3
}

impl TrustEngine {
    /// Κ2: Initialisiere Trust für neue Entität
    pub fn initialize_trust(&mut self, id: &UniversalId) {
        if !self.trust_vectors.contains_key(id) {
            self.trust_vectors.insert(id.clone(), TrustVector6D::default());
            // 𝕎₀ = (0.5, 0.5, 0.5, 0.5, 0.5, 0.5)
        }
    }

    /// Κ4: Aktualisiere Trust basierend auf Event
    pub fn process_event(&mut self, event: &Event) -> TrustResult<()> {
        self.initialize_trust(&event.author);

        if let Some(dimension) = event.primary_trust_dimension() {
            let delta = if event.is_negative_trust() {
                -self.config.negative_rate  // Κ4: 2× schneller bei negativ
            } else {
                self.config.positive_rate
            };

            if let Some(trust) = self.trust_vectors.get_mut(&event.author) {
                trust.update(dimension, delta as f32);
            }
        }
        Ok(())
    }

    /// Κ5: Kombiniere Trust aus mehreren Quellen
    /// 𝕎_comb = 1 − ∏(1 − 𝕎ⱼ)
    pub fn combine_trust(&self, sources: &[(UniversalId, f64)]) -> f64 {
        let trusts: Vec<f32> = sources.iter().map(|(_, t)| *t as f32).collect();
        TrustCombination::combine_all(&trusts) as f64
    }

    /// Τ1: Ketten-Trust über mehrere Hops
    pub fn chain_trust(&self, chain: &[UniversalId], context: ContextType) -> f64 {
        if chain.len() < 2 { return 1.0; }
        let mut trusts = Vec::new();
        for window in chain.windows(2) {
            let trust = self.get_direct_trust(&window[0], &window[1], context)
                .unwrap_or(self.config.default_trust);
            trusts.push(trust as f32);
        }
        TrustCombination::chain_trust(&trusts) as f64
    }

    /// Berechne gewichtete Trust-Norm für Kontext
    pub fn contextual_trust_norm(&self, id: &UniversalId, context: ContextType) -> f32 {
        self.trust_vectors.get(id)
            .map(|t| t.weighted_norm(&context.weights()))
            .unwrap_or(self.config.default_trust as f32)
    }
}
```

### 2.2 EventEngine (`event_engine.rs`) – Κ9-Κ12

Die Event Engine verwaltet den kausalen DAG und garantiert alle DAG-Invarianten.

```rust
// Aus: backend/src/core/event_engine.rs (733 Zeilen)

/// Event Engine - verarbeitet und validiert Events im DAG (Κ9-Κ12)
pub struct EventEngine {
    /// In-Memory Event-Index
    events: HashMap<EventId, Event>,
    /// Kinder-Index (child → parents)
    children_index: HashMap<EventId, HashSet<EventId>>,
    /// Genesis-Events (keine Parents)
    genesis_events: HashSet<EventId>,
    /// Konfiguration
    config: EventEngineConfig,
}

#[derive(Debug, Clone)]
pub struct EventEngineConfig {
    pub min_witnesses: usize,          // 3
    pub witness_trust_threshold: f64,  // 0.5
    pub max_parents: usize,            // 10
}

impl EventEngine {
    /// Κ9: Validiere Event-Struktur (DAG-Integrität)
    pub fn validate_structure(&self, event: &Event) -> EventResult<()> {
        // Prüfe ob Parents existieren
        for parent_id in &event.parents {
            if !self.events.contains_key(parent_id) {
                return Err(EventError::ParentNotFound(parent_id.clone()));
            }
        }
        // Prüfe auf Zyklen
        if self.would_create_cycle(&event.id, &event.parents) {
            return Err(EventError::CycleDetected);
        }
        // Prüfe max Parents
        if event.parents.len() > self.config.max_parents {
            return Err(EventError::InvalidPayload(format!(
                "Too many parents: {} > {}", event.parents.len(), self.config.max_parents)));
        }
        Ok(())
    }

    /// Κ12: Füge Event zum DAG hinzu
    pub fn add_event(&mut self, event: Event) -> EventResult<EventId> {
        if self.events.contains_key(&event.id) {
            return Err(EventError::DuplicateEvent(event.id.clone()));
        }
        self.validate_structure(&event)?;

        let event_id = event.id.clone();

        // Update Children-Index
        for parent_id in &event.parents {
            self.children_index.entry(parent_id.clone()).or_default().insert(event_id.clone());
        }

        // Genesis-Event?
        if event.parents.is_empty() {
            self.genesis_events.insert(event_id.clone());
        }

        self.events.insert(event_id.clone(), event);
        Ok(event_id)
    }

    /// Prüft ob das Hinzufügen eines Events einen Zyklus erzeugen würde
    fn would_create_cycle(&self, event_id: &EventId, parents: &[EventId]) -> bool {
        for parent_id in parents {
            let mut visited = HashSet::new();
            let mut queue = vec![parent_id.clone()];
            while let Some(current) = queue.pop() {
                if &current == event_id { return true; }
                if visited.contains(&current) { continue; }
                visited.insert(current.clone());
                if let Some(children) = self.children_index.get(&current) {
                    for child in children { queue.push(child.clone()); }
                }
            }
        }
        false
    }
}
```

### 2.3 SurprisalCalculator (`surprisal.rs`) – Κ15a, Κ15d

```rust
// Aus: backend/src/core/surprisal.rs (334 Zeilen)

/// Surprisal Calculator - berechnet Information-Surprisal (Κ15a, Κ15d)
pub struct SurprisalCalculator {
    /// Count-Min Sketch für Event-Frequenzen (Κ15d: Approximation)
    sketch: CountMinSketch,
    /// Total Events gezählt
    total_count: u64,
    /// Event-Typ Zähler
    type_counts: HashMap<String, u64>,
}

impl SurprisalCalculator {
    /// Κ15a: Berechne Shannon-Surprisal für ein Event
    /// ℐ(e|s) = −log₂ P(e | ℂ(s))
    pub fn calculate_surprisal(&self, event: &Event) -> f64 {
        let event_key = self.event_to_key(event);
        let frequency = self.sketch.estimate(&event_key) as f64;
        let total = self.total_count.max(1) as f64;

        // Laplace smoothing
        let probability = (frequency + 1.0) / (total + 2.0);

        // Shannon-Surprisal in bits
        -probability.log2()
    }

    /// Κ15a: Trust-gedämpfte Surprisal
    /// 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)
    pub fn calculate_dampened_surprisal(&self, event: &Event, trust: &TrustVector6D) -> Surprisal {
        let raw = self.calculate_surprisal(event);
        let norm = trust.weighted_norm(&[1.0; 6]);

        Surprisal {
            raw_bits: raw,
            trust_norm: norm,
            event_id: None,
            computed_at: TemporalCoord::default(),
        }
    }

    /// Beobachte ein Event (update Frequenz-Schätzung)
    pub fn observe(&mut self, event: &Event) {
        let key = self.event_to_key(event);
        self.sketch.increment(&key);
        self.total_count += 1;
    }
}

/// Count-Min Sketch (Κ15d) - Probabilistische Frequenz-Schätzung
pub struct CountMinSketch {
    table: Vec<Vec<u64>>,
    width: usize,   // 1024 buckets
    depth: usize,   // 5 hash functions
    seeds: Vec<u64>,
}
```

### 2.4 WorldFormulaEngine (`world_formula.rs`) – Κ15b-d

```rust
// Aus: backend/src/core/world_formula.rs (727 Zeilen)

/// World Formula Engine - berechnet 𝔼 (Κ15b-d)
///
/// 𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)
pub struct WorldFormulaEngine {
    /// Contributions pro UniversalId
    contributions: HashMap<UniversalId, WorldFormulaContribution>,
    /// Letzte Berechnung
    last_computed: Option<WorldFormulaStatus>,
    /// Konfiguration
    config: WorldFormulaConfig,

    // Cached global state für inkrementelle Updates (Performance!)
    cached_total_e: f64,
    cached_total_activity: f64,
    cached_total_trust_norm: f64,
    cached_human_verified: usize,
}

#[derive(Debug, Clone)]
pub struct WorldFormulaConfig {
    pub activity_window_days: u64,    // τ = 90 Tage
    pub activity_threshold: u64,      // κ = 10
    pub temporal_decay_rate: f64,     // 0.99
    pub default_context: ContextType,
}

impl WorldFormulaEngine {
    /// Inkrementelles Update (O(1) amortisiert)
    pub fn update_contribution(
        &mut self,
        did: DID,
        trust: TrustVector6D,
        recent_events: u64,
        causal_history_size: u64,
        surprisal: Surprisal,
        human_factor: HumanFactor,
    ) {
        // Alten Beitrag abziehen (wenn vorhanden)
        if let Some(old) = self.contributions.get(&did.id) {
            self.cached_total_e -= old.compute();
            // ... weitere Felder
        }

        // Neuen Beitrag berechnen und addieren
        let contribution = WorldFormulaContribution::new(did.id.clone(), 0)
            .with_activity(Activity { recent_events, tau_seconds: self.config.activity_window_days * 86400,
                kappa: self.config.activity_threshold, computed_at: TemporalCoord::default() })
            .with_trust(&trust)
            .with_causal_history(causal_history_size)
            .with_surprisal(surprisal)
            .with_human_factor(human_factor);

        let new_e = contribution.compute();
        self.cached_total_e += new_e;
        self.contributions.insert(did.id, contribution);
    }

    /// O(1) Zugriff auf gecachten globalen State
    pub fn get_cached_global(&self) -> WorldFormulaStatus {
        let entity_count = self.contributions.len() as u64;
        WorldFormulaStatus {
            total_e: self.cached_total_e,
            delta_24h: self.last_computed.as_ref()
                .map(|prev| self.cached_total_e - prev.total_e).unwrap_or(0.0),
            entity_count,
            avg_activity: self.cached_total_activity / entity_count.max(1) as f64,
            avg_trust_norm: self.cached_total_trust_norm / entity_count.max(1) as f64,
            human_verified_ratio: self.cached_human_verified as f64 / entity_count.max(1) as f64,
            realm_id: None,
            computed_at: TemporalCoord::default(),
        }
    }
}
```

### 2.5 ConsensusEngine (`consensus.rs`) – Κ18

```rust
// Aus: backend/src/core/consensus.rs

/// Consensus Engine - Gewichteter Partition-Konsens (Κ18)
pub struct ConsensusEngine {
    trust_engine: Arc<TrustEngine>,
    threshold: f64,  // θ_konsens = 2/3
}

impl ConsensusEngine {
    /// Κ18: Gewichteter Partition-Konsens
    /// Ψ(Σ)(φ) = Σ 𝕎(s)·[s ⊢ φ] / Σ 𝕎(s)
    pub fn compute_consensus(&self, partition: &Partition, proposal: &Proposal) -> ConsensusResult {
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;

        for member in partition.members() {
            let trust = self.trust_engine.get_trust(&member.id);
            let weight = trust.map(|t| t.weighted_norm(&ContextType::Governance.weights()))
                .unwrap_or(0.5) as f64;

            let vote = member.vote_on(proposal);  // 1.0 = ja, 0.0 = nein
            weighted_sum += weight * vote;
            weight_total += weight;
        }

        let psi = if weight_total > 0.0 { weighted_sum / weight_total } else { 0.0 };

        ConsensusResult {
            value: psi,
            accepted: psi > self.threshold,
            participation: partition.members().len(),
        }
    }
}
```

```

---

## III. ECLVM Layer (Κ25) – `eclvm/`

Die ECLVM (Erynoa Configuration Language Virtual Machine) ist eine vollständige stack-basierte, gas-metered VM für deterministische Policy-Ausführung.

### 3.0 ECLVM Layer Interne Verbindungen

```

╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║ ║
║ ECLVM LAYER – INTERNE VERBINDUNGS-MATRIX ║
║ ║
║ ┌─────────────────────────────────────────────────────────────────────────────────────────────┐ ║
║ │ │ ║
║ │ ECL Source ════► Parser ════► AST ════► Compiler ════► Bytecode ════► ECLVM Runtime │ ║
║ │ │ │ │ ║
║ │ │ ┌─────────────────────────────────────────────────────────────┐│ │ ║
║ │ │ │ ECLVM Runtime (1416 Zeilen) ││ │ ║
║ │ │ │ ││ │ ║
║ │ │ │ ┌──────────┐ ┌────────┐ ┌──────────┐ ┌────────────────┐ ││ │ ║
║ │ │ │ │ Stack │ │ IP │ │ Gas │ │ HostInterface │ ││ │ ║
║ │ │ │ │ [Value] │ │(usize) │ │ Meter │ │ │ ││ │ ║
║ │ │ │ │ max:1024 │ │ │ │ │ │ ┌────────────┐ │ ││ │ ║
║ │ │ │ └──────────┘ └────────┘ └────┬─────┘ │ │ trust*get │ │ ││ │ ║
║ │ │ │ │ │ │ event_emit │ │ ││ │ ║
║ │ │ │ │ │ │ storage*\* │ │ ││ │ ║
║ │ │ │ │ │ │ time_now │ │ ││ │ ║
║ │ │ │ │ │ └────────────┘ │ ││ │ ║
║ │ │ │ │ │ │ │ ││ │ ║
║ │ │ │ ▼ └───────┼────────┘ ││ │ ║
║ │ │ │ ┌───────────┐ │ ││ │ ║
║ │ │ │ │ GasMeter │ │ ││ │ ║
║ │ │ │ │ │ │ ││ │ ║
║ │ │ │ │ consume() │◄──────────┘ ││ │ ║
║ │ │ │ │ ─► Gas │ Host calls ││ │ ║
║ │ │ │ │ State │ consume gas││ │ ║
║ │ │ │ └─────┬─────┘ ││ │ ║
║ │ │ │ │ ││ │ ║
║ │ │ └────────────────────────────────┼────────────────────────────┘│ │ ║
║ │ │ │ │ │ ║
║ │ │ │ StateGraph: ECLVM ──DependsOn──► Gas │ ║
║ │ │ ▼ │ │ ║
║ │ │ ┌──────────────────┐ │ │ ║
║ │ │ │ ManaManager │ │ │ ║
║ │ │ │ │ │ │ ║
║ │ │ │ BandwidthTiers: │ │ │ ║
║ │ │ │ - Tier1: 1KB/s │ │ │ ║
║ │ │ │ - Tier2: 10KB/s │ │ │ ║
║ │ │ │ - Tier3: 100KB/s│ │ │ ║
║ │ │ │ │ │ │ ║
║ │ │ │ consume_mana() │ │ │ ║
║ │ │ │ ─► Mana State │ │ │ ║
║ │ │ └────────┬─────────┘ │ │ ║
║ │ │ │ │ │ ║
║ │ │ │ StateGraph: ECLVM ──DependsOn──► Mana │ ║
║ │ │ ▼ │ │ ║
║ │ │ ┌────────────────────────────────────────────────────────────────┐ │ ║
║ │ │ │ ECLPolicy / ECLBlueprint │ │ ║
║ │ │ │ │ │ ║
║ │ │ │ ┌─────────────────┐ ┌─────────────────┐ │ │ ║
║ │ │ │ │ Gateway Policy │ │ Realm Blueprint │ │ │ ║
║ │ │ │ │ │ │ │ │ │ ║
║ │ │ │ │ validate_cross()│ │ store_templates │ │ │ ║
║ │ │ │ │ apply_rules() │ │ initial_setup() │ │ │ ║
║ │ │ │ └────────┬────────┘ └────────┬────────┘ │ │ ║
║ │ │ │ │ │ │ │ ║
║ │ │ │ │ │ │ │ ║
║ │ │ │ ▼ ▼ │ │ ║
║ │ │ │ StateGraph: ECLPolicy ──Validates──► Gateway, Realm │ │ ║
║ │ │ │ │ │ ║
║ │ │ └────────────────────────────────────────────────────────────────┘ │ ║
║ │ │ │ ║
║ └─────────┴──────────────────────────────────────────────────────────────────────────────────┘ ║
║ ║
║ OPCODE GAS-KOSTEN (aus bytecode.rs): ║
║ ║
║ │ OpCode │ Gas │ Kategorie │ Beschreibung │ ║
║ │────────────│─────│──────────────│─────────────────────────────────────────────│ ║
║ │ Push │ 1 │ Stack │ Wert auf Stack │ ║
║ │ Pop │ 1 │ Stack │ Wert vom Stack │ ║
║ │ Add/Sub/.. │ 3 │ Arithmetic │ Basis-Operationen │ ║
║ │ Eq/Lt/Gt │ 3 │ Comparison │ Vergleiche │ ║
║ │ Jump/JumpIf│ 8 │ Control Flow │ Sprünge │ ║
║ │ Call │ 10 │ Function │ Funktionsaufruf │ ║
║ │ TrustGet │ 25 │ Host Call │ Trust-Abfrage (→ TrustEngine) │ ║
║ │ EventEmit │ 100 │ Host Call │ Event-Emission (→ EventEngine) │ ║
║ │ StorageR │ 50 │ Host Call │ Storage-Read │ ║
║ │ StorageW │ 200 │ Host Call │ Storage-Write │ ║
║ │ SigVerify │ 500 │ Crypto │ Signatur-Verifikation │ ║
║ ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝

```

### 3.1 Architektur-Übersicht

```

┌─────────────────────────────────────────────────────────────────────────────────┐
│ ECLVM Pipeline │
├─────────────────────────────────────────────────────────────────────────────────┤
│ │
│ ┌─────────┐ ┌──────────┐ ┌──────────┐ ┌─────────┐ │
│ │ ECL │───▶│ Parser │───▶│ Compiler │───▶│Bytecode │ │
│ │ Text │ │ (Lexer) │ │(AST→Op) │ │(OpCode) │ │
│ └─────────┘ └──────────┘ └──────────┘ └────┬────┘ │
│ │ │
│ ┌───────────────────────────────────────────────────┘ │
│ │ │
│ ▼ │
│ ┌─────────────────────────────────────────────────────────────────────────┐ │
│ │ ECLVM Runtime │ │
│ │ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌───────────┐ ┌────────────┐ │ │
│ │ │ Stack │ │ IP │ │ Gas │ │ Host │ │ CallStack │ │ │
│ │ │[Value] │ │(usize) │ │ Meter │ │ Interface │ │ [usize] │ │ │
│ │ └─────────┘ └─────────┘ └─────────┘ └───────────┘ └────────────┘ │ │
│ │ │ │ │
│ │ ▼ │ │
│ │ ┌─────────────────┐ │ │
│ │ │ Erynoa Core │ │ │
│ │ │ (Trust, Events) │ │ │
│ │ └─────────────────┘ │ │
│ └─────────────────────────────────────────────────────────────────────────┘ │
│ │
└─────────────────────────────────────────────────────────────────────────────────┘

````

### 3.2 ECLVM Runtime (`eclvm/runtime/vm.rs`)

```rust
// Aus: backend/src/eclvm/runtime/vm.rs (1416 Zeilen)

/// ECLVM - Die Erynoa Configuration Language Virtual Machine
pub struct ECLVM<'a> {
    /// Der Operanden-Stack
    stack: Vec<Value>,
    /// Instruction Pointer
    ip: usize,
    /// Das Bytecode-Programm
    program: Vec<OpCode>,
    /// Gas Meter für DoS-Schutz
    gas: GasMeter,
    /// Host Interface für externe Aufrufe (Trust, Events, etc.)
    host: &'a dyn HostInterface,
    /// Call Stack für Funktionsaufrufe
    call_stack: Vec<usize>,
    /// Max Stack-Tiefe (DoS-Schutz)
    max_stack_depth: usize,  // 1024
}

/// Ergebnis einer VM-Ausführung
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub value: Value,
    pub gas_used: u64,
    pub logs: Vec<String>,
}

impl<'a> ECLVM<'a> {
    /// Führe das Programm aus (optimierte Main-Loop)
    pub fn run(&mut self) -> Result<ExecutionResult> {
        while self.ip < self.program.len() {
            let op = self.program[self.ip].clone();
            self.ip += 1;

            // 1. Gas abziehen
            self.gas.consume(op.gas_cost())?;

            // 2. Stack-Tiefe prüfen
            if self.stack.len() > self.max_stack_depth {
                return Err(ApiError::Internal(anyhow!("Stack overflow")));
            }

            // 3. Operation ausführen
            match self.execute_instruction(op)? {
                ControlFlow::Continue => {}
                ControlFlow::Return(result) => {
                    return Ok(ExecutionResult {
                        value: result,
                        gas_used: self.gas.consumed(),
                        logs: Vec::new(),
                    });
                }
                ControlFlow::Error(msg) => {
                    return Err(ApiError::Internal(anyhow!("{}", msg)));
                }
            }
        }

        Ok(ExecutionResult {
            value: self.stack.pop().unwrap_or(Value::Null),
            gas_used: self.gas.consumed(),
            logs: Vec::new(),
        })
    }
}
````

### 3.3 OpCodes und Values (`eclvm/bytecode.rs`)

```rust
// Aus: backend/src/eclvm/bytecode.rs

/// ECL Value-Typen
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
    TrustVector([f32; 6]),
    DID(String),
}

/// ECL OpCodes mit Gas-Kosten
#[derive(Debug, Clone)]
pub enum OpCode {
    // Stack Manipulation (Gas: 1)
    PushConst(Value), Pop, Dup, Swap, Pick(usize),

    // Arithmetik (Gas: 1-2)
    Add, Sub, Mul, Div, Mod, Neg, Abs, Floor, Ceil,

    // Vergleiche (Gas: 1)
    Eq, Ne, Lt, Le, Gt, Ge,

    // Logik (Gas: 1)
    And, Or, Not,

    // Control Flow (Gas: 1-3)
    Jump(usize), JumpIf(usize), JumpIfNot(usize),
    Call(usize), Return, Halt,

    // Trust Operations (Gas: 10-50)
    LoadTrust(TrustDimIndex),        // Push trust dimension
    LoadTrustNorm,                   // Push weighted norm
    UpdateTrust(TrustDimIndex),      // Update trust (requires auth)
    GetTrustVector,                  // Get full 6D vector

    // Event Operations (Gas: 20-100)
    EmitEvent(String),               // Emit named event
    QueryEvents,                     // Query event history

    // Realm Operations (Gas: 10-30)
    GetCurrentRealm,
    CheckCredential,
    ValidateCrossing,
}

impl OpCode {
    /// Gas-Kosten pro Operation
    pub fn gas_cost(&self) -> u64 {
        match self {
            OpCode::PushConst(_) | OpCode::Pop | OpCode::Dup | OpCode::Swap => 1,
            OpCode::Add | OpCode::Sub | OpCode::Mul => 1,
            OpCode::Div | OpCode::Mod => 2,
            OpCode::LoadTrust(_) | OpCode::LoadTrustNorm => 10,
            OpCode::UpdateTrust(_) => 50,
            OpCode::EmitEvent(_) => 100,
            _ => 1,
        }
    }
}
```

### 3.4 ManaManager (`eclvm/mana.rs`)

```rust
// Aus: backend/src/eclvm/mana.rs

/// Mana Manager - Bandwidth-Ressourcen
pub struct ManaManager {
    accounts: HashMap<UniversalId, ManaAccount>,
    config: ManaConfig,
}

#[derive(Debug, Clone)]
pub struct ManaConfig {
    pub base_regen_rate: f64,     // Basis-Regeneration pro Sekunde
    pub max_mana: u64,            // Maximum Mana pro Account
    pub burst_multiplier: f64,   // Burst-Bonus bei hohem Trust
}

/// Mana-Account pro Entity
#[derive(Debug, Clone)]
pub struct ManaAccount {
    pub current: u64,
    pub max: u64,
    pub tier: BandwidthTier,
    pub last_regen: Instant,
}

/// Bandwidth-Tiers basierend auf Trust
#[derive(Debug, Clone, Copy)]
pub enum BandwidthTier {
    Basic,      // Trust < 0.3
    Standard,   // Trust 0.3-0.6
    Premium,    // Trust 0.6-0.8
    Elite,      // Trust > 0.8
}
```

### 3.5 ProgrammableGateway (`eclvm/programmable_gateway.rs`)

```rust
// Aus: backend/src/eclvm/programmable_gateway.rs

/// Programmable Gateway - ECL-Policies für Realm-Crossings
pub struct ProgrammableGateway {
    policies: HashMap<RealmId, CompiledPolicy>,
    standard_policies: StandardPolicies,
}

/// Kompilierte ECL-Policy
pub struct CompiledPolicy {
    pub bytecode: Vec<OpCode>,
    pub required_gas: u64,
    pub policy_type: PolicyType,
}

/// Standard-Policies
pub struct StandardPolicies {
    pub min_trust_check: CompiledPolicy,
    pub credential_check: CompiledPolicy,
    pub rate_limit_check: CompiledPolicy,
}

impl ProgrammableGateway {
    /// Evaluiere Crossing-Policy
    pub fn evaluate_crossing(
        &self,
        did: &DID,
        from_realm: &RealmId,
        to_realm: &RealmId,
        host: &dyn HostInterface,
    ) -> GatewayDecision {
        let policy = self.policies.get(to_realm)
            .unwrap_or(&self.standard_policies.min_trust_check);

        let mut vm = ECLVM::new(policy.bytecode.clone(), policy.required_gas, host);
        // Push arguments
        vm.push(Value::DID(did.to_uri()));
        vm.push(Value::String(from_realm.to_string()));

        match vm.run() {
            Ok(result) => match result.value {
                Value::Bool(true) => GatewayDecision::Allow,
                Value::Bool(false) => GatewayDecision::Deny("Policy rejected".into()),
                _ => GatewayDecision::Deny("Invalid policy return type".into()),
            },
            Err(e) => GatewayDecision::Deny(format!("Policy error: {}", e)),
        }
    }
}
```

---

## IV. Storage Layer (Κ1, Κ6-Κ8) – `local/`

### 4.0 Storage Layer Interne Verbindungen

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                      ║
║   STORAGE LAYER – INTERNE VERBINDUNGS-MATRIX                                                        ║
║                                                                                                      ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                                             │   ║
║   │    DecentralizedStorage ══════════════════════════════════════════════════════════════════ │   ║
║   │         │                                                                                   │   ║
║   │         │  Fjall Keyspace (LSM-Tree, Single Binary)                                        │   ║
║   │         │                                                                                   │   ║
║   │         ├─────────────────┬─────────────────┬─────────────────┬─────────────────┐          │   ║
║   │         │                 │                 │                 │                 │          │   ║
║   │         ▼                 ▼                 ▼                 ▼                 ▼          │   ║
║   │   ┌───────────┐    ┌───────────┐    ┌───────────┐    ┌───────────┐    ┌──────────────┐   │   ║
║   │   │IdentityS. │    │ EventStore│    │ TrustStore│    │ContentS.  │    │ RealmStorage │   │   ║
║   │   │           │    │           │    │           │    │ (CAS)     │    │              │   │   ║
║   │   │ DIDs      │    │ DAG-Nodes │    │ TrustVec  │    │ BLAKE3    │    │ StoreTemplate│   │   ║
║   │   │ Keys      │    │ Parents   │    │ Relations │    │ addressed │    │ Schema       │   │   ║
║   │   │ Deleg.    │    │ Witnesses │    │ History   │    │           │    │ Hierarchy    │   │   ║
║   │   └─────┬─────┘    └─────┬─────┘    └─────┬─────┘    └─────┬─────┘    └──────┬───────┘   │   ║
║   │         │                │                │                │                 │          │   ║
║   │         │  ┌─────────────┼────────────────┼────────────────┼─────────────────┤          │   ║
║   │         │  │             │                │                │                 │          │   ║
║   │         │  │    StateGraph: KvStore, EventStore, Archive ──Aggregates──►    │          │   ║
║   │         │  │             │                │                │                 │          │   ║
║   │         │  │             ▼                ▼                ▼                 ▼          │   ║
║   │         │  │    ┌────────────────────────────────────────────────────────────────┐     │   ║
║   │         │  │    │                          Archive                               │     │   ║
║   │         │  │    │                   (ψ_archive Morphismus)                        │     │   ║
║   │         │  │    │                                                                │     │   ║
║   │         │  │    │   epoch_size: 10000        ┌─────────────────────────────┐     │     │   ║
║   │         │  │    │   compression: zstd        │ Merkle-Proofs               │     │     │   ║
║   │         │  │    │                            │                             │     │     │   ║
║   │         │  │    │   archive_epoch() ───────► │ Root = H(H(L0|L1)|H(L2|L3)) │     │     │   ║
║   │         │  │    │   get_proof()              │                             │     │     │   ║
║   │         │  │    │   verify_membership()      └─────────────────────────────┘     │     │   ║
║   │         │  │    │                                                                │     │   ║
║   │         │  │    └────────────────────────────────────────────────────────────────┘     │   ║
║   │         │  │                                                                           │   ║
║   │         │  └───────────────────────────────────────────────────────────────────────────┘   ║
║   │         │                                                                                   │   ║
║   └─────────┴───────────────────────────────────────────────────────────────────────────────────┘   ║
║                                                                                                      ║
║   VERBINDUNGEN ZU ANDEREN LAYERN:                                                                   ║
║                                                                                                      ║
║   IdentityStore ────────► DID (domain/identity.rs)                                                 ║
║                 ────────► DIDDocument, DelegationGraph                                              ║
║                 ◄──────── Peer Layer (GatewayGuard credential checks)                              ║
║                                                                                                      ║
║   EventStore ───────────► EventEngine (core/event_engine.rs)                                       ║
║              ◄─────────── Event-Validierung und DAG-Updates                                         ║
║              ───────────► Archive (ψ_archive nach Epoch-Abschluss)                                  ║
║                                                                                                      ║
║   TrustStore ───────────► TrustEngine (core/trust_engine.rs)                                       ║
║              ◄─────────── Trust-Updates nach Event-Processing                                       ║
║              ───────────► AntiCalcification (protection/)                                           ║
║                                                                                                      ║
║   ContentStore ─────────► ECLVM (eclvm/) - Bytecode-Storage                                        ║
║                ─────────► Attestations, Credentials                                                 ║
║                                                                                                      ║
║   RealmStorage ─────────► GatewayGuard (peer/) - Store-Initialisierung bei Crossing               ║
║                ─────────► ECLBlueprint (eclvm/) - Template-Instanziierung                          ║
║                                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 4.1 DecentralizedStorage (`local/mod.rs`)

Der dezentrale Storage-Manager basiert auf Fjall (embedded LSM-Tree) für eine Single-Binary Architektur.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     STORAGE PARTITIONEN (Fjall Keyspace)                    │
├─────────────┬─────────────┬─────────────┬─────────────┬─────────────────────┤
│ identities  │   events    │    trust    │   content   │    realm_storage    │
│ (DIDs,Keys) │  (DAG)      │ (Vektoren)  │  (CAS)      │  (Dynamische Stores)│
└─────────────┴─────────────┴─────────────┴─────────────┴─────────────────────┘
                      │                                         │
       ┌──────────────┴──────────────┐         ┌───────────────┴───────────────┐
       │      Cold Storage Archive   │         │      Blueprint Marketplace     │
       │   (ψ_archive Morphismus)    │         │   (Dezentraler Template-Store) │
       └─────────────────────────────┘         └───────────────────────────────┘
```

```rust
// Aus: backend/src/local/mod.rs

/// Dezentraler Storage-Manager
#[derive(Clone)]
pub struct DecentralizedStorage {
    /// Fjall Keyspace Instance
    keyspace: Arc<Keyspace>,
    /// Identity Store (DIDs, Keys)
    pub identities: IdentityStore,
    /// Event Store (DAG)
    pub events: EventStore,
    /// Trust Store (Trust-Vektoren)
    pub trust: TrustStore,
    /// Content Addressable Storage (BLAKE3)
    pub content: ContentStore,
    /// Realm Storage (Dynamische Stores)
    pub realm: RealmStorage,
}

impl DecentralizedStorage {
    /// Öffnet oder erstellt den Storage im angegebenen Verzeichnis
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let keyspace = Arc::new(fjall::Config::new(path.as_ref().join("data")).open()?);

        let identities = IdentityStore::new(&keyspace)?;
        let events = EventStore::new(&keyspace)?;
        let trust = TrustStore::new(&keyspace)?;
        let content = ContentStore::new(&keyspace)?;
        let realm = RealmStorage::new(&keyspace, RealmStorageConfig::default())?;

        Ok(Self { keyspace, identities, events, trust, content, realm })
    }

    /// Öffnet einen temporären In-Memory Storage (für Tests)
    pub fn open_temporary() -> Result<Self> {
        let folder = tempfile::tempdir()?;
        Self::open(folder.path())
    }
}
```

### 4.2 RealmStorage (`local/realm_storage.rs`)

```rust
// Aus: backend/src/local/realm_storage.rs

/// Per-Realm dynamische Stores mit intelligentem Prefixing
pub struct RealmStorage {
    keyspace: Arc<Keyspace>,
    partition: Arc<fjall::Partition>,
    config: RealmStorageConfig,
    schemas: HashMap<String, StoreSchema>,
}

#[derive(Debug, Clone)]
pub struct RealmStorageConfig {
    pub max_stores_per_realm: usize,     // 100
    pub max_key_size: usize,             // 1024
    pub max_value_size: usize,           // 1MB
    pub enable_schema_evolution: bool,   // true
}

/// Store-Templates für automatische Initialisierung
#[derive(Debug, Clone)]
pub struct StoreTemplate {
    pub name: String,
    pub schema: StoreSchema,
    pub store_type: StoreType,
}

#[derive(Debug, Clone)]
pub enum StoreType {
    KeyValue,         // Standard KV
    TimeSeries,       // Zeitreihen
    DocumentStore,    // JSON-Dokumente
    GraphStore,       // Graph-Beziehungen
}
```

### 4.3 Archive (`local/archive.rs`) – ψ_archive Morphismus

```rust
// Aus: backend/src/local/archive.rs

/// Cold Storage Archive (ψ_archive Morphismus)
///
/// Archiviert alte Events mit Merkle-Proofs für Verifizierbarkeit.
pub struct Archive {
    keyspace: Arc<Keyspace>,
    partition: Arc<fjall::Partition>,
    config: ArchiveConfig,
}

#[derive(Debug, Clone)]
pub struct ArchiveConfig {
    pub epoch_size: u64,              // Events pro Epoch
    pub compression_level: u32,       // zstd compression
    pub merkle_tree_depth: usize,     // Proof-Tiefe
}

/// Epoch-Metadaten
pub struct EpochMetadata {
    pub epoch_id: u64,
    pub event_count: u64,
    pub merkle_root: [u8; 32],
    pub compressed_size: u64,
    pub created_at: TemporalCoord,
}

/// Merkle-Proof für archivierte Events
pub struct MerkleProof {
    pub event_id: EventId,
    pub epoch_id: u64,
    pub path: Vec<[u8; 32]>,
    pub root: [u8; 32],
}
```

### 4.4 ContentStore (`local/content_store.rs`) – BLAKE3 CAS

```rust
// Aus: backend/src/local/content_store.rs

/// Content Addressable Storage mit BLAKE3
pub struct ContentStore {
    keyspace: Arc<Keyspace>,
    partition: Arc<fjall::Partition>,
}

/// Content-ID = BLAKE3 Hash
pub type ContentId = [u8; 32];

/// Gespeicherter Content mit Metadaten
pub struct StoredContent {
    pub id: ContentId,
    pub data: Vec<u8>,
    pub metadata: ContentMetadata,
}

pub struct ContentMetadata {
    pub content_type: String,
    pub size: u64,
    pub created_at: TemporalCoord,
    pub author: Option<UniversalId>,
}

impl ContentStore {
    /// Store content, returns BLAKE3 hash
    pub fn put(&self, data: &[u8], metadata: ContentMetadata) -> Result<ContentId> {
        let id = blake3::hash(data).into();
        // Store in Fjall partition
        self.partition.insert(id, /* serialized data + metadata */)?;
        Ok(id)
    }

    /// Get content by BLAKE3 hash
    pub fn get(&self, id: &ContentId) -> Result<Option<StoredContent>> {
        self.partition.get(id).map(|opt| opt.map(|bytes| /* deserialize */))
    }
}
```

---

## V. Protection Layer (Κ19-Κ21) – `protection/`

Der Protection Layer schützt das Netzwerk vor Machtkonzentration, Sybil-Angriffen und Gaming-Versuchen.

### 5.0 Protection Layer Interne Verbindungen

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                      ║
║   PROTECTION LAYER – INTERNE VERBINDUNGS-MATRIX                                                     ║
║                                                                                                      ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                                             │   ║
║   │    NetworkMetrics ══════════════════════► CalibrationEngine ══════════════► ParameterUpdate│   ║
║   │         │         - gini_coefficient              │                              │          │   ║
║   │         │         - churn_rate_24h               │                              │          │   ║
║   │         │         - estimated_sybil_ratio        │  PID-Controller              │          │   ║
║   │         │         - avg_latency_ms               │  EMA-Glättung                ▼          │   ║
║   │         │         - trust_entropy                │                        ┌──────────┐     │   ║
║   │         │                                        │                        │ Anti-    │     │   ║
║   │         │                                        └───────────────────────►│ Calcifi- │     │   ║
║   │         │                                                                 │ cation   │     │   ║
║   │         │                                                                 │          │     │   ║
║   │         │    ┌─────────────────────────────────────────────────────────► │ - entity │     │   ║
║   │         │    │                                                            │   _exp   │     │   ║
║   │         │    │     TrustEngine (Core) ─────────────────────────────────►  │ - decay  │     │   ║
║   │         │    │          │                                                 │   _rate  │     │   ║
║   │         │    │          │ trust_vectors                                   │ - alarm  │     │   ║
║   │         │    │          │                                                 │   _params│     │   ║
║   │         │    │          ▼                                                 └────┬─────┘     │   ║
║   │         │    │     ┌──────────────────┐                                        │           │   ║
║   │         │    │     │  PowerCalculator │◄───────────────────────────────────────┘           │   ║
║   │         │    │     │                  │  check_power_cap()                                 │   ║
║   │         │    │     │  power(s) =      │  apply_temporal_decay()                            │   ║
║   │         │    │     │   trust_norm ×   │  check_concentration_alarm()                       │   ║
║   │         │    │     │   activity_score │                                                    │   ║
║   │         │    │     └────────┬─────────┘                                                    │   ║
║   │         │    │              │                                                              │   ║
║   │         │    │              │ power_values[]                                               │   ║
║   │         │    │              ▼                                                              │   ║
║   │         │    │     ┌────────────────────┐                                                  │   ║
║   │         │    │     │  DiversityMonitor  │ ◄──────────────────────────────────────┐        │   ║
║   │         │    │     │  (Κ20)             │                                        │        │   ║
║   │         │    │     │                    │  H(X) = -Σ p(x) · log₂(p(x))          │        │   ║
║   │         │    │     │  - did_type        │  min_entropy: 2.0                      │        │   ║
║   │         │    │     │  - geo_region      │  max_single_category: 0.5              │        │   ║
║   │         │    │     │  - activity_type   │                                        │        │   ║
║   │         │    │     └────────┬───────────┘                                        │        │   ║
║   │         │    │              │                                                    │        │   ║
║   │         │    │              │ entropy_values, category_distribution              │        │   ║
║   │         │    │              ▼                                                    │        │   ║
║   │         │    │     ┌────────────────────┐                                        │        │   ║
║   │         │    │     │ QuadraticGovernance│ ────────────────────────────┐          │        │   ║
║   │         │    │     │ (Κ21)              │                             │          │        │   ║
║   │         │    │     │                    │                             │          │        │   ║
║   │         │    └────►│ vote_weight =      │                             │          │        │   ║
║   │         │          │  √votes × trust    │                             │          │        │   ║
║   │         │          │                    │                             ▼          │        │   ║
║   │         │          │ cost(n) = n²       │                   ┌────────────────┐   │        │   ║
║   │         │          │ initial: 100 cred  │                   │ ConsensusEngine│   │        │   ║
║   │         │          └──────────────────┬─┘                   │ (Core)         │◄──┘        │   ║
║   │         │                             │                     │                │            │   ║
║   │         │                             │ weighted_votes       │  Ψ(Σ)(φ) mit  │            │   ║
║   │         │                             └────────────────────►│  Diversity-   │            │   ║
║   │         │                                                    │  Korrekturen  │            │   ║
║   │         │                                                    └───────────────┘            │   ║
║   │         │                                                                                  │   ║
║   └─────────┴──────────────────────────────────────────────────────────────────────────────────┘   ║
║                                                                                                      ║
║   FEEDBACK-SCHLEIFEN:                                                                               ║
║                                                                                                      ║
║   1. Anti-Calcification → TrustEngine:                                                              ║
║      - Bei power_cap_exceeded: trust_vectors werden gedämpft                                        ║
║      - StateGraph: AntiCalcification ──Triggers──► Trust                                            ║
║                                                                                                      ║
║   2. Calibration → Gas/Mana:                                                                        ║
║      - PID-Output → gas_costs anpassen (höhere Kosten bei hoher Konzentration)                      ║
║      - StateGraph: Calibration ──Triggers──► Gas, Mana                                              ║
║                                                                                                      ║
║   3. Diversity → Consensus:                                                                         ║
║      - Bei low_entropy: Consensus-Gewichte anpassen                                                 ║
║      - StateGraph: Diversity ──Validates──► Consensus                                               ║
║                                                                                                      ║
║   4. Quadratic → Trust:                                                                             ║
║      - vote_weight = √votes × trust_norm                                                            ║
║      - StateGraph: Quadratic ──DependsOn──► Trust                                                   ║
║                                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 5.1 AntiCalcification (`anti_calcification.rs`) – Κ19

Parameter wurden durch Simulation optimiert und validiert.

```rust
// Aus: backend/src/protection/anti_calcification.rs

/// Anti-Calcification Engine (Κ19)
///
/// Verhindert Machtkonzentration durch:
/// - Power Caps: √(Σ power) / |S|^(1/4)
/// - Temporal Decay: power × e^(-decay_rate × days)
/// - Alarm bei top_percentage% mit >alarm_power_threshold% der Macht
pub struct AntiCalcification {
    config: AntiCalcificationConfig,
    power_history: HashMap<UniversalId, PowerHistory>,
}

/// Simulation-optimierte Parameter (Κ19)
#[derive(Debug, Clone)]
pub struct AntiCalcificationConfig {
    /// Entity Exponent für Power Cap (optimiert: 0.25)
    /// Gültiger Bereich: 0.20-0.30
    pub entity_exponent: f64,     // 0.25 = |S|^(1/4)

    /// Decay Rate pro Tag (optimiert: 0.006)
    /// Gültiger Bereich: 0.003-0.012
    pub decay_rate_per_day: f64,  // 0.006 = 6‰/Tag

    /// Alarm-Schwelle: Top X% Entities (optimiert: 0.03 = 3%)
    pub alarm_top_percentage: f64,

    /// Alarm wenn Top-Gruppe >X% der Macht hält (optimiert: 0.42 = 42%)
    pub alarm_power_threshold: f64,
}

impl Default for AntiCalcificationConfig {
    fn default() -> Self {
        Self {
            entity_exponent: 0.25,          // |S|^(1/4)
            decay_rate_per_day: 0.006,      // 6‰ Decay pro Tag
            alarm_top_percentage: 0.03,     // Top 3%
            alarm_power_threshold: 0.42,    // 42% Macht-Schwelle
        }
    }
}

impl AntiCalcification {
    /// Κ19: Power Cap Berechnung
    /// cap = √(Σ power) / |S|^(entity_exponent)
    pub fn compute_power_cap(&self, total_power: f64, entity_count: usize) -> f64 {
        let sqrt_total = total_power.sqrt();
        let entity_factor = (entity_count as f64).powf(self.config.entity_exponent);
        sqrt_total / entity_factor
    }

    /// Κ19: Temporal Decay anwenden
    /// new_power = current_power × e^(-decay_rate × days_since_activity)
    pub fn apply_temporal_decay(&self, power: f64, days_since_activity: f64) -> f64 {
        power * (-self.config.decay_rate_per_day * days_since_activity).exp()
    }

    /// Alarm-Check: Ist Macht zu konzentriert?
    pub fn check_concentration_alarm(&self, powers: &[(UniversalId, f64)]) -> Option<ConcentrationAlarm> {
        let total: f64 = powers.iter().map(|(_, p)| p).sum();
        let mut sorted: Vec<_> = powers.iter().map(|(_, p)| *p).collect();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());

        let top_count = (powers.len() as f64 * self.config.alarm_top_percentage).ceil() as usize;
        let top_power: f64 = sorted.iter().take(top_count).sum();
        let top_ratio = top_power / total;

        if top_ratio > self.config.alarm_power_threshold {
            Some(ConcentrationAlarm {
                top_percentage: self.config.alarm_top_percentage,
                power_held: top_ratio,
                threshold: self.config.alarm_power_threshold,
            })
        } else {
            None
        }
    }
}
```

### 5.2 AdaptiveCalibration (`adaptive_calibration.rs`) – PID-Controller

```rust
// Aus: backend/src/protection/adaptive_calibration.rs

/// Calibration Engine - Adaptive Parameteranpassung mit PID-Controller
pub struct CalibrationEngine {
    pid: PIDController,
    target_metrics: TargetMetrics,
    parameter_bounds: ParameterBounds,
    smoothing: ExponentialMovingAverage,
}

/// PID Controller für smooth Parameter-Updates
#[derive(Debug, Clone)]
pub struct PIDController {
    pub kp: f64,     // Proportional Gain
    pub ki: f64,     // Integral Gain
    pub kd: f64,     // Derivative Gain
    integral: f64,
    last_error: f64,
}

/// Netzwerk-Metriken für Kalibrierung
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub gini_coefficient: f64,      // Macht-Ungleichheit (0-1)
    pub churn_rate_24h: f64,        // Fluktuation der Entitäten
    pub estimated_sybil_ratio: f64, // Geschätzte Sybil-Quote
    pub avg_latency_ms: f64,        // Durchschnittliche Latenz
}

/// Sichere Grenzen für automatische Kalibrierung
#[derive(Debug, Clone)]
pub struct ParameterBounds {
    pub trust_positive_rate: (f64, f64),     // (0.05, 0.2)
    pub trust_negative_rate: (f64, f64),     // (0.1, 0.4)
    pub decay_rate_per_day: (f64, f64),      // (0.003, 0.012)
    pub entity_exponent: (f64, f64),         // (0.20, 0.30)
}

impl CalibrationEngine {
    /// Berechne Parameter-Adjustment basierend auf Metriken
    pub fn compute_adjustment(&mut self, metrics: &NetworkMetrics) -> ParameterAdjustment {
        // Ziel: Gini-Koeffizient unter 0.5 halten
        let gini_error = metrics.gini_coefficient - 0.5;
        let pid_output = self.pid.update(gini_error);

        // EMA smoothing für stability
        let smoothed = self.smoothing.update(pid_output);

        ParameterAdjustment {
            decay_rate_delta: smoothed * 0.001,  // Sehr vorsichtige Anpassung
            entity_exponent_delta: smoothed * 0.01,
        }
    }
}
```

### 5.3 DiversityMonitor (`diversity.rs`) – Κ20

```rust
// Aus: backend/src/protection/diversity.rs

/// Diversity Monitor - Shannon Entropy für Interaktions-Diversität (Κ20)
pub struct DiversityMonitor {
    config: DiversityConfig,
    interaction_history: HashMap<UniversalId, InteractionHistory>,
}

#[derive(Debug, Clone)]
pub struct DiversityConfig {
    /// Minimum Shannon Entropy (optimiert: 2.0)
    pub min_entropy: f64,
    /// Max Single Category Dominance (optimiert: 0.5 = 50%)
    pub max_single_category: f64,
    /// Alarm wenn Entropy unter diesem Wert
    pub alarm_entropy_threshold: f64,  // 1.5
}

impl DiversityMonitor {
    /// Κ20: Berechne Shannon Entropy
    /// H(X) = -Σ p(x) · log₂(p(x))
    pub fn calculate_shannon_entropy(&self, distribution: &[f64]) -> f64 {
        let total: f64 = distribution.iter().sum();
        if total == 0.0 { return 0.0; }

        distribution.iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| {
                let prob = p / total;
                -prob * prob.log2()
            })
            .sum()
    }

    /// Κ20: Diversity Multiplier für Trust-Berechnung
    pub fn compute_diversity_multiplier(&self, entity: &UniversalId) -> f64 {
        let history = match self.interaction_history.get(entity) {
            Some(h) => h,
            None => return 1.0,  // Keine History = neutral
        };

        let entropy = self.calculate_shannon_entropy(&history.category_counts);
        let normalized = (entropy / self.config.min_entropy).min(1.0);

        // Penalty für single-category dominance
        let max_ratio = history.max_category_ratio();
        let dominance_penalty = if max_ratio > self.config.max_single_category {
            1.0 - (max_ratio - self.config.max_single_category)
        } else {
            1.0
        };

        normalized * dominance_penalty
    }

    /// Collusion Detection via Interaktionsmuster
    pub fn detect_collusion(&self, entity_a: &UniversalId, entity_b: &UniversalId) -> CollusionScore {
        let jaccard = self.jaccard_similarity(entity_a, entity_b);
        let temporal = self.temporal_correlation(entity_a, entity_b);
        let exclusivity = self.interaction_exclusivity(entity_a, entity_b);

        CollusionScore {
            jaccard_similarity: jaccard,
            temporal_correlation: temporal,
            exclusivity_ratio: exclusivity,
            combined_score: 0.4 * jaccard + 0.3 * temporal + 0.3 * exclusivity,
        }
    }
}
```

### 5.4 QuadraticGovernance (`quadratic.rs`) – Κ21

```rust
// Aus: backend/src/protection/quadratic.rs

/// Quadratic Governance - Stimm-Kosten: vote_cost(n) = n²
pub struct QuadraticGovernance {
    config: QuadraticConfig,
    vote_credits: HashMap<UniversalId, VoteCredits>,
}

#[derive(Debug, Clone)]
pub struct QuadraticConfig {
    /// Initiale Voting Credits pro Entity
    pub initial_credits: u64,      // 100
    /// Quorum-Ratio für Proposal-Gültigkeit
    pub quorum_ratio: f64,         // 0.1 (10%)
    /// Approval-Schwelle
    pub approval_threshold: f64,   // 0.5 (50%)
}

impl QuadraticGovernance {
    /// Κ21: Stimm-Kosten berechnen
    /// cost(n) = n² (n Stimmen kosten n² Credits)
    pub fn vote_cost(&self, votes: u64) -> u64 {
        votes * votes
    }

    /// Stimm-Gewicht berechnen mit Trust-Faktor
    pub fn compute_vote_weight(&self, votes: u64, trust_norm: f32) -> f64 {
        // vote_weight = √votes × trust_norm
        (votes as f64).sqrt() * trust_norm as f64
    }

    /// Proposal-Ergebnis berechnen
    pub fn compute_result(&self, votes: &[(UniversalId, i64)], eligible_voters: usize) -> ProposalResult {
        let total_abs_votes: f64 = votes.iter().map(|(_, v)| v.abs() as f64).sum();
        let total_weighted: f64 = votes.iter().map(|(_, v)| *v as f64).sum();

        let participation = votes.len() as f64 / eligible_voters as f64;
        let has_quorum = participation >= self.config.quorum_ratio;

        let approval_ratio = if total_abs_votes > 0.0 {
            (total_weighted + total_abs_votes) / (2.0 * total_abs_votes)
        } else {
            0.0
        };

        ProposalResult {
            approved: has_quorum && approval_ratio >= self.config.approval_threshold,
            approval_ratio,
            participation,
            has_quorum,
        }
    }
}
```

---

## VI. P2P Network Layer – `peer/p2p/`

### 6.1 ErynoaBehaviour (`p2p/behaviour.rs`)

```rust
// Aus: backend/src/peer/p2p/behaviour.rs

/// Libp2p Network Behaviour für Erynoa
#[derive(NetworkBehaviour)]
pub struct ErynoaBehaviour {
    /// GossipSub für Event-Propagation
    pub gossipsub: gossipsub::Behaviour,
    /// Kademlia DHT für Peer-Discovery
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    /// Request-Response für direkte Queries
    pub request_response: request_response::Behaviour<ErynoaCodec>,
    /// mDNS für lokale Discovery
    pub mdns: mdns::tokio::Behaviour,
    /// Identify für Peer-Info
    pub identify: identify::Behaviour,
}

/// Topics für GossipSub
pub struct GossipTopics {
    pub events: gossipsub::IdentTopic,        // Event-Propagation
    pub trust: gossipsub::IdentTopic,         // Trust-Updates
    pub consensus: gossipsub::IdentTopic,     // Consensus-Nachrichten
    pub realm: gossipsub::IdentTopic,         // Realm-Management
}
```

### 6.2 P2P Config

```rust
// Aus: backend/src/peer/p2p/config.rs

#[derive(Debug, Clone)]
pub struct P2PConfig {
    /// Listen Address
    pub listen_addr: Multiaddr,         // /ip4/0.0.0.0/tcp/4001
    /// Bootstrap Peers
    pub bootstrap_peers: Vec<Multiaddr>,
    /// Max Connections
    pub max_connections: u32,           // 256
    /// GossipSub Mesh Parameters
    pub mesh_n: usize,                  // 6
    pub mesh_n_low: usize,              // 4
    pub mesh_n_high: usize,             // 12
    /// Kademlia Replication Factor
    pub kad_replication: NonZeroUsize,  // 20
}
```

---

## VII. Execution Layer (IPS ℳ) – `execution/`

Der Execution Layer implementiert die IPS-Monade ℳ für kontrollierte Seiteneffekte.

### 7.1 IPS-Monade (aus `execution/context.rs`)

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   IPS-MONADE ℳ = State(WorldState) × Writer(Vec<Event>) × Error             │
│                                                                              │
│   ┌────────────────┐     ┌────────────────┐     ┌────────────────┐          │
│   │   WorldState   │     │  Event Writer  │     │   Result<T,E>  │          │
│   │                │     │                │     │                │          │
│   │  • epoch       │  ×  │  • emit_raw()  │  ×  │  • Ok(value)   │          │
│   │  • lamport     │     │  • emit_event()│     │  • Err(error)  │          │
│   │  • root_realm  │     │  • events_list │     │                │          │
│   │  • active_sagas│     │                │     │                │          │
│   └────────────────┘     └────────────────┘     └────────────────┘          │
│                                                                              │
│   Rust-Implementierung:                                                      │
│   fn operation(ctx: &mut ExecutionContext) -> Result<T, ExecutionError>      │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

```rust
// Aus: backend/src/execution/context.rs (795 Zeilen)

/// Globaler Zustand der Welt (State-Komponente der Monade)
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    pub epoch: u64,                          // Aktuelle Epoche
    pub lamport: u64,                        // Lamport-Clock
    pub root_realm: Option<UniversalId>,     // Root-Realm-ID
    pub active_sagas: Vec<UniversalId>,      // Aktive Saga-IDs
}

impl WorldState {
    /// Inkrementiere Lamport-Clock
    pub fn tick(&mut self) -> u64 {
        self.lamport += 1;
        self.lamport
    }

    /// Synchronisiere mit empfangener Lamport-Clock
    pub fn sync(&mut self, received: u64) {
        self.lamport = self.lamport.max(received) + 1;
    }
}

/// Trust-Kontext für die aktuelle Operation (Κ2-Κ5)
#[derive(Debug, Clone)]
pub struct TrustContext {
    pub executor_id: UniversalId,            // Identität des Ausführenden
    pub executor_trust: TrustVector6D,       // Trust-Vektor des Ausführenden
    pub delegation_chain: Vec<DelegationHop>, // Delegations-Kette (Κ8)
    pub effective_trust: TrustVector6D,      // Effektiver Trust nach Delegation
}

/// Ein Hop in der Delegations-Kette
#[derive(Debug, Clone)]
pub struct DelegationHop {
    pub delegator: UniversalId,
    pub delegate: UniversalId,
    pub trust_factor: f32,  // Trust-Faktor der Delegation (0, 1]
}
```

### 7.2 ExecutionContext (Core Execution)

```rust
// Aus: backend/src/execution/context.rs

/// Execution-Context kapselt alle Seiteneffekte (IPS-Monade ℳ)
pub struct ExecutionContext {
    // State-Komponente
    pub state: WorldState,

    // Reader-Komponente (Berechtigungen)
    pub trust_context: TrustContext,

    // Writer-Komponente (Events)
    emitted_events: Vec<Event>,

    // Resources (Limits)
    pub gas_remaining: u64,
    pub gas_initial: u64,
    pub mana_remaining: u64,
    pub mana_initial: u64,

    // Cost-Tracking
    accumulated_cost: Cost,

    // Timing
    started_at: Instant,
}

impl ExecutionContext {
    /// Verbrauche Gas (Compute-Ressource)
    pub fn consume_gas(&mut self, amount: u64) -> ExecutionResult<()> {
        if self.gas_remaining < amount {
            return Err(ExecutionError::GasExhausted {
                required: amount,
                available: self.gas_remaining,
            });
        }
        self.gas_remaining -= amount;
        Ok(())
    }

    /// Verbrauche Mana (Bandwidth-Ressource)
    pub fn consume_mana(&mut self, amount: u64) -> ExecutionResult<()> {
        if self.mana_remaining < amount {
            return Err(ExecutionError::ManaExhausted {
                required: amount,
                available: self.mana_remaining,
            });
        }
        self.mana_remaining -= amount;
        Ok(())
    }

    /// Emittiere Event (Writer-Aspekt)
    pub fn emit_raw(&mut self, event_type: &str, payload: &[u8]) {
        let event_id = UniversalId::new(UniversalId::TAG_EVENT, self.state.epoch as u16, payload);
        self.emitted_events.push(Event::new(event_id, event_type, payload.to_vec()));
        self.state.tick();
    }

    /// Κ4: Trust-Gate prüfen
    pub fn require_trust(&self, minimum: f32) -> ExecutionResult<()> {
        if !self.trust_context.meets_requirement(minimum) {
            return Err(ExecutionError::TrustGateBlocked {
                required: minimum as f64,
                actual: self.trust_context.effective_trust.weighted_norm(&[1.0; 6]) as f64,
            });
        }
        Ok(())
    }
}
```

### 7.3 TrackedContext (State-Integration)

```rust
// Aus: backend/src/execution/tracked.rs (499 Zeilen)

/// Tracked Execution Context - ExecutionContext mit State-Integration
///
/// Wraps ExecutionContext und propagiert alle Operationen zum UnifiedState.
pub struct TrackedContext {
    inner: ExecutionContext,         // Inner ExecutionContext
    integrator: StateIntegrator,     // State Integrator für Updates
    context_id: u64,                 // Unique Context ID
    initial_gas: u64,                // Gas zu Beginn
    initial_mana: u64,               // Mana zu Beginn
    events_count: u64,               // Events emittiert
}

impl TrackedContext {
    /// Verbrauche Gas mit State-Tracking
    pub fn consume_gas(&mut self, amount: u64) -> ExecutionResult<()> {
        let result = self.inner.consume_gas(amount);

        match &result {
            Ok(_) => {
                // Propagiere zu UnifiedState
                self.integrator.on_gas_consumed(amount);
            }
            Err(ExecutionError::GasExhausted { required, available }) => {
                self.integrator.on_out_of_gas(*required, *available);
            }
            _ => {}
        }

        result
    }

    /// Verbrauche Mana mit State-Tracking
    pub fn consume_mana(&mut self, amount: u64) -> ExecutionResult<()> {
        let result = self.inner.consume_mana(amount);

        if result.is_ok() {
            self.integrator.on_mana_consumed(amount);
        }

        result
    }
}
```

### 7.4 Execution → State-Graph Verbindungen

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                        │
│   EXECUTION LAYER → STATE INTEGRATION                                                 │
│                                                                                        │
│   TrackedContext                                                                      │
│        │                                                                              │
│        │ consume_gas()                                                                │
│        ├───────────────────► StateIntegrator.on_gas_consumed()                        │
│        │                              │                                               │
│        │                              ▼                                               │
│        │                     UnifiedState.execution.gas.consumed += amount           │
│        │                              │                                               │
│        │                              ▼                                               │
│        │                     StateGraph: Gas ──Aggregates──► Execution               │
│        │                                                                              │
│        │ consume_mana()                                                               │
│        ├───────────────────► StateIntegrator.on_mana_consumed()                       │
│        │                              │                                               │
│        │                              ▼                                               │
│        │                     UnifiedState.execution.mana.consumed += amount          │
│        │                                                                              │
│        │ emit_event()                                                                 │
│        ├───────────────────► StateIntegrator.on_event_added()                         │
│        │                              │                                               │
│        │                              ▼                                               │
│        │                     UnifiedState.core.events.total += 1                     │
│        │                     UnifiedState.core.events.execution_triggered += 1       │
│        │                              │                                               │
│        │                              ▼                                               │
│        │                     StateGraph: Execution ──Triggers──► Event               │
│        │                                                                              │
│        │ require_trust()                                                              │
│        └───────────────────► TrustContext.meets_requirement()                         │
│                                       │                                               │
│                                       ▼                                               │
│                              StateGraph: Execution ──DependsOn──► Trust              │
│                                                                                        │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

---

## VIII. Unified State Architektur – `core/state.rs`

Die zentrale State-Verwaltung (4389 Zeilen) implementiert hierarchisches, thread-safe State-Management.

### 8.1 UnifiedState Struktur

```rust
// Aus: backend/src/core/state.rs

/// Der zentrale State-Container für alle Erynoa-Module
pub struct UnifiedState {
    /// Core Logic State (Κ2-Κ18)
    pub core: CoreState,
    /// Execution State (IPS ℳ)
    pub execution: ExecutionState,
    /// ECLVM State (Κ25)
    pub eclvm: ECLVMState,
    /// Peer Layer State (Κ22-Κ24)
    pub peer: PeerState,
    /// Protection State (Κ19-Κ21)
    pub protection: ProtectionState,
    /// Storage State
    pub storage: StorageState,
    /// P2P Network State
    pub p2p: P2PState,
    /// StateGraph für Relationship-Queries
    pub graph: StateGraph,
    /// Created Timestamp
    pub created_at: Instant,
}
```

### 8.2 State-Hierarchie (Visual)

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              UNIFIED STATE                                               │
│                                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────────────────┐   │
│  │                          CoreState (Κ2-Κ18)                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐           │   │
│  │  │ TrustState   │──│ EventState   │──│ FormulaState │──│ Consensus  │           │   │
│  │  │              │  │              │  │              │  │   State    │           │   │
│  │  │ • entities   │  │ • total      │  │ • total_e    │  │ • epoch    │           │   │
│  │  │ • relations  │  │ • genesis    │  │ • activity   │  │ • round    │           │   │
│  │  │ • updates    │  │ • finalized  │  │ • trust_norm │  │ • validators│          │   │
│  │  │ • violations │  │ • max_depth  │  │ • human_ver  │  │ • byzantine │          │   │
│  │  │              │  │              │  │              │  │            │           │   │
│  │  │ TRACKING:    │  │ TRACKING:    │  │              │  │            │           │   │
│  │  │ • triggered_ │  │ • trust_     │  │              │  │            │           │   │
│  │  │   events     │  │   triggered  │  │              │  │            │           │   │
│  │  │ • event_     │  │ • consensus_ │  │              │  │            │           │   │
│  │  │   triggered  │  │   validated  │  │              │  │            │           │   │
│  │  │ • realm_     │  │ • eclvm_     │  │              │  │            │           │   │
│  │  │   triggered  │  │   triggered  │  │              │  │            │           │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘           │   │
│  └─────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────────────────┐   │
│  │                       ExecutionState (IPS ℳ)                                     │   │
│  │  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐                   │   │
│  │  │    GasState    │   │   ManaState    │   │ExecutionsState │                   │   │
│  │  │                │   │                │   │                │                   │   │
│  │  │ • consumed     │   │ • consumed     │   │ • active       │                   │   │
│  │  │ • refunded     │   │ • regenerated  │   │ • total        │                   │   │
│  │  │ • out_of_gas   │   │ • rate_limited │   │ • successful   │                   │   │
│  │  │ • current_price│   │ • regen_rate   │   │ • failed       │                   │   │
│  │  │                │   │                │   │ • events_emit  │                   │   │
│  │  │ TRACKING:      │   │ TRACKING:      │   │                │                   │   │
│  │  │ • calibration_ │   │ • calibration_ │   │ TRACKING:      │                   │   │
│  │  │   adjustments  │   │   adjustments  │   │ • saga_triggered│                  │   │
│  │  │ • trust_dep    │   │ • trust_dep    │   │ • gas_aggreg   │                   │   │
│  │  └────────────────┘   └────────────────┘   └────────────────┘                   │   │
│  └─────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────────────────┐   │
│  │                       ProtectionState (Κ19-Κ21)                                  │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐           │   │
│  │  │  Anomaly     │  │  Diversity   │  │  Quadratic   │  │AntiCalc    │           │   │
│  │  │  Detection   │──│  Monitor     │──│  Governance  │──│  State     │           │   │
│  │  │              │  │              │  │              │  │            │           │   │
│  │  │ • detected   │  │ • entropy    │  │ • votes      │  │ • power_   │           │   │
│  │  │ • suppressed │  │ • monoculture│  │ • proposals  │  │   caps     │           │   │
│  │  │ • false_pos  │  │ • warnings   │  │ • quorums    │  │ • decays   │           │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘           │   │
│  │                                                                                  │   │
│  │  ┌────────────────────────────────────────────────────────────────────────┐     │   │
│  │  │                    CalibrationState (PID-Controller)                    │     │   │
│  │  │  • current_params   • adjustment_history   • network_metrics           │     │   │
│  │  └────────────────────────────────────────────────────────────────────────┘     │   │
│  └─────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────────────────┐   │
│  │                         PeerState (Κ22-Κ24)                                      │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐           │   │
│  │  │   Gateway    │  │ SagaComposer │  │ IntentParser │  │ RealmState │           │   │
│  │  │   State      │──│  State       │──│  State       │──│            │           │   │
│  │  │              │  │              │  │              │  │ ISOLATION: │           │   │
│  │  │ • crossings  │  │ • composed   │  │ • parsed     │  │ • trusts   │           │   │
│  │  │ • denied     │  │ • compensated│  │ • validation │  │ • rules    │           │   │
│  │  │ • realms_reg │  │ • cross_realm│  │ • errors     │  │ • metrics  │           │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └────────────┘           │   │
│  └─────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                          │
│  ┌─────────────────────────────────────────────────────────────────────────────────┐   │
│  │                         ECLVMState (Κ25)                                         │   │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                           │   │
│  │  │   VM State   │  │ PolicyState  │  │BlueprintState│                           │   │
│  │  │              │  │              │  │              │                           │   │
│  │  │ • executions │  │ • compiled   │  │ • published  │                           │   │
│  │  │ • gas_used   │  │ • executed   │  │ • deployed   │                           │   │
│  │  │ • mana_used  │  │ • cache_hits │  │ • instantiated│                          │   │
│  │  │ • stack_ovf  │  │ • violations │  │ • verified   │                           │   │
│  │  └──────────────┘  └──────────────┘  └──────────────┘                           │   │
│  └─────────────────────────────────────────────────────────────────────────────────┘   │
│                                                                                          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

### 8.3 Relationship-Tracking Counters

Jeder State hat **Relationship-Tracking Counters** die die StateGraph-Beziehungen quantifizieren:

```rust
// Aus: backend/src/core/state.rs

/// TrustState mit Beziehungs-Tracking
pub struct TrustState {
    // ... (normale Counter)

    // Relationship-Tracking
    pub triggered_events: AtomicU64,         // Trust → Event
    pub event_triggered_updates: AtomicU64,  // Event → Trust
    pub realm_triggered_updates: AtomicU64,  // Realm → Trust
}

/// EventState mit Trigger-Tracking
pub struct EventState {
    // ... (normale Counter)

    // Relationship-Tracking
    pub trust_triggered: AtomicU64,          // Trust → Event
    pub consensus_validated: AtomicU64,      // Consensus → Event
    pub execution_triggered: AtomicU64,      // Execution → Event
    pub gateway_triggered: AtomicU64,        // Gateway → Event
    pub realm_triggered: AtomicU64,          // Realm → Event
    pub eclvm_triggered: AtomicU64,          // ECLVM → Event
    pub policy_triggered: AtomicU64,         // ECLPolicy → Event
    pub blueprint_triggered: AtomicU64,      // ECLBlueprint → Event
    pub swarm_triggered: AtomicU64,          // Swarm → Event
    pub gossip_triggered: AtomicU64,         // Gossip → Event
}

/// GasState mit Dependency-Tracking
pub struct GasState {
    // ... (normale Counter)

    // Relationship-Tracking
    pub calibration_adjustments: AtomicU64,  // Calibration → Gas
    pub trust_dependency_updates: AtomicU64, // Gas ← Trust
}
```

---

## IX. Cross-Cutting Concerns

### 9.1 Kryptographie

```rust
// Aus: backend/src/domain/crypto.rs

pub mod crypto {
    use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};

    /// Ed25519 für einzelne Signaturen (schnell, kompakt)
    pub fn sign_ed25519(keypair: &Keypair, message: &[u8]) -> Signature {
        keypair.sign(message)
    }

    pub fn verify_ed25519(public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool {
        public_key.verify(message, signature).is_ok()
    }

    /// BLS12-381 für Signatur-Aggregation (Consensus, Multi-Party)
    pub fn aggregate_bls_signatures(signatures: &[BlsSignature]) -> BlsSignature {
        // Aggregiert N Signaturen zu einer (konstante Größe)
        BlsSignature::aggregate(signatures)
    }

    /// BLAKE3 für Content-Addressierung (CAS)
    pub fn content_id(data: &[u8]) -> [u8; 32] {
        blake3::hash(data).into()
    }
}
```

### 9.2 Telemetrie (`telemetry.rs`)

```rust
// Aus: backend/src/telemetry.rs

/// Telemetry Setup mit OpenTelemetry + Jaeger
pub fn setup_telemetry(config: &TelemetryConfig) -> Result<()> {
    // Subscriber mit JSON-Formatierung
    let subscriber = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().json());

    // Optional: Jaeger Export
    if config.jaeger_enabled {
        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name("erynoa-backend")
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;
        // ...
    }
    Ok(())
}
```

### 9.3 Deployment-Modi

| Modus      | Storage            | Consensus | RAM   | Disk   | Use Case   |
| ---------- | ------------------ | --------- | ----- | ------ | ---------- |
| Full Node  | Komplette Historie | Ja        | 8GB+  | 100GB+ | Server     |
| Light Node | Eigene + Proofs    | Nein      | 2GB   | 10GB   | Desktop    |
| Browser    | Session-only       | Nein      | -     | -      | Web-App    |
| Mobile     | Light + Offline    | Nein      | 512MB | 1GB    | Smartphone |

---

## X. Axiom-Mapping: Vollständige Code-Referenzen

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                          ║
║   AXIOM → SOURCE CODE MAPPING (DEEP)                                                                    ║
║                                                                                                          ║
║   ┌────────┬─────────────────────────────────────────────────────────────────────────────────────────┐  ║
║   │ AXIOM  │ IMPLEMENTIERUNG                                                                         │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ1     │ local/realm_storage.rs: RealmStorage::validate_hierarchy()                              │  ║
║   │        │   └── StateGraph: Realm ──DependsOn──► ECLPolicy (für Rule-Vererbung)                  │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ2     │ core/trust_engine.rs: TrustEngine::initialize_trust() [default=0.5]                     │  ║
║   │        │   └── execution/context.rs: TrustContext::direct() / delegated()                       │  ║
║   │        │   └── StateGraph: Trust ──DependsOn──► WorldFormula                                    │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ3     │ domain/trust.rs: TrustVector6D [r,i,c,p,v,ω] (6 Dimensionen)                           │  ║
║   │        │   └── execution/context.rs: TrustContext.executor_trust                                │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ4     │ core/trust_engine.rs: TrustEngineConfig [neg_rate=2×pos_rate]                          │  ║
║   │        │   └── execution/context.rs: ExecutionContext::require_trust()                          │  ║
║   │        │   └── core/state.rs: TrustState.asymmetry_ratio() → sollte ~2:1 sein                   │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ5     │ core/trust_engine.rs: TrustEngine::combine_trust()                                      │  ║
║   │        │   └── core/engine.rs: TrustUpdater::combine() [Gas: 30 per source]                     │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ6-Κ8  │ domain/identity.rs: DID, DIDDocument, DelegationGraph                                   │  ║
║   │        │   └── execution/context.rs: DelegationHop, TrustContext::delegated()                   │  ║
║   │        │   └── StateGraph: Trust ──Validates──► (Identität-Beziehungen)                         │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ9-Κ12 │ core/event_engine.rs: EventEngine [DAG, cycle detection via BFS]                        │  ║
║   │        │   └── core/engine.rs: EventProcessor::validate() [Gas: 200 + 50/parent + 100 cycle]   │  ║
║   │        │   └── core/state.rs: EventState mit 10 Trigger-Counters                                │  ║
║   │        │   └── StateGraph: Event ◄──Triggers──► Trust (bidirektional)                           │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ13-Κ14│ peer/saga_composer.rs: SagaComposer, SagaAction [Lock, Transfer, Mint, Burn, WaitFor]   │  ║
║   │        │   └── core/state_integration.rs: SagaObserver                                          │  ║
║   │        │   └── StateGraph: SagaComposer ──Triggers──► Execution                                 │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ15a   │ core/surprisal.rs: SurprisalCalculator [Count-Min Sketch 1024×5]                        │  ║
║   │        │   └── core/engine.rs: formula_gas::SURPRISAL = 80                                      │  ║
║   │        │   └── 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)                                                            │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ15b-d │ core/world_formula.rs: WorldFormulaEngine [inkrementell, O(1) cached]                   │  ║
║   │        │   └── core/engine.rs: FormulaComputer [Gas: 150 contrib, 500 global]                   │  ║
║   │        │   └── 𝔼 = Σ 𝔸(s) · σ⃗(‖𝕎(s)‖ · ln|ℂ(s)| · 𝒮(s)) · Ĥ(s) · w(s,t)                        │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ16    │ core/world_formula.rs: HumanFactor in WorldFormulaContribution                          │  ║
║   │        │   └── Ĥ(s) = 1 wenn human_verified, sonst 0                                            │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ17    │ core/world_formula.rs: WorldFormulaConfig::temporal_decay_rate = 0.99                   │  ║
║   │        │   └── w(s,t) = temporal_decay^(t - t_activity)                                         │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ18    │ core/consensus.rs: ConsensusEngine::compute_consensus() [θ=2/3]                         │  ║
║   │        │   └── Ψ(Σ)(φ) = Σ 𝕎(s)·[s ⊢ φ] / Σ 𝕎(s)                                               │  ║
║   │        │   └── StateGraph: Consensus ──Validates──► Event                                       │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ19    │ protection/anti_calcification.rs [exp=0.25, decay=0.006/day, alarm=3%@42%]              │  ║
║   │        │   └── protection/adaptive_calibration.rs: PID-Controller für dynamische Anpassung     │  ║
║   │        │   └── StateGraph: AntiCalcification ──Triggers──► Trust (Power-Caps)                  │  ║
║   │        │   └── StateGraph: Calibration ──Triggers──► Gas, Mana (Preisanpassung)                │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ20    │ protection/diversity.rs: DiversityMonitor [Shannon entropy, min=2.0]                    │  ║
║   │        │   └── H(X) = -Σ p(x) · log₂(p(x))                                                      │  ║
║   │        │   └── StateGraph: Diversity ──Validates──► Trust, Consensus                            │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ21    │ protection/quadratic.rs: QuadraticGovernance [cost(n)=n², initial=100 credits]          │  ║
║   │        │   └── vote_weight = √votes × trust_norm                                                │  ║
║   │        │   └── StateGraph: Quadratic ──DependsOn──► Trust                                       │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ22    │ peer/saga_composer.rs: SagaComposer::compose()                                          │  ║
║   │        │   └── peer/intent_parser.rs: IntentParser → Goal → Saga                                │  ║
║   │        │   └── StateGraph: SagaComposer ──Aggregates──► IntentParser                            │  ║
║   │        │   └── StateGraph: SagaComposer ──DependsOn──► ECLVM                                    │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ23    │ peer/gateway.rs: GatewayGuard::validate_crossing()                                      │  ║
║   │        │   └── CrossingResult: allowed, dampened_trust, stores_to_initialize                    │  ║
║   │        │   └── core/state_integration.rs: GatewayObserver                                       │  ║
║   │        │   └── StateGraph: Gateway ──DependsOn──► Trust, Realm, ECLPolicy                       │  ║
║   │        │   └── StateGraph: Gateway ──Triggers──► Event (Crossing-Events)                        │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ24    │ domain/trust.rs: TrustDampeningMatrix::apply() [‖M‖≤1]                                  │  ║
║   │        │   └── 𝕎_target = M_ctx × 𝕎_source                                                      │  ║
║   │        │   └── core/state_integration.rs: on_trust_dampened()                                   │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ25    │ eclvm/runtime/vm.rs: ECLVM [1416 Zeilen, gas-metered, stack max=1024]                   │  ║
║   │        │   └── eclvm/bytecode.rs: OpCode mit Gas-Kosten (1-100)                                 │  ║
║   │        │   └── eclvm/mana.rs: ManaManager mit BandwidthTiers                                    │  ║
║   │        │   └── eclvm/programmable_gateway.rs: ProgrammableGateway                               │  ║
║   │        │   └── StateGraph: ECLVM ──DependsOn──► Gas, Mana, Trust                                │  ║
║   │        │   └── StateGraph: ECLPolicy ──Validates──► Gateway, Realm                              │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ26    │ domain/realm.rs: RealmConfig::default_open = true                                       │  ║
║   │        │   └── peer/gateway.rs: Default-Crossing erlaubt wenn min_trust erfüllt                │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ27    │ Documentation requirements (dieses Dokument!)                                           │  ║
║   ├────────┼─────────────────────────────────────────────────────────────────────────────────────────┤  ║
║   │ Κ28    │ domain/fee.rs: FeeCalculator [max_fee bounds]                                           │  ║
║   │        │   └── execution/mod.rs: gas_costs, mana_costs Module                                   │  ║
║   └────────┴─────────────────────────────────────────────────────────────────────────────────────────┘  ║
║                                                                                                          ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### Parameter-Übersicht (aus Code extrahiert)

| Parameter                 | Wert  | Datei                 | Axiom |
| ------------------------- | ----- | --------------------- | ----- |
| `default_trust`           | 0.5   | trust_engine.rs       | Κ2    |
| `positive_rate`           | 0.1   | trust_engine.rs       | Κ4    |
| `negative_rate`           | 0.2   | trust_engine.rs       | Κ4    |
| `min_witnesses`           | 3     | event_engine.rs       | Κ12   |
| `witness_trust_threshold` | 0.5   | event_engine.rs       | Κ12   |
| `max_parents`             | 10    | event_engine.rs       | Κ9    |
| `activity_window_days`    | 90    | world_formula.rs      | Κ15c  |
| `activity_threshold`      | 10    | world_formula.rs      | Κ15c  |
| `temporal_decay_rate`     | 0.99  | world_formula.rs      | Κ17   |
| `entity_exponent`         | 0.25  | anti_calcification.rs | Κ19   |
| `decay_rate_per_day`      | 0.006 | anti_calcification.rs | Κ19   |
| `alarm_top_percentage`    | 0.03  | anti_calcification.rs | Κ19   |
| `alarm_power_threshold`   | 0.42  | anti_calcification.rs | Κ19   |
| `min_entropy`             | 2.0   | diversity.rs          | Κ20   |
| `max_single_category`     | 0.5   | diversity.rs          | Κ20   |
| `initial_credits`         | 100   | quadratic.rs          | Κ21   |
| `quorum_ratio`            | 0.1   | quadratic.rs          | Κ21   |
| `consensus_threshold`     | 2/3   | consensus.rs          | Κ18   |
| `default_min_trust`       | 0.3   | gateway.rs            | Κ23   |
| `max_stack_depth`         | 1024  | eclvm/runtime/vm.rs   | Κ25   |

---

*Weiter zu [04-STATE-MANAGEMENT.md](04-STATE-MANAGEMENT.md) für das interne Zustandsmanagement.*
