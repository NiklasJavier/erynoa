# state.rs Refactoring-Plan: Identity-Layer Integration

> **Status:** Planung
> **Bezug:** DID-IDENTITY-SPECIFICATION.md, state.rs (16.610 Zeilen)
> **Geschätzter Umfang:** ~2.500 neue Zeilen, ~800 modifizierte Zeilen
> **Stand:** Februar 2026

---

## Übersicht

Dieses Dokument beschreibt den vollständigen Refactoring-Plan für `state.rs` zur Integration des Identity-Layers gemäß der DID-Identity-Spezifikation. Der Plan ist in **8 Phasen** unterteilt, die sequentiell oder teilweise parallel ausgeführt werden können.

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        REFACTORING ÜBERSICHT                                     │
│                                                                                  │
│  Phase 1: Foundations ──▶ Phase 2: IdentityState ──▶ Phase 3: StateComponent    │
│       │                        │                          │                      │
│       ▼                        ▼                          ▼                      │
│  Phase 4: StateEvent ───▶ Phase 5: StateGraph ────▶ Phase 6: Integration        │
│       │                        │                          │                      │
│       ▼                        ▼                          ▼                      │
│  Phase 7: Migration ────▶ Phase 8: Testing & Cleanup                            │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Foundations (Vorarbeiten)

### 1.1 Neue Typen & Traits

**Datei:** `backend/src/core/identity_types.rs` (NEU)

```rust
// Zu erstellen:
pub enum IdentityMode {
    Interactive = 0,
    AgentManaged = 1,
    Ephemeral = 2,
    Test = 3,
}

pub struct WalletAddress {
    pub chain_id: String,      // CAIP-2 Format
    pub address: String,
    pub derivation_path: String,
    pub derived_from: UniversalId,
}

pub struct RealmMembership {
    pub realm_id: UniversalId,
    pub root_did: UniversalId,
    pub realm_sub_did: Option<UniversalId>,
    pub joined_at: TemporalCoord,
    pub local_trust: f64,
    pub role: RealmRole,
    pub realm_delegations: Vec<UniversalId>,
}

pub enum RealmRole {
    Member,
    Moderator,
    Admin,
    Owner,
}

pub trait SecureKeyStore: Send + Sync {
    fn sign(&self, key_id: UniversalId, payload: &[u8]) -> Result<[u8; 64], IdentityError>;
    fn verify(&self, key_id: UniversalId, payload: &[u8], signature: &[u8]) -> bool;
    fn derive_key(&self, parent: UniversalId, path: &str) -> Result<UniversalId, IdentityError>;
    fn export_public_key(&self, key_id: UniversalId) -> Result<[u8; 32], IdentityError>;
}

pub trait PasskeyManager: Send + Sync {
    fn sign_with_confirmation(&self, payload: &[u8]) -> Result<[u8; 64], IdentityError>;
    fn is_available(&self) -> bool;
}

pub trait IdentityResolver: Send + Sync {
    fn resolve(&self, id: UniversalId) -> Option<DID>;
    fn verify(&self, signer: UniversalId, payload: &[u8], signature: &[u8]) -> bool;
    fn shard_for_identity(&self, id: &UniversalId) -> u64;
    fn total_shards(&self) -> u64;
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum IdentityError {
    #[error("Not bootstrapped")]
    NotBootstrapped,
    #[error("No device key available")]
    NoDeviceKey,
    #[error("Key store not initialized")]
    KeyStoreNotInitialized,
    #[error("Passkey not available")]
    PasskeyNotAvailable,
    #[error("Signature not allowed in current mode")]
    SignatureNotAllowed,
    #[error("Invalid trust factor: {0}")]
    InvalidTrustFactor(f32),
    #[error("Derivation failed: {0}")]
    DerivationFailed(String),
    #[error("Unknown identity: {0:?}")]
    UnknownIdentity(UniversalId),
}
```

**Aufgaben:**
- [ ] Neue Datei `identity_types.rs` erstellen
- [ ] Alle Typen implementieren
- [ ] In `mod.rs` exportieren
- [ ] Unit-Tests für Typen

**Abhängigkeiten:** Keine
**Geschätzte Zeilen:** ~300

---

### 1.2 Domain-Erweiterungen

**Datei:** `backend/src/domain/unified/identity.rs` (ERWEITERN)

```rust
// Hinzuzufügen:
impl DID {
    /// Prüfe ob DID zu einem bestimmten Namespace gehört
    pub fn is_namespace(&self, ns: DIDNamespace) -> bool {
        self.namespace == ns
    }

    /// Erstelle Device-Sub-DID
    pub fn derive_device(root: &DID, device_index: u32) -> Self { ... }

    /// Erstelle Agent-Sub-DID
    pub fn derive_agent(root: &DID, agent_index: u32) -> Self { ... }

    /// Erstelle Realm-Sub-DID
    pub fn derive_realm(root: &DID, realm_id: &UniversalId) -> Self { ... }
}

impl DIDDocument {
    /// Füge Device-Key hinzu
    pub fn add_device_key(&mut self, device_did: &DID) { ... }

    /// Finde Delegation für Delegate
    pub fn find_delegation_for(&self, delegate: &UniversalId) -> Option<&Delegation> { ... }
}
```

**Aufgaben:**
- [ ] `DID::derive_*` Methoden implementieren
- [ ] `DIDDocument` erweitern
- [ ] Tests aktualisieren

**Abhängigkeiten:** Phase 1.1
**Geschätzte Zeilen:** ~150

---

## Phase 2: IdentityState Implementierung

### 2.1 IdentityState Struktur

**Datei:** `backend/src/core/state.rs` (EINFÜGEN nach Zeile ~2380, vor StateRelation)

