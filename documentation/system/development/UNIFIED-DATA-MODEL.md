# Unified Data Model (UDM) – Erynoa Datenarchitektur

> **Version:** 1.0.0
> **Datum:** Februar 2026
> **Status:** Architectural Specification
> **Basis:** IPS-01-imp.md (Mathematisches Logik-Modell)
> **Ziel:** Zukunftssichere, optimierte Datenstrukturen für alle Erynoa-Subsysteme

---

## Executive Summary

Dieses Dokument definiert die **unifizierte Datenarchitektur** für Erynoa, basierend auf dem
mathematischen IPS-Modell. Die Strukturen sind auf folgende Ziele optimiert:

1. **Zukunftssicherheit**: Versionierte Schemas, Forward-/Backward-Kompatibilität
2. **Performance**: Zero-Copy wo möglich, Cache-freundliche Layouts
3. **Konsistenz**: Einheitliche Patterns über alle Subsysteme
4. **Erweiterbarkeit**: Plugin-fähig, Schema-Evolution ohne Breaking Changes
5. **Beweisbarkeit**: Mathematische Invarianten aus IPS-Modell prüfbar

---

## I. Kern-Primitive (Foundation Layer)

### 1.1 Universeller Identifikator

```
╔════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                        ║
║   DESIGN: UniversalId – Content-Addressed mit Type-Tag                                ║
║                                                                                        ║
║   ════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                        ║
║   Layout (32 Bytes):                                                                  ║
║                                                                                        ║
║       ┌──────────┬────────────┬─────────────────────────────────────────┐             ║
║       │ Type Tag │  Version   │            BLAKE3 Hash (28 bytes)       │             ║
║       │ (2 bytes)│  (2 bytes) │                                         │             ║
║       └──────────┴────────────┴─────────────────────────────────────────┘             ║
║                                                                                        ║
║   Type Tags:                                                                          ║
║       0x0001 = DID            0x0010 = Blueprint                                      ║
║       0x0002 = Event          0x0011 = Deployment                                     ║
║       0x0003 = Realm          0x0012 = Rating                                         ║
║       0x0004 = Trust          0x0020 = Message                                        ║
║       0x0005 = Saga           0x0021 = Topic                                          ║
║       0x0006 = Schema         0x0022 = Connection                                     ║
║       0x0007 = Store          0x0030 = Program (ECLVM)                                ║
║       0x0008 = Policy         0x0031 = State                                          ║
║       0x00FF = Custom         (Erweiterbar bis 0xFFFF)                                ║
║                                                                                        ║
║   Vorteile:                                                                           ║
║       • Typ-Information ohne Lookup erkennbar                                         ║
║       • Version erlaubt Schema-Migration                                              ║
║       • 28-byte Hash: 2²²⁴ Kollisionsresistenz (Post-Quantum-sicher)                  ║
║       • Direktes Key-Sorting in LSM-Tree nach Typ                                     ║
║                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════╝
```

```rust
/// Unified Identifier für alle Erynoa-Objekte
///
/// Invariante: id.type_tag() korrespondiert immer mit dem tatsächlichen Objekttyp
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct UniversalId([u8; 32]);

impl UniversalId {
    /// Type Tags (erste 2 Bytes)
    pub const TAG_DID: u16        = 0x0001;
    pub const TAG_EVENT: u16      = 0x0002;
    pub const TAG_REALM: u16      = 0x0003;
    pub const TAG_TRUST: u16      = 0x0004;
    pub const TAG_SAGA: u16       = 0x0005;
    pub const TAG_SCHEMA: u16     = 0x0006;
    pub const TAG_STORE: u16      = 0x0007;
    pub const TAG_POLICY: u16     = 0x0008;
    pub const TAG_BLUEPRINT: u16  = 0x0010;
    pub const TAG_DEPLOYMENT: u16 = 0x0011;
    pub const TAG_RATING: u16     = 0x0012;
    pub const TAG_MESSAGE: u16    = 0x0020;
    pub const TAG_TOPIC: u16      = 0x0021;
    pub const TAG_CONNECTION: u16 = 0x0022;
    pub const TAG_PROGRAM: u16    = 0x0030;
    pub const TAG_STATE: u16      = 0x0031;

    /// Erstelle ID aus Typ, Version und Content-Hash
    pub fn new(type_tag: u16, version: u16, content: &[u8]) -> Self {
        let mut bytes = [0u8; 32];
        bytes[0..2].copy_from_slice(&type_tag.to_be_bytes());
        bytes[2..4].copy_from_slice(&version.to_be_bytes());

        let hash = blake3::hash(content);
        bytes[4..32].copy_from_slice(&hash.as_bytes()[0..28]);

        Self(bytes)
    }

    /// Type Tag extrahieren (O(1))
    #[inline]
    pub fn type_tag(&self) -> u16 {
        u16::from_be_bytes([self.0[0], self.0[1]])
    }

    /// Version extrahieren (O(1))
    #[inline]
    pub fn version(&self) -> u16 {
        u16::from_be_bytes([self.0[2], self.0[3]])
    }

    /// Als Bytes für Storage
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Prefix für Range-Queries (nur Type-Tag)
    pub fn type_prefix(type_tag: u16) -> [u8; 2] {
        type_tag.to_be_bytes()
    }
}
```

### 1.2 Temporale Koordinate

```
╔════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                        ║
║   DESIGN: TemporalCoord – Hybride Zeit mit Kausalordnung                              ║
║                                                                                        ║
║   ════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                        ║
║   Anforderung: Konsistente Ordnung auch bei Clock-Drift                               ║
║                                                                                        ║
║   Layout (16 Bytes):                                                                  ║
║                                                                                        ║
║       ┌─────────────────────┬──────────────────┬─────────────────────┐                ║
║       │   Wall-Clock (8B)   │  Lamport (4B)    │   Node-Hash (4B)    │                ║
║       │   Mikrosekunden     │  Logische Zeit   │   Tie-Breaker       │                ║
║       └─────────────────────┴──────────────────┴─────────────────────┘                ║
║                                                                                        ║
║   Ordnung (Total):                                                                    ║
║       t₁ < t₂ ⟺ (wall₁, lamport₁, node₁) <ₗₑₓ (wall₂, lamport₂, node₂)              ║
║                                                                                        ║
║   Kausale Konsistenz:                                                                 ║
║       happens_before(e₁, e₂) ⟹ coord(e₁) < coord(e₂)                                 ║
║                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════╝
```

```rust
/// Hybride logisch-physische Zeitkoordinate
///
/// Garantiert: happens_before(a, b) ⟹ a.coord < b.coord
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C, packed)]
pub struct TemporalCoord {
    /// Wall-Clock Zeit in Mikrosekunden seit Unix-Epoch
    wall_time: u64,
    /// Lamport-Timestamp für kausale Ordnung
    lamport: u32,
    /// Hash des Node-Identifiers (Tie-Breaker)
    node_hash: u32,
}

impl TemporalCoord {
    /// Erstelle neue Koordinate
    pub fn now(lamport: u32, node_id: &UniversalId) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let wall_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;

        let node_hash = u32::from_be_bytes(
            node_id.as_bytes()[28..32].try_into().unwrap()
        );

        Self { wall_time, lamport, node_hash }
    }

    /// Update Lamport-Clock (bei Event-Empfang)
    pub fn receive_update(&mut self, remote: &TemporalCoord) {
        self.lamport = self.lamport.max(remote.lamport) + 1;
        // Wall-time wird nicht angepasst (Node-lokal)
    }

    /// Tick für lokales Event
    pub fn tick(&mut self) {
        self.lamport += 1;
    }

    /// Als Bytes für Storage-Key
    pub fn to_bytes(&self) -> [u8; 16] {
        let mut bytes = [0u8; 16];
        bytes[0..8].copy_from_slice(&self.wall_time.to_be_bytes());
        bytes[8..12].copy_from_slice(&self.lamport.to_be_bytes());
        bytes[12..16].copy_from_slice(&self.node_hash.to_be_bytes());
        bytes
    }
}
```

---

## II. Identitäts-Schicht (DID & Trust)

### 2.1 Dezentrale Identität (DID)

```
╔════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                        ║
║   DESIGN: DID – Erweiterbar mit Kapabilitäts-Slots                                    ║
║                                                                                        ║
║   ════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                        ║
║   Kern-Struktur:                                                                      ║
║                                                                                        ║
║       ┌────────────────────────────────────────────────────────────────────┐          ║
║       │                          DIDDocument                               │          ║
║       ├────────────────────────────────────────────────────────────────────┤          ║
║       │  id            : UniversalId                     [32 bytes]        │          ║
║       │  namespace     : Namespace (enum)                [1 byte]          │          ║
║       │  controller    : Option<UniversalId>             [33 bytes]        │          ║
║       │  created_at    : TemporalCoord                   [16 bytes]        │          ║
║       │  updated_at    : TemporalCoord                   [16 bytes]        │          ║
║       ├────────────────────────────────────────────────────────────────────┤          ║
║       │  verification_methods: Vec<VerificationMethod>   [variable]        │          ║
║       │  capabilities       : CapabilitySet              [variable]        │          ║
║       │  services           : Vec<ServiceEndpoint>       [variable]        │          ║
║       │  attestations       : Vec<AttestationRef>        [variable]        │          ║
║       │  delegations        : DelegationTree             [variable]        │          ║
║       ├────────────────────────────────────────────────────────────────────┤          ║
║       │  extension_slots    : BTreeMap<u16, Bytes>       [variable]        │          ║
║       └────────────────────────────────────────────────────────────────────┘          ║
║                                                                                        ║
║   Extension Slots (Zukunftssicher):                                                   ║
║       0x0001 = Recovery-Keys                                                          ║
║       0x0002 = Biometric-Binding                                                      ║
║       0x0003 = Hardware-Attestation (TEE, TPM)                                        ║
║       0x0004 = Cross-Chain-Links                                                      ║
║       0x0005 = AI-Agent-Manifest                                                      ║
║       0x0006..0xFFFF = Custom Extensions                                              ║
║                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════╝
```

