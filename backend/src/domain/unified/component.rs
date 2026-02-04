//! # State-Komponenten und Layer
//!
//! Definiert State-Komponenten-Identifikatoren für das Erynoa-System.
//!
//! ## Typen
//!
//! - [`StateComponent`]: Identifikator für alle State-Komponenten
//! - [`StateRelation`]: Beziehungstypen zwischen Komponenten
//! - [`ComponentLayer`]: Logische Gruppierung von Komponenten
//!
//! ## Architektur
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────┐
//! │                      IDENTITY-LAYER                           │
//! │  Identity │ Credential │ KeyManagement                        │
//! ├────────────────────────────────────────────────────────────────┤
//! │                        CORE-LAYER                              │
//! │  Trust │ Event │ WorldFormula │ Consensus                     │
//! ├────────────────────────────────────────────────────────────────┤
//! │                     EXECUTION-LAYER                            │
//! │  Gas │ Mana │ Execution │ ECLVM │ ECLPolicy │ ECLBlueprint    │
//! ├────────────────────────────────────────────────────────────────┤
//! │                    PROTECTION-LAYER                            │
//! │  Anomaly │ Diversity │ Quadratic │ AntiCalcification │ Calib  │
//! ├────────────────────────────────────────────────────────────────┤
//! │                      STORAGE-LAYER                             │
//! │  KvStore │ EventStore │ Archive │ Blueprint                   │
//! ├────────────────────────────────────────────────────────────────┤
//! │                        PEER-LAYER                              │
//! │  Gateway │ SagaComposer │ IntentParser │ Realm │ Room │ Part  │
//! ├────────────────────────────────────────────────────────────────┤
//! │                        P2P-LAYER                               │
//! │  Swarm │ Gossip │ Kademlia │ Relay │ NatTraversal │ Privacy   │
//! ├────────────────────────────────────────────────────────────────┤
//! │                      ENGINE-LAYER                              │
//! │  UI │ DataLogic │ API │ Governance │ Controller │ BPComposer  │
//! └────────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// StateComponent - Komponenten-Identifikator
// ============================================================================