```rust
// ============================================================================
// IDENTITY STATE LAYER (NEU)
// ============================================================================

/// Identity-State-Layer für DID-Management
///
/// # Architektur
///
/// ```text
/// IdentityState
/// ├── Atomics (High-Frequency)
/// │   ├── bootstrap_completed
/// │   ├── mode
/// │   ├── sub_dids_total
/// │   └── ... (12 weitere)
/// ├── RwLock (Complex State)
/// │   ├── root_did
/// │   ├── root_document
/// │   ├── sub_dids
/// │   ├── delegations
/// │   └── realm_memberships
/// └── Handles (Orthogonal)
///     ├── key_store
///     └── passkey_manager
/// ```
///
/// # StateGraph-Beziehungen
///
/// - Trust DependsOn Identity
/// - Identity Triggers Trust
/// - Event DependsOn Identity
/// - Identity Triggers Event
/// - Swarm DependsOn Identity
/// - Controller DependsOn Identity
/// - ... (38 Kanten total)
#[derive(Debug)]
pub struct IdentityState {
    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-FREQUENCY ATOMICS (Lock-free)
    // ─────────────────────────────────────────────────────────────────────────

    /// Bootstrap abgeschlossen?
    pub bootstrap_completed: AtomicBool,

    /// Root-DID erstellt (Timestamp ms)
    pub root_created_at_ms: AtomicU64,

    /// Aktueller Modus (0=Interactive, 1=AgentManaged, 2=Ephemeral, 3=Test)
    pub mode: AtomicU8,

    /// Gesamtanzahl abgeleiteter Sub-DIDs
    pub sub_dids_total: AtomicU64,

    /// Gesamtanzahl abgeleiteter Wallet-Adressen
    pub addresses_total: AtomicU64,

    /// Aktive Delegationen
    pub active_delegations: AtomicU64,

    /// Widerrufene Delegationen
    pub revoked_delegations: AtomicU64,

    /// Credentials ausgestellt
    pub credentials_issued: AtomicU64,

    /// Credentials verifiziert
    pub credentials_verified: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // RELATIONSHIP COUNTERS (StateGraph-Tracking)
    // ─────────────────────────────────────────────────────────────────────────

    /// Identity → Triggers → Event
    pub events_triggered: AtomicU64,

    /// Identity → Triggers → Trust (Initial Trust-Entries)
    pub trust_entries_created: AtomicU64,

    /// Identity → Triggers → Realm (Join/Leave)
    pub realm_memberships_changed: AtomicU64,

    /// Gas verbraucht für Identity-Ops
    pub gas_consumed: AtomicU64,

    /// Mana verbraucht für Identity-Ops
    pub mana_consumed: AtomicU64,

    /// Signaturen erstellt
    pub signatures_created: AtomicU64,

    /// Signaturen verifiziert
    pub signatures_verified: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // COMPLEX STATE (RwLock-protected)
    // ─────────────────────────────────────────────────────────────────────────

    /// Root-DID (None vor Bootstrap)
    pub root_did: RwLock<Option<crate::domain::unified::identity::DID>>,

    /// DID-Document (None vor Bootstrap)
    pub root_document: RwLock<Option<crate::domain::unified::identity::DIDDocument>>,

    /// Device-Sub-DID (aktuelles Gerät)
    pub current_device_did: RwLock<Option<crate::domain::unified::identity::DID>>,

    /// Sub-DIDs nach Typ (device, agent, realm, custom)
    pub sub_dids: RwLock<HashMap<String, Vec<crate::domain::unified::identity::DID>>>,

    /// Sub-DID-Zähler nach Namespace
    pub sub_did_counts: RwLock<HashMap<crate::domain::unified::identity::DIDNamespace, u64>>,

    /// Wallet-Adressen nach Chain (CAIP-2 Format)
    pub wallets: RwLock<HashMap<String, Vec<WalletAddress>>>,

    /// Aktive Delegationen (delegate_id → Delegation)
    pub delegations: RwLock<HashMap<UniversalId, crate::domain::unified::identity::Delegation>>,

    /// Realm-Memberships (realm_id → membership_info)
    pub realm_memberships: RwLock<HashMap<UniversalId, RealmMembership>>,

    // ─────────────────────────────────────────────────────────────────────────
    // ORTHOGONAL HANDLES
    // ─────────────────────────────────────────────────────────────────────────

    /// Secure Key-Store Handle (TEE/TPM Abstraction)
    pub key_store: Option<Arc<dyn SecureKeyStore>>,

    /// WebAuthn/Passkey Manager Handle
    pub passkey_manager: Option<Arc<dyn PasskeyManager>>,
}
```

**Aufgaben:**
- [ ] `IdentityState` struct einfügen
- [ ] `IdentityState::new()` implementieren
- [ ] `IdentityState::snapshot()` implementieren
- [ ] `IdentityState::health_score()` implementieren
- [ ] Alle record_* Methoden implementieren

**Abhängigkeiten:** Phase 1.1, 1.2
**Geschätzte Zeilen:** ~600

---

### 2.2 IdentitySnapshot

**Datei:** `backend/src/core/state.rs` (EINFÜGEN nach IdentityState)

```rust
/// Snapshot für Persistence/CQRS (keine Keys!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySnapshot {
    pub bootstrap_completed: bool,
    pub root_created_at_ms: u64,
    pub mode: IdentityMode,
    pub sub_dids_total: u64,
    pub addresses_total: u64,
    pub active_delegations: u64,
    pub revoked_delegations: u64,
    pub credentials_issued: u64,
    pub credentials_verified: u64,
    pub events_triggered: u64,
    pub trust_entries_created: u64,
    pub realm_memberships_changed: u64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub signatures_created: u64,
    pub signatures_verified: u64,
    pub root_did: Option<String>,  // DID URI String
    pub sub_did_counts: HashMap<String, u64>,  // Namespace → Count
    pub realm_membership_count: usize,
    pub wallet_chains: Vec<String>,  // Liste der Chains mit Wallets
}
```

**Aufgaben:**
- [ ] `IdentitySnapshot` struct einfügen
- [ ] Serialization testen

**Abhängigkeiten:** Phase 2.1
**Geschätzte Zeilen:** ~50

---

### 2.3 Identity-Methoden

**Datei:** `backend/src/core/state.rs` (EINFÜGEN als impl-Block)

```rust
impl IdentityState {
    // ─────────────────────────────────────────────────────────────────────────
    // BOOTSTRAP
    // ─────────────────────────────────────────────────────────────────────────

