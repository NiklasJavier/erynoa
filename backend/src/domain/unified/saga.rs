//! # Unified Data Model – Saga
//!
//! Multi-Step Transaktionen gemäß Axiome Κ22-Κ24.
//!
//! ## Axiom-Referenz
//!
//! - **Κ22 (Saga-Composer)**: `∀ Intent I : ∃! Saga S : resolve(I) = S`
//! - **Κ23 (Gateway-Guard)**: `cross(s, 𝒞₁, 𝒞₂) requires G(s, 𝒞₂) = true`
//! - **Κ24 (Atomare Kompensation)**: `fail(Sᵢ) → compensate(S₁..Sᵢ₋₁)`
//!
//! ## Migration von domain/saga.rs
//!
//! - Alle IDs auf `UniversalId`
//! - `DateTime<Utc>` → `TemporalCoord`
//! - Budget-Integration mit `unified/cost.rs`

use super::cost::{Budget, Cost};
use super::primitives::{TemporalCoord, UniversalId};
use super::realm::RealmId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// SagaId – Type Alias für UniversalId
// ============================================================================

/// Saga-Identifikator (Content-Addressed via UniversalId)
pub type SagaId = UniversalId;

/// Erstelle SagaId aus Intent-ID
pub fn saga_id_from_intent(intent_id: &UniversalId) -> SagaId {
    UniversalId::new(UniversalId::TAG_SAGA, 1, intent_id.as_bytes())
}

// ============================================================================
// Intent (Κ22)
// ============================================================================

/// Ein Intent repräsentiert eine Nutzer-Absicht (Κ22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// Eindeutige ID
    pub id: UniversalId,
    /// Quelle (wer hat den Intent erstellt)
    pub source: UniversalId,
    /// Ziel-Zustand
    pub goal: Goal,
    /// Constraints für die Ausführung
    pub constraints: Vec<Constraint>,
    /// Budget für die Ausführung
    pub budget: Budget,
    /// Kontext (aktuelles Realm)
    pub context_realm: RealmId,
    /// Erstellungszeitpunkt
    pub created_at: TemporalCoord,
    /// Timeout in Sekunden
    pub timeout_seconds: u64,
}

impl Intent {
    /// Erstelle neuen Intent
    pub fn new(source: UniversalId, goal: Goal, context_realm: RealmId, lamport: u32) -> Self {
        // Content für ID
        let mut content = Vec::new();
        content.extend_from_slice(source.as_bytes());
        content.extend_from_slice(goal.type_tag().as_bytes());
        content.extend_from_slice(context_realm.as_bytes());

        let id = UniversalId::new(UniversalId::TAG_SAGA, 1, &content);
        let created_at = TemporalCoord::now(lamport, &id);

        Self {
            id,
            source,
            goal,
            constraints: vec![],
            budget: Budget::default(),
            context_realm,
            created_at,
            timeout_seconds: 3600, // 1 Stunde Default
        }
    }

    /// Mit Constraint
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Mit Budget
    pub fn with_budget(mut self, budget: Budget) -> Self {
        self.budget = budget;
        self
    }

    /// Mit Timeout (in Sekunden)
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Prüfe ob ein Constraint erfüllt ist
    pub fn constraint_satisfied(&self, constraint_type: &str) -> bool {
        self.constraints
            .iter()
            .any(|c| c.type_tag() == constraint_type)
    }
}

// ============================================================================
// Goal
// ============================================================================

/// Ziel eines Intents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Goal {
    /// Wert-Transfer
    Transfer {
        to: UniversalId,
        amount: u64,
        asset_type: String,
    },
    /// Attestation erstellen
    Attest { subject: UniversalId, claim: String },
    /// Delegation erstellen (Κ8)
    Delegate {
        to: UniversalId,
        capabilities: Vec<String>,
        trust_factor: f32,
        ttl_seconds: u64,
    },
    /// Query ausführen
    Query { predicate: String },
    /// Entität erstellen
    Create {
        entity_type: String,
        params: HashMap<String, serde_json::Value>,
    },
    /// Komplexes Ziel (natürlichsprachlich)
    Complex {
        description: String,
        sub_goals: Vec<Goal>,
    },
}

