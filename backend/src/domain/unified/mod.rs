//! # Unified Data Model
//!
//! Unifizierte, zukunftssichere Datenstrukturen für das Erynoa-System.
//!
//! ## Module
//!
//! - [`primitives`]: Kern-Primitive (UniversalId, TemporalCoord)
//! - [`cost`]: Kosten-Algebra (Cost, Budget, CostTable)
//! - [`trust`]: Trust-Strukturen (TrustVector6D, TrustRecord)
//! - [`identity`]: DID-Strukturen (DID, DIDDocument, Delegation)
//! - [`event`]: Kausale Events (Event, FinalityState)
//!
//! ## Design-Prinzipien
//!
//! 1. **Zukunftssicher**: Versionierte Schemas, Extension Slots
//! 2. **Performance**: Cache-aligned Structs, Zero-Copy IDs
//! 3. **Konsistenz**: Unified Cost-Algebra, Shared Primitives
//! 4. **Erweiterbar**: Enum-Varianten mit Future-Slots
//! 5. **Beweisbar**: Compile-Time Size Checks, Runtime Invariant Checker
//!
//! ## Axiom-Mapping
//!
//! | Axiom | Datenstruktur |
//! |-------|---------------|
//! | Κ2-Κ5 | `TrustVector6D`, `TrustRecord` |
//! | Κ6-Κ8 | `DID`, `DIDDocument`, `Delegation` |
//! | Κ9-Κ12 | `Event`, `FinalityState`, `EventPayload` |
//! | Κ15a-d | (siehe `core/world_formula.rs`) |
//!
//! ## Beispiel
//!
//! ```rust
//! use erynoa_api::domain::unified::{
//!     UniversalId, TemporalCoord, TrustVector6D, Cost
//! };
//!
//! // Erstelle Event-ID
//! let event_id = UniversalId::new(UniversalId::TAG_EVENT, 1, b"event data");
//!
//! // Erstelle Zeitkoordinate
//! let node_id = UniversalId::new(UniversalId::TAG_DID, 1, b"node");
//! let coord = TemporalCoord::now(42, &node_id);
//!
//! // Trust-Vektor für Newcomer
//! let trust = TrustVector6D::NEWCOMER;
//!
//! // Kosten für Operation
//! let cost = Cost::new(100, 50, 0.1);
//! ```

pub mod cost;
pub mod event;
pub mod formula;
pub mod identity;
pub mod message;
pub mod primitives;
pub mod realm;
pub mod saga;
pub mod trust;

// Re-exports für einfachen Zugriff
pub use cost::{Budget, BudgetExhausted, Cost, CostTable};
pub use event::{
    event_id_from_content, Event, EventError, EventId, EventPayload, FinalityError, FinalityLevel,
    FinalityState, Hash32, SagaStepResult, Signature64, VoteDirection, WitnessAttestation,
};
pub use formula::{
    Activity, AttestationLevel, HumanFactor, Surprisal, SurprisalComponents, TemporalWeight,
    WorldFormulaContribution, WorldFormulaStatus,
};
pub use identity::{
    Capability, DIDDocument, DIDNamespace, Delegation, IdentityError, VerificationMethod,
    VerificationMethodType, DID,
};
pub use message::{
    AttestationMessage, DhtRecordMessage, EventMessage, MessagePayload, P2PMessage, P2PProtocol,
    PeerInfoMessage, PingMessage, PongMessage, RealmJoinMessage, SagaIntentMessage,
    SyncRequestMessage, SyncResponseMessage, SyncType, TrustClaimMessage,
};
pub use primitives::{TemporalCoord, UniversalId};
pub use realm::{
    realm_id_from_name, GovernanceType, MemberRole, Partition, Realm, RealmError, RealmId,
    RealmMembership, RealmRules, RootRealm, Rule, RuleCategory, StoreTemplate, StoreType,
    VirtualRealm, ROOT_REALM_ID,
};
pub use saga::{
    saga_id_from_intent, Constraint, Goal, Intent, RealmCrossing, Saga, SagaAction,
    SagaCompensation, SagaError, SagaId, SagaStatus, SagaStep, StepResult, StepStatus,
};
pub use trust::{
    ContextType, DailyTrustStats, TrustCombination, TrustDampeningMatrix, TrustDimension,
    TrustHistory, TrustHistoryEntry, TrustRecord, TrustUpdateReason, TrustVector6D,
};

// ============================================================================
// Invarianten-Prüfer
// ============================================================================

/// Invarianten-Prüfer für Runtime-Checks
pub struct InvariantChecker;

impl InvariantChecker {
    /// Κ8: Delegation Trust-Decay
    ///
    /// Prüft: `𝕋(delegate) ≤ trust_factor × 𝕋(delegator)`
    pub fn check_delegation_trust_factor(trust_factor: f32) -> Result<(), InvariantViolation> {
        if trust_factor <= 0.0 || trust_factor > 1.0 {
            return Err(InvariantViolation::K8InvalidTrustFactor { trust_factor });
        }
        Ok(())
    }