```rust
/// DID Document mit Erweiterbarkeit
#[derive(Clone, Serialize, Deserialize)]
pub struct DIDDocument {
    // === Kern-Identität ===
    pub id: UniversalId,
    pub namespace: Namespace,
    pub controller: Option<UniversalId>,  // Self-sovereign wenn None
    pub created_at: TemporalCoord,
    pub updated_at: TemporalCoord,

    // === Kryptographische Methoden ===
    pub verification_methods: Vec<VerificationMethod>,

    // === Kapabilitäten (Κ8) ===
    pub capabilities: CapabilitySet,

    // === Service Endpoints ===
    pub services: Vec<ServiceEndpoint>,

    // === Attestationen (Links) ===
    pub attestations: Vec<AttestationRef>,

    // === Delegations-Baum (Κ8: Trust-Decay) ===
    pub delegations: DelegationTree,

    // === Zukunftssichere Extensions ===
    pub extension_slots: BTreeMap<u16, Bytes>,
}

/// Namespace-Typen (erweiterbar)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Namespace {
    Self_ = 0x01,    // Natürliche Personen
    Guild = 0x02,    // Organisationen, DAOs
    Spirit = 0x03,   // KI-Agenten
    Thing = 0x04,    // IoT, physische Assets
    Vessel = 0x05,   // Container, Transport
    Source = 0x06,   // Datenquellen
    Craft = 0x07,    // Dienstleistungen
    Vault = 0x08,    // Speicher
    Pact = 0x09,     // Verträge
    Circle = 0x0A,   // Gruppen
    // Future: 0x0B..0xFF reserviert
}

/// Verifikationsmethode mit Multi-Algo Support
#[derive(Clone, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    pub method_type: KeyType,
    pub controller: UniversalId,
    pub public_key: PublicKeyMaterial,
    pub purpose: Vec<KeyPurpose>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum KeyType {
    Ed25519VerificationKey2020,
    X25519KeyAgreementKey2020,
    EcdsaSecp256k1VerificationKey2019,
    // Post-Quantum Ready:
    Dilithium3VerificationKey2024,
    Kyber1024KeyAgreementKey2024,
    // Future: Custom(u16)
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum KeyPurpose {
    Authentication,
    AssertionMethod,
    KeyAgreement,
    CapabilityInvocation,
    CapabilityDelegation,
}

/// Kapabilitäts-Set mit Bit-Flags für schnelle Prüfung
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    /// Basis-Kapabilitäten (Bit-Flags)
    pub base_flags: u64,
    /// Erweiterte Kapabilitäten (String-basiert)
    pub extended: HashSet<String>,
    /// Realm-spezifische Kapabilitäten
    pub realm_caps: BTreeMap<UniversalId, u64>,
}

impl CapabilitySet {
    // Basis-Capability Flags
    pub const CAP_READ: u64         = 1 << 0;
    pub const CAP_WRITE: u64        = 1 << 1;
    pub const CAP_EXECUTE: u64      = 1 << 2;
    pub const CAP_DELEGATE: u64     = 1 << 3;
    pub const CAP_ADMIN: u64        = 1 << 4;
    pub const CAP_ATTEST: u64       = 1 << 5;
    pub const CAP_GOVERN: u64       = 1 << 6;
    pub const CAP_TRANSFER: u64     = 1 << 7;
    // ... bis zu 64 Basis-Capabilities

    /// Schnelle Capability-Prüfung O(1)
    #[inline]
    pub fn has(&self, cap: u64) -> bool {
        self.base_flags & cap == cap
    }
}

/// Delegations-Baum mit Trust-Decay (Κ8)
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct DelegationTree {
    /// Aktive Delegationen
    pub active: Vec<Delegation>,
    /// Widerrufene (für Audit)
    pub revoked: Vec<(UniversalId, TemporalCoord)>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub id: UniversalId,
    pub delegate: UniversalId,
    pub capabilities: CapabilitySet,
    pub trust_factor: f32,           // Κ8: 𝕋(delegate) ≤ trust_factor × 𝕋(delegator)
    pub created_at: TemporalCoord,
    pub expires_at: Option<TemporalCoord>,
    pub conditions: Vec<DelegationCondition>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum DelegationCondition {
    RealmMembership(UniversalId),
    MinTrust(f32),
    TimeWindow { start: TemporalCoord, end: TemporalCoord },
    UsageLimit { max_uses: u32, current: u32 },
    PolicyCheck(UniversalId),  // ECLVM-Policy zur Laufzeit
}
```

### 2.2 Trust-Vektor (6D)

```
╔════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                        ║
║   DESIGN: TrustRecord – Kompakt mit History-Kompression                               ║
║                                                                                        ║
║   ════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                        ║
║   Speicher-Layout (Optimiert für Cache):                                              ║
║                                                                                        ║
║       ┌────────────────────────────────────────────────────────────────────┐          ║
║       │                          TrustRecord                               │          ║
║       ├────────────────────────────────────────────────────────────────────┤          ║
║       │  subject_id    : UniversalId                     [32 bytes]        │          ║
║       │  updated_at    : TemporalCoord                   [16 bytes]        │          ║
║       │  vector        : TrustVector6D (packed)          [24 bytes]        │  72B     ║
║       ├────────────────────────────────────────────────────────────────────┤  (1 CL)  ║
║       │  confidence    : f32 × 6                         [24 bytes]        │          ║
║       │  sample_count  : u32 × 6                         [24 bytes]        │          ║
║       ├────────────────────────────────────────────────────────────────────┤          ║
║       │  contexts      : ContextMap                      [variable]        │          ║
║       │  history       : CompressedHistory               [variable]        │          ║
║       └────────────────────────────────────────────────────────────────────┘          ║
║                                                                                        ║
║   Context-spezifische Gewichtung:                                                     ║
║       w(Finance) = [0.25, 0.25, 0.15, 0.15, 0.10, 0.10]  (R, I hoch)                 ║
║       w(Social)  = [0.10, 0.15, 0.10, 0.30, 0.25, 0.10]  (P, V hoch)                 ║
║       w(Govern)  = [0.15, 0.20, 0.10, 0.10, 0.10, 0.35]  (Ω hoch)                    ║
║                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════╝
```

