//! Event Store
//!
//! Persistiert den Event-DAG lokal mit Metriken-Tracking.
//!
//! ## Phase 2 Features
//!
//! - DAG-spezifische Metriken (max_depth, avg_parents)
//! - Finality-Tracking
//! - Snapshot-Pattern für konsistente Reads

use anyhow::Result;
use fjall::Keyspace;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use super::metrics::{StoreMetrics, StoreMetricsSnapshot};
use super::KvStore;
use crate::domain::{Event, EventId, FinalityLevel, FinalityState};

/// Persistiertes Event mit Metadaten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// Das Event selbst
    pub event: Event,
    /// Finalitätsstatus
    pub finality: FinalityState,
    /// Anzahl Bestätigungen
    pub confirmations: u32,
    /// Persistierungszeitpunkt
    pub stored_at: i64,
}

/// Event Store für DAG-Persistierung
///
/// Jetzt mit integriertem Metriken-Tracking gemäß `state.rs` Patterns.
#[derive(Clone)]
pub struct EventStore {
    /// Events (event_id -> StoredEvent)
    events: KvStore,
    /// Parent-Index (parent_id -> vec![child_ids])
    children: KvStore,
    /// Subject-Index (subject_did -> vec![event_ids])
    by_subject: KvStore,
    /// Realm-Index (realm_id -> vec![event_ids])
    by_realm: KvStore,

    // ─────────────────────────────────────────────────────────────────────────
    // METRICS (Phase 2)
    // ─────────────────────────────────────────────────────────────────────────
    /// Gemeinsame Metriken
    metrics: Arc<StoreMetrics>,

    /// Maximale DAG-Tiefe (Atomic für lock-free Updates)
    max_depth: Arc<AtomicU64>,

    /// Durchschnittliche Anzahl Parents pro Event
    avg_parents: Arc<RwLock<f64>>,

    /// Anzahl finalisierter Events
    finalized_count: Arc<AtomicU64>,

    /// Anzahl Events mit >= 1 Bestätigung
    confirmed_count: Arc<AtomicU64>,
}

impl EventStore {
    /// Erstellt einen neuen Event Store
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        let store = Self {
            events: KvStore::new(keyspace, "events")?,
            children: KvStore::new(keyspace, "event_children")?,
            by_subject: KvStore::new(keyspace, "events_by_subject")?,
            by_realm: KvStore::new(keyspace, "events_by_realm")?,
            metrics: Arc::new(StoreMetrics::new()),
            max_depth: Arc::new(AtomicU64::new(0)),
            avg_parents: Arc::new(RwLock::new(0.0)),
            finalized_count: Arc::new(AtomicU64::new(0)),
            confirmed_count: Arc::new(AtomicU64::new(0)),
        };

        // Initial count setzen
        store.metrics.set_count(store.events.len() as u64);

