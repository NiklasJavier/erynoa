# DID-Identitätslösung – Spezifikation v2.0

> **Status:** Spezifikation (Implementierung ausstehend)
> **Bezug:** `state.rs`, `state_integration.rs`, `domain/unified/identity.rs`, `domain/unified/primitives.rs`
> **Stand:** Februar 2026
> **Axiom-Referenz:** Κ6 (Existenz-Eindeutigkeit), Κ7 (Permanenz), Κ8 (Delegations-Struktur)

---

## Zusammenfassung

Diese Spezifikation definiert das **hierarchische DID-System** für Erynoa mit vollständiger Integration in das `UnifiedState`-Modell. Die Lösung baut auf den bestehenden Domain-Primitiven (`UniversalId`, `TemporalCoord`, `DIDNamespace`) auf und erweitert sie um einen dedizierten **Identity-State-Layer** mit StateGraph-Beziehungen, Event-Sourcing, Observer-Pattern und Self-Healing-Mechanismen.

**Kernprinzipien:**

1. **Axiom Κ6** – Existenz-Eindeutigkeit: Jede Entität hat exakt eine Root-DID
2. **Axiom Κ7** – Permanenz: Einmal erstellte DIDs sind unveränderlich
3. **Axiom Κ8** – Trust-Decay bei Delegation: `𝕋(delegate) ≤ trust_factor × 𝕋(delegator)`

---

## 1. Architektur-Übersicht

### 1.1 Einordnung im UnifiedState

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              UnifiedState                                        │
│                                                                                  │
│  ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐             │
│  │ IdentityState│   │ CoreState  │   │ExecutionState│  │ProtectionState│        │
│  │  (NEU)      │──▶│  Trust     │◀──│  Gas/Mana  │   │  Anomaly    │           │
│  │             │   │  Event     │   │  Execution │   │  Diversity  │           │
│  │  • RootDID  │   │  Formula   │   └────────────┘   └────────────┘           │
│  │  • SubDIDs  │   │  Consensus │                                              │
│  │  • Wallets  │   └────────────┘                                              │
│  │  • Creds    │                                                                │
│  └──────┬──────┘                                                                │
│         │                                                                        │
│         ▼                                                                        │
│  ┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐             │
│  │ PeerState  │   │ P2PState   │   │StorageState│   │Engine-Layer │            │
│  │  Gateway   │◀──│  Swarm     │   │  KvStore   │   │  UI/API/Gov │            │
│  │  Realm     │   │  Gossip    │   │  EventStore│   │  Controller │            │
│  │  Saga      │   │  Privacy   │   │  Archive   │   │  DataLogic  │            │
│  └────────────┘   └────────────┘   └────────────┘   └────────────┘             │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

**Position:** Der neue `IdentityState` wird als **fundamentale Schicht** direkt vor `CoreState` positioniert, da alle Trust-Operationen, Realm-Memberships und Controller-Permissions auf Identitäten basieren.

### 1.2 Hierarchisches DID-Modell

```text
Root-Seed (Passkey/WebAuthn – Hardware-bound, biometrisch/PIN)
│
└─▶ Root-DID (did:erynoa:self:<universal-id-hex>)
    │
    │  ┌─────────────────────────────────────────────────────────────────┐
    │  │  UniversalId (32 Bytes)                                         │
    │  │  ┌──────────┬────────────┬─────────────────────────────────────┐│
    │  │  │ TAG_DID  │  Version   │      BLAKE3(namespace + pubkey)     ││
    │  │  │ 0x0001   │  (2 bytes) │            (28 bytes)                ││
    │  │  └──────────┴────────────┴─────────────────────────────────────┘│
    │  └─────────────────────────────────────────────────────────────────┘
    │
    ├─▶ Device Sub-DIDs (did:erynoa:self:<derived-id>)
    │   ├─▶ Device #1 (HD-Pfad: m/44'/erynoa'/0'/device/0)
    │   ├─▶ Device #2 (HD-Pfad: m/44'/erynoa'/0'/device/1)
    │   └─▶ ... (automatisch pro Gerät)
    │
    ├─▶ Agent Sub-DIDs (did:erynoa:spirit:<derived-id>)
    │   ├─▶ Agent #1 (delegiert, HD-Pfad: m/44'/erynoa'/0'/agent/0)
    │   └─▶ ... (für autonome Operationen, Κ8: trust_factor ≤ 1.0)
    │
    ├─▶ Realm Sub-DIDs (did:erynoa:circle:<derived-id>)
    │   └─▶ Pro-Realm-Identity (isoliert, per-Realm-Trust)
    │
    └─▶ Wallet-Adressen (Chain-spezifisch, CAIP-2)
        ├─▶ ETH: m/44'/60'/0'/0/n
        ├─▶ SOL: m/44'/501'/0'/0/n
        └─▶ ... (BIP44-konform)
```

### 1.3 DIDNamespace-Integration

Die bestehenden 10 Namespaces aus `identity.rs` werden vollständig genutzt:

| Namespace | Byte | Verwendung | Human-Capable | AI |
|-----------|------|------------|---------------|----|
| `self` | 0x01 | Natürliche Personen, Root-DIDs | ✓ | – |
| `guild` | 0x02 | Organisationen, DAOs, Firmen | ✓ | – |
| `spirit` | 0x03 | KI-Agenten, autonome Systeme | – | ✓ |
| `thing` | 0x04 | IoT-Geräte, physische Assets | – | – |
| `vessel` | 0x05 | Container, Transportmittel | – | – |
| `source` | 0x06 | Datenquellen, APIs | – | – |
| `craft` | 0x07 | Dienstleistungen, Handwerke | – | – |
| `vault` | 0x08 | Speicher, Safes | – | – |
| `pact` | 0x09 | Verträge, Vereinbarungen | – | – |
| `circle` | 0x0A | Gruppen, Communities, Realms | ✓ | – |

**Namespace-Regeln:**
- Root-DIDs nutzen primär `self` (Personen) oder `guild` (Organisationen)
- Agent-Sub-DIDs nutzen `spirit` (Κ8: Trust-Decay automatisch angewandt)
- Realm-Memberships können isolierte `circle`-DIDs erhalten
- IoT/Machine-Identitäten nutzen `thing` mit strikteren Trust-Limits

---

## 2. Erzeugungsmodi

### 2.1 Modi-Übersicht

| Modus | Trigger | Seed-Quelle | Key-Speicherung | Trust-Level | Use-Case |
|-------|---------|-------------|-----------------|-------------|----------|
| **Interactive** | User-Prompt (UI) | WebAuthn/Passkey | Hardware-bound (TEE/SE) | Höchste | Persönliche Devices |
| **Agent-Managed** | API-Call | BIP39 Seed-Phrase | Encrypted Software-Key | Hoch | Server/Headless Nodes |
| **Ephemeral** | Session-Start | CSPRNG | Memory-only | Mittel | Anonyme Sessions |
| **Test** | `#[cfg(test)]` | Deterministisch | Fake | Niedrig | Unit-Tests |

### 2.2 Interactive Mode (Default)

```text
User ──▶ [WebAuthn Challenge] ──▶ Authenticator (Fingerprint/Face/PIN)
                                        │
                                        ▼
                              ┌─────────────────────┐
                              │ Secure Enclave/TPM  │
                              │  PRF Output (32B)   │
                              └─────────┬───────────┘
                                        │
                                        ▼
                         ┌──────────────────────────────┐
                         │ HKDF-SHA256(PRF, "erynoa-v1")│
                         │ ──▶ Master-Seed (64 Bytes)   │
                         └──────────────┬───────────────┘
                                        │
                                        ▼
                         ┌──────────────────────────────┐
                         │ BIP32 Root Key Derivation    │
                         │ m/44'/erynoa'/0'             │
                         └──────────────┬───────────────┘
                                        │
                                        ▼
                         ┌──────────────────────────────┐
                         │ Root-DID = DID::new_self(pk) │
                         │ UniversalId.type_tag = 0x0001│
                         └──────────────────────────────┘
```

**Eigenschaften:**
- Private Key **nie** exportierbar (Hardware-bound)
- Jede Root-Signatur erfordert User-Confirmation (Biometrie/PIN)
- Recovery nur über vorab registrierte Recovery-Keys (Extension Slot `0x0001`)

### 2.3 Agent-Managed Mode

```rust
// API-Call für headless Nodes
IdentityState::create_agent_managed(seed_phrase: &str) -> Result<RootDID, IdentityError>
```

**Eigenschaften:**
- Private Key verschlüsselt im Software-Keystore
- Autonome Signaturen ohne User-Confirmation
- **Modus-Wechsel:** Interactive → Agent-Managed ist **einmalig und irreversibel** (Event: `IdentityModeChanged`)
- Trust-Penalty: Agent-Managed Root-DIDs starten mit `base_trust × 0.8`

### 2.4 Ephemeral Mode

Für anonyme Sessions ohne persistente Identität:

```rust
// Kurzlebige Session-Identity
let ephemeral = IdentityState::create_ephemeral(ttl: Duration);
// UniversalId mit TAG_DID, aber ephemeral_flag gesetzt
// Auto-Cleanup nach TTL
```

**Einschränkungen:**
- Keine Realm-Memberships
- Keine Trust-Akkumulation
- Keine Wallet-Derivation
- Max TTL: 24h

---

## 3. UnifiedState-Integration

### 3.1 StateComponent-Erweiterung

```rust
pub enum StateComponent {
    // ... bestehende Komponenten ...

    // ═══════════════════════════════════════════════════════════════════
    // IDENTITY-LAYER (NEU – fundamentale Schicht)
    // ═══════════════════════════════════════════════════════════════════

    /// Identity-Management: DIDs, Sub-DIDs, Wallets, Credentials
    Identity,

    /// Credential-Store: Verifiable Credentials, Attestations
    Credential,

    /// Key-Management: HD-Derivation, Rotation, Recovery
    KeyManagement,
}
```

