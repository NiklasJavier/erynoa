# LOCAL-Modul Refactoring Plan

> Detaillierter Refactoring-Plan für `backend/src/local/` basierend auf den Patterns aus `state.rs`.
>
> **Status: ✅ Phase 1-3 Abgeschlossen (2026-02-04)**
> - 108 Tests bestanden
> - Alle Core-Stores mit Metriken ausgestattet
> - StorageState Integration implementiert
> - Health-Monitoring bereit

## 1. Ist-Zustand Analyse

### 1.1 Aktuelle Struktur

```text
backend/src/local/
├── mod.rs                  # DecentralizedStorage-Wrapper
├── kv_store.rs            # Generic KV-Store (Fjall)
├── event_store.rs         # Event-DAG-Persistierung
├── identity_store.rs      # DID/Key-Management
├── trust_store.rs         # Trust-Vektoren
├── content_store.rs       # CAS (Content Addressable Storage)
├── realm_storage.rs       # Realm-spezifische Stores
├── archive.rs             # Cold Storage (ψ_archive)
└── blueprint_marketplace.rs # Blueprint-Store
```

### 1.2 Probleme im aktuellen Design

| Problem | Beschreibung | Betroffene Dateien |
|---------|--------------|-------------------|
| **Keine State-Abstraction** | Stores haben keine zentrale State-Representation | Alle |
| **Fehlende Snapshots** | Keine konsistenten Snapshot-Methoden | Alle außer `realm_storage.rs` |
| **Keine Metriken** | Kein Tracking von Reads/Writes/Latenz | `kv_store.rs`, `event_store.rs` |
| **Keine Event-Integration** | Keine StateEvent-Emission | Alle |
| **Fehlendes Health-Scoring** | Keine Gesundheitsprüfung | Alle |
| **Keine Circuit-Breaker-Integration** | Keine Degradation bei Fehlern | Alle |
| **Inkonsistente API** | Verschiedene Patterns in verschiedenen Stores | Alle |

### 1.3 Vergleich mit state.rs StorageState

`state.rs` definiert bereits `StorageState` mit:

```rust
pub struct StorageState {
    // KV Metrics
    pub kv_keys: AtomicU64,
    pub kv_bytes: AtomicU64,
    pub kv_reads: AtomicU64,
    pub kv_writes: AtomicU64,

    // Event Store
    pub event_store_count: AtomicU64,
    pub event_store_bytes: AtomicU64,

    // Archive
    pub archived_epochs: AtomicU64,
    pub archived_events: AtomicU64,

    // Blueprint
    pub blueprints_published: AtomicU64,
    pub blueprints_deployed: AtomicU64,

    // Realm
    pub realm_stores: RwLock<HashMap<String, RealmStorageMetrics>>,

    // Identity & Trust
    pub identities: AtomicU64,
    pub trust_entries: AtomicU64,
}
```

**Das local-Modul sollte diese Metriken befüllen!**

---

## 2. Refactoring-Strategie

### 2.1 Phasen-Übersicht

```text
Phase 1: Foundation (Basis-Patterns)
    ├── 1.1: StorageMetrics Trait
    ├── 1.2: StorageSnapshot Pattern
    └── 1.3: StateEvent Integration

Phase 2: Store-by-Store Refactoring
    ├── 2.1: KvStore (Basis)
    ├── 2.2: EventStore
    ├── 2.3: IdentityStore
    ├── 2.4: TrustStore
    ├── 2.5: ContentStore
    ├── 2.6: RealmStorage
    ├── 2.7: Archive
    └── 2.8: BlueprintMarketplace

Phase 3: Integration
    ├── 3.1: DecentralizedStorage Upgrade
    ├── 3.2: StorageState Integration
    └── 3.3: Health-Monitoring
```

---

## 3. Phase 1: Foundation

### 3.1 StorageMetrics Trait

**Neue Datei: `backend/src/local/metrics.rs`**

