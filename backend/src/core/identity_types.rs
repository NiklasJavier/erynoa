//! # Identity Types for State Management
//!
//! Typen und Traits für das Identity-Layer in state.rs.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                     IDENTITY TYPES                                  │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  IdentityMode       - Betriebsmodi (Interactive, Agent, etc.)      │
//! │  WalletAddress      - Chain-spezifische Wallet-Adressen            │
//! │  RealmMembership    - Realm-Mitgliedschafts-Info                   │
//! │  RealmRole          - Rollen innerhalb eines Realms                │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  SecureKeyStore     - Trait für Hardware-gebundene Key-Ops         │
//! │  PasskeyManager     - Trait für WebAuthn/Passkey-Integration       │
//! │  IdentityResolver   - Trait für Cross-Shard Identity-Resolution    │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  IdentityError      - Fehlertypen für Identity-Operationen         │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Axiom-Referenz
//!
//! - **Κ6 (Existenz-Eindeutigkeit)**: Jede Entity hat exakt eine Root-DID
//! - **Κ7 (Permanenz)**: Einmal erstellte DIDs sind unveränderlich
//! - **Κ8 (Delegations-Struktur)**: Trust-Decay bei Delegation

use crate::domain::unified::primitives::{TemporalCoord, UniversalId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ============================================================================
// IDENTITY MODE
// ============================================================================

/// Betriebsmodus der Identity
///
/// Bestimmt wie Signaturen erstellt werden und welche Operationen erlaubt sind.
///
/// # Modi
///
/// - **Interactive**: Erfordert User-Confirmation für Root-Signaturen (WebAuthn)
/// - **AgentManaged**: Autonome Signaturen erlaubt (Software-Keys)
/// - **Ephemeral**: Kurzlebige Session-Identity ohne Persistenz
/// - **Test**: Deterministischer Modus für Unit-Tests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum IdentityMode {
    /// Erfordert User-Confirmation (Biometrie/PIN) für Root-Signaturen
    /// Keys sind Hardware-bound (TEE/Secure Enclave)
    Interactive = 0,

    /// Autonome Signaturen erlaubt
    /// Keys sind Software-verschlüsselt auf dem Peer
    /// Trust-Penalty: Startet mit base_trust × 0.8
    AgentManaged = 1,

    /// Kurzlebige Session ohne Persistenz
    /// Keine Realm-Memberships, keine Trust-Akkumulation
    /// Max TTL: 24h
    Ephemeral = 2,

    /// Deterministischer Modus für Unit-Tests
    /// Fake-Keys, keine echte Kryptographie
    Test = 3,
}

impl IdentityMode {
    /// Konvertiere von u8
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Interactive,
            1 => Self::AgentManaged,
            2 => Self::Ephemeral,
            _ => Self::Test,
        }
    }

    /// Ist dieser Modus für Produktion geeignet?
    pub fn is_production_safe(&self) -> bool {
        matches!(self, Self::Interactive | Self::AgentManaged)
    }

    /// Erlaubt autonome Signaturen ohne User-Confirmation?
    pub fn allows_autonomous_signing(&self) -> bool {
        matches!(self, Self::AgentManaged | Self::Ephemeral | Self::Test)
    }

    /// Erlaubt Realm-Memberships?
    pub fn allows_realm_membership(&self) -> bool {
        !matches!(self, Self::Ephemeral)
    }

    /// Erlaubt Trust-Akkumulation?
    pub fn allows_trust_accumulation(&self) -> bool {
        !matches!(self, Self::Ephemeral | Self::Test)
    }

    /// Trust-Penalty-Faktor (1.0 = keine Penalty)
    pub fn trust_penalty_factor(&self) -> f64 {
        match self {
            Self::Interactive => 1.0,
            Self::AgentManaged => 0.8,
            Self::Ephemeral => 0.5,
            Self::Test => 1.0,
        }
    }
}

impl Default for IdentityMode {
    fn default() -> Self {
        Self::Interactive
    }
}

impl std::fmt::Display for IdentityMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Interactive => write!(f, "interactive"),
            Self::AgentManaged => write!(f, "agent-managed"),
            Self::Ephemeral => write!(f, "ephemeral"),
            Self::Test => write!(f, "test"),
        }
    }
}

// ============================================================================
// WALLET ADDRESS
// ============================================================================

