# 🪪 Identitätslösung: Multi-DID Architektur mit Wallet-Ableitung

> **Teil von:** Projekt Pluto
> **Kategorie:** Kernentität (DNA)
> **Status:** Tiefenanalyse abgeschlossen

---

## 1. Fundamentales Konzept: Hierarchische DID-Architektur

### 1.1 Die DID ist das ZENTRUM aller Identität

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                       DID = DEZENTRALE IDENTITÄT                             ║
║                                                                              ║
║   Format: did:erynoa:<namespace>:<universal-id-hex>                         ║
║                                                                              ║
║   • Content-addressed (BLAKE3 Hash)                                         ║
║   • Self-certifying (Ed25519 Public Key = ID)                               ║
║   • Hierarchisch ableitbar (Root → Sub-DIDs)                                ║
║   • NO central registry – verifizierbar durch Public Key                    ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 1.2 Die 10 Namespaces

| Byte | Namespace | Beschreibung | Beispiel |
|------|-----------|--------------|----------|
| `0x01` | **Self_** | Natürliche Personen | `did:erynoa:self:abc...` |
| `0x02` | **Guild** | Organisationen, DAOs | `did:erynoa:guild:abc...` |
| `0x03` | **Spirit** | KI-Agenten, autonome Systeme | `did:erynoa:spirit:abc...` |
| `0x04` | **Thing** | IoT-Geräte, physische Assets | `did:erynoa:thing:abc...` |
| `0x05` | **Vessel** | Container, Transportmittel | `did:erynoa:vessel:abc...` |
| `0x06` | **Source** | Datenquellen, APIs | `did:erynoa:source:abc...` |
| `0x07` | **Craft** | Dienstleistungen, Services | `did:erynoa:craft:abc...` |
| `0x08` | **Vault** | Speicher, Safes | `did:erynoa:vault:abc...` |
| `0x09` | **Pact** | Verträge, Vereinbarungen | `did:erynoa:pact:abc...` |
| `0x0A` | **Circle** | Gruppen, Realms | `did:erynoa:circle:abc...` |

---

## 2. Die Hierarchie: Root-DID → Sub-DIDs

### 2.1 Der DID-Baum

```text
                            ┌────────────────────────┐
                            │      ROOT-DID          │
                            │  (did:erynoa:self:...)  │
                            │  Ed25519 Public Key    │
                            └───────────┬────────────┘
                                        │
            ┌───────────────────────────┼───────────────────────────┐
            │                           │                           │
            ▼                           ▼                           ▼
    ┌───────────────┐          ┌───────────────┐          ┌────────────────┐
    │  DEVICE-DID   │          │  AGENT-DID    │          │  REALM-DID     │
    │ (Self_)       │          │ (Spirit)      │          │ (Circle)       │
    │ m/44'/ery/0/  │          │ m/44'/ery/0/  │          │ Realm-isoliert │
    │   device/0    │          │   agent/0     │          │                │
    └───────────────┘          └───────────────┘          └────────────────┘
            │                           │                           │
            ▼                           ▼                           ▼
    ┌───────────────┐          ┌───────────────┐          ┌────────────────┐
    │ WALLET:       │          │ Skills:       │          │ Lokaler Trust  │
    │ - ETH (eip155)│          │ - execute:*   │          │ Realm-lokale   │
    │ - SOL         │          │ - attest:*    │          │   Aktivität    │
    │ - BTC         │          │ - delegate:2  │          │                │
    └───────────────┘          └───────────────┘          └────────────────┘
```

### 2.2 Ableitungs-Formeln

