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
//! │  identity_types   - Identity Types & Traits (Κ6-Κ8)               │
//! │  event_engine     - Event-Verarbeitung (Κ9-Κ12)                    │
//! │  trust_engine     - Trust-Berechnung (Κ2-Κ5)                       │
//! │  surprisal        - Surprisal-Berechnung (Κ15a)                    │
//! │  world_formula    - Weltformel-Engine (Κ15b-d)                     │
//! │  consensus        - Konsensus-Mechanismus (Κ18)                    │
//! │  engine           - ExecutionContext-aware Wrapper (Phase 3)       │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

pub mod consensus;
pub mod eclvm_state_host;
pub mod engine;
pub mod event_engine;
pub mod identity_types;
pub mod state;
pub mod state_coordination;
pub mod state_integration;
pub mod surprisal;
pub mod trust_engine;
pub mod world_formula;

// Re-exports - State-backed ECL Host (Gap 2)
pub use eclvm_state_host::StateBackedHost;

// Re-exports - Legacy engines
pub use consensus::ConsensusEngine;
pub use event_engine::EventEngine;
pub use surprisal::SurprisalCalculator;
pub use trust_engine::TrustEngine;
pub use world_formula::WorldFormulaEngine;

// Re-exports - Unified State Management
pub use state::{
    create_unified_state,
    // Engine-Layer State (Phase 2 Additions)
    APIState,
    APISnapshot,
    AnomalySeverity,
    BlueprintActionType,
    BlueprintComposerState,
    BlueprintComposerSnapshot,
    // ECLVM State (Erynoa Core Language Virtual Machine)
    BlueprintStatus,
    // ─────────────────────────────────────────────────────────────────────────
    // Architektur-Verbesserungen Phase 6.4: Zustand-Abstraktion für ECLVM
    // ─────────────────────────────────────────────────────────────────────────
    // Budget für ECLVM-Ausführung
    BudgetExhaustionReason,
    CircuitBreaker,
    CircuitBreakerSnapshot,
    CommitResult,
    // Core State
    ConsensusState,
    ConsensusSnapshot,
    ControllerState,
    ControllerSnapshot,
    CoreState,
    CoreSnapshot,
    DataLogicState,
    DataLogicSnapshot,
    DeltaType,
    ECLPolicyType,
    ECLVMBudget,
    ECLVMBudgetLimits,
    ECLVMBudgetSnapshot,
    // State Context für ECLVM
    ECLVMExecutionSummary,
    ECLVMState,
    ECLVMStateContext,
    ECLVMSnapshot,
    // Architektur-Verbesserungen Phase 6.1
    // Event-Inversion (P2P/Core Entkopplung)
    EventBus,
    EventBusSnapshot,
    EventPriority,
    EventState,
    EventSnapshot,
    ExecutionState,
    ExecutionSnapshot,
    FormulaState,
    FormulaSnapshot,
    // Architektur-Verbesserungen Phase 6.2
    // Multi-Level Gas Metering
    GasLayer,
    GatewayState,
    GatewaySnapshot,
    GossipState,
    GossipSnapshot,
    GovernanceState,
    GovernanceSnapshot,
    // Differential State Snapshots (Merkle)
    Hashable,
    IdentityViewData,
    IntentParserState,
    IntentParserSnapshot,
    KademliaState,
    KademliaSnapshot,
    MemberRole,
    MembershipAction,
    MerkleDelta,
    MerkleHash,
    MerkleNode,
    MerkleStateTracker,
    MerkleTrackerSnapshot,
    MultiGas,
    MultiGasSnapshot,
    MutationResult,
    NatStatus,
    NetworkEvent,
    NetworkMetric,
    // P2P State
    P2PState,
    P2PSnapshot,
    // Peer State (Κ22-Κ24)
    PeerState,
    PeerSnapshot,
    PrivacyState,
    PrivacySnapshot,
    ProtectionState,
    ProtectionSnapshot,
    // Realm-specific Engine States
    RealmAPIState,
    RealmAction,
    RealmControllerState,
    RealmECLState,
    RealmECLSnapshot,
    RealmGovernanceState,
    // Self-Healing Realm-Isolierung
    RealmQuota,
    RealmQuotaSnapshot,
    // Realm State (Κ22-Κ24) - Per-Realm Isolation
    RealmSpecificState,
    RealmSpecificSnapshot,
    RealmState,
    RealmSnapshot,
    RealmUIState,
    RealmViewData,
    RelayState,
    RelaySnapshot,
    // Resource-Typen für Quotas
    ResourceType,
    RollbackResult,
    SagaComposerState,
    SagaComposerSnapshot,
    SharedUnifiedState,
    StateBroadcaster,
    BroadcasterSnapshot,
    StateComponent,
    // CQRS Light (State Delta Broadcasting)
    StateDelta,
    // ─────────────────────────────────────────────────────────────────────────
    // Architektur-Verbesserungen Phase 6.3: Event-Sourcing
    // ─────────────────────────────────────────────────────────────────────────
    // Event-Typen
    StateEvent,
    StateEventLog,
    EventLogSnapshot,
    StateGraph,
    // State Handle (Write Access)
    StateHandle,
    StateRelation,
    // State View (Read-Only)
    StateView,
    StorageBackend,
    // Storage als orthogonale Schicht
    StorageHandle,
    StorageMetrics,
    StorageState,
    StorageSnapshot,
    SwarmState as CoreSwarmState,
    SwarmSnapshot as CoreSwarmSnapshot,
    // Circuit Breaker Pattern
    SystemMode,
    // Transaction Guard (RAII)
    TransactionGuard,
    TrustDistribution,
    // Hilfs-Enums für Events
    TrustReason,
    TrustState,
    TrustSnapshot,
    UIState,
    UISnapshot,
    UnifiedState,
    UnifiedSnapshot,
    WrappedStateEvent,
};

