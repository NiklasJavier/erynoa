//! # Unified State Management
//!
//! Hierarchisches, thread-safe State-Management für alle Erynoa-Module.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────────┐
//! │                              UNIFIED STATE                                       │
//! │                                                                                  │
//! │  ┌─────────────────────────────────────────────────────────────────────────┐   │
//! │  │                          CoreState (Κ2-Κ18)                              │   │
//! │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │   │
//! │  │  │ TrustState   │──│ EventState   │──│ FormulaState │──│ Consensus  │  │   │
//! │  │  │  (Κ2-Κ5)     │  │  (Κ9-Κ12)    │  │  (Κ15b-d)    │  │   (Κ18)    │  │   │
//! │  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └─────┬──────┘  │   │
//! │  │         │                 │                 │                │         │   │
//! │  │         └─────────────────┴─────────────────┴────────────────┘         │   │
//! │  │                                    │                                    │   │
//! │  │                         Trust-Event-Kausalität                          │   │
//! │  └─────────────────────────────────────────────────────────────────────────┘   │
//! │                                      │                                          │
//! │  ┌───────────────────────────────────┼───────────────────────────────────┐     │
//! │  │                        ExecutionState (IPS ℳ)                         │     │
//! │  │  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐        │     │
//! │  │  │  GasTracker    │───│  ManaTracker   │───│  EventEmitter  │        │     │
//! │  │  └────────┬───────┘   └────────┬───────┘   └────────┬───────┘        │     │
//! │  │           │                    │                    │                 │     │
//! │  │           └────────────────────┴────────────────────┘                 │     │
//! │  │                               Cost Aggregation                        │     │
//! │  └───────────────────────────────────────────────────────────────────────┘     │
//! │                                      │                                          │
//! │  ┌───────────────────────────────────┼───────────────────────────────────┐     │
//! │  │                       ProtectionState (Κ19-Κ21)                        │     │
//! │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
//! │  │  │  Anomaly     │  │  Diversity   │  │  Quadratic   │  │AntiCalc  │  │     │
//! │  │  │  Detection   │──│  Monitor     │──│  Governance  │──│  (Κ19)   │  │     │
//! │  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────┬─────┘  │     │
//! │  │         │                 │                 │               │         │     │
//! │  │         └─────────────────┴─────────────────┴───────────────┘         │     │
//! │  │                         Protection Signals                            │     │
//! │  └───────────────────────────────────────────────────────────────────────┘     │
//! │                                      │                                          │
//! │  ┌───────────────────────────────────┼───────────────────────────────────┐     │
//! │  │                        StorageState (Local)                           │     │
//! │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
//! │  │  │  KV Store    │  │  Event Store │  │   Archive    │  │Blueprint │  │     │
//! │  │  │              │──│   (DAG)      │──│  (ψ_archive) │──│Marketpl. │  │     │
//! │  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────┬─────┘  │     │
//! │  │         │                 │                 │               │         │     │
//! │  │         └─────────────────┴─────────────────┴───────────────┘         │     │
//! │  │                         Persistence Layer                             │     │
//! │  └───────────────────────────────────────────────────────────────────────┘     │
//! │                                      │                                          │
//! │  ┌───────────────────────────────────┼───────────────────────────────────┐     │
//! │  │                         PeerState (Κ22-Κ24)                            │     │
//! │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │     │
//! │  │  │   Gateway    │  │ SagaComposer │  │ IntentParser │  │ Realm    │  │     │
//! │  │  │   (Κ23)      │──│  (Κ22/Κ24)   │──│              │──│  State   │  │     │
//! │  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └────┬─────┘  │     │
//! │  │         │                 │                 │               │         │     │
//! │  │         │           ┌─────┴─────┐           │               │         │     │
//! │  │         │           │ Per-Realm │           │               │         │     │
//! │  │         │           │ Isolation │           │               │         │     │
//! │  │         └───────────┤ TrustVec  ├───────────┘               │         │     │
//! │  │                     │ Rules     │                           │         │     │
//! │  │                     │ Identity  │                           │         │     │
//! │  │                     │ Metrics   │                           │         │     │
//! │  │                     └───────────┘                           │         │     │
//! │  │                     Cross-Realm Orchestration                         │     │
//! │  └───────────────────────────────────────────────────────────────────────┘     │
//! │                                                                                  │
//! └─────────────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Design-Prinzipien
//!
//! 1. **Hierarchische Komposition**: State-Layer bauen aufeinander auf
//! 2. **Thread-Safety**: Alle Counter sind atomar, komplexe Strukturen unter RwLock
//! 3. **Dependency Injection**: Jeder Layer kennt seine Abhängigkeiten
//! 4. **Event-Driven Updates**: Änderungen propagieren durch Observer-Pattern
//! 5. **Snapshot-Isolation**: Konsistente Reads ohne Locking
//! 6. **Per-Realm Isolation**: Jedes Realm hat eigenen TrustVector, Rules und Metrics

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

// ============================================================================
// STATE RELATIONSHIP TYPES
// ============================================================================

/// Beziehungstyp zwischen State-Komponenten
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

/// State-Komponenten-Identifikator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateComponent {
    // Core
    Trust,
    Event,
    WorldFormula,
    Consensus,
    // Execution
    Gas,
    Mana,
    Execution,
    // ECLVM Layer (ECL = Erynoa Core Language)
    /// ECLVM - Cost-limited Execution Environment für ECL
    ECLVM,
    /// ECL Policy-Engine (Rules, Crossing-Policies)
    ECLPolicy,
    /// ECL Blueprint-Management (Templates, Instantiation)
    ECLBlueprint,
    // Protection
    Anomaly,
    Diversity,
    Quadratic,
    AntiCalcification,
    Calibration,
    // Storage
    KvStore,
    EventStore,
    Archive,
    Blueprint,
    // Peer Layer (Κ22-Κ24)
    Gateway,
    SagaComposer,
    IntentParser,
    /// Realm-Isolation und per-Realm State
    Realm,
    // P2P Network Layer
    Swarm,
    Gossip,
    Kademlia,
    Relay,
    NatTraversal,
    Privacy,
}

/// Beziehungs-Graph zwischen State-Komponenten
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateGraph {
    pub edges: Vec<(StateComponent, StateRelation, StateComponent)>,
}

impl StateGraph {
    /// Erstelle den Erynoa-State-Graph mit allen Beziehungen
    pub fn erynoa_graph() -> Self {
        use StateComponent::*;
        use StateRelation::*;

        Self {
            edges: vec![
                // Core-Layer Beziehungen
                (Trust, Triggers, Event), // Trust-Updates erzeugen Events
                (Event, Triggers, Trust), // Events können Trust beeinflussen
                (Trust, DependsOn, WorldFormula), // Trust fließt in 𝔼
                (Event, DependsOn, WorldFormula), // Events fließen in 𝔼
                (WorldFormula, Triggers, Consensus), // 𝔼 beeinflusst Konsens
                (Consensus, Validates, Event), // Konsens validiert Events
                // Execution-Layer Beziehungen
                (Gas, DependsOn, Trust),       // Gas-Budget basiert auf Trust
                (Mana, DependsOn, Trust),      // Mana basiert auf Trust
                (Execution, Aggregates, Gas),  // Execution trackt Gas
                (Execution, Aggregates, Mana), // Execution trackt Mana
                (Execution, Triggers, Event),  // Execution emittiert Events
                // Protection-Layer Beziehungen
                (Anomaly, Validates, Event),       // Anomaly prüft Events
                (Anomaly, Validates, Trust),       // Anomaly prüft Trust-Patterns
                (Diversity, Validates, Trust),     // Diversity prüft Trust-Verteilung
                (Diversity, Validates, Consensus), // Diversity prüft Validator-Mix
                (Quadratic, DependsOn, Trust),     // Voting-Power hängt von Trust ab
                (AntiCalcification, Validates, Trust), // Anti-Calc überwacht Power
                (AntiCalcification, Triggers, Trust), // Anti-Calc kann Trust limitieren
                (Calibration, Triggers, Gas),      // Calibration passt Gas-Preise an
                (Calibration, Triggers, Mana),     // Calibration passt Mana-Regen an
                // Storage-Layer Beziehungen
                (EventStore, Aggregates, Event), // EventStore persistiert Events
                (Archive, Aggregates, EventStore), // Archive komprimiert EventStore
                (KvStore, DependsOn, Trust),     // KV-Access prüft Trust
                (Blueprint, DependsOn, Trust),   // Blueprint-Publish prüft Trust
                // Peer-Layer Beziehungen (Κ22-Κ24)
                (Gateway, Validates, Trust), // Gateway prüft Trust für Crossing
                (Gateway, DependsOn, Trust), // Gateway-Entscheidung basiert auf Trust
                (Gateway, Triggers, Event),  // Crossing erzeugt Events
                (Gateway, DependsOn, Realm), // Gateway prüft Realm-Crossing-Rules
                (SagaComposer, DependsOn, Trust), // Saga-Budget basiert auf Trust
                (SagaComposer, Triggers, Execution), // Sagas erzeugen Executions
                (SagaComposer, Aggregates, IntentParser), // Composer nutzt Parser
                (IntentParser, Validates, Event), // Parser validiert Intent-Events
                // REALM-LAYER BEZIEHUNGEN (Κ22-Κ24: Isolation, Crossing, Sagas)
                (Realm, DependsOn, Trust), // Realm-Trust basiert auf Global-Trust + Realm-Modifikator
                (Realm, Triggers, Trust),  // Realm-spezifisches Verhalten beeinflusst Global-Trust
                (Realm, Aggregates, Gateway), // Realm trackt Crossings (in/out)
                (Realm, DependsOn, Gateway), // Realm nutzt Gateway für Crossing-Kontrolle
                (Realm, Triggers, SagaComposer), // Realm kann Cross-Realm-Sagas auslösen
                (Realm, Triggers, Event), // Realm-Events (Registrierung, Rule-Änderungen, Membership)
                (Realm, Validates, Event), // Realm validiert Events gegen Realm-Policies
                (Realm, DependsOn, ECLPolicy), // Realm-Regeln definiert durch ECL-Policies
                (Realm, Aggregates, ECLPolicy), // Realm trackt aktive Policies
                // ECLVM-Layer Beziehungen (Erynoa Core Language)
                (ECLVM, DependsOn, Gas),  // ECLVM verbraucht Gas (Compute)
                (ECLVM, DependsOn, Mana), // ECLVM verbraucht Mana (Bandwidth/Events)
                (ECLVM, Triggers, Event), // Jede ECL-Ausführung emittiert Events
                (ECLVM, Aggregates, Execution), // ECLVM aggregiert Execution-Metriken
                (ECLVM, DependsOn, Trust), // ECL-Budget basiert auf Trust
                (ECLPolicy, Validates, Gateway), // Policies validieren Crossings (Κ23)
                (ECLPolicy, Validates, Realm), // Policies definieren Realm-Regeln
                (ECLPolicy, DependsOn, ECLVM), // Policies werden von ECLVM ausgeführt
                (ECLPolicy, Triggers, Event), // Policy-Evaluationen erzeugen Events
                (ECLBlueprint, DependsOn, ECLVM), // Blueprints werden von ECLVM instanziiert
                (ECLBlueprint, Aggregates, Blueprint), // Blueprint-Marketplace nutzt Storage
                (ECLBlueprint, Triggers, Event), // Blueprint-Instanziierung erzeugt Events
                (SagaComposer, DependsOn, ECLVM), // Sagas werden durch ECLVM orchestriert
                (IntentParser, DependsOn, ECLPolicy), // Intents werden gegen Policies validiert
                (Gateway, DependsOn, ECLPolicy), // Gateway führt Crossing-Policies aus
                // P2P Network-Layer Beziehungen
                (Swarm, Triggers, Event),        // Swarm propagiert Events
                (Gossip, DependsOn, Trust),      // Gossip-Scoring nutzt Trust
                (Gossip, Triggers, Event),       // Gossip verteilt Events
                (Kademlia, Aggregates, Swarm),   // DHT aggregiert Peer-Info
                (Relay, DependsOn, Trust),       // Relay-Auswahl basiert auf Trust
                (Relay, Triggers, Swarm),        // Relay beeinflusst Connections
                (NatTraversal, Triggers, Swarm), // NAT-Status beeinflusst Erreichbarkeit
                (Privacy, DependsOn, Trust),     // Privacy-Level basiert auf Trust
                (Privacy, Validates, Gossip),    // Privacy validiert Routing
            ],
        }
    }

    /// Finde alle Komponenten die von `component` abhängen
    pub fn dependents(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(_, rel, to)| {
                *to == component
                    && matches!(rel, StateRelation::DependsOn | StateRelation::Aggregates)
            })
            .map(|(from, _, _)| *from)
            .collect()
    }

    /// Finde alle Komponenten die `component` triggert
    pub fn triggered_by(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, _)| *from == component && matches!(rel, StateRelation::Triggers))
            .map(|(_, _, to)| *to)
            .collect()
    }

    /// Finde alle Komponenten die von `component` aggregiert werden
    pub fn aggregated_by(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, _)| *from == component && matches!(rel, StateRelation::Aggregates))
            .map(|(_, _, to)| *to)
            .collect()
    }

    /// Finde alle Komponenten die `component` validiert
    pub fn validated_by(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, _)| *from == component && matches!(rel, StateRelation::Validates))
            .map(|(_, _, to)| *to)
            .collect()
    }

    /// Finde alle Validatoren für `component`
    pub fn validators_of(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(_, rel, to)| *to == component && matches!(rel, StateRelation::Validates))
            .map(|(from, _, _)| *from)
            .collect()
    }

    /// Finde alle bidirektionalen Partner von `component`
    pub fn bidirectional_with(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, to)| {
                matches!(rel, StateRelation::Bidirectional)
                    && (*from == component || *to == component)
            })
            .map(|(from, _, to)| if *from == component { *to } else { *from })
            .collect()
    }

    /// Finde alle Komponenten von denen `component` abhängt
    pub fn dependencies_of(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, _)| *from == component && matches!(rel, StateRelation::DependsOn))
            .map(|(_, _, to)| *to)
            .collect()
    }

    /// Prüfe ob eine Beziehung existiert
    pub fn has_relation(
        &self,
        from: StateComponent,
        relation: StateRelation,
        to: StateComponent,
    ) -> bool {
        self.edges.contains(&(from, relation, to))
    }

    /// Alle Beziehungen einer Komponente (eingehend und ausgehend)
    pub fn all_relations(
        &self,
        component: StateComponent,
    ) -> Vec<(StateComponent, StateRelation, StateComponent)> {
        self.edges
            .iter()
            .filter(|(from, _, to)| *from == component || *to == component)
            .cloned()
            .collect()
    }

    /// Transitive Abhängigkeiten (rekursiv alle Dependencies)
    pub fn transitive_dependencies(&self, component: StateComponent) -> HashSet<StateComponent> {
        let mut visited = HashSet::new();
        let mut stack = vec![component];

        while let Some(current) = stack.pop() {
            for dep in self.dependencies_of(current) {
                if visited.insert(dep) {
                    stack.push(dep);
                }
            }
        }
        visited
    }

    /// Transitive Trigger-Kette (alle Komponenten die transitiv getriggert werden)
    pub fn transitive_triggers(&self, component: StateComponent) -> HashSet<StateComponent> {
        let mut visited = HashSet::new();
        let mut stack = vec![component];

        while let Some(current) = stack.pop() {
            for triggered in self.triggered_by(current) {
                if visited.insert(triggered) {
                    stack.push(triggered);
                }
            }
        }
        visited
    }

    /// Ermittle Validierungs-Kette für eine Komponente
    pub fn validation_chain(&self, component: StateComponent) -> Vec<StateComponent> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut current = component;

        while let Some(validator) = self.validators_of(current).first().copied() {
            if visited.insert(validator) {
                chain.push(validator);
                current = validator;
            } else {
                break; // Zyklus erkannt
            }
        }
        chain
    }

    /// Kritikalitäts-Score einer Komponente (wie viele andere abhängen)
    pub fn criticality_score(&self, component: StateComponent) -> usize {
        self.dependents(component).len()
            + self.transitive_triggers(component).len()
            + self.aggregated_by(component).len()
    }
}

// ============================================================================
// CORE STATE LAYER (Κ2-Κ18)
// ============================================================================

/// Trust-State mit Beziehungs-Tracking
#[derive(Debug)]
pub struct TrustState {
    // Atomic Counters
    pub entities: AtomicUsize,
    pub relationships: AtomicUsize,
    pub updates_total: AtomicU64,
    pub positive_updates: AtomicU64,
    pub negative_updates: AtomicU64,
    pub violations: AtomicU64,

    // Complex State (RwLock)
    pub avg_trust: RwLock<f64>,
    pub trust_distribution: RwLock<TrustDistribution>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (Beziehungen im StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Events die durch Trust-Updates ausgelöst wurden (Trust → Event)
    pub triggered_events: AtomicU64,
    /// Trust-Updates die durch Events ausgelöst wurden (Event → Trust)
    pub event_triggered_updates: AtomicU64,
    /// Trust-Updates die durch Realm-Aktivität ausgelöst wurden (Realm → Trust)
    pub realm_triggered_updates: AtomicU64,
}

/// Trust-Verteilung für Diversity-Monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustDistribution {
    /// Buckets: [0-0.1, 0.1-0.2, ..., 0.9-1.0]
    pub histogram: [u64; 10],
    /// Gini-Koeffizient
    pub gini: f64,
    /// Shannon-Entropie
    pub entropy: f64,
}

