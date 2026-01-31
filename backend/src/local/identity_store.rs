//! Identity Store
//!
//! Speichert DIDs und zugehörige Schlüsselpaare lokal.
//! Ermöglicht Challenge-Response Authentifizierung ohne externen Auth-Server.

use anyhow::{bail, Context, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use fjall::Keyspace;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use super::KvStore;
use crate::domain::did::{DID, DIDNamespace};

/// Gespeicherte Identität
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredIdentity {
    /// DID
    pub did: DID,
    /// Public Key (Ed25519, 32 bytes, hex-encoded)
    pub public_key: String,
    /// Private Key (nur für lokale Identitäten, 32 bytes, hex-encoded)
    /// None für externe/bekannte Identitäten
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
    /// Erstellungszeitpunkt
    pub created_at: i64,
    /// Optionale Metadaten
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// Identity Store für lokale DID-Verwaltung
#[derive(Clone)]
pub struct IdentityStore {
    /// Alle bekannten Identitäten (did -> StoredIdentity)
    identities: KvStore,
    /// Public Keys Index (pubkey -> did)
    pubkey_index: KvStore,
}

impl IdentityStore {
    /// Erstellt einen neuen Identity Store
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        Ok(Self {
            identities: KvStore::new(keyspace, "identities")?,
            pubkey_index: KvStore::new(keyspace, "pubkey_index")?,
        })
    }

    /// Generiert eine neue lokale Identität mit Schlüsselpaar
    pub fn create_identity(&self, namespace: DIDNamespace) -> Result<StoredIdentity> {
        // Ed25519 Schlüsselpaar generieren
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();

        // Public Key als Hex
        let public_key_hex = hex::encode(verifying_key.as_bytes());
        let private_key_hex = hex::encode(signing_key.to_bytes());

        // DID erstellen (basierend auf Public Key, erste 16 Zeichen als unique_id)
        let did = DID::new(namespace, &public_key_hex[..16]);

        let identity = StoredIdentity {
            did: did.clone(),
            public_key: public_key_hex.clone(),
            private_key: Some(private_key_hex),
            created_at: chrono::Utc::now().timestamp(),
            metadata: std::collections::HashMap::new(),
        };

        // Speichern
        self.identities.put(did.to_string(), &identity)?;
        self.pubkey_index.put(&public_key_hex, &did.to_string())?;

        Ok(identity)
    }

    /// Importiert eine externe Identität (nur Public Key)
    pub fn import_identity(&self, did: DID, public_key: &str) -> Result<StoredIdentity> {
        let identity = StoredIdentity {
            did: did.clone(),
            public_key: public_key.to_string(),
            private_key: None,
            created_at: chrono::Utc::now().timestamp(),
            metadata: std::collections::HashMap::new(),
        };

        self.identities.put(did.to_string(), &identity)?;
        self.pubkey_index.put(public_key, &did.to_string())?;

        Ok(identity)
    }

    /// Holt eine Identität per DID
    pub fn get(&self, did: &DID) -> Result<Option<StoredIdentity>> {
        self.identities.get(did.to_string())
    }

    /// Holt eine Identität per Public Key
    pub fn get_by_pubkey(&self, public_key: &str) -> Result<Option<StoredIdentity>> {
        if let Some(did_str) = self.pubkey_index.get::<_, String>(public_key)? {
            self.identities.get(&did_str)
        } else {
            Ok(None)
        }
    }

    /// Signiert Daten mit dem Private Key einer lokalen Identität
    pub fn sign(&self, did: &DID, data: &[u8]) -> Result<Vec<u8>> {
        let identity = self.get(did)?
            .context("Identity not found")?;

        let private_key_hex = identity.private_key
            .context("Cannot sign with external identity (no private key)")?;

        let private_key_bytes = hex::decode(&private_key_hex)
            .context("Invalid private key")?;

        let signing_key = SigningKey::try_from(private_key_bytes.as_slice())
            .map_err(|e| anyhow::anyhow!("Invalid signing key: {}", e))?;

        let signature = signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    /// Verifiziert eine Signatur
    pub fn verify(&self, did: &DID, data: &[u8], signature: &[u8]) -> Result<bool> {
        let identity = self.get(did)?
            .context("Identity not found")?;

        let public_key_bytes = hex::decode(&identity.public_key)
            .context("Invalid public key")?;

        let verifying_key = VerifyingKey::try_from(public_key_bytes.as_slice())
            .map_err(|e| anyhow::anyhow!("Invalid verifying key: {}", e))?;

        let sig_bytes: [u8; 64] = signature.try_into()
            .map_err(|_| anyhow::anyhow!("Invalid signature length"))?;
        let sig = Signature::from_bytes(&sig_bytes);

        Ok(verifying_key.verify(data, &sig).is_ok())
    }

    /// Erstellt eine Challenge für Auth
    pub fn create_challenge(&self) -> String {
        let random_bytes: [u8; 32] = rand::random();
        let timestamp = chrono::Utc::now().timestamp();
        format!("erynoa-auth:{}:{}", hex::encode(random_bytes), timestamp)
    }

    /// Verifiziert eine signierte Challenge
    pub fn verify_challenge(
        &self,
        did: &DID,
        challenge: &str,
        signature: &[u8],
        max_age_secs: i64,
    ) -> Result<bool> {
        // Challenge-Format prüfen
        let parts: Vec<&str> = challenge.split(':').collect();
        if parts.len() != 3 || parts[0] != "erynoa-auth" {
            bail!("Invalid challenge format");
        }

        // Timestamp prüfen
        let timestamp: i64 = parts[2].parse().context("Invalid timestamp")?;
        let now = chrono::Utc::now().timestamp();
        if now - timestamp > max_age_secs {
            bail!("Challenge expired");
        }

        // Signatur verifizieren
        self.verify(did, challenge.as_bytes(), signature)
    }

    /// Listet alle lokalen Identitäten (mit Private Key)
    pub fn list_local(&self) -> Result<Vec<StoredIdentity>> {
        let mut local = Vec::new();
        for result in self.identities.iter::<StoredIdentity>() {
            let (_, identity) = result?;
            if identity.private_key.is_some() {
                local.push(identity);
            }
        }
        Ok(local)
    }

    /// Listet alle bekannten Identitäten
    pub fn list_all(&self) -> Result<Vec<StoredIdentity>> {
        let mut all = Vec::new();
        for result in self.identities.iter::<StoredIdentity>() {
            let (_, identity) = result?;
            all.push(identity);
        }
        Ok(all)
    }

    /// Anzahl der gespeicherten Identitäten
    pub fn count(&self) -> usize {
        self.identities.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> IdentityStore {
        let folder = tempfile::tempdir().unwrap();
        let keyspace = fjall::Config::new(folder.path()).open().unwrap();
        IdentityStore::new(&keyspace).unwrap()
    }

    #[test]
    fn test_create_identity() {
        let store = create_test_store();

        let identity = store.create_identity(DIDNamespace::Self_).unwrap();

        assert!(identity.private_key.is_some());
        assert_eq!(identity.did.namespace, DIDNamespace::Self_);
    }

    #[test]
    fn test_sign_verify() {
        let store = create_test_store();

        let identity = store.create_identity(DIDNamespace::Self_).unwrap();
        let data = b"Hello, Erynoa!";

        let signature = store.sign(&identity.did, data).unwrap();
        let valid = store.verify(&identity.did, data, &signature).unwrap();

        assert!(valid);
    }

    #[test]
    fn test_verify_wrong_data() {
        let store = create_test_store();

        let identity = store.create_identity(DIDNamespace::Self_).unwrap();
        let data = b"Hello, Erynoa!";
        let wrong_data = b"Wrong data";

        let signature = store.sign(&identity.did, data).unwrap();
        let valid = store.verify(&identity.did, wrong_data, &signature).unwrap();

        assert!(!valid);
    }

    #[test]
    fn test_challenge_auth() {
        let store = create_test_store();

        let identity = store.create_identity(DIDNamespace::Self_).unwrap();
        let challenge = store.create_challenge();

        let signature = store.sign(&identity.did, challenge.as_bytes()).unwrap();
        let valid = store.verify_challenge(&identity.did, &challenge, &signature, 60).unwrap();

        assert!(valid);
    }

    #[test]
    fn test_get_by_pubkey() {
        let store = create_test_store();

        let identity = store.create_identity(DIDNamespace::Self_).unwrap();
        let retrieved = store.get_by_pubkey(&identity.public_key).unwrap();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().did, identity.did);
    }
}
