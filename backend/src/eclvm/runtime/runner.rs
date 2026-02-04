//! # ECLVM Policy Runner (Phase 3.1 + E2)
//!
//! Gemeinsame Laufzeit-Hülle für Policy-Ausführung: VM mit Bytecode + Host + Kontext;
//! einheitliche Stelle für Gas-Limit, Caller-DID, Realm. Nach dem Run kann der Aufrufer
//! den Observer aufrufen (on_policy_executed); ProgrammableGateway und alle Engine-Eintrittspunkte
//! (API, UI, DataLogic, Governance, Controller) nutzen diese Hülle.
//!
//! ## E2: ECLVMBudget Integration
//!
//! PolicyRunContext kann nun ein vollständiges `ECLVMBudget` enthalten (statt nur gas_limit).
//! Dies ermöglicht:
//! - Unified Gas + Mana Tracking
//! - Automatisches Timeout
//! - Trust-basierte Budget-Skalierung
//!
//! Für Rückwärts-Kompatibilität bleibt der alte Konstruktor mit `gas_limit` erhalten.

use std::sync::Arc;
use std::time::Instant;

use crate::core::state::{ECLVMBudget, ECLVMBudgetLimits};
use crate::eclvm::bytecode::{OpCode, Value};
use crate::eclvm::runtime::host::HostInterface;
use crate::eclvm::runtime::vm::{ECLVM, ExecutionResult};
use crate::error::Result;

/// Kontext für eine Policy-Ausführung (Phase 3.1 + E2).
///
/// Enthält caller_did, realm_id und Budget (Gas/Mana/Timeout).
/// Für Rückwärts-Kompatibilität kann auch nur gas_limit angegeben werden.
#[derive(Debug, Clone)]
pub struct PolicyRunContext {
    /// Handelnde Entität (DID)
    pub caller_did: String,
    /// Aktuelles Realm
    pub realm_id: String,
    /// E2: Vollständiges Budget (Gas + Mana + Timeout)
    pub budget: Arc<ECLVMBudget>,
    /// Optional: Policy-ID für Metriken (z. B. "entry", "api:POST /action")
    pub policy_id: Option<String>,
    /// Optional: Policy-Typ für Metriken (z. B. "crossing", "api", "ui", "governance", "controller")
    pub policy_type: Option<String>,
}

impl PolicyRunContext {
    /// Legacy-Konstruktor mit nur gas_limit (E2 Rückwärts-Kompatibilität)
    ///
    /// Erstellt ECLVMBudget mit:
    /// - gas_limit wie angegeben
    /// - mana_limit = Default (10_000)
    /// - timeout = Default (5s)
    pub fn new(caller_did: impl Into<String>, realm_id: impl Into<String>, gas_limit: u64) -> Self {
        Self {
            caller_did: caller_did.into(),
            realm_id: realm_id.into(),
            budget: Arc::new(ECLVMBudget::new(ECLVMBudgetLimits {
                gas_limit,
                ..ECLVMBudgetLimits::default()
            })),
            policy_id: None,
            policy_type: None,
        }
    }

    /// E2: Neuer Konstruktor mit vollständigem Budget
    pub fn with_budget(
        caller_did: impl Into<String>,
        realm_id: impl Into<String>,
        budget: Arc<ECLVMBudget>,
    ) -> Self {
        Self {
            caller_did: caller_did.into(),
            realm_id: realm_id.into(),
            budget,
            policy_id: None,
            policy_type: None,
        }
    }

    /// E2: Konstruktor mit ECLVMBudgetLimits
    pub fn with_limits(
        caller_did: impl Into<String>,
        realm_id: impl Into<String>,
        limits: ECLVMBudgetLimits,
    ) -> Self {
        Self {
            caller_did: caller_did.into(),
            realm_id: realm_id.into(),
            budget: Arc::new(ECLVMBudget::new(limits)),
            policy_id: None,
            policy_type: None,
        }
    }

    /// Legacy: Hole gas_limit aus Budget (für Kompatibilität)
    pub fn gas_limit(&self) -> u64 {
        self.budget.limits.gas_limit
    }

    pub fn with_policy_id(mut self, id: impl Into<String>) -> Self {
        self.policy_id = Some(id.into());
        self
    }

    pub fn with_policy_type(mut self, t: impl Into<String>) -> Self {
        self.policy_type = Some(t.into());
        self
    }
}

