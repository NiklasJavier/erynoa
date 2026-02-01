//! # Erynoa Host - Die "Sinne" der ECLVM
//!
//! Implementiert das HostInterface für echten Zugriff auf Erynoa-Backend.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         ECLVM                                   │
//! │                           │                                     │
//! │                    HostInterface                                │
//! │                           │                                     │
//! └───────────────────────────┼─────────────────────────────────────┘
//!                             │
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      ErynoaHost                                 │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
//! │  │ TrustStore  │  │IdentityStore│  │ EventStore  │              │
//! │  │ get_trust() │  │ resolve()   │  │ get_depth() │              │
//! │  └─────────────┘  └─────────────┘  └─────────────┘              │
//! │                                                                 │
//! │  ┌─────────────────────────────────────────────────────────────┐│
//! │  │                    RealmStorage                             ││
//! │  │  store_get() | store_put() | store_query() | ...            ││
//! │  │  Intelligentes Prefixing | Schema-Validierung               ││
//! │  └─────────────────────────────────────────────────────────────┘│
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Sicherheit
//!
//! - Trust/Identity/Event-Zugriffe sind read-only
//! - Store-Operationen respektieren Schema-Validierung
//! - Jede Operation ist Gas-metered (in der VM)
//! - Personal-Stores nur für eigene DID zugänglich

use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::did::DID as LegacyDID;
use crate::domain::DID;
use crate::eclvm::runtime::host::{HostInterface, HostStoreValue, StoreContext};
use crate::error::Result;
use crate::local::realm_storage::StoreValue;
use crate::local::DecentralizedStorage;

/// Erynoa Host - Verbindet ECLVM mit dem echten Backend
pub struct ErynoaHost {
    /// Dezentraler Storage (enthält Trust, Identities, Events, Realm-Storage)
    storage: Arc<DecentralizedStorage>,

    /// Credential-Verifikation (vereinfacht: Schema -> DIDs die es haben)
    /// In Produktion: Verifiable Credentials mit Signaturprüfung
    credential_schemas: std::collections::HashMap<String, std::collections::HashSet<String>>,

    /// Log-Callback (optional)
    log_callback: Option<Box<dyn Fn(&str) + Send + Sync>>,

    /// Aktueller Store-Kontext (Realm + Caller-DID)
    store_context: Option<StoreContext>,
}

impl ErynoaHost {
    /// Erstelle neuen ErynoaHost mit Storage
    pub fn new(storage: Arc<DecentralizedStorage>) -> Self {
        Self {
            storage,
            credential_schemas: std::collections::HashMap::new(),
            log_callback: None,
            store_context: None,
        }
    }

    /// Builder: Füge Credential-Schema hinzu
    pub fn with_credential_holders(mut self, schema: &str, holders: Vec<String>) -> Self {
        self.credential_schemas
            .insert(schema.to_string(), holders.into_iter().collect());
        self
    }

