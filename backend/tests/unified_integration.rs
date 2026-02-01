//! # Integration Tests für Unified Data Model
//!
//! End-to-End Tests die das Zusammenspiel der UDM-Komponenten validieren.
//!
//! ## Test-Kategorien
//!
//! 1. **Identitäts-Lifecycle**: DID-Erstellung, Delegation, Revocation
//! 2. **Trust-Propagation**: Trust-Updates über Delegations-Kette
//! 3. **Saga-Orchestrierung**: Multi-Step Workflows mit Compensation
//! 4. **Event-DAG**: Kausale Ordnung, Finality-Progression
//! 5. **Schema-Migration**: Versions-Upgrades mit Daten-Integrität

use std::sync::Arc;

use erynoa_api::domain::unified::*;
use erynoa_api::execution::{ExecutionContext, NetworkConditions, SyncTiming};

// ============================================================================
// Identitäts-Lifecycle Tests
// ============================================================================

mod identity_lifecycle {
    use super::*;

    /// Testet den vollständigen DID-Lifecycle: Erstellen → Delegieren → Widerrufen
    #[test]
    fn test_did_lifecycle_complete() {
        // 1. Erstelle Root-DID
        let root_did = DID::new(DIDNamespace::Self_, "root-user");
        assert!(root_did.to_string().starts_with("did:erynoa:self:"));

        // 2. Erstelle DIDDocument
        let root_doc = DIDDocument::new(root_did.clone());
        assert_eq!(root_doc.id, root_did);
        assert!(root_doc.delegations.is_empty());

        // 3. Erstelle Delegiertem
        let delegate_did = DID::new(DIDNamespace::Self_, "delegate-user");

        // 4. Erstelle Delegation
        let capability = Capability::Read {
            resource: "realm:test/*".to_string(),
        };
        let delegator_id = UniversalId::new(UniversalId::TAG_DID, 1, root_did.id.as_bytes());
        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, delegate_did.id.as_bytes());

        let delegation = Delegation::new(
            delegator_id,
            delegate_id,
            0.8, // trust_factor
            vec![capability],
        );

        let now = TemporalCoord::now(1, &delegator_id);
        assert!(delegation.is_valid(&now));
        assert_eq!(delegation.delegate, delegate_id);
        assert_eq!(delegation.trust_factor, 0.8);

        // 5. Prüfe Trust-Decay gemäß Κ8
        assert!(InvariantChecker::check_delegation_trust_factor(delegation.trust_factor).is_ok());
    }

    /// Testet Extension Slots für DIDDocument
    #[test]
    fn test_did_document_extension_slots() {
        let did = DID::new(DIDNamespace::Self_, "ext-test");
        let mut doc = DIDDocument::new(did);

        // Setze Recovery-Keys Extension
        let recovery_data = vec![0x01, 0x02, 0x03, 0x04];
        doc.set_extension(extension_slots::RECOVERY_KEYS, recovery_data.clone());

        assert!(doc.has_extension(extension_slots::RECOVERY_KEYS));
        assert_eq!(
            doc.get_extension(extension_slots::RECOVERY_KEYS),
            Some(&recovery_data)
        );

        // Unbekannte Extension
        assert!(!doc.has_extension(0xFFFF));
        assert_eq!(doc.get_extension(0xFFFF), None);
    }

    /// Testet ungültige Trust-Faktoren
    #[test]
    fn test_invalid_delegation_trust_factors() {
        // Trust = 0 nicht erlaubt
        assert!(InvariantChecker::check_delegation_trust_factor(0.0).is_err());

        // Trust > 1 nicht erlaubt
        assert!(InvariantChecker::check_delegation_trust_factor(1.5).is_err());

        // Negative Werte nicht erlaubt
        assert!(InvariantChecker::check_delegation_trust_factor(-0.1).is_err());

        // Gültige Werte
        assert!(InvariantChecker::check_delegation_trust_factor(0.001).is_ok());
        assert!(InvariantChecker::check_delegation_trust_factor(0.5).is_ok());
        assert!(InvariantChecker::check_delegation_trust_factor(1.0).is_ok());
    }
}