    pub fn bootstrap_interactive(&self, passkey_manager: Arc<dyn PasskeyManager>) -> Result<(), IdentityError> { ... }
    pub fn bootstrap_agent_managed(&self, seed_phrase: &str, key_store: Arc<dyn SecureKeyStore>) -> Result<(), IdentityError> { ... }
    pub fn bootstrap_ephemeral(&self, ttl: Duration) -> Result<(), IdentityError> { ... }
    #[cfg(test)]
    pub fn bootstrap_test_mode(&self) { ... }

    // ─────────────────────────────────────────────────────────────────────────
    // DID OPERATIONS
    // ─────────────────────────────────────────────────────────────────────────

    pub fn root_did(&self) -> Result<Option<DID>, IdentityError> { ... }
    pub fn current_device_did(&self) -> Result<Option<DID>, IdentityError> { ... }
    pub fn derive_sub_did(&self, namespace: DIDNamespace, path: &str) -> Result<DID, IdentityError> { ... }
    pub fn exists(&self, id: &UniversalId) -> bool { ... }

    // ─────────────────────────────────────────────────────────────────────────
    // SIGNING
    // ─────────────────────────────────────────────────────────────────────────

    pub fn sign_with_device(&self, payload: &[u8]) -> Result<[u8; 64], IdentityError> { ... }
    pub fn sign_with_root(&self, payload: &[u8]) -> Result<[u8; 64], IdentityError> { ... }
    pub fn verify(&self, signer: UniversalId, payload: &[u8], signature: &[u8]) -> bool { ... }

    // ─────────────────────────────────────────────────────────────────────────
    // DELEGATION (Κ8)
    // ─────────────────────────────────────────────────────────────────────────

    pub fn add_delegation(&self, delegation: Delegation) -> Result<(), IdentityError> { ... }
    pub fn revoke_delegation(&self, delegation_id: &UniversalId) -> Result<(), IdentityError> { ... }
    pub fn get_delegation(&self, delegate: &UniversalId) -> Option<Delegation> { ... }

    // ─────────────────────────────────────────────────────────────────────────
    // CREDENTIALS
    // ─────────────────────────────────────────────────────────────────────────

    pub fn issue_credential(&self, issuer: UniversalId, subject: UniversalId, cred_type: &str, claim_hash: &[u8; 32]) -> Result<UniversalId, IdentityError> { ... }
    pub fn verify_credential(&self, credential_id: &UniversalId) -> Result<bool, IdentityError> { ... }

    // ─────────────────────────────────────────────────────────────────────────
    // REALM MEMBERSHIP
    // ─────────────────────────────────────────────────────────────────────────

    pub fn join_realm(&self, realm_id: UniversalId, role: RealmRole) -> Result<(), IdentityError> { ... }
    pub fn leave_realm(&self, realm_id: &UniversalId) -> Result<(), IdentityError> { ... }
    pub fn realm_membership(&self, identity: &UniversalId, realm: &UniversalId) -> Option<RealmMembership> { ... }

    // ─────────────────────────────────────────────────────────────────────────
    // HELPER
    // ─────────────────────────────────────────────────────────────────────────

    pub fn mode(&self) -> IdentityMode { ... }
    pub fn is_bootstrapped(&self) -> bool { ... }
}
```

**Aufgaben:**
- [ ] Alle Methoden implementieren
- [ ] Error-Handling durchgängig
- [ ] Gas/Mana-Tracking in relevanten Methoden

**Abhängigkeiten:** Phase 2.1, 2.2
**Geschätzte Zeilen:** ~500

---

## Phase 3: StateComponent-Erweiterung

### 3.1 Neue StateComponents

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN ~Zeile 2403)

```rust
/// State-Komponenten-Identifikator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateComponent {
    // ═══════════════════════════════════════════════════════════════════════════
    // IDENTITY-LAYER (NEU – Position: nach Core, vor Execution)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Identity-Management: DIDs, Sub-DIDs, Memberships
    Identity,

    /// Credential-Store: Verifiable Credentials, Attestations
    Credential,

    /// Key-Management: HD-Derivation, Rotation, Recovery
    KeyManagement,

    // Core (bestehend)
    Trust,
    Event,
    WorldFormula,
    Consensus,

    // ... (restliche bestehende Komponenten) ...
}
```

**Aufgaben:**
- [ ] `Identity`, `Credential`, `KeyManagement` zu enum hinzufügen
- [ ] Reihenfolge: Nach Core, vor Execution
- [ ] Alle Match-Statements aktualisieren die StateComponent verwenden

**Abhängigkeiten:** Phase 2
**Geschätzte Zeilen:** ~10 (aber viele Match-Updates)

---

### 3.2 StateComponent Match-Updates

**Dateien:** Alle Dateien die `StateComponent` matchen

```rust
// Beispiel: primary_component() in StateEvent
impl StateEvent {
    pub fn primary_component(&self) -> StateComponent {
        match self {
            // NEU: Identity-Events
            Self::IdentityBootstrapped { .. } |
            Self::IdentityModeChanged { .. } |
            Self::SubDIDDerived { .. } |
            Self::WalletDerived { .. } |
            Self::DelegationCreated { .. } |
            Self::DelegationRevoked { .. } |
            Self::CredentialIssued { .. } |
            Self::CredentialVerified { .. } |
            Self::KeyRotated { .. } |
            Self::RecoveryInitiated { .. } |
            Self::IdentityAnomalyDetected { .. } => StateComponent::Identity,

            // ... bestehende ...
        }
    }
}
```

**Aufgaben:**
- [ ] `StateEvent::primary_component()` erweitern
- [ ] `StateEvent::is_critical()` erweitern
- [ ] `StateEvent::realm_context()` erweitern
- [ ] `StateEvent::estimated_size_bytes()` erweitern
- [ ] Alle anderen Match-Statements prüfen

**Abhängigkeiten:** Phase 3.1
**Geschätzte Zeilen:** ~100

---

## Phase 4: StateEvent-Erweiterung

### 4.1 Neue Identity-Events

**Datei:** `backend/src/core/state.rs` (EINFÜGEN in StateEvent enum, ~Zeile 877)

```rust
pub enum StateEvent {
    // ... bestehende Events ...