    /// Κ4: Asymmetrie-Prüfung
    ///
    /// Negative Updates müssen stärker gewichtet sein.
    pub fn check_asymmetric_update(
        dim: TrustDimension,
        delta: f32,
        applied_delta: f32,
    ) -> Result<(), InvariantViolation> {
        if delta < 0.0 {
            let expected_factor = dim.asymmetry_factor();
            let actual_factor = applied_delta / delta;

            if (actual_factor - expected_factor).abs() > 0.01 {
                return Err(InvariantViolation::K4AsymmetryMismatch {
                    expected: expected_factor,
                    actual: actual_factor,
                });
            }
        }
        Ok(())
    }

    /// Κ9: Kausale Ordnung
    ///
    /// Prüft: Parent-Events sind kausal vor diesem Event.
    pub fn check_causal_order(
        event_coord: &TemporalCoord,
        parent_coord: &TemporalCoord,
    ) -> Result<(), InvariantViolation> {
        if parent_coord >= event_coord {
            return Err(InvariantViolation::K9CausalViolation {
                event: *event_coord,
                parent: *parent_coord,
            });
        }
        Ok(())
    }

    /// Cost-Algebra: Prüfe Halbring-Eigenschaften
    pub fn check_cost_algebra(c1: Cost, c2: Cost, c3: Cost) -> Result<(), InvariantViolation> {
        // Assoziativität von seq: (c1 ⊕ c2) ⊕ c3 = c1 ⊕ (c2 ⊕ c3)
        let left = c1.seq(c2).seq(c3);
        let right = c1.seq(c2.seq(c3));

        // Floating-Point Toleranz
        let gas_ok = left.gas == right.gas;
        let mana_ok = left.mana == right.mana;
        let risk_ok = (left.trust_risk - right.trust_risk).abs() < 0.0001;

        if !gas_ok || !mana_ok || !risk_ok {
            return Err(InvariantViolation::CostAlgebraViolation {
                property: "associativity".to_string(),
            });
        }

        Ok(())
    }
}

/// Invarianten-Verletzung
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)] // Κ-Präfixe für Axiom-Referenzen beibehalten
pub enum InvariantViolation {
    /// Κ1: Regelset des Kindes ist kein Superset des Eltern-Regelsets
    K1MonotoneRules,

    /// Κ4: Asymmetrie-Faktor stimmt nicht
    K4AsymmetryMismatch { expected: f32, actual: f32 },

    /// Κ8: Trust-Faktor außerhalb (0, 1]
    K8InvalidTrustFactor { trust_factor: f32 },

    /// Κ9: Parent-Event ist nicht kausal vor Event
    K9CausalViolation {
        event: TemporalCoord,
        parent: TemporalCoord,
    },

    /// Κ10: Finalität ist gesunken
    K10FinalityRegression { old: u8, new: u8 },

    /// Kosten-Algebra verletzt
    CostAlgebraViolation { property: String },
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::K1MonotoneRules => {
                write!(f, "Κ1 violated: Child ruleset is not superset of parent")
            }
            Self::K4AsymmetryMismatch { expected, actual } => {
                write!(
                    f,
                    "Κ4 violated: Expected asymmetry {}, got {}",
                    expected, actual
                )
            }
            Self::K8InvalidTrustFactor { trust_factor } => {
                write!(
                    f,
                    "Κ8 violated: Trust factor {} not in (0, 1]",
                    trust_factor
                )
            }
            Self::K9CausalViolation { event, parent } => {
                write!(
                    f,
                    "Κ9 violated: Parent {:?} not before event {:?}",
                    parent, event
                )
            }
            Self::K10FinalityRegression { old, new } => {
                write!(
                    f,
                    "Κ10 violated: Finality regressed from {} to {}",
                    old, new
                )
            }
            Self::CostAlgebraViolation { property } => {
                write!(f, "Cost algebra violated: {}", property)
            }
        }
    }
}

impl std::error::Error for InvariantViolation {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invariant_delegation_trust() {
        assert!(InvariantChecker::check_delegation_trust_factor(0.5).is_ok());
        assert!(InvariantChecker::check_delegation_trust_factor(1.0).is_ok());
        assert!(InvariantChecker::check_delegation_trust_factor(0.0).is_err());
        assert!(InvariantChecker::check_delegation_trust_factor(1.5).is_err());
        assert!(InvariantChecker::check_delegation_trust_factor(-0.1).is_err());
    }

    #[test]
    fn test_invariant_causal_order() {
        let parent = TemporalCoord::new(1000, 5, 1);
        let event = TemporalCoord::new(1001, 6, 1);

        assert!(InvariantChecker::check_causal_order(&event, &parent).is_ok());
        assert!(InvariantChecker::check_causal_order(&parent, &event).is_err());
    }

    #[test]
    fn test_cost_algebra_properties() {
        let c1 = Cost::new(10, 5, 0.1);
        let c2 = Cost::new(20, 10, 0.2);
        let c3 = Cost::new(30, 15, 0.15);

        assert!(InvariantChecker::check_cost_algebra(c1, c2, c3).is_ok());
    }
}
