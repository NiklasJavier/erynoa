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

pub mod action;
pub mod component;
pub mod config;
pub mod cost;
pub mod event;
pub mod formula;
pub mod identity;
pub mod message;
pub mod primitives;
pub mod realm;
pub mod saga;
pub mod schema;
pub mod system;
pub mod trust;

// Re-exports für einfachen Zugriff
pub use action::{BlueprintAction, MembershipAction, NetworkMetric, RealmAction};
pub use component::{ComponentLayer, StateComponent, StateRelation};
pub use config::{
    global_config, init_global_config, ActivityConfig, ConfigValidationError, HumanFactorConfig,
    TemporalConfig, TrustConfig, WorldFormulaConfig, WorldFormulaConfigBuilder,
};
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
    extension_slots, Capability, DIDDocument, DIDNamespace, Delegation, IdentityError,
    VerificationMethod, VerificationMethodType, DID,
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
pub use schema::{
    append_field_migration, identity_migration, MigrationError, MigrationFn, SchemaRegistry,
};
pub use system::{AnomalySeverity, EventPriority, SystemMode};
pub use trust::{
    ContextType, DailyTrustStats, TrustCombination, TrustDampeningMatrix, TrustDimension,
    TrustHistory, TrustHistoryEntry, TrustRecord, TrustUpdateReason, TrustVector6D,
};

// ============================================================================
// Invarianten-Prüfer
// ============================================================================

/// Invarianten-Prüfer für Runtime-Checks
///
/// Prüft alle Axiome Κ1-Κ15 zur Laufzeit.
pub struct InvariantChecker;

impl InvariantChecker {
    // ========================================================================
    // Κ1: Realm-Hierarchie (Monotone Regelvererbung)
    // ========================================================================

    /// Κ1: Kind-Realm muss Regelset des Eltern-Realms enthalten
    ///
    /// ```text
    /// child.rules ⊇ parent.rules
    /// ```
    pub fn check_realm_rule_inheritance(
        parent_rules: &[Rule],
        child_rules: &[Rule],
    ) -> Result<(), InvariantViolation> {
        // Prüfe: Jede Regel des Parents muss im Child enthalten sein (nach ID)
        for parent_rule in parent_rules {
            let found = child_rules.iter().any(|child_rule| {
                child_rule.id == parent_rule.id && child_rule.category == parent_rule.category
            });

            if !found {
                return Err(InvariantViolation::K1MonotoneRules {
                    missing_rule: format!("{:?}:{}", parent_rule.category, parent_rule.id),
                });
            }
        }
        Ok(())
    }

    /// Κ1: Prüfe Realm-Hierarchie Tiefe
    ///
    /// Verhindert unbegrenzte Verschachtelung.
    pub fn check_realm_depth(depth: u32, max_depth: u32) -> Result<(), InvariantViolation> {
        if depth > max_depth {
            return Err(InvariantViolation::K1MaxDepthExceeded { depth, max_depth });
        }
        Ok(())
    }

    // ========================================================================
    // Κ4: Asymmetrische Trust-Updates
    // ========================================================================

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

    // ========================================================================
    // Κ8: Delegation Trust-Decay
    // ========================================================================

    /// Κ8: Delegation Trust-Decay
    ///
    /// Prüft: `𝕋(delegate) ≤ trust_factor × 𝕋(delegator)`
    pub fn check_delegation_trust_factor(trust_factor: f32) -> Result<(), InvariantViolation> {
        if trust_factor <= 0.0 || trust_factor > 1.0 {
            return Err(InvariantViolation::K8InvalidTrustFactor { trust_factor });
        }
        Ok(())
    }

    /// Κ8: Prüfe Trust-Decay in Delegationskette
    ///
    /// Trust muss mit jeder Delegation monoton abnehmen oder gleich bleiben.
    pub fn check_delegation_chain_decay(chain_trusts: &[f32]) -> Result<(), InvariantViolation> {
        for window in chain_trusts.windows(2) {
            if window[1] > window[0] {
                return Err(InvariantViolation::K8DelegationTrustIncreased {
                    delegator_trust: window[0],
                    delegate_trust: window[1],
                });
            }
        }
        Ok(())
    }

    // ========================================================================
    // Κ9: Kausale Ordnung
    // ========================================================================

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

    /// Κ9: Prüfe mehrere Parents auf Kausalität
    pub fn check_causal_parents(
        event_coord: &TemporalCoord,
        parent_coords: &[TemporalCoord],
    ) -> Result<(), InvariantViolation> {
        for parent in parent_coords {
            Self::check_causal_order(event_coord, parent)?;
        }
        Ok(())
    }