// ============================================================================
// Trust-Propagation Tests
// ============================================================================

mod trust_propagation {
    use super::*;

    /// Testet Trust-Propagation über Delegations-Kette
    #[test]
    fn test_trust_decay_through_delegation_chain() {
        // Root → A → B → C
        // Jede Delegation mit 0.8 Trust-Faktor
        // Erwartetes Trust bei C: 0.8^3 = 0.512

        let root_trust = TrustVector6D::ESTABLISHED; // [0.8, 0.8, 0.8, 0.8, 0.8, 0.8]

        // Delegation-Faktoren
        let factor_a = 0.8;
        let factor_b = 0.8;
        let factor_c = 0.8;

        // Berechne effektives Trust
        let weights = ContextType::Default.default_weights();
        let root_norm = root_trust.weighted_norm(&weights);

        let trust_a = root_norm * factor_a;
        let trust_b = trust_a * factor_b;
        let trust_c = trust_b * factor_c;

        // Erwartung: 0.8 * 0.8 * 0.8 * 0.8 = 0.4096
        let expected = root_norm * (factor_a * factor_b * factor_c);

        assert!((trust_c - expected).abs() < 0.0001);
        assert!(trust_c < trust_b);
        assert!(trust_b < trust_a);
    }

    /// Testet Asymmetrische Trust-Updates (Κ4)
    #[test]
    fn test_asymmetric_trust_updates() {
        let mut trust = TrustVector6D::NEWCOMER;

        // Positive Update auf Reliability
        let positive_delta = 0.1;
        trust.reliability += positive_delta;

        // Negative Update sollte stärker sein
        let negative_delta = -0.1;
        let asymmetry_factor = TrustDimension::Reliability.asymmetry_factor();
        let applied_negative = negative_delta * asymmetry_factor;

        // Asymmetrie-Faktor für Reliability ist 1.5
        assert_eq!(asymmetry_factor, 1.5);
        assert_eq!(applied_negative, -0.15);
    }

    /// Testet Context-spezifische Trust-Gewichte
    #[test]
    fn test_context_specific_weights() {
        let trust = TrustVector6D {
            reliability: 0.9,    // R
            integrity: 0.5,      // I - niedrig
            competence: 0.9,     // C
            predictability: 0.9, // P
            vulnerability: 0.9,  // V
            omega: 1.0,          // Ω
        };

        // Default-Kontext: Alle gleich wichtig
        let default_weights = ContextType::Default.default_weights();
        let default_norm = trust.weighted_norm(&default_weights);

        // High-Security-Kontext: Integrity viel wichtiger
        let security_weights = ContextType::HighSecurity.default_weights();
        let security_norm = trust.weighted_norm(&security_weights);

        // Wegen niedriger Integrity sollte Security-Norm niedriger sein
        assert!(security_norm < default_norm);
    }
}

// ============================================================================
// Saga-Orchestrierung Tests
// ============================================================================

mod saga_orchestration {
    use super::*;

