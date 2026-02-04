# 🏰 Realm-Architektur: Isolierte Welten im Nervensystem

> **Teil von:** Projekt Pluto
> **Kategorie:** Kernarchitektur
> **Status:** Tiefenanalyse & Feature-Spezifikation

---

## 1. Vision: Das Realm als souveräne Einheit

### 1.1 Philosophie

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    REALM = SOUVERÄNE EINHEIT                                 ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Ein Realm ist nicht nur eine Partition oder ein Container.                ║
║   Ein Realm ist eine souveräne Einheit mit:                                 ║
║                                                                              ║
║   🏛️ EIGENEN REGELN       → Κ1 (Monotone Regelvererbung)                   ║
║   👥 EIGENEN MITGLIEDERN  → Membership + Roles                              ║
║   🔐 EIGENEM TRUST        → Realm-lokaler Trust (Κ24)                       ║
║   📊 EIGENEN STORES       → Isolierte Daten                                 ║
║   📜 EIGENEN POLICIES     → ECL-Gateway + Governance                        ║
║   💰 EIGENEM MANA-BUDGET  → Self-Healing Quotas                             ║
║                                                                              ║
║   Denke an:                                                                  ║
║   - Ein Discord-Server mit eigenen Regeln und Roles                         ║
║   - Eine DAO mit eigenem Treasury und Governance                            ║
║   - Ein Subnet in einem Netzwerk mit eigener Policy                         ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 1.2 Hierarchie: RootRealm → VirtualRealm → Partition

```text
                              ┌──────────────────────┐
                              │     ROOT REALM       │
                              │  (28 Kern-Axiome)    │
                              │  Κ1 - Κ28            │
                              │  min_trust = 0       │
                              └──────────┬───────────┘
                                         │
            ┌────────────────────────────┼────────────────────────────┐
            │                            │                            │
            ▼                            ▼                            ▼
   ┌────────────────┐           ┌────────────────┐           ┌────────────────┐
   │ EU-Realm       │           │ Gaming-Realm   │           │ DAO-Realm      │
   │ VirtualRealm   │           │ VirtualRealm   │           │ VirtualRealm   │
   ├────────────────┤           ├────────────────┤           ├────────────────┤
   │ +GDPR          │           │ +Fair-Play     │           │ +Token-Vote    │
   │ +MiCA          │           │ +Anti-Cheat    │           │ +Treasury      │
   │ min_trust=0.5  │           │ min_trust=0.3  │           │ min_trust=0.7  │
   └───────┬────────┘           └───────┬────────┘           └────────────────┘
           │                            │
    ┌──────┴──────┐              ┌──────┴──────┐
    ▼             ▼              ▼             ▼
┌──────────┐ ┌──────────┐  ┌──────────┐ ┌──────────┐
│ DE-Shard │ │ FR-Shard │  │ Shard-0  │ │ Shard-1  │
│Partition │ │Partition │  │Partition │ │Partition │
│ 0/2      │ │ 1/2      │  │ 0/4      │ │ 1/4      │
└──────────┘ └──────────┘  └──────────┘ └──────────┘
```

---

## 2. Code-Analyse: Bestehende Implementierung

### 2.1 Realm-Definition (`domain/unified/realm.rs`)

```rust
/// Basis-Trait für alle Realm-Typen
pub trait Realm: Send + Sync {
    fn id(&self) -> &RealmId;
    fn name(&self) -> &str;
    fn parent(&self) -> Option<&RealmId>;  // Hierarchie!
    fn rules(&self) -> &RealmRules;        // Κ1
    fn min_trust(&self) -> f32;            // Join-Requirement
    fn governance_type(&self) -> GovernanceType;
}

/// Governance-Typen (Κ21)
pub enum GovernanceType {
    Quadratic,      // Κ21: √tokens = votes
    Token,          // 1 token = 1 vote
    Reputation,     // Trust-weighted voting
    Delegated,      // Liquid Democracy
}

/// Regel-Kategorien
pub enum RuleCategory {
    Compliance,     // GDPR, MiCA, etc.
    Governance,     // Voting-Regeln
    Trust,          // Trust-Requirements
    Economic,       // Mana/Token-Regeln
    Technical,      // API-Limits, etc.
}
```

### 2.2 RealmState (`core/state.rs`)