impl Goal {
    /// Type-Tag für Serialisierung
    pub fn type_tag(&self) -> &'static str {
        match self {
            Self::Transfer { .. } => "transfer",
            Self::Attest { .. } => "attest",
            Self::Delegate { .. } => "delegate",
            Self::Query { .. } => "query",
            Self::Create { .. } => "create",
            Self::Complex { .. } => "complex",
        }
    }
}

// ============================================================================
// Constraint
// ============================================================================

/// Constraint für Intent-Ausführung
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Constraint {
    /// Minimaler Trust des Counterparts
    MinTrust { value: f32 },
    /// Maximale Kosten
    MaxCost { cost: Cost },
    /// Zeitlimit (absoluter Lamport-Zeitpunkt)
    Deadline { lamport: u32 },
    /// Nur bestimmte Realms
    RealmRestriction { realm_ids: Vec<RealmId> },
    /// Nur mit menschlich verifizierten Entities
    HumanOnly,
    /// Custom Constraint
    Custom {
        name: String,
        value: serde_json::Value,
    },
}

impl Constraint {
    /// Type-Tag
    pub fn type_tag(&self) -> &'static str {
        match self {
            Self::MinTrust { .. } => "min_trust",
            Self::MaxCost { .. } => "max_cost",
            Self::Deadline { .. } => "deadline",
            Self::RealmRestriction { .. } => "realm_restriction",
            Self::HumanOnly => "human_only",
            Self::Custom { .. } => "custom",
        }
    }
}

// ============================================================================
// Saga (Κ22)
// ============================================================================

/// Eine Saga ist die Auflösung eines Intents in atomare Schritte (Κ22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Saga {
    /// Eindeutige ID
    pub id: SagaId,
    /// Ursprünglicher Intent
    pub intent_id: UniversalId,
    /// Ausführende DID
    pub executor: UniversalId,
    /// Schritte in topologischer Reihenfolge
    pub steps: Vec<SagaStep>,
    /// Aktueller Status
    pub status: SagaStatus,
    /// Verbrauchtes Budget
    pub consumed_budget: Cost,
    /// Erstellungszeitpunkt
    pub created_at: TemporalCoord,
    /// Letztes Update
    pub updated_at: TemporalCoord,
}

impl Saga {
    /// Erstelle neue Saga aus Intent
    pub fn from_intent(intent: &Intent, steps: Vec<SagaStep>, lamport: u32) -> Self {
        let id = saga_id_from_intent(&intent.id);
        let coord = TemporalCoord::now(lamport, &id);

        Self {
            id,
            intent_id: intent.id,
            executor: intent.source,
            steps,
            status: SagaStatus::Pending,
            consumed_budget: Cost::ZERO,
            created_at: coord,
            updated_at: coord,
        }
    }

    /// Aktueller Schritt-Index
    pub fn current_step_index(&self) -> Option<usize> {
        self.steps
            .iter()
            .position(|s| s.status == StepStatus::Pending)
    }

    /// Alle abgeschlossenen Schritte
    pub fn completed_steps(&self) -> Vec<&SagaStep> {
        self.steps
            .iter()
            .filter(|s| s.status == StepStatus::Completed)
            .collect()
    }

    /// Ist die Saga erfolgreich abgeschlossen?
    pub fn is_completed(&self) -> bool {
        matches!(self.status, SagaStatus::Completed)
    }

    /// Ist die Saga fehlgeschlagen?
    pub fn is_failed(&self) -> bool {
        matches!(
            self.status,
            SagaStatus::Failed { .. } | SagaStatus::Compensated
        )
    }

