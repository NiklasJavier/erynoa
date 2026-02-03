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

### 0.5 Datenfluss-Beispiele: Intent-basierte Realm-Interaktion

Da Erynoa **keine Token** im klassischen Sinne verwendet, sondern ein **reputations- und intent-basiertes System** ist, zeigen die folgenden Beispiele realistische Use Cases für die dynamische Realm-, Raum- und Partitions-Architektur.

---

#### 0.5.1 Architektur: Virtual Realms, Räume und Partitionen

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                         │
│   HIERARCHISCHE STRUKTUR: Realm → VirtualRealm → Raum → Partition                                     │
│                                                                                                         │
│   ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│   │                              ROOT REALM (Κ1-Κ28)                                                 │ │
│   │                              Unveränderliche Kern-Axiome                                         │ │
│   └─────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│                                          │                                                             │
│               ┌──────────────────────────┼──────────────────────────┐                                 │
│               ▼                          ▼                          ▼                                 │
│   ┌───────────────────────┐  ┌───────────────────────┐  ┌───────────────────────┐                    │
│   │   VIRTUAL REALM       │  │   VIRTUAL REALM       │  │   VIRTUAL REALM       │                    │
│   │   "research.academy"  │  │   "commerce.market"   │  │   "governance.dao"    │                    │
│   │                       │  │                       │  │                       │                    │
│   │   ECL-Policy:         │  │   ECL-Policy:         │  │   ECL-Policy:         │                    │
│   │   • min_trust: 0.4    │  │   • min_trust: 0.3    │  │   • min_trust: 0.6    │                    │
│   │   • controller_peer   │  │   • controller_peer   │  │   • controller_peer   │                    │
│   │   • UI-Definition     │  │   • UI-Definition     │  │   • UI-Definition     │                    │
│   │   • Datenlogik        │  │   • Datenlogik        │  │   • Datenlogik        │                    │
│   └───────────┬───────────┘  └───────────────────────┘  └───────────────────────┘                    │
│               │                                                                                        │
│       ┌───────┴───────┐                                                                               │
│       ▼               ▼                                                                               │
│   ┌───────────┐   ┌───────────┐                                                                       │
│   │   RAUM    │   │   RAUM    │   Räume = Interaktionskontexte innerhalb eines VirtualRealms          │
│   │ "lab-42"  │   │"seminar-A"│   • Eigene ECL-UI-Definition                                          │
│   │           │   │           │   • Eigene Datenlogik                                                  │
│   │ ┌───────┐ │   │ ┌───────┐ │   • Controller-Peer kann Struktur in Echtzeit ändern                  │
│   │ │Part-1 │ │   │ │Part-1 │ │                                                                        │
│   │ │Part-2 │ │   │ │Part-2 │ │   Partitionen = Isolierte Daten-/Arbeitseinheiten                     │
│   │ │Part-3 │ │   │ │       │ │   • Feingranulare Zugriffskontrolle                                   │
│   │ └───────┘ │   │ └───────┘ │   • Eigene Event-DAGs                                                  │
│   └───────────┘   └───────────┘   • Per-Partition Trust-Kontexte                                       │
│                                                                                                         │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

#### 0.5.2 ECL-Layer: Unified Configuration Language

ECL (Erynoa Configuration Language) definiert **deklarativ** alle Aspekte eines Realms/Raums:

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                         │
│   ECL UNIFIED DEFINITION LAYER                                                                         │
│                                                                                                         │
│   ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│   │                                                                                                 │ │
│   │   ECL-Definition = {                                                                            │ │
│   │       policy:      Zugangs- und Verhaltensregeln                                               │ │
│   │       structure:   Raum- und Partitions-Topologie                                              │ │
│   │       ui:          Deklarative Interaktionsoberfläche                                          │ │
│   │       datalogic:   Datenverarbeitung und -transformation                                       │ │
│   │       controller:  Steuerungs-Peer mit Änderungsrechten                                        │ │
│   │   }                                                                                             │ │
│   │                                                                                                 │ │
│   └─────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                                         │
│   ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐             │
│   │   POLICY-ECL     │  │  STRUCTURE-ECL   │  │     UI-ECL       │  │  DATALOGIC-ECL   │             │
│   │                  │  │                  │  │                  │  │                  │             │
│   │  realm "lab" {   │  │  structure {     │  │  ui {            │  │  datalogic {     │             │
│   │    min_trust:0.5 │  │    rooms: [      │  │    layout: grid  │  │    on_event {    │             │
│   │    credentials:  │  │      "main",     │  │    components: [ │  │      filter:     │             │
│   │      ["verified"]│  │      "archive"   │  │      panel {     │  │        type=att  │             │
│   │    controller:   │  │    ]             │  │        title:... │  │      transform:  │             │
│   │      did:ery:adm │  │    partitions: { │  │        bindings: │  │        aggregate │             │
│   │    governance:   │  │      "main": 3,  │  │          data... │  │      emit:       │             │
│   │      quadratic   │  │      "archive":1 │  │      }           │  │        summary   │             │
│   │  }               │  │    }             │  │    ]             │  │    }             │             │
│   │                  │  │  }               │  │  }               │  │  }               │             │
│   └──────────────────┘  └──────────────────┘  └──────────────────┘  └──────────────────┘             │
│          │                      │                      │                      │                       │
│          │                      │                      │                      │                       │
│          └──────────────────────┴──────────────────────┴──────────────────────┘                       │
│                                          │                                                             │
│                                          ▼                                                             │
│                              ┌───────────────────────┐                                                │
│                              │    ECL RUNTIME        │                                                │
│                              │                       │                                                │
│                              │  • Hot-Reload fähig   │                                                │
│                              │  • Gas-metered        │                                                │
│                              │  • Intent-reaktiv     │                                                │
│                              │  • Per-Peer-Rendering │                                                │
│                              └───────────────────────┘                                                │
│                                                                                                         │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

#### 0.5.3 Controller-Peer Architektur

Jedes Virtual Realm / Raum kann einen **Controller-Peer** definieren, der erweiterte Rechte besitzt:

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                         │
│   CONTROLLER-PEER SYSTEM                                                                               │
│                                                                                                         │
│   ┌─────────────────────────────────────────────────────────────────────────────────────────────────┐ │
│   │                                                                                                 │ │
│   │   Controller-Peer (in ECL-Policy definiert)                                                     │ │
│   │   ════════════════════════════════════════                                                      │ │
│   │                                                                                                 │ │
│   │   Fähigkeiten:                                                                                  │ │
│   │   ┌─────────────────────────────────────────────────────────────────────────────────────┐      │ │
│   │   │                                                                                     │      │ │
│   │   │   1. STRUKTUR-STEUERUNG                                                             │      │ │
│   │   │      • Räume erstellen/löschen                                                      │      │ │
│   │   │      • Partitionen hinzufügen/entfernen                                             │      │ │
│   │   │      • Hierarchie-Anpassungen (unter Κ1-Konformität)                               │      │ │
│   │   │                                                                                     │      │ │
│   │   │   2. RECHTE-VERGABE                                                                 │      │ │
│   │   │      • Trust-Anforderungen pro Raum/Partition anpassen                             │      │ │
│   │   │      • Credential-Anforderungen definieren                                          │      │ │
│   │   │      • Sub-Controller delegieren (Κ8)                                              │      │ │
│   │   │                                                                                     │      │ │
│   │   │   3. UI-STEUERUNG                                                                   │      │ │
│   │   │      • Layout in Echtzeit ändern                                                    │      │ │
│   │   │      • Komponenten ein-/ausblenden                                                  │      │ │
│   │   │      • Daten-Bindings aktualisieren                                                │      │ │
│   │   │                                                                                     │      │ │
│   │   │   4. DATENLOGIK-STEUERUNG                                                           │      │ │
│   │   │      • Event-Filter anpassen                                                        │      │ │
│   │   │      • Transformationen ändern                                                      │      │ │
│   │   │      • Aggregations-Regeln definieren                                               │      │ │
│   │   │                                                                                     │      │ │
│   │   └─────────────────────────────────────────────────────────────────────────────────────┘      │ │
│   │                                                                                                 │ │
│   │   Einschränkungen (Axiom-Konformität):                                                         │ │
│   │   ┌─────────────────────────────────────────────────────────────────────────────────────┐      │ │
│   │   │   • Κ1: Kann Regeln NUR hinzufügen, nie entfernen                                  │      │ │
│   │   │   • Κ19: Unterliegt Power-Cap (keine unbegrenzte Kontrolle)                        │      │ │
│   │   │   • Κ21: Änderungen können per Quadratic Governance überstimmt werden              │      │ │
│   │   │   • Κ23: Crossing-Validierung gilt auch für Controller                             │      │ │
│   │   └─────────────────────────────────────────────────────────────────────────────────────┘      │ │
│   │                                                                                                 │ │
│   └─────────────────────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                                         │
│   Delegation-Kette:                                                                                    │
│                                                                                                         │
│   ┌─────────────┐    delegate()    ┌─────────────┐    delegate()    ┌─────────────┐                   │
│   │ Root-Admin  │ ───────────────► │ Realm-Ctrl  │ ───────────────► │ Raum-Ctrl   │                   │
│   │ (Trust:0.9) │   factor: 0.8    │ (Trust:0.72)│   factor: 0.7    │ (Trust:0.5) │                   │
│   └─────────────┘                  └─────────────┘                  └─────────────┘                   │
│                                                                                                         │
│   Effektiver Trust nimmt mit jeder Delegation ab (Κ8, Κ24)                                            │
│                                                                                                         │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

#### 0.5.4 Datenfluss-Beispiel 1: Raum-Erstellung durch Controller-Intent

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                          │
│   USE CASE: Controller erstellt neuen Raum mit UI und Datenlogik                                        │
│                                                                                                          │
│   CONTROLLER (did:ery:admin)                                                                            │
│     │                                                                                                    │
│     │ Intent: "Erstelle Raum 'workshop' mit Präsentations-UI und Event-Aggregation"                    │
│     ▼                                                                                                    │
│   ┌────────────────────────┐                                                                            │
│   │   IntentParser         │  on_intent_parsed(type="realm_modify", sub="create_room")                  │
│   │                        │                                                                            │
│   │  parse_structured() ───┼─────────────────────┐                                                      │
│   │  ↓                     │                     │                                                      │
│   │  Goal::RealmModify {   │                     ▼                                                      │
│   │    action: CreateRoom, │             ┌──────────────┐                                               │
│   │    room_id: "workshop",│             │ IntentParser │ (State-Update)                                │
│   │    ecl_config: {       │             │    State     │                                               │
│   │      ui: {...},        │             └──────────────┘                                               │
│   │      datalogic: {...}, │                                                                            │
│   │      partitions: 2     │                                                                            │
│   │    }                   │                                                                            │
│   │  }                     │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ 1. Controller-Validierung                                                              │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   GatewayGuard         │  on_controller_action(realm="research.academy", action="create_room")      │
│   │                        │                                                                            │
│   │  validate_controller() │──────────────────────┐                                                     │
│   │  ↓                     │                      │                                                     │
│   │  CHECK:                │                      ▼                                                     │
│   │  • is_controller(did)? │              ┌──────────────┐                                              │
│   │  • trust >= min_trust? │              │  Gateway     │ (State-Update)                               │
│   │  • action_permitted?   │              │   State      │                                              │
│   │  → ALL PASS ✓          │              └──────────────┘                                              │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ 2. Saga-Komposition (Κ22)                                                              │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   SagaComposer         │  on_saga_composed(steps=4, goal="create_room")                             │
│   │                        │                                                                            │
│   │  compose() ────────────┼──────────────────────┐                                                     │
│   │  ↓                     │                      │                                                     │
│   │  Saga {                │                      ▼                                                     │
│   │    steps: [            │              ┌──────────────┐                                              │
│   │      ValidateECL,      │              │ SagaComposer │ (State-Update)                               │
│   │      CreateRoomStruct, │              │    State     │                                              │
│   │      DeployUI,         │              └──────────────┘                                              │
│   │      InitPartitions    │                                                                            │
│   │    ]                   │                                                                            │
│   │  }                     │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ 3. ECL-Kompilierung und Validierung                                                    │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   ECLVM                │  on_policy_compiled(policy_id="workshop_ui", bytecode_size=1420)           │
│   │                        │                                                                            │
│   │  compile_ecl() ────────┼──────────────────────┐                                                     │
│   │  ↓                     │                      │                                                     │
│   │  • Parse UI-Definition │                      ▼                                                     │
│   │  • Parse DataLogic     │              ┌──────────────┐                                              │
│   │  • Validate Κ1         │              │   ECLVM      │ (State-Update)                               │
│   │  • Bytecode-Gen        │              │    State     │                                              │
│   │  Gas: 850              │              └──────────────┘                                              │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ 4. Struktur-Erstellung im Storage                                                      │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   RealmStorage         │  on_room_created(realm="research.academy", room="workshop")                │
│   │                        │                                                                            │
│   │  create_room() ────────┼──────────────────────┐                                                     │
│   │  ↓                     │                      │                                                     │
│   │  • Create room prefix  │                      ▼                                                     │
│   │  • Init 2 partitions   │              ┌──────────────┐                                              │
│   │  • Store ECL-Bytecode  │              │  Storage     │ (State-Update)                               │
│   │  • Register UI         │              │   State      │                                              │
│   │  Mana: 120             │              └──────────────┘                                              │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ 5. Event-Emission                                                                      │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   EventEngine          │  on_event_added(type="room_created", realm="research.academy")             │
│   │                        │                                                                            │
│   │  add_event() ──────────┼──────────────────────┐                                                     │
│   │  ↓                     │                      │                                                     │
│   │  Event {               │                      ▼                                                     │
│   │    type: "room_created"│              ┌──────────────┐                                              │
│   │    payload: {...}      │              │    Event     │ (State-Update)                               │
│   │    author: did:ery:adm │              │    State     │                                              │
│   │  }                     │              └──────────────┘                                              │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ 6. Trust-Update für Controller                                                         │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   TrustEngine          │  on_trust_update(entity=admin, dim=Competence, delta=+0.02)                │
│   │                        │                                                                            │
│   │  process_event() ──────┼──────────────────────┐                                                     │
│   │  ↓                     │                      │                                                     │
│   │  update_trust(         │                      ▼                                                     │
│   │    admin,              │              ┌──────────────┐                                              │
│   │    dimension=c,        │              │    Trust     │ (State-Update)                               │
│   │    delta=+0.02         │              │    State     │                                              │
│   │  )                     │              └──────────────┘                                              │
│   └────────────────────────┘                                                                            │
│                                                                                                          │
│   ERGEBNIS:                                                                                             │
│   • Neuer Raum "workshop" mit 2 Partitionen                                                             │
│   • UI-Definition aktiv und renderbar für alle Raum-Mitglieder                                          │
│   • DataLogic bereit für Event-Verarbeitung                                                             │
│   • Controller-Trust leicht erhöht (erfolgreiche Aktion)                                                │
│                                                                                                          │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

#### 0.5.5 Datenfluss-Beispiel 2: Dynamische UI-Anpassung per Intent

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                          │
│   USE CASE: Controller passt UI in Echtzeit an (Live-Präsentation → Diskussion)                         │
│                                                                                                          │
│   CONTROLLER                                                                                            │
│     │                                                                                                    │
│     │ Intent: "Wechsle Layout von 'presentation' zu 'discussion' mit Voting-Panel"                      │
│     ▼                                                                                                    │
│   ┌────────────────────────┐                                                                            │
│   │   IntentParser         │                                                                            │
│   │                        │                                                                            │
│   │  Goal::UIModify {      │                                                                            │
│   │    room: "workshop",   │                                                                            │
│   │    layout: "discussion"│                                                                            │
│   │    add_components: [   │                                                                            │
│   │      VotingPanel       │                                                                            │
│   │    ]                   │                                                                            │
│   │  }                     │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Controller-Validierung + ECL-Delta-Kompilierung                                        │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   ECLVM                │                                                                            │
│   │                        │                                                                            │
│   │  compile_ui_delta() ───┼──► ECL-Diff: { layout: "discussion", +VotingPanel }                       │
│   │  ↓                     │                                                                            │
│   │  validate_delta():     │                                                                            │
│   │  • Κ1-Konform? ✓       │    (UI-Änderungen verletzen keine Regeln)                                 │
│   │  • Trust sufficient? ✓ │                                                                            │
│   │  Gas: 180              │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Hot-Reload an alle Raum-Teilnehmer                                                     │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   P2P GossipSub        │                                                                            │
│   │                        │                                                                            │
│   │  broadcast_to_room() ──┼──► Topic: "research.academy/workshop/ui"                                  │
│   │  ↓                     │                                                                            │
│   │  Message {             │                                                                            │
│   │    type: "ui_update",  │                                                                            │
│   │    delta: {...},       │                                                                            │
│   │    signature: sig(ctrl)│                                                                            │
│   │  }                     │                                                                            │
│   │  Mana: 50 (broadcast)  │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│   ═════════════╪═════════════════════════════════════════════════════════════════════════════════════   │
│                │                                                                                        │
│                │ ALLE PEERS IM RAUM empfangen Update                                                    │
│                ▼                                                                                        │
│   ┌────────────────────────┐  ┌────────────────────────┐  ┌────────────────────────┐                   │
│   │   Peer A               │  │   Peer B               │  │   Peer C               │                   │
│   │                        │  │                        │  │                        │                   │
│   │  on_ui_update() ───────┼  │  on_ui_update() ───────┼  │  on_ui_update() ───────┼                   │
│   │  ↓                     │  │  ↓                     │  │  ↓                     │                   │
│   │  1. Verify signature   │  │  1. Verify signature   │  │  1. Verify signature   │                   │
│   │  2. Check controller   │  │  2. Check controller   │  │  2. Check controller   │                   │
│   │  3. Apply ECL-delta    │  │  3. Apply ECL-delta    │  │  3. Apply ECL-delta    │                   │
│   │  4. Re-render UI       │  │  4. Re-render UI       │  │  4. Re-render UI       │                   │
│   │                        │  │                        │  │                        │                   │
│   │  ┌──────────────────┐  │  │  ┌──────────────────┐  │  │  ┌──────────────────┐  │                   │
│   │  │  NEUES UI:       │  │  │  │  NEUES UI:       │  │  │  │  NEUES UI:       │  │                   │
│   │  │  ┌────┬────┐     │  │  │  │  ┌────┬────┐     │  │  │  │  ┌────┬────┐     │  │                   │
│   │  │  │Chat│Vote│     │  │  │  │  │Chat│Vote│     │  │  │  │  │Chat│Vote│     │  │                   │
│   │  │  └────┴────┘     │  │  │  │  └────┴────┘     │  │  │  │  └────┴────┘     │  │                   │
│   │  │  Discussion Mode │  │  │  │  Discussion Mode │  │  │  │  Discussion Mode │  │                   │
│   │  └──────────────────┘  │  │  └──────────────────┘  │  │  └──────────────────┘  │                   │
│   └────────────────────────┘  └────────────────────────┘  └────────────────────────┘                   │
│                                                                                                          │
│   LATENZ: < 100ms (GossipSub + lokales ECL-Rendering)                                                   │
│                                                                                                          │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