```rust
/// 6D Trust-Vektor (Κ2-Κ5)
///
/// Kompaktes Layout: 24 Bytes für den Vektor (4 Bytes pro Dimension)
#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(C, align(8))]
pub struct TrustVector6D {
    /// R - Reliability (Verhaltens-Historie)
    pub r: f32,
    /// I - Integrity (Aussage-Konsistenz)
    pub i: f32,
    /// C - Competence (Fähigkeits-Nachweis)
    pub c: f32,
    /// P - Prestige (Externe Attestation)
    pub p: f32,
    /// V - Vigilance (Anomalie-Erkennung)
    pub v: f32,
    /// Ω - Omega (Axiom-Treue)
    pub omega: f32,
}

impl TrustVector6D {
    /// Newcomer-Werte (Sybil-Schutz)
    pub const NEWCOMER: Self = Self {
        r: 0.1, i: 0.1, c: 0.1, p: 0.1, v: 0.1, omega: 0.1
    };

    /// Default für etablierte Entitäten
    pub const DEFAULT: Self = Self {
        r: 0.5, i: 0.5, c: 0.5, p: 0.5, v: 0.5, omega: 0.5
    };

    /// Gewichtete Norm (Κ3)
    #[inline]
    pub fn weighted_norm(&self, weights: &[f32; 6]) -> f32 {
        let arr = self.to_array();
        let mut sum = 0.0;
        for i in 0..6 {
            sum += weights[i] * arr[i] * arr[i];
        }
        sum.sqrt()
    }

    /// Als Array für SIMD
    #[inline]
    pub fn to_array(&self) -> [f32; 6] {
        [self.r, self.i, self.c, self.p, self.v, self.omega]
    }

    /// Bayesian Update (Κ4: Asymmetrie)
    pub fn update(&mut self, dim: TrustDimension, delta: f32) {
        let current = self.get(dim);
        let asymmetry = dim.asymmetry_factor();

        let new_value = if delta < 0.0 {
            // Negative Updates stärker gewichtet (Κ4)
            (current + delta * asymmetry).clamp(0.0, 1.0)
        } else {
            (current + delta).clamp(0.0, 1.0)
        };

        self.set(dim, new_value);
    }

    /// Probabilistische Kombination (Κ5)
    pub fn combine(&self, other: &Self) -> Self {
        Self {
            r: 1.0 - (1.0 - self.r) * (1.0 - other.r),
            i: 1.0 - (1.0 - self.i) * (1.0 - other.i),
            c: 1.0 - (1.0 - self.c) * (1.0 - other.c),
            p: 1.0 - (1.0 - self.p) * (1.0 - other.p),
            v: 1.0 - (1.0 - self.v) * (1.0 - other.v),
            omega: 1.0 - (1.0 - self.omega) * (1.0 - other.omega),
        }
    }

    #[inline]
    fn get(&self, dim: TrustDimension) -> f32 {
        match dim {
            TrustDimension::Reliability => self.r,
            TrustDimension::Integrity => self.i,
            TrustDimension::Competence => self.c,
            TrustDimension::Prestige => self.p,
            TrustDimension::Vigilance => self.v,
            TrustDimension::Omega => self.omega,
        }
    }

    #[inline]
    fn set(&mut self, dim: TrustDimension, value: f32) {
        match dim {
            TrustDimension::Reliability => self.r = value,
            TrustDimension::Integrity => self.i = value,
            TrustDimension::Competence => self.c = value,
            TrustDimension::Prestige => self.p = value,
            TrustDimension::Vigilance => self.v = value,
            TrustDimension::Omega => self.omega = value,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TrustDimension {
    Reliability = 0,
    Integrity = 1,
    Competence = 2,
    Prestige = 3,
    Vigilance = 4,
    Omega = 5,
}

impl TrustDimension {
    /// Asymmetrie-Faktor (Κ4)
    #[inline]
    pub fn asymmetry_factor(&self) -> f32 {
        match self {
            Self::Reliability | Self::Integrity |
            Self::Competence | Self::Prestige => 1.5,
            Self::Vigilance | Self::Omega => 2.0,
        }
    }
}

/// Vollständiger Trust-Record mit History
#[derive(Clone, Serialize, Deserialize)]
pub struct TrustRecord {
    // === Identifikation ===
    pub subject_id: UniversalId,
    pub updated_at: TemporalCoord,

    // === Aktueller Vektor ===
    pub vector: TrustVector6D,

    // === Bayesian Posterior Confidence ===
    pub confidence: [f32; 6],
    pub sample_count: [u32; 6],

    // === Kontext-spezifische Overrides ===
    pub contexts: BTreeMap<ContextType, TrustVector6D>,

    // === Komprimierte History ===
    pub history: CompressedTrustHistory,
}

/// Kontext-Typen für Trust-Gewichtung
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum ContextType {
    Default = 0x00,
    Finance = 0x01,
    Social = 0x02,
    Governance = 0x03,
    Technical = 0x04,
    Creative = 0x05,
    // Future: 0x06..0xFF
}

impl ContextType {
    /// Standard-Gewichte pro Kontext
    pub fn default_weights(&self) -> [f32; 6] {
        match self {
            Self::Default => [0.17, 0.17, 0.17, 0.17, 0.16, 0.16],
            Self::Finance => [0.25, 0.25, 0.15, 0.15, 0.10, 0.10],
            Self::Social => [0.10, 0.15, 0.10, 0.30, 0.25, 0.10],
            Self::Governance => [0.15, 0.20, 0.10, 0.10, 0.10, 0.35],
            Self::Technical => [0.15, 0.15, 0.35, 0.10, 0.15, 0.10],
            Self::Creative => [0.10, 0.15, 0.25, 0.25, 0.15, 0.10],
        }
    }
}

/// Komprimierte Trust-History mit Retention-Policy
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct CompressedTrustHistory {
    /// Letzte 24h: Volle Auflösung (max 1000 Einträge)
    pub recent: VecDeque<TrustHistoryEntry>,
    /// Letzte 30d: Stündliche Aggregation
    pub hourly: VecDeque<AggregatedTrustEntry>,
    /// Letzte 365d: Tägliche Aggregation
    pub daily: VecDeque<AggregatedTrustEntry>,
    /// Ältere: Monatliche Aggregation (unbegrenzt)
    pub monthly: VecDeque<AggregatedTrustEntry>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TrustHistoryEntry {
    pub timestamp: TemporalCoord,
    pub dimension: TrustDimension,
    pub delta: f32,
    pub reason: TrustUpdateReason,
    pub evidence: Option<UniversalId>,  // Link zum Event
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AggregatedTrustEntry {
    pub period_start: TemporalCoord,
    pub period_end: TemporalCoord,
    pub vector_avg: TrustVector6D,
    pub vector_min: TrustVector6D,
    pub vector_max: TrustVector6D,
    pub update_count: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum TrustUpdateReason {
    DirectInteraction,
    AttestationReceived,
    DelegationCreated,
    DelegationRevoked,
    VouchReceived,
    VouchRevoked,
    PolicyViolation,
    PositiveContribution,
    AnomalyDetected,
    SystemAdjustment,
    // Future: Custom(u16)
}
```

---

## III. Event-DAG (Kausale Ordnung)

### 3.1 Event-Struktur

```
╔════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                        ║
║   DESIGN: Event – DAG-Node mit Finalitäts-Tracking                                    ║
║                                                                                        ║
║   ════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                        ║
║   Event-Struktur (Κ9-Κ12):                                                            ║
║                                                                                        ║
║       ┌────────────────────────────────────────────────────────────────────┐          ║
║       │                            Event                                   │          ║
║       ├────────────────────────────────────────────────────────────────────┤          ║
║       │  id            : UniversalId (TAG_EVENT)         [32 bytes]        │          ║
║       │  creator       : UniversalId (DID)               [32 bytes]        │          ║
║       │  coord         : TemporalCoord                   [16 bytes]        │          ║
║       │  realm         : UniversalId                     [32 bytes]        │          ║
║       ├────────────────────────────────────────────────────────────────────┤          ║
║       │  parents       : SmallVec<[UniversalId; 2]>      [~64 bytes]       │          ║
║       │  payload       : EventPayload                    [variable]        │          ║
║       │  signature     : Signature                       [64 bytes]        │          ║
║       ├────────────────────────────────────────────────────────────────────┤          ║
║       │  finality      : FinalityState                   [variable]        │          ║
║       │  metadata      : EventMetadata                   [variable]        │          ║
║       └────────────────────────────────────────────────────────────────────┘          ║
║                                                                                        ║
║   DAG-Invarianten:                                                                    ║
║       • Jedes Event hat 1-2 Parents (außer Genesis)                                   ║
║       • Parents sind kausal vor diesem Event                                          ║
║       • Signatur vom Creator mit gültiger Verification-Method                         ║
║       • Finality steigt monoton: NASCENT → ... → ETERNAL                              ║
║                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════╝
```

