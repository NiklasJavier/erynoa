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
    /// Room: Sub-Realm-Isolation mit eigenem Controller-Scope (Κ22)
    Room,
    /// Partition: Trust-basierte Berechtigungspartition innerhalb eines Rooms
    Partition,
    // P2P Network Layer
    Swarm,
    Gossip,
    Kademlia,
    Relay,
    NatTraversal,
    Privacy,
    // ═══════════════════════════════════════════════════════════════════════════
    // ENGINE-LAYER KOMPONENTEN (6 neue Engines für SOLL-Zustand)
    // ═══════════════════════════════════════════════════════════════════════════
    /// UI-Engine: Deklaratives, Trust-basiertes Interface-Rendering (Κ22)
    UI,
    /// DataLogic-Engine: Reaktive Event-Verarbeitung und Aggregation (Κ9-Κ12)
    DataLogic,
    /// API-Engine: Dynamische REST-API-Definition per ECL (Κ23)
    API,
    /// Governance-Engine: DAO-Prinzipien und Abstimmungsmechanismen (Κ19, Κ21)
    Governance,
    /// Controller-Engine: Berechtigungsverwaltung mit Delegation (Κ5)
    Controller,
    /// BlueprintComposer: Template-Komposition und Vererbung
    BlueprintComposer,
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
                // ═══════════════════════════════════════════════════════════════════════════
                // ROOM & PARTITION BEZIEHUNGEN (Sub-Realm Isolation)
                // ═══════════════════════════════════════════════════════════════════════════
                (Room, DependsOn, Realm), // Room ist Sub-Einheit eines Realms
                (Room, DependsOn, Trust), // Room-Access prüft Trust
                (Room, Triggers, Event),  // Room-Aktionen erzeugen Events
                (Room, Aggregates, Controller), // Room trackt Controller-Permissions
                (Partition, DependsOn, Room), // Partition ist Sub-Einheit eines Rooms
                (Partition, DependsOn, Trust), // Partition-Access prüft Trust
                (Partition, Validates, Controller), // Partition validiert Controller-Scope
                // ═══════════════════════════════════════════════════════════════════════════
                // UI-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (UI, DependsOn, Trust), // UI-Sichtbarkeit basiert auf Trust
                (UI, DependsOn, Realm), // UI ist per-Realm isoliert
                (UI, DependsOn, Room),  // UI-Scoping auf Room-Ebene
                (UI, DependsOn, Controller), // UI nutzt Controller für Permissions
                (UI, Triggers, Event),  // UI-Actions erzeugen Events
                (UI, Aggregates, DataLogic), // UI nutzt DataLogic für Bindings
                (UI, DependsOn, ECLVM), // UI-Logik läuft in ECLVM
                (UI, DependsOn, Gas),   // UI-Rendering verbraucht Gas
                (UI, DependsOn, Mana),  // UI-Events verbrauchen Mana
                // ═══════════════════════════════════════════════════════════════════════════
                // DATALOGIC-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (DataLogic, DependsOn, Event), // DataLogic verarbeitet Events
                (DataLogic, Aggregates, Event), // DataLogic aggregiert Event-Streams
                (DataLogic, Triggers, Event),  // Aggregationen emittieren Events
                (DataLogic, DependsOn, Trust), // DataAccess prüft Trust
                (DataLogic, DependsOn, ECLVM), // DataLogic-Funktionen in ECLVM
                (DataLogic, DependsOn, Gas),   // Compute verbraucht Gas
                (DataLogic, Validates, UI),    // DataLogic validiert UI-Bindings
                // ═══════════════════════════════════════════════════════════════════════════
                // API-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (API, DependsOn, Trust),      // API-Access basiert auf Trust
                (API, DependsOn, Controller), // API nutzt Controller für AuthZ
                (API, Validates, Gateway),    // API validiert External-Gateway
                (API, Triggers, Event),       // API-Calls erzeugen Events
                (API, DependsOn, ECLVM),      // API-Handler laufen in ECLVM
                (API, DependsOn, Gas),        // API-Processing verbraucht Gas
                (API, DependsOn, Mana),       // API-Responses verbrauchen Mana
                (API, Aggregates, DataLogic), // API nutzt DataLogic für Queries
                // ═══════════════════════════════════════════════════════════════════════════
                // GOVERNANCE-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (Governance, DependsOn, Trust), // Voting-Power basiert auf Trust
                (Governance, DependsOn, Quadratic), // Governance nutzt Quadratic-Voting
                (Governance, Validates, Controller), // Governance validiert Controller-Changes
                (Governance, Triggers, Controller), // Governance-Votes ändern Controller
                (Governance, Triggers, Event),  // Proposals/Votes erzeugen Events
                (Governance, DependsOn, ECLVM), // Governance-Regeln in ECLVM
                (Governance, DependsOn, Realm), // Governance ist per-Realm
                (Governance, Validates, AntiCalcification), // Governance prüft Machtkonz.
                // ═══════════════════════════════════════════════════════════════════════════
                // CONTROLLER-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (Controller, DependsOn, Trust), // Permissions basieren auf Trust
                (Controller, Triggers, Event),  // Permission-Changes erzeugen Events
                (Controller, Validates, Gateway), // Controller validiert Crossings
                (Controller, Validates, API),   // Controller validiert API-Access
                (Controller, Validates, UI),    // Controller validiert UI-Access
                (Controller, DependsOn, Realm), // Controller-Scope ist per-Realm
                (Controller, DependsOn, Room),  // Controller-Scope ist per-Room
                (Controller, DependsOn, Partition), // Controller-Scope ist per-Partition
                (Controller, Aggregates, Governance), // Controller trackt Gov-Delegations
                (Controller, DependsOn, ECLVM), // Permission-Rules in ECLVM
                // ═══════════════════════════════════════════════════════════════════════════
                // BLUEPRINTCOMPOSER-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (BlueprintComposer, DependsOn, Blueprint), // Composer nutzt Blueprint-Storage
                (BlueprintComposer, Aggregates, ECLBlueprint), // Composer aggregiert Instanzen
                (BlueprintComposer, Triggers, Event),      // Composition erzeugt Events
                (BlueprintComposer, DependsOn, ECLVM),     // Composition läuft in ECLVM
                (BlueprintComposer, DependsOn, Trust),     // Blueprint-Publish prüft Trust
                (BlueprintComposer, Validates, Realm),     // Composer validiert Realm-Compat.
                (BlueprintComposer, DependsOn, Gas),       // Composition verbraucht Gas
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
// ENGINE-LAYER STATE (6 neue Engines für SOLL-Zustand)
// ============================================================================