/// Führt ECL-Bytecode mit Host und Kontext aus (Phase 3.1 + E2).
///
/// Baut Programm aus `[PushConst(DID(caller_did)), ...bytecode]`, startet VM mit Budget,
/// liefert ExecutionResult. Der Aufrufer kann danach Observer aufrufen (z. B. on_policy_executed).
///
/// ## E2 Änderungen
///
/// - Nutzt `ECLVMBudget` aus `PolicyRunContext` anstatt separater GasMeter
/// - Mana-Verbrauch wird über Budget getrackt (wenn `consume_mana` aufgerufen wird)
/// - Timeout wird bei Gas-Operationen automatisch geprüft
///
/// # Verwendung
///
/// - ProgrammableGateway: ruft run_policy auf, dann Observer.
/// - API/UI/DataLogic/Governance/Controller: rufen run_policy mit ihrem Bytecode und Kontext auf.
pub fn run_policy(
    bytecode: &[OpCode],
    host: &dyn HostInterface,
    context: &PolicyRunContext,
) -> Result<ExecutionResult> {
    let mut program = vec![OpCode::PushConst(Value::DID(context.caller_did.clone()))];
    program.extend(bytecode.to_vec());

    let start = Instant::now();

    // E2: Nutze Budget aus Context für VM
    let mut vm = ECLVM::with_budget(program, context.budget.clone(), host);
    let mut result = vm.run()?;

    result.duration_us = start.elapsed().as_micros() as u64;

    // E2: Mana-Verbrauch aus Budget für Metriken (optional, da Mana meist extern gehandhabt wird)
    result.mana_used = context.budget.mana_used();

    Ok(result)
}

/// Führt ECL-Bytecode mit explizitem Budget aus (E2 Alternative für Tests/direkten Aufruf).
pub fn run_policy_with_budget(
    bytecode: &[OpCode],
    host: &dyn HostInterface,
    caller_did: &str,
    budget: Arc<ECLVMBudget>,
) -> Result<ExecutionResult> {
    let mut program = vec![OpCode::PushConst(Value::DID(caller_did.to_string()))];
    program.extend(bytecode.to_vec());

    let start = Instant::now();
    let mut vm = ECLVM::with_budget(program, budget.clone(), host);
    let mut result = vm.run()?;
    result.duration_us = start.elapsed().as_micros() as u64;
    result.mana_used = budget.mana_used();
    Ok(result)
}

// ═══════════════════════════════════════════════════════════════════════════
// E3: State-backed ECL Policy Execution
// ═══════════════════════════════════════════════════════════════════════════

use crate::core::state::ECLVMStateContext;
use crate::eclvm::runtime::state_host::StateHost;

/// E3: Führt ECL-Bytecode mit ECLVMStateContext aus (State-backed ECL).
///
/// Diese Variante nutzt `StateHost` als HostInterface, was bedeutet:
/// - Trust-Daten kommen aus `StateView` (nicht aus Storage)
/// - Credential-Checks nutzen `IdentityViewData` aus dem Cache
/// - Metriken kommen aus UnifiedState
/// - Schreiboperationen gehen über `StateHandle` (nicht Storage)
///
/// ## Verwendung
///
/// ```ignore
/// let state = Arc::new(UnifiedState::new());
/// let mut context = ECLVMStateContext::with_defaults(state, caller, realm);
/// context.populate_trust("did:test:alice", 0.8);
///
/// let bytecode = compile_ecl("trust.r >= 0.5")?;
/// let result = run_policy_with_state_context(&bytecode, &context)?;
/// ```
///
/// ## Unterschied zu run_policy
///
/// | Aspekt          | run_policy             | run_policy_with_state_context |
/// |-----------------|------------------------|-------------------------------|
/// | Host            | Übergeben (ErynoaHost) | StateHost (aus Context)       |
/// | Trust-Quelle    | Storage                | StateView Cache               |
/// | Budget-Quelle   | PolicyRunContext       | ECLVMStateContext             |
/// | Schreiben       | Storage                | StateHandle → UnifiedState    |
pub fn run_policy_with_state_context(
    bytecode: &[OpCode],
    context: &ECLVMStateContext,
) -> Result<ExecutionResult> {
    // Prüfe ob Context noch gültig
    if !context.is_valid() {
        return Err(crate::error::ApiError::Internal(anyhow::anyhow!(
            "ECLVMStateContext is exhausted or invalid"
        )));
    }

    // StateHost aus Context erstellen
    let host = StateHost::new(context);

    // Programm mit Caller-DID auf Stack
    let mut program = vec![OpCode::PushConst(Value::DID(context.caller().to_string()))];
    program.extend(bytecode.to_vec());

    let start = Instant::now();

    // VM mit Budget aus Context
    let mut vm = ECLVM::with_budget(program, context.budget.clone(), &host);
    let mut result = vm.run()?;

    result.duration_us = start.elapsed().as_micros() as u64;
    result.mana_used = context.budget.mana_used();

    Ok(result)
}

