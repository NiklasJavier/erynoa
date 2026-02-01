//! # Saga Composer
//!
//! Komponiert Sagas aus Intents gemäß Κ22 und Κ24.
//!
//! ## Axiom-Referenz
//!
//! - **Κ22 (Saga-Composer)**: `∀ Intent I : ∃! Saga S : resolve(I) = S`
//! - **Κ24 (Atomare Kompensation)**: `fail(Sᵢ) → compensate(S₁..Sᵢ₋₁)`
//!
//! ## Komposition
//!
//! ```text
//! Intent(Transfer 100 ERY to Bob)
//!         │
//!         ▼
//! ┌───────────────────────────────────────┐
//! │ Step 1: Lock(Alice, 100)              │
//! │   ↳ Compensation: Unlock(lock_id)     │
//! ├───────────────────────────────────────┤
//! │ Step 2: Transfer(Alice → Bob, 100)    │
//! │   ↳ Compensation: ReverseTransfer     │
//! └───────────────────────────────────────┘
//! ```

use crate::domain::{
    Constraint, Goal, Intent, RealmId, Saga, SagaAction, SagaCompensation, SagaStep, UniversalId,
    DID,
};
use thiserror::Error;

/// Fehler bei Saga-Komposition
#[derive(Debug, Error)]
pub enum CompositionError {
    #[error("Cannot compose saga for goal: {0}")]
    UnsupportedGoal(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Insufficient budget: need {need}, have {have}")]
    InsufficientBudget { need: u64, have: u64 },

    #[error("Cross-realm operation requires gateway check")]
    MissingGatewayCheck,
}

/// Ergebnis von Composition-Operationen
pub type CompositionResult<T> = Result<T, CompositionError>;

/// Saga Composer (Κ22, Κ24)
///
/// ```text
/// ┌──────────────────────────────────────────────────────────────┐
/// │                    SagaComposer                              │
/// │                                                              │
/// │    Intent                                                    │
/// │        │                                                     │
/// │        ▼                                                     │
/// │    ┌─────────────┐                                          │
/// │    │  Analyze    │  Goal, Constraints, Budget               │
/// │    └──────┬──────┘                                          │
/// │           │                                                  │
/// │           ▼                                                  │
/// │    ┌─────────────┐                                          │
/// │    │  Plan Steps │  Κ22: resolve(I) = S                     │
/// │    └──────┬──────┘                                          │
/// │           │                                                  │
/// │           ▼                                                  │
/// │    ┌─────────────┐                                          │
/// │    │  Add Comp.  │  Κ24: Compensations                      │
/// │    └──────┬──────┘                                          │
/// │           │                                                  │
/// │           ▼                                                  │
/// │        Saga                                                  │
/// └──────────────────────────────────────────────────────────────┘
/// ```
pub struct SagaComposer {
    /// Konfiguration
    config: SagaComposerConfig,
}

/// Konfiguration für SagaComposer
#[derive(Debug, Clone)]
pub struct SagaComposerConfig {
    /// Standard-Lock-Dauer in Sekunden
    pub default_lock_duration: u64,

    /// Aktiviere automatische Kompensation
    pub auto_compensation: bool,

    /// Maximum Schritte pro Saga
    pub max_steps: usize,
}

impl Default for SagaComposerConfig {
    fn default() -> Self {
        Self {
            default_lock_duration: 3600, // 1 Stunde
            auto_compensation: true,
            max_steps: 20,
        }
    }
}

impl SagaComposer {
    /// Erstelle neuen SagaComposer
    pub fn new(config: SagaComposerConfig) -> Self {
        Self { config }
    }

    /// Erstelle mit Default-Config
    pub fn default() -> Self {
        Self::new(SagaComposerConfig::default())
    }

    /// Κ22: Komponiere Saga aus Intent
    pub fn compose(&self, intent: &Intent) -> CompositionResult<Saga> {
        let steps = match &intent.goal {
            Goal::Transfer {
                to,
                amount,
                asset_type,
            } => self.compose_transfer(intent.source_did(), to, *amount, asset_type)?,
            Goal::Attest { subject, claim } => {
                self.compose_attest(intent.source_did(), subject, claim)?
            }
            Goal::Delegate {
                to,
                capabilities,
                ttl_seconds,
                ..  // trust_factor nicht hier verwendet
            } => self.compose_delegate(intent.source_did(), to, capabilities, *ttl_seconds)?,
            Goal::Query { predicate } => self.compose_query(intent.source_did(), predicate)?,
            Goal::Create {
                entity_type,
                params,
            } => self.compose_create(intent.source_did(), entity_type, params)?,
            Goal::Complex {
                description,
                sub_goals,
            } => self.compose_complex(intent.source_did(), description, sub_goals)?,
        };

        // Prüfe Constraints
        self.validate_constraints(&steps, &intent.constraints)?;

        Ok(Saga::from_intent(intent, steps, 0))
    }