    // ═══════════════════════════════════════════════════════════════════════════
    // IDENTITY EVENTS (NEU)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Root-DID erstellt (Bootstrap abgeschlossen)
    IdentityBootstrapped {
        root_did: UniversalId,
        namespace: crate::domain::unified::identity::DIDNamespace,
        mode: IdentityMode,
        timestamp_ms: u64,
    },

    /// Modus gewechselt (Interactive → AgentManaged)
    IdentityModeChanged {
        root_did: UniversalId,
        old_mode: IdentityMode,
        new_mode: IdentityMode,
        timestamp_ms: u64,
    },

    /// Sub-DID abgeleitet
    SubDIDDerived {
        root_did: UniversalId,
        sub_did: UniversalId,
        namespace: crate::domain::unified::identity::DIDNamespace,
        derivation_path: String,
        purpose: String,
        gas_used: u64,
        realm_id: Option<UniversalId>,
    },

    /// Wallet-Adresse abgeleitet
    WalletDerived {
        did: UniversalId,
        chain_id: String,
        address: String,
        derivation_path: String,
    },

    /// Delegation erstellt (Κ8)
    DelegationCreated {
        delegator: UniversalId,
        delegate: UniversalId,
        trust_factor: f32,
        capabilities: Vec<String>,
        valid_until: Option<u64>,
    },

    /// Delegation widerrufen
    DelegationRevoked {
        delegation_id: UniversalId,
        delegator: UniversalId,
        delegate: UniversalId,
        reason: String,
    },

    /// Credential ausgestellt
    CredentialIssued {
        issuer: UniversalId,
        subject: UniversalId,
        credential_type: String,
        claim_hash: [u8; 32],
    },

    /// Credential verifiziert
    CredentialVerified {
        verifier: UniversalId,
        credential_id: UniversalId,
        valid: bool,
    },

    /// Key rotiert
    KeyRotated {
        did: UniversalId,
        old_key_id: UniversalId,
        new_key_id: UniversalId,
        reason: String,
    },

    /// Recovery initiiert
    RecoveryInitiated {
        did: UniversalId,
        recovery_key_id: UniversalId,
        initiated_at: u64,
    },

    /// Identity-Anomalie erkannt
    IdentityAnomalyDetected {
        did: UniversalId,
        anomaly_type: String,
        severity: String,
        details: String,
    },

    /// Cross-Shard Identity aufgelöst
    CrossShardIdentityResolved {
        identity_id: UniversalId,
        source_shard: u64,
        target_shard: u64,
        success: bool,
        latency_ms: u64,
    },
}
```

**Aufgaben:**
- [ ] 12 neue Event-Varianten hinzufügen
- [ ] Serialization für alle neuen Typen sicherstellen
- [ ] Dokumentation für jede Variante

**Abhängigkeiten:** Phase 3
**Geschätzte Zeilen:** ~150

---

### 4.2 Event-Trait-Implementierungen

**Datei:** `backend/src/core/state.rs` (ERWEITERN impl StateEvent)

```rust
impl StateEvent {
    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            // NEU: Identity-Critical
            Self::IdentityBootstrapped { .. } |
            Self::IdentityModeChanged { .. } |
            Self::KeyRotated { .. } |
            Self::RecoveryInitiated { .. } |
            Self::IdentityAnomalyDetected { severity, .. } if severity == "critical" |