#### 0.5.6 Datenfluss-Beispiel 3: Attestierung mit Reputation-Update

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                          │
│   USE CASE: Peer attestiert Wissen eines anderen Peers (kein Token-Transfer!)                           │
│                                                                                                          │
│   PEER ALICE (Trust: 0.65)                                                                              │
│     │                                                                                                    │
│     │ Intent: "Attestiere Bob's Expertise in 'machine-learning'"                                        │
│     ▼                                                                                                    │
│   ┌────────────────────────┐                                                                            │
│   │   IntentParser         │  on_intent_parsed(type="attest", claim="expertise:ml")                     │
│   │                        │                                                                            │
│   │  Goal::Attest {        │                                                                            │
│   │    subject: did:bob,   │                                                                            │
│   │    claim: "expertise", │                                                                            │
│   │    domain: "ml",       │                                                                            │
│   │    confidence: 0.8     │                                                                            │
│   │  }                     │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Saga-Komposition                                                                       │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   SagaComposer         │                                                                            │
│   │                        │                                                                            │
│   │  Saga {                │                                                                            │
│   │    steps: [            │                                                                            │
│   │      ValidateClaim,    │   ← Prüft: Hat Alice Interaktion mit Bob?                                 │
│   │      CheckDomain,      │   ← Prüft: Ist Alice im Domain "ml" kompetent?                            │
│   │      EmitAttestation,  │   ← Event ins DAG                                                          │
│   │      UpdateTrust       │   ← Bob's Trust erhöht sich                                                │
│   │    ]                   │                                                                            │
│   │  }                     │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Attestation-Validierung                                                                │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   ECLVM (Realm-Policy) │                                                                            │
│   │                        │                                                                            │
│   │  execute_policy() ─────┼──► Policy: "attestation_rules"                                            │
│   │  ↓                     │                                                                            │
│   │  CHECKS:               │                                                                            │
│   │  • alice.trust.c >= 0.5│   ← Attestierer braucht Kompetenz                                         │
│   │  • interaction_count   │   ← Mind. 3 Interaktionen                                                  │
│   │    (alice, bob) >= 3   │                                                                            │
│   │  • no_self_attest      │   ← Keine Selbst-Attestierung                                              │
│   │  → ALL PASS ✓          │                                                                            │
│   │  Gas: 280              │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Event-Erstellung (Κ9-Κ12)                                                              │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   EventEngine          │                                                                            │
│   │                        │                                                                            │
│   │  Event {               │                                                                            │
│   │    id: ev_att_001,     │                                                                            │
│   │    type: "attestation",│                                                                            │
│   │    author: alice,      │                                                                            │
│   │    payload: {          │                                                                            │
│   │      subject: bob,     │                                                                            │
│   │      claim: "ml_expert"│                                                                            │
│   │      confidence: 0.8,  │                                                                            │
│   │      attester_trust:   │                                                                            │
│   │        0.65            │                                                                            │
│   │    },                  │                                                                            │
│   │    parents: [prev_ev]  │                                                                            │
│   │  }                     │                                                                            │
│   │  Gas: 400              │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Trust-Updates (bidirektional)                                                          │
│                ▼                                                                                        │
│   ┌────────────────────────────────────────────────────────────────────────────────────────────────┐   │
│   │                                                                                                │   │
│   │   TrustEngine                                                                                  │   │
│   │                                                                                                │   │
│   │   BOB (Empfänger):                                  ALICE (Attestiererin):                     │   │
│   │   ┌─────────────────────────────────┐               ┌─────────────────────────────────┐       │   │
│   │   │ Dimension C (Competence):       │               │ Dimension V (Vigilance):        │       │   │
│   │   │   delta = confidence ×          │               │   delta = +0.01                 │       │   │
│   │   │           attester_trust × 0.1  │               │   (Aktive Teilnahme belohnt)    │       │   │
│   │   │        = 0.8 × 0.65 × 0.1       │               │                                 │       │   │
│   │   │        = +0.052                 │               │                                 │       │   │
│   │   │                                 │               │                                 │       │   │
│   │   │ Dimension P (Prestige):         │               │                                 │       │   │
│   │   │   delta = +0.03                 │               │                                 │       │   │
│   │   │   (Externe Anerkennung)         │               │                                 │       │   │
│   │   └─────────────────────────────────┘               └─────────────────────────────────┘       │   │
│   │                                                                                                │   │
│   └────────────────────────────────────────────────────────────────────────────────────────────────┘   │
│                │                                                                                        │
│                │ WorldFormula-Update (Κ15)                                                             │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   WorldFormulaEngine   │                                                                            │
│   │                        │                                                                            │
│   │  Inkrementelles Update:│                                                                            │
│   │  • Bob.contribution ↑  │   (höhere Trust-Norm → höherer Beitrag zu 𝔼)                              │
│   │  • Alice.activity ↑    │   (Event erhöht Aktivitätspräsenz)                                        │
│   │  • Global 𝔼 += 0.003   │                                                                            │
│   └────────────────────────┘                                                                            │
│                                                                                                          │
│   ERGEBNIS:                                                                                             │
│   • Bob's Expertise ist nun attestiert und querybar                                                     │
│   • Bob's Trust erhöht (besonders Competence + Prestige)                                                │
│   • Alice erhält kleine Vigilance-Erhöhung                                                              │
│   • Event ist im DAG permanent gespeichert                                                              │
│   • Kein "Transfer" von irgendetwas - nur Reputation-Update                                             │
│                                                                                                          │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

#### 0.5.7 Datenfluss-Beispiel 4: Cross-Realm Zugang mit dynamischem Interface

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                          │
│   USE CASE: Peer betritt neues Realm - Interface passt sich automatisch an                              │
│                                                                                                          │
│   PEER CHARLIE (aktuell in "general" Realm)                                                             │
│     │                                                                                                    │
│     │ Intent: "Betrete research.academy/lab-42"                                                         │
│     ▼                                                                                                    │
│   ┌────────────────────────┐                                                                            │
│   │   IntentParser         │                                                                            │
│   │                        │                                                                            │
│   │  Goal::CrossRealm {    │                                                                            │
│   │    from: "general",    │                                                                            │
│   │    to: "research/lab42"│                                                                            │
│   │  }                     │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Gateway-Validierung (Κ23)                                                              │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   GatewayGuard         │                                                                            │
│   │                        │                                                                            │
│   │  validate_crossing() ──┼──► CrossingResult                                                         │
│   │  ↓                     │                                                                            │
│   │  CHECKS:               │                                                                            │
│   │  • trust_norm: 0.58    │   ← Charlie's aggregierter Trust                                          │
│   │  • min_required: 0.5   │   ← lab-42 erfordert 0.5                                                  │
│   │  • credentials: OK     │   ← "verified_researcher" vorhanden                                       │
│   │  → ALLOWED ✓           │                                                                            │
│   │                        │                                                                            │
│   │  Trust-Dampening (Κ24):│                                                                            │
│   │  • dampening_factor:0.8│                                                                            │
│   │  • new_trust: 0.46     │   ← Trust im neuen Realm                                                  │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Store-Initialisierung für Charlie                                                      │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   RealmStorage         │                                                                            │
│   │                        │                                                                            │
│   │  initialize_stores() ──┼──► Templates aus lab-42 Policy                                            │
│   │  ↓                     │                                                                            │
│   │  Erstellt:             │                                                                            │
│   │  • personal_notes      │   ← Charlies private Notizen                                               │
│   │  • experiment_log      │   ← Experiment-Tracking                                                    │
│   │  • collab_inbox        │   ← Kollaborations-Anfragen                                                │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ UI-Rendering basierend auf ECL                                                         │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   ECLVM (UI-Engine)    │                                                                            │
│   │                        │                                                                            │
│   │  render_for_peer() ────┼──► Lädt: lab-42.ui.ecl                                                    │
│   │  ↓                     │                                                                            │
│   │  UI-Kontext:           │                                                                            │
│   │  • peer_trust: 0.46    │   ← Bestimmt sichtbare Komponenten                                        │
│   │  • credentials: [...]  │   ← Bestimmt Features                                                      │
│   │  • role: "researcher"  │   ← Bestimmt Layout-Variante                                              │
│   │                        │                                                                            │
│   │  Generiert UI:         │                                                                            │
│   │  ┌──────────────────────────────────────────────────────────────────┐                              │
│   │  │  LAB-42 INTERFACE (trust >= 0.4)                                 │                              │
│   │  │  ┌────────────────┬────────────────┬────────────────────────┐   │                              │
│   │  │  │ EXPERIMENTS    │ DISCUSSIONS    │ MY NOTES              │   │                              │
│   │  │  │                │                │                        │   │                              │
│   │  │  │ [list view]    │ [thread view]  │ [private editor]      │   │                              │
│   │  │  │                │                │                        │   │                              │
│   │  │  │ ─────────────  │ ─────────────  │ ─────────────────────  │   │                              │
│   │  │  │ Exp #42 ●      │ Thread: ML... │ + New Note             │   │                              │
│   │  │  │ Exp #41 ○      │ Thread: QC... │                        │   │                              │
│   │  │  └────────────────┴────────────────┴────────────────────────┘   │                              │
│   │  │                                                                  │                              │
│   │  │  [HIDDEN: Admin-Panel - requires trust >= 0.7]                  │                              │
│   │  │  [HIDDEN: Archive-Access - requires credential "archivist"]     │                              │
│   │  └──────────────────────────────────────────────────────────────────┘                              │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ DataLogic-Aktivierung                                                                  │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   ECLVM (DataLogic)    │                                                                            │
│   │                        │                                                                            │
│   │  activate_bindings() ──┼──► Lädt: lab-42.datalogic.ecl                                             │
│   │  ↓                     │                                                                            │
│   │  Bindings aktiv:       │                                                                            │
│   │  • experiments →       │   ← Filter: nur laufende                                                   │
│   │      filter(active)    │                                                                            │
│   │  • discussions →       │   ← Aggregation: nach Aktivität                                            │
│   │      sort(activity)    │                                                                            │
│   │  • my_notes →          │   ← Transform: lokaler Sync                                                │
│   │      sync(local)       │                                                                            │
│   └────────────────────────┘                                                                            │
│                                                                                                          │
│   ERGEBNIS:                                                                                             │
│   • Charlie sieht personalisiertes UI basierend auf seinem Trust-Level                                  │
│   • Komponenten sind trust-gated (Admin-Panel versteckt)                                                │
│   • Daten-Bindings sind aktiv und reaktiv                                                               │
│   • Personal Stores wurden automatisch initialisiert                                                    │
│   • Alles durch ECL deklarativ definiert und hot-reloadable                                            │
│                                                                                                          │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

#### 0.5.8 Datenfluss-Beispiel 5: Governance-Abstimmung mit Quadratic Voting

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                          │
│   USE CASE: Realm-Mitglieder stimmen über Policy-Änderung ab                                            │
│                                                                                                          │
│   CONTROLLER oder HIGH-TRUST PEER                                                                       │
│     │                                                                                                    │
│     │ Intent: "Starte Abstimmung: Erhöhe min_trust für lab-42 von 0.5 auf 0.6"                         │
│     ▼                                                                                                    │
│   ┌────────────────────────┐                                                                            │
│   │   IntentParser         │  on_intent_parsed(type="governance", sub="create_proposal")                │
│   │                        │                                                                            │
│   │  Goal::Governance {    │                                                                            │
│   │    action: Propose,    │                                                                            │
│   │    proposal: {         │                                                                            │
│   │      change: "min_trust│                                                                            │
│   │        = 0.6",         │                                                                            │
│   │      scope: "lab-42",  │                                                                            │
│   │      duration: 7days   │                                                                            │
│   │    }                   │                                                                            │
│   │  }                     │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Proposal-Erstellung                                                                    │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   QuadraticGovernance  │                                                                            │
│   │                        │                                                                            │
│   │  create_proposal() ────┼──► Proposal gespeichert, Voting aktiv                                     │
│   │  ↓                     │                                                                            │
│   │  Proposal {            │                                                                            │
│   │    id: prop_001,       │                                                                            │
│   │    type: "policy_mod", │                                                                            │
│   │    target: "min_trust",│                                                                            │
│   │    new_value: 0.6,     │                                                                            │
│   │    quorum: 10%,        │   ← Mind. 10% müssen abstimmen                                            │
│   │    threshold: 50%,     │   ← Mind. 50% Zustimmung                                                  │
│   │    ends_at: +7d        │                                                                            │
│   │  }                     │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│   ═════════════╪═════════════════════════════════════════════════════════════════════════════════════   │
│                │                                                                                        │
│   VOTING PHASE │ (7 Tage, alle Realm-Mitglieder können abstimmen)                                      │
│                │                                                                                        │
│   ┌────────────┴───────────────────────────────────────────────────────────────────────────────────┐   │
│   │                                                                                                │   │
│   │   PEER ALICE                    PEER BOB                      PEER CHARLIE                    │   │
│   │   Trust: 0.65                   Trust: 0.72                   Trust: 0.46                     │   │
│   │   Credits: 100                  Credits: 85                   Credits: 100                    │   │
│   │                                                                                                │   │
│   │   Intent: "Vote +4             Intent: "Vote -2               Intent: "Vote +1               │   │
│   │           für prop_001"                für prop_001"                   für prop_001"          │   │
│   │                                                                                                │   │
│   │   ┌─────────────────────┐      ┌─────────────────────┐       ┌─────────────────────┐         │   │
│   │   │ QuadraticVoting     │      │ QuadraticVoting     │       │ QuadraticVoting     │         │   │
│   │   │                     │      │                     │       │                     │         │   │
│   │   │ votes: 4            │      │ votes: 2 (gegen)    │       │ votes: 1            │         │   │
│   │   │ cost: 4² = 16 cred  │      │ cost: 2² = 4 cred   │       │ cost: 1² = 1 cred   │         │   │
│   │   │ remaining: 84 cred  │      │ remaining: 81 cred  │       │ remaining: 99 cred  │         │   │
│   │   │                     │      │                     │       │                     │         │   │
│   │   │ weight = √4 × 0.65  │      │ weight = √2 × 0.72  │       │ weight = √1 × 0.46  │         │   │
│   │   │       = 1.3         │      │       = 1.02        │       │       = 0.46        │         │   │
│   │   │                     │      │                     │       │                     │         │   │
│   │   │ → +1.3 für Proposal │      │ → -1.02 für Prop.   │       │ → +0.46 für Prop.   │         │   │
│   │   └─────────────────────┘      └─────────────────────┘       └─────────────────────┘         │   │
│   │                                                                                                │   │
│   └────────────────────────────────────────────────────────────────────────────────────────────────┘   │
│                │                                                                                        │
│   ═════════════╪═════════════════════════════════════════════════════════════════════════════════════   │
│                │                                                                                        │
│                │ Nach 7 Tagen: Ergebnis-Berechnung                                                     │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   ConsensusEngine      │  (kombiniert mit QuadraticGovernance)                                     │
│   │                        │                                                                            │
│   │  compute_result() ─────┼──► Aggregation aller Votes                                                │
│   │  ↓                     │                                                                            │
│   │  Berechnung:           │                                                                            │
│   │                        │                                                                            │
│   │  Total Pro:  +1.3 + 0.46 = +1.76                                                                   │
│   │  Total Con:  -1.02           = -1.02                                                                │
│   │  Net Weight: +0.74                                                                                  │
│   │                        │                                                                            │
│   │  Participation: 3/20 = 15% ✓ (> 10% quorum)                                                        │
│   │  Approval: (1.76 + 1.02 + 0.74) / (2 × 2.78) = 63% ✓                                               │
│   │                        │                                                                            │
│   │  → PROPOSAL ACCEPTED   │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Policy-Update durch ECLVM                                                              │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   ECLVM                │                                                                            │
│   │                        │                                                                            │
│   │  apply_policy_change() │──► lab-42.policy.ecl aktualisiert                                         │
│   │  ↓                     │                                                                            │
│   │  CHANGE:               │                                                                            │
│   │  - min_trust: 0.5      │                                                                            │
│   │  + min_trust: 0.6      │                                                                            │
│   │                        │                                                                            │
│   │  Gas: 350              │                                                                            │
│   └────────────┬───────────┘                                                                            │
│                │                                                                                        │
│                │ Broadcast an alle Peers                                                                │
│                ▼                                                                                        │
│   ┌────────────────────────┐                                                                            │
│   │   P2P GossipSub        │                                                                            │
│   │                        │                                                                            │
│   │  Nachricht:            │                                                                            │
│   │  {                     │                                                                            │
│   │    type: "policy_update",                                                                           │
│   │    proposal: prop_001, │                                                                            │
│   │    result: "accepted", │                                                                            │
│   │    new_policy: {...}   │                                                                            │
│   │  }                     │                                                                            │
│   └────────────────────────┘                                                                            │
│                                                                                                          │
│   AUSWIRKUNG:                                                                                           │
│   • Charlie (Trust 0.46) verliert Zugang zu lab-42 (< 0.6)                                             │
│   • Alice und Bob behalten Zugang                                                                       │
│   • Charlie kann Trust aufbauen oder Credential erwerben, um wieder Zugang zu bekommen                 │
│   • Keine Token wurden "verbraucht" - nur Credits für künftige Abstimmungen reduziert                  │
│                                                                                                          │
└──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
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