    /// Komponiere Transfer-Saga
    fn compose_transfer(
        &self,
        from: &UniversalId,
        to: &UniversalId,
        amount: u64,
        asset_type: &str,
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();

        // Step 1: Lock Funds
        let lock_step = SagaStep::new(
            0,
            format!("Lock {} {} from {}", amount, asset_type, from.to_hex()),
            SagaAction::Lock {
                owner: from.clone(),
                amount,
                asset_type: asset_type.to_string(),
                lock_id: None,
                release_conditions: vec![],
            },
        )
        .with_compensation(SagaCompensation::new(
            "Unlock funds",
            SagaAction::Unlock {
                lock_id: UniversalId::NULL, // Placeholder - wird bei Ausführung ersetzt
                to: None,
            },
        ));
        steps.push(lock_step);

        // Step 2: Execute Transfer
        let transfer_step = SagaStep::new(
            1,
            format!("Transfer {} {} to {}", amount, asset_type, to.to_hex()),
            SagaAction::Transfer {
                from: from.clone(),
                to: to.clone(),
                amount,
                asset_type: asset_type.to_string(),
            },
        )
        .with_dependencies(vec![0]);
        steps.push(transfer_step);

        Ok(steps)
    }

    /// Komponiere Attestation-Saga
    fn compose_attest(
        &self,
        attester: &UniversalId,
        subject: &UniversalId,
        claim: &str,
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();

        // Step 1: Validiere Attester-Berechtigung
        let validate_step = SagaStep::new(
            0,
            format!("Validate attester {} credentials", attester.to_hex()),
            SagaAction::WaitFor {
                timeout_lamport: 0,
                condition: format!("trust({}) >= 0.5", attester.to_hex()),
                timeout_seconds: 30,
            },
        );
        steps.push(validate_step);

        // Step 2: Erstelle Attestation-Event
        // (In der echten Implementierung würde hier ein Event erstellt)
        let attest_step = SagaStep::new(
            1,
            format!(
                "Create attestation for {} on subject {}",
                claim,
                subject.to_hex()
            ),
            SagaAction::WaitFor {
                timeout_lamport: 0,
                condition: format!("attestation({}, {})", attester.to_hex(), subject.to_hex()),
                timeout_seconds: 60,
            },
        )
        .with_dependencies(vec![0]);
        steps.push(attest_step);

        Ok(steps)
    }

    /// Komponiere Delegation-Saga
    fn compose_delegate(
        &self,
        from: &UniversalId,
        to: &UniversalId,
        capabilities: &[String],
        ttl_seconds: u64,
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();

        // Step 1: Validiere Delegator hat diese Capabilities
        let validate_step = SagaStep::new(
            0,
            format!(
                "Validate {} has capabilities {:?}",
                from.to_hex(),
                capabilities
            ),
            SagaAction::WaitFor {
                timeout_lamport: 0,
                condition: format!(
                    "capabilities({}) includes {:?}",
                    from.to_hex(),
                    capabilities
                ),
                timeout_seconds: 30,
            },
        );
        steps.push(validate_step);

        // Step 2: Erstelle Delegation-Event
        let delegate_step = SagaStep::new(
            1,
            format!(
                "Delegate {:?} from {} to {} for {}s",
                capabilities,
                from.to_hex(),
                to.to_hex(),
                ttl_seconds
            ),
            SagaAction::WaitFor {
                timeout_lamport: 0,
                condition: format!("delegation({}, {})", from.to_hex(), to.to_hex()),
                timeout_seconds: 60,
            },
        )
        .with_dependencies(vec![0])
        .with_compensation(SagaCompensation::new(
            "Revoke delegation",
            SagaAction::WaitFor {
                timeout_lamport: 0,
                condition: format!("revoke_delegation({}, {})", from.to_hex(), to.to_hex()),
                timeout_seconds: 30,
            },
        ));
        steps.push(delegate_step);

        Ok(steps)
    }