    // ========================================================================
    // Κ10: Finalitäts-Monotonie
    // ========================================================================

    /// Κ10: Finalitäts-Monotonie
    ///
    /// Prüft: Finalitätslevel darf nur steigen, nie sinken.
    pub fn check_finality_monotonic(
        old_level: FinalityLevel,
        new_level: FinalityLevel,
    ) -> Result<(), InvariantViolation> {
        let old_ord = old_level as u8;
        let new_ord = new_level as u8;

        if new_ord < old_ord {
            return Err(InvariantViolation::K10FinalityRegression {
                old: old_ord,
                new: new_ord,
            });
        }
        Ok(())
    }

    /// Κ10: Prüfe Finalitäts-Progression
    ///
    /// Finalität muss die korrekte Progression durchlaufen.
    pub fn check_finality_progression(
        old_level: FinalityLevel,
        new_level: FinalityLevel,
    ) -> Result<(), InvariantViolation> {
        // Erlaubte Übergänge: Nur zum nächsten oder gleichen Level
        // Nascent(0) -> Validated(1) -> Witnessed(2) -> Anchored(3) -> Eternal(4)
        let old_ord = old_level as u8;
        let new_ord = new_level as u8;

        // Gleichbleiben ist immer OK
        if old_ord == new_ord {
            return Ok(());
        }

        // Progression um maximal 1 Stufe ist OK
        if new_ord == old_ord + 1 {
            return Ok(());
        }

        // Sprünge > 1 können auch erlaubt sein (z.B. Nascent -> Witnessed bei schneller Bestätigung)
        if new_ord > old_ord {
            return Ok(());
        }

        // Regression ist nicht erlaubt
        Err(InvariantViolation::K10InvalidProgression {
            from: old_level,
            to: new_level,
        })
    }

    // ========================================================================
    // Cost-Algebra
    // ========================================================================

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

    /// Cost-Algebra: Prüfe Distributivität
    ///
    /// seq distribuiert über par: a ⊕ (b ⊗ c) = (a ⊕ b) ⊗ (a ⊕ c)
    pub fn check_cost_distributivity(a: Cost, b: Cost, c: Cost) -> Result<(), InvariantViolation> {
        let left = a.seq(b.par(c));
        let right = a.seq(b).par(a.seq(c));

        // Beachte: Distributivität gilt nur approximativ wegen der verschiedenen
        // Operationen (sum vs max) in par
        let gas_ok = (left.gas as i64 - right.gas as i64).abs() <= 1;
        let mana_ok = (left.mana as i64 - right.mana as i64).abs() <= 1;
        let risk_ok = (left.trust_risk - right.trust_risk).abs() < 0.01;

        if !gas_ok || !mana_ok || !risk_ok {
            return Err(InvariantViolation::CostAlgebraViolation {
                property: "distributivity".to_string(),
            });
        }

        Ok(())
    }

    // ========================================================================
    // Realm-Invarianten
    // ========================================================================

    /// Realm: Prüfe Partition-Vollständigkeit
    ///
    /// Alle Members müssen einer Partition zugeordnet sein.
    pub fn check_partition_coverage(
        total_members: u64,
        partition_members: u64,
    ) -> Result<(), InvariantViolation> {
        if partition_members > total_members {
            return Err(InvariantViolation::RealmPartitionOverflow {
                total: total_members,
                partition: partition_members,
            });
        }
        Ok(())
    }

    /// Realm: Prüfe Governance-Quorum
    ///
    /// Quorum muss zwischen 0 und Mitgliederzahl liegen.
    pub fn check_governance_quorum(
        quorum: u64,
        total_members: u64,
    ) -> Result<(), InvariantViolation> {
        if quorum > total_members {
            return Err(InvariantViolation::RealmInvalidQuorum {
                quorum,
                total_members,
            });
        }
        if quorum == 0 && total_members > 0 {
            return Err(InvariantViolation::RealmZeroQuorum);
        }
        Ok(())
    }

    /// Realm: Prüfe Store-Konsistenz
    ///
    /// Store-Typ muss mit Template kompatibel sein.
    pub fn check_store_compatibility(
        store_type: StoreType,
        template: &StoreTemplate,
    ) -> Result<(), InvariantViolation> {
        // Prüfe, ob der Store-Typ mit dem Template übereinstimmt
        if store_type != template.store_type {
            return Err(InvariantViolation::RealmStoreIncompatible {
                store_type,
                template_type: format!("{:?}", template.store_type),
            });
        }
        Ok(())
    }

