//! # Programmable Gateway Guard
//!
//! ECLVM-integrierter Gateway Guard für programmierbare Realm-Policies.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                    Programmable Gateway                             │
//! │                                                                     │
//! │   Request: cross(alice, root, finance)                              │
//! │       │                                                             │
//! │       ▼                                                             │
//! │   ┌───────────────────────────────────────────────────────────┐     │
//! │   │  1. Load Policy: realm.finance.entry_policy               │     │
//! │   │  2. Build Context: { sender: alice, target: finance }     │     │
//! │   │  3. Execute ECLVM                                         │     │
//! │   │  4. Return: allow/deny                                    │     │
//! │   └───────────────────────────────────────────────────────────┘     │
//! │                                                                     │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Policies
//!
//! Policies werden als kompilierter Bytecode gespeichert und können:
//! - Trust-Schwellen prüfen
//! - Credentials verifizieren
//! - Zeitbasierte Regeln anwenden
//! - Komplexe Logik mit AND/OR kombinieren

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::state::{ECLVMBudget, ECLVMBudgetLimits};
use crate::domain::{RealmId, TrustVector6D, DID};
use crate::eclvm::bytecode::{OpCode, TrustDimIndex, Value};
use crate::eclvm::mana::{ManaConfig, ManaManager};
use crate::eclvm::runtime::host::HostInterface;
use crate::eclvm::runtime::runner::{run_policy, PolicyRunContext};
use crate::error::{ApiError, Result};

// =============================================================================
// Policy Execution Observer (Phase 1.1 – ECLVMState-Metriken)
// =============================================================================

/// Observer für Policy-Ausführung und Crossing-Entscheidungen.
/// Wird nach jeder ECL-Policy-Ausführung bzw. Crossing-Evaluation aufgerufen,
/// damit StateIntegrator ECLVMState (policies_executed, crossing_*, etc.) befüllen kann.
pub trait PolicyExecutionObserver: Send + Sync {
    /// Policy ausgeführt (Gas/Mana/Dauer für Metriken).
    fn on_policy_executed(
        &self,
        policy_id: &str,
        policy_type: &str,
        passed: bool,
        gas_used: u64,
        mana_used: u64,
        duration_us: u64,
        realm_id: Option<&str>,
    );

    /// Crossing-Policy evaluiert (von Realm A nach Realm B).
    fn on_crossing_policy_evaluated(
        &self,
        from_realm: &str,
        to_realm: &str,
        entity_id: &str,
        allowed: bool,
        trust_score: f64,
        policy_id: Option<&str>,
    );
}

// =============================================================================
// ECL Crossing Evaluator (Phase 1.2 – Gateway-Vereinheitlichung)
// =============================================================================
//
// Trait für GatewayGuard: optional ECL-basierte Crossing-Prüfung pro Realm.
// ProgrammableGateway implementiert dieses Trait.

/// ECL-basierte Crossing-Validierung (für GatewayGuard-Integration).
pub trait EclCrossingEvaluator: Send + Sync {
    /// Validiere Crossing mit ECL-Entry-Policy des Ziel-Realms.
    /// Returns: Ok(true) = erlaubt, Ok(false) = verweigert, Err = Fehler (z. B. VM/Policy-Fehler).
    fn validate_ecl_crossing(
        &self,
        sender: &DID,
        sender_trust: &TrustVector6D,
        from_realm: &RealmId,
        to_realm: &RealmId,
    ) -> Result<bool>;
}

impl<H: HostInterface + Send + Sync> EclCrossingEvaluator for ProgrammableGateway<H> {
    fn validate_ecl_crossing(
        &self,
        sender: &DID,
        sender_trust: &TrustVector6D,
        from_realm: &RealmId,
        to_realm: &RealmId,
    ) -> Result<bool> {
        let decision = self.validate_crossing(sender, sender_trust, from_realm, to_realm)?;
        Ok(decision.allowed)
    }
}

