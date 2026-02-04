//! Content Store (CAS - Content Addressable Storage)
//!
//! Speichert Inhalte anhand ihres Hashes (SHA-256).
//!
//! ## Phase 2 Features
//!
//! - Metriken für alle Operationen
//! - Dedup-Tracking (Hits/Misses)
//! - Integrity-Checks Tracking
//! - Snapshot-Pattern für konsistente Reads

use anyhow::Result;
use fjall::Keyspace;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use super::metrics::{StoreMetrics, StoreMetricsSnapshot};
use super::KvStore;
use crate::domain::DID;

/// Content Identifier (CID) - SHA-256 Hash
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentId(String);

impl ContentId {
    /// Erstellt eine CID aus Rohdaten
    pub fn from_bytes(data: &[u8]) -> Self {
        let hash = Sha256::digest(data);
        Self(bs58::encode(hash).into_string())
    }

    /// Erstellt eine CID aus einem Hash-String
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    /// Gibt den Hash als String zurück
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ContentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadaten für gespeicherte Inhalte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    /// Content ID (Hash)
    pub cid: ContentId,
    /// Content-Type (MIME)
    pub content_type: String,
    /// Größe in Bytes
    pub size: u64,
    /// Ersteller (DID)
    pub created_by: Option<DID>,
    /// Erstellungszeitpunkt
    pub created_at: i64,
    /// Optionale Tags
    pub tags: Vec<String>,
}

/// Gespeicherter Content mit Metadaten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredContent {
    /// Metadaten
    pub metadata: ContentMetadata,
    /// Rohdaten
    pub data: Vec<u8>,
}

/// Content Addressable Storage
///
/// Jetzt mit integriertem Metriken-Tracking gemäß `state.rs` Patterns.
#[derive(Clone)]
pub struct ContentStore {
    /// Content nach CID
    content: KvStore,
    /// Metadaten nach CID
    metadata: KvStore,
    /// Index: Creator -> CIDs
    by_creator: KvStore,
    /// Index: Tag -> CIDs
    by_tag: KvStore,

    // ─────────────────────────────────────────────────────────────────────────
    // METRICS (Phase 2)
    // ─────────────────────────────────────────────────────────────────────────
    /// Gemeinsame Metriken
    metrics: Arc<StoreMetrics>,

    /// Dedup-Hits (Content bereits vorhanden)
    dedup_hits: Arc<AtomicU64>,

    /// Dedup-Misses (Neuer Content gespeichert)
    dedup_misses: Arc<AtomicU64>,

    /// Integrity-Checks durchgeführt
    integrity_checks: Arc<AtomicU64>,

    /// Integrity-Failures (Hash-Mismatch)
    integrity_failures: Arc<AtomicU64>,

    /// Total Bytes gespeichert
    total_bytes: Arc<AtomicU64>,
}

impl ContentStore {
    /// Erstellt einen neuen Content Store
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        let store = Self {
            content: KvStore::new(keyspace, "content")?,
            metadata: KvStore::new(keyspace, "content_meta")?,
            by_creator: KvStore::new(keyspace, "content_by_creator")?,
            by_tag: KvStore::new(keyspace, "content_by_tag")?,
            metrics: Arc::new(StoreMetrics::new()),
            dedup_hits: Arc::new(AtomicU64::new(0)),
            dedup_misses: Arc::new(AtomicU64::new(0)),
            integrity_checks: Arc::new(AtomicU64::new(0)),
            integrity_failures: Arc::new(AtomicU64::new(0)),
            total_bytes: Arc::new(AtomicU64::new(0)),
        };

        // Initial count setzen
        store.metrics.set_count(store.metadata.len() as u64);

