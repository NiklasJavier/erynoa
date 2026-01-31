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
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Sicherheit
//!
//! - Alle Zugriffe sind read-only
//! - Keine Mutation des Zustands aus der VM heraus
//! - Jede Operation ist Gas-metered (in der VM)

use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::DID;
use crate::eclvm::runtime::host::HostInterface;
use crate::error::Result;
use crate::local::DecentralizedStorage;

/// Erynoa Host - Verbindet ECLVM mit dem echten Backend
pub struct ErynoaHost {
    /// Dezentraler Storage (enthält Trust, Identities, Events)
    storage: Arc<DecentralizedStorage>,

    /// Credential-Verifikation (vereinfacht: Schema -> DIDs die es haben)
    /// In Produktion: Verifiable Credentials mit Signaturprüfung
    credential_schemas: std::collections::HashMap<String, std::collections::HashSet<String>>,

    /// Log-Callback (optional)
    log_callback: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl ErynoaHost {
    /// Erstelle neuen ErynoaHost mit Storage
    pub fn new(storage: Arc<DecentralizedStorage>) -> Self {
        Self {
            storage,
            credential_schemas: std::collections::HashMap::new(),
            log_callback: None,
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
}

impl HostInterface for ErynoaHost {
    fn get_trust_vector(&self, did: &str) -> Result<[f64; 6]> {
        // Parse DID
        let did_parsed = DID::from_str(did).unwrap_or_else(|_| DID::new_self("unknown"));

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
        // Parse DID
        let did_parsed = match DID::from_str(did) {
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
}