    // ========================================================================
    // SystemMode-Invarianten (Circuit Breaker)
    // ========================================================================

    /// SystemMode: Prüfe gültige Transition
    ///
    /// Erlaubte Transitionen:
    /// - Normal → Degraded, EmergencyShutdown
    /// - Degraded → Normal, EmergencyShutdown
    /// - EmergencyShutdown → Normal (nur mit Admin-Reset)
    ///
    /// EmergencyShutdown → Normal erfordert expliziten Admin-Reset.
    pub fn check_system_mode_transition(
        old: SystemMode,
        new: SystemMode,
        admin_reset: bool,
    ) -> Result<(), InvariantViolation> {
        // Gleicher Modus ist immer OK
        if old == new {
            return Ok(());
        }

        match (old, new) {
            // Normal kann zu allen Modi wechseln
            (SystemMode::Normal, _) => Ok(()),

            // Degraded kann zu Normal (Recovery) oder Emergency wechseln
            (SystemMode::Degraded, SystemMode::Normal) => Ok(()),
            (SystemMode::Degraded, SystemMode::EmergencyShutdown) => Ok(()),

            // Emergency → Normal nur mit Admin-Reset
            (SystemMode::EmergencyShutdown, SystemMode::Normal) => {
                if admin_reset {
                    Ok(())
                } else {
                    Err(InvariantViolation::SystemModeInvalidTransition {
                        from: old,
                        to: new,
                        reason: "Emergency→Normal requires admin_reset=true".to_string(),
                    })
                }
            }

            // Emergency → Degraded nicht erlaubt (muss erst zu Normal)
            (SystemMode::EmergencyShutdown, SystemMode::Degraded) => {
                Err(InvariantViolation::SystemModeInvalidTransition {
                    from: old,
                    to: new,
                    reason: "Emergency→Degraded not allowed, use Emergency→Normal→Degraded"
                        .to_string(),
                })
            }

            // Same mode transitions (already handled above, but needed for exhaustiveness)
            (SystemMode::Degraded, SystemMode::Degraded)
            | (SystemMode::EmergencyShutdown, SystemMode::EmergencyShutdown) => Ok(()),
        }
    }

