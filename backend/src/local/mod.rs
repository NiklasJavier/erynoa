//! Dezentrale Storage-Schicht
//!
//! Embedded Key-Value Store basierend auf Fjall.
//! Ersetzt PostgreSQL für eine Single-Binary Architektur.
//!
//! ## Intelligente Speicherverwaltung
//!
//! Das System nutzt nur wenige globale Partitionen mit intelligenter Prefixing-Strategie:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                     STORAGE PARTITIONEN                                     │
//! ├─────────────┬─────────────┬─────────────┬─────────────┬─────────────────────┤
//! │ identities  │   events    │    trust    │   content   │    realm_storage    │
//! │ (DIDs,Keys) │  (DAG)      │ (Vektoren)  │  (CAS)      │  (Dynamische Stores)│
//! └─────────────┴─────────────┴─────────────┴─────────────┴─────────────────────┘
//!                      │                                         │
//!       ┌──────────────┴──────────────┐         ┌───────────────┴───────────────┐
//!       │      Cold Storage Archive   │         │      Blueprint Marketplace     │
//!       │   (ψ_archive Morphismus)    │         │   (Dezentraler Template-Store) │
//!       └─────────────────────────────┘         └───────────────────────────────┘
//! ```
//!
//! ## Metriken-Framework (Phase 1)
//!
//! Alle Stores implementieren einheitliche Metriken gemäß `state.rs` Patterns:
//! - `StorageMetrics` Trait für konsistente API
//! - `StoreMetrics` für Atomic-Counter und Latenz-Tracking
//! - `StoreMetricsSnapshot` für konsistente Reads
//! - Health-Score-Berechnung pro Store

pub mod archive;
pub mod blueprint_marketplace;
mod content_store;
mod event_store;
mod identity_store;
mod kv_store;
pub mod metrics;
pub mod realm_storage;
mod trust_store;

pub use blueprint_marketplace::{
    // Blueprint-Typen
    Blueprint,
    BlueprintBuilder,
    BlueprintCategory,
    BlueprintDeployment,
    BlueprintId,
    BlueprintLicense,
    // Marketplace
    BlueprintMarketplace,
    BlueprintPolicy,
    BlueprintRating,
    BlueprintSaga,
    // Metriken & Stats
    BlueprintStats,
    BlueprintStore,
    CreatorAnalytics,
    DeploymentResult,
    MarketplaceConfig,
    MarketplaceStats,
    NoveltyCalculator,
    PolicyType,
    // Ergebnisse
    PublishResult,
    RatingResult,
    SagaAction,
    SagaStep,
    SearchQuery,
    SearchResult,
    SemVer,
};
pub use content_store::{ContentId, ContentMetadata, ContentStore, ContentStoreSnapshot, StoredContent};
pub use event_store::{EventStore, EventStoreSnapshot, StoredEvent};
pub use identity_store::{IdentityStore, IdentityStoreSnapshot, StoredIdentity};
pub use kv_store::KvStore;
pub use trust_store::TrustStoreSnapshot;
pub use realm_storage::{
    PrefixBuilder,
    RealmStorage,
    RealmStorageConfig,
    // Schema-Evolution-Typen
    SchemaChange,
    SchemaChangeStatus,
    SchemaChangelogEntry,
    SchemaEvolutionResult,
    // Schema-Typen
    SchemaFieldType,
    SchemaHistory,
    StoreOptions,
    StoreSchema,
    StoreTemplate,
    StoreType,
    StoreValue,
};
pub use trust_store::{StoredTrust, TrustStore};

// Cold Storage Archive (ψ_archive Morphismus)
pub use archive::{
    Archive, ArchiveConfig, ArchiveError, ArchiveResult, ArchiveStats, EpochMetadata, MerkleProof,
};

// Storage Metrics Framework (Phase 1)
pub use metrics::{
    AggregateMetricsSnapshot, StorageMetrics, StoreMetrics, StoreMetricsSnapshot,
};

