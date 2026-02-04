# ⚡ Sharding-Architektur: Horizontale Skalierung im Nervensystem

> **Teil von:** Projekt Pluto
> **Kategorie:** Kerninfrastruktur
> **Status:** Tiefenanalyse & Pluto-Abstimmung

---

## 1. Vision: Millionen von Realms – Ein System

### 1.1 Skalierungsziel

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    ERYNOA SHARDING VISION                                    ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Ziel: 1+ Million Realms mit:                                              ║
║   - O(1) Realm-Lookup                                                       ║
║   - Lock-free Concurrent Access                                             ║
║   - Lazy Loading (Hot Realms im Memory)                                     ║
║   - Automatische Eviction (LRU)                                             ║
║   - Horizontale Skalierung (Multi-Node ready)                               ║
║                                                                              ║
║   Performance-Garantien:                                                     ║
║   - Read: O(1) bei Cache-Hit                                                ║
║   - Write: O(1) lock-free                                                   ║
║   - Memory: ~1-10GB für 10K-100K hot Realms                                 ║
║   - Contention: Nahezu 0 bei unabhängigen Realms                            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 1.2 Architektur-Übersicht

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                    LazyShardedRealmState Architecture                        │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                         Shard Selection                                 ││
│  │                  FxHash(realm_id) % num_shards                         ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                    │                                         │
│         ┌──────────────────────────┼──────────────────────────┐             │
│         ▼                          ▼                          ▼             │
│  ┌─────────────┐           ┌─────────────┐           ┌─────────────┐       │
│  │  Shard 0    │           │  Shard 1    │           │  Shard N-1  │       │
│  │ ┌─────────┐ │           │ ┌─────────┐ │           │ ┌─────────┐ │       │
│  │ │DashMap  │ │           │ │DashMap  │ │           │ │DashMap  │ │       │
│  │ │realm→Arc│ │           │ │realm→Arc│ │           │ │realm→Arc│ │       │
│  │ └─────────┘ │           │ └─────────┘ │           │ └─────────┘ │       │
│  │ ┌─────────┐ │           │ ┌─────────┐ │           │ ┌─────────┐ │       │
│  │ │LRU Cache│ │           │ │LRU Cache│ │           │ │LRU Cache│ │       │
│  │ │(access) │ │           │ │(access) │ │           │ │(access) │ │       │
│  │ └─────────┘ │           │ └─────────┘ │           │ └─────────┘ │       │
│  └─────────────┘           └─────────────┘           └─────────────┘       │
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                     Background Eviction Tasks                           ││
│  │              Per-Shard async task, 10min interval                       ││
│  │              Removes LRU entries beyond max_per_shard                   ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                              │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                     Lazy Loading Pipeline                               ││
│  │   get_or_load() → Cache Miss → Storage Load → Event Replay → Insert    ││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Implementierung in state.rs

### 2.1 ShardingConfig

```rust
/// Konfiguration für Realm-Sharding
pub struct ShardingConfig {
    /// Anzahl der Shards (Default: 64, optimiert für moderne CPUs)
    pub num_shards: usize,           // 64

    /// Max Einträge pro Shard bevor Eviction beginnt
    pub max_per_shard: usize,        // 20_000

    /// Eviction-Intervall in Sekunden
    pub eviction_interval_secs: u64, // 600 (10 Minuten)

    /// LRU-Kapazität pro Shard für Access-Tracking
    pub lru_capacity_per_shard: usize, // 25_000

    /// Ob Lazy Loading aktiviert ist
    pub lazy_loading_enabled: bool,  // true

    /// Ob Event-Replay bei Load aktiviert ist
    pub event_replay_on_load: bool,  // true
}
```

#### Konfigurationsprofile

| Profil | num_shards | max_per_shard | eviction_interval | Use Case |
|--------|------------|---------------|-------------------|----------|
| **minimal** | 4 | 100 | 60s | Tests |
| **default** | 64 | 20.000 | 10min | Development |
| **production** | 128 | 50.000 | 5min | Production |
| **auto_scaled** | CPU×4 | 30.000 | 10min | Auto |

### 2.2 LazyShardedRealmState