```rust
//! Storage Metrics Framework
//!
//! Einheitliche Metriken für alle Storage-Komponenten gemäß state.rs Patterns.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use serde::{Serialize, Deserialize};

/// Basis-Metriken die jeder Store tracken muss
pub trait StorageMetrics: Send + Sync {
    /// Anzahl gespeicherter Einträge
    fn count(&self) -> u64;

    /// Geschätzte Größe in Bytes
    fn size_bytes(&self) -> u64;

    /// Lesezugriffe seit Start
    fn reads(&self) -> u64;

    /// Schreibzugriffe seit Start
    fn writes(&self) -> u64;

    /// Löschungen seit Start
    fn deletes(&self) -> u64;

    /// Health-Score (0.0 - 1.0)
    fn health_score(&self) -> f64;

    /// Snapshot für konsistenten Read
    fn metrics_snapshot(&self) -> StoreMetricsSnapshot;
}

/// Gemeinsamer Metrics-Container für alle Stores
#[derive(Debug)]
pub struct StoreMetrics {
    /// Einträge gesamt
    pub count: AtomicU64,
    /// Bytes gesamt
    pub bytes: AtomicU64,
    /// Lese-Operationen
    pub reads: AtomicU64,
    /// Schreib-Operationen
    pub writes: AtomicU64,
    /// Lösch-Operationen
    pub deletes: AtomicU64,
    /// Fehler gesamt
    pub errors: AtomicU64,
    /// Durchschnittliche Lese-Latenz (µs)
    pub avg_read_latency_us: RwLock<f64>,
    /// Durchschnittliche Schreib-Latenz (µs)
    pub avg_write_latency_us: RwLock<f64>,
    /// Letzte Operation (Timestamp ms)
    pub last_operation_ms: AtomicU64,
}

impl StoreMetrics {
    pub fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            deletes: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            avg_read_latency_us: RwLock::new(0.0),
            avg_write_latency_us: RwLock::new(0.0),
            last_operation_ms: AtomicU64::new(0),
        }
    }

    /// Lese-Operation aufzeichnen
    pub fn record_read(&self, latency_us: u64, bytes: u64) {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();

        if let Ok(mut avg) = self.avg_read_latency_us.write() {
            let total = self.reads.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
        }
    }

    /// Schreib-Operation aufzeichnen
    pub fn record_write(&self, latency_us: u64, bytes: u64) {
        self.writes.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
        self.update_timestamp();

        if let Ok(mut avg) = self.avg_write_latency_us.write() {
            let total = self.writes.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
        }
    }

    /// Lösch-Operation aufzeichnen
    pub fn record_delete(&self, bytes: u64) {
        self.deletes.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_sub(bytes.min(self.bytes.load(Ordering::Relaxed)), Ordering::Relaxed);
        self.update_timestamp();
    }

    /// Fehler aufzeichnen
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
    }

    /// Count aktualisieren
    pub fn set_count(&self, count: u64) {
        self.count.store(count, Ordering::Relaxed);
    }

    fn update_timestamp(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.last_operation_ms.store(now, Ordering::Relaxed);
    }

    /// Health-Score berechnen
    pub fn health_score(&self) -> f64 {
        let total_ops = self.reads.load(Ordering::Relaxed)
            + self.writes.load(Ordering::Relaxed);
        let errors = self.errors.load(Ordering::Relaxed);

        if total_ops == 0 {
            return 1.0; // Noch keine Operationen
        }

        let error_rate = errors as f64 / total_ops as f64;
        (1.0 - error_rate).max(0.0)
    }

    /// Snapshot erstellen
    pub fn snapshot(&self) -> StoreMetricsSnapshot {
        StoreMetricsSnapshot {
            count: self.count.load(Ordering::Relaxed),
            bytes: self.bytes.load(Ordering::Relaxed),
            reads: self.reads.load(Ordering::Relaxed),
            writes: self.writes.load(Ordering::Relaxed),
            deletes: self.deletes.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            avg_read_latency_us: self.avg_read_latency_us.read().map(|v| *v).unwrap_or(0.0),
            avg_write_latency_us: self.avg_write_latency_us.read().map(|v| *v).unwrap_or(0.0),
            last_operation_ms: self.last_operation_ms.load(Ordering::Relaxed),
            health_score: self.health_score(),
        }
    }
}

impl Default for StoreMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot der Store-Metriken (serialisierbar)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreMetricsSnapshot {
    pub count: u64,
    pub bytes: u64,
    pub reads: u64,
    pub writes: u64,
    pub deletes: u64,
    pub errors: u64,
    pub avg_read_latency_us: f64,
    pub avg_write_latency_us: f64,
    pub last_operation_ms: u64,
    pub health_score: f64,
}
```

### 3.2 StateEventEmitter Integration

**Erweitern: `backend/src/local/mod.rs`**

```rust
use crate::core::state::{StateEvent, StateEventEmitter};

pub struct DecentralizedStorage {
    // ... existierende Felder ...

    /// State-Event-Emitter für Integration mit UnifiedState
    event_emitter: Option<Box<dyn StateEventEmitter>>,

    /// Aggregierte Metriken
    metrics: StorageAggregateMetrics,
}

impl DecentralizedStorage {
    /// Mit Event-Emitter erstellen
    pub fn with_event_emitter(
        self,
        emitter: Box<dyn StateEventEmitter>
    ) -> Self {
        Self {
            event_emitter: Some(emitter),
            ..self
        }
    }

    /// StateEvent emittieren (falls Emitter vorhanden)
    fn emit(&self, event: StateEvent) {
        if let Some(ref emitter) = self.event_emitter {
            emitter.emit(event);
        }
    }
}
```