// Re-exports - State Integration (Observer Pattern)
pub use state_integration::{
    // Engine-Layer Observers (Phase 4 Additions)
    APIObserver,
    BlueprintComposerObserver,
    CompositeObserver,
    // Core Observers
    ConsensusObserver,
    ControllerObserver,
    DataLogicObserver,
    // ECLVM Observer (ECL/ECLVM Integration) + Adapter für ProgrammableGateway (Phase 1.1)
    ECLVMObserverAdapter,
    ECLVMObserver,
    EventObserver,
    ExecutionObserver,
    FormulaObserver,
    // Peer Observers (Κ22-Κ24)
    GatewayObserver,
    GossipObserver,
    GovernanceObserver,
    IntentObserver,
    KademliaObserver,
    ObservableEngine,
    PrivacyObserver,
    ProtectionObserver,
    // Realm Observer (Κ22-Κ24)
    RealmObserver,
    RelayObserver,
    SagaObserver,
    // Engine-Layer Shared Observer Types
    SharedAPIObserver,
    SharedBlueprintComposerObserver,
    // Shared Types
    SharedConsensusObserver,
    SharedControllerObserver,
    SharedDataLogicObserver,
    // ECLVM Shared Observer
    SharedECLVMObserver,
    SharedEventObserver,
    SharedExecutionObserver,
    SharedFormulaObserver,
    SharedGatewayObserver,
    SharedGossipObserver,
    SharedGovernanceObserver,
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
    SharedUIObserver,
    StateIntegrator,
    StorageObserver,
    // P2P Observers
    SwarmObserver,
    TrustObserver,
    UIObserver,
};

// Re-exports - State Coordination
pub use state_coordination::{
    HealthReport, HealthStatus, Invariant, InvariantResult, InvariantSeverity, StateCoordinator,
    StateTransaction, TransactionError,
};

// Re-exports - Identity Types (Phase 1: Foundations)
pub use identity_types::{
    // Enums
    IdentityError,
    IdentityMode,
    KeyStoreType,
    PasskeyType,
    RealmRole,
    // Structs
    RealmMembership,
    WalletAddress,
    // Traits
    IdentityResolver,
    PasskeyManager,
    SecureKeyStore,
    // Shared Types
    SharedIdentityResolver,
    SharedKeyStore,
    SharedPasskeyManager,
};

// Re-exports - Unified engine layer (Phase 3)
pub use engine::{
    event_gas, formula_gas, trust_gas, EventProcessor, FinalityTracker, FormulaComputer,
    TrustUpdater,
};