        Ok(store)
    }

    /// Speichert ein Event mit Metriken-Tracking
    pub fn put(&self, event: Event) -> Result<()> {
        let start = Instant::now();
        let event_id = event.id.to_string();
        let parents_count = event.parents.len();
        let depth = event.parents.len() as u64;

        let stored = StoredEvent {
            event: event.clone(),
            finality: event.finality.clone(),
            confirmations: 0,
            stored_at: chrono::Utc::now().timestamp(),
        };

        // Event speichern
        self.events.put(&event_id, &stored)?;

        // Parent-Index aktualisieren
        for parent in &event.parents {
            let parent_key = parent.to_string();
            let mut children: Vec<String> = self.children.get(&parent_key)?.unwrap_or_default();
            if !children.contains(&event_id) {
                children.push(event_id.clone());
                self.children.put(&parent_key, &children)?;
            }
        }

        // Subject-Index aktualisieren
        let subject_key = event.author.to_string();
        let mut subject_events: Vec<String> =
            self.by_subject.get(&subject_key)?.unwrap_or_default();
        if !subject_events.contains(&event_id) {
            subject_events.push(event_id.clone());
            self.by_subject.put(&subject_key, &subject_events)?;
        }

        // Realm-Index aktualisieren (wenn Realm im Payload vorhanden)
        // Note: unified Event hat kein direktes realm_id Feld mehr
        // TODO: Extract realm_id from payload if needed for indexing

        // Metriken aktualisieren
        let latency = start.elapsed().as_micros() as u64;
        self.metrics.record_write(latency, 256); // ~256 bytes avg
        self.metrics.increment_count();

        // DAG-Metriken
        self.update_dag_metrics(depth, parents_count);

        Ok(())
    }

    /// Aktualisiert DAG-spezifische Metriken
    fn update_dag_metrics(&self, depth: u64, parents: usize) {
        // Max-Depth atomic update (CAS loop)
        loop {
            let current = self.max_depth.load(Ordering::Relaxed);
            if depth <= current {
                break;
            }
            if self
                .max_depth
                .compare_exchange(current, depth, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        // Avg parents (rolling average)
        if let Ok(mut avg) = self.avg_parents.write() {
            let total = self.count() as f64;
            if total > 0.0 {
                *avg = (*avg * (total - 1.0) + parents as f64) / total;
            }
        }
    }

    /// Holt ein Event per ID
    pub fn get(&self, id: &EventId) -> Result<Option<StoredEvent>> {
        let start = Instant::now();

        let result = self.events.get(id.to_string());

        let latency = start.elapsed().as_micros() as u64;
        if result.is_ok() {
            self.metrics.record_read(
                latency,
                if result.as_ref().unwrap().is_some() {
                    256
                } else {
                    0
                },
            );
        }

        result
    }

    /// Holt nur das Event (ohne Metadaten)
    pub fn get_event(&self, id: &EventId) -> Result<Option<Event>> {
        Ok(self.get(id)?.map(|s| s.event))
    }

    /// Prüft ob ein Event existiert
    pub fn contains(&self, id: &EventId) -> Result<bool> {
        self.events.contains(id.to_string())
    }

    /// Holt alle Kind-Events
    pub fn get_children(&self, parent_id: &EventId) -> Result<Vec<EventId>> {
        let children: Vec<String> = self
            .children
            .get(parent_id.to_string())?
            .unwrap_or_default();
        Ok(children
            .into_iter()
            .filter_map(|s| {
                // Format: "type:hex" - extrahiere nur den Hex-Teil
                if let Some(hex_part) = s.split(':').last() {
                    EventId::from_hex(hex_part).ok()
                } else {
                    EventId::from_hex(&s).ok()
                }
            })
            .collect())
    }

    /// Holt alle Events eines Subjects
    pub fn get_by_subject(&self, subject_did: &str) -> Result<Vec<StoredEvent>> {
        let ids: Vec<String> = self.by_subject.get(subject_did)?.unwrap_or_default();
        let mut events = Vec::new();
        for id in ids {
            if let Some(stored) = self.events.get::<_, StoredEvent>(&id)? {
                events.push(stored);
            }
        }
        Ok(events)
    }

    /// Holt alle Events eines Realms
    pub fn get_by_realm(&self, realm_id: &str) -> Result<Vec<StoredEvent>> {
        let ids: Vec<String> = self.by_realm.get(realm_id)?.unwrap_or_default();
        let mut events = Vec::new();
        for id in ids {
            if let Some(stored) = self.events.get::<_, StoredEvent>(&id)? {
                events.push(stored);
            }
        }
        Ok(events)
    }

    /// Aktualisiert den Finalitätsstatus
    pub fn update_finality(
        &self,
        id: &EventId,
        finality: FinalityState,
        confirmations: u32,
    ) -> Result<()> {
        if let Some(mut stored) = self.get(id)? {
            let was_finalized = stored.finality.level == FinalityLevel::Eternal;
            let was_confirmed = stored.confirmations > 0;

            stored.finality = finality.clone();
            stored.confirmations = confirmations;
            self.events.put(id.to_string(), &stored)?;

            // Finality-Metriken aktualisieren
            let is_finalized = finality.level == FinalityLevel::Eternal;
            let is_confirmed = confirmations > 0;

            if !was_finalized && is_finalized {
                self.finalized_count.fetch_add(1, Ordering::Relaxed);
            }

            if !was_confirmed && is_confirmed {
                self.confirmed_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        Ok(())
    }

    /// Zählt alle Events
    pub fn count(&self) -> usize {
        self.events.len()
    }

    /// Holt die neuesten Events (nach stored_at)
    pub fn get_recent(&self, limit: usize) -> Result<Vec<StoredEvent>> {
        let mut all: Vec<StoredEvent> = self
            .events
            .iter::<StoredEvent>()
            .filter_map(Result::ok)
            .map(|(_, e)| e)
            .collect();

        all.sort_by(|a, b| b.stored_at.cmp(&a.stored_at));
        all.truncate(limit);

        Ok(all)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // METRICS API (Phase 2)
    // ─────────────────────────────────────────────────────────────────────────

    /// Snapshot der Event-Store-Metriken
    pub fn snapshot(&self) -> EventStoreSnapshot {
        EventStoreSnapshot {
            count: self.count() as u64,
            max_depth: self.max_depth.load(Ordering::Relaxed),
            avg_parents: self.avg_parents.read().map(|v| *v).unwrap_or(0.0),
            finalized: self.finalized_count.load(Ordering::Relaxed),
            confirmed: self.confirmed_count.load(Ordering::Relaxed),
            metrics: self.metrics.snapshot(),
        }
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

/// Snapshot der EventStore-Metriken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreSnapshot {
    /// Anzahl Events
    pub count: u64,

    /// Maximale DAG-Tiefe
    pub max_depth: u64,

    /// Durchschnittliche Parents pro Event
    pub avg_parents: f64,

    /// Finalisierte Events
    pub finalized: u64,

    /// Bestätigte Events (confirmations > 0)
    pub confirmed: u64,

    /// Basis-Metriken
    pub metrics: StoreMetricsSnapshot,
}

impl EventStoreSnapshot {
    /// Finalization-Rate (0.0 - 1.0)
    pub fn finalization_rate(&self) -> f64 {
        if self.count > 0 {
            self.finalized as f64 / self.count as f64
        } else {
            0.0
        }
    }

    /// Confirmation-Rate (0.0 - 1.0)
    pub fn confirmation_rate(&self) -> f64 {
        if self.count > 0 {
            self.confirmed as f64 / self.count as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DIDNamespace, EventPayload, DID};

    fn create_test_store() -> EventStore {
        let folder = tempfile::tempdir().unwrap();
        let keyspace = fjall::Config::new(folder.path()).open().unwrap();
        EventStore::new(&keyspace).unwrap()
    }

    fn create_test_event() -> Event {
        let author = DID::new(DIDNamespace::Self_, b"test123");
        Event::genesis(author.id.clone(), author, 0)
    }

    #[test]
    fn test_put_get() {
        let store = create_test_store();
        let event = create_test_event();
        let id = event.id.clone();

        store.put(event.clone()).unwrap();

        let retrieved = store.get(&id).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().event.id, id);
    }

    #[test]
    fn test_parent_index() {
        let store = create_test_store();

        // Genesis Event
        let genesis = create_test_event();
        let genesis_id = genesis.id.clone();
        store.put(genesis.clone()).unwrap();

        // Child Event mit Genesis als Parent
        let author = DID::new(DIDNamespace::Self_, b"test456");
        let child = Event::new(
            author.id.clone(),
            vec![genesis_id.clone()],
            EventPayload::Attest {
                subject: author.id.clone(),
                claim: "test claim".to_string(),
                evidence_hash: None,
            },
            1,
        );
        store.put(child.clone()).unwrap();

        // Children abrufen
        let children = store.get_children(&genesis_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0], child.id);
    }

    #[test]
    fn test_count() {
        let store = create_test_store();

        assert_eq!(store.count(), 0);

        store.put(create_test_event()).unwrap();
        assert_eq!(store.count(), 1);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 2: Metrics Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_event_store_snapshot() {
        let store = create_test_store();

        // Events hinzufügen
        store.put(create_test_event()).unwrap();

        let snapshot = store.snapshot();

        assert_eq!(snapshot.count, 1);
        assert!(snapshot.metrics.writes >= 1);
        assert!(store.is_healthy());
    }

    #[test]
    fn test_dag_metrics() {
        let store = create_test_store();

        // Genesis Event (depth 0, 0 parents)
        let genesis = create_test_event();
        let genesis_id = genesis.id.clone();
        store.put(genesis).unwrap();

        // Child Event (depth 1, 1 parent)
        let author = DID::new(DIDNamespace::Self_, b"test456");
        let child = Event::new(
            author.id.clone(),
            vec![genesis_id.clone()],
            EventPayload::Attest {
                subject: author.id.clone(),
                claim: "test".to_string(),
                evidence_hash: None,
            },
            1,
        );
        store.put(child).unwrap();

        let snapshot = store.snapshot();

        assert_eq!(snapshot.count, 2);
        assert_eq!(snapshot.max_depth, 1);
        // avg_parents sollte zwischen 0 und 1 liegen (0 für genesis, 1 für child)
        assert!(snapshot.avg_parents >= 0.0);
        assert!(snapshot.avg_parents <= 1.0);
    }

    #[test]
    fn test_finality_tracking() {
        let store = create_test_store();

        let event = create_test_event();
        let event_id = event.id.clone();
        store.put(event).unwrap();

        // Initial: nicht finalisiert
        let snapshot = store.snapshot();
        assert_eq!(snapshot.finalized, 0);

        // Finalisieren - create an Eternal FinalityState
        let eternal_state = FinalityState {
            level: FinalityLevel::Eternal,
            probability: 1.0,
            witness_count: 10,
            min_witness_trust: 0.8,
            anchor_hash: None,
            anchor_system: None,
            updated_at: crate::domain::TemporalCoord::new(1000, 0, 0),
        };
        store
            .update_finality(&event_id, eternal_state, 10)
            .unwrap();

        let snapshot = store.snapshot();
        assert_eq!(snapshot.finalized, 1);
        assert_eq!(snapshot.confirmed, 1);
        assert!((snapshot.finalization_rate() - 1.0).abs() < 0.001);
    }
}