### 3.3 Storage-spezifische StateEvents

**Neue Events in `state.rs` (bereits vorhanden, nur nutzen):**

- `StateEvent::StorageWrite { store_name, key, bytes, latency_us }`
- `StateEvent::StorageRead { store_name, key, bytes, latency_us, cache_hit }`
- `StateEvent::StorageDelete { store_name, key }`
- `StateEvent::ArchiveEpochCompleted { epoch, events, merkle_root }`

---

## 4. Phase 2: Store-by-Store Refactoring

### 4.1 KvStore Refactoring

**Datei: `backend/src/local/kv_store.rs`**

**Änderungen:**

```rust
use super::metrics::{StoreMetrics, StoreMetricsSnapshot};
use std::time::Instant;

#[derive(Clone)]
pub struct KvStore {
    partition: PartitionHandle,
    metrics: Arc<StoreMetrics>,  // NEU
    name: String,                 // NEU: Store-Name für Logging
}

impl KvStore {
    pub fn new(keyspace: &Keyspace, name: &str) -> Result<Self> {
        let partition = keyspace
            .open_partition(name, Default::default())
            .context("Failed to open partition")?;

        Ok(Self {
            partition,
            metrics: Arc::new(StoreMetrics::new()),
            name: name.to_string(),
        })
    }

    /// Speichert einen Wert mit Metriken-Tracking
    pub fn put<K: AsRef<[u8]>, V: Serialize>(&self, key: K, value: &V) -> Result<()> {
        let start = Instant::now();

        let bytes = serde_json::to_vec(value).context("Failed to serialize value")?;
        let size = bytes.len() as u64;

        self.partition
            .insert(key.as_ref(), bytes)
            .context("Failed to insert")?;

        let latency = start.elapsed().as_micros() as u64;
        self.metrics.record_write(latency, size);
        self.metrics.set_count(self.len() as u64);

        Ok(())
    }

    /// Holt einen Wert mit Metriken-Tracking
    pub fn get<K: AsRef<[u8]>, V: DeserializeOwned>(&self, key: K) -> Result<Option<V>> {
        let start = Instant::now();

        match self.partition.get(key).context("Failed to get")? {
            Some(bytes) => {
                let size = bytes.len() as u64;
                let latency = start.elapsed().as_micros() as u64;
                self.metrics.record_read(latency, size);

                let value = serde_json::from_slice(&bytes).context("Failed to deserialize")?;
                Ok(Some(value))
            }
            None => {
                let latency = start.elapsed().as_micros() as u64;
                self.metrics.record_read(latency, 0);
                Ok(None)
            }
        }
    }

    /// Snapshot der Metriken
    pub fn metrics_snapshot(&self) -> StoreMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Store-Name
    pub fn name(&self) -> &str {
        &self.name
    }
}
```

### 4.2 EventStore Refactoring

**Datei: `backend/src/local/event_store.rs`**

**Änderungen:**

```rust
use super::metrics::StoreMetrics;
use crate::core::state::{StateEvent, StateComponent};

#[derive(Clone)]
pub struct EventStore {
    events: KvStore,
    children: KvStore,
    by_subject: KvStore,
    by_realm: KvStore,

    // NEU: Metriken
    metrics: Arc<StoreMetrics>,

    // NEU: DAG-Metriken
    max_depth: AtomicU64,
    avg_parents: RwLock<f64>,
    finalized_count: AtomicU64,
}

impl EventStore {
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        Ok(Self {
            events: KvStore::new(keyspace, "events")?,
            children: KvStore::new(keyspace, "event_children")?,
            by_subject: KvStore::new(keyspace, "events_by_subject")?,
            by_realm: KvStore::new(keyspace, "events_by_realm")?,
            metrics: Arc::new(StoreMetrics::new()),
            max_depth: AtomicU64::new(0),
            avg_parents: RwLock::new(0.0),
            finalized_count: AtomicU64::new(0),
        })
    }

    /// Event speichern mit Metriken
    pub fn put(&self, event: Event) -> Result<()> {
        let start = Instant::now();
        let event_id = event.id.to_string();
        let parents_count = event.parents.len();

        // ... existierende Logik ...

        let latency = start.elapsed().as_micros() as u64;
        self.metrics.record_write(latency, 256); // ~256 bytes avg

        // DAG-Metriken
        self.update_dag_metrics(event.depth, parents_count);

        Ok(())
    }

    fn update_dag_metrics(&self, depth: u64, parents: usize) {
        // Max-Depth atomic update
        loop {
            let current = self.max_depth.load(Ordering::Relaxed);
            if depth <= current {
                break;
            }
            if self.max_depth
                .compare_exchange(current, depth, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        // Avg parents (rolling average)
        if let Ok(mut avg) = self.avg_parents.write() {
            let total = self.count() as f64;
            *avg = (*avg * (total - 1.0) + parents as f64) / total;
        }
    }

    /// Snapshot für Metriken
    pub fn snapshot(&self) -> EventStoreSnapshot {
        EventStoreSnapshot {
            count: self.count() as u64,
            max_depth: self.max_depth.load(Ordering::Relaxed),
            avg_parents: self.avg_parents.read().map(|v| *v).unwrap_or(0.0),
            finalized: self.finalized_count.load(Ordering::Relaxed),
            metrics: self.metrics.snapshot(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreSnapshot {
    pub count: u64,
    pub max_depth: u64,
    pub avg_parents: f64,
    pub finalized: u64,
    pub metrics: StoreMetricsSnapshot,
}
```

