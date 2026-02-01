//! # Unified Data Model – Events
//!
//! Kausale Events im DAG gemäß Axiome Κ9-Κ12.
//!
//! ## Axiom-Referenz
//!
//! - **Κ9 (Kausale Struktur)**: `ℂ = (E, ⊲)` ist ein DAG
//! - **Κ10 (Bezeugung-Finalität)**: `⟦e⟧ → □⟦e⟧` (Permanenz der Bezeugung)
//! - **Κ11 (Prozess-Korrektheit)**: `{pre} Π {post}`
//! - **Κ12 (Event-Erzeugung)**: `∀Π : ⟦Π⟧ → Δ|ℂ| ≥ 1`
//!
//! ## Migration von domain/event.rs
//!
//! - `EventId` ist jetzt `UniversalId` (TAG_EVENT)
//! - `FinalityState` statt `FinalityLevel` mit mehr Details
//! - `TemporalCoord` statt `DateTime<Utc>`

use super::identity::DID;
use super::primitives::{TemporalCoord, UniversalId};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// EventId – Type Alias für UniversalId
// ============================================================================

/// Event-Identifikator (Content-Addressed via UniversalId)
pub type EventId = UniversalId;

/// Erstelle EventId aus Content
pub fn event_id_from_content(content: &[u8]) -> EventId {
    UniversalId::new(UniversalId::TAG_EVENT, 1, content)
}

// ============================================================================
// Signature Wrapper (für Serde-Kompatibilität)
// ============================================================================

/// 64-Byte Signatur (Ed25519)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature64(pub [u8; 64]);

impl Signature64 {
    /// Null-Signatur
    pub const NULL: Self = Self([0u8; 64]);

    /// Erstelle aus Slice
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 64 {
            return None;
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(slice);
        Some(Self(arr))
    }

    /// Als Slice
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

impl Default for Signature64 {
    fn default() -> Self {
        Self::NULL
    }
}

impl Serialize for Signature64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Signature64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        Self::from_slice(&bytes).ok_or_else(|| serde::de::Error::custom("invalid signature length"))
    }
}

// ============================================================================
// Hash32 Wrapper (für Serde-Kompatibilität)
// ============================================================================

/// 32-Byte Hash (SHA256, Blake3)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hash32(pub [u8; 32]);

impl Hash32 {
    /// Null-Hash
    pub const NULL: Self = Self([0u8; 32]);

    /// Erstelle aus Slice
    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() != 32 {
            return None;
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(slice);
        Some(Self(arr))
    }

    /// Als Slice
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl Default for Hash32 {
    fn default() -> Self {
        Self::NULL
    }
}

impl Serialize for Hash32 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Hash32 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
        Self::from_slice(&bytes).ok_or_else(|| serde::de::Error::custom("invalid hash length"))
    }
}

// ============================================================================
// FinalityLevel
// ============================================================================

/// Finalitätslevel eines Events (Κ10)
///
/// ```text
/// NASCENT → VALIDATED → WITNESSED → ANCHORED → ETERNAL
/// (0.5)     (0.9)       (0.99)      (0.999)    (1 - 10⁻⁵⁰)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum FinalityLevel {
    /// Neu erstellt, noch nicht validiert
    Nascent = 0,
    /// Signatur gültig, Parents existieren
    Validated = 1,
    /// Von n Witnesses mit Trust ≥ θ bestätigt
    Witnessed = 2,
    /// Merkle-Root in externem System verankert
    Anchored = 3,
    /// Irreversibel (nach k Bestätigungen)
    Eternal = 4,
}

impl FinalityLevel {
    /// Wahrscheinlichkeit dass Event nicht revertiert wird
    pub fn probability(&self) -> f64 {
        match self {
            Self::Nascent => 0.5,
            Self::Validated => 0.9,
            Self::Witnessed => 0.99,
            Self::Anchored => 0.999,
            Self::Eternal => 1.0 - 1e-50,
        }
    }

    /// Ist final genug für kritische Operationen?
    pub fn is_sufficient_for_critical(&self) -> bool {
        matches!(self, Self::Witnessed | Self::Anchored | Self::Eternal)
    }