    /// Komponiere Query-Saga
    fn compose_query(
        &self,
        _querier: &UniversalId,
        predicate: &str,
    ) -> CompositionResult<Vec<SagaStep>> {
        let step = SagaStep::new(
            0,
            format!("Execute query: {}", predicate),
            SagaAction::WaitFor {
                timeout_lamport: 0,
                condition: format!("query({})", predicate),
                timeout_seconds: 300,
            },
        );

        Ok(vec![step])
    }

    /// Komponiere Create-Saga
    fn compose_create(
        &self,
        creator: &UniversalId,
        entity_type: &str,
        params: &std::collections::HashMap<String, serde_json::Value>,
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut steps = Vec::new();

        // Je nach Entity-Typ unterschiedliche Schritte
        match entity_type {
            "asset" | "token" => {
                let amount = params.get("amount").and_then(|v| v.as_u64()).unwrap_or(0);

                steps.push(
                    SagaStep::new(
                        0,
                        format!("Mint {} {} for {}", amount, entity_type, creator.to_hex()),
                        SagaAction::Mint {
                            to: creator.clone(),
                            amount,
                            asset_type: entity_type.to_string(),
                            authorization: None,
                        },
                    )
                    .with_compensation(SagaCompensation::new(
                        "Burn minted assets",
                        SagaAction::Burn {
                            from: creator.clone(),
                            amount,
                            asset_type: entity_type.to_string(),
                            authorization: None,
                        },
                    )),
                );
            }
            _ => {
                steps.push(SagaStep::new(
                    0,
                    format!("Create {} for {}", entity_type, creator.to_hex()),
                    SagaAction::WaitFor {
                        timeout_lamport: 0,
                        condition: format!("create({}, {})", entity_type, creator.to_hex()),
                        timeout_seconds: 60,
                    },
                ));
            }
        }

        Ok(steps)
    }

    /// Komponiere Complex-Saga (mehrere Ziele)
    fn compose_complex(
        &self,
        source: &UniversalId,
        _description: &str,
        parsed_goals: &[Goal],
    ) -> CompositionResult<Vec<SagaStep>> {
        let mut all_steps = Vec::new();
        let mut step_index = 0;

        for goal in parsed_goals {
            let goal_steps = match goal {
                Goal::Transfer {
                    to,
                    amount,
                    asset_type,
                } => self.compose_transfer(source, to, *amount, asset_type)?,
                Goal::Attest { subject, claim } => self.compose_attest(source, subject, claim)?,
                _ => {
                    // Fallback
                    vec![SagaStep::new(
                        step_index,
                        format!("Process goal: {:?}", goal),
                        SagaAction::WaitFor {
                            timeout_lamport: 0,
                            condition: "goal_processed".to_string(),
                            timeout_seconds: 60,
                        },
                    )]
                }
            };

            // Re-Index und Dependencies anpassen
            let previous_last = if all_steps.is_empty() {
                None
            } else {
                Some(step_index - 1)
            };

            for mut step in goal_steps {
                step.index = step_index;
                if let Some(prev) = previous_last {
                    if step.dependencies.is_empty() {
                        step.dependencies = vec![prev];
                    }
                }
                all_steps.push(step);
                step_index += 1;
            }
        }

        Ok(all_steps)
    }

    /// Validiere Constraints gegen geplante Steps
    fn validate_constraints(
        &self,
        steps: &[SagaStep],
        constraints: &[Constraint],
    ) -> CompositionResult<()> {
        for constraint in constraints {
            match constraint {
                Constraint::MinTrust { value } => {
                    // In echter Implementierung: prüfe Trust aller Counterparts
                    if *value > 1.0 {
                        return Err(CompositionError::ConstraintViolation(format!(
                            "Invalid MinTrust: {}",
                            value
                        )));
                    }
                }
                Constraint::MaxCost { amount, cost, .. } => {
                    // Berechne geschätzte Kosten
                    let estimated_cost = steps.len() as u64 * 10; // Vereinfacht: 10 pro Step
                    let max_amount = amount.unwrap_or(cost.gas);
                    if estimated_cost > max_amount {
                        return Err(CompositionError::InsufficientBudget {
                            need: estimated_cost,
                            have: max_amount,
                        });
                    }
                }
                _ => {}
            }
        }

        // Prüfe max Steps
        if steps.len() > self.config.max_steps {
            return Err(CompositionError::ConstraintViolation(format!(
                "Too many steps: {} > {}",
                steps.len(),
                self.config.max_steps
            )));
        }

        Ok(())
    }