            // ... bestehende critical events ...
        )
    }

    pub fn realm_context(&self) -> Option<&UniversalId> {
        match self {
            // NEU
            Self::SubDIDDerived { realm_id, .. } => realm_id.as_ref(),

            // ... bestehende ...
        }
    }

    pub fn estimated_size_bytes(&self) -> usize {
        match self {
            // NEU
            Self::IdentityBootstrapped { .. } => 64,
            Self::IdentityModeChanged { .. } => 48,
            Self::SubDIDDerived { .. } => 128,
            Self::WalletDerived { .. } => 160,
            Self::DelegationCreated { capabilities, .. } => 96 + capabilities.len() * 32,
            Self::DelegationRevoked { .. } => 112,
            Self::CredentialIssued { .. } => 128,
            Self::CredentialVerified { .. } => 64,
            Self::KeyRotated { .. } => 128,
            Self::RecoveryInitiated { .. } => 64,
            Self::IdentityAnomalyDetected { details, .. } => 96 + details.len(),
            Self::CrossShardIdentityResolved { .. } => 56,

            // ... bestehende ...
        }
    }
}
```

**Aufgaben:**
- [ ] `primary_component()` für alle neuen Events
- [ ] `is_critical()` erweitern
- [ ] `realm_context()` erweitern
- [ ] `estimated_size_bytes()` erweitern

**Abhängigkeiten:** Phase 4.1
**Geschätzte Zeilen:** ~80

---

## Phase 5: StateGraph-Erweiterung

### 5.1 Identity-Kanten

**Datei:** `backend/src/core/state.rs` (ERWEITERN `StateGraph::erynoa_graph()` ~Zeile 2474)

```rust
impl StateGraph {
    pub fn erynoa_graph() -> Self {
        use StateComponent::*;
        use StateRelation::*;

        Self {
            edges: vec![
                // ═══════════════════════════════════════════════════════════════
                // IDENTITY-LAYER BEZIEHUNGEN (NEU – 38 Kanten)
                // ═══════════════════════════════════════════════════════════════

                // Core-Abhängigkeiten
                (Trust, DependsOn, Identity),
                (Identity, Triggers, Trust),
                (Event, DependsOn, Identity),
                (Identity, Triggers, Event),
                (Consensus, DependsOn, Identity),

                // Execution-Abhängigkeiten
                (Execution, DependsOn, Identity),
                (Identity, DependsOn, Execution),
                (Identity, DependsOn, Gas),
                (Identity, DependsOn, Mana),

                // Realm-Integration
                (Realm, DependsOn, Identity),
                (Identity, Triggers, Realm),
                (Room, DependsOn, Identity),
                (Partition, DependsOn, Identity),

                // Controller/Auth
                (Controller, DependsOn, Identity),
                (Identity, Validates, Controller),
                (Controller, Aggregates, Identity),

                // Gateway/Crossing
                (Gateway, DependsOn, Identity),
                (Gateway, Validates, Identity),

                // ECLVM
                (ECLVM, DependsOn, Identity),
                (ECLPolicy, DependsOn, Identity),

                // P2P Network
                (Swarm, DependsOn, Identity),
                (Swarm, Validates, Identity),
                (Gossip, DependsOn, Identity),
                (Privacy, DependsOn, Identity),

                // Protection
                (Anomaly, Validates, Identity),
                (Identity, Triggers, Anomaly),
                (AntiCalcification, Validates, Identity),

                // Credential-Sub-System
                (Credential, DependsOn, Identity),
                (Credential, Validates, Identity),
                (Identity, Aggregates, Credential),

                // Key-Management-Sub-System
                (KeyManagement, DependsOn, Identity),
                (Identity, Aggregates, KeyManagement),
                (KeyManagement, Triggers, Event),

                // Storage
                (KvStore, Aggregates, Identity),
                (Identity, DependsOn, KvStore),

                // Engine-Layer
                (UI, DependsOn, Identity),
                (API, DependsOn, Identity),
                (Governance, DependsOn, Identity),

                // ... bestehende Kanten ...
            ],
        }
    }
}
```

**Aufgaben:**
- [ ] 38 neue Kanten hinzufügen
- [ ] Reihenfolge: Identity-Kanten vor Core-Kanten
- [ ] Dokumentation der Kanten-Bedeutung

**Abhängigkeiten:** Phase 3, 4
**Geschätzte Zeilen:** ~60

---

## Phase 6: UnifiedState-Integration

### 6.1 UnifiedState erweitern

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN UnifiedState struct)

```rust
pub struct UnifiedState {
    pub started_at: Instant,

    // ═══════════════════════════════════════════════════════════════════════════
    // IDENTITY-LAYER (NEU – Position: vor Core)
    // ═══════════════════════════════════════════════════════════════════════════
    pub identity: IdentityState,

    // Core Layer
    pub core: CoreState,

    // ... restliche bestehende Felder ...
}
```

**Aufgaben:**
- [ ] `identity: IdentityState` Feld hinzufügen
- [ ] Position: Vor `core`

**Abhängigkeiten:** Phase 2, 5
**Geschätzte Zeilen:** ~5

---

### 6.2 UnifiedState::new() erweitern

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN UnifiedState::new())

```rust
impl UnifiedState {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            identity: IdentityState::new(),  // NEU
            core: CoreState::new(),
            // ... restliche Initialisierung ...
        }
    }
}
```

**Aufgaben:**
- [ ] `identity: IdentityState::new()` hinzufügen

**Abhängigkeiten:** Phase 6.1
**Geschätzte Zeilen:** ~2

---

### 6.3 UnifiedSnapshot erweitern

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN UnifiedSnapshot)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSnapshot {
    pub timestamp_ms: u64,
    pub identity: IdentitySnapshot,  // NEU
    pub core: CoreSnapshot,
    // ... restliche Felder ...
}
```

**Aufgaben:**
- [ ] `identity: IdentitySnapshot` Feld hinzufügen

**Abhängigkeiten:** Phase 6.2
**Geschätzte Zeilen:** ~2

---

### 6.4 UnifiedState::snapshot() erweitern

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN)

```rust
impl UnifiedState {
    pub fn snapshot(&self) -> UnifiedSnapshot {
        UnifiedSnapshot {
            timestamp_ms: /* ... */,
            identity: self.identity.snapshot(),  // NEU
            core: self.core.snapshot(),
            // ... restliche Snapshots ...
        }
    }
}
```

**Aufgaben:**
- [ ] `identity: self.identity.snapshot()` hinzufügen

**Abhängigkeiten:** Phase 6.3
**Geschätzte Zeilen:** ~2

---

### 6.5 apply_state_event() erweitern

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN)

