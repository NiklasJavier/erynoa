//! # State-Backed ECL Host (Gap 2)
//!
//! Host-Implementierung, die ausschließlich aus **ECLVMStateContext** (StateView)
//! liest. Ermöglicht „State-only“ ECL-Ausführung ohne DecentralizedStorage
//! (z. B. Tests oder Policies, die nur gegen UnifiedState/Snapshot arbeiten).
//!
//! Siehe ECL-HOST-STATE-CONTEXT.md und ECL-STATE-RS-GAP-ANALYSIS.md (Gap 3.2).

use std::sync::Arc;

use crate::error::{ApiError, Result};
use crate::eclvm::runtime::host::HostInterface;

use super::state::ECLVMStateContext;

/// Host, der nur aus ECLVMStateContext/StateView liest (kein Storage).
///
/// - **Lese-Operationen:** get_trust_vector, resolve_did, get_identity-äquivalent
///   werden über `context.view` (StateView) bedient.
/// - **Schreib-Operationen:** store_put, set_store_context etc. geben
///   `NotSupported` zurück (optional später über StateHandle erweiterbar).
pub struct StateBackedHost {
    context: Arc<ECLVMStateContext>,
}

impl StateBackedHost {
    pub fn new(context: Arc<ECLVMStateContext>) -> Self {
        Self { context }
    }

    /// Referenz auf den zugrunde liegenden Kontext (z. B. für refresh_view_from_snapshot).
    pub fn context(&self) -> &ECLVMStateContext {
        &self.context
    }
}

impl HostInterface for StateBackedHost {
    fn get_trust_vector(&self, did: &str) -> Result<[f64; 6]> {
        let t = self
            .context
            .get_trust(did)
            .ok_or_else(|| ApiError::NotFound(format!("Trust for DID not in state view: {}", did)))?;
        Ok([t, t, t, t, t, t])
    }

    fn has_credential(&self, _did: &str, _schema: &str) -> Result<bool> {
        Ok(false)
    }

    fn get_balance(&self, _did: &str) -> Result<u64> {
        Ok(0)
    }

    fn resolve_did(&self, did: &str) -> Result<bool> {
        let known = self.context.get_trust(did).is_some()
            || self.context.view.get_identity(did).is_some();
        Ok(known)
    }

    fn get_timestamp(&self) -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn log(&self, message: &str) {
        tracing::trace!("[StateBackedHost] {}", message);
    }
}
