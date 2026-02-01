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
use crate::domain::did::{DIDNamespace, DID};

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
    /// Bürge (DID des vouching Users)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voucher: Option<String>,
    /// Stake-Ratio des Bürgen (0.0 - 0.3)
    #[serde(default)]
    pub vouch_stake: f64,
}

/// Vouching-Record für Bürgen-Tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VouchRecord {
    /// DID des Bürgen
    pub voucher_did: String,
    /// DID des Newcomers
    pub newcomer_did: String,
    /// Stake-Ratio (0.0 - 0.3)
    pub stake_ratio: f64,
    /// Zeitpunkt des Vouching
    pub vouched_at: i64,
    /// Status (active, revoked, penalty_applied)
    pub status: VouchStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VouchStatus {
    /// Vouching ist aktiv
    Active,
    /// Vouching wurde widerrufen (vor Fehlverhalten)
    Revoked,
    /// Penalty wurde angewendet (nach Fehlverhalten des Newcomers)
    PenaltyApplied,
}

/// Identity Store für lokale DID-Verwaltung
#[derive(Clone)]
pub struct IdentityStore {
    /// Alle bekannten Identitäten (did -> StoredIdentity)
    identities: KvStore,
    /// Public Keys Index (pubkey -> did)
    pubkey_index: KvStore,
    /// Vouching Records (voucher_did:newcomer_did -> VouchRecord)
    vouch_records: KvStore,
    /// Passkey Credentials (credential_id -> StoredPasskeyCredential)
    passkey_credentials: KvStore,
    /// Passkey DID Index (did -> credential_id)
    passkey_did_index: KvStore,
}