```rust
/// Event im DAG (Κ9-Κ12)
#[derive(Clone, Serialize, Deserialize)]
pub struct Event {
    // === Identifikation ===
    pub id: UniversalId,
    pub creator: UniversalId,
    pub coord: TemporalCoord,
    pub realm: UniversalId,

    // === DAG-Struktur ===
    /// Parent-Events (1-2, außer Genesis)
    /// SmallVec optimiert für den häufigen Fall von 1-2 Parents
    pub parents: SmallVec<[UniversalId; 2]>,

    // === Inhalt ===
    pub payload: EventPayload,

    // === Kryptographie ===
    pub signature: Signature,

    // === Finalität (Κ10) ===
    pub finality: FinalityState,

    // === Extensible Metadata ===
    pub metadata: EventMetadata,
}

impl Event {
    /// Content-Hash für ID-Berechnung
    pub fn compute_id(&self) -> UniversalId {
        let content = self.canonical_bytes();
        UniversalId::new(UniversalId::TAG_EVENT, 1, &content)
    }

    /// Kanonische Serialisierung (für Signatur/Hash)
    pub fn canonical_bytes(&self) -> Vec<u8> {
        // Deterministisch: CBOR in kanonischer Form
        let mut encoder = cbor::Encoder::new();
        encoder.encode_struct_canonical(&CanonicalEvent {
            creator: &self.creator,
            coord: &self.coord,
            realm: &self.realm,
            parents: &self.parents,
            payload: &self.payload,
        });
        encoder.finish()
    }

    /// Prüfe ob Event final genug für kritische Ops
    #[inline]
    pub fn is_critical_ready(&self) -> bool {
        self.finality.level >= FinalityLevel::Witnessed
    }
}

/// Event-Payload (typisiert)
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventPayload {
    // === Identität ===
    Genesis {
        did: UniversalId,
        public_keys: Vec<VerificationMethod>
    },
    DIDUpdate {
        changes: Vec<DIDChange>
    },

    // === Trust ===
    Attestation {
        subject: UniversalId,
        claim_type: ClaimType,
        claim_data: Bytes,
        evidence: Option<Bytes>,
    },
    VouchGrant {
        beneficiary: UniversalId,
        stake_ratio: f32,
        duration_days: u32,
    },
    VouchRevoke {
        vouch_id: UniversalId,
        reason: String,
    },

    // === Wert-Transfer ===
    Transfer {
        from: UniversalId,
        to: UniversalId,
        amount: u128,
        asset: AssetId,
        memo: Option<String>,
    },
    Mint {
        to: UniversalId,
        amount: u128,
        asset: AssetId,
    },
    Burn {
        from: UniversalId,
        amount: u128,
        asset: AssetId,
    },

    // === Delegation (Κ8) ===
    DelegationGrant {
        delegate: UniversalId,
        capabilities: CapabilitySet,
        trust_factor: f32,
        expires_at: Option<TemporalCoord>,
    },
    DelegationRevoke {
        delegation_id: UniversalId,
    },

    // === Governance (Κ21) ===
    ProposalCreate {
        title: String,
        description: String,
        proposal_type: ProposalType,
        voting_params: VotingParams,
    },
    Vote {
        proposal_id: UniversalId,
        choice: VoteChoice,
        weight: u64,  // Quadratisch: actual_votes = sqrt(weight)
    },
    ProposalExecute {
        proposal_id: UniversalId,
        execution_proof: Bytes,
    },

    // === Saga (Κ22-Κ24) ===
    SagaStart {
        intent: Intent,
        steps: Vec<SagaStepDef>,
    },
    SagaStep {
        saga_id: UniversalId,
        step_index: u32,
        result: SagaStepResult,
    },
    SagaComplete {
        saga_id: UniversalId,
        final_state: SagaFinalState,
    },
    SagaCompensate {
        saga_id: UniversalId,
        from_step: u32,
        reason: String,
    },

    // === Blueprint ===
    BlueprintPublish {
        blueprint_id: UniversalId,
        manifest_hash: [u8; 32],
    },
    BlueprintDeploy {
        blueprint_id: UniversalId,
        realm: UniversalId,
        config: Bytes,
    },
    BlueprintRate {
        blueprint_id: UniversalId,
        rating: u8,  // 1-5
        review: Option<String>,
    },

    // === Storage ===
    StoreCreate {
        store_id: UniversalId,
        schema: StoreSchema,
    },
    SchemaEvolve {
        store_id: UniversalId,
        changes: Vec<SchemaChange>,
    },

    // === Extensible ===
    Custom {
        type_id: u16,
        version: u16,
        data: Bytes,
    },
}

/// Finalitäts-Zustand (Κ10)
#[derive(Clone, Serialize, Deserialize)]
pub struct FinalityState {
    pub level: FinalityLevel,
    pub witnesses: Vec<WitnessAttestation>,
    pub anchor: Option<AnchorProof>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum FinalityLevel {
    /// Neu erstellt, noch nicht validiert (P = 0.5)
    Nascent = 0,
    /// Signatur gültig, Parents existieren (P = 0.9)
    Validated = 1,
    /// Von n Witnesses bestätigt (P = 0.99)
    Witnessed = 2,
    /// In externem System verankert (P = 0.999)
    Anchored = 3,
    /// Irreversibel (P ≈ 1 - 10⁻⁵⁰)
    Eternal = 4,
}

impl FinalityLevel {
    /// Reversion-Wahrscheinlichkeit
    pub fn reversion_probability(&self) -> f64 {
        match self {
            Self::Nascent => 0.5,
            Self::Validated => 0.1,
            Self::Witnessed => 0.01,
            Self::Anchored => 0.001,
            Self::Eternal => 1e-50,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WitnessAttestation {
    pub witness: UniversalId,
    pub timestamp: TemporalCoord,
    pub signature: Signature,
    pub trust_at_time: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AnchorProof {
    pub anchor_type: AnchorType,
    pub block_ref: String,
    pub merkle_proof: Vec<[u8; 32]>,
    pub timestamp: TemporalCoord,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AnchorType {
    Ethereum,
    Bitcoin,
    Solana,
    // Future: Custom(String)
}

/// Extensible Event-Metadata
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Gas verbraucht (wenn Policy ausgeführt)
    pub gas_used: Option<u64>,
    /// Mana verbraucht
    pub mana_used: Option<u64>,
    /// P2P-Propagation Info
    pub propagation: Option<PropagationInfo>,
    /// Custom Extensions
    pub extensions: BTreeMap<u16, Bytes>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PropagationInfo {
    pub first_seen: TemporalCoord,
    pub peers_received_from: Vec<UniversalId>,
    pub hop_count: u8,
}
```

---

## IV. Realm-Hierarchie (Governance)

### 4.1 Realm-Struktur

```
╔════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                        ║
║   DESIGN: Realm – Hierarchisch mit Policy-Composition                                 ║
║                                                                                        ║
║   ════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                        ║
║   Hierarchie (Κ1: Monotone Regelvererbung):                                           ║
║                                                                                        ║
║       RootRealm (28 Kern-Axiome)                                                      ║
║           │                                                                           ║
║           ├── VirtualRealm "EU" (+ GDPR, MiCA)                                        ║
║           │       │                                                                   ║
║           │       ├── Partition "DE-Finance" (+ BaFin)                                ║
║           │       │                                                                   ║
║           │       └── Partition "FR-Social"                                           ║
║           │                                                                           ║
║           └── VirtualRealm "Global" (minimal)                                         ║
║                   │                                                                   ║
║                   └── Partition "Gaming"                                              ║
║                                                                                        ║
║   Invariante: rules(child) ⊇ rules(parent)                                            ║
║                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════╝
```

```rust
/// Realm-Definition
#[derive(Clone, Serialize, Deserialize)]
pub struct Realm {
    // === Identifikation ===
    pub id: UniversalId,
    pub name: String,
    pub realm_type: RealmType,
    pub created_at: TemporalCoord,

    // === Hierarchie ===
    pub parent: Option<UniversalId>,
    pub depth: u8,  // 0 = Root

    // === Regelset (Κ1) ===
    pub rules: RuleSet,

    // === Governance ===
    pub governance: GovernanceConfig,

    // === Mitgliedschaft ===
    pub membership: MembershipConfig,

    // === Storage-Templates ===
    pub storage_templates: Vec<StoreTemplate>,

    // === Policies ===
    pub policies: Vec<PolicyBinding>,

    // === Metadata ===
    pub metadata: RealmMetadata,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RealmType {
    Root = 0,
    Virtual = 1,
    Partition = 2,
}

/// Regelset mit Vererbungs-Tracking
#[derive(Clone, Serialize, Deserialize)]
pub struct RuleSet {
    /// Aktive Regeln
    pub rules: BTreeMap<RuleId, Rule>,
    /// Inherited von Parent (Cache)
    pub inherited: HashSet<RuleId>,
    /// Lokal hinzugefügt
    pub local: HashSet<RuleId>,
}

pub type RuleId = String;

#[derive(Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: RuleId,
    pub name: String,
    pub category: RuleCategory,
    pub description: String,
    pub policy: Option<UniversalId>,  // ECLVM-Policy
    pub mandatory: bool,
    pub effective_from: TemporalCoord,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum RuleCategory {
    Compliance = 0,   // GDPR, MiCA, etc.
    Governance = 1,   // Abstimmungsregeln
    Trust = 2,        // Trust-Thresholds
    Economic = 3,     // Fees, Limits
    Technical = 4,    // Performance, Sicherheit
    Axiom = 5,        // Kern-Axiome (Κ1-Κ28)
}

impl RuleSet {
    /// Prüft Superset-Eigenschaft (Κ1)
    pub fn is_valid_child_of(&self, parent: &RuleSet) -> bool {
        parent.rules.keys().all(|id| self.rules.contains_key(id))
    }

    /// Füge lokale Regel hinzu
    pub fn add_local_rule(&mut self, rule: Rule) {
        let id = rule.id.clone();
        self.rules.insert(id.clone(), rule);
        self.local.insert(id);
    }
}

/// Governance-Konfiguration
#[derive(Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Abstimmungs-Mechanismus
    pub voting: VotingMechanism,
    /// Quorum-Anforderung
    pub quorum: QuorumRequirement,
    /// Proposer-Anforderungen
    pub proposer_requirements: ProposerRequirements,
    /// Execution-Delay
    pub execution_delay_hours: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum VotingMechanism {
    /// Κ21: Quadratisch
    Quadratic { cap_per_voter: u64 },
    /// Einfache Token-gewichtete Abstimmung
    TokenWeighted,
    /// Holokratisch (Rollen-basiert)
    Holacratic { role_weights: BTreeMap<String, f32> },
    /// Futarchie (Prediction-Markets)
    Futarchy { market_duration_hours: u32 },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct QuorumRequirement {
    /// Minimum Beteiligung (Prozent)
    pub min_participation: f32,
    /// Minimum Zustimmung (Prozent)
    pub min_approval: f32,
    /// Minimum Trust-gewichtete Zustimmung
    pub min_trust_weighted_approval: f32,
}

/// Mitgliedschafts-Konfiguration
#[derive(Clone, Serialize, Deserialize)]
pub struct MembershipConfig {
    /// Beitritts-Modus
    pub join_mode: JoinMode,
    /// Minimum Trust für Beitritt
    pub min_trust: f32,
    /// Minimum Trust-Dimension (z.B. Ω für Governance)
    pub min_trust_dimension: Option<(TrustDimension, f32)>,
    /// Maximale Mitglieder (None = unbegrenzt)
    pub max_members: Option<u64>,
    /// Auto-Kick bei Trust-Verfall
    pub auto_remove_threshold: Option<f32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum JoinMode {
    /// Offen für alle mit min_trust
    Open,
    /// Einladung erforderlich
    InviteOnly { min_vouches: u32 },
    /// Antrag mit Governance-Abstimmung
    ApplicationRequired,
    /// Automatisch durch Attestation
    AttestationBased { required_claims: Vec<ClaimType> },
}

/// Store-Template für automatische Einrichtung
#[derive(Clone, Serialize, Deserialize)]
pub struct StoreTemplate {
    pub name: String,
    pub store_type: StoreType,
    pub schema: StoreSchema,
    pub default_permissions: PermissionSet,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum StoreType {
    /// Realm-weiter geteilter Store
    Shared = 0,
    /// Pro-Mitglied isoliert
    Personal = 1,
    /// Pro-Gruppe (Circle)
    Group = 2,
}

/// Policy-Binding an Realm
#[derive(Clone, Serialize, Deserialize)]
pub struct PolicyBinding {
    pub policy_id: UniversalId,
    pub trigger: PolicyTrigger,
    pub priority: u8,  // Höher = früher ausgeführt
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PolicyTrigger {
    /// Bei jedem Event
    AllEvents,
    /// Bei bestimmten Event-Typen
    EventTypes(Vec<String>),
    /// Bei Membership-Änderungen
    MembershipChange,
    /// Bei Gateway-Crossing
    GatewayGuard,
    /// Custom (ECLVM-Bedingung)
    Custom(UniversalId),
}
```

