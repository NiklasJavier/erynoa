//! Low-Level Key-Value Store
//!
//! Wrapper um Fjall mit Type-Safe Serialisierung.

use anyhow::{Context, Result};
use fjall::{Keyspace, PartitionHandle};
use serde::{de::DeserializeOwned, Serialize};

/// Generic Key-Value Store über einer Fjall Partition
#[derive(Clone)]
pub struct KvStore {
    partition: PartitionHandle,
}

impl KvStore {
    /// Erstellt einen neuen KvStore mit dem angegebenen Partition-Namen
    pub fn new(keyspace: &Keyspace, name: &str) -> Result<Self> {
        let partition = keyspace
            .open_partition(name, Default::default())
            .context("Failed to open partition")?;
        Ok(Self { partition })
    }

    /// Speichert einen Wert
    pub fn put<K: AsRef<[u8]>, V: Serialize>(&self, key: K, value: &V) -> Result<()> {
        let bytes = serde_json::to_vec(value).context("Failed to serialize value")?;
        self.partition
            .insert(key.as_ref(), bytes)
            .context("Failed to insert")?;
        Ok(())
    }

    /// Holt einen Wert
    pub fn get<K: AsRef<[u8]>, V: DeserializeOwned>(&self, key: K) -> Result<Option<V>> {
        match self.partition.get(key).context("Failed to get")? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes).context("Failed to deserialize")?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Löscht einen Wert
    pub fn delete<K: AsRef<[u8]>>(&self, key: K) -> Result<bool> {
        let existed = self.partition.get(key.as_ref())?.is_some();
        self.partition
            .remove(key.as_ref())
            .context("Failed to delete")?;
        Ok(existed)
    }

    /// Prüft ob ein Key existiert
    pub fn contains<K: AsRef<[u8]>>(&self, key: K) -> Result<bool> {
        Ok(self
            .partition
            .get(key)
            .context("Failed to check key")?
            .is_some())
    }

    /// Iteriert über alle Key-Value Paare
    pub fn iter<V: DeserializeOwned>(&self) -> impl Iterator<Item = Result<(Vec<u8>, V)>> + '_ {
        self.partition.iter().map(|result| {
            let (key, value) = result.context("Failed to iterate")?;
            let deserialized: V =
                serde_json::from_slice(&value).context("Failed to deserialize")?;
            Ok((key.to_vec(), deserialized))
        })
    }

    /// Iteriert über Keys mit einem Prefix
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
}