```rust
/// Aggregierter Realm State für alle Realms
pub struct RealmState {
    /// Alle registrierten Realms
    pub realms: RwLock<HashMap<String, RealmSpecificState>>,

    /// Gesamt-Anzahl Realms
    pub total_realms: AtomicUsize,

    /// Aktuell aktive Cross-Realm-Crossings
    pub active_crossings: AtomicU64,

    /// Cross-Realm-Sagas (Κ22/Κ24)
    pub total_cross_realm_sagas: AtomicU64,

    /// Fehlgeschlagene Crossing-Versuche
    pub crossing_failures: AtomicU64,

    /// Root-Realm ID
    pub root_realm_id: RwLock<Option<String>>,
}
```

### 2.3 RealmSpecificState (pro Realm)

```rust
/// State für ein einzelnes Realm
pub struct RealmSpecificState {
    // ═══════════════════════════════════════════════════════════════════════
    // TRUST & GOVERNANCE
    // ═══════════════════════════════════════════════════════════════════════
    pub trust: RwLock<TrustVector6D>,           // Realm-Trust
    pub min_trust: RwLock<f32>,                 // Min-Trust für Join
    pub governance_type: RwLock<String>,        // Governance-Typ

    // ═══════════════════════════════════════════════════════════════════════
    // MEMBERSHIP (UniversalId-based)
    // ═══════════════════════════════════════════════════════════════════════
    pub identity_count: AtomicUsize,
    pub members_by_id: RwLock<HashSet<UniversalId>>,
    pub pending_members_by_id: RwLock<HashSet<UniversalId>>,
    pub banned_members_by_id: RwLock<HashSet<UniversalId>>,
    pub admins_by_id: RwLock<HashSet<UniversalId>>,
    pub member_realm_dids: RwLock<HashMap<UniversalId, UniversalId>>,  // Root → Sub-DID

    // ═══════════════════════════════════════════════════════════════════════
    // POLICIES & RULES (Κ1)
    // ═══════════════════════════════════════════════════════════════════════
    pub active_policies: RwLock<Vec<String>>,   // ECL-Policies
    pub active_rules: RwLock<Vec<String>>,      // Regel-IDs

    // ═══════════════════════════════════════════════════════════════════════
    // ISOLATION & CROSSING (Κ23/Κ24)
    // ═══════════════════════════════════════════════════════════════════════
    pub isolation_level: AtomicU8,              // 0=Public, 1=Members, 2=Strict
    pub crossings_in: AtomicU64,
    pub crossings_out: AtomicU64,
    pub crossings_denied: AtomicU64,
    pub crossing_allowlist: RwLock<HashSet<String>>,
    pub crossing_blocklist: RwLock<HashSet<String>>,

    // ═══════════════════════════════════════════════════════════════════════
    // SAGAS (Κ22)
    // ═══════════════════════════════════════════════════════════════════════
    pub sagas_initiated: AtomicU64,
    pub cross_realm_sagas_involved: AtomicU64,
    pub sagas_failed: AtomicU64,
    pub compensations_executed: AtomicU64,

    // ═══════════════════════════════════════════════════════════════════════
    // SELF-HEALING QUOTAS
    // ═══════════════════════════════════════════════════════════════════════
    pub quota: RealmQuota,                      // Mana/Storage-Budget
}
```

### 2.4 RealmStorage (`local/realm_storage.rs`)

```rust
/// Intelligente Speicherstruktur für Realm-Daten
///
/// Key-Struktur:
/// realm:{realm_id}:shared:store:{name}:{key}
/// realm:{realm_id}:personal:{did}:store:{name}:{key}

pub struct RealmStorage {
    /// Partition für Metadaten (Schemas, Policies)
    pub meta: PartitionHandle,

    /// Partition für dynamische Daten
    pub data: PartitionHandle,

    /// Schema-Cache (Realm:Store → Schema)
    schema_cache: RwLock<HashMap<String, StoreSchema>>,
}

/// Schema für einen dynamischen Store
pub struct StoreSchema {
    pub name: String,
    pub version: u32,
    pub fields: HashMap<String, SchemaFieldType>,
    pub personal: bool,
    pub max_entries: u64,
    pub indices: Vec<String>,
}
```

### 2.5 LazyShardedRealmState (Skalierung)

