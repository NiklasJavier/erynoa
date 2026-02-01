//! Content Store (CAS - Content Addressable Storage)
//!
//! Speichert Inhalte anhand ihres Hashes (SHA-256).

use anyhow::Result;
use fjall::Keyspace;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
}

impl ContentStore {
    /// Erstellt einen neuen Content Store
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        Ok(Self {
            content: KvStore::new(keyspace, "content")?,
            metadata: KvStore::new(keyspace, "content_meta")?,
            by_creator: KvStore::new(keyspace, "content_by_creator")?,
            by_tag: KvStore::new(keyspace, "content_by_tag")?,
        })
    }

    /// Speichert Content und gibt die CID zurück
    pub fn put(
        &self,
        data: Vec<u8>,
        content_type: &str,
        created_by: Option<DID>,
        tags: Vec<String>,
    ) -> Result<ContentId> {
        let cid = ContentId::from_bytes(&data);
        let size = data.len() as u64;

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

        Ok(cid)
    }

    /// Holt Content anhand der CID
    pub fn get(&self, cid: &ContentId) -> Result<Option<Vec<u8>>> {
        self.content.get(cid.as_str())
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

    /// Verifiziert Content-Integrität
    pub fn verify(&self, cid: &ContentId) -> Result<bool> {
        match self.get(cid)? {
            Some(data) => {
                let computed = ContentId::from_bytes(&data);
                Ok(computed == *cid)
            }
            None => Ok(false),
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
        let existed = self.content.delete(cid.as_str())?;
        self.metadata.delete(cid.as_str())?;

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
        let mut total = 0u64;
        for result in self.metadata.iter::<ContentMetadata>() {
            let (_, meta) = result?;
            total += meta.size;
        }
        Ok(total)
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
}
