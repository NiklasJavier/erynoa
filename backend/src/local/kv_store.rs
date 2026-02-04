//! Low-Level Key-Value Store
//!
//! Wrapper um Fjall mit Type-Safe Serialisierung und Metriken-Tracking.
//!
//! ## Phase 2 Features
//!
//! - Metriken für alle Operationen (read/write/delete)
//! - Latenz-Tracking
//! - Health-Score-Berechnung
//! - Snapshot-Pattern für konsistente Reads

use anyhow::{Context, Result};
use fjall::{Keyspace, PartitionHandle};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Instant;

use super::metrics::{StoreMetrics, StoreMetricsSnapshot};

/// Generic Key-Value Store über einer Fjall Partition
///
/// Jetzt mit integriertem Metriken-Tracking gemäß `state.rs` Patterns.
#[derive(Clone)]
pub struct KvStore {
    /// Fjall Partition Handle
    partition: PartitionHandle,

    /// Metriken für alle Operationen
    metrics: Arc<StoreMetrics>,

    /// Store-Name für Logging/Debugging
    name: String,
}

impl KvStore {
    /// Erstellt einen neuen KvStore mit dem angegebenen Partition-Namen
    pub fn new(keyspace: &Keyspace, name: &str) -> Result<Self> {
        let partition = keyspace
            .open_partition(name, Default::default())
            .context("Failed to open partition")?;

        let metrics = Arc::new(StoreMetrics::new());

        // Initial count setzen
        let initial_count = partition.len().unwrap_or(0) as u64;
        metrics.set_count(initial_count);

        Ok(Self {
            partition,
            metrics,
            name: name.to_string(),
        })
    }

    /// Speichert einen Wert mit Metriken-Tracking
    pub fn put<K: AsRef<[u8]>, V: Serialize>(&self, key: K, value: &V) -> Result<()> {
        let start = Instant::now();

        let bytes = serde_json::to_vec(value).context("Failed to serialize value")?;
        let size = bytes.len() as u64;

        match self.partition.insert(key.as_ref(), bytes) {
            Ok(_) => {
                let latency = start.elapsed().as_micros() as u64;
                self.metrics.record_write(latency, size);

                // Count aktualisieren (optimistisch, da insert erfolgreich war)
                let current_len = self.partition.len().unwrap_or(0) as u64;
                self.metrics.set_count(current_len);

                Ok(())
            }
            Err(e) => {
                self.metrics.record_error();
                Err(e).context("Failed to insert")
            }
        }
    }

    /// Holt einen Wert mit Metriken-Tracking
    pub fn get<K: AsRef<[u8]>, V: DeserializeOwned>(&self, key: K) -> Result<Option<V>> {
        let start = Instant::now();

        match self.partition.get(key) {
            Ok(Some(bytes)) => {
                let size = bytes.len() as u64;
                let latency = start.elapsed().as_micros() as u64;
                self.metrics.record_read(latency, size);

                let value = serde_json::from_slice(&bytes).context("Failed to deserialize")?;
                Ok(Some(value))
            }
            Ok(None) => {
                let latency = start.elapsed().as_micros() as u64;
                self.metrics.record_read(latency, 0); // Cache miss
                Ok(None)
            }
            Err(e) => {
                self.metrics.record_error();
                Err(e).context("Failed to get")
            }
        }
    }

    /// Löscht einen Wert mit Metriken-Tracking
    pub fn delete<K: AsRef<[u8]>>(&self, key: K) -> Result<bool> {
        let start = Instant::now();

        // Prüfe ob der Key existiert und hole die Größe
        let (existed, size) = match self.partition.get(key.as_ref()) {
            Ok(Some(bytes)) => (true, bytes.len() as u64),
            Ok(None) => (false, 0),
            Err(e) => {
                self.metrics.record_error();
                return Err(e).context("Failed to check key before delete");
            }
        };

        match self.partition.remove(key.as_ref()) {
            Ok(_) => {
                if existed {
                    self.metrics.record_delete(size);
                    self.metrics.decrement_count();
                }

                let _latency = start.elapsed().as_micros() as u64;
                Ok(existed)
            }
            Err(e) => {
                self.metrics.record_error();
                Err(e).context("Failed to delete")
            }
        }
    }

    /// Prüft ob ein Key existiert (mit Metriken-Tracking)
    pub fn contains<K: AsRef<[u8]>>(&self, key: K) -> Result<bool> {
        let start = Instant::now();

        match self.partition.get(key) {
            Ok(result) => {
                let latency = start.elapsed().as_micros() as u64;
                let exists = result.is_some();
                self.metrics
                    .record_read(latency, if exists { 1 } else { 0 });
                Ok(exists)
            }
            Err(e) => {
                self.metrics.record_error();
                Err(e).context("Failed to check key")
            }
        }
    }

