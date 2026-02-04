# 📦 Dezentraler PackageManager: Blueprint → Package Transformation

> **Teil von:** Projekt Pluto
> **Kategorie:** Kernarchitektur
> **Status:** Strategische Transformation

---

## 1. Vision: Von Blueprints zu Packages

### 1.1 Das Mapping

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    BLUEPRINT → PACKAGE TRANSFORMATION                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   ALTES KONZEPT              →    NEUES KONZEPT                             ║
║   ────────────────────────────────────────────────────────────────────────  ║
║   Blueprint                  →    Package                                   ║
║   BlueprintComposer          →    PackageResolver                           ║
║   BlueprintMarketplace       →    PackageRegistry (P2P)                     ║
║   Deploy/Instantiate         →    Add Dependency + Install                  ║
║   Composition/Vererbung      →    Dependency Tree + Overrides               ║
║   BlueprintStats             →    PackageMetrics                            ║
║   BlueprintRating            →    PackageAttestation                        ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 1.2 Warum diese Transformation?

| Aspekt | Blueprint (alt) | PackageManager (neu) |
|--------|-----------------|----------------------|
| **Mindset** | Templates für Realms | Wiederverwendbare Module |
| **Dependency** | Flach (predecessor/fork) | Baum (dependencies tree) |
| **Resolution** | Manuell | Automatisch (SemVer) |
| **Isolation** | Realm-spezifisch | Realm + Projekt-Scope |
| **Vergleich** | Einzigartig in Erynoa | Vertraut (Cargo/npm) |

---

## 2. Vergleich: Cargo/npm vs. Erynoa PackageManager

| Feature | Cargo/npm | Erynoa PackageManager |
|---------|-----------|----------------------|
| **Registry** | Zentral (crates.io/npm) | Dezentral (P2P, Gossip) |
| **Publish** | Zentral mit Auth | Trust-basiert (DID-Signatur) |
| **Resolution** | SemVer, Lockfile | SemVer + ECL-Policies |
| **Installation** | Global/Projekt | Realm-isoliert (Sandbox) |
| **Sicherheit** | Signaturen (optional) | Trust + Realm-Policies + DID-Auth |
| **Hosting** | Server | P2P (StorageState + Gossip) |
| **Monetarisierung** | npm Enterprise | Mana-Gebühren (ECL-definiert) |

---

## 3. Package Manifest (in ECL)

### 3.1 Neue Manifest-Struktur

```ecl
package "my-treasury-dapp" {
    // ═══════════════════════════════════════════════════════════════════════
    // METADATEN
    // ═══════════════════════════════════════════════════════════════════════
    name: "my-treasury-dapp",
    version: "1.2.3",                          // SemVer
    description: "Realm Treasury Management with Voting",
    author_did: "did:erynoa:self:abc123...",   // Publisher DID
    license: "open",                           // open, attribution, commercial

    // ═══════════════════════════════════════════════════════════════════════
    // TRUST-REQUIREMENTS (Κ2-Κ5)
    // ═══════════════════════════════════════════════════════════════════════
    publish_requirements: {
        min_trust_r: 0.8,                      // Reliability ≥ 0.8
        min_trust_omega: 1.5,                  // Ω-Alignment ≥ 1.5
        min_novelty: 3.0,                      // Novelty-Score ≥ 3.0
    },

    install_requirements: {
        min_realm_trust: 0.5,                  // Installer Trust in Realm
    },

    // ═══════════════════════════════════════════════════════════════════════
    // DEPENDENCIES (SemVer + Policy)
    // ═══════════════════════════════════════════════════════════════════════
    dependencies: {
        "simple-chat": "^1.0.0",               // Compatible mit 1.x.x
        "voting-extension": "~2.1.0",          // Compatible mit 2.1.x
        "walletconnect-integration": "2.0.0",  // Exakt 2.0.0
        "erynoa-core": ">=0.9.0 <2.0.0",       // Range
    },

    dev_dependencies: {
        "test-harness": "^1.0.0",
    },

    optional_dependencies: {
        "premium-analytics": "^3.0.0",
    },

    // ═══════════════════════════════════════════════════════════════════════
    // RESOLUTION-POLICY
    // ═══════════════════════════════════════════════════════════════════════
    resolution_policy: {
        conflict_strategy: "prefer_highest_trust",  // oder "prefer_latest"
        allow_pre_release: false,
        trust_threshold: 0.7,                       // Min-Trust für Dependencies
    },

    // ═══════════════════════════════════════════════════════════════════════
    // INHALT
    // ═══════════════════════════════════════════════════════════════════════
    stores: [
        {
            name: "treasury",
            schema: { balance: "u128", owner: "did" },
            personal: false,
        },
        {
            name: "proposals",
            schema: { title: "string", votes: "u64", status: "enum" },
            personal: false,
        },
    ],

    policies: [
        {
            name: "treasury-access",
            type: "store_access",
            ecl: "allow if caller.role >= Moderator",
        },
    ],

    ui: {
        main: "TreasuryDashboard",
        components: ["ProposalList", "VotingPanel", "BalanceDisplay"],
    },

    logic: {
        handlers: ["on_deposit", "on_withdraw", "on_vote"],
    },

    // ═══════════════════════════════════════════════════════════════════════
    // WALLETCONNECT INTEGRATION
    // ═══════════════════════════════════════════════════════════════════════
    wallet_derivation: {
        chains: ["eip155:1", "eip155:137", "solana:mainnet"],
    },

    walletconnect_v2: {
        app_name: "Treasury dApp",
        app_icon: "https://cdn.erynoa.io/treasury-icon.png",
        supported_methods: ["eth_sendTransaction", "personal_sign"],
    },
}
```