```rust
/// Lock-free, sharded Realm-State für Millionen von Realms
pub struct LazyShardedRealmState {
    /// Shards: Jeder ist eine lock-free DashMap
    shards: Box<[DashMap<String, Arc<RealmSpecificState>>]>,

    /// LRU pro Shard für Eviction
    lru_caches: Box<[TokioRwLock<LruCache<String, ()>>]>,

    /// Per-Shard Statistiken
    shard_stats: Box<[ShardStatistics]>,

    /// Storage-Loader für Lazy Loading
    storage_loader: Option<Arc<dyn RealmStorageLoader>>,

    /// Konfiguration
    config: ShardingConfig,
}

// Performance:
// - Read: O(1) bei Cache-Hit
// - Write: O(1) lock-free
// - Memory: Nur aktive Realms im Speicher
// - Contention: Nahezu 0 bei unabhängigen Realms
```

---

## 3. Axiom-Integration

### 3.1 Relevante Axiome

| Axiom | Bedeutung für Realms |
|-------|---------------------|
| **Κ1** | Monotone Regelvererbung: `rules(Child) ⊇ rules(Parent)` |
| **Κ21** | Quadratisches Voting: `votes = √tokens` |
| **Κ22** | Saga-Pattern: Atomare Cross-Realm-Operationen |
| **Κ23** | Realm-Crossing: Trust-Dämpfung bei Grenzübertritt |
| **Κ24** | Realm-lokaler Trust: Isoliertes Trust-System |

### 3.2 Κ1: Monotone Regelvererbung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   AXIOM Κ1: MONOTONE REGELVERERBUNG                                          ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Definition:                                                                ║
║   ∀ 𝒞₁ ⊂ 𝒞₂ : rules(𝒞₁) ⊇ rules(𝒞₂)                                       ║
║                                                                              ║
║   Bedeutung:                                                                 ║
║   - Kind-Realms erben ALLE Regeln des Parent                                ║
║   - Regeln können nur HINZUGEFÜGT werden, nie entfernt                      ║
║   - Root-Realm enthält die 28 Kern-Axiome (unveränderlich)                  ║
║                                                                              ║
║   Beispiel:                                                                  ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │  Root-Realm: {Κ1, Κ2, ..., Κ28}  (28 Regeln)                       │   ║
║   │       │                                                             │   ║
║   │       ▼                                                             │   ║
║   │  EU-Realm: {Κ1, ..., Κ28, GDPR, MiCA}  (30 Regeln) ✓               │   ║
║   │       │                                                             │   ║
║   │       ▼                                                             │   ║
║   │  DE-Shard: {Κ1, ..., Κ28, GDPR, MiCA, BAFIN}  (31 Regeln) ✓        │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   Violation-Check:                                                           ║
║   validate_k1(&self, parent_rules) → Result<(), K1Violation>                ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 3.3 Κ23: Realm-Crossing Trust-Dämpfung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   AXIOM Κ23: REALM-CROSSING TRUST-DÄMPFUNG                                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Bei Crossing von Realm A nach Realm B:                                    ║
║                                                                              ║
║   effective_trust_in_B = trust_in_A × crossing_factor                       ║
║                                                                              ║
║   wobei crossing_factor ∈ (0, 1] abhängt von:                               ║
║   - Verwandtschaft der Realms (gemeinsamer Parent = höher)                  ║
║   - Allowlist/Blocklist-Status                                              ║
║   - Trust-Level beider Realms                                               ║
║                                                                              ║
║   Beispiel:                                                                  ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │  Alice in EU-Realm: Trust = 0.9                                     │   ║
║   │       │                                                             │   ║
║   │       │ crossing_factor = 0.8 (sibling-Realm)                       │   ║
║   │       ▼                                                             │   ║
║   │  Alice in Gaming-Realm: effective_trust = 0.9 × 0.8 = 0.72         │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   Allowlist/Blocklist:                                                       ║
║   - Allowlist: crossing_factor = 1.0 (vertrauenswürdig)                     ║
║   - Blocklist: crossing_factor = 0.0 (blockiert)                            ║
║   - Neutral: Policy entscheidet                                             ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 3.4 Κ24: Realm-lokaler Trust

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   AXIOM Κ24: REALM-LOKALER TRUST                                             ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Jedes Realm hat ein eigenes Trust-System:                                 ║
║                                                                              ║
║   TrustVector6D_realm = {                                                    ║
║       R: Reliability (realm-spezifisch),                                    ║
║       I: Integrity (realm-spezifisch),                                      ║
║       C: Capability (realm-spezifisch),                                     ║
║       T: Tenure (realm-spezifisch),                                         ║
║       S: Social (realm-spezifisch),                                         ║
║       Ω: Alignment (Κ2-Konformität im Realm)                                ║
║   }                                                                          ║
║                                                                              ║
║   Bedeutung:                                                                 ║
║   - Trust-Aktionen in Realm A beeinflussen NICHT Trust in Realm B           ║
║   - Begrenzte Trust-Portabilität (via Κ23 Dämpfung)                         ║
║   - Realm kann eigene Trust-Regeln definieren                               ║
║                                                                              ║
║   Beispiel:                                                                  ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │  Alice in Gaming-Realm: Trust = 0.95 (viele Siege, fair play)      │   ║
║   │  Alice in DAO-Realm: Trust = 0.4 (neu, wenig Proposals)            │   ║
║   │                                                                     │   ║
║   │  → Unterschiedliche Trust-Dimensionen relevant!                     │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 4. Synergien mit Pluto-Komponenten