    /// Nächster Level
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Nascent => Some(Self::Validated),
            Self::Validated => Some(Self::Witnessed),
            Self::Witnessed => Some(Self::Anchored),
            Self::Anchored => Some(Self::Eternal),
            Self::Eternal => None,
        }
    }

    /// Von u8
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Nascent),
            1 => Some(Self::Validated),
            2 => Some(Self::Witnessed),
            3 => Some(Self::Anchored),
            4 => Some(Self::Eternal),
            _ => None,
        }
    }
}

impl Default for FinalityLevel {
    fn default() -> Self {
        Self::Nascent
    }
}

impl fmt::Display for FinalityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Nascent => write!(f, "Nascent"),
            Self::Validated => write!(f, "Validated"),
            Self::Witnessed => write!(f, "Witnessed"),
            Self::Anchored => write!(f, "Anchored"),
            Self::Eternal => write!(f, "Eternal"),
        }
    }
}

// ============================================================================
// FinalityState – Erweiterter Finalitäts-Zustand
// ============================================================================

/// Erweiterter Finalitäts-Zustand mit Details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalityState {
    /// Aktuelles Level
    pub level: FinalityLevel,
    /// Berechnete Wahrscheinlichkeit (kann von level.probability() abweichen)
    pub probability: f64,
    /// Anzahl der Witnesses
    pub witness_count: u32,
    /// Minimaler Trust der Witnesses
    pub min_witness_trust: f32,
    /// Anchor-Hash (wenn Anchored/Eternal)
    pub anchor_hash: Option<Hash32>,
    /// Anchor-System (z.B. "ethereum", "bitcoin", "cosmos")
    pub anchor_system: Option<String>,
    /// Letztes Update
    pub updated_at: TemporalCoord,
}

impl FinalityState {
    /// Erstelle neuen Nascent-State
    pub fn nascent(coord: TemporalCoord) -> Self {
        Self {
            level: FinalityLevel::Nascent,
            probability: 0.5,
            witness_count: 0,
            min_witness_trust: 0.0,
            anchor_hash: None,
            anchor_system: None,
            updated_at: coord,
        }
    }

    /// Transition zu Validated
    pub fn validate(&mut self, coord: TemporalCoord) -> Result<(), FinalityError> {
        if self.level != FinalityLevel::Nascent {
            return Err(FinalityError::InvalidTransition {
                from: self.level,
                to: FinalityLevel::Validated,
            });
        }

        self.level = FinalityLevel::Validated;
        self.probability = 0.9;
        self.updated_at = coord;
        Ok(())
    }

    /// Transition zu Witnessed
    pub fn witness(
        &mut self,
        witness_count: u32,
        min_trust: f32,
        coord: TemporalCoord,
    ) -> Result<(), FinalityError> {
        if self.level != FinalityLevel::Validated {
            return Err(FinalityError::InvalidTransition {
                from: self.level,
                to: FinalityLevel::Witnessed,
            });
        }

        self.level = FinalityLevel::Witnessed;
        self.witness_count = witness_count;
        self.min_witness_trust = min_trust;
        // Probability basiert auf Witness-Count
        self.probability = 1.0 - (0.1_f64).powi(witness_count as i32);
        self.updated_at = coord;
        Ok(())
    }

    /// Transition zu Anchored
    pub fn anchor(
        &mut self,
        hash: Hash32,
        system: impl Into<String>,
        coord: TemporalCoord,
    ) -> Result<(), FinalityError> {
        if self.level != FinalityLevel::Witnessed {
            return Err(FinalityError::InvalidTransition {
                from: self.level,
                to: FinalityLevel::Anchored,
            });
        }

        self.level = FinalityLevel::Anchored;
        self.probability = 0.999;
        self.anchor_hash = Some(hash);
        self.anchor_system = Some(system.into());
        self.updated_at = coord;
        Ok(())
    }

    /// Transition zu Eternal
    pub fn eternalize(&mut self, coord: TemporalCoord) -> Result<(), FinalityError> {
        if self.level != FinalityLevel::Anchored {
            return Err(FinalityError::InvalidTransition {
                from: self.level,
                to: FinalityLevel::Eternal,
            });
        }

        self.level = FinalityLevel::Eternal;
        self.probability = 1.0 - 1e-50;
        self.updated_at = coord;
        Ok(())
    }

    /// Ist das Event final genug für kritische Operationen?
    pub fn is_sufficient_for_critical(&self) -> bool {
        self.level.is_sufficient_for_critical()
    }
}