```rust
// Device-DID Ableitung
fn derive_device(root: &DID, device_index: u32) -> DID {
    let content = [root.public_key, b"device", device_index.to_be_bytes()].concat();
    let derived_key = blake3::hash(&content);
    DID::new(DIDNamespace::Self_, &derived_key)
}

// Agent-DID Ableitung
fn derive_agent(root: &DID, agent_index: u32) -> DID {
    let content = [root.public_key, b"agent", agent_index.to_be_bytes()].concat();
    let derived_key = blake3::hash(&content);
    DID::new(DIDNamespace::Spirit, &derived_key)
}

// Realm-DID Ableitung
fn derive_realm(root: &DID, realm_id: &UniversalId) -> DID {
    let content = [root.public_key, b"realm", realm_id.as_bytes()].concat();
    let derived_key = blake3::hash(&content);
    DID::new(DIDNamespace::Circle, &derived_key)
}

// Custom Ableitung
fn derive_custom(root: &DID, namespace: DIDNamespace, context: &str, index: u32) -> DID {
    let content = [root.public_key, context.as_bytes(), index.to_be_bytes()].concat();
    let derived_key = blake3::hash(&content);
    DID::new(namespace, &derived_key)
}
```

### 2.3 BIP44-ähnliche Derivation-Pfade

```text
m / 44' / erynoa' / 0' / <zweck> / <index>

Beispiele:
- m/44'/erynoa'/0'/device/0   → Erstes Gerät
- m/44'/erynoa'/0'/device/1   → Zweites Gerät
- m/44'/erynoa'/0'/agent/0    → Erster KI-Agent
- m/44'/erynoa'/0'/agent/5    → Sechster KI-Agent
- m/44'/erynoa'/0'/realm/xyz  → Realm-spezifische Identität
```

---

## 3. Die Betriebsmodi (IdentityMode)

### 3.1 Vier Modi

| Modus | Code | Signaturen | Trust-Penalty | Realm-Membership |
|-------|------|------------|---------------|------------------|
| **Interactive** | 0 | User-Confirmation (WebAuthn) | 1.0 (keine) | ✅ |
| **AgentManaged** | 1 | Autonom (Software-Key) | 0.8 | ✅ |
| **Ephemeral** | 2 | Autonom (flüchtig) | 0.5 | ❌ |
| **Test** | 3 | Deterministisch (Fake) | 1.0 | ✅ |

### 3.2 Auswirkungen auf Trust

```rust
// Trust-Penalty basierend auf Mode
pub fn trust_penalty_factor(&self) -> f64 {
    match self {
        Interactive => 1.0,      // Vertrauenswürdig: Hardware-bound Keys
        AgentManaged => 0.8,     // Weniger vertrauenswürdig: Software-Keys
        Ephemeral => 0.5,        // Temporär: keine Historie
        Test => 1.0,             // Für Tests – keine Penalty
    }
}

// Mana/Gas Berechnung mit Mode-Penalty
effective_trust = raw_trust × trust_penalty_factor
max_mana = base_mana × (1 + effective_trust × 100)
```

---

## 4. Wallet-Adressen: Multi-Chain Integration

### 4.1 CAIP-2 Format

```text
chain_id = <namespace>:<reference>

Beispiele:
- eip155:1      → Ethereum Mainnet
- eip155:137    → Polygon
- solana:mainnet → Solana
- cosmos:cosmoshub-4 → Cosmos Hub
```

### 4.2 WalletAddress Struktur

```rust
pub struct WalletAddress {
    /// Chain-ID im CAIP-2 Format
    pub chain_id: String,        // z.B. "eip155:1"

    /// Adresse auf der Chain
    pub address: String,         // z.B. "0x..."

    /// BIP44 Derivation-Pfad
    pub derivation_path: String, // z.B. "m/44'/60'/0'/0/0"

    /// Von welcher DID abgeleitet
    pub derived_from: UniversalId,

    /// Erstellungszeitpunkt
    pub created_at: u64,

    /// Primäre Adresse für diese Chain?
    pub is_primary: bool,
}
```