/// State-Komponenten-Identifikator
///
/// Identifiziert alle Komponenten des Erynoa-State-Systems für:
/// - Event-Sourcing: Welche Komponente hat Event erzeugt
/// - Dependency-Tracking: Beziehungen zwischen Komponenten
/// - Monitoring: Metriken pro Komponente
/// - Circuit Breaker: Komponenten-spezifische Isolation
///
/// ## Layer-Zuordnung
///
/// | Layer | Komponenten |
/// |-------|-------------|
/// | Identity | Identity, Credential, KeyManagement |
/// | Core | Trust, Event, WorldFormula, Consensus |
/// | Execution | Gas, Mana, Execution, ECLVM, ECLPolicy, ECLBlueprint |
/// | Protection | Anomaly, Diversity, Quadratic, AntiCalcification, Calibration |
/// | Storage | KvStore, EventStore, Archive, Blueprint |
/// | Peer | Gateway, SagaComposer, IntentParser, Realm, Room, Partition |
/// | P2P | Swarm, Gossip, Kademlia, Relay, NatTraversal, Privacy |
/// | Engine | UI, DataLogic, API, Governance, Controller, BlueprintComposer |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StateComponent {
    // ========================================================================
    // IDENTITY-LAYER (Κ6-Κ8)
    // ========================================================================
    /// Identity: DID-Management, Root-DIDs, Sub-DIDs (Κ6-Κ8)
    Identity,
    /// Credential: Verifiable Credentials, Attestations
    Credential,
    /// KeyManagement: Key-Rotation, Recovery, Hardware-Security
    KeyManagement,

    // ========================================================================
    // CORE-LAYER
    // ========================================================================
    /// Trust-Engine: Trust-Vektoren, Reputation (Κ2-Κ5)
    Trust,
    /// Event-Engine: Kausale Events, DAG (Κ9-Κ12)
    Event,
    /// WorldFormula: Berechnungen nach Weltformel (Κ15a-d)
    WorldFormula,
    /// Consensus: BFT-Konsens, Finalisierung
    Consensus,

    // ========================================================================
    // EXECUTION-LAYER
    // ========================================================================
    /// Gas-Metering: Multi-Layer Gas-Tracking
    Gas,
    /// Mana-System: Regeneratives Resource-Budget
    Mana,
    /// Execution: ExecutionContext-Management
    Execution,
    /// ECLVM: Cost-limited Execution Environment für ECL
    ECLVM,
    /// ECL Policy-Engine: Rules, Crossing-Policies
    ECLPolicy,
    /// ECL Blueprint-Management: Templates, Instantiation
    ECLBlueprint,

    // ========================================================================
    // PROTECTION-LAYER
    // ========================================================================
    /// Anomaly-Detection: Verhaltensanalyse, Outlier-Detection
    Anomaly,
    /// Diversity-Monitoring: Gini-Koeffizient, Dezentralisierung (Κ19)
    Diversity,
    /// Quadratic-Mechanisms: Quadratic Voting/Funding (Κ21)
    Quadratic,
    /// AntiCalcification: Aktivitäts-Decay, Rotation
    AntiCalcification,
    /// Calibration: Parameter-Tuning, Self-Healing
    Calibration,

    // ========================================================================
    // STORAGE-LAYER
    // ========================================================================
    /// KV-Store: Persistenter Key-Value Store
    KvStore,
    /// Event-Store: Event-Sourcing Backend
    EventStore,
    /// Archive: Langzeit-Archivierung
    Archive,
    /// Blueprint-Store: Blueprint-Templates und Instanzen
    Blueprint,

    // ========================================================================
    // PEER-LAYER (Κ22-Κ24)
    // ========================================================================
    /// Gateway: Realm-Crossing-Koordination (Κ23)
    Gateway,
    /// SagaComposer: Multi-Step-Transaction-Orchestrierung (Κ24)
    SagaComposer,
    /// IntentParser: Intent-zu-Saga-Transformation (Κ22)
    IntentParser,
    /// Realm: Realm-Isolation und per-Realm State (Κ1)
    Realm,
    /// Room: Sub-Realm-Isolation mit eigenem Controller-Scope (Κ22)
    Room,
    /// Partition: Trust-basierte Berechtigungspartition innerhalb eines Rooms
    Partition,

    // ========================================================================
    // P2P NETWORK-LAYER
    // ========================================================================
    /// Swarm: libp2p Swarm-Management
    Swarm,
    /// Gossip: GossipSub Protokoll
    Gossip,
    /// Kademlia: DHT für Peer-Discovery
    Kademlia,
    /// Relay: Circuit-Relay für NAT-Traversal
    Relay,
    /// NatTraversal: NAT-Hole-Punching
    NatTraversal,
    /// Privacy: Onion-Routing, Cover-Traffic
    Privacy,

    // ========================================================================
    // ENGINE-LAYER (6 Engines für SOLL-Zustand)
    // ========================================================================
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

impl StateComponent {
    /// Hole den Layer zu dem diese Komponente gehört
    pub fn layer(&self) -> ComponentLayer {
        match self {
            // Identity Layer
            StateComponent::Identity
            | StateComponent::Credential
            | StateComponent::KeyManagement => ComponentLayer::Identity,

            // Core Layer
            StateComponent::Trust
            | StateComponent::Event
            | StateComponent::WorldFormula
            | StateComponent::Consensus => ComponentLayer::Core,

            // Execution Layer
            StateComponent::Gas
            | StateComponent::Mana
            | StateComponent::Execution
            | StateComponent::ECLVM
            | StateComponent::ECLPolicy
            | StateComponent::ECLBlueprint => ComponentLayer::Execution,

            // Protection Layer
            StateComponent::Anomaly
            | StateComponent::Diversity
            | StateComponent::Quadratic
            | StateComponent::AntiCalcification
            | StateComponent::Calibration => ComponentLayer::Protection,

            // Storage Layer
            StateComponent::KvStore
            | StateComponent::EventStore
            | StateComponent::Archive
            | StateComponent::Blueprint => ComponentLayer::Storage,

            // Peer Layer
            StateComponent::Gateway
            | StateComponent::SagaComposer
            | StateComponent::IntentParser
            | StateComponent::Realm
            | StateComponent::Room
            | StateComponent::Partition => ComponentLayer::Peer,

            // P2P Layer
            StateComponent::Swarm
            | StateComponent::Gossip
            | StateComponent::Kademlia
            | StateComponent::Relay
            | StateComponent::NatTraversal
            | StateComponent::Privacy => ComponentLayer::P2P,

            // Engine Layer
            StateComponent::UI
            | StateComponent::DataLogic
            | StateComponent::API
            | StateComponent::Governance
            | StateComponent::Controller
            | StateComponent::BlueprintComposer => ComponentLayer::Engine,
        }
    }

