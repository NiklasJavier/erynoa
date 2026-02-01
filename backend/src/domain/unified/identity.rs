//! # Unified Data Model – Identity (DID)
//!
//! Dezentrale Identifikatoren gemäß Axiome Κ6-Κ8.
//!
//! ## Axiom-Referenz
//!
//! - **Κ6 (Existenz-Eindeutigkeit)**: `∀ entity e : ∃! did ∈ DID : identity(e) = did`
//! - **Κ7 (Permanenz)**: `⟨s⟩ ∧ ⟦create(s)⟧ ⟹ □⟨s⟩`
//! - **Κ8 (Delegations-Struktur)**: `s ⊳ s' → 𝕋(s') ≤ 𝕋(s)`
//!
//! ## Migration von domain/did.rs
//!
//! Diese Datei verwendet `UniversalId` statt String-basierter IDs.

use super::primitives::{TemporalCoord, UniversalId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ============================================================================
// DIDNamespace
// ============================================================================

/// DID Namespace gemäß Erynoa-Spezifikation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[repr(u8)]
pub enum DIDNamespace {
    /// Natürliche Personen
    Self_ = 0x01,
    /// Organisationen, Firmen, DAOs
    Guild = 0x02,
    /// KI-Agenten, autonome Systeme
    Spirit = 0x03,
    /// IoT-Geräte, physische Assets
    Thing = 0x04,
    /// Container, Transportmittel
    Vessel = 0x05,
    /// Datenquellen, APIs
    Source = 0x06,
    /// Dienstleistungen, Handwerke
    Craft = 0x07,
    /// Speicher, Safes
    Vault = 0x08,
    /// Verträge, Vereinbarungen
    Pact = 0x09,
    /// Gruppen, Communities
    Circle = 0x0A,
}

impl DIDNamespace {
    /// Alle Namespaces
    pub const ALL: [Self; 10] = [
        Self::Self_,
        Self::Guild,
        Self::Spirit,
        Self::Thing,
        Self::Vessel,
        Self::Source,
        Self::Craft,
        Self::Vault,
        Self::Pact,
        Self::Circle,
    ];

    /// Prüft ob dieser Namespace menschliche Entitäten repräsentieren kann
    pub fn is_human_capable(&self) -> bool {
        matches!(self, Self::Self_ | Self::Guild | Self::Circle)
    }

    /// Prüft ob dieser Namespace KI-Agenten repräsentiert
    pub fn is_ai(&self) -> bool {
        matches!(self, Self::Spirit)
    }

    /// Von Byte-Wert
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Self::Self_),
            0x02 => Some(Self::Guild),
            0x03 => Some(Self::Spirit),
            0x04 => Some(Self::Thing),
            0x05 => Some(Self::Vessel),
            0x06 => Some(Self::Source),
            0x07 => Some(Self::Craft),
            0x08 => Some(Self::Vault),
            0x09 => Some(Self::Pact),
            0x0A => Some(Self::Circle),
            _ => None,
        }
    }

    /// Als Byte-Wert
    pub fn as_byte(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for DIDNamespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Self_ => write!(f, "self"),
            Self::Guild => write!(f, "guild"),
            Self::Spirit => write!(f, "spirit"),
            Self::Thing => write!(f, "thing"),
            Self::Vessel => write!(f, "vessel"),
            Self::Source => write!(f, "source"),
            Self::Craft => write!(f, "craft"),
            Self::Vault => write!(f, "vault"),
            Self::Pact => write!(f, "pact"),
            Self::Circle => write!(f, "circle"),
        }
    }
}

impl FromStr for DIDNamespace {
    type Err = IdentityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "self" => Ok(Self::Self_),
            "guild" => Ok(Self::Guild),
            "spirit" => Ok(Self::Spirit),
            "thing" => Ok(Self::Thing),
            "vessel" => Ok(Self::Vessel),
            "source" => Ok(Self::Source),
            "craft" => Ok(Self::Craft),
            "vault" => Ok(Self::Vault),
            "pact" => Ok(Self::Pact),
            "circle" => Ok(Self::Circle),
            _ => Err(IdentityError::InvalidNamespace(s.to_string())),
        }
    }
}