### 4.3 Derivation von DID zu Wallet

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                     DID → WALLET DERIVATION                                 │
│                                                                              │
│   1. Root-DID hat Ed25519 Public Key (32 Bytes)                            │
│                                                                              │
│   2. Für EVM-Chains (eip155):                                              │
│      - Derive ECDSA key (secp256k1) via BIP44                              │
│      - m/44'/60'/0'/0/<index>                                              │
│      - Address = keccak256(publicKey)[12..32]                              │
│                                                                              │
│   3. Für Solana:                                                            │
│      - Derive Ed25519 key via BIP44                                        │
│      - m/44'/501'/0'/0'                                                    │
│      - Address = base58(publicKey)                                         │
│                                                                              │
│   4. Für Cosmos:                                                            │
│      - Derive secp256k1 key via BIP44                                      │
│      - m/44'/118'/0'/0/<index>                                             │
│      - Address = bech32("cosmos", ripemd160(sha256(publicKey)))            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 5. DID-Document: Die Verifikations-Basis

### 5.1 Struktur

```rust
pub struct DIDDocument {
    /// Die DID selbst
    pub id: DID,

    /// Verifikationsmethoden (Keys)
    pub verification_methods: Vec<VerificationMethod>,

    /// Authentifizierungs-Methoden (für Login)
    pub authentication: Vec<UniversalId>,

    /// Assertion-Methoden (für Claims)
    pub assertion_method: Vec<UniversalId>,

    /// Delegationen von dieser DID
    pub delegations: Vec<Delegation>,

    /// Letztes Update
    pub updated_at: TemporalCoord,

    /// Extension Slots (zukunftssicher)
    pub extension_slots: BTreeMap<u16, Vec<u8>>,
}
```

### 5.2 Verifikationsmethoden

```rust
pub struct VerificationMethod {
    /// ID der Methode (UniversalId)
    pub id: UniversalId,

    /// Controller (wer kontrolliert diesen Key)
    pub controller: UniversalId,

    /// Typ: Ed25519, Secp256k1, X25519
    pub method_type: VerificationMethodType,

    /// Öffentlicher Schlüssel (32 Bytes)
    pub public_key: [u8; 32],
}
```

### 5.3 Extension Slots (Zukunftssicherheit)

| Slot ID | Name | Beschreibung |
|---------|------|--------------|
| `0x0001` | RECOVERY_KEYS | Recovery-Keys für Key-Rotation |
| `0x0002` | BIOMETRIC_BINDING | Biometrische Bindung |
| `0x0003` | HARDWARE_ATTESTATION | TEE/TPM Attestation |
| `0x0004` | CROSS_CHAIN_LINKS | Links zu anderen Chains |
| `0x0005` | AI_AGENT_MANIFEST | KI-Agent-Konfiguration |
| `0x0006+` | Custom | Benutzerdefiniert |

---

## 6. Delegation: Trust-Vererbung (Κ8)

### 6.1 Das Delegations-Modell

```text
s ⊳ s' → 𝕋(s') ≤ trust_factor × 𝕋(s)

Wobei:
- s  = Delegator (Quelle)
- s' = Delegate (Ziel)
- 𝕋  = Trust-Funktion
- trust_factor ∈ (0, 1]
```

### 6.2 Delegation Struktur

```rust
pub struct Delegation {
    /// Eindeutige ID
    pub id: UniversalId,

    /// Delegierender (Quelle)
    pub delegator: UniversalId,

    /// Delegierter (Ziel)
    pub delegate: UniversalId,

    /// Trust-Faktor ∈ (0, 1] – Κ8 Trust-Decay
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
```

### 6.3 Capabilities (Fähigkeiten)

| Capability | Format | Beschreibung |
|------------|--------|--------------|
| **All** | `*` | Alle Fähigkeiten (gefährlich!) |
| **Read** | `read:resource` | Lesezugriff auf Ressource |
| **Write** | `write:resource` | Schreibzugriff auf Ressource |
| **Execute** | `execute:action` | Aktion ausführen |
| **Delegate** | `delegate:N` | Weiterdelegieren (max N Tiefe) |
| **Attest** | `attest:type1,type2` | Claims attestieren |
| **Custom** | `custom:name:params` | Benutzerdefiniert |

### 6.4 Ketten-Delegation