### 3.2 StateGraph-Beziehungen

```rust
impl StateGraph {
    pub fn erynoa_graph() -> Self {
        use StateComponent::*;
        use StateRelation::*;

        Self {
            edges: vec![
                // ═══════════════════════════════════════════════════════════
                // IDENTITY-LAYER BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════

                // Identity ist fundamental für Trust
                (Trust, DependsOn, Identity),      // Trust-Einträge keyed by UniversalId
                (Identity, Triggers, Trust),       // Neue Identity → Initial Trust-Entry

                // Identity ist fundamental für Events
                (Event, DependsOn, Identity),      // Events haben author: UniversalId
                (Identity, Triggers, Event),       // Identity-Ops emittieren Events

                // Execution benötigt Identity für Kontext
                (Execution, DependsOn, Identity),  // Execution-Context hat identity_id
                (Identity, DependsOn, Execution),  // Sub-DID-Derivation verbraucht Gas

                // Realm-Membership basiert auf Identity
                (Realm, DependsOn, Identity),      // Membership via UniversalId
                (Identity, Triggers, Realm),       // Join/Leave Events

                // Controller-Permissions basieren auf Identity
                (Controller, DependsOn, Identity), // AuthZ prüft Identity
                (Identity, Validates, Controller), // Identity validiert Permission-Grants

                // Gateway prüft Identity für Crossings
                (Gateway, DependsOn, Identity),    // Crossing-Auth via Identity
                (Gateway, Validates, Identity),    // Gateway validiert Identity-Claims

                // ECLVM nutzt Identity für Host-Funktionen
                (ECLVM, DependsOn, Identity),      // derive_subdid(), sign(), verify()

                // P2P-Auth via Identity
                (Swarm, DependsOn, Identity),      // Peer-Auth via Device-Sub-DID
                (Swarm, Validates, Identity),      // P2P validiert Identity-Signatur

                // Privacy-Layer für Identity-Schutz
                (Privacy, DependsOn, Identity),    // Anonymisierungs-Regeln pro Identity
                (Identity, DependsOn, Privacy),    // Ephemeral-Mode nutzt Privacy

                // Credential-Management
                (Credential, DependsOn, Identity), // Credentials gebunden an Identity
                (Credential, Validates, Identity), // Credentials attestieren Identity
                (Identity, Aggregates, Credential),// Identity trackt eigene Credentials

                // Key-Management
                (KeyManagement, DependsOn, Identity),  // Keys gehören zu Identity
                (Identity, Aggregates, KeyManagement), // Identity trackt eigene Keys
                (KeyManagement, Triggers, Event),      // Key-Rotation emittiert Events

                // Anomalie-Erkennung für Identity
                (Anomaly, Validates, Identity),    // Ungewöhnliche Identity-Patterns
                (Identity, Triggers, Anomaly),     // Suspicious Activity Reports

                // Anti-Calcification prüft Identity-Power
                (AntiCalcification, Validates, Identity), // Verhindert Identity-Monopole

                // Storage für Identity-Persistence
                (KvStore, Aggregates, Identity),   // Identity-Docs persistent
                (Identity, DependsOn, KvStore),    // Identity lädt aus Storage
            ],
        }
    }
}
```

### 3.3 IdentityState-Struktur

```rust
/// Identity-State-Layer (homogen zu TrustState, ExecutionState)
pub struct IdentityState {
    // ─────────────────────────────────────────────────────────────────
    // HIGH-FREQUENCY ATOMICS (Lock-free)
    // ─────────────────────────────────────────────────────────────────

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

    // ─────────────────────────────────────────────────────────────────
    // RELATIONSHIP COUNTERS (StateGraph-Tracking)
    // ─────────────────────────────────────────────────────────────────

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

    // ─────────────────────────────────────────────────────────────────
    // COMPLEX STATE (RwLock-protected)
    // ─────────────────────────────────────────────────────────────────

    /// Root-DID (None vor Bootstrap)
    pub root_did: RwLock<Option<DID>>,

    /// DID-Document (None vor Bootstrap)
    pub root_document: RwLock<Option<DIDDocument>>,

    /// Sub-DIDs nach Typ (device, agent, realm, custom)
    pub sub_dids: RwLock<HashMap<String, Vec<DID>>>,

    /// Sub-DID-Zähler nach Namespace
    pub sub_did_counts: RwLock<HashMap<DIDNamespace, u64>>,

    /// Wallet-Adressen nach Chain (CAIP-2 Format)
    pub wallets: RwLock<HashMap<String, Vec<WalletAddress>>>,

    /// Aktive Delegationen (delegate_id → Delegation)
    pub delegations: RwLock<HashMap<UniversalId, Delegation>>,

    /// Realm-Memberships (realm_id → membership_info)
    pub realm_memberships: RwLock<HashMap<UniversalId, RealmMembership>>,

    // ─────────────────────────────────────────────────────────────────
    // ORTHOGONAL HANDLES (außerhalb von state.rs implementiert)
    // ─────────────────────────────────────────────────────────────────

    /// Secure Key-Store Handle (TEE/TPM Abstraction)
    pub key_store: Option<Arc<dyn SecureKeyStore>>,

    /// WebAuthn/Passkey Manager Handle
    pub passkey_manager: Option<Arc<dyn PasskeyManager>>,
}

impl IdentityState {
    /// Erstelle neuen IdentityState (vor Bootstrap)
    pub fn new() -> Self {
        Self {
            bootstrap_completed: AtomicBool::new(false),
            root_created_at_ms: AtomicU64::new(0),
            mode: AtomicU8::new(0),
            sub_dids_total: AtomicU64::new(0),
            addresses_total: AtomicU64::new(0),
            active_delegations: AtomicU64::new(0),
            revoked_delegations: AtomicU64::new(0),
            credentials_issued: AtomicU64::new(0),
            credentials_verified: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
            trust_entries_created: AtomicU64::new(0),
            realm_memberships_changed: AtomicU64::new(0),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            root_did: RwLock::new(None),
            root_document: RwLock::new(None),
            sub_dids: RwLock::new(HashMap::new()),
            sub_did_counts: RwLock::new(HashMap::new()),
            wallets: RwLock::new(HashMap::new()),
            delegations: RwLock::new(HashMap::new()),
            realm_memberships: RwLock::new(HashMap::new()),
            key_store: None,
            passkey_manager: None,
        }
    }

    /// Snapshot für CQRS/Persistence
    pub fn snapshot(&self) -> IdentitySnapshot {
        IdentitySnapshot {
            bootstrap_completed: self.bootstrap_completed.load(Ordering::Acquire),
            root_created_at_ms: self.root_created_at_ms.load(Ordering::Acquire),
            mode: IdentityMode::from_u8(self.mode.load(Ordering::Acquire)),
            sub_dids_total: self.sub_dids_total.load(Ordering::Acquire),
            addresses_total: self.addresses_total.load(Ordering::Acquire),
            active_delegations: self.active_delegations.load(Ordering::Acquire),
            revoked_delegations: self.revoked_delegations.load(Ordering::Acquire),
            credentials_issued: self.credentials_issued.load(Ordering::Acquire),
            credentials_verified: self.credentials_verified.load(Ordering::Acquire),
            events_triggered: self.events_triggered.load(Ordering::Acquire),
            trust_entries_created: self.trust_entries_created.load(Ordering::Acquire),
            realm_memberships_changed: self.realm_memberships_changed.load(Ordering::Acquire),
            gas_consumed: self.gas_consumed.load(Ordering::Acquire),
            mana_consumed: self.mana_consumed.load(Ordering::Acquire),
            root_did: self.root_did.read().unwrap().clone(),
            sub_did_counts: self.sub_did_counts.read().unwrap().clone(),
            realm_membership_count: self.realm_memberships.read().unwrap().len(),
        }
    }

    /// Health-Score für calculate_health() (0-100)
    pub fn health_score(&self) -> f64 {
        let mut score = 100.0;

        // Bootstrap muss abgeschlossen sein
        if !self.bootstrap_completed.load(Ordering::Acquire) {
            return 0.0;
        }

        // Viele Revocations sind ein Warnsignal
        let active = self.active_delegations.load(Ordering::Acquire) as f64;
        let revoked = self.revoked_delegations.load(Ordering::Acquire) as f64;
        if active + revoked > 0.0 {
            let revocation_rate = revoked / (active + revoked);
            if revocation_rate > 0.5 {
                score -= (revocation_rate - 0.5) * 40.0; // Max -20 bei 100% revoked
            }
        }

        // Zu viele Sub-DIDs ohne Nutzung
        let sub_dids = self.sub_dids_total.load(Ordering::Acquire);
        let events = self.events_triggered.load(Ordering::Acquire);
        if sub_dids > 100 && events < sub_dids {
            score -= 10.0; // Potenzielle Sybil-Vorbereitung
        }

        score.max(0.0)
    }
}
```

### 3.4 IdentitySnapshot

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
    pub root_did: Option<DID>,
    pub sub_did_counts: HashMap<DIDNamespace, u64>,
    pub realm_membership_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum IdentityMode {
    Interactive = 0,
    AgentManaged = 1,
    Ephemeral = 2,
    Test = 3,
}

impl IdentityMode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::Interactive,
            1 => Self::AgentManaged,
            2 => Self::Ephemeral,
            _ => Self::Test,
        }
    }
}
```

---

## 4. StateEvent-Erweiterung

### 4.1 Identity-Events

```rust
pub enum StateEvent {
    // ... bestehende Events ...

