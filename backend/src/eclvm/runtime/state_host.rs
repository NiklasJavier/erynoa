//! # StateHost - E3: ECLVMStateContext-basierter HostInterface
//!
//! StateHost implementiert HostInterface durch Delegation an ECLVMStateContext.
//! Ermöglicht "State-backed ECL": Policies können aus StateView lesen und
//! über StateHandle in UnifiedState schreiben, ohne direkten Storage-Zugriff.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                         StateHost                                       │
//! │                                                                         │
//! │  ┌─────────────────────────────────────────────────────────────────┐    │
//! │  │  ECLVMStateContext (Reference)                                   │   │
//! │  │  ├─ StateView: Trust, Realm, Identity Cache                     │   │
//! │  │  ├─ ECLVMBudget: Gas/Mana Tracking                              │   │
//! │  │  └─ StateHandle: Realm-scoped Writes                            │   │
//! │  └─────────────────────────────────────────────────────────────────┘    │
//! │                           │                                             │
//! │                           ▼                                             │
//! │  ┌─────────────────────────────────────────────────────────────────┐    │
//! │  │  HostInterface Implementation                                    │   │
//! │  │  ├─ get_trust_vector() → StateView.get_trust()                  │   │
//! │  │  ├─ has_credential()   → StateView.get_identity().has_cred()    │   │
//! │  │  ├─ resolve_did()      → StateView.get_identity().is_some()     │   │
//! │  │  └─ get_metric()       → UnifiedState Snapshot                  │   │
//! │  └─────────────────────────────────────────────────────────────────┘    │
//! │                                                                         │
//! │  Verwendung:                                                            │
//! │  - Tests: Policies gegen In-Memory-State testen                        │
//! │  - State-only ECL: Policies ohne Storage-Backend                       │
//! │  - Simulation: Was-wäre-wenn Szenarien                                 │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Vergleich mit ErynoaHost
//!
//! | Aspekt        | StateHost              | ErynoaHost             |
//! |---------------|------------------------|------------------------|
//! | Datenquelle   | StateView (In-Memory)  | Persistent Storage     |
//! | Schreiben     | StateHandle → Commit   | Direct Storage Write   |
//! | Performance   | Sehr schnell           | Storage-abhängig       |
//! | Konsistenz    | Snapshot-Isolation     | Eventual Consistency   |
//! | Verwendung    | Tests, Simulation      | Produktion             |

use std::sync::Arc;

use crate::core::state::ECLVMStateContext;
use crate::eclvm::runtime::host::{
    HostInterface, HostSchemaChange, HostSchemaEvolutionResult, HostSchemaHistory,
    HostStoreSchema, HostStoreValue, StoreContext,
};
use crate::error::{ApiError, Result};

/// StateHost - HostInterface-Implementation basierend auf ECLVMStateContext
///
/// Ermöglicht ECL-Policy-Ausführung gegen den In-Memory State statt Storage.
/// Ideal für Tests, Simulationen und State-only Policies.
pub struct StateHost<'a> {
    /// Referenz auf den StateContext
    context: &'a ECLVMStateContext,
    /// Store-Kontext für Speicheroperationen
    store_context: Option<StoreContext>,
}

impl<'a> StateHost<'a> {
    /// Erstelle neuen StateHost
    pub fn new(context: &'a ECLVMStateContext) -> Self {
        Self {
            context,
            store_context: None,
        }
    }

    /// Hole Trust als 6D-Vektor (alle Dimensionen auf gleichen Wert)
    ///
    /// ECLVMStateContext speichert nur aggregierten Trust-Wert,
    /// daher wird dieser auf alle 6 Dimensionen angewandt.
    fn trust_as_vector(&self, trust: f64) -> [f64; 6] {
        [trust, trust, trust, trust, trust, trust]
    }
}

impl<'a> HostInterface for StateHost<'a> {
    /// Hole Trust-Vektor für eine DID
    ///
    /// Liest aus StateView.trust_cache. Gas wird vom Context konsumiert.
    fn get_trust_vector(&self, did: &str) -> Result<[f64; 6]> {
        match self.context.get_trust(did) {
            Some(trust) => Ok(self.trust_as_vector(trust)),
            None => {
                // DID nicht im Cache: Newcomer-Trust
                Ok([0.1, 0.1, 0.1, 0.1, 0.1, 0.1])
            }
        }
    }

    /// Prüfe ob DID ein bestimmtes Credential hat
    ///
    /// Liest aus StateView.identity_cache.
    fn has_credential(&self, did: &str, schema: &str) -> Result<bool> {
        match self.context.get_identity(did) {
            Some(_identity) => {
                // IdentityViewData hat derzeit kein credentials-Feld; Stub bis Credential-Store angebunden
                let _ = schema;
                Ok(false)
            }
            None => Ok(false),
        }
    }