/// Kompilierte Policy für ein Realm
#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    /// Name der Policy
    pub name: String,
    /// Beschreibung
    pub description: String,
    /// Kompilierter Bytecode
    pub bytecode: Vec<OpCode>,
    /// Geschätzter Gas-Verbrauch
    pub estimated_gas: u64,
}

impl CompiledPolicy {
    /// Erstelle neue Policy
    pub fn new(name: impl Into<String>, bytecode: Vec<OpCode>) -> Self {
        let bytecode_clone = bytecode.clone();
        Self {
            name: name.into(),
            description: String::new(),
            estimated_gas: bytecode_clone.iter().map(|op| op.gas_cost()).sum(),
            bytecode,
        }
    }

    /// Mit Beschreibung
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Programmable Gateway Guard (E2: ECLVMBudget Integration)
pub struct ProgrammableGateway<H: HostInterface> {
    /// Host Interface für VM
    host: Arc<H>,

    /// Policies pro Realm (realm_id -> policy_name -> policy)
    policies: HashMap<RealmId, HashMap<String, CompiledPolicy>>,

    /// Default Entry Policy (wenn Realm keine eigene hat)
    default_entry_policy: CompiledPolicy,

    /// Legacy: Mana Manager für Rate Limiting (wird noch für ManaStatus genutzt)
    mana_manager: ManaManager,

    /// E2: Default Budget-Limits (können pro Ausführung überschrieben werden)
    default_budget_limits: ECLVMBudgetLimits,

    /// E2: Ob ECLVMBudget verwendet werden soll (statt separatem ManaManager)
    use_unified_budget: bool,

    /// Optional: Observer für Metriken (ECLVMState via StateIntegrator)
    observer: Option<Arc<dyn PolicyExecutionObserver>>,
}

impl<H: HostInterface> ProgrammableGateway<H> {
    /// Erstelle neuen Gateway
    pub fn new(host: Arc<H>) -> Self {
        Self {
            host,
            policies: HashMap::new(),
            default_entry_policy: Self::create_default_entry_policy(),
            mana_manager: ManaManager::new(ManaConfig::default()),
            default_budget_limits: ECLVMBudgetLimits {
                gas_limit: 50_000,
                mana_limit: 10_000,
                max_stack_depth: 1024,
                timeout_ms: 5_000,
            },
            use_unified_budget: true, // E2: Default ist unified Budget
            observer: None,
        }
    }

    /// Mit Custom Mana-Config (Legacy)
    pub fn with_mana_config(mut self, config: ManaConfig) -> Self {
        self.mana_manager = ManaManager::new(config);
        self.use_unified_budget = false; // Fallback auf Legacy
        self
    }

    /// E2: Mit Custom Budget-Limits
    pub fn with_budget_limits(mut self, limits: ECLVMBudgetLimits) -> Self {
        self.default_budget_limits = limits;
        self.use_unified_budget = true;
        self
    }

    /// Mit Custom Gas-Limit (Legacy-Kompatibilität)
    pub fn with_max_gas(mut self, max_gas: u64) -> Self {
        self.default_budget_limits.gas_limit = max_gas;
        self
    }

    /// Optional: Observer für ECLVMState-Metriken (StateIntegrator)
    pub fn with_observer(mut self, observer: Arc<dyn PolicyExecutionObserver>) -> Self {
        self.observer = Some(observer);
        self
    }

    /// Default Entry Policy: Trust.R >= 0.3
    fn create_default_entry_policy() -> CompiledPolicy {
        CompiledPolicy::new(
            "default_entry",
            vec![
                // Sender-DID ist bereits auf Stack (wird von execute_policy gepusht)
                OpCode::LoadTrust,
                OpCode::TrustDim(TrustDimIndex::R),
                OpCode::PushConst(Value::Number(0.3)),
                OpCode::Gte,
                OpCode::Return,
            ],
        )
        .with_description("Default entry policy: Trust.R >= 0.3")
    }

    /// Registriere Policy für Realm
    pub fn register_policy(
        &mut self,
        realm: RealmId,
        policy_name: impl Into<String>,
        policy: CompiledPolicy,
    ) {
        self.policies
            .entry(realm)
            .or_default()
            .insert(policy_name.into(), policy);
    }