// ============================================================================
// DID – Dezentraler Identifikator
// ============================================================================

/// Dezentraler Identifikator (DID)
///
/// Format: `did:erynoa:<namespace>:<universal-id-hex>`
///
/// # Unterschied zu domain/did.rs
///
/// - `id` ist `UniversalId` (32 Bytes) statt `String`
/// - `created_at` ist `TemporalCoord` statt `DateTime<Utc>`
/// - Content-addressed durch BLAKE3 Hash
///
/// # Beispiel
///
/// ```rust
/// use erynoa_api::domain::unified::identity::{DID, DIDNamespace};
///
/// let did = DID::new(DIDNamespace::Self_, b"alice-public-key");
/// assert!(did.to_string().starts_with("did:erynoa:self:"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DID {
    /// Universelle ID (Content-Addressed)
    pub id: UniversalId,
    /// Namespace
    pub namespace: DIDNamespace,
    /// Öffentlicher Schlüssel (Ed25519, 32 Bytes)
    pub public_key: [u8; 32],
    /// Erstellungszeitpunkt
    pub created_at: TemporalCoord,
}

impl DID {
    /// Erstelle neue DID mit Namespace und Public Key
    pub fn new(namespace: DIDNamespace, public_key: &[u8]) -> Self {
        // Public Key auf 32 Bytes normalisieren
        let mut pk = [0u8; 32];
        let len = public_key.len().min(32);
        pk[..len].copy_from_slice(&public_key[..len]);

        // UniversalId aus Namespace + Public Key generieren
        let mut content = vec![namespace.as_byte()];
        content.extend_from_slice(&pk);

        let id = UniversalId::new(UniversalId::TAG_DID, 1, &content);

        Self {
            id,
            namespace,
            public_key: pk,
            created_at: TemporalCoord::now(0, &id),
        }
    }

    /// Erstelle DID mit vorgegebener TemporalCoord
    pub fn with_coord(namespace: DIDNamespace, public_key: &[u8], coord: TemporalCoord) -> Self {
        let mut pk = [0u8; 32];
        let len = public_key.len().min(32);
        pk[..len].copy_from_slice(&public_key[..len]);

        let mut content = vec![namespace.as_byte()];
        content.extend_from_slice(&pk);

        let id = UniversalId::new(UniversalId::TAG_DID, 1, &content);

        Self {
            id,
            namespace,
            public_key: pk,
            created_at: coord,
        }
    }

    /// Kurzform für `did:erynoa:self:<id>`
    pub fn new_self(public_key: &[u8]) -> Self {
        Self::new(DIDNamespace::Self_, public_key)
    }

    /// Kurzform für `did:erynoa:guild:<id>`
    pub fn new_guild(public_key: &[u8]) -> Self {
        Self::new(DIDNamespace::Guild, public_key)
    }

    /// Kurzform für `did:erynoa:spirit:<id>`
    pub fn new_spirit(public_key: &[u8]) -> Self {
        Self::new(DIDNamespace::Spirit, public_key)
    }

    /// Prüft ob diese DID eine menschliche Entität repräsentieren kann
    pub fn is_human_capable(&self) -> bool {
        self.namespace.is_human_capable()
    }

    /// Prüft ob diese DID ein KI-Agent ist
    pub fn is_ai(&self) -> bool {
        self.namespace.is_ai()
    }

    /// Public Key als Hex-String
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key)
    }

    /// Als URI-String (did:erynoa:namespace:id)
    pub fn to_uri(&self) -> String {
        format!("did:erynoa:{}:{}", self.namespace, self.id.to_hex())
    }
}

impl PartialEq for DID {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for DID {}

impl std::hash::Hash for DID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for DID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_uri())
    }
}