```rust
impl UnifiedState {
    pub fn apply_state_event(&self, event: &StateEvent) {
        match event {
            // ═══════════════════════════════════════════════════════════════
            // IDENTITY EVENTS (NEU)
            // ═══════════════════════════════════════════════════════════════

            StateEvent::IdentityBootstrapped { mode, timestamp_ms, .. } => {
                self.identity.bootstrap_completed.store(true, Ordering::Release);
                self.identity.mode.store(*mode as u8, Ordering::Release);
                self.identity.root_created_at_ms.store(*timestamp_ms, Ordering::Release);
                self.identity.events_triggered.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::IdentityModeChanged { new_mode, .. } => {
                self.identity.mode.store(*new_mode as u8, Ordering::Release);
                self.identity.events_triggered.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::SubDIDDerived { namespace, gas_used, realm_id, .. } => {
                self.identity.sub_dids_total.fetch_add(1, Ordering::AcqRel);
                self.identity.gas_consumed.fetch_add(*gas_used, Ordering::AcqRel);
                if let Ok(mut counts) = self.identity.sub_did_counts.write() {
                    *counts.entry(*namespace).or_insert(0) += 1;
                }
                if realm_id.is_some() {
                    self.identity.realm_memberships_changed.fetch_add(1, Ordering::AcqRel);
                }
            }

            StateEvent::WalletDerived { .. } => {
                self.identity.addresses_total.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::DelegationCreated { .. } => {
                self.identity.active_delegations.fetch_add(1, Ordering::AcqRel);
                self.identity.trust_entries_created.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::DelegationRevoked { .. } => {
                self.identity.active_delegations.fetch_sub(1, Ordering::AcqRel);
                self.identity.revoked_delegations.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::CredentialIssued { .. } => {
                self.identity.credentials_issued.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::CredentialVerified { .. } => {
                self.identity.credentials_verified.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::KeyRotated { .. } | StateEvent::RecoveryInitiated { .. } => {
                self.identity.events_triggered.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::IdentityAnomalyDetected { severity, .. } => {
                if severity == "critical" {
                    self.circuit_breaker.record_critical_anomaly();
                }
            }

            StateEvent::CrossShardIdentityResolved { .. } => {
                // ShardMonitor tracking (falls aktiviert)
            }

            // ... bestehende Event-Handler ...
        }
    }
}
```

**Aufgaben:**
- [ ] Match-Zweige für alle 12 Identity-Events
- [ ] Atomic-Updates korrekt
- [ ] Integration mit Circuit Breaker

**Abhängigkeiten:** Phase 4, 6.4
**Geschätzte Zeilen:** ~80

---

### 6.6 calculate_health() erweitern

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN)

```rust
impl UnifiedState {
    pub fn calculate_health(&self) -> f64 {
        let mut score = 100.0;

        // ═══════════════════════════════════════════════════════════════════
        // IDENTITY HEALTH (3% Gewicht) - NEU
        // ═══════════════════════════════════════════════════════════════════
        let identity_health = self.identity.health_score();
        score -= (100.0 - identity_health) * 0.03;

        // Core Health (25% Gewicht)
        // ... bestehend ...

        score.max(0.0).min(100.0)
    }
}
```

**Aufgaben:**
- [ ] Identity-Health mit 3% Gewicht hinzufügen

**Abhängigkeiten:** Phase 6.5
**Geschätzte Zeilen:** ~5

---

## Phase 7: Migration bestehender Strukturen

### 7.1 RealmSpecificState Migration

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN RealmSpecificState)

```rust
pub struct RealmSpecificState {
    // MIGRATION: HashSet<String> → HashSet<UniversalId>

    /// Explizite Mitgliederliste (Identity UniversalIds)
    pub members: RwLock<HashSet<UniversalId>>,

    /// Legacy: String-basierte Member-IDs (deprecated, für Migration)
    #[deprecated(note = "Use members with UniversalId")]
    pub members_legacy: RwLock<HashSet<String>>,

    /// NEU: Mapping für Realm-spezifische Sub-DIDs
    pub member_realm_dids: RwLock<HashMap<UniversalId, UniversalId>>,

    // Pending/Banned/Admins analog migrieren
    pub pending_members: RwLock<HashSet<UniversalId>>,
    pub banned_members: RwLock<HashSet<UniversalId>>,
    pub admins: RwLock<HashSet<UniversalId>>,

    // ... restliche Felder unverändert ...
}
```

**Aufgaben:**
- [ ] `members` Typ ändern
- [ ] `members_legacy` für Migration hinzufügen
- [ ] `member_realm_dids` hinzufügen
- [ ] Alle Methoden aktualisieren (`add_member`, `is_member`, etc.)
- [ ] Migration-Helper für Legacy-Daten

**Abhängigkeiten:** Phase 2
**Geschätzte Zeilen:** ~150

---

### 7.2 TrustState Erweiterung

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN TrustState)

```rust
// Innerhalb CoreState oder als separater Block

/// Trust-Eintrag mit Identity-Integration
#[derive(Debug, Clone)]
pub struct TrustEntry {
    pub identity_id: UniversalId,
    pub global_trust: f64,
    pub per_realm_trust: HashMap<UniversalId, f64>,
    pub last_update: u64,  // Timestamp ms
    pub update_count: u64,
}

impl TrustState {
    // NEU: Trust-Map keyed by UniversalId
    pub trust_by_id: DashMap<UniversalId, TrustEntry>,

    pub fn get_trust(&self, identity: &UniversalId) -> Option<f64> { ... }
    pub fn get_realm_trust(&self, identity: &UniversalId, realm: &UniversalId) -> Option<f64> { ... }
    pub fn update_trust_with_identity(&self, identity: UniversalId, delta: f64, identity_state: &IdentityState) -> Result<f64, TrustError> { ... }
}
```

**Aufgaben:**
- [ ] `TrustEntry` struct hinzufügen
- [ ] `trust_by_id: DashMap<UniversalId, TrustEntry>` hinzufügen
- [ ] Neue Methoden implementieren
- [ ] Migration von String-basierten Trust-Einträgen

**Abhängigkeiten:** Phase 2
**Geschätzte Zeilen:** ~100

---

### 7.3 NetworkEvent Erweiterung

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN NetworkEvent)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub id: u64,
    pub event_type: String,
    pub payload: Vec<u8>,
    pub priority: EventPriority,
    pub peer_id: Option<String>,

    // NEU
    pub peer_universal_id: Option<UniversalId>,
    pub signature: Option<[u8; 64]>,
    #[serde(skip)]
    pub signature_verified: Option<bool>,

    pub realm_id: Option<String>,
    pub timestamp_ms: u64,
}