### 3.2 Manifest-Erweiterungen vs. Blueprint

| Field | Blueprint (alt) | Package (neu) |
|-------|-----------------|---------------|
| `dependencies` | ❌ | ✅ SemVer-basiert |
| `dev_dependencies` | ❌ | ✅ Nur für Entwicklung |
| `optional_dependencies` | ❌ | ✅ Optional installierbar |
| `resolution_policy` | ❌ | ✅ Trust-basierte Resolution |
| `publish_requirements` | Hardcoded | ✅ Konfigurierbar |
| `wallet_derivation` | ❌ | ✅ Multi-Chain |
| `walletconnect_v2` | ❌ | ✅ WC V2 Metadata |

---

## 4. Package Lifecycle

### 4.1 Vollständiger Flow

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PACKAGE LIFECYCLE                                   │
└─────────────────────────────────────────────────────────────────────────────┘

1. CREATE (lokal)
   ────────────────────────────────────────────────────────────────────────
   Developer schreibt package.ecl Manifest + Code
   │
   ▼

2. VALIDATE (lokal)
   ────────────────────────────────────────────────────────────────────────
   $ erynoa package validate
   │
   ├── Manifest-Syntax prüfen
   ├── Dependencies existieren?
   ├── SemVer-Constraints valide?
   ├── ECL-Policies kompilierbar?
   └── Schemas valide?
   │
   ▼

3. BUILD (lokal)
   ────────────────────────────────────────────────────────────────────────
   $ erynoa package build
   │
   ├── Dependencies resolven (Lockfile erstellen)
   ├── ECL kompilieren
   ├── Content-Hash berechnen (BLAKE3)
   └── Package-Bundle erstellen
   │
   ▼

4. PUBLISH (P2P)
   ────────────────────────────────────────────────────────────────────────
   $ erynoa package publish
   │
   ├── Trust-Check: R ≥ 0.8, Ω ≥ 1.5
   ├── DID-Signatur erstellen
   ├── Novelty-Score berechnen
   ├── Mana reservieren
   ├── An Registry (StorageState) übertragen
   ├── Gossip an Peers
   └── Event: PackagePublished { package_id, version, publisher_did }
   │
   ▼

5. DISCOVER (P2P)
   ────────────────────────────────────────────────────────────────────────
   Peers synchronisieren via Gossip
   │
   ├── Package-Metadaten via P2PState
   ├── Trust-gewichtetes Ranking
   ├── Novelty/Diversity-Boost
   └── Searchable Index
   │
   ▼

6. INSTALL (in Realm)
   ────────────────────────────────────────────────────────────────────────
   realm.install("my-treasury-dapp", "^1.2.0")
   │
   ├── Dependency Resolution (SemVer + Trust-Policy)
   ├── Lockfile erstellen/aktualisieren
   ├── Trust-Check für jedes Package
   ├── Download Content (P2P)
   ├── Realm-Compatibility-Check
   ├── Stores erstellen
   ├── Policies installieren
   └── Event: PackageInstalled { realm_id, package_id, resolved_tree }
   │
   ▼