```rust
/// Lock-free, sharded Realm-State mit Lazy Loading und LRU Eviction
pub struct LazyShardedRealmState {
    /// Shards: Jeder ist eine lock-free DashMap
    shards: Box<[DashMap<String, Arc<RealmSpecificState>>]>,

    /// LRU pro Shard für Access-Tracking (async-fähig)
    lru_caches: Box<[TokioRwLock<LruCache<String, ()>>]>,

    /// Per-Shard Statistiken (atomic)
    shard_stats: Box<[ShardStatistics]>,

    /// Storage-Loader für Lazy Loading
    storage_loader: Option<Arc<dyn RealmStorageLoader>>,

    /// Konfiguration
    config: ShardingConfig,

    /// Global Realm-Count (approximate)
    total_realms_approx: AtomicUsize,

    /// Global Cache Counters
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    evictions: AtomicU64,
}
```

### 2.3 Shard-Selection (FxHash)

```rust
/// Deterministischer Hash für Shard-Selection
#[inline]
fn fx_hash_str(s: &str) -> u64 {
    let mut hasher = FxHasher::default();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Berechne Shard-Index für Realm-ID
#[inline]
fn shard_index(&self, realm_id: &str) -> usize {
    (fx_hash_str(realm_id) as usize) % self.shards.len()
}
```

**Warum FxHash?**
- O(1) Hash-Berechnung
- Deterministisch (gleiche ID → gleicher Shard)
- Schneller als kryptografische Hashes
- Ausreichend für Load-Balancing

### 2.4 API-Methoden

```rust
impl LazyShardedRealmState {
    /// Hole Realm synchron (nur Cache, kein Lazy Load)
    pub fn get_cached(&self, realm_id: &str) -> Option<Arc<RealmSpecificState>>;

    /// Hole oder lade Realm asynchron (mit Lazy Loading + Event Replay)
    pub async fn get_or_load(&self, realm_id: &str)
        -> Result<Arc<RealmSpecificState>, RealmLoadError>;

    /// Registriere neues Realm (synchron)
    pub fn register(&self, realm_id: &str, state: RealmSpecificState) -> bool;

    /// Registriere oder update Realm
    pub fn upsert(&self, realm_id: &str, state: RealmSpecificState);

    /// Entferne Realm aus Cache
    pub fn remove(&self, realm_id: &str) -> Option<Arc<RealmSpecificState>>;

    /// Prüfe ob Realm im Cache ist
    pub fn contains(&self, realm_id: &str) -> bool;

    /// Führe Eviction für alle Shards durch
    pub async fn evict_all(&self) -> usize;

    /// Starte Background-Eviction-Tasks
    pub fn spawn_eviction_tasks(self: Arc<Self>);

    /// Hole Statistiken für alle Shards
    pub fn stats(&self) -> ShardingStats;
}
```

---

## 3. ShardMonitor: Sicherheit für horizontale Skalierung

### 3.1 Risiken bei Sharding

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SHARDING-RISIKEN & LÖSUNGEN                                                ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   RISIKO 1: LOKALE TRUST-VERZERRUNG (Shard-Bias)                            ║
║   ────────────────────────────────────────────────────────────────────────  ║
║   Problem:                                                                   ║
║   Angreifer flutet einen Shard mit Fake-Realms/Entities                     ║
║   → Trust lokal verzerrt (z.B. viele positive Updates nur in Shard 5)       ║
║                                                                              ║
║   Lösung: Shard-Entropy-Score                                               ║
║   - Trackt lokale Vielfalt der Update-Quellen                               ║
║   - Abweichung > 50% → "biased Shard" → Alarm + Dämpfung                    ║
║                                                                              ║
║   ────────────────────────────────────────────────────────────────────────  ║
║                                                                              ║
║   RISIKO 2: CROSS-SHARD-ANGRIFFE                                            ║
║   ────────────────────────────────────────────────────────────────────────  ║
║   Problem:                                                                   ║
║   Angreifer aus "toxischem" Shard versucht in andere Shards                 ║
║   einzudringen (viele failed Sagas/Crossings)                               ║
║                                                                              ║
║   Lösung: Shard-Reputation                                                   ║
║   - Reputation (0.0–1.0) basierend auf Fehlerrate                           ║
║   - Niedrige Reputation → höhere Gas-Kosten für Outbound-Requests           ║
║   - Hohe Fehlerrate → temporäre Quarantäne                                  ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 3.2 ShardMonitor Implementation