impl NetworkEvent {
    // NEU
    pub fn signed(/* ... */) -> Result<Self, IdentityError> { ... }
    pub fn verify_signature(&mut self, identity_resolver: &dyn IdentityResolver) -> bool { ... }
}
```

**Aufgaben:**
- [ ] Neue Felder hinzufügen
- [ ] `signed()` Methode implementieren
- [ ] `verify_signature()` Methode implementieren
- [ ] Serialization für `[u8; 64]` sicherstellen

**Abhängigkeiten:** Phase 2
**Geschätzte Zeilen:** ~100

---

### 7.4 SwarmState Erweiterung

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN SwarmState)

```rust
#[derive(Debug)]
pub struct SwarmState {
    pub peer_id: RwLock<String>,

    // NEU
    pub peer_universal_id: RwLock<Option<UniversalId>>,

    // ... restliche Felder unverändert ...
}

impl SwarmState {
    // NEU
    pub fn set_peer_identity(&self, device_sub_did: &DID) {
        if let Ok(mut pid) = self.peer_id.write() {
            *pid = device_sub_did.to_uri();
        }
        if let Ok(mut uid) = self.peer_universal_id.write() {
            *uid = Some(device_sub_did.id);
        }
    }
}
```

**Aufgaben:**
- [ ] `peer_universal_id` Feld hinzufügen
- [ ] `set_peer_identity()` Methode hinzufügen
- [ ] `SwarmSnapshot` aktualisieren

**Abhängigkeiten:** Phase 2
**Geschätzte Zeilen:** ~30

---

### 7.5 MembershipChange Event Migration

**Datei:** `backend/src/core/state.rs` (MODIFIZIEREN StateEvent)

```rust
pub enum StateEvent {
    // ...

    /// Mitgliedschafts-Änderung (erweitert)
    MembershipChange {
        realm_id: String,
        /// NEU: UniversalId (primär)
        identity_id: UniversalId,
        /// Legacy: String-Form für API-Kompatibilität
        identity_did_string: String,
        action: MembershipAction,
        new_role: Option<MemberRole>,
        /// NEU: UniversalId statt Option<String>
        initiated_by: Option<UniversalId>,
    },

    // ...
}
```

**Aufgaben:**
- [ ] `identity_id: UniversalId` hinzufügen
- [ ] `identity_did_string` für Kompatibilität behalten
- [ ] `initiated_by` Typ ändern
- [ ] Serialization prüfen

**Abhängigkeiten:** Phase 4
**Geschätzte Zeilen:** ~20

---

## Phase 8: Testing & Cleanup

### 8.1 Unit-Tests für IdentityState

**Datei:** `backend/src/core/state.rs` (ERWEITERN tests Modul)

```rust
#[cfg(test)]
mod identity_tests {
    use super::*;

    #[test]
    fn test_identity_state_new() {
        let state = IdentityState::new();
        assert!(!state.is_bootstrapped());
        assert_eq!(state.mode(), IdentityMode::Interactive);
    }

    #[test]
    fn test_identity_bootstrap_test_mode() {
        let state = IdentityState::new();
        state.bootstrap_test_mode();
        assert!(state.is_bootstrapped());
        assert_eq!(state.mode(), IdentityMode::Test);
    }

    #[test]
    fn test_identity_snapshot() {
        let state = IdentityState::new();
        state.bootstrap_test_mode();
        let snap = state.snapshot();
        assert!(snap.bootstrap_completed);
    }

    #[test]
    fn test_identity_health_score() {
        let state = IdentityState::new();
        assert_eq!(state.health_score(), 0.0); // Vor Bootstrap

        state.bootstrap_test_mode();
        assert_eq!(state.health_score(), 100.0); // Nach Bootstrap
    }

    #[test]
    fn test_delegation_trust_factor() {
        // Κ8: Trust-Factor muss in (0, 1] liegen
        let delegation = Delegation::new(
            UniversalId::NULL,
            UniversalId::NULL,
            0.8,
            vec![],
        );
        assert_eq!(delegation.trust_factor, 0.8);
    }

    #[test]
    #[should_panic]
    fn test_delegation_invalid_trust_factor() {
        Delegation::new(
            UniversalId::NULL,
            UniversalId::NULL,
            1.5, // Ungültig!
            vec![],
        );
    }

    // ... weitere Tests ...
}
```

**Aufgaben:**
- [ ] Mindestens 20 Unit-Tests für IdentityState
- [ ] Tests für alle Bootstrap-Modi
- [ ] Tests für Delegation (Κ8)
- [ ] Tests für Snapshots

**Abhängigkeiten:** Phase 7
**Geschätzte Zeilen:** ~300

---

### 8.2 Integration-Tests

**Datei:** `backend/tests/identity_integration.rs` (NEU)

```rust
#[tokio::test]
async fn test_identity_unified_state_integration() {
    let state = create_unified_state();

    // Bootstrap
    state.identity.bootstrap_test_mode();

    // Verify StateGraph propagation
    assert!(state.identity.is_bootstrapped());
}

#[tokio::test]
async fn test_identity_event_flow() {
    let state = create_unified_state();
    state.identity.bootstrap_test_mode();

    let event = StateEvent::SubDIDDerived {
        root_did: UniversalId::NULL,
        sub_did: UniversalId::new(UniversalId::TAG_DID, 1, b"test"),
        namespace: DIDNamespace::Spirit,
        derivation_path: "m/44'/erynoa'/0'/agent/0".to_string(),
        purpose: "agent".to_string(),
        gas_used: 100,
        realm_id: None,
    };

    state.apply_state_event(&event);

    assert_eq!(state.identity.sub_dids_total.load(Ordering::Acquire), 1);
    assert_eq!(state.identity.gas_consumed.load(Ordering::Acquire), 100);
}