7. RUN (in Realm)
   ────────────────────────────────────────────────────────────────────────
   Package läuft mit Realm-Policies
   │
   ├── UI wird gerendert
   ├── Logic-Handler aktiv
   ├── WalletConnect Sessions möglich
   └── Gas/Mana-Tracking
   │
   ▼

8. UPGRADE (in Realm)
   ────────────────────────────────────────────────────────────────────────
   realm.upgrade("my-treasury-dapp", "^2.0.0")
   │
   ├── Neuer Resolution-Pass
   ├── Breaking-Change-Detection
   ├── Migration-Scripts ausführen
   ├── Rollback bei Fehler
   └── Event: PackageUpgraded { realm_id, old_version, new_version }
   │
   ▼

9. DEPRECATE/REVOKE (Publisher)
   ────────────────────────────────────────────────────────────────────────
   $ erynoa package deprecate my-treasury-dapp@1.2.3
   │
   ├── Signatur mit Publisher-DID
   ├── Gossip an alle Peers
   ├── Health-Warnung bei installierten Realms
   └── Event: PackageDeprecated { package_id, reason }
```

---

## 5. Dependency Resolution

### 5.1 Resolver-Algorithmus

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                    PACKAGE RESOLVER ALGORITHM                               │
└─────────────────────────────────────────────────────────────────────────────┘

Input: root_package, resolution_policy
Output: resolved_tree (Map<PackageId, ResolvedVersion>)

1. COLLECT
   ─────────────────────────────────────────────────────────────────────────
   DFS durch alle Dependencies → Collect(version_constraints)

2. FILTER
   ─────────────────────────────────────────────────────────────────────────
   Für jedes Package:
   │
   ├── Versions filtern: SemVer-Constraint erfüllt?
   ├── Trust filtern: Publisher-Trust ≥ threshold?
   ├── Realm-Compatibility: Package erlaubt in diesem Realm?
   └── Pre-Release: Nur wenn allow_pre_release = true

3. SOLVE
   ─────────────────────────────────────────────────────────────────────────
   SAT-Solver oder Backtracking:
   │
   ├── Konflikt? → Strategie anwenden:
   │   ├── prefer_highest_trust: Wähle Version mit höchstem Publisher-Trust
   │   ├── prefer_latest: Wähle neueste kompatible Version
   │   ├── prefer_stable: Wähle keine pre-release
   │   └── prefer_minimal: Wähle niedrigste kompatible Version
   │
   └── Keine Lösung? → ResolutionConflict Event

4. LOCK
   ─────────────────────────────────────────────────────────────────────────
   Schreibe package.lock:
   │
   └── { package_id, resolved_version, content_hash, publisher_did }

5. VERIFY
   ─────────────────────────────────────────────────────────────────────────
   Für jedes resolved Package:
   │
   ├── Signatur verifizieren
   ├── Content-Hash verifizieren
   └── Trust-Score aktuell?
```

### 5.2 Lockfile Format

```ecl
// package.lock (ECL-Format)
lockfile {
    version: 1,
    generated_at: 1707065304,
    realm_id: "did:erynoa:circle:realm123...",

    packages: [
        {
            id: "my-treasury-dapp",
            version: "1.2.3",
            content_hash: "blake3:abcdef123456...",
            publisher_did: "did:erynoa:self:abc123...",
            publisher_trust_at_resolve: 0.89,
            resolved_via: "prefer_highest_trust",
        },
        {
            id: "simple-chat",
            version: "1.5.2",
            content_hash: "blake3:fedcba654321...",
            publisher_did: "did:erynoa:self:xyz789...",
            publisher_trust_at_resolve: 0.92,
            resolved_via: "prefer_highest_trust",
        },
        // ...
    ],

    resolution_conflicts: [],
    resolution_time_ms: 142,
}
```

---

## 6. State-Integration: PackageManagerState

### 6.1 Neuer StateLayer