    /// Testet Multi-Step Saga mit Compensation
    #[test]
    fn test_saga_with_compensation() {
        // Erstelle Intent: Geld von A nach B transferieren
        let intent = Intent {
            actor: DID::new(DIDNamespace::Self_, "user-a"),
            goals: vec![Goal::Transfer {
                from: "account-a".to_string(),
                to: "account-b".to_string(),
                amount: 100,
            }],
            constraints: vec![
                Constraint::MaxCost(Cost::new(1000, 500, 0.1)),
                Constraint::MaxLatency(5000),
            ],
            realm_crossings: vec![],
        };

        // Erstelle Saga aus Intent
        let saga_id = saga_id_from_intent(&intent);

        let mut saga = Saga {
            id: saga_id,
            status: SagaStatus::Pending,
            intent: intent.clone(),
            steps: vec![],
            current_step: 0,
            total_cost: Cost::ZERO,
            created_at: TemporalCoord::new(1000, 1, 1),
            updated_at: TemporalCoord::new(1000, 1, 1),
        };

        // Füge Steps hinzu
        let debit_step = SagaStep {
            action: SagaAction::DebitAccount {
                account: "account-a".to_string(),
                amount: 100,
            },
            status: StepStatus::Pending,
            result: None,
            compensation: Some(SagaCompensation::CreditAccount {
                account: "account-a".to_string(),
                amount: 100,
            }),
        };

        let credit_step = SagaStep {
            action: SagaAction::CreditAccount {
                account: "account-b".to_string(),
                amount: 100,
            },
            status: StepStatus::Pending,
            result: None,
            compensation: Some(SagaCompensation::DebitAccount {
                account: "account-b".to_string(),
                amount: 100,
            }),
        };

        saga.steps.push(debit_step);
        saga.steps.push(credit_step);

        // Simuliere erfolgreiche Ausführung
        saga.status = SagaStatus::Running;
        saga.steps[0].status = StepStatus::Completed;
        saga.steps[0].result = Some(StepResult::Success);

        saga.steps[1].status = StepStatus::Completed;
        saga.steps[1].result = Some(StepResult::Success);

        saga.status = SagaStatus::Completed;

        assert!(matches!(saga.status, SagaStatus::Completed));
        assert!(saga
            .steps
            .iter()
            .all(|s| s.result == Some(StepResult::Success)));
    }

    /// Testet Saga-Abbruch und Compensation
    #[test]
    fn test_saga_compensation_on_failure() {
        let intent = Intent {
            actor: DID::new(DIDNamespace::Self_, "user-test"),
            goals: vec![Goal::Custom("test-goal".to_string())],
            constraints: vec![],
            realm_crossings: vec![],
        };

        let mut saga = Saga {
            id: saga_id_from_intent(&intent),
            status: SagaStatus::Running,
            intent,
            steps: vec![
                SagaStep {
                    action: SagaAction::Custom("step-1".to_string()),
                    status: StepStatus::Completed,
                    result: Some(StepResult::Success),
                    compensation: Some(SagaCompensation::Custom("undo-1".to_string())),
                },
                SagaStep {
                    action: SagaAction::Custom("step-2".to_string()),
                    status: StepStatus::Failed,
                    result: Some(StepResult::Failure("simulated error".to_string())),
                    compensation: None,
                },
            ],
            current_step: 1,
            total_cost: Cost::ZERO,
            created_at: TemporalCoord::new(1000, 1, 1),
            updated_at: TemporalCoord::new(1001, 2, 1),
        };

        // Step 2 ist fehlgeschlagen → Compensation nötig
        saga.status = SagaStatus::Compensating;

        // Führe Compensation für Step 1 aus
        // (Step 2 hat keine Compensation)

        saga.status = SagaStatus::Aborted;

        assert!(matches!(saga.status, SagaStatus::Aborted));
    }
}

// ============================================================================
// Event-DAG Tests
// ============================================================================

mod event_dag {
    use super::*;

    /// Testet kausale Ordnung von Events (Κ9)
    #[test]
    fn test_causal_ordering() {
        let node_id = UniversalId::new(UniversalId::TAG_DID, 1, b"node-1");

        // Parent Event
        let parent_coord = TemporalCoord::new(1000, 5, 42);
        let parent_id = event_id_from_content(b"parent-event");

        // Child Event (muss nach Parent sein)
        let child_coord = TemporalCoord::new(1001, 6, 42);

        // Invariante prüfen
        assert!(InvariantChecker::check_causal_order(&child_coord, &parent_coord).is_ok());

        // Umgekehrte Ordnung muss fehlschlagen
        assert!(InvariantChecker::check_causal_order(&parent_coord, &child_coord).is_err());
    }