/// Chain-spezifische Wallet-Adresse
///
/// Abgeleitet via BIP44 von einem DID.
///
/// # CAIP-2 Format
///
/// `chain_id` folgt dem CAIP-2 Format:
/// - `eip155:1` - Ethereum Mainnet
/// - `eip155:137` - Polygon
/// - `solana:mainnet` - Solana
/// - `cosmos:cosmoshub-4` - Cosmos Hub
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WalletAddress {
    /// Chain-ID im CAIP-2 Format (z.B. "eip155:1")
    pub chain_id: String,

    /// Adresse auf der Chain (z.B. "0x...")
    pub address: String,

    /// BIP44 Derivation-Pfad (z.B. "m/44'/60'/0'/0/0")
    pub derivation_path: String,

    /// UniversalId des DIDs von dem abgeleitet wurde
    pub derived_from: UniversalId,

    /// Erstellungszeitpunkt
    pub created_at: u64,

    /// Ist diese Adresse die primäre für diese Chain?
    pub is_primary: bool,
}

impl WalletAddress {
    /// Erstelle neue Wallet-Adresse
    pub fn new(
        chain_id: impl Into<String>,
        address: impl Into<String>,
        derivation_path: impl Into<String>,
        derived_from: UniversalId,
    ) -> Self {
        Self {
            chain_id: chain_id.into(),
            address: address.into(),
            derivation_path: derivation_path.into(),
            derived_from,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            is_primary: false,
        }
    }

    /// Markiere als primäre Adresse
    pub fn as_primary(mut self) -> Self {
        self.is_primary = true;
        self
    }

    /// Extrahiere Chain-Namespace aus CAIP-2 ID
    pub fn chain_namespace(&self) -> Option<&str> {
        self.chain_id.split(':').next()
    }

    /// Extrahiere Chain-Reference aus CAIP-2 ID
    pub fn chain_reference(&self) -> Option<&str> {
        self.chain_id.split(':').nth(1)
    }

    /// Ist dies eine EVM-kompatible Chain?
    pub fn is_evm(&self) -> bool {
        self.chain_namespace() == Some("eip155")
    }

    /// Validiere Adressformat
    pub fn validate(&self) -> Result<(), IdentityError> {
        if self.chain_id.is_empty() {
            return Err(IdentityError::InvalidChainId(self.chain_id.clone()));
        }

        if self.address.is_empty() {
            return Err(IdentityError::InvalidAddress("Empty address".to_string()));
        }

        // EVM-Adresse muss mit 0x beginnen und 42 Zeichen lang sein
        if self.is_evm() {
            if !self.address.starts_with("0x") || self.address.len() != 42 {
                return Err(IdentityError::InvalidAddress(format!(
                    "Invalid EVM address: {}",
                    self.address
                )));
            }
        }

        Ok(())
    }
}

// ============================================================================
// REALM MEMBERSHIP
// ============================================================================

/// Rolle innerhalb eines Realms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RealmRole {
    /// Normales Mitglied
    Member = 0,
    /// Moderator mit erweiterten Rechten
    Moderator = 1,
    /// Administrator
    Admin = 2,
    /// Realm-Eigentümer (höchste Rechte)
    Owner = 3,
}

impl RealmRole {
    /// Von u8 konvertieren
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Member,
            1 => Self::Moderator,
            2 => Self::Admin,
            _ => Self::Owner,
        }
    }

    /// Ist diese Rolle höher oder gleich einer anderen?
    pub fn is_at_least(&self, other: Self) -> bool {
        (*self as u8) >= (other as u8)
    }

    /// Kann diese Rolle eine andere Rolle vergeben?
    pub fn can_assign(&self, role: Self) -> bool {
        match self {
            Self::Owner => true, // Owner kann alles
            Self::Admin => role != Self::Owner && role != Self::Admin,
            Self::Moderator => role == Self::Member,
            Self::Member => false,
        }
    }

    /// Trust-Multiplikator für diese Rolle im Realm
    pub fn trust_multiplier(&self) -> f64 {
        match self {
            Self::Member => 1.0,
            Self::Moderator => 1.1,
            Self::Admin => 1.2,
            Self::Owner => 1.3,
        }
    }
}

impl Default for RealmRole {
    fn default() -> Self {
        Self::Member
    }
}

impl std::fmt::Display for RealmRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Member => write!(f, "member"),
            Self::Moderator => write!(f, "moderator"),
            Self::Admin => write!(f, "admin"),
            Self::Owner => write!(f, "owner"),
        }
    }
}