```rust
/// PackageManager State – Ersetzt/Erweitert BlueprintComposerState
#[derive(Debug)]
pub struct PackageManagerState {
    // ═══════════════════════════════════════════════════════════════════════
    // PUBLISHING
    // ═══════════════════════════════════════════════════════════════════════
    /// Packages publiziert (gesamt)
    pub packages_published: AtomicU64,
    /// Packages publiziert (diese Session)
    pub packages_published_session: AtomicU64,
    /// Publish-Failures (Trust-Check, Novelty, etc.)
    pub publish_failures: AtomicU64,
    /// Versions publiziert (Updates)
    pub versions_published: AtomicU64,
    /// Deprecations
    pub deprecations: AtomicU64,

    // ═══════════════════════════════════════════════════════════════════════
    // RESOLUTION
    // ═══════════════════════════════════════════════════════════════════════
    /// Dependencies resolved (gesamt)
    pub dependencies_resolved: AtomicU64,
    /// Resolution-Konflikte
    pub resolution_conflicts: AtomicU64,
    /// Resolution-Fehler (unlösbar)
    pub resolution_errors: AtomicU64,
    /// Durchschnittliche Resolution-Zeit (ms)
    pub avg_resolution_time_ms: RwLock<f64>,
    /// Maximale Dependency-Tiefe
    pub max_dependency_depth: AtomicU64,

    // ═══════════════════════════════════════════════════════════════════════
    // INSTALLATION
    // ═══════════════════════════════════════════════════════════════════════
    /// Packages installiert (gesamt)
    pub packages_installed: AtomicU64,
    /// Installations-Fehler
    pub installation_errors: AtomicU64,
    /// Upgrades durchgeführt
    pub upgrades_executed: AtomicU64,
    /// Rollbacks durchgeführt
    pub rollbacks_executed: AtomicU64,

    // ═══════════════════════════════════════════════════════════════════════
    // REGISTRY (P2P)
    // ═══════════════════════════════════════════════════════════════════════
    /// Packages in lokaler Registry
    pub registry_packages: AtomicU64,
    /// Registry-Sync-Operationen
    pub registry_syncs: AtomicU64,
    /// Packages heruntergeladen (von Peers)
    pub packages_downloaded: AtomicU64,
    /// Packages hochgeladen (zu Peers)
    pub packages_uploaded: AtomicU64,

    // ═══════════════════════════════════════════════════════════════════════
    // CACHING
    // ═══════════════════════════════════════════════════════════════════════
    /// Cache-Hits
    pub cache_hits: AtomicU64,
    /// Cache-Misses
    pub cache_misses: AtomicU64,
    /// Cache-Size (Bytes)
    pub cache_size_bytes: AtomicU64,

    // ═══════════════════════════════════════════════════════════════════════
    // RESOURCE-VERBRAUCH
    // ═══════════════════════════════════════════════════════════════════════
    /// Gas verbraucht
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht
    pub mana_consumed: AtomicU64,

    // ═══════════════════════════════════════════════════════════════════════
    // RELATIONSHIP-TRACKING (StateGraph)
    // ═══════════════════════════════════════════════════════════════════════
    /// Trust-Dependency-Updates (PackageManager ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Identity-Dependency-Updates (PackageManager ← Identity/DID)
    pub identity_dependency_updates: AtomicU64,
    /// Realm-Validations (PackageManager ✓ Realm)
    pub realm_validations: AtomicU64,
    /// P2P-Syncs (PackageManager ↔ P2P)
    pub p2p_syncs: AtomicU64,
    /// Events getriggert
    pub events_triggered: AtomicU64,
}
```

### 6.2 StateGraph-Relationen

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                   PACKAGEMANAGER IM STATEGRAPH                              │
│                                                                              │
│   PackageManager ────────────────────────────────────────────────────────── │
│     │                                                                        │
│     ├── DependsOn ──► Identity/DID    (Publisher-Auth, Signatur)           │
│     ├── DependsOn ──► Trust           (Publish/Install Trust-Checks)       │
│     ├── DependsOn ──► Gas             (Resolution/Install kosten Gas)      │
│     ├── DependsOn ──► Mana            (Publish/Install kosten Mana)        │
│     │                                                                        │
│     ├── Aggregates ──► Storage        (Packages in StorageState)           │
│     ├── Aggregates ──► ResolvedTree   (Dependency-Trees pro Realm)         │
│     │                                                                        │
│     ├── Triggers ──► Event            (Publish/Install/Upgrade Events)     │
│     │                                                                        │
│     ├── Validates ──► Realm           (Realm-Compatibility)                │
│     │                                                                        │
│     ├── Bidirectional ◄─► P2P         (Registry-Sync via Gossip)           │
│     │                                                                        │
│     └── Bidirectional ◄─► ECLVM       (Resolution-Policies als ECL)        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. StateEvents für PackageManager