### 4.1 Realm × Identity (Multi-DID)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   REALM × IDENTITY: SUB-DID PRO REALM                                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Root-DID: did:erynoa:self:alice123...                                     ║
║       │                                                                      ║
║       ├── Realm-DID für EU-Realm: did:erynoa:circle:eu-alice...            ║
║       │   └── Isolierte Aktivitäten in EU-Realm                             ║
║       │                                                                      ║
║       ├── Realm-DID für Gaming-Realm: did:erynoa:circle:gamer-alice...     ║
║       │   └── Gaming-spezifische Reputation                                 ║
║       │                                                                      ║
║       └── Wallet-Derivation pro Realm:                                      ║
║           ├── EU-Realm: m/44'/erynoa'/0'/realm/eu/0                         ║
║           └── Gaming-Realm: m/44'/erynoa'/0'/realm/gaming/0                 ║
║                                                                              ║
║   Vorteile:                                                                  ║
║   - Privacy: Aktivitäten in Realm A nicht mit Realm B korrelierbar          ║
║   - Isolation: Kompromittierte Realm-DID ≠ kompromittierte Root-DID         ║
║   - Flexibilität: Verschiedene Wallets/Keys pro Realm                       ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 4.2 Realm × Trust (Lokaler Trust)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   REALM × TRUST: REALM-LOKALER TRUST                                         ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Jedes Realm verwaltet eigenen TrustVector6D:                              ║
║                                                                              ║
║   RealmSpecificState {                                                       ║
║       trust: TrustVector6D,           // Realm-Trust (Aggregat)             ║
║       member_trusts: HashMap<DID, TrustVector6D>,  // Pro-Member-Trust      ║
║   }                                                                          ║
║                                                                              ║
║   Trust-Aktionen bleiben im Realm:                                          ║
║   - Positive Interaktion in Gaming-Realm → Gaming-Trust ↑                   ║
║   - Betrug im Gaming-Realm → Gaming-Trust ↓                                 ║
║   - KEINE automatische Auswirkung auf andere Realms                         ║
║                                                                              ║
║   Aber: Schwerwiegende Verstöße können "leaken" (Κ5 Bounds):                ║
║   - Trust < 0.1 in einem Realm → Warnung in allen Realms                    ║
║   - Trust = 0 (permanenter Ban) → Cross-Realm-Markierung                    ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 4.3 Realm × PackageManager

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   REALM × PACKAGEMANAGER                                                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Packages werden PRO REALM installiert:                                    ║
║                                                                              ║
║   RealmSpecificState {                                                       ║
║       installed_packages: HashMap<PackageId, InstalledPackage>,             ║
║       package_lockfile: PackageLockfile,                                    ║
║       package_overrides: HashMap<PackageId, PackageOverrides>,              ║
║   }                                                                          ║
║                                                                              ║
║   Features:                                                                  ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │  1. REALM-ISOLATION                                                 │   ║
║   │     └── Package X in Realm A kann nicht auf Realm B zugreifen        │   ║
║   │                                                                     │   ║
║   │  2. REALM-OVERRIDES                                                 │   ║
║   │     └── Realm kann Package-Policies überschreiben                   │   ║
║   │         └── z.B. eigene Voting-Rules statt Package-Default          │   ║
║   │                                                                     │   ║
║   │  3. CROSS-REALM DEDUPLICATION                                       │   ║
║   │     └── Content nur einmal gespeichert                              │   ║
║   │     └── Config pro Realm                                            │   ║
║   │                                                                     │   ║
║   │  4. REALM-TEMPLATES                                                 │   ║
║   │     └── Meta-Package definiert komplettes Realm                     │   ║
║   │     └── z.B. "social-media-starter" mit 5 Sub-Packages              │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 4.4 Realm × Gas/Mana (Quotas)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   REALM × GAS/MANA: SELF-HEALING QUOTAS                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Jedes Realm hat eigenes Mana-Budget:                                      ║
║                                                                              ║
║   RealmQuota {                                                               ║
║       mana_budget: u64,              // Gesamt-Budget                       ║
║       mana_used: AtomicU64,          // Aktuell verbraucht                  ║
║       mana_regeneration: f64,        // Pro Sekunde                         ║
║       storage_quota_bytes: u64,      // Max. Storage                        ║
║       storage_used_bytes: AtomicU64, // Aktuell belegt                      ║
║   }                                                                          ║
║                                                                              ║
║   Self-Healing:                                                              ║
║   - Quota-Überschreitung → temporäre Drosselung                             ║
║   - Mana regeneriert über Zeit                                              ║
║   - bei 0 Mana: Read-Only-Mode (keine Writes)                               ║
║                                                                              ║
║   Quota-Health:                                                              ║
║   quota_health = 1.0 - (mana_used / mana_budget)                            ║
║                                                                              ║
║   Bei quota_health < 0.2:                                                    ║
║   - Event: RealmQuotaWarning                                                ║
║   - Throttling aktiviert                                                    ║
║   - Admins benachrichtigt                                                   ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 4.5 Realm × P2P (Gossip)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   REALM × P2P: REALM-SCOPED GOSSIP                                           ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Gossip ist Realm-scoped:                                                  ║
║                                                                              ║
║   Realm-Topic: /erynoa/realm/{realm_id}/events                              ║                                                                              ║
║   - Nur Realm-Members abonnieren                                            ║
║   - Isolation: Events leaken nicht                                          ║
║                                                                              ║
║   Cross-Realm-Topics (eingeschränkt):                                       ║
║   /erynoa/cross-realm/sagas → Saga-Koordination                             ║
║   /erynoa/cross-realm/announcements → Öffentliche Announcements             ║
║                                                                              ║
║   Peer-Discovery pro Realm:                                                 ║
║   - Kademlia DHT Key: /realm/{realm_id}/peers                              ║
║   - Finde Peers die im selben Realm sind                                   ║
║   - Optimiert Gossip (weniger Hops)                                        ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 5. Feature-Spezifikationen