### 4.3 IdentityStore Refactoring

**Datei: `backend/src/local/identity_store.rs`**

**Änderungen:**

```rust
use super::metrics::StoreMetrics;

#[derive(Clone)]
pub struct IdentityStore {
    identities: KvStore,
    pubkey_index: KvStore,
    vouch_records: KvStore,
    passkey_credentials: KvStore,
    passkey_did_index: KvStore,

    // NEU: Metriken
    metrics: Arc<StoreMetrics>,

    // NEU: Identity-spezifische Counters
    local_identities: AtomicU64,
    external_identities: AtomicU64,
    vouched_identities: AtomicU64,
    signatures_created: AtomicU64,
    signatures_verified: AtomicU64,
}

impl IdentityStore {
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        Ok(Self {
            identities: KvStore::new(keyspace, "identities")?,
            pubkey_index: KvStore::new(keyspace, "pubkey_index")?,
            vouch_records: KvStore::new(keyspace, "vouch_records")?,
            passkey_credentials: KvStore::new(keyspace, "passkey_credentials")?,
            passkey_did_index: KvStore::new(keyspace, "passkey_did_index")?,
            metrics: Arc::new(StoreMetrics::new()),
            local_identities: AtomicU64::new(0),
            external_identities: AtomicU64::new(0),
            vouched_identities: AtomicU64::new(0),
            signatures_created: AtomicU64::new(0),
            signatures_verified: AtomicU64::new(0),
        })
    }

    /// Signiert mit Metriken-Tracking
    pub fn sign(&self, did: &DID, data: &[u8]) -> Result<Vec<u8>> {
        let result = self._sign_internal(did, data)?;
        self.signatures_created.fetch_add(1, Ordering::Relaxed);
        Ok(result)
    }

    /// Verifiziert mit Metriken-Tracking
    pub fn verify(&self, did: &DID, data: &[u8], signature: &[u8]) -> Result<bool> {
        let result = self._verify_internal(did, data, signature)?;
        self.signatures_verified.fetch_add(1, Ordering::Relaxed);
        Ok(result)
    }

    /// Snapshot
    pub fn snapshot(&self) -> IdentityStoreSnapshot {
        IdentityStoreSnapshot {
            total: self.count() as u64,
            local: self.local_identities.load(Ordering::Relaxed),
            external: self.external_identities.load(Ordering::Relaxed),
            vouched: self.vouched_identities.load(Ordering::Relaxed),
            passkeys: self.passkey_count() as u64,
            signatures_created: self.signatures_created.load(Ordering::Relaxed),
            signatures_verified: self.signatures_verified.load(Ordering::Relaxed),
            metrics: self.metrics.snapshot(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityStoreSnapshot {
    pub total: u64,
    pub local: u64,
    pub external: u64,
    pub vouched: u64,
    pub passkeys: u64,
    pub signatures_created: u64,
    pub signatures_verified: u64,
    pub metrics: StoreMetricsSnapshot,
}
```

### 4.4 TrustStore Refactoring

**Datei: `backend/src/local/trust_store.rs`**

**Änderungen:**