```rust
/// Package-bezogene StateEvents
pub enum StateEvent {
    // ═══════════════════════════════════════════════════════════════════════
    // PUBLISHING
    // ═══════════════════════════════════════════════════════════════════════
    PackagePublished {
        package_id: String,
        version: SemVer,
        publisher_did: UniversalId,
        content_hash: String,
        novelty_score: f64,
        mana_cost: u64,
    },

    PackageDeprecated {
        package_id: String,
        version: Option<SemVer>,  // None = alle Versionen
        reason: String,
        publisher_did: UniversalId,
    },

    PublishFailed {
        package_id: String,
        reason: PublishFailureReason,  // TrustTooLow, NoveltyTooLow, etc.
        publisher_did: UniversalId,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // RESOLUTION
    // ═══════════════════════════════════════════════════════════════════════
    DependencyResolved {
        realm_id: UniversalId,
        root_package: String,
        resolved_count: u64,
        resolution_time_ms: u64,
        max_depth: u64,
    },

    ResolutionConflict {
        realm_id: UniversalId,
        package_a: String,
        version_a: SemVer,
        package_b: String,
        version_b: SemVer,
        resolution_strategy: String,
    },

    ResolutionFailed {
        realm_id: UniversalId,
        root_package: String,
        reason: ResolutionFailureReason,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // INSTALLATION
    // ═══════════════════════════════════════════════════════════════════════
    PackageInstalled {
        realm_id: UniversalId,
        package_id: String,
        version: SemVer,
        dependency_count: u64,
        installer_did: UniversalId,
        mana_cost: u64,
    },

    PackageUpgraded {
        realm_id: UniversalId,
        package_id: String,
        old_version: SemVer,
        new_version: SemVer,
        migration_executed: bool,
    },

    PackageUninstalled {
        realm_id: UniversalId,
        package_id: String,
        reason: String,
    },

    InstallationFailed {
        realm_id: UniversalId,
        package_id: String,
        reason: InstallationFailureReason,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // REGISTRY
    // ═══════════════════════════════════════════════════════════════════════
    RegistrySynced {
        peer_id: String,
        packages_received: u64,
        packages_sent: u64,
    },

    PackageDownloaded {
        package_id: String,
        version: SemVer,
        from_peer: String,
        size_bytes: u64,
    },
}
```

---

## 8. Sicherheit & Privacy

### 8.1 Publish-Sicherheit

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PUBLISH SECURITY                                    │
└─────────────────────────────────────────────────────────────────────────────┘

1. IDENTITY VERIFICATION
   ─────────────────────────────────────────────────────────────────────────
   Publisher muss gültige DID haben
   │
   ├── DID-Signatur erforderlich
   ├── Signing-Key muss in DIDDocument sein
   └── Mode muss production_safe sein (Interactive oder AgentManaged)

2. TRUST CHECK (Κ2-Κ5)
   ─────────────────────────────────────────────────────────────────────────
   Publisher muss Trust-Schwellen erreichen:
   │
   ├── Reliability (R) ≥ min_upload_trust_r (default: 0.8)
   ├── Omega (Ω) ≥ min_upload_trust_omega (default: 1.5)
   └── Bei AgentManaged: R × 0.8 wird verwendet (Trust-Penalty)

3. NOVELTY CHECK
   ─────────────────────────────────────────────────────────────────────────
   Package muss Novelty-Schwelle erreichen:
   │
   ├── Novelty-Score ≥ min_novelty_score (default: 3.0)
   ├── Verhindert Spam/Duplikate
   └── Belohnt Innovation

4. CONTENT INTEGRITY
   ─────────────────────────────────────────────────────────────────────────
   Package-Content ist unveränderlich:
   │
   ├── Content-Hash (BLAKE3) = Package-ID
   ├── Signatur über Content-Hash
   └── Jede Änderung = neue Version
```

### 8.2 Install-Sicherheit

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                         INSTALL SECURITY                                    │
└─────────────────────────────────────────────────────────────────────────────┘

1. DEPENDENCY TRUST CHECK
   ─────────────────────────────────────────────────────────────────────────
   Jedes Package im Dependency-Tree:
   │
   ├── Publisher-Trust ≥ trust_threshold (aus resolution_policy)
   ├── Package nicht deprecated
   ├── Content-Hash verifiziert
   └── Signatur valide

2. REALM ISOLATION
   ─────────────────────────────────────────────────────────────────────────
   Packages laufen in Realm-Sandbox:
   │
   ├── Kein Zugriff auf andere Realms
   ├── Nur deklarierte Stores/Policies
   ├── Gas/Mana-Limits
   └── Realm-Policies haben Vorrang

3. LICENSE COMPLIANCE
   ─────────────────────────────────────────────────────────────────────────
   Lizenz-Regeln werden durchgesetzt:
   │
   ├── Restricted: Nur erlaubte Realms
   ├── Commercial: Mana-Gebühr bezahlt?
   └── Attribution: Credit im Realm sichtbar?
```