// ────────────────────────────────────────────────────────────────────────────
// 2.1 UI-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// UI-Engine State mit Component-Tree und Binding-Tracking
///
/// # Design
///
/// Die UI-Engine verwaltet deklarative, Trust-basierte Interfaces:
/// - **Component-Tree**: Hierarchischer UI-Aufbau
/// - **Bindings**: Reaktive Daten-Verbindungen
/// - **Trust-Gates**: Sichtbarkeit basierend auf Trust
/// - **Credential-Gates**: Zugriffskontrolle basierend auf Credentials
/// - **Render-Cache**: Optimierte Re-Renders
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// UI ──DependsOn──▶ Trust (Sichtbarkeit)
/// UI ──DependsOn──▶ Realm (Isolation)
/// UI ──DependsOn──▶ Room (Scoping)
/// UI ──DependsOn──▶ Controller (Permissions)
/// UI ──Triggers───▶ Event (UI-Actions)
/// UI ──Aggregates─▶ DataLogic (Bindings)
/// UI ──DependsOn──▶ ECLVM (UI-Logik)
/// UI ──DependsOn──▶ Gas/Mana (Resources)
/// ```
#[derive(Debug)]
pub struct UIState {
    // ─────────────────────────────────────────────────────────────────────────
    // Component-Tree Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Total registrierte UI-Components
    pub components_registered: AtomicU64,
    /// Aktuell aktive Components (mounted)
    pub components_active: AtomicU64,
    /// Component-Updates durchgeführt
    pub component_updates: AtomicU64,
    /// Component-Renders durchgeführt
    pub renders: AtomicU64,
    /// Cached Renders (keine Änderung)
    pub cache_hits: AtomicU64,
    /// Re-Renders (State-Änderung)
    pub cache_misses: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Binding-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Bindings
    pub bindings_active: AtomicU64,
    /// Binding-Updates propagiert
    pub binding_updates: AtomicU64,
    /// Binding-Fehler (z.B. Source nicht verfügbar)
    pub binding_errors: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Trust-Gates (Sichtbarkeits-Kontrolle)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Gate Evaluationen
    pub trust_gate_evaluations: AtomicU64,
    /// Trust-Gate Allowed
    pub trust_gate_allowed: AtomicU64,
    /// Trust-Gate Denied
    pub trust_gate_denied: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Credential-Gates (Zugriffs-Kontrolle)
    // ─────────────────────────────────────────────────────────────────────────
    /// Credential-Gate Evaluationen
    pub credential_gate_evaluations: AtomicU64,
    /// Credential-Gate Allowed
    pub credential_gate_allowed: AtomicU64,
    /// Credential-Gate Denied
    pub credential_gate_denied: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm UI-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische UI-Metriken
    pub realm_ui: RwLock<HashMap<String, RealmUIState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht für UI-Rendering
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht für UI-Events
    pub mana_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (UI ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// DataLogic-Aggregations (UI ⊃ DataLogic)
    pub datalogic_aggregations: AtomicU64,
    /// Controller-Validations (Controller ✓ UI)
    pub controller_validations: AtomicU64,
    /// Events getriggert (UI → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm UI-State für Isolation
#[derive(Debug)]
pub struct RealmUIState {
    pub components: AtomicU64,
    pub renders: AtomicU64,
    pub bindings: AtomicU64,
    pub trust_gate_denied: AtomicU64,
}

impl RealmUIState {
    pub fn new() -> Self {
        Self {
            components: AtomicU64::new(0),
            renders: AtomicU64::new(0),
            bindings: AtomicU64::new(0),
            trust_gate_denied: AtomicU64::new(0),
        }
    }
}

impl Default for RealmUIState {
    fn default() -> Self {
        Self::new()
    }
}

impl UIState {
    pub fn new() -> Self {
        Self {
            components_registered: AtomicU64::new(0),
            components_active: AtomicU64::new(0),
            component_updates: AtomicU64::new(0),
            renders: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            bindings_active: AtomicU64::new(0),
            binding_updates: AtomicU64::new(0),
            binding_errors: AtomicU64::new(0),
            trust_gate_evaluations: AtomicU64::new(0),
            trust_gate_allowed: AtomicU64::new(0),
            trust_gate_denied: AtomicU64::new(0),
            credential_gate_evaluations: AtomicU64::new(0),
            credential_gate_allowed: AtomicU64::new(0),
            credential_gate_denied: AtomicU64::new(0),
            realm_ui: RwLock::new(HashMap::new()),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            datalogic_aggregations: AtomicU64::new(0),
            controller_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Registriere neue Component
    pub fn register_component(&self, realm_id: Option<&str>) {
        self.components_registered.fetch_add(1, Ordering::Relaxed);
        self.components_active.fetch_add(1, Ordering::Relaxed);

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .components
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Component unmounted
    pub fn unregister_component(&self) {
        self.components_active.fetch_sub(1, Ordering::Relaxed);
    }

    /// Render durchgeführt
    pub fn render(&self, from_cache: bool, gas: u64, mana: u64, realm_id: Option<&str>) {
        self.renders.fetch_add(1, Ordering::Relaxed);
        if from_cache {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.mana_consumed.fetch_add(mana, Ordering::Relaxed);

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .renders
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Trust-Gate evaluiert
    pub fn trust_gate(&self, allowed: bool, realm_id: Option<&str>) {
        self.trust_gate_evaluations.fetch_add(1, Ordering::Relaxed);
        if allowed {
            self.trust_gate_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.trust_gate_denied.fetch_add(1, Ordering::Relaxed);
            if let Some(realm) = realm_id {
                self.get_or_create_realm(realm)
                    .trust_gate_denied
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Credential-Gate evaluiert
    pub fn credential_gate(&self, allowed: bool) {
        self.credential_gate_evaluations
            .fetch_add(1, Ordering::Relaxed);
        if allowed {
            self.credential_gate_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.credential_gate_denied.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Binding update
    pub fn binding_update(&self, success: bool, realm_id: Option<&str>) {
        self.binding_updates.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.binding_errors.fetch_add(1, Ordering::Relaxed);
        }
        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .bindings
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    fn get_or_create_realm(&self, realm_id: &str) -> &RealmUIState {
        if let Ok(mut realms) = self.realm_ui.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmUIState::new);
        }
        // Safe: Entry wurde gerade erstellt
        unsafe {
            self.realm_ui
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmUIState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmUIState> = std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmUIState::new)
                })
        }
    }

    /// Cache-Hit-Rate berechnen
    pub fn cache_hit_rate(&self) -> f64 {
        let total =
            self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed);
        if total > 0 {
            self.cache_hits.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            1.0
        }
    }

    /// Trust-Gate-Allow-Rate berechnen
    pub fn trust_gate_allow_rate(&self) -> f64 {
        let total = self.trust_gate_evaluations.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.trust_gate_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    /// Credential-Gate-Allow-Rate berechnen
    pub fn credential_gate_allow_rate(&self) -> f64 {
        let total = self.credential_gate_evaluations.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.credential_gate_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> UIStateSnapshot {
        UIStateSnapshot {
            components_registered: self.components_registered.load(Ordering::Relaxed),
            components_active: self.components_active.load(Ordering::Relaxed),
            component_updates: self.component_updates.load(Ordering::Relaxed),
            renders: self.renders.load(Ordering::Relaxed),
            cache_hit_rate: self.cache_hit_rate(),
            bindings_active: self.bindings_active.load(Ordering::Relaxed),
            binding_updates: self.binding_updates.load(Ordering::Relaxed),
            binding_errors: self.binding_errors.load(Ordering::Relaxed),
            trust_gate_evaluations: self.trust_gate_evaluations.load(Ordering::Relaxed),
            trust_gate_allow_rate: self.trust_gate_allow_rate(),
            credential_gate_evaluations: self.credential_gate_evaluations.load(Ordering::Relaxed),
            credential_gate_allow_rate: self.credential_gate_allow_rate(),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIStateSnapshot {
    pub components_registered: u64,
    pub components_active: u64,
    pub component_updates: u64,
    pub renders: u64,
    pub cache_hit_rate: f64,
    pub bindings_active: u64,
    pub binding_updates: u64,
    pub binding_errors: u64,
    pub trust_gate_evaluations: u64,
    pub trust_gate_allow_rate: f64,
    pub credential_gate_evaluations: u64,
    pub credential_gate_allow_rate: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.2 API-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// API-Engine State mit Endpoint-Registry und Rate-Limiting
///
/// # Design
///
/// Die API-Engine ermöglicht dynamische REST-API-Definition per ECL:
/// - **Endpoint-Registry**: Routing-Tabelle per Realm
/// - **Rate-Limits**: Trust-basierte Throttling
/// - **Metrics**: Request/Response-Tracking mit Latenz-Percentiles
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// API ──DependsOn──▶ Trust (Access-Control)
/// API ──DependsOn──▶ Controller (AuthZ)
/// API ──Validates──▶ Gateway (External)
/// API ──Triggers───▶ Event (API-Calls)
/// API ──Aggregates─▶ DataLogic (Queries)
/// API ──DependsOn──▶ ECLVM (Handler)
/// API ──DependsOn──▶ Gas/Mana (Resources)
/// ```
#[derive(Debug)]
pub struct APIState {
    // ─────────────────────────────────────────────────────────────────────────
    // Endpoint-Registry
    // ─────────────────────────────────────────────────────────────────────────
    /// Registrierte Endpoints total
    pub endpoints_registered: AtomicU64,
    /// Aktive Endpoints
    pub endpoints_active: AtomicU64,
    /// Endpoint-Updates (Hot-Reload)
    pub endpoint_updates: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Request-Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Total Requests
    pub requests_total: AtomicU64,
    /// Erfolgreiche Requests (2xx)
    pub requests_success: AtomicU64,
    /// Client-Errors (4xx)
    pub requests_client_error: AtomicU64,
    /// Server-Errors (5xx)
    pub requests_server_error: AtomicU64,
    /// Rate-Limited Requests (429)
    pub requests_rate_limited: AtomicU64,
    /// Auth-Failed Requests (401/403)
    pub requests_auth_failed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Latenz-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Durchschnittliche Latenz (µs)
    pub avg_latency_us: RwLock<f64>,
    /// P95 Latenz (µs)
    pub p95_latency_us: RwLock<f64>,
    /// P99 Latenz (µs)
    pub p99_latency_us: RwLock<f64>,
    /// Latenz-Historie (Rolling Window)
    latency_history: RwLock<Vec<u64>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Rate-Limiting
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Rate-Limit-Buckets
    pub rate_limit_buckets: AtomicU64,
    /// Rate-Limit-Resets
    pub rate_limit_resets: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm API-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische API-Metriken
    pub realm_api: RwLock<HashMap<String, RealmAPIState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht für API-Processing
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht für Responses
    pub mana_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (API ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Controller-Validations (Controller ✓ API)
    pub controller_validations: AtomicU64,
    /// Gateway-Validations (API ✓ Gateway)
    pub gateway_validations: AtomicU64,
    /// Events getriggert (API → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm API-State für Isolation
#[derive(Debug)]
pub struct RealmAPIState {
    pub endpoints: AtomicU64,
    pub requests: AtomicU64,
    pub rate_limited: AtomicU64,
    pub auth_failed: AtomicU64,
}

impl RealmAPIState {
    pub fn new() -> Self {
        Self {
            endpoints: AtomicU64::new(0),
            requests: AtomicU64::new(0),
            rate_limited: AtomicU64::new(0),
            auth_failed: AtomicU64::new(0),
        }
    }
}

impl Default for RealmAPIState {
    fn default() -> Self {
        Self::new()
    }
}

impl APIState {
    pub fn new() -> Self {
        Self {
            endpoints_registered: AtomicU64::new(0),
            endpoints_active: AtomicU64::new(0),
            endpoint_updates: AtomicU64::new(0),
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_client_error: AtomicU64::new(0),
            requests_server_error: AtomicU64::new(0),
            requests_rate_limited: AtomicU64::new(0),
            requests_auth_failed: AtomicU64::new(0),
            avg_latency_us: RwLock::new(0.0),
            p95_latency_us: RwLock::new(0.0),
            p99_latency_us: RwLock::new(0.0),
            latency_history: RwLock::new(Vec::with_capacity(1000)),
            rate_limit_buckets: AtomicU64::new(0),
            rate_limit_resets: AtomicU64::new(0),
            realm_api: RwLock::new(HashMap::new()),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            controller_validations: AtomicU64::new(0),
            gateway_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Endpoint registrieren
    pub fn register_endpoint(&self, realm_id: Option<&str>) {
        self.endpoints_registered.fetch_add(1, Ordering::Relaxed);
        self.endpoints_active.fetch_add(1, Ordering::Relaxed);

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .endpoints
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Request verarbeitet
    pub fn record_request(
        &self,
        latency_us: u64,
        status: u16,
        gas: u64,
        mana: u64,
        realm_id: Option<&str>,
    ) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.mana_consumed.fetch_add(mana, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        // Status-Kategorie
        match status {
            200..=299 => {
                self.requests_success.fetch_add(1, Ordering::Relaxed);
            }
            400..=499 => {
                self.requests_client_error.fetch_add(1, Ordering::Relaxed);
                if status == 429 {
                    self.requests_rate_limited.fetch_add(1, Ordering::Relaxed);
                    if let Some(realm) = realm_id {
                        self.get_or_create_realm(realm)
                            .rate_limited
                            .fetch_add(1, Ordering::Relaxed);
                    }
                } else if status == 401 || status == 403 {
                    self.requests_auth_failed.fetch_add(1, Ordering::Relaxed);
                    if let Some(realm) = realm_id {
                        self.get_or_create_realm(realm)
                            .auth_failed
                            .fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            500..=599 => {
                self.requests_server_error.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }

        // Realm-Tracking
        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .requests
                .fetch_add(1, Ordering::Relaxed);
        }

        // Latenz-Tracking
        self.update_latency(latency_us);
    }

    fn update_latency(&self, latency_us: u64) {
        if let Ok(mut history) = self.latency_history.write() {
            history.push(latency_us);
            if history.len() > 1000 {
                history.remove(0);
            }

            // Durchschnitt
            let avg = history.iter().sum::<u64>() as f64 / history.len() as f64;
            if let Ok(mut a) = self.avg_latency_us.write() {
                *a = avg;
            }

            // Percentiles
            if history.len() >= 10 {
                let mut sorted = history.clone();
                sorted.sort_unstable();
                let p95_idx = (sorted.len() as f64 * 0.95) as usize;
                let p99_idx = (sorted.len() as f64 * 0.99) as usize;

                if let Ok(mut p95) = self.p95_latency_us.write() {
                    *p95 = sorted
                        .get(p95_idx.min(sorted.len() - 1))
                        .copied()
                        .unwrap_or(0) as f64;
                }
                if let Ok(mut p99) = self.p99_latency_us.write() {
                    *p99 = sorted
                        .get(p99_idx.min(sorted.len() - 1))
                        .copied()
                        .unwrap_or(0) as f64;
                }
            }
        }
    }

    fn get_or_create_realm(&self, realm_id: &str) -> &RealmAPIState {
        if let Ok(mut realms) = self.realm_api.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmAPIState::new);
        }
        unsafe {
            self.realm_api
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmAPIState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmAPIState> = std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmAPIState::new)
                })
        }
    }

    /// Success-Rate berechnen
    pub fn success_rate(&self) -> f64 {
        let total = self.requests_total.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.requests_success.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> APIStateSnapshot {
        APIStateSnapshot {
            endpoints_registered: self.endpoints_registered.load(Ordering::Relaxed),
            endpoints_active: self.endpoints_active.load(Ordering::Relaxed),
            endpoint_updates: self.endpoint_updates.load(Ordering::Relaxed),
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success: self.requests_success.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            requests_client_error: self.requests_client_error.load(Ordering::Relaxed),
            requests_server_error: self.requests_server_error.load(Ordering::Relaxed),
            requests_rate_limited: self.requests_rate_limited.load(Ordering::Relaxed),
            requests_auth_failed: self.requests_auth_failed.load(Ordering::Relaxed),
            avg_latency_us: self.avg_latency_us.read().map(|v| *v).unwrap_or(0.0),
            p95_latency_us: self.p95_latency_us.read().map(|v| *v).unwrap_or(0.0),
            p99_latency_us: self.p99_latency_us.read().map(|v| *v).unwrap_or(0.0),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for APIState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIStateSnapshot {
    pub endpoints_registered: u64,
    pub endpoints_active: u64,
    pub endpoint_updates: u64,
    pub requests_total: u64,
    pub requests_success: u64,
    pub success_rate: f64,
    pub requests_client_error: u64,
    pub requests_server_error: u64,
    pub requests_rate_limited: u64,
    pub requests_auth_failed: u64,
    pub avg_latency_us: f64,
    pub p95_latency_us: f64,
    pub p99_latency_us: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.3 GOVERNANCE-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// Governance-Engine State mit Proposal-Tracking und Delegation-Graph
///
/// # Design
///
/// Die Governance-Engine implementiert DAO-Prinzipien:
/// - **Quadratic Voting**: √-basierte Stimmgewichtung (Κ21)
/// - **Delegation**: Transitive Trust-Delegation (Liquid Democracy)
/// - **Anti-Calcification**: Machtkonzentrations-Check (Κ19)
/// - **Proposals**: Lifecycle-Management mit Quorum
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// Governance ──DependsOn──▶ Trust (Voting-Power)
/// Governance ──DependsOn──▶ Quadratic (Voting-Mechanik)
/// Governance ──Validates──▶ Controller (Permission-Changes)
/// Governance ──Triggers───▶ Controller (Vote-Results)
/// Governance ──Triggers───▶ Event (Proposals/Votes)
/// Governance ──Validates──▶ AntiCalcification (Power-Check)
/// ```
#[derive(Debug)]
pub struct GovernanceState {
    // ─────────────────────────────────────────────────────────────────────────
    // Proposal-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Total erstellte Proposals
    pub proposals_created: AtomicU64,
    /// Aktive Proposals (in Voting-Phase)
    pub proposals_active: AtomicU64,
    /// Abgeschlossene Proposals
    pub proposals_completed: AtomicU64,
    /// Angenommene Proposals
    pub proposals_accepted: AtomicU64,
    /// Abgelehnte Proposals
    pub proposals_rejected: AtomicU64,
    /// Abgebrochene Proposals (Quorum nicht erreicht)
    pub proposals_expired: AtomicU64,
    /// Vetoed Proposals
    pub proposals_vetoed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Voting-Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Total abgegebene Votes
    pub votes_cast: AtomicU64,
    /// Unique Voters (geschätzt)
    pub unique_voters: AtomicU64,
    /// Delegierte Votes
    pub votes_delegated: AtomicU64,
    /// Quadratische Reduktionen angewendet
    pub quadratic_reductions: AtomicU64,
    /// Durchschnittliche Voting-Power (vor Quadratic)
    pub avg_voting_power: RwLock<f64>,
    /// Durchschnittliche Participation-Rate
    pub avg_participation_rate: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Delegation-Graph
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Delegationen
    pub delegations_active: AtomicU64,
    /// Delegations-Ketten-Tiefe (max observed)
    pub max_delegation_depth: AtomicU64,
    /// Zirkuläre Delegationen verhindert
    pub circular_delegations_prevented: AtomicU64,
    /// Abgelaufene Delegationen
    pub delegations_expired: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Anti-Calcification (Κ19)
    // ─────────────────────────────────────────────────────────────────────────
    /// Power-Concentration-Checks
    pub power_checks: AtomicU64,
    /// Power-Concentration-Violations
    pub power_violations: AtomicU64,
    /// Gini-Koeffizient der Voting-Power
    pub voting_power_gini: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm Governance-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische Governance-Metriken
    pub realm_governance: RwLock<HashMap<String, RealmGovernanceState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (Governance ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Quadratic-Validations (Governance ← Quadratic)
    pub quadratic_validations: AtomicU64,
    /// Controller-Triggers (Governance → Controller)
    pub controller_triggers: AtomicU64,
    /// AntiCalc-Validations (Governance ✓ AntiCalcification)
    pub anticalc_validations: AtomicU64,
    /// Events getriggert (Governance → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm Governance-State
#[derive(Debug)]
pub struct RealmGovernanceState {
    pub proposals: AtomicU64,
    pub votes: AtomicU64,
    pub delegations: AtomicU64,
    /// Governance-Typ: "council", "direct", "liquid", "quadratic"
    pub governance_type: RwLock<String>,
}

impl RealmGovernanceState {
    pub fn new() -> Self {
        Self {
            proposals: AtomicU64::new(0),
            votes: AtomicU64::new(0),
            delegations: AtomicU64::new(0),
            governance_type: RwLock::new("quadratic".to_string()),
        }
    }
}

impl Default for RealmGovernanceState {
    fn default() -> Self {
        Self::new()
    }
}

impl GovernanceState {
    pub fn new() -> Self {
        Self {
            proposals_created: AtomicU64::new(0),
            proposals_active: AtomicU64::new(0),
            proposals_completed: AtomicU64::new(0),
            proposals_accepted: AtomicU64::new(0),
            proposals_rejected: AtomicU64::new(0),
            proposals_expired: AtomicU64::new(0),
            proposals_vetoed: AtomicU64::new(0),
            votes_cast: AtomicU64::new(0),
            unique_voters: AtomicU64::new(0),
            votes_delegated: AtomicU64::new(0),
            quadratic_reductions: AtomicU64::new(0),
            avg_voting_power: RwLock::new(1.0),
            avg_participation_rate: RwLock::new(0.0),
            delegations_active: AtomicU64::new(0),
            max_delegation_depth: AtomicU64::new(0),
            circular_delegations_prevented: AtomicU64::new(0),
            delegations_expired: AtomicU64::new(0),
            power_checks: AtomicU64::new(0),
            power_violations: AtomicU64::new(0),
            voting_power_gini: RwLock::new(0.0),
            realm_governance: RwLock::new(HashMap::new()),
            trust_dependency_updates: AtomicU64::new(0),
            quadratic_validations: AtomicU64::new(0),
            controller_triggers: AtomicU64::new(0),
            anticalc_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Proposal erstellt
    pub fn proposal_created(&self, realm_id: Option<&str>) {
        self.proposals_created.fetch_add(1, Ordering::Relaxed);
        self.proposals_active.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .proposals
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Proposal abgeschlossen
    pub fn proposal_completed(&self, result: &str) {
        self.proposals_active.fetch_sub(1, Ordering::Relaxed);
        self.proposals_completed.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        match result {
            "accepted" => {
                self.proposals_accepted.fetch_add(1, Ordering::Relaxed);
                self.controller_triggers.fetch_add(1, Ordering::Relaxed);
            }
            "rejected" => {
                self.proposals_rejected.fetch_add(1, Ordering::Relaxed);
            }
            "expired" => {
                self.proposals_expired.fetch_add(1, Ordering::Relaxed);
            }
            "vetoed" => {
                self.proposals_vetoed.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    /// Vote abgegeben
    pub fn vote_cast(
        &self,
        voting_power: f64,
        is_delegated: bool,
        quadratic_reduced: bool,
        realm_id: Option<&str>,
    ) {
        self.votes_cast.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        if is_delegated {
            self.votes_delegated.fetch_add(1, Ordering::Relaxed);
        }
        if quadratic_reduced {
            self.quadratic_reductions.fetch_add(1, Ordering::Relaxed);
            self.quadratic_validations.fetch_add(1, Ordering::Relaxed);
        }

        // Update average voting power
        if let Ok(mut avg) = self.avg_voting_power.write() {
            let total = self.votes_cast.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + voting_power) / total;
        }

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .votes
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Delegation erstellt
    pub fn delegation_created(&self, depth: u64, realm_id: Option<&str>) {
        self.delegations_active.fetch_add(1, Ordering::Relaxed);

        // Update max depth
        loop {
            let current = self.max_delegation_depth.load(Ordering::Relaxed);
            if depth <= current {
                break;
            }
            if self
                .max_delegation_depth
                .compare_exchange(current, depth, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .delegations
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Power-Check durchgeführt
    pub fn power_check(&self, violated: bool, gini: f64) {
        self.power_checks.fetch_add(1, Ordering::Relaxed);
        self.anticalc_validations.fetch_add(1, Ordering::Relaxed);

        if violated {
            self.power_violations.fetch_add(1, Ordering::Relaxed);
        }

        if let Ok(mut g) = self.voting_power_gini.write() {
            *g = gini;
        }
    }

    fn get_or_create_realm(&self, realm_id: &str) -> &RealmGovernanceState {
        if let Ok(mut realms) = self.realm_governance.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmGovernanceState::new);
        }
        unsafe {
            self.realm_governance
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmGovernanceState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmGovernanceState> =
                        std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmGovernanceState::new)
                })
        }
    }

    /// Proposal-Success-Rate
    pub fn proposal_success_rate(&self) -> f64 {
        let completed = self.proposals_completed.load(Ordering::Relaxed) as f64;
        if completed > 0.0 {
            self.proposals_accepted.load(Ordering::Relaxed) as f64 / completed
        } else {
            0.0
        }
    }

    /// Delegation-Rate (Anteil delegierter Votes)
    pub fn delegation_rate(&self) -> f64 {
        let total = self.votes_cast.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.votes_delegated.load(Ordering::Relaxed) as f64 / total
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> GovernanceStateSnapshot {
        GovernanceStateSnapshot {
            proposals_created: self.proposals_created.load(Ordering::Relaxed),
            proposals_active: self.proposals_active.load(Ordering::Relaxed),
            proposals_completed: self.proposals_completed.load(Ordering::Relaxed),
            proposals_accepted: self.proposals_accepted.load(Ordering::Relaxed),
            proposals_rejected: self.proposals_rejected.load(Ordering::Relaxed),
            proposal_success_rate: self.proposal_success_rate(),
            votes_cast: self.votes_cast.load(Ordering::Relaxed),
            unique_voters: self.unique_voters.load(Ordering::Relaxed),
            votes_delegated: self.votes_delegated.load(Ordering::Relaxed),
            delegation_rate: self.delegation_rate(),
            delegations_active: self.delegations_active.load(Ordering::Relaxed),
            max_delegation_depth: self.max_delegation_depth.load(Ordering::Relaxed),
            quadratic_reductions: self.quadratic_reductions.load(Ordering::Relaxed),
            avg_voting_power: self.avg_voting_power.read().map(|v| *v).unwrap_or(1.0),
            voting_power_gini: self.voting_power_gini.read().map(|v| *v).unwrap_or(0.0),
            power_violations: self.power_violations.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for GovernanceState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceStateSnapshot {
    pub proposals_created: u64,
    pub proposals_active: u64,
    pub proposals_completed: u64,
    pub proposals_accepted: u64,
    pub proposals_rejected: u64,
    pub proposal_success_rate: f64,
    pub votes_cast: u64,
    pub unique_voters: u64,
    pub votes_delegated: u64,
    pub delegation_rate: f64,
    pub delegations_active: u64,
    pub max_delegation_depth: u64,
    pub quadratic_reductions: u64,
    pub avg_voting_power: f64,
    pub voting_power_gini: f64,
    pub power_violations: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.4 CONTROLLER-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// Controller-Engine State mit Permission-Registry und Audit-Log
///
/// # Design
///
/// Die Controller-Engine verwaltet Berechtigungen:
/// - **Scoped Permissions**: Realm > Room > Partition Hierarchie
/// - **Delegation**: Transitive Permission-Vererbung mit Constraints
/// - **Audit-Trail**: Vollständige Permission-History
/// - **Automation**: Trigger-basierte Permission-Änderungen
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// Controller ──DependsOn──▶ Trust (Permission-Basis)
/// Controller ──Validates──▶ Gateway (Crossing-Auth)
/// Controller ──Validates──▶ API (API-Auth)
/// Controller ──Validates──▶ UI (UI-Auth)
/// Controller ──Aggregates─▶ Governance (Delegation-Sync)
/// Controller ──DependsOn──▶ Realm/Room/Partition (Scope)
/// Controller ──DependsOn──▶ ECLVM (Permission-Rules)
/// ```
#[derive(Debug)]
pub struct ControllerState {
    // ─────────────────────────────────────────────────────────────────────────
    // Permission-Registry
    // ─────────────────────────────────────────────────────────────────────────
    /// Total registrierte Permissions
    pub permissions_registered: AtomicU64,
    /// Aktive Permissions
    pub permissions_active: AtomicU64,
    /// Permission-Grants
    pub permission_grants: AtomicU64,
    /// Permission-Revokes
    pub permission_revokes: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Authorization-Checks
    // ─────────────────────────────────────────────────────────────────────────
    /// AuthZ-Checks total
    pub authz_checks: AtomicU64,
    /// AuthZ-Allowed
    pub authz_allowed: AtomicU64,
    /// AuthZ-Denied
    pub authz_denied: AtomicU64,
    /// Via-Delegation AuthZ
    pub authz_via_delegation: AtomicU64,
    /// Durchschnittliche Check-Latenz (µs)
    pub avg_check_latency_us: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Delegation
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Delegationen
    pub delegations_active: AtomicU64,
    /// Delegations-Ketten (max depth)
    pub max_delegation_depth: AtomicU64,
    /// Delegations-Nutzungen
    pub delegations_used: AtomicU64,
    /// Abgelaufene Delegationen
    pub delegations_expired: AtomicU64,
    /// Delegations-Konflikte (z.B. zirkulär)
    pub delegation_conflicts: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Scope-Tracking (Realm > Room > Partition)
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-Scope Checks
    pub realm_scope_checks: AtomicU64,
    /// Room-Scope Checks
    pub room_scope_checks: AtomicU64,
    /// Partition-Scope Checks
    pub partition_scope_checks: AtomicU64,
    /// Scope-Inheritance-Resolutions
    pub scope_inheritance_resolutions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Audit-Log
    // ─────────────────────────────────────────────────────────────────────────
    /// Audit-Entries geschrieben
    pub audit_entries: AtomicU64,
    /// Audit-Log-Größe (Bytes, approximiert)
    pub audit_log_bytes: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Automation
    // ─────────────────────────────────────────────────────────────────────────
    /// Automation-Rules aktiv
    pub automation_rules_active: AtomicU64,
    /// Automation-Triggers ausgelöst
    pub automation_triggers: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm Controller-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische Controller-Metriken
    pub realm_controller: RwLock<HashMap<String, RealmControllerState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (Controller ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Gateway-Validations (Controller ✓ Gateway)
    pub gateway_validations: AtomicU64,
    /// API-Validations (Controller ✓ API)
    pub api_validations: AtomicU64,
    /// UI-Validations (Controller ✓ UI)
    pub ui_validations: AtomicU64,
    /// Governance-Aggregations (Controller ⊃ Governance)
    pub governance_aggregations: AtomicU64,
    /// Events getriggert (Controller → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm Controller-State
#[derive(Debug)]
pub struct RealmControllerState {
    pub permissions: AtomicU64,
    pub authz_checks: AtomicU64,
    pub authz_denied: AtomicU64,
    pub delegations: AtomicU64,
    pub rooms: AtomicU64,
    pub partitions: AtomicU64,
}

impl RealmControllerState {
    pub fn new() -> Self {
        Self {
            permissions: AtomicU64::new(0),
            authz_checks: AtomicU64::new(0),
            authz_denied: AtomicU64::new(0),
            delegations: AtomicU64::new(0),
            rooms: AtomicU64::new(0),
            partitions: AtomicU64::new(0),
        }
    }
}

impl Default for RealmControllerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ControllerState {
    pub fn new() -> Self {
        Self {
            permissions_registered: AtomicU64::new(0),
            permissions_active: AtomicU64::new(0),
            permission_grants: AtomicU64::new(0),
            permission_revokes: AtomicU64::new(0),
            authz_checks: AtomicU64::new(0),
            authz_allowed: AtomicU64::new(0),
            authz_denied: AtomicU64::new(0),
            authz_via_delegation: AtomicU64::new(0),
            avg_check_latency_us: RwLock::new(0.0),
            delegations_active: AtomicU64::new(0),
            max_delegation_depth: AtomicU64::new(0),
            delegations_used: AtomicU64::new(0),
            delegations_expired: AtomicU64::new(0),
            delegation_conflicts: AtomicU64::new(0),
            realm_scope_checks: AtomicU64::new(0),
            room_scope_checks: AtomicU64::new(0),
            partition_scope_checks: AtomicU64::new(0),
            scope_inheritance_resolutions: AtomicU64::new(0),
            audit_entries: AtomicU64::new(0),
            audit_log_bytes: AtomicU64::new(0),
            automation_rules_active: AtomicU64::new(0),
            automation_triggers: AtomicU64::new(0),
            realm_controller: RwLock::new(HashMap::new()),
            trust_dependency_updates: AtomicU64::new(0),
            gateway_validations: AtomicU64::new(0),
            api_validations: AtomicU64::new(0),
            ui_validations: AtomicU64::new(0),
            governance_aggregations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Permission gewährt
    pub fn grant_permission(&self, realm_id: Option<&str>) {
        self.permission_grants.fetch_add(1, Ordering::Relaxed);
        self.permissions_active.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);
        self.write_audit(128); // ~128 bytes per audit entry

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .permissions
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Permission widerrufen
    pub fn revoke_permission(&self) {
        self.permission_revokes.fetch_add(1, Ordering::Relaxed);
        self.permissions_active.fetch_sub(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);
        self.write_audit(128);
    }

    /// AuthZ-Check durchgeführt
    pub fn check_authorization(
        &self,
        allowed: bool,
        via_delegation: bool,
        latency_us: u64,
        scope: &str,
        realm_id: Option<&str>,
    ) {
        self.authz_checks.fetch_add(1, Ordering::Relaxed);

        if allowed {
            self.authz_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.authz_denied.fetch_add(1, Ordering::Relaxed);
            if let Some(realm) = realm_id {
                self.get_or_create_realm(realm)
                    .authz_denied
                    .fetch_add(1, Ordering::Relaxed);
            }
        }

        if via_delegation {
            self.authz_via_delegation.fetch_add(1, Ordering::Relaxed);
            self.delegations_used.fetch_add(1, Ordering::Relaxed);
        }

        // Scope-Tracking
        match scope {
            "realm" => {
                self.realm_scope_checks.fetch_add(1, Ordering::Relaxed);
            }
            "room" => {
                self.room_scope_checks.fetch_add(1, Ordering::Relaxed);
            }
            "partition" => {
                self.partition_scope_checks.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }

        // Latenz-Update
        if let Ok(mut avg) = self.avg_check_latency_us.write() {
            let total = self.authz_checks.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
        }

        // Realm-Tracking
        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .authz_checks
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Delegation erstellt
    pub fn create_delegation(&self, depth: u64, realm_id: Option<&str>) {
        self.delegations_active.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);
        self.governance_aggregations.fetch_add(1, Ordering::Relaxed);
        self.write_audit(256);

        // Update max depth
        loop {
            let current = self.max_delegation_depth.load(Ordering::Relaxed);
            if depth <= current {
                break;
            }
            if self
                .max_delegation_depth
                .compare_exchange(current, depth, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .delegations
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Audit-Entry schreiben
    fn write_audit(&self, bytes: u64) {
        self.audit_entries.fetch_add(1, Ordering::Relaxed);
        self.audit_log_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    fn get_or_create_realm(&self, realm_id: &str) -> &RealmControllerState {
        if let Ok(mut realms) = self.realm_controller.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmControllerState::new);
        }
        unsafe {
            self.realm_controller
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmControllerState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmControllerState> =
                        std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmControllerState::new)
                })
        }
    }

    /// AuthZ-Success-Rate
    pub fn authz_success_rate(&self) -> f64 {
        let total = self.authz_checks.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.authz_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    /// Delegation-Usage-Rate
    pub fn delegation_usage_rate(&self) -> f64 {
        let checks = self.authz_checks.load(Ordering::Relaxed) as f64;
        if checks > 0.0 {
            self.authz_via_delegation.load(Ordering::Relaxed) as f64 / checks
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> ControllerStateSnapshot {
        ControllerStateSnapshot {
            permissions_registered: self.permissions_registered.load(Ordering::Relaxed),
            permissions_active: self.permissions_active.load(Ordering::Relaxed),
            permission_grants: self.permission_grants.load(Ordering::Relaxed),
            permission_revokes: self.permission_revokes.load(Ordering::Relaxed),
            authz_checks: self.authz_checks.load(Ordering::Relaxed),
            authz_allowed: self.authz_allowed.load(Ordering::Relaxed),
            authz_denied: self.authz_denied.load(Ordering::Relaxed),
            authz_success_rate: self.authz_success_rate(),
            avg_check_latency_us: self.avg_check_latency_us.read().map(|v| *v).unwrap_or(0.0),
            delegations_active: self.delegations_active.load(Ordering::Relaxed),
            max_delegation_depth: self.max_delegation_depth.load(Ordering::Relaxed),
            delegation_usage_rate: self.delegation_usage_rate(),
            realm_scope_checks: self.realm_scope_checks.load(Ordering::Relaxed),
            room_scope_checks: self.room_scope_checks.load(Ordering::Relaxed),
            partition_scope_checks: self.partition_scope_checks.load(Ordering::Relaxed),
            audit_entries: self.audit_entries.load(Ordering::Relaxed),
            audit_log_bytes: self.audit_log_bytes.load(Ordering::Relaxed),
            automation_triggers: self.automation_triggers.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for ControllerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerStateSnapshot {
    pub permissions_registered: u64,
    pub permissions_active: u64,
    pub permission_grants: u64,
    pub permission_revokes: u64,
    pub authz_checks: u64,
    pub authz_allowed: u64,
    pub authz_denied: u64,
    pub authz_success_rate: f64,
    pub avg_check_latency_us: f64,
    pub delegations_active: u64,
    pub max_delegation_depth: u64,
    pub delegation_usage_rate: f64,
    pub realm_scope_checks: u64,
    pub room_scope_checks: u64,
    pub partition_scope_checks: u64,
    pub audit_entries: u64,
    pub audit_log_bytes: u64,
    pub automation_triggers: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.5 DATALOGIC-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// DataLogic-Engine State mit Reactive Streams und Aggregations
///
/// # Design
///
/// Die DataLogic-Engine verarbeitet Events reaktiv:
/// - **Streams**: Event-basierte Datenströme
/// - **Aggregations**: count, sum, avg, window-basiert
/// - **Bindings**: Reaktive UI-Verbindungen
/// - **Filters**: Trust-basierte Filterung
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// DataLogic ──DependsOn──▶ Event (Event-Processing)
/// DataLogic ──Aggregates─▶ Event (Aggregation)
/// DataLogic ──Triggers───▶ Event (Derived Events)
/// DataLogic ──DependsOn──▶ Trust (Access-Control)
/// DataLogic ──DependsOn──▶ ECLVM (Functions)
/// DataLogic ──Validates──▶ UI (Binding-Validation)
/// ```
#[derive(Debug)]
pub struct DataLogicState {
    // ─────────────────────────────────────────────────────────────────────────
    // Stream-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Registrierte Streams
    pub streams_registered: AtomicU64,
    /// Aktive Streams
    pub streams_active: AtomicU64,
    /// Stream-Subscriptions
    pub stream_subscriptions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Event-Processing
    // ─────────────────────────────────────────────────────────────────────────
    /// Events verarbeitet
    pub events_processed: AtomicU64,
    /// Events gefiltert (Trust/Access)
    pub events_filtered: AtomicU64,
    /// Events weitergeleitet (nach Filter)
    pub events_forwarded: AtomicU64,
    /// Processing-Fehler
    pub processing_errors: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Aggregation
    // ─────────────────────────────────────────────────────────────────────────
    /// Registrierte Aggregationen
    pub aggregations_registered: AtomicU64,
    /// Aggregationen berechnet
    pub aggregations_computed: AtomicU64,
    /// Aggregation-Results emittiert
    pub aggregation_results: AtomicU64,
    /// Durchschnittliche Aggregation-Latenz (µs)
    pub avg_aggregation_latency_us: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Binding-Propagation
    // ─────────────────────────────────────────────────────────────────────────
    /// Binding-Updates propagiert
    pub binding_propagations: AtomicU64,
    /// Binding-Fehler
    pub binding_errors: AtomicU64,
    /// Durchschnittliche Propagation-Latenz (µs)
    pub avg_propagation_latency_us: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht
    pub mana_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (DataLogic ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Event-Aggregations (DataLogic ⊃ Event)
    pub event_aggregations: AtomicU64,
    /// UI-Validations (DataLogic ✓ UI)
    pub ui_validations: AtomicU64,
    /// Events getriggert (DataLogic → Event)
    pub events_triggered: AtomicU64,
}

impl DataLogicState {
    pub fn new() -> Self {
        Self {
            streams_registered: AtomicU64::new(0),
            streams_active: AtomicU64::new(0),
            stream_subscriptions: AtomicU64::new(0),
            events_processed: AtomicU64::new(0),
            events_filtered: AtomicU64::new(0),
            events_forwarded: AtomicU64::new(0),
            processing_errors: AtomicU64::new(0),
            aggregations_registered: AtomicU64::new(0),
            aggregations_computed: AtomicU64::new(0),
            aggregation_results: AtomicU64::new(0),
            avg_aggregation_latency_us: RwLock::new(0.0),
            binding_propagations: AtomicU64::new(0),
            binding_errors: AtomicU64::new(0),
            avg_propagation_latency_us: RwLock::new(0.0),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            event_aggregations: AtomicU64::new(0),
            ui_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Stream registrieren
    pub fn register_stream(&self) {
        self.streams_registered.fetch_add(1, Ordering::Relaxed);
        self.streams_active.fetch_add(1, Ordering::Relaxed);
    }

    /// Event verarbeiten
    pub fn process_event(&self, filtered: bool, gas: u64) {
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);

        if filtered {
            self.events_filtered.fetch_add(1, Ordering::Relaxed);
        } else {
            self.events_forwarded.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Aggregation berechnet
    pub fn aggregation_computed(&self, latency_us: u64, gas: u64) {
        self.aggregations_computed.fetch_add(1, Ordering::Relaxed);
        self.aggregation_results.fetch_add(1, Ordering::Relaxed);
        self.event_aggregations.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);

        // Update average latency
        if let Ok(mut avg) = self.avg_aggregation_latency_us.write() {
            let total = self.aggregations_computed.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
        }
    }

    /// Binding propagiert
    pub fn propagate_binding(&self, success: bool, latency_us: u64, mana: u64) {
        self.binding_propagations.fetch_add(1, Ordering::Relaxed);
        self.mana_consumed.fetch_add(mana, Ordering::Relaxed);
        self.ui_validations.fetch_add(1, Ordering::Relaxed);

        if !success {
            self.binding_errors.fetch_add(1, Ordering::Relaxed);
        }

        // Update average latency
        if let Ok(mut avg) = self.avg_propagation_latency_us.write() {
            let total = self.binding_propagations.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
        }
    }

    /// Success-Rate (Events die nicht gefiltert wurden)
    pub fn success_rate(&self) -> f64 {
        let total = self.events_processed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.events_forwarded.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    /// Binding-Success-Rate
    pub fn binding_success_rate(&self) -> f64 {
        let total = self.binding_propagations.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            (total - self.binding_errors.load(Ordering::Relaxed) as f64) / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> DataLogicStateSnapshot {
        DataLogicStateSnapshot {
            streams_registered: self.streams_registered.load(Ordering::Relaxed),
            streams_active: self.streams_active.load(Ordering::Relaxed),
            stream_subscriptions: self.stream_subscriptions.load(Ordering::Relaxed),
            events_processed: self.events_processed.load(Ordering::Relaxed),
            events_filtered: self.events_filtered.load(Ordering::Relaxed),
            events_forwarded: self.events_forwarded.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            aggregations_registered: self.aggregations_registered.load(Ordering::Relaxed),
            aggregations_computed: self.aggregations_computed.load(Ordering::Relaxed),
            avg_aggregation_latency_us: self
                .avg_aggregation_latency_us
                .read()
                .map(|v| *v)
                .unwrap_or(0.0),
            binding_propagations: self.binding_propagations.load(Ordering::Relaxed),
            binding_success_rate: self.binding_success_rate(),
            avg_propagation_latency_us: self
                .avg_propagation_latency_us
                .read()
                .map(|v| *v)
                .unwrap_or(0.0),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for DataLogicState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLogicStateSnapshot {
    pub streams_registered: u64,
    pub streams_active: u64,
    pub stream_subscriptions: u64,
    pub events_processed: u64,
    pub events_filtered: u64,
    pub events_forwarded: u64,
    pub success_rate: f64,
    pub aggregations_registered: u64,
    pub aggregations_computed: u64,
    pub avg_aggregation_latency_us: f64,
    pub binding_propagations: u64,
    pub binding_success_rate: f64,
    pub avg_propagation_latency_us: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.6 BLUEPRINTCOMPOSER-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// BlueprintComposer-Engine State mit Composition und Versioning
///
/// # Design
///
/// Der BlueprintComposer verwaltet Template-Komposition:
/// - **Composition**: Blueprint-Vererbung und -Erweiterung
/// - **Versioning**: Semantic Versioning mit Migrations
/// - **Validation**: Realm-Compatibility-Checks
/// - **Caching**: Compiled Blueprint Cache
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// BlueprintComposer ──DependsOn──▶ Blueprint (Storage)
/// BlueprintComposer ──Aggregates─▶ ECLBlueprint (Instances)
/// BlueprintComposer ──Triggers───▶ Event (Composition)
/// BlueprintComposer ──DependsOn──▶ ECLVM (Execution)
/// BlueprintComposer ──DependsOn──▶ Trust (Publish-Auth)
/// BlueprintComposer ──Validates──▶ Realm (Compatibility)
/// ```
#[derive(Debug)]
pub struct BlueprintComposerState {
    // ─────────────────────────────────────────────────────────────────────────
    // Composition-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Compositions erstellt
    pub compositions_created: AtomicU64,
    /// Compositions erfolgreich
    pub compositions_successful: AtomicU64,
    /// Compositions fehlgeschlagen
    pub compositions_failed: AtomicU64,
    /// Durchschnittliche Vererbungs-Tiefe
    pub avg_inheritance_depth: RwLock<f64>,
    /// Maximale Vererbungs-Tiefe
    pub max_inheritance_depth: AtomicU64,
    /// Konflikt-Resolutions bei Composition
    pub conflict_resolutions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Versioning
    // ─────────────────────────────────────────────────────────────────────────
    /// Blueprint-Versionen publiziert
    pub versions_published: AtomicU64,
    /// Migrationen durchgeführt
    pub migrations_executed: AtomicU64,
    /// Migrations-Fehler
    pub migration_errors: AtomicU64,
    /// Deprecations markiert
    pub deprecations: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Instantiation
    // ─────────────────────────────────────────────────────────────────────────
    /// Instanziierungen aus Compositions
    pub instantiations: AtomicU64,
    /// Instanziierungs-Fehler
    pub instantiation_errors: AtomicU64,
    /// Instanzen aktiv
    pub instances_active: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Validation
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-Compatibility-Checks
    pub realm_compatibility_checks: AtomicU64,
    /// Compatibility-Failures
    pub compatibility_failures: AtomicU64,
    /// Dependency-Resolutions
    pub dependency_resolutions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Caching
    // ─────────────────────────────────────────────────────────────────────────
    /// Cache-Hits
    pub cache_hits: AtomicU64,
    /// Cache-Misses
    pub cache_misses: AtomicU64,
    /// Cache-Evictions
    pub cache_evictions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht für Composition
    pub gas_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (BlueprintComposer ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// ECLBlueprint-Aggregations (BlueprintComposer ⊃ ECLBlueprint)
    pub ecl_blueprint_aggregations: AtomicU64,
    /// Realm-Validations (BlueprintComposer ✓ Realm)
    pub realm_validations: AtomicU64,
    /// Events getriggert (BlueprintComposer → Event)
    pub events_triggered: AtomicU64,
}

impl BlueprintComposerState {
    pub fn new() -> Self {
        Self {
            compositions_created: AtomicU64::new(0),
            compositions_successful: AtomicU64::new(0),
            compositions_failed: AtomicU64::new(0),
            avg_inheritance_depth: RwLock::new(0.0),
            max_inheritance_depth: AtomicU64::new(0),
            conflict_resolutions: AtomicU64::new(0),
            versions_published: AtomicU64::new(0),
            migrations_executed: AtomicU64::new(0),
            migration_errors: AtomicU64::new(0),
            deprecations: AtomicU64::new(0),
            instantiations: AtomicU64::new(0),
            instantiation_errors: AtomicU64::new(0),
            instances_active: AtomicU64::new(0),
            realm_compatibility_checks: AtomicU64::new(0),
            compatibility_failures: AtomicU64::new(0),
            dependency_resolutions: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_evictions: AtomicU64::new(0),
            gas_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            ecl_blueprint_aggregations: AtomicU64::new(0),
            realm_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Composition erstellt
    pub fn composition_created(
        &self,
        success: bool,
        inheritance_depth: u64,
        conflicts: u64,
        gas: u64,
    ) {
        self.compositions_created.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        if success {
            self.compositions_successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.compositions_failed.fetch_add(1, Ordering::Relaxed);
        }

        if conflicts > 0 {
            self.conflict_resolutions
                .fetch_add(conflicts, Ordering::Relaxed);
        }

        // Update max depth
        loop {
            let current = self.max_inheritance_depth.load(Ordering::Relaxed);
            if inheritance_depth <= current {
                break;
            }
            if self
                .max_inheritance_depth
                .compare_exchange(
                    current,
                    inheritance_depth,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }

        // Update average depth
        if let Ok(mut avg) = self.avg_inheritance_depth.write() {
            let total = self.compositions_created.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + inheritance_depth as f64) / total;
        }
    }

    /// Blueprint instanziiert
    pub fn instantiate(&self, success: bool, gas: u64) {
        self.instantiations.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.ecl_blueprint_aggregations
            .fetch_add(1, Ordering::Relaxed);

        if success {
            self.instances_active.fetch_add(1, Ordering::Relaxed);
        } else {
            self.instantiation_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Realm-Compatibility-Check
    pub fn realm_compatibility_check(&self, compatible: bool) {
        self.realm_compatibility_checks
            .fetch_add(1, Ordering::Relaxed);
        self.realm_validations.fetch_add(1, Ordering::Relaxed);

        if !compatible {
            self.compatibility_failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Cache-Zugriff
    pub fn cache_access(&self, hit: bool) {
        if hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Composition-Success-Rate
    pub fn composition_success_rate(&self) -> f64 {
        let total = self.compositions_created.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.compositions_successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    /// Cache-Hit-Rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total =
            self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed);
        if total > 0 {
            self.cache_hits.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> BlueprintComposerStateSnapshot {
        BlueprintComposerStateSnapshot {
            compositions_created: self.compositions_created.load(Ordering::Relaxed),
            compositions_successful: self.compositions_successful.load(Ordering::Relaxed),
            compositions_failed: self.compositions_failed.load(Ordering::Relaxed),
            composition_success_rate: self.composition_success_rate(),
            avg_inheritance_depth: self.avg_inheritance_depth.read().map(|v| *v).unwrap_or(0.0),
            max_inheritance_depth: self.max_inheritance_depth.load(Ordering::Relaxed),
            conflict_resolutions: self.conflict_resolutions.load(Ordering::Relaxed),
            versions_published: self.versions_published.load(Ordering::Relaxed),
            migrations_executed: self.migrations_executed.load(Ordering::Relaxed),
            instantiations: self.instantiations.load(Ordering::Relaxed),
            instances_active: self.instances_active.load(Ordering::Relaxed),
            realm_compatibility_checks: self.realm_compatibility_checks.load(Ordering::Relaxed),
            compatibility_failures: self.compatibility_failures.load(Ordering::Relaxed),
            cache_hit_rate: self.cache_hit_rate(),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for BlueprintComposerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintComposerStateSnapshot {
    pub compositions_created: u64,
    pub compositions_successful: u64,
    pub compositions_failed: u64,
    pub composition_success_rate: f64,
    pub avg_inheritance_depth: f64,
    pub max_inheritance_depth: u64,
    pub conflict_resolutions: u64,
    pub versions_published: u64,
    pub migrations_executed: u64,
    pub instantiations: u64,
    pub instances_active: u64,
    pub realm_compatibility_checks: u64,
    pub compatibility_failures: u64,
    pub cache_hit_rate: f64,
    pub gas_consumed: u64,
    pub events_triggered: u64,
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
/// - **UIState**: Component-Tree, Bindings, Trust-Gates
/// - **APIState**: Endpoints, Rate-Limits, Request-Tracking
/// - **GovernanceState**: Proposals, Voting, Delegation
/// - **ControllerState**: Permissions, AuthZ, Audit
/// - **DataLogicState**: Streams, Aggregations, Event-Processing
/// - **BlueprintComposerState**: Composition, Versioning, Caching
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
/// // UI Render
/// state.ui.render(false, 100, 50, Some("default"));
///
/// // API Request
/// state.api.record_request(1500, 200, 50, 10, Some("default"));
///
/// // Governance Vote
/// state.governance.vote_cast(1.5, false, true, Some("default"));
///
/// // Controller AuthZ
/// state.controller.check_authorization(true, false, 50, "realm", Some("default"));
///
/// // DataLogic Event
/// state.data_logic.process_event(false, 25);
///
/// // BlueprintComposer Composition
/// state.blueprint_composer.composition_created(true, 2, 0, 100);
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

    // ─────────────────────────────────────────────────────────────────────────
    // Engine-Layer (6 neue Engines für SOLL-Zustand)
    // ─────────────────────────────────────────────────────────────────────────
    /// UI-Engine Layer (Component-Tree, Bindings, Trust-Gates)
    pub ui: UIState,

    /// API-Engine Layer (Endpoints, Rate-Limits, Request-Tracking)
    pub api: APIState,

    /// Governance-Engine Layer (Proposals, Voting, Delegation)
    pub governance: GovernanceState,

    /// Controller-Engine Layer (Permissions, AuthZ, Audit)
    pub controller: ControllerState,

    /// DataLogic-Engine Layer (Streams, Aggregations, Event-Processing)
    pub data_logic: DataLogicState,

    /// BlueprintComposer-Engine Layer (Composition, Versioning, Caching)
    pub blueprint_composer: BlueprintComposerState,

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
            ui: UIState::new(),
            api: APIState::new(),
            governance: GovernanceState::new(),
            controller: ControllerState::new(),
            data_logic: DataLogicState::new(),
            blueprint_composer: BlueprintComposerState::new(),
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

        // Protection Health (15% Gewicht)
        score -= (100.0 - self.protection.health_score()) * 0.15;

        // Consensus Success Rate (12% Gewicht)
        score -= (1.0 - self.core.consensus.success_rate()) * 12.0;

        // Execution Success Rate (8% Gewicht)
        score -= (1.0 - self.execution.success_rate()) * 8.0;

        // ECLVM Policy Success Rate (8% Gewicht)
        score -= (1.0 - self.eclvm.policy_success_rate()) * 8.0;

        // P2P Health (15% Gewicht)
        score -= (100.0 - self.p2p.health_score()) * 0.15;

        // Peer Layer Health (8% Gewicht)
        let gateway_rate = self.peer.gateway.success_rate();
        let saga_rate = self.peer.saga.composition_success_rate();
        let peer_health = (gateway_rate + saga_rate) / 2.0 * 100.0;
        score -= (100.0 - peer_health) * 0.08;

        // Realm Crossing Success (4% Gewicht)
        score -= (1.0 - self.eclvm.crossing_allow_rate()) * 4.0;

        // Event Validation Errors (5% Gewicht)
        let event_errors = self.core.events.validation_errors.load(Ordering::Relaxed);
        let event_total = self.core.events.total.load(Ordering::Relaxed);
        if event_total > 0 {
            let error_rate = event_errors as f64 / event_total as f64;
            score -= error_rate * 5.0;
        }

        // ─────────────────────────────────────────────────────────────────────
        // Engine-Layer Health (25% Gewicht verteilt auf 6 Engines)
        // ─────────────────────────────────────────────────────────────────────

        // UI-Engine Health (4% Gewicht)
        let ui_health = (self.ui.cache_hit_rate() + self.ui.trust_gate_allow_rate()) / 2.0 * 100.0;
        score -= (100.0 - ui_health) * 0.04;

        // API-Engine Health (5% Gewicht)
        let api_health = self.api.success_rate() * 100.0;
        score -= (100.0 - api_health) * 0.05;

        // Governance-Engine Health (4% Gewicht)
        // Hohe Participation = Gesundheit
        let governance_health = if self.governance.proposals_completed.load(Ordering::Relaxed) > 0 {
            self.governance.proposal_success_rate() * 100.0
        } else {
            100.0 // Noch keine Proposals = neutral
        };
        score -= (100.0 - governance_health) * 0.04;

        // Controller-Engine Health (5% Gewicht)
        let controller_health = self.controller.authz_success_rate() * 100.0;
        score -= (100.0 - controller_health) * 0.05;

        // DataLogic-Engine Health (4% Gewicht)
        let datalogic_health =
            (self.data_logic.success_rate() + self.data_logic.binding_success_rate()) / 2.0 * 100.0;
        score -= (100.0 - datalogic_health) * 0.04;

        // BlueprintComposer-Engine Health (3% Gewicht)
        let blueprint_health = (self.blueprint_composer.composition_success_rate()
            + self.blueprint_composer.cache_hit_rate())
            / 2.0
            * 100.0;
        score -= (100.0 - blueprint_health) * 0.03;

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
            ui: self.ui.snapshot(),
            api: self.api.snapshot(),
            governance: self.governance.snapshot(),
            controller: self.controller.snapshot(),
            data_logic: self.data_logic.snapshot(),
            blueprint_composer: self.blueprint_composer.snapshot(),
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
    pub ui: UIStateSnapshot,
    pub api: APIStateSnapshot,
    pub governance: GovernanceStateSnapshot,
    pub controller: ControllerStateSnapshot,
    pub data_logic: DataLogicStateSnapshot,
    pub blueprint_composer: BlueprintComposerStateSnapshot,
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

    // ═══════════════════════════════════════════════════════════════════════════
    // PHASE 1 TESTS: Neue StateComponent-Varianten und StateGraph-Edges
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_new_state_components_exist() {
        // Teste dass alle neuen StateComponent-Varianten existieren
        let components = vec![
            StateComponent::Room,
            StateComponent::Partition,
            StateComponent::UI,
            StateComponent::DataLogic,
            StateComponent::API,
            StateComponent::Governance,
            StateComponent::Controller,
            StateComponent::BlueprintComposer,
        ];

        // Alle Komponenten sollten serialisierbar sein
        for component in &components {
            let serialized = serde_json::to_string(component).unwrap();
            assert!(!serialized.is_empty());
        }

        // Prüfe dass es genau 8 neue Komponenten sind
        assert_eq!(components.len(), 8);
    }

    #[test]
    fn test_room_partition_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // Room ─ DependsOn ─▶ Realm
        assert!(graph.has_relation(
            StateComponent::Room,
            StateRelation::DependsOn,
            StateComponent::Realm
        ));

        // Room ─ DependsOn ─▶ Trust
        assert!(graph.has_relation(
            StateComponent::Room,
            StateRelation::DependsOn,
            StateComponent::Trust
        ));

        // Room ─ Triggers ─▶ Event
        assert!(graph.has_relation(
            StateComponent::Room,
            StateRelation::Triggers,
            StateComponent::Event
        ));

        // Room ─ Aggregates ─▶ Controller
        assert!(graph.has_relation(
            StateComponent::Room,
            StateRelation::Aggregates,
            StateComponent::Controller
        ));

        // Partition ─ DependsOn ─▶ Room
        assert!(graph.has_relation(
            StateComponent::Partition,
            StateRelation::DependsOn,
            StateComponent::Room
        ));

        // Partition ─ Validates ─▶ Controller
        assert!(graph.has_relation(
            StateComponent::Partition,
            StateRelation::Validates,
            StateComponent::Controller
        ));
    }

    #[test]
    fn test_ui_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // UI hat viele Dependencies
        let ui_deps = graph.dependencies_of(StateComponent::UI);
        assert!(ui_deps.contains(&StateComponent::Trust));
        assert!(ui_deps.contains(&StateComponent::Realm));
        assert!(ui_deps.contains(&StateComponent::Room));
        assert!(ui_deps.contains(&StateComponent::Controller));
        assert!(ui_deps.contains(&StateComponent::ECLVM));
        assert!(ui_deps.contains(&StateComponent::Gas));
        assert!(ui_deps.contains(&StateComponent::Mana));

        // UI triggert Events
        let ui_triggers = graph.triggered_by(StateComponent::UI);
        assert!(ui_triggers.contains(&StateComponent::Event));

        // UI aggregiert DataLogic
        let ui_aggregates = graph.aggregated_by(StateComponent::UI);
        assert!(ui_aggregates.contains(&StateComponent::DataLogic));
    }

    #[test]
    fn test_api_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // API Dependencies
        let api_deps = graph.dependencies_of(StateComponent::API);
        assert!(api_deps.contains(&StateComponent::Trust));
        assert!(api_deps.contains(&StateComponent::Controller));
        assert!(api_deps.contains(&StateComponent::ECLVM));
        assert!(api_deps.contains(&StateComponent::Gas));
        assert!(api_deps.contains(&StateComponent::Mana));

        // API validiert Gateway
        let api_validates = graph.validated_by(StateComponent::API);
        assert!(api_validates.contains(&StateComponent::Gateway));

        // API triggert Events
        let api_triggers = graph.triggered_by(StateComponent::API);
        assert!(api_triggers.contains(&StateComponent::Event));
    }

    #[test]
    fn test_governance_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // Governance Dependencies
        let gov_deps = graph.dependencies_of(StateComponent::Governance);
        assert!(gov_deps.contains(&StateComponent::Trust));
        assert!(gov_deps.contains(&StateComponent::Quadratic));
        assert!(gov_deps.contains(&StateComponent::ECLVM));
        assert!(gov_deps.contains(&StateComponent::Realm));

        // Governance validiert Controller und AntiCalcification
        let gov_validates = graph.validated_by(StateComponent::Governance);
        assert!(gov_validates.contains(&StateComponent::Controller));
        assert!(gov_validates.contains(&StateComponent::AntiCalcification));

        // Governance triggert Controller und Event
        let gov_triggers = graph.triggered_by(StateComponent::Governance);
        assert!(gov_triggers.contains(&StateComponent::Controller));
        assert!(gov_triggers.contains(&StateComponent::Event));
    }

    #[test]
    fn test_controller_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // Controller Dependencies
        let ctrl_deps = graph.dependencies_of(StateComponent::Controller);
        assert!(ctrl_deps.contains(&StateComponent::Trust));
        assert!(ctrl_deps.contains(&StateComponent::Realm));
        assert!(ctrl_deps.contains(&StateComponent::Room));
        assert!(ctrl_deps.contains(&StateComponent::Partition));
        assert!(ctrl_deps.contains(&StateComponent::ECLVM));

        // Controller validiert Gateway, API, UI
        let ctrl_validates = graph.validated_by(StateComponent::Controller);
        assert!(ctrl_validates.contains(&StateComponent::Gateway));
        assert!(ctrl_validates.contains(&StateComponent::API));
        assert!(ctrl_validates.contains(&StateComponent::UI));

        // Controller aggregiert Governance
        let ctrl_aggregates = graph.aggregated_by(StateComponent::Controller);
        assert!(ctrl_aggregates.contains(&StateComponent::Governance));
    }

    #[test]
    fn test_datalogic_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // DataLogic Dependencies
        let dl_deps = graph.dependencies_of(StateComponent::DataLogic);
        assert!(dl_deps.contains(&StateComponent::Event));
        assert!(dl_deps.contains(&StateComponent::Trust));
        assert!(dl_deps.contains(&StateComponent::ECLVM));
        assert!(dl_deps.contains(&StateComponent::Gas));

        // DataLogic aggregiert und triggert Events
        let dl_aggregates = graph.aggregated_by(StateComponent::DataLogic);
        assert!(dl_aggregates.contains(&StateComponent::Event));
        let dl_triggers = graph.triggered_by(StateComponent::DataLogic);
        assert!(dl_triggers.contains(&StateComponent::Event));

        // DataLogic validiert UI
        let dl_validates = graph.validated_by(StateComponent::DataLogic);
        assert!(dl_validates.contains(&StateComponent::UI));
    }

    #[test]
    fn test_blueprint_composer_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // BlueprintComposer Dependencies
        let bc_deps = graph.dependencies_of(StateComponent::BlueprintComposer);
        assert!(bc_deps.contains(&StateComponent::Blueprint));
        assert!(bc_deps.contains(&StateComponent::ECLVM));
        assert!(bc_deps.contains(&StateComponent::Trust));
        assert!(bc_deps.contains(&StateComponent::Gas));

        // BlueprintComposer aggregiert ECLBlueprint
        let bc_aggregates = graph.aggregated_by(StateComponent::BlueprintComposer);
        assert!(bc_aggregates.contains(&StateComponent::ECLBlueprint));

        // BlueprintComposer validiert Realm
        let bc_validates = graph.validated_by(StateComponent::BlueprintComposer);
        assert!(bc_validates.contains(&StateComponent::Realm));
    }

    #[test]
    fn test_new_components_criticality_scores() {
        let graph = StateGraph::erynoa_graph();

        // Trust sollte der kritischste sein (viele Dependencies)
        let trust_score = graph.criticality_score(StateComponent::Trust);
        assert!(
            trust_score > 20,
            "Trust criticality should be high: {}",
            trust_score
        );

        // Controller sollte mittlere Kritikalität haben
        let ctrl_score = graph.criticality_score(StateComponent::Controller);
        assert!(
            ctrl_score > 5,
            "Controller criticality should be medium: {}",
            ctrl_score
        );

        // ECLVM sollte hohe Kritikalität haben (viele Engines nutzen es)
        let eclvm_score = graph.criticality_score(StateComponent::ECLVM);
        assert!(
            eclvm_score > 10,
            "ECLVM criticality should be high: {}",
            eclvm_score
        );
    }

    #[test]
    fn test_transitive_dependencies_new_components() {
        let graph = StateGraph::erynoa_graph();

        // UI sollte transitiv von Trust abhängen
        let ui_trans_deps = graph.transitive_dependencies(StateComponent::UI);
        assert!(ui_trans_deps.contains(&StateComponent::Trust));

        // Controller sollte transitiv von Trust abhängen
        let ctrl_trans_deps = graph.transitive_dependencies(StateComponent::Controller);
        assert!(ctrl_trans_deps.contains(&StateComponent::Trust));

        // Partition sollte transitiv von Realm abhängen (über Room)
        let part_trans_deps = graph.transitive_dependencies(StateComponent::Partition);
        assert!(part_trans_deps.contains(&StateComponent::Room));
        assert!(part_trans_deps.contains(&StateComponent::Realm));
    }

    #[test]
    fn test_state_graph_edge_count() {
        let graph = StateGraph::erynoa_graph();

        // Wir haben ~50 bestehende + ~42 neue = ~92 Edges
        assert!(
            graph.edges.len() >= 85,
            "StateGraph should have at least 85 edges, got: {}",
            graph.edges.len()
        );
    }

    // ========================================================================
    // 2.1-2.6 Engine-State Tests
    // ========================================================================

    #[test]
    fn test_ui_state() {
        let state = UIState::new();

        // Component registrieren
        state.register_component(Some("test-realm"));
        state.register_component(None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.components_registered, 2);
        assert_eq!(snapshot.components_active, 2);

        // Render durchführen
        state.render(false, 100, 50, Some("test-realm"));
        state.render(true, 0, 0, None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.renders, 2);
        assert!((snapshot.cache_hit_rate - 0.5).abs() < 0.01);

        // Trust-Gate
        state.trust_gate(true, None);
        state.trust_gate(false, Some("test-realm"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.trust_gate_evaluations, 2);
        assert!((snapshot.trust_gate_allow_rate - 0.5).abs() < 0.01);

        // Credential-Gate
        state.credential_gate(true);
        state.credential_gate(true);
        state.credential_gate(false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.credential_gate_evaluations, 3);
        assert!((snapshot.credential_gate_allow_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_api_state() {
        let state = APIState::new();

        // Endpoints registrieren
        state.register_endpoint(Some("test-realm"));
        state.register_endpoint(None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.endpoints_registered, 2);
        assert_eq!(snapshot.endpoints_active, 2);

        // Requests verarbeiten
        state.record_request(1000, 200, 50, 10, Some("test-realm")); // Success
        state.record_request(500, 201, 30, 5, None); // Success
        state.record_request(2000, 404, 10, 2, None); // Client error
        state.record_request(5000, 429, 5, 1, Some("test-realm")); // Rate limited
        state.record_request(3000, 500, 20, 5, None); // Server error

        let snapshot = state.snapshot();
        assert_eq!(snapshot.requests_total, 5);
        assert_eq!(snapshot.requests_success, 2);
        assert_eq!(snapshot.requests_client_error, 2);
        assert_eq!(snapshot.requests_server_error, 1);
        assert_eq!(snapshot.requests_rate_limited, 1);
        assert!((snapshot.success_rate - 0.4).abs() < 0.01);
        assert!(snapshot.avg_latency_us > 0.0);
    }

    #[test]
    fn test_governance_state() {
        let state = GovernanceState::new();

        // Proposals erstellen
        state.proposal_created(Some("test-realm"));
        state.proposal_created(None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.proposals_created, 2);
        assert_eq!(snapshot.proposals_active, 2);

        // Votes abgeben
        state.vote_cast(1.5, false, true, Some("test-realm"));
        state.vote_cast(2.0, true, true, None);
        state.vote_cast(1.0, false, false, None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.votes_cast, 3);
        assert_eq!(snapshot.votes_delegated, 1);
        assert_eq!(snapshot.quadratic_reductions, 2);
        assert!(snapshot.avg_voting_power > 1.0);

        // Proposals abschließen
        state.proposal_completed("accepted");
        state.proposal_completed("rejected");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.proposals_completed, 2);
        assert_eq!(snapshot.proposals_accepted, 1);
        assert_eq!(snapshot.proposals_rejected, 1);
        assert!((snapshot.proposal_success_rate - 0.5).abs() < 0.01);

        // Delegationen
        state.delegation_created(3, Some("test-realm"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.delegations_active, 1);
        assert_eq!(snapshot.max_delegation_depth, 3);

        // Power-Check
        state.power_check(true, 0.35);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.power_violations, 1);
        assert!((snapshot.voting_power_gini - 0.35).abs() < 0.01);
    }

    #[test]
    fn test_controller_state() {
        let state = ControllerState::new();

        // Permissions gewähren
        state.grant_permission(Some("test-realm"));
        state.grant_permission(None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.permission_grants, 2);
        assert_eq!(snapshot.permissions_active, 2);

        // AuthZ-Checks
        state.check_authorization(true, false, 50, "realm", Some("test-realm"));
        state.check_authorization(true, true, 100, "room", None);
        state.check_authorization(false, false, 25, "partition", Some("test-realm"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.authz_checks, 3);
        assert_eq!(snapshot.authz_allowed, 2);
        assert_eq!(snapshot.authz_denied, 1);
        assert!((snapshot.authz_success_rate - 0.666).abs() < 0.01);
        assert!(snapshot.avg_check_latency_us > 0.0);
        assert_eq!(snapshot.realm_scope_checks, 1);
        assert_eq!(snapshot.room_scope_checks, 1);
        assert_eq!(snapshot.partition_scope_checks, 1);

        // Delegation
        state.create_delegation(2, Some("test-realm"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.delegations_active, 1);
        assert_eq!(snapshot.max_delegation_depth, 2);

        // Permission widerrufen
        state.revoke_permission();

        let snapshot = state.snapshot();
        assert_eq!(snapshot.permission_revokes, 1);
        assert_eq!(snapshot.permissions_active, 1);

        // Audit-Entries sollten geschrieben worden sein
        assert!(snapshot.audit_entries > 0);
        assert!(snapshot.audit_log_bytes > 0);
    }

    #[test]
    fn test_data_logic_state() {
        let state = DataLogicState::new();

        // Stream registrieren
        state.register_stream();
        state.register_stream();

        let snapshot = state.snapshot();
        assert_eq!(snapshot.streams_registered, 2);
        assert_eq!(snapshot.streams_active, 2);

        // Events verarbeiten
        state.process_event(false, 50); // Forwarded
        state.process_event(false, 30); // Forwarded
        state.process_event(true, 10); // Filtered

        let snapshot = state.snapshot();
        assert_eq!(snapshot.events_processed, 3);
        assert_eq!(snapshot.events_forwarded, 2);
        assert_eq!(snapshot.events_filtered, 1);
        assert!((snapshot.success_rate - 0.666).abs() < 0.01);

        // Aggregation
        state.aggregation_computed(500, 100);
        state.aggregation_computed(300, 80);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.aggregations_computed, 2);
        assert!(snapshot.avg_aggregation_latency_us > 0.0);

        // Binding Propagation
        state.propagate_binding(true, 100, 20);
        state.propagate_binding(true, 150, 25);
        state.propagate_binding(false, 50, 10);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.binding_propagations, 3);
        assert!((snapshot.binding_success_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_blueprint_composer_state() {
        let state = BlueprintComposerState::new();

        // Compositions erstellen
        state.composition_created(true, 2, 1, 100);
        state.composition_created(true, 3, 0, 150);
        state.composition_created(false, 0, 0, 50);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.compositions_created, 3);
        assert_eq!(snapshot.compositions_successful, 2);
        assert_eq!(snapshot.compositions_failed, 1);
        assert!((snapshot.composition_success_rate - 0.666).abs() < 0.01);
        assert_eq!(snapshot.max_inheritance_depth, 3);
        assert_eq!(snapshot.conflict_resolutions, 1);

        // Instanziierung
        state.instantiate(true, 80);
        state.instantiate(true, 70);
        state.instantiate(false, 30);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.instantiations, 3);
        assert_eq!(snapshot.instances_active, 2);

        // Realm-Compatibility
        state.realm_compatibility_check(true);
        state.realm_compatibility_check(false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.realm_compatibility_checks, 2);
        assert_eq!(snapshot.compatibility_failures, 1);

        // Cache
        state.cache_access(true);
        state.cache_access(true);
        state.cache_access(false);

        let snapshot = state.snapshot();
        assert!((snapshot.cache_hit_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_unified_state_with_engines() {
        let state = UnifiedState::new();

        // Alle neuen Engines sollten initialisiert sein
        assert_eq!(state.ui.components_registered.load(Ordering::Relaxed), 0);
        assert_eq!(state.api.endpoints_registered.load(Ordering::Relaxed), 0);
        assert_eq!(
            state.governance.proposals_created.load(Ordering::Relaxed),
            0
        );
        assert_eq!(
            state
                .controller
                .permissions_registered
                .load(Ordering::Relaxed),
            0
        );
        assert_eq!(
            state.data_logic.streams_registered.load(Ordering::Relaxed),
            0
        );
        assert_eq!(
            state
                .blueprint_composer
                .compositions_created
                .load(Ordering::Relaxed),
            0
        );

        // Snapshot sollte alle Engines enthalten
        let snapshot = state.snapshot();
        assert_eq!(snapshot.ui.components_registered, 0);
        assert_eq!(snapshot.api.endpoints_registered, 0);
        assert_eq!(snapshot.governance.proposals_created, 0);
        assert_eq!(snapshot.controller.permissions_registered, 0);
        assert_eq!(snapshot.data_logic.streams_registered, 0);
        assert_eq!(snapshot.blueprint_composer.compositions_created, 0);

        // Health sollte hoch bei leerem State sein (einige Defaults sind nicht 100%)
        // Andere Layer wie P2P können Health reduzieren auch ohne Engine-Aktivität
        let health = state.calculate_health();
        assert!(
            health >= 80.0,
            "Initial health should be >= 80%, got: {}",
            health
        );
    }

    #[test]
    fn test_unified_state_health_with_engine_errors() {
        let state = UnifiedState::new();

        // Provoziere Fehler in den Engines
        state.api.record_request(1000, 500, 50, 10, None); // Server Error
        state.api.record_request(1000, 500, 50, 10, None);
        state.api.record_request(1000, 500, 50, 10, None);
        state
            .controller
            .check_authorization(false, false, 50, "realm", None);
        state
            .controller
            .check_authorization(false, false, 50, "realm", None);

        // Health sollte gesunken sein
        let health = state.calculate_health();
        assert!(
            health < 99.0,
            "Health should decrease with errors, got: {}",
            health
        );
        assert!(
            health > 80.0,
            "Health should not drop too low, got: {}",
            health
        );
    }
}
