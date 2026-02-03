//! # Erynoa Core Logic Layer
//!
//! Implementiert die Business-Logik gemäß V4.1 Axiomen.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        CORE LOGIC LAYER                            │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  state            - Unified State Management (ALL modules)         │
//! │  state_integration- Observer-based State Integration               │
//! │  event_engine     - Event-Verarbeitung (Κ9-Κ12)                    │
//! │  trust_engine     - Trust-Berechnung (Κ2-Κ5)                       │
//! │  surprisal        - Surprisal-Berechnung (Κ15a)                    │
//! │  world_formula    - Weltformel-Engine (Κ15b-d)                     │
//! │  consensus        - Konsensus-Mechanismus (Κ18)                    │
//! │  engine           - ExecutionContext-aware Wrapper (Phase 3)       │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

pub mod consensus;
pub mod engine;
pub mod event_engine;
pub mod state;
pub mod state_coordination;
pub mod state_integration;
pub mod surprisal;
pub mod trust_engine;
pub mod world_formula;

// Re-exports - Legacy engines
pub use consensus::ConsensusEngine;
pub use event_engine::EventEngine;
pub use surprisal::SurprisalCalculator;
pub use trust_engine::TrustEngine;
pub use world_formula::WorldFormulaEngine;

// Re-exports - Unified State Management
pub use state::{
    create_unified_state,
    // ECLVM State (Erynoa Core Language Virtual Machine)
    BlueprintStatus,
    // Core State
    ConsensusState,
    ConsensusStateSnapshot,
    CoreState,
    CoreStateSnapshot,
    ECLPolicyType,
    ECLVMState,
    ECLVMStateSnapshot,
    EventState,
    EventStateSnapshot,
    ExecutionState,
    ExecutionStateSnapshot,
    FormulaState,
    FormulaStateSnapshot,
    GatewayState,
    GatewayStateSnapshot,
    GossipState,
    GossipStateSnapshot,
    IntentParserState,
    IntentParserStateSnapshot,
    KademliaState,
    KademliaStateSnapshot,
    NatStatus,
    // P2P State
    P2PState,
    P2PStateSnapshot,
    // Peer State (Κ22-Κ24)
    PeerState,
    PeerStateSnapshot,
    PrivacyState,
    PrivacyStateSnapshot,
    ProtectionState,
    ProtectionStateSnapshot,
    RealmECLState,
    RealmECLStateSnapshot,
    // Realm State (Κ22-Κ24) - Per-Realm Isolation
    RealmSpecificState,
    RealmSpecificStateSnapshot,
    RealmState,
    RealmStateSnapshot,
    RelayState,
    RelayStateSnapshot,
    SagaComposerState,
    SagaComposerStateSnapshot,
    SharedUnifiedState,
    StateComponent,
    StateGraph,
    StateRelation,
    StorageState,
    StorageStateSnapshot,
    SwarmState as CoreSwarmState,
    SwarmStateSnapshot as CoreSwarmStateSnapshot,
    TrustDistribution,
    TrustState,
    TrustStateSnapshot,
    UnifiedState,
    UnifiedStateSnapshot,
};

// Re-exports - State Integration (Observer Pattern)
pub use state_integration::{
    CompositeObserver,
    // Core Observers
    ConsensusObserver,
    // ECLVM Observer (ECL/ECLVM Integration)
    ECLVMObserver,
    EventObserver,
    ExecutionObserver,
    FormulaObserver,
    // Peer Observers (Κ22-Κ24)
    GatewayObserver,
    GossipObserver,
    IntentObserver,
    KademliaObserver,
    ObservableEngine,
    PrivacyObserver,
    ProtectionObserver,
    // Realm Observer (Κ22-Κ24)
    RealmObserver,
    RelayObserver,
    SagaObserver,
    // Shared Types
    SharedConsensusObserver,
    // ECLVM Shared Observer
    SharedECLVMObserver,
    SharedEventObserver,
    SharedExecutionObserver,
    SharedFormulaObserver,
    SharedGatewayObserver,
    SharedGossipObserver,
    SharedIntentObserver,
    SharedKademliaObserver,
    SharedPrivacyObserver,
    SharedProtectionObserver,
    SharedRealmObserver,
    SharedRelayObserver,
    SharedSagaObserver,
    SharedStorageObserver,
    SharedSwarmObserver,
    SharedTrustObserver,
    StateIntegrator,
    StorageObserver,
    // P2P Observers
    SwarmObserver,
    TrustObserver,
};

// Re-exports - State Coordination
pub use state_coordination::{
    HealthReport, HealthStatus, Invariant, InvariantResult, InvariantSeverity, StateCoordinator,
    StateTransaction, TransactionError,
};

// Re-exports - Unified engine layer (Phase 3)
pub use engine::{
    event_gas, formula_gas, trust_gas, EventProcessor, FinalityTracker, FormulaComputer,
    TrustUpdater,
};