/// Realm-Mitgliedschafts-Information
///
/// Speichert alle relevanten Daten für die Mitgliedschaft einer Identity in einem Realm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmMembership {
    /// Realm-ID
    pub realm_id: UniversalId,

    /// Root-DID des Mitglieds
    pub root_did: UniversalId,

    /// Realm-spezifische Sub-DID (optional, für isolierte Realms)
    pub realm_sub_did: Option<UniversalId>,

    /// Beitrittszeitpunkt
    pub joined_at: TemporalCoord,

    /// Realm-lokaler Trust (kann von Global-Trust abweichen)
    pub local_trust: f64,

    /// Rolle im Realm
    pub role: RealmRole,

    /// Aktive Delegationen innerhalb dieses Realms
    pub realm_delegations: Vec<UniversalId>,

    /// Ist die Mitgliedschaft aktiv?
    pub is_active: bool,

    /// Letzter Aktivitätszeitpunkt
    pub last_activity_at: Option<u64>,
}

impl RealmMembership {
    /// Erstelle neue Realm-Mitgliedschaft
    pub fn new(realm_id: UniversalId, root_did: UniversalId, role: RealmRole) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        Self {
            realm_id,
            root_did,
            realm_sub_did: None,
            joined_at: TemporalCoord::new(now, 0, 0),
            local_trust: 0.5, // Initial Trust im Realm
            role,
            realm_delegations: Vec::new(),
            is_active: true,
            last_activity_at: Some(now / 1000), // ms
        }
    }

    /// Mit Realm-spezifischer Sub-DID
    pub fn with_realm_sub_did(mut self, sub_did: UniversalId) -> Self {
        self.realm_sub_did = Some(sub_did);
        self
    }

    /// Mit initialem Trust
    pub fn with_trust(mut self, trust: f64) -> Self {
        self.local_trust = trust.clamp(0.0, 1.0);
        self
    }

    /// Berechne effektiven Trust (mit Rollen-Multiplikator)
    pub fn effective_trust(&self) -> f64 {
        (self.local_trust * self.role.trust_multiplier()).min(1.0)
    }

    /// Update Aktivität
    pub fn record_activity(&mut self) {
        self.last_activity_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        );
    }

    /// Mitgliedschaft deaktivieren (Soft-Delete)
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Füge Delegation hinzu
    pub fn add_delegation(&mut self, delegation_id: UniversalId) {
        if !self.realm_delegations.contains(&delegation_id) {
            self.realm_delegations.push(delegation_id);
        }
    }

    /// Entferne Delegation
    pub fn remove_delegation(&mut self, delegation_id: &UniversalId) {
        self.realm_delegations.retain(|d| d != delegation_id);
    }
}

// ============================================================================
// TRAITS: SECURE KEY STORE
// ============================================================================

/// Trait für sicheres Key-Management
///
/// Abstrahiert Hardware-gebundene (TEE, TPM, Secure Enclave) und
/// Software-basierte Key-Storage.
///
/// # Implementierungen
///
/// - `TeeKeyStore` - Trusted Execution Environment (z.B. Intel SGX)
/// - `TpmKeyStore` - Trusted Platform Module
/// - `SecureEnclaveKeyStore` - Apple Secure Enclave
/// - `SoftwareKeyStore` - Verschlüsselter Software-Speicher
/// - `TestKeyStore` - Fake-Implementierung für Tests
pub trait SecureKeyStore: Send + Sync + std::fmt::Debug {
    /// Signiere Payload mit einem Key
    ///
    /// # Errors
    ///
    /// - `IdentityError::KeyNotFound` - Key existiert nicht
    /// - `IdentityError::SignatureFailed` - Signatur fehlgeschlagen
    fn sign(&self, key_id: UniversalId, payload: &[u8]) -> Result<[u8; 64], IdentityError>;

    /// Verifiziere Signatur
    fn verify(&self, key_id: UniversalId, payload: &[u8], signature: &[u8]) -> bool;

    /// Leite neuen Key ab (HD Derivation)
    ///
    /// # Arguments
    ///
    /// - `parent` - Parent-Key UniversalId
    /// - `path` - BIP32-kompatibler Derivation-Pfad
    ///
    /// # Returns
    ///
    /// UniversalId des neuen Keys
    fn derive_key(&self, parent: UniversalId, path: &str) -> Result<UniversalId, IdentityError>;