impl Default for FinalityState {
    fn default() -> Self {
        Self::nascent(TemporalCoord::default())
    }
}

// ============================================================================
// Event
// ============================================================================

/// Ein Event im kausalen DAG (Κ9-Κ12)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Eindeutige ID (Content-Addressed)
    pub id: EventId,
    /// Temporale Koordinate
    pub coord: TemporalCoord,
    /// Parent-Events (für DAG-Struktur, Κ9)
    pub parents: Vec<EventId>,
    /// Ersteller des Events
    pub author: UniversalId,
    /// Event-Payload
    pub payload: EventPayload,
    /// Finalitäts-Zustand (Κ10)
    pub finality: FinalityState,
    /// Signatur des Autors
    pub signature: Signature64,
}

impl Event {
    /// Erstelle neues Event
    pub fn new(
        author: UniversalId,
        parents: Vec<EventId>,
        payload: EventPayload,
        lamport: u32,
    ) -> Self {
        // Content für ID-Generierung
        let mut content = Vec::new();
        content.extend_from_slice(author.as_bytes());
        for p in &parents {
            content.extend_from_slice(p.as_bytes());
        }
        // Payload-Hash (vereinfacht)
        content.extend_from_slice(payload.type_tag().as_bytes());

        let id = event_id_from_content(&content);
        let coord = TemporalCoord::now(lamport, &id);

        Self {
            id,
            coord,
            parents,
            author,
            payload,
            finality: FinalityState::nascent(coord),
            signature: Signature64::NULL,
        }
    }

    /// Signiere das Event
    pub fn sign(&mut self, signature: Signature64) {
        self.signature = signature;
    }

    /// Prüfe kausale Ordnung (Κ9)
    pub fn is_causally_after(&self, other: &Event) -> bool {
        // Entweder direkter Parent oder transitiv
        self.parents.contains(&other.id) || self.coord > other.coord
    }

    /// Ist das Event ein Genesis-Event?
    pub fn is_genesis(&self) -> bool {
        self.parents.is_empty()
    }

    /// Validiere Parent-Existenz
    pub fn validate_parents<F>(&self, exists: F) -> Result<(), EventError>
    where
        F: Fn(&EventId) -> bool,
    {
        for parent in &self.parents {
            if !exists(parent) {
                return Err(EventError::ParentNotFound(*parent));
            }
        }
        Ok(())
    }

    /// Timestamp Accessor (Kompatibilität mit altem API)
    #[inline]
    pub fn timestamp(&self) -> u64 {
        self.coord.wall_time()
    }

    /// Primäre Trust-Dimension dieses Events
    pub fn primary_trust_dimension(&self) -> Option<super::trust::TrustDimension> {
        use super::trust::TrustDimension;
        match &self.payload {
            EventPayload::Transfer { .. } => Some(TrustDimension::Reliability),
            EventPayload::Attest { .. } => Some(TrustDimension::Prestige),
            EventPayload::Delegate { .. } => Some(TrustDimension::Reliability),
            EventPayload::Witness { .. } => Some(TrustDimension::Integrity),
            EventPayload::CredentialIssue { .. } => Some(TrustDimension::Competence),
            EventPayload::CredentialRevoke { .. } => Some(TrustDimension::Vigilance),
            _ => None,
        }
    }

    /// Ist dieses Event ein negativer Trust-Event?
    pub fn is_negative_trust(&self) -> bool {
        matches!(
            &self.payload,
            EventPayload::DelegationRevoke { .. } | EventPayload::CredentialRevoke { .. }
        )
    }