    // ═══════════════════════════════════════════════════════════════════
    // IDENTITY EVENTS
    // ═══════════════════════════════════════════════════════════════════

    /// Root-DID erstellt (Bootstrap abgeschlossen)
    IdentityBootstrapped {
        root_did: UniversalId,
        namespace: DIDNamespace,
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
        namespace: DIDNamespace,
        derivation_path: String,
        purpose: String, // "device", "agent", "realm", "custom"
        gas_used: u64,
        realm_id: Option<UniversalId>,
    },

    /// Wallet-Adresse abgeleitet
    WalletDerived {
        did: UniversalId,
        chain_id: String, // CAIP-2 Format (e.g., "eip155:1")
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
        severity: String, // "low", "medium", "high", "critical"
        details: String,
    },
}

impl StateEvent {
    pub fn primary_component(&self) -> StateComponent {
        match self {
            // Identity-Events → StateComponent::Identity
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

            // ... andere Events ...
        }
    }

    pub fn is_critical(&self) -> bool {
        matches!(
            self,
            Self::IdentityBootstrapped { .. } |
            Self::IdentityModeChanged { .. } |
            Self::KeyRotated { .. } |
            Self::RecoveryInitiated { .. } |
            Self::IdentityAnomalyDetected { severity, .. } if severity == "critical"
        )
    }

    pub fn realm_context(&self) -> Option<&UniversalId> {
        match self {
            Self::SubDIDDerived { realm_id, .. } => realm_id.as_ref(),
            // ... andere realm-spezifische Events ...
            _ => None,
        }
    }

    pub fn estimated_size_bytes(&self) -> usize {
        match self {
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
            _ => 64,
        }
    }
}
```

---

## 5. Observer-Pattern Integration

### 5.1 IdentityObserver Trait

```rust
/// Identity-Observer für state_integration.rs
pub trait IdentityObserver: Send + Sync {
    // ─────────────────────────────────────────────────────────────────
    // BOOTSTRAP & MODE
    // ─────────────────────────────────────────────────────────────────

    /// Bootstrap abgeschlossen
    fn on_identity_bootstrapped(
        &self,
        root_did: &UniversalId,
        namespace: DIDNamespace,
        mode: IdentityMode,
    );

    /// Modus gewechselt
    fn on_mode_changed(
        &self,
        root_did: &UniversalId,
        old_mode: IdentityMode,
        new_mode: IdentityMode,
    );

    // ─────────────────────────────────────────────────────────────────
    // SUB-DID & WALLET DERIVATION
    // ─────────────────────────────────────────────────────────────────

    /// Sub-DID abgeleitet
    fn on_sub_did_derived(
        &self,
        root_did: &UniversalId,
        sub_did: &UniversalId,
        namespace: DIDNamespace,
        purpose: &str,
        realm_id: Option<&UniversalId>,
    );

    /// Wallet abgeleitet
    fn on_wallet_derived(
        &self,
        did: &UniversalId,
        chain_id: &str,
        address: &str,
    );

    // ─────────────────────────────────────────────────────────────────
    // DELEGATION (Κ8)
    // ─────────────────────────────────────────────────────────────────

    /// Delegation erstellt
    fn on_delegation_created(
        &self,
        delegator: &UniversalId,
        delegate: &UniversalId,
        trust_factor: f32,
        capabilities: &[String],
    );

    /// Delegation widerrufen
    fn on_delegation_revoked(
        &self,
        delegator: &UniversalId,
        delegate: &UniversalId,
        reason: &str,
    );

    // ─────────────────────────────────────────────────────────────────
    // CREDENTIALS
    // ─────────────────────────────────────────────────────────────────

    /// Credential ausgestellt
    fn on_credential_issued(
        &self,
        issuer: &UniversalId,
        subject: &UniversalId,
        credential_type: &str,
    );

    /// Credential verifiziert
    fn on_credential_verified(
        &self,
        verifier: &UniversalId,
        credential_id: &UniversalId,
        valid: bool,
    );

    // ─────────────────────────────────────────────────────────────────
    // KEY MANAGEMENT
    // ─────────────────────────────────────────────────────────────────

    /// Key rotiert
    fn on_key_rotated(
        &self,
        did: &UniversalId,
        reason: &str,
    );

    /// Recovery initiiert
    fn on_recovery_initiated(
        &self,
        did: &UniversalId,
    );

    // ─────────────────────────────────────────────────────────────────
    // ANOMALY
    // ─────────────────────────────────────────────────────────────────

    /// Anomalie erkannt
    fn on_identity_anomaly(
        &self,
        did: &UniversalId,
        anomaly_type: &str,
        severity: &str,
    );
}

pub type SharedIdentityObserver = Arc<dyn IdentityObserver>;
```

### 5.2 StateIntegrator-Erweiterung

```rust
impl IdentityObserver for StateIntegrator {
    fn on_identity_bootstrapped(
        &self,
        root_did: &UniversalId,
        namespace: DIDNamespace,
        mode: IdentityMode,
    ) {
        // Update IdentityState
        self.state.identity.bootstrap_completed.store(true, Ordering::Release);
        self.state.identity.mode.store(mode as u8, Ordering::Release);
        self.state.identity.root_created_at_ms.store(
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64,
            Ordering::Release,
        );

        // Trigger Trust-Init
        self.state.identity.trust_entries_created.fetch_add(1, Ordering::AcqRel);

        // Event emittieren
        self.state.identity.events_triggered.fetch_add(1, Ordering::AcqRel);

        // Cross-Module Propagation
        self.propagate_update(StateComponent::Identity);
    }

    fn on_sub_did_derived(
        &self,
        root_did: &UniversalId,
        sub_did: &UniversalId,
        namespace: DIDNamespace,
        purpose: &str,
        realm_id: Option<&UniversalId>,
    ) {
        // Update Counters
        self.state.identity.sub_dids_total.fetch_add(1, Ordering::AcqRel);

        // Update per-Namespace Counter
        {
            let mut counts = self.state.identity.sub_did_counts.write().unwrap();
            *counts.entry(namespace).or_insert(0) += 1;
        }

        // Gas/Mana Tracking (Identity DependsOn Execution)
        let gas_cost = 100; // Basis-Kosten für Derivation
        self.state.identity.gas_consumed.fetch_add(gas_cost, Ordering::AcqRel);

        // Realm-Tracking falls realm_id vorhanden
        if realm_id.is_some() {
            self.state.identity.realm_memberships_changed.fetch_add(1, Ordering::AcqRel);
        }

        self.propagate_update(StateComponent::Identity);
    }

    fn on_delegation_created(
        &self,
        delegator: &UniversalId,
        delegate: &UniversalId,
        trust_factor: f32,
        capabilities: &[String],
    ) {
        // Κ8: Trust-Decay Validierung
        assert!(trust_factor > 0.0 && trust_factor <= 1.0);

        self.state.identity.active_delegations.fetch_add(1, Ordering::AcqRel);
        self.state.identity.events_triggered.fetch_add(1, Ordering::AcqRel);

        // Trust-System benachrichtigen (Identity Triggers Trust)
        self.state.identity.trust_entries_created.fetch_add(1, Ordering::AcqRel);

        self.propagate_update(StateComponent::Identity);
    }

    fn on_delegation_revoked(
        &self,
        delegator: &UniversalId,
        delegate: &UniversalId,
        reason: &str,
    ) {
        self.state.identity.active_delegations.fetch_sub(1, Ordering::AcqRel);
        self.state.identity.revoked_delegations.fetch_add(1, Ordering::AcqRel);

        self.propagate_update(StateComponent::Identity);
    }

    fn on_identity_anomaly(
        &self,
        did: &UniversalId,
        anomaly_type: &str,
        severity: &str,
    ) {
        // Protection-System benachrichtigen
        self.state.protection.anomaly.record_anomaly(anomaly_type, severity);

        // Bei kritischen Anomalien: Circuit Breaker
        if severity == "critical" {
            self.state.circuit_breaker.record_critical_anomaly();
        }

        self.propagate_update(StateComponent::Identity);
        self.propagate_update(StateComponent::Anomaly);
    }
}
```

---

## 6. UnifiedState-Erweiterung

### 6.1 Struktur

```rust
pub struct UnifiedState {
    pub started_at: Instant,

    // ═══════════════════════════════════════════════════════════════════
    // IDENTITY-LAYER (NEU – fundamentale Schicht)
    // ═══════════════════════════════════════════════════════════════════
    pub identity: IdentityState,

    // Core Layer
    pub core: CoreState,

    // Execution Layer
    pub execution: ExecutionState,

    // ... restliche Layers ...
}