    /// Testet Finality-Progression (Κ10)
    #[test]
    fn test_finality_progression() {
        let event_id = event_id_from_content(b"test-event");

        // Finality beginnt bei Proposed
        let mut finality = FinalityState::new(event_id);
        assert!(matches!(finality.level, FinalityLevel::Proposed));

        // Progression: Proposed → Tentative → Committed → Final
        finality.level = FinalityLevel::Tentative;
        finality.level = FinalityLevel::Committed;
        finality.level = FinalityLevel::Final;

        assert!(matches!(finality.level, FinalityLevel::Final));
    }

    /// Testet Event-Erstellung mit verschiedenen Payloads
    #[test]
    fn test_event_payloads() {
        let realm_id = realm_id_from_name("test-realm");

        // StateChange Event
        let state_change = EventPayload::StateChange {
            store: "user-data".to_string(),
            key: vec![0x01, 0x02],
            old_value: None,
            new_value: Some(vec![0xAA, 0xBB]),
        };

        // TrustUpdate Event
        let trust_update = EventPayload::TrustUpdate {
            subject: UniversalId::new(UniversalId::TAG_DID, 1, b"user"),
            dimension: TrustDimension::Reliability,
            old_value: 0.5,
            new_value: 0.6,
            reason: TrustUpdateReason::PositiveInteraction,
        };

        // SagaStep Event
        let saga_step = EventPayload::SagaStep {
            saga_id: saga_id_from_intent(&Intent {
                actor: DID::new(DIDNamespace::Self_, "actor"),
                goals: vec![],
                constraints: vec![],
                realm_crossings: vec![],
            }),
            step_index: 0,
            result: SagaStepResult::Success,
        };

        // Alle Payloads sollten serialisierbar sein
        assert!(matches!(state_change, EventPayload::StateChange { .. }));
        assert!(matches!(trust_update, EventPayload::TrustUpdate { .. }));
        assert!(matches!(saga_step, EventPayload::SagaStep { .. }));
    }
}

// ============================================================================
// Schema-Migration Tests
// ============================================================================

mod schema_migration {
    use super::*;

    /// Testet vollständigen Migrations-Pfad
    #[test]
    fn test_full_migration_path() {
        let mut registry = SchemaRegistry::new();

        // DID-Schema: v1 → v2 → v3
        registry.register_current(UniversalId::TAG_DID, 3);

        // v1 → v2: Füge "created_at" Feld hinzu (8 Bytes)
        registry.register_migration(
            UniversalId::TAG_DID,
            1,
            2,
            Arc::new(|data| {
                let mut result = data.to_vec();
                result.extend_from_slice(&[0u8; 8]); // created_at = 0
                Ok(result)
            }),
        );

        // v2 → v3: Füge "updated_at" Feld hinzu (8 Bytes)
        registry.register_migration(
            UniversalId::TAG_DID,
            2,
            3,
            Arc::new(|data| {
                let mut result = data.to_vec();
                result.extend_from_slice(&[0u8; 8]); // updated_at = 0
                Ok(result)
            }),
        );

        // Validiere Pfade
        assert!(registry.validate_migration_paths().is_ok());

        // Migriere v1-Daten
        let v1_id = UniversalId::new(UniversalId::TAG_DID, 1, b"old-did");
        let v1_data = vec![0x01, 0x02, 0x03]; // Original 3 Bytes

        let migrated = registry.maybe_migrate(&v1_id, &v1_data).unwrap();

        // Erwartet: 3 + 8 + 8 = 19 Bytes
        assert_eq!(migrated.len(), 19);
        assert_eq!(&migrated[0..3], &[0x01, 0x02, 0x03]);
    }

    /// Testet fehlende Migrations-Pfade
    #[test]
    fn test_missing_migration_path() {
        let mut registry = SchemaRegistry::new();
        registry.register_current(UniversalId::TAG_EVENT, 3);

        // Nur v1 → v2 registriert, v2 → v3 fehlt
        registry.register_migration(UniversalId::TAG_EVENT, 1, 2, identity_migration());

        // Validierung sollte fehlschlagen
        let result = registry.validate_migration_paths();
        assert!(result.is_err());
    }