/// Unterstützte Goal-Typen (erweitert für Realm-Management und UI)
pub enum Goal {
    // === Klassische Aktionen ===
    Attest { subject: UniversalId, claim: String, domain: Option<String>, confidence: f64 },
    Delegate { to: UniversalId, capabilities: Vec<String>, trust_factor: f64, ttl_seconds: u64 },
    Query { predicate: String },
    Create { entity_type: String, params: HashMap<String, Value> },
    Complex { description: String, sub_goals: Vec<Goal> },

    // === Realm-Management (Controller-Aktionen) ===
    RealmModify {
        action: RealmAction,
        target_realm: RealmId,
        ecl_config: Option<ECLConfig>,
    },

    // === UI-Steuerung (Echtzeit) ===
    UIModify {
        room: String,
        layout: Option<String>,
        add_components: Vec<UIComponent>,
        remove_components: Vec<String>,
        update_bindings: Vec<DataBinding>,
    },

    // === Governance ===
    Governance {
        action: GovernanceAction,
        proposal: Option<Proposal>,
        vote: Option<Vote>,
    },

    // === Cross-Realm Navigation ===
    CrossRealm {
        from: RealmId,
        to: RealmId,
    },
}

/// Realm-Aktionen für Controller
pub enum RealmAction {
    CreateRoom { room_id: String, partitions: usize },
    DeleteRoom { room_id: String },
    CreatePartition { room_id: String, partition_id: String },
    UpdatePolicy { policy_delta: ECLDelta },
    SetController { new_controller: UniversalId },
    DelegateControl { to: UniversalId, scope: ControlScope },
}

/// Governance-Aktionen
pub enum GovernanceAction {
    Propose,
    Vote,
    Execute,
    Cancel,
}

/// UI-Komponenten-Definition (ECL-basiert)
pub struct UIComponent {
    pub id: String,
    pub component_type: String,   // "panel", "list", "form", "chart", etc.
    pub layout: LayoutConfig,
    pub bindings: Vec<DataBinding>,
    pub trust_gate: Option<f32>,  // Min-Trust um Komponente zu sehen
    pub credential_gate: Vec<String>,
}

/// Daten-Binding für reaktive UI
pub struct DataBinding {
    pub source: String,           // "events.filter(type='attestation')"
    pub target: String,           // "component.list.items"
    pub transform: Option<String>, // "aggregate(by=author)" (ECL)
}
```

### 1.1.1 ECL-Konfigurationsstruktur

ECL (Erynoa Configuration Language) definiert Realms, Räume und Partitionen deklarativ:

```rust
// Aus: backend/src/eclvm/ecl_types.rs (konzeptionell)

/// Vollständige ECL-Konfiguration für ein Realm/Raum
pub struct ECLConfig {
    /// Policy-Definition (Zugangsregeln, Verhaltensregeln)
    pub policy: ECLPolicy,
    /// Struktur-Definition (Räume, Partitionen)
    pub structure: ECLStructure,
    /// UI-Definition (Interaktionsoberfläche)
    pub ui: ECLUI,
    /// DataLogic-Definition (Event-Verarbeitung)
    pub datalogic: ECLDataLogic,
    /// Controller-Definition
    pub controller: ECLController,
}

/// Policy-ECL (Zugangs- und Verhaltensregeln)
pub struct ECLPolicy {
    pub min_trust: f32,
    pub required_credentials: Vec<String>,
    pub governance_type: GovernanceType,
    pub rules: Vec<Rule>,
    pub custom_checks: Vec<CompiledPolicy>,  // Kompilierter ECL-Bytecode
}

/// Struktur-ECL (Räume und Partitionen)
pub struct ECLStructure {
    pub rooms: Vec<RoomDefinition>,
    pub partitions: HashMap<String, Vec<PartitionDefinition>>,
    pub hierarchy_rules: Vec<HierarchyRule>,
}

/// UI-ECL (Deklarative Oberfläche)
pub struct ECLUI {
    pub layout: LayoutType,           // Grid, Stack, Split, etc.
    pub components: Vec<UIComponent>,
    pub responsive_rules: Vec<ResponsiveRule>,
    pub trust_visibility: TrustVisibilityMap,  // Welche Elemente bei welchem Trust
}

/// DataLogic-ECL (Event-Verarbeitung und Transformation)
pub struct ECLDataLogic {
    pub event_handlers: Vec<EventHandler>,
    pub filters: Vec<Filter>,
    pub transforms: Vec<Transform>,
    pub aggregations: Vec<Aggregation>,
    pub outputs: Vec<Output>,
}

/// Controller-Definition
pub struct ECLController {
    pub controller_did: UniversalId,
    pub delegations: Vec<ControllerDelegation>,
    pub permissions: ControllerPermissions,
    pub overridable_by_governance: bool,
}

/// Controller-Berechtigungen (unter Axiom-Konformität)
pub struct ControllerPermissions {
    pub can_modify_structure: bool,
    pub can_modify_ui: bool,
    pub can_modify_datalogic: bool,
    pub can_modify_policy: bool,      // Nur Hinzufügen, nie Entfernen (Κ1)
    pub can_delegate: bool,
    pub can_set_trust_requirements: bool,
}
```

### 1.1.2 ECL-Syntax Beispiele

```ecl
// Beispiel: Vollständige Realm-Definition in ECL

