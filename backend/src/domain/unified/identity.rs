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

/// Extension Slot IDs (UDM §2.1)
///
/// Zukunftssichere Erweiterungen für DIDDocument
pub mod extension_slots {
    /// Recovery-Keys für Key-Rotation
    pub const RECOVERY_KEYS: u16 = 0x0001;
    /// Biometrische Bindung
    pub const BIOMETRIC_BINDING: u16 = 0x0002;
    /// Hardware-Attestation (TEE, TPM)
    pub const HARDWARE_ATTESTATION: u16 = 0x0003;
    /// Cross-Chain-Links
    pub const CROSS_CHAIN_LINKS: u16 = 0x0004;
    /// AI-Agent-Manifest
    pub const AI_AGENT_MANIFEST: u16 = 0x0005;
    // Custom Extensions: 0x0006..0xFFFF
}

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
    /// Zukunftssichere Extension Slots (UDM §2.1)
    #[serde(default)]
    pub extension_slots: std::collections::BTreeMap<u16, Vec<u8>>,
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
            extension_slots: std::collections::BTreeMap::new(),
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

    /// Setze Extension Slot
    pub fn set_extension(&mut self, slot_id: u16, data: Vec<u8>) {
        self.extension_slots.insert(slot_id, data);
    }

    /// Hole Extension Slot
    pub fn get_extension(&self, slot_id: u16) -> Option<&Vec<u8>> {
        self.extension_slots.get(&slot_id)
    }

    /// Prüfe ob Extension existiert
    pub fn has_extension(&self, slot_id: u16) -> bool {
        self.extension_slots.contains_key(&slot_id)
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
// DID Derivation Methods (Phase 1: Foundations)
// ============================================================================

impl DID {
    /// Erstelle Device-Sub-DID von einer Root-DID
    ///
    /// # Arguments
    ///
    /// - `root` - Die Root-DID von der abgeleitet wird
    /// - `device_index` - Index des Geräts (0-basiert)
    ///
    /// # Returns
    ///
    /// Neue DID mit Namespace `Self_` und deterministisch abgeleiteter ID
    ///
    /// # Example
    ///
    /// ```rust
    /// use erynoa_api::domain::unified::identity::DID;
    ///
    /// let root = DID::new_self(b"root-key");
    /// let device = DID::derive_device(&root, 0);
    /// assert!(device.to_uri().starts_with("did:erynoa:self:"));
    /// ```
    pub fn derive_device(root: &DID, device_index: u32) -> Self {
        let derivation_content = [
            root.public_key.as_slice(),
            b"device",
            &device_index.to_be_bytes(),
        ]
        .concat();

        let derived_key = blake3::hash(&derivation_content);
        let mut pk = [0u8; 32];
        pk.copy_from_slice(derived_key.as_bytes());

        Self::new(DIDNamespace::Self_, &pk)
    }

    /// Erstelle Agent-Sub-DID von einer Root-DID
    ///
    /// Agent-DIDs nutzen den `Spirit` Namespace für KI-Agenten/autonome Systeme.
    ///
    /// # Arguments
    ///
    /// - `root` - Die Root-DID von der abgeleitet wird
    /// - `agent_index` - Index des Agents (0-basiert)
    ///
    /// # Returns
    ///
    /// Neue DID mit Namespace `Spirit`
    pub fn derive_agent(root: &DID, agent_index: u32) -> Self {
        let derivation_content = [
            root.public_key.as_slice(),
            b"agent",
            &agent_index.to_be_bytes(),
        ]
        .concat();

        let derived_key = blake3::hash(&derivation_content);
        let mut pk = [0u8; 32];
        pk.copy_from_slice(derived_key.as_bytes());

        Self::new(DIDNamespace::Spirit, &pk)
    }

    /// Erstelle Realm-spezifische Sub-DID
    ///
    /// Realm-DIDs nutzen den `Circle` Namespace für isolierte Realm-Identitäten.
    ///
    /// # Arguments
    ///
    /// - `root` - Die Root-DID von der abgeleitet wird
    /// - `realm_id` - UniversalId des Realms
    ///
    /// # Returns
    ///
    /// Neue DID mit Namespace `Circle`, spezifisch für dieses Realm
    pub fn derive_realm(root: &DID, realm_id: &UniversalId) -> Self {
        let derivation_content = [
            root.public_key.as_slice(),
            b"realm",
            realm_id.as_bytes().as_slice(),
        ]
        .concat();

        let derived_key = blake3::hash(&derivation_content);
        let mut pk = [0u8; 32];
        pk.copy_from_slice(derived_key.as_bytes());

        Self::new(DIDNamespace::Circle, &pk)
    }

    /// Erstelle Custom-Sub-DID mit beliebigem Namespace und Kontext
    ///
    /// # Arguments
    ///
    /// - `root` - Die Root-DID von der abgeleitet wird
    /// - `namespace` - Ziel-Namespace für die neue DID
    /// - `context` - Kontext-String für die Derivation (z.B. "vault-backup")
    /// - `index` - Index für mehrere DIDs mit gleichem Kontext
    ///
    /// # Returns
    ///
    /// Neue DID mit dem angegebenen Namespace
    pub fn derive_custom(root: &DID, namespace: DIDNamespace, context: &str, index: u32) -> Self {
        let derivation_content = [
            root.public_key.as_slice(),
            context.as_bytes(),
            &index.to_be_bytes(),
        ]
        .concat();

        let derived_key = blake3::hash(&derivation_content);
        let mut pk = [0u8; 32];
        pk.copy_from_slice(derived_key.as_bytes());

        Self::new(namespace, &pk)
    }

    /// Prüfe ob diese DID zu einem bestimmten Namespace gehört
    pub fn is_namespace(&self, ns: DIDNamespace) -> bool {
        self.namespace == ns
    }

    /// Prüfe ob diese DID eine Device-DID sein könnte
    ///
    /// Hinweis: Dies prüft nur den Namespace, nicht die tatsächliche Herkunft
    pub fn is_device_capable(&self) -> bool {
        matches!(self.namespace, DIDNamespace::Self_)
    }

    /// Prüfe ob diese DID eine Agent-DID ist
    pub fn is_agent(&self) -> bool {
        matches!(self.namespace, DIDNamespace::Spirit)
    }

    /// Prüfe ob diese DID eine Realm-DID ist
    pub fn is_realm_did(&self) -> bool {
        matches!(self.namespace, DIDNamespace::Circle)
    }

    /// Berechne Derivation-Pfad-String (BIP44-ähnlich)
    ///
    /// # Arguments
    ///
    /// - `purpose` - Zweck der Derivation ("device", "agent", "realm", etc.)
    /// - `index` - Index innerhalb des Zwecks
    ///
    /// # Returns
    ///
    /// Pfad-String wie "m/44'/erynoa'/0'/device/0"
    pub fn derivation_path(purpose: &str, index: u32) -> String {
        format!("m/44'/erynoa'/0'/{}/{}", purpose, index)
    }
}

// ============================================================================
// DIDDocument Extensions (Phase 1: Foundations)
// ============================================================================

impl DIDDocument {
    /// Füge Device-Key als Verifikationsmethode hinzu
    ///
    /// # Arguments
    ///
    /// - `device_did` - Die Device-Sub-DID
    pub fn add_device_key(&mut self, device_did: &DID) {
        let vm = VerificationMethod {
            id: device_did.id,
            controller: self.id.id, // Root-DID ist Controller
            method_type: VerificationMethodType::Ed25519,
            public_key: device_did.public_key,
        };

        // Prüfe ob bereits vorhanden
        if !self.verification_methods.iter().any(|v| v.id == vm.id) {
            self.verification_methods.push(vm.clone());
            self.authentication.push(vm.id);
        }

        self.updated_at = TemporalCoord::now(0, &device_did.id);
    }

    /// Füge Agent-Key als Verifikationsmethode hinzu (mit capabilityDelegation)
    ///
    /// # Arguments
    ///
    /// - `agent_did` - Die Agent-Sub-DID
    pub fn add_agent_key(&mut self, agent_did: &DID) {
        let vm = VerificationMethod {
            id: agent_did.id,
            controller: self.id.id,
            method_type: VerificationMethodType::Ed25519,
            public_key: agent_did.public_key,
        };

        if !self.verification_methods.iter().any(|v| v.id == vm.id) {
            self.verification_methods.push(vm.clone());
            // Agents bekommen nur assertion_method, nicht authentication
            self.assertion_method.push(vm.id);
        }

        self.updated_at = TemporalCoord::now(0, &agent_did.id);
    }

    /// Finde Delegation für einen bestimmten Delegate
    ///
    /// # Arguments
    ///
    /// - `delegate` - UniversalId des Delegates
    ///
    /// # Returns
    ///
    /// Referenz auf die Delegation falls gefunden
    pub fn find_delegation_for(&self, delegate: &UniversalId) -> Option<&Delegation> {
        self.delegations.iter().find(|d| d.delegate == *delegate)
    }

    /// Finde alle aktiven Delegationen
    ///
    /// # Arguments
    ///
    /// - `now` - Aktueller Zeitpunkt für Gültigkeitsprüfung
    ///
    /// # Returns
    ///
    /// Liste der aktiven Delegationen
    pub fn active_delegations(&self, now: &TemporalCoord) -> Vec<&Delegation> {
        self.delegations.iter().filter(|d| d.is_valid(now)).collect()
    }

    /// Prüfe ob eine Verifikationsmethode existiert
    pub fn has_verification_method(&self, method_id: &UniversalId) -> bool {
        self.verification_methods.iter().any(|vm| &vm.id == method_id)
    }

    /// Hole alle Verifikationsmethoden eines bestimmten Typs
    pub fn verification_methods_by_type(
        &self,
        method_type: VerificationMethodType,
    ) -> Vec<&VerificationMethod> {
        self.verification_methods
            .iter()
            .filter(|vm| vm.method_type == method_type)
            .collect()
    }

    /// Anzahl der Verifikationsmethoden
    pub fn verification_method_count(&self) -> usize {
        self.verification_methods.len()
    }

    /// Anzahl der aktiven Delegationen
    pub fn delegation_count(&self) -> usize {
        self.delegations.len()
    }

    /// Widerrufe eine Delegation
    ///
    /// # Arguments
    ///
    /// - `delegation_id` - UniversalId der Delegation
    ///
    /// # Returns
    ///
    /// `true` wenn die Delegation gefunden und widerrufen wurde
    pub fn revoke_delegation(&mut self, delegation_id: &UniversalId) -> bool {
        if let Some(delegation) = self.delegations.iter_mut().find(|d| &d.id == delegation_id) {
            delegation.revoke();
            self.updated_at = TemporalCoord::now(0, delegation_id);
            true
        } else {
            false
        }
    }
}

// ============================================================================
// Capability Extensions (Phase 1: Foundations)
// ============================================================================

impl Capability {
    /// Parse Capability aus String
    ///
    /// Unterstützte Formate:
    /// - `"*"` → All
    /// - `"read:resource"` → Read { resource }
    /// - `"write:resource"` → Write { resource }
    /// - `"execute:action"` → Execute { action }
    /// - `"delegate:N"` → Delegate { max_depth: N }
    /// - `"attest:type1,type2"` → Attest { claim_types }
    /// - `"custom:name:params"` → Custom { name, params }
    pub fn parse(s: &str) -> Result<Self, IdentityError> {
        let s = s.trim();

        if s == "*" {
            return Ok(Self::All);
        }

        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() < 2 {
            return Err(IdentityError::InvalidFormat(format!(
                "Invalid capability format: {}",
                s
            )));
        }

        let (prefix, value) = (parts[0], parts[1]);

        match prefix {
            "read" => Ok(Self::Read {
                resource: value.to_string(),
            }),
            "write" => Ok(Self::Write {
                resource: value.to_string(),
            }),
            "execute" => Ok(Self::Execute {
                action: value.to_string(),
            }),
            "delegate" => {
                let depth = value.parse::<u8>().map_err(|_| {
                    IdentityError::InvalidFormat(format!("Invalid delegate depth: {}", value))
                })?;
                Ok(Self::Delegate { max_depth: depth })
            }
            "attest" => {
                let types: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();
                Ok(Self::Attest { claim_types: types })
            }
            "custom" => {
                let custom_parts: Vec<&str> = value.splitn(2, ':').collect();
                if custom_parts.len() < 2 {
                    Ok(Self::Custom {
                        name: value.to_string(),
                        params: String::new(),
                    })
                } else {
                    Ok(Self::Custom {
                        name: custom_parts[0].to_string(),
                        params: custom_parts[1].to_string(),
                    })
                }
            }
            _ => Err(IdentityError::InvalidFormat(format!(
                "Unknown capability type: {}",
                prefix
            ))),
        }
    }

    /// Konvertiere Capability zu String
    pub fn to_string_repr(&self) -> String {
        match self {
            Self::All => "*".to_string(),
            Self::Read { resource } => format!("read:{}", resource),
            Self::Write { resource } => format!("write:{}", resource),
            Self::Execute { action } => format!("execute:{}", action),
            Self::Delegate { max_depth } => format!("delegate:{}", max_depth),
            Self::Attest { claim_types } => format!("attest:{}", claim_types.join(",")),
            Self::Custom { name, params } => {
                if params.is_empty() {
                    format!("custom:{}", name)
                } else {
                    format!("custom:{}:{}", name, params)
                }
            }
        }
    }

    /// Prüfe ob diese Capability eine andere impliziert
    ///
    /// Z.B. `All` impliziert alle anderen, `write:*` impliziert `write:specific`
    pub fn implies(&self, other: &Self) -> bool {
        match (self, other) {
            // All impliziert alles
            (Self::All, _) => true,

            // Gleiche Typen mit Wildcard
            (Self::Read { resource: a }, Self::Read { resource: b }) => {
                a == "*" || a == b || (a.ends_with('*') && b.starts_with(&a[..a.len() - 1]))
            }
            (Self::Write { resource: a }, Self::Write { resource: b }) => {
                a == "*" || a == b || (a.ends_with('*') && b.starts_with(&a[..a.len() - 1]))
            }
            (Self::Execute { action: a }, Self::Execute { action: b }) => {
                a == "*" || a == b || (a.ends_with('*') && b.starts_with(&a[..a.len() - 1]))
            }

            // Delegate: höhere max_depth impliziert niedrigere
            (Self::Delegate { max_depth: a }, Self::Delegate { max_depth: b }) => a >= b,

            // Attest: alle claim_types müssen enthalten sein
            (Self::Attest { claim_types: a }, Self::Attest { claim_types: b }) => {
                b.iter().all(|t| a.contains(t))
            }

            // Custom: exakter Match
            (
                Self::Custom {
                    name: n1,
                    params: p1,
                },
                Self::Custom {
                    name: n2,
                    params: p2,
                },
            ) => n1 == n2 && p1 == p2,

            _ => false,
        }
    }

    /// Ist dies eine gefährliche Capability?
    pub fn is_dangerous(&self) -> bool {
        match self {
            Self::All => true,
            Self::Write { resource } if resource == "*" => true,
            Self::Delegate { max_depth } if *max_depth > 3 => true,
            _ => false,
        }
    }
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

    #[error("Derivation failed: {0}")]
    DerivationFailed(String),

    #[error("Key not found: {0:?}")]
    KeyNotFound(UniversalId),
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

    // ========================================================================
    // Phase 1: Foundations - New Tests
    // ========================================================================

    #[test]
    fn test_did_derive_device() {
        let root = DID::new_self(b"root-key-32-bytes-for-testing!!");
        let device0 = DID::derive_device(&root, 0);
        let device1 = DID::derive_device(&root, 1);

        // Device-DIDs sollten Self-Namespace haben
        assert!(device0.is_namespace(DIDNamespace::Self_));
        assert!(device1.is_namespace(DIDNamespace::Self_));

        // Verschiedene Indices → verschiedene DIDs
        assert_ne!(device0.id, device1.id);

        // Deterministisch: gleicher Index → gleiche DID
        let device0_again = DID::derive_device(&root, 0);
        assert_eq!(device0.id, device0_again.id);
    }

    #[test]
    fn test_did_derive_agent() {
        let root = DID::new_self(b"root-key-32-bytes-for-testing!!");
        let agent = DID::derive_agent(&root, 0);

        // Agent-DIDs sollten Spirit-Namespace haben
        assert!(agent.is_namespace(DIDNamespace::Spirit));
        assert!(agent.is_agent());
        assert!(agent.is_ai());
    }

    #[test]
    fn test_did_derive_realm() {
        let root = DID::new_self(b"root-key-32-bytes-for-testing!!");
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");

        let realm_did = DID::derive_realm(&root, &realm_id);

        // Realm-DIDs sollten Circle-Namespace haben
        assert!(realm_did.is_namespace(DIDNamespace::Circle));
        assert!(realm_did.is_realm_did());
    }

    #[test]
    fn test_did_derive_custom() {
        let root = DID::new_self(b"root-key-32-bytes-for-testing!!");

        let vault_did = DID::derive_custom(&root, DIDNamespace::Vault, "backup", 0);
        assert!(vault_did.is_namespace(DIDNamespace::Vault));

        let pact_did = DID::derive_custom(&root, DIDNamespace::Pact, "contract", 0);
        assert!(pact_did.is_namespace(DIDNamespace::Pact));
    }

    #[test]
    fn test_did_derivation_path() {
        assert_eq!(
            DID::derivation_path("device", 0),
            "m/44'/erynoa'/0'/device/0"
        );
        assert_eq!(
            DID::derivation_path("agent", 5),
            "m/44'/erynoa'/0'/agent/5"
        );
    }

    #[test]
    fn test_did_document_add_device_key() {
        let root = DID::new_self(b"root-key");
        let mut doc = DIDDocument::new(root.clone());

        let device = DID::derive_device(&root, 0);
        doc.add_device_key(&device);

        assert_eq!(doc.verification_method_count(), 2); // Root + Device
        assert!(doc.has_verification_method(&device.id));
        assert!(doc.authentication.contains(&device.id));
    }

    #[test]
    fn test_did_document_add_agent_key() {
        let root = DID::new_self(b"root-key");
        let mut doc = DIDDocument::new(root.clone());

        let agent = DID::derive_agent(&root, 0);
        doc.add_agent_key(&agent);

        assert_eq!(doc.verification_method_count(), 2);
        assert!(doc.has_verification_method(&agent.id));
        // Agent in assertion_method, nicht authentication
        assert!(!doc.authentication.contains(&agent.id));
        assert!(doc.assertion_method.contains(&agent.id));
    }

    #[test]
    fn test_did_document_find_delegation() {
        let root = DID::new_self(b"root-key");
        let mut doc = DIDDocument::new(root);

        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        let delegation = Delegation::new(
            doc.id.id,
            delegate_id,
            0.8,
            vec![Capability::Read {
                resource: "*".into(),
            }],
        );

        doc.add_delegation(delegation);

        assert!(doc.find_delegation_for(&delegate_id).is_some());
        assert_eq!(doc.delegation_count(), 1);
    }

    #[test]
    fn test_did_document_revoke_delegation() {
        let root = DID::new_self(b"root-key");
        let mut doc = DIDDocument::new(root);

        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        let delegation = Delegation::new(doc.id.id, delegate_id, 0.8, vec![]);
        let delegation_id = delegation.id;

        doc.add_delegation(delegation);

        assert!(doc.revoke_delegation(&delegation_id));

        let now = TemporalCoord::now(0, &delegation_id);
        assert!(doc.active_delegations(&now).is_empty());
    }

    #[test]
    fn test_capability_parse() {
        // All
        assert!(matches!(Capability::parse("*").unwrap(), Capability::All));

        // Read
        assert!(matches!(
            Capability::parse("read:documents").unwrap(),
            Capability::Read { resource } if resource == "documents"
        ));

        // Write
        assert!(matches!(
            Capability::parse("write:*").unwrap(),
            Capability::Write { resource } if resource == "*"
        ));

        // Execute
        assert!(matches!(
            Capability::parse("execute:transfer").unwrap(),
            Capability::Execute { action } if action == "transfer"
        ));

        // Delegate
        assert!(matches!(
            Capability::parse("delegate:3").unwrap(),
            Capability::Delegate { max_depth: 3 }
        ));

        // Attest
        let attest = Capability::parse("attest:kyc,age").unwrap();
        assert!(matches!(
            attest,
            Capability::Attest { claim_types } if claim_types == vec!["kyc", "age"]
        ));

        // Custom
        assert!(matches!(
            Capability::parse("custom:myaction:param1").unwrap(),
            Capability::Custom { name, params } if name == "myaction" && params == "param1"
        ));
    }

    #[test]
    fn test_capability_implies() {
        // All implies everything
        assert!(Capability::All.implies(&Capability::Read {
            resource: "any".into()
        }));

        // Wildcard implies specific
        assert!(Capability::Read {
            resource: "*".into()
        }
        .implies(&Capability::Read {
            resource: "specific".into()
        }));

        // Prefix wildcard
        assert!(Capability::Read {
            resource: "docs/*".into()
        }
        .implies(&Capability::Read {
            resource: "docs/file.txt".into()
        }));

        // Delegate depth
        assert!(Capability::Delegate { max_depth: 5 }.implies(&Capability::Delegate { max_depth: 3 }));
        assert!(!Capability::Delegate { max_depth: 2 }
            .implies(&Capability::Delegate { max_depth: 5 }));
    }

    #[test]
    fn test_capability_to_string_roundtrip() {
        let caps = vec![
            Capability::All,
            Capability::Read {
                resource: "docs".into(),
            },
            Capability::Write {
                resource: "*".into(),
            },
            Capability::Execute {
                action: "transfer".into(),
            },
            Capability::Delegate { max_depth: 3 },
            Capability::Attest {
                claim_types: vec!["kyc".into(), "age".into()],
            },
            Capability::Custom {
                name: "custom".into(),
                params: "params".into(),
            },
        ];

        for cap in caps {
            let s = cap.to_string_repr();
            let parsed = Capability::parse(&s).unwrap();
            assert_eq!(cap.to_string_repr(), parsed.to_string_repr());
        }
    }

    #[test]
    fn test_capability_is_dangerous() {
        assert!(Capability::All.is_dangerous());
        assert!(Capability::Write {
            resource: "*".into()
        }
        .is_dangerous());
        assert!(Capability::Delegate { max_depth: 5 }.is_dangerous());

        assert!(!Capability::Read {
            resource: "docs".into()
        }
        .is_dangerous());
        assert!(!Capability::Delegate { max_depth: 2 }.is_dangerous());
    }
}