    /// Prüfe ob Komponente zum Core-Layer gehört
    #[inline]
    pub fn is_core(&self) -> bool {
        matches!(self.layer(), ComponentLayer::Core)
    }

    /// Prüfe ob Komponente zum Protection-Layer gehört
    #[inline]
    pub fn is_protection(&self) -> bool {
        matches!(self.layer(), ComponentLayer::Protection)
    }

    /// Prüfe ob Komponente zum Identity-Layer gehört
    #[inline]
    pub fn is_identity(&self) -> bool {
        matches!(self.layer(), ComponentLayer::Identity)
    }

    /// Prüfe ob Komponente zum P2P-Layer gehört
    #[inline]
    pub fn is_p2p(&self) -> bool {
        matches!(self.layer(), ComponentLayer::P2P)
    }

    /// Prüfe ob Komponente kritisch für System-Stabilität ist
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            StateComponent::Trust
                | StateComponent::Consensus
                | StateComponent::Identity
                | StateComponent::Anomaly
                | StateComponent::Gateway
        )
    }

    /// Human-readable Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            // Identity
            StateComponent::Identity => "DID and identity management",
            StateComponent::Credential => "Verifiable credentials",
            StateComponent::KeyManagement => "Key rotation and recovery",

            // Core
            StateComponent::Trust => "Trust vector computation",
            StateComponent::Event => "Causal event DAG",
            StateComponent::WorldFormula => "World formula calculations",
            StateComponent::Consensus => "BFT consensus",

            // Execution
            StateComponent::Gas => "Multi-layer gas metering",
            StateComponent::Mana => "Regenerative resource budget",
            StateComponent::Execution => "Execution context management",
            StateComponent::ECLVM => "ECL virtual machine",
            StateComponent::ECLPolicy => "ECL policy engine",
            StateComponent::ECLBlueprint => "ECL blueprint management",

            // Protection
            StateComponent::Anomaly => "Anomaly detection",
            StateComponent::Diversity => "Diversity monitoring",
            StateComponent::Quadratic => "Quadratic mechanisms",
            StateComponent::AntiCalcification => "Activity decay",
            StateComponent::Calibration => "Parameter tuning",

            // Storage
            StateComponent::KvStore => "Key-value store",
            StateComponent::EventStore => "Event sourcing backend",
            StateComponent::Archive => "Long-term archive",
            StateComponent::Blueprint => "Blueprint storage",

            // Peer
            StateComponent::Gateway => "Realm crossing coordination",
            StateComponent::SagaComposer => "Multi-step transaction orchestration",
            StateComponent::IntentParser => "Intent to saga transformation",
            StateComponent::Realm => "Realm isolation",
            StateComponent::Room => "Sub-realm isolation",
            StateComponent::Partition => "Trust-based partitioning",

            // P2P
            StateComponent::Swarm => "libp2p swarm management",
            StateComponent::Gossip => "GossipSub protocol",
            StateComponent::Kademlia => "DHT peer discovery",
            StateComponent::Relay => "Circuit relay",
            StateComponent::NatTraversal => "NAT hole punching",
            StateComponent::Privacy => "Onion routing",

            // Engine
            StateComponent::UI => "Trust-based UI rendering",
            StateComponent::DataLogic => "Reactive event processing",
            StateComponent::API => "Dynamic API definition",
            StateComponent::Governance => "DAO governance",
            StateComponent::Controller => "Permission management",
            StateComponent::BlueprintComposer => "Template composition",
        }
    }
}

impl std::fmt::Display for StateComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ============================================================================
// ComponentLayer - Logische Gruppierung
// ============================================================================