```rust
/// Shard-spezifische Überwachung für das Immunsystem
pub struct ShardMonitor {
    /// Aktivität pro Shard (ShardIndex → Update-Count)
    pub shard_activity: DashMap<u64, AtomicU64>,

    /// Lokale Trust-Entropy pro Shard (ShardIndex → Entropy)
    /// Entropy nahe 1.0 = gesund (diverse Quellen)
    /// Entropy nahe 0.0 = verdächtig (wenige Quellen dominieren)
    pub shard_entropy: DashMap<u64, AtomicF64>,

    /// Cross-Shard-Failures (SourceShard → Failures)
    pub cross_shard_failures: DashMap<u64, AtomicU64>,

    /// Cross-Shard-Successes (für Reputation-Berechnung)
    pub cross_shard_successes: DashMap<u64, AtomicU64>,

    /// Dynamische Shard-Reputation (0.0 = toxisch, 1.0 = gesund)
    pub shard_reputation: DashMap<u64, AtomicF64>,

    /// Bias-Alarme pro Shard
    pub bias_alarms: DashMap<u64, AtomicU64>,

    /// Quarantäne-Status pro Shard
    pub quarantined_shards: DashMap<u64, AtomicBool>,
}
```

### 3.3 ShardMonitorConfig

```rust
pub struct ShardMonitorConfig {
    /// Anzahl der erwarteten Shards
    pub expected_shards: usize,           // 64

    /// Bias-Threshold (50% = Entropy < 50% von Global → Alarm)
    pub bias_threshold: f64,              // 0.5

    /// EWMA Decay-Faktor für Entropy-Updates
    pub entropy_decay: f64,               // 0.9

    /// Failure-Threshold für Quarantäne
    pub quarantine_failure_threshold: u64, // 100

    /// Reputation-Penalty pro Failure
    pub reputation_penalty_per_failure: f64, // 0.1

    /// Reputation-Bonus pro Success
    pub reputation_bonus_per_success: f64,   // 0.01

    /// Max Penalty-Multiplikator für Cross-Shard-Kosten
    pub max_penalty_multiplier: f64,       // 5.0
}
```

### 3.4 Integration in ProtectionState

```rust
pub struct ProtectionState {
    pub anomaly: AnomalyState,
    pub diversity: DiversityState,
    pub quadratic: QuadraticState,
    pub anti_calcification: AntiCalcificationState,
    pub calibration: CalibrationState,

    /// Shard Monitor für horizontale Skalierungssicherheit
    shard_monitor: Option<Arc<ShardMonitor>>,
}

impl ProtectionState {
    /// Berechne System-Health inkl. Shard-Metriken
    pub fn health_score(&self) -> f64 {
        let mut score = 100.0;

        // ... andere Checks ...

        // Shard-Monitor: Quarantinierte Shards und niedrige Reputationen
        if let Some(shard_mon) = &self.shard_monitor {
            let snapshot = shard_mon.snapshot();

            // Quarantinierte Shards sind kritisch
            score -= (snapshot.quarantined_shard_count * 15) as f64;

            // Niedrige Success-Rate ist Warning
            if snapshot.cross_shard_success_rate < 0.9 {
                score -= 10.0;
            }
            if snapshot.cross_shard_success_rate < 0.7 {
                score -= 20.0;
            }
        }

        score.max(0.0).min(100.0)
    }
}
```

---

## 4. Sharding-Metriken

### 4.1 Per-Shard Statistiken

```rust
pub struct ShardStats {
    /// Shard-Index
    pub index: usize,
    /// Aktuelle Anzahl geladener Realms
    pub loaded_count: usize,
    /// LRU-Cache Größe
    pub lru_size: usize,
    /// Anzahl Cache-Hits
    pub cache_hits: u64,
    /// Anzahl Cache-Misses (Lazy Loads)
    pub cache_misses: u64,
    /// Anzahl Evictions
    pub evictions: u64,
    /// Letzte Eviction-Zeit
    pub last_eviction_ms: u64,
}
```