```text
Alice (Trust 0.9)
   │
   └─→ Delegation an Bob (trust_factor: 0.8)
        │
        │  Bob's effektiver Trust = 0.9 × 0.8 = 0.72
        │
        └─→ Delegation an Charlie (trust_factor: 0.7)
             │
             │  Charlie's effektiver Trust = 0.72 × 0.7 = 0.504
             │
             └─→ max_depth erreicht (wenn Delegation: delegate:2)
                 → Keine weitere Delegation möglich
```

---

## 7. Realm-Membership: Isolierte Identitäten

### 7.1 Konzept

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                         REALM-ISOLIERUNG                                    │
│                                                                              │
│   Ein User kann in mehreren Realms mit VERSCHIEDENEN Identitäten sein:     │
│                                                                              │
│   Root-DID: did:erynoa:self:abc...                                         │
│                                                                              │
│   ├── Realm "Work"                                                          │
│   │   └── Realm-DID: did:erynoa:circle:work123...                          │
│   │       └── local_trust: 0.9 (Veteran)                                   │
│   │       └── role: Admin                                                  │
│   │                                                                          │
│   ├── Realm "Gaming"                                                        │
│   │   └── Realm-DID: did:erynoa:circle:game456...                          │
│   │       └── local_trust: 0.3 (Newcomer)                                  │
│   │       └── role: Member                                                 │
│   │                                                                          │
│   └── Realm "Finance"                                                       │
│       └── Realm-DID: did:erynoa:circle:fin789...                           │
│       └── local_trust: 0.7 (Active)                                        │
│       └── role: Moderator                                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 RealmMembership Struktur

```rust
pub struct RealmMembership {
    /// Realm-ID
    pub realm_id: UniversalId,

    /// Root-DID des Mitglieds
    pub root_did: UniversalId,

    /// Realm-spezifische Sub-DID (für isolierte Realms)
    pub realm_sub_did: Option<UniversalId>,

    /// Beitrittszeitpunkt
    pub joined_at: TemporalCoord,

    /// Realm-lokaler Trust (kann von Global-Trust abweichen)
    pub local_trust: f64,

    /// Rolle im Realm
    pub role: RealmRole,  // Member, Moderator, Admin, Owner

    /// Aktive Delegationen innerhalb dieses Realms
    pub realm_delegations: Vec<UniversalId>,

    /// Ist die Mitgliedschaft aktiv?
    pub is_active: bool,

    /// Letzter Aktivitätszeitpunkt
    pub last_activity_at: Option<u64>,
}
```

### 7.3 Effektiver Trust mit Rolle

```rust
pub fn effective_trust(&self) -> f64 {
    let role_multiplier = match self.role {
        Member => 1.0,
        Moderator => 1.1,
        Admin => 1.2,
        Owner => 1.3,
    };

    (self.local_trust * role_multiplier).min(1.0)
}
```

---

## 8. P2P-Integration: DID ↔ PeerId ↔ UniversalId

### 8.1 Die Drei-Identifier-Brücke

```text
┌───────────────────────────────────────────────────────────────────────────┐
│                  IDENTITY CONVERSION TRIANGLE                             │
│                                                                           │
│                          DID                                              │
│                    (Erynoa Identity)                                      │
│                   /              \                                        │
│                  /                \                                       │
│                 /                  \                                      │
│                /                    \                                     │
│    PeerId ──────────────────────────── UniversalId                       │
│  (libp2p)                              (Content-Addressed)               │
│                                                                           │
│  All three share the same Ed25519 Public Key as foundation               │
└───────────────────────────────────────────────────────────────────────────┘
```

### 8.2 Konvertierungen