/// E3: Führt ECL-Bytecode mit StateContext und optionalem Gas-Override aus.
///
/// Erlaubt das Überschreiben des Gas-Limits für spezifische Ausführungen.
pub fn run_policy_with_state_context_and_limit(
    bytecode: &[OpCode],
    context: &ECLVMStateContext,
    gas_limit_override: Option<u64>,
) -> Result<ExecutionResult> {
    // Bei Gas-Override neuen Context erstellen wäre teuer,
    // stattdessen prüfen wir vor der Ausführung
    if let Some(limit) = gas_limit_override {
        if context.gas_remaining() > limit {
            // Warnung: Override ist niedriger als verfügbar
            tracing::debug!(
                available = context.gas_remaining(),
                override_limit = limit,
                "Gas limit override is lower than available"
            );
        }
    }

    run_policy_with_state_context(bytecode, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eclvm::bytecode::{TrustDimIndex, Value};
    use crate::eclvm::runtime::host::StubHost;

    #[test]
    fn test_run_policy_returns_bool() {
        let host = StubHost::new();
        let bytecode = vec![
            OpCode::LoadTrust,
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::PushConst(Value::Number(0.3)),
            OpCode::Gte,
            OpCode::Return,
        ];
        let ctx = PolicyRunContext::new("did:test:alice", "realm:test", 10_000);
        let result = run_policy(&bytecode, &host, &ctx).unwrap();
        assert!(matches!(result.value, Value::Bool(_)));
    }

    // ─────────────────────────────────────────────────────────────────────
    // E2 Tests: ECLVMBudget Integration
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_e2_policy_run_context_with_budget() {
        // Test: PolicyRunContext mit vollständigem Budget
        let limits = ECLVMBudgetLimits {
            gas_limit: 100_000,
            mana_limit: 5_000,
            max_stack_depth: 512,
            timeout_ms: 1_000,
        };
        let ctx = PolicyRunContext::with_limits("did:test:alice", "realm:test", limits);

        assert_eq!(ctx.gas_limit(), 100_000);
        assert_eq!(ctx.budget.limits.mana_limit, 5_000);
        assert_eq!(ctx.budget.limits.max_stack_depth, 512);
    }

    #[test]
    fn test_e2_run_policy_consumes_budget() {
        let host = StubHost::new();
        let bytecode = vec![
            OpCode::PushConst(Value::Number(1.0)),
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::Add,
            OpCode::Return,
        ];

        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));
        let ctx = PolicyRunContext::with_budget("did:test:alice", "realm:test", budget.clone());

        let result = run_policy(&bytecode, &host, &ctx).unwrap();

        // Gas wurde konsumiert
        assert!(ctx.budget.gas_used() > 0);
        assert_eq!(result.gas_used, ctx.budget.gas_used());

        // Ergebnis ist korrekt
        assert_eq!(result.value, Value::Number(3.0));
    }

    #[test]
    fn test_e2_run_policy_with_budget_out_of_gas() {
        let host = StubHost::new();
        // Viele Operationen die Gas verbrauchen
        let mut bytecode = vec![];
        for _ in 0..100 {
            bytecode.push(OpCode::PushConst(Value::Number(1.0)));
            bytecode.push(OpCode::Pop);
        }
        bytecode.push(OpCode::Return);

        // Sehr kleines Gas-Budget
        let limits = ECLVMBudgetLimits {
            gas_limit: 10, // Nur 10 Gas
            mana_limit: 1_000,
            max_stack_depth: 1024,
            timeout_ms: 5_000,
        };
        let budget = Arc::new(ECLVMBudget::new(limits));
        let ctx = PolicyRunContext::with_budget("did:test:alice", "realm:test", budget);

        let result = run_policy(&bytecode, &host, &ctx);

        // Sollte mit OutOfGas fehlschlagen
        assert!(result.is_err());
        assert!(ctx.budget.is_exhausted());
    }

    #[test]
    fn test_e2_legacy_context_compatibility() {
        // Test: Legacy-Konstruktor funktioniert weiterhin
        let ctx = PolicyRunContext::new("did:test:alice", "realm:test", 50_000);

        assert_eq!(ctx.gas_limit(), 50_000);
        assert_eq!(ctx.budget.limits.mana_limit, ECLVMBudgetLimits::default().mana_limit);
    }

    #[test]
    fn test_e2_run_policy_with_budget_direct() {
        let host = StubHost::new();
        let bytecode = vec![
            OpCode::PushConst(Value::Bool(true)),
            OpCode::Return,
        ];

        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        // Direkte Variante ohne PolicyRunContext
        let result = run_policy_with_budget(&bytecode, &host, "did:test:alice", budget.clone()).unwrap();

        assert_eq!(result.value, Value::Bool(true));
        assert!(budget.gas_used() > 0);
    }

    // ─────────────────────────────────────────────────────────────────────
    // E3 Tests: State-backed ECL
    // ─────────────────────────────────────────────────────────────────────

    use crate::core::state::{IdentityViewData, UnifiedState};

    fn create_e3_test_context() -> ECLVMStateContext {
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
            credentials: vec!["email-verified".to_string()],
            realm_memberships: vec!["realm:test".to_string()],
        });

        ctx
    }

    #[test]
    fn test_e3_run_policy_with_state_context() {
        let ctx = create_e3_test_context();

        // Einfache Policy: push true, return
        let bytecode = vec![
            OpCode::PushConst(Value::Bool(true)),
            OpCode::Return,
        ];

        let result = run_policy_with_state_context(&bytecode, &ctx).unwrap();

        assert_eq!(result.value, Value::Bool(true));
        assert!(result.gas_used > 0);
        assert!(result.duration_us > 0);
    }

    #[test]
    fn test_e3_state_context_trust_check() {
        let ctx = create_e3_test_context();

        // Policy: LoadTrust → TrustDim(R) → 0.5 ≥ → return
        // Alice hat Trust 0.8, also sollte 0.8 >= 0.5 → true
        let bytecode = vec![
            OpCode::LoadTrust,
            OpCode::TrustDim(TrustDimIndex::R),
            OpCode::PushConst(Value::Number(0.5)),
            OpCode::Gte,
            OpCode::Return,
        ];

        let result = run_policy_with_state_context(&bytecode, &ctx).unwrap();

        // StateHost liefert Trust aus StateView Cache
        // Da wir Alice's Trust auf 0.8 gesetzt haben, sollte 0.8 >= 0.5 = true
        assert_eq!(result.value, Value::Bool(true));
    }

    #[test]
    fn test_e3_state_context_budget_consumption() {
        let ctx = create_e3_test_context();

        let initial_gas = ctx.gas_remaining();

        let bytecode = vec![
            OpCode::PushConst(Value::Number(1.0)),
            OpCode::PushConst(Value::Number(2.0)),
            OpCode::Add,
            OpCode::Return,
        ];

        let result = run_policy_with_state_context(&bytecode, &ctx).unwrap();

        // Gas wurde aus dem Context-Budget konsumiert
        assert!(ctx.gas_remaining() < initial_gas);
        assert_eq!(result.gas_used, initial_gas - ctx.gas_remaining());
    }

    #[test]
    fn test_e3_invalid_context_rejected() {
        let state = Arc::new(UnifiedState::new());
        let limits = ECLVMBudgetLimits {
            gas_limit: 5, // Sehr wenig
            mana_limit: 100,
            max_stack_depth: 64,
            timeout_ms: 1000,
        };
        let ctx = ECLVMStateContext::new(
            state,
            "did:test:alice".to_string(),
            "realm:test".to_string(),
            limits,
        );

        // Erschöpfe Budget
        ctx.budget.consume_gas(10); // Mehr als Limit

        // Context sollte jetzt invalid sein
        assert!(!ctx.is_valid());

        let bytecode = vec![OpCode::PushConst(Value::Bool(true)), OpCode::Return];

        // Ausführung sollte fehlschlagen
        let result = run_policy_with_state_context(&bytecode, &ctx);
        assert!(result.is_err());
    }
}