### 5.1 Feature: Realm Gateway Policies

```ecl
// ECL-Policy die beim Realm-Join ausgeführt wird
gateway_policy "my-realm-gateway" {
    // Trust-Requirements
    requirements: {
        min_trust_r: 0.5,
        min_trust_omega: 1.2,
    },

    // Verification
    verification: {
        require_attestation: "erynoa:kyc:verified",
        // oder
        require_vouching: {
            min_vouchers: 2,
            voucher_min_trust: 0.7,
        },
    },

    // Actions on join
    on_join: {
        // Erstelle personal Stores
        create_personal_stores: ["profile", "inbox", "settings"],

        // Initial Trust
        initial_local_trust: 0.3,

        // Initial Role
        initial_role: "member",

        // Event
        emit_event: { type: "member_joined", public: true },
    },

    // Mana-Cost für Join
    join_cost: 100,
}
```

### 5.2 Feature: Realm Governance

```ecl
// Governance-Policy für Realm
governance_policy "dao-governance" {
    // Voting-Typ (Κ21)
    voting_type: "quadratic",  // √tokens = votes

    // Proposal-Requirements
    proposal_requirements: {
        min_trust_to_propose: 0.6,
        min_tokens_to_propose: 100,
        min_discussion_period: "48h",
        min_voting_period: "72h",
    },

    // Quorum
    quorum: {
        min_participation: 0.1,  // 10% aller Tokens
        min_approval: 0.5,       // 50% Zustimmung
    },

    // Execution
    execution: {
        timelock: "24h",         // Verzögerung vor Execution
        veto_threshold: 0.33,    // 33% können blockieren

        // Automatische Execution via ECLVM
        auto_execute: true,
        execution_gas_limit: 100000,
    },

    // Treasury
    treasury: {
        authorized_signers: 3,
        required_signers: 2,      // 2-of-3 Multisig
        max_single_spend: 10000,  // ohne Proposal
    },
}
```

