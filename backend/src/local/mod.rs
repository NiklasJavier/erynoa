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
//! ```

mod content_store;
mod event_store;
mod identity_store;
mod kv_store;
pub mod realm_storage;
mod trust_store;

pub use content_store::{ContentId, ContentMetadata, ContentStore, StoredContent};
pub use event_store::{EventStore, StoredEvent};
pub use identity_store::{IdentityStore, StoredIdentity};
pub use kv_store::KvStore;
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

use anyhow::Result;
use fjall::Keyspace;
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