### 8.3 Privacy durch P2P

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PRIVACY (P2P)                                       │
└─────────────────────────────────────────────────────────────────────────────┘

1. METADATA-ONLY SYNC
   ─────────────────────────────────────────────────────────────────────────
   Via Gossip werden nur synchronisiert:
   │
   ├── Package-ID (Content-Hash)
   ├── Version
   ├── Publisher-DID
   ├── Novelty/Trust-Scores
   └── Dependencies (IDs only)

   NICHT synchronisiert:
   ├── Wer hat was installiert
   ├── Realm-spezifische Konfigurationen
   └── Usage-Statistiken (außer aggregiert)

2. EPHEMERAL INSTALLATION
   ─────────────────────────────────────────────────────────────────────────
   Für maximale Privacy:
   │
   ├── Ephemeral-Mode: Keine Persistenz
   ├── Keine Trust-Tracks
   └── Keine Realm-Membership-Records
```

---

## 9. Pluto-Integration: Neue Architektur

### 9.1 Neue Verzeichnisstruktur

```text
backend/src/nervous_system/
│
├── package_manager/                     # 📦 PackageManager (NEU)
│   ├── mod.rs
│   │
│   ├── package/
│   │   ├── mod.rs
│   │   ├── manifest.rs                  # Package-Manifest Parsing
│   │   ├── semver.rs                    # SemVer-Implementierung
│   │   ├── content.rs                   # Package-Content (Stores, Policies)
│   │   └── license.rs                   # Lizenz-Typen
│   │
│   ├── resolver/
│   │   ├── mod.rs
│   │   ├── algorithm.rs                 # Resolution-Algorithmus
│   │   ├── constraints.rs               # Version-Constraints
│   │   ├── lockfile.rs                  # Lockfile Parsing/Writing
│   │   ├── conflict.rs                  # Konflikt-Strategien
│   │   └── policy.rs                    # ECL Resolution-Policies
│   │
│   ├── registry/
│   │   ├── mod.rs
│   │   ├── local.rs                     # Lokale Registry (StorageState)
│   │   ├── p2p.rs                       # P2P-Sync (Gossip)
│   │   ├── index.rs                     # Suchindex
│   │   └── novelty.rs                   # Novelty-Berechnung
│   │
│   ├── installer/
│   │   ├── mod.rs
│   │   ├── download.rs                  # Package-Download (P2P)
│   │   ├── verify.rs                    # Signatur/Hash-Verification
│   │   ├── deploy.rs                    # Stores/Policies erstellen
│   │   └── migration.rs                 # Version-Migrationen
│   │
│   ├── cli/
│   │   ├── mod.rs
│   │   ├── commands.rs                  # validate, build, publish, etc.
│   │   └── output.rs                    # Formatierung
│   │
│   └── state.rs                         # PackageManagerState
│
└── state/
    └── package_manager.rs               # Integration in UnifiedState
```

### 9.2 Migration von Blueprint zu Package

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                    MIGRATION: BLUEPRINT → PACKAGE                           │
└─────────────────────────────────────────────────────────────────────────────┘

PHASE 1: DUALER BETRIEB (Woche 1-2)
─────────────────────────────────────────────────────────────────────────────
├── PackageManagerState neben BlueprintComposerState
├── Blueprint bleibt API-kompatibel
├── Package-Manifest als neues Format
└── Neue Packages nutzen neues System

PHASE 2: MIGRATION (Woche 3-4)
─────────────────────────────────────────────────────────────────────────────
├── Migrate Blueprint → Package (automatisch)
│   ├── predecessor → dependencies (flatten)
│   ├── forked_from → dependencies + credit
│   └── Stores/Policies → Package-Content
│
├── BlueprintMarketplace → PackageRegistry
└── BlueprintComposer → PackageResolver

PHASE 3: DEPRECATION (Woche 5)
─────────────────────────────────────────────────────────────────────────────
├── Blueprint-API deprecated
├── Weiterleitung auf Package-API
└── Warnungen bei Blueprint-Nutzung

PHASE 4: CLEANUP (Woche 6+)
─────────────────────────────────────────────────────────────────────────────
├── Blueprint-Code entfernen
├── BlueprintComposerState entfernen
└── Nur PackageManager bleibt
```

