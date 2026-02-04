//! # ECL-Eintrittspunkte (Phase 3.2–3.6 + E3)
//!
//! Zentrale Registrierung und Ausführung von ECL-Bytecode für API, UI, DataLogic,
//! Governance und Controller. Jede Engine kann Handler (Bytecode) pro Schlüssel
//! registrieren; bei Aufruf wird `run_policy` mit Host und Observer ausgeführt.
//!
//! ## E3: State-backed ECL
//!
//! Zusätzliche `run_*_with_state` Methoden ermöglichen die Ausführung mit
//! ECLVMStateContext statt dem gespeicherten Host. Dies ist nützlich für:
//! - Tests mit simuliertem State
//! - Policies die nur In-Memory-State lesen/schreiben
//! - Was-wäre-wenn Szenarien

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::state::ECLVMStateContext;
use crate::eclvm::bytecode::{OpCode, Value};
use crate::eclvm::programmable_gateway::PolicyExecutionObserver;
use crate::eclvm::runtime::host::HostInterface;
use crate::eclvm::runtime::runner::{run_policy, run_policy_with_state_context, PolicyRunContext};
use crate::error::{ApiError, Result};

/// Standard-Gas-Limit pro ECL-Ausführung (Engine-Eintrittspunkte)
pub const DEFAULT_ENGINE_GAS_LIMIT: u64 = 50_000;

/// ECL-Eintrittspunkte: Registrierung und Ausführung pro Engine (Phase 3.2–3.6).
///
/// - **API:** Route-ID → Bytecode (z. B. POST /realm/:id/action)
/// - **UI:** Component-ID → Bytecode (Sichtbarkeit/Trust-Gate)
/// - **DataLogic:** Stream-/Aggregations-ID → Bytecode (Filter/Aggregation)
/// - **Governance:** Proposal-Type/Realm → Bytecode (Vote/Proposal-Entscheidung)
/// - **Controller:** Permission/Resource → Bytecode (AuthZ)
pub struct EclEntrypoints<H: HostInterface> {
    host: Arc<H>,
    observer: Option<Arc<dyn PolicyExecutionObserver>>,
    api_handlers: HashMap<String, Vec<OpCode>>,
    ui_handlers: HashMap<String, Vec<OpCode>>,
    datalogic_handlers: HashMap<String, Vec<OpCode>>,
    governance_handlers: HashMap<String, Vec<OpCode>>,
    controller_handlers: HashMap<String, Vec<OpCode>>,
}

impl<H: HostInterface + Send + Sync> EclEntrypoints<H> {
    pub fn new(host: Arc<H>) -> Self {
        Self {
            host,
            observer: None,
            api_handlers: HashMap::new(),
            ui_handlers: HashMap::new(),
            datalogic_handlers: HashMap::new(),
            governance_handlers: HashMap::new(),
            controller_handlers: HashMap::new(),
        }
    }