### 5.3 Feature: Realm Isolation Levels

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   ISOLATION LEVELS                                                           ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Level 0: PUBLIC                                                            ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   - Jeder kann Realm-Daten lesen (public Stores)                            ║
║   - Jeder kann joinen (nach Gateway-Policy)                                 ║
║   - Cross-Realm-Crossing erlaubt                                            ║
║                                                                              ║
║   Level 1: MEMBERS ONLY                                                      ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   - Nur Members können Realm-Daten lesen                                    ║
║   - Join erfordert Invitation oder Approval                                 ║
║   - Cross-Realm-Crossing nur mit Member-Status                              ║
║                                                                              ║
║   Level 2: STRICT                                                            ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   - Verschlüsselte Realm-Daten (Ende-zu-Ende)                               ║
║   - Join erfordert Multi-Vouching + KYC                                     ║
║   - Cross-Realm-Crossing blockiert (nur via explicit Bridge)                ║
║   - Realm-spezifische Keys für Encryption                                   ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.4 Feature: Cross-Realm Sagas (Κ22)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   CROSS-REALM SAGAS: ATOMARE OPERATIONEN                                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Saga: "transfer-reputation"                                                ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   Ziel: Reputation von Realm A nach Realm B übertragen                      ║
║                                                                              ║
║   Schritte:                                                                  ║
║   1. [Realm A] Reserve reputation points                                    ║
║   2. [Realm A] Apply crossing factor (Κ23)                                  ║
║   3. [Realm B] Verify identity (Realm-DID)                                  ║
║   4. [Realm B] Credit adjusted reputation                                   ║
║   5. [Realm A] Finalize (deduct reputation)                                 ║
║                                                                              ║
║   Compensation bei Fehler:                                                  ║
║   - Schritt 5 failed → Reverse Schritt 4, 3, 2, 1                          ║
║   - Jeder Schritt hat Compensation-Action                                   ║
║                                                                              ║
║   Saga-Coordinator:                                                          ║
║   - Läuft auf Initiator-Node                                                ║
║   - Tracked alle Schritte                                                   ║
║   - Timeout für jeden Schritt                                               ║
║   - Automatic Rollback bei Timeout                                          ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