    /// Füge Realm-Crossing-Schritte hinzu (Κ23)
    pub fn add_realm_crossing(
        &self,
        mut saga: Saga,
        from_realm: RealmId,
        to_realm: RealmId,
        did: &DID,
    ) -> Saga {
        // Füge Gateway-Check als ersten Schritt ein
        let gateway_step = SagaStep::new(
            0,
            format!(
                "Gateway check for {} crossing {} → {}",
                did.to_uri(),
                from_realm.to_hex(),
                to_realm.to_hex()
            ),
            SagaAction::GatewayCheck {
                subject: did.id.clone(),
                target_realm: to_realm.clone(),
                required_trust: 0.0, // Default - kein spezifischer Trust benötigt
            },
        )
        .with_realm_crossing(from_realm, to_realm);

        // Re-Index existierende Steps
        for step in &mut saga.steps {
            step.index += 1;
            step.dependencies = step.dependencies.iter().map(|d| d + 1).collect();
        }

        // Füge Gateway-Step vorne ein
        saga.steps.insert(0, gateway_step);

        // Erste echte Step hängt von Gateway ab
        if saga.steps.len() > 1 {
            saga.steps[1].dependencies.push(0);
        }

        saga
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SagaStatus;

    #[test]
    fn test_compose_transfer() {
        let composer = SagaComposer::default();
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        let intent = Intent::new(
            alice.clone(),
            Goal::Transfer {
                to: bob.clone(),
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            RealmId::root(),
        );

        let saga = composer.compose(&intent).unwrap();

        assert_eq!(saga.steps.len(), 2);
        assert_eq!(saga.status, SagaStatus::Pending);

        // Erster Schritt: Lock
        match &saga.steps[0].action {
            SagaAction::Lock { did, amount, .. } => {
                assert_eq!(did, &alice);
                assert_eq!(*amount, 100);
            }
            _ => panic!("Expected Lock action"),
        }

        // Hat Compensation
        assert!(saga.steps[0].compensation.is_some());
    }

    #[test]
    fn test_compose_delegation() {
        let composer = SagaComposer::default();
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        let intent = Intent::new(
            alice.clone(),
            Goal::Delegate {
                to: bob.clone(),
                capabilities: vec!["transfer".to_string()],
                ttl_seconds: 86400,
            },
            RealmId::root(),
        );

        let saga = composer.compose(&intent).unwrap();

        assert_eq!(saga.steps.len(), 2);
    }

    #[test]
    fn test_max_cost_constraint() {
        let composer = SagaComposer::default();
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        let intent = Intent::new(
            alice.clone(),
            Goal::Transfer {
                to: bob,
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            RealmId::root(),
        )
        .with_constraint(Constraint::MaxCost {
            cost: Cost::default(),
            amount: Some(1), // Zu niedrig
            asset_type: Some("ERY".to_string()),
        });

        let result = composer.compose(&intent);
        assert!(matches!(
            result,
            Err(CompositionError::InsufficientBudget { .. })
        ));
    }

    #[test]
    fn test_add_realm_crossing() {
        use crate::domain::realm_id_from_name;
        let composer = SagaComposer::default();
        let alice = DID::new_self(b"alice");
        let bob = DID::new_self(b"bob");

        let intent = Intent::new(
            alice.clone(),
            Goal::Transfer {
                to: bob,
                amount: 100,
                asset_type: "ERY".to_string(),
            },
            RealmId::root(),
        );

        let saga = composer.compose(&intent).unwrap();
        let saga_with_crossing = composer.add_realm_crossing(
            saga,
            realm_id_from_name("realm:erynoa:gaming"),
            realm_id_from_name("realm:erynoa:finance"),
            &alice,
        );

        // Jetzt 3 Steps: Gateway + Lock + Transfer
        assert_eq!(saga_with_crossing.steps.len(), 3);

        // Erster Step ist Gateway
        match &saga_with_crossing.steps[0].action {
            SagaAction::GatewayCheck { .. } => {}
            _ => panic!("Expected GatewayCheck action"),
        }
    }
}