    pub fn with_observer(mut self, observer: Arc<dyn PolicyExecutionObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    pub fn register_api_handler(&mut self, route_id: impl Into<String>, bytecode: Vec<OpCode>) {
        self.api_handlers.insert(route_id.into(), bytecode);
    }

    pub fn register_ui_handler(&mut self, component_id: impl Into<String>, bytecode: Vec<OpCode>) {
        self.ui_handlers.insert(component_id.into(), bytecode);
    }

    pub fn register_datalogic_handler(
        &mut self,
        stream_or_agg_id: impl Into<String>,
        bytecode: Vec<OpCode>,
    ) {
        self.datalogic_handlers
            .insert(stream_or_agg_id.into(), bytecode);
    }

    pub fn register_governance_handler(
        &mut self,
        proposal_type_or_realm: impl Into<String>,
        bytecode: Vec<OpCode>,
    ) {
        self.governance_handlers
            .insert(proposal_type_or_realm.into(), bytecode);
    }

    pub fn register_controller_handler(
        &mut self,
        permission_or_resource: impl Into<String>,
        bytecode: Vec<OpCode>,
    ) {
        self.controller_handlers
            .insert(permission_or_resource.into(), bytecode);
    }

    fn run_and_notify(
        &self,
        bytecode: &[OpCode],
        policy_id: &str,
        policy_type: &str,
        caller_did: &str,
        realm_id: &str,
        gas_limit: u64,
    ) -> Result<Value> {
        let ctx = PolicyRunContext::new(caller_did, realm_id, gas_limit)
            .with_policy_id(policy_id)
            .with_policy_type(policy_type);
        let result = run_policy(bytecode, self.host.as_ref(), &ctx)?;
        if let Some(ref obs) = self.observer {
            let passed = result.value.as_bool().unwrap_or(false);
            obs.on_policy_executed(
                policy_id,
                policy_type,
                passed,
                result.gas_used,
                0,
                result.duration_us,
                Some(realm_id),
            );
        }
        Ok(result.value)
    }

    /// API-Engine: Führe ECL-Handler für Route aus (Phase 3.2).
    pub fn run_api(
        &self,
        route_id: &str,
        caller_did: &str,
        realm_id: &str,
        gas_limit: Option<u64>,
    ) -> Result<Value> {
        let bytecode = self
            .api_handlers
            .get(route_id)
            .ok_or_else(|| ApiError::NotFound(format!("ECL handler not found: {}", route_id)))?;
        self.run_and_notify(
            bytecode,
            route_id,
            "api",
            caller_did,
            realm_id,
            gas_limit.unwrap_or(DEFAULT_ENGINE_GAS_LIMIT),
        )
    }

    /// UI-Engine: Sichtbarkeit/Trust-Gate für Komponente (Phase 3.3).
    pub fn run_ui(
        &self,
        component_id: &str,
        caller_did: &str,
        realm_id: &str,
        gas_limit: Option<u64>,
    ) -> Result<Value> {
        let bytecode = self
            .ui_handlers
            .get(component_id)
            .ok_or_else(|| ApiError::NotFound(format!("ECL UI handler not found: {}", component_id)))?;
        self.run_and_notify(
            bytecode,
            component_id,
            "ui",
            caller_did,
            realm_id,
            gas_limit.unwrap_or(DEFAULT_ENGINE_GAS_LIMIT),
        )
    }

    /// DataLogic-Engine: Filter/Aggregation (Phase 3.4).
    pub fn run_datalogic(
        &self,
        stream_or_agg_id: &str,
        caller_did: &str,
        realm_id: &str,
        gas_limit: Option<u64>,
    ) -> Result<Value> {
        let bytecode = self.datalogic_handlers.get(stream_or_agg_id).ok_or_else(|| {
            ApiError::NotFound(format!(
                "ECL datalogic handler not found: {}",
                stream_or_agg_id
            ))
        })?;
        self.run_and_notify(
            bytecode,
            stream_or_agg_id,
            "datalogic",
            caller_did,
            realm_id,
            gas_limit.unwrap_or(DEFAULT_ENGINE_GAS_LIMIT),
        )
    }

    /// Governance-Engine: Vote/Proposal-Entscheidung (Phase 3.5).
    pub fn run_governance(
        &self,
        proposal_type_or_realm: &str,
        voter_did: &str,
        realm_id: &str,
        gas_limit: Option<u64>,
    ) -> Result<Value> {
        let bytecode = self.governance_handlers.get(proposal_type_or_realm).ok_or_else(|| {
            ApiError::NotFound(format!(
                "ECL governance handler not found: {}",
                proposal_type_or_realm
            ))
        })?;
        self.run_and_notify(
            bytecode,
            proposal_type_or_realm,
            "governance",
            voter_did,
            realm_id,
            gas_limit.unwrap_or(DEFAULT_ENGINE_GAS_LIMIT),
        )
    }

    /// Controller-Engine: AuthZ (Phase 3.6).
    pub fn run_controller(
        &self,
        permission_or_resource: &str,
        caller_did: &str,
        realm_id: &str,
        gas_limit: Option<u64>,
    ) -> Result<Value> {
        let bytecode = self
            .controller_handlers
            .get(permission_or_resource)
            .ok_or_else(|| {
                ApiError::NotFound(format!(
                    "ECL controller handler not found: {}",
                    permission_or_resource
                ))
            })?;
        self.run_and_notify(
            bytecode,
            permission_or_resource,
            "controller",
            caller_did,
            realm_id,
            gas_limit.unwrap_or(DEFAULT_ENGINE_GAS_LIMIT),
        )
    }

    pub fn has_api_handler(&self, route_id: &str) -> bool {
        self.api_handlers.contains_key(route_id)
    }
    pub fn has_ui_handler(&self, component_id: &str) -> bool {
        self.ui_handlers.contains_key(component_id)
    }
    pub fn has_controller_handler(&self, permission_or_resource: &str) -> bool {
        self.controller_handlers.contains_key(permission_or_resource)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // E3: State-backed ECL Methods
    // ═══════════════════════════════════════════════════════════════════════

    /// E3: Helper für State-backed Ausführung mit Observer-Notification
    fn run_with_state_and_notify(
        &self,
        bytecode: &[OpCode],
        policy_id: &str,
        policy_type: &str,
        context: &ECLVMStateContext,
    ) -> Result<Value> {
        let result = run_policy_with_state_context(bytecode, context)?;

        if let Some(ref obs) = self.observer {
            let passed = result.value.as_bool().unwrap_or(false);
            obs.on_policy_executed(
                policy_id,
                policy_type,
                passed,
                result.gas_used,
                result.mana_used,
                result.duration_us,
                Some(context.realm()),
            );
        }

        Ok(result.value)
    }

    /// E3: API-Handler mit StateContext ausführen
    ///
    /// Nutzt StateHost statt dem gespeicherten Host.
    /// Ideal für Tests oder State-only Policies.
    pub fn run_api_with_state(
        &self,
        route_id: &str,
        context: &ECLVMStateContext,
    ) -> Result<Value> {
        let bytecode = self
            .api_handlers
            .get(route_id)
            .ok_or_else(|| ApiError::NotFound(format!("ECL handler not found: {}", route_id)))?;

        self.run_with_state_and_notify(bytecode, route_id, "api", context)
    }

    /// E3: UI-Handler mit StateContext ausführen
    pub fn run_ui_with_state(
        &self,
        component_id: &str,
        context: &ECLVMStateContext,
    ) -> Result<Value> {
        let bytecode = self
            .ui_handlers
            .get(component_id)
            .ok_or_else(|| ApiError::NotFound(format!("ECL UI handler not found: {}", component_id)))?;

        self.run_with_state_and_notify(bytecode, component_id, "ui", context)
    }

    /// E3: DataLogic-Handler mit StateContext ausführen
    pub fn run_datalogic_with_state(
        &self,
        stream_or_agg_id: &str,
        context: &ECLVMStateContext,
    ) -> Result<Value> {
        let bytecode = self.datalogic_handlers.get(stream_or_agg_id).ok_or_else(|| {
            ApiError::NotFound(format!(
                "ECL datalogic handler not found: {}",
                stream_or_agg_id
            ))
        })?;

        self.run_with_state_and_notify(bytecode, stream_or_agg_id, "datalogic", context)
    }

    /// E3: Governance-Handler mit StateContext ausführen
    pub fn run_governance_with_state(
        &self,
        proposal_type_or_realm: &str,
        context: &ECLVMStateContext,
    ) -> Result<Value> {
        let bytecode = self.governance_handlers.get(proposal_type_or_realm).ok_or_else(|| {
            ApiError::NotFound(format!(
                "ECL governance handler not found: {}",
                proposal_type_or_realm
            ))
        })?;

        self.run_with_state_and_notify(bytecode, proposal_type_or_realm, "governance", context)
    }

    /// E3: Controller-Handler mit StateContext ausführen
    pub fn run_controller_with_state(
        &self,
        permission_or_resource: &str,
        context: &ECLVMStateContext,
    ) -> Result<Value> {
        let bytecode = self
            .controller_handlers
            .get(permission_or_resource)
            .ok_or_else(|| {
                ApiError::NotFound(format!(
                    "ECL controller handler not found: {}",
                    permission_or_resource
                ))
            })?;

        self.run_with_state_and_notify(bytecode, permission_or_resource, "controller", context)
    }
}