use anyhow::Result;
use fjall::Keyspace;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Dezentraler Storage-Manager
///
/// Verwaltet alle lokalen Daten in einem einzigen Verzeichnis.
/// Kein externer Datenbank-Server erforderlich.
///
/// ## Partitionen
///
/// - `identities`: DIDs und kryptographische Schlüssel
/// - `events`: Kausaler Event-DAG
/// - `trust`: Trust-Vektoren zwischen Entitäten
/// - `content`: Content Addressable Storage (BLAKE3)
/// - `realm_storage`: Dynamische Realm-Stores mit Prefixing
#[derive(Clone)]
pub struct DecentralizedStorage {
    /// Fjall Keyspace Instance
    keyspace: Arc<Keyspace>,
    /// Identity Store (DIDs, Keys)
    pub identities: IdentityStore,
    /// Event Store (DAG)
    pub events: EventStore,
    /// Trust Store (Trust-Vektoren)
    pub trust: TrustStore,
    /// Content Addressable Storage
    pub content: ContentStore,
    /// Realm Storage (Dynamische Stores)
    pub realm: RealmStorage,
}

impl DecentralizedStorage {
    /// Öffnet oder erstellt den Storage im angegebenen Verzeichnis
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let keyspace = Arc::new(fjall::Config::new(path.as_ref().join("data")).open()?);

        let identities = IdentityStore::new(&keyspace)?;
        let events = EventStore::new(&keyspace)?;
        let trust = TrustStore::new(&keyspace)?;
        let content = ContentStore::new(&keyspace)?;
        let realm = RealmStorage::new(&keyspace, RealmStorageConfig::default())?;