```rust
// DID → PeerId
pub fn did_to_peer_id(did: &DID) -> Result<PeerId> {
    let ed25519_pk = ed25519::PublicKey::try_from_bytes(&did.public_key)?;
    let public_key = PublicKey::from(ed25519_pk);
    Ok(PeerId::from(public_key))
}

// PeerId → DID (mit bekanntem Public Key)
pub fn peer_id_to_did(public_key: &PublicKey) -> Result<DID> {
    let bytes = public_key.try_into_ed25519()?.to_bytes();
    Ok(DID::new_self(&bytes))
}

// DID → UniversalId
// Direkt: did.id ist die UniversalId

// PeerId → UniversalId (über DID)
pub fn peer_id_to_universal_id(public_key: &PublicKey) -> Result<UniversalId> {
    let did = peer_id_to_did(public_key)?;
    Ok(did.id)
}
```

### 8.3 PeerIdentity (Kombination)

```rust
pub struct PeerIdentity {
    /// Erynoa DID
    pub did: DID,

    /// UniversalId (aus DID.id)
    universal_id: UniversalId,

    /// libp2p Keypair (Ed25519)
    keypair: Keypair,

    /// libp2p PeerId
    pub peer_id: PeerId,
}
```

---

## 9. IdentityState: Das State.rs Layer

### 9.1 Hierarchische Struktur

```rust
pub struct IdentityState {
    // ─────────────────────────────────────────────────────────────────────
    // ROOT IDENTITY
    // ─────────────────────────────────────────────────────────────────────
    root_did: RwLock<Option<DID>>,
    root_document: RwLock<Option<DIDDocument>>,

    // ─────────────────────────────────────────────────────────────────────
    // SUB-DIDs
    // ─────────────────────────────────────────────────────────────────────
    current_device_did: RwLock<Option<DID>>,
    sub_dids: RwLock<HashMap<String, Vec<DID>>>,  // "device" → [DID...]
    sub_did_counts: RwLock<HashMap<DIDNamespace, u64>>,

    // ─────────────────────────────────────────────────────────────────────
    // DELEGATION
    // ─────────────────────────────────────────────────────────────────────
    delegations: RwLock<HashMap<UniversalId, Delegation>>,
    active_delegations_count: AtomicU64,
    revoked_delegations_count: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────
    // REALM MEMBERSHIP
    // ─────────────────────────────────────────────────────────────────────
    realm_memberships: RwLock<HashMap<UniversalId, RealmMembership>>,
    realm_memberships_changed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────
    // WALLET ADDRESSES
    // ─────────────────────────────────────────────────────────────────────
    wallets: RwLock<HashMap<String, Vec<WalletAddress>>>,  // chain_id → [wallet...]

    // ─────────────────────────────────────────────────────────────────────
    // MODE & STATUS
    // ─────────────────────────────────────────────────────────────────────
    mode: AtomicU8,  // Interactive, AgentManaged, Ephemeral, Test
    bootstrap_completed: AtomicBool,

    // ─────────────────────────────────────────────────────────────────────
    // KEY MANAGEMENT
    // ─────────────────────────────────────────────────────────────────────
    key_store: Option<SharedKeyStore>,
    passkey_manager: Option<SharedPasskeyManager>,

    // ─────────────────────────────────────────────────────────────────────
    // METRICS
    // ─────────────────────────────────────────────────────────────────────
    gas_consumed: AtomicU64,
    mana_consumed: AtomicU64,
    signatures_created: AtomicU64,
    signatures_verified: AtomicU64,
    events_triggered: AtomicU64,
    trust_entries_created: AtomicU64,
}
```

### 9.2 Bootstrap-Flow

```text
User-Aktion
    │
    ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                          BOOTSTRAP FLOW                                   │
│                                                                           │
│  1. IdentityState.bootstrap_interactive(public_key)                      │
│     oder bootstrap_agent() / bootstrap_ephemeral() / bootstrap_test()    │
│     │                                                                     │
│     ▼                                                                     │
│  2. Erstelle Root-DID                                                    │
│     DID::new_self(public_key)                                            │
│     │                                                                     │
│     ▼                                                                     │
│  3. Erstelle DIDDocument                                                 │
│     DIDDocument::new(did)                                                │
│     │                                                                     │
│     ▼                                                                     │
│  4. Speichere in State                                                   │
│     root_did = Some(did)                                                 │
│     root_document = Some(doc)                                            │
│     mode = Interactive/AgentManaged/...                                  │
│     bootstrap_completed = true                                           │
│     │                                                                     │
│     ▼                                                                     │
│  5. Leite Device-DID ab (optional)                                       │
│     derive_device_did(0)                                                 │
│     │                                                                     │
│     ▼                                                                     │
│  6. Emittiere StateEvent::IdentityBootstrapped                           │
│                                                                           │
└───────────────────────────────────────────────────────────────────────────┘
```