    /// Registriere Entry Policy für Realm
    pub fn register_entry_policy(&mut self, realm: RealmId, policy: CompiledPolicy) {
        self.register_policy(realm, "entry", policy);
    }

    /// Hole Policy für Realm
    pub fn get_policy(&self, realm: &RealmId, policy_name: &str) -> Option<&CompiledPolicy> {
        self.policies
            .get(realm)
            .and_then(|policies| policies.get(policy_name))
    }

    /// Führe Policy aus (Phase 3.1 + E2: ECLVMBudget)
    ///
    /// Returns: (allowed, gas_used, mana_used, duration_us) für Metriken/Observer
    pub fn execute_policy(
        &self,
        policy: &CompiledPolicy,
        sender: &DID,
        sender_trust: &TrustVector6D,
    ) -> Result<(bool, u64, u64, u64)> {
        if self.use_unified_budget {
            // E2: Unified ECLVMBudget für Gas + Mana + Timeout
            self.execute_policy_with_budget(policy, sender, sender_trust)
        } else {
            // Legacy: Separater ManaManager
            self.execute_policy_legacy(policy, sender, sender_trust)
        }
    }

    /// E2: Policy-Ausführung mit ECLVMBudget
    fn execute_policy_with_budget(
        &self,
        policy: &CompiledPolicy,
        sender: &DID,
        sender_trust: &TrustVector6D,
    ) -> Result<(bool, u64, u64, u64)> {
        // 1. Erstelle Budget mit Trust-basierter Skalierung
        let trust_factor = sender_trust.r as f64; // R-Dimension für Reliability
        let limits = self.default_budget_limits.with_trust_factor(trust_factor);
        let budget = Arc::new(ECLVMBudget::new(limits));

        // 2. Pre-Flight: Mana für Policy reservieren
        // Bei niedrigem Trust: weniger Mana-Budget, also eher Rate-Limited
        let mana_cost = (policy.estimated_gas / 10).max(10); // ~10% des Gas als Mana
        if !budget.consume_mana(mana_cost) {
            return Err(ApiError::RateLimited {
                retry_after: std::time::Duration::from_secs(10),
            });
        }

        // 3. Runner: VM mit Bytecode + Host + Budget
        let ctx = PolicyRunContext::with_budget(
            sender.to_uri(),
            "", // realm wird bei validate_crossing/validate_entry gesetzt
            budget.clone(),
        )
        .with_policy_id(&policy.name)
        .with_policy_type("crossing");

        let result = run_policy(&policy.bytecode, self.host.as_ref(), &ctx)?;

        // 4. Interpretiere Ergebnis
        let allowed = match result.value {
            Value::Bool(a) => a,
            _ => {
                return Err(ApiError::Internal(anyhow::anyhow!(
                    "Policy returned non-boolean value: {:?}",
                    result.value
                )));
            }
        };

        // 5. Hole finale Metriken aus Budget
        let gas_used = budget.gas_used();
        let mana_used = budget.mana_used();

        Ok((allowed, gas_used, mana_used, result.duration_us))
    }

    /// Legacy: Policy-Ausführung mit separatem ManaManager
    fn execute_policy_legacy(
        &self,
        policy: &CompiledPolicy,
        sender: &DID,
        sender_trust: &TrustVector6D,
    ) -> Result<(bool, u64, u64, u64)> {
        // 1. Pre-Flight: Mana-Check
        self.mana_manager
            .preflight_check(&sender.to_uri(), sender_trust, policy.estimated_gas)?;

        // 2. Runner: VM mit Bytecode + Host + Kontext
        let ctx = PolicyRunContext::new(
            sender.to_uri(),
            "",
            self.default_budget_limits.gas_limit,
        )
        .with_policy_id(&policy.name)
        .with_policy_type("crossing");
        let result = run_policy(&policy.bytecode, self.host.as_ref(), &ctx)?;

        // 3. Deduct Mana
        self.mana_manager
            .deduct(&sender.to_uri(), sender_trust, result.gas_used)?;

        // 4. Interpretiere Ergebnis
        let allowed = match result.value {
            Value::Bool(a) => a,
            _ => {
                return Err(ApiError::Internal(anyhow::anyhow!(
                    "Policy returned non-boolean value: {:?}",
                    result.value
                )));
            }
        };
        Ok((allowed, result.gas_used, 0, result.duration_us))
    }