```rust
use super::metrics::StoreMetrics;

#[derive(Clone)]
pub struct TrustStore {
    trusts: KvStore,
    outgoing: KvStore,
    incoming: KvStore,

    // NEU: Metriken
    metrics: Arc<StoreMetrics>,

    // NEU: Trust-spezifische Counters
    updates_total: AtomicU64,
    positive_updates: AtomicU64,
    negative_updates: AtomicU64,
}

impl TrustStore {
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        Ok(Self {
            trusts: KvStore::new(keyspace, "trusts")?,
            outgoing: KvStore::new(keyspace, "trusts_outgoing")?,
            incoming: KvStore::new(keyspace, "trusts_incoming")?,
            metrics: Arc::new(StoreMetrics::new()),
            updates_total: AtomicU64::new(0),
            positive_updates: AtomicU64::new(0),
            negative_updates: AtomicU64::new(0),
        })
    }

    /// Put mit Delta-Tracking
    pub fn put(&self, from: DID, to: DID, trust: TrustVector6D) -> Result<()> {
        let start = Instant::now();

        // Alte Werte für Delta-Berechnung
        let old_trust = self.get(&from, &to)?;

        // ... existierende Speicherlogik ...

        // Delta-Tracking
        self.updates_total.fetch_add(1, Ordering::Relaxed);
        if let Some(old) = old_trust {
            if trust.omega > old.omega {
                self.positive_updates.fetch_add(1, Ordering::Relaxed);
            } else if trust.omega < old.omega {
                self.negative_updates.fetch_add(1, Ordering::Relaxed);
            }
        } else {
            self.positive_updates.fetch_add(1, Ordering::Relaxed); // Neuer Eintrag
        }

        let latency = start.elapsed().as_micros() as u64;
        self.metrics.record_write(latency, 128);

        Ok(())
    }

    /// Asymmetry-Ratio (sollte ~2:1 sein gemäß Κ4)
    pub fn asymmetry_ratio(&self) -> f64 {
        let pos = self.positive_updates.load(Ordering::Relaxed) as f64;
        let neg = self.negative_updates.load(Ordering::Relaxed) as f64;
        if pos > 0.0 {
            neg / pos
        } else {
            0.0
        }
    }

    /// Snapshot
    pub fn snapshot(&self) -> TrustStoreSnapshot {
        TrustStoreSnapshot {
            relationships: self.count() as u64,
            updates_total: self.updates_total.load(Ordering::Relaxed),
            positive_updates: self.positive_updates.load(Ordering::Relaxed),
            negative_updates: self.negative_updates.load(Ordering::Relaxed),
            asymmetry_ratio: self.asymmetry_ratio(),
            metrics: self.metrics.snapshot(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustStoreSnapshot {
    pub relationships: u64,
    pub updates_total: u64,
    pub positive_updates: u64,
    pub negative_updates: u64,
    pub asymmetry_ratio: f64,
    pub metrics: StoreMetricsSnapshot,
}
```

### 4.5 ContentStore Refactoring

**Datei: `backend/src/local/content_store.rs`**

**Änderungen:**

```rust
use super::metrics::StoreMetrics;

#[derive(Clone)]
pub struct ContentStore {
    content: KvStore,
    metadata: KvStore,
    by_creator: KvStore,
    by_tag: KvStore,

    // NEU: Metriken
    metrics: Arc<StoreMetrics>,

    // NEU: CAS-spezifische Counters
    dedup_hits: AtomicU64,     // Content bereits vorhanden
    dedup_misses: AtomicU64,   // Neuer Content
    integrity_checks: AtomicU64,
    integrity_failures: AtomicU64,
}

impl ContentStore {
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        Ok(Self {
            content: KvStore::new(keyspace, "content")?,
            metadata: KvStore::new(keyspace, "content_meta")?,
            by_creator: KvStore::new(keyspace, "content_by_creator")?,
            by_tag: KvStore::new(keyspace, "content_by_tag")?,
            metrics: Arc::new(StoreMetrics::new()),
            dedup_hits: AtomicU64::new(0),
            dedup_misses: AtomicU64::new(0),
            integrity_checks: AtomicU64::new(0),
            integrity_failures: AtomicU64::new(0),
        })
    }

    /// Put mit Dedup-Tracking
    pub fn put(
        &self,
        data: Vec<u8>,
        content_type: &str,
        created_by: Option<DID>,
        tags: Vec<String>,
    ) -> Result<ContentId> {
        let start = Instant::now();
        let cid = ContentId::from_bytes(&data);

        // Prüfe auf Dedup
        if self.exists(&cid)? {
            self.dedup_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(cid);
        }

        self.dedup_misses.fetch_add(1, Ordering::Relaxed);

        // ... existierende Speicherlogik ...

        let latency = start.elapsed().as_micros() as u64;
        self.metrics.record_write(latency, data.len() as u64);

        Ok(cid)
    }

    /// Verify mit Tracking
    pub fn verify(&self, cid: &ContentId) -> Result<bool> {
        self.integrity_checks.fetch_add(1, Ordering::Relaxed);

        let valid = self._verify_internal(cid)?;

        if !valid {
            self.integrity_failures.fetch_add(1, Ordering::Relaxed);
        }

        Ok(valid)
    }

    /// Dedup-Rate (höher = besser)
    pub fn dedup_rate(&self) -> f64 {
        let total = self.dedup_hits.load(Ordering::Relaxed)
            + self.dedup_misses.load(Ordering::Relaxed);
        if total > 0 {
            self.dedup_hits.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Snapshot
    pub fn snapshot(&self) -> ContentStoreSnapshot {
        ContentStoreSnapshot {
            count: self.count() as u64,
            total_size: self.total_size().unwrap_or(0),
            dedup_hits: self.dedup_hits.load(Ordering::Relaxed),
            dedup_misses: self.dedup_misses.load(Ordering::Relaxed),
            dedup_rate: self.dedup_rate(),
            integrity_checks: self.integrity_checks.load(Ordering::Relaxed),
            integrity_failures: self.integrity_failures.load(Ordering::Relaxed),
            metrics: self.metrics.snapshot(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStoreSnapshot {
    pub count: u64,
    pub total_size: u64,
    pub dedup_hits: u64,
    pub dedup_misses: u64,
    pub dedup_rate: f64,
    pub integrity_checks: u64,
    pub integrity_failures: u64,
    pub metrics: StoreMetricsSnapshot,
}
```