---

## 10. Pluto-Integration: Nervensystem-Mapping

### 10.1 Neue Verzeichnisstruktur

```text
backend/src/nervous_system/
│
├── identity/                        # 🪪 Identity-Layer
│   ├── mod.rs
│   ├── did/
│   │   ├── mod.rs
│   │   ├── namespace.rs             # DIDNamespace (10 Typen)
│   │   ├── did.rs                   # DID Struktur
│   │   ├── document.rs              # DIDDocument
│   │   └── derivation.rs            # Ableitungsmethoden
│   │
│   ├── delegation/
│   │   ├── mod.rs
│   │   ├── delegation.rs            # Delegation Struktur
│   │   ├── capability.rs            # Capability enum
│   │   └── trust_decay.rs           # Κ8 Trust-Decay
│   │
│   ├── realm/
│   │   ├── mod.rs
│   │   ├── membership.rs            # RealmMembership
│   │   └── role.rs                  # RealmRole
│   │
│   ├── wallet/
│   │   ├── mod.rs
│   │   ├── address.rs               # WalletAddress
│   │   ├── chains.rs                # CAIP-2 Definitionen
│   │   └── derivation.rs            # BIP44 Ableitung
│   │
│   ├── key_store/
│   │   ├── mod.rs
│   │   ├── traits.rs                # SecureKeyStore Trait
│   │   ├── tee.rs                   # TEE-Implementierung
│   │   ├── tpm.rs                   # TPM-Implementierung
│   │   ├── software.rs              # Software-Implementierung
│   │   └── test.rs                  # Test-Implementierung
│   │
│   ├── passkey/
│   │   ├── mod.rs
│   │   ├── traits.rs                # PasskeyManager Trait
│   │   └── webauthn.rs              # WebAuthn-Implementierung
│   │
│   ├── mode.rs                      # IdentityMode
│   └── errors.rs                    # IdentityError
│
└── state/
    └── identity.rs                  # IdentityState
```

### 10.2 StateGraph-Relationen

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                   IDENTITY BEZIEHUNGEN IM STATEGRAPH                        │
│                                                                              │
│   Identity ─────────────────────────────────────────────────────────────────│
│     │                                                                        │
│     ├── Triggers ──► Trust         (Identity-Aktionen erzeugen Trust-Δ)    │
│     ├── Triggers ──► Event         (Identity-Änderungen = Events)          │
│     ├── DependsOn ◄─ Gas           (Identity-Ops kosten Gas)               │
│     ├── DependsOn ◄─ Mana          (Identity-Ops kosten Mana)              │
│     │                                                                        │
│     ├── Bidirectional ◄─► Delegation (Delegationen sind Teil von Identity) │
│     ├── Aggregates ◄─────── Wallet   (Wallets gehören zu Identity)         │
│     ├── Aggregates ◄─────── SubDID   (Sub-DIDs gehören zu Identity)        │
│     │                                                                        │
│     └── DependsOn ◄─ Realm         (Realm-Membership braucht Identity)     │
│                                                                              │
│   Delegation ───────────────────────────────────────────────────────────────│
│     │                                                                        │
│     ├── DependsOn ◄─ Trust         (Delegation-Scope durch Trust)          │
│     ├── Triggers ──► Event         (Delegation-Änderungen = Events)        │
│     └── Validates ──► Capability   (Validiert Fähigkeitsberechtigung)      │
│                                                                              │
│   Wallet ───────────────────────────────────────────────────────────────────│
│     │                                                                        │
│     └── DependsOn ◄─ Identity      (Wallet abgeleitet von DID)             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 10.3 StateEvents für Identity