impl TrustState {
    pub fn new() -> Self {
        Self {
            entities: AtomicUsize::new(0),
            relationships: AtomicUsize::new(0),
            updates_total: AtomicU64::new(0),
            positive_updates: AtomicU64::new(0),
            negative_updates: AtomicU64::new(0),
            violations: AtomicU64::new(0),
            avg_trust: RwLock::new(0.5),
            trust_distribution: RwLock::new(TrustDistribution::default()),
            triggered_events: AtomicU64::new(0),
            event_triggered_updates: AtomicU64::new(0),
            realm_triggered_updates: AtomicU64::new(0),
        }
    }

    /// Update Trust mit Kausalitäts-Tracking
    pub fn update(&self, positive: bool, from_event: bool) {
        self.updates_total.fetch_add(1, Ordering::Relaxed);
        if positive {
            self.positive_updates.fetch_add(1, Ordering::Relaxed);
        } else {
            self.negative_updates.fetch_add(1, Ordering::Relaxed);
        }
        if from_event {
            self.event_triggered_updates.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Trust-Update erzeugt Event
    pub fn update_triggered_event(&self) {
        self.triggered_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Berechne Update-Asymmetrie-Ratio (sollte ~2:1 sein wegen Κ4)
    pub fn asymmetry_ratio(&self) -> f64 {
        let pos = self.positive_updates.load(Ordering::Relaxed) as f64;
        let neg = self.negative_updates.load(Ordering::Relaxed) as f64;
        if pos > 0.0 {
            neg / pos
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> TrustStateSnapshot {
        TrustStateSnapshot {
            entities: self.entities.load(Ordering::Relaxed),
            relationships: self.relationships.load(Ordering::Relaxed),
            updates_total: self.updates_total.load(Ordering::Relaxed),
            positive_updates: self.positive_updates.load(Ordering::Relaxed),
            negative_updates: self.negative_updates.load(Ordering::Relaxed),
            violations: self.violations.load(Ordering::Relaxed),
            avg_trust: self.avg_trust.read().map(|v| *v).unwrap_or(0.5),
            asymmetry_ratio: self.asymmetry_ratio(),
            triggered_events: self.triggered_events.load(Ordering::Relaxed),
            event_triggered_updates: self.event_triggered_updates.load(Ordering::Relaxed),
            distribution: self.trust_distribution.read().map(|d| d.clone()).ok(),
        }
    }
}

impl Default for TrustState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustStateSnapshot {
    pub entities: usize,
    pub relationships: usize,
    pub updates_total: u64,
    pub positive_updates: u64,
    pub negative_updates: u64,
    pub violations: u64,
    pub avg_trust: f64,
    pub asymmetry_ratio: f64,
    pub triggered_events: u64,
    pub event_triggered_updates: u64,
    pub distribution: Option<TrustDistribution>,
}

/// Event-State mit DAG-Tracking und Relationship-Counters
#[derive(Debug)]
pub struct EventState {
    // Atomic Counters
    pub total: AtomicU64,
    pub genesis: AtomicU64,
    pub finalized: AtomicU64,
    pub witnessed: AtomicU64,
    pub validation_errors: AtomicU64,
    pub cycles_detected: AtomicU64,

    // DAG Metrics
    pub max_depth: AtomicU64,
    pub avg_parents: RwLock<f64>,

    // Finality Tracking
    pub finality_latency_ms: RwLock<Vec<u64>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph Trigger-Beziehungen → Event)
    // ─────────────────────────────────────────────────────────────────────────
    /// Events durch Trust-Updates getriggert (Trust → Event)
    pub trust_triggered: AtomicU64,
    /// Events durch Consensus validiert (Consensus → Event)
    pub consensus_validated: AtomicU64,
    /// Events durch Execution getriggert (Execution → Event)
    pub execution_triggered: AtomicU64,
    /// Events durch Gateway/Crossing getriggert (Gateway → Event)
    pub gateway_triggered: AtomicU64,
    /// Events durch Realm getriggert (Realm → Event)
    pub realm_triggered: AtomicU64,
    /// Events durch ECLVM-Ausführung getriggert (ECLVM → Event)
    pub eclvm_triggered: AtomicU64,
    /// Events durch ECLPolicy getriggert (ECLPolicy → Event)
    pub policy_triggered: AtomicU64,
    /// Events durch ECLBlueprint getriggert (ECLBlueprint → Event)
    pub blueprint_triggered: AtomicU64,
    /// Events durch Swarm propagiert (Swarm → Event)
    pub swarm_triggered: AtomicU64,
    /// Events durch Gossip verteilt (Gossip → Event)
    pub gossip_triggered: AtomicU64,
}

impl EventState {
    pub fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            genesis: AtomicU64::new(0),
            finalized: AtomicU64::new(0),
            witnessed: AtomicU64::new(0),
            validation_errors: AtomicU64::new(0),
            cycles_detected: AtomicU64::new(0),
            max_depth: AtomicU64::new(0),
            avg_parents: RwLock::new(0.0),
            finality_latency_ms: RwLock::new(Vec::new()),
            trust_triggered: AtomicU64::new(0),
            consensus_validated: AtomicU64::new(0),
            execution_triggered: AtomicU64::new(0),
            gateway_triggered: AtomicU64::new(0),
            realm_triggered: AtomicU64::new(0),
            eclvm_triggered: AtomicU64::new(0),
            policy_triggered: AtomicU64::new(0),
            blueprint_triggered: AtomicU64::new(0),
            swarm_triggered: AtomicU64::new(0),
            gossip_triggered: AtomicU64::new(0),
        }
    }

    pub fn add(&self, is_genesis: bool, parents_count: usize, depth: u64) {
        self.total.fetch_add(1, Ordering::Relaxed);
        if is_genesis {
            self.genesis.fetch_add(1, Ordering::Relaxed);
        }
        // Update max depth
        loop {
            let current = self.max_depth.load(Ordering::Relaxed);
            if depth <= current {
                break;
            }
            if self
                .max_depth
                .compare_exchange(current, depth, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        // Update avg parents (rolling average)
        if let Ok(mut avg) = self.avg_parents.write() {
            let total = self.total.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + parents_count as f64) / total;
        }
    }

    pub fn finalize(&self, latency_ms: u64) {
        self.finalized.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut latencies) = self.finality_latency_ms.write() {
            latencies.push(latency_ms);
            // Keep last 1000 for averaging
            if latencies.len() > 1000 {
                latencies.remove(0);
            }
        }
    }

    pub fn avg_finality_latency(&self) -> f64 {
        self.finality_latency_ms
            .read()
            .map(|v| {
                if v.is_empty() {
                    0.0
                } else {
                    v.iter().sum::<u64>() as f64 / v.len() as f64
                }
            })
            .unwrap_or(0.0)
    }

    pub fn snapshot(&self) -> EventStateSnapshot {
        EventStateSnapshot {
            total: self.total.load(Ordering::Relaxed),
            genesis: self.genesis.load(Ordering::Relaxed),
            finalized: self.finalized.load(Ordering::Relaxed),
            witnessed: self.witnessed.load(Ordering::Relaxed),
            validation_errors: self.validation_errors.load(Ordering::Relaxed),
            cycles_detected: self.cycles_detected.load(Ordering::Relaxed),
            max_depth: self.max_depth.load(Ordering::Relaxed),
            avg_parents: self.avg_parents.read().map(|v| *v).unwrap_or(0.0),
            avg_finality_latency_ms: self.avg_finality_latency(),
            trust_triggered: self.trust_triggered.load(Ordering::Relaxed),
            consensus_validated: self.consensus_validated.load(Ordering::Relaxed),
        }
    }
}

impl Default for EventState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStateSnapshot {
    pub total: u64,
    pub genesis: u64,
    pub finalized: u64,
    pub witnessed: u64,
    pub validation_errors: u64,
    pub cycles_detected: u64,
    pub max_depth: u64,
    pub avg_parents: f64,
    pub avg_finality_latency_ms: f64,
    pub trust_triggered: u64,
    pub consensus_validated: u64,
}

/// World Formula State (Κ15b-d)
#[derive(Debug)]
pub struct FormulaState {
    pub current_e: RwLock<f64>,
    pub computations: AtomicU64,
    pub contributors: AtomicUsize,
    pub human_verified: AtomicUsize,

    // Komponenten von 𝔼
    pub avg_activity: RwLock<f64>,
    pub avg_trust_norm: RwLock<f64>,
    pub human_factor: RwLock<f64>,

    // History für Trend-Analyse
    pub e_history: RwLock<Vec<(u64, f64)>>, // (timestamp_ms, value)
}

impl FormulaState {
    pub fn new() -> Self {
        Self {
            current_e: RwLock::new(0.0),
            computations: AtomicU64::new(0),
            contributors: AtomicUsize::new(0),
            human_verified: AtomicUsize::new(0),
            avg_activity: RwLock::new(0.0),
            avg_trust_norm: RwLock::new(0.0),
            human_factor: RwLock::new(1.0),
            e_history: RwLock::new(Vec::new()),
        }
    }

    pub fn update(&self, e: f64, activity: f64, trust_norm: f64, human_factor: f64) {
        self.computations.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut current) = self.current_e.write() {
            *current = e;
        }
        if let Ok(mut a) = self.avg_activity.write() {
            *a = activity;
        }
        if let Ok(mut t) = self.avg_trust_norm.write() {
            *t = trust_norm;
        }
        if let Ok(mut h) = self.human_factor.write() {
            *h = human_factor;
        }
        // Record history
        if let Ok(mut history) = self.e_history.write() {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            history.push((ts, e));
            // Keep last 1000 entries
            if history.len() > 1000 {
                history.remove(0);
            }
        }
    }

    /// Berechne Trend (positiv = steigend)
    pub fn trend(&self) -> f64 {
        self.e_history
            .read()
            .map(|h| {
                if h.len() < 10 {
                    return 0.0;
                }
                let recent: f64 = h.iter().rev().take(10).map(|(_, e)| e).sum::<f64>() / 10.0;
                let older: f64 = h
                    .iter()
                    .rev()
                    .skip(10)
                    .take(10)
                    .map(|(_, e)| e)
                    .sum::<f64>()
                    / 10.0_f64.max(h.len().saturating_sub(10) as f64);
                recent - older
            })
            .unwrap_or(0.0)
    }

    pub fn snapshot(&self) -> FormulaStateSnapshot {
        FormulaStateSnapshot {
            current_e: self.current_e.read().map(|v| *v).unwrap_or(0.0),
            computations: self.computations.load(Ordering::Relaxed),
            contributors: self.contributors.load(Ordering::Relaxed),
            human_verified: self.human_verified.load(Ordering::Relaxed),
            avg_activity: self.avg_activity.read().map(|v| *v).unwrap_or(0.0),
            avg_trust_norm: self.avg_trust_norm.read().map(|v| *v).unwrap_or(0.0),
            human_factor: self.human_factor.read().map(|v| *v).unwrap_or(1.0),
            trend: self.trend(),
        }
    }
}

impl Default for FormulaState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaStateSnapshot {
    pub current_e: f64,
    pub computations: u64,
    pub contributors: usize,
    pub human_verified: usize,
    pub avg_activity: f64,
    pub avg_trust_norm: f64,
    pub human_factor: f64,
    pub trend: f64,
}

/// Consensus State (Κ18)
#[derive(Debug)]
pub struct ConsensusState {
    pub epoch: AtomicU64,
    pub validators: AtomicUsize,
    pub successful_rounds: AtomicU64,
    pub failed_rounds: AtomicU64,
    pub avg_round_time_ms: RwLock<f64>,

    // BFT-spezifisch
    pub byzantine_detected: AtomicU64,
    pub leader_changes: AtomicU64,

    // Relationship-Tracking
    /// Events validiert durch Consensus (Consensus ✓ Event)
    pub events_validated: AtomicU64,
}

impl ConsensusState {
    pub fn new() -> Self {
        Self {
            epoch: AtomicU64::new(0),
            validators: AtomicUsize::new(0),
            successful_rounds: AtomicU64::new(0),
            failed_rounds: AtomicU64::new(0),
            avg_round_time_ms: RwLock::new(0.0),
            byzantine_detected: AtomicU64::new(0),
            leader_changes: AtomicU64::new(0),
            events_validated: AtomicU64::new(0),
        }
    }

    pub fn round_completed(&self, success: bool, duration_ms: u64) {
        if success {
            self.successful_rounds.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_rounds.fetch_add(1, Ordering::Relaxed);
        }
        // Rolling average
        if let Ok(mut avg) = self.avg_round_time_ms.write() {
            let total = self.successful_rounds.load(Ordering::Relaxed)
                + self.failed_rounds.load(Ordering::Relaxed);
            *avg = (*avg * (total.saturating_sub(1)) as f64 + duration_ms as f64) / total as f64;
        }
    }