impl UnifiedState {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            identity: IdentityState::new(),  // NEU
            core: CoreState::new(),
            execution: ExecutionState::new(),
            // ...
        }
    }

    pub fn snapshot(&self) -> UnifiedSnapshot {
        UnifiedSnapshot {
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            identity: self.identity.snapshot(),  // NEU
            core: self.core.snapshot(),
            execution: self.execution.snapshot(),
            // ...
        }
    }
}
```

### 6.2 apply_state_event-Erweiterung

```rust
impl UnifiedState {
    pub fn apply_state_event(&self, event: &StateEvent) {
        match event {
            // ═══════════════════════════════════════════════════════════
            // IDENTITY EVENTS
            // ═══════════════════════════════════════════════════════════

            StateEvent::IdentityBootstrapped { root_did, namespace, mode, timestamp_ms } => {
                self.identity.bootstrap_completed.store(true, Ordering::Release);
                self.identity.mode.store(*mode as u8, Ordering::Release);
                self.identity.root_created_at_ms.store(*timestamp_ms, Ordering::Release);
                self.identity.events_triggered.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::IdentityModeChanged { old_mode, new_mode, .. } => {
                self.identity.mode.store(*new_mode as u8, Ordering::Release);
                self.identity.events_triggered.fetch_add(1, Ordering::AcqRel);
            }

            StateEvent::SubDIDDerived { namespace, gas_used, realm_id, .. } => {
                self.identity.sub_dids_total.fetch_add(1, Ordering::AcqRel);
                self.identity.gas_consumed.fetch_add(*gas_used, Ordering::AcqRel);
                {
                    let mut counts = self.identity.sub_did_counts.write().unwrap();
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

            // ... andere Events ...
        }
    }
}
```

### 6.3 calculate_health-Erweiterung

```rust
impl UnifiedState {
    pub fn calculate_health(&self) -> f64 {
        let mut score = 100.0;

        // ═══════════════════════════════════════════════════════════════
        // IDENTITY HEALTH (3% Gewicht)
        // ═══════════════════════════════════════════════════════════════
        let identity_health = self.identity.health_score();
        score -= (100.0 - identity_health) * 0.03;

        // Core Health (25% Gewicht)
        // ...

        // Execution Health (15% Gewicht)
        // ...

        score.max(0.0).min(100.0)
    }
}
```

---

## 7. ECLVM Host-Funktionen

### 7.1 Identity-Bezogene Host-Funktionen

```rust
/// ECLVM Host-Funktionen für Identity-Operationen
impl ErynoaHost {
    // ─────────────────────────────────────────────────────────────────
    // SUB-DID DERIVATION
    // ─────────────────────────────────────────────────────────────────

    /// Leite Sub-DID ab (verbraucht Gas)
    ///
    /// # ECL Syntax
    /// ```ecl
    /// let agent_did = derive_subdid("agent", "trading-bot", 0);
    /// ```
    pub fn derive_subdid(
        &mut self,
        sub_type: &str,      // "device", "agent", "realm", "custom"
        identifier: &str,     // Kontext-ID
        index: u32,
    ) -> Result<UniversalId, EclError> {
        // Gas-Kosten
        let gas_cost = match sub_type {
            "device" => 500,
            "agent" => 1000,
            "realm" => 2000,
            _ => 750,
        };

        self.consume_gas(gas_cost)?;

        // Namespace bestimmen
        let namespace = match sub_type {
            "device" => DIDNamespace::Self_,
            "agent" => DIDNamespace::Spirit,
            "realm" => DIDNamespace::Circle,
            _ => DIDNamespace::Self_,
        };

        // Derivation-Pfad
        let path = format!("m/44'/erynoa'/0'/{}/{}/{}", sub_type, identifier, index);

        // Actual Derivation (über IdentityState)
        let sub_did = self.state.identity.derive_sub_did(namespace, &path)?;

        // Event emittieren
        self.emit_event(StateEvent::SubDIDDerived {
            root_did: self.state.identity.root_did()?,
            sub_did: sub_did.id,
            namespace,
            derivation_path: path,
            purpose: sub_type.to_string(),
            gas_used: gas_cost,
            realm_id: self.current_realm_id(),
        })?;

        Ok(sub_did.id)
    }

    // ─────────────────────────────────────────────────────────────────
    // SIGNING
    // ─────────────────────────────────────────────────────────────────

    /// Signiere Payload mit DID
    ///
    /// # ECL Syntax
    /// ```ecl
    /// let sig = sign_with_did(my_did, payload_hash);
    /// ```
    pub fn sign_with_did(
        &mut self,
        did: UniversalId,
        payload: &[u8],
    ) -> Result<Signature, EclError> {
        // Gas-Kosten
        self.consume_gas(200)?;

        // Modus prüfen
        let mode = self.state.identity.mode();

        match mode {
            IdentityMode::Interactive => {
                // User-Confirmation erforderlich
                self.request_user_confirmation(ConfirmationType::Sign {
                    did,
                    payload_preview: payload[..32.min(payload.len())].to_vec(),
                })?;
            }
            IdentityMode::AgentManaged => {
                // Autonome Signatur erlaubt
            }
            IdentityMode::Ephemeral | IdentityMode::Test => {
                // Eingeschränkte Signatur
            }
        }

        self.state.identity.sign(did, payload)
    }

    // ─────────────────────────────────────────────────────────────────
    // VERIFICATION
    // ─────────────────────────────────────────────────────────────────

    /// Verifiziere Signatur
    ///
    /// # ECL Syntax
    /// ```ecl
    /// if verify_signature(did, payload, signature) { ... }
    /// ```
    pub fn verify_signature(
        &mut self,
        did: UniversalId,
        payload: &[u8],
        signature: &Signature,
    ) -> Result<bool, EclError> {
        self.consume_gas(100)?;
        self.state.identity.verify(did, payload, signature)
    }

    // ─────────────────────────────────────────────────────────────────
    // DELEGATION (Κ8)
    // ─────────────────────────────────────────────────────────────────

    /// Erstelle Delegation
    ///
    /// # ECL Syntax
    /// ```ecl
    /// delegate(agent_did, 0.7, ["read:*", "execute:transfer"]);
    /// ```
    pub fn delegate(
        &mut self,
        delegate: UniversalId,
        trust_factor: f32,
        capabilities: Vec<String>,
    ) -> Result<UniversalId, EclError> {
        // Κ8 Validierung
        if trust_factor <= 0.0 || trust_factor > 1.0 {
            return Err(EclError::InvalidTrustFactor(trust_factor));
        }

        self.consume_gas(500)?;
        self.consume_mana(100)?;

        let delegator = self.current_identity()?;

        let delegation = Delegation::new(
            delegator,
            delegate,
            trust_factor,
            capabilities.iter().map(|s| Capability::parse(s)).collect::<Result<_, _>>()?,
        );

        self.state.identity.add_delegation(delegation.clone())?;

        self.emit_event(StateEvent::DelegationCreated {
            delegator,
            delegate,
            trust_factor,
            capabilities,
            valid_until: delegation.valid_until.map(|t| t.wall_time()),
        })?;

        Ok(delegation.id)
    }

    // ─────────────────────────────────────────────────────────────────
    // CREDENTIAL OPERATIONS
    // ─────────────────────────────────────────────────────────────────

    /// Stelle Credential aus
    ///
    /// # ECL Syntax
    /// ```ecl
    /// issue_credential(subject_did, "KYC", claims);
    /// ```
    pub fn issue_credential(
        &mut self,
        subject: UniversalId,
        credential_type: &str,
        claims: &[u8],
    ) -> Result<UniversalId, EclError> {
        self.consume_gas(1000)?;
        self.consume_mana(200)?;

        let issuer = self.current_identity()?;
        let claim_hash = blake3::hash(claims);

        let credential_id = self.state.identity.issue_credential(
            issuer,
            subject,
            credential_type,
            claim_hash.as_bytes(),
        )?;

        self.emit_event(StateEvent::CredentialIssued {
            issuer,
            subject,
            credential_type: credential_type.to_string(),
            claim_hash: *claim_hash.as_bytes(),
        })?;

        Ok(credential_id)
    }
}
```

---

## 8. Realm-Integration

### 8.1 Per-Realm Identity-Isolation

```rust
/// Realm-spezifische Identity-Informationen
pub struct RealmMembership {
    /// Realm-ID
    pub realm_id: UniversalId,

    /// Root-DID des Members
    pub root_did: UniversalId,

    /// Realm-spezifische Sub-DID (optional)
    pub realm_sub_did: Option<UniversalId>,

    /// Beitrittszeitpunkt
    pub joined_at: TemporalCoord,

    /// Realm-lokaler Trust (kann von Global-Trust abweichen)
    pub local_trust: f64,

    /// Rolle im Realm
    pub role: RealmRole,

    /// Aktive Delegationen innerhalb dieses Realms
    pub realm_delegations: Vec<UniversalId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealmRole {
    Member,
    Moderator,
    Admin,
    Owner,
}
```

### 8.2 Crossing mit Identity-Validierung

```rust
impl Gateway {
    /// Crossing-Request mit Identity-Validierung
    pub fn request_crossing(
        &self,
        identity: &UniversalId,
        from_realm: &UniversalId,
        to_realm: &UniversalId,
    ) -> Result<CrossingToken, GatewayError> {
        // 1. Identity-Validierung
        let did = self.state.identity.resolve(identity)?;
        if did.is_none() {
            return Err(GatewayError::UnknownIdentity(*identity));
        }

        // 2. Realm-Membership prüfen (Source-Realm)
        let membership = self.state.identity.realm_membership(identity, from_realm)?;
        if membership.is_none() {
            return Err(GatewayError::NotMemberOfSourceRealm);
        }

        // 3. ECL-Crossing-Policy evaluieren
        let policy_result = self.evaluate_crossing_policy(
            identity,
            from_realm,
            to_realm,
            &membership.unwrap(),
        )?;

        if !policy_result.allowed {
            return Err(GatewayError::CrossingDenied(policy_result.reason));
        }

        // 4. Trust-Dampening anwenden (Κ8-ähnlich für Crossings)
        let dampened_trust = policy_result.source_trust * policy_result.dampening_factor;

        // 5. Crossing-Token erstellen
        Ok(CrossingToken {
            identity: *identity,
            from_realm: *from_realm,
            to_realm: *to_realm,
            trust_at_crossing: dampened_trust,
            valid_until: TemporalCoord::now(0, identity) + Duration::from_secs(300),
            crossing_id: UniversalId::new(UniversalId::TAG_CUSTOM, 1, &[]),
        })
    }
}
```

---

## 9. Protection & Self-Healing

### 9.1 Identity-Anomalie-Erkennung

```rust
impl ProtectionState {
    /// Identity-spezifische Anomalie-Checks
    pub fn check_identity_anomalies(&self, identity_state: &IdentityState) -> Vec<IdentityAnomaly> {
        let mut anomalies = Vec::new();

        // 1. Sybil-Verdacht: Zu viele Sub-DIDs in kurzer Zeit
        let sub_dids = identity_state.sub_dids_total.load(Ordering::Acquire);
        let events = identity_state.events_triggered.load(Ordering::Acquire);
        if sub_dids > 50 && events / sub_dids < 5 {
            anomalies.push(IdentityAnomaly {
                anomaly_type: "sybil_preparation".to_string(),
                severity: "high".to_string(),
                details: format!("{} Sub-DIDs mit nur {} Events", sub_dids, events),
            });
        }

        // 2. Delegation-Missbrauch: Hohe Revocation-Rate
        let active = identity_state.active_delegations.load(Ordering::Acquire) as f64;
        let revoked = identity_state.revoked_delegations.load(Ordering::Acquire) as f64;
        if revoked > 10.0 && revoked / (active + revoked) > 0.7 {
            anomalies.push(IdentityAnomaly {
                anomaly_type: "delegation_churn".to_string(),
                severity: "medium".to_string(),
                details: format!("70%+ Revocations: {} von {}", revoked as u64, (active + revoked) as u64),
            });
        }

        // 3. Gas-Anomalie: Übermäßiger Gas-Verbrauch für Identity-Ops
        let gas = identity_state.gas_consumed.load(Ordering::Acquire);
        if gas > 1_000_000 && sub_dids > 0 && gas / sub_dids > 10_000 {
            anomalies.push(IdentityAnomaly {
                anomaly_type: "gas_abuse".to_string(),
                severity: "medium".to_string(),
                details: format!("Hoher Gas-Verbrauch pro Sub-DID: {}", gas / sub_dids),
            });
        }

        anomalies
    }
}

pub struct IdentityAnomaly {
    pub anomaly_type: String,
    pub severity: String,
    pub details: String,
}
```

### 9.2 RealmQuota für Identity-Ops

```rust
impl RealmQuota {
    /// Neue Resource-Types für Identity
    pub const IDENTITY_DERIVATIONS: ResourceType = ResourceType::Custom("identity_derivations");
    pub const DELEGATIONS: ResourceType = ResourceType::Custom("delegations");
    pub const CREDENTIALS: ResourceType = ResourceType::Custom("credentials");

    /// Default-Limits
    pub fn default_identity_limits() -> HashMap<ResourceType, u64> {
        let mut limits = HashMap::new();
        limits.insert(Self::IDENTITY_DERIVATIONS, 100);  // Max 100 Sub-DIDs pro Realm
        limits.insert(Self::DELEGATIONS, 50);            // Max 50 aktive Delegationen
        limits.insert(Self::CREDENTIALS, 200);           // Max 200 Credentials
        limits
    }
}
```

---

## 10. Merkle-Integration

### 10.1 Identity-State im Merkle-Tree

```rust
impl MerkleStateTracker {
    /// Update Identity-Komponente im Merkle-Tree
    pub fn update_identity(&mut self, identity_state: &IdentityState) {
        let snapshot = identity_state.snapshot();
        let serialized = bincode::serialize(&snapshot).unwrap();

        self.update_component(StateComponent::Identity, &serialized);
    }
}
```

### 10.2 Identity-Proof für Light-Clients

```rust
/// Merkle-Proof für Identity-Verifikation
pub struct IdentityProof {
    /// Root-DID
    pub root_did: UniversalId,

    /// Merkle-Root zum Zeitpunkt der Proof-Erstellung
    pub merkle_root: MerkleHash,

    /// Proof-Pfad
    pub proof_path: Vec<MerkleHash>,

    /// Timestamp
    pub timestamp: TemporalCoord,
}

impl IdentityProof {
    /// Verifiziere Proof gegen aktuellen Merkle-Root
    pub fn verify(&self, current_root: &MerkleHash) -> bool {
        // Proof-Verifikation
        let mut current = blake3::hash(&bincode::serialize(&self.root_did).unwrap());

        for sibling in &self.proof_path {
            current = if current.as_bytes() < sibling.as_bytes() {
                blake3::hash(&[current.as_bytes(), sibling.as_bytes()].concat())
            } else {
                blake3::hash(&[sibling.as_bytes(), current.as_bytes()].concat())
            };
        }

        current.as_bytes() == current_root.as_bytes()
    }
}
```

---

## 11. CQRS & Event-Sourcing

### 11.1 StateDelta für Identity

```rust
impl StateBroadcaster {
    /// Broadcast Identity-Delta
    pub fn broadcast_identity_delta(
        &self,
        delta_type: DeltaType,
        data: IdentityDeltaData,
    ) {
        let delta = StateDelta {
            sequence: self.next_sequence(),
            component: StateComponent::Identity,
            delta_type,
            data: bincode::serialize(&data).unwrap(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            realm_id: None,
        };

        let _ = self.sender.send(delta);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IdentityDeltaData {
    Bootstrapped {
        root_did: UniversalId,
        mode: IdentityMode,
    },
    SubDIDDerived {
        sub_did: UniversalId,
        namespace: DIDNamespace,
    },
    DelegationChanged {
        delegation_id: UniversalId,
        active: bool,
    },
    CredentialIssued {
        credential_id: UniversalId,
    },
}
```

### 11.2 Event-Replay für Identity-Recovery

```rust
impl StateEventLog {
    /// Replay Identity-Events für Recovery
    pub fn replay_identity_events(&self, identity_state: &IdentityState) {
        let identity_events: Vec<_> = self.history.read().unwrap()
            .iter()
            .filter(|e| e.component == StateComponent::Identity)
            .cloned()
            .collect();

        for wrapped in identity_events {
            identity_state.apply_event(&wrapped.event);
        }
    }
}
```

---

## 12. Zukunfts-Erweiterungen

### 12.1 Extension Slots (bereits in identity.rs definiert)

```rust
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

    // ═══════════════════════════════════════════════════════════════════
    // GEPLANTE ERWEITERUNGEN (Reserviert)
    // ═══════════════════════════════════════════════════════════════════

    /// Zero-Knowledge Proof of Humanity
    pub const ZK_POH: u16 = 0x0010;
    /// Decentralized Social Graph
    pub const SOCIAL_GRAPH: u16 = 0x0011;
    /// Reputation Portability
    pub const REPUTATION_EXPORT: u16 = 0x0012;
    /// Multi-Party Computation Keys
    pub const MPC_KEYS: u16 = 0x0013;
    /// Post-Quantum Key Upgrade Path
    pub const PQ_UPGRADE: u16 = 0x0014;
}
```

### 12.2 Post-Quantum Migration Path

```rust
/// Zukünftige Migration zu Post-Quantum Kryptographie
pub struct PQMigrationConfig {
    /// Aktivierungsdatum
    pub activation_date: TemporalCoord,

    /// PQ-Algorithmus (z.B. CRYSTALS-Dilithium)
    pub algorithm: PQAlgorithm,

    /// Hybrid-Modus (Ed25519 + PQ parallel)
    pub hybrid_period_days: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum PQAlgorithm {
    Dilithium3,
    Falcon512,
    SphincsPlus128s,
}
```

### 12.3 Social Recovery

```rust
/// Social Recovery für Root-Key-Verlust
pub struct SocialRecoveryConfig {
    /// Guardians (vertrauenswürdige Kontakte)
    pub guardians: Vec<UniversalId>,

    /// Threshold (M-of-N)
    pub threshold: u8,

    /// Verzögerung vor Recovery-Aktivierung
    pub delay_hours: u32,

    /// Benachrichtigungs-Kanal
    pub notification_channel: NotificationChannel,
}
```

---

## 13. Implementierungs-Checkliste

### 13.1 state.rs Änderungen

| Bereich | Änderung | Priorität |
|---------|----------|-----------|
| `StateComponent` | `Identity`, `Credential`, `KeyManagement` hinzufügen | P0 |
| `StateGraph::erynoa_graph()` | Identity-Beziehungen (siehe 3.2) | P0 |
| `StateEvent` | Identity-Events (siehe 4.1) | P0 |
| `UnifiedState` | `identity: IdentityState` Feld | P0 |
| `UnifiedSnapshot` | `identity: IdentitySnapshot` Feld | P0 |
| `apply_state_event()` | Match für Identity-Events | P0 |
| `calculate_health()` | Identity-Health (3% Gewicht) | P1 |

### 13.2 state_integration.rs Änderungen

| Bereich | Änderung | Priorität |
|---------|----------|-----------|
| `IdentityObserver` Trait | Neu erstellen (siehe 5.1) | P0 |
| `StateIntegrator` | `impl IdentityObserver` | P0 |
| `CompositeObserver` | `IdentityObserver` integrieren | P1 |

### 13.3 Neue Module

| Modul | Inhalt | Priorität |
|-------|--------|-----------|
| `core/identity.rs` | `IdentityState`, `IdentitySnapshot`, `IdentityMode` | P0 |
| `core/key_store.rs` | `SecureKeyStore` Trait, Implementierungen | P0 |
| `core/credential.rs` | `CredentialStore`, VC-Management | P1 |
| `eclvm/identity_host.rs` | ECLVM Host-Funktionen | P1 |

### 13.4 Domain-Integration

| Bereich | Änderung | Priorität |
|---------|----------|-----------|
| `domain/unified/identity.rs` | `WalletAddress`, `RealmMembership` hinzufügen | P1 |
| `domain/unified/primitives.rs` | Keine Änderungen nötig | - |

---

## 14. Test-Strategie

### 14.1 Unit-Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_bootstrap() {
        let mut state = IdentityState::new();

        // Test-Mode aktivieren
        state.bootstrap_test_mode();

        assert!(state.bootstrap_completed.load(Ordering::Acquire));
        assert_eq!(state.mode.load(Ordering::Acquire), IdentityMode::Test as u8);
    }

    #[test]
    fn test_sub_did_derivation() {
        let mut state = IdentityState::new();
        state.bootstrap_test_mode();

        let sub_did = state.derive_sub_did(
            DIDNamespace::Spirit,
            "m/44'/erynoa'/0'/agent/test/0",
        ).unwrap();

        assert_eq!(sub_did.namespace, DIDNamespace::Spirit);
        assert_eq!(state.sub_dids_total.load(Ordering::Acquire), 1);
    }

    #[test]
    fn test_delegation_trust_factor() {
        // Κ8: Trust-Factor muss in (0, 1] liegen
        let result = Delegation::new(
            UniversalId::NULL,
            UniversalId::NULL,
            1.5,  // Ungültig!
            vec![],
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_health_score() {
        let state = IdentityState::new();

        // Vor Bootstrap: Health = 0
        assert_eq!(state.health_score(), 0.0);

        state.bootstrap_completed.store(true, Ordering::Release);

        // Nach Bootstrap: Health = 100
        assert_eq!(state.health_score(), 100.0);
    }
}
```

### 14.2 Integration-Tests

```rust
#[tokio::test]
async fn test_identity_event_flow() {
    let state = create_unified_state();
    let integrator = StateIntegrator::new(state.clone());

    // Bootstrap triggern
    integrator.on_identity_bootstrapped(
        &UniversalId::new(UniversalId::TAG_DID, 1, b"test"),
        DIDNamespace::Self_,
        IdentityMode::Test,
    );

    // Verify StateGraph propagation
    assert!(state.identity.bootstrap_completed.load(Ordering::Acquire));
    assert_eq!(state.identity.trust_entries_created.load(Ordering::Acquire), 1);
}
```

---

## 15. P2P-Integration & NetworkEvent

### 15.1 Peer-ID als Device-Sub-DID

Die bestehende `SwarmState.peer_id` und `NetworkEvent.peer_id` werden mit dem Identity-System verknüpft:

```rust
impl SwarmState {
    /// Peer-ID ist nun kanonisch das Device-Sub-DID
    /// Format: did:erynoa:self:<derived-id>
    pub peer_id: RwLock<String>,  // ← Device-Sub-DID String

    /// Zusätzlich: UniversalId für interne Referenz
    pub peer_universal_id: RwLock<Option<UniversalId>>,
}

impl SwarmState {
    /// Setze Peer-ID von IdentityState (beim Bootstrap)
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

### 15.2 NetworkEvent mit Identity-Signatur

```rust
/// Erweitertes NetworkEvent mit Identity-Signatur
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub id: u64,
    pub event_type: String,
    pub payload: Vec<u8>,
    pub priority: EventPriority,

    /// Peer-ID = Device-Sub-DID des Senders (bei Ingress) oder Empfängers (bei Egress)
    pub peer_id: Option<String>,

    /// NEU: UniversalId des Peers (für O(1) Lookup)
    pub peer_universal_id: Option<UniversalId>,

    /// Realm-Kontext
    pub realm_id: Option<String>,

    pub timestamp_ms: u64,

    /// NEU: Signatur über (id, event_type, payload_hash, timestamp_ms)
    /// Signiert mit Device-Sub-DID des Senders
    pub signature: Option<[u8; 64]>,

    /// NEU: Signatur-Verifikations-Status (nach Empfang)
    #[serde(skip)]
    pub signature_verified: Option<bool>,
}

impl NetworkEvent {
    /// Erstelle signiertes NetworkEvent
    pub fn signed(
        event_type: impl Into<String>,
        payload: Vec<u8>,
        priority: EventPriority,
        identity_state: &IdentityState,
    ) -> Result<Self, IdentityError> {
        let mut event = Self::new(event_type, payload, priority);

        // Setze Peer-ID
        if let Some(device_did) = identity_state.current_device_did()? {
            event.peer_id = Some(device_did.to_uri());
            event.peer_universal_id = Some(device_did.id);
        }

        // Signiere Event
        let sign_payload = Self::sign_payload(&event);
        let signature = identity_state.sign_with_device(&sign_payload)?;
        event.signature = Some(signature);

        Ok(event)
    }

    /// Payload für Signatur (deterministisch)
    fn sign_payload(event: &Self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(128);
        payload.extend_from_slice(&event.id.to_be_bytes());
        payload.extend_from_slice(event.event_type.as_bytes());
        payload.extend_from_slice(&blake3::hash(&event.payload).as_bytes()[..16]);
        payload.extend_from_slice(&event.timestamp_ms.to_be_bytes());
        payload
    }

    /// Verifiziere Signatur (bei Ingress)
    pub fn verify_signature(&mut self, identity_resolver: &dyn IdentityResolver) -> bool {
        let Some(signature) = &self.signature else {
            self.signature_verified = Some(false);
            return false;
        };

        let Some(peer_id) = &self.peer_universal_id else {
            self.signature_verified = Some(false);
            return false;
        };

        let sign_payload = Self::sign_payload(self);
        let result = identity_resolver.verify(*peer_id, &sign_payload, signature);
        self.signature_verified = Some(result);
        result
    }
}
```

### 15.3 EventBus mit Identity-Validierung

```rust
impl EventBus {
    /// Event in Ingress-Queue einreihen MIT Signatur-Verifikation
    pub fn try_send_ingress_verified(
        &self,
        mut event: NetworkEvent,
        identity_resolver: &dyn IdentityResolver,
    ) -> Result<(), NetworkEvent> {
        // Signatur verifizieren
        if !event.verify_signature(identity_resolver) {
            // Ungültige Signatur → Drop mit Anomalie-Meldung
            self.dropped_count.fetch_add(1, Ordering::Relaxed);
            return Err(event);
        }

        self.try_send_ingress(event)
    }
}
```

### 15.4 GossipState mit Trust-basiertem Scoring

```rust
impl GossipState {
    /// Message validieren mit Identity-Trust-Check
    pub fn validate_message(
        &self,
        peer_universal_id: &UniversalId,
        trust_state: &TrustState,
    ) -> bool {
        // Hole Trust für diesen Peer
        let trust = trust_state.get_trust(peer_universal_id).unwrap_or(0.0);

        // Minimum-Trust für Gossip-Participation
        const MIN_GOSSIP_TRUST: f64 = 0.1;

        if trust < MIN_GOSSIP_TRUST {
            self.messages_rejected.fetch_add(1, Ordering::Relaxed);
            return false;
        }

        self.messages_validated.fetch_add(1, Ordering::Relaxed);
        true
    }
}
```

---

## 16. Signatur-System & WrappedStateEvent

### 16.1 WrappedStateEvent-Signatur

Die bestehende `WrappedStateEvent.signature` wird mit IdentityState integriert:

```rust
impl WrappedStateEvent {
    /// Signiere Event mit Identity
    pub fn sign(
        &mut self,
        identity_state: &IdentityState,
    ) -> Result<(), IdentityError> {
        // Signatur-Payload: id + timestamp + event_hash
        let mut sign_data = Vec::with_capacity(64);
        sign_data.extend_from_slice(&self.id);
        sign_data.extend_from_slice(&self.timestamp_ms.to_be_bytes());
        sign_data.extend_from_slice(&blake3::hash(&bincode::serialize(&self.event).unwrap()).as_bytes()[..16]);

        let signature = identity_state.sign_with_device(&sign_data)?;
        self.signature = Some(signature.to_vec());

        Ok(())
    }

    /// Verifiziere Event-Signatur
    pub fn verify(
        &self,
        signer_id: &UniversalId,
        identity_resolver: &dyn IdentityResolver,
    ) -> bool {
        let Some(signature) = &self.signature else {
            return false;
        };

        let mut sign_data = Vec::with_capacity(64);
        sign_data.extend_from_slice(&self.id);
        sign_data.extend_from_slice(&self.timestamp_ms.to_be_bytes());
        sign_data.extend_from_slice(&blake3::hash(&bincode::serialize(&self.event).unwrap()).as_bytes()[..16]);

        identity_resolver.verify(*signer_id, &sign_data, signature)
    }
}
```

### 16.2 Signature64 Integration

```rust
/// Kompatibilität mit domain::Signature64
impl From<[u8; 64]> for Signature64 {
    fn from(bytes: [u8; 64]) -> Self {
        Signature64(bytes)
    }
}

impl IdentityState {
    /// Signiere mit Device-Sub-DID (Ed25519)
    pub fn sign_with_device(&self, payload: &[u8]) -> Result<[u8; 64], IdentityError> {
        let device_did = self.current_device_did()?
            .ok_or(IdentityError::NoDeviceKey)?;

        let key_store = self.key_store.as_ref()
            .ok_or(IdentityError::KeyStoreNotInitialized)?;

        key_store.sign(device_did.id, payload)
    }

    /// Signiere mit Root-DID (erfordert User-Confirmation bei Interactive Mode)
    pub fn sign_with_root(&self, payload: &[u8]) -> Result<[u8; 64], IdentityError> {
        let mode = IdentityMode::from_u8(self.mode.load(Ordering::Acquire));

        match mode {
            IdentityMode::Interactive => {
                // Erfordert User-Confirmation (WebAuthn)
                let passkey = self.passkey_manager.as_ref()
                    .ok_or(IdentityError::PasskeyNotAvailable)?;
                passkey.sign_with_confirmation(payload)
            }
            IdentityMode::AgentManaged => {
                // Autonome Signatur erlaubt
                let key_store = self.key_store.as_ref()
                    .ok_or(IdentityError::KeyStoreNotInitialized)?;
                let root_did = self.root_did()?
                    .ok_or(IdentityError::NotBootstrapped)?;
                key_store.sign(root_did.id, payload)
            }
            _ => Err(IdentityError::SignatureNotAllowed),
        }
    }
}
```

### 16.3 Consensus-Attestation mit Identity

```rust
impl Consensus {
    /// Füge Attestation hinzu (signiert mit Witness-Identity)
    pub fn add_attestation_signed(
        &mut self,
        ctx: &mut ExecutionContext,
        event_id: EventId,
        witness_identity: &IdentityState,
    ) -> ExecutionResult<FinalityCheck> {
        ctx.consume_gas(Self::GAS_ATTESTATION)?;

        // Hole Witness-DID
        let witness_did = witness_identity.root_did()?
            .ok_or(ExecutionError::IdentityNotBootstrapped)?;

        // Signiere Attestation
        let attest_payload = Self::attestation_payload(&event_id, &witness_did);
        let signature = witness_identity.sign_with_device(&attest_payload)
            .map_err(|_| ExecutionError::SignatureFailed)?;

        // Erstelle Attestation
        let attestation = WitnessAttestation {
            event_id: event_id.clone(),
            witness: witness_did.id,
            trust_at_witness: self.get_witness_trust(&witness_did.id)?,
            signature: Signature64::from(signature),
            attested_at: TemporalCoord::now(ctx.lamport(), &event_id),
        };

        self.record_attestation(attestation)
    }

    fn attestation_payload(event_id: &EventId, witness: &DID) -> Vec<u8> {
        let mut payload = Vec::with_capacity(96);
        payload.extend_from_slice(event_id.as_bytes());
        payload.extend_from_slice(witness.id.as_bytes());
        payload
    }
}
```

---

## 17. Sharding & Cross-Shard Identity

### 17.1 ShardMonitor-Integration

Die bestehende `ShardMonitor` in `ProtectionState` wird mit Identity erweitert:

```rust
impl ShardMonitor {
    /// Tracke Identity-Operation pro Shard
    pub fn record_identity_operation(&self, identity_id: &UniversalId, shard_id: u64) {
        // Aktivität inkrementieren
        self.shard_activity
            .entry(shard_id)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        // Identity-Distribution tracken (für Sybil-Detection)
        // Viele Identitäten aus einem Shard = verdächtig
    }

    /// Prüfe ob Identity aus einem biased Shard kommt
    pub fn is_identity_from_biased_shard(&self, shard_id: u64) -> bool {
        self.is_shard_biased(shard_id)
    }

    /// Berechne Trust-Penalty für Identitäten aus niedrig-reputablen Shards
    pub fn identity_trust_penalty(&self, shard_id: u64) -> f64 {
        let reputation = self.get_reputation(shard_id);
        if reputation < 0.5 {
            // Penalty: Bis zu 30% Trust-Reduktion bei Reputation 0.0
            0.3 * (1.0 - reputation * 2.0)
        } else {
            0.0
        }
    }
}
```

### 17.2 Shard-Aware Identity-Resolution

```rust
/// IdentityResolver mit Shard-Awareness
pub trait IdentityResolver: Send + Sync {
    /// Resolve Identity aus lokalem oder Remote-Shard
    fn resolve(&self, id: UniversalId) -> Option<DID>;

    /// Verifiziere Signatur
    fn verify(&self, signer: UniversalId, payload: &[u8], signature: &[u8]) -> bool;

    /// Ermittle Shard für eine Identity
    fn shard_for_identity(&self, id: &UniversalId) -> u64 {
        // Consistent Hashing basierend auf UniversalId
        let hash = blake3::hash(id.as_bytes());
        let shard_bits = u64::from_be_bytes(hash.as_bytes()[..8].try_into().unwrap());
        shard_bits % self.total_shards()
    }

    fn total_shards(&self) -> u64;
}

/// Cross-Shard Identity-Resolution
pub struct CrossShardIdentityResolver {
    /// Lokaler Shard-ID
    local_shard: u64,
    /// Total Shards
    total_shards: u64,
    /// Lokaler Identity-Cache
    local_cache: DashMap<UniversalId, DID>,
    /// Remote-Shard-Clients
    remote_clients: HashMap<u64, Arc<dyn ShardClient>>,
    /// ShardMonitor für Reputation-Check
    shard_monitor: Arc<ShardMonitor>,
}

impl CrossShardIdentityResolver {
    /// Resolve Identity (lokal oder remote)
    pub async fn resolve_async(&self, id: UniversalId) -> Option<DID> {
        let target_shard = self.shard_for_identity(&id);

        if target_shard == self.local_shard {
            // Lokal verfügbar
            self.local_cache.get(&id).map(|r| r.clone())
        } else {
            // Cross-Shard Query
            if self.shard_monitor.is_shard_quarantined(target_shard) {
                // Shard ist quarantiniert → Ablehnen
                return None;
            }

            let client = self.remote_clients.get(&target_shard)?;

            // Multi-Gas Kosten mit Shard-Reputation-Penalty
            let penalty = self.shard_monitor.cross_shard_penalty(target_shard);

            match client.resolve_identity(id).await {
                Ok(did) => {
                    self.shard_monitor.record_cross_shard_success(target_shard);
                    Some(did)
                }
                Err(_) => {
                    self.shard_monitor.record_cross_shard_failure(target_shard);
                    None
                }
            }
        }
    }
}
```

### 17.3 Identity-Events in Sharded Environment

```rust
/// Erweitertes StateEvent für Sharding
pub enum StateEvent {
    // ... bestehende Events ...

    /// Cross-Shard Identity-Resolution
    CrossShardIdentityResolved {
        identity_id: UniversalId,
        source_shard: u64,
        target_shard: u64,
        success: bool,
        latency_ms: u64,
    },

    /// Shard-Bias für Identity erkannt
    ShardIdentityBiasDetected {
        shard_id: u64,
        identity_count: u64,
        entropy: f64,
        threshold: f64,
    },
}
```

---

## 18. RealmSpecificState Identity-Konsistenz

### 18.1 Members als UniversalId

Die bestehende `RealmSpecificState.members: HashSet<String>` wird auf `UniversalId` umgestellt:

```rust
pub struct RealmSpecificState {
    // ─────────────────────────────────────────────────────────────────────────
    // MEMBERSHIP & IDENTITIES (Explizite Isolation)
    // ─────────────────────────────────────────────────────────────────────────

    /// Explizite Mitgliederliste (Identity UniversalIds)
    /// NEU: Migriert von HashSet<String> zu HashSet<UniversalId>
    pub members: RwLock<HashSet<UniversalId>>,

    /// Legacy: String-basierte Member-IDs (für Migration)
    #[deprecated(note = "Use members with UniversalId")]
    pub members_legacy: RwLock<HashSet<String>>,

    /// Mapping: UniversalId → Realm-spezifische Sub-DID (falls isoliert)
    pub member_realm_dids: RwLock<HashMap<UniversalId, UniversalId>>,

    /// Pending Membership-Requests (UniversalId)
    pub pending_members: RwLock<HashSet<UniversalId>>,

    /// Gebannte Identitäten (UniversalId)
    pub banned_members: RwLock<HashSet<UniversalId>>,

    /// Realm-Owner/Admin-Identitäten (UniversalId)
    pub admins: RwLock<HashSet<UniversalId>>,
}

impl RealmSpecificState {
    /// Füge Member hinzu (mit Identity-Validierung)
    pub fn add_member_validated(
        &self,
        identity_id: UniversalId,
        identity_state: &IdentityState,
    ) -> Result<(), RealmError> {
        // Validiere dass Identity existiert
        if !identity_state.exists(&identity_id) {
            return Err(RealmError::UnknownIdentity(identity_id));
        }

        // Prüfe ob gebannt
        if self.is_banned(&identity_id) {
            return Err(RealmError::IdentityBanned(identity_id));
        }

        // Füge hinzu
        if let Ok(mut members) = self.members.write() {
            if members.insert(identity_id) {
                self.identity_count.fetch_add(1, Ordering::Relaxed);
            }
        }

        // Entferne aus pending
        if let Ok(mut pending) = self.pending_members.write() {
            pending.remove(&identity_id);
        }

        Ok(())
    }

    /// Prüfe Membership mit UniversalId
    pub fn is_member(&self, identity_id: &UniversalId) -> bool {
        self.members
            .read()
            .map(|m| m.contains(identity_id))
            .unwrap_or(false)
    }
}
```

### 18.2 MembershipChange Event mit UniversalId

```rust
/// Mitgliedschafts-Änderung (erweitert)
pub enum StateEvent {
    MembershipChange {
        realm_id: String,
        /// NEU: UniversalId statt String
        identity_id: UniversalId,
        /// Legacy: String-Form für API-Kompatibilität
        identity_did_string: String,
        action: MembershipAction,
        new_role: Option<MemberRole>,
        initiated_by: Option<UniversalId>,
    },
}
```

---

## 19. TrustState Identity-Integration

### 19.1 Trust keyed by UniversalId

```rust
pub struct TrustState {
    // ... bestehende Felder ...

    /// NEU: Trust-Einträge keyed by UniversalId (primär)
    pub trust_by_id: DashMap<UniversalId, TrustEntry>,

    /// Legacy: Trust-Einträge keyed by String (für Migration)
    #[deprecated]
    pub trust_by_string: DashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct TrustEntry {
    pub identity_id: UniversalId,
    pub global_trust: f64,
    pub per_realm_trust: HashMap<UniversalId, f64>,
    pub last_update: TemporalCoord,
    pub update_count: u64,
}

impl TrustState {
    /// Get Trust für Identity
    pub fn get_trust(&self, identity: &UniversalId) -> Option<f64> {
        self.trust_by_id.get(identity).map(|e| e.global_trust)
    }

    /// Get Realm-spezifischer Trust
    pub fn get_realm_trust(&self, identity: &UniversalId, realm: &UniversalId) -> Option<f64> {
        self.trust_by_id
            .get(identity)
            .and_then(|e| e.per_realm_trust.get(realm).copied())
    }

    /// Update Trust (mit Identity-Validierung)
    pub fn update_trust(
        &self,
        identity: UniversalId,
        delta: f64,
        reason: TrustReason,
        identity_state: &IdentityState,
    ) -> Result<f64, TrustError> {
        // Validiere dass Identity existiert
        if !identity_state.exists(&identity) {
            return Err(TrustError::UnknownIdentity(identity));
        }

        let new_trust = self.trust_by_id
            .entry(identity)
            .or_insert_with(|| TrustEntry {
                identity_id: identity,
                global_trust: 0.5, // Initial Trust
                per_realm_trust: HashMap::new(),
                last_update: TemporalCoord::GENESIS,
                update_count: 0,
            })
            .and_modify(|e| {
                e.global_trust = (e.global_trust + delta).clamp(0.0, 1.0);
                e.update_count += 1;
            })
            .global_trust;

        Ok(new_trust)
    }
}
```

### 19.2 TrustUpdate Event mit UniversalId

```rust
pub enum StateEvent {
    TrustUpdate {
        /// NEU: UniversalId (primär)
        entity_id: UniversalId,
        /// Legacy: String-Form
        entity_id_string: String,
        delta: f64,
        reason: TrustReason,
        from_realm: Option<UniversalId>,
        triggered_events: u64,
        new_trust: f64,
    },
}
```

---

## 20. Controller-Delegation Synchronisation

### 20.1 ControllerState ↔ IdentityState Sync

Die bestehende `ControllerState` Delegation-Tracking wird mit `IdentityState` synchronisiert:

```rust
impl StateIntegrator {
    /// Synchronisiere Delegation zwischen Identity und Controller
    fn sync_delegation(
        &self,
        delegator: UniversalId,
        delegate: UniversalId,
        trust_factor: f32,
        capabilities: &[Capability],
    ) {
        // 1. IdentityState trackt die Delegation semantisch
        self.state.identity.active_delegations.fetch_add(1, Ordering::AcqRel);

        // 2. ControllerState trackt AuthZ-relevante Metriken
        let depth = self.calculate_delegation_depth(&delegator, &delegate);
        self.state.controller.create_delegation(depth, None);

        // 3. Trust-Propagation (Κ8)
        let delegator_trust = self.state.core.trust.get_trust(&delegator).unwrap_or(0.5);
        let delegate_trust = delegator_trust * trust_factor as f64;

        // Update Delegate-Trust (kann nicht höher sein als Delegator)
        let current_delegate_trust = self.state.core.trust.get_trust(&delegate).unwrap_or(0.0);
        if delegate_trust > current_delegate_trust {
            self.state.core.trust.update_trust_direct(&delegate, delegate_trust);
        }

        // 4. Events emittieren
        self.state.identity.trust_entries_created.fetch_add(1, Ordering::AcqRel);
    }

    fn calculate_delegation_depth(&self, delegator: &UniversalId, delegate: &UniversalId) -> u64 {
        // Traversiere Delegations-Chain um Tiefe zu ermitteln
        let delegations = self.state.identity.delegations.read().unwrap();

        let mut depth = 1u64;
        let mut current = *delegator;

        while let Some(parent_delegation) = delegations.values()
            .find(|d| d.delegate == current)
        {
            depth += 1;
            current = parent_delegation.delegator;

            // Zyklus-Detection
            if current == *delegate || depth > 10 {
                break;
            }
        }

        depth
    }
}
```

### 20.2 AuthZ-Check mit Identity-Resolution

```rust
impl ControllerState {
    /// AuthZ-Check mit Identity-Integration
    pub fn check_authorization_with_identity(
        &self,
        subject: UniversalId,
        action: &str,
        resource: &str,
        identity_state: &IdentityState,
        trust_state: &TrustState,
    ) -> AuthZResult {
        let start = std::time::Instant::now();

        // 1. Validiere Identity existiert
        if !identity_state.exists(&subject) {
            return AuthZResult::denied("Unknown identity");
        }

        // 2. Prüfe direkte Permission
        if self.has_direct_permission(&subject, action, resource) {
            let latency = start.elapsed().as_micros() as u64;
            self.check_authorization(true, false, latency, "direct", None);
            return AuthZResult::allowed();
        }

        // 3. Prüfe via Delegation
        if let Some(delegation) = self.find_delegation_for(&subject, action, resource, identity_state) {
            // Κ8: Trust-Decay prüfen
            let delegator_trust = trust_state.get_trust(&delegation.delegator).unwrap_or(0.0);
            let effective_trust = delegator_trust * delegation.trust_factor as f64;

            let required_trust = self.required_trust_for_action(action);
            if effective_trust >= required_trust {
                let latency = start.elapsed().as_micros() as u64;
                self.check_authorization(true, true, latency, "delegation", None);
                return AuthZResult::allowed_via_delegation(delegation.id);
            }
        }

        // 4. Abgelehnt
        let latency = start.elapsed().as_micros() as u64;
        self.check_authorization(false, false, latency, "direct", None);
        AuthZResult::denied("Insufficient permission")
    }
}
```

---

## 21. Vollständige StateGraph-Beziehungen (Final)

Die finale Liste aller Identity-bezogenen StateGraph-Kanten:

```rust
// ═══════════════════════════════════════════════════════════════════════════
// IDENTITY-LAYER BEZIEHUNGEN (vollständig)
// ═══════════════════════════════════════════════════════════════════════════

// Core-Abhängigkeiten
(Trust, DependsOn, Identity),           // Trust-Einträge keyed by UniversalId
(Identity, Triggers, Trust),            // Neue Identity → Initial Trust
(Event, DependsOn, Identity),           // Events haben author: UniversalId
(Identity, Triggers, Event),            // Identity-Ops emittieren Events
(Consensus, DependsOn, Identity),       // Attestations signiert mit Identity

// Execution-Abhängigkeiten
(Execution, DependsOn, Identity),       // Execution-Context hat identity_id
(Identity, DependsOn, Execution),       // Sub-DID-Derivation verbraucht Gas
(Identity, DependsOn, Gas),             // Identity-Ops verbrauchen Gas
(Identity, DependsOn, Mana),            // Signatur-Ops verbrauchen Mana

// Realm-Integration
(Realm, DependsOn, Identity),           // Membership via UniversalId
(Identity, Triggers, Realm),            // Join/Leave Events
(Room, DependsOn, Identity),            // Room-Access via Identity
(Partition, DependsOn, Identity),       // Partition-Access via Identity

// Controller/Auth
(Controller, DependsOn, Identity),      // AuthZ prüft Identity
(Identity, Validates, Controller),      // Identity validiert Permission-Grants
(Controller, Aggregates, Identity),     // Controller trackt Delegations von Identity

// Gateway/Crossing
(Gateway, DependsOn, Identity),         // Crossing-Auth via Identity
(Gateway, Validates, Identity),         // Gateway validiert Identity-Claims

// ECLVM
(ECLVM, DependsOn, Identity),           // derive_subdid(), sign(), verify()
(ECLPolicy, DependsOn, Identity),       // Policy-Evaluation nutzt Identity

// P2P Network
(Swarm, DependsOn, Identity),           // peer_id = Device-Sub-DID
(Swarm, Validates, Identity),           // P2P validiert Identity-Signatur
(Gossip, DependsOn, Identity),          // Message-Validation via Identity-Trust
(Privacy, DependsOn, Identity),         // Anonymisierung pro Identity

// Protection
(Anomaly, Validates, Identity),         // Ungewöhnliche Identity-Patterns
(Identity, Triggers, Anomaly),          // Suspicious Activity Reports
(AntiCalcification, Validates, Identity), // Verhindert Identity-Monopole

// Credential-Sub-System
(Credential, DependsOn, Identity),      // Credentials gebunden an Identity
(Credential, Validates, Identity),      // Credentials attestieren Identity
(Identity, Aggregates, Credential),     // Identity trackt eigene Credentials

// Key-Management-Sub-System
(KeyManagement, DependsOn, Identity),   // Keys gehören zu Identity
(Identity, Aggregates, KeyManagement),  // Identity trackt eigene Keys
(KeyManagement, Triggers, Event),       // Key-Rotation emittiert Events

// Storage
(KvStore, Aggregates, Identity),        // Identity-Docs persistent
(Identity, DependsOn, KvStore),         // Identity lädt aus Storage

// Engine-Layer
(UI, DependsOn, Identity),              // UI-Gates basieren auf Identity
(API, DependsOn, Identity),             // API-Auth basiert auf Identity
(Governance, DependsOn, Identity),      // Voting via Identity
```

---

## 22. Referenzen

- **W3C DID Core v1.0**: https://www.w3.org/TR/did-core/
- **Verifiable Credentials v2.0**: https://www.w3.org/TR/vc-data-model-2.0/
- **BIP32/39/44**: https://github.com/bitcoin/bips
- **WebAuthn Level 2**: https://www.w3.org/TR/webauthn-2/
- **CAIP-2 (Chain ID)**: https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md
- **BBS+ Signatures**: https://w3c-ccg.github.io/ldp-bbs2020/
- **Ed25519**: RFC 8032

---

## 23. Änderungshistorie

| Version | Datum | Änderungen |
|---------|-------|------------|
| 1.0 | Feb 2026 | Initiale Spezifikation |
| 2.0 | Feb 2026 | Vollständige Überarbeitung: UniversalId-Integration, P2P-Signaturen, Sharding, Controller-Sync |

---

> **Letzte Aktualisierung:** Februar 2026
> **Autor:** System-Architektur
> **Review-Status:** Pending