/// Logische Layer-Gruppierung von Komponenten
///
/// Definiert die hierarchische Schichtung des Erynoa-Systems.
/// Höhere Layer können auf niedrigere zugreifen, aber nicht umgekehrt.
///
/// ## Hierarchie (von unten nach oben)
///
/// 1. `Storage` - Persistenz-Layer
/// 2. `P2P` - Netzwerk-Layer
/// 3. `Identity` - Identitäts-Layer
/// 4. `Core` - Kern-Business-Logic
/// 5. `Execution` - Ausführungs-Layer
/// 6. `Protection` - Schutz-Layer
/// 7. `Peer` - Realm/Saga-Layer
/// 8. `Engine` - Anwendungs-Layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComponentLayer {
    /// Identity-Layer: DID, Credentials, Keys
    Identity,
    /// Core-Layer: Trust, Events, Consensus
    Core,
    /// Execution-Layer: Gas, Mana, ECLVM
    Execution,
    /// Protection-Layer: Anomaly, Diversity
    Protection,
    /// Storage-Layer: KV, Events, Archive
    Storage,
    /// Peer-Layer: Gateway, Saga, Realm
    Peer,
    /// P2P-Layer: Swarm, Gossip, DHT
    P2P,
    /// Engine-Layer: UI, API, Governance
    Engine,
}

impl ComponentLayer {
    /// Hole alle Komponenten in diesem Layer
    pub fn components(&self) -> &'static [StateComponent] {
        match self {
            ComponentLayer::Identity => &[
                StateComponent::Identity,
                StateComponent::Credential,
                StateComponent::KeyManagement,
            ],
            ComponentLayer::Core => &[
                StateComponent::Trust,
                StateComponent::Event,
                StateComponent::WorldFormula,
                StateComponent::Consensus,
            ],
            ComponentLayer::Execution => &[
                StateComponent::Gas,
                StateComponent::Mana,
                StateComponent::Execution,
                StateComponent::ECLVM,
                StateComponent::ECLPolicy,
                StateComponent::ECLBlueprint,
            ],
            ComponentLayer::Protection => &[
                StateComponent::Anomaly,
                StateComponent::Diversity,
                StateComponent::Quadratic,
                StateComponent::AntiCalcification,
                StateComponent::Calibration,
            ],
            ComponentLayer::Storage => &[
                StateComponent::KvStore,
                StateComponent::EventStore,
                StateComponent::Archive,
                StateComponent::Blueprint,
            ],
            ComponentLayer::Peer => &[
                StateComponent::Gateway,
                StateComponent::SagaComposer,
                StateComponent::IntentParser,
                StateComponent::Realm,
                StateComponent::Room,
                StateComponent::Partition,
            ],
            ComponentLayer::P2P => &[
                StateComponent::Swarm,
                StateComponent::Gossip,
                StateComponent::Kademlia,
                StateComponent::Relay,
                StateComponent::NatTraversal,
                StateComponent::Privacy,
            ],
            ComponentLayer::Engine => &[
                StateComponent::UI,
                StateComponent::DataLogic,
                StateComponent::API,
                StateComponent::Governance,
                StateComponent::Controller,
                StateComponent::BlueprintComposer,
            ],
        }
    }

    /// Human-readable Beschreibung
    pub fn description(&self) -> &'static str {
        match self {
            ComponentLayer::Identity => "Identity and credential management",
            ComponentLayer::Core => "Core business logic",
            ComponentLayer::Execution => "Execution and resource management",
            ComponentLayer::Protection => "System protection and monitoring",
            ComponentLayer::Storage => "Persistence layer",
            ComponentLayer::Peer => "Realm and saga coordination",
            ComponentLayer::P2P => "Peer-to-peer networking",
            ComponentLayer::Engine => "Application engines",
        }
    }
}

impl std::fmt::Display for ComponentLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// ============================================================================
// StateRelation - Beziehungstypen
// ============================================================================

/// Beziehungstyp zwischen State-Komponenten
///
/// Verwendet für:
/// - Dependency-Graph-Konstruktion
/// - Impact-Analyse bei Änderungen
/// - Visualisierung der System-Architektur
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

impl StateRelation {
    /// Hole das inverse Relation (falls existent)
    pub fn inverse(&self) -> Option<Self> {
        match self {
            StateRelation::DependsOn => Some(StateRelation::Triggers),
            StateRelation::Triggers => Some(StateRelation::DependsOn),
            StateRelation::Bidirectional => Some(StateRelation::Bidirectional),
            StateRelation::Aggregates => None,
            StateRelation::Validates => None,
        }
    }