// ECL-Definition
saga "transfer-reputation" {
    participants: [realm_a, realm_b],

    steps: [
        {
            id: "reserve",
            realm: "realm_a",
            action: "reserve_reputation(amount)",
            compensate: "release_reservation()",
            timeout: "10s",
        },
        {
            id: "apply_crossing",
            realm: "realm_a",
            action: "apply_crossing_factor(realm_b)",
            depends_on: ["reserve"],
        },
        {
            id: "verify_identity",
            realm: "realm_b",
            action: "verify_realm_did(caller)",
            depends_on: ["apply_crossing"],
            compensate: "revoke_verification()",
        },
        {
            id: "credit",
            realm: "realm_b",
            action: "credit_reputation(adjusted_amount)",
            depends_on: ["verify_identity"],
            compensate: "debit_reputation(adjusted_amount)",
        },
        {
            id: "finalize",
            realm: "realm_a",
            action: "finalize_deduction()",
            depends_on: ["credit"],
        },
    ],
}
```

### 5.5 Feature: Realm Discovery

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   REALM DISCOVERY                                                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Öffentliche Realms sind discoverable:                                     ║
║                                                                              ║
║   $ erynoa realm search "gaming"                                            ║
║                                                                              ║
║   Results:                                                                   ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │  1. Arena-Champions (gaming, competitive)                           │   ║
║   │     Members: 15,420 | Min-Trust: 0.3 | Governance: Reputation       │   ║
║   │     "Competitive gaming with fair-play focus"                       │   ║
║   │                                                                     │   ║
║   │  2. Casual-Gamers (gaming, social)                                  │   ║
║   │     Members: 42,100 | Min-Trust: 0.1 | Governance: Quadratic        │   ║
║   │     "Relaxed gaming community for all skill levels"                 │   ║
║   │                                                                     │   ║
║   │  3. NFT-Gaming-DAO (gaming, nft, dao)                              │   ║
║   │     Members: 8,750 | Min-Trust: 0.5 | Governance: Token             │   ║
║   │     "DAO for NFT-based games"                                       │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   Ranking basiert auf:                                                       ║
║   - Realm-Trust                                                             ║
║   - Member-Count                                                            ║
║   - Activity                                                                ║
║   - Relevanz zum Search-Query                                               ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 6. Pluto-Integration: Neue Architektur

### 6.1 Vorgeschlagene Verzeichnisstruktur

```text
backend/src/nervous_system/realm/
│
├── mod.rs                               # Re-exports
│
├── core/
│   ├── realm.rs                         # Realm Trait + Typen
│   ├── root_realm.rs                    # Root-Realm (28 Axiome)
│   ├── virtual_realm.rs                 # VirtualRealm
│   └── partition.rs                     # Partition (Shard)
│
├── membership/
│   ├── member.rs                        # Membership-Typen
│   ├── roles.rs                         # Rollen (Member, Mod, Admin)
│   └── gateway.rs                       # Gateway-Policies
│
├── rules/
│   ├── rule.rs                          # Rule-Typen
│   ├── category.rs                      # RuleCategory
│   ├── inheritance.rs                   # Κ1 Validation
│   └── builtin.rs                       # Kern-Axiome
│
├── governance/
│   ├── types.rs                         # GovernanceType
│   ├── quadratic.rs                     # Κ21 Quadratic Voting
│   ├── proposals.rs                     # Proposal-System
│   └── treasury.rs                      # Treasury-Management
│
├── crossing/
│   ├── crossing.rs                      # Κ23 Crossing-Logic
│   ├── trust_damping.rs                 # Trust-Dämpfung
│   └── allowlist.rs                     # Allow/Blocklist
│
├── saga/
│   ├── saga.rs                          # Κ22 Saga-Pattern
│   ├── coordinator.rs                   # Saga-Coordinator
│   └── compensation.rs                  # Compensation-Logic
│
├── storage/
│   ├── realm_storage.rs                 # RealmStorage
│   ├── schema.rs                        # StoreSchema
│   ├── prefix.rs                        # PrefixBuilder
│   └── evolution.rs                     # Schema-Evolution
│
├── state/
│   ├── realm_state.rs                   # RealmState (Aggregat)
│   ├── realm_specific.rs                # RealmSpecificState
│   ├── sharded.rs                       # LazyShardedRealmState
│   └── quota.rs                         # RealmQuota
│
└── events/
    ├── realm_events.rs                  # Realm-spezifische Events
    └── crossing_events.rs               # Crossing-Events
```

### 6.2 StateGraph-Integration

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                        REALM IM STATEGRAPH                                   │
│                                                                              │
│   Realm ────────────────────────────────────────────────────────────────    │
│     │                                                                        │
│     ├── DependsOn ──► Identity    (Membership, Realm-Sub-DIDs)             │
│     ├── DependsOn ──► Trust       (Realm-lokaler Trust, Κ24)               │
│     ├── DependsOn ──► Gas/Mana    (Quotas, Self-Healing)                   │
│     │                                                                        │
│     ├── Aggregates ──► Storage    (Realm-spezifische Stores)               │
│     ├── Aggregates ──► Packages   (Installed Packages)                     │
│     │                                                                        │
│     ├── Triggers ──► Event        (Join, Leave, Crossing, Saga)            │
│     │                                                                        │
│     ├── Validates ──► Rules       (Κ1 Monotone Vererbung)                  │
│     │                                                                        │
│     ├── Bidirectional ◄─► P2P     (Gossip, Realm-Topics)                   │
│     │                                                                        │
│     └── Bidirectional ◄─► ECLVM   (Policies, Governance)                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. StateEvents für Realm

```rust
pub enum StateEvent {
    // ═══════════════════════════════════════════════════════════════════════
    // REALM LIFECYCLE
    // ═══════════════════════════════════════════════════════════════════════
    RealmCreated {
        realm_id: UniversalId,
        name: String,
        parent_id: Option<UniversalId>,
        creator_did: UniversalId,
        governance_type: GovernanceType,
    },