---

## 10. CLI-Befehle

### 10.1 Package-Management

```bash
# Neues Package erstellen
$ erynoa package init my-awesome-app
Created package.ecl in ./my-awesome-app/

# Manifest validieren
$ erynoa package validate
✓ Manifest valid
✓ 3 dependencies found
✓ ECL policies compile

# Dependencies resolven (ohne Install)
$ erynoa package resolve
Resolving dependencies...
✓ simple-chat@1.5.2
✓ voting-extension@2.1.0
✓ walletconnect-integration@2.0.0
Resolved 3 packages in 142ms
Wrote package.lock

# Package bauen
$ erynoa package build
Building my-awesome-app@1.0.0...
✓ Compiled 2 policies
✓ Generated content hash: blake3:abc123...
✓ Bundle size: 24.5 KB

# Package publizieren
$ erynoa package publish
Publishing my-awesome-app@1.0.0...
✓ Trust check passed (R=0.89, Ω=1.7)
✓ Novelty score: 5.2
✓ Signed with did:erynoa:self:abc123...
✓ Published to registry
✓ Gossip sent to 42 peers
Package ID: blake3:abc123...

# In Realm installieren
$ erynoa realm install my-awesome-app --realm my-realm
Installing in realm my-realm...
✓ Resolved 4 packages
✓ Downloaded 2 packages from peers
✓ Verified all signatures
✓ Created 3 stores
✓ Installed 2 policies
Installed my-awesome-app@1.0.0

# Upgrade
$ erynoa realm upgrade my-awesome-app@2.0.0 --realm my-realm
Upgrading in realm my-realm...
✓ Resolved new dependency tree
✓ Breaking changes detected
✓ Running migration scripts...
✓ Migrated 1 store
Upgraded to my-awesome-app@2.0.0

# Package suchen
$ erynoa package search treasury
Searching registry...

my-treasury-dapp@1.2.3
  Publisher: did:erynoa:self:abc123...
  Trust: 0.89 | Novelty: 5.2 | Installs: 142
  "Realm Treasury Management with Voting"

simple-treasury@0.5.0
  Publisher: did:erynoa:self:xyz789...
  Trust: 0.75 | Novelty: 3.8 | Installs: 23
  "Basic treasury functionality"
```

---

## 11. Axiom-Integration

| Axiom | Anwendung im PackageManager |
|-------|----------------------------|
| **Κ2** | Publisher-Trust ∈ [0,1] für Publish/Install |
| **Κ4** | Asymmetrische Updates: Schlechte Packages sinken schneller |
| **Κ5** | Trust-Kombination bei Dependencies: t₁ ⊕ t₂ |
| **Κ6** | Publisher-DID = eindeutige Identität |
| **Κ7** | Package-Content immutable (Hash = ID) |
| **Κ8** | Trust-Decay bei Dependency-Chains |
| **Κ19** | Gini-Check: Kein Publisher dominiert Registry |
| **Κ24** | Realm-Crossing: Packages in isolierten Realms |

---

## 12. Zusammenfassung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ERYNOA PACKAGEMANAGER: VISION                             ║
║                                                                              ║
║   📦 Der dezentrale, trust-basierte Package Manager:                        ║
║                                                                              ║
║   ✓ DEZENTRAL: P2P-Registry via Gossip (kein zentraler Server)             ║
║   ✓ TRUST-BASIERT: Publish/Install erfordern Trust (Κ2-Κ5)                 ║
║   ✓ REALM-ISOLIERT: Packages in Sandbox (Κ24)                              ║
║   ✓ SEMVER-KOMPATIBEL: Vertraute Versionierung (^, ~, ranges)              ║
║   ✓ ECL-NATIVE: Resolution-Policies in ECL                                 ║
║   ✓ SYBIL-RESISTENT: Novelty + Trust-Checks                                ║
║   ✓ CARGO/NPM-FEEL: Vertraute CLI (init, build, publish, install)          ║
║                                                                              ║
║   Blueprints werden zu Packages.                                            ║
║   Das System wird zum dezentralen App-Ökosystem.                            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```
