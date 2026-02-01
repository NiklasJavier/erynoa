//! Event Store
//!
//! Persistiert den Event-DAG lokal.

use anyhow::Result;
use fjall::Keyspace;
use serde::{Deserialize, Serialize};

use super::KvStore;
use crate::domain::{Event, EventId, FinalityState};

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
}

impl EventStore {
    /// Erstellt einen neuen Event Store
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        Ok(Self {
            events: KvStore::new(keyspace, "events")?,
            children: KvStore::new(keyspace, "event_children")?,
            by_subject: KvStore::new(keyspace, "events_by_subject")?,
            by_realm: KvStore::new(keyspace, "events_by_realm")?,
        })
    }

    /// Speichert ein Event
    pub fn put(&self, event: Event) -> Result<()> {
        let event_id = event.id.to_string();

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

        Ok(())
    }

    /// Holt ein Event per ID
    pub fn get(&self, id: &EventId) -> Result<Option<StoredEvent>> {
        self.events.get(id.to_string())
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
            .filter_map(|s| EventId::from_hex(&s).ok())
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
            stored.finality = finality;
            stored.confirmations = confirmations;
            self.events.put(id.to_string(), &stored)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DIDNamespace, EventPayload, UniversalId, DID};

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
}