    /// Führe nächsten Schritt aus
    pub fn advance(&mut self, result: StepResult, lamport: u32) -> Result<(), SagaError> {
        let idx = self.current_step_index().ok_or(SagaError::NoMoreSteps)?;

        if let StepResult::Success { cost, .. } = &result {
            self.consumed_budget = self.consumed_budget.seq(*cost);
        }

        self.steps[idx].status = match &result {
            StepResult::Success { .. } => StepStatus::Completed,
            StepResult::Failed { .. } => StepStatus::Failed,
        };
        self.steps[idx].result = Some(result.clone());

        // Update Status
        if matches!(result, StepResult::Failed { .. }) {
            self.status = SagaStatus::Failed {
                at_step: idx,
                error: result.error_message().unwrap_or_default(),
            };
        } else if self.steps.iter().all(|s| s.status == StepStatus::Completed) {
            self.status = SagaStatus::Completed;
        } else {
            self.status = SagaStatus::InProgress {
                current_step: idx + 1,
            };
        }

        self.updated_at = TemporalCoord::now(lamport, &self.id);
        Ok(())
    }
}

// ============================================================================
// SagaStatus
// ============================================================================

/// Status einer Saga
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum SagaStatus {
    /// Noch nicht gestartet
    Pending,
    /// In Ausführung
    InProgress { current_step: usize },
    /// Erfolgreich abgeschlossen
    Completed,
    /// Fehlgeschlagen (Κ24: Compensation läuft)
    Failed { at_step: usize, error: String },
    /// Kompensation abgeschlossen
    Compensated,
    /// Timeout
    TimedOut,
    /// Abgebrochen durch User
    Cancelled,
}

impl fmt::Display for SagaStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress { current_step } => write!(f, "in_progress({})", current_step),
            Self::Completed => write!(f, "completed"),
            Self::Failed { at_step, error } => write!(f, "failed(step={}, {})", at_step, error),
            Self::Compensated => write!(f, "compensated"),
            Self::TimedOut => write!(f, "timed_out"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

// ============================================================================
// SagaStep
// ============================================================================

/// Ein Schritt in einer Saga
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaStep {
    /// Schritt-Index
    pub index: usize,
    /// Beschreibung
    pub description: String,
    /// Aktion
    pub action: SagaAction,
    /// Kompensations-Aktion (Κ24)
    pub compensation: Option<SagaCompensation>,
    /// Status
    pub status: StepStatus,
    /// Abhängigkeiten (Indizes anderer Schritte)
    pub dependencies: Vec<usize>,
    /// Ergebnis (wenn abgeschlossen)
    pub result: Option<StepResult>,
    /// Realm-Crossing erforderlich? (Κ23)
    pub realm_crossing: Option<RealmCrossing>,
    /// Erwartete Kosten
    pub expected_cost: Cost,
}

impl SagaStep {
    /// Erstelle neuen Schritt
    pub fn new(index: usize, description: impl Into<String>, action: SagaAction) -> Self {
        Self {
            index,
            description: description.into(),
            action,
            compensation: None,
            status: StepStatus::Pending,
            dependencies: vec![],
            result: None,
            realm_crossing: None,
            expected_cost: Cost::ZERO,
        }
    }

    /// Mit Compensation (Κ24)
    pub fn with_compensation(mut self, compensation: SagaCompensation) -> Self {
        self.compensation = Some(compensation);
        self
    }

    /// Mit Abhängigkeiten
    pub fn with_dependencies(mut self, deps: Vec<usize>) -> Self {
        self.dependencies = deps;
        self
    }

    /// Mit Realm-Crossing (Κ23)
    pub fn with_realm_crossing(mut self, from: RealmId, to: RealmId) -> Self {
        self.realm_crossing = Some(RealmCrossing { from, to });
        self
    }

    /// Mit erwarteten Kosten
    pub fn with_expected_cost(mut self, cost: Cost) -> Self {
        self.expected_cost = cost;
        self
    }

    /// Kann ausgeführt werden? (alle Dependencies erfüllt)
    pub fn can_execute(&self, completed_indices: &[usize]) -> bool {
        self.dependencies
            .iter()
            .all(|dep| completed_indices.contains(dep))
    }
}

// ============================================================================
// StepStatus
// ============================================================================

/// Status eines Saga-Schritts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Ausstehend
    Pending,
    /// In Ausführung
    Running,
    /// Abgeschlossen
    Completed,
    /// Fehlgeschlagen
    Failed,
    /// Kompensiert
    Compensated,
    /// Übersprungen
    Skipped,
}