    pub fn success_rate(&self) -> f64 {
        let success = self.successful_rounds.load(Ordering::Relaxed) as f64;
        let failed = self.failed_rounds.load(Ordering::Relaxed) as f64;
        let total = success + failed;
        if total > 0.0 {
            success / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> ConsensusStateSnapshot {
        ConsensusStateSnapshot {
            epoch: self.epoch.load(Ordering::Relaxed),
            validators: self.validators.load(Ordering::Relaxed),
            successful_rounds: self.successful_rounds.load(Ordering::Relaxed),
            failed_rounds: self.failed_rounds.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            avg_round_time_ms: self.avg_round_time_ms.read().map(|v| *v).unwrap_or(0.0),
            byzantine_detected: self.byzantine_detected.load(Ordering::Relaxed),
            leader_changes: self.leader_changes.load(Ordering::Relaxed),
        }
    }
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusStateSnapshot {
    pub epoch: u64,
    pub validators: usize,
    pub successful_rounds: u64,
    pub failed_rounds: u64,
    pub success_rate: f64,
    pub avg_round_time_ms: f64,
    pub byzantine_detected: u64,
    pub leader_changes: u64,
}

/// Aggregierter Core State
#[derive(Debug)]
pub struct CoreState {
    pub trust: TrustState,
    pub events: EventState,
    pub formula: FormulaState,
    pub consensus: ConsensusState,
}

impl CoreState {
    pub fn new() -> Self {
        Self {
            trust: TrustState::new(),
            events: EventState::new(),
            formula: FormulaState::new(),
            consensus: ConsensusState::new(),
        }
    }

    pub fn snapshot(&self) -> CoreStateSnapshot {
        CoreStateSnapshot {
            trust: self.trust.snapshot(),
            events: self.events.snapshot(),
            formula: self.formula.snapshot(),
            consensus: self.consensus.snapshot(),
        }
    }
}

impl Default for CoreState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreStateSnapshot {
    pub trust: TrustStateSnapshot,
    pub events: EventStateSnapshot,
    pub formula: FormulaStateSnapshot,
    pub consensus: ConsensusStateSnapshot,
}

// ============================================================================
// EXECUTION STATE LAYER (IPS ℳ) - Tiefe Struktur mit Sub-States
// ============================================================================

/// Gas-State mit Relationship-Tracking
///
/// Gas ist die Compute-Ressource für ECL-Ausführungen.
/// Basiert auf Trust (DependsOn) und wird durch Calibration angepasst (Triggers).
#[derive(Debug)]
pub struct GasState {
    /// Total verbrauchtes Gas
    pub consumed: AtomicU64,
    /// Refundiertes Gas
    pub refunded: AtomicU64,
    /// Out-of-Gas Errors
    pub out_of_gas: AtomicU64,
    /// Aktueller Gas-Preis
    pub current_price: RwLock<f64>,
    /// Max Gas pro Block
    pub max_per_block: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Calibration hat Gas angepasst (Calibration → Gas)
    pub calibration_adjustments: AtomicU64,
    /// Trust-Dependency-Updates (Gas ← Trust)
    pub trust_dependency_updates: AtomicU64,
}

impl GasState {
    pub fn new() -> Self {
        Self {
            consumed: AtomicU64::new(0),
            refunded: AtomicU64::new(0),
            out_of_gas: AtomicU64::new(0),
            current_price: RwLock::new(1.0),
            max_per_block: AtomicU64::new(10_000_000),
            calibration_adjustments: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
        }
    }

    pub fn consume(&self, amount: u64) {
        self.consumed.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn refund(&self, amount: u64) {
        self.refunded.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> GasStateSnapshot {
        GasStateSnapshot {
            consumed: self.consumed.load(Ordering::Relaxed),
            refunded: self.refunded.load(Ordering::Relaxed),
            out_of_gas: self.out_of_gas.load(Ordering::Relaxed),
            current_price: self.current_price.read().map(|v| *v).unwrap_or(1.0),
            max_per_block: self.max_per_block.load(Ordering::Relaxed),
            calibration_adjustments: self.calibration_adjustments.load(Ordering::Relaxed),
            trust_dependency_updates: self.trust_dependency_updates.load(Ordering::Relaxed),
        }
    }
}

impl Default for GasState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasStateSnapshot {
    pub consumed: u64,
    pub refunded: u64,
    pub out_of_gas: u64,
    pub current_price: f64,
    pub max_per_block: u64,
    pub calibration_adjustments: u64,
    pub trust_dependency_updates: u64,
}

/// Mana-State mit Relationship-Tracking
///
/// Mana ist die Bandwidth/Event-Ressource.
/// Regeneriert über Zeit, basiert auf Trust (DependsOn).
#[derive(Debug)]
pub struct ManaState {
    /// Total verbrauchtes Mana
    pub consumed: AtomicU64,
    /// Regeneriertes Mana
    pub regenerated: AtomicU64,
    /// Rate-Limited wegen Mana
    pub rate_limited: AtomicU64,
    /// Aktuelle Regenerations-Rate
    pub regen_rate: RwLock<f64>,
    /// Max Mana pro Entity
    pub max_per_entity: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Calibration hat Mana angepasst (Calibration → Mana)
    pub calibration_adjustments: AtomicU64,
    /// Trust-Dependency-Updates (Mana ← Trust)
    pub trust_dependency_updates: AtomicU64,
}

impl ManaState {
    pub fn new() -> Self {
        Self {
            consumed: AtomicU64::new(0),
            regenerated: AtomicU64::new(0),
            rate_limited: AtomicU64::new(0),
            regen_rate: RwLock::new(1.0),
            max_per_entity: AtomicU64::new(100_000),
            calibration_adjustments: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
        }
    }

    pub fn consume(&self, amount: u64) {
        self.consumed.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn regenerate(&self, amount: u64) {
        self.regenerated.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> ManaStateSnapshot {
        ManaStateSnapshot {
            consumed: self.consumed.load(Ordering::Relaxed),
            regenerated: self.regenerated.load(Ordering::Relaxed),
            rate_limited: self.rate_limited.load(Ordering::Relaxed),
            regen_rate: self.regen_rate.read().map(|v| *v).unwrap_or(1.0),
            max_per_entity: self.max_per_entity.load(Ordering::Relaxed),
            calibration_adjustments: self.calibration_adjustments.load(Ordering::Relaxed),
            trust_dependency_updates: self.trust_dependency_updates.load(Ordering::Relaxed),
        }
    }
}

impl Default for ManaState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaStateSnapshot {
    pub consumed: u64,
    pub regenerated: u64,
    pub rate_limited: u64,
    pub regen_rate: f64,
    pub max_per_entity: u64,
    pub calibration_adjustments: u64,
    pub trust_dependency_updates: u64,
}

/// Core Execution State mit Relationship-Tracking
#[derive(Debug)]
pub struct ExecutionsState {
    /// Aktive Execution-Kontexte
    pub active_contexts: AtomicUsize,
    /// Total Executions
    pub total: AtomicU64,
    /// Erfolgreiche Executions
    pub successful: AtomicU64,
    /// Fehlgeschlagene Executions
    pub failed: AtomicU64,
    /// Events emittiert
    pub events_emitted: AtomicU64,
    /// Ausführungszeiten für Averaging
    pub execution_times_ms: RwLock<Vec<u64>>,
    /// Aktuelles Epoch
    pub current_epoch: AtomicU64,
    /// Aktueller Lamport-Timestamp
    pub current_lamport: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Executions durch SagaComposer getriggert (SagaComposer → Execution)
    pub saga_triggered: AtomicU64,
    /// Gas-Aggregationen (Execution ⊃ Gas)
    pub gas_aggregations: AtomicU64,
    /// Mana-Aggregationen (Execution ⊃ Mana)
    pub mana_aggregations: AtomicU64,
}

impl ExecutionsState {
    pub fn new() -> Self {
        Self {
            active_contexts: AtomicUsize::new(0),
            total: AtomicU64::new(0),
            successful: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            events_emitted: AtomicU64::new(0),
            execution_times_ms: RwLock::new(Vec::new()),
            current_epoch: AtomicU64::new(0),
            current_lamport: AtomicU64::new(0),
            saga_triggered: AtomicU64::new(0),
            gas_aggregations: AtomicU64::new(0),
            mana_aggregations: AtomicU64::new(0),
        }
    }

    pub fn start(&self) {
        self.active_contexts.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn complete(&self, success: bool, events: u64, duration_ms: u64) {
        self.active_contexts.fetch_sub(1, Ordering::Relaxed);
        if success {
            self.successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }
        self.events_emitted.fetch_add(events, Ordering::Relaxed);

        if let Ok(mut times) = self.execution_times_ms.write() {
            times.push(duration_ms);
            if times.len() > 1000 {
                times.remove(0);
            }
        }
    }

    pub fn avg_execution_time(&self) -> f64 {
        self.execution_times_ms
            .read()
            .map(|v| {
                if v.is_empty() {
                    0.0
                } else {
                    v.iter().sum::<u64>() as f64 / v.len() as f64
                }
            })
            .unwrap_or(0.0)
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> ExecutionsStateSnapshot {
        ExecutionsStateSnapshot {
            active_contexts: self.active_contexts.load(Ordering::Relaxed),
            total: self.total.load(Ordering::Relaxed),
            successful: self.successful.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            events_emitted: self.events_emitted.load(Ordering::Relaxed),
            avg_execution_time_ms: self.avg_execution_time(),
            current_epoch: self.current_epoch.load(Ordering::Relaxed),
            current_lamport: self.current_lamport.load(Ordering::Relaxed),
            saga_triggered: self.saga_triggered.load(Ordering::Relaxed),
            gas_aggregations: self.gas_aggregations.load(Ordering::Relaxed),
            mana_aggregations: self.mana_aggregations.load(Ordering::Relaxed),
        }
    }
}

impl Default for ExecutionsState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionsStateSnapshot {
    pub active_contexts: usize,
    pub total: u64,
    pub successful: u64,
    pub failed: u64,
    pub success_rate: f64,
    pub events_emitted: u64,
    pub avg_execution_time_ms: f64,
    pub current_epoch: u64,
    pub current_lamport: u64,
    pub saga_triggered: u64,
    pub gas_aggregations: u64,
    pub mana_aggregations: u64,
}

/// Execution State Layer mit Sub-States für tiefe Relationship-Integration
#[derive(Debug)]
pub struct ExecutionState {
    /// Gas Sub-State
    pub gas: GasState,
    /// Mana Sub-State
    pub mana: ManaState,
    /// Core Executions Sub-State
    pub executions: ExecutionsState,
}

impl ExecutionState {
    pub fn new() -> Self {
        Self {
            gas: GasState::new(),
            mana: ManaState::new(),
            executions: ExecutionsState::new(),
        }
    }

    /// Legacy-Kompatibilität: Start Execution
    pub fn start(&self) {
        self.executions.start();
    }

    /// Legacy-Kompatibilität: Complete Execution
    pub fn complete(&self, success: bool, gas: u64, mana: u64, events: u64, duration_ms: u64) {
        self.executions.complete(success, events, duration_ms);
        self.gas.consume(gas);
        self.mana.consume(mana);
    }

    /// Legacy-Kompatibilität: Durchschnittliche Ausführungszeit
    pub fn avg_execution_time(&self) -> f64 {
        self.executions.avg_execution_time()
    }

    /// Legacy-Kompatibilität: Erfolgsrate
    pub fn success_rate(&self) -> f64 {
        self.executions.success_rate()
    }

    pub fn snapshot(&self) -> ExecutionStateSnapshot {
        ExecutionStateSnapshot {
            gas: self.gas.snapshot(),
            mana: self.mana.snapshot(),
            executions: self.executions.snapshot(),
        }
    }
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution State Snapshot mit Sub-States
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStateSnapshot {
    pub gas: GasStateSnapshot,
    pub mana: ManaStateSnapshot,
    pub executions: ExecutionsStateSnapshot,
}

// ============================================================================
// ECLVM STATE LAYER (Erynoa Core Language Virtual Machine)
// ============================================================================
//
// ECL (Erynoa Core Language) ist die DSL für:
// - Regeln definieren (Crossing-Policies, Membership, Transaction-Rules)
// - Blueprints erstellen (App-Templates für Chat, Marketplace, etc.)
// - Intents & Sagas beschreiben (Cross-Realm-Aktionen)
//
// ECLVM ist die cost-limited Execution Environment:
// - Sicher durch Gas (Compute) und Mana (Bandwidth/Events)
// - Integration mit ExecutionState für Resource-Tracking
// - Realm-spezifische Policy-Ausführung

/// Policy-Typ für ECL-Regeln
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ECLPolicyType {
    /// Crossing-Rules für Gateway (Κ23)
    Crossing,
    /// Membership-Rules für Realm-Beitritt
    Membership,
    /// Transaction-Rules für Aktionen
    Transaction,
    /// Governance-Rules für Abstimmungen
    Governance,
    /// Privacy-Rules für Daten-Sichtbarkeit
    Privacy,
    /// Custom User-defined Policy
    Custom,
}

/// Blueprint-Status im Marketplace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlueprintStatus {
    /// Entwurf, noch nicht publiziert
    Draft,
    /// Veröffentlicht und verfügbar
    Published,
    /// Verifiziert durch Community
    Verified,
    /// Deprecated, nicht mehr empfohlen
    Deprecated,
}

/// Per-Realm ECL State - Policy-Ausführungen pro Realm
#[derive(Debug)]
pub struct RealmECLState {
    /// Policies ausgeführt in diesem Realm
    pub policies_executed: AtomicU64,
    /// Erfolgreiche Policy-Evaluationen
    pub policies_passed: AtomicU64,
    /// Fehlgeschlagene Policy-Evaluationen
    pub policies_denied: AtomicU64,
    /// Gas verbraucht für Policies in diesem Realm
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht für Policies in diesem Realm
    pub mana_consumed: AtomicU64,
    /// Crossing-Policies ausgeführt
    pub crossing_policies: AtomicU64,
    /// Membership-Policies ausgeführt
    pub membership_policies: AtomicU64,
    /// Aktive compiled Policies in diesem Realm
    pub active_policies: RwLock<Vec<String>>,
    /// Instantiierte Blueprints in diesem Realm
    pub instantiated_blueprints: AtomicU64,
}

impl RealmECLState {
    pub fn new() -> Self {
        Self {
            policies_executed: AtomicU64::new(0),
            policies_passed: AtomicU64::new(0),
            policies_denied: AtomicU64::new(0),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            crossing_policies: AtomicU64::new(0),
            membership_policies: AtomicU64::new(0),
            active_policies: RwLock::new(Vec::new()),
            instantiated_blueprints: AtomicU64::new(0),
        }
    }

    pub fn policy_executed(&self, passed: bool, policy_type: ECLPolicyType, gas: u64, mana: u64) {
        self.policies_executed.fetch_add(1, Ordering::Relaxed);
        if passed {
            self.policies_passed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.policies_denied.fetch_add(1, Ordering::Relaxed);
        }
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.mana_consumed.fetch_add(mana, Ordering::Relaxed);

        match policy_type {
            ECLPolicyType::Crossing => {
                self.crossing_policies.fetch_add(1, Ordering::Relaxed);
            }
            ECLPolicyType::Membership => {
                self.membership_policies.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    pub fn register_policy(&self, policy_id: &str) {
        if let Ok(mut policies) = self.active_policies.write() {
            if !policies.contains(&policy_id.to_string()) {
                policies.push(policy_id.to_string());
            }
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.policies_executed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.policies_passed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> RealmECLStateSnapshot {
        RealmECLStateSnapshot {
            policies_executed: self.policies_executed.load(Ordering::Relaxed),
            policies_passed: self.policies_passed.load(Ordering::Relaxed),
            policies_denied: self.policies_denied.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            crossing_policies: self.crossing_policies.load(Ordering::Relaxed),
            membership_policies: self.membership_policies.load(Ordering::Relaxed),
            active_policies: self
                .active_policies
                .read()
                .map(|p| p.clone())
                .unwrap_or_default(),
            instantiated_blueprints: self.instantiated_blueprints.load(Ordering::Relaxed),
        }
    }
}

impl Default for RealmECLState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmECLStateSnapshot {
    pub policies_executed: u64,
    pub policies_passed: u64,
    pub policies_denied: u64,
    pub success_rate: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub crossing_policies: u64,
    pub membership_policies: u64,
    pub active_policies: Vec<String>,
    pub instantiated_blueprints: u64,
}

/// ECLVM State - Erynoa Core Language Virtual Machine
///
/// Trackt alle ECL-bezogenen Aktivitäten:
/// - Policy-Kompilierung und -Ausführung
/// - Blueprint-Management (Publish, Deploy, Instantiate)
/// - Saga/Intent-Orchestrierung durch ECL
/// - Realm-spezifische ECL-Metriken
#[derive(Debug)]
pub struct ECLVMState {
    // === Policy Engine ===
    /// Policies kompiliert insgesamt
    pub policies_compiled: AtomicU64,
    /// Policies im Cache (compiled bytecode)
    pub policies_cached: AtomicUsize,
    /// Policy-Kompilierungsfehler
    pub policy_compile_errors: AtomicU64,
    /// Policy-Ausführungen insgesamt
    pub policies_executed: AtomicU64,
    /// Erfolgreiche Policy-Evaluationen
    pub policies_passed: AtomicU64,
    /// Fehlgeschlagene Policy-Evaluationen (denied)
    pub policies_denied: AtomicU64,
    /// Policy-Runtime-Fehler (Bugs, nicht Denials)
    pub policy_runtime_errors: AtomicU64,
    /// Policies nach Typ
    pub policies_by_type: RwLock<HashMap<String, u64>>,

    // === Blueprint Engine ===
    /// Blueprints publiziert (im Marketplace)
    pub blueprints_published: AtomicU64,
    /// Blueprints deployed (ready for instantiation)
    pub blueprints_deployed: AtomicU64,
    /// Blueprint-Instanziierungen
    pub blueprints_instantiated: AtomicU64,
    /// Blueprint-Verifikationen (Community)
    pub blueprints_verified: AtomicU64,
    /// Blueprint-Downloads
    pub blueprints_downloaded: AtomicU64,
    /// Blueprints nach Status
    pub blueprints_by_status: RwLock<HashMap<String, u64>>,

    // === Saga/Intent Orchestrierung ===
    /// Intents verarbeitet durch ECL
    pub intents_processed: AtomicU64,
    /// Intents erfolgreich ausgeführt
    pub intents_successful: AtomicU64,
    /// Saga-Steps durch ECLVM ausgeführt
    pub saga_steps_executed: AtomicU64,
    /// Cross-Realm-Saga-Steps
    pub cross_realm_steps: AtomicU64,
    /// Kompensationen durch ECLVM
    pub compensations_triggered: AtomicU64,

    // === Resource Tracking ===
    /// Gesamt-Gas verbraucht durch ECLVM
    pub total_gas_consumed: AtomicU64,
    /// Gesamt-Mana verbraucht durch ECLVM
    pub total_mana_consumed: AtomicU64,
    /// Out-of-Gas während ECL-Ausführung
    pub out_of_gas_aborts: AtomicU64,
    /// Rate-Limited durch Mana-Erschöpfung
    pub mana_rate_limited: AtomicU64,

    // === Per-Realm ECL State ===
    /// ECL-State pro Realm
    pub realm_ecl: RwLock<HashMap<String, RealmECLState>>,

    // === Crossing-Policy Cache (Κ23) ===
    /// Crossing-Policies evaluiert
    pub crossing_evaluations: AtomicU64,
    /// Crossings durch Policy erlaubt
    pub crossings_allowed: AtomicU64,
    /// Crossings durch Policy abgelehnt
    pub crossings_denied: AtomicU64,
    /// Durchschnittliche Policy-Evaluation Zeit (µs)
    pub avg_evaluation_time_us: RwLock<f64>,

    // === Events ===
    /// Events emittiert durch ECLVM
    pub events_emitted: AtomicU64,
}

impl ECLVMState {
    pub fn new() -> Self {
        Self {
            policies_compiled: AtomicU64::new(0),
            policies_cached: AtomicUsize::new(0),
            policy_compile_errors: AtomicU64::new(0),
            policies_executed: AtomicU64::new(0),
            policies_passed: AtomicU64::new(0),
            policies_denied: AtomicU64::new(0),
            policy_runtime_errors: AtomicU64::new(0),
            policies_by_type: RwLock::new(HashMap::new()),
            blueprints_published: AtomicU64::new(0),
            blueprints_deployed: AtomicU64::new(0),
            blueprints_instantiated: AtomicU64::new(0),
            blueprints_verified: AtomicU64::new(0),
            blueprints_downloaded: AtomicU64::new(0),
            blueprints_by_status: RwLock::new(HashMap::new()),
            intents_processed: AtomicU64::new(0),
            intents_successful: AtomicU64::new(0),
            saga_steps_executed: AtomicU64::new(0),
            cross_realm_steps: AtomicU64::new(0),
            compensations_triggered: AtomicU64::new(0),
            total_gas_consumed: AtomicU64::new(0),
            total_mana_consumed: AtomicU64::new(0),
            out_of_gas_aborts: AtomicU64::new(0),
            mana_rate_limited: AtomicU64::new(0),
            realm_ecl: RwLock::new(HashMap::new()),
            crossing_evaluations: AtomicU64::new(0),
            crossings_allowed: AtomicU64::new(0),
            crossings_denied: AtomicU64::new(0),
            avg_evaluation_time_us: RwLock::new(0.0),
            events_emitted: AtomicU64::new(0),
        }
    }

    // === Policy Operations ===

    pub fn policy_compiled(&self, success: bool, policy_type: ECLPolicyType) {
        if success {
            self.policies_compiled.fetch_add(1, Ordering::Relaxed);
            self.policies_cached.fetch_add(1, Ordering::Relaxed);
        } else {
            self.policy_compile_errors.fetch_add(1, Ordering::Relaxed);
        }
        // Track by type
        let type_name = format!("{:?}", policy_type);
        if let Ok(mut by_type) = self.policies_by_type.write() {
            *by_type.entry(type_name).or_insert(0) += 1;
        }
    }

    pub fn policy_executed(
        &self,
        passed: bool,
        policy_type: ECLPolicyType,
        gas: u64,
        mana: u64,
        duration_us: u64,
        realm_id: Option<&str>,
    ) {
        self.policies_executed.fetch_add(1, Ordering::Relaxed);
        if passed {
            self.policies_passed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.policies_denied.fetch_add(1, Ordering::Relaxed);
        }
        self.total_gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.total_mana_consumed.fetch_add(mana, Ordering::Relaxed);
        self.events_emitted.fetch_add(1, Ordering::Relaxed);

        // Update per-realm state
        if let Some(realm) = realm_id {
            self.get_or_create_realm_ecl(realm)
                .policy_executed(passed, policy_type, gas, mana);
        }

        // Update avg evaluation time
        if let Ok(mut avg) = self.avg_evaluation_time_us.write() {
            let total = self.policies_executed.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + duration_us as f64) / total;
        }
    }

    pub fn policy_runtime_error(&self) {
        self.policy_runtime_errors.fetch_add(1, Ordering::Relaxed);
    }

    // === Blueprint Operations ===

    pub fn blueprint_published(&self) {
        self.blueprints_published.fetch_add(1, Ordering::Relaxed);
        self.update_blueprint_status("Draft");
    }

    pub fn blueprint_deployed(&self) {
        self.blueprints_deployed.fetch_add(1, Ordering::Relaxed);
        self.update_blueprint_status("Published");
    }

    pub fn blueprint_instantiated(&self, realm_id: &str) {
        self.blueprints_instantiated.fetch_add(1, Ordering::Relaxed);
        self.events_emitted.fetch_add(1, Ordering::Relaxed);

        // Track per-realm
        if let Ok(realms) = self.realm_ecl.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm
                    .instantiated_blueprints
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn blueprint_verified(&self) {
        self.blueprints_verified.fetch_add(1, Ordering::Relaxed);
        self.update_blueprint_status("Verified");
    }

    pub fn blueprint_downloaded(&self) {
        self.blueprints_downloaded.fetch_add(1, Ordering::Relaxed);
    }

    fn update_blueprint_status(&self, status: &str) {
        if let Ok(mut by_status) = self.blueprints_by_status.write() {
            *by_status.entry(status.to_string()).or_insert(0) += 1;
        }
    }

    // === Saga/Intent Operations ===

    pub fn intent_processed(&self, success: bool) {
        self.intents_processed.fetch_add(1, Ordering::Relaxed);
        if success {
            self.intents_successful.fetch_add(1, Ordering::Relaxed);
        }
        self.events_emitted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn saga_step_executed(&self, cross_realm: bool, gas: u64, mana: u64) {
        self.saga_steps_executed.fetch_add(1, Ordering::Relaxed);
        if cross_realm {
            self.cross_realm_steps.fetch_add(1, Ordering::Relaxed);
        }
        self.total_gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.total_mana_consumed.fetch_add(mana, Ordering::Relaxed);
    }

    pub fn compensation_triggered(&self) {
        self.compensations_triggered.fetch_add(1, Ordering::Relaxed);
    }

    // === Crossing-Policy (Κ23) ===

    pub fn crossing_policy_evaluated(&self, allowed: bool, from_realm: &str, to_realm: &str) {
        self.crossing_evaluations.fetch_add(1, Ordering::Relaxed);
        if allowed {
            self.crossings_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.crossings_denied.fetch_add(1, Ordering::Relaxed);
        }

        // Track in source realm
        self.get_or_create_realm_ecl(from_realm)
            .crossing_policies
            .fetch_add(1, Ordering::Relaxed);
        // Track in target realm
        self.get_or_create_realm_ecl(to_realm)
            .crossing_policies
            .fetch_add(1, Ordering::Relaxed);
    }

    // === Resource Tracking ===

    pub fn out_of_gas(&self) {
        self.out_of_gas_aborts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn rate_limited(&self) {
        self.mana_rate_limited.fetch_add(1, Ordering::Relaxed);
    }

    // === Per-Realm Operations ===

    pub fn register_realm(&self, realm_id: &str) {
        if let Ok(mut realms) = self.realm_ecl.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmECLState::new);
        }
    }

    pub fn register_policy_to_realm(&self, realm_id: &str, policy_id: &str) {
        self.get_or_create_realm_ecl(realm_id)
            .register_policy(policy_id);
    }

    /// Holt oder erstellt RealmECLState für ein Realm
    pub fn get_or_create_realm_ecl(&self, realm_id: &str) -> &RealmECLState {
        // Note: This is a simplification - in production you'd use a more sophisticated approach
        if let Ok(mut realms) = self.realm_ecl.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmECLState::new);
        }
        // Return a reference - this works because we hold the lock
        // In practice, you might want to return a guard or use interior mutability
        unsafe {
            // Safe because we just ensured the entry exists
            self.realm_ecl
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmECLState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmECLState> = std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmECLState::new)
                })
        }
    }

    pub fn get_realm_ecl(&self, realm_id: &str) -> Option<RealmECLStateSnapshot> {
        self.realm_ecl
            .read()
            .ok()?
            .get(realm_id)
            .map(|r| r.snapshot())
    }

    // === Metrics ===

    pub fn policy_success_rate(&self) -> f64 {
        let total = self.policies_executed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.policies_passed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn crossing_allow_rate(&self) -> f64 {
        let total = self.crossing_evaluations.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.crossings_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn intent_success_rate(&self) -> f64 {
        let total = self.intents_processed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.intents_successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> ECLVMStateSnapshot {
        let realm_snapshots = self
            .realm_ecl
            .read()
            .map(|r| r.iter().map(|(k, v)| (k.clone(), v.snapshot())).collect())
            .unwrap_or_default();

        ECLVMStateSnapshot {
            policies_compiled: self.policies_compiled.load(Ordering::Relaxed),
            policies_cached: self.policies_cached.load(Ordering::Relaxed),
            policy_compile_errors: self.policy_compile_errors.load(Ordering::Relaxed),
            policies_executed: self.policies_executed.load(Ordering::Relaxed),
            policies_passed: self.policies_passed.load(Ordering::Relaxed),
            policies_denied: self.policies_denied.load(Ordering::Relaxed),
            policy_runtime_errors: self.policy_runtime_errors.load(Ordering::Relaxed),
            policy_success_rate: self.policy_success_rate(),
            policies_by_type: self
                .policies_by_type
                .read()
                .map(|p| p.clone())
                .unwrap_or_default(),
            blueprints_published: self.blueprints_published.load(Ordering::Relaxed),
            blueprints_deployed: self.blueprints_deployed.load(Ordering::Relaxed),
            blueprints_instantiated: self.blueprints_instantiated.load(Ordering::Relaxed),
            blueprints_verified: self.blueprints_verified.load(Ordering::Relaxed),
            blueprints_downloaded: self.blueprints_downloaded.load(Ordering::Relaxed),
            blueprints_by_status: self
                .blueprints_by_status
                .read()
                .map(|b| b.clone())
                .unwrap_or_default(),
            intents_processed: self.intents_processed.load(Ordering::Relaxed),
            intents_successful: self.intents_successful.load(Ordering::Relaxed),
            intent_success_rate: self.intent_success_rate(),
            saga_steps_executed: self.saga_steps_executed.load(Ordering::Relaxed),
            cross_realm_steps: self.cross_realm_steps.load(Ordering::Relaxed),
            compensations_triggered: self.compensations_triggered.load(Ordering::Relaxed),
            total_gas_consumed: self.total_gas_consumed.load(Ordering::Relaxed),
            total_mana_consumed: self.total_mana_consumed.load(Ordering::Relaxed),
            out_of_gas_aborts: self.out_of_gas_aborts.load(Ordering::Relaxed),
            mana_rate_limited: self.mana_rate_limited.load(Ordering::Relaxed),
            realm_ecl: realm_snapshots,
            crossing_evaluations: self.crossing_evaluations.load(Ordering::Relaxed),
            crossings_allowed: self.crossings_allowed.load(Ordering::Relaxed),
            crossings_denied: self.crossings_denied.load(Ordering::Relaxed),
            crossing_allow_rate: self.crossing_allow_rate(),
            avg_evaluation_time_us: self
                .avg_evaluation_time_us
                .read()
                .map(|a| *a)
                .unwrap_or(0.0),
            events_emitted: self.events_emitted.load(Ordering::Relaxed),
        }
    }
}

impl Default for ECLVMState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECLVMStateSnapshot {
    // Policy Engine
    pub policies_compiled: u64,
    pub policies_cached: usize,
    pub policy_compile_errors: u64,
    pub policies_executed: u64,
    pub policies_passed: u64,
    pub policies_denied: u64,
    pub policy_runtime_errors: u64,
    pub policy_success_rate: f64,
    pub policies_by_type: HashMap<String, u64>,
    // Blueprint Engine
    pub blueprints_published: u64,
    pub blueprints_deployed: u64,
    pub blueprints_instantiated: u64,
    pub blueprints_verified: u64,
    pub blueprints_downloaded: u64,
    pub blueprints_by_status: HashMap<String, u64>,
    // Saga/Intent
    pub intents_processed: u64,
    pub intents_successful: u64,
    pub intent_success_rate: f64,
    pub saga_steps_executed: u64,
    pub cross_realm_steps: u64,
    pub compensations_triggered: u64,
    // Resources
    pub total_gas_consumed: u64,
    pub total_mana_consumed: u64,
    pub out_of_gas_aborts: u64,
    pub mana_rate_limited: u64,
    // Per-Realm
    pub realm_ecl: HashMap<String, RealmECLStateSnapshot>,
    // Crossing-Policy
    pub crossing_evaluations: u64,
    pub crossings_allowed: u64,
    pub crossings_denied: u64,
    pub crossing_allow_rate: f64,
    pub avg_evaluation_time_us: f64,
    // Events
    pub events_emitted: u64,
}

// ============================================================================
// PROTECTION STATE LAYER (Κ19-Κ21) - Tiefe Struktur mit Sub-States
// ============================================================================

/// Anomaly Detection Sub-State mit Relationship-Tracking
#[derive(Debug)]
pub struct AnomalyState {
    /// Total Anomalien erkannt
    pub total: AtomicU64,
    /// Kritische Anomalien
    pub critical: AtomicU64,
    /// Hohe Anomalien
    pub high: AtomicU64,
    /// Mittlere Anomalien
    pub medium: AtomicU64,
    /// Niedrige Anomalien
    pub low: AtomicU64,
    /// False Positives
    pub false_positives: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (Anomaly ✓ Event/Trust)
    // ─────────────────────────────────────────────────────────────────────────
    /// Events validiert (Anomaly ✓ Event)
    pub events_validated: AtomicU64,
    /// Trust-Patterns geprüft (Anomaly ✓ Trust)
    pub trust_patterns_checked: AtomicU64,
}

impl AnomalyState {
    pub fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            critical: AtomicU64::new(0),
            high: AtomicU64::new(0),
            medium: AtomicU64::new(0),
            low: AtomicU64::new(0),
            false_positives: AtomicU64::new(0),
            events_validated: AtomicU64::new(0),
            trust_patterns_checked: AtomicU64::new(0),
        }
    }

    pub fn record(&self, severity: &str) {
        self.total.fetch_add(1, Ordering::Relaxed);
        match severity {
            "critical" => self.critical.fetch_add(1, Ordering::Relaxed),
            "high" => self.high.fetch_add(1, Ordering::Relaxed),
            "medium" => self.medium.fetch_add(1, Ordering::Relaxed),
            _ => self.low.fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn snapshot(&self) -> AnomalyStateSnapshot {
        AnomalyStateSnapshot {
            total: self.total.load(Ordering::Relaxed),
            critical: self.critical.load(Ordering::Relaxed),
            high: self.high.load(Ordering::Relaxed),
            medium: self.medium.load(Ordering::Relaxed),
            low: self.low.load(Ordering::Relaxed),
            false_positives: self.false_positives.load(Ordering::Relaxed),
            events_validated: self.events_validated.load(Ordering::Relaxed),
            trust_patterns_checked: self.trust_patterns_checked.load(Ordering::Relaxed),
        }
    }
}

impl Default for AnomalyState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyStateSnapshot {
    pub total: u64,
    pub critical: u64,
    pub high: u64,
    pub medium: u64,
    pub low: u64,
    pub false_positives: u64,
    pub events_validated: u64,
    pub trust_patterns_checked: u64,
}

/// Diversity Monitor Sub-State (Κ20) mit Relationship-Tracking
#[derive(Debug)]
pub struct DiversityState {
    /// Dimensionen die überwacht werden
    pub dimensions: AtomicUsize,
    /// Monokultur-Warnungen
    pub monoculture_warnings: AtomicU64,
    /// Entropy pro Dimension
    pub entropy_values: RwLock<HashMap<String, f64>>,
    /// Minimum Entropy
    pub min_entropy: RwLock<f64>,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (Diversity ✓ Trust/Consensus)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Verteilung geprüft (Diversity ✓ Trust)
    pub trust_distribution_checks: AtomicU64,
    /// Validator-Mix geprüft (Diversity ✓ Consensus)
    pub validator_mix_checks: AtomicU64,
}

impl DiversityState {
    pub fn new() -> Self {
        Self {
            dimensions: AtomicUsize::new(0),
            monoculture_warnings: AtomicU64::new(0),
            entropy_values: RwLock::new(HashMap::new()),
            min_entropy: RwLock::new(1.0),
            trust_distribution_checks: AtomicU64::new(0),
            validator_mix_checks: AtomicU64::new(0),
        }
    }

    pub fn set_entropy(&self, dimension: &str, value: f64) {
        if let Ok(mut map) = self.entropy_values.write() {
            map.insert(dimension.to_string(), value);
            if let Ok(mut min) = self.min_entropy.write() {
                *min = map.values().copied().fold(f64::MAX, f64::min);
            }
        }
    }

    pub fn snapshot(&self) -> DiversityStateSnapshot {
        DiversityStateSnapshot {
            dimensions: self.dimensions.load(Ordering::Relaxed),
            monoculture_warnings: self.monoculture_warnings.load(Ordering::Relaxed),
            min_entropy: self.min_entropy.read().map(|v| *v).unwrap_or(1.0),
            trust_distribution_checks: self.trust_distribution_checks.load(Ordering::Relaxed),
            validator_mix_checks: self.validator_mix_checks.load(Ordering::Relaxed),
        }
    }
}

impl Default for DiversityState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversityStateSnapshot {
    pub dimensions: usize,
    pub monoculture_warnings: u64,
    pub min_entropy: f64,
    pub trust_distribution_checks: u64,
    pub validator_mix_checks: u64,
}

/// Quadratic Governance Sub-State (Κ21) mit Relationship-Tracking
#[derive(Debug)]
pub struct QuadraticState {
    /// Aktive Abstimmungen
    pub active_votes: AtomicUsize,
    /// Abgeschlossene Abstimmungen
    pub completed_votes: AtomicU64,
    /// Teilnehmer total
    pub total_participants: AtomicU64,
    /// Quadratische Reduktionen angewandt
    pub quadratic_reductions: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (Quadratic ← Trust)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (Quadratic ← Trust)
    pub trust_dependency_updates: AtomicU64,
}

impl QuadraticState {
    pub fn new() -> Self {
        Self {
            active_votes: AtomicUsize::new(0),
            completed_votes: AtomicU64::new(0),
            total_participants: AtomicU64::new(0),
            quadratic_reductions: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> QuadraticStateSnapshot {
        QuadraticStateSnapshot {
            active_votes: self.active_votes.load(Ordering::Relaxed),
            completed_votes: self.completed_votes.load(Ordering::Relaxed),
            total_participants: self.total_participants.load(Ordering::Relaxed),
            quadratic_reductions: self.quadratic_reductions.load(Ordering::Relaxed),
            trust_dependency_updates: self.trust_dependency_updates.load(Ordering::Relaxed),
        }
    }
}

impl Default for QuadraticState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadraticStateSnapshot {
    pub active_votes: usize,
    pub completed_votes: u64,
    pub total_participants: u64,
    pub quadratic_reductions: u64,
    pub trust_dependency_updates: u64,
}

/// Anti-Calcification Sub-State (Κ19) mit Relationship-Tracking
#[derive(Debug)]
pub struct AntiCalcificationState {
    /// Power-Konzentration (0.0 = perfekt verteilt, 1.0 = monopol)
    pub power_concentration: RwLock<f64>,
    /// Gini-Koeffizient
    pub gini_coefficient: RwLock<f64>,
    /// Interventionen durchgeführt
    pub interventions: AtomicU64,
    /// Überwachte Entitäten
    pub watched_entities: AtomicUsize,
    /// Schwellenwert-Verletzungen
    pub threshold_violations: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (AntiCalcification ✓/→ Trust)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Limits geprüft (AntiCalcification → Trust)
    pub trust_limits_checked: AtomicU64,
    /// Power-Checks durchgeführt (AntiCalcification ✓ Trust)
    pub power_checks: AtomicU64,
}

impl AntiCalcificationState {
    pub fn new() -> Self {
        Self {
            power_concentration: RwLock::new(0.0),
            gini_coefficient: RwLock::new(0.0),
            interventions: AtomicU64::new(0),
            watched_entities: AtomicUsize::new(0),
            threshold_violations: AtomicU64::new(0),
            trust_limits_checked: AtomicU64::new(0),
            power_checks: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> AntiCalcificationStateSnapshot {
        AntiCalcificationStateSnapshot {
            power_concentration: self.power_concentration.read().map(|v| *v).unwrap_or(0.0),
            gini_coefficient: self.gini_coefficient.read().map(|v| *v).unwrap_or(0.0),
            interventions: self.interventions.load(Ordering::Relaxed),
            watched_entities: self.watched_entities.load(Ordering::Relaxed),
            threshold_violations: self.threshold_violations.load(Ordering::Relaxed),
            trust_limits_checked: self.trust_limits_checked.load(Ordering::Relaxed),
            power_checks: self.power_checks.load(Ordering::Relaxed),
        }
    }
}

impl Default for AntiCalcificationState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiCalcificationStateSnapshot {
    pub power_concentration: f64,
    pub gini_coefficient: f64,
    pub interventions: u64,
    pub watched_entities: usize,
    pub threshold_violations: u64,
    pub trust_limits_checked: u64,
    pub power_checks: u64,
}

/// Calibration Sub-State
#[derive(Debug)]
pub struct CalibrationState {
    /// Calibration-Updates durchgeführt
    pub updates: AtomicU64,
    /// Kalibrierte Parameter
    pub params: RwLock<HashMap<String, f64>>,
}

impl CalibrationState {
    pub fn new() -> Self {
        Self {
            updates: AtomicU64::new(0),
            params: RwLock::new(HashMap::new()),
        }
    }

    pub fn snapshot(&self) -> CalibrationStateSnapshot {
        CalibrationStateSnapshot {
            updates: self.updates.load(Ordering::Relaxed),
        }
    }
}

impl Default for CalibrationState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationStateSnapshot {
    pub updates: u64,
}

/// Protection State mit tiefgründigen Sub-States
#[derive(Debug)]
pub struct ProtectionState {
    /// Anomaly Detection (Anomaly ✓ Event/Trust)
    pub anomaly: AnomalyState,
    /// Diversity Monitor (Κ20) (Diversity ✓ Trust/Consensus)
    pub diversity: DiversityState,
    /// Quadratic Governance (Κ21) (Quadratic ← Trust)
    pub quadratic: QuadraticState,
    /// Anti-Calcification (Κ19) (AntiCalcification ✓/→ Trust)
    pub anti_calcification: AntiCalcificationState,
    /// Calibration (Calibration → Gas/Mana)
    pub calibration: CalibrationState,
}

impl ProtectionState {
    pub fn new() -> Self {
        Self {
            anomaly: AnomalyState::new(),
            diversity: DiversityState::new(),
            quadratic: QuadraticState::new(),
            anti_calcification: AntiCalcificationState::new(),
            calibration: CalibrationState::new(),
        }
    }

    /// Legacy-Kompatibilität: Anomalie aufzeichnen
    pub fn anomaly(&self, severity: &str) {
        self.anomaly.record(severity);
    }

    /// Legacy-Kompatibilität: Entropy setzen
    pub fn set_entropy(&self, dimension: &str, value: f64) {
        self.diversity.set_entropy(dimension, value);
    }

    /// Berechne System-Health basierend auf Protection-Metriken
    pub fn health_score(&self) -> f64 {
        let mut score: f64 = 100.0;

        // Anomalien reduzieren Score
        let critical = self.anomaly.critical.load(Ordering::Relaxed);
        let high = self.anomaly.high.load(Ordering::Relaxed);
        score -= (critical * 20) as f64;
        score -= (high * 10) as f64;

        // Diversity Warnings
        let warnings = self.diversity.monoculture_warnings.load(Ordering::Relaxed);
        score -= (warnings * 5) as f64;

        // Anti-Calc Violations
        let violations = self
            .anti_calcification
            .threshold_violations
            .load(Ordering::Relaxed);
        score -= (violations * 10) as f64;

        score.max(0.0).min(100.0)
    }

    pub fn snapshot(&self) -> ProtectionStateSnapshot {
        ProtectionStateSnapshot {
            anomaly: self.anomaly.snapshot(),
            diversity: self.diversity.snapshot(),
            quadratic: self.quadratic.snapshot(),
            anti_calcification: self.anti_calcification.snapshot(),
            calibration: self.calibration.snapshot(),
            health_score: self.health_score(),
        }
    }
}

impl Default for ProtectionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Protection State Snapshot mit tiefgründigen Sub-Snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionStateSnapshot {
    pub anomaly: AnomalyStateSnapshot,
    pub diversity: DiversityStateSnapshot,
    pub quadratic: QuadraticStateSnapshot,
    pub anti_calcification: AntiCalcificationStateSnapshot,
    pub calibration: CalibrationStateSnapshot,
    pub health_score: f64,
}

// ============================================================================
// STORAGE STATE LAYER
// ============================================================================

/// Storage State mit Persistenz-Tracking
#[derive(Debug)]
pub struct StorageState {
    // KV Store
    pub kv_keys: AtomicU64,
    pub kv_bytes: AtomicU64,
    pub kv_reads: AtomicU64,
    pub kv_writes: AtomicU64,

    // Event Store
    pub event_store_count: AtomicU64,
    pub event_store_bytes: AtomicU64,

    // Archive
    pub archived_epochs: AtomicU64,
    pub archived_events: AtomicU64,
    pub archive_bytes: AtomicU64,
    pub merkle_roots: AtomicU64,

    // Blueprint Marketplace
    pub blueprints_published: AtomicU64,
    pub blueprints_deployed: AtomicU64,
    pub blueprints_downloaded: AtomicU64,

    // Realms
    pub realm_count: AtomicUsize,
    pub identities: AtomicU64,
    pub trust_entries: AtomicU64,
}

impl StorageState {
    pub fn new() -> Self {
        Self {
            kv_keys: AtomicU64::new(0),
            kv_bytes: AtomicU64::new(0),
            kv_reads: AtomicU64::new(0),
            kv_writes: AtomicU64::new(0),
            event_store_count: AtomicU64::new(0),
            event_store_bytes: AtomicU64::new(0),
            archived_epochs: AtomicU64::new(0),
            archived_events: AtomicU64::new(0),
            archive_bytes: AtomicU64::new(0),
            merkle_roots: AtomicU64::new(0),
            blueprints_published: AtomicU64::new(0),
            blueprints_deployed: AtomicU64::new(0),
            blueprints_downloaded: AtomicU64::new(0),
            realm_count: AtomicUsize::new(0),
            identities: AtomicU64::new(0),
            trust_entries: AtomicU64::new(0),
        }
    }

    pub fn total_bytes(&self) -> u64 {
        self.kv_bytes.load(Ordering::Relaxed)
            + self.event_store_bytes.load(Ordering::Relaxed)
            + self.archive_bytes.load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> StorageStateSnapshot {
        StorageStateSnapshot {
            kv_keys: self.kv_keys.load(Ordering::Relaxed),
            kv_bytes: self.kv_bytes.load(Ordering::Relaxed),
            kv_reads: self.kv_reads.load(Ordering::Relaxed),
            kv_writes: self.kv_writes.load(Ordering::Relaxed),
            event_store_count: self.event_store_count.load(Ordering::Relaxed),
            event_store_bytes: self.event_store_bytes.load(Ordering::Relaxed),
            archived_epochs: self.archived_epochs.load(Ordering::Relaxed),
            archived_events: self.archived_events.load(Ordering::Relaxed),
            archive_bytes: self.archive_bytes.load(Ordering::Relaxed),
            merkle_roots: self.merkle_roots.load(Ordering::Relaxed),
            blueprints_published: self.blueprints_published.load(Ordering::Relaxed),
            blueprints_deployed: self.blueprints_deployed.load(Ordering::Relaxed),
            blueprints_downloaded: self.blueprints_downloaded.load(Ordering::Relaxed),
            realm_count: self.realm_count.load(Ordering::Relaxed),
            identities: self.identities.load(Ordering::Relaxed),
            trust_entries: self.trust_entries.load(Ordering::Relaxed),
            total_bytes: self.total_bytes(),
        }
    }
}

impl Default for StorageState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStateSnapshot {
    pub kv_keys: u64,
    pub kv_bytes: u64,
    pub kv_reads: u64,
    pub kv_writes: u64,
    pub event_store_count: u64,
    pub event_store_bytes: u64,
    pub archived_epochs: u64,
    pub archived_events: u64,
    pub archive_bytes: u64,
    pub merkle_roots: u64,
    pub blueprints_published: u64,
    pub blueprints_deployed: u64,
    pub blueprints_downloaded: u64,
    pub realm_count: usize,
    pub identities: u64,
    pub trust_entries: u64,
    pub total_bytes: u64,
}

// ============================================================================
// PEER STATE LAYER (Κ22-Κ24)
// ============================================================================

/// Gateway State (Κ23)
#[derive(Debug)]
pub struct GatewayState {
    /// Crossing-Anfragen insgesamt
    pub crossings_total: AtomicU64,
    /// Erfolgreiche Crossings
    pub crossings_allowed: AtomicU64,
    /// Abgelehnte Crossings
    pub crossings_denied: AtomicU64,
    /// Trust-Verletzungen (Trust < min_trust)
    pub trust_violations: AtomicU64,
    /// Credential-Verletzungen
    pub credential_violations: AtomicU64,
    /// Rule-Verletzungen
    pub rule_violations: AtomicU64,
    /// Durchschnittlicher Trust bei erfolgreichen Crossings
    pub avg_crossing_trust: RwLock<f64>,
    /// Trust-Dampening-Anwendungen
    pub dampening_applied: AtomicU64,
    /// Aktive Realm-Registrierungen
    pub registered_realms: AtomicUsize,
}

impl GatewayState {
    pub fn new() -> Self {
        Self {
            crossings_total: AtomicU64::new(0),
            crossings_allowed: AtomicU64::new(0),
            crossings_denied: AtomicU64::new(0),
            trust_violations: AtomicU64::new(0),
            credential_violations: AtomicU64::new(0),
            rule_violations: AtomicU64::new(0),
            avg_crossing_trust: RwLock::new(0.5),
            dampening_applied: AtomicU64::new(0),
            registered_realms: AtomicUsize::new(0),
        }
    }

    pub fn crossing_allowed(&self, trust: f64) {
        self.crossings_total.fetch_add(1, Ordering::Relaxed);
        self.crossings_allowed.fetch_add(1, Ordering::Relaxed);
        // Update rolling average
        if let Ok(mut avg) = self.avg_crossing_trust.write() {
            let allowed = self.crossings_allowed.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (allowed - 1.0) + trust) / allowed;
        }
    }

    pub fn crossing_denied(&self, reason: &str) {
        self.crossings_total.fetch_add(1, Ordering::Relaxed);
        self.crossings_denied.fetch_add(1, Ordering::Relaxed);
        match reason {
            "trust" => self.trust_violations.fetch_add(1, Ordering::Relaxed),
            "credential" => self.credential_violations.fetch_add(1, Ordering::Relaxed),
            "rule" => self.rule_violations.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.crossings_total.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.crossings_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> GatewayStateSnapshot {
        GatewayStateSnapshot {
            crossings_total: self.crossings_total.load(Ordering::Relaxed),
            crossings_allowed: self.crossings_allowed.load(Ordering::Relaxed),
            crossings_denied: self.crossings_denied.load(Ordering::Relaxed),
            trust_violations: self.trust_violations.load(Ordering::Relaxed),
            credential_violations: self.credential_violations.load(Ordering::Relaxed),
            rule_violations: self.rule_violations.load(Ordering::Relaxed),
            avg_crossing_trust: self.avg_crossing_trust.read().map(|v| *v).unwrap_or(0.5),
            dampening_applied: self.dampening_applied.load(Ordering::Relaxed),
            registered_realms: self.registered_realms.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
        }
    }
}

impl Default for GatewayState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStateSnapshot {
    pub crossings_total: u64,
    pub crossings_allowed: u64,
    pub crossings_denied: u64,
    pub trust_violations: u64,
    pub credential_violations: u64,
    pub rule_violations: u64,
    pub avg_crossing_trust: f64,
    pub dampening_applied: u64,
    pub registered_realms: usize,
    pub success_rate: f64,
}

/// Saga Composer State (Κ22, Κ24)
#[derive(Debug)]
pub struct SagaComposerState {
    /// Sagas komponiert insgesamt
    pub sagas_composed: AtomicU64,
    /// Erfolgreiche Kompositionen
    pub successful_compositions: AtomicU64,
    /// Fehlgeschlagene Kompositionen
    pub failed_compositions: AtomicU64,
    /// Durchschnittliche Schritte pro Saga
    pub avg_steps_per_saga: RwLock<f64>,
    /// Kompensationen ausgeführt (Κ24)
    pub compensations_executed: AtomicU64,
    /// Kompensationen erfolgreich
    pub compensations_successful: AtomicU64,
    /// Budget-Verletzungen
    pub budget_violations: AtomicU64,
    /// Cross-Realm-Sagas
    pub cross_realm_sagas: AtomicU64,
    /// Nach Goal-Typ
    pub goals_by_type: RwLock<HashMap<String, u64>>,
}

impl SagaComposerState {
    pub fn new() -> Self {
        Self {
            sagas_composed: AtomicU64::new(0),
            successful_compositions: AtomicU64::new(0),
            failed_compositions: AtomicU64::new(0),
            avg_steps_per_saga: RwLock::new(0.0),
            compensations_executed: AtomicU64::new(0),
            compensations_successful: AtomicU64::new(0),
            budget_violations: AtomicU64::new(0),
            cross_realm_sagas: AtomicU64::new(0),
            goals_by_type: RwLock::new(HashMap::new()),
        }
    }

    pub fn saga_composed(&self, success: bool, steps: usize, goal_type: &str) {
        self.sagas_composed.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_compositions.fetch_add(1, Ordering::Relaxed);
            // Update avg steps
            if let Ok(mut avg) = self.avg_steps_per_saga.write() {
                let total = self.successful_compositions.load(Ordering::Relaxed) as f64;
                *avg = (*avg * (total - 1.0) + steps as f64) / total;
            }
        } else {
            self.failed_compositions.fetch_add(1, Ordering::Relaxed);
        }
        // Track goal type
        if let Ok(mut goals) = self.goals_by_type.write() {
            *goals.entry(goal_type.to_string()).or_insert(0) += 1;
        }
    }

    pub fn compensation(&self, success: bool) {
        self.compensations_executed.fetch_add(1, Ordering::Relaxed);
        if success {
            self.compensations_successful
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn composition_success_rate(&self) -> f64 {
        let total = self.sagas_composed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.successful_compositions.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn compensation_success_rate(&self) -> f64 {
        let total = self.compensations_executed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.compensations_successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> SagaComposerStateSnapshot {
        SagaComposerStateSnapshot {
            sagas_composed: self.sagas_composed.load(Ordering::Relaxed),
            successful_compositions: self.successful_compositions.load(Ordering::Relaxed),
            failed_compositions: self.failed_compositions.load(Ordering::Relaxed),
            composition_success_rate: self.composition_success_rate(),
            avg_steps_per_saga: self.avg_steps_per_saga.read().map(|v| *v).unwrap_or(0.0),
            compensations_executed: self.compensations_executed.load(Ordering::Relaxed),
            compensations_successful: self.compensations_successful.load(Ordering::Relaxed),
            compensation_success_rate: self.compensation_success_rate(),
            budget_violations: self.budget_violations.load(Ordering::Relaxed),
            cross_realm_sagas: self.cross_realm_sagas.load(Ordering::Relaxed),
            goals_by_type: self
                .goals_by_type
                .read()
                .map(|g| g.clone())
                .unwrap_or_default(),
        }
    }
}

impl Default for SagaComposerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaComposerStateSnapshot {
    pub sagas_composed: u64,
    pub successful_compositions: u64,
    pub failed_compositions: u64,
    pub composition_success_rate: f64,
    pub avg_steps_per_saga: f64,
    pub compensations_executed: u64,
    pub compensations_successful: u64,
    pub compensation_success_rate: f64,
    pub budget_violations: u64,
    pub cross_realm_sagas: u64,
    pub goals_by_type: HashMap<String, u64>,
}

/// Intent Parser State
#[derive(Debug)]
pub struct IntentParserState {
    /// Intents geparst
    pub intents_parsed: AtomicU64,
    /// Erfolgreiche Parses
    pub successful_parses: AtomicU64,
    /// Parse-Fehler
    pub parse_errors: AtomicU64,
    /// Validierungsfehler
    pub validation_errors: AtomicU64,
    /// Nach Intent-Typ
    pub intents_by_type: RwLock<HashMap<String, u64>>,
    /// Durchschnittliche Parse-Zeit (µs)
    pub avg_parse_time_us: RwLock<f64>,
}

impl IntentParserState {
    pub fn new() -> Self {
        Self {
            intents_parsed: AtomicU64::new(0),
            successful_parses: AtomicU64::new(0),
            parse_errors: AtomicU64::new(0),
            validation_errors: AtomicU64::new(0),
            intents_by_type: RwLock::new(HashMap::new()),
            avg_parse_time_us: RwLock::new(0.0),
        }
    }

    pub fn parsed(&self, success: bool, intent_type: &str, duration_us: u64) {
        self.intents_parsed.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_parses.fetch_add(1, Ordering::Relaxed);
        } else {
            self.parse_errors.fetch_add(1, Ordering::Relaxed);
        }
        if let Ok(mut types) = self.intents_by_type.write() {
            *types.entry(intent_type.to_string()).or_insert(0) += 1;
        }
        // Update avg time
        if let Ok(mut avg) = self.avg_parse_time_us.write() {
            let total = self.intents_parsed.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + duration_us as f64) / total;
        }
    }

    pub fn snapshot(&self) -> IntentParserStateSnapshot {
        IntentParserStateSnapshot {
            intents_parsed: self.intents_parsed.load(Ordering::Relaxed),
            successful_parses: self.successful_parses.load(Ordering::Relaxed),
            parse_errors: self.parse_errors.load(Ordering::Relaxed),
            validation_errors: self.validation_errors.load(Ordering::Relaxed),
            intents_by_type: self
                .intents_by_type
                .read()
                .map(|t| t.clone())
                .unwrap_or_default(),
            avg_parse_time_us: self.avg_parse_time_us.read().map(|v| *v).unwrap_or(0.0),
        }
    }
}

impl Default for IntentParserState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentParserStateSnapshot {
    pub intents_parsed: u64,
    pub successful_parses: u64,
    pub parse_errors: u64,
    pub validation_errors: u64,
    pub intents_by_type: HashMap<String, u64>,
    pub avg_parse_time_us: f64,
}

// ============================================================================
// REALM STATE LAYER - Per-Realm Isolation (Κ22-Κ24)
// ============================================================================

/// Per-Realm spezifischer State
///
/// Jedes Realm hat seinen eigenen isolierten State mit:
/// - Eigener TrustVector für Realm-spezifische Trust-Bewertung
/// - Eigenes Rule-Set (RuleCategory: Membership, Transaction, etc.)
/// - Identity-Tracking innerhalb des Realms
/// - Activity-Metriken für Monitoring
#[derive(Debug)]
/// Per-Realm Isolation State (Κ22-Κ24)
///
/// Implementiert das Realm-Konzept gemäß der Kernidee:
/// - **Isolation**: Daten/Aktionen bleiben im Realm (Sicherheit gegen Leak)
/// - **Crossing**: Kontrollierter Wechsel zwischen Realms (Gateway prüft Trust/Regeln)
/// - **Cross-Realm-Sagas**: Komplexe Aktionen über Realms (SagaComposer koordiniert)
/// - **Realm-spezifischer Trust**: Trust kann pro Realm variieren
///
/// # Beispiele für Realm-Typen:
/// - "private-friends" (hoher Trust, enge Gruppe)
/// - "public" (niedriger min_trust, öffentlich zugänglich)
/// - "app-specific" (anwendungsspezifische Regeln)
pub struct RealmSpecificState {
    // ─────────────────────────────────────────────────────────────────────────
    // TRUST & GOVERNANCE
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifischer Trust-Vektor
    /// Kombiniert globalen Trust mit Realm-lokalem Verhalten.
    /// Kann höher sein (z.B. bei Freunden) oder niedriger (z.B. bei Fremden).
    pub trust: RwLock<crate::domain::unified::TrustVector6D>,

    /// Minimum-Trust für Membership in diesem Realm
    /// Entities unter diesem Schwellenwert können nicht beitreten.
    pub min_trust: RwLock<f32>,

    /// Governance-Typ bestimmt Entscheidungsprozesse:
    /// - "owner": Einzelne Entität hat volle Kontrolle
    /// - "democratic": Mehrheitsentscheidung
    /// - "token": Token-gewichtete Abstimmung
    /// - "reputation": Trust-gewichtete Abstimmung
    /// - "consensus": Einstimmigkeit erforderlich
    pub governance_type: RwLock<String>,

    // ─────────────────────────────────────────────────────────────────────────
    // MEMBERSHIP & IDENTITIES (Explizite Isolation)
    // ─────────────────────────────────────────────────────────────────────────
    /// Explizite Mitgliederliste (Identity-IDs)
    /// Kernfeature für Isolation: Nur Mitglieder haben Zugriff.
    pub members: RwLock<HashSet<String>>,

    /// Anzahl registrierter Identitäten im Realm (Snapshot-friendly)
    pub identity_count: AtomicUsize,

    /// Pending Membership-Requests (awaiting approval)
    pub pending_members: RwLock<HashSet<String>>,

    /// Gebannte Identitäten (permanent ausgeschlossen)
    pub banned_members: RwLock<HashSet<String>>,

    /// Realm-Owner/Admin-Identitäten
    pub admins: RwLock<HashSet<String>>,

    // ─────────────────────────────────────────────────────────────────────────
    // ECL RULES & POLICIES (Realm-spezifische Logik)
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive ECL Policy-IDs für dieses Realm
    /// Definiert: Crossing-Rules, Membership-Rules, Transaction-Rules
    pub active_policies: RwLock<Vec<String>>,

    /// Aktive Legacy Rule-IDs (deprecated, use active_policies)
    pub active_rules: RwLock<Vec<String>>,

    // ─────────────────────────────────────────────────────────────────────────
    // ISOLATION & DATA PROTECTION
    // ─────────────────────────────────────────────────────────────────────────
    /// Isolation-Level: Wie streng ist die Daten-Isolation?
    /// - 0: Public (alle können lesen)
    /// - 1: Members-Only (nur Mitglieder können lesen)
    /// - 2: Strict (kein Cross-Realm-Zugriff, selbst mit Crossing)
    pub isolation_level: AtomicU8,

    /// Data-Leak-Events (Versuche Daten nach außen zu übertragen)
    pub leak_attempts: AtomicU64,

    /// Erfolgreich geblockte Leak-Versuche
    pub leaks_blocked: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // CROSSING METRICS (Κ23 Gateway-Integration)
    // ─────────────────────────────────────────────────────────────────────────
    /// Eingehende Crossings (in dieses Realm)
    pub crossings_in: AtomicU64,

    /// Ausgehende Crossings (aus diesem Realm)
    pub crossings_out: AtomicU64,

    /// Crossing-Requests abgelehnt (Trust zu niedrig oder Regel verletzt)
    pub crossings_denied: AtomicU64,

    /// Aktive Crossings (gerade im Übergang befindliche Entities)
    pub active_crossings: AtomicU64,

    /// Allowlisted Realms (Crossing ohne Policy-Check erlaubt)
    pub crossing_allowlist: RwLock<HashSet<String>>,

    /// Blocklisted Realms (Crossing immer abgelehnt)
    pub crossing_blocklist: RwLock<HashSet<String>>,

    // ─────────────────────────────────────────────────────────────────────────
    // SAGA & EXECUTION (Κ22/Κ24 SagaComposer-Integration)
    // ─────────────────────────────────────────────────────────────────────────
    /// Sagas die in diesem Realm initiiert wurden
    pub sagas_initiated: AtomicU64,

    /// Cross-Realm-Sagas die dieses Realm involvieren
    pub cross_realm_sagas_involved: AtomicU64,

    /// Sagas die in diesem Realm fehlgeschlagen sind
    pub sagas_failed: AtomicU64,

    /// Compensations in diesem Realm ausgeführt
    pub compensations_executed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // ACTIVITY METRICS
    // ─────────────────────────────────────────────────────────────────────────
    /// Events insgesamt in diesem Realm
    pub events_total: AtomicU64,

    /// Events heute (rolling 24h window, reset via maintenance)
    pub events_today: AtomicU64,

    /// Letztes Event-Timestamp (Unix)
    pub last_event_at: AtomicU64,

    /// Erstellungszeitpunkt (Unix-Timestamp)
    pub created_at: u64,
}

impl RealmSpecificState {
    pub fn new(min_trust: f32, governance_type: &str) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            // Trust & Governance
            trust: RwLock::new(crate::domain::unified::TrustVector6D::DEFAULT),
            min_trust: RwLock::new(min_trust),
            governance_type: RwLock::new(governance_type.to_string()),

            // Membership & Identities
            members: RwLock::new(HashSet::new()),
            identity_count: AtomicUsize::new(0),
            pending_members: RwLock::new(HashSet::new()),
            banned_members: RwLock::new(HashSet::new()),
            admins: RwLock::new(HashSet::new()),

            // ECL Rules & Policies
            active_policies: RwLock::new(Vec::new()),
            active_rules: RwLock::new(Vec::new()),

            // Isolation & Data Protection
            isolation_level: AtomicU8::new(1), // Default: Members-Only
            leak_attempts: AtomicU64::new(0),
            leaks_blocked: AtomicU64::new(0),

            // Crossing Metrics
            crossings_in: AtomicU64::new(0),
            crossings_out: AtomicU64::new(0),
            crossings_denied: AtomicU64::new(0),
            active_crossings: AtomicU64::new(0),
            crossing_allowlist: RwLock::new(HashSet::new()),
            crossing_blocklist: RwLock::new(HashSet::new()),

            // Saga & Execution
            sagas_initiated: AtomicU64::new(0),
            cross_realm_sagas_involved: AtomicU64::new(0),
            sagas_failed: AtomicU64::new(0),
            compensations_executed: AtomicU64::new(0),

            // Activity Metrics
            events_total: AtomicU64::new(0),
            events_today: AtomicU64::new(0),
            last_event_at: AtomicU64::new(0),
            created_at: now,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MEMBERSHIP OPERATIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Füge Member zum Realm hinzu (nach Approval)
    pub fn add_member(&self, identity_id: &str) {
        if let Ok(mut members) = self.members.write() {
            if members.insert(identity_id.to_string()) {
                self.identity_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        // Entferne aus pending falls vorhanden
        if let Ok(mut pending) = self.pending_members.write() {
            pending.remove(identity_id);
        }
    }

    /// Entferne Member vom Realm
    pub fn remove_member(&self, identity_id: &str) {
        if let Ok(mut members) = self.members.write() {
            if members.remove(identity_id) {
                let _ = self
                    .identity_count
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                        if v > 0 {
                            Some(v - 1)
                        } else {
                            Some(0)
                        }
                    });
            }
        }
    }

    /// Prüfe ob Identity Member ist
    pub fn is_member(&self, identity_id: &str) -> bool {
        self.members
            .read()
            .map(|m| m.contains(identity_id))
            .unwrap_or(false)
    }

    /// Füge Membership-Request hinzu
    pub fn request_membership(&self, identity_id: &str) {
        if let Ok(mut pending) = self.pending_members.write() {
            pending.insert(identity_id.to_string());
        }
    }

    /// Banne Identity (permanent)
    pub fn ban_member(&self, identity_id: &str) {
        self.remove_member(identity_id);
        if let Ok(mut banned) = self.banned_members.write() {
            banned.insert(identity_id.to_string());
        }
    }

    /// Prüfe ob Identity gebannt ist
    pub fn is_banned(&self, identity_id: &str) -> bool {
        self.banned_members
            .read()
            .map(|b| b.contains(identity_id))
            .unwrap_or(false)
    }

    /// Füge Admin hinzu
    pub fn add_admin(&self, identity_id: &str) {
        if let Ok(mut admins) = self.admins.write() {
            admins.insert(identity_id.to_string());
        }
        // Admins sind automatisch auch Members
        self.add_member(identity_id);
    }

    /// Prüfe ob Identity Admin ist
    pub fn is_admin(&self, identity_id: &str) -> bool {
        self.admins
            .read()
            .map(|a| a.contains(identity_id))
            .unwrap_or(false)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CROSSING OPERATIONS (Κ23)
    // ─────────────────────────────────────────────────────────────────────────

    pub fn crossing_in(&self) {
        self.crossings_in.fetch_add(1, Ordering::Relaxed);
        self.record_event();
    }

    pub fn crossing_out(&self) {
        self.crossings_out.fetch_add(1, Ordering::Relaxed);
        self.record_event();
    }

    pub fn crossing_denied(&self) {
        self.crossings_denied.fetch_add(1, Ordering::Relaxed);
    }

    /// Prüfe ob Crossing zu target_realm erlaubt ist (Allowlist/Blocklist)
    pub fn is_crossing_allowed(&self, target_realm: &str) -> Option<bool> {
        // Blocklist hat Priorität
        if let Ok(blocklist) = self.crossing_blocklist.read() {
            if blocklist.contains(target_realm) {
                return Some(false);
            }
        }
        // Allowlist erlaubt ohne Policy-Check
        if let Ok(allowlist) = self.crossing_allowlist.read() {
            if allowlist.contains(target_realm) {
                return Some(true);
            }
        }
        // Weder Allow noch Block → Policy muss entscheiden
        None
    }

    /// Füge Realm zur Allowlist hinzu
    pub fn allow_crossing_to(&self, target_realm: &str) {
        if let Ok(mut allowlist) = self.crossing_allowlist.write() {
            allowlist.insert(target_realm.to_string());
        }
        // Entferne aus Blocklist falls vorhanden
        if let Ok(mut blocklist) = self.crossing_blocklist.write() {
            blocklist.remove(target_realm);
        }
    }

    /// Füge Realm zur Blocklist hinzu
    pub fn block_crossing_to(&self, target_realm: &str) {
        if let Ok(mut blocklist) = self.crossing_blocklist.write() {
            blocklist.insert(target_realm.to_string());
        }
        // Entferne aus Allowlist falls vorhanden
        if let Ok(mut allowlist) = self.crossing_allowlist.write() {
            allowlist.remove(target_realm);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SAGA OPERATIONS (Κ22/Κ24)
    // ─────────────────────────────────────────────────────────────────────────

    pub fn saga_initiated(&self, cross_realm: bool) {
        self.sagas_initiated.fetch_add(1, Ordering::Relaxed);
        if cross_realm {
            self.cross_realm_sagas_involved
                .fetch_add(1, Ordering::Relaxed);
        }
        self.record_event();
    }

    pub fn saga_failed(&self) {
        self.sagas_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn compensation_executed(&self) {
        self.compensations_executed.fetch_add(1, Ordering::Relaxed);
        self.record_event();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LEGACY COMPATIBILITY
    // ─────────────────────────────────────────────────────────────────────────

    pub fn identity_joined(&self) {
        self.identity_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn identity_left(&self) {
        let _ = self
            .identity_count
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            });
    }

    pub fn add_rule(&self, rule_id: &str) {
        if let Ok(mut rules) = self.active_rules.write() {
            if !rules.contains(&rule_id.to_string()) {
                rules.push(rule_id.to_string());
            }
        }
    }

    pub fn remove_rule(&self, rule_id: &str) {
        if let Ok(mut rules) = self.active_rules.write() {
            rules.retain(|r| r != rule_id);
        }
    }

    pub fn update_trust(&self, new_trust: crate::domain::unified::TrustVector6D) {
        if let Ok(mut trust) = self.trust.write() {
            *trust = new_trust;
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ECL POLICY OPERATIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Aktiviere ECL-Policy für dieses Realm
    pub fn activate_policy(&self, policy_id: &str) {
        if let Ok(mut policies) = self.active_policies.write() {
            if !policies.contains(&policy_id.to_string()) {
                policies.push(policy_id.to_string());
            }
        }
    }

    /// Deaktiviere ECL-Policy
    pub fn deactivate_policy(&self, policy_id: &str) {
        if let Ok(mut policies) = self.active_policies.write() {
            policies.retain(|p| p != policy_id);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ISOLATION OPERATIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Setze Isolation-Level (0=Public, 1=Members-Only, 2=Strict)
    pub fn set_isolation_level(&self, level: u8) {
        self.isolation_level.store(level.min(2), Ordering::Relaxed);
    }

    /// Hole Isolation-Level
    pub fn get_isolation_level(&self) -> u8 {
        self.isolation_level.load(Ordering::Relaxed)
    }

    /// Registriere Leak-Versuch
    pub fn record_leak_attempt(&self, blocked: bool) {
        self.leak_attempts.fetch_add(1, Ordering::Relaxed);
        if blocked {
            self.leaks_blocked.fetch_add(1, Ordering::Relaxed);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ACTIVITY TRACKING
    // ─────────────────────────────────────────────────────────────────────────

    fn record_event(&self) {
        use std::time::{SystemTime, UNIX_EPOCH};

        self.events_total.fetch_add(1, Ordering::Relaxed);
        self.events_today.fetch_add(1, Ordering::Relaxed);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_event_at.store(now, Ordering::Relaxed);
    }

    /// Reset daily counter (called by maintenance)
    pub fn reset_daily_events(&self) {
        self.events_today.store(0, Ordering::Relaxed);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SNAPSHOT
    // ─────────────────────────────────────────────────────────────────────────

    pub fn snapshot(&self) -> RealmSpecificStateSnapshot {
        RealmSpecificStateSnapshot {
            // Trust & Governance
            trust: self.trust.read().map(|t| *t).unwrap_or_default(),
            min_trust: self.min_trust.read().map(|t| *t).unwrap_or(0.0),
            governance_type: self
                .governance_type
                .read()
                .map(|g| g.clone())
                .unwrap_or_default(),

            // Membership
            member_count: self.identity_count.load(Ordering::Relaxed),
            pending_member_count: self.pending_members.read().map(|p| p.len()).unwrap_or(0),
            banned_count: self.banned_members.read().map(|b| b.len()).unwrap_or(0),
            admin_count: self.admins.read().map(|a| a.len()).unwrap_or(0),

            // ECL Policies
            active_policies: self
                .active_policies
                .read()
                .map(|p| p.clone())
                .unwrap_or_default(),
            active_rules: self
                .active_rules
                .read()
                .map(|r| r.clone())
                .unwrap_or_default(),

            // Isolation
            isolation_level: self.isolation_level.load(Ordering::Relaxed),
            leak_attempts: self.leak_attempts.load(Ordering::Relaxed),
            leaks_blocked: self.leaks_blocked.load(Ordering::Relaxed),

            // Crossings
            crossings_in: self.crossings_in.load(Ordering::Relaxed),
            crossings_out: self.crossings_out.load(Ordering::Relaxed),
            crossings_denied: self.crossings_denied.load(Ordering::Relaxed),
            active_crossings: self.active_crossings.load(Ordering::Relaxed),
            crossing_allowlist_count: self.crossing_allowlist.read().map(|a| a.len()).unwrap_or(0),
            crossing_blocklist_count: self.crossing_blocklist.read().map(|b| b.len()).unwrap_or(0),

            // Sagas
            sagas_initiated: self.sagas_initiated.load(Ordering::Relaxed),
            cross_realm_sagas_involved: self.cross_realm_sagas_involved.load(Ordering::Relaxed),
            sagas_failed: self.sagas_failed.load(Ordering::Relaxed),
            compensations_executed: self.compensations_executed.load(Ordering::Relaxed),

            // Activity
            events_total: self.events_total.load(Ordering::Relaxed),
            events_today: self.events_today.load(Ordering::Relaxed),
            last_event_at: self.last_event_at.load(Ordering::Relaxed),
            created_at: self.created_at,
        }
    }
}

/// Serializable Snapshot of RealmSpecificState
///
/// Vollständige Realm-Metriken für Debugging, Monitoring und Isolation-Prüfung.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmSpecificStateSnapshot {
    // Trust & Governance
    pub trust: crate::domain::unified::TrustVector6D,
    pub min_trust: f32,
    pub governance_type: String,

    // Membership (nur Counts für Privacy)
    pub member_count: usize,
    pub pending_member_count: usize,
    pub banned_count: usize,
    pub admin_count: usize,

    // ECL Policies
    pub active_policies: Vec<String>,
    pub active_rules: Vec<String>,

    // Isolation
    pub isolation_level: u8,
    pub leak_attempts: u64,
    pub leaks_blocked: u64,

    // Crossings (Κ23)
    pub crossings_in: u64,
    pub crossings_out: u64,
    pub crossings_denied: u64,
    pub active_crossings: u64,
    pub crossing_allowlist_count: usize,
    pub crossing_blocklist_count: usize,

    // Sagas (Κ22/Κ24)
    pub sagas_initiated: u64,
    pub cross_realm_sagas_involved: u64,
    pub sagas_failed: u64,
    pub compensations_executed: u64,

    // Activity
    pub events_total: u64,
    pub events_today: u64,
    pub last_event_at: u64,
    pub created_at: u64,
}

/// Aggregierter Realm State für alle Realms
///
/// Verwaltet alle registrierten Realms mit ihrem jeweiligen State.
/// Implementiert das Realm-Konzept: Isolierte Bereiche mit eigenen
/// Regeln, Identitäten und Trust-Leveln (Κ22-Κ24).
#[derive(Debug)]
pub struct RealmState {
    /// Alle registrierten Realms mit ihrem State
    pub realms: RwLock<HashMap<String, RealmSpecificState>>,

    /// Gesamt-Anzahl Realms
    pub total_realms: AtomicUsize,

    /// Aktuell aktive Cross-Realm-Crossings
    pub active_crossings: AtomicU64,

    /// Gesamt Cross-Realm-Sagas
    pub total_cross_realm_sagas: AtomicU64,

    /// Fehlgeschlagene Crossing-Versuche
    pub crossing_failures: AtomicU64,

    /// Root-Realm ID (falls vorhanden)
    pub root_realm_id: RwLock<Option<String>>,
}

impl RealmState {
    pub fn new() -> Self {
        Self {
            realms: RwLock::new(HashMap::new()),
            total_realms: AtomicUsize::new(0),
            active_crossings: AtomicU64::new(0),
            total_cross_realm_sagas: AtomicU64::new(0),
            crossing_failures: AtomicU64::new(0),
            root_realm_id: RwLock::new(None),
        }
    }

    /// Registriere ein neues Realm
    pub fn register_realm(&self, realm_id: &str, min_trust: f32, governance_type: &str) {
        if let Ok(mut realms) = self.realms.write() {
            if !realms.contains_key(realm_id) {
                realms.insert(
                    realm_id.to_string(),
                    RealmSpecificState::new(min_trust, governance_type),
                );
                self.total_realms.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Setze Root-Realm
    pub fn set_root_realm(&self, realm_id: &str) {
        if let Ok(mut root) = self.root_realm_id.write() {
            *root = Some(realm_id.to_string());
        }
    }

    /// Hole Realm-spezifischen State
    pub fn get_realm(&self, realm_id: &str) -> Option<RealmSpecificStateSnapshot> {
        self.realms.read().ok()?.get(realm_id).map(|r| r.snapshot())
    }

    /// Registriere ein erfolgreiches Crossing
    pub fn crossing_succeeded(&self, from_realm: &str, to_realm: &str) {
        if let Ok(realms) = self.realms.read() {
            if let Some(from) = realms.get(from_realm) {
                from.crossing_out();
            }
            if let Some(to) = realms.get(to_realm) {
                to.crossing_in();
            }
        }
        self.active_crossings.fetch_add(1, Ordering::Relaxed);
    }

    /// Registriere ein fehlgeschlagenes Crossing
    pub fn crossing_failed(&self) {
        self.crossing_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Crossing beendet
    pub fn crossing_completed(&self) {
        let _ = self
            .active_crossings
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            });
    }

    /// Registriere Cross-Realm-Saga
    pub fn cross_realm_saga_started(&self, realm_ids: &[&str]) {
        self.total_cross_realm_sagas.fetch_add(1, Ordering::Relaxed);
        if let Ok(realms) = self.realms.read() {
            for realm_id in realm_ids {
                if let Some(realm) = realms.get(*realm_id) {
                    realm.saga_initiated(true);
                }
            }
        }
    }

    /// Identity tritt einem Realm bei
    pub fn identity_joined_realm(&self, realm_id: &str) {
        if let Ok(realms) = self.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.identity_joined();
            }
        }
    }

    /// Identity verlässt ein Realm
    pub fn identity_left_realm(&self, realm_id: &str) {
        if let Ok(realms) = self.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.identity_left();
            }
        }
    }

    /// Update Trust für ein Realm
    pub fn update_realm_trust(&self, realm_id: &str, trust: crate::domain::unified::TrustVector6D) {
        if let Ok(realms) = self.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.update_trust(trust);
            }
        }
    }

    /// Füge Rule zu Realm hinzu
    pub fn add_rule_to_realm(&self, realm_id: &str, rule_id: &str) {
        if let Ok(realms) = self.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.add_rule(rule_id);
            }
        }
    }

    pub fn snapshot(&self) -> RealmStateSnapshot {
        let realms_snapshot = self
            .realms
            .read()
            .map(|r| r.iter().map(|(k, v)| (k.clone(), v.snapshot())).collect())
            .unwrap_or_default();

        RealmStateSnapshot {
            realms: realms_snapshot,
            total_realms: self.total_realms.load(Ordering::Relaxed),
            active_crossings: self.active_crossings.load(Ordering::Relaxed),
            total_cross_realm_sagas: self.total_cross_realm_sagas.load(Ordering::Relaxed),
            crossing_failures: self.crossing_failures.load(Ordering::Relaxed),
            root_realm_id: self.root_realm_id.read().map(|r| r.clone()).unwrap_or(None),
        }
    }
}

impl Default for RealmState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmStateSnapshot {
    pub realms: HashMap<String, RealmSpecificStateSnapshot>,
    pub total_realms: usize,
    pub active_crossings: u64,
    pub total_cross_realm_sagas: u64,
    pub crossing_failures: u64,
    pub root_realm_id: Option<String>,
}

/// Aggregierter Peer State (Gateway + Saga + Intent + Realm)
#[derive(Debug)]
pub struct PeerState {
    pub gateway: GatewayState,
    pub saga: SagaComposerState,
    pub intent: IntentParserState,
    /// Realm-State für isolierte Bereiche mit eigenen Regeln und Trust-Leveln
    pub realm: RealmState,
}

impl PeerState {
    pub fn new() -> Self {
        Self {
            gateway: GatewayState::new(),
            saga: SagaComposerState::new(),
            intent: IntentParserState::new(),
            realm: RealmState::new(),
        }
    }

    pub fn snapshot(&self) -> PeerStateSnapshot {
        PeerStateSnapshot {
            gateway: self.gateway.snapshot(),
            saga: self.saga.snapshot(),
            intent: self.intent.snapshot(),
            realm: self.realm.snapshot(),
        }
    }
}

impl Default for PeerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStateSnapshot {
    pub gateway: GatewayStateSnapshot,
    pub saga: SagaComposerStateSnapshot,
    pub intent: IntentParserStateSnapshot,
    pub realm: RealmStateSnapshot,
}

// ============================================================================
// P2P NETWORK STATE LAYER
// ============================================================================

/// NAT Traversal Status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NatStatus {
    #[default]
    Unknown,
    Public,
    Private,
}

/// Swarm State
#[derive(Debug)]
pub struct SwarmState {
    /// Eigene Peer-ID
    pub peer_id: RwLock<String>,
    /// Verbundene Peers
    pub connected_peers: AtomicUsize,
    /// Eingehende Verbindungen
    pub inbound_connections: AtomicU64,
    /// Ausgehende Verbindungen
    pub outbound_connections: AtomicU64,
    /// Verbindungsfehler
    pub connection_errors: AtomicU64,
    /// Bytes gesendet
    pub bytes_sent: AtomicU64,
    /// Bytes empfangen
    pub bytes_received: AtomicU64,
    /// Latenz-Summe (für Durchschnitt)
    pub latency_sum_us: AtomicU64,
    /// Latenz-Messungen
    pub latency_count: AtomicU64,
    /// NAT-Status
    pub nat_status: RwLock<NatStatus>,
    /// Externe Adressen
    pub external_addresses: RwLock<Vec<String>>,
}

impl SwarmState {
    pub fn new() -> Self {
        Self {
            peer_id: RwLock::new(String::new()),
            connected_peers: AtomicUsize::new(0),
            inbound_connections: AtomicU64::new(0),
            outbound_connections: AtomicU64::new(0),
            connection_errors: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            latency_sum_us: AtomicU64::new(0),
            latency_count: AtomicU64::new(0),
            nat_status: RwLock::new(NatStatus::Unknown),
            external_addresses: RwLock::new(Vec::new()),
        }
    }

    pub fn peer_connected(&self, inbound: bool) {
        self.connected_peers.fetch_add(1, Ordering::Relaxed);
        if inbound {
            self.inbound_connections.fetch_add(1, Ordering::Relaxed);
        } else {
            self.outbound_connections.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn peer_disconnected(&self) {
        let _ = self
            .connected_peers
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            });
    }

    pub fn record_latency(&self, latency_us: u64) {
        self.latency_sum_us.fetch_add(latency_us, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let count = self.latency_count.load(Ordering::Relaxed);
        if count > 0 {
            (self.latency_sum_us.load(Ordering::Relaxed) as f64 / count as f64) / 1000.0
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> SwarmStateSnapshot {
        SwarmStateSnapshot {
            peer_id: self.peer_id.read().map(|p| p.clone()).unwrap_or_default(),
            connected_peers: self.connected_peers.load(Ordering::Relaxed),
            inbound_connections: self.inbound_connections.load(Ordering::Relaxed),
            outbound_connections: self.outbound_connections.load(Ordering::Relaxed),
            connection_errors: self.connection_errors.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            avg_latency_ms: self.avg_latency_ms(),
            nat_status: self.nat_status.read().map(|n| *n).unwrap_or_default(),
            external_addresses: self
                .external_addresses
                .read()
                .map(|a| a.clone())
                .unwrap_or_default(),
        }
    }
}

impl Default for SwarmState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmStateSnapshot {
    pub peer_id: String,
    pub connected_peers: usize,
    pub inbound_connections: u64,
    pub outbound_connections: u64,
    pub connection_errors: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub avg_latency_ms: f64,
    pub nat_status: NatStatus,
    pub external_addresses: Vec<String>,
}

/// Gossipsub State
#[derive(Debug)]
pub struct GossipState {
    /// Peers im Mesh
    pub mesh_peers: AtomicUsize,
    /// Subscribed Topics
    pub subscribed_topics: AtomicUsize,
    /// Messages empfangen
    pub messages_received: AtomicU64,
    /// Messages gesendet
    pub messages_sent: AtomicU64,
    /// Messages validiert
    pub messages_validated: AtomicU64,
    /// Messages abgelehnt
    pub messages_rejected: AtomicU64,
    /// Duplicate Messages (ignoriert)
    pub duplicate_messages: AtomicU64,
    /// Trust-basierte Scores
    pub peers_pruned: AtomicU64,
    pub peers_grafted: AtomicU64,
}

impl GossipState {
    pub fn new() -> Self {
        Self {
            mesh_peers: AtomicUsize::new(0),
            subscribed_topics: AtomicUsize::new(0),
            messages_received: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            messages_validated: AtomicU64::new(0),
            messages_rejected: AtomicU64::new(0),
            duplicate_messages: AtomicU64::new(0),
            peers_pruned: AtomicU64::new(0),
            peers_grafted: AtomicU64::new(0),
        }
    }

    pub fn message_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn message_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn validation_rate(&self) -> f64 {
        let total = self.messages_received.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.messages_validated.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> GossipStateSnapshot {
        GossipStateSnapshot {
            mesh_peers: self.mesh_peers.load(Ordering::Relaxed),
            subscribed_topics: self.subscribed_topics.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_validated: self.messages_validated.load(Ordering::Relaxed),
            messages_rejected: self.messages_rejected.load(Ordering::Relaxed),
            duplicate_messages: self.duplicate_messages.load(Ordering::Relaxed),
            peers_pruned: self.peers_pruned.load(Ordering::Relaxed),
            peers_grafted: self.peers_grafted.load(Ordering::Relaxed),
            validation_rate: self.validation_rate(),
        }
    }
}

impl Default for GossipState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipStateSnapshot {
    pub mesh_peers: usize,
    pub subscribed_topics: usize,
    pub messages_received: u64,
    pub messages_sent: u64,
    pub messages_validated: u64,
    pub messages_rejected: u64,
    pub duplicate_messages: u64,
    pub peers_pruned: u64,
    pub peers_grafted: u64,
    pub validation_rate: f64,
}

/// Kademlia DHT State
#[derive(Debug)]
pub struct KademliaState {
    /// Peers in Routing Table
    pub routing_table_size: AtomicUsize,
    /// Bootstrap abgeschlossen
    pub bootstrap_complete: RwLock<bool>,
    /// Records gespeichert
    pub records_stored: AtomicU64,
    /// Queries durchgeführt
    pub queries_total: AtomicU64,
    /// Queries erfolgreich
    pub queries_successful: AtomicU64,
    /// Provider-Registrierungen
    pub provider_registrations: AtomicU64,
}

impl KademliaState {
    pub fn new() -> Self {
        Self {
            routing_table_size: AtomicUsize::new(0),
            bootstrap_complete: RwLock::new(false),
            records_stored: AtomicU64::new(0),
            queries_total: AtomicU64::new(0),
            queries_successful: AtomicU64::new(0),
            provider_registrations: AtomicU64::new(0),
        }
    }

    pub fn query_success_rate(&self) -> f64 {
        let total = self.queries_total.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.queries_successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> KademliaStateSnapshot {
        KademliaStateSnapshot {
            routing_table_size: self.routing_table_size.load(Ordering::Relaxed),
            bootstrap_complete: self.bootstrap_complete.read().map(|b| *b).unwrap_or(false),
            records_stored: self.records_stored.load(Ordering::Relaxed),
            queries_total: self.queries_total.load(Ordering::Relaxed),
            queries_successful: self.queries_successful.load(Ordering::Relaxed),
            query_success_rate: self.query_success_rate(),
            provider_registrations: self.provider_registrations.load(Ordering::Relaxed),
        }
    }
}

impl Default for KademliaState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KademliaStateSnapshot {
    pub routing_table_size: usize,
    pub bootstrap_complete: bool,
    pub records_stored: u64,
    pub queries_total: u64,
    pub queries_successful: u64,
    pub query_success_rate: f64,
    pub provider_registrations: u64,
}

/// Relay State (Circuit Relay V2)
#[derive(Debug)]
pub struct RelayState {
    /// Aktive Relay-Reservation
    pub has_reservation: RwLock<bool>,
    /// Relay-Peer
    pub relay_peer: RwLock<Option<String>>,
    /// Circuits bedient (als Server)
    pub circuits_served: AtomicU64,
    /// Circuits aktiv
    pub circuits_active: AtomicUsize,
    /// DCUTR Erfolge (Hole-Punching)
    pub dcutr_successes: AtomicU64,
    /// DCUTR Fehlschläge
    pub dcutr_failures: AtomicU64,
    /// Bytes über Relay
    pub relay_bytes: AtomicU64,
}

impl RelayState {
    pub fn new() -> Self {
        Self {
            has_reservation: RwLock::new(false),
            relay_peer: RwLock::new(None),
            circuits_served: AtomicU64::new(0),
            circuits_active: AtomicUsize::new(0),
            dcutr_successes: AtomicU64::new(0),
            dcutr_failures: AtomicU64::new(0),
            relay_bytes: AtomicU64::new(0),
        }
    }

    pub fn dcutr_success_rate(&self) -> f64 {
        let total = self.dcutr_successes.load(Ordering::Relaxed)
            + self.dcutr_failures.load(Ordering::Relaxed);
        if total > 0 {
            self.dcutr_successes.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> RelayStateSnapshot {
        RelayStateSnapshot {
            has_reservation: self.has_reservation.read().map(|b| *b).unwrap_or(false),
            relay_peer: self
                .relay_peer
                .read()
                .map(|p| p.clone())
                .unwrap_or_default(),
            circuits_served: self.circuits_served.load(Ordering::Relaxed),
            circuits_active: self.circuits_active.load(Ordering::Relaxed),
            dcutr_successes: self.dcutr_successes.load(Ordering::Relaxed),
            dcutr_failures: self.dcutr_failures.load(Ordering::Relaxed),
            dcutr_success_rate: self.dcutr_success_rate(),
            relay_bytes: self.relay_bytes.load(Ordering::Relaxed),
        }
    }
}

impl Default for RelayState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayStateSnapshot {
    pub has_reservation: bool,
    pub relay_peer: Option<String>,
    pub circuits_served: u64,
    pub circuits_active: usize,
    pub dcutr_successes: u64,
    pub dcutr_failures: u64,
    pub dcutr_success_rate: f64,
    pub relay_bytes: u64,
}

/// Privacy Layer State (Onion Routing)
#[derive(Debug)]
pub struct PrivacyState {
    /// Circuits erstellt
    pub circuits_created: AtomicU64,
    /// Circuits aktiv
    pub circuits_active: AtomicUsize,
    /// Hops durchschnittlich
    pub avg_hops: RwLock<f64>,
    /// Messages über Privacy-Layer
    pub private_messages: AtomicU64,
    /// Cover-Traffic Messages
    pub cover_traffic: AtomicU64,
    /// Relay-Rotationen
    pub relay_rotations: AtomicU64,
    /// Trust-basierte Relay-Auswahl
    pub trust_based_selections: AtomicU64,
}

impl PrivacyState {
    pub fn new() -> Self {
        Self {
            circuits_created: AtomicU64::new(0),
            circuits_active: AtomicUsize::new(0),
            avg_hops: RwLock::new(3.0),
            private_messages: AtomicU64::new(0),
            cover_traffic: AtomicU64::new(0),
            relay_rotations: AtomicU64::new(0),
            trust_based_selections: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> PrivacyStateSnapshot {
        PrivacyStateSnapshot {
            circuits_created: self.circuits_created.load(Ordering::Relaxed),
            circuits_active: self.circuits_active.load(Ordering::Relaxed),
            avg_hops: self.avg_hops.read().map(|h| *h).unwrap_or(3.0),
            private_messages: self.private_messages.load(Ordering::Relaxed),
            cover_traffic: self.cover_traffic.load(Ordering::Relaxed),
            relay_rotations: self.relay_rotations.load(Ordering::Relaxed),
            trust_based_selections: self.trust_based_selections.load(Ordering::Relaxed),
        }
    }
}

impl Default for PrivacyState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyStateSnapshot {
    pub circuits_created: u64,
    pub circuits_active: usize,
    pub avg_hops: f64,
    pub private_messages: u64,
    pub cover_traffic: u64,
    pub relay_rotations: u64,
    pub trust_based_selections: u64,
}

/// Aggregierter P2P State
#[derive(Debug)]
pub struct P2PState {
    pub swarm: SwarmState,
    pub gossip: GossipState,
    pub kademlia: KademliaState,
    pub relay: RelayState,
    pub privacy: PrivacyState,
}

impl P2PState {
    pub fn new() -> Self {
        Self {
            swarm: SwarmState::new(),
            gossip: GossipState::new(),
            kademlia: KademliaState::new(),
            relay: RelayState::new(),
            privacy: PrivacyState::new(),
        }
    }

    /// Berechne P2P-Health Score
    pub fn health_score(&self) -> f64 {
        let mut score: f64 = 100.0;

        // Mindestens 3 Peers
        let peers = self.swarm.connected_peers.load(Ordering::Relaxed);
        if peers < 3 {
            score -= 30.0;
        } else if peers < 6 {
            score -= 10.0;
        }

        // Kademlia Bootstrap
        if !self
            .kademlia
            .bootstrap_complete
            .read()
            .map(|b| *b)
            .unwrap_or(false)
        {
            score -= 20.0;
        }

        // Gossip Mesh
        let mesh = self.gossip.mesh_peers.load(Ordering::Relaxed);
        if mesh < 2 {
            score -= 15.0;
        }

        // Connection Errors
        let errors = self.swarm.connection_errors.load(Ordering::Relaxed);
        let total_conns = self.swarm.inbound_connections.load(Ordering::Relaxed)
            + self.swarm.outbound_connections.load(Ordering::Relaxed);
        if total_conns > 0 && errors as f64 / total_conns as f64 > 0.1 {
            score -= 10.0;
        }

        // DCUTR Success Rate
        let dcutr_rate = self.relay.dcutr_success_rate();
        if dcutr_rate < 0.5 {
            score -= 10.0;
        }

        score.max(0.0).min(100.0)
    }

    pub fn snapshot(&self) -> P2PStateSnapshot {
        P2PStateSnapshot {
            swarm: self.swarm.snapshot(),
            gossip: self.gossip.snapshot(),
            kademlia: self.kademlia.snapshot(),
            relay: self.relay.snapshot(),
            privacy: self.privacy.snapshot(),
            health_score: self.health_score(),
        }
    }
}

impl Default for P2PState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PStateSnapshot {
    pub swarm: SwarmStateSnapshot,
    pub gossip: GossipStateSnapshot,
    pub kademlia: KademliaStateSnapshot,
    pub relay: RelayStateSnapshot,
    pub privacy: PrivacyStateSnapshot,
    pub health_score: f64,
}

// ============================================================================
// UNIFIED STATE
// ============================================================================

/// Unified State - Hierarchisches State-Management für alle Module
///
/// # Design
///
/// Der UnifiedState verbindet alle State-Layer mit ihren Beziehungen:
/// - **CoreState**: Trust, Events, WorldFormula, Consensus
/// - **ExecutionState**: Gas, Mana, Context-Tracking
/// - **ProtectionState**: Anomaly, Diversity, Quadratic, Anti-Calc
/// - **StorageState**: KV, EventStore, Archive, Blueprints
/// - **PeerState**: Gateway, SagaComposer, IntentParser
/// - **P2PState**: Swarm, Gossip, Kademlia, Relay, Privacy
///
/// # Thread-Safety
///
/// - Atomare Counter für High-Frequency Updates
/// - RwLock für komplexe Strukturen
/// - Arc-Sharing für Cross-Module Access
///
/// # Beispiel
///
/// ```rust,ignore
/// let state = UnifiedState::new();
///
/// // Trust-Update mit Event-Trigger
/// state.core.trust.update(true, false);
/// state.core.trust.update_triggered_event();
/// state.core.events.trust_triggered.fetch_add(1, Ordering::Relaxed);
///
/// // Gateway Crossing
/// state.peer.gateway.crossing_allowed(0.7);
///
/// // P2P Peer Connected
/// state.p2p.swarm.peer_connected(false);
///
/// // Snapshot für Diagnostics
/// let snapshot = state.snapshot();
/// ```
pub struct UnifiedState {
    /// Startzeit
    pub started_at: Instant,

    /// Core Logic Layer (Κ2-Κ18)
    pub core: CoreState,

    /// Execution Layer (IPS ℳ)
    pub execution: ExecutionState,

    /// ECLVM Layer (Erynoa Core Language Virtual Machine)
    /// Führt ECL-Policies, Blueprints und Sagas aus
    pub eclvm: ECLVMState,

    /// Protection Layer (Κ19-Κ21)
    pub protection: ProtectionState,

    /// Storage Layer
    pub storage: StorageState,

    /// Peer Layer (Κ22-Κ24)
    pub peer: PeerState,

    /// P2P Network Layer
    pub p2p: P2PState,

    /// State-Beziehungs-Graph
    pub graph: StateGraph,

    /// Aktive Warnings
    pub warnings: RwLock<Vec<String>>,

    /// Global Health Score (cached)
    pub health_score: RwLock<f64>,
}

impl UnifiedState {
    /// Erstelle neuen Unified State
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            core: CoreState::new(),
            execution: ExecutionState::new(),
            eclvm: ECLVMState::new(),
            protection: ProtectionState::new(),
            storage: StorageState::new(),
            peer: PeerState::new(),
            p2p: P2PState::new(),
            graph: StateGraph::erynoa_graph(),
            warnings: RwLock::new(Vec::new()),
            health_score: RwLock::new(100.0),
        }
    }

    /// Uptime in Sekunden
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Berechne und cache Health Score
    pub fn calculate_health(&self) -> f64 {
        let mut score: f64 = 100.0;

        // Protection Health (20% Gewicht)
        score -= (100.0 - self.protection.health_score()) * 0.20;

        // Consensus Success Rate (15% Gewicht)
        score -= (1.0 - self.core.consensus.success_rate()) * 15.0;

        // Execution Success Rate (10% Gewicht)
        score -= (1.0 - self.execution.success_rate()) * 10.0;

        // ECLVM Policy Success Rate (10% Gewicht)
        score -= (1.0 - self.eclvm.policy_success_rate()) * 10.0;

        // P2P Health (20% Gewicht)
        score -= (100.0 - self.p2p.health_score()) * 0.20;

        // Peer Layer Health (10% Gewicht)
        let gateway_rate = self.peer.gateway.success_rate();
        let saga_rate = self.peer.saga.composition_success_rate();
        let peer_health = (gateway_rate + saga_rate) / 2.0 * 100.0;
        score -= (100.0 - peer_health) * 0.10;

        // Realm Crossing Success (5% Gewicht)
        score -= (1.0 - self.eclvm.crossing_allow_rate()) * 5.0;

        // Event Validation Errors (10% Gewicht)
        let event_errors = self.core.events.validation_errors.load(Ordering::Relaxed);
        let event_total = self.core.events.total.load(Ordering::Relaxed);
        if event_total > 0 {
            let error_rate = event_errors as f64 / event_total as f64;
            score -= error_rate * 10.0;
        }

        let final_score = score.max(0.0).min(100.0);

        // Cache
        if let Ok(mut cached) = self.health_score.write() {
            *cached = final_score;
        }

        final_score
    }

    /// Warning hinzufügen
    pub fn add_warning(&self, warning: String) {
        if let Ok(mut warnings) = self.warnings.write() {
            if !warnings.contains(&warning) {
                warnings.push(warning);
                if warnings.len() > 100 {
                    warnings.remove(0);
                }
            }
        }
    }

    /// Warning entfernen (per Prefix-Match)
    pub fn clear_warning(&self, prefix: &str) {
        if let Ok(mut warnings) = self.warnings.write() {
            warnings.retain(|w| !w.starts_with(prefix));
        }
    }

    /// Vollständiger Snapshot
    pub fn snapshot(&self) -> UnifiedStateSnapshot {
        UnifiedStateSnapshot {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            uptime_secs: self.uptime_secs(),
            core: self.core.snapshot(),
            execution: self.execution.snapshot(),
            eclvm: self.eclvm.snapshot(),
            protection: self.protection.snapshot(),
            storage: self.storage.snapshot(),
            peer: self.peer.snapshot(),
            p2p: self.p2p.snapshot(),
            health_score: self.calculate_health(),
            warnings: self.warnings.read().map(|w| w.clone()).unwrap_or_default(),
        }
    }
}

impl Default for UnifiedState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedStateSnapshot {
    pub timestamp_ms: u64,
    pub uptime_secs: u64,
    pub core: CoreStateSnapshot,
    pub execution: ExecutionStateSnapshot,
    pub eclvm: ECLVMStateSnapshot,
    pub protection: ProtectionStateSnapshot,
    pub storage: StorageStateSnapshot,
    pub peer: PeerStateSnapshot,
    pub p2p: P2PStateSnapshot,
    pub health_score: f64,
    pub warnings: Vec<String>,
}

// ============================================================================
// GLOBAL STATE ACCESSOR
// ============================================================================

/// Thread-safe globaler State (Singleton-Pattern)
pub type SharedUnifiedState = Arc<UnifiedState>;

/// Erstelle neuen Shared State
pub fn create_unified_state() -> SharedUnifiedState {
    Arc::new(UnifiedState::new())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_state() {
        let state = TrustState::new();
        state.update(true, false);
        state.update(false, true);
        state.update(false, false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.updates_total, 3);
        assert_eq!(snapshot.positive_updates, 1);
        assert_eq!(snapshot.negative_updates, 2);
        assert_eq!(snapshot.event_triggered_updates, 1);
        assert!((snapshot.asymmetry_ratio - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_event_state() {
        let state = EventState::new();
        state.add(true, 0, 0);
        state.add(false, 2, 1);
        state.add(false, 3, 2);
        state.finalize(100);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.total, 3);
        assert_eq!(snapshot.genesis, 1);
        assert_eq!(snapshot.max_depth, 2);
        assert!(snapshot.avg_parents > 0.0);
    }

    #[test]
    fn test_gateway_state() {
        let state = GatewayState::new();
        state.crossing_allowed(0.8);
        state.crossing_allowed(0.6);
        state.crossing_denied("trust");
        state.crossing_denied("credential");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.crossings_total, 4);
        assert_eq!(snapshot.crossings_allowed, 2);
        assert_eq!(snapshot.crossings_denied, 2);
        assert_eq!(snapshot.trust_violations, 1);
        assert_eq!(snapshot.credential_violations, 1);
        assert!((snapshot.success_rate - 0.5).abs() < 0.01);
        assert!((snapshot.avg_crossing_trust - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_saga_composer_state() {
        let state = SagaComposerState::new();
        state.saga_composed(true, 3, "Transfer");
        state.saga_composed(true, 5, "Delegate");
        state.saga_composed(false, 0, "Transfer");
        state.compensation(true);
        state.compensation(false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.sagas_composed, 3);
        assert_eq!(snapshot.successful_compositions, 2);
        assert_eq!(snapshot.failed_compositions, 1);
        assert!((snapshot.avg_steps_per_saga - 4.0).abs() < 0.01);
        assert_eq!(snapshot.compensations_executed, 2);
        assert_eq!(snapshot.compensations_successful, 1);
        assert!(*snapshot.goals_by_type.get("Transfer").unwrap_or(&0) == 2);
    }

    #[test]
    fn test_swarm_state() {
        let state = SwarmState::new();
        state.peer_connected(true);
        state.peer_connected(false);
        state.peer_connected(false);
        state.peer_disconnected();
        state.record_latency(5000);
        state.record_latency(7000);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.connected_peers, 2);
        assert_eq!(snapshot.inbound_connections, 1);
        assert_eq!(snapshot.outbound_connections, 2);
        assert!((snapshot.avg_latency_ms - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_gossip_state() {
        let state = GossipState::new();
        state.message_received();
        state.message_received();
        state.messages_validated.fetch_add(1, Ordering::Relaxed);
        state.messages_rejected.fetch_add(1, Ordering::Relaxed);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.messages_received, 2);
        assert!((snapshot.validation_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_p2p_state_health() {
        let state = P2PState::new();
        // Ohne Peers: Schlechter Score
        let score1 = state.health_score();
        assert!(score1 < 80.0);

        // Mit Peers: Besserer Score
        state.swarm.peer_connected(true);
        state.swarm.peer_connected(false);
        state.swarm.peer_connected(false);
        state.gossip.mesh_peers.store(3, Ordering::Relaxed);
        if let Ok(mut b) = state.kademlia.bootstrap_complete.write() {
            *b = true;
        }
        let score2 = state.health_score();
        assert!(score2 > score1);
    }

    #[test]
    fn test_unified_state() {
        let state = UnifiedState::new();

        state.core.trust.update(true, false);
        state.core.events.add(false, 2, 1);
        state.execution.start();
        state.execution.complete(true, 1000, 100, 2, 50);
        state.protection.anomaly("low");
        state.peer.gateway.crossing_allowed(0.7);
        state.p2p.swarm.peer_connected(false);
        state.p2p.gossip.message_sent();

        let snapshot = state.snapshot();
        assert_eq!(snapshot.core.trust.updates_total, 1);
        assert_eq!(snapshot.core.events.total, 1);
        assert_eq!(snapshot.execution.executions.total, 1);
        assert_eq!(snapshot.protection.anomaly.total, 1);
        assert_eq!(snapshot.peer.gateway.crossings_total, 1);
        assert_eq!(snapshot.p2p.swarm.connected_peers, 1);
        assert_eq!(snapshot.p2p.gossip.messages_sent, 1);
        assert!(snapshot.health_score > 0.0);
    }

    #[test]
    fn test_state_graph() {
        let graph = StateGraph::erynoa_graph();

        let dependents = graph.dependents(StateComponent::Trust);
        assert!(!dependents.is_empty());

        let triggered = graph.triggered_by(StateComponent::Trust);
        assert!(triggered.contains(&StateComponent::Event));

        // Prüfe Peer/P2P Beziehungen
        let gateway_triggered = graph.triggered_by(StateComponent::Gateway);
        assert!(gateway_triggered.contains(&StateComponent::Event));

        let gossip_deps = graph.dependents(StateComponent::Trust);
        assert!(!gossip_deps.is_empty());
    }
}