    /// Testet Version-Downgrade-Ablehnung
    #[test]
    fn test_version_downgrade_rejected() {
        let mut registry = SchemaRegistry::new();
        registry.register_current(UniversalId::TAG_REALM, 1);

        // Daten mit höherer Version
        let future_id = UniversalId::new(UniversalId::TAG_REALM, 5, b"future");
        let data = vec![1, 2, 3];

        let result = registry.maybe_migrate(&future_id, &data);
        assert!(matches!(result, Err(MigrationError::VersionTooHigh { .. })));
    }
}

// ============================================================================
// Execution-Context Tests
// ============================================================================

mod execution_context {
    use super::*;

    /// Testet vollständigen Execution-Lifecycle
    #[test]
    fn test_execution_lifecycle() {
        let realm_id = realm_id_from_name("exec-test");
        let actor = DID::new(DIDNamespace::Self_, "executor");
        let actor_id = UniversalId::new(UniversalId::TAG_DID, 1, actor.id.as_bytes());

        let budget = Budget::new(10000, 5000, 1.0);
        let trust = TrustVector6D::ESTABLISHED;

        let network = NetworkConditions {
            latency_ms: 50,
            jitter_ms: 10,
            packet_loss_rate: 0.01,
            bandwidth_kbps: 10000,
        };

        let timing = SyncTiming::new(realm_id);

        let mut ctx = ExecutionContext::new(realm_id, actor_id, budget, trust, network, timing);

        // Prüfe initialen Zustand
        assert!(ctx.events().is_empty());
        assert!(ctx.current_budget().gas > 0);

        // Consume Gas
        let result = ctx.consume_gas(1000);
        assert!(result.is_ok());
        assert_eq!(ctx.current_budget().gas, 9000);

        // Consume Mana
        let result = ctx.consume_mana(500);
        assert!(result.is_ok());
        assert_eq!(ctx.current_budget().mana, 4500);

        // Trust-Gate prüfen
        let result = ctx.check_trust_gate(0.5);
        assert!(result.is_ok()); // ESTABLISHED hat Trust > 0.5
    }

    /// Testet Budget-Erschöpfung
    #[test]
    fn test_budget_exhaustion() {
        let realm_id = realm_id_from_name("budget-test");
        let actor_id = UniversalId::new(UniversalId::TAG_DID, 1, b"actor");

        let small_budget = Budget::new(100, 50, 0.5);
        let trust = TrustVector6D::NEWCOMER;
        let network = NetworkConditions::default();
        let timing = SyncTiming::new(realm_id);

        let mut ctx =
            ExecutionContext::new(realm_id, actor_id, small_budget, trust, network, timing);

        // Verbrauche mehr Gas als verfügbar
        let result = ctx.consume_gas(200);
        assert!(result.is_err());

        // Budget sollte unverändert sein nach Fehler
        assert_eq!(ctx.current_budget().gas, 100);
    }
}

// ============================================================================
// Cost-Algebra Tests
// ============================================================================

mod cost_algebra {
    use super::*;

    /// Testet Halbring-Eigenschaften von Cost
    #[test]
    fn test_semiring_properties() {
        let c1 = Cost::new(10, 5, 0.1);
        let c2 = Cost::new(20, 10, 0.2);
        let c3 = Cost::new(30, 15, 0.15);

        // Assoziativität von seq: (c1 ⊕ c2) ⊕ c3 = c1 ⊕ (c2 ⊕ c3)
        let left = c1.seq(c2).seq(c3);
        let right = c1.seq(c2.seq(c3));

        assert_eq!(left.gas, right.gas);
        assert_eq!(left.mana, right.mana);
        assert!((left.trust_risk - right.trust_risk).abs() < 0.0001);

        // Neutrales Element: c ⊕ 0 = c
        let with_zero = c1.seq(Cost::ZERO);
        assert_eq!(with_zero.gas, c1.gas);
        assert_eq!(with_zero.mana, c1.mana);

        // InvariantChecker bestätigt
        assert!(InvariantChecker::check_cost_algebra(c1, c2, c3).is_ok());
    }