### 4.2 Aggregierte Statistiken

```rust
pub struct ShardingStats {
    /// Konfiguration
    pub num_shards: usize,
    pub max_per_shard: usize,

    /// Aggregierte Metriken
    pub total_loaded_realms: usize,
    pub total_cache_hits: u64,
    pub total_cache_misses: u64,
    pub total_evictions: u64,

    /// Hit-Rate (%)
    pub cache_hit_rate_percent: f64,

    /// Load-Verteilung (Standardabweichung)
    pub load_distribution_stddev: f64,

    /// Per-Shard Details
    pub shards: Vec<ShardStats>,
}
```

### 4.3 Security-Metriken (ShardMonitor)

```rust
pub struct ShardMonitorSnapshot {
    pub global_entropy: f64,
    pub total_shards_monitored: usize,
    pub total_cross_shard_failures: u64,
    pub total_cross_shard_successes: u64,
    pub cross_shard_success_rate: f64,
    pub quarantined_shard_count: usize,
    pub per_shard: Vec<ShardSecuritySnapshot>,
}

pub struct ShardSecuritySnapshot {
    pub shard_id: u64,
    pub activity: u64,
    pub entropy: f64,
    pub reputation: f64,
    pub failures: u64,
    pub successes: u64,
    pub bias_alarms: u64,
    pub is_quarantined: bool,
}
```

---

## 5. Pluto-Synergien: Sharding × Nervensystem

