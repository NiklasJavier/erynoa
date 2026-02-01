//! # Erynoa Domain Module
//!
//! Kerntypen für das Erynoa-Protokoll gemäß V4.1 Axiomen.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        DOMAIN MODULE                                │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  unified    - Zukunftssichere Datenstrukturen (UDM)                │
//! │  did        - Dezentrale Identifikatoren (Κ6-Κ8)                   │
//! │  event      - Kausale Events im DAG (Κ9-Κ12)                       │
//! │  trust      - 6D Trust-Vektor 𝕎 (Κ2-Κ5)                           │
//! │  realm      - Realm-Hierarchie (Κ1)                                │
//! │  saga       - Multi-Step Transaktionen (Κ22-Κ24)                   │
//! │  formula    - Weltformel-Komponenten (Κ15a-d)                      │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Unified Data Model (UDM)
//!
//! Das `unified` Modul enthält die optimierten, zukunftssicheren Datenstrukturen:
//!
//! - [`unified::UniversalId`]: Content-addressed Identifier mit Type-Tag
//! - [`unified::TemporalCoord`]: Hybride Zeit mit Lamport-Clocks
//! - [`unified::TrustVector6D`]: Kompakter 6D Trust-Vektor (24 Bytes)
//! - [`unified::Cost`]: Kosten-Algebra (Gas × Mana × Trust-Risk)
//!
//! Siehe [`unified`] Modul-Dokumentation für Details.

pub mod did;
pub mod event;
pub mod formula;
pub mod realm;
pub mod saga;
pub mod trust;
pub mod unified;

// Re-exports for convenience
pub use did::{DIDNamespace, Delegation, DID};
pub use event::{Event, EventId, EventPayload, FinalityLevel, WitnessAttestation};
pub use formula::{Activity, HumanFactor, Surprisal, WorldFormulaContribution, WorldFormulaStatus};
pub use realm::{Partition, Realm, RealmId, RealmRules, RootRealm, VirtualRealm};
pub use saga::{
    Budget, Constraint, Goal, Intent, Saga, SagaAction, SagaCompensation, SagaStatus, SagaStep,
};
pub use trust::{
    ContextType, TrustCombination, TrustDampeningMatrix, TrustDimension, TrustVector6D,
};