    /// Exportiere öffentlichen Schlüssel
    fn export_public_key(&self, key_id: UniversalId) -> Result<[u8; 32], IdentityError>;

    /// Prüfe ob Key existiert
    fn has_key(&self, key_id: UniversalId) -> bool;

    /// Lösche Key (falls erlaubt)
    fn delete_key(&self, key_id: UniversalId) -> Result<(), IdentityError>;

    /// Ist dieser KeyStore Hardware-backed?
    fn is_hardware_backed(&self) -> bool;

    /// Key-Store Typ
    fn store_type(&self) -> KeyStoreType;
}

/// Typ des Key-Stores
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyStoreType {
    /// Trusted Execution Environment
    Tee,
    /// Trusted Platform Module
    Tpm,
    /// Apple Secure Enclave
    SecureEnclave,
    /// Android Keystore
    AndroidKeystore,
    /// Software-basiert (verschlüsselt)
    Software,
    /// Test/Mock
    Test,
}

impl KeyStoreType {
    /// Ist dieser Typ Hardware-backed?
    pub fn is_hardware_backed(&self) -> bool {
        !matches!(self, Self::Software | Self::Test)
    }
}

// ============================================================================
// TRAITS: PASSKEY MANAGER
// ============================================================================

/// Trait für WebAuthn/Passkey-Integration
///
/// Ermöglicht User-Confirmation für kritische Operationen via
/// Biometrie, PIN oder Security Key.
pub trait PasskeyManager: Send + Sync + std::fmt::Debug {
    /// Signiere mit User-Confirmation
    ///
    /// Zeigt dem User einen Confirmation-Dialog und wartet auf
    /// Biometrie/PIN/Security-Key Bestätigung.
    ///
    /// # Blocking
    ///
    /// Diese Methode blockiert bis User bestätigt oder abbricht.
    fn sign_with_confirmation(&self, payload: &[u8]) -> Result<[u8; 64], IdentityError>;

    /// Ist Passkey verfügbar?
    fn is_available(&self) -> bool;

    /// Ist Biometrie verfügbar?
    fn is_biometric_available(&self) -> bool;

    /// Registriere neuen Passkey
    fn register_passkey(&self, user_id: &str) -> Result<Vec<u8>, IdentityError>;

    /// Passkey-Typ
    fn passkey_type(&self) -> PasskeyType;
}

/// Typ des Passkey-Managers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PasskeyType {
    /// Platform Authenticator (z.B. Touch ID, Windows Hello)
    Platform,
    /// Cross-Platform (z.B. YubiKey)
    CrossPlatform,
    /// Software-Emulation (nur für Tests)
    Emulated,
}

// ============================================================================
// TRAITS: IDENTITY RESOLVER
// ============================================================================

/// Trait für Identity-Resolution (lokal und Cross-Shard)
///
/// Ermöglicht das Auflösen von UniversalIds zu DIDs,
/// sowohl lokal als auch über Shard-Grenzen hinweg.
pub trait IdentityResolver: Send + Sync + std::fmt::Debug {
    /// Resolve UniversalId zu DID
    fn resolve(&self, id: UniversalId) -> Option<crate::domain::unified::identity::DID>;

    /// Verifiziere Signatur für eine Identity
    fn verify(&self, signer: UniversalId, payload: &[u8], signature: &[u8]) -> bool;

    /// Hole den öffentlichen Schlüssel für eine Identity (Phase 7)
    /// Returns Ed25519 Public Key (32 bytes) falls verfügbar
    fn resolve_public_key(&self, id: &UniversalId) -> Option<Vec<u8>> {
        // Default: Extrahiere aus DID-Document falls vorhanden
        self.resolve(*id).and_then(|did| {
            // DID enthält die UniversalId, aus der wir den Public Key ableiten könnten
            // In Production: Lookup im DID-Document
            Some(did.id.as_bytes()[..32].to_vec())
        })
    }

    /// Ermittle Shard für eine Identity (Consistent Hashing)
    fn shard_for_identity(&self, id: &UniversalId) -> u64 {
        // Default: Consistent Hashing basierend auf ersten 8 Bytes
        let hash = blake3::hash(id.as_bytes());
        let shard_bits = u64::from_be_bytes(hash.as_bytes()[..8].try_into().unwrap());
        shard_bits % self.total_shards()
    }

    /// Anzahl der Shards
    fn total_shards(&self) -> u64;