    /// SystemMode: Prüfe ob Operation im aktuellen Modus erlaubt ist
    pub fn check_operation_allowed_in_mode(
        mode: SystemMode,
        operation: &str,
        requires_normal: bool,
    ) -> Result<(), InvariantViolation> {
        match mode {
            SystemMode::Normal => Ok(()),
            SystemMode::Degraded => {
                if requires_normal {
                    Err(InvariantViolation::OperationNotAllowedInMode {
                        mode,
                        operation: operation.to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            SystemMode::EmergencyShutdown => Err(InvariantViolation::OperationNotAllowedInMode {
                mode,
                operation: operation.to_string(),
            }),
        }
    }

    // ========================================================================
    // Κ19: Diversity / Gini-Invarianten
    // ========================================================================

    /// Κ19: Prüfe Gini-Koeffizient gegen Threshold
    ///
    /// Der Gini-Koeffizient misst die Ungleichverteilung (0 = perfekte Gleichheit, 1 = maximale Ungleichheit).
    /// Erynoa verwendet diesen zur Dezentralisierungs-Überwachung.
    pub fn check_gini_threshold(gini: f64, threshold: f64) -> Result<(), InvariantViolation> {
        if !(0.0..=1.0).contains(&gini) {
            return Err(InvariantViolation::K19InvalidGini { gini });
        }
        if !(0.0..=1.0).contains(&threshold) {
            return Err(InvariantViolation::K19InvalidThreshold { threshold });
        }
        if gini > threshold {
            return Err(InvariantViolation::K19GiniExceeded { gini, threshold });
        }
        Ok(())
    }

    /// Κ19: Berechne Gini-Koeffizient für eine Verteilung
    ///
    /// Verwendet die Standard-Formel: G = (2 * Σᵢ i*xᵢ) / (n * Σᵢ xᵢ) - (n+1)/n
    pub fn calculate_gini(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let n = values.len() as f64;
        let sum: f64 = values.iter().sum();

        if sum == 0.0 {
            return 0.0;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let weighted_sum: f64 = sorted
            .iter()
            .enumerate()
            .map(|(i, &x)| (i as f64 + 1.0) * x)
            .sum();

        (2.0 * weighted_sum) / (n * sum) - (n + 1.0) / n
    }

    // ========================================================================
    // Quota-Invarianten (Resource-Limits)
    // ========================================================================

    /// Quota: Prüfe Resource-Nutzung gegen Limit
    pub fn check_quota(used: u64, limit: u64, resource: &str) -> Result<(), InvariantViolation> {
        if used > limit {
            return Err(InvariantViolation::QuotaExceeded {
                resource: resource.to_string(),
                used,
                limit,
            });
        }
        Ok(())
    }

    /// Quota: Prüfe ob Quota-Warnschwelle erreicht ist (80%)
    pub fn check_quota_warning(used: u64, limit: u64) -> bool {
        if limit == 0 {
            return false;
        }
        let usage_percent = (used as f64 / limit as f64) * 100.0;
        usage_percent >= 80.0
    }

    /// Quota: Berechne verbleibende Kapazität
    pub fn remaining_quota(used: u64, limit: u64) -> u64 {
        limit.saturating_sub(used)
    }

    // ========================================================================
    // EventPriority-Invarianten
    // ========================================================================

    /// EventPriority: Prüfe ob Event-Priorität für Queue-Kapazität akzeptiert wird
    ///
    /// Bei hoher Last werden niedrig-priorisierte Events abgelehnt.
    pub fn check_priority_admission(
        priority: EventPriority,
        queue_usage_percent: f64,
    ) -> Result<(), InvariantViolation> {
        let min_priority = if queue_usage_percent >= 95.0 {
            EventPriority::Critical
        } else if queue_usage_percent >= 80.0 {
            EventPriority::High
        } else if queue_usage_percent >= 60.0 {
            EventPriority::Normal
        } else {
            EventPriority::Low
        };

        if priority > min_priority {
            // priority > min_priority bedeutet niedrigere Priorität (höherer Ordinalwert)
            return Err(InvariantViolation::PriorityRejected {
                priority,
                min_required: min_priority,
                queue_usage_percent,
            });
        }
        Ok(())
    }

    /// EventPriority: Prüfe konsistente Prioritäts-Ordnung
    ///
    /// Critical < High < Normal < Low (aufsteigend in Ordinalwert)
    pub fn check_priority_ordering(
        higher: EventPriority,
        lower: EventPriority,
    ) -> Result<(), InvariantViolation> {
        if higher >= lower {
            return Err(InvariantViolation::PriorityOrderViolation { higher, lower });
        }
        Ok(())
    }
}

/// Invarianten-Verletzung
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)] // Κ-Präfixe für Axiom-Referenzen beibehalten
pub enum InvariantViolation {
    // ========================================================================
    // Κ1: Realm-Hierarchie
    // ========================================================================
    /// Κ1: Regelset des Kindes ist kein Superset des Eltern-Regelsets
    K1MonotoneRules { missing_rule: String },

    /// Κ1: Maximale Realm-Tiefe überschritten
    K1MaxDepthExceeded { depth: u32, max_depth: u32 },

    // ========================================================================
    // Κ4: Asymmetrische Trust-Updates
    // ========================================================================
    /// Κ4: Asymmetrie-Faktor stimmt nicht
    K4AsymmetryMismatch { expected: f32, actual: f32 },

    // ========================================================================
    // Κ8: Delegation Trust-Decay
    // ========================================================================
    /// Κ8: Trust-Faktor außerhalb (0, 1]
    K8InvalidTrustFactor { trust_factor: f32 },

    /// Κ8: Trust hat in Delegation zugenommen
    K8DelegationTrustIncreased {
        delegator_trust: f32,
        delegate_trust: f32,
    },

    // ========================================================================
    // Κ9: Kausale Ordnung
    // ========================================================================
    /// Κ9: Parent-Event ist nicht kausal vor Event
    K9CausalViolation {
        event: TemporalCoord,
        parent: TemporalCoord,
    },

    // ========================================================================
    // Κ10: Finalitäts-Monotonie
    // ========================================================================
    /// Κ10: Finalität ist gesunken
    K10FinalityRegression { old: u8, new: u8 },

    /// Κ10: Ungültige Finalitäts-Progression
    K10InvalidProgression {
        from: FinalityLevel,
        to: FinalityLevel,
    },

    // ========================================================================
    // Cost-Algebra
    // ========================================================================
    /// Kosten-Algebra verletzt
    CostAlgebraViolation { property: String },

    // ========================================================================
    // Realm-Invarianten
    // ========================================================================
    /// Partition hat mehr Members als Realm
    RealmPartitionOverflow { total: u64, partition: u64 },

    /// Quorum größer als Mitgliederzahl
    RealmInvalidQuorum { quorum: u64, total_members: u64 },

    /// Zero-Quorum bei nicht-leerem Realm
    RealmZeroQuorum,

    /// Store-Typ inkompatibel mit Template
    RealmStoreIncompatible {
        store_type: StoreType,
        template_type: String,
    },

    // ========================================================================
    // SystemMode-Invarianten
    // ========================================================================
    /// Ungültige SystemMode-Transition
    SystemModeInvalidTransition {
        from: SystemMode,
        to: SystemMode,
        reason: String,
    },

    /// Operation im aktuellen Modus nicht erlaubt
    OperationNotAllowedInMode { mode: SystemMode, operation: String },

    // ========================================================================
    // Κ19: Diversity / Gini-Invarianten
    // ========================================================================
    /// Κ19: Gini-Koeffizient überschreitet Threshold
    K19GiniExceeded { gini: f64, threshold: f64 },

    /// Κ19: Ungültiger Gini-Wert (muss 0.0-1.0 sein)
    K19InvalidGini { gini: f64 },

    /// Κ19: Ungültiger Threshold (muss 0.0-1.0 sein)
    K19InvalidThreshold { threshold: f64 },

    // ========================================================================
    // Quota-Invarianten
    // ========================================================================
    /// Quota überschritten
    QuotaExceeded {
        resource: String,
        used: u64,
        limit: u64,
    },

    // ========================================================================
    // EventPriority-Invarianten
    // ========================================================================
    /// Event-Priorität bei hoher Last abgelehnt
    PriorityRejected {
        priority: EventPriority,
        min_required: EventPriority,
        queue_usage_percent: f64,
    },

    /// Prioritäts-Ordnung verletzt
    PriorityOrderViolation {
        higher: EventPriority,
        lower: EventPriority,
    },
}

impl std::fmt::Display for InvariantViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Κ1
            Self::K1MonotoneRules { missing_rule } => {
                write!(
                    f,
                    "Κ1 violated: Child ruleset missing parent rule: {}",
                    missing_rule
                )
            }
            Self::K1MaxDepthExceeded { depth, max_depth } => {
                write!(
                    f,
                    "Κ1 violated: Realm depth {} exceeds max {}",
                    depth, max_depth
                )
            }

            // Κ4
            Self::K4AsymmetryMismatch { expected, actual } => {
                write!(
                    f,
                    "Κ4 violated: Expected asymmetry {}, got {}",
                    expected, actual
                )
            }

            // Κ8
            Self::K8InvalidTrustFactor { trust_factor } => {
                write!(
                    f,
                    "Κ8 violated: Trust factor {} not in (0, 1]",
                    trust_factor
                )
            }
            Self::K8DelegationTrustIncreased {
                delegator_trust,
                delegate_trust,
            } => {
                write!(
                    f,
                    "Κ8 violated: Delegate trust {} > delegator trust {}",
                    delegate_trust, delegator_trust
                )
            }

            // Κ9
            Self::K9CausalViolation { event, parent } => {
                write!(
                    f,
                    "Κ9 violated: Parent {:?} not before event {:?}",
                    parent, event
                )
            }

            // Κ10
            Self::K10FinalityRegression { old, new } => {
                write!(
                    f,
                    "Κ10 violated: Finality regressed from {} to {}",
                    old, new
                )
            }
            Self::K10InvalidProgression { from, to } => {
                write!(
                    f,
                    "Κ10 violated: Invalid finality progression from {:?} to {:?}",
                    from, to
                )
            }

            // Cost-Algebra
            Self::CostAlgebraViolation { property } => {
                write!(f, "Cost algebra violated: {}", property)
            }

            // Realm
            Self::RealmPartitionOverflow { total, partition } => {
                write!(
                    f,
                    "Realm invariant violated: Partition {} exceeds total {}",
                    partition, total
                )
            }
            Self::RealmInvalidQuorum {
                quorum,
                total_members,
            } => {
                write!(
                    f,
                    "Realm invariant violated: Quorum {} exceeds members {}",
                    quorum, total_members
                )
            }
            Self::RealmZeroQuorum => {
                write!(
                    f,
                    "Realm invariant violated: Zero quorum in non-empty realm"
                )
            }
            Self::RealmStoreIncompatible {
                store_type,
                template_type,
            } => {
                write!(
                    f,
                    "Realm invariant violated: Store {:?} incompatible with template {}",
                    store_type, template_type
                )
            }

            // SystemMode
            Self::SystemModeInvalidTransition { from, to, reason } => {
                write!(
                    f,
                    "SystemMode transition {:?}→{:?} invalid: {}",
                    from, to, reason
                )
            }
            Self::OperationNotAllowedInMode { mode, operation } => {
                write!(
                    f,
                    "Operation '{}' not allowed in mode {:?}",
                    operation, mode
                )
            }

            // Κ19
            Self::K19GiniExceeded { gini, threshold } => {
                write!(
                    f,
                    "Κ19 violated: Gini coefficient {:.4} exceeds threshold {:.4}",
                    gini, threshold
                )
            }
            Self::K19InvalidGini { gini } => {
                write!(f, "Κ19 violated: Gini coefficient {:.4} not in [0, 1]", gini)
            }
            Self::K19InvalidThreshold { threshold } => {
                write!(
                    f,
                    "Κ19 violated: Threshold {:.4} not in [0, 1]",
                    threshold
                )
            }

            // Quota
            Self::QuotaExceeded {
                resource,
                used,
                limit,
            } => {
                write!(
                    f,
                    "Quota exceeded for '{}': used {} > limit {}",
                    resource, used, limit
                )
            }

            // Priority
            Self::PriorityRejected {
                priority,
                min_required,
                queue_usage_percent,
            } => {
                write!(
                    f,
                    "Priority {:?} rejected at {:.1}% queue usage (min required: {:?})",
                    priority, queue_usage_percent, min_required
                )
            }
            Self::PriorityOrderViolation { higher, lower } => {
                write!(
                    f,
                    "Priority order violated: {:?} should be higher than {:?}",
                    higher, lower
                )
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

    #[test]
    fn test_k1_realm_rule_inheritance() {
        let parent_rules = vec![Rule::new(
            "rule1",
            "Compliance Rule",
            RuleCategory::Compliance,
            "GDPR compliance",
        )];

        // Child hat alle Parent-Regeln → OK
        let child_rules = vec![
            Rule::new(
                "rule1",
                "Compliance Rule",
                RuleCategory::Compliance,
                "GDPR compliance",
            ),
            Rule::new(
                "rule2",
                "Trust Rule",
                RuleCategory::Trust,
                "Trust threshold",
            ),
        ];

        assert!(
            InvariantChecker::check_realm_rule_inheritance(&parent_rules, &child_rules).is_ok()
        );

        // Child fehlt Parent-Regel → Error
        let incomplete_child = vec![Rule::new(
            "rule2",
            "Trust Rule",
            RuleCategory::Trust,
            "Trust threshold",
        )];

        assert!(
            InvariantChecker::check_realm_rule_inheritance(&parent_rules, &incomplete_child)
                .is_err()
        );
    }

    #[test]
    fn test_k1_realm_depth() {
        assert!(InvariantChecker::check_realm_depth(5, 10).is_ok());
        assert!(InvariantChecker::check_realm_depth(10, 10).is_ok());
        assert!(InvariantChecker::check_realm_depth(11, 10).is_err());
    }

    #[test]
    fn test_k8_delegation_chain() {
        // Trust nimmt ab → OK
        assert!(InvariantChecker::check_delegation_chain_decay(&[1.0, 0.8, 0.6]).is_ok());

        // Trust nimmt zu → Error
        assert!(InvariantChecker::check_delegation_chain_decay(&[0.5, 0.7]).is_err());

        // Gleicher Trust → OK
        assert!(InvariantChecker::check_delegation_chain_decay(&[0.8, 0.8]).is_ok());
    }

    #[test]
    fn test_k9_causal_parents() {
        let event = TemporalCoord::new(100, 0, 0);
        let parents = vec![TemporalCoord::new(50, 0, 0), TemporalCoord::new(80, 0, 0)];

        assert!(InvariantChecker::check_causal_parents(&event, &parents).is_ok());

        // Ein Parent ist nach Event → Error
        let bad_parents = vec![TemporalCoord::new(50, 0, 0), TemporalCoord::new(150, 0, 0)];

        assert!(InvariantChecker::check_causal_parents(&event, &bad_parents).is_err());
    }

    #[test]
    fn test_k10_finality_monotonic() {
        assert!(InvariantChecker::check_finality_monotonic(
            FinalityLevel::Nascent,
            FinalityLevel::Validated
        )
        .is_ok());

        assert!(InvariantChecker::check_finality_monotonic(
            FinalityLevel::Validated,
            FinalityLevel::Eternal
        )
        .is_ok());

        // Regression → Error
        assert!(InvariantChecker::check_finality_monotonic(
            FinalityLevel::Eternal,
            FinalityLevel::Validated
        )
        .is_err());
    }

    #[test]
    fn test_k10_finality_progression() {
        // Gültige Progressionen
        assert!(InvariantChecker::check_finality_progression(
            FinalityLevel::Nascent,
            FinalityLevel::Validated
        )
        .is_ok());

        assert!(InvariantChecker::check_finality_progression(
            FinalityLevel::Witnessed,
            FinalityLevel::Eternal
        )
        .is_ok());

        // Regression → Error
        assert!(InvariantChecker::check_finality_progression(
            FinalityLevel::Eternal,
            FinalityLevel::Nascent
        )
        .is_err());
    }

    #[test]
    fn test_realm_quorum() {
        // Gültiges Quorum
        assert!(InvariantChecker::check_governance_quorum(5, 10).is_ok());

        // Quorum > Members → Error
        assert!(InvariantChecker::check_governance_quorum(15, 10).is_err());

        // Zero Quorum bei non-empty Realm → Error
        assert!(InvariantChecker::check_governance_quorum(0, 10).is_err());

        // Zero Quorum bei empty Realm → OK
        assert!(InvariantChecker::check_governance_quorum(0, 0).is_ok());
    }

    #[test]
    fn test_realm_partition() {
        assert!(InvariantChecker::check_partition_coverage(100, 50).is_ok());
        assert!(InvariantChecker::check_partition_coverage(100, 100).is_ok());
        assert!(InvariantChecker::check_partition_coverage(100, 150).is_err());
    }

    #[test]
    fn test_config_module() {
        // Teste, dass das Config-Modul korrekt exportiert wird
        let config = WorldFormulaConfig::default();
        assert!(config.validate().is_ok());

        let custom = WorldFormulaConfig::builder()
            .asymmetry_base(1.8)
            .activity_tau_days(60)
            .build();

        assert_eq!(custom.trust.asymmetry_base, 1.8);
        assert_eq!(custom.activity.tau_days, 60);
    }

    // ========================================================================
    // Phase 5: Neue Invarianten-Tests
    // ========================================================================

    #[test]
    fn test_system_mode_transition() {
        // Normal → Degraded: OK
        assert!(InvariantChecker::check_system_mode_transition(
            SystemMode::Normal,
            SystemMode::Degraded,
            false
        )
        .is_ok());

        // Normal → Emergency: OK
        assert!(InvariantChecker::check_system_mode_transition(
            SystemMode::Normal,
            SystemMode::EmergencyShutdown,
            false
        )
        .is_ok());

        // Degraded → Normal: OK (Recovery)
        assert!(InvariantChecker::check_system_mode_transition(
            SystemMode::Degraded,
            SystemMode::Normal,
            false
        )
        .is_ok());

        // Emergency → Normal ohne Admin: Error
        assert!(InvariantChecker::check_system_mode_transition(
            SystemMode::EmergencyShutdown,
            SystemMode::Normal,
            false
        )
        .is_err());

        // Emergency → Normal mit Admin: OK
        assert!(InvariantChecker::check_system_mode_transition(
            SystemMode::EmergencyShutdown,
            SystemMode::Normal,
            true
        )
        .is_ok());

        // Emergency → Degraded: Error
        assert!(InvariantChecker::check_system_mode_transition(
            SystemMode::EmergencyShutdown,
            SystemMode::Degraded,
            false
        )
        .is_err());
    }

    #[test]
    fn test_operation_allowed_in_mode() {
        // Normal: Alle Operationen erlaubt
        assert!(
            InvariantChecker::check_operation_allowed_in_mode(SystemMode::Normal, "crossing", true)
                .is_ok()
        );

        // Degraded: Normal-Only Operationen blockiert
        assert!(InvariantChecker::check_operation_allowed_in_mode(
            SystemMode::Degraded,
            "crossing",
            true
        )
        .is_err());

        // Degraded: Nicht-Normal-Only Operationen erlaubt
        assert!(InvariantChecker::check_operation_allowed_in_mode(
            SystemMode::Degraded,
            "read",
            false
        )
        .is_ok());

        // Emergency: Alle Operationen blockiert
        assert!(InvariantChecker::check_operation_allowed_in_mode(
            SystemMode::EmergencyShutdown,
            "any",
            false
        )
        .is_err());
    }

    #[test]
    fn test_k19_gini_threshold() {
        // Gini unter Threshold: OK
        assert!(InvariantChecker::check_gini_threshold(0.3, 0.5).is_ok());
        assert!(InvariantChecker::check_gini_threshold(0.5, 0.5).is_ok());

        // Gini über Threshold: Error
        assert!(InvariantChecker::check_gini_threshold(0.6, 0.5).is_err());

        // Ungültiger Gini: Error
        assert!(InvariantChecker::check_gini_threshold(-0.1, 0.5).is_err());
        assert!(InvariantChecker::check_gini_threshold(1.1, 0.5).is_err());

        // Ungültiger Threshold: Error
        assert!(InvariantChecker::check_gini_threshold(0.3, 1.5).is_err());
    }

    #[test]
    fn test_calculate_gini() {
        // Perfekte Gleichheit
        let equal = vec![10.0, 10.0, 10.0, 10.0];
        let gini_equal = InvariantChecker::calculate_gini(&equal);
        assert!(gini_equal.abs() < 0.01);

        // Hohe Ungleichheit
        let unequal = vec![0.0, 0.0, 0.0, 100.0];
        let gini_unequal = InvariantChecker::calculate_gini(&unequal);
        assert!(gini_unequal > 0.7);

        // Leere Liste
        assert_eq!(InvariantChecker::calculate_gini(&[]), 0.0);
    }

    #[test]
    fn test_quota_checks() {
        // Unter Limit: OK
        assert!(InvariantChecker::check_quota(500, 1000, "events").is_ok());

        // Am Limit: OK
        assert!(InvariantChecker::check_quota(1000, 1000, "events").is_ok());

        // Über Limit: Error
        assert!(InvariantChecker::check_quota(1500, 1000, "events").is_err());

        // Warning-Schwelle
        assert!(!InvariantChecker::check_quota_warning(500, 1000)); // 50% - kein Warning
        assert!(InvariantChecker::check_quota_warning(800, 1000)); // 80% - Warning
        assert!(InvariantChecker::check_quota_warning(950, 1000)); // 95% - Warning

        // Remaining
        assert_eq!(InvariantChecker::remaining_quota(300, 1000), 700);
        assert_eq!(InvariantChecker::remaining_quota(1000, 1000), 0);
        assert_eq!(InvariantChecker::remaining_quota(1200, 1000), 0); // saturating
    }

    #[test]
    fn test_priority_admission() {
        // Niedrige Last: Alle Prioritäten erlaubt
        assert!(
            InvariantChecker::check_priority_admission(EventPriority::Low, 30.0).is_ok()
        );

        // Mittlere Last (60%+): Nur Normal+ erlaubt
        assert!(
            InvariantChecker::check_priority_admission(EventPriority::Normal, 65.0).is_ok()
        );
        assert!(
            InvariantChecker::check_priority_admission(EventPriority::Low, 65.0).is_err()
        );

        // Hohe Last (80%+): Nur High+ erlaubt
        assert!(
            InvariantChecker::check_priority_admission(EventPriority::High, 85.0).is_ok()
        );
        assert!(
            InvariantChecker::check_priority_admission(EventPriority::Normal, 85.0).is_err()
        );

        // Kritische Last (95%+): Nur Critical erlaubt
        assert!(
            InvariantChecker::check_priority_admission(EventPriority::Critical, 98.0).is_ok()
        );
        assert!(
            InvariantChecker::check_priority_admission(EventPriority::High, 98.0).is_err()
        );
    }

    #[test]
    fn test_priority_ordering() {
        // Korrekte Ordnung
        assert!(
            InvariantChecker::check_priority_ordering(EventPriority::Critical, EventPriority::High)
                .is_ok()
        );
        assert!(
            InvariantChecker::check_priority_ordering(EventPriority::High, EventPriority::Normal)
                .is_ok()
        );

        // Falsche Ordnung
        assert!(
            InvariantChecker::check_priority_ordering(EventPriority::Normal, EventPriority::High)
                .is_err()
        );
        assert!(
            InvariantChecker::check_priority_ordering(EventPriority::Normal, EventPriority::Normal)
                .is_err()
        );
    }
}
