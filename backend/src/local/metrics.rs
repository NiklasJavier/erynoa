//! # Storage Metrics Framework
//!
//! Einheitliche Metriken für alle Storage-Komponenten gemäß `state.rs` Patterns.
//!
//! ## Design-Prinzipien
//!
//! - **Atomic Counters**: High-frequency Metrics sind lock-free
//! - **RwLock**: Für komplexe Berechnungen (Durchschnitte)
//! - **Snapshot-Pattern**: Konsistente Reads ohne Locking
//! - **Health-Score**: Automatische Gesundheitsbewertung
//!
//! ## Verwendung
//!
//! ```rust,ignore
//! use crate::local::metrics::{StoreMetrics, StorageMetrics};
//!
//! struct MyStore {
//!     data: KvStore,
//!     metrics: Arc<StoreMetrics>,
//! }
//!
//! impl MyStore {
//!     fn put(&self, key: &str, value: &[u8]) -> Result<()> {
//!         let start = Instant::now();
//!         self.data.put(key, value)?;
//!         let latency = start.elapsed().as_micros() as u64;
//!         self.metrics.record_write(latency, value.len() as u64);
//!         Ok(())
//!     }
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// STORAGE METRICS TRAIT
// ============================================================================

/// Basis-Metriken die jeder Store tracken muss.
///
/// Dieses Trait definiert die minimale Metriken-API, die alle Storage-Komponenten
/// implementieren sollten, um konsistentes Monitoring zu ermöglichen.
pub trait StorageMetrics: Send + Sync {
    /// Anzahl gespeicherter Einträge
    fn entry_count(&self) -> u64;

    /// Geschätzte Größe in Bytes
    fn size_bytes(&self) -> u64;

    /// Lesezugriffe seit Start
    fn read_count(&self) -> u64;

    /// Schreibzugriffe seit Start
    fn write_count(&self) -> u64;

    /// Löschungen seit Start
    fn delete_count(&self) -> u64;

    /// Fehler seit Start
    fn error_count(&self) -> u64;

    /// Health-Score (0.0 - 1.0)
    ///
    /// Berechnet basierend auf Error-Rate und anderen Faktoren.
    /// - 1.0: Perfekt gesund
    /// - 0.9+: Gesund
    /// - 0.7-0.9: Warnung
    /// - <0.7: Kritisch
    fn health_score(&self) -> f64;

    /// Snapshot für konsistenten Read
    fn metrics_snapshot(&self) -> StoreMetricsSnapshot;
}

// ============================================================================
// STORE METRICS IMPLEMENTATION
// ============================================================================

/// Gemeinsamer Metrics-Container für alle Stores.
///
/// Thread-safe durch Atomic-Operationen für alle häufigen Updates.
/// RwLock nur für Durchschnittsberechnungen.
///
/// # Beispiel
///
/// ```rust,ignore
/// let metrics = StoreMetrics::new();
///
/// // Bei Lese-Operation
/// metrics.record_read(latency_us, bytes_read);
///
/// // Bei Schreib-Operation
/// metrics.record_write(latency_us, bytes_written);
///
/// // Snapshot für Reporting
/// let snapshot = metrics.snapshot();
/// ```
#[derive(Debug)]
pub struct StoreMetrics {
    // ─────────────────────────────────────────────────────────────────────────
    // COUNTERS (Atomic, lock-free)
    // ─────────────────────────────────────────────────────────────────────────
    /// Einträge gesamt
    pub count: AtomicU64,

    /// Bytes gesamt (geschätzt)
    pub bytes: AtomicU64,

    /// Lese-Operationen
    pub reads: AtomicU64,

    /// Schreib-Operationen
    pub writes: AtomicU64,

    /// Lösch-Operationen
    pub deletes: AtomicU64,

    /// Fehler gesamt
    pub errors: AtomicU64,

    /// Cache-Hits (falls applicable)
    pub cache_hits: AtomicU64,