### 5.1 Sharding × Trust

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SHARDING × TRUST                                                           ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Integration:                                                               ║
║   - Shard-Entropy fließt in Trust-Calibration (Κ19)                         ║
║   - Cross-Shard-Reputation beeinflusst Trust-Portabilität (Κ23)             ║
║   - Quarantinierte Shards: Trust-Updates ignoriert (Anomalie-Protection)    ║
║                                                                              ║
║   StateEvent:                                                                ║
║   CrossShardIdentityResolved {                                              ║
║       identity_id: UniversalId,                                             ║
║       source_shard: u64,                                                    ║
║       target_shard: u64,                                                    ║
║   }                                                                          ║
║                                                                              ║
║   Wenn source_shard.reputation < 0.5:                                        ║
║   → Trust-Update mit Dämpfung: trust_delta × reputation                     ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.2 Sharding × Gas/Mana

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SHARDING × GAS/MANA                                                        ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Cross-Shard-Penalty:                                                       ║
║                                                                              ║
║   effective_gas = base_gas × shard_penalty_multiplier                       ║
║                                                                              ║
║   wobei:                                                                     ║
║   shard_penalty_multiplier = 1.0 + (1.0 - shard_reputation) × max_penalty   ║
║                                                                              ║
║   Beispiel:                                                                  ║
║   - shard_reputation = 1.0 → multiplier = 1.0 (keine Strafe)                ║
║   - shard_reputation = 0.5 → multiplier = 3.0 (3× teurer)                   ║
║   - shard_reputation = 0.0 → multiplier = 5.0 (5× teurer)                   ║
║                                                                              ║
║   → Incentiviert gutes Verhalten pro Shard                                  ║
║   → Macht Angriffe exponentiell teurer                                      ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.3 Sharding × Realm

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SHARDING × REALM                                                           ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Realm-Partitionierung:                                                     ║
║                                                                              ║
║   Realm-ID: "eu-realm-gaming-001"                                           ║
║   Shard-Index = FxHash("eu-realm-gaming-001") % 64 = 23                     ║
║                                                                              ║
║   Bedeutung:                                                                 ║
║   - Realm State lebt in Shard 23                                            ║
║   - Events für dieses Realm → Shard 23                                      ║
║   - Crossing-Source für Κ23 = Shard 23                                      ║
║                                                                              ║
║   Child-Realms (Partitions):                                                 ║
║   - "eu-realm-gaming-001/partition-0" → Shard 17                            ║
║   - "eu-realm-gaming-001/partition-1" → Shard 41                            ║
║   - Andere Shards! (Hash-basiert)                                           ║
║                                                                              ║
║   Vorteil: Natürliche Load-Verteilung                                       ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.4 Sharding × PackageManager

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SHARDING × PACKAGEMANAGER                                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Package-Registry ist NICHT sharded (read-heavy, write-rare):              ║
║   → Globaler DashMap für Packages                                           ║
║   → Sync via Gossip (P2P)                                                   ║
║                                                                              ║
║   Aber: Package-Installations sind Realm-scoped:                            ║
║   → Realm A installiert Package X → lebt in Shard(Realm A)                  ║
║   → Keine Cross-Shard-Abhängigkeit                                          ║
║                                                                              ║
║   Content-Deduplication:                                                     ║
║   → Package-Content ist global (CAS-Storage)                                ║
║   → Package-Config ist Realm-local (Shard-lokal)                            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.5 Sharding × P2P

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SHARDING × P2P/GOSSIP                                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Shard-aware Gossip-Topics:                                                 ║
║                                                                              ║
║   /erynoa/shard/{shard_id}/events                                           ║
║   → Events für alle Realms in diesem Shard                                  ║
║                                                                              ║
║   Vorteil: Node kann nur relevante Shards abonnieren                        ║
║                                                                              ║
║   Multi-Node Sharding (Future):                                             ║
║   ┌──────────────┐     ┌──────────────┐     ┌──────────────┐               ║
║   │   Node A     │     │   Node B     │     │   Node C     │               ║
║   │ Shards 0-21  │     │ Shards 22-42 │     │ Shards 43-63 │               ║
║   └──────────────┘     └──────────────┘     └──────────────┘               ║
║                                                                              ║
║   → Shards werden auf Nodes verteilt                                        ║
║   → Cross-Node = Cross-Shard (via P2P)                                      ║
║   → ShardMonitor läuft pro Node                                             ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 5.6 Sharding × Event-Sourcing

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   SHARDING × EVENT-SOURCING                                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Lazy Loading Pipeline:                                                     ║
║                                                                              ║
║   1. get_or_load(realm_id)                                                  ║
║   2. Cache-Miss → Shard-Index berechnen                                     ║
║   3. Storage-Load: RealmBaseSnapshot aus Fjall                              ║
║   4. Event-Replay: Events seit letztem Snapshot                             ║
║   5. State in DashMap einfügen                                              ║
║   6. LRU-Touch für Eviction-Tracking                                        ║
║                                                                              ║
║   Event-Replay für State-Recovery:                                          ║
║   impl RealmSpecificState {                                                  ║
║       pub fn apply_state_event(&self, event: &WrappedStateEvent) {           ║
║           match &event.event {                                               ║
║               StateEvent::MembershipChange { .. } => { ... }                ║
║               StateEvent::TrustUpdate { .. } => { ... }                     ║
║               StateEvent::RealmCrossing { .. } => { ... }                   ║
║           }                                                                  ║
║       }                                                                      ║
║   }                                                                          ║
║                                                                              ║
║   → Stateless Realms möglich (on-demand Reconstruction)                     ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 6. StateGraph-Integration

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                    SHARDING IM STATEGRAPH                                    │
│                                                                              │
│   Sharding ────────────────────────────────────────────────────────────────  │
│     │                                                                        │
│     ├── DependsOn ──► Realm       (Realm-Shard-Mapping)                     │
│     ├── DependsOn ──► Storage     (Lazy Loading, Event-Sourcing)            │
│     │                                                                        │
│     ├── Aggregates ──► Trust      (Shard-Reputation zu Trust-Calibration)   │
│     ├── Aggregates ──► Gas/Mana   (Cross-Shard-Penalties)                   │
│     │                                                                        │
│     ├── Triggers ──► Event        (CrossShardIdentityResolved)              │
│     │                                                                        │
│     ├── Validates ──► Protection  (ShardMonitor in ProtectionState)         │
│     │                                                                        │
│     └── Bidirectional ◄─► P2P     (Shard-Gossip, Multi-Node-Sync)           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Konfigurationsempfehlungen

### 7.1 Development