```rust
/// Identity-bezogene StateEvents
pub enum StateEvent {
    // Bootstrap
    IdentityBootstrapped {
        root_did: UniversalId,
        mode: IdentityMode,
        has_device_key: bool,
    },

    // Sub-DID Ableitung
    SubDIDDerived {
        root_did: UniversalId,
        sub_did: UniversalId,
        purpose: String,  // "device", "agent", "realm:xyz"
        namespace: DIDNamespace,
    },

    // Delegation
    DelegationCreated {
        delegator: UniversalId,
        delegate: UniversalId,
        trust_factor: f32,
        capabilities: Vec<String>,
    },
    DelegationRevoked {
        delegation_id: UniversalId,
    },

    // Realm Membership
    RealmJoined {
        root_did: UniversalId,
        realm_id: UniversalId,
        realm_sub_did: Option<UniversalId>,
        role: RealmRole,
    },
    RealmLeft {
        root_did: UniversalId,
        realm_id: UniversalId,
    },

    // Wallet
    WalletAddressAdded {
        did: UniversalId,
        chain_id: String,
        address: String,
    },

    // Credential
    CredentialIssued {
        issuer: UniversalId,
        subject: UniversalId,
        claim_type: String,
    },
    CredentialVerified {
        verifier: UniversalId,
        credential_id: UniversalId,
        valid: bool,
    },
}
```

---

## 11. Axiom-Mapping

| Axiom | Beschreibung | Implementierung |
|-------|--------------|-----------------|
| **Κ6** | Existenz-Eindeutigkeit: ∀ entity e : ∃! did | `DID::new()` mit Content-Addressing |
| **Κ7** | Permanenz: Einmal erstellt = unveränderlich | `UniversalId` ist immutable |
| **Κ8** | Delegations-Struktur: 𝕋(s') ≤ trust_factor × 𝕋(s) | `Delegation.trust_factor` |
| **Κ2** | Trust ∈ [0, 1] | `local_trust.clamp(0.0, 1.0)` |
| **Κ4** | Asymmetrische Evolution | `IdentityMode.trust_penalty_factor()` |
| **Κ24** | Realm-Crossing Dämpfung | `RealmMembership.local_trust` |

---

## 12. Zusammenfassung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ERYNOA IDENTITY ARCHITEKTUR                               ║
║                                                                              ║
║   📌 Ein User hat EINE Root-DID (did:erynoa:self:...)                       ║
║   📌 Daraus werden VIELE Sub-DIDs abgeleitet:                               ║
║      → Device-DIDs (Self_) für Geräte                                       ║
║      → Agent-DIDs (Spirit) für KI-Bots                                      ║
║      → Realm-DIDs (Circle) für isolierte Gruppen                            ║
║      → Vault-DIDs (Vault), Pact-DIDs (Pact), ...                            ║
║                                                                              ║
║   📌 Von jeder DID werden WALLET-ADRESSEN abgeleitet:                       ║
║      → ETH (eip155:1)                                                       ║
║      → Polygon (eip155:137)                                                 ║
║      → Solana, Cosmos, ...                                                  ║
║                                                                              ║
║   📌 DIDs können DELEGATIONEN haben:                                        ║
║      → Trust-Decay (Κ8): Delegate hat max trust_factor × Delegator-Trust   ║
║      → Capability-basiert: read, write, execute, delegate, attest          ║
║                                                                              ║
║   📌 Realm-Memberships sind ISOLIERT:                                       ║
║      → Lokaler Trust pro Realm                                              ║
║      → Eigene Realm-Sub-DID (optional)                                      ║
║      → Realm-spezifische Delegationen                                       ║
║                                                                              ║
║   📌 Alles ist CONTENT-ADDRESSED:                                           ║
║      → UniversalId = BLAKE3(Namespace + PublicKey)                          ║
║      → Deterministisch ableitbar                                            ║
║      → Kein zentrales Registry nötig                                        ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```