    /// Ist Identity lokal verfügbar?
    fn is_local(&self, id: &UniversalId) -> bool {
        self.shard_for_identity(id) == self.local_shard()
    }

    /// Lokaler Shard-Index
    fn local_shard(&self) -> u64;

    /// Prüfe ob Identity existiert
    fn exists(&self, id: &UniversalId) -> bool {
        self.resolve(*id).is_some()
    }
}

// ============================================================================
// IDENTITY ERROR
// ============================================================================

/// Fehlertypen für Identity-Operationen
#[derive(Debug, Clone, thiserror::Error)]
pub enum IdentityError {
    /// Identity nicht bootstrapped
    #[error("Identity not bootstrapped - call bootstrap first")]
    NotBootstrapped,

    /// Kein Device-Key verfügbar
    #[error("No device key available")]
    NoDeviceKey,

    /// KeyStore nicht initialisiert
    #[error("Key store not initialized")]
    KeyStoreNotInitialized,

    /// Passkey nicht verfügbar
    #[error("Passkey not available on this device")]
    PasskeyNotAvailable,

    /// Signatur im aktuellen Modus nicht erlaubt
    #[error("Signature not allowed in mode: {0}")]
    SignatureNotAllowed(String),

    /// User hat Confirmation abgebrochen
    #[error("User cancelled confirmation")]
    UserCancelled,

    /// Signatur fehlgeschlagen
    #[error("Signature operation failed: {0}")]
    SignatureFailed(String),

    /// Key nicht gefunden
    #[error("Key not found: {0:?}")]
    KeyNotFound(UniversalId),

    /// Ungültiger Trust-Factor (Κ8)
    #[error("Invalid trust factor: {0} (must be in (0, 1])")]
    InvalidTrustFactor(f32),

    /// Derivation fehlgeschlagen
    #[error("Key derivation failed: {0}")]
    DerivationFailed(String),

    /// Ungültiger Derivation-Pfad
    #[error("Invalid derivation path: {0}")]
    InvalidDerivationPath(String),

    /// Identity existiert nicht
    #[error("Unknown identity: {0:?}")]
    UnknownIdentity(UniversalId),

    /// Identity bereits bootstrapped
    #[error("Identity already bootstrapped")]
    AlreadyBootstrapped,

    /// Modus-Wechsel nicht erlaubt
    #[error("Mode change from {from} to {to} not allowed")]
    ModeChangeNotAllowed { from: IdentityMode, to: IdentityMode },

    /// Ungültige Chain-ID
    #[error("Invalid chain ID: {0}")]
    InvalidChainId(String),

    /// Ungültige Adresse
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Realm-Membership nicht gefunden
    #[error("Not a member of realm: {0:?}")]
    NotRealmMember(UniversalId),

    /// Credential-Fehler
    #[error("Credential error: {0}")]
    CredentialError(String),

    /// Recovery fehlgeschlagen
    #[error("Recovery failed: {0}")]
    RecoveryFailed(String),

    /// Rate-Limit überschritten
    #[error("Rate limit exceeded for operation: {0}")]
    RateLimitExceeded(String),

    /// Internal Error
    #[error("Internal identity error: {0}")]
    Internal(String),
}

impl IdentityError {
    /// Ist dieser Fehler kritisch (sollte geloggt werden)?
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::KeyNotFound(_)
                | Self::SignatureFailed(_)
                | Self::DerivationFailed(_)
                | Self::RecoveryFailed(_)
                | Self::Internal(_)
        )
    }

    /// Ist dieser Fehler durch User verursacht?
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            Self::UserCancelled
                | Self::InvalidTrustFactor(_)
                | Self::InvalidDerivationPath(_)
                | Self::InvalidChainId(_)
                | Self::InvalidAddress(_)
        )
    }
}

// ============================================================================
// SHARED TYPES
// ============================================================================

/// Shared SecureKeyStore (thread-safe)
pub type SharedKeyStore = Arc<dyn SecureKeyStore>;

/// Shared PasskeyManager (thread-safe)
pub type SharedPasskeyManager = Arc<dyn PasskeyManager>;

/// Shared IdentityResolver (thread-safe)
pub type SharedIdentityResolver = Arc<dyn IdentityResolver>;

// ============================================================================
// TEST IMPLEMENTATIONS
// ============================================================================