    /// Erstelle Genesis-Event
    pub fn genesis(author: UniversalId, did: DID, lamport: u32) -> Self {
        Self::new(
            author,
            vec![],
            EventPayload::Genesis {
                did: did.clone(),
                public_key_hex: did.public_key_hex(),
            },
            lamport,
        )
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Event {}

impl std::hash::Hash for Event {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// ============================================================================
// EventPayload
// ============================================================================

/// Event-Payload-Typen
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    /// Genesis-Event für neue DID
    Genesis { did: DID, public_key_hex: String },

    /// Wert-Transfer
    Transfer {
        from: UniversalId,
        to: UniversalId,
        amount: u64,
        asset_type: String,
    },

    /// Asset-Erzeugung
    Mint {
        to: UniversalId,
        amount: u64,
        asset_type: String,
    },

    /// Asset-Vernichtung
    Burn {
        from: UniversalId,
        amount: u64,
        asset_type: String,
    },

    /// Attestation (Trust-relevant)
    Attest {
        subject: UniversalId,
        claim: String,
        evidence_hash: Option<Hash32>,
    },

    /// Delegation (Κ8)
    Delegate {
        from: UniversalId,
        to: UniversalId,
        capabilities: Vec<String>,
        trust_factor: f32,
    },

    /// Delegation-Widerruf
    DelegationRevoke { delegation_id: UniversalId },

    /// Governance-Vorschlag
    Proposal {
        proposer: UniversalId,
        title: String,
        description_hash: Hash32,
    },

    /// Governance-Abstimmung (Κ21: Quadratisch)
    Vote {
        voter: UniversalId,
        proposal_id: UniversalId,
        direction: VoteDirection,
        weight: u64,
    },

    /// Saga-Schritt (Κ22-Κ24)
    SagaStep {
        saga_id: UniversalId,
        step_index: u32,
        action: String,
        result: SagaStepResult,
    },

    /// Witness-Attestation (Κ10)
    Witness {
        event_id: EventId,
        witness: UniversalId,
        trust_at_witness: f32,
    },

    /// Anchor-Bestätigung
    AnchorConfirm {
        event_ids: Vec<EventId>,
        anchor_hash: Hash32,
        anchor_system: String,
    },

    /// Custom Event
    Custom { event_type: String, data: Vec<u8> },

    /// Credential Issue (für Attestations)
    CredentialIssue {
        subject: UniversalId,
        credential_type: String,
        claims: std::collections::HashMap<String, serde_json::Value>,
    },

    /// Credential Revoke
    CredentialRevoke {
        credential_id: UniversalId,
        reason: String,
    },

    /// Trust Update (für explizite Trust-Änderungen)
    TrustUpdate {
        subject: UniversalId,
        dimension: super::trust::TrustDimension,
        delta: f32,
        reason: String,
    },
}

impl EventPayload {
    /// Type-Tag für Serialisierung
    pub fn type_tag(&self) -> &'static str {
        match self {
            Self::Genesis { .. } => "genesis",
            Self::Transfer { .. } => "transfer",
            Self::Mint { .. } => "mint",
            Self::Burn { .. } => "burn",
            Self::Attest { .. } => "attest",
            Self::Delegate { .. } => "delegate",
            Self::DelegationRevoke { .. } => "delegation_revoke",
            Self::Proposal { .. } => "proposal",
            Self::Vote { .. } => "vote",
            Self::SagaStep { .. } => "saga_step",
            Self::Witness { .. } => "witness",
            Self::AnchorConfirm { .. } => "anchor_confirm",
            Self::Custom { .. } => "custom",
            Self::CredentialIssue { .. } => "credential_issue",
            Self::CredentialRevoke { .. } => "credential_revoke",
            Self::TrustUpdate { .. } => "trust_update",
        }
    }
}

// ============================================================================
// VoteDirection
// ============================================================================

/// Abstimmungsrichtung
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VoteDirection {
    For,
    Against,
    Abstain,
}

// ============================================================================
// SagaStepResult
// ============================================================================

/// Ergebnis eines Saga-Schritts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SagaStepResult {
    Success { output: Vec<u8> },
    Failed { reason: String },
    Compensated,
    Pending,
}

// ============================================================================
// WitnessAttestation
// ============================================================================

/// Witness-Attestation für ein Event (Κ10)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessAttestation {
    /// Event das bezeugt wird
    pub event_id: EventId,
    /// Witness DID
    pub witness: UniversalId,
    /// Trust des Witness zum Zeitpunkt der Bezeugung
    pub trust_at_witness: f32,
    /// Signatur
    pub signature: Signature64,
    /// Zeitstempel
    pub attested_at: TemporalCoord,
}

impl WitnessAttestation {
    /// Erstelle neue Attestation
    pub fn new(event_id: EventId, witness: UniversalId, trust: f32) -> Self {
        let coord = TemporalCoord::now(0, &event_id);
        Self {
            event_id,
            witness,
            trust_at_witness: trust,
            signature: Signature64::NULL,
            attested_at: coord,
        }
    }