    /// Prüfe ob Relation transitiv ist
    pub fn is_transitive(&self) -> bool {
        matches!(
            self,
            StateRelation::DependsOn | StateRelation::Triggers | StateRelation::Aggregates
        )
    }

    /// ASCII-Symbol für Visualisierung
    pub fn symbol(&self) -> &'static str {
        match self {
            StateRelation::DependsOn => "←",
            StateRelation::Triggers => "→",
            StateRelation::Bidirectional => "↔",
            StateRelation::Aggregates => "⊃",
            StateRelation::Validates => "✓",
        }
    }
}

impl std::fmt::Display for StateRelation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

// ============================================================================
// Compile-Time Size Checks
// ============================================================================

const _: () = {
    // StateComponent sollte klein bleiben (1-2 bytes)
    assert!(std::mem::size_of::<StateComponent>() <= 2);
    assert!(std::mem::size_of::<ComponentLayer>() == 1);
    assert!(std::mem::size_of::<StateRelation>() == 1);
};

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_layer_mapping() {
        assert_eq!(StateComponent::Trust.layer(), ComponentLayer::Core);
        assert_eq!(StateComponent::Identity.layer(), ComponentLayer::Identity);
        assert_eq!(StateComponent::Gateway.layer(), ComponentLayer::Peer);
        assert_eq!(StateComponent::Swarm.layer(), ComponentLayer::P2P);
        assert_eq!(StateComponent::UI.layer(), ComponentLayer::Engine);
    }

    #[test]
    fn test_component_is_critical() {
        assert!(StateComponent::Trust.is_critical());
        assert!(StateComponent::Consensus.is_critical());
        assert!(StateComponent::Identity.is_critical());
        assert!(!StateComponent::Archive.is_critical());
    }

    #[test]
    fn test_component_layer_predicates() {
        assert!(StateComponent::Trust.is_core());
        assert!(StateComponent::Identity.is_identity());
        assert!(StateComponent::Anomaly.is_protection());
        assert!(StateComponent::Swarm.is_p2p());
    }

    #[test]
    fn test_layer_components() {
        let core_components = ComponentLayer::Core.components();
        assert!(core_components.contains(&StateComponent::Trust));
        assert!(core_components.contains(&StateComponent::Event));
        assert!(!core_components.contains(&StateComponent::Gateway));
    }

    #[test]
    fn test_relation_inverse() {
        assert_eq!(
            StateRelation::DependsOn.inverse(),
            Some(StateRelation::Triggers)
        );
        assert_eq!(
            StateRelation::Triggers.inverse(),
            Some(StateRelation::DependsOn)
        );
        assert_eq!(
            StateRelation::Bidirectional.inverse(),
            Some(StateRelation::Bidirectional)
        );
        assert_eq!(StateRelation::Aggregates.inverse(), None);
    }

    #[test]
    fn test_relation_symbols() {
        assert_eq!(StateRelation::DependsOn.symbol(), "←");
        assert_eq!(StateRelation::Triggers.symbol(), "→");
        assert_eq!(StateRelation::Bidirectional.symbol(), "↔");
    }

    #[test]
    fn test_serde_roundtrip() {
        let component = StateComponent::Gateway;
        let json = serde_json::to_string(&component).unwrap();
        let parsed: StateComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(component, parsed);

        let layer = ComponentLayer::Peer;
        let json = serde_json::to_string(&layer).unwrap();
        let parsed: ComponentLayer = serde_json::from_str(&json).unwrap();
        assert_eq!(layer, parsed);
    }

    #[test]
    fn test_all_components_have_layer() {
        // Ensure all components return a valid layer
        let components = [
            StateComponent::Identity,
            StateComponent::Trust,
            StateComponent::Gas,
            StateComponent::Anomaly,
            StateComponent::KvStore,
            StateComponent::Gateway,
            StateComponent::Swarm,
            StateComponent::UI,
        ];

        for component in components {
            let _ = component.layer(); // Should not panic
            let _ = component.description(); // Should not panic
        }
    }
}