    /// Validiere Realm-Entry
    pub fn validate_entry(
        &self,
        sender: &DID,
        sender_trust: &TrustVector6D,
        target_realm: &RealmId,
    ) -> Result<GatewayDecision> {
        // Hole Entry Policy für Ziel-Realm (oder Default)
        let policy = self
            .get_policy(target_realm, "entry")
            .unwrap_or(&self.default_entry_policy);

        // E2: execute_policy liefert jetzt auch mana_used
        let (allowed, gas_used, mana_used, duration_us) = self.execute_policy(policy, sender, sender_trust)?;

        Ok(GatewayDecision {
            allowed,
            sender: sender.clone(),
            target_realm: target_realm.clone(),
            policy_name: policy.name.clone(),
            message: if allowed {
                "Entry allowed".to_string()
            } else {
                format!("Entry denied by policy '{}'", policy.name)
            },
            gas_used,
            mana_used, // E2: Jetzt korrekt aus Budget
            duration_us,
        })
    }

    /// Validiere Realm-Crossing (von einem Realm zu einem anderen)
    pub fn validate_crossing(
        &self,
        sender: &DID,
        sender_trust: &TrustVector6D,
        from_realm: &RealmId,
        to_realm: &RealmId,
    ) -> Result<GatewayDecision> {
        let decision = self.validate_entry(sender, sender_trust, to_realm)?;

        if let Some(ref obs) = self.observer {
            let trust_score = sender_trust.weighted_norm(&[1.0_f32; 6]) as f64;
            let to_realm_str = decision.target_realm.to_string();
            let from_realm_str = from_realm.to_string();
            obs.on_policy_executed(
                &decision.policy_name,
                "crossing",
                decision.allowed,
                decision.gas_used,
                decision.mana_used,
                decision.duration_us,
                Some(&to_realm_str),
            );
            obs.on_crossing_policy_evaluated(
                &from_realm_str,
                &to_realm_str,
                &sender.to_uri(),
                decision.allowed,
                trust_score,
                Some(&decision.policy_name),
            );
        }

        Ok(decision)
    }
}

/// Ergebnis einer Gateway-Entscheidung
#[derive(Debug, Clone)]
pub struct GatewayDecision {
    pub allowed: bool,
    pub sender: DID,
    pub target_realm: RealmId,
    pub policy_name: String,
    pub message: String,
    /// Gas verbraucht bei ECL-Ausführung (für Metriken)
    pub gas_used: u64,
    /// Mana verbraucht (optional; VM trackt derzeit nur Gas)
    pub mana_used: u64,
    /// Ausführungsdauer in Mikrosekunden (für avg_evaluation_time_us)
    pub duration_us: u64,
}

// ═══════════════════════════════════════════════════════════════════════════
// Standard Policies - Vorgefertigte Policies
// ═══════════════════════════════════════════════════════════════════════════

/// Vorgefertigte Standard-Policies
pub struct StandardPolicies;

impl StandardPolicies {
    /// Public Realm: Jeder darf rein (Trust > 0)
    pub fn public_realm() -> CompiledPolicy {
        CompiledPolicy::new(
            "public",
            vec![
                OpCode::LoadTrust,
                OpCode::TrustDim(TrustDimIndex::R),
                OpCode::PushConst(Value::Number(0.0)),
                OpCode::Gt,
                OpCode::Return,
            ],
        )
        .with_description("Public realm: Any non-zero trust allowed")
    }