    /// Signiere die Attestation
    pub fn sign(&mut self, signature: Signature64) {
        self.signature = signature;
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Fehler bei Event-Operationen
#[derive(Debug, Clone, thiserror::Error)]
pub enum EventError {
    #[error("Parent event not found: {0}")]
    ParentNotFound(EventId),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Causal order violation: {child} is not after {parent}")]
    CausalOrderViolation { parent: EventId, child: EventId },

    #[error("Event already exists: {0}")]
    AlreadyExists(EventId),
}

/// Fehler bei Finalitäts-Operationen
#[derive(Debug, Clone, thiserror::Error)]
pub enum FinalityError {
    #[error("Invalid finality transition from {from} to {to}")]
    InvalidTransition {
        from: FinalityLevel,
        to: FinalityLevel,
    },

    #[error("Finality regression not allowed (Κ10)")]
    RegressionNotAllowed,

    #[error("Insufficient witnesses: {actual} < {required}")]
    InsufficientWitnesses { required: u32, actual: u32 },
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let author = UniversalId::new(UniversalId::TAG_DID, 1, b"author");
        let event = Event::new(
            author,
            vec![],
            EventPayload::Custom {
                event_type: "test".into(),
                data: vec![],
            },
            1,
        );

        assert!(event.is_genesis());
        assert_eq!(event.finality.level, FinalityLevel::Nascent);
    }

    #[test]
    fn test_finality_transitions() {
        let coord = TemporalCoord::default();
        let mut state = FinalityState::nascent(coord);

        assert_eq!(state.level, FinalityLevel::Nascent);

        state.validate(coord).unwrap();
        assert_eq!(state.level, FinalityLevel::Validated);

        state.witness(3, 0.7, coord).unwrap();
        assert_eq!(state.level, FinalityLevel::Witnessed);
        assert_eq!(state.witness_count, 3);

        state.anchor(Hash32([1u8; 32]), "ethereum", coord).unwrap();
        assert_eq!(state.level, FinalityLevel::Anchored);
        assert!(state.anchor_hash.is_some());

        state.eternalize(coord).unwrap();
        assert_eq!(state.level, FinalityLevel::Eternal);
    }

    #[test]
    fn test_invalid_finality_transition() {
        let coord = TemporalCoord::default();
        let mut state = FinalityState::nascent(coord);

        // Kann nicht direkt zu Witnessed springen
        let result = state.witness(3, 0.7, coord);
        assert!(result.is_err());
    }

    #[test]
    fn test_event_causal_order() {
        let author = UniversalId::new(UniversalId::TAG_DID, 1, b"author");

        let event1 = Event::new(
            author,
            vec![],
            EventPayload::Custom {
                event_type: "first".into(),
                data: vec![],
            },
            1,
        );

        let event2 = Event::new(
            author,
            vec![event1.id],
            EventPayload::Custom {
                event_type: "second".into(),
                data: vec![],
            },
            2,
        );

        assert!(event2.is_causally_after(&event1));
        assert!(!event1.is_causally_after(&event2));
    }

    #[test]
    fn test_finality_level_ordering() {
        assert!(FinalityLevel::Nascent < FinalityLevel::Validated);
        assert!(FinalityLevel::Validated < FinalityLevel::Witnessed);
        assert!(FinalityLevel::Witnessed < FinalityLevel::Anchored);
        assert!(FinalityLevel::Anchored < FinalityLevel::Eternal);
    }

    #[test]
    fn test_event_payload_type_tag() {
        let payload = EventPayload::Transfer {
            from: UniversalId::NULL,
            to: UniversalId::NULL,
            amount: 100,
            asset_type: "ERY".into(),
        };

        assert_eq!(payload.type_tag(), "transfer");
    }

    #[test]
    fn test_signature64_serde() {
        let sig = Signature64([0xAB; 64]);
        let json = serde_json::to_string(&sig).unwrap();
        assert!(json.contains("ab"));

        let decoded: Signature64 = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.0, [0xAB; 64]);
    }

    #[test]
    fn test_hash32_serde() {
        let hash = Hash32([0xCD; 32]);
        let json = serde_json::to_string(&hash).unwrap();
        assert!(json.contains("cd"));

        let decoded: Hash32 = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.0, [0xCD; 32]);
    }
}