```rust
ShardingConfig {
    num_shards: 4,
    max_per_shard: 100,
    eviction_interval_secs: 60,
    lru_capacity_per_shard: 150,
    lazy_loading_enabled: false,  // Einfacheres Debugging
    event_replay_on_load: false,
}

ShardMonitorConfig::relaxed()  // Keine Quarantäne bei Tests
```

### 7.2 Production (Single-Node)

```rust
ShardingConfig::production()
// = num_shards: 128, max_per_shard: 50_000, eviction: 5min

ShardMonitorConfig::default()
// = bias_threshold: 0.5, quarantine_threshold: 100
```

### 7.3 Production (Multi-Node, High-Security)

```rust
ShardingConfig {
    num_shards: 256,           // Mehr Shards für bessere Verteilung
    max_per_shard: 25_000,     // Weniger pro Shard (verteilt auf Nodes)
    eviction_interval_secs: 180, // Aggressiveres Eviction
    lru_capacity_per_shard: 30_000,
    lazy_loading_enabled: true,
    event_replay_on_load: true,
}

ShardMonitorConfig::strict()
// = quarantine_threshold: 50, max_penalty: 10×
```

---

## 8. CLI-Befehle (Vorschlag)

```bash
# Sharding-Status
$ erynoa shard status
Shards: 64
Total Loaded Realms: 42,150
Cache Hit Rate: 94.2%
Cross-Shard Success Rate: 99.1%
Quarantined Shards: 0

# Per-Shard Details
$ erynoa shard list
Shard  | Loaded | Hits    | Misses | Evictions | Reputation
-------|--------|---------|--------|-----------|------------
0      | 658    | 12,450  | 234    | 89        | 1.00
1      | 712    | 14,890  | 198    | 102       | 1.00
...
23     | 489    | 8,120   | 890    | 45        | 0.72  ⚠️
...

# Shard analysieren
$ erynoa shard analyze 23
Shard 23 Analysis:
- Entropy: 0.42 (LOW - potential bias)
- Bias Alarms: 3
- Cross-Shard Failures: 28
- Reputation: 0.72
Recommendation: Monitor closely

# Shard quarantinieren (manuell)
$ erynoa shard quarantine 23
⚠️ Shard 23 quarantined. Cross-shard operations blocked.

# Shard freigeben
$ erynoa shard release 23
✓ Shard 23 released from quarantine.

# Eviction erzwingen
$ erynoa shard evict --all
Evicted 12,450 realms from 64 shards.
```

---

## 9. Zusammenfassung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    SHARDING-ARCHITEKTUR: KERNPUNKTE                          ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   ⚡ PERFORMANCE                                                             ║
║      → O(1) Realm-Lookup via FxHash                                         ║
║      → Lock-free DashMap pro Shard                                          ║
║      → LRU-Eviction (nur hot Realms im Memory)                              ║
║      → Background-Eviction-Tasks (non-blocking)                             ║
║                                                                              ║
║   🔄 LAZY LOADING                                                            ║
║      → Cache-Miss → Storage-Load → Event-Replay                             ║
║      → Millionen Realms mit ~10GB RAM                                       ║
║      → Stateless möglich (on-demand Reconstruction)                         ║
║                                                                              ║
║   🛡️ SHARD-MONITOR (Sicherheit)                                             ║
║      → Bias-Detection (Entropy-Tracking)                                    ║
║      → Cross-Shard-Reputation                                               ║
║      → Automatische Quarantäne                                              ║
║      → Gas-Penalties für toxische Shards                                    ║
║                                                                              ║
║   🧠 PLUTO-SYNERGIEN                                                         ║
║      → Trust: Shard-Reputation → Trust-Calibration                          ║
║      → Gas: Cross-Shard-Penalties                                           ║
║      → Realm: Natürliche Load-Verteilung                                    ║
║      → P2P: Shard-aware Gossip                                              ║
║      → Event-Sourcing: State-Recovery via Replay                            ║
║                                                                              ║
║   📊 SKALIERUNG                                                              ║
║      → Single-Node: 1M+ Realms                                              ║
║      → Multi-Node: Horizontale Skalierung via P2P                           ║
║      → Auto-Scaling: CPU-basierte Shard-Anzahl                              ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```