impl IdentityStore {
    /// Erstellt einen neuen Identity Store
    pub fn new(keyspace: &Keyspace) -> Result<Self> {
        Ok(Self {
            identities: KvStore::new(keyspace, "identities")?,
            pubkey_index: KvStore::new(keyspace, "pubkey_index")?,
            vouch_records: KvStore::new(keyspace, "vouch_records")?,
            passkey_credentials: KvStore::new(keyspace, "passkey_credentials")?,
            passkey_did_index: KvStore::new(keyspace, "passkey_did_index")?,
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
            voucher: None,
            vouch_stake: 0.0,
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
            voucher: None,
            vouch_stake: 0.0,
        };

        self.identities.put(did.to_string(), &identity)?;
        self.pubkey_index.put(public_key, &did.to_string())?;

        Ok(identity)
    }

    /// Erstellt eine gebürgte Identität (Vouching/Invite-System)
    ///
    /// Der Bürge (voucher) "staked" einen Teil seines Trust-Kapitals.
    /// Bei Fehlverhalten des Newcomers wird auch der Bürge bestraft.
    pub fn create_vouched_identity(
        &self,
        namespace: DIDNamespace,
        voucher_did: &DID,
        stake_ratio: f64,
    ) -> Result<StoredIdentity> {
        // Stake begrenzen auf max 30%
        let stake = stake_ratio.clamp(0.0, 0.3);

        // Prüfen ob Bürge existiert
        let voucher = self
            .get(voucher_did)?
            .context("Voucher identity not found")?;

        // Bürge muss lokale Identität sein (mit Private Key)
        if voucher.private_key.is_none() {
            bail!("Only local identities can vouch for newcomers");
        }

        // Ed25519 Schlüsselpaar für Newcomer generieren
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let public_key_hex = hex::encode(verifying_key.as_bytes());
        let private_key_hex = hex::encode(signing_key.to_bytes());

        let did = DID::new(namespace, &public_key_hex[..16]);

        let identity = StoredIdentity {
            did: did.clone(),
            public_key: public_key_hex.clone(),
            private_key: Some(private_key_hex),
            created_at: chrono::Utc::now().timestamp(),
            metadata: std::collections::HashMap::new(),
            voucher: Some(voucher_did.to_string()),
            vouch_stake: stake,
        };

        // Vouching Record speichern
        let vouch_key = format!("{}:{}", voucher_did, did);
        let vouch_record = VouchRecord {
            voucher_did: voucher_did.to_string(),
            newcomer_did: did.to_string(),
            stake_ratio: stake,
            vouched_at: chrono::Utc::now().timestamp(),
            status: VouchStatus::Active,
        };

        self.identities.put(did.to_string(), &identity)?;
        self.pubkey_index.put(&public_key_hex, &did.to_string())?;
        self.vouch_records.put(&vouch_key, &vouch_record)?;

        Ok(identity)
    }

    /// Holt alle Vouching-Records für einen Bürgen
    pub fn get_vouches_by_voucher(&self, voucher_did: &DID) -> Result<Vec<VouchRecord>> {
        let prefix = voucher_did.to_string();
        let mut records = Vec::new();

        // Iteriere über alle Records und filtere nach Voucher
        for result in self.vouch_records.iter::<VouchRecord>() {
            let (key, record) = result?;
            if key.starts_with(prefix.as_bytes()) {
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Markiert einen Vouch-Record als "Penalty Applied"
    pub fn apply_vouch_penalty(&self, voucher_did: &DID, newcomer_did: &DID) -> Result<()> {
        let key = format!("{}:{}", voucher_did, newcomer_did);

        if let Some(mut record) = self.vouch_records.get::<_, VouchRecord>(&key)? {
            record.status = VouchStatus::PenaltyApplied;
            self.vouch_records.put(&key, &record)?;
        }

        Ok(())
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
        let identity = self.get(did)?.context("Identity not found")?;

        let private_key_hex = identity
            .private_key
            .context("Cannot sign with external identity (no private key)")?;

        let private_key_bytes = hex::decode(&private_key_hex).context("Invalid private key")?;

        let signing_key = SigningKey::try_from(private_key_bytes.as_slice())
            .map_err(|e| anyhow::anyhow!("Invalid signing key: {}", e))?;

        let signature = signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    /// Verifiziert eine Signatur
    pub fn verify(&self, did: &DID, data: &[u8], signature: &[u8]) -> Result<bool> {
        let identity = self.get(did)?.context("Identity not found")?;

        let public_key_bytes = hex::decode(&identity.public_key).context("Invalid public key")?;

        let verifying_key = VerifyingKey::try_from(public_key_bytes.as_slice())
            .map_err(|e| anyhow::anyhow!("Invalid verifying key: {}", e))?;

        let sig_bytes: [u8; 64] = signature
            .try_into()
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

    // ========================================================================
    // PASSKEY CREDENTIAL METHODS
    // ========================================================================

    /// Speichert ein Passkey Credential
    pub fn store_passkey_credential(
        &self,
        credential: &crate::api::v1::auth::StoredPasskeyCredential,
    ) -> Result<()> {
        // Speichere Credential unter credential_id
        self.passkey_credentials
            .put(&credential.credential_id, credential)?;

        // Index: DID -> credential_id
        self.passkey_did_index
            .put(&credential.did, &credential.credential_id)?;

        // Optional: Auch als StoredIdentity speichern für Kompatibilität
        // mit dem bestehenden Identitätssystem
        let namespace = credential
            .namespace
            .parse::<DIDNamespace>()
            .unwrap_or(DIDNamespace::Self_);

        let did = DID::new(namespace, &credential.public_key_hex[..16]);

        let identity = StoredIdentity {
            did: did.clone(),
            public_key: credential.public_key_hex.clone(),
            private_key: None, // Passkey = kein lokaler Private Key
            created_at: credential.created_at,
            metadata: {
                let mut m = std::collections::HashMap::new();
                m.insert("passkey".to_string(), "true".to_string());
                m.insert(
                    "credential_id".to_string(),
                    credential.credential_id.clone(),
                );
                m.insert("algorithm".to_string(), credential.algorithm.to_string());
                m
            },
            voucher: None,
            vouch_stake: 0.0,
        };

        self.identities.put(credential.did.clone(), &identity)?;
        self.pubkey_index
            .put(&credential.public_key_hex, &credential.did)?;

        Ok(())
    }

    /// Holt ein Passkey Credential per Credential ID
    pub fn get_passkey_credential(
        &self,
        credential_id: &str,
    ) -> Result<Option<crate::api::v1::auth::StoredPasskeyCredential>> {
        self.passkey_credentials.get(credential_id)
    }

    /// Holt ein Passkey Credential per DID
    pub fn get_passkey_credential_by_did(
        &self,
        did: &str,
    ) -> Result<Option<crate::api::v1::auth::StoredPasskeyCredential>> {
        if let Some(credential_id) = self.passkey_did_index.get::<_, String>(did)? {
            self.passkey_credentials.get(&credential_id)
        } else {
            Ok(None)
        }
    }

    /// Aktualisiert den last_used_at Timestamp eines Passkey Credentials
    pub fn update_passkey_last_used(&self, credential_id: &str) -> Result<()> {
        if let Some(mut credential) =
            self.passkey_credentials
                .get::<_, crate::api::v1::auth::StoredPasskeyCredential>(credential_id)?
        {
            credential.last_used_at = Some(chrono::Utc::now().timestamp());
            credential.sign_count += 1;
            self.passkey_credentials.put(credential_id, &credential)?;
        }
        Ok(())
    }

    /// Löscht ein Passkey Credential
    pub fn delete_passkey_credential(&self, credential_id: &str) -> Result<()> {
        if let Some(credential) =
            self.passkey_credentials
                .get::<_, crate::api::v1::auth::StoredPasskeyCredential>(credential_id)?
        {
            // Lösche Indizes
            self.passkey_did_index.delete(&credential.did)?;
            self.pubkey_index.delete(&credential.public_key_hex)?;
            self.identities.delete(&credential.did)?;

            // Lösche Credential
            self.passkey_credentials.delete(credential_id)?;
        }
        Ok(())
    }

    /// Listet alle Passkey Credentials
    pub fn list_passkey_credentials(
        &self,
    ) -> Result<Vec<crate::api::v1::auth::StoredPasskeyCredential>> {
        let mut credentials = Vec::new();
        for result in self
            .passkey_credentials
            .iter::<crate::api::v1::auth::StoredPasskeyCredential>()
        {
            let (_, credential) = result?;
            credentials.push(credential);
        }
        Ok(credentials)
    }

    /// Anzahl der gespeicherten Passkey Credentials
    pub fn passkey_count(&self) -> usize {
        self.passkey_credentials.len()
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
        let valid = store
            .verify_challenge(&identity.did, &challenge, &signature, 60)
            .unwrap();

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

    #[test]
    fn test_vouched_identity() {
        let store = create_test_store();

        // Bürge erstellen
        let voucher = store.create_identity(DIDNamespace::Self_).unwrap();

        // Gebürgte Identität erstellen
        let newcomer = store
            .create_vouched_identity(
                DIDNamespace::Self_,
                &voucher.did,
                0.2, // 20% stake
            )
            .unwrap();

        // Newcomer hat Voucher-Referenz
        assert_eq!(newcomer.voucher, Some(voucher.did.to_string()));
        assert!((newcomer.vouch_stake - 0.2).abs() < 0.001);

        // Vouch-Record wurde gespeichert
        let vouches = store.get_vouches_by_voucher(&voucher.did).unwrap();
        assert_eq!(vouches.len(), 1);
        assert_eq!(vouches[0].newcomer_did, newcomer.did.to_string());
        assert_eq!(vouches[0].status, VouchStatus::Active);
    }

    #[test]
    fn test_vouch_penalty() {
        let store = create_test_store();

        let voucher = store.create_identity(DIDNamespace::Self_).unwrap();
        let newcomer = store
            .create_vouched_identity(DIDNamespace::Self_, &voucher.did, 0.2)
            .unwrap();

        // Penalty anwenden
        store
            .apply_vouch_penalty(&voucher.did, &newcomer.did)
            .unwrap();

        // Status geändert
        let vouches = store.get_vouches_by_voucher(&voucher.did).unwrap();
        assert_eq!(vouches[0].status, VouchStatus::PenaltyApplied);
    }

    #[test]
    fn test_vouch_stake_clamped() {
        let store = create_test_store();

        let voucher = store.create_identity(DIDNamespace::Self_).unwrap();

        // Stake > 30% wird auf 30% begrenzt
        let newcomer = store
            .create_vouched_identity(
                DIDNamespace::Self_,
                &voucher.did,
                0.9, // 90% wird auf 30% begrenzt
            )
            .unwrap();

        assert!((newcomer.vouch_stake - 0.3).abs() < 0.001);
    }
}
