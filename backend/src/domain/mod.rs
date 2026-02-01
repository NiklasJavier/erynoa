//! # Erynoa Domain Module
//!
//! Kerntypen für das Erynoa-Protokoll gemäß V4.1 Axiomen.
//!
//! ## Architektur (Phase 6 - Finalisiert)
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        DOMAIN MODULE                                │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  unified/   - Zukunftssichere Datenstrukturen (UDM)                │
//! │    primitives  - UniversalId, TemporalCoord                        │
//! │    identity    - DID, DIDDocument, Delegation (Κ6-Κ8)              │
//! │    event       - Event, FinalityState (Κ9-Κ12)                     │
//! │    trust       - TrustVector6D, TrustRecord (Κ2-Κ5)               │
//! │    realm       - Realm-Hierarchie (Κ1)                             │
//! │    saga        - Multi-Step Transaktionen (Κ22-Κ24)                │
//! │    formula     - Weltformel-Komponenten (Κ15a-d)                   │
//! │    cost        - Kosten-Algebra (Gas × Mana × Trust-Risk)          │
//! │    message     - P2P-Nachrichtentypen                              │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Unified Data Model (UDM)
//!
//! Das `unified` Modul enthält die optimierten, zukunftssicheren Datenstrukturen:
//!
//! - [`UniversalId`]: Content-addressed Identifier mit Type-Tag
//! - [`TemporalCoord`]: Hybride Zeit mit Lamport-Clocks
//! - [`TrustVector6D`]: Kompakter 6D Trust-Vektor (24 Bytes)
//! - [`Cost`]: Kosten-Algebra (Gas × Mana × Trust-Risk)
//!
//! ## v0.3.0 (Februar 2026)
//!
//! Alte Legacy-Module wurden entfernt. Alle Typen sind jetzt in `unified/`.

// ============================================================================
// Unified Data Model (Single Source of Truth)
// ============================================================================
pub mod unified;

// ============================================================================
// Re-Exports aus Unified
// ============================================================================

// Primitives
pub use unified::{TemporalCoord, UniversalId};

// Identity
pub use unified::{
    Capability, DIDDocument, DIDNamespace, Delegation, IdentityError, VerificationMethod,
    VerificationMethodType, DID,
};

// Event
pub use unified::{
    event_id_from_content, Event, EventError, EventId, EventPayload, FinalityError, FinalityLevel,
    FinalityState, Hash32, SagaStepResult, Signature64, VoteDirection, WitnessAttestation,
};

// Trust
pub use unified::{
    ContextType, DailyTrustStats, TrustCombination, TrustDampeningMatrix, TrustDimension,
    TrustHistory, TrustHistoryEntry, TrustRecord, TrustUpdateReason, TrustVector6D,
};

// Realm
pub use unified::{
    realm_id_from_name, GovernanceType, MemberRole, Partition, Realm, RealmError, RealmId,
    RealmMembership, RealmRules, RootRealm, Rule, RuleCategory, VirtualRealm, ROOT_REALM_ID,
};

// Saga
pub use unified::{
    saga_id_from_intent, Constraint, Goal, Intent, RealmCrossing, Saga, SagaAction,
    SagaCompensation, SagaError, SagaId, SagaStatus, SagaStep, StepResult, StepStatus,
};

// Formula
pub use unified::{
    Activity, AttestationLevel, HumanFactor, Surprisal, SurprisalComponents, TemporalWeight,
    WorldFormulaContribution, WorldFormulaStatus,
};

// Cost
pub use unified::{Budget, BudgetExhausted, Cost, CostTable};

// Message
pub use unified::{
    AttestationMessage, DhtRecordMessage, EventMessage, MessagePayload, P2PMessage, P2PProtocol,
    PeerInfoMessage, PingMessage, PongMessage, RealmJoinMessage, SagaIntentMessage,
    SyncRequestMessage, SyncResponseMessage, SyncType, TrustClaimMessage,
};

// Invarianten-Prüfer
pub use unified::{InvariantChecker, InvariantViolation};