    /// Verified Users Only: Trust.R >= 0.5 UND email-verified
    pub fn verified_users() -> CompiledPolicy {
        CompiledPolicy::new(
            "verified_users",
            vec![
                // DID ist auf Stack
                OpCode::Dup, // [did, did]
                // Trust Check
                OpCode::LoadTrust,
                OpCode::TrustDim(TrustDimIndex::R),
                OpCode::PushConst(Value::Number(0.5)),
                OpCode::Gte, // [did, trust_ok]
                // Credential Check
                OpCode::Swap, // [trust_ok, did]
                OpCode::PushConst(Value::String("email-verified".into())),
                OpCode::HasCredential, // [trust_ok, has_cred]
                // Beide müssen true sein
                OpCode::And,
                OpCode::Return,
            ],
        )
        .with_description("Verified users: Trust.R >= 0.5 AND email-verified credential")
    }

    /// High Trust Only: Trust.R >= 0.7
    pub fn high_trust() -> CompiledPolicy {
        CompiledPolicy::new(
            "high_trust",
            vec![
                OpCode::LoadTrust,
                OpCode::TrustDim(TrustDimIndex::R),
                OpCode::PushConst(Value::Number(0.7)),
                OpCode::Gte,
                OpCode::Return,
            ],
        )
        .with_description("High trust: Trust.R >= 0.7")
    }

    /// Finance Realm: Trust.R >= 0.7 UND KYC
    pub fn finance_realm() -> CompiledPolicy {
        CompiledPolicy::new(
            "finance_entry",
            vec![
                OpCode::Dup,
                // Trust Check
                OpCode::LoadTrust,
                OpCode::TrustDim(TrustDimIndex::R),
                OpCode::PushConst(Value::Number(0.7)),
                OpCode::Gte,
                // KYC Check
                OpCode::Swap,
                OpCode::PushConst(Value::String("kyc-verified".into())),
                OpCode::HasCredential,
                // Both required
                OpCode::And,
                OpCode::Return,
            ],
        )
        .with_description("Finance realm: Trust.R >= 0.7 AND KYC verified")
    }

    /// Invite Only: Muss "invited" Credential haben
    pub fn invite_only() -> CompiledPolicy {
        CompiledPolicy::new(
            "invite_only",
            vec![
                OpCode::PushConst(Value::String("invited".into())),
                OpCode::HasCredential,
                OpCode::Return,
            ],
        )
        .with_description("Invite only: Must have 'invited' credential")
    }

    /// Dynamic Trust: Trust basierend auf Realm-Parameter
    /// `min_trust` wird als erster Wert auf dem Stack erwartet
    pub fn dynamic_trust(min_trust: f64) -> CompiledPolicy {
        CompiledPolicy::new(
            format!("trust_min_{}", min_trust),
            vec![
                OpCode::LoadTrust,
                OpCode::TrustDim(TrustDimIndex::R),
                OpCode::PushConst(Value::Number(min_trust)),
                OpCode::Gte,
                OpCode::Return,
            ],
        )
        .with_description(format!("Dynamic trust: Trust.R >= {}", min_trust))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::realm_id_from_name;
    use crate::eclvm::runtime::host::StubHost;

    #[allow(dead_code)]
    fn setup_gateway() -> ProgrammableGateway<StubHost> {
        let host = Arc::new(StubHost::new());
        ProgrammableGateway::new(host)
    }

    #[test]
    fn test_default_entry_policy_allows_trusted() {
        let alice = DID::new_self(b"alice");
        let alice_uri = alice.to_uri();
        let host = Arc::new(StubHost::new().with_trust(&alice_uri, [0.8, 0.8, 0.8, 0.8, 0.8, 0.8]));
        let gateway = ProgrammableGateway::new(host);

        let alice_trust = TrustVector6D::new(0.8, 0.8, 0.8, 0.8, 0.8, 0.8);

        let decision = gateway
            .validate_entry(&alice, &alice_trust, &realm_id_from_name("realm:test"))
            .unwrap();

        assert!(decision.allowed);
    }

    #[test]
    fn test_default_entry_policy_denies_newcomer() {
        let bob = DID::new_self(b"bob");
        let bob_uri = bob.to_uri();
        let host = Arc::new(StubHost::new().with_trust(&bob_uri, [0.1, 0.1, 0.1, 0.1, 0.1, 0.1]));
        let gateway = ProgrammableGateway::new(host);

        let bob_trust = TrustVector6D::newcomer(); // 0.1

        let decision = gateway
            .validate_entry(&bob, &bob_trust, &realm_id_from_name("realm:test"))
            .unwrap();

        assert!(!decision.allowed);
    }

    #[test]
    fn test_custom_policy_high_trust() {
        let alice = DID::new_self(b"alice");
        let alice_uri = alice.to_uri();
        let host = Arc::new(StubHost::new().with_trust(&alice_uri, [0.8, 0.8, 0.8, 0.8, 0.8, 0.8]));
        let mut gateway = ProgrammableGateway::new(host);

        let finance = realm_id_from_name("realm:erynoa:finance");
        gateway.register_entry_policy(finance.clone(), StandardPolicies::high_trust());

        let alice_trust = TrustVector6D::new(0.8, 0.8, 0.8, 0.8, 0.8, 0.8);

        let decision = gateway
            .validate_entry(&alice, &alice_trust, &finance)
            .unwrap();
        assert!(decision.allowed);

        // Medium trust user should be denied
        let charlie = DID::new_self(b"charlie");
        let charlie_uri = charlie.to_uri();
        let charlie_trust = TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5);

        // Need to update host mock for charlie
        let host2 =
            Arc::new(StubHost::new().with_trust(&charlie_uri, [0.5, 0.5, 0.5, 0.5, 0.5, 0.5]));
        let mut gateway2 = ProgrammableGateway::new(host2);
        gateway2.register_entry_policy(finance.clone(), StandardPolicies::high_trust());

        let decision2 = gateway2
            .validate_entry(&charlie, &charlie_trust, &finance)
            .unwrap();
        assert!(!decision2.allowed);
    }