---

## V. Kosten-Algebra (Unified Metering)

### 5.1 Resource-Kosten

```
╔════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                        ║
║   DESIGN: Unified Cost – Gas × Mana × Trust-Risk                                      ║
║                                                                                        ║
║   ════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                        ║
║   Kosten-Algebra 𝒦 aus IPS-Modell:                                                    ║
║                                                                                        ║
║       κ = (gas, mana, trust_risk)                                                     ║
║                                                                                        ║
║       Sequentiell: κ₁ ⊕ κ₂ = (g₁+g₂, m₁+m₂, 1-(1-t₁)(1-t₂))                          ║
║       Parallel:    κ₁ ⊗ κ₂ = (max(g₁,g₂), m₁+m₂, max(t₁,t₂))                         ║
║                                                                                        ║
║   Subsystem-Zuordnung:                                                                ║
║       Gas   → ECLVM (Computation)                                                     ║
║       Mana  → Storage + P2P (Resources)                                               ║
║       Trust → Risiko bei Erynoa-Interaktion                                           ║
║                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════╝
```

```rust
/// Unified Cost für alle Operationen
///
/// Implementiert die Kosten-Algebra 𝒦 aus IPS-Modell
#[derive(Clone, Copy, Default, Serialize, Deserialize)]
#[repr(C)]
pub struct Cost {
    /// Gas für Computation (ECLVM)
    pub gas: u64,
    /// Mana für Storage/Network
    pub mana: u64,
    /// Trust-Risk ∈ [0, 1]
    pub trust_risk: f32,
}

impl Cost {
    pub const ZERO: Self = Self { gas: 0, mana: 0, trust_risk: 0.0 };
    pub const BLOCKING: Self = Self { gas: u64::MAX, mana: u64::MAX, trust_risk: 1.0 };

    /// Sequentielle Komposition (⊕)
    #[inline]
    pub fn seq(self, other: Self) -> Self {
        Self {
            gas: self.gas.saturating_add(other.gas),
            mana: self.mana.saturating_add(other.mana),
            trust_risk: 1.0 - (1.0 - self.trust_risk) * (1.0 - other.trust_risk),
        }
    }

    /// Parallele Komposition (⊗)
    #[inline]
    pub fn par(self, other: Self) -> Self {
        Self {
            gas: self.gas.max(other.gas),
            mana: self.mana.saturating_add(other.mana),
            trust_risk: self.trust_risk.max(other.trust_risk),
        }
    }

    /// Skalierung mit Faktor
    pub fn scale(self, factor: f32) -> Self {
        Self {
            gas: (self.gas as f32 * factor) as u64,
            mana: (self.mana as f32 * factor) as u64,
            trust_risk: (self.trust_risk * factor).clamp(0.0, 1.0),
        }
    }
}

/// Budget für Intent/Saga-Ausführung
#[derive(Clone, Serialize, Deserialize)]
pub struct Budget {
    /// Maximale Kosten
    pub max_cost: Cost,
    /// Bereits verbraucht
    pub spent: Cost,
    /// Asset für Bezahlung
    pub payment_asset: AssetId,
    /// Reservierter Betrag
    pub reserved_amount: u128,
}

impl Budget {
    /// Prüfe ob Operation bezahlbar
    pub fn can_afford(&self, cost: &Cost) -> bool {
        let remaining = self.remaining();
        cost.gas <= remaining.gas
            && cost.mana <= remaining.mana
            && cost.trust_risk <= remaining.trust_risk
    }

    /// Verbleibend
    pub fn remaining(&self) -> Cost {
        Cost {
            gas: self.max_cost.gas.saturating_sub(self.spent.gas),
            mana: self.max_cost.mana.saturating_sub(self.spent.mana),
            trust_risk: (self.max_cost.trust_risk - self.spent.trust_risk).max(0.0),
        }
    }

    /// Verbrauche Kosten
    pub fn consume(&mut self, cost: Cost) -> Result<(), BudgetExhausted> {
        if !self.can_afford(&cost) {
            return Err(BudgetExhausted);
        }
        self.spent = self.spent.seq(cost);
        Ok(())
    }
}

/// Kosten-Tabelle pro Operation
pub struct CostTable {
    // ECLVM OpCodes
    pub vm_push_const: Cost,
    pub vm_add: Cost,
    pub vm_call_base: Cost,
    pub vm_call_per_arg: Cost,
    pub vm_host_call: Cost,

    // Storage
    pub storage_get: Cost,
    pub storage_put_base: Cost,
    pub storage_put_per_kb: Cost,
    pub storage_query_base: Cost,
    pub storage_query_per_result: Cost,

    // P2P
    pub p2p_publish: Cost,
    pub p2p_sync_request: Cost,
    pub p2p_connect: Cost,

    // Blueprint
    pub blueprint_upload_base: Cost,
    pub blueprint_upload_per_kb: Cost,
    pub blueprint_deploy: Cost,
    pub blueprint_rate: Cost,
}

impl Default for CostTable {
    fn default() -> Self {
        Self {
            // ECLVM
            vm_push_const: Cost { gas: 1, mana: 0, trust_risk: 0.0 },
            vm_add: Cost { gas: 2, mana: 0, trust_risk: 0.0 },
            vm_call_base: Cost { gas: 10, mana: 0, trust_risk: 0.0 },
            vm_call_per_arg: Cost { gas: 2, mana: 0, trust_risk: 0.0 },
            vm_host_call: Cost { gas: 50, mana: 10, trust_risk: 0.1 },

            // Storage
            storage_get: Cost { gas: 0, mana: 5, trust_risk: 0.0 },
            storage_put_base: Cost { gas: 0, mana: 10, trust_risk: 0.0 },
            storage_put_per_kb: Cost { gas: 0, mana: 10, trust_risk: 0.0 },
            storage_query_base: Cost { gas: 0, mana: 20, trust_risk: 0.0 },
            storage_query_per_result: Cost { gas: 0, mana: 5, trust_risk: 0.0 },

            // P2P
            p2p_publish: Cost { gas: 0, mana: 10, trust_risk: 0.01 },
            p2p_sync_request: Cost { gas: 0, mana: 5, trust_risk: 0.05 },
            p2p_connect: Cost { gas: 0, mana: 20, trust_risk: 0.1 },

            // Blueprint
            blueprint_upload_base: Cost { gas: 0, mana: 500, trust_risk: 0.05 },
            blueprint_upload_per_kb: Cost { gas: 0, mana: 20, trust_risk: 0.0 },
            blueprint_deploy: Cost { gas: 0, mana: 50, trust_risk: 0.02 },
            blueprint_rate: Cost { gas: 0, mana: 10, trust_risk: 0.02 },
        }
    }
}

#[derive(Debug)]
pub struct BudgetExhausted;
```

---

## VI. Saga-Orchestration (Κ22-Κ24)

### 6.1 Intent & Saga