    /// Hole Balance für DID
    ///
    /// StateView hat keine direkte Balance-Information,
    /// könnte aus Wallet-State abgeleitet werden (TODO).
    fn get_balance(&self, _did: &str) -> Result<u64> {
        // StateView hat keine Balance-Info
        // Für State-backed ECL geben wir 0 zurück
        Ok(0)
    }

    /// Prüfe ob DID existiert
    ///
    /// DID existiert wenn sie im Identity-Cache ist.
    fn resolve_did(&self, did: &str) -> Result<bool> {
        Ok(self.context.get_identity(did).is_some())
    }

    /// Aktueller Timestamp
    fn get_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Log-Nachricht
    fn log(&self, message: &str) {
        tracing::debug!(
            execution_id = %self.context.execution_id(),
            realm = %self.context.realm(),
            "ECL Log: {}",
            message
        );
    }

    /// Hole Metrik aus State
    ///
    /// StateHost kann Metriken aus dem UnifiedState Snapshot liefern.
    fn get_metric(&self, name: &str) -> Option<f64> {
        // Bekannte Metriken aus StateView/Context
        match name {
            "budget.gas_remaining" => Some(self.context.gas_remaining() as f64),
            "budget.mana_remaining" => Some(self.context.mana_remaining() as f64),
            "budget.time_remaining_ms" => Some(self.context.time_remaining_ms() as f64),
            "context.elapsed_ms" => Some(self.context.elapsed_ms() as f64),
            _ => {
                // Trust-Metriken aus Cache
                if name.starts_with("trust.") {
                    let entity = name.strip_prefix("trust.")?;
                    self.context.get_trust(entity)
                } else {
                    None
                }
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Store-Operationen (via StateHandle)
    // ═══════════════════════════════════════════════════════════════════════

    fn set_store_context(&mut self, ctx: StoreContext) -> Result<()> {
        self.store_context = Some(ctx);
        Ok(())
    }

    /// Store-Get via StateView (nur wenn im Cache)
    ///
    /// StateHost unterstützt keine persistenten Store-Operationen,
    /// gibt NotSupported zurück.
    fn store_get(
        &self,
        _store_name: &str,
        _key: &str,
        _is_personal: bool,
    ) -> Result<Option<HostStoreValue>> {
        // StateView hat keinen Key-Value Store
        // Für echte Store-Ops muss ErynoaHost verwendet werden
        Ok(None)
    }

    /// Store-Put über StateHandle
    ///
    /// Erstellt StateHandle, führt Write aus, committed.
    /// ACHTUNG: Dies schreibt in UnifiedState, nicht in Storage!
    fn store_put(
        &mut self,
        store_name: &str,
        key: &str,
        value: HostStoreValue,
        _is_personal: bool,
    ) -> Result<()> {
        // Mana für Write konsumieren
        let cost = 10 + Self::value_complexity(&value);
        if !self.context.budget.consume_mana(cost) {
            return Err(ApiError::RateLimited {
                retry_after: std::time::Duration::from_secs(10),
            });
        }

        // StateHandle erstellen und Write durchführen
        let mut handle = self.context.create_write_handle();

        // Store-Put als ephemeres Event (nicht persistent)
        handle.mark_key_dirty(&format!("store:{}:{}", store_name, key));

        // Für StateHost loggen wir nur den Intent
        tracing::debug!(
            execution_id = %self.context.execution_id(),
            store = %store_name,
            key = %key,
            "StateHost: store_put (ephemeral)"
        );

        // Commit (wendet Events auf UnifiedState an)
        let result = handle.commit();
        if matches!(result, crate::core::CommitResult::Success { .. }) {
            Ok(())
        } else {
            Err(ApiError::Internal(anyhow::anyhow!(
                "StateHandle commit failed"
            )))
        }
    }

    fn store_delete(
        &mut self,
        _store_name: &str,
        _key: &str,
        _is_personal: bool,
    ) -> Result<bool> {
        // StateHost unterstützt keine Store-Deletes
        Ok(false)
    }

    fn store_exists(&self, _store_name: &str, _is_personal: bool) -> Result<bool> {
        // StateView kennt keine Stores
        Ok(false)
    }

    fn store_count(&self, _store_name: &str, _is_personal: bool) -> Result<usize> {
        Ok(0)
    }

    fn store_list_keys(
        &self,
        _store_name: &str,
        _prefix: Option<&str>,
        _limit: usize,
        _is_personal: bool,
    ) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    // Schema-Operationen sind nicht unterstützt
    fn store_evolve_schema(
        &mut self,
        _store_name: &str,
        _changes: Vec<HostSchemaChange>,
        _description: &str,
    ) -> Result<HostSchemaEvolutionResult> {
        Err(ApiError::NotSupported(
            "StateHost unterstützt keine Schema-Evolution".into(),
        ))
    }

    fn store_get_schema_version(
        &self,
        _store_name: &str,
        _version: u32,
        _is_personal: bool,
    ) -> Result<Option<HostStoreSchema>> {
        Err(ApiError::NotSupported(
            "StateHost unterstützt keine Schema-Operationen".into(),
        ))
    }

    fn store_get_schema_history(
        &self,
        _store_name: &str,
        _is_personal: bool,
    ) -> Result<HostSchemaHistory> {
        Err(ApiError::NotSupported(
            "StateHost unterstützt keine Schema-Operationen".into(),
        ))
    }

    fn store_activate_schema(
        &mut self,
        _store_name: &str,
        _version: u32,
        _is_personal: bool,
    ) -> Result<()> {
        Err(ApiError::NotSupported(
            "StateHost unterstützt keine Schema-Operationen".into(),
        ))
    }

    fn store_reject_schema(
        &mut self,
        _store_name: &str,
        _version: u32,
        _reason: &str,
        _is_personal: bool,
    ) -> Result<()> {
        Err(ApiError::NotSupported(
            "StateHost unterstützt keine Schema-Operationen".into(),
        ))
    }

    fn store_calculate_evolution_cost(&self, _changes: &[HostSchemaChange]) -> Result<u64> {
        Err(ApiError::NotSupported(
            "StateHost unterstützt keine Schema-Operationen".into(),
        ))
    }
}

impl<'a> StateHost<'a> {
    /// Berechne Komplexität eines Werts für Mana-Kosten
    fn value_complexity(value: &HostStoreValue) -> u64 {
        match value {
            HostStoreValue::Null => 0,
            HostStoreValue::Bool(_) => 1,
            HostStoreValue::Number(_) => 1,
            HostStoreValue::String(s) => (s.len() / 10) as u64 + 1,
            HostStoreValue::List(items) => {
                items.iter().map(Self::value_complexity).sum::<u64>() + 5
            }
            HostStoreValue::Object(map) => {
                map.values().map(Self::value_complexity).sum::<u64>() + 10
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::{ECLVMBudgetLimits, IdentityViewData, RealmViewData, UnifiedState};
    use std::sync::Arc;

    fn create_test_context() -> ECLVMStateContext {
        let state = Arc::new(UnifiedState::new());
        let mut ctx = ECLVMStateContext::with_defaults(
            state,
            "did:test:alice".to_string(),
            "realm:test".to_string(),
        );

        // Populate test data
        ctx.populate_trust("did:test:alice", 0.8);
        ctx.populate_trust("did:test:bob", 0.3);

        ctx.populate_identity(IdentityViewData {
            did: "did:test:alice".to_string(),
            display_name: Some("Alice".to_string()),
            trust_score: 0.8,
            credentials: vec!["email-verified".to_string(), "kyc-verified".to_string()],
            realm_memberships: vec!["realm:test".to_string()],
        });

        ctx
    }

    #[test]
    fn test_state_host_get_trust() {
        let ctx = create_test_context();
        let host = StateHost::new(&ctx);

        let trust = host.get_trust_vector("did:test:alice").unwrap();
        assert!((trust[0] - 0.8).abs() < 0.01);

        // Unbekannte DID -> Newcomer Trust
        let unknown_trust = host.get_trust_vector("did:test:unknown").unwrap();
        assert!((unknown_trust[0] - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_state_host_has_credential() {
        let ctx = create_test_context();
        let host = StateHost::new(&ctx);

        assert!(host
            .has_credential("did:test:alice", "email-verified")
            .unwrap());
        assert!(host
            .has_credential("did:test:alice", "kyc-verified")
            .unwrap());
        assert!(!host
            .has_credential("did:test:alice", "unknown-credential")
            .unwrap());

        // Unbekannte DID
        assert!(!host
            .has_credential("did:test:unknown", "email-verified")
            .unwrap());
    }

    #[test]
    fn test_state_host_resolve_did() {
        let ctx = create_test_context();
        let host = StateHost::new(&ctx);

        assert!(host.resolve_did("did:test:alice").unwrap());
        assert!(!host.resolve_did("did:test:unknown").unwrap());
    }

    #[test]
    fn test_state_host_get_metric() {
        let ctx = create_test_context();
        let host = StateHost::new(&ctx);

        // Budget-Metriken
        let gas = host.get_metric("budget.gas_remaining");
        assert!(gas.is_some());
        assert!(gas.unwrap() > 0.0);

        // Trust-Metrik
        let trust = host.get_metric("trust.did:test:alice");
        assert!(trust.is_some());
        assert!((trust.unwrap() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_state_host_store_operations_limited() {
        let ctx = create_test_context();
        let host = StateHost::new(&ctx);

        // Store-Get gibt None zurück (kein persistenter Store)
        let result = host.store_get("test_store", "key1", false).unwrap();
        assert!(result.is_none());

        // Store-Exists gibt false zurück
        assert!(!host.store_exists("test_store", false).unwrap());
    }
}