    RealmUpdated {
        realm_id: UniversalId,
        changes: Vec<RealmChange>,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // MEMBERSHIP
    // ═══════════════════════════════════════════════════════════════════════
    MemberJoined {
        realm_id: UniversalId,
        member_did: UniversalId,
        realm_sub_did: Option<UniversalId>,
        initial_role: MemberRole,
        mana_paid: u64,
    },

    MemberLeft {
        realm_id: UniversalId,
        member_did: UniversalId,
        reason: LeaveReason,
    },

    MemberBanned {
        realm_id: UniversalId,
        member_did: UniversalId,
        banned_by: UniversalId,
        reason: String,
    },

    RoleChanged {
        realm_id: UniversalId,
        member_did: UniversalId,
        old_role: MemberRole,
        new_role: MemberRole,
        changed_by: UniversalId,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // RULES & POLICIES (Κ1)
    // ═══════════════════════════════════════════════════════════════════════
    RuleAdded {
        realm_id: UniversalId,
        rule_id: String,
        category: RuleCategory,
        added_by: UniversalId,
    },

    PolicyActivated {
        realm_id: UniversalId,
        policy_id: String,
        policy_type: PolicyType,
    },

    PolicyDeactivated {
        realm_id: UniversalId,
        policy_id: String,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // CROSSING (Κ23)
    // ═══════════════════════════════════════════════════════════════════════
    CrossingAttempted {
        from_realm: UniversalId,
        to_realm: UniversalId,
        identity_did: UniversalId,
        crossing_factor: f64,
    },

    CrossingSucceeded {
        from_realm: UniversalId,
        to_realm: UniversalId,
        identity_did: UniversalId,
        effective_trust: f64,
    },

    CrossingDenied {
        from_realm: UniversalId,
        to_realm: UniversalId,
        identity_did: UniversalId,
        reason: CrossingDenialReason,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // SAGA (Κ22)
    // ═══════════════════════════════════════════════════════════════════════
    SagaStarted {
        saga_id: String,
        saga_type: String,
        participant_realms: Vec<UniversalId>,
        initiator_did: UniversalId,
    },

    SagaStepCompleted {
        saga_id: String,
        step_id: String,
        realm_id: UniversalId,
    },

    SagaCompleted {
        saga_id: String,
        success: bool,
        compensations_executed: u32,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // QUOTA
    // ═══════════════════════════════════════════════════════════════════════
    QuotaWarning {
        realm_id: UniversalId,
        quota_type: QuotaType,
        current_usage: u64,
        limit: u64,
    },

    QuotaExceeded {
        realm_id: UniversalId,
        quota_type: QuotaType,
        throttling_enabled: bool,
    },
}
```

---

## 8. Zusammenfassung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    REALM-ARCHITEKTUR: KERNPUNKTE                             ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   🏰 SOUVERÄNE EINHEITEN                                                    ║
║      → Eigene Regeln, Mitglieder, Trust, Stores, Policies                  ║
║      → Hierarchie: Root → Virtual → Partition                              ║
║                                                                              ║
║   📜 MONOTONE VERERBUNG (Κ1)                                                ║
║      → Regeln können nur hinzugefügt werden                                 ║
║      → Root = 28 Kern-Axiome                                                ║
║                                                                              ║
║   🔐 ISOLATION + CROSSING (Κ23)                                             ║
║      → 3 Isolation-Levels: Public, Members, Strict                          ║
║      → Trust-Dämpfung bei Grenzübertritt                                    ║
║      → Allow/Blocklists                                                     ║
║                                                                              ║
║   🌐 LOKALER TRUST (Κ24)                                                    ║
║      → Jedes Realm hat eigenen TrustVector6D                                ║
║      → Begrenzte Portabilität via Crossing                                  ║
║                                                                              ║
║   🎭 REALM-SUB-DIDs                                                          ║
║      → Isolierte Identitäten pro Realm                                      ║
║      → Privacy: Aktivitäten nicht korrelierbar                              ║
║                                                                              ║
║   📦 PACKAGE-ISOLATION                                                       ║
║      → Packages pro Realm installiert                                       ║
║      → Realm-Overrides für Policies                                         ║
║                                                                              ║
║   ⚡ SELF-HEALING QUOTAS                                                     ║
║      → Mana-Budget pro Realm                                                ║
║      → Automatisches Throttling                                             ║
║                                                                              ║
║   🔄 CROSS-REALM SAGAS (Κ22)                                                ║
║      → Atomare Multi-Realm-Operationen                                      ║
║      → Automatische Compensations                                           ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```