```rust
/// Intent repräsentiert eine Nutzer-Absicht (Κ22)
#[derive(Clone, Serialize, Deserialize)]
pub struct Intent {
    pub id: UniversalId,
    pub source: UniversalId,  // DID des Initiators
    pub goal: Goal,
    pub constraints: Vec<Constraint>,
    pub budget: Budget,
    pub context_realm: UniversalId,
    pub created_at: TemporalCoord,
    pub expires_at: TemporalCoord,
}

/// Ziel eines Intents
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Goal {
    // === Atomare Ziele ===
    Transfer {
        to: UniversalId,
        amount: u128,
        asset: AssetId,
    },
    Attest {
        subject: UniversalId,
        claim: ClaimType,
        data: Bytes,
    },
    Delegate {
        to: UniversalId,
        capabilities: CapabilitySet,
        duration_days: u32,
    },
    Query {
        store: UniversalId,
        predicate: QueryPredicate,
    },
    Create {
        entity_type: EntityType,
        params: BTreeMap<String, Value>,
    },

    // === Komponierte Ziele ===
    Sequence(Vec<Goal>),
    Parallel(Vec<Goal>),
    Conditional {
        condition: UniversalId,  // Policy-ID
        then_goal: Box<Goal>,
        else_goal: Option<Box<Goal>>,
    },

    // === High-Level (NLP-parseable) ===
    Natural {
        description: String,
        parsed: Option<Box<Goal>>,
    },
}

/// Constraint für Intent-Ausführung
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Constraint {
    MinTrust { dimension: Option<TrustDimension>, value: f32 },
    MaxCost { cost: Cost },
    Deadline { at: TemporalCoord },
    RealmRestriction { realms: Vec<UniversalId> },
    HumanOnly,
    GatewayRequired { gateway: UniversalId },
    PolicyMustPass { policy: UniversalId },
}

/// Saga ist die Auflösung eines Intents (Κ22)
#[derive(Clone, Serialize, Deserialize)]
pub struct Saga {
    pub id: UniversalId,
    pub intent_id: UniversalId,
    pub executor: UniversalId,
    pub steps: Vec<SagaStep>,
    pub status: SagaStatus,
    pub created_at: TemporalCoord,
    pub updated_at: TemporalCoord,
    pub timeout_at: TemporalCoord,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SagaStep {
    pub index: u32,
    pub action: SagaAction,
    pub compensation: Option<SagaAction>,
    pub status: StepStatus,
    pub result: Option<StepResult>,
    pub cost_estimate: Cost,
    pub cost_actual: Option<Cost>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SagaAction {
    // === Core Actions ===
    CreateEvent { payload: EventPayload },
    UpdateTrust { subject: UniversalId, updates: Vec<TrustUpdate> },

    // === Storage Actions ===
    StorageRead { store: UniversalId, key: Bytes },
    StorageWrite { store: UniversalId, key: Bytes, value: Bytes },

    // === Policy Execution ===
    ExecutePolicy { policy: UniversalId, input: Bytes },

    // === Cross-Realm (Κ23) ===
    GatewayCross { from_realm: UniversalId, to_realm: UniversalId },

    // === External ===
    ExternalCall { endpoint: String, payload: Bytes },
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SagaStatus {
    Pending = 0,
    InProgress = 1,
    Completed = 2,
    Failed = 3,
    Compensating = 4,
    Compensated = 5,
    TimedOut = 6,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum StepStatus {
    Pending = 0,
    Running = 1,
    Completed = 2,
    Failed = 3,
    Compensated = 4,
    Skipped = 5,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub success: bool,
    pub output: Option<Bytes>,
    pub error: Option<String>,
    pub events_emitted: Vec<UniversalId>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TrustUpdate {
    pub dimension: TrustDimension,
    pub delta: f32,
    pub reason: TrustUpdateReason,
}
```

---

## VII. Storage-Schema (RealmStorage)

### 7.1 Schema-Definition

```rust
/// Store-Schema mit Evolution-Support
#[derive(Clone, Serialize, Deserialize)]
pub struct StoreSchema {
    pub id: UniversalId,
    pub version: u32,
    pub name: String,
    pub fields: Vec<SchemaField>,
    pub indexes: Vec<SchemaIndex>,
    pub constraints: Vec<SchemaConstraint>,
    pub evolution_policy: EvolutionPolicy,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub field_type: FieldType,
    pub nullable: bool,
    pub default: Option<Value>,
    pub description: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum FieldType {
    // === Primitiv ===
    Bool,
    Int32,
    Int64,
    Float32,
    Float64,
    String,
    Bytes,

    // === Temporal ===
    Timestamp,
    Duration,

    // === Erynoa-spezifisch ===
    UniversalId,
    TrustVector,
    TemporalCoord,

    // === Komplex ===
    Array(Box<FieldType>),
    Map { key: Box<FieldType>, value: Box<FieldType> },
    Optional(Box<FieldType>),
    OneOf(Vec<FieldType>),

    // === Referenz ===
    Reference { target_schema: UniversalId },

    // === Custom ===
    Custom { type_id: u16, codec: String },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SchemaIndex {
    pub name: String,
    pub fields: Vec<String>,
    pub unique: bool,
    pub index_type: IndexType,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum IndexType {
    BTree,
    Hash,
    FullText,
    Spatial,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SchemaConstraint {
    PrimaryKey { fields: Vec<String> },
    ForeignKey {
        fields: Vec<String>,
        reference: UniversalId,
        ref_fields: Vec<String>
    },
    Check { expression: String },
    Unique { fields: Vec<String> },
}

/// Evolution-Policy für Schema-Änderungen
#[derive(Clone, Serialize, Deserialize)]
pub struct EvolutionPolicy {
    /// Erlaubte Änderungen ohne Migration
    pub auto_compatible: Vec<ChangeType>,
    /// Migration-Script für Breaking Changes
    pub migration_policy: MigrationPolicy,
    /// Retention für alte Versionen
    pub version_retention: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    AddNullableField,
    AddFieldWithDefault,
    AddIndex,
    RemoveIndex,
    WidenType,  // Int32 → Int64
}

#[derive(Clone, Serialize, Deserialize)]
pub enum MigrationPolicy {
    /// Reject Breaking Changes
    Strict,
    /// Lazy Migration on Read
    LazyMigrate,
    /// Batch Migration Required
    RequireBatchMigration,
}

/// Schema-Change für Evolution
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SchemaChange {
    AddField { field: SchemaField },
    RemoveField { name: String },
    RenameField { old_name: String, new_name: String },
    ChangeType { name: String, new_type: FieldType, migration: Option<String> },
    AddIndex { index: SchemaIndex },
    RemoveIndex { name: String },
    AddConstraint { constraint: SchemaConstraint },
    RemoveConstraint { name: String },
}
```

---

## VIII. Blueprint (Template-System)

### 8.1 Blueprint-Struktur

```rust
/// Blueprint für Realm-Templates
#[derive(Clone, Serialize, Deserialize)]
pub struct Blueprint {
    // === Identifikation ===
    pub id: UniversalId,  // Content-Hash
    pub name: String,
    pub version: SemVer,
    pub author: UniversalId,

    // === Inhalt ===
    pub stores: Vec<BlueprintStore>,
    pub policies: Vec<BlueprintPolicy>,
    pub sagas: Vec<BlueprintSaga>,
    pub rules: Vec<Rule>,

    // === Dependencies ===
    pub dependencies: Vec<BlueprintDependency>,

    // === Metadata ===
    pub category: BlueprintCategory,
    pub license: BlueprintLicense,
    pub description: String,
    pub documentation_url: Option<String>,
    pub tags: Vec<String>,

    // === Timestamps ===
    pub created_at: TemporalCoord,
    pub updated_at: TemporalCoord,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub prerelease: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlueprintStore {
    pub name: String,
    pub store_type: StoreType,
    pub schema: StoreSchema,
    pub default_permissions: PermissionSet,
    pub initial_data: Option<Bytes>,  // CBOR
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlueprintPolicy {
    pub name: String,
    pub policy_type: PolicyType,
    pub trigger: PolicyTrigger,
    pub source: PolicySource,
    pub gas_limit: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum PolicySource {
    /// Inline ECL-Code
    Inline(String),
    /// Kompilierter ECLVM-Bytecode
    Bytecode(Bytes),
    /// Referenz auf existierende Policy
    Reference(UniversalId),
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum PolicyType {
    Gateway = 0,      // Realm-Eintritt
    Validator = 1,    // Event-Validierung
    Trigger = 2,      // Reaktion auf Events
    Query = 3,        // Daten-Zugriff
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlueprintSaga {
    pub name: String,
    pub intent_pattern: Goal,
    pub steps: Vec<SagaStepDef>,
    pub timeout_hours: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SagaStepDef {
    pub action: SagaAction,
    pub compensation: Option<SagaAction>,
    pub retry_policy: RetryPolicy,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u8,
    pub backoff_ms: u32,
    pub backoff_multiplier: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BlueprintDependency {
    pub blueprint_id: UniversalId,
    pub version_constraint: VersionConstraint,
    pub optional: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum VersionConstraint {
    Exact(SemVer),
    Compatible(SemVer),  // ^1.2.3
    Range { min: SemVer, max: SemVer },
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlueprintCategory {
    Finance = 0,
    Social = 1,
    Governance = 2,
    Identity = 3,
    Gaming = 4,
    IoT = 5,
    AI = 6,
    Utility = 7,
    // Future: 8..255
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlueprintLicense {
    MIT = 0,
    Apache2 = 1,
    GPL3 = 2,
    CreativeCommons = 3,
    Proprietary = 4,
    Custom = 255,
}

/// Blueprint-Deployment
#[derive(Clone, Serialize, Deserialize)]
pub struct BlueprintDeployment {
    pub id: UniversalId,
    pub blueprint_id: UniversalId,
    pub blueprint_version: SemVer,
    pub realm: UniversalId,
    pub deployer: UniversalId,
    pub config: DeploymentConfig,
    pub created_at: TemporalCoord,
    pub status: DeploymentStatus,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub store_overrides: BTreeMap<String, StoreOverride>,
    pub policy_overrides: BTreeMap<String, PolicyOverride>,
    pub parameter_values: BTreeMap<String, Value>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StoreOverride {
    pub permissions: Option<PermissionSet>,
    pub initial_data: Option<Bytes>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PolicyOverride {
    pub enabled: bool,
    pub gas_limit: Option<u64>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum DeploymentStatus {
    Pending = 0,
    Deploying = 1,
    Active = 2,
    Paused = 3,
    Deprecated = 4,
    Failed = 5,
}
```

---

## IX. P2P-Nachrichten

### 9.1 Message-Struktur