    #[test]
    fn test_verified_users_policy() {
        let alice = DID::new_self(b"alice");
        let alice_uri = alice.to_uri();
        let host = Arc::new(
            StubHost::new()
                .with_trust(&alice_uri, [0.8, 0.8, 0.8, 0.8, 0.8, 0.8])
                .with_credential(&alice_uri, "email-verified"),
        );
        let mut gateway = ProgrammableGateway::new(host);

        let verified = realm_id_from_name("realm:verified");
        gateway.register_entry_policy(verified.clone(), StandardPolicies::verified_users());

        let alice_trust = TrustVector6D::new(0.8, 0.8, 0.8, 0.8, 0.8, 0.8);

        let decision = gateway
            .validate_entry(&alice, &alice_trust, &verified)
            .unwrap();
        assert!(decision.allowed);
    }

    #[test]
    fn test_verified_users_denied_without_credential() {
        let host = Arc::new(
            StubHost::new().with_trust("did:erynoa:self:bob", [0.8, 0.8, 0.8, 0.8, 0.8, 0.8]), // Kein email-verified Credential!
        );
        let mut gateway = ProgrammableGateway::new(host);

        let verified = realm_id_from_name("realm:verified");
        gateway.register_entry_policy(verified.clone(), StandardPolicies::verified_users());

        let bob = DID::new_self(b"bob");
        let bob_trust = TrustVector6D::new(0.8, 0.8, 0.8, 0.8, 0.8, 0.8);

        let decision = gateway.validate_entry(&bob, &bob_trust, &verified).unwrap();
        assert!(!decision.allowed);
    }

    #[test]
    fn test_public_realm_allows_anyone() {
        let host = Arc::new(
            StubHost::new().with_trust("did:erynoa:self:newcomer", [0.1, 0.1, 0.1, 0.1, 0.1, 0.1]),
        );
        let mut gateway = ProgrammableGateway::new(host);

        let public = realm_id_from_name("realm:public");
        gateway.register_entry_policy(public.clone(), StandardPolicies::public_realm());

        let newcomer = DID::new_self(b"newcomer");
        let newcomer_trust = TrustVector6D::newcomer();

        let decision = gateway
            .validate_entry(&newcomer, &newcomer_trust, &public)
            .unwrap();
        assert!(decision.allowed);
    }
}
