//! # Saga Types
//!
//! **DEPRECATED:** Verwende stattdessen `domain::unified::saga`.
//!
//! Dieses Modul wird in einer zukünftigen Version entfernt.
//! Migration: `use crate::domain::unified::{Saga, SagaId, Intent, Goal, Constraint};`
//!
//! ## Axiom-Referenz
//!
//! - **Κ22 (Saga-Composer)**: `∀ Intent I : ∃! Saga S : resolve(I) = S`
//! - **Κ23 (Gateway-Guard)**: `cross(s, 𝒞₁, 𝒞₂) requires G(s, 𝒞₂) = true`
//! - **Κ24 (Atomare Kompensation)**: `fail(Sᵢ) → compensate(S₁..Sᵢ₋₁)`

#![deprecated(
    since = "0.2.0",
    note = "Use `domain::unified::saga` instead. This module will be removed in v0.3.0."
)]

use crate::domain::{RealmId, DID};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ein Intent repräsentiert eine Nutzer-Absicht (Κ22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    /// Eindeutige ID
    pub id: String,
    /// Quelle (wer hat den Intent erstellt)
    pub source_did: DID,
    /// Ziel-Zustand
    pub goal: Goal,
    /// Constraints für die Ausführung
    pub constraints: Vec<Constraint>,
    /// Budget für die Ausführung
    pub budget: Budget,
    /// Kontext (aktuelles Realm)
    pub context_realm: RealmId,
    /// Erstellungszeitpunkt
    pub created_at: DateTime<Utc>,
    /// Timeout
    pub timeout: Duration,
}

impl Intent {
    /// Erstelle neuen Intent
    pub fn new(source_did: DID, goal: Goal, context_realm: RealmId) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            source_did,
            goal,
            constraints: vec![],
            budget: Budget::default(),
            context_realm,
            created_at: Utc::now(),
            timeout: Duration::hours(1),
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

    /// Mit Timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Ziel eines Intents
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Goal {
    /// Wert-Transfer
    Transfer {
        to: DID,
        amount: u64,
        asset_type: String,
    },
    /// Attestation erstellen
    Attest { subject: DID, claim: String },
    /// Delegation erstellen
    Delegate {
        to: DID,
        capabilities: Vec<String>,
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
        parsed_goals: Vec<Goal>,
    },
}

/// Constraint für Intent-Ausführung
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Constraint {
    /// Minimaler Trust des Counterparts
    MinTrust { value: f64 },
    /// Maximale Kosten
    MaxCost { amount: u64, asset_type: String },
    /// Zeitlimit
    Deadline { at: DateTime<Utc> },
    /// Nur bestimmte Realms
    RealmRestriction { realm_ids: Vec<RealmId> },
    /// Nur mit menschlich verifizierten Entities
    HumanOnly,
}

/// Budget für Intent-Ausführung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Maximaler Betrag
    pub max_amount: u64,
    /// Asset-Typ
    pub asset_type: String,
    /// Reserviert für Gas/Fees
    pub reserved_for_fees: u64,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            max_amount: 0,
            asset_type: "ERY".to_string(),
            reserved_for_fees: 0,
        }
    }
}

/// Eine Saga ist die Auflösung eines Intents in atomare Schritte (Κ22)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Saga {
    /// Eindeutige ID
    pub id: String,
    /// Ursprünglicher Intent
    pub intent_id: String,
    /// Ausführende DID
    pub executor: DID,
    /// Schritte in topologischer Reihenfolge
    pub steps: Vec<SagaStep>,
    /// Aktueller Status
    pub status: SagaStatus,
    /// Timeout
    pub timeout_at: DateTime<Utc>,
    /// Erstellungszeitpunkt
    pub created_at: DateTime<Utc>,
    /// Letztes Update
    pub updated_at: DateTime<Utc>,
}

impl Saga {
    /// Erstelle neue Saga aus Intent
    pub fn from_intent(intent: &Intent, steps: Vec<SagaStep>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            intent_id: intent.id.clone(),
            executor: intent.source_did.clone(),
            steps,
            status: SagaStatus::Pending,
            timeout_at: now + intent.timeout,
            created_at: now,
            updated_at: now,
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
}

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
    /// Realm-Crossing erforderlich?
    pub realm_crossing: Option<RealmCrossing>,
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

    /// Mit Realm-Crossing
    pub fn with_realm_crossing(mut self, from: RealmId, to: RealmId) -> Self {
        self.realm_crossing = Some(RealmCrossing { from, to });
        self
    }
}

/// Aktion eines Saga-Schritts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SagaAction {
    /// Funds sperren
    Lock {
        did: DID,
        amount: u64,
        asset_type: String,
    },
    /// Funds entsperren
    Unlock { lock_id: String },
    /// Transfer ausführen
    Transfer {
        from: DID,
        to: DID,
        amount: u64,
        asset_type: String,
    },
    /// Asset minten
    Mint {
        to: DID,
        amount: u64,
        asset_type: String,
    },
    /// Asset burnen
    Burn {
        from: DID,
        amount: u64,
        asset_type: String,
    },
    /// Gateway-Check (Κ23)
    GatewayCheck { did: DID, target_realm: RealmId },
    /// Externe Chain-Operation
    ExternalChain {
        chain: String,
        operation: String,
        params: serde_json::Value,
    },
    /// Warten auf Bedingung
    WaitFor {
        condition: String,
        timeout_seconds: u64,
    },
}

/// Kompensations-Aktion (Κ24)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaCompensation {
    /// Beschreibung
    pub description: String,
    /// Aktion
    pub action: SagaAction,
    /// Priorität (höher = früher ausführen bei Rollback)
    pub priority: u32,
}

impl SagaCompensation {
    pub fn new(description: impl Into<String>, action: SagaAction) -> Self {
        Self {
            description: description.into(),
            action,
            priority: 0,
        }
    }
}

/// Status eines Saga-Schritts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Noch nicht gestartet
    Pending,
    /// In Ausführung
    Running,
    /// Erfolgreich
    Completed,
    /// Fehlgeschlagen
    Failed,
    /// Übersprungen
    Skipped,
    /// Kompensiert
    Compensated,
}

/// Ergebnis eines Schritts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Erfolg?
    pub success: bool,
    /// Transaktions-ID (wenn extern)
    pub tx_id: Option<String>,
    /// Event-ID (wenn intern)
    pub event_id: Option<String>,
    /// Fehlermeldung
    pub error: Option<String>,
    /// Dauer in ms
    pub duration_ms: u64,
    /// Zusätzliche Daten
    pub data: Option<serde_json::Value>,
}

/// Realm-Crossing Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmCrossing {
    pub from: RealmId,
    pub to: RealmId,
}

// Tests moved to domain::unified::saga - this module is deprecated
// #[cfg(test)]
// mod tests { ... }