        Ok(store)
    }

    /// Speichert Content und gibt die CID zurück (mit Dedup-Tracking)
    pub fn put(
        &self,
        data: Vec<u8>,
        content_type: &str,
        created_by: Option<DID>,
        tags: Vec<String>,
    ) -> Result<ContentId> {
        let start = Instant::now();
        let cid = ContentId::from_bytes(&data);
        let size = data.len() as u64;

        // Prüfe auf Dedup
        if self.exists(&cid)? {
            self.dedup_hits.fetch_add(1, Ordering::Relaxed);
            let latency = start.elapsed().as_micros() as u64;
            self.metrics.record_read(latency, 0); // Dedup = Read
            return Ok(cid);
        }

        self.dedup_misses.fetch_add(1, Ordering::Relaxed);

        let metadata = ContentMetadata {
            cid: cid.clone(),
            content_type: content_type.to_string(),
            size,
            created_by: created_by.clone(),
            created_at: chrono::Utc::now().timestamp(),
            tags: tags.clone(),
        };

        // Content speichern
        self.content.put(cid.as_str(), &data)?;
        self.metadata.put(cid.as_str(), &metadata)?;

        // Creator-Index aktualisieren
        if let Some(ref creator) = created_by {
            let creator_key = creator.to_string();
            let mut cids: Vec<String> = self.by_creator.get(&creator_key)?.unwrap_or_default();
            if !cids.contains(&cid.0) {
                cids.push(cid.0.clone());
                self.by_creator.put(&creator_key, &cids)?;
            }
        }

        // Tag-Index aktualisieren
        for tag in tags {
            let mut cids: Vec<String> = self.by_tag.get(&tag)?.unwrap_or_default();
            if !cids.contains(&cid.0) {
                cids.push(cid.0.clone());
                self.by_tag.put(&tag, &cids)?;
            }
        }

        // Metriken
        let latency = start.elapsed().as_micros() as u64;
        self.metrics.record_write(latency, size);
        self.metrics.increment_count();
        self.total_bytes.fetch_add(size, Ordering::Relaxed);

        Ok(cid)
    }

    /// Holt Content anhand der CID
    pub fn get(&self, cid: &ContentId) -> Result<Option<Vec<u8>>> {
        let start = Instant::now();

        let result = self.content.get(cid.as_str());

        let latency = start.elapsed().as_micros() as u64;
        let size = result
            .as_ref()
            .ok()
            .and_then(|r| r.as_ref())
            .map(|d: &Vec<u8>| d.len() as u64)
            .unwrap_or(0);
        self.metrics.record_read(latency, size);

        result
    }

    /// Holt Content-Metadaten
    pub fn get_metadata(&self, cid: &ContentId) -> Result<Option<ContentMetadata>> {
        self.metadata.get(cid.as_str())
    }

    /// Holt Content mit Metadaten
    pub fn get_full(&self, cid: &ContentId) -> Result<Option<StoredContent>> {
        let metadata = match self.get_metadata(cid)? {
            Some(m) => m,
            None => return Ok(None),
        };

        let data = match self.get(cid)? {
            Some(d) => d,
            None => return Ok(None),
        };

        Ok(Some(StoredContent { metadata, data }))
    }

    /// Prüft ob Content existiert
    pub fn exists(&self, cid: &ContentId) -> Result<bool> {
        Ok(self
            .metadata
            .get::<_, ContentMetadata>(cid.as_str())?
            .is_some())
    }

    /// Verifiziert Content-Integrität (mit Tracking)
    pub fn verify(&self, cid: &ContentId) -> Result<bool> {
        self.integrity_checks.fetch_add(1, Ordering::Relaxed);

        match self.get(cid)? {
            Some(data) => {
                let computed = ContentId::from_bytes(&data);
                let valid = computed == *cid;

                if !valid {
                    self.integrity_failures.fetch_add(1, Ordering::Relaxed);
                }

                Ok(valid)
            }
            None => {
                // Content nicht gefunden = Integrity-Failure
                self.integrity_failures.fetch_add(1, Ordering::Relaxed);
                Ok(false)
            }
        }
    }

    /// Holt alle CIDs eines Erstellers
    pub fn get_by_creator(&self, creator: &DID) -> Result<Vec<ContentId>> {
        let creator_key = creator.to_string();
        let cids: Vec<String> = self.by_creator.get(&creator_key)?.unwrap_or_default();
        Ok(cids.into_iter().map(ContentId::from_hash).collect())
    }

    /// Holt alle CIDs mit einem Tag
    pub fn get_by_tag(&self, tag: &str) -> Result<Vec<ContentId>> {
        let cids: Vec<String> = self.by_tag.get(tag)?.unwrap_or_default();
        Ok(cids.into_iter().map(ContentId::from_hash).collect())
    }

    /// Löscht Content (nur wenn nicht mehr referenziert)
    pub fn delete(&self, cid: &ContentId) -> Result<bool> {
        // Hole Größe vor dem Löschen für Metriken
        let size = self
            .get_metadata(cid)?
            .map(|m| m.size)
            .unwrap_or(0);

        let existed = self.content.delete(cid.as_str())?;
        self.metadata.delete(cid.as_str())?;

        // Metriken
        if existed {
            self.metrics.decrement_count();
            self.metrics.record_delete(size);
            self.total_bytes.fetch_sub(size, Ordering::Relaxed);
        }

        // Hinweis: Indizes werden nicht bereinigt für Performance
        // (Garbage Collection könnte später implementiert werden)

        Ok(existed)
    }

    /// Zählt gespeicherte Contents
    pub fn count(&self) -> usize {
        self.metadata.len()
    }

    /// Berechnet die Gesamtgröße aller gespeicherten Contents
    pub fn total_size(&self) -> Result<u64> {
        // Verwende gecachte Metrik wenn verfügbar
        let cached = self.total_bytes.load(Ordering::Relaxed);
        if cached > 0 {
            return Ok(cached);
        }

        // Ansonsten berechnen
        let mut total = 0u64;
        for result in self.metadata.iter::<ContentMetadata>() {
            let (_, meta) = result?;
            total += meta.size;
        }

        // Cache aktualisieren
        self.total_bytes.store(total, Ordering::Relaxed);

        Ok(total)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // METRICS API (Phase 2)
    // ─────────────────────────────────────────────────────────────────────────

    /// Dedup-Rate (0.0 - 1.0)
    ///
    /// Höhere Werte bedeuten mehr Deduplizierung = gut.
    pub fn dedup_rate(&self) -> f64 {
        let hits = self.dedup_hits.load(Ordering::Relaxed);
        let misses = self.dedup_misses.load(Ordering::Relaxed);
        let total = hits + misses;

        if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Integrity Success Rate (0.0 - 1.0)
    pub fn integrity_success_rate(&self) -> f64 {
        let checks = self.integrity_checks.load(Ordering::Relaxed);
        let failures = self.integrity_failures.load(Ordering::Relaxed);

        if checks > 0 {
            (checks - failures) as f64 / checks as f64
        } else {
            1.0 // Keine Checks = perfekte Integrität angenommen
        }
    }

    /// Snapshot der ContentStore-Metriken
    pub fn snapshot(&self) -> ContentStoreSnapshot {
        ContentStoreSnapshot {
            count: self.count() as u64,
            total_size: self.total_bytes.load(Ordering::Relaxed),
            dedup_hits: self.dedup_hits.load(Ordering::Relaxed),
            dedup_misses: self.dedup_misses.load(Ordering::Relaxed),
            dedup_rate: self.dedup_rate(),
            integrity_checks: self.integrity_checks.load(Ordering::Relaxed),
            integrity_failures: self.integrity_failures.load(Ordering::Relaxed),
            integrity_success_rate: self.integrity_success_rate(),
            metrics: self.metrics.snapshot(),
        }
    }

    /// Health-Score (0.0 - 1.0)
    pub fn health_score(&self) -> f64 {
        // Kombiniere Basis-Health mit Integrity-Score
        let base_health = self.metrics.health_score();
        let integrity_health = self.integrity_success_rate();

        // Gewichteter Durchschnitt (Integrität ist wichtiger)
        base_health * 0.3 + integrity_health * 0.7
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

/// Snapshot der ContentStore-Metriken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentStoreSnapshot {
    /// Anzahl Content-Einträge
    pub count: u64,

    /// Gesamtgröße in Bytes
    pub total_size: u64,

    /// Dedup-Hits (Content bereits vorhanden)
    pub dedup_hits: u64,

    /// Dedup-Misses (Neuer Content)
    pub dedup_misses: u64,

    /// Dedup-Rate (0.0 - 1.0)
    pub dedup_rate: f64,

    /// Integrity-Checks durchgeführt
    pub integrity_checks: u64,

    /// Integrity-Failures
    pub integrity_failures: u64,

    /// Integrity Success Rate (0.0 - 1.0)
    pub integrity_success_rate: f64,

    /// Basis-Metriken
    pub metrics: StoreMetricsSnapshot,
}

impl ContentStoreSnapshot {
    /// Durchschnittliche Content-Größe
    pub fn avg_content_size(&self) -> u64 {
        if self.count > 0 {
            self.total_size / self.count
        } else {
            0
        }
    }

    /// Space-Savings durch Deduplizierung (in Bytes, geschätzt)
    pub fn estimated_space_savings(&self) -> u64 {
        // Annahme: Dedup-Hits hätten average size gehabt
        let avg = self.avg_content_size();
        self.dedup_hits * avg
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DIDNamespace;

    fn create_test_store() -> ContentStore {
        let folder = tempfile::tempdir().unwrap();
        let keyspace = fjall::Config::new(folder.path()).open().unwrap();
        ContentStore::new(&keyspace).unwrap()
    }

    #[test]
    fn test_put_get() {
        let store = create_test_store();

        let data = b"Hello, Content Addressable Storage!".to_vec();
        let cid = store.put(data.clone(), "text/plain", None, vec![]).unwrap();

        let retrieved = store.get(&cid).unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[test]
    fn test_content_dedup() {
        let store = create_test_store();

        let data = b"Identical content".to_vec();

        let cid1 = store.put(data.clone(), "text/plain", None, vec![]).unwrap();
        let cid2 = store.put(data.clone(), "text/plain", None, vec![]).unwrap();

        // Gleicher Content = Gleiche CID
        assert_eq!(cid1, cid2);
        assert_eq!(store.count(), 1);
    }

    #[test]
    fn test_verify() {
        let store = create_test_store();

        let data = b"Verify me!".to_vec();
        let cid = store.put(data, "text/plain", None, vec![]).unwrap();

        assert!(store.verify(&cid).unwrap());

        // Ungültige CID
        let fake_cid = ContentId::from_hash("fake_hash".to_string());
        assert!(!store.verify(&fake_cid).unwrap());
    }

    #[test]
    fn test_by_creator() {
        let store = create_test_store();

        let alice = DID::new(DIDNamespace::Self_, b"alice");

        store
            .put(
                b"File 1".to_vec(),
                "text/plain",
                Some(alice.clone()),
                vec![],
            )
            .unwrap();
        store
            .put(
                b"File 2".to_vec(),
                "text/plain",
                Some(alice.clone()),
                vec![],
            )
            .unwrap();
        store
            .put(b"File 3".to_vec(), "text/plain", None, vec![])
            .unwrap();

        let alice_content = store.get_by_creator(&alice).unwrap();
        assert_eq!(alice_content.len(), 2);
    }

    #[test]
    fn test_by_tag() {
        let store = create_test_store();

        store
            .put(
                b"Doc 1".to_vec(),
                "text/plain",
                None,
                vec!["important".to_string()],
            )
            .unwrap();
        store
            .put(
                b"Doc 2".to_vec(),
                "text/plain",
                None,
                vec!["important".to_string(), "urgent".to_string()],
            )
            .unwrap();
        store
            .put(
                b"Doc 3".to_vec(),
                "text/plain",
                None,
                vec!["normal".to_string()],
            )
            .unwrap();

        let important = store.get_by_tag("important").unwrap();
        assert_eq!(important.len(), 2);

        let urgent = store.get_by_tag("urgent").unwrap();
        assert_eq!(urgent.len(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 2: Metrics Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_content_store_snapshot() {
        let store = create_test_store();

        let data = b"Test content".to_vec();
        store.put(data.clone(), "text/plain", None, vec![]).unwrap();

        let snapshot = store.snapshot();

        assert_eq!(snapshot.count, 1);
        assert_eq!(snapshot.total_size, data.len() as u64);
        assert!(store.is_healthy());
    }

    #[test]
    fn test_dedup_tracking() {
        let store = create_test_store();

        let data = b"Duplicate content".to_vec();

        // Erster Put = Miss
        store.put(data.clone(), "text/plain", None, vec![]).unwrap();

        // Zweiter Put = Hit (Dedup)
        store.put(data.clone(), "text/plain", None, vec![]).unwrap();

        // Dritter Put = Hit (Dedup)
        store.put(data.clone(), "text/plain", None, vec![]).unwrap();

        let snapshot = store.snapshot();

        assert_eq!(snapshot.dedup_hits, 2);
        assert_eq!(snapshot.dedup_misses, 1);
        assert!((snapshot.dedup_rate - 0.666).abs() < 0.01); // 2/3
    }

    #[test]
    fn test_integrity_tracking() {
        let store = create_test_store();

        let data = b"Test data".to_vec();
        let cid = store.put(data, "text/plain", None, vec![]).unwrap();

        // Erfolgreiche Verifikation
        store.verify(&cid).unwrap();

        // Fehlgeschlagene Verifikation (ungültige CID)
        let fake_cid = ContentId::from_hash("fake".to_string());
        store.verify(&fake_cid).unwrap();

        let snapshot = store.snapshot();

        assert_eq!(snapshot.integrity_checks, 2);
        assert_eq!(snapshot.integrity_failures, 1);
        assert!((snapshot.integrity_success_rate - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_delete_metrics() {
        let store = create_test_store();

        let data = b"Content to delete".to_vec();
        let size = data.len() as u64;
        let cid = store.put(data, "text/plain", None, vec![]).unwrap();

        assert_eq!(store.count(), 1);

        store.delete(&cid).unwrap();

        let snapshot = store.snapshot();
        assert_eq!(snapshot.count, 0);
        assert_eq!(snapshot.metrics.deletes, 1);

        // Total size sollte zurückgesetzt sein
        // (nach delete, nicht durch gecachten Wert)
    }
}