```rust
/// P2P-Nachricht
#[derive(Clone, Serialize, Deserialize)]
pub struct P2PMessage {
    pub id: UniversalId,
    pub sender: UniversalId,
    pub topic: TopicId,
    pub payload: MessagePayload,
    pub signature: Signature,
    pub timestamp: TemporalCoord,
    pub ttl_hops: u8,
}

pub type TopicId = String;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessagePayload {
    // === Event-Propagation ===
    EventAnnounce {
        event_id: UniversalId,
        event_hash: [u8; 32],
        finality: FinalityLevel,
    },
    EventRequest {
        event_ids: Vec<UniversalId>,
    },
    EventResponse {
        events: Vec<Event>,
    },

    // === Trust-Updates ===
    TrustUpdate {
        subject: UniversalId,
        vector: TrustVector6D,
        evidence: UniversalId,  // Event-ID
    },

    // === Sync-Protokoll ===
    SyncRequest {
        realm: UniversalId,
        since: TemporalCoord,
        merkle_root: Option<[u8; 32]>,
    },
    SyncResponse {
        realm: UniversalId,
        events: Vec<Event>,
        merkle_proof: Option<MerkleProof>,
        has_more: bool,
    },

    // === Witness-Attestation ===
    WitnessRequest {
        event_id: UniversalId,
    },
    WitnessAttestation {
        event_id: UniversalId,
        witness: UniversalId,
        signature: Signature,
    },

    // === Direct ===
    DirectMessage {
        recipient: UniversalId,
        encrypted_payload: Bytes,
        ephemeral_key: [u8; 32],
    },

    // === Discovery ===
    PeerAnnounce {
        capabilities: Vec<PeerCapability>,
        realms: Vec<UniversalId>,
    },
    RealmAnnounce {
        realm: UniversalId,
        member_count: u64,
        trust_threshold: f32,
    },
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum PeerCapability {
    FullNode = 0,
    LightClient = 1,
    Witness = 2,
    Relay = 3,
    Archive = 4,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub root: [u8; 32],
    pub path: Vec<MerkleNode>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub position: MerklePosition,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum MerklePosition {
    Left,
    Right,
}
```

---

## X. Weltformel-Beitrag

### 10.1 Unified Contribution

```rust
/// Vollständiger Weltformel-Beitrag (Κ15a-d)
///
/// 𝔼(s) = 𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)
#[derive(Clone, Serialize, Deserialize)]
pub struct WorldFormulaContribution {
    // === Subject ===
    pub subject: UniversalId,
    pub context: ContextType,
    pub calculated_at: TemporalCoord,

    // === Komponenten ===

    /// Aktivität 𝔸(s) ∈ [0,1]
    pub activity: Activity,

    /// Trust-Vektor 𝕎(s)
    pub trust: TrustVector6D,

    /// Gewichtete Trust-Norm ‖𝕎(s)‖_w
    pub trust_norm: f32,

    /// Kausale Geschichte |ℂ(s)|
    pub causal_history_size: u64,

    /// Trust-gedämpfte Surprisal 𝒮(s)
    pub surprisal: Surprisal,

    /// Human-Factor Ĥ(s)
    pub human_factor: HumanFactor,

    /// Temporale Gewichtung w(s,t)
    pub temporal_weight: f32,

    // === Berechnetes Ergebnis ===
    pub contribution: f64,
}

/// Aktivitäts-Metrik
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Activity {
    /// Events im Zeitfenster
    pub recent_events: u64,
    /// Zeitfenster τ in Sekunden
    pub tau_seconds: u64,
    /// Aktivitäts-Schwelle κ
    pub kappa: u64,
}

impl Activity {
    /// 𝔸(s) = n / (n + κ)
    pub fn value(&self) -> f64 {
        let n = self.recent_events as f64;
        let k = self.kappa as f64;
        n / (n + k)
    }
}

/// Surprisal-Metrik (Κ15a)
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Surprisal {
    /// Raw Shannon-Surprisal ℐ = -log₂ P
    pub raw: f64,
    /// Trust-gedämpft 𝒮 = ‖𝕎‖² · ℐ
    pub dampened: f64,
}

/// Human-Alignment-Faktor
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum HumanFactor {
    NotVerified = 0,        // Ĥ = 1.0
    BasicAttestation = 1,   // Ĥ = 1.2
    FullAttestation = 2,    // Ĥ = 1.5
}

impl HumanFactor {
    pub fn value(&self) -> f64 {
        match self {
            Self::NotVerified => 1.0,
            Self::BasicAttestation => 1.2,
            Self::FullAttestation => 1.5,
        }
    }
}

impl WorldFormulaContribution {
    /// Berechne Beitrag (Κ15b)
    pub fn calculate(&mut self) {
        let a = self.activity.value();
        let trust_contribution = self.trust_norm as f64;
        let history = (self.causal_history_size as f64).ln();
        let s = self.surprisal.dampened;
        let h = self.human_factor.value();
        let w = self.temporal_weight as f64;

        // Inner term für Sigmoid
        let inner = trust_contribution * history * s;

        // Sigmoid σ⃗(x) = 1 / (1 + e^(-x))
        let sigmoid = 1.0 / (1.0 + (-inner).exp());

        // Vollständige Formel
        self.contribution = a * sigmoid * h * w;
    }
}
```

---

## XI. Utility-Typen

### 11.1 Gemeinsame Typen

```rust
/// Asset-Identifikator
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId {
    /// Realm, das Asset definiert
    pub realm: UniversalId,
    /// Asset-Name
    pub name: String,
}

impl AssetId {
    /// Native Erynoa Token
    pub fn ery() -> Self {
        Self {
            realm: UniversalId::new(UniversalId::TAG_REALM, 1, b"root"),
            name: "ERY".to_string(),
        }
    }
}

/// Generischer Value-Typ
#[derive(Clone, Serialize, Deserialize)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Bytes(Bytes),
    Array(Vec<Value>),
    Map(BTreeMap<String, Value>),
    UniversalId(UniversalId),
    TrustVector(TrustVector6D),
    TemporalCoord(TemporalCoord),
}

/// Permission-Set für Storage-Zugriff
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    /// Owner kann alles
    pub owner: UniversalId,
    /// Lese-Berechtigung
    pub readers: AccessControl,
    /// Schreib-Berechtigung
    pub writers: AccessControl,
    /// Admin-Berechtigung
    pub admins: AccessControl,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum AccessControl {
    /// Niemand (außer Owner)
    None,
    /// Nur Owner
    OwnerOnly,
    /// Bestimmte DIDs
    Whitelist(HashSet<UniversalId>),
    /// Alle Realm-Mitglieder
    RealmMembers,
    /// Alle mit Trust ≥ Threshold
    TrustThreshold(f32),
    /// Policy-basiert
    Policy(UniversalId),
    /// Öffentlich
    Public,
}

/// Kryptographische Signatur
#[derive(Clone, Serialize, Deserialize)]
pub struct Signature {
    pub key_id: String,
    pub algorithm: SignatureAlgorithm,
    pub value: Bytes,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    Ed25519,
    EcdsaSecp256k1,
    Dilithium3,  // Post-Quantum
}

/// Claim-Typen für Attestationen
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClaimType {
    // === Standard ===
    HumanVerification,
    AgeOver(u8),
    Residence(String),  // ISO Country Code
    KycLevel(u8),

    // === Professional ===
    Qualification(String),
    Membership(String),
    Employment(String),

    // === Technical ===
    DeviceAttestation,
    TeeVerification,

    // === Custom ===
    Custom(String),
}

/// Query-Prädikat für Storage
#[derive(Clone, Serialize, Deserialize)]
pub enum QueryPredicate {
    Equals { field: String, value: Value },
    Range { field: String, min: Option<Value>, max: Option<Value> },
    Contains { field: String, value: Value },
    And(Vec<QueryPredicate>),
    Or(Vec<QueryPredicate>),
    Not(Box<QueryPredicate>),
}

/// Entity-Typen
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum EntityType {
    DID,
    Event,
    Realm,
    Store,
    Schema,
    Blueprint,
    Saga,
    Policy,
}
```

---

## XII. Storage-Prefixes (Fjall-DB)

### 12.1 Prefix-Schema

```
╔════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                        ║
║   PREFIX-SCHEMA für Fjall-DB (LSM-Tree optimiert)                                     ║
║                                                                                        ║
║   ════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                        ║
║   Prefix-Struktur (hierarchisch, sortierbar):                                         ║
║                                                                                        ║
║       [type:2][realm:32][store:16][key:variable]                                      ║
║                                                                                        ║
║   Beispiele:                                                                          ║
║       01 | {realm_id} | {store_name_hash} | {user_key}        → Storage Data          ║
║       02 | {realm_id} | 00..00            | {event_id}        → Event Index           ║
║       03 | {realm_id} | 00..00            | {did_hash}        → Trust Data            ║
║       04 | 00..00     | 00..00            | {blueprint_id}    → Blueprint Store       ║
║       05 | {realm_id} | 00..00            | {saga_id}         → Saga State            ║
║                                                                                        ║
║   Range-Queries:                                                                      ║
║       • Alle Daten eines Realms: prefix = [type][realm_id]                            ║
║       • Alle Events eines Realms: prefix = [02][realm_id]                             ║
║       • Alle Stores eines Realms: prefix = [01][realm_id]                             ║
║                                                                                        ║
╚════════════════════════════════════════════════════════════════════════════════════════╝
```