/// Test-Implementierung für SecureKeyStore
#[cfg(test)]
#[derive(Debug, Default)]
pub struct TestKeyStore {
    keys: std::sync::RwLock<std::collections::HashMap<UniversalId, [u8; 32]>>,
}

#[cfg(test)]
impl TestKeyStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_key(&self, id: UniversalId, key: [u8; 32]) {
        self.keys.write().unwrap().insert(id, key);
    }
}

#[cfg(test)]
impl SecureKeyStore for TestKeyStore {
    fn sign(&self, key_id: UniversalId, payload: &[u8]) -> Result<[u8; 64], IdentityError> {
        if !self.has_key(key_id) {
            return Err(IdentityError::KeyNotFound(key_id));
        }
        // Fake Signatur: Hash von Key + Payload
        let mut sig = [0u8; 64];
        let hash = blake3::hash(&[key_id.as_bytes(), payload].concat());
        sig[..32].copy_from_slice(hash.as_bytes());
        Ok(sig)
    }

    fn verify(&self, key_id: UniversalId, payload: &[u8], signature: &[u8]) -> bool {
        if let Ok(expected) = self.sign(key_id, payload) {
            signature == expected
        } else {
            false
        }
    }

    fn derive_key(&self, parent: UniversalId, path: &str) -> Result<UniversalId, IdentityError> {
        if !self.has_key(parent) {
            return Err(IdentityError::KeyNotFound(parent));
        }
        let derived_id = UniversalId::new(
            UniversalId::TAG_DID,
            1,
            &[parent.as_bytes(), path.as_bytes()].concat(),
        );
        let derived_key = blake3::hash(&[parent.as_bytes(), path.as_bytes()].concat());
        self.keys
            .write()
            .unwrap()
            .insert(derived_id, *derived_key.as_bytes());
        Ok(derived_id)
    }

    fn export_public_key(&self, key_id: UniversalId) -> Result<[u8; 32], IdentityError> {
        self.keys
            .read()
            .unwrap()
            .get(&key_id)
            .copied()
            .ok_or(IdentityError::KeyNotFound(key_id))
    }

    fn has_key(&self, key_id: UniversalId) -> bool {
        self.keys.read().unwrap().contains_key(&key_id)
    }

    fn delete_key(&self, key_id: UniversalId) -> Result<(), IdentityError> {
        self.keys
            .write()
            .unwrap()
            .remove(&key_id)
            .ok_or(IdentityError::KeyNotFound(key_id))
            .map(|_| ())
    }

    fn is_hardware_backed(&self) -> bool {
        false
    }

    fn store_type(&self) -> KeyStoreType {
        KeyStoreType::Test
    }
}

/// Test-Implementierung für PasskeyManager
#[cfg(test)]
#[derive(Debug, Default)]
pub struct TestPasskeyManager;

#[cfg(test)]
impl PasskeyManager for TestPasskeyManager {
    fn sign_with_confirmation(&self, payload: &[u8]) -> Result<[u8; 64], IdentityError> {
        // Automatische Bestätigung im Test
        let mut sig = [0u8; 64];
        let hash = blake3::hash(payload);
        sig[..32].copy_from_slice(hash.as_bytes());
        Ok(sig)
    }

    fn is_available(&self) -> bool {
        true
    }

    fn is_biometric_available(&self) -> bool {
        false
    }

    fn register_passkey(&self, _user_id: &str) -> Result<Vec<u8>, IdentityError> {
        Ok(vec![0u8; 32]) // Fake credential
    }