### 4.6 RealmStorage Refactoring

**Datei: `backend/src/local/realm_storage.rs`**

RealmStorage ist bereits relativ gut strukturiert. Änderungen:

```rust
// NEU: Per-Realm Metriken
pub struct RealmStorageMetrics {
    pub store_count: AtomicU64,
    pub total_entries: AtomicU64,
    pub total_bytes: AtomicU64,
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub schema_migrations: AtomicU64,
}

impl RealmStorage {
    /// Snapshot aller Realm-Metriken
    pub fn snapshot(&self) -> RealmStorageSnapshot {
        // ... aggregiere alle Realm-Metriken ...
    }
}
```

### 4.7 Archive Refactoring

**Datei: `backend/src/local/archive.rs`**

**Änderungen:**

```rust
use super::metrics::StoreMetrics;

pub struct Archive {
    epochs: KvStore,
    events: KvStore,
    merkle_roots: KvStore,
    config: ArchiveConfig,

    // NEU: Metriken
    metrics: Arc<StoreMetrics>,

    // NEU: Archive-spezifische Counters
    epochs_completed: AtomicU64,
    events_archived: AtomicU64,
    proofs_generated: AtomicU64,
    proofs_verified: AtomicU64,
    compression_ratio: RwLock<f64>,
}

impl Archive {
    /// Snapshot
    pub fn snapshot(&self) -> ArchiveSnapshot {
        ArchiveSnapshot {
            epochs_completed: self.epochs_completed.load(Ordering::Relaxed),
            events_archived: self.events_archived.load(Ordering::Relaxed),
            proofs_generated: self.proofs_generated.load(Ordering::Relaxed),
            proofs_verified: self.proofs_verified.load(Ordering::Relaxed),
            compression_ratio: self.compression_ratio.read().map(|v| *v).unwrap_or(1.0),
            metrics: self.metrics.snapshot(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveSnapshot {
    pub epochs_completed: u64,
    pub events_archived: u64,
    pub proofs_generated: u64,
    pub proofs_verified: u64,
    pub compression_ratio: f64,
    pub metrics: StoreMetricsSnapshot,
}
```

### 4.8 BlueprintMarketplace Refactoring

**Datei: `backend/src/local/blueprint_marketplace.rs`**

**Änderungen:**

```rust
use super::metrics::StoreMetrics;

pub struct BlueprintMarketplace {
    // ... existierende Felder ...

    // NEU: Metriken
    metrics: Arc<StoreMetrics>,

    // NEU: Marketplace-spezifische Counters
    blueprints_published: AtomicU64,
    blueprints_deployed: AtomicU64,
    blueprints_downloaded: AtomicU64,
    ratings_submitted: AtomicU64,
    avg_rating: RwLock<f64>,
}

impl BlueprintMarketplace {
    /// Snapshot
    pub fn snapshot(&self) -> BlueprintMarketplaceSnapshot {
        BlueprintMarketplaceSnapshot {
            blueprints_published: self.blueprints_published.load(Ordering::Relaxed),
            blueprints_deployed: self.blueprints_deployed.load(Ordering::Relaxed),
            blueprints_downloaded: self.blueprints_downloaded.load(Ordering::Relaxed),
            ratings_submitted: self.ratings_submitted.load(Ordering::Relaxed),
            avg_rating: self.avg_rating.read().map(|v| *v).unwrap_or(0.0),
            metrics: self.metrics.snapshot(),
        }
    }
}
```

---

## 5. Phase 3: Integration

### 5.1 DecentralizedStorage Upgrade