    /// Builder: Setze Log-Callback
    pub fn with_log_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.log_callback = Some(Box::new(callback));
        self
    }

    /// Builder: Setze initialen Store-Kontext
    pub fn with_store_context(mut self, ctx: StoreContext) -> Self {
        self.store_context = Some(ctx);
        self
    }

    /// Registriere Credential für DID
    pub fn grant_credential(&mut self, did: &str, schema: &str) {
        self.credential_schemas
            .entry(schema.to_string())
            .or_default()
            .insert(did.to_string());
    }

    /// Widerrufe Credential
    pub fn revoke_credential(&mut self, did: &str, schema: &str) {
        if let Some(holders) = self.credential_schemas.get_mut(schema) {
            holders.remove(did);
        }
    }

    /// Helper: Konvertiere HostStoreValue zu internem StoreValue
    fn host_to_store_value(&self, value: HostStoreValue) -> StoreValue {
        match value {
            HostStoreValue::Null => StoreValue::Null,
            HostStoreValue::String(s) => StoreValue::String(s),
            HostStoreValue::Number(n) => StoreValue::Number(n),
            HostStoreValue::Bool(b) => StoreValue::Bool(b),
            HostStoreValue::List(items) => StoreValue::List(
                items
                    .into_iter()
                    .map(|v| self.host_to_store_value(v))
                    .collect(),
            ),
            HostStoreValue::Object(map) => StoreValue::Object(
                map.into_iter()
                    .map(|(k, v)| (k, self.host_to_store_value(v)))
                    .collect(),
            ),
        }
    }

    /// Helper: Konvertiere internes StoreValue zu HostStoreValue
    fn store_to_host_value(&self, value: StoreValue) -> HostStoreValue {
        match value {
            StoreValue::Null => HostStoreValue::Null,
            StoreValue::String(s) => HostStoreValue::String(s),
            StoreValue::Number(n) => HostStoreValue::Number(n),
            StoreValue::Bool(b) => HostStoreValue::Bool(b),
            StoreValue::Did(did) => HostStoreValue::String(did), // DID als String
            StoreValue::Timestamp(ts) => HostStoreValue::Number(ts as f64), // Timestamp als Number
            StoreValue::Bytes(bytes) => {
                // Bytes als Base64-String
                HostStoreValue::String(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &bytes,
                ))
            }
            StoreValue::List(items) => HostStoreValue::List(
                items
                    .into_iter()
                    .map(|v| self.store_to_host_value(v))
                    .collect(),
            ),
            StoreValue::Object(map) => HostStoreValue::Object(
                map.into_iter()
                    .map(|(k, v)| (k, self.store_to_host_value(v)))
                    .collect(),
            ),
        }
    }

    /// Helper: Parse DID aus Kontext
    fn get_context_did(&self) -> Result<DID> {
        let ctx = self.store_context.as_ref().ok_or_else(|| {
            crate::error::ApiError::InvalidState("Kein Store-Kontext gesetzt".into())
        })?;

        DID::from_str(&ctx.caller_did)
            .map_err(|e| crate::error::ApiError::Validation(format!("Ungültige Caller-DID: {}", e)))
    }

    /// Helper: Hole Realm-ID aus Kontext
    fn get_context_realm(&self) -> Result<&str> {
        self.store_context
            .as_ref()
            .map(|ctx| ctx.realm_id.as_str())
            .ok_or_else(|| {
                crate::error::ApiError::InvalidState("Kein Store-Kontext gesetzt".into())
            })
    }
}

impl HostInterface for ErynoaHost {
    fn get_trust_vector(&self, did: &str) -> Result<[f64; 6]> {
        // Parse DID (Legacy DID für TrustStore Kompatibilität)
        let did_parsed =
            LegacyDID::from_str(did).unwrap_or_else(|_| LegacyDID::new_self("unknown"));

        // Hole aggregierte Reputation für diese DID
        match self.storage.trust.compute_reputation(&did_parsed) {
            Ok(trust) => Ok([trust.r, trust.i, trust.c, trust.p, trust.v, trust.omega]),
            Err(_) => {
                // Fehler beim Zugriff → Newcomer (fail-safe)
                Ok([0.1, 0.1, 0.1, 0.1, 0.1, 0.1])
            }
        }
    }

    fn has_credential(&self, did: &str, schema: &str) -> Result<bool> {
        let has = self
            .credential_schemas
            .get(schema)
            .map(|holders| holders.contains(did))
            .unwrap_or(false);
        Ok(has)
    }

    fn get_balance(&self, _did: &str) -> Result<u64> {
        // TODO: Implementiere wenn Token-System existiert
        // Für jetzt: Jeder hat 0 Balance (kein Token-System)
        Ok(0)
    }

    fn resolve_did(&self, did: &str) -> Result<bool> {
        // Parse DID (Legacy DID für IdentityStore Kompatibilität)
        let did_parsed = match LegacyDID::from_str(did) {
            Ok(d) => d,
            Err(_) => return Ok(false),
        };

        // Prüfe ob DID im Identity Store existiert
        match self.storage.identities.get(&did_parsed) {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(_) => Ok(false), // Fail-safe: Als nicht existent behandeln
        }
    }