// ... weitere Integration-Tests ...
```

**Aufgaben:**
- [ ] Integration-Tests erstellen
- [ ] Event-Flow testen
- [ ] Cross-Component Interaktionen testen

**Abhängigkeiten:** Phase 8.1
**Geschätzte Zeilen:** ~200

---

### 8.3 Dokumentation aktualisieren

**Aufgaben:**
- [ ] Inline-Dokumentation für alle neuen Typen
- [ ] StateGraph-Diagramm aktualisieren
- [ ] API-Dokumentation generieren
- [ ] STATE-RS-DEEP-DIVE.md aktualisieren

**Abhängigkeiten:** Phase 8.2
**Geschätzte Zeilen:** ~100 (Kommentare)

---

### 8.4 Cleanup & Deprecation

**Aufgaben:**
- [ ] `#[deprecated]` für Legacy-Felder hinzufügen
- [ ] Migration-Pfad dokumentieren
- [ ] Unused Imports entfernen
- [ ] Clippy-Warnungen beheben
- [ ] Formatting mit `cargo fmt`

**Abhängigkeiten:** Phase 8.3
**Geschätzte Zeilen:** -50 (Cleanup)

---

## Zusammenfassung

### Gesamtübersicht

| Phase | Beschreibung | Geschätzte Zeilen | Abhängigkeiten |
|-------|--------------|-------------------|----------------|
| 1.1 | Neue Typen & Traits | ~300 | – |
| 1.2 | Domain-Erweiterungen | ~150 | 1.1 |
| 2.1 | IdentityState Struktur | ~600 | 1.1, 1.2 |
| 2.2 | IdentitySnapshot | ~50 | 2.1 |
| 2.3 | Identity-Methoden | ~500 | 2.1, 2.2 |
| 3.1 | Neue StateComponents | ~10 | 2 |
| 3.2 | StateComponent Match-Updates | ~100 | 3.1 |
| 4.1 | Neue Identity-Events | ~150 | 3 |
| 4.2 | Event-Trait-Implementierungen | ~80 | 4.1 |
| 5.1 | Identity-Kanten im StateGraph | ~60 | 3, 4 |
| 6.1-6.6 | UnifiedState-Integration | ~100 | 2, 5 |
| 7.1 | RealmSpecificState Migration | ~150 | 2 |
| 7.2 | TrustState Erweiterung | ~100 | 2 |
| 7.3 | NetworkEvent Erweiterung | ~100 | 2 |
| 7.4 | SwarmState Erweiterung | ~30 | 2 |
| 7.5 | MembershipChange Migration | ~20 | 4 |
| 8.1 | Unit-Tests | ~300 | 7 |
| 8.2 | Integration-Tests | ~200 | 8.1 |
| 8.3 | Dokumentation | ~100 | 8.2 |
| 8.4 | Cleanup | -50 | 8.3 |
| **Total** | | **~3.050** | |

### Empfohlene Reihenfolge

```text
Week 1:  Phase 1.1 → 1.2 → 2.1 → 2.2
Week 2:  Phase 2.3 → 3.1 → 3.2
Week 3:  Phase 4.1 → 4.2 → 5.1
Week 4:  Phase 6.1-6.6 (parallel mit 7.1-7.2)
Week 5:  Phase 7.3 → 7.4 → 7.5
Week 6:  Phase 8.1 → 8.2 → 8.3 → 8.4
```

### Risiken & Mitigationen

| Risiko | Wahrscheinlichkeit | Impact | Mitigation |
|--------|-------------------|--------|------------|
| Breaking Changes in StateEvent Serialization | Mittel | Hoch | Versionierte Serialization, Migration-Helper |
| Performance-Regression durch RwLock-Nutzung | Niedrig | Mittel | Benchmarks vor/nach, DashMap wo möglich |
| Inkompatibilität mit bestehenden Checkpoints | Hoch | Hoch | Checkpoint-Version-Feld, Fallback-Loader |
| Komplexität durch 38 neue StateGraph-Kanten | Niedrig | Niedrig | Dokumentation, Tests |

### Rückwärtskompatibilität

1. **Legacy-Felder:** Alle migrierten Felder (z.B. `members_legacy`) bleiben für 2 Releases erhalten
2. **API-Strings:** `identity_did_string` in Events für externe API-Konsumenten
3. **Checkpoint-Format:** Version-Tag für neue Checkpoint-Struktur
4. **Migration:** Automatische Migration von String-IDs zu UniversalId beim ersten Load

---

## Checkliste für Review

- [x] Phase 1 vollständig implementiert und getestet (43 Tests)
- [x] Phase 2 vollständig implementiert und getestet (18 Tests)
- [x] Phase 3 vollständig implementiert und getestet (implizit in Phase 4)
- [x] Phase 4 vollständig implementiert und getestet (8 Tests)
- [x] Phase 5 vollständig implementiert und getestet (12 Tests)
- [x] Phase 6 vollständig implementiert und getestet (9 Tests)
- [x] Phase 7 vollständig implementiert und getestet (17 Tests)
- [x] Phase 8 vollständig implementiert und getestet (29 Unit + 21 Integration Tests)
- [ ] Alle Clippy-Warnungen behoben (minor warnings remain)
- [x] Alle Tests grün (725 lib + 21 integration = 746 Tests total)
- [x] Dokumentation aktualisiert (STATE-RS-DEEP-DIVE.md)
- [ ] Performance-Benchmarks durchgeführt
- [ ] Code-Review abgeschlossen

---

> **Letzte Aktualisierung:** 3. Februar 2026
> **Autor:** System-Architektur
> **Review-Status:** Implementation Complete