    fn passkey_type(&self) -> PasskeyType {
        PasskeyType::Emulated
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_mode_conversion() {
        assert_eq!(IdentityMode::from_u8(0), IdentityMode::Interactive);
        assert_eq!(IdentityMode::from_u8(1), IdentityMode::AgentManaged);
        assert_eq!(IdentityMode::from_u8(2), IdentityMode::Ephemeral);
        assert_eq!(IdentityMode::from_u8(3), IdentityMode::Test);
        assert_eq!(IdentityMode::from_u8(99), IdentityMode::Test);
    }

    #[test]
    fn test_identity_mode_properties() {
        assert!(IdentityMode::Interactive.is_production_safe());
        assert!(IdentityMode::AgentManaged.is_production_safe());
        assert!(!IdentityMode::Ephemeral.is_production_safe());
        assert!(!IdentityMode::Test.is_production_safe());

        assert!(!IdentityMode::Interactive.allows_autonomous_signing());
        assert!(IdentityMode::AgentManaged.allows_autonomous_signing());
        assert!(IdentityMode::Ephemeral.allows_autonomous_signing());

        assert!(IdentityMode::Interactive.allows_realm_membership());
        assert!(!IdentityMode::Ephemeral.allows_realm_membership());
    }

    #[test]
    fn test_identity_mode_trust_penalty() {
        assert_eq!(IdentityMode::Interactive.trust_penalty_factor(), 1.0);
        assert_eq!(IdentityMode::AgentManaged.trust_penalty_factor(), 0.8);
        assert_eq!(IdentityMode::Ephemeral.trust_penalty_factor(), 0.5);
    }

    #[test]
    fn test_wallet_address_creation() {
        let did = UniversalId::new(UniversalId::TAG_DID, 1, b"test");
        let wallet = WalletAddress::new(
            "eip155:1",
            "0x1234567890123456789012345678901234567890",
            "m/44'/60'/0'/0/0",
            did,
        );

        assert_eq!(wallet.chain_namespace(), Some("eip155"));
        assert_eq!(wallet.chain_reference(), Some("1"));
        assert!(wallet.is_evm());
        assert!(wallet.validate().is_ok());
    }

    #[test]
    fn test_wallet_address_validation() {
        let did = UniversalId::new(UniversalId::TAG_DID, 1, b"test");

        // Ungültige EVM-Adresse
        let invalid_wallet = WalletAddress::new("eip155:1", "0x123", "m/44'/60'/0'/0/0", did);

        assert!(invalid_wallet.validate().is_err());
    }

    #[test]
    fn test_realm_role_hierarchy() {
        assert!(RealmRole::Owner.is_at_least(RealmRole::Admin));
        assert!(RealmRole::Admin.is_at_least(RealmRole::Moderator));
        assert!(RealmRole::Moderator.is_at_least(RealmRole::Member));
        assert!(!RealmRole::Member.is_at_least(RealmRole::Moderator));
    }

    #[test]
    fn test_realm_role_assignment() {
        assert!(RealmRole::Owner.can_assign(RealmRole::Admin));
        assert!(RealmRole::Admin.can_assign(RealmRole::Moderator));
        assert!(!RealmRole::Admin.can_assign(RealmRole::Owner));
        assert!(!RealmRole::Member.can_assign(RealmRole::Member));
    }

    #[test]
    fn test_realm_membership_creation() {
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"test-user");

        let membership = RealmMembership::new(realm_id, root_did, RealmRole::Member);

        assert_eq!(membership.realm_id, realm_id);
        assert_eq!(membership.root_did, root_did);
        assert_eq!(membership.role, RealmRole::Member);
        assert!(membership.is_active);
        assert_eq!(membership.effective_trust(), 0.5); // 0.5 * 1.0
    }

    #[test]
    fn test_realm_membership_effective_trust() {
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"test-user");

        let membership = RealmMembership::new(realm_id, root_did, RealmRole::Admin).with_trust(0.8);

        // 0.8 * 1.2 (Admin multiplier) = 0.96
        assert!((membership.effective_trust() - 0.96).abs() < 0.001);
    }

    #[test]
    fn test_test_key_store() {
        let store = TestKeyStore::new();
        let key_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-key");
        let key = [1u8; 32];

        store.add_key(key_id, key);

        assert!(store.has_key(key_id));
        assert!(store.export_public_key(key_id).is_ok());

        let signature = store.sign(key_id, b"test payload").unwrap();
        assert!(store.verify(key_id, b"test payload", &signature));
        assert!(!store.verify(key_id, b"wrong payload", &signature));
    }

    #[test]
    fn test_test_key_store_derivation() {
        let store = TestKeyStore::new();
        let parent_id = UniversalId::new(UniversalId::TAG_DID, 1, b"parent");
        let parent_key = [1u8; 32];

        store.add_key(parent_id, parent_key);

        let child_id = store.derive_key(parent_id, "m/44'/0'/0'/0/0").unwrap();

        assert!(store.has_key(child_id));
        assert_ne!(parent_id, child_id);
    }

    #[test]
    fn test_identity_error_classification() {
        assert!(IdentityError::KeyNotFound(UniversalId::NULL).is_critical());
        assert!(IdentityError::SignatureFailed("test".to_string()).is_critical());

        assert!(IdentityError::UserCancelled.is_user_error());
        assert!(IdentityError::InvalidTrustFactor(1.5).is_user_error());
    }
}