    /// Cache-Misses (falls applicable)
    pub cache_misses: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // LATENCY TRACKING (RwLock für Durchschnitte)
    // ─────────────────────────────────────────────────────────────────────────
    /// Durchschnittliche Lese-Latenz (µs)
    pub avg_read_latency_us: RwLock<f64>,

    /// Durchschnittliche Schreib-Latenz (µs)
    pub avg_write_latency_us: RwLock<f64>,

    /// Maximale Lese-Latenz (µs)
    pub max_read_latency_us: AtomicU64,

    /// Maximale Schreib-Latenz (µs)
    pub max_write_latency_us: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // TIMESTAMPS
    // ─────────────────────────────────────────────────────────────────────────
    /// Erstellungszeitpunkt (Unix ms)
    pub created_at_ms: u64,

    /// Letzte Operation (Unix ms)
    pub last_operation_ms: AtomicU64,

    /// Letzte erfolgreiche Operation (Unix ms)
    pub last_success_ms: AtomicU64,

    /// Letzter Fehler (Unix ms)
    pub last_error_ms: AtomicU64,
}

impl StoreMetrics {
    /// Erstelle neue Metrics-Instanz
    pub fn new() -> Self {
        let now = current_time_ms();

        Self {
            count: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            reads: AtomicU64::new(0),
            writes: AtomicU64::new(0),
            deletes: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            avg_read_latency_us: RwLock::new(0.0),
            avg_write_latency_us: RwLock::new(0.0),
            max_read_latency_us: AtomicU64::new(0),
            max_write_latency_us: AtomicU64::new(0),
            created_at_ms: now,
            last_operation_ms: AtomicU64::new(now),
            last_success_ms: AtomicU64::new(now),
            last_error_ms: AtomicU64::new(0),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RECORDING METHODS
    // ─────────────────────────────────────────────────────────────────────────

    /// Lese-Operation aufzeichnen
    ///
    /// # Arguments
    /// * `latency_us` - Latenz in Mikrosekunden
    /// * `bytes` - Gelesene Bytes (0 wenn nicht gefunden)
    pub fn record_read(&self, latency_us: u64, bytes: u64) {
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.update_timestamp();
        self.last_success_ms
            .store(current_time_ms(), Ordering::Relaxed);

        // Update average latency (rolling average)
        if let Ok(mut avg) = self.avg_read_latency_us.write() {
            let total = self.reads.load(Ordering::Relaxed) as f64;
            if total > 0.0 {
                *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
            }
        }

        // Update max latency
        update_max(&self.max_read_latency_us, latency_us);

        // Cache tracking (if bytes > 0, it was a hit conceptually)
        if bytes > 0 {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Schreib-Operation aufzeichnen
    ///
    /// # Arguments
    /// * `latency_us` - Latenz in Mikrosekunden
    /// * `bytes` - Geschriebene Bytes
    pub fn record_write(&self, latency_us: u64, bytes: u64) {
        self.writes.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
        self.update_timestamp();
        self.last_success_ms
            .store(current_time_ms(), Ordering::Relaxed);

        // Update average latency (rolling average)
        if let Ok(mut avg) = self.avg_write_latency_us.write() {
            let total = self.writes.load(Ordering::Relaxed) as f64;
            if total > 0.0 {
                *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
            }
        }

        // Update max latency
        update_max(&self.max_write_latency_us, latency_us);
    }

    /// Lösch-Operation aufzeichnen
    ///
    /// # Arguments
    /// * `bytes` - Gelöschte Bytes (geschätzt)
    pub fn record_delete(&self, bytes: u64) {
        self.deletes.fetch_add(1, Ordering::Relaxed);

        // Bytes subtrahieren (aber nicht unter 0)
        let current = self.bytes.load(Ordering::Relaxed);
        let new_value = current.saturating_sub(bytes);
        self.bytes.store(new_value, Ordering::Relaxed);

        self.update_timestamp();
        self.last_success_ms
            .store(current_time_ms(), Ordering::Relaxed);
    }

    /// Fehler aufzeichnen
    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        self.last_error_ms
            .store(current_time_ms(), Ordering::Relaxed);
        self.update_timestamp();
    }

    /// Entry-Count setzen (nach vollständigem Scan)
    pub fn set_count(&self, count: u64) {
        self.count.store(count, Ordering::Relaxed);
    }

    /// Entry-Count inkrementieren (bei Insert)
    pub fn increment_count(&self) {
        self.count.fetch_add(1, Ordering::Relaxed);
    }

    /// Entry-Count dekrementieren (bei Delete)
    pub fn decrement_count(&self) {
        let current = self.count.load(Ordering::Relaxed);
        if current > 0 {
            self.count.store(current - 1, Ordering::Relaxed);
        }
    }

    /// Bytes-Count direkt setzen (nach Scan)
    pub fn set_bytes(&self, bytes: u64) {
        self.bytes.store(bytes, Ordering::Relaxed);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // COMPUTED METRICS
    // ─────────────────────────────────────────────────────────────────────────

    /// Berechne Health-Score (0.0 - 1.0)
    ///
    /// Faktoren:
    /// - Error-Rate (Hauptfaktor)
    /// - Zeit seit letztem Fehler
    /// - Latenz-Anomalien
    pub fn health_score(&self) -> f64 {
        let total_ops = self.reads.load(Ordering::Relaxed)
            + self.writes.load(Ordering::Relaxed)
            + self.deletes.load(Ordering::Relaxed);

        let errors = self.errors.load(Ordering::Relaxed);

        // Noch keine Operationen = perfekt gesund
        if total_ops == 0 {
            return 1.0;
        }

        // Error-Rate berechnen
        let error_rate = errors as f64 / total_ops as f64;

        // Basis-Score aus Error-Rate (sanftere Kurve)
        // Bei 10% Fehlerrate → 0.7 Score, bei 50% → 0.25 Score
        let mut score = (1.0 - error_rate * 1.5).max(0.1);

        // Penalty für kürzliche Fehler
        let last_error = self.last_error_ms.load(Ordering::Relaxed);
        let now = current_time_ms();
        if last_error > 0 {
            let seconds_since_error = (now.saturating_sub(last_error)) / 1000;
            if seconds_since_error < 60 {
                score *= 0.8; // Kürzlicher Fehler: 20% Penalty
            } else if seconds_since_error < 300 {
                score *= 0.9; // Fehler in letzten 5 Min: 10% Penalty
            }
        }

        // Penalty für hohe Latenz (> 100ms = langsam)
        let avg_write = self.avg_write_latency_us.read().map(|v| *v).unwrap_or(0.0);
        if avg_write > 100_000.0 {
            // > 100ms
            score *= 0.9;
        }

        score.clamp(0.0, 1.0)
    }

    /// Cache-Hit-Rate (0.0 - 1.0)
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.load(Ordering::Relaxed);
        let misses = self.cache_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Operations per Second (basierend auf Uptime)
    pub fn ops_per_second(&self) -> f64 {
        let total_ops = self.reads.load(Ordering::Relaxed)
            + self.writes.load(Ordering::Relaxed)
            + self.deletes.load(Ordering::Relaxed);

        let uptime_secs = (current_time_ms() - self.created_at_ms) / 1000;
        if uptime_secs > 0 {
            total_ops as f64 / uptime_secs as f64
        } else {
            0.0
        }
    }

    /// Uptime in Sekunden
    pub fn uptime_secs(&self) -> u64 {
        (current_time_ms() - self.created_at_ms) / 1000
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SNAPSHOT
    // ─────────────────────────────────────────────────────────────────────────

    /// Erstelle konsistenten Snapshot aller Metriken
    pub fn snapshot(&self) -> StoreMetricsSnapshot {
        StoreMetricsSnapshot {
            count: self.count.load(Ordering::Relaxed),
            bytes: self.bytes.load(Ordering::Relaxed),
            reads: self.reads.load(Ordering::Relaxed),
            writes: self.writes.load(Ordering::Relaxed),
            deletes: self.deletes.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            avg_read_latency_us: self.avg_read_latency_us.read().map(|v| *v).unwrap_or(0.0),
            avg_write_latency_us: self.avg_write_latency_us.read().map(|v| *v).unwrap_or(0.0),
            max_read_latency_us: self.max_read_latency_us.load(Ordering::Relaxed),
            max_write_latency_us: self.max_write_latency_us.load(Ordering::Relaxed),
            created_at_ms: self.created_at_ms,
            last_operation_ms: self.last_operation_ms.load(Ordering::Relaxed),
            last_success_ms: self.last_success_ms.load(Ordering::Relaxed),
            last_error_ms: self.last_error_ms.load(Ordering::Relaxed),
            health_score: self.health_score(),
            cache_hit_rate: self.cache_hit_rate(),
            ops_per_second: self.ops_per_second(),
            uptime_secs: self.uptime_secs(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // PRIVATE HELPERS
    // ─────────────────────────────────────────────────────────────────────────

    fn update_timestamp(&self) {
        self.last_operation_ms
            .store(current_time_ms(), Ordering::Relaxed);
    }
}

impl Default for StoreMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageMetrics for StoreMetrics {
    fn entry_count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    fn size_bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }

    fn read_count(&self) -> u64 {
        self.reads.load(Ordering::Relaxed)
    }

    fn write_count(&self) -> u64 {
        self.writes.load(Ordering::Relaxed)
    }

    fn delete_count(&self) -> u64 {
        self.deletes.load(Ordering::Relaxed)
    }

    fn error_count(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }

    fn health_score(&self) -> f64 {
        StoreMetrics::health_score(self)
    }

    fn metrics_snapshot(&self) -> StoreMetricsSnapshot {
        self.snapshot()
    }
}

// ============================================================================
// SNAPSHOT TYPE
// ============================================================================

/// Snapshot der Store-Metriken (serialisierbar, cloneable).
///
/// Alle Werte sind zu einem Zeitpunkt konsistent erfasst.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StoreMetricsSnapshot {
    // Counters
    pub count: u64,
    pub bytes: u64,
    pub reads: u64,
    pub writes: u64,
    pub deletes: u64,
    pub errors: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,

    // Latency
    pub avg_read_latency_us: f64,
    pub avg_write_latency_us: f64,
    pub max_read_latency_us: u64,
    pub max_write_latency_us: u64,

    // Timestamps
    pub created_at_ms: u64,
    pub last_operation_ms: u64,
    pub last_success_ms: u64,
    pub last_error_ms: u64,

    // Computed
    pub health_score: f64,
    pub cache_hit_rate: f64,
    pub ops_per_second: f64,
    pub uptime_secs: u64,
}

impl Default for StoreMetricsSnapshot {
    fn default() -> Self {
        Self {
            count: 0,
            bytes: 0,
            reads: 0,
            writes: 0,
            deletes: 0,
            errors: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_read_latency_us: 0.0,
            avg_write_latency_us: 0.0,
            max_read_latency_us: 0,
            max_write_latency_us: 0,
            created_at_ms: 0,
            last_operation_ms: 0,
            last_success_ms: 0,
            last_error_ms: 0,
            health_score: 1.0,
            cache_hit_rate: 0.0,
            ops_per_second: 0.0,
            uptime_secs: 0,
        }
    }
}

impl StoreMetricsSnapshot {
    /// Ist der Store gesund?
    pub fn is_healthy(&self) -> bool {
        self.health_score >= 0.9
    }

    /// Hat der Store kürzlich Fehler?
    pub fn has_recent_errors(&self) -> bool {
        let now = current_time_ms();
        self.last_error_ms > 0 && (now - self.last_error_ms) < 300_000 // 5 Minuten
    }

    /// Totale Operationen
    pub fn total_operations(&self) -> u64 {
        self.reads + self.writes + self.deletes
    }

    /// Write/Read Ratio
    pub fn write_read_ratio(&self) -> f64 {
        if self.reads > 0 {
            self.writes as f64 / self.reads as f64
        } else if self.writes > 0 {
            f64::INFINITY
        } else {
            0.0
        }
    }
}

// ============================================================================
// AGGREGATE METRICS
// ============================================================================

/// Aggregierte Metriken über mehrere Stores.
///
/// Verwendet für `DecentralizedStorage` um alle Sub-Stores zusammenzufassen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateMetricsSnapshot {
    /// Anzahl der aggregierten Stores
    pub store_count: usize,

    /// Gesamt-Einträge über alle Stores
    pub total_entries: u64,

    /// Gesamt-Bytes über alle Stores
    pub total_bytes: u64,

    /// Gesamt-Reads
    pub total_reads: u64,

    /// Gesamt-Writes
    pub total_writes: u64,

    /// Gesamt-Deletes
    pub total_deletes: u64,

    /// Gesamt-Errors
    pub total_errors: u64,

    /// Durchschnittlicher Health-Score
    pub avg_health_score: f64,

    /// Minimaler Health-Score (schwächstes Glied)
    pub min_health_score: f64,

    /// Stores mit Problemen (health < 0.9)
    pub unhealthy_stores: Vec<String>,

    /// Gesamt-OPS
    pub total_ops_per_second: f64,
}

impl AggregateMetricsSnapshot {
    /// Erstelle aus einzelnen Store-Snapshots
    pub fn from_stores(stores: &[(String, StoreMetricsSnapshot)]) -> Self {
        if stores.is_empty() {
            return Self::empty();
        }

        let mut total_entries = 0u64;
        let mut total_bytes = 0u64;
        let mut total_reads = 0u64;
        let mut total_writes = 0u64;
        let mut total_deletes = 0u64;
        let mut total_errors = 0u64;
        let mut total_ops_per_second = 0.0;
        let mut health_sum = 0.0;
        let mut min_health = 1.0f64;
        let mut unhealthy_stores = Vec::new();

        for (name, snapshot) in stores {
            total_entries += snapshot.count;
            total_bytes += snapshot.bytes;
            total_reads += snapshot.reads;
            total_writes += snapshot.writes;
            total_deletes += snapshot.deletes;
            total_errors += snapshot.errors;
            total_ops_per_second += snapshot.ops_per_second;
            health_sum += snapshot.health_score;

            if snapshot.health_score < min_health {
                min_health = snapshot.health_score;
            }

            if snapshot.health_score < 0.9 {
                unhealthy_stores.push(name.clone());
            }
        }

        let store_count = stores.len();
        let avg_health_score = health_sum / store_count as f64;

        Self {
            store_count,
            total_entries,
            total_bytes,
            total_reads,
            total_writes,
            total_deletes,
            total_errors,
            avg_health_score,
            min_health_score: min_health,
            unhealthy_stores,
            total_ops_per_second,
        }
    }

    /// Leeres Aggregat
    pub fn empty() -> Self {
        Self {
            store_count: 0,
            total_entries: 0,
            total_bytes: 0,
            total_reads: 0,
            total_writes: 0,
            total_deletes: 0,
            total_errors: 0,
            avg_health_score: 1.0,
            min_health_score: 1.0,
            unhealthy_stores: Vec::new(),
            total_ops_per_second: 0.0,
        }
    }

    /// Ist das gesamte System gesund?
    pub fn is_healthy(&self) -> bool {
        self.min_health_score >= 0.9
    }

    /// Gesamt-Operationen
    pub fn total_operations(&self) -> u64 {
        self.total_reads + self.total_writes + self.total_deletes
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Aktuelle Zeit in Millisekunden (Unix Epoch)
fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Atomic Max-Update
fn update_max(atomic: &AtomicU64, value: u64) {
    loop {
        let current = atomic.load(Ordering::Relaxed);
        if value <= current {
            break;
        }
        if atomic
            .compare_exchange(current, value, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            break;
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_metrics_new() {
        let metrics = StoreMetrics::new();

        assert_eq!(metrics.count.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.reads.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.writes.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.errors.load(Ordering::Relaxed), 0);
        assert!(metrics.created_at_ms > 0);
    }

    #[test]
    fn test_record_read() {
        let metrics = StoreMetrics::new();

        metrics.record_read(100, 1024);

        assert_eq!(metrics.reads.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.cache_hits.load(Ordering::Relaxed), 1);

        let avg = metrics.avg_read_latency_us.read().unwrap();
        assert!(((*avg) - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_record_write() {
        let metrics = StoreMetrics::new();

        metrics.record_write(200, 2048);

        assert_eq!(metrics.writes.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.bytes.load(Ordering::Relaxed), 2048);

        let avg = metrics.avg_write_latency_us.read().unwrap();
        assert!(((*avg) - 200.0).abs() < 0.001);
    }

    #[test]
    fn test_record_delete() {
        let metrics = StoreMetrics::new();

        // Erst Bytes hinzufügen
        metrics.bytes.store(5000, Ordering::Relaxed);
        metrics.count.store(10, Ordering::Relaxed);

        metrics.record_delete(1000);

        assert_eq!(metrics.deletes.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.bytes.load(Ordering::Relaxed), 4000);
    }

    #[test]
    fn test_health_score_perfect() {
        let metrics = StoreMetrics::new();

        // Einige erfolgreiche Operationen
        metrics.record_read(100, 1024);
        metrics.record_write(100, 1024);

        assert!(metrics.health_score() >= 0.99);
    }

    #[test]
    fn test_health_score_with_errors() {
        let metrics = StoreMetrics::new();

        // Erfolgreiche Operationen
        for _ in 0..10 {
            metrics.record_read(100, 1024);
        }

        // Ein Fehler
        metrics.record_error();

        // Score sollte unter 1.0 aber über 0.0 sein
        let score = metrics.health_score();
        assert!(score < 1.0);
        assert!(score > 0.0);
    }

    #[test]
    fn test_snapshot() {
        let metrics = StoreMetrics::new();

        metrics.record_read(100, 1024);
        metrics.record_write(200, 2048);
        metrics.set_count(5);

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.count, 5);
        assert_eq!(snapshot.reads, 1);
        assert_eq!(snapshot.writes, 1);
        assert_eq!(snapshot.bytes, 2048);
        assert!(snapshot.health_score > 0.0);
    }

    #[test]
    fn test_cache_hit_rate() {
        let metrics = StoreMetrics::new();

        // 3 Hits (bytes > 0)
        metrics.record_read(100, 1024);
        metrics.record_read(100, 1024);
        metrics.record_read(100, 1024);

        // 1 Miss (bytes = 0)
        metrics.record_read(100, 0);

        let rate = metrics.cache_hit_rate();
        assert!((rate - 0.75).abs() < 0.001); // 3/4 = 0.75
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
        assert_eq!(aggregate.total_writes, 2);
        assert!(aggregate.is_healthy());
    }

    #[test]
    fn test_increment_decrement_count() {
        let metrics = StoreMetrics::new();

        metrics.increment_count();
        metrics.increment_count();
        assert_eq!(metrics.count.load(Ordering::Relaxed), 2);

        metrics.decrement_count();
        assert_eq!(metrics.count.load(Ordering::Relaxed), 1);

        // Sollte nicht unter 0 gehen
        metrics.decrement_count();
        metrics.decrement_count();
        assert_eq!(metrics.count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_max_latency_tracking() {
        let metrics = StoreMetrics::new();

        metrics.record_read(100, 1024);
        metrics.record_read(500, 1024);
        metrics.record_read(200, 1024);

        assert_eq!(metrics.max_read_latency_us.load(Ordering::Relaxed), 500);
    }

    #[test]
    fn test_snapshot_computed_fields() {
        let metrics = StoreMetrics::new();

        metrics.record_read(100, 1024);
        metrics.record_write(100, 2048);

        let snapshot = metrics.snapshot();

        assert_eq!(snapshot.total_operations(), 2);
        assert!(snapshot.is_healthy());
        assert!(!snapshot.has_recent_errors());
    }
}