    fn get_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    fn log(&self, message: &str) {
        if let Some(callback) = &self.log_callback {
            callback(message);
        } else {
            tracing::debug!(target: "eclvm", message = %message, "VM log");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Realm Storage Operationen (Κ24: Datenintegrität)
    // ═══════════════════════════════════════════════════════════════════════

    fn set_store_context(&mut self, ctx: StoreContext) -> Result<()> {
        self.store_context = Some(ctx);
        Ok(())
    }

    fn store_get(
        &self,
        store_name: &str,
        key: &str,
        is_personal: bool,
    ) -> Result<Option<HostStoreValue>> {
        let realm_id = self.get_context_realm()?;

        let result = if is_personal {
            let did = self.get_context_did()?;
            self.storage
                .realm
                .get_personal(realm_id, &did, store_name, key)?
        } else {
            self.storage.realm.get_shared(realm_id, store_name, key)?
        };

        Ok(result.map(|v| self.store_to_host_value(v)))
    }

    fn store_put(
        &mut self,
        store_name: &str,
        key: &str,
        value: HostStoreValue,
        is_personal: bool,
    ) -> Result<()> {
        let realm_id = self.get_context_realm()?.to_string();
        let store_value = self.host_to_store_value(value);

        if is_personal {
            let did = self.get_context_did()?;
            self.storage
                .realm
                .put_personal(&realm_id, &did, store_name, key, store_value)?;
        } else {
            self.storage
                .realm
                .put_shared(&realm_id, store_name, key, store_value)?;
        }

        Ok(())
    }

    fn store_delete(&mut self, store_name: &str, key: &str, is_personal: bool) -> Result<bool> {
        let realm_id = self.get_context_realm()?.to_string();

        let existed = if is_personal {
            let did = self.get_context_did()?;
            self.storage
                .realm
                .delete_personal(&realm_id, &did, store_name, key)?
        } else {
            self.storage
                .realm
                .delete_shared(&realm_id, store_name, key)?
        };

        Ok(existed)
    }

    fn store_get_nested(
        &self,
        store_name: &str,
        key: &str,
        path: &str,
        is_personal: bool,
    ) -> Result<Option<HostStoreValue>> {
        let realm_id = self.get_context_realm()?;

        let result = if is_personal {
            let did = self.get_context_did()?;
            self.storage
                .realm
                .get_nested_personal(realm_id, &did, store_name, key, path)?
        } else {
            self.storage
                .realm
                .get_nested_shared(realm_id, store_name, key, path)?
        };

        Ok(result.map(|v| self.store_to_host_value(v)))
    }

    fn store_put_nested(
        &mut self,
        store_name: &str,
        key: &str,
        path: &str,
        value: HostStoreValue,
        is_personal: bool,
    ) -> Result<()> {
        let realm_id = self.get_context_realm()?.to_string();
        let store_value = self.host_to_store_value(value);

        if is_personal {
            let did = self.get_context_did()?;
            self.storage.realm.put_nested_personal(
                &realm_id,
                &did,
                store_name,
                key,
                path,
                store_value,
            )?;
        } else {
            self.storage
                .realm
                .put_nested_shared(&realm_id, store_name, key, path, store_value)?;
        }

        Ok(())
    }

    fn store_append_list(
        &mut self,
        store_name: &str,
        key: &str,
        path: &str,
        value: HostStoreValue,
        is_personal: bool,
    ) -> Result<usize> {
        let realm_id = self.get_context_realm()?.to_string();
        let store_value = self.host_to_store_value(value);

        let new_len = if is_personal {
            let did = self.get_context_did()?;
            self.storage.realm.append_to_list_personal(
                &realm_id,
                &did,
                store_name,
                key,
                path,
                store_value,
            )?
        } else {
            self.storage.realm.append_to_list_shared(
                &realm_id,
                store_name,
                key,
                path,
                store_value,
            )?
        };

        Ok(new_len)
    }

    fn store_exists(&self, store_name: &str, is_personal: bool) -> Result<bool> {
        let realm_id = self.get_context_realm()?;

        if is_personal {
            let did = self.get_context_did()?;
            Ok(self
                .storage
                .realm
                .store_exists_personal(realm_id, &did, store_name)?)
        } else {
            Ok(self
                .storage
                .realm
                .store_exists_shared(realm_id, store_name)?)
        }
    }

    fn store_count(&self, store_name: &str, is_personal: bool) -> Result<usize> {
        let realm_id = self.get_context_realm()?;

        if is_personal {
            let did = self.get_context_did()?;
            Ok(self
                .storage
                .realm
                .count_personal(realm_id, &did, store_name)?)
        } else {
            Ok(self.storage.realm.count_shared(realm_id, store_name)?)
        }
    }

    fn store_query_by_index(
        &self,
        store_name: &str,
        index_field: &str,
        value: &HostStoreValue,
        limit: usize,
    ) -> Result<Vec<String>> {
        let realm_id = self.get_context_realm()?;

        // Konvertiere HostStoreValue zu String für Index-Query
        let index_value = match value {
            HostStoreValue::String(s) => s.clone(),
            HostStoreValue::Number(n) => n.to_string(),
            HostStoreValue::Bool(b) => b.to_string(),
            _ => {
                return Err(crate::error::ApiError::Validation(
                    "Index-Wert muss String, Number oder Bool sein".into(),
                ))
            }
        };

        Ok(self.storage.realm.query_by_index_shared(
            realm_id,
            store_name,
            index_field,
            &index_value,
            limit,
        )?)
    }

    fn store_list_keys(
        &self,
        store_name: &str,
        prefix: Option<&str>,
        limit: usize,
        is_personal: bool,
    ) -> Result<Vec<String>> {
        let realm_id = self.get_context_realm()?;

        if is_personal {
            let did = self.get_context_did()?;
            Ok(self
                .storage
                .realm
                .list_keys_personal(realm_id, &did, store_name, prefix, limit)?)
        } else {
            Ok(self
                .storage
                .realm
                .list_keys_shared(realm_id, store_name, prefix, limit)?)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Context Builder - Für Policy-Ausführung
// ═══════════════════════════════════════════════════════════════════════════

/// Execution Context für eine Policy
///
/// Enthält alle Variablen die der Policy zur Verfügung stehen.
#[derive(Debug, Clone)]
pub struct PolicyContext {
    /// Die handelnde Entität (sender)
    pub sender: String,

    /// Optionaler Empfänger (bei Transfers etc.)
    pub receiver: Option<String>,

    /// Aktuelles Realm
    pub realm: String,

    /// Zusätzliche Parameter (z.B. amount, action, etc.)
    pub params: std::collections::HashMap<String, ContextValue>,
}

/// Wert im Policy-Context
#[derive(Debug, Clone)]
pub enum ContextValue {
    Number(f64),
    String(String),
    Bool(bool),
    Did(String),
}

impl PolicyContext {
    /// Neuer Context mit Sender
    pub fn new(sender: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            receiver: None,
            realm: "root".into(),
            params: std::collections::HashMap::new(),
        }
    }

    /// Builder: Setze Receiver
    pub fn with_receiver(mut self, receiver: impl Into<String>) -> Self {
        self.receiver = Some(receiver.into());
        self
    }

    /// Builder: Setze Realm
    pub fn with_realm(mut self, realm: impl Into<String>) -> Self {
        self.realm = realm.into();
        self
    }

    /// Builder: Füge Parameter hinzu
    pub fn with_param(mut self, key: impl Into<String>, value: ContextValue) -> Self {
        self.params.insert(key.into(), value);
        self
    }

    /// Konvertiere zu VM-Values für Stack-Initialisierung
    pub fn to_init_bytecode(&self) -> Vec<crate::eclvm::OpCode> {
        use crate::eclvm::{OpCode, Value};

        let mut ops = vec![
            // Sender DID auf Stack
            OpCode::PushConst(Value::DID(self.sender.clone())),
        ];

        // Receiver wenn vorhanden
        if let Some(ref recv) = self.receiver {
            ops.push(OpCode::PushConst(Value::DID(recv.clone())));
        }

        ops
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TrustVector6D;

    fn setup_storage() -> (Arc<DecentralizedStorage>, tempfile::TempDir) {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let storage = Arc::new(DecentralizedStorage::open(temp_dir.path()).unwrap());
        (storage, temp_dir)
    }

    #[test]
    fn test_erynoa_host_unknown_did_returns_newcomer_trust() {
        let (storage, _temp) = setup_storage();
        let host = ErynoaHost::new(storage);

        let trust = host.get_trust_vector("did:erynoa:self:unknown").unwrap();

        // Unbekannte DID mit keinen Trust-Einträgen = Newcomer Trust 0.1
        // (compute_reputation gibt default bei 0 Einträgen)
        assert!(trust[0] <= 0.5); // R sollte niedrig sein
    }

    #[test]
    fn test_erynoa_host_credential_check() {
        let (storage, _temp) = setup_storage();

        let mut host = ErynoaHost::new(storage);
        host.grant_credential("did:erynoa:self:alice", "email-verified");

        assert!(host
            .has_credential("did:erynoa:self:alice", "email-verified")
            .unwrap());
        assert!(!host
            .has_credential("did:erynoa:self:bob", "email-verified")
            .unwrap());
        assert!(!host
            .has_credential("did:erynoa:self:alice", "kyc-verified")
            .unwrap());
    }

    #[test]
    fn test_erynoa_host_resolve_did() {
        let (storage, _temp) = setup_storage();

        // Erstelle Identity für Alice
        let _identity = storage
            .identities
            .create_identity(crate::domain::DIDNamespace::Self_)
            .unwrap();

        let host = ErynoaHost::new(Arc::clone(&storage));

        // Unbekannte DID sollte nicht auflösbar sein
        assert!(!host.resolve_did("did:erynoa:self:unknown").unwrap());
    }

    #[test]
    fn test_policy_context_builder() {
        let ctx = PolicyContext::new("did:erynoa:self:alice")
            .with_receiver("did:erynoa:self:bob")
            .with_realm("marketplace")
            .with_param("amount", ContextValue::Number(100.0));

        assert_eq!(ctx.sender, "did:erynoa:self:alice");
        assert_eq!(ctx.receiver, Some("did:erynoa:self:bob".to_string()));
        assert_eq!(ctx.realm, "marketplace");
    }

    #[test]
    fn test_erynoa_host_with_real_trust() {
        let (storage, _temp) = setup_storage();

        // Erstelle zwei Identities
        let alice = storage
            .identities
            .create_identity(crate::domain::DIDNamespace::Self_)
            .unwrap();
        let bob = storage
            .identities
            .create_identity(crate::domain::DIDNamespace::Self_)
            .unwrap();

        // Bob vertraut Alice
        let trust = TrustVector6D::new(0.8, 0.7, 0.6, 0.5, 0.4, 0.3);
        storage
            .trust
            .put(bob.did.clone(), alice.did.clone(), trust)
            .unwrap();

        let host = ErynoaHost::new(Arc::clone(&storage));

        // Alice's Reputation sollte jetzt den Trust von Bob widerspiegeln
        let alice_trust = host.get_trust_vector(&alice.did.to_uri()).unwrap();

        // Mit nur einem Vertrauenden entspricht die Reputation dem Trust
        assert!((alice_trust[0] - 0.8).abs() < 0.01); // R
        assert!((alice_trust[1] - 0.7).abs() < 0.01); // I
    }

    #[test]
    fn test_erynoa_host_store_operations() {
        let (storage, _temp) = setup_storage();

        // Erstelle Alice Identity
        let alice = storage
            .identities
            .create_identity(crate::domain::DIDNamespace::Self_)
            .unwrap();

        // Erstelle Store zuerst
        use crate::local::realm_storage::{SchemaFieldType, StoreSchema};
        let realm_id = crate::domain::realm_id_from_name("test-realm");
        let schema = StoreSchema::new("posts", false)
            .with_field("title", SchemaFieldType::String)
            .with_field("likes", SchemaFieldType::Number);
        storage
            .realm
            .create_store(&realm_id, &alice.did, schema)
            .unwrap();

        let mut host = ErynoaHost::new(Arc::clone(&storage));

        // Setze Store-Kontext
        host.set_store_context(StoreContext::new("test-realm", alice.did.to_uri()))
            .unwrap();

        // Speichere Wert in Shared Store
        host.store_put(
            "posts",
            "post-1",
            HostStoreValue::Object({
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "title".to_string(),
                    HostStoreValue::String("Hello World".to_string()),
                );
                m.insert("likes".to_string(), HostStoreValue::Number(42.0));
                m
            }),
            false,
        )
        .unwrap();

        // Lese Wert zurück
        let value = host.store_get("posts", "post-1", false).unwrap().unwrap();
        match value {
            HostStoreValue::Object(map) => {
                assert_eq!(
                    map.get("title"),
                    Some(&HostStoreValue::String("Hello World".to_string()))
                );
                assert_eq!(map.get("likes"), Some(&HostStoreValue::Number(42.0)));
            }
            _ => panic!("Expected Object"),
        }
    }

    #[test]
    fn test_erynoa_host_personal_store() {
        let (storage, _temp) = setup_storage();

        let alice = storage
            .identities
            .create_identity(crate::domain::DIDNamespace::Self_)
            .unwrap();

        // Erstelle Personal-Store
        use crate::local::realm_storage::{SchemaFieldType, StoreSchema};
        let realm_id = crate::domain::realm_id_from_name("test-realm");
        let schema = StoreSchema::new("profile", true) // personal = true
            .with_field("theme", SchemaFieldType::String);
        storage
            .realm
            .create_store(&realm_id, &alice.did, schema)
            .unwrap();

        let mut host = ErynoaHost::new(Arc::clone(&storage));
        host.set_store_context(StoreContext::new("test-realm", alice.did.to_uri()))
            .unwrap();

        // Speichere persönliche Daten
        host.store_put(
            "profile",
            "settings",
            HostStoreValue::Object({
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "theme".to_string(),
                    HostStoreValue::String("dark".to_string()),
                );
                m
            }),
            true, // Personal
        )
        .unwrap();

        // Lese persönliche Daten
        let value = host
            .store_get("profile", "settings", true)
            .unwrap()
            .unwrap();
        match value {
            HostStoreValue::Object(map) => {
                assert_eq!(
                    map.get("theme"),
                    Some(&HostStoreValue::String("dark".to_string()))
                );
            }
            _ => panic!("Expected Object"),
        }
    }

    #[test]
    fn test_erynoa_host_nested_operations() {
        let (storage, _temp) = setup_storage();

        let alice = storage
            .identities
            .create_identity(crate::domain::DIDNamespace::Self_)
            .unwrap();

        // Erstelle Store mit nested Object
        use crate::local::realm_storage::{SchemaFieldType, StoreSchema};
        let realm_id = crate::domain::realm_id_from_name("test-realm");
        let schema = StoreSchema::new("users", false).with_field(
            "profile",
            SchemaFieldType::Object {
                fields: {
                    let mut m = std::collections::HashMap::new();
                    m.insert("name".to_string(), SchemaFieldType::String);
                    m.insert("age".to_string(), SchemaFieldType::Number);
                    m
                },
            },
        );
        storage
            .realm
            .create_store(&realm_id, &alice.did, schema)
            .unwrap();

        let mut host = ErynoaHost::new(Arc::clone(&storage));
        host.set_store_context(StoreContext::new("test-realm", alice.did.to_uri()))
            .unwrap();

        // Speichere komplexes Objekt
        host.store_put(
            "users",
            "user-1",
            HostStoreValue::Object({
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "profile".to_string(),
                    HostStoreValue::Object({
                        let mut p = std::collections::HashMap::new();
                        p.insert(
                            "name".to_string(),
                            HostStoreValue::String("Alice".to_string()),
                        );
                        p.insert("age".to_string(), HostStoreValue::Number(30.0));
                        p
                    }),
                );
                m
            }),
            false,
        )
        .unwrap();

        // Lese verschachtelten Wert
        let name = host
            .store_get_nested("users", "user-1", "profile.name", false)
            .unwrap()
            .unwrap();
        assert_eq!(name, HostStoreValue::String("Alice".to_string()));

        // Aktualisiere verschachtelten Wert
        host.store_put_nested(
            "users",
            "user-1",
            "profile.age",
            HostStoreValue::Number(31.0),
            false,
        )
        .unwrap();

        let age = host
            .store_get_nested("users", "user-1", "profile.age", false)
            .unwrap()
            .unwrap();
        assert_eq!(age, HostStoreValue::Number(31.0));
    }

    #[test]
    fn test_erynoa_host_list_operations() {
        let (storage, _temp) = setup_storage();

        let alice = storage
            .identities
            .create_identity(crate::domain::DIDNamespace::Self_)
            .unwrap();

        // Erstelle Store mit List
        use crate::local::realm_storage::{SchemaFieldType, StoreSchema};
        let realm_id = crate::domain::realm_id_from_name("test-realm");
        let schema = StoreSchema::new("posts", false)
            .with_field("title", SchemaFieldType::String)
            .with_field(
                "tags",
                SchemaFieldType::List {
                    item_type: Box::new(SchemaFieldType::String),
                },
            );
        storage
            .realm
            .create_store(&realm_id, &alice.did, schema)
            .unwrap();

        let mut host = ErynoaHost::new(Arc::clone(&storage));
        host.set_store_context(StoreContext::new("test-realm", alice.did.to_uri()))
            .unwrap();

        // Erstelle Objekt mit leerer Liste
        host.store_put(
            "posts",
            "post-1",
            HostStoreValue::Object({
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "title".to_string(),
                    HostStoreValue::String("Post".to_string()),
                );
                m.insert("tags".to_string(), HostStoreValue::List(vec![]));
                m
            }),
            false,
        )
        .unwrap();

        // Füge Tags hinzu
        let len1 = host
            .store_append_list(
                "posts",
                "post-1",
                "tags",
                HostStoreValue::String("rust".to_string()),
                false,
            )
            .unwrap();
        assert_eq!(len1, 1);

        let len2 = host
            .store_append_list(
                "posts",
                "post-1",
                "tags",
                HostStoreValue::String("erynoa".to_string()),
                false,
            )
            .unwrap();
        assert_eq!(len2, 2);

        // Prüfe Liste
        let tags = host
            .store_get_nested("posts", "post-1", "tags", false)
            .unwrap()
            .unwrap();
        match tags {
            HostStoreValue::List(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], HostStoreValue::String("rust".to_string()));
                assert_eq!(items[1], HostStoreValue::String("erynoa".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_erynoa_host_store_count_and_keys() {
        let (storage, _temp) = setup_storage();

        let alice = storage
            .identities
            .create_identity(crate::domain::DIDNamespace::Self_)
            .unwrap();

        // Erstelle Store
        use crate::local::realm_storage::{SchemaFieldType, StoreSchema};
        let realm_id = crate::domain::realm_id_from_name("test-realm");
        let schema = StoreSchema::new("items", false).with_field("value", SchemaFieldType::Number);
        storage
            .realm
            .create_store(&realm_id, &alice.did, schema)
            .unwrap();

        let mut host = ErynoaHost::new(Arc::clone(&storage));
        host.set_store_context(StoreContext::new("test-realm", alice.did.to_uri()))
            .unwrap();

        // Füge mehrere Einträge hinzu
        for i in 0..5 {
            host.store_put(
                "items",
                &format!("item-{}", i),
                HostStoreValue::Number(i as f64),
                false,
            )
            .unwrap();
        }

        // Zähle Einträge
        let count = host.store_count("items", false).unwrap();
        assert_eq!(count, 5);

        // Liste Keys
        let keys = host.store_list_keys("items", None, 10, false).unwrap();
        assert_eq!(keys.len(), 5);

        // Liste Keys mit Prefix
        let keys_filtered = host
            .store_list_keys("items", Some("item-1"), 10, false)
            .unwrap();
        assert_eq!(keys_filtered.len(), 1);
    }
}