realm "research.academy" {
    // === POLICY ===
    policy {
        min_trust: 0.4
        credentials: ["verified_researcher", "academic_institution"]
        governance: quadratic

        // Custom Policy-Check (kompiliert zu Bytecode)
        check "interaction_history" {
            require: sender.interactions_count >= 5
            message: "Mindestens 5 Interaktionen erforderlich"
        }
    }

    // === CONTROLLER ===
    controller {
        did: "did:ery:academy_admin"
        permissions: [structure, ui, datalogic]
        delegate_to: ["did:ery:lab_lead_alice", "did:ery:lab_lead_bob"]
        governance_override: true  // Governance kann Controller überstimmen
    }

    // === STRUKTUR ===
    structure {
        rooms: [
            room "main_hall" {
                partitions: 1
                purpose: "general_discussion"
            },
            room "lab_42" {
                partitions: 3
                purpose: "experiments"
                min_trust: 0.5  // Höher als Realm-Default
            },
            room "archive" {
                partitions: 1
                purpose: "historical_data"
                credentials: ["archivist"]
            }
        ]
    }

    // === UI ===
    ui {
        layout: responsive_grid

        component "header" {
            type: panel
            position: top
            content: {
                title: "Research Academy"
                show_trust: true
                show_membership: true
            }
        }

        component "room_navigator" {
            type: navigation
            position: left
            trust_gate: 0.0  // Alle sehen
            data_binding: rooms.list
        }

        component "main_content" {
            type: dynamic_panel
            position: center
            data_binding: current_room.content
        }

        component "admin_panel" {
            type: panel
            position: right
            trust_gate: 0.7  // Nur High-Trust
            credential_gate: ["admin"]
            data_binding: admin.metrics
        }
    }

    // === DATALOGIC ===
    datalogic {
        on_event "attestation" {
            filter: event.realm == self.id
            transform: aggregate(by: "subject", metric: "count")
            emit: "attestation_summary"
            update_ui: "stats_panel.attestations"
        }

        on_event "room_join" {
            filter: event.type == "crossing" && event.to == self.id
            action: initialize_personal_stores(event.peer)
            emit: "member_joined"
        }

        on_event "governance_result" {
            filter: event.proposal.scope == self.id
            action: apply_policy_change(event.result)
            broadcast: all_members
        }
    }
}
```

### 1.2 Saga Composer (`saga_composer.rs`) – Κ22, Κ24

Der Saga Composer zerlegt komplexe Intents in atomare, kompensierbare Schritte.

```
╔════════════════════════════════════════════════════════════════════════════════╗
║                                                                                ║
║   SAGA COMPOSER – INTENT → SAGA (Κ22)                                         ║
║                                                                                ║
║   INPUT: Intent { goal: RealmModify(CreateRoom("workshop")) }                 ║
║                                                                                ║
║   OUTPUT: Saga [                                                              ║
║       Step 0: ValidateController  | Compensation: None                        ║
║       Step 1: CompileECL          | Compensation: None                        ║
║       Step 2: CreateRoomStruct    | Compensation: DeleteRoom                  ║
║       Step 3: InitPartitions      | Compensation: DeletePartitions            ║
║       Step 4: DeployUI            | Compensation: RollbackUI                  ║
║       Step 5: EmitEvent           | Dependencies: [2,3,4]                     ║
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

/// Saga-Aktionen (erweitert für Realm-Management)
pub enum SagaAction {
    // === Validierung ===
    ValidateController { realm: RealmId, required_permission: String },
    ValidateECL { ecl_source: String },
    ValidateTrust { min_trust: f32 },

    // === Realm-Struktur ===
    CreateRoom { realm: RealmId, room_id: String, partitions: usize },
    DeleteRoom { realm: RealmId, room_id: String },
    CreatePartition { room_id: String, partition_id: String },

    // === UI-Operationen ===
    DeployUI { room_id: String, ecl_ui: Vec<u8> },
    UpdateUI { room_id: String, ecl_delta: Vec<u8> },
    BroadcastUIUpdate { room_id: String, delta: Vec<u8> },

    // === DataLogic ===
    ActivateDataLogic { room_id: String, ecl_datalogic: Vec<u8> },

    // === Storage ===
    InitializeStores { peer: UniversalId, templates: Vec<StoreTemplate> },

    // === Events ===
    EmitEvent { event_type: String, payload: Vec<u8> },

    // === Legacy (für Kompatibilität) ===
    WaitFor { timeout_lamport: u64, condition: String, timeout_seconds: u64 },
}

impl SagaComposer {
    /// Κ22: Komponiere Saga aus Intent
    pub fn compose(&self, intent: &Intent) -> CompositionResult<Saga> {
        let steps = match &intent.goal {
            Goal::Attest { subject, claim, .. } =>
                self.compose_attest(intent.source_did(), subject, claim)?,
            Goal::Delegate { to, capabilities, ttl_seconds, .. } =>
                self.compose_delegate(intent.source_did(), to, capabilities, *ttl_seconds)?,
            Goal::Query { predicate } =>
                self.compose_query(intent.source_did(), predicate)?,
            Goal::Create { entity_type, params } =>
                self.compose_create(intent.source_did(), entity_type, params)?,
            Goal::Complex { description, sub_goals } =>
                self.compose_complex(intent.source_did(), description, sub_goals)?,

            // === Neue Realm-Management Goals ===
            Goal::RealmModify { action, target_realm, ecl_config } =>
                self.compose_realm_modify(intent.source_did(), action, target_realm, ecl_config)?,
            Goal::UIModify { room, layout, add_components, remove_components, update_bindings } =>
                self.compose_ui_modify(intent.source_did(), room, layout, add_components, remove_components, update_bindings)?,
            Goal::Governance { action, proposal, vote } =>
                self.compose_governance(intent.source_did(), action, proposal, vote)?,
            Goal::CrossRealm { from, to } =>
                self.compose_cross_realm(intent.source_did(), from, to)?,
        };

        self.validate_constraints(&steps, &intent.constraints)?;
        Ok(Saga::from_intent(intent, steps, 0))
    }

    /// Komponiere Realm-Modifikations-Saga
    fn compose_realm_modify(
        &self,
        source: &UniversalId,
        action: &RealmAction,
        target_realm: &RealmId,
        ecl_config: &Option<ECLConfig>,
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();

        // 1. Controller-Validierung
        steps.push(SagaStep::new(
            SagaAction::ValidateController {
                realm: target_realm.clone(),
                required_permission: action.required_permission(),
            },
            None,  // Keine Kompensation nötig
        ));

        // 2. ECL-Validierung (wenn vorhanden)
        if let Some(config) = ecl_config {
            steps.push(SagaStep::new(
                SagaAction::ValidateECL {
                    ecl_source: config.to_ecl_string(),
                },
                None,
            ));
        }

        // 3. Aktion ausführen
        match action {
            RealmAction::CreateRoom { room_id, partitions } => {
                steps.push(SagaStep::new(
                    SagaAction::CreateRoom {
                        realm: target_realm.clone(),
                        room_id: room_id.clone(),
                        partitions: *partitions,
                    },
                    Some(SagaAction::DeleteRoom {
                        realm: target_realm.clone(),
                        room_id: room_id.clone(),
                    }),
                ));

                // UI deployen wenn vorhanden
                if let Some(config) = ecl_config {
                    steps.push(SagaStep::new(
                        SagaAction::DeployUI {
                            room_id: room_id.clone(),
                            ecl_ui: config.ui.compile()?,
                        },
                        Some(SagaAction::UpdateUI {
                            room_id: room_id.clone(),
                            ecl_delta: vec![],  // Leere UI
                        }),
                    ));
                }
            }
            // ... weitere Aktionen
        }

        // 4. Event emittieren
        steps.push(SagaStep::new(
            SagaAction::EmitEvent {
                event_type: format!("realm_{}", action.event_name()),
                payload: action.to_event_payload(),
            },
            None,
        ));

        Ok(steps)
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

Die ECLVM (Erynoa Configuration Language Virtual Machine) ist eine vollständige stack-basierte, gas-metered VM für deterministische Policy-Ausführung, erweitert um **UI-Rendering**, **DataLogic** und **Controller-Management**.

### 3.0 ECLVM Layer – Erweiterte Architektur

```

╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║ ║
║ ECLVM LAYER – UNIFIED ECL ARCHITECTURE ║
║ ║
║ ┌─────────────────────────────────────────────────────────────────────────────────────────────┐ ║
║ │ │ ║
║ │ ECL Source ══► Parser ══► AST ══► Compiler ══► Bytecode ══► ECLVM Runtime │ ║
║ │ │ │ │ ║
║ │ │ Unterstützt: │ │ ║
║ │ │ • Policy-ECL │ │ ║
║ │ │ • Structure-ECL │ │ ║
║ │ │ • UI-ECL ◄────────────── NEU │ │ ║
║ │ │ • DataLogic-ECL ◄──────── NEU │ │ ║
║ │ │ • Controller-ECL ◄──────── NEU │ │ ║
║ │ │ ║
║ └─────────────────────────────────────────────────────────────────────────────────────────────┘ ║
║ ║
║ ┌─────────────────────────────────────────────────────────────────────────────────────────────┐ ║
║ │ │ ║
║ │ ECLVM RUNTIME (erweitert) │ ║
║ │ │ ║
║ │ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ │ ║
║ │ │ POLICY │ │ UI │ │ DATALOGIC │ │ CONTROLLER │ │ ║
║ │ │ ENGINE │ │ ENGINE │ │ ENGINE │ │ ENGINE │ │ ║
║ │ │ │ │ │ │ │ │ │ │ ║
║ │ │ validate*│ │ render_for* │ │ process*│ │ validate* │ │ ║
║ │ │ crossing() │ │ peer() │ │ event() │ │ action() │ │ ║
║ │ │ apply_rules()│ │ apply_delta()│ │ transform() │ │ delegate() │ │ ║
║ │ │ check_creds()│ │ broadcast() │ │ aggregate() │ │ revoke() │ │ ║
║ │ └──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └──────┬───────┘ │ ║
║ │ │ │ │ │ │ ║
║ │ │ │ │ │ │ ║
║ │ ┌──────┴───────┐ ┌──────┴───────┐ │ ║
║ │ │ API │ │ GOVERNANCE │ │ ║
║ │ │ ENGINE │ │ ENGINE │ │ ║
║ │ │ │ │ │ │ ║
║ │ │ handle_req() │ │ create_prop()│ ◄──── NEU │ ║
║ │ │ register_ep()│ │ cast_vote() │ (DAO) │ ║
║ │ │ gen_openapi()│ │ execute() │ │ ║
║ │ └──────┬───────┘ └──────┬───────┘ │ ║
║ │ │ │ │ ║
║ │ └─────────────────┴─────────────────┴─────────────────┘ │ ║
║ │ │ │ ║
║ │ ▼ │ ║
║ │ ┌──────────────────┐ │ ║
║ │ │ SHARED STATE │ │ ║
║ │ │ │ │ ║
║ │ │ • Gas/Mana │ │ ║
║ │ │ • Trust Context │ │ ║
║ │ │ • Realm State │ │ ║
║ │ │ • UI State │ ◄──── NEU │ ║
║ │ │ • Binding State │ ◄──── NEU │ ║
║ │ │ • API Registry │ ◄──── NEU │ ║
║ │ │ • Proposals │ ◄──── NEU │ ║
║ │ └──────────────────┘ │ ║
║ │ │ ║
║ └─────────────────────────────────────────────────────────────────────────────────────────────┘ ║
║ ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝

````

### 3.0.1 UI-Engine (Neu)

Die UI-Engine rendert deklarative UI-Definitionen für jeden Peer individuell basierend auf Trust und Credentials:

```rust
// Konzeptionell: backend/src/eclvm/ui_engine.rs

/// UI-Engine für deklaratives Interface-Rendering
pub struct UIEngine {
    /// Kompilierte UI-Definitionen
    compiled_uis: HashMap<RoomId, CompiledUI>,
    /// Component-Registry
    components: ComponentRegistry,
    /// Binding-Manager
    bindings: BindingManager,
}

/// Kompilierte UI-Definition
pub struct CompiledUI {
    pub bytecode: Vec<OpCode>,
    pub layout: LayoutDefinition,
    pub components: Vec<CompiledComponent>,
    pub trust_gates: HashMap<String, f32>,
    pub credential_gates: HashMap<String, Vec<String>>,
}

/// UI-Komponente mit Trust-Gate
pub struct CompiledComponent {
    pub id: String,
    pub component_type: ComponentType,
    pub bytecode: Vec<OpCode>,         // Für dynamische Inhalte
    pub bindings: Vec<CompiledBinding>,
    pub trust_gate: Option<f32>,
    pub credential_gate: Vec<String>,
}

/// Daten-Binding (reaktiv)
pub struct CompiledBinding {
    pub source_expression: Vec<OpCode>,  // "events.filter(...)"
    pub target_path: String,             // "component.list.items"
    pub transform: Option<Vec<OpCode>>,  // "aggregate(...)"
    pub update_trigger: UpdateTrigger,   // OnEvent, OnInterval, OnDemand
}

impl UIEngine {
    /// Rendere UI für spezifischen Peer (Trust-basiert)
    pub fn render_for_peer(
        &self,
        room_id: &RoomId,
        peer: &DID,
        peer_trust: &TrustVector6D,
        peer_credentials: &[String],
    ) -> Result<RenderedUI> {
        let ui = self.compiled_uis.get(room_id)
            .ok_or(UIError::RoomNotFound)?;

        let trust_norm = peer_trust.weighted_norm(&[1.0; 6]);

        let mut visible_components = Vec::new();

        for component in &ui.components {
            // Trust-Gate prüfen
            if let Some(min_trust) = component.trust_gate {
                if trust_norm < min_trust {
                    continue;  // Komponente nicht sichtbar
                }
            }

            // Credential-Gate prüfen
            let has_all_creds = component.credential_gate.iter()
                .all(|c| peer_credentials.contains(c));
            if !has_all_creds {
                continue;  // Komponente nicht sichtbar
            }

            // Bindings aktivieren
            let bound_component = self.activate_bindings(component, peer)?;
            visible_components.push(bound_component);
        }

        Ok(RenderedUI {
            layout: ui.layout.clone(),
            components: visible_components,
            peer_trust: trust_norm,
        })
    }

    /// Wende UI-Delta an (Hot-Reload)
    pub fn apply_delta(
        &mut self,
        room_id: &RoomId,
        delta: &UIDelta,
        controller: &DID,
    ) -> Result<()> {
        // 1. Validiere Controller-Berechtigung
        self.validate_controller(room_id, controller)?;

        // 2. Kompiliere Delta
        let compiled_delta = self.compile_delta(delta)?;

        // 3. Wende an
        if let Some(ui) = self.compiled_uis.get_mut(room_id) {
            ui.apply_delta(compiled_delta)?;
        }

        Ok(())
    }
}
````

### 3.0.2 DataLogic-Engine (Neu)

Die DataLogic-Engine verarbeitet Events reaktiv und transformiert Daten für UI-Bindings:

```rust
// Konzeptionell: backend/src/eclvm/datalogic_engine.rs

/// DataLogic-Engine für Event-Verarbeitung
pub struct DataLogicEngine {
    /// Event-Handler pro Raum
    handlers: HashMap<RoomId, Vec<CompiledEventHandler>>,
    /// Aggregation-State
    aggregations: HashMap<String, AggregationState>,
    /// Output-Emitter
    outputs: OutputEmitter,
}

/// Kompilierter Event-Handler
pub struct CompiledEventHandler {
    pub event_filter: Vec<OpCode>,     // Filterbedingung
    pub transform: Vec<OpCode>,        // Transformation
    pub action: HandlerAction,         // Was tun?
    pub output: Option<String>,        // Event emittieren?
}

/// Handler-Aktionen
pub enum HandlerAction {
    EmitEvent { event_type: String },
    UpdateUI { component_path: String },
    StoreData { store_id: String },
    Aggregate { aggregation_id: String, metric: AggregationMetric },
    Broadcast { scope: BroadcastScope },
}

impl DataLogicEngine {
    /// Verarbeite eingehendes Event
    pub fn process_event(
        &mut self,
        room_id: &RoomId,
        event: &Event,
        vm: &mut ECLVM,
    ) -> Result<Vec<DataLogicOutput>> {
        let handlers = self.handlers.get(room_id)
            .ok_or(DataLogicError::RoomNotFound)?;

        let mut outputs = Vec::new();

        for handler in handlers {
            // 1. Filter prüfen
            vm.push(event.to_value());
            let filter_result = vm.execute(&handler.event_filter)?;

            if filter_result != Value::Bool(true) {
                continue;  // Event passt nicht zum Filter
            }

            // 2. Transformation anwenden
            let transformed = vm.execute(&handler.transform)?;

            // 3. Aktion ausführen
            match &handler.action {
                HandlerAction::UpdateUI { component_path } => {
                    outputs.push(DataLogicOutput::UIUpdate {
                        path: component_path.clone(),
                        value: transformed,
                    });
                }
                HandlerAction::Aggregate { aggregation_id, metric } => {
                    self.update_aggregation(aggregation_id, metric, &transformed)?;
                }
                HandlerAction::EmitEvent { event_type } => {
                    outputs.push(DataLogicOutput::EmitEvent {
                        event_type: event_type.clone(),
                        payload: transformed,
                    });
                }
                HandlerAction::Broadcast { scope } => {
                    outputs.push(DataLogicOutput::Broadcast {
                        scope: scope.clone(),
                        data: transformed,
                    });
                }
                _ => {}
            }
        }

        Ok(outputs)
    }
}
```

### 3.0.3 Controller-Engine (Neu)

Die Controller-Engine verwaltet Berechtigungen und Delegationen:

```rust
// Konzeptionell: backend/src/eclvm/controller_engine.rs

/// Controller-Engine für Berechtigungsverwaltung
pub struct ControllerEngine {
    /// Controller pro Realm/Raum
    controllers: HashMap<ScopeId, ControllerConfig>,
    /// Delegations-Graph (Κ8)
    delegations: DelegationGraph,
    /// Audit-Log
    audit: AuditLog,
}

/// Controller-Konfiguration
pub struct ControllerConfig {
    pub primary_controller: UniversalId,
    pub permissions: ControllerPermissions,
    pub delegates: Vec<Delegation>,
    pub governance_override: bool,
}

/// Delegation an Sub-Controller
pub struct Delegation {
    pub delegate: UniversalId,
    pub scope: ControlScope,
    pub permissions: ControllerPermissions,
    pub trust_factor: f32,  // Κ8: Trust-Dämpfung
    pub expires_at: Option<TemporalCoord>,
}

/// Kontroll-Scope
pub enum ControlScope {
    FullRealm,
    Room(String),
    Partition(String, String),
    Component(String),
}

impl ControllerEngine {
    /// Validiere Controller-Aktion
    pub fn validate_action(
        &self,
        scope: &ScopeId,
        actor: &UniversalId,
        action: &ControllerAction,
    ) -> Result<ValidationResult> {
        // 1. Ist Actor Controller oder Delegate?
        let config = self.controllers.get(scope)
            .ok_or(ControllerError::ScopeNotFound)?;

        let (is_controller, effective_permissions) =
            if &config.primary_controller == actor {
                (true, config.permissions.clone())
            } else if let Some(delegation) = self.find_delegation(config, actor) {
                (true, delegation.permissions.clone())
            } else {
                (false, ControllerPermissions::none())
            };

        if !is_controller {
            return Err(ControllerError::NotAuthorized);
        }

        // 2. Hat Controller Berechtigung für diese Aktion?
        let required_permission = action.required_permission();
        if !effective_permissions.has(required_permission) {
            return Err(ControllerError::InsufficientPermission);
        }

        // 3. Κ1-Prüfung: Kann Regeln nur hinzufügen, nie entfernen
        if let ControllerAction::ModifyPolicy { policy_delta } = action {
            if policy_delta.removes_rules() {
                return Err(ControllerError::AxiomViolation(
                    "Κ1: Regeln können nur hinzugefügt, nie entfernt werden".into()
                ));
            }
        }

        // 4. Κ19-Prüfung: Power-Cap
        let actor_power = self.calculate_actor_power(actor)?;
        if actor_power > self.power_cap() {
            return Err(ControllerError::PowerCapExceeded);
        }

        Ok(ValidationResult::Allowed {
            effective_permissions,
            power_used: actor_power,
        })
    }

    /// Delegiere Kontrolle (Κ8)
    pub fn delegate(
        &mut self,
        scope: &ScopeId,
        from: &UniversalId,
        to: &UniversalId,
        delegation: Delegation,
    ) -> Result<()> {
        // Validiere dass 'from' berechtigt ist zu delegieren
        self.validate_action(scope, from, &ControllerAction::Delegate)?;

        // Prüfe Zyklen (Κ8: Delegation ist DAG)
        if self.delegations.would_create_cycle(from, to) {
            return Err(ControllerError::DelegationCycleDetected);
        }

        // Trust-Dampening anwenden
        let effective_trust = self.calculate_delegation_trust(from, &delegation)?;

        // Speichere Delegation
        self.delegations.add(from, to, delegation, effective_trust)?;

        // Audit
        self.audit.log(AuditEntry::Delegation {
            from: from.clone(),
            to: to.clone(),
            scope: scope.clone(),
            timestamp: TemporalCoord::now(),
        });

        Ok(())
    }
}
```

### 3.0.4 API-Engine (Neu) – Dynamische REST-API per ECL

Die API-Engine ermöglicht die deklarative Definition von REST-APIs für externe Systeme:

```rust
// Konzeptionell: backend/src/eclvm/api_engine.rs

/// API-Engine für dynamische REST-API-Definition per ECL
pub struct APIEngine {
    /// Registrierte API-Endpoints pro Realm/Raum
    endpoints: HashMap<ScopeId, Vec<CompiledEndpoint>>,
    /// Rate-Limiter pro Client
    rate_limiters: HashMap<UniversalId, RateLimiter>,
    /// API-Key-Registry (für externe Systeme)
    api_keys: APIKeyRegistry,
    /// Schema-Validator
    schema_validator: SchemaValidator,
}

/// Kompilierter API-Endpoint
pub struct CompiledEndpoint {
    pub path: String,                      // "/api/v1/room/{room_id}/events"
    pub method: HttpMethod,                // GET, POST, PUT, DELETE, PATCH
    pub handler: Vec<OpCode>,              // ECL-Bytecode für Handler-Logik
    pub request_schema: Option<JSONSchema>, // Input-Validierung
    pub response_schema: Option<JSONSchema>,// Output-Struktur
    pub auth: EndpointAuth,                // Authentifizierung
    pub rate_limit: RateLimitConfig,       // Rate-Limiting
    pub trust_gate: Option<f32>,           // Minimaler Trust für Zugriff
    pub credential_gate: Vec<String>,      // Benötigte Credentials
    pub caching: CacheConfig,              // Response-Caching
}

/// Authentifizierungsmodi für Endpoints
pub enum EndpointAuth {
    /// Keine Authentifizierung (öffentlich)
    Public,
    /// API-Key basiert (für externe Services)
    APIKey {
        scopes: Vec<String>,  // "read:events", "write:data"
    },
    /// Peer-DID-basiert (für Erynoa-Peers)
    PeerAuth {
        required_trust: f32,
        required_credentials: Vec<String>,
    },
    /// Webhook-Signatur (für eingehende Webhooks)
    WebhookSignature {
        algorithm: SignatureAlgorithm,
        header_name: String,
    },
    /// OAuth2 (für Third-Party-Apps)
    OAuth2 {
        provider: String,
        scopes: Vec<String>,
    },
}

/// Rate-Limit-Konfiguration
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub tier_multipliers: HashMap<TrustTier, f32>, // Höherer Trust = mehr Requests
}

impl APIEngine {
    /// Registriere neuen Endpoint aus ECL-Definition
    pub fn register_endpoint(
        &mut self,
        scope: &ScopeId,
        definition: &ECLAPIEndpoint,
        controller: &UniversalId,
    ) -> Result<EndpointId> {
        // 1. Validiere Controller-Berechtigung
        self.validate_api_permission(scope, controller)?;

        // 2. Kompiliere Handler-Logik
        let compiled_handler = self.compile_handler(&definition.handler)?;

        // 3. Validiere Schemas
        if let Some(req_schema) = &definition.request_schema {
            self.schema_validator.validate_schema(req_schema)?;
        }

        // 4. Prüfe auf Pfad-Konflikte
        self.check_path_conflicts(scope, &definition.path, &definition.method)?;

        // 5. Registriere Endpoint
        let endpoint = CompiledEndpoint {
            path: definition.path.clone(),
            method: definition.method.clone(),
            handler: compiled_handler,
            request_schema: definition.request_schema.clone(),
            response_schema: definition.response_schema.clone(),
            auth: definition.auth.clone(),
            rate_limit: definition.rate_limit.clone().unwrap_or_default(),
            trust_gate: definition.trust_gate,
            credential_gate: definition.credential_gate.clone(),
            caching: definition.caching.clone().unwrap_or_default(),
        };

        let endpoint_id = EndpointId::new();
        self.endpoints.entry(scope.clone())
            .or_insert_with(Vec::new)
            .push(endpoint);

        Ok(endpoint_id)
    }

    /// Handle eingehenden API-Request
    pub async fn handle_request(
        &self,
        scope: &ScopeId,
        request: APIRequest,
        vm: &mut ECLVM,
    ) -> Result<APIResponse> {
        // 1. Route finden
        let endpoint = self.find_endpoint(scope, &request.path, &request.method)?;

        // 2. Authentifizierung prüfen
        let auth_context = self.authenticate(&request, &endpoint.auth).await?;

        // 3. Rate-Limit prüfen
        self.check_rate_limit(&auth_context.client_id, &endpoint.rate_limit)?;

        // 4. Trust-Gate prüfen (falls Peer-Auth)
        if let Some(min_trust) = endpoint.trust_gate {
            if auth_context.trust_level < min_trust {
                return Err(APIError::InsufficientTrust);
            }
        }

        // 5. Request validieren
        if let Some(schema) = &endpoint.request_schema {
            self.schema_validator.validate(&request.body, schema)?;
        }

        // 6. Handler ausführen (ECL-Bytecode)
        vm.push(request.to_value());
        vm.push(auth_context.to_value());
        let result = vm.execute(&endpoint.handler)?;

        // 7. Response validieren
        if let Some(schema) = &endpoint.response_schema {
            self.schema_validator.validate(&result, schema)?;
        }

        // 8. Caching-Header setzen
        let cache_headers = self.generate_cache_headers(&endpoint.caching);

        Ok(APIResponse {
            status: 200,
            body: result,
            headers: cache_headers,
        })
    }

    /// Generiere OpenAPI-Spec aus registrierten Endpoints
    pub fn generate_openapi_spec(&self, scope: &ScopeId) -> Result<OpenAPISpec> {
        let endpoints = self.endpoints.get(scope)
            .ok_or(APIError::ScopeNotFound)?;

        let mut spec = OpenAPISpec::new("Erynoa Realm API", "1.0.0");

        for endpoint in endpoints {
            spec.add_path(
                &endpoint.path,
                &endpoint.method,
                PathOperation {
                    request_body: endpoint.request_schema.clone(),
                    response: endpoint.response_schema.clone(),
                    security: self.auth_to_security(&endpoint.auth),
                    rate_limit: Some(endpoint.rate_limit.clone()),
                },
            );
        }

        Ok(spec)
    }
}

/// ECL API-Definition Syntax
pub struct ECLAPIEndpoint {
    pub path: String,
    pub method: HttpMethod,
    pub handler: String,  // ECL-Code
    pub request_schema: Option<JSONSchema>,
    pub response_schema: Option<JSONSchema>,
    pub auth: EndpointAuth,
    pub rate_limit: Option<RateLimitConfig>,
    pub trust_gate: Option<f32>,
    pub credential_gate: Vec<String>,
    pub caching: Option<CacheConfig>,
}
```

**ECL-Syntax für API-Definition:**

```ecl
// API-Definition in ECL für einen Raum
api "event-api" {
    version = "1.0"
    base_path = "/api/v1"

    // Globale Authentifizierung
    auth {
        default = "api_key"
        api_key_header = "X-Erynoa-Key"
    }

    // Events abrufen
    endpoint GET "/events" {
        description = "Liste aller Events im Raum"

        auth = api_key { scopes = ["read:events"] }
        trust_gate = 0.3

        rate_limit {
            requests_per_minute = 100
            burst = 20
        }

        response_schema = {
            type = "array"
            items = { $ref = "#/schemas/Event" }
        }

        handler = """
            // ECL-Handler-Code
            events.filter(|e| e.timestamp > params.since)
                  .take(params.limit || 50)
                  .map(|e| e.to_public())
        """

        caching {
            ttl = 30  // Sekunden
            vary_by = ["Authorization", "Accept"]
        }
    }

    // Event erstellen
    endpoint POST "/events" {
        description = "Neues Event erstellen"

        auth = peer_auth {
            min_trust = 0.5
            credentials = ["event_creator"]
        }

        request_schema = {
            type = "object"
            required = ["type", "payload"]
            properties = {
                type = { type = "string", enum = ["message", "update", "action"] }
                payload = { type = "object" }
            }
        }

        handler = """
            let event = Event::new(request.type, request.payload)
            event.validate()?
            events.emit(event)
            return { id: event.id, status: "created" }
        """
    }

    // Webhook für externe Systeme
    endpoint POST "/webhooks/incoming/{provider}" {
        description = "Webhook-Empfang von externen Systemen"

        auth = webhook_signature {
            algorithm = "hmac-sha256"
            header = "X-Webhook-Signature"
        }

        handler = """
            let provider = params.provider
            let payload = request.body

            match provider {
                "github" => process_github_webhook(payload),
                "stripe" => process_stripe_webhook(payload),
                _ => return error("Unknown provider")
            }
        """
    }
}

// Schema-Definitionen
schemas {
    Event = {
        type = "object"
        properties = {
            id = { type = "string", format = "uuid" }
            type = { type = "string" }
            payload = { type = "object" }
            timestamp = { type = "string", format = "date-time" }
            author = { type = "string", format = "did" }
        }
    }
}
```

### 3.0.5 Governance-Engine (Neu) – DAO-Prinzipien per ECL

Die Governance-Engine implementiert dezentrale Entscheidungsfindung mit DAO-Patterns:

```rust
// Konzeptionell: backend/src/eclvm/governance_engine.rs

/// Governance-Engine für DAO-basierte Entscheidungen
pub struct GovernanceEngine {
    /// Governance-Konfigurationen pro Scope
    configs: HashMap<ScopeId, GovernanceConfig>,
    /// Aktive Proposals
    proposals: HashMap<ProposalId, Proposal>,
    /// Abstimmungshistorie
    vote_history: VoteHistory,
    /// Timelock-Queue (verzögerte Ausführung)
    timelock_queue: TimelockQueue,
}

/// Governance-Konfiguration (DAO-Settings)
pub struct GovernanceConfig {
    /// Governance-Modus
    pub mode: GovernanceMode,
    /// Abstimmungsregeln
    pub voting_rules: VotingRules,
    /// Wer kann Proposals erstellen?
    pub proposal_threshold: ProposalThreshold,
    /// Timelock-Dauer vor Ausführung
    pub timelock_duration: Duration,
    /// Quorum-Berechnung
    pub quorum_calculator: QuorumCalculator,
    /// Veto-Mechanismus
    pub veto_config: Option<VetoConfig>,
    /// Delegation erlaubt?
    pub delegation_enabled: bool,
    /// Trust-Gewichtung bei Abstimmungen
    pub trust_weighted_voting: bool,
}

/// Governance-Modi
pub enum GovernanceMode {
    /// Einzelner Controller (klassisch)
    SingleController {
        controller: UniversalId,
    },
    /// Multi-Sig (mehrere müssen zustimmen)
    MultiSig {
        signers: Vec<UniversalId>,
        threshold: u32,  // z.B. 3 von 5
    },
    /// DAO (Token/Trust-basierte Abstimmung)
    DAO {
        voting_power: VotingPowerSource,
    },
    /// Optimistic (Änderungen gelten, wenn kein Veto)
    Optimistic {
        challenge_period: Duration,
        veto_threshold: f32,
    },
    /// Futarchy (Prediction-Market-basiert)
    Futarchy {
        market_duration: Duration,
        resolution_source: ResolutionSource,
    },
    /// Conviction Voting (Zeit-gewichtete Stimmen)
    ConvictionVoting {
        decay_rate: f32,
        max_conviction: f32,
    },
    /// Liquid Democracy (delegierte Stimmen)
    LiquidDemocracy {
        max_delegation_depth: u32,
    },
}

/// Abstimmungskraft-Quelle
pub enum VotingPowerSource {
    /// 1 Peer = 1 Stimme
    EqualVoting,
    /// Trust-basiert (6D-Vektor)
    TrustBased {
        dimension_weights: [f32; 6],
    },
    /// Reputation-basiert (aus Attestations)
    ReputationBased {
        reputation_metric: String,
    },
    /// Aktivitäts-basiert
    ActivityBased {
        lookback_period: Duration,
        activity_weights: ActivityWeights,
    },
    /// Quadratic Voting (√ der Stimmen)
    Quadratic,
    /// Custom (ECL-definiert)
    Custom {
        calculator: Vec<OpCode>,
    },
}

/// Abstimmungsregeln
pub struct VotingRules {
    /// Abstimmungsdauer
    pub voting_period: Duration,
    /// Minimales Quorum (% der stimmberechtigten)
    pub min_quorum: f32,
    /// Zustimmungsschwelle für Annahme
    pub approval_threshold: f32,
    /// Erlaubte Stimmentypen
    pub vote_options: VoteOptions,
    /// Änderung der Stimme erlaubt?
    pub vote_change_allowed: bool,
    /// Stimme geheim bis Ende?
    pub secret_until_end: bool,
}

/// Stimmenoptionen
pub enum VoteOptions {
    /// Ja/Nein
    Binary,
    /// Ja/Nein/Enthaltung
    YesNoAbstain,
    /// Für/Gegen mit Gewichtung
    ForAgainstWeighted,
    /// Multiple Choice
    MultipleChoice { max_choices: u32 },
    /// Rangfolge (Ranked Choice)
    RankedChoice { candidates: u32 },
}

/// Proposal
pub struct Proposal {
    pub id: ProposalId,
    pub proposer: UniversalId,
    pub title: String,
    pub description: String,
    pub actions: Vec<ProposalAction>,
    pub created_at: TemporalCoord,
    pub voting_starts: TemporalCoord,
    pub voting_ends: TemporalCoord,
    pub state: ProposalState,
    pub votes: HashMap<UniversalId, Vote>,
    pub execution_hash: Hash,
}

/// Proposal-Aktionen (was bei Annahme passiert)
pub enum ProposalAction {
    /// ECL-Policy ändern
    ModifyPolicy {
        scope: ScopeId,
        policy_delta: ECLPolicyDelta,
    },
    /// UI ändern
    ModifyUI {
        room_id: RoomId,
        ui_delta: UIDelta,
    },
    /// API ändern
    ModifyAPI {
        endpoint_changes: Vec<APIChange>,
    },
    /// Struktur ändern (Raum/Partition)
    ModifyStructure {
        structure_delta: StructureDelta,
    },
    /// Controller ändern
    ModifyController {
        new_controller: ControllerConfig,
    },
    /// Governance selbst ändern
    ModifyGovernance {
        governance_delta: GovernanceDelta,
    },
    /// Budget/Ressourcen allokieren
    AllocateResources {
        allocations: Vec<ResourceAllocation>,
    },
    /// Custom ECL-Code ausführen
    ExecuteECL {
        code: Vec<OpCode>,
    },
}

/// Proposal-Status
pub enum ProposalState {
    Draft,
    Pending,          // Wartet auf Voting-Start
    Active,           // Abstimmung läuft
    Succeeded,        // Angenommen, in Timelock
    Defeated,         // Abgelehnt
    Queued,           // In Timelock-Queue
    Executed,         // Ausgeführt
    Cancelled,        // Abgebrochen
    Vetoed,           // Durch Veto gestoppt
    Expired,          // Timelock abgelaufen ohne Ausführung
}

impl GovernanceEngine {
    /// Erstelle neues Proposal
    pub fn create_proposal(
        &mut self,
        scope: &ScopeId,
        proposer: &UniversalId,
        proposal: ProposalDraft,
    ) -> Result<ProposalId> {
        let config = self.configs.get(scope)
            .ok_or(GovernanceError::ScopeNotConfigured)?;

        // 1. Prüfe Proposal-Berechtigung
        let proposer_power = self.calculate_voting_power(scope, proposer)?;
        if !config.proposal_threshold.can_propose(proposer_power) {
            return Err(GovernanceError::InsufficientProposalPower);
        }

        // 2. Validiere Aktionen (Κ1-konform)
        for action in &proposal.actions {
            self.validate_action_axiom_compliance(action)?;
        }

        // 3. Berechne Execution-Hash (für Manipulation-Schutz)
        let execution_hash = self.compute_execution_hash(&proposal.actions)?;

        // 4. Erstelle Proposal
        let id = ProposalId::new();
        let now = TemporalCoord::now();

        let full_proposal = Proposal {
            id: id.clone(),
            proposer: proposer.clone(),
            title: proposal.title,
            description: proposal.description,
            actions: proposal.actions,
            created_at: now,
            voting_starts: now + config.voting_rules.delay_before_voting,
            voting_ends: now + config.voting_rules.delay_before_voting
                            + config.voting_rules.voting_period,
            state: ProposalState::Pending,
            votes: HashMap::new(),
            execution_hash,
        };

        self.proposals.insert(id.clone(), full_proposal);

        Ok(id)
    }

    /// Stimme für Proposal ab
    pub fn cast_vote(
        &mut self,
        proposal_id: &ProposalId,
        voter: &UniversalId,
        vote: VoteChoice,
    ) -> Result<()> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        // 1. Prüfe ob Abstimmung aktiv
        if proposal.state != ProposalState::Active {
            return Err(GovernanceError::VotingNotActive);
        }

        let scope = self.get_proposal_scope(proposal)?;
        let config = self.configs.get(&scope).unwrap();

        // 2. Prüfe ob bereits abgestimmt (und ob Änderung erlaubt)
        if proposal.votes.contains_key(voter) && !config.voting_rules.vote_change_allowed {
            return Err(GovernanceError::AlreadyVoted);
        }

        // 3. Berechne Stimmkraft
        let voting_power = self.calculate_voting_power(&scope, voter)?;

        // 4. Bei Liquid Democracy: Prüfe Delegationen
        let effective_power = if config.delegation_enabled {
            self.calculate_delegated_power(&scope, voter, proposal_id)?
        } else {
            voting_power
        };

        // 5. Speichere Stimme
        proposal.votes.insert(voter.clone(), Vote {
            choice: vote,
            power: effective_power,
            timestamp: TemporalCoord::now(),
        });

        // 6. Log für Audit
        self.vote_history.log(VoteEntry {
            proposal_id: proposal_id.clone(),
            voter: voter.clone(),
            choice: vote,
            power: effective_power,
        });

        Ok(())
    }

    /// Führe angenommenes Proposal aus
    pub fn execute_proposal(
        &mut self,
        proposal_id: &ProposalId,
        executor: &UniversalId,
    ) -> Result<ExecutionReceipt> {
        let proposal = self.proposals.get_mut(proposal_id)
            .ok_or(GovernanceError::ProposalNotFound)?;

        // 1. Prüfe Status
        if proposal.state != ProposalState::Queued {
            return Err(GovernanceError::NotReadyForExecution);
        }

        // 2. Prüfe Timelock abgelaufen
        let timelock_entry = self.timelock_queue.get(proposal_id)?;
        if timelock_entry.execute_after > TemporalCoord::now() {
            return Err(GovernanceError::TimelockNotExpired);
        }

        // 3. Verifiziere Execution-Hash (keine Manipulation)
        let current_hash = self.compute_execution_hash(&proposal.actions)?;
        if current_hash != proposal.execution_hash {
            return Err(GovernanceError::ExecutionHashMismatch);
        }

        // 4. Führe Aktionen aus
        let mut results = Vec::new();
        for action in &proposal.actions {
            let result = self.execute_action(action)?;
            results.push(result);
        }

        // 5. Update Status
        proposal.state = ProposalState::Executed;

        Ok(ExecutionReceipt {
            proposal_id: proposal_id.clone(),
            executed_at: TemporalCoord::now(),
            executor: executor.clone(),
            action_results: results,
        })
    }

    /// Berechne Stimmkraft basierend auf Konfiguration
    fn calculate_voting_power(
        &self,
        scope: &ScopeId,
        voter: &UniversalId,
    ) -> Result<f32> {
        let config = self.configs.get(scope)
            .ok_or(GovernanceError::ScopeNotConfigured)?;

        match &config.mode {
            GovernanceMode::DAO { voting_power } => {
                match voting_power {
                    VotingPowerSource::EqualVoting => Ok(1.0),

                    VotingPowerSource::TrustBased { dimension_weights } => {
                        let trust = self.get_voter_trust(voter)?;
                        Ok(trust.weighted_norm(dimension_weights))
                    }

                    VotingPowerSource::Quadratic => {
                        // √ der "Basis-Power" (z.B. Aktivität)
                        let base_power = self.get_base_power(voter)?;
                        Ok(base_power.sqrt())
                    }

                    VotingPowerSource::Custom { calculator } => {
                        // ECL-Code zur Berechnung ausführen
                        let mut vm = ECLVM::new();
                        vm.push(voter.to_value());
                        vm.execute(calculator)?;
                        vm.pop_f32()
                    }

                    _ => Ok(1.0),
                }
            }

            GovernanceMode::ConvictionVoting { decay_rate, max_conviction } => {
                // Zeit-gewichtete Überzeugung
                self.calculate_conviction(scope, voter, *decay_rate, *max_conviction)
            }

            _ => Ok(1.0),
        }
    }
}
```

**ECL-Syntax für Governance-Definition:**

```ecl
// Governance-Definition in ECL
governance "realm-dao" {
    mode = dao {
        voting_power = trust_based {
            weights = [0.3, 0.2, 0.2, 0.1, 0.1, 0.1]  // 6D-Trust-Gewichte
        }
    }

    voting_rules {
        voting_period = "7d"
        min_quorum = 0.15           // 15% müssen abstimmen
        approval_threshold = 0.66   // 66% Zustimmung nötig
        vote_options = yes_no_abstain
        vote_change_allowed = true
        secret_until_end = false
    }

    proposal_threshold {
        min_trust = 0.5
        or_credentials = ["council_member", "senior_contributor"]
    }

    timelock_duration = "48h"

    delegation_enabled = true

    // Veto für kritische Änderungen
    veto {
        enabled = true
        veto_holders = credential("veto_power")
        veto_window = "24h"
        applies_to = [
            "ModifyGovernance",
            "ModifyController",
        ]
    }

    // Verschiedene Regeln für verschiedene Aktionstypen
    action_rules {
        // UI-Änderungen: Schnell
        ModifyUI {
            voting_period = "24h"
            min_quorum = 0.05
            approval_threshold = 0.5
            timelock = "1h"
        }

        // Policy-Änderungen: Standard
        ModifyPolicy {
            voting_period = "7d"
            min_quorum = 0.15
            approval_threshold = 0.66
        }

        // Governance-Änderungen: Streng
        ModifyGovernance {
            voting_period = "14d"
            min_quorum = 0.30
            approval_threshold = 0.75
            timelock = "7d"
        }
    }
}

// Liquid Democracy Variante
governance "liquid-realm" {
    mode = liquid_democracy {
        max_delegation_depth = 5
    }

    voting_rules {
        voting_period = "5d"
        min_quorum = 0.10
        approval_threshold = 0.5
        vote_options = ranked_choice { candidates = 5 }
    }

    // Automatische Delegation basierend auf Tags
    auto_delegation {
        enabled = true
        tag_based = true
        // Peers können Experten für bestimmte Themen folgen
        topic_tags = ["security", "ui", "protocol", "economics"]
    }
}

// Optimistic Governance (für schnelle Entscheidungen)
governance "optimistic-realm" {
    mode = optimistic {
        challenge_period = "48h"
        veto_threshold = 0.10  // 10% können Veto einlegen
    }

    // Trusted Proposers können ohne Voting ändern
    trusted_proposers {
        role = "maintainer"
        min_trust = 0.8
    }

    // Außer für diese kritischen Aktionen
    always_vote_on = [
        "ModifyGovernance",
        "ModifyController",
        "AllocateResources { amount > 1000 }",
    ]
}
```

### 3.0.6 Extended Controller-Engine mit DAO-Integration

Die Controller-Engine integriert nun nahtlos mit der Governance-Engine:

```rust
// Erweiterung: backend/src/eclvm/controller_engine.rs

/// Erweiterte Controller-Konfiguration mit DAO-Support
pub struct ExtendedControllerConfig {
    /// Basis-Controller-Einstellungen
    pub base: ControllerConfig,
    /// Governance-Integration
    pub governance: Option<GovernanceBinding>,
    /// Automatische Aktionen
    pub automation: Vec<AutomationRule>,
    /// Audit-Einstellungen
    pub audit_config: AuditConfig,
}

/// Bindung an Governance-Engine
pub struct GovernanceBinding {
    /// Welche Governance-Config gilt?
    pub governance_id: String,
    /// Welche Aktionen benötigen Governance?
    pub governed_actions: Vec<GovernedActionRule>,
    /// Emergency-Bypass erlaubt?
    pub emergency_bypass: Option<EmergencyConfig>,
}

/// Regel für governance-pflichtige Aktionen
pub struct GovernedActionRule {
    /// Aktion-Pattern (z.B. "ModifyPolicy:*" oder "ModifyUI:critical")
    pub action_pattern: String,
    /// Ab welchem Schweregrad?
    pub severity_threshold: ActionSeverity,
    /// Immer Governance oder nur bei bestimmten Bedingungen?
    pub condition: Option<Vec<OpCode>>,
}

/// Automatisierungsregeln
pub struct AutomationRule {
    pub id: String,
    pub trigger: AutomationTrigger,
    pub condition: Option<Vec<OpCode>>,
    pub action: AutomationAction,
    pub requires_governance: bool,
}

/// Automatisierungs-Trigger
pub enum AutomationTrigger {
    /// Zeitbasiert (Cron)
    Schedule { cron: String },
    /// Event-basiert
    OnEvent { event_pattern: String },
    /// Schwellwert-basiert
    Threshold { metric: String, operator: ThresholdOp, value: f32 },
    /// External Webhook
    Webhook { endpoint_id: EndpointId },
}

impl ControllerEngine {
    /// Validiere Aktion mit Governance-Check
    pub fn validate_action_with_governance(
        &self,
        scope: &ScopeId,
        actor: &UniversalId,
        action: &ControllerAction,
        governance_engine: &GovernanceEngine,
    ) -> Result<ValidationResult> {
        // 1. Basis-Validierung
        let base_result = self.validate_action(scope, actor, action)?;

        // 2. Prüfe ob Governance erforderlich
        let config = self.controllers.get(scope).unwrap();
        if let Some(gov_binding) = &config.governance {
            if self.requires_governance(action, gov_binding) {
                return Ok(ValidationResult::RequiresGovernance {
                    governance_id: gov_binding.governance_id.clone(),
                    required_action: action.clone(),
                    estimated_voting_period: governance_engine
                        .estimate_voting_period(&gov_binding.governance_id, action)?,
                });
            }
        }

        Ok(base_result)
    }

    /// Führe Aktion aus (direkt oder via Governance)
    pub async fn execute_action(
        &mut self,
        scope: &ScopeId,
        actor: &UniversalId,
        action: ControllerAction,
        governance_engine: &mut GovernanceEngine,
    ) -> Result<ActionResult> {
        // Validiere mit Governance-Check
        let validation = self.validate_action_with_governance(
            scope, actor, &action, governance_engine
        )?;

        match validation {
            ValidationResult::Allowed { .. } => {
                // Direkt ausführen
                self.execute_action_internal(scope, &action).await
            }

            ValidationResult::RequiresGovernance { governance_id, .. } => {
                // Proposal erstellen
                let proposal_id = governance_engine.create_proposal(
                    scope,
                    actor,
                    ProposalDraft {
                        title: format!("Action: {}", action.describe()),
                        description: action.detailed_description(),
                        actions: vec![action.to_proposal_action()],
                    },
                )?;

                Ok(ActionResult::ProposalCreated {
                    proposal_id,
                    governance_id,
                    message: "Aktion benötigt Governance-Abstimmung".into(),
                })
            }

            _ => Err(ControllerError::ValidationFailed),
        }
    }
}
```

**ECL-Syntax für erweiterte Controller-Definition:**

```ecl
// Controller mit DAO-Integration
controller "community-controlled" {
    // Basis-Controller (für Emergency)
    primary = did:erynoa:admin123

    // Governance-Bindung
    governance {
        config = "realm-dao"  // Referenz auf governance-Definition

        // Welche Aktionen brauchen Abstimmung?
        governed_actions {
            // Alle Policy-Änderungen
            "ModifyPolicy:*" {
                severity = minor
                always_vote = true
            }

            // UI-Änderungen nur bei kritischen Komponenten
            "ModifyUI:*" {
                severity = major
                condition = """
                    action.affects_critical_components()
                    || action.changes_permissions()
                """
            }

            // API-Änderungen immer
            "ModifyAPI:*" {
                severity = major
                always_vote = true
            }

            // Struktur-Änderungen immer
            "ModifyStructure:*" {
                severity = critical
                always_vote = true
            }
        }

        // Emergency-Bypass
        emergency {
            enabled = true
            bypass_holders = [
                did:erynoa:emergency_council_1,
                did:erynoa:emergency_council_2,
            ]
            requires_multisig = 2  // 2 von 2
            max_duration = "24h"
            post_action = "mandatory_review"
        }
    }

    // Berechtigungen (wenn keine Governance nötig)
    permissions {
        modify_ui_minor = true
        view_analytics = true
        manage_api_keys = true
    }

    // Automatisierung
    automation {
        // Automatische Trust-Anpassung
        rule "auto-trust-decay" {
            trigger = schedule { cron = "0 0 * * *" }  // Täglich
            action = """
                members.filter(|m| m.inactive_days > 90)
                       .each(|m| m.trust *= 0.95)
            """
            requires_governance = false
        }

        // Automatischer Raum-Cleanup
        rule "auto-cleanup" {
            trigger = threshold {
                metric = "room.storage_usage_percent"
                operator = gt
                value = 90
            }
            action = """
                old_data = room.data.filter(|d| d.age > "30d")
                archive(old_data)
            """
            requires_governance = false  // Kleine Aktionen
        }

        // Automatische Eskalation
        rule "security-escalation" {
            trigger = event { pattern = "security:alert:*" }
            condition = """
                event.severity >= "high"
            """
            action = """
                notify_all(role = "security_team")
                if event.severity == "critical" {
                    enable_emergency_mode()
                }
            """
            requires_governance = false  // Sicherheit hat Vorrang
        }
    }

    // Delegation mit Trust-Limits
    delegation {
        enabled = true
        max_depth = 3
        trust_decay_per_level = 0.8  // 80% Trust weitergegeben

        // Bestimmte Berechtigungen nicht delegierbar
        non_delegatable = [
            "modify_governance",
            "emergency_actions",
        ]
    }

    // Audit-Konfiguration
    audit {
        log_all_actions = true
        retention = "365d"
        export_format = "json"
        notify_on = ["governance_change", "emergency_action"]
    }
}
```

### 3.0.7 Erweiterte Use-Cases durch ECL-Integration

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                      ║
║   ECL USE-CASE MATRIX – ALLE FÄHIGKEITEN                                                            ║
║                                                                                                      ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                                             │   ║
║   │   USE-CASE                          │  ECL-KOMPONENTEN                                     │   ║
║   │   ─────────────────────────────────────────────────────────────────────────────────────   │   ║
║   │                                                                                             │   ║
║   │   1. Community-Forum                │  structure + ui + api + governance(dao)              │   ║
║   │      └─ Threads, Votes, Moderation  │  └─ Moderatoren durch Voting gewählt                │   ║
║   │                                                                                             │   ║
║   │   2. Kollaboratives Dokument        │  structure + ui + datalogic + controller             │   ║
║   │      └─ Real-time Editing           │  └─ Änderungen via DataLogic synchronisiert         │   ║
║   │                                                                                             │   ║
║   │   3. Marktplatz                     │  structure + ui + api + governance(optimistic)       │   ║
║   │      └─ Listings, Escrow, Reviews   │  └─ Neue Listings ohne Vote, Regeln mit Vote        │   ║
║   │                                                                                             │   ║
║   │   4. DAO Treasury                   │  controller(multisig) + governance(conviction)       │   ║
║   │      └─ Budget-Allokation           │  └─ Conviction Voting für Ausgaben                  │   ║
║   │                                                                                             │   ║
║   │   5. Identity-Provider              │  api + policy + attestations                         │   ║
║   │      └─ OAuth2, Credentials         │  └─ REST-API für externe Auth                       │   ║
║   │                                                                                             │   ║
║   │   6. IoT-Gateway                    │  api(webhooks) + datalogic + automation              │   ║
║   │      └─ Sensor-Daten, Alerts        │  └─ Automatische Reaktionen auf Schwellwerte        │   ║
║   │                                                                                             │   ║
║   │   7. Event-Streaming-Platform       │  api + datalogic + ui(reactive)                      │   ║
║   │      └─ Pub/Sub, Dashboards         │  └─ Live-Updates via Bindings                       │   ║
║   │                                                                                             │   ║
║   │   8. Governance-as-a-Service        │  governance(*) + api + ui                            │   ║
║   │      └─ Abstimmungen, Proposals     │  └─ Verschiedene Governance-Modi                    │   ║
║   │                                                                                             │   ║
║   │   9. Supply-Chain-Tracking          │  attestations + api + datalogic                      │   ║
║   │      └─ Provenance, Verification    │  └─ Externe Systeme via API angebunden              │   ║
║   │                                                                                             │   ║
║   │  10. Multi-Tenant SaaS              │  structure(partitions) + api + controller(tenant)    │   ║
║   │      └─ Isolation, Customization    │  └─ Jeder Tenant eigene Partition                   │   ║
║   │                                                                                             │   ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────┘   ║
║                                                                                                      ║
║   INTEGRATION FLOW:                                                                                  ║
║                                                                                                      ║
║   ┌────────────┐     ┌───────────┐     ┌───────────┐     ┌────────────┐     ┌───────────┐          ║
║   │ External   │────►│ API       │────►│ DataLogic │────►│ Governance │────►│ State     │          ║
║   │ System     │◄────│ Engine    │◄────│ Engine    │◄────│ Engine     │◄────│ Update    │          ║
║   └────────────┘     └───────────┘     └───────────┘     └────────────┘     └───────────┘          ║
║         │                 │                 │                 │                 │                   ║
║         │                 ▼                 ▼                 ▼                 │                   ║
║         │           ┌───────────┐     ┌───────────┐     ┌───────────┐          │                   ║
║         └──────────►│ UI Engine │────►│Controller │────►│ ECLVM     │◄─────────┘                   ║
║                     │ (Render)  │     │ Engine    │     │ Runtime   │                              ║
║                     └───────────┘     └───────────┘     └───────────┘                              ║
║                                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 3.0.8 Vollständiges ECL-Beispiel: Community-Platform

```ecl
// ═══════════════════════════════════════════════════════════════════════════════
// VOLLSTÄNDIGES ECL-BEISPIEL: Community-Platform mit DAO-Governance
// ═══════════════════════════════════════════════════════════════════════════════

realm "community-platform" {
    version = "1.0"
    description = "Dezentrale Community-Platform mit DAO-Governance"

    // ─────────────────────────────────────────────────────────────────────────
    // STRUKTUR: Räume und Partitionen
    // ─────────────────────────────────────────────────────────────────────────

    structure {
        room "general" {
            description = "Allgemeiner Diskussionsraum"

            partition "announcements" {
                write_access = credential("announcer")
                read_access = all
            }

            partition "discussions" {
                write_access = trust_min(0.3)
                read_access = all
            }
        }

        room "governance" {
            description = "DAO-Governance und Abstimmungen"

            partition "proposals" {
                write_access = trust_min(0.5) | credential("council")
                read_access = all
            }

            partition "voting" {
                write_access = trust_min(0.2)  // Jeder vertrauenswürdige Peer
                read_access = all
            }
        }

        room "projects" {
            description = "Projekt-Workspaces"
            dynamic = true  // Räume können dynamisch erstellt werden

            template "project-room" {
                partitions = ["docs", "tasks", "discussion"]
                default_policy = inherit_from("projects")
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // POLICY: Regeln und Berechtigungen
    // ─────────────────────────────────────────────────────────────────────────

    policy {
        // Basis-Regeln (Κ1: nur hinzufügen, nie entfernen)
        base_rules {
            no_spam = rate_limit(posts_per_hour = 10)
            no_harassment = content_filter(blocklist = "harassment")
            require_verification = attestation("email_verified")
        }

        // Trust-basierte Berechtigungen
        trust_tiers {
            newcomer { min = 0.0, max = 0.3 }
            member { min = 0.3, max = 0.6 }
            trusted { min = 0.6, max = 0.8 }
            elder { min = 0.8, max = 1.0 }
        }

        // Moderations-Regeln
        moderation {
            report_threshold = 3  // 3 Reports = Review
            auto_hide_threshold = 5  // 5 Reports = Auto-Hide
            appeal_period = "48h"

            moderators = credential("moderator") | trust_tier("elder")
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // UI: Interface-Definition
    // ─────────────────────────────────────────────────────────────────────────

    ui {
        theme {
            primary_color = "#4A90D9"
            font_family = "Inter, sans-serif"
        }

        layout "main" {
            component sidebar {
                type = navigation
                items = [
                    { icon = "home", route = "/", label = "Home" },
                    { icon = "chat", route = "/rooms", label = "Räume" },
                    { icon = "vote", route = "/governance", label = "Governance" },
                    { icon = "user", route = "/profile", label = "Profil" },
                ]
            }

            component content {
                type = router_outlet
                transitions = "fade"
            }
        }

        page "room-view" {
            route = "/rooms/:room_id"

            component room_header {
                type = header
                title = bind("room.name")
                subtitle = bind("room.description")

                // Nur für Moderatoren sichtbar
                actions {
                    trust_gate = 0.6
                    items = [
                        { icon = "settings", action = "room_settings" },
                        { icon = "users", action = "manage_members" },
                    ]
                }
            }

            component post_list {
                type = list

                data = bind("""
                    events
                        .filter(|e| e.type == "post" && e.room == params.room_id)
                        .sort_by(|e| -e.timestamp)
                        .take(50)
                """)

                item_template = """
                    <PostCard
                        author={item.author}
                        content={item.content}
                        timestamp={item.timestamp}
                        reactions={item.reactions}
                        onReact={react_to_post(item.id)}
                    />
                """
            }

            component post_composer {
                type = rich_editor
                trust_gate = 0.3
                credential_gate = ["email_verified"]

                on_submit = """
                    let post = Post::new(input.content)
                    post.room = params.room_id
                    events.emit("post", post)
                """
            }
        }

        page "governance-view" {
            route = "/governance"

            component proposal_list {
                type = tabs

                tab "active" {
                    label = "Aktive Abstimmungen"
                    content = bind("""
                        governance.proposals
                            .filter(|p| p.state == "active")
                            .map(|p| ProposalCard(p))
                    """)
                }

                tab "pending" {
                    label = "Ausstehend"
                    content = bind("""
                        governance.proposals
                            .filter(|p| p.state == "pending")
                            .map(|p| ProposalCard(p))
                    """)
                }

                tab "history" {
                    label = "Vergangene"
                    content = bind("""
                        governance.proposals
                            .filter(|p| p.state in ["executed", "defeated"])
                            .sort_by(|p| -p.voting_ends)
                            .take(50)
                    """)
                }
            }

            component create_proposal_button {
                type = button
                label = "Neues Proposal"
                trust_gate = 0.5

                on_click = "navigate('/governance/new')"
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // DATALOGIC: Event-Verarbeitung und Aggregation
    // ─────────────────────────────────────────────────────────────────────────

    datalogic {
        // Post-Verarbeitung
        handler "process_post" {
            trigger = event("post")

            action = """
                let post = event.payload

                // Spam-Check
                if is_spam(post.content) {
                    return reject("Spam detected")
                }

                // Trust des Autors aktualisieren (positive Aktion)
                author_trust = get_trust(post.author)
                author_trust.update(contribution = 0.001)

                // Benachrichtigungen
                mentions = extract_mentions(post.content)
                for mention in mentions {
                    notify(mention, "new_mention", post)
                }

                // Für UI-Binding
                emit("post_created", post)
            """
        }

        // Voting-Verarbeitung
        handler "process_vote" {
            trigger = event("governance:vote")

            action = """
                let vote = event.payload
                let proposal = get_proposal(vote.proposal_id)

                // Trust-gewichtete Stimmkraft berechnen
                let voter_trust = get_trust(vote.voter)
                let voting_power = voter_trust.weighted_norm([0.3, 0.2, 0.2, 0.1, 0.1, 0.1])

                // Vote speichern
                proposal.votes.add(vote.voter, {
                    choice: vote.choice,
                    power: voting_power,
                    timestamp: now()
                })

                // Live-Update für UI
                emit("proposal_vote_updated", {
                    proposal_id: vote.proposal_id,
                    total_votes: proposal.votes.count(),
                    for_votes: proposal.votes.sum_power("for"),
                    against_votes: proposal.votes.sum_power("against"),
                })
            """
        }

        // Aggregation für Analytics
        aggregation "daily_activity" {
            source = events.filter(|e| e.type in ["post", "reaction", "vote"])

            group_by = [date(event.timestamp), event.type]

            metrics = {
                count: count(),
                unique_users: count_distinct(event.author),
                avg_trust: avg(get_trust(event.author).norm()),
            }

            output = store("analytics:daily_activity")
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // API: REST-Schnittstelle für externe Systeme
    // ─────────────────────────────────────────────────────────────────────────

    api "public-api" {
        version = "1.0"
        base_path = "/api/v1"

        // Globale Einstellungen
        defaults {
            auth = api_key { scopes = ["read"] }
            rate_limit { requests_per_minute = 60 }
        }

        // Öffentliche Endpoints
        endpoint GET "/rooms" {
            description = "Liste aller öffentlichen Räume"
            auth = public

            response_schema = { type = "array", items = { $ref = "#/schemas/Room" } }

            handler = """
                rooms.filter(|r| r.visibility == "public")
                     .map(|r| r.to_public_view())
            """

            caching { ttl = 60 }
        }

        endpoint GET "/rooms/:room_id/posts" {
            description = "Posts in einem Raum"

            query_params = {
                limit = { type = "integer", default = 50, max = 100 }
                since = { type = "string", format = "date-time" }
            }

            handler = """
                let room = get_room(params.room_id)?

                events
                    .filter(|e| e.type == "post" && e.room == room.id)
                    .filter(|e| params.since ? e.timestamp > params.since : true)
                    .take(params.limit)
                    .map(|e| e.to_public_view())
            """
        }

        endpoint POST "/rooms/:room_id/posts" {
            description = "Neuen Post erstellen"
            auth = peer_auth { min_trust = 0.3 }

            request_schema = {
                type = "object"
                required = ["content"]
                properties = {
                    content = { type = "string", maxLength = 10000 }
                    reply_to = { type = "string", format = "uuid" }
                }
            }

            handler = """
                let room = get_room(params.room_id)?

                // Policy-Check
                room.policy.validate_post(auth.peer, request.content)?

                let post = Post::new(request.content)
                post.author = auth.peer
                post.room = room.id
                post.reply_to = request.reply_to

                events.emit("post", post)

                return { id: post.id, status: "created" }
            """
        }

        // Governance-Endpoints
        endpoint GET "/governance/proposals" {
            description = "Liste aller Proposals"

            query_params = {
                state = { type = "string", enum = ["active", "pending", "executed", "defeated"] }
            }

            handler = """
                governance.proposals
                    .filter(|p| params.state ? p.state == params.state : true)
                    .map(|p| p.to_public_view())
            """
        }

        endpoint POST "/governance/proposals/:proposal_id/vote" {
            description = "Für Proposal abstimmen"
            auth = peer_auth { min_trust = 0.2 }

            request_schema = {
                type = "object"
                required = ["choice"]
                properties = {
                    choice = { type = "string", enum = ["for", "against", "abstain"] }
                }
            }

            handler = """
                governance.cast_vote(
                    params.proposal_id,
                    auth.peer,
                    request.choice
                )
            """
        }

        // Webhook für externe Integrationen
        endpoint POST "/webhooks/incoming/:provider" {
            description = "Eingehende Webhooks von externen Services"
            auth = webhook_signature { algorithm = "hmac-sha256", header = "X-Signature" }

            handler = """
                match params.provider {
                    "github" => {
                        let payload = parse_github_webhook(request.body)
                        emit("external:github", payload)
                    }
                    "discord" => {
                        let payload = parse_discord_webhook(request.body)
                        emit("external:discord", payload)
                    }
                    _ => return error(400, "Unknown provider")
                }
            """
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // GOVERNANCE: DAO-Konfiguration
    // ─────────────────────────────────────────────────────────────────────────

    governance {
        mode = liquid_democracy {
            max_delegation_depth = 4
        }

        voting_rules {
            voting_period = "7d"
            min_quorum = 0.15
            approval_threshold = 0.66
            vote_options = yes_no_abstain
            vote_change_allowed = true
        }

        proposal_threshold {
            min_trust = 0.5
            or_credential = "council_member"
        }

        timelock_duration = "48h"

        // Verschiedene Regeln für verschiedene Aktionen
        action_overrides {
            ModifyUI {
                voting_period = "3d"
                min_quorum = 0.05
                timelock = "4h"
            }

            ModifyGovernance {
                voting_period = "14d"
                min_quorum = 0.25
                approval_threshold = 0.75
                timelock = "7d"
            }
        }

        // Council-Veto für kritische Änderungen
        veto {
            veto_holders = credential("council_member")
            veto_threshold = 3  // 3 Council-Mitglieder müssen zustimmen
            applies_to = ["ModifyGovernance", "EmergencyAction"]
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CONTROLLER: Verwaltung und Automatisierung
    // ─────────────────────────────────────────────────────────────────────────

    controller {
        // Kein einzelner Controller - alles via Governance
        governance_controlled = true

        // Automatisierungen
        automation {
            rule "weekly_trust_recalculation" {
                trigger = schedule { cron = "0 0 * * 0" }  // Sonntag 00:00
                action = """
                    for member in members {
                        member.trust.recalculate_from_activity(period = "7d")
                    }
                """
            }

            rule "inactive_member_warning" {
                trigger = schedule { cron = "0 0 1 * *" }  // 1. jeden Monats
                action = """
                    inactive = members.filter(|m| m.last_activity < now() - "90d")
                    for member in inactive {
                        notify(member, "inactivity_warning", {
                            days_inactive: (now() - member.last_activity).days()
                        })
                    }
                """
            }
        }

        // Audit
        audit {
            log_all = true
            retention = "2y"
            public_log = true  // Transparenz
        }
    }
}
```

### 3.0.9 Blueprint-Integration – Wiederverwendbare Templates als Kern

Blueprints sind **immutable, versionierte, content-adressierte Templates**, die alle ECL-Komponenten in einem wiederverwendbaren Paket bündeln. Sie bilden das Rückgrat des Erynoa-Ökosystems.

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                      ║
║   BLUEPRINT ARCHITEKTUR – TEMPLATE-BASIERTES ÖKOSYSTEM                                              ║
║                                                                                                      ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                                             │   ║
║   │   BLUEPRINT = Immutables Template (BLAKE3-Hash als ID)                                     │   ║
║   │                                                                                             │   ║
║   │   ┌─────────────────────────────────────────────────────────────────────────────────────┐ │   ║
║   │   │                                                                                     │ │   ║
║   │   │   ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────┐           │ │   ║
║   │   │   │ STRUCTURE │ │  POLICY   │ │    UI     │ │ DATALOGIC │ │    API    │           │ │   ║
║   │   │   │           │ │           │ │           │ │           │ │           │           │ │   ║
║   │   │   │ • Räume   │ │ • Gateway │ │ • Layouts │ │ • Handler │ │ • Endpts  │           │ │   ║
║   │   │   │ • Partit. │ │ • Access  │ │ • Pages   │ │ • Aggreg. │ │ • Schemas │           │ │   ║
║   │   │   │ • Stores  │ │ • Trust   │ │ • Comps   │ │ • Bindings│ │ • Auth    │           │ │   ║
║   │   │   └─────┬─────┘ └─────┬─────┘ └─────┬─────┘ └─────┬─────┘ └─────┬─────┘           │ │   ║
║   │   │         │             │             │             │             │                 │ │   ║
║   │   │         └─────────────┴─────────────┴─────────────┴─────────────┘                 │ │   ║
║   │   │                                     │                                             │ │   ║
║   │   │                         ┌───────────┴───────────┐                                 │ │   ║
║   │   │                         │                       │                                 │ │   ║
║   │   │                         ▼                       ▼                                 │ │   ║
║   │   │             ┌───────────────────┐   ┌───────────────────┐                         │ │   ║
║   │   │             │    GOVERNANCE     │   │    CONTROLLER     │                         │ │   ║
║   │   │             │                   │   │                   │                         │ │   ║
║   │   │             │ • Voting Rules    │   │ • Permissions     │                         │ │   ║
║   │   │             │ • Proposal Types  │   │ • Automation      │                         │ │   ║
║   │   │             │ • Quorum/Threshold│   │ • Delegation      │                         │ │   ║
║   │   │             └───────────────────┘   └───────────────────┘                         │ │   ║
║   │   │                                                                                     │ │   ║
║   │   │   + METADATEN: Name, Version, Creator, License, Category, Tags                     │ │   ║
║   │   │   + METRIKEN: Novelty-Score, Diversity-Contribution, Complexity                    │ │   ║
║   │   │                                                                                     │ │   ║
║   │   └─────────────────────────────────────────────────────────────────────────────────────┘ │   ║
║   │                                                                                             │   ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────┘   ║
║                                                                                                      ║
║   BLUEPRINT LIFECYCLE:                                                                              ║
║                                                                                                      ║
║   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐                     ║
║   │ CREATE   │───►│ VALIDATE │───►│ PUBLISH  │───►│ DEPLOY   │───►│ EVOLVE   │                     ║
║   │          │    │          │    │          │    │          │    │          │                     ║
║   │ ECL-Code │    │ Compile  │    │ Market-  │    │ Realm    │    │ Fork/    │                     ║
║   │ schreiben│    │ + Analyze│    │ place    │    │ Instance │    │ Version  │                     ║
║   └──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘                     ║
║                                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

#### 3.0.9.1 Extended Blueprint-Struktur mit ECL-Engines

```rust
// Konzeptionell: backend/src/local/blueprint_marketplace.rs (erweitert)

/// Ein Blueprint ist ein vollständiges, wiederverwendbares Template
/// das alle ECL-Engines miteinander verbindet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedBlueprint {
    // ─────────────────────────────────────────────────────────────────────────
    // Identifikation (immutable nach Erstellung)
    // ─────────────────────────────────────────────────────────────────────────
    /// BLAKE3-Hash als eindeutige ID
    pub id: BlueprintId,
    /// Semantische Version (SemVer)
    pub version: SemVer,
    /// Content-Hash für Integritätsprüfung
    pub content_hash: Hash,

    // ─────────────────────────────────────────────────────────────────────────
    // Metadaten
    // ─────────────────────────────────────────────────────────────────────────
    pub name: String,
    pub description: String,
    pub creator_did: UniversalId,
    pub created_at: TemporalCoord,
    pub tags: Vec<String>,
    pub category: BlueprintCategory,
    pub license: BlueprintLicense,

    // ─────────────────────────────────────────────────────────────────────────
    // ECL-Komponenten (alle Engine-Definitionen)
    // ─────────────────────────────────────────────────────────────────────────

    /// Struktur: Räume, Partitionen, Stores
    pub structure: BlueprintStructure,

    /// Policy: Gateway-Regeln, Zugriffskontrolle, Trust-Tiers
    pub policy: BlueprintPolicy,

    /// UI: Layouts, Pages, Components mit Trust-Gates
    pub ui: BlueprintUI,

    /// DataLogic: Event-Handler, Aggregationen, Bindings
    pub datalogic: BlueprintDataLogic,

    /// API: REST-Endpoints für externe Systeme
    pub api: BlueprintAPI,

    /// Governance: DAO-Modus, Voting-Rules, Proposals
    pub governance: BlueprintGovernance,

    /// Controller: Permissions, Delegation, Automation
    pub controller: BlueprintController,

    // ─────────────────────────────────────────────────────────────────────────
    // Versionierung & Abhängigkeiten
    // ─────────────────────────────────────────────────────────────────────────

    /// Vorgänger-Version
    pub predecessor: Option<BlueprintId>,
    /// Fork-Quelle
    pub forked_from: Option<BlueprintId>,
    /// Abhängigkeiten von anderen Blueprints
    pub dependencies: Vec<BlueprintDependency>,
    /// Inkompatibilitäten (Blueprints die nicht kombiniert werden können)
    pub incompatible_with: Vec<BlueprintId>,

    // ─────────────────────────────────────────────────────────────────────────
    // Automatisch berechnete Metriken (Κ19, Κ20)
    // ─────────────────────────────────────────────────────────────────────────

    /// Komplexitäts-Score
    pub complexity: u64,
    /// Novelty-Score (Surprisal vs. existierende Blueprints)
    pub novelty_score: f64,
    /// Diversity-Contribution (neue Konzepte für Ökosystem)
    pub diversity_contribution: f64,
    /// Ω-Beitrag zum Gesamtsystem
    pub omega_contribution: f64,
}

/// Blueprint-Abhängigkeit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintDependency {
    pub blueprint_id: BlueprintId,
    pub version_constraint: VersionConstraint,
    pub import_as: String,  // Namespace
    pub components: Vec<ImportedComponent>,
}

/// Welche Komponenten importiert werden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportedComponent {
    Structure { rooms: Vec<String> },
    Policy { policies: Vec<String> },
    UI { layouts: Vec<String>, pages: Vec<String> },
    DataLogic { handlers: Vec<String> },
    API { endpoints: Vec<String> },
    Governance { modes: Vec<String> },
    All,
}

/// Blueprint-Struktur-Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintStructure {
    /// Raum-Templates
    pub rooms: Vec<RoomTemplate>,
    /// Partition-Templates
    pub partitions: Vec<PartitionTemplate>,
    /// Store-Schemas
    pub stores: Vec<StoreSchema>,
    /// Dynamische Raum-Erstellung erlaubt?
    pub allow_dynamic_rooms: bool,
    /// Template für dynamische Räume
    pub dynamic_room_template: Option<String>,
}

/// Blueprint-UI-Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintUI {
    /// Theme-Variablen
    pub theme: HashMap<String, String>,
    /// Layout-Definitionen
    pub layouts: Vec<UILayout>,
    /// Page-Definitionen
    pub pages: Vec<UIPage>,
    /// Wiederverwendbare Komponenten
    pub components: Vec<UIComponent>,
    /// Trust-Gate-Defaults
    pub default_trust_gates: HashMap<String, f32>,
}

/// Blueprint-API-Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintAPI {
    /// API-Version
    pub version: String,
    /// Base-Path
    pub base_path: String,
    /// Endpoints
    pub endpoints: Vec<APIEndpoint>,
    /// Globale Auth-Einstellungen
    pub default_auth: EndpointAuth,
    /// Rate-Limit-Defaults
    pub default_rate_limit: RateLimitConfig,
    /// Schema-Definitionen
    pub schemas: HashMap<String, JSONSchema>,
}

/// Blueprint-Governance-Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintGovernance {
    /// Governance-Modus
    pub mode: GovernanceMode,
    /// Voting-Regeln
    pub voting_rules: VotingRules,
    /// Proposal-Templates
    pub proposal_templates: Vec<ProposalTemplate>,
    /// Action-spezifische Overrides
    pub action_overrides: HashMap<String, VotingRules>,
    /// Veto-Konfiguration
    pub veto_config: Option<VetoConfig>,
}

/// Blueprint-Controller-Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintController {
    /// Primary Controller (kann leer sein bei DAO)
    pub primary: Option<ControllerSpec>,
    /// Governance-gesteuert?
    pub governance_controlled: bool,
    /// Permissions-Template
    pub permissions: ControllerPermissions,
    /// Automation-Rules
    pub automation: Vec<AutomationRule>,
    /// Delegation-Settings
    pub delegation_config: DelegationConfig,
    /// Audit-Config
    pub audit_config: AuditConfig,
}
```

#### 3.0.9.2 Blueprint-Engine: Deployment & Instantiation

```rust
// Konzeptionell: backend/src/eclvm/blueprint_engine.rs

/// Blueprint-Engine koordiniert das Deployment von Blueprints
pub struct BlueprintEngine {
    /// Marketplace-Referenz
    marketplace: Arc<BlueprintMarketplace>,
    /// ECLVM für Code-Ausführung
    vm: ECLVM,
    /// Engine-Referenzen
    ui_engine: Arc<RwLock<UIEngine>>,
    api_engine: Arc<RwLock<APIEngine>>,
    governance_engine: Arc<RwLock<GovernanceEngine>>,
    controller_engine: Arc<RwLock<ControllerEngine>>,
    datalogic_engine: Arc<RwLock<DataLogicEngine>>,
}

impl BlueprintEngine {
    /// Deploye Blueprint in ein Realm
    pub async fn deploy(
        &mut self,
        blueprint_id: &BlueprintId,
        target_realm: &RealmId,
        deployer: &UniversalId,
        config: DeploymentConfig,
    ) -> Result<DeploymentResult> {
        // 1. Blueprint laden und validieren
        let blueprint = self.marketplace.get_blueprint(blueprint_id)?;
        self.validate_deployment_eligibility(&blueprint, target_realm, deployer)?;

        // 2. Abhängigkeiten auflösen und deployen
        for dep in &blueprint.dependencies {
            self.ensure_dependency_deployed(target_realm, dep).await?;
        }

        // 3. Mana-Kosten berechnen und reservieren
        let mana_cost = self.calculate_deployment_cost(&blueprint, &config)?;
        self.reserve_mana(deployer, mana_cost)?;

        // 4. Struktur deployen (Räume, Partitionen, Stores)
        let structure_result = self.deploy_structure(
            target_realm,
            &blueprint.structure,
            &config.structure_overrides,
        ).await?;

        // 5. Policies deployen
        let policy_result = self.deploy_policies(
            target_realm,
            &blueprint.policy,
            &config.policy_overrides,
        ).await?;

        // 6. UI deployen
        self.ui_engine.write().deploy_ui(
            target_realm,
            &blueprint.ui,
            &config.ui_overrides,
        )?;

        // 7. DataLogic deployen
        self.datalogic_engine.write().deploy_handlers(
            target_realm,
            &blueprint.datalogic,
        )?;

        // 8. API registrieren
        self.api_engine.write().register_blueprint_api(
            target_realm,
            &blueprint.api,
        )?;

        // 9. Governance deployen
        self.governance_engine.write().deploy_governance(
            target_realm,
            &blueprint.governance,
        )?;

        // 10. Controller konfigurieren
        self.controller_engine.write().deploy_controller(
            target_realm,
            &blueprint.controller,
            deployer,
        )?;

        // 11. Deployment-Event emittieren
        let deployment = Deployment {
            id: DeploymentId::new(),
            blueprint_id: blueprint_id.clone(),
            blueprint_version: blueprint.version.clone(),
            realm_id: target_realm.clone(),
            deployer: deployer.clone(),
            deployed_at: TemporalCoord::now(),
            config: config.clone(),
        };

        self.emit_deployment_event(&deployment)?;

        // 12. Trust-Boost für Blueprint-Creator
        self.apply_creator_trust_boost(&blueprint.creator_did)?;

        Ok(DeploymentResult {
            deployment,
            structure: structure_result,
            policy: policy_result,
            mana_consumed: mana_cost,
        })
    }

    /// Upgrade existierendes Deployment auf neue Blueprint-Version
    pub async fn upgrade(
        &mut self,
        deployment_id: &DeploymentId,
        new_blueprint_id: &BlueprintId,
        upgrader: &UniversalId,
    ) -> Result<UpgradeResult> {
        let deployment = self.get_deployment(deployment_id)?;
        let old_blueprint = self.marketplace.get_blueprint(&deployment.blueprint_id)?;
        let new_blueprint = self.marketplace.get_blueprint(new_blueprint_id)?;

        // 1. Versions-Kompatibilität prüfen
        self.check_version_compatibility(&old_blueprint, &new_blueprint)?;

        // 2. Prüfe ob Governance-Approval nötig
        if self.requires_governance_for_upgrade(&deployment, &new_blueprint) {
            return self.create_upgrade_proposal(
                deployment_id,
                new_blueprint_id,
                upgrader
            ).await;
        }

        // 3. Migration planen
        let migration_plan = self.plan_migration(&old_blueprint, &new_blueprint)?;

        // 4. Migration ausführen
        for step in migration_plan.steps {
            self.execute_migration_step(&deployment.realm_id, step).await?;
        }

        // 5. Deployment aktualisieren
        self.update_deployment(deployment_id, new_blueprint_id)?;

        Ok(UpgradeResult {
            old_version: old_blueprint.version,
            new_version: new_blueprint.version,
            migration_steps: migration_plan.steps.len(),
            breaking_changes: migration_plan.breaking_changes,
        })
    }

    /// Fork Blueprint mit Modifikationen
    pub async fn fork(
        &self,
        source_blueprint_id: &BlueprintId,
        modifications: BlueprintModifications,
        forker: &UniversalId,
    ) -> Result<BlueprintId> {
        let source = self.marketplace.get_blueprint(source_blueprint_id)?;

        // 1. Prüfe Fork-Berechtigung (Lizenz)
        self.validate_fork_license(&source.license)?;

        // 2. Wende Modifikationen an
        let mut forked = source.clone();
        forked.apply_modifications(modifications)?;

        // 3. Neue ID generieren
        forked.id = forked.compute_id();
        forked.forked_from = Some(source_blueprint_id.clone());
        forked.creator_did = forker.clone();
        forked.version = SemVer::initial();

        // 4. Novelty berechnen (relativ zum Original)
        forked.novelty_score = self.calculate_fork_novelty(&source, &forked)?;

        // 5. Im Marketplace publizieren
        self.marketplace.publish(forked.clone(), forker)?;

        // 6. Credit-Chain zum Original
        self.create_attribution_link(source_blueprint_id, &forked.id)?;

        Ok(forked.id)
    }
}
```

#### 3.0.9.3 Blueprint Marketplace Integration

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                      ║
║   BLUEPRINT MARKETPLACE – DEZENTRALES TEMPLATE-ÖKOSYSTEM                                            ║
║                                                                                                      ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                                             │   ║
║   │   MARKETPLACE als dedizierter Realm (shared:blueprints)                                    │   ║
║   │                                                                                             │   ║
║   │   ┌───────────────┐    ┌───────────────┐    ┌───────────────┐    ┌───────────────┐        │   ║
║   │   │ DISCOVERY     │    │ RATINGS       │    │ DEPLOYMENTS   │    │ ECONOMICS     │        │   ║
║   │   │               │    │               │    │               │    │               │        │   ║
║   │   │ • Search      │    │ • Attestation │    │ • Statistics  │    │ • License Fees│        │   ║
║   │   │ • Categories  │    │ • Trust-based │    │ • Usage       │    │ • Creator Rev │        │   ║
║   │   │ • Tags        │    │ • Bayesian    │    │ • Forks       │    │ • Mana Costs  │        │   ║
║   │   │ • Semantic    │    │   Updates     │    │ • Versions    │    │               │        │   ║
║   │   └───────┬───────┘    └───────┬───────┘    └───────┬───────┘    └───────┬───────┘        │   ║
║   │           │                    │                    │                    │                │   ║
║   │           └────────────────────┴────────────────────┴────────────────────┘                │   ║
║   │                                          │                                                │   ║
║   │                                          ▼                                                │   ║
║   │                        ┌─────────────────────────────────────┐                            │   ║
║   │                        │     RANKING ALGORITHM (Κ19/Κ20)    │                            │   ║
║   │                        │                                     │                            │   ║
║   │                        │  Score = Deployments × Ratings ×    │                            │   ║
║   │                        │          Diversity × (1 - Calcif.)  │                            │   ║
║   │                        │                                     │                            │   ║
║   │                        │  Novelty = Surprisal vs. existing   │                            │   ║
║   │                        │  Trust-Wt = Ω-alignment of raters   │                            │   ║
║   │                        └─────────────────────────────────────┘                            │   ║
║   │                                                                                             │   ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────┘   ║
║                                                                                                      ║
║   GAMING-RESISTENZ (Axiome Κ19/Κ20):                                                                ║
║                                                                                                      ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                                             │   ║
║   │   Upload Requirements:                                                                      │   ║
║   │   • Trust-R > 0.8 (hohe Reputation)                                                        │   ║
║   │   • Ω > 1.5 (System-Alignment)                                                             │   ║
║   │   • Novelty > 3.0 (nicht-trivial anders)                                                   │   ║
║   │                                                                                             │   ║
║   │   Anti-Manipulation:                                                                        │   ║
║   │   • Ratings: Ω-gewichtet, Anomaly-Detection                                                │   ║
║   │   • Power-Cap: Kein Creator dominiert Listings                                             │   ║
║   │   • Diversity-Boost: Innovative Blueprints steigen auf                                     │   ║
║   │   • Sybil-Resistenz: Trust-Decay bei Verdacht                                              │   ║
║   │                                                                                             │   ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────┘   ║
║                                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

#### 3.0.9.4 Blueprint-Komposition: Mehrere Blueprints kombinieren

```rust
// Konzeptionell: backend/src/eclvm/blueprint_composer.rs

/// Blueprint-Composer für das Kombinieren mehrerer Blueprints
pub struct BlueprintComposer {
    marketplace: Arc<BlueprintMarketplace>,
    conflict_resolver: ConflictResolver,
}

/// Kompositions-Manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionManifest {
    /// Name der Komposition
    pub name: String,
    /// Basis-Blueprint
    pub base: BlueprintRef,
    /// Erweiterungen (in Reihenfolge)
    pub extensions: Vec<BlueprintExtension>,
    /// Konflikt-Auflösungen
    pub conflict_resolutions: HashMap<String, ConflictResolution>,
    /// Overrides
    pub overrides: BlueprintModifications,
}

/// Blueprint-Referenz
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintRef {
    pub id: BlueprintId,
    pub version: Option<VersionConstraint>,
}

/// Erweiterung mit selektiven Imports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintExtension {
    pub blueprint: BlueprintRef,
    pub imports: Vec<ImportedComponent>,
    pub namespace: Option<String>,
    pub priority: u32,  // Höher = überschreibt bei Konflikten
}

/// Konflikt-Auflösung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Wähle spezifische Version
    UseBlueprint(BlueprintId),
    /// Merge (nur für kompatible Typen)
    Merge,
    /// Beides behalten mit Namespace
    Namespace { a: String, b: String },
    /// Custom ECL-Code
    Custom { ecl_code: String },
}

impl BlueprintComposer {
    /// Komponiere mehrere Blueprints zu einem neuen
    pub fn compose(&self, manifest: CompositionManifest) -> Result<ExtendedBlueprint> {
        // 1. Basis-Blueprint laden
        let mut result = self.marketplace.get_blueprint(&manifest.base.id)?;

        // 2. Erweiterungen in Reihenfolge anwenden
        for extension in manifest.extensions.iter().sorted_by_key(|e| e.priority) {
            let ext_blueprint = self.marketplace.get_blueprint(&extension.blueprint.id)?;

            // Selektive Imports anwenden
            for import in &extension.imports {
                self.apply_import(&mut result, &ext_blueprint, import, &extension.namespace)?;
            }
        }

        // 3. Konflikte auflösen
        for (conflict_key, resolution) in &manifest.conflict_resolutions {
            self.resolve_conflict(&mut result, conflict_key, resolution)?;
        }

        // 4. Overrides anwenden
        result.apply_modifications(manifest.overrides)?;

        // 5. Neue ID generieren
        result.id = result.compute_id();

        // 6. Abhängigkeiten aktualisieren
        result.dependencies = self.collect_dependencies(&manifest)?;

        Ok(result)
    }

    /// Wende Import an
    fn apply_import(
        &self,
        target: &mut ExtendedBlueprint,
        source: &ExtendedBlueprint,
        import: &ImportedComponent,
        namespace: &Option<String>,
    ) -> Result<()> {
        match import {
            ImportedComponent::Structure { rooms } => {
                for room_name in rooms {
                    if let Some(room) = source.structure.rooms.iter()
                        .find(|r| &r.name == room_name)
                    {
                        let mut imported = room.clone();
                        if let Some(ns) = namespace {
                            imported.name = format!("{}:{}", ns, imported.name);
                        }
                        target.structure.rooms.push(imported);
                    }
                }
            }

            ImportedComponent::UI { layouts, pages } => {
                for layout_name in layouts {
                    if let Some(layout) = source.ui.layouts.iter()
                        .find(|l| &l.name == layout_name)
                    {
                        target.ui.layouts.push(layout.clone());
                    }
                }
                // ... pages analog
            }

            ImportedComponent::API { endpoints } => {
                for endpoint_path in endpoints {
                    if let Some(endpoint) = source.api.endpoints.iter()
                        .find(|e| &e.path == endpoint_path)
                    {
                        let mut imported = endpoint.clone();
                        if let Some(ns) = namespace {
                            imported.path = format!("/{}{}", ns, imported.path);
                        }
                        target.api.endpoints.push(imported);
                    }
                }
            }

            ImportedComponent::All => {
                // Alle Komponenten importieren (mit Namespace-Prefix)
                self.apply_import(target, source, &ImportedComponent::Structure {
                    rooms: source.structure.rooms.iter().map(|r| r.name.clone()).collect()
                }, namespace)?;
                // ... andere Komponenten analog
            }

            _ => {}
        }

        Ok(())
    }
}
```

**ECL-Syntax für Blueprint-Komposition:**

```ecl
// Blueprint-Komposition: Community + Marketplace + Governance
compose "advanced-community-marketplace" {
    version = "1.0"

    // Basis: Community-Platform
    base = blueprint("community-platform:1.0")

    // Erweiterung 1: Marketplace-Funktionalität
    extend blueprint("marketplace-core:2.1") {
        namespace = "market"
        priority = 10

        import {
            structure = ["listings", "orders", "reviews"]
            ui = ["product_page", "checkout_flow"]
            api = ["/products/*", "/orders/*"]
            datalogic = ["order_processing", "review_aggregation"]
        }
    }

    // Erweiterung 2: Erweiterte Governance
    extend blueprint("advanced-governance:1.5") {
        namespace = "gov"
        priority = 20

        import {
            governance = ["conviction_voting", "futarchy"]
            ui = ["advanced_proposal_view", "delegation_graph"]
        }
    }

    // Erweiterung 3: Analytics
    extend blueprint("analytics-dashboard:1.0") {
        namespace = "analytics"
        priority = 5

        import {
            ui = ["dashboard", "charts"]
            datalogic = ["metrics_aggregation"]
            api = ["/analytics/*"]
        }
    }

    // Konflikt-Auflösungen
    conflicts {
        // Beide haben "main" Layout - merge
        "ui:layout:main" = merge

        // Unterschiedliche Governance-Modi
        "governance:mode" = use_blueprint("advanced-governance:1.5")

        // API-Pfad-Konflikt
        "api:/users" = namespace {
            community = "/community/users"
            market = "/market/users"
        }
    }

    // Overrides nach Komposition
    overrides {
        // Angepasstes Theme
        ui.theme.primary_color = "#FF6B35"

        // Kombinierte Trust-Gates
        policy.trust_tiers {
            vendor { min = 0.4, max = 0.7, can_list_products = true }
        }

        // Marketplace-spezifische Governance
        governance.action_overrides {
            "market:CreateListing" {
                voting_period = "0"  // Keine Abstimmung nötig
                auto_approve = true
            }
            "market:DisputeResolution" {
                voting_period = "3d"
                min_quorum = 0.1
                jury_selection = random(5)
            }
        }
    }
}
```

#### 3.0.9.5 Blueprint-Templates: Vordefinierte Muster

```ecl
// ═══════════════════════════════════════════════════════════════════════════════
// BLUEPRINT-TEMPLATE-BIBLIOTHEK
// ═══════════════════════════════════════════════════════════════════════════════

// Template 1: Basis-Social-App
template "social-base" {
    category = social
    license = open

    structure {
        room "feed" {
            partition "posts" { write = trust_min(0.2) }
            partition "comments" { write = trust_min(0.1) }
        }
        room "profiles" {
            partition "public" { read = all }
            partition "private" { read = owner_only }
        }
    }

    ui {
        layout "feed-layout" { /* ... */ }
        page "feed-page" { /* ... */ }
        component "post-card" { /* ... */ }
    }

    datalogic {
        handler "on_post" { /* ... */ }
        aggregation "trending" { /* ... */ }
    }

    api {
        endpoint GET "/feed" { /* ... */ }
        endpoint POST "/posts" { /* ... */ }
    }

    governance {
        mode = dao { voting_power = trust_based }
    }
}

// Template 2: DAO-Treasury
template "dao-treasury" {
    category = governance
    license = open

    structure {
        room "treasury" {
            partition "proposals" { write = trust_min(0.5) }
            partition "allocations" { write = governance_only }
        }
    }

    governance {
        mode = conviction_voting {
            decay_rate = 0.9
            max_conviction = 10.0
        }

        voting_rules {
            voting_period = "14d"
            min_quorum = 0.20
        }
    }

    datalogic {
        handler "on_allocation_approved" {
            trigger = event("governance:proposal:executed")
            condition = "event.proposal.type == 'allocation'"
            action = """
                let allocation = event.proposal.allocation
                treasury.allocate(allocation.recipient, allocation.amount)
                emit("treasury:allocated", allocation)
            """
        }
    }
}

// Template 3: API-Gateway
template "api-gateway" {
    category = infrastructure
    license = open

    api {
        version = "1.0"

        // Health-Check
        endpoint GET "/health" {
            auth = public
            handler = "return { status: 'ok', timestamp: now() }"
        }

        // API-Key-Management
        endpoint POST "/keys" {
            auth = peer_auth { min_trust = 0.7 }
            handler = """
                let key = generate_api_key(auth.peer)
                store("api_keys", key.id, key)
                return { key: key.value, expires: key.expires_at }
            """
        }

        // Rate-Limit-Status
        endpoint GET "/rate-limit" {
            auth = api_key
            handler = """
                let limits = get_rate_limits(auth.client_id)
                return limits
            """
        }
    }

    datalogic {
        handler "track_api_usage" {
            trigger = event("api:request")
            action = """
                metrics.increment("api:requests", {
                    endpoint: event.path,
                    method: event.method,
                    client: event.client_id
                })
            """
        }
    }
}

// Template 4: Multi-Tenant-Base
template "multi-tenant" {
    category = infrastructure
    license = commercial { mana_fee = 100 }

    structure {
        room "tenants" {
            dynamic = true

            template "tenant-room" {
                partitions = ["data", "config", "users"]

                policy {
                    isolation = strict  // Kein Cross-Tenant-Zugriff
                    inherit_from_parent = false
                }
            }
        }

        room "admin" {
            partition "tenant-management" {
                write = credential("admin")
            }
        }
    }

    controller {
        // Tenant-Controller-Delegation
        delegation {
            enabled = true
            per_tenant = true  // Jeder Tenant hat eigenen Controller

            delegatable_permissions = [
                "manage_users",
                "configure_ui",
                "view_analytics"
            ]

            non_delegatable = [
                "delete_tenant",
                "access_other_tenants"
            ]
        }
    }

    api {
        endpoint POST "/tenants" {
            auth = peer_auth { credential = "admin" }
            handler = """
                let tenant = create_tenant(request.name, request.config)
                let room = create_room_from_template("tenant-room", tenant.id)
                delegate_control(room.id, request.tenant_admin)
                return tenant
            """
        }

        endpoint GET "/tenants/:tenant_id/*" {
            auth = tenant_scoped  // Automatische Tenant-Isolation
        }
    }
}
```

#### 3.0.9.6 Blueprint ↔ Engine Verbindungsmatrix

```
╔══════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                      ║
║   BLUEPRINT ↔ ENGINE VERBINDUNGSMATRIX                                                              ║
║                                                                                                      ║
║   ┌─────────────────┬────────────┬────────────┬────────────┬────────────┬────────────┬────────────┐ ║
║   │ Blueprint-Komp. │ UI-Engine  │ API-Engine │ DataLogic  │ Governance │ Controller │ Policy-Eng │ ║
║   ├─────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────┤ ║
║   │                 │            │            │            │            │            │            │ ║
║   │ structure       │ render()   │ routes     │ stores     │     -      │ scopes     │ gateways   │ ║
║   │ (Räume/Parts)   │ navigation │ params     │ partitions │            │            │            │ ║
║   │                 │            │            │            │            │            │            │ ║
║   ├─────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────┤ ║
║   │                 │            │            │            │            │            │            │ ║
║   │ policy          │ trust_gate │ auth       │ access     │ proposal   │ permission │ validate() │ ║
║   │ (Regeln)        │ cred_gate  │ rate_limit │ filter     │ threshold  │ delegation │            │ ║
║   │                 │            │            │            │            │            │            │ ║
║   ├─────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────┤ ║
║   │                 │            │            │            │            │            │            │ ║
║   │ ui              │ layouts    │ openapi    │ bindings   │ proposal   │ admin      │     -      │ ║
║   │ (Interface)     │ pages      │ docs       │ reactive   │ UI         │ panels     │            │ ║
║   │                 │ components │            │            │            │            │            │ ║
║   │                 │            │            │            │            │            │            │ ║
║   ├─────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────┤ ║
║   │                 │            │            │            │            │            │            │ ║
║   │ datalogic       │ bind()     │ handler    │ handlers   │ vote       │ automation │ event      │ ║
║   │ (Events)        │ updates    │ transform  │ aggreg.    │ counting   │ triggers   │ emission   │ ║
║   │                 │            │            │            │            │            │            │ ║
║   ├─────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────┤ ║
║   │                 │            │            │            │            │            │            │ ║
║   │ api             │     -      │ endpoints  │ webhooks   │     -      │ key mgmt   │ auth       │ ║
║   │ (REST)          │            │ schemas    │ events     │            │            │ validate   │ ║
║   │                 │            │            │            │            │            │            │ ║
║   ├─────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────┤ ║
║   │                 │            │            │            │            │            │            │ ║
║   │ governance      │ voting UI  │ proposal   │ vote       │ modes      │ overrides  │ action     │ ║
║   │ (DAO)           │ delegation │ endpoints  │ tallying   │ rules      │ veto       │ threshold  │ ║
║   │                 │            │            │            │            │            │            │ ║
║   ├─────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────┤ ║
║   │                 │            │            │            │            │            │            │ ║
║   │ controller      │ admin UI   │ mgmt API   │ audit      │ governed   │ primary    │ delegate   │ ║
║   │ (Verwaltung)    │            │            │ logging    │ actions    │ delegation │ validate   │ ║
║   │                 │            │            │            │            │            │            │ ║
║   └─────────────────┴────────────┴────────────┴────────────┴────────────┴────────────┴────────────┘ ║
║                                                                                                      ║
║   DATENFLUSS BEI BLUEPRINT-DEPLOYMENT:                                                              ║
║                                                                                                      ║
║   Blueprint ──┬──► StructureEngine: Räume/Partitionen erstellen                                     ║
║               │                                                                                      ║
║               ├──► PolicyEngine: Gateway-Policies registrieren                                      ║
║               │                                                                                      ║
║               ├──► UIEngine: Layouts/Pages/Components kompilieren                                   ║
║               │                                                                                      ║
║               ├──► DataLogicEngine: Handler/Aggregations aktivieren                                 ║
║               │                                                                                      ║
║               ├──► APIEngine: Endpoints registrieren, OpenAPI generieren                            ║
║               │                                                                                      ║
║               ├──► GovernanceEngine: Voting-Rules konfigurieren                                     ║
║               │                                                                                      ║
║               └──► ControllerEngine: Permissions/Automation setzen                                  ║
║                                                                                                      ║
╚══════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

### 3.0.10 ECLVM Layer Interne Verbindungen

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

_Weiter zu [04-STATE-MANAGEMENT.md](04-STATE-MANAGEMENT.md) für das interne Zustandsmanagement._
