//! # Event Types
//!
//! **DEPRECATED:** Verwende stattdessen `domain::unified::event`.
//!
//! Dieses Modul wird in einer zukünftigen Version entfernt.
//! Migration: `use crate::domain::unified::{Event, EventId, EventPayload, FinalityState};`
//!
//! ## Axiom-Referenz
//!
//! - **Κ9 (Kausale Struktur)**: `ℂ = (E, ⊲)` ist ein DAG
//! - **Κ10 (Bezeugung-Finalität)**: `⟦e⟧ → □⟦e⟧` (Permanenz der Bezeugung)
//! - **Κ11 (Prozess-Korrektheit)**: `{pre} Π {post}`
//! - **Κ12 (Event-Erzeugung)**: `∀Π : ⟦Π⟧ → Δ|ℂ| ≥ 1`

#![deprecated(
    since = "0.2.0",
    note = "Use `domain::unified::event` instead. This module will be removed in v0.3.0."
)]

use crate::domain::DID;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Event-Identifikator (Hash des Event-Inhalts)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(pub String);

impl EventId {
    /// Erstelle aus Hash-String
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Generiere neue EventId basierend auf Content-Hash
    pub fn from_content(content: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        Self(format!("event:sha3:{}", hex::encode(&result[..16])))
    }

    /// Als Byte-Slice
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl From<&str> for EventId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for EventId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Finalitätslevel eines Events (Κ10)
///
/// ```text
/// NASCENT → VALIDATED → WITNESSED → ANCHORED → ETERNAL
/// (0.5)     (0.9)       (0.99)      (0.999)    (1 - 10⁻⁵⁰)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FinalityLevel {
    /// Neu erstellt, noch nicht validiert
    Nascent,
    /// Signatur gültig, Parents existieren
    Validated,
    /// Von n Witnesses mit Trust ≥ θ bestätigt
    Witnessed,
    /// Merkle-Root in externem System verankert
    Anchored,
    /// Irreversibel (nach k Bestätigungen)
    Eternal,
}

impl FinalityLevel {
    /// Wahrscheinlichkeit dass Event nicht revertiert wird
    pub fn probability(&self) -> f64 {
        match self {
            FinalityLevel::Nascent => 0.5,
            FinalityLevel::Validated => 0.9,
            FinalityLevel::Witnessed => 0.99,
            FinalityLevel::Anchored => 0.999,
            FinalityLevel::Eternal => 1.0 - 1e-50,
        }
    }

    /// Ist final genug für kritische Operationen?
    pub fn is_sufficient_for_critical(&self) -> bool {
        matches!(
            self,
            FinalityLevel::Witnessed | FinalityLevel::Anchored | FinalityLevel::Eternal
        )
    }
}

impl Default for FinalityLevel {
    fn default() -> Self {
        Self::Nascent
    }
}

/// Event-Typen im System
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    /// Genesis-Event für neue DID
    Genesis { did: DID, public_key: String },

    /// Wert-Transfer
    Transfer {
        from: DID,
        to: DID,
        amount: u64,
        asset_type: String,
    },

    /// Asset-Erzeugung
    Mint {
        to: DID,
        amount: u64,
        asset_type: String,
    },

    /// Asset-Vernichtung
    Burn {
        from: DID,
        amount: u64,
        asset_type: String,
    },

    /// Attestation (Trust-relevant)
    Attest {
        subject: DID,
        claim: String,
        evidence: Option<String>,
    },

    /// Credential-Ausstellung
    CredentialIssue {
        holder: DID,
        credential_type: String,
        claims: serde_json::Value,
    },

    /// Credential-Widerruf
    CredentialRevoke {
        credential_id: String,
        reason: String,
    },

    /// Delegation (Κ8)
    Delegate {
        from: DID,
        to: DID,
        capabilities: Vec<String>,
        trust_factor: f64,
    },

    /// Delegation-Widerruf
    DelegationRevoke { delegation_id: String },

    /// Governance-Vorschlag
    Proposal {
        proposer: DID,
        title: String,
        description: String,
        changes: serde_json::Value,
    },

    /// Governance-Abstimmung (Κ21: Quadratisch)
    Vote {
        voter: DID,
        proposal_id: String,
        direction: VoteDirection,
        weight: u64,
    },

    /// Saga-Schritt (Κ22-Κ24)
    SagaStep {
        saga_id: String,
        step_index: u32,
        action: String,
        result: SagaStepResult,
    },

    /// Custom Event
    Custom {
        event_type: String,
        data: serde_json::Value,
    },
}

/// Abstimmungsrichtung
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoteDirection {
    For,
    Against,
    Abstain,
}

/// Ergebnis eines Saga-Schritts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SagaStepResult {
    Success,
    Failed { error: String },
    Compensated,
}

