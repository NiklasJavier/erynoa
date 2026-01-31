//! # Host Interface
//!
//! Die Sandbox-Schnittstelle zwischen ECLVM und dem Erynoa-Backend.
//!
//! Die VM darf nicht direkt auf Datenbank/Storage zugreifen.
//! Stattdessen werden alle externen Operationen über dieses Interface geleitet.

use crate::error::Result;

/// Host Interface - Schnittstelle zum Erynoa Backend
///
/// Implementiere dieses Trait um der ECLVM Zugriff auf
/// Trust-Daten, Credentials und andere Erynoa-Funktionen zu geben.
pub trait HostInterface: Send + Sync {
    /// Hole Trust-Vektor für eine DID
    ///
    /// Gibt [R, I, C, P, V, Ω] zurück
    fn get_trust_vector(&self, did: &str) -> Result<[f64; 6]>;

    /// Prüfe ob DID ein bestimmtes Credential hat
    fn has_credential(&self, did: &str, schema: &str) -> Result<bool>;

    /// Hole Balance für DID
    fn get_balance(&self, did: &str) -> Result<u64>;

    /// Prüfe ob DID existiert
    fn resolve_did(&self, did: &str) -> Result<bool>;

    /// Aktueller Timestamp (Unix Seconds)
    fn get_timestamp(&self) -> u64;

    /// Log-Nachricht (für Debugging)
    fn log(&self, message: &str);
}

/// Stub-Implementation für Tests (gibt Default-Werte zurück)
#[derive(Debug, Clone, Default)]
pub struct StubHost {
    /// Default Trust-Wert
    pub default_trust: [f64; 6],
    /// Simulierte Balances (DID -> Balance)
    pub balances: std::collections::HashMap<String, u64>,
    /// Simulierte Credentials (DID -> Vec<Schema>)
    pub credentials: std::collections::HashMap<String, Vec<String>>,
    /// Simulierte DIDs
    pub known_dids: std::collections::HashSet<String>,
    /// Log-Nachrichten
    pub logs: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
}

impl StubHost {
    /// Erstelle neuen StubHost mit Default-Trust 0.5
    pub fn new() -> Self {
        Self {
            default_trust: [0.5, 0.5, 0.5, 0.5, 0.5, 0.5],
            balances: std::collections::HashMap::new(),
            credentials: std::collections::HashMap::new(),
            known_dids: std::collections::HashSet::new(),
            logs: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Setze Trust für eine DID
    pub fn with_trust(mut self, did: &str, trust: [f64; 6]) -> Self {
        // Speichere spezifischen Trust (hier vereinfacht über default)
        self.default_trust = trust;
        self.known_dids.insert(did.to_string());
        self
    }

    /// Füge DID mit Balance hinzu
    pub fn with_balance(mut self, did: &str, balance: u64) -> Self {
        self.balances.insert(did.to_string(), balance);
        self.known_dids.insert(did.to_string());
        self
    }

    /// Füge Credential hinzu
    pub fn with_credential(mut self, did: &str, schema: &str) -> Self {
        self.credentials
            .entry(did.to_string())
            .or_default()
            .push(schema.to_string());
        self.known_dids.insert(did.to_string());
        self
    }

    /// Hole geloggte Nachrichten
    pub fn get_logs(&self) -> Vec<String> {
        self.logs.lock().unwrap().clone()
    }
}

impl HostInterface for StubHost {
    fn get_trust_vector(&self, did: &str) -> Result<[f64; 6]> {
        if self.known_dids.contains(did) || self.known_dids.is_empty() {
            Ok(self.default_trust)
        } else {
            // Unbekannte DID: Newcomer Trust
            Ok([0.1, 0.1, 0.1, 0.1, 0.1, 0.1])
        }
    }

    fn has_credential(&self, did: &str, schema: &str) -> Result<bool> {
        Ok(self
            .credentials
            .get(did)
            .map(|creds| creds.contains(&schema.to_string()))
            .unwrap_or(false))
    }

    fn get_balance(&self, did: &str) -> Result<u64> {
        Ok(*self.balances.get(did).unwrap_or(&0))
    }

    fn resolve_did(&self, did: &str) -> Result<bool> {
        Ok(self.known_dids.contains(did) || self.known_dids.is_empty())
    }

    fn get_timestamp(&self) -> u64 {
        // Für Tests: fester Timestamp
        1700000000
    }

    fn log(&self, message: &str) {
        self.logs.lock().unwrap().push(message.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_host_default() {
        let host = StubHost::new();

        let trust = host.get_trust_vector("did:erynoa:self:alice").unwrap();
        assert_eq!(trust, [0.5, 0.5, 0.5, 0.5, 0.5, 0.5]);

        let balance = host.get_balance("did:erynoa:self:alice").unwrap();
        assert_eq!(balance, 0);
    }

    #[test]
    fn test_stub_host_with_balance() {
        let host = StubHost::new().with_balance("did:erynoa:self:alice", 1000);

        let balance = host.get_balance("did:erynoa:self:alice").unwrap();
        assert_eq!(balance, 1000);
    }

    #[test]
    fn test_stub_host_with_credential() {
        let host = StubHost::new().with_credential("did:erynoa:self:alice", "email-verified");

        assert!(host
            .has_credential("did:erynoa:self:alice", "email-verified")
            .unwrap());
        assert!(!host
            .has_credential("did:erynoa:self:alice", "kyc-verified")
            .unwrap());
    }

    #[test]
    fn test_stub_host_logging() {
        let host = StubHost::new();

        host.log("Test message 1");
        host.log("Test message 2");

        let logs = host.get_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], "Test message 1");
    }
}