        Ok(Self {
            keyspace,
            identities,
            events,
            trust,
            content,
            realm,
        })
    }

    /// Öffnet einen temporären In-Memory Storage (für Tests)
    pub fn open_temporary() -> Result<Self> {
        let folder = tempfile::tempdir()?;
        let keyspace = Arc::new(fjall::Config::new(folder.path()).open()?);

        let identities = IdentityStore::new(&keyspace)?;
        let events = EventStore::new(&keyspace)?;
        let trust = TrustStore::new(&keyspace)?;
        let content = ContentStore::new(&keyspace)?;
        let realm = RealmStorage::new(&keyspace, RealmStorageConfig::default())?;

        Ok(Self {
            keyspace,
            identities,
            events,
            trust,
            content,
            realm,
        })
    }

    /// Öffnet Storage mit benutzerdefinierter Realm-Konfiguration
    pub fn open_with_config<P: AsRef<Path>>(
        path: P,
        realm_config: RealmStorageConfig,
    ) -> Result<Self> {
        let keyspace = Arc::new(fjall::Config::new(path.as_ref().join("data")).open()?);

        let identities = IdentityStore::new(&keyspace)?;
        let events = EventStore::new(&keyspace)?;
        let trust = TrustStore::new(&keyspace)?;
        let content = ContentStore::new(&keyspace)?;
        let realm = RealmStorage::new(&keyspace, realm_config)?;

        Ok(Self {
            keyspace,
            identities,
            events,
            trust,
            content,
            realm,
        })
    }

    /// Health Check
    pub async fn ping(&self) -> Result<()> {
        // Fjall persist ist synchron, aber wir wrappen es für Konsistenz
        self.keyspace.persist(fjall::PersistMode::SyncAll)?;
        Ok(())
    }

    /// Flush alle Daten auf die Festplatte
    pub fn flush(&self) -> Result<()> {
        self.keyspace.persist(fjall::PersistMode::SyncAll)?;
        Ok(())
    }

    /// Gibt die Anzahl der gespeicherten Identitäten zurück
    pub fn identity_count(&self) -> usize {
        self.identities.count()
    }

    /// Gibt die Anzahl der gespeicherten Events zurück
    pub fn event_count(&self) -> usize {
        self.events.count()
    }

    /// Gibt die Anzahl der Trust-Beziehungen zurück
    pub fn trust_count(&self) -> usize {
        self.trust.count()
    }

    /// Gibt die Anzahl der gespeicherten Contents zurück
    pub fn content_count(&self) -> usize {
        self.content.count()
    }

    /// Gibt die Anzahl der Realm-Stores zurück
    pub fn realm_store_count(&self, realm_id: &crate::domain::RealmId) -> usize {
        self.realm
            .list_stores(realm_id)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// Zugriff auf den Keyspace (für fortgeschrittene Operationen)
    pub fn keyspace(&self) -> &Arc<Keyspace> {
        &self.keyspace
    }

    // ─────────────────────────────────────────────────────────────────────────
    // METRICS & HEALTH (Phase 2)
    // ─────────────────────────────────────────────────────────────────────────

    /// Erstelle aggregierten Snapshot aller Stores (Basis-Version)
    ///
    /// Liefert konsistente Metriken für Monitoring und Diagnostik.
    pub fn snapshot(&self) -> DecentralizedStorageSnapshot {
        DecentralizedStorageSnapshot {
            identity_count: self.identities.count() as u64,
            event_count: self.events.count() as u64,
            trust_count: self.trust.count() as u64,
            content_count: self.content.count() as u64,
            health_score: self.health_score(),
            keyspace_accessible: self.is_keyspace_accessible(),
        }
    }

    /// Erstelle detaillierten Snapshot aller Stores (Phase 2)
    ///
    /// Enthält alle store-spezifischen Metriken.
    pub fn detailed_snapshot(&self) -> DetailedStorageSnapshot {
        DetailedStorageSnapshot {
            identities: self.identities.snapshot(),
            events: self.events.snapshot(),
            trust: self.trust.snapshot(),
            content: self.content.snapshot(),
            health_score: self.health_score(),
            keyspace_accessible: self.is_keyspace_accessible(),
        }
    }

    /// Gesamt-Health-Score über alle Stores (0.0 - 1.0)
    ///
    /// Berechnet als Durchschnitt der Einzelscores (Phase 2).
    /// Gibt 0.0 zurück wenn Keyspace nicht erreichbar.
    pub fn health_score(&self) -> f64 {
        if !self.is_keyspace_accessible() {
            return 0.0;
        }

        // Aggregiere Health-Scores aller Stores (Phase 2)
        let scores = [
            self.identities.health_score(),
            self.events.health_score(),
            self.trust.health_score(),
            self.content.health_score(),
        ];

        scores.iter().sum::<f64>() / scores.len() as f64
    }

    /// Prüfe ob der Keyspace erreichbar ist
    pub fn is_keyspace_accessible(&self) -> bool {
        self.keyspace.persist(fjall::PersistMode::Buffer).is_ok()
    }

    /// Ist das Storage-System gesund?
    pub fn is_healthy(&self) -> bool {
        self.health_score() >= 0.9
    }

    /// Detaillierte Diagnose (Phase 2 erweitert)
    pub fn diagnose(&self) -> StorageDiagnostics {
        let keyspace_ok = self.is_keyspace_accessible();
        StorageDiagnostics {
            keyspace_accessible: keyspace_ok,
            identity_store_ok: keyspace_ok && self.identities.is_healthy(),
            event_store_ok: keyspace_ok && self.events.is_healthy(),
            trust_store_ok: keyspace_ok && self.trust.is_healthy(),
            content_store_ok: keyspace_ok && self.content.is_healthy(),
            health_score: self.health_score(),
        }
    }

    /// Aggregierte Metriken für alle Stores
    pub fn aggregate_metrics(&self) -> AggregateMetricsSnapshot {
        let stores = vec![
            ("identities".to_string(), self.identities.metrics().snapshot()),
            ("events".to_string(), self.events.metrics().snapshot()),
            ("trust".to_string(), self.trust.metrics().snapshot()),
            ("content".to_string(), self.content.metrics().snapshot()),
        ];

        AggregateMetricsSnapshot::from_stores(&stores)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PHASE 3: StorageState Integration
    // ─────────────────────────────────────────────────────────────────────────

    /// Synchronisiert lokale Metriken mit dem zentralen StorageState
    ///
    /// Diese Methode aktualisiert alle Atomic-Felder im StorageState
    /// basierend auf den aktuellen lokalen Store-Metriken.
    ///
    /// # Verwendung
    ///
    /// ```rust,ignore
    /// use crate::core::state::StorageState;
    ///
    /// let storage = DecentralizedStorage::open(path)?;
    /// let storage_state = StorageState::new();
    ///
    /// // Periodisch aufrufen (z.B. alle 5 Sekunden)
    /// storage.update_storage_state(&storage_state);
    /// ```
    pub fn update_storage_state(&self, storage_state: &crate::core::state::StorageState) {
        use std::sync::atomic::Ordering;

        // KV Store Metriken (aus Content Store, da das der primäre KV-User ist)
        let content_snap = self.content.snapshot();
        storage_state
            .kv_keys
            .store(content_snap.count, Ordering::Relaxed);
        storage_state
            .kv_bytes
            .store(content_snap.total_size, Ordering::Relaxed);
        storage_state
            .kv_reads
            .store(content_snap.metrics.reads, Ordering::Relaxed);
        storage_state
            .kv_writes
            .store(content_snap.metrics.writes, Ordering::Relaxed);

        // Event Store
        let event_snap = self.events.snapshot();
        storage_state
            .event_store_count
            .store(event_snap.count, Ordering::Relaxed);
        storage_state
            .event_store_bytes
            .store(event_snap.metrics.bytes, Ordering::Relaxed);

        // Identity & Trust
        let identity_snap = self.identities.snapshot();
        storage_state
            .identities
            .store(identity_snap.total, Ordering::Relaxed);

        let trust_snap = self.trust.snapshot();
        storage_state
            .trust_entries
            .store(trust_snap.relationships, Ordering::Relaxed);

        // Realm Count
        // Note: realm.store_count() würde ein realm_id brauchen,
        // daher nutzen wir hier eine Schätzung basierend auf identity_count
        // TODO: Bessere Realm-Metrik implementieren
    }

    /// Erstellt einen StorageStateUpdate für die Integration
    ///
    /// Gibt ein Struct zurück, das alle Werte enthält, die an StorageState
    /// gesendet werden sollen. Nützlich für logging/debugging.
    pub fn storage_state_update(&self) -> StorageStateUpdate {
        let content_snap = self.content.snapshot();
        let event_snap = self.events.snapshot();
        let identity_snap = self.identities.snapshot();
        let trust_snap = self.trust.snapshot();

        StorageStateUpdate {
            kv_keys: content_snap.count,
            kv_bytes: content_snap.total_size,
            kv_reads: content_snap.metrics.reads,
            kv_writes: content_snap.metrics.writes,
            event_store_count: event_snap.count,
            event_store_bytes: event_snap.metrics.bytes,
            identities: identity_snap.total,
            trust_entries: trust_snap.relationships,
            health_score: self.health_score(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SNAPSHOT & DIAGNOSTICS TYPES
// ─────────────────────────────────────────────────────────────────────────────

/// Aggregierter Snapshot aller Storage-Komponenten
///
/// Gemäß `state.rs` Snapshot-Pattern für konsistente Reads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecentralizedStorageSnapshot {
    /// Anzahl gespeicherter Identitäten
    pub identity_count: u64,

    /// Anzahl gespeicherter Events
    pub event_count: u64,

    /// Anzahl Trust-Beziehungen
    pub trust_count: u64,

    /// Anzahl Content-Einträge
    pub content_count: u64,

    /// Aggregierter Health-Score (0.0 - 1.0)
    pub health_score: f64,

    /// Ist der Keyspace erreichbar?
    pub keyspace_accessible: bool,
}

impl DecentralizedStorageSnapshot {
    /// Ist das System gesund?
    pub fn is_healthy(&self) -> bool {
        self.health_score >= 0.9 && self.keyspace_accessible
    }

    /// Totale Einträge über alle Stores
    pub fn total_entries(&self) -> u64 {
        self.identity_count + self.event_count + self.trust_count + self.content_count
    }
}

impl Default for DecentralizedStorageSnapshot {
    fn default() -> Self {
        Self {
            identity_count: 0,
            event_count: 0,
            trust_count: 0,
            content_count: 0,
            health_score: 1.0,
            keyspace_accessible: false,
        }
    }
}

/// Detaillierter Snapshot aller Storage-Komponenten (Phase 2)
///
/// Enthält alle store-spezifischen Metriken für tiefe Einblicke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedStorageSnapshot {
    /// Identity-Store Snapshot
    pub identities: IdentityStoreSnapshot,

    /// Event-Store Snapshot
    pub events: EventStoreSnapshot,

    /// Trust-Store Snapshot
    pub trust: TrustStoreSnapshot,

    /// Content-Store Snapshot
    pub content: ContentStoreSnapshot,

    /// Aggregierter Health-Score (0.0 - 1.0)
    pub health_score: f64,

    /// Ist der Keyspace erreichbar?
    pub keyspace_accessible: bool,
}

impl DetailedStorageSnapshot {
    /// Ist das System gesund?
    pub fn is_healthy(&self) -> bool {
        self.health_score >= 0.9 && self.keyspace_accessible
    }

    /// Totale Einträge über alle Stores
    pub fn total_entries(&self) -> u64 {
        self.identities.total
            + self.events.count
            + self.trust.relationships
            + self.content.count
    }

    /// Totale Bytes über alle Stores
    pub fn total_bytes(&self) -> u64 {
        self.identities.metrics.bytes
            + self.events.metrics.bytes
            + self.trust.metrics.bytes
            + self.content.total_size
    }
}

/// Detaillierte Diagnose für Troubleshooting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDiagnostics {
    /// Keyspace erreichbar?
    pub keyspace_accessible: bool,

    /// Identity Store OK?
    pub identity_store_ok: bool,

    /// Event Store OK?
    pub event_store_ok: bool,

    /// Trust Store OK?
    pub trust_store_ok: bool,

    /// Content Store OK?
    pub content_store_ok: bool,

    /// Gesamt-Health-Score
    pub health_score: f64,
}

impl StorageDiagnostics {
    /// Sind alle Stores OK?
    pub fn all_ok(&self) -> bool {
        self.keyspace_accessible
            && self.identity_store_ok
            && self.event_store_ok
            && self.trust_store_ok
            && self.content_store_ok
    }

    /// Liste der problematischen Stores
    pub fn problematic_stores(&self) -> Vec<&'static str> {
        let mut problems = Vec::new();

        if !self.keyspace_accessible {
            problems.push("keyspace");
        }
        if !self.identity_store_ok {
            problems.push("identities");
        }
        if !self.event_store_ok {
            problems.push("events");
        }
        if !self.trust_store_ok {
            problems.push("trust");
        }
        if !self.content_store_ok {
            problems.push("content");
        }

        problems
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PHASE 3: StorageState Integration Types
// ─────────────────────────────────────────────────────────────────────────────

/// Update-Paket für StorageState Integration
///
/// Enthält alle Werte, die an `core::state::StorageState` gesendet werden.
/// Kann für Logging, Debugging oder Message-Passing verwendet werden.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStateUpdate {
    /// Anzahl KV-Schlüssel (aus ContentStore)
    pub kv_keys: u64,

    /// KV-Bytes gesamt
    pub kv_bytes: u64,

    /// KV-Lesezugriffe
    pub kv_reads: u64,

    /// KV-Schreibzugriffe
    pub kv_writes: u64,

    /// Anzahl Events im Store
    pub event_store_count: u64,

    /// Event-Store Bytes
    pub event_store_bytes: u64,

    /// Anzahl Identitäten
    pub identities: u64,

    /// Anzahl Trust-Einträge
    pub trust_entries: u64,

    /// Health-Score (0.0 - 1.0)
    pub health_score: f64,
}

impl StorageStateUpdate {
    /// Ist das Storage-System gesund?
    pub fn is_healthy(&self) -> bool {
        self.health_score >= 0.9
    }

    /// Totale Einträge
    pub fn total_entries(&self) -> u64 {
        self.kv_keys + self.event_store_count + self.identities + self.trust_entries
    }

    /// Totale Bytes
    pub fn total_bytes(&self) -> u64 {
        self.kv_bytes + self.event_store_bytes
    }
}

impl Default for StorageStateUpdate {
    fn default() -> Self {
        Self {
            kv_keys: 0,
            kv_bytes: 0,
            kv_reads: 0,
            kv_writes: 0,
            event_store_count: 0,
            event_store_bytes: 0,
            identities: 0,
            trust_entries: 0,
            health_score: 1.0,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PHASE 3: Health Monitoring Helper
// ─────────────────────────────────────────────────────────────────────────────

/// Health-Monitor für periodische Überwachung
///
/// Kann in einem Background-Task verwendet werden, um den Storage-Status
/// regelmäßig zu prüfen und bei Problemen zu alarmieren.
#[derive(Debug, Clone)]
pub struct StorageHealthMonitor {
    /// Schwellenwert für Warnung (default: 0.7)
    pub warning_threshold: f64,

    /// Schwellenwert für kritisch (default: 0.5)
    pub critical_threshold: f64,

    /// Letzter bekannter Health-Score
    pub last_health_score: f64,

    /// Anzahl aufeinanderfolgender ungesunder Checks
    pub consecutive_unhealthy: u32,
}

impl StorageHealthMonitor {
    /// Erstellt einen neuen Monitor mit Default-Schwellenwerten
    pub fn new() -> Self {
        Self {
            warning_threshold: 0.7,
            critical_threshold: 0.5,
            last_health_score: 1.0,
            consecutive_unhealthy: 0,
        }
    }

    /// Erstellt einen Monitor mit benutzerdefinierten Schwellenwerten
    pub fn with_thresholds(warning: f64, critical: f64) -> Self {
        Self {
            warning_threshold: warning,
            critical_threshold: critical,
            last_health_score: 1.0,
            consecutive_unhealthy: 0,
        }
    }

    /// Prüft den Storage und gibt den Status zurück
    pub fn check(&mut self, storage: &DecentralizedStorage) -> HealthCheckResult {
        let health_score = storage.health_score();
        let diagnostics = storage.diagnose();

        self.last_health_score = health_score;

        let status = if health_score >= 0.9 {
            self.consecutive_unhealthy = 0;
            HealthStatus::Healthy
        } else if health_score >= self.warning_threshold {
            self.consecutive_unhealthy += 1;
            HealthStatus::Warning
        } else if health_score >= self.critical_threshold {
            self.consecutive_unhealthy += 1;
            HealthStatus::Critical
        } else {
            self.consecutive_unhealthy += 1;
            HealthStatus::Failed
        };

        // Problematic stores berechnen bevor diagnostics bewegt wird
        let problematic_stores = diagnostics.problematic_stores();

        HealthCheckResult {
            status,
            health_score,
            diagnostics,
            consecutive_unhealthy: self.consecutive_unhealthy,
            problematic_stores,
        }
    }
}

impl Default for StorageHealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Ergebnis eines Health-Checks
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Status-Klassifikation
    pub status: HealthStatus,

    /// Numerischer Health-Score
    pub health_score: f64,

    /// Detaillierte Diagnostik
    pub diagnostics: StorageDiagnostics,

    /// Anzahl aufeinanderfolgender ungesunder Checks
    pub consecutive_unhealthy: u32,

    /// Liste problematischer Stores
    pub problematic_stores: Vec<&'static str>,
}

impl HealthCheckResult {
    /// Sollte ein Alert ausgelöst werden?
    pub fn should_alert(&self) -> bool {
        matches!(self.status, HealthStatus::Critical | HealthStatus::Failed)
    }

    /// Ist das System betriebsbereit?
    pub fn is_operational(&self) -> bool {
        !matches!(self.status, HealthStatus::Failed)
    }
}

/// Health-Status-Klassifikation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Alles OK (score >= 0.9)
    Healthy,
    /// Warnung (score >= warning_threshold)
    Warning,
    /// Kritisch (score >= critical_threshold)
    Critical,
    /// Fehlgeschlagen (score < critical_threshold)
    Failed,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "HEALTHY"),
            HealthStatus::Warning => write!(f, "WARNING"),
            HealthStatus::Critical => write!(f, "CRITICAL"),
            HealthStatus::Failed => write!(f, "FAILED"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Test Utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Test-Utility-Modul für einheitliche Fjall-Keyspace-Erstellung
#[cfg(test)]
pub mod test_utils {
    use fjall::Keyspace;
    use std::sync::Arc;

    /// Erstellt einen temporären Fjall Keyspace für Tests
    ///
    /// # Returns
    /// Tuple aus (TempDir, Arc<Keyspace>)
    /// TempDir muss im Scope bleiben, sonst wird das Verzeichnis gelöscht!
    ///
    /// # Example
    /// ```ignore
    /// let (_dir, keyspace) = test_keyspace();
    /// let store = MyStore::new(&keyspace).unwrap();
    /// ```
    pub fn test_keyspace() -> (tempfile::TempDir, Arc<Keyspace>) {
        let dir = tempfile::tempdir().expect("Failed to create temp directory");
        let keyspace = Arc::new(
            fjall::Config::new(dir.path())
                .open()
                .expect("Failed to open keyspace"),
        );
        (dir, keyspace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DIDNamespace;

    #[test]
    fn test_decentralized_storage_temporary() {
        let storage = DecentralizedStorage::open_temporary().unwrap();

        assert_eq!(storage.identity_count(), 0);
        assert_eq!(storage.event_count(), 0);
        assert_eq!(storage.trust_count(), 0);
        assert_eq!(storage.content_count(), 0);
    }

    #[test]
    fn test_storage_integration() {
        let storage = DecentralizedStorage::open_temporary().unwrap();

        // Identity erstellen
        let _identity = storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .unwrap();

        assert_eq!(storage.identity_count(), 1);

        // Flush
        storage.flush().unwrap();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 1: Metrics Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_storage_snapshot() {
        let storage = DecentralizedStorage::open_temporary().unwrap();

        let snapshot = storage.snapshot();

        assert_eq!(snapshot.identity_count, 0);
        assert_eq!(snapshot.event_count, 0);
        assert_eq!(snapshot.trust_count, 0);
        assert_eq!(snapshot.content_count, 0);
        assert!(snapshot.keyspace_accessible);
        assert!(snapshot.is_healthy());
    }

    #[test]
    fn test_storage_health_score() {
        let storage = DecentralizedStorage::open_temporary().unwrap();

        let health = storage.health_score();

        // Frischer Storage sollte perfekt gesund sein
        assert!(health >= 0.9);
        assert!(storage.is_healthy());
    }

    #[test]
    fn test_storage_diagnostics() {
        let storage = DecentralizedStorage::open_temporary().unwrap();

        let diag = storage.diagnose();

        assert!(diag.keyspace_accessible);
        assert!(diag.identity_store_ok);
        assert!(diag.event_store_ok);
        assert!(diag.trust_store_ok);
        assert!(diag.content_store_ok);
        assert!(diag.all_ok());
        assert!(diag.problematic_stores().is_empty());
    }

    #[test]
    fn test_snapshot_after_operations() {
        let storage = DecentralizedStorage::open_temporary().unwrap();

        // Identity erstellen
        storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .unwrap();

        let snapshot = storage.snapshot();

        assert_eq!(snapshot.identity_count, 1);
        assert_eq!(snapshot.total_entries(), 1);
    }

    #[test]
    fn test_store_metrics_basic() {
        let metrics = StoreMetrics::new();

        // Initial values
        assert_eq!(metrics.count.load(std::sync::atomic::Ordering::Relaxed), 0);
        assert!(metrics.health_score() >= 0.99); // Perfect health initially

        // Record some operations
        metrics.record_write(100, 1024);
        metrics.record_read(50, 512);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.writes, 1);
        assert_eq!(snapshot.reads, 1);
        assert!(snapshot.is_healthy());
    }

    #[test]
    fn test_aggregate_metrics() {
        let metrics1 = StoreMetrics::new();
        let metrics2 = StoreMetrics::new();

        metrics1.record_write(100, 1000);
        metrics1.set_count(10);

        metrics2.record_write(100, 2000);
        metrics2.set_count(20);

        let stores = vec![
            ("store1".to_string(), metrics1.snapshot()),
            ("store2".to_string(), metrics2.snapshot()),
        ];

        let aggregate = AggregateMetricsSnapshot::from_stores(&stores);

        assert_eq!(aggregate.store_count, 2);
        assert_eq!(aggregate.total_entries, 30);
        assert_eq!(aggregate.total_bytes, 3000);
        assert!(aggregate.is_healthy());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 3: Integration Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_storage_state_update() {
        let storage = DecentralizedStorage::open_temporary().unwrap();

        // Daten hinzufügen
        storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .unwrap();
        storage
            .content
            .put(b"test content".to_vec(), "text/plain", None, vec![])
            .unwrap();

        let update = storage.storage_state_update();

        assert_eq!(update.identities, 1);
        assert_eq!(update.kv_keys, 1);
        assert!(update.kv_bytes > 0);
        assert!(update.is_healthy());
    }

    #[test]
    fn test_detailed_snapshot() {
        let storage = DecentralizedStorage::open_temporary().unwrap();

        // Daten hinzufügen
        storage
            .identities
            .create_identity(DIDNamespace::Self_)
            .unwrap();

        let snapshot = storage.detailed_snapshot();

        assert_eq!(snapshot.identities.total, 1);
        assert!(snapshot.is_healthy());
        assert!(snapshot.total_entries() >= 1);
    }

    #[test]
    fn test_health_monitor() {
        let storage = DecentralizedStorage::open_temporary().unwrap();
        let mut monitor = StorageHealthMonitor::new();

        let result = monitor.check(&storage);

        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.is_operational());
        assert!(!result.should_alert());
        assert_eq!(result.consecutive_unhealthy, 0);
    }

    #[test]
    fn test_health_monitor_thresholds() {
        let monitor = StorageHealthMonitor::with_thresholds(0.8, 0.6);

        assert_eq!(monitor.warning_threshold, 0.8);
        assert_eq!(monitor.critical_threshold, 0.6);
    }

    #[test]
    fn test_storage_state_update_default() {
        let update = StorageStateUpdate::default();

        assert_eq!(update.kv_keys, 0);
        assert_eq!(update.identities, 0);
        assert!(update.is_healthy());
        assert_eq!(update.total_entries(), 0);
    }

    #[test]
    fn test_health_status_display() {
        assert_eq!(format!("{}", HealthStatus::Healthy), "HEALTHY");
        assert_eq!(format!("{}", HealthStatus::Warning), "WARNING");
        assert_eq!(format!("{}", HealthStatus::Critical), "CRITICAL");
        assert_eq!(format!("{}", HealthStatus::Failed), "FAILED");
    }
}