// ============================================================================
// SagaAction
// ============================================================================

/// Aktion eines Saga-Schritts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SagaAction {
    /// Wert-Transfer
    Transfer {
        from: UniversalId,
        to: UniversalId,
        amount: u64,
        asset_type: String,
    },
    /// Attestation erstellen
    Attest {
        attester: UniversalId,
        subject: UniversalId,
        claim: String,
    },
    /// Delegation erstellen
    Delegate {
        from: UniversalId,
        to: UniversalId,
        capabilities: Vec<String>,
        trust_factor: f32,
    },
    /// ECL-Policy ausführen
    ExecutePolicy {
        policy_id: String,
        params: HashMap<String, serde_json::Value>,
    },
    /// Gateway passieren (Κ23)
    CrossRealm {
        from_realm: RealmId,
        to_realm: RealmId,
        subject: UniversalId,
    },
    /// Custom Action
    Custom { action_type: String, data: Vec<u8> },
}

// ============================================================================
// SagaCompensation (Κ24)
// ============================================================================

/// Kompensation für Saga-Schritt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaCompensation {
    /// Beschreibung
    pub description: String,
    /// Aktion zur Kompensation
    pub action: SagaAction,
}

impl SagaCompensation {
    /// Erstelle neue Kompensation
    pub fn new(description: impl Into<String>, action: SagaAction) -> Self {
        Self {
            description: description.into(),
            action,
        }
    }
}

// ============================================================================
// RealmCrossing (Κ23)
// ============================================================================

/// Realm-Crossing Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmCrossing {
    /// Quell-Realm
    pub from: RealmId,
    /// Ziel-Realm
    pub to: RealmId,
}

// ============================================================================
// StepResult
// ============================================================================

/// Ergebnis eines Saga-Schritts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum StepResult {
    /// Erfolgreich
    Success {
        /// Output-Daten
        output: Vec<u8>,
        /// Tatsächliche Kosten
        cost: Cost,
    },
    /// Fehlgeschlagen
    Failed {
        /// Fehlermeldung
        error: String,
        /// Ist der Fehler retriable?
        retriable: bool,
    },
}

impl StepResult {
    /// Fehlermeldung (wenn fehlgeschlagen)
    pub fn error_message(&self) -> Option<String> {
        match self {
            Self::Failed { error, .. } => Some(error.clone()),
            _ => None,
        }
    }

    /// Ist erfolgreich?
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Fehler bei Saga-Operationen
#[derive(Debug, Clone, thiserror::Error)]
pub enum SagaError {
    #[error("No more steps to execute")]
    NoMoreSteps,

    #[error("Step dependency not satisfied: step {step} depends on {dependency}")]
    DependencyNotSatisfied { step: usize, dependency: usize },

    #[error("Budget exhausted: required {required:?}, available {available:?}")]
    BudgetExhausted { required: Cost, available: Cost },

    #[error("Realm crossing denied: from {from} to {to}")]
    RealmCrossingDenied { from: RealmId, to: RealmId },

    #[error("Saga already completed")]
    AlreadyCompleted,

    #[error("Saga timed out")]
    TimedOut,