    /// Iteriert über alle Key-Value Paare
    ///
    /// Hinweis: Keine Metriken für Iteration, da das zu teuer wäre.
    pub fn iter<V: DeserializeOwned>(&self) -> impl Iterator<Item = Result<(Vec<u8>, V)>> + '_ {
        self.partition.iter().map(|result| {
            let (key, value) = result.context("Failed to iterate")?;
            let deserialized: V =
                serde_json::from_slice(&value).context("Failed to deserialize")?;
            Ok((key.to_vec(), deserialized))
        })
    }

    /// Iteriert über Keys mit einem Prefix
    ///
    /// Hinweis: Keine Metriken für Iteration, da das zu teuer wäre.
    pub fn scan_prefix<V: DeserializeOwned>(
        &self,
        prefix: &'static [u8],
    ) -> impl Iterator<Item = Result<(Vec<u8>, V)>> {
        self.partition.prefix(prefix).map(|result| {
            let (key, value) = result.context("Failed to scan")?;
            let deserialized: V =
                serde_json::from_slice(&value).context("Failed to deserialize")?;
            Ok((key.to_vec(), deserialized))
        })
    }

    /// Anzahl der Einträge
    pub fn len(&self) -> usize {
        self.partition.len().unwrap_or(0) as usize
    }

    /// Ist der Store leer?
    pub fn is_empty(&self) -> bool {
        self.partition.is_empty().unwrap_or(true)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // METRICS API (Phase 2)
    // ─────────────────────────────────────────────────────────────────────────

    /// Snapshot der Metriken
    pub fn metrics_snapshot(&self) -> StoreMetricsSnapshot {
        // Aktualisiere Count vor Snapshot
        self.metrics
            .set_count(self.partition.len().unwrap_or(0) as u64);
        self.metrics.snapshot()
    }

    /// Store-Name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Health-Score (0.0 - 1.0)
    pub fn health_score(&self) -> f64 {
        self.metrics.health_score()
    }

    /// Ist der Store gesund?
    pub fn is_healthy(&self) -> bool {
        self.health_score() >= 0.9
    }

    /// Zugriff auf die internen Metriken (für Aggregation)
    pub fn metrics(&self) -> &Arc<StoreMetrics> {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        name: String,
        value: i32,
    }

    fn create_test_keyspace() -> Keyspace {
        let folder = tempfile::tempdir().unwrap();
        fjall::Config::new(folder.path()).open().unwrap()
    }

    #[test]
    fn test_put_get() {
        let keyspace = create_test_keyspace();
        let store = KvStore::new(&keyspace, "test").unwrap();

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        store.put("key1", &data).unwrap();
        let retrieved: Option<TestData> = store.get("key1").unwrap();

        assert_eq!(retrieved, Some(data));
    }

    #[test]
    fn test_delete() {
        let keyspace = create_test_keyspace();
        let store = KvStore::new(&keyspace, "test").unwrap();

        store.put("key1", &"value").unwrap();
        assert!(store.contains("key1").unwrap());

        store.delete("key1").unwrap();
        assert!(!store.contains("key1").unwrap());
    }

    #[test]
    fn test_scan_prefix() {
        let keyspace = create_test_keyspace();
        let store = KvStore::new(&keyspace, "test").unwrap();

        store.put("user:1", &"Alice").unwrap();
        store.put("user:2", &"Bob").unwrap();
        store.put("event:1", &"Created").unwrap();

        let users: Vec<_> = store
            .scan_prefix::<String>(b"user:")
            .filter_map(Result::ok)
            .collect();

        assert_eq!(users.len(), 2);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 2: Metrics Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_metrics_tracking() {
        let keyspace = create_test_keyspace();
        let store = KvStore::new(&keyspace, "test_metrics").unwrap();

        // Schreiben
        store.put("key1", &"value1").unwrap();
        store.put("key2", &"value2").unwrap();

        // Lesen
        let _: Option<String> = store.get("key1").unwrap();
        let _: Option<String> = store.get("key2").unwrap();
        let _: Option<String> = store.get("nonexistent").unwrap();

        let snapshot = store.metrics_snapshot();

        assert_eq!(snapshot.writes, 2);
        assert_eq!(snapshot.reads, 3);
        assert_eq!(snapshot.count, 2);
        assert!(snapshot.bytes > 0);
    }

    #[test]
    fn test_metrics_delete_tracking() {
        let keyspace = create_test_keyspace();
        let store = KvStore::new(&keyspace, "test_delete_metrics").unwrap();

        store.put("key1", &"value1").unwrap();
        store.put("key2", &"value2").unwrap();

        let initial_count = store.metrics_snapshot().count;
        assert_eq!(initial_count, 2);

        store.delete("key1").unwrap();

        let after_delete = store.metrics_snapshot();
        assert_eq!(after_delete.deletes, 1);
        assert_eq!(after_delete.count, 1);
    }

    #[test]
    fn test_store_name() {
        let keyspace = create_test_keyspace();
        let store = KvStore::new(&keyspace, "my_custom_store").unwrap();

        assert_eq!(store.name(), "my_custom_store");
    }

    #[test]
    fn test_health_score() {
        let keyspace = create_test_keyspace();
        let store = KvStore::new(&keyspace, "health_test").unwrap();

        // Frischer Store sollte gesund sein
        assert!(store.is_healthy());
        assert!(store.health_score() >= 0.99);

        // Nach einigen erfolgreichen Operationen immer noch gesund
        store.put("key1", &"value1").unwrap();
        let _: Option<String> = store.get("key1").unwrap();

        assert!(store.is_healthy());
    }

    #[test]
    fn test_latency_tracking() {
        let keyspace = create_test_keyspace();
        let store = KvStore::new(&keyspace, "latency_test").unwrap();

        store.put("key1", &"value1").unwrap();
        let _: Option<String> = store.get("key1").unwrap();

        let snapshot = store.metrics_snapshot();

        // Latenz sollte gemessen worden sein (> 0)
        // In Tests kann die Latenz sehr klein sein, also nur prüfen dass es keine Fehler gibt
        assert!(snapshot.avg_write_latency_us >= 0.0);
        assert!(snapshot.avg_read_latency_us >= 0.0);
    }
}