impl FromStr for DID {
    type Err = IdentityError;

    /// Parse DID aus URI-String
    ///
    /// Format: `did:erynoa:<namespace>:<hex-id>`
    ///
    /// # Beispiele
    ///
    /// ```rust
    /// use std::str::FromStr;
    /// use erynoa_api::domain::unified::identity::DID;
    ///
    /// let did = DID::from_str("did:erynoa:self:abc123").unwrap();
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DID::parse(s)
    }
}

impl DID {
    /// Parse DID aus String
    ///
    /// Unterstützte Formate:
    /// - `did:erynoa:<namespace>:<hex-id>`
    /// - `did:erynoa:<namespace>:<hex-id>#key-<n>`
    /// - Kurzform: `<namespace>:<hex-id>`
    pub fn parse(s: &str) -> Result<Self, IdentityError> {
        let s = s.trim();

        // Entferne eventuelle Key-Referenz (#key-1)
        let s = s.split('#').next().unwrap_or(s);

        // Parse verschiedene Formate
        let (namespace_str, id_hex) = if s.starts_with("did:erynoa:") {
            // Vollständiges Format: did:erynoa:namespace:id
            let parts: Vec<&str> = s.strip_prefix("did:erynoa:").unwrap().split(':').collect();
            if parts.len() < 2 {
                return Err(IdentityError::InvalidDIDFormat(s.to_string()));
            }
            (parts[0], parts[1])
        } else if s.contains(':') {
            // Kurzformat: namespace:id
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() < 2 {
                return Err(IdentityError::InvalidDIDFormat(s.to_string()));
            }
            (parts[0], parts[1])
        } else {
            // Nur ID, nehme "self" als Default
            ("self", s)
        };

        // Parse Namespace
        let namespace = DIDNamespace::from_str(namespace_str)?;

        // Parse ID als Hex
        let id_bytes = hex::decode(id_hex)
            .map_err(|_| IdentityError::InvalidDIDFormat(format!("invalid hex: {}", id_hex)))?;

        // Erstelle DID (public_key aus ID extrahieren oder 0-initialisieren)
        let mut public_key = [0u8; 32];
        if id_bytes.len() >= 32 {
            public_key.copy_from_slice(&id_bytes[..32]);
        } else {
            public_key[..id_bytes.len()].copy_from_slice(&id_bytes);
        }

        Ok(Self::new(namespace, &public_key))
    }

    /// Generiere zufällige DID (für Tests)
    pub fn generate() -> Self {
        let mut rng_bytes = [0u8; 32];
        // Einfache Pseudo-Random basierend auf Zeit
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let seed = now.as_nanos() as u64;
        for (i, byte) in rng_bytes.iter_mut().enumerate() {
            *byte = ((seed >> (i % 8 * 8)) ^ (i as u64 * 0x9E3779B97F4A7C15)) as u8;
        }
        Self::new(DIDNamespace::Self_, &rng_bytes)
    }
}

// ============================================================================
// DIDDocument
// ============================================================================

/// DID Document mit Verifikationsmethoden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    /// Die DID selbst
    pub id: DID,
    /// Verifikationsmethoden
    pub verification_methods: Vec<VerificationMethod>,
    /// Authentifizierungs-Methoden (Referenzen)
    pub authentication: Vec<UniversalId>,
    /// Assertion-Methoden (Referenzen)
    pub assertion_method: Vec<UniversalId>,
    /// Delegationen von dieser DID
    pub delegations: Vec<Delegation>,
    /// Letztes Update
    pub updated_at: TemporalCoord,
}

impl DIDDocument {
    /// Erstelle neues DID Document
    pub fn new(did: DID) -> Self {
        let coord = did.created_at;
        let primary_vm = VerificationMethod {
            id: did.id,
            controller: did.id,
            method_type: VerificationMethodType::Ed25519,
            public_key: did.public_key,
        };

        Self {
            id: did,
            verification_methods: vec![primary_vm.clone()],
            authentication: vec![primary_vm.id],
            assertion_method: vec![primary_vm.id],
            delegations: Vec::new(),
            updated_at: coord,
        }
    }