    #[error("Compensation failed: {0}")]
    CompensationFailed(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_creation() {
        let source = UniversalId::new(UniversalId::TAG_DID, 1, b"alice");
        let realm = UniversalId::new(UniversalId::TAG_REALM, 1, b"eu-realm");

        let intent = Intent::new(
            source,
            Goal::Transfer {
                to: UniversalId::new(UniversalId::TAG_DID, 1, b"bob"),
                amount: 100,
                asset_type: "ERY".into(),
            },
            realm,
            1,
        );

        assert!(intent.constraints.is_empty());
        assert_eq!(intent.timeout_seconds, 3600);
    }

    #[test]
    fn test_saga_from_intent() {
        let source = UniversalId::new(UniversalId::TAG_DID, 1, b"alice");
        let realm = UniversalId::new(UniversalId::TAG_REALM, 1, b"eu-realm");

        let intent = Intent::new(
            source,
            Goal::Transfer {
                to: UniversalId::new(UniversalId::TAG_DID, 1, b"bob"),
                amount: 100,
                asset_type: "ERY".into(),
            },
            realm,
            1,
        );

        let step = SagaStep::new(
            0,
            "Transfer 100 ERY to Bob",
            SagaAction::Transfer {
                from: source,
                to: UniversalId::new(UniversalId::TAG_DID, 1, b"bob"),
                amount: 100,
                asset_type: "ERY".into(),
            },
        );

        let saga = Saga::from_intent(&intent, vec![step], 2);

        assert_eq!(saga.status, SagaStatus::Pending);
        assert_eq!(saga.steps.len(), 1);
        assert_eq!(saga.consumed_budget, Cost::ZERO);
    }

    #[test]
    fn test_saga_advance() {
        let source = UniversalId::new(UniversalId::TAG_DID, 1, b"alice");
        let realm = UniversalId::new(UniversalId::TAG_REALM, 1, b"eu-realm");

        let intent = Intent::new(
            source,
            Goal::Transfer {
                to: UniversalId::new(UniversalId::TAG_DID, 1, b"bob"),
                amount: 100,
                asset_type: "ERY".into(),
            },
            realm,
            1,
        );

        let step = SagaStep::new(
            0,
            "Transfer",
            SagaAction::Transfer {
                from: source,
                to: UniversalId::new(UniversalId::TAG_DID, 1, b"bob"),
                amount: 100,
                asset_type: "ERY".into(),
            },
        );

        let mut saga = Saga::from_intent(&intent, vec![step], 2);

        saga.advance(
            StepResult::Success {
                output: vec![],
                cost: Cost::new(10, 5, 0.0),
            },
            3,
        )
        .unwrap();

        assert_eq!(saga.status, SagaStatus::Completed);
        assert!(saga.is_completed());
    }

    #[test]
    fn test_step_dependencies() {
        let step = SagaStep::new(
            1,
            "Step 1",
            SagaAction::Custom {
                action_type: "test".into(),
                data: vec![],
            },
        )
        .with_dependencies(vec![0]);

        assert!(!step.can_execute(&[]));
        assert!(step.can_execute(&[0]));
        assert!(step.can_execute(&[0, 2]));
    }

    #[test]
    fn test_saga_failure() {
        let source = UniversalId::new(UniversalId::TAG_DID, 1, b"alice");
        let realm = UniversalId::new(UniversalId::TAG_REALM, 1, b"eu-realm");

        let intent = Intent::new(
            source,
            Goal::Query {
                predicate: "test".into(),
            },
            realm,
            1,
        );

        let step = SagaStep::new(
            0,
            "Query",
            SagaAction::Custom {
                action_type: "query".into(),
                data: vec![],
            },
        );

        let mut saga = Saga::from_intent(&intent, vec![step], 2);

        saga.advance(
            StepResult::Failed {
                error: "Query failed".into(),
                retriable: false,
            },
            3,
        )
        .unwrap();

        assert!(saga.is_failed());
        assert!(matches!(saga.status, SagaStatus::Failed { at_step: 0, .. }));
    }
}