/// Ein Event im kausalen DAG (Κ9)
///
/// ```text
///                    ┌─────┐
///                    │ e₁  │ Genesis
///                    └──┬──┘
///              ┌───────┴───────┐
///              ▼               ▼
///          ┌─────┐         ┌─────┐
///          │ e₂  │         │ e₃  │
///          └──┬──┘         └──┬──┘
///             │    ┌──────────┘
///             ▼    ▼
///          ┌─────────┐
///          │   e₄    │  (e₂ ⊲ e₄) ∧ (e₃ ⊲ e₄)
///          └────┬────┘
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Eindeutige ID (Hash)
    pub id: EventId,

    /// Kausale Vorgänger (⊲ Relation, Κ9)
    pub parents: Vec<EventId>,

    /// Ersteller des Events
    pub author: DID,

    /// Logischer Zeitstempel
    pub timestamp: DateTime<Utc>,

    /// Event-Inhalt
    pub payload: EventPayload,

    /// Ed25519-Signatur
    pub signature: Option<String>,

    /// Aktuelles Finalitätslevel (Κ10)
    pub finality: FinalityLevel,

    /// Realm in dem dieses Event existiert
    pub realm_id: Option<String>,
}

impl Event {
    /// Erstelle neues Event
    pub fn new(author: DID, payload: EventPayload, parents: Vec<EventId>) -> Self {
        let timestamp = Utc::now();

        // Temporäre ID (wird nach Signatur finalisiert)
        let temp_content =
            serde_json::to_vec(&(&author, &payload, &parents, &timestamp)).unwrap_or_default();
        let id = EventId::from_content(&temp_content);

        Self {
            id,
            parents,
            author,
            timestamp,
            payload,
            signature: None,
            finality: FinalityLevel::Nascent,
            realm_id: None,
        }
    }

    /// Erstelle Genesis-Event für neue DID
    pub fn genesis(did: DID, public_key: String) -> Self {
        Self::new(
            did.clone(),
            EventPayload::Genesis { did, public_key },
            vec![], // Genesis hat keine Parents
        )
    }

    /// Prüft ob dieses Event ein negativer Trust-Indikator ist
    pub fn is_negative_trust(&self) -> bool {
        matches!(
            &self.payload,
            EventPayload::CredentialRevoke { .. }
                | EventPayload::DelegationRevoke { .. }
                | EventPayload::SagaStep {
                    result: SagaStepResult::Failed { .. },
                    ..
                }
        )
    }

    /// Prüft ob dieses Event Trust-relevant ist
    pub fn is_trust_relevant(&self) -> bool {
        matches!(
            &self.payload,
            EventPayload::Attest { .. }
                | EventPayload::CredentialIssue { .. }
                | EventPayload::CredentialRevoke { .. }
                | EventPayload::Transfer { .. }
        )
    }

    /// Welche Trust-Dimension wird primär beeinflusst?
    pub fn primary_trust_dimension(&self) -> Option<crate::domain::TrustDimension> {
        use crate::domain::TrustDimension;
        match &self.payload {
            EventPayload::Transfer { .. } => Some(TrustDimension::Reliability),
            EventPayload::Attest { .. } => Some(TrustDimension::Integrity),
            EventPayload::CredentialIssue { .. } => Some(TrustDimension::Competence),
            EventPayload::Vote { .. } => Some(TrustDimension::Omega),
            _ => None,
        }
    }

    /// Setze Signatur
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self.finality = FinalityLevel::Validated;
        self
    }

    /// Setze Realm
    pub fn in_realm(mut self, realm_id: String) -> Self {
        self.realm_id = Some(realm_id);
        self
    }
}

/// Witness-Attestation für ein Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessAttestation {
    /// Event das bezeugt wird
    pub event_id: EventId,

    /// Witness DID
    pub witness: DID,

    /// Trust-gewichtete Stimme
    pub trust_weight: f64,

    /// Signatur des Witness
    pub signature: String,

    /// Zeitstempel
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let author = DID::new_self("alice");
        let event = Event::new(
            author.clone(),
            EventPayload::Transfer {
                from: author.clone(),
                to: DID::new_self("bob"),
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            vec![],
        );

        assert_eq!(event.finality, FinalityLevel::Nascent);
        assert!(event.id.0.starts_with("event:sha3:"));
    }

    #[test]
    fn test_genesis_event() {
        let did = DID::new_self("alice");
        let event = Event::genesis(did.clone(), "pubkey123".to_string());

        assert!(event.parents.is_empty());
        matches!(event.payload, EventPayload::Genesis { .. });
    }

    #[test]
    fn test_finality_ordering() {
        assert!(FinalityLevel::Nascent < FinalityLevel::Validated);
        assert!(FinalityLevel::Validated < FinalityLevel::Witnessed);
        assert!(FinalityLevel::Witnessed < FinalityLevel::Anchored);
        assert!(FinalityLevel::Anchored < FinalityLevel::Eternal);
    }
}