    /// Füge Verifikationsmethode hinzu
    pub fn add_verification_method(&mut self, method: VerificationMethod) {
        self.verification_methods.push(method);
    }

    /// Füge Delegation hinzu (Κ8)
    pub fn add_delegation(&mut self, delegation: Delegation) {
        self.delegations.push(delegation);
    }

    /// Finde Verifikationsmethode nach ID
    pub fn find_verification_method(&self, id: &UniversalId) -> Option<&VerificationMethod> {
        self.verification_methods.iter().find(|vm| &vm.id == id)
    }
}

// ============================================================================
// VerificationMethod
// ============================================================================

/// Typ der Verifikationsmethode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationMethodType {
    /// Ed25519 (Standard)
    Ed25519,
    /// Secp256k1 (Bitcoin/Ethereum kompatibel)
    Secp256k1,
    /// X25519 (Key Agreement)
    X25519,
}

/// Verifikationsmethode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    /// ID der Methode
    pub id: UniversalId,
    /// Controller (wer kontrolliert diesen Key)
    pub controller: UniversalId,
    /// Typ
    pub method_type: VerificationMethodType,
    /// Öffentlicher Schlüssel
    pub public_key: [u8; 32],
}

// ============================================================================
// Delegation (Κ8)
// ============================================================================

/// Delegation einer Berechtigung (Κ8: Trust-Decay)
///
/// # Κ8: Delegations-Struktur
///
/// ```text
/// s ⊳ s' → 𝕋(s') ≤ trust_factor × 𝕋(s)
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// Eindeutige ID der Delegation
    pub id: UniversalId,
    /// Delegierender (Quelle)
    pub delegator: UniversalId,
    /// Delegierter (Ziel)
    pub delegate: UniversalId,
    /// Trust-Faktor ∈ (0, 1] (Κ8)
    pub trust_factor: f32,
    /// Delegierte Fähigkeiten
    pub capabilities: Vec<Capability>,
    /// Gültig bis (optional)
    pub valid_until: Option<TemporalCoord>,
    /// Erstellt am
    pub created_at: TemporalCoord,
    /// Ist widerrufen?
    pub revoked: bool,
}

impl Delegation {
    /// Erstelle neue Delegation
    ///
    /// # Panics
    ///
    /// Panics wenn `trust_factor` nicht in (0, 1] liegt.
    pub fn new(
        delegator: UniversalId,
        delegate: UniversalId,
        trust_factor: f32,
        capabilities: Vec<Capability>,
    ) -> Self {
        assert!(
            trust_factor > 0.0 && trust_factor <= 1.0,
            "Trust factor must be in (0, 1]"
        );

        let content = [
            delegator.as_bytes().as_slice(),
            delegate.as_bytes().as_slice(),
            &trust_factor.to_le_bytes(),
        ]
        .concat();

        let id = UniversalId::new(UniversalId::TAG_DID, 1, &content);
        let coord = TemporalCoord::now(0, &id);

        Self {
            id,
            delegator,
            delegate,
            trust_factor,
            capabilities,
            valid_until: None,
            created_at: coord,
            revoked: false,
        }
    }

    /// Mit Gültigkeitsdauer
    pub fn with_validity(mut self, until: TemporalCoord) -> Self {
        self.valid_until = Some(until);
        self
    }

    /// Ist die Delegation gültig?
    pub fn is_valid(&self, now: &TemporalCoord) -> bool {
        if self.revoked {
            return false;
        }

        if let Some(until) = &self.valid_until {
            return now < until;
        }

        true
    }

    /// Widerrufe die Delegation
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}

// ============================================================================
// Capability
// ============================================================================