```rust
/// Storage-Prefix-Builder
pub struct PrefixBuilder;

impl PrefixBuilder {
    // Prefix-Type-Tags
    const TYPE_STORE: u8 = 0x01;
    const TYPE_EVENT: u8 = 0x02;
    const TYPE_TRUST: u8 = 0x03;
    const TYPE_BLUEPRINT: u8 = 0x04;
    const TYPE_SAGA: u8 = 0x05;
    const TYPE_SCHEMA: u8 = 0x06;
    const TYPE_INDEX: u8 = 0x07;

    /// Build Storage-Key
    pub fn store_key(
        realm: &UniversalId,
        store_name: &str,
        key: &[u8]
    ) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(2 + 32 + 16 + key.len());
        prefix.push(Self::TYPE_STORE);
        prefix.push(0x00);  // Reserved
        prefix.extend_from_slice(realm.as_bytes());

        // Store-Name als 16-byte Hash
        let name_hash = blake3::hash(store_name.as_bytes());
        prefix.extend_from_slice(&name_hash.as_bytes()[0..16]);

        prefix.extend_from_slice(key);
        prefix
    }

    /// Build Event-Index-Key
    pub fn event_key(realm: &UniversalId, event_id: &UniversalId) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(2 + 32 + 16 + 32);
        prefix.push(Self::TYPE_EVENT);
        prefix.push(0x00);
        prefix.extend_from_slice(realm.as_bytes());
        prefix.extend_from_slice(&[0u8; 16]);  // Placeholder
        prefix.extend_from_slice(event_id.as_bytes());
        prefix
    }

    /// Build Trust-Key
    pub fn trust_key(realm: &UniversalId, subject: &UniversalId) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(2 + 32 + 16 + 32);
        prefix.push(Self::TYPE_TRUST);
        prefix.push(0x00);
        prefix.extend_from_slice(realm.as_bytes());
        prefix.extend_from_slice(&[0u8; 16]);
        prefix.extend_from_slice(subject.as_bytes());
        prefix
    }

    /// Build Blueprint-Key (Global)
    pub fn blueprint_key(blueprint_id: &UniversalId) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(2 + 32 + 16 + 32);
        prefix.push(Self::TYPE_BLUEPRINT);
        prefix.push(0x00);
        prefix.extend_from_slice(&[0u8; 32]);  // Global (no realm)
        prefix.extend_from_slice(&[0u8; 16]);
        prefix.extend_from_slice(blueprint_id.as_bytes());
        prefix
    }

    /// Build Saga-Key
    pub fn saga_key(realm: &UniversalId, saga_id: &UniversalId) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(2 + 32 + 16 + 32);
        prefix.push(Self::TYPE_SAGA);
        prefix.push(0x00);
        prefix.extend_from_slice(realm.as_bytes());
        prefix.extend_from_slice(&[0u8; 16]);
        prefix.extend_from_slice(saga_id.as_bytes());
        prefix
    }

    /// Range für alle Entries eines Typs in einem Realm
    pub fn realm_range(type_tag: u8, realm: &UniversalId) -> (Vec<u8>, Vec<u8>) {
        let mut start = Vec::with_capacity(2 + 32);
        start.push(type_tag);
        start.push(0x00);
        start.extend_from_slice(realm.as_bytes());

        let mut end = start.clone();
        // Increment last byte for exclusive end
        if let Some(last) = end.last_mut() {
            *last = last.wrapping_add(1);
        }

        (start, end)
    }
}
```

---

## XIII. Versionierung & Migration

### 13.1 Schema-Registry

```rust
/// Schema-Registry für Versionierung
pub struct SchemaRegistry {
    /// Aktuelle Versionen pro Typ
    current_versions: BTreeMap<u16, u16>,
    /// Migrations-Funktionen
    migrations: BTreeMap<(u16, u16, u16), MigrationFn>,
}

type MigrationFn = Box<dyn Fn(&[u8]) -> Result<Vec<u8>, MigrationError> + Send + Sync>;

impl SchemaRegistry {
    /// Registriere aktuellen Schema-Version
    pub fn register_current(&mut self, type_tag: u16, version: u16) {
        self.current_versions.insert(type_tag, version);
    }

    /// Registriere Migration
    pub fn register_migration(
        &mut self,
        type_tag: u16,
        from_version: u16,
        to_version: u16,
        migration: MigrationFn,
    ) {
        self.migrations.insert((type_tag, from_version, to_version), migration);
    }

    /// Migriere Daten wenn nötig
    pub fn maybe_migrate(&self, id: &UniversalId, data: &[u8]) -> Result<Vec<u8>, MigrationError> {
        let type_tag = id.type_tag();
        let data_version = id.version();

        let current = self.current_versions.get(&type_tag)
            .ok_or(MigrationError::UnknownType)?;

        if data_version == *current {
            return Ok(data.to_vec());
        }

        // Finde Migration-Pfad
        let mut version = data_version;
        let mut result = data.to_vec();

        while version < *current {
            let next_version = version + 1;
            let migration = self.migrations.get(&(type_tag, version, next_version))
                .ok_or(MigrationError::NoMigrationPath)?;

            result = migration(&result)?;
            version = next_version;
        }

        Ok(result)
    }
}

#[derive(Debug)]
pub enum MigrationError {
    UnknownType,
    NoMigrationPath,
    MigrationFailed(String),
}
```

---

## XIV. Invarianten & Prüfungen

### 14.1 Compile-Time Invarianten

```rust
// Größen-Garantien (für Cache-Effizienz)
const _: () = {
    assert!(std::mem::size_of::<UniversalId>() == 32);
    assert!(std::mem::size_of::<TemporalCoord>() == 16);
    assert!(std::mem::size_of::<TrustVector6D>() == 24);
    assert!(std::mem::size_of::<Cost>() == 20);  // 8 + 8 + 4
};

// Alignment-Garantien
const _: () = {
    assert!(std::mem::align_of::<TrustVector6D>() >= 4);
    assert!(std::mem::align_of::<TemporalCoord>() >= 4);
};
```

### 14.2 Runtime-Invarianten

```rust
/// Invarianten-Prüfer
pub struct InvariantChecker;

impl InvariantChecker {
    /// Κ1: Monotone Regelvererbung
    pub fn check_realm_hierarchy(child: &Realm, parent: &Realm) -> Result<(), InvariantViolation> {
        if !child.rules.is_valid_child_of(&parent.rules) {
            return Err(InvariantViolation::K1_MonotoneRules);
        }
        Ok(())
    }

    /// Κ8: Delegation Trust-Decay
    pub fn check_delegation(
        delegator_trust: &TrustVector6D,
        delegation: &Delegation,
    ) -> Result<(), InvariantViolation> {
        // 𝕋(delegate) ≤ trust_factor × 𝕋(delegator)
        let max_delegate_trust = delegator_trust.weighted_norm(&ContextType::Default.default_weights())
            * delegation.trust_factor;

        // OK wenn trust_factor ∈ (0, 1]
        if delegation.trust_factor <= 0.0 || delegation.trust_factor > 1.0 {
            return Err(InvariantViolation::K8_InvalidTrustFactor);
        }

        Ok(())
    }

    /// Κ9: DAG-Struktur
    pub fn check_event_dag(event: &Event, parents: &[Event]) -> Result<(), InvariantViolation> {
        // Alle Parents müssen kausal vor diesem Event liegen
        for parent in parents {
            if parent.coord >= event.coord {
                return Err(InvariantViolation::K9_CausalViolation);
            }
        }
        Ok(())
    }

    /// Κ10: Finalitäts-Monotonie
    pub fn check_finality_monotone(
        old: FinalityLevel,
        new: FinalityLevel,
    ) -> Result<(), InvariantViolation> {
        if new < old {
            return Err(InvariantViolation::K10_FinalityRegression);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum InvariantViolation {
    K1_MonotoneRules,
    K8_InvalidTrustFactor,
    K9_CausalViolation,
    K10_FinalityRegression,
    // ... weitere
}
```

---

## XV. Zusammenfassung

### Design-Prinzipien

| Prinzip            | Implementierung                                            |
| ------------------ | ---------------------------------------------------------- |
| **Zukunftssicher** | Versionierte IDs, Extension Slots, Schema Registry         |
| **Performance**    | Cache-aligned Structs, Zero-Copy IDs, Prefix-basierte Keys |
| **Konsistenz**     | Unified Cost-Algebra, Shared Primitives                    |
| **Erweiterbar**    | Enum-Varianten mit Future-Slots, Custom-Payloads           |
| **Beweisbar**      | Compile-Time Size Checks, Runtime Invariant Checker        |

### Axiom-Mapping

| Axiom   | Datenstruktur                             |
| ------- | ----------------------------------------- |
| Κ1      | `RuleSet::is_valid_child_of()`            |
| Κ2-Κ5   | `TrustVector6D`, `TrustRecord`            |
| Κ6-Κ8   | `DIDDocument`, `Delegation`               |
| Κ9-Κ12  | `Event`, `FinalityState`, DAG-Invarianten |
| Κ15a-d  | `WorldFormulaContribution`, `Surprisal`   |
| Κ22-Κ24 | `Intent`, `Saga`, `SagaStep`              |

### Nächste Schritte

1. **Codegen**: Generiere `backend/src/domain/unified/` Modul aus diesem Spec
2. **Migration**: Migriere bestehende Typen auf neue Strukturen
3. **Tests**: Property-based Tests für Invarianten
4. **Benchmarks**: Verifiziere Performance-Annahmen
5. **Documentation**: API-Docs mit Axiom-Referenzen

---

_Dieses Dokument ist die autoritative Referenz für alle Erynoa-Datenstrukturen._