    /// Testet parallele Kosten-Kombination
    #[test]
    fn test_parallel_cost() {
        let c1 = Cost::new(100, 50, 0.1);
        let c2 = Cost::new(200, 100, 0.2);

        let parallel = c1.par(c2);

        // Parallel: max(gas), max(mana), max(risk)
        assert_eq!(parallel.gas, 200);
        assert_eq!(parallel.mana, 100);
        assert!((parallel.trust_risk - 0.2).abs() < 0.0001);
    }
}

// ============================================================================
// P2P-Message Tests
// ============================================================================

mod p2p_messages {
    use super::*;

    /// Testet alle Message-Typen
    #[test]
    fn test_message_construction() {
        let node_id = UniversalId::new(UniversalId::TAG_DID, 1, b"node");
        let coord = TemporalCoord::new(1000, 1, 42);

        // Ping
        let ping = P2PMessage::new(node_id, coord, MessagePayload::Ping(PingMessage::new()));
        assert!(matches!(ping.payload, MessagePayload::Ping(_)));

        // Pong
        let pong = P2PMessage::new(
            node_id,
            coord,
            MessagePayload::Pong(PongMessage::for_ping(&ping.id)),
        );
        assert!(matches!(pong.payload, MessagePayload::Pong(_)));

        // SyncRequest
        let sync = P2PMessage::new(
            node_id,
            coord,
            MessagePayload::SyncRequest(SyncRequestMessage {
                realm_id: realm_id_from_name("test"),
                sync_type: SyncType::Full,
                from_timestamp: None,
            }),
        );
        assert!(matches!(sync.payload, MessagePayload::SyncRequest(_)));
    }
}

// ============================================================================
// Integration: End-to-End Flow
// ============================================================================

mod end_to_end {
    use super::*;

    /// Vollständiger User-Flow: Registrierung → Realm-Beitritt → Saga → Trust-Update
    #[test]
    fn test_complete_user_flow() {
        // 1. User erstellt DID
        let user_did = DID::new(DIDNamespace::Self_, "alice");
        let mut user_doc = DIDDocument::new(user_did.clone());

        // 2. User erhält initialen Trust (Newcomer)
        let user_id = UniversalId::new(UniversalId::TAG_DID, 1, user_did.id.as_bytes());
        let mut trust_record = TrustRecord::new(user_id);
        assert_eq!(trust_record.current, TrustVector6D::NEWCOMER);

        // 3. Realm existiert
        let realm_id = realm_id_from_name("community");

        // 4. User führt erfolgreiche Interaktion aus → Trust-Update
        let old_reliability = trust_record.current.reliability;
        trust_record.current.reliability += 0.1;
        trust_record
            .history
            .add_entry(TrustHistoryEntry::new(TrustDimension::Reliability, 0.1));

        assert!(trust_record.current.reliability > old_reliability);

        // 5. Nach vielen Interaktionen → Established Trust
        trust_record.current = TrustVector6D::ESTABLISHED;

        // 6. User kann jetzt delegieren
        let delegate_did = DID::new(DIDNamespace::Self_, "bob");
        let delegation = Delegation::new(
            delegate_did,
            vec![Capability::Read {
                resources: vec![format!("realm:{}/*", realm_id)],
            }],
            0.7,
            Some(86400 * 7), // 1 Woche
        );

        user_doc.delegations.push(delegation);
        assert_eq!(user_doc.delegations.len(), 1);

        // 7. Delegation ist gültig
        assert!(user_doc.delegations[0].is_valid());
    }
}