/// Fähigkeit die delegiert werden kann
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Alle Fähigkeiten (gefährlich!)
    All,
    /// Lesen von Daten
    Read { resource: String },
    /// Schreiben von Daten
    Write { resource: String },
    /// Ausführen von Aktionen
    Execute { action: String },
    /// Delegieren (Ketten-Delegation)
    Delegate { max_depth: u8 },
    /// Attestieren
    Attest { claim_types: Vec<String> },
    /// Custom Capability
    Custom { name: String, params: String },
}

// ============================================================================
// Errors
// ============================================================================

/// Fehler bei Identity-Operationen
#[derive(Debug, Clone, thiserror::Error)]
pub enum IdentityError {
    #[error("Invalid namespace: {0}")]
    InvalidNamespace(String),

    #[error("Invalid DID format: {0}")]
    InvalidFormat(String),

    #[error("Invalid DID format: {0}")]
    InvalidDIDFormat(String),

    #[error("Invalid trust factor: {0} (must be in (0, 1])")]
    InvalidTrustFactor(f32),

    #[error("Delegation expired")]
    DelegationExpired,

    #[error("Delegation revoked")]
    DelegationRevoked,

    #[error("Capability not granted: {0}")]
    CapabilityNotGranted(String),
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_did_creation() {
        let pk = b"test-public-key-32-bytes-long!!";
        let did = DID::new_self(pk);

        assert_eq!(did.namespace, DIDNamespace::Self_);
        assert!(did.is_human_capable());
        assert!(!did.is_ai());
        assert!(did.to_uri().starts_with("did:erynoa:self:"));
    }

    #[test]
    fn test_did_equality() {
        let pk = b"same-public-key-32-bytes-long!!";
        let did1 = DID::new_self(pk);
        let did2 = DID::new_self(pk);

        // Gleicher Public Key → Gleiche ID
        assert_eq!(did1.id, did2.id);
        assert_eq!(did1, did2);
    }

    #[test]
    fn test_namespace_parsing() {
        assert_eq!("self".parse::<DIDNamespace>().unwrap(), DIDNamespace::Self_);
        assert_eq!(
            "guild".parse::<DIDNamespace>().unwrap(),
            DIDNamespace::Guild
        );
        assert!("invalid".parse::<DIDNamespace>().is_err());
    }

    #[test]
    fn test_delegation() {
        let delegator = UniversalId::new(UniversalId::TAG_DID, 1, b"delegator");
        let delegate = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");

        let delegation = Delegation::new(
            delegator,
            delegate,
            0.8,
            vec![Capability::Read {
                resource: "*".into(),
            }],
        );

        assert_eq!(delegation.trust_factor, 0.8);
        assert!(!delegation.revoked);
    }

    #[test]
    #[should_panic(expected = "Trust factor must be in (0, 1]")]
    fn test_invalid_trust_factor() {
        let delegator = UniversalId::new(UniversalId::TAG_DID, 1, b"delegator");
        let delegate = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");

        // Trust factor > 1 sollte fehlschlagen
        Delegation::new(delegator, delegate, 1.5, vec![]);
    }

    #[test]
    fn test_delegation_validity() {
        let delegator = UniversalId::new(UniversalId::TAG_DID, 1, b"delegator");
        let delegate = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");

        let mut delegation = Delegation::new(delegator, delegate, 0.8, vec![]);
        let now = TemporalCoord::now(1, &delegation.id);

        assert!(delegation.is_valid(&now));

        delegation.revoke();
        assert!(!delegation.is_valid(&now));
    }

    #[test]
    fn test_did_document() {
        let did = DID::new_self(b"test-key");
        let doc = DIDDocument::new(did.clone());

        assert_eq!(doc.id, did);
        assert_eq!(doc.verification_methods.len(), 1);
        assert_eq!(doc.authentication.len(), 1);
    }

    #[test]
    fn test_namespace_byte_roundtrip() {
        for ns in DIDNamespace::ALL {
            let byte = ns.as_byte();
            let parsed = DIDNamespace::from_byte(byte).unwrap();
            assert_eq!(ns, parsed);
        }
    }
}
