//! Dezentrale Storage-Schicht
//!
//! Embedded Key-Value Store basierend auf Fjall.
//! Ersetzt PostgreSQL für eine Single-Binary Architektur.

mod content_store;
mod event_store;
mod identity_store;
mod kv_store;
mod trust_store;

pub use content_store::{ContentId, ContentMetadata, ContentStore, StoredContent};
pub use event_store::{EventStore, StoredEvent};
pub use identity_store::{IdentityStore, StoredIdentity};
pub use kv_store::KvStore;
pub use trust_store::{StoredTrust, TrustStore};

use anyhow::Result;
use fjall::Keyspace;
use std::path::Path;
use std::sync::Arc;

/// Dezentraler Storage-Manager
///
/// Verwaltet alle lokalen Daten in einem einzigen Verzeichnis.
/// Kein externer Datenbank-Server erforderlich.
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
}

impl DecentralizedStorage {
    /// Öffnet oder erstellt den Storage im angegebenen Verzeichnis
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let keyspace = Arc::new(fjall::Config::new(path.as_ref().join("data")).open()?);

        let identities = IdentityStore::new(&keyspace)?;
        let events = EventStore::new(&keyspace)?;
        let trust = TrustStore::new(&keyspace)?;
        let content = ContentStore::new(&keyspace)?;

        Ok(Self {
            keyspace,
            identities,
            events,
            trust,
            content,
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

        Ok(Self {
            keyspace,
            identities,
            events,
            trust,
            content,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::did::DIDNamespace;

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
}