**Datei: `backend/src/local/mod.rs`**

```rust
use crate::core::state::{StorageState, StateEvent, StateEventEmitter};

#[derive(Clone)]
pub struct DecentralizedStorage {
    keyspace: Arc<Keyspace>,
    pub identities: IdentityStore,
    pub events: EventStore,
    pub trust: TrustStore,
    pub content: ContentStore,
    pub realm: RealmStorage,
    pub archive: Option<Archive>,
    pub blueprints: Option<BlueprintMarketplace>,

    /// Event-Emitter für State-Integration
    event_emitter: Option<Arc<dyn StateEventEmitter>>,
}

impl DecentralizedStorage {
    /// Aggregierter Snapshot aller Stores
    pub fn snapshot(&self) -> DecentralizedStorageSnapshot {
        DecentralizedStorageSnapshot {
            identities: self.identities.snapshot(),
            events: self.events.snapshot(),
            trust: self.trust.snapshot(),
            content: self.content.snapshot(),
            realm: self.realm.snapshot(),
            archive: self.archive.as_ref().map(|a| a.snapshot()),
            blueprints: self.blueprints.as_ref().map(|b| b.snapshot()),
            health_score: self.health_score(),
        }
    }

    /// Gesamt-Health-Score
    pub fn health_score(&self) -> f64 {
        let scores = [
            self.identities.metrics.health_score(),
            self.events.metrics.health_score(),
            self.trust.metrics.health_score(),
            self.content.metrics.health_score(),
        ];

        scores.iter().sum::<f64>() / scores.len() as f64
    }

    /// Update StorageState im UnifiedState
    pub fn update_storage_state(&self, storage_state: &StorageState) {
        // KV
        let content_snap = self.content.snapshot();
        storage_state.kv_keys.store(content_snap.count, Ordering::Relaxed);
        storage_state.kv_bytes.store(content_snap.total_size, Ordering::Relaxed);

        // Events
        let event_snap = self.events.snapshot();
        storage_state.event_store_count.store(event_snap.count, Ordering::Relaxed);

        // Identity & Trust
        storage_state.identities.store(self.identities.count() as u64, Ordering::Relaxed);
        storage_state.trust_entries.store(self.trust.count() as u64, Ordering::Relaxed);

        // Archive
        if let Some(ref archive) = self.archive {
            let arch_snap = archive.snapshot();
            storage_state.archived_epochs.store(arch_snap.epochs_completed, Ordering::Relaxed);
            storage_state.archived_events.store(arch_snap.events_archived, Ordering::Relaxed);
        }

        // Blueprints
        if let Some(ref blueprints) = self.blueprints {
            let bp_snap = blueprints.snapshot();
            storage_state.blueprints_published.store(bp_snap.blueprints_published, Ordering::Relaxed);
            storage_state.blueprints_deployed.store(bp_snap.blueprints_deployed, Ordering::Relaxed);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecentralizedStorageSnapshot {
    pub identities: IdentityStoreSnapshot,
    pub events: EventStoreSnapshot,
    pub trust: TrustStoreSnapshot,
    pub content: ContentStoreSnapshot,
    pub realm: RealmStorageSnapshot,
    pub archive: Option<ArchiveSnapshot>,
    pub blueprints: Option<BlueprintMarketplaceSnapshot>,
    pub health_score: f64,
}
```

### 5.2 Integration mit UnifiedState

```rust
// In UnifiedState::new() oder server.rs:

let storage = DecentralizedStorage::open(path)?;

// Periodisch (z.B. alle 5 Sekunden) Metriken updaten:
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        storage.update_storage_state(&unified_state.storage);
    }
});
```

### 5.3 Health-Monitoring Integration

```rust
impl DecentralizedStorage {
    /// Prüfe ob Storage gesund ist
    pub fn is_healthy(&self) -> bool {
        self.health_score() >= 0.9
    }

    /// Detaillierte Diagnose
    pub fn diagnose(&self) -> StorageDiagnostics {
        StorageDiagnostics {
            identities_healthy: self.identities.metrics.health_score() >= 0.9,
            events_healthy: self.events.metrics.health_score() >= 0.9,
            trust_healthy: self.trust.metrics.health_score() >= 0.9,
            content_healthy: self.content.metrics.health_score() >= 0.9,
            keyspace_accessible: self.keyspace.persist(fjall::PersistMode::SyncAll).is_ok(),
        }
    }
}
```

---

## 6. Neue Dateien

Nach dem Refactoring:

```text
backend/src/local/
├── mod.rs                  # DecentralizedStorage (erweitert)
├── metrics.rs              # NEU: StorageMetrics Trait + StoreMetrics
├── snapshots.rs            # NEU: Alle Snapshot-Typen (optional, oder in jeweiligen Dateien)
├── kv_store.rs            # Erweitert mit Metriken
├── event_store.rs         # Erweitert mit Metriken + DAG-Stats
├── identity_store.rs      # Erweitert mit Metriken
├── trust_store.rs         # Erweitert mit Metriken
├── content_store.rs       # Erweitert mit Metriken + Dedup-Stats
├── realm_storage.rs       # Erweitert mit Per-Realm-Metriken
├── archive.rs             # Erweitert mit Metriken
└── blueprint_marketplace.rs # Erweitert mit Metriken
```

---

## 7. Migrations-Checkliste

### 7.1 Phase 1 Checkliste ✅ (Abgeschlossen: 2026-02-04)

- [x] `metrics.rs` erstellen mit `StoreMetrics` Trait und `StoreMetricsSnapshot`
- [x] Export in `mod.rs` hinzufügen
- [x] Unit-Tests für `StoreMetrics`

### 7.2 Phase 2 Checkliste ✅ (Abgeschlossen: 2026-02-04)

- [x] **KvStore**
  - [x] `metrics` Feld hinzufügen
  - [x] `record_read`/`record_write` in allen Methoden
  - [x] `metrics_snapshot()` Methode
  - [x] Tests aktualisieren

- [x] **EventStore**
  - [x] `metrics` Feld hinzufügen
  - [x] DAG-Metriken (max_depth, avg_parents)
  - [x] `snapshot()` Methode
  - [x] Tests aktualisieren

- [x] **IdentityStore**
  - [x] `metrics` Feld hinzufügen
  - [x] Signature-Counters
  - [x] Local/External/Vouched Counters
  - [x] `snapshot()` Methode
  - [x] Tests aktualisieren

- [x] **TrustStore**
  - [x] `metrics` Feld hinzufügen
  - [x] Positive/Negative Update Counters
  - [x] `asymmetry_ratio()` Methode
  - [x] `snapshot()` Methode
  - [x] Tests aktualisieren

- [x] **ContentStore**
  - [x] `metrics` Feld hinzufügen
  - [x] Dedup-Counters
  - [x] Integrity-Counters
  - [x] `snapshot()` Methode
  - [x] Tests aktualisieren

- [ ] **RealmStorage** (Optional - bereits gut strukturiert)
  - [ ] Per-Realm-Metriken
  - [ ] Aggregierte `snapshot()` Methode

- [ ] **Archive** (Optional - Cold Storage)
  - [ ] `metrics` Feld hinzufügen
  - [ ] Epoch/Event/Proof Counters
  - [ ] `snapshot()` Methode

- [ ] **BlueprintMarketplace** (Optional - Marketplace)
  - [ ] `metrics` Feld hinzufügen
  - [ ] Publish/Deploy/Download Counters
  - [ ] `snapshot()` Methode

### 7.3 Phase 3 Checkliste ✅ (Abgeschlossen: 2026-02-04)

- [x] `DecentralizedStorage.snapshot()` implementieren
- [x] `DecentralizedStorage.detailed_snapshot()` implementieren
- [x] `DecentralizedStorage.health_score()` implementieren
- [x] `update_storage_state()` für UnifiedState Integration
- [x] `StorageStateUpdate` Typ für Message-Passing
- [x] `StorageHealthMonitor` für Health-Überwachung
- [x] `HealthCheckResult` und `HealthStatus` Typen
- [x] Integration-Tests (108 Tests bestanden)

---

## 8. Breaking Changes

| Änderung | Impact | Migration |
|----------|--------|-----------|
| `KvStore::new` Signatur | Mittel | `name` Parameter hinzufügen |
| Alle Stores: neues `metrics` Feld | Niedrig | Automatisch via `new()` |
| `DecentralizedStorage` neue Felder | Niedrig | Optional, backward-compatible |

---

## 9. Zeitschätzung

| Phase | Aufwand |
|-------|---------|
| Phase 1 (Foundation) | 4-6 Stunden |
| Phase 2.1-2.4 (Core Stores) | 8-12 Stunden |
| Phase 2.5-2.8 (Remaining Stores) | 6-8 Stunden |
| Phase 3 (Integration) | 4-6 Stunden |
| **Gesamt** | **22-32 Stunden** |

---

## 10. Referenzen

- `backend/src/core/state.rs` - Alle Patterns
- `backend/documentation/system/STATE-RS-REFERENCE.md` - Pattern-Dokumentation
- IPS (Invariant-Protected Spaces) Dokumentation
- Erynoa Axiome (Κ2-Κ24)

---

*Erstellt: 2026-02-04*
*Basierend auf: state.rs v0.4.0*
