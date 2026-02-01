//! # Integration Tests für Unified Data Model
//!
//! Fokussierte Tests für Schema-Registry und Kern-Invarianten.

use std::sync::Arc;

use erynoa_api::domain::unified::*;

// ============================================================================
// Identitäts-Lifecycle Tests
// ============================================================================

mod identity_lifecycle {
    use super::*;

    /// Testet DID-Erstellung und Delegation
    #[test]
    fn test_did_lifecycle_complete() {
        // Erstelle Root-DID
        let root_did = DID::new(DIDNamespace::Self_, b"root-user-key");
        assert!(root_did.to_string().starts_with("did:erynoa:self:"));

        // Erstelle DIDDocument
        let root_doc = DIDDocument::new(root_did.clone());
        assert_eq!(root_doc.id, root_did);
        assert!(root_doc.delegations.is_empty());

        // Erstelle Delegation
        let delegate_did = DID::new(DIDNamespace::Self_, b"delegate-key");
        let delegator_id = UniversalId::new(UniversalId::TAG_DID, 1, root_did.id.as_bytes());
        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, delegate_did.id.as_bytes());

        let delegation = Delegation::new(
            delegator_id,
            delegate_id,
            0.8,
            vec![Capability::Read {
                resource: "realm:test/*".to_string(),
            }],
        );

        let now = TemporalCoord::now(1, &delegator_id);
        assert!(delegation.is_valid(&now));
        assert_eq!(delegation.trust_factor, 0.8);

        // Prüfe Trust-Decay gemäß Κ8
        assert!(InvariantChecker::check_delegation_trust_factor(delegation.trust_factor).is_ok());
    }

    /// Testet Extension Slots für DIDDocument
    #[test]
    fn test_did_document_extension_slots() {
        let did = DID::new(DIDNamespace::Self_, b"ext-test-key");
        let mut doc = DIDDocument::new(did);

        let recovery_data = vec![0x01, 0x02, 0x03, 0x04];
        doc.set_extension(extension_slots::RECOVERY_KEYS, recovery_data.clone());

        assert!(doc.has_extension(extension_slots::RECOVERY_KEYS));
        assert_eq!(
            doc.get_extension(extension_slots::RECOVERY_KEYS),
            Some(&recovery_data)
        );
        assert!(!doc.has_extension(0xFFFF));
    }

    /// Testet ungültige Trust-Faktoren
    #[test]
    fn test_invalid_delegation_trust_factors() {
        assert!(InvariantChecker::check_delegation_trust_factor(0.0).is_err());
        assert!(InvariantChecker::check_delegation_trust_factor(1.5).is_err());
        assert!(InvariantChecker::check_delegation_trust_factor(-0.1).is_err());
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
        let root_trust = TrustVector6D::DEFAULT;
        let factor_a = 0.8;
        let factor_b = 0.8;
        let factor_c = 0.8;

        let weights = ContextType::Default.default_weights();
        let root_norm = root_trust.weighted_norm(&weights);

        let trust_a = root_norm * factor_a;
        let trust_b = trust_a * factor_b;
        let trust_c = trust_b * factor_c;

        let expected = root_norm * (factor_a * factor_b * factor_c);
        assert!((trust_c - expected).abs() < 0.0001);
        assert!(trust_c < trust_b);
        assert!(trust_b < trust_a);
    }

    /// Testet Asymmetrische Trust-Updates (Κ4)
    #[test]
    fn test_asymmetric_trust_updates() {
        let asymmetry_factor = TrustDimension::Reliability.asymmetry_factor();
        assert_eq!(asymmetry_factor, 1.5);

        let negative_delta = -0.1;
        let applied_negative = negative_delta * asymmetry_factor;
        assert_eq!(applied_negative, -0.15);
    }

    /// Testet Context-spezifische Trust-Gewichte
    #[test]
    fn test_context_specific_weights() {
        let trust = TrustVector6D::new(
            0.9, // R
            0.5, // I - niedrig
            0.9, // C
            0.9, // P
            0.9, // V
            1.0, // Ω
        );

        let default_weights = ContextType::Default.default_weights();
        let finance_weights = ContextType::Finance.default_weights();

        let default_norm = trust.weighted_norm(&default_weights);
        let finance_norm = trust.weighted_norm(&finance_weights);

        // Verschiedene Kontexte sollten verschiedene Normen geben
        assert!(default_norm > 0.0);
        assert!(finance_norm > 0.0);
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
        registry.register_current(UniversalId::TAG_DID, 3);

        registry.register_migration(
            UniversalId::TAG_DID,
            1,
            2,
            Arc::new(|data| {
                let mut result = data.to_vec();
                result.extend_from_slice(&[0u8; 8]);
                Ok(result)
            }),
        );

        registry.register_migration(
            UniversalId::TAG_DID,
            2,
            3,
            Arc::new(|data| {
                let mut result = data.to_vec();
                result.extend_from_slice(&[0u8; 8]);
                Ok(result)
            }),
        );

        assert!(registry.validate_migration_paths().is_ok());

        let v1_id = UniversalId::new(UniversalId::TAG_DID, 1, b"old-did");
        let v1_data = vec![0x01, 0x02, 0x03];

        let migrated = registry.maybe_migrate(&v1_id, &v1_data).unwrap();
        assert_eq!(migrated.len(), 19); // 3 + 8 + 8
    }

    /// Testet fehlende Migrations-Pfade
    #[test]
    fn test_missing_migration_path() {
        let mut registry = SchemaRegistry::new();
        registry.register_current(UniversalId::TAG_EVENT, 3);
        registry.register_migration(UniversalId::TAG_EVENT, 1, 2, identity_migration());

        let result = registry.validate_migration_paths();
        assert!(result.is_err());
    }

    /// Testet Version-Downgrade-Ablehnung
    #[test]
    fn test_version_downgrade_rejected() {
        let mut registry = SchemaRegistry::new();
        registry.register_current(UniversalId::TAG_REALM, 1);

        let future_id = UniversalId::new(UniversalId::TAG_REALM, 5, b"future");
        let result = registry.maybe_migrate(&future_id, &[1, 2, 3]);

        assert!(matches!(result, Err(MigrationError::VersionTooHigh { .. })));
    }

    /// Testet Standard-Registry
    #[test]
    fn test_default_registry() {
        let registry = SchemaRegistry::with_defaults();

        assert_eq!(registry.current_version(UniversalId::TAG_EVENT), Some(1));
        assert_eq!(registry.current_version(UniversalId::TAG_DID), Some(1));
        assert_eq!(registry.current_version(UniversalId::TAG_REALM), Some(1));
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
        let parent_coord = TemporalCoord::new(1000, 5, 42);
        let child_coord = TemporalCoord::new(1001, 6, 42);

        assert!(InvariantChecker::check_causal_order(&child_coord, &parent_coord).is_ok());
        assert!(InvariantChecker::check_causal_order(&parent_coord, &child_coord).is_err());
    }

    /// Testet Event-ID-Erstellung
    #[test]
    fn test_event_id_from_content() {
        let event_id = event_id_from_content(b"test-event");
        assert_eq!(event_id.type_tag(), UniversalId::TAG_EVENT);
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

        // Assoziativität
        let left = c1.seq(c2).seq(c3);
        let right = c1.seq(c2.seq(c3));

        assert_eq!(left.gas, right.gas);
        assert_eq!(left.mana, right.mana);
        assert!((left.trust_risk - right.trust_risk).abs() < 0.0001);

        // Neutrales Element
        let with_zero = c1.seq(Cost::ZERO);
        assert_eq!(with_zero.gas, c1.gas);

        assert!(InvariantChecker::check_cost_algebra(c1, c2, c3).is_ok());
    }

    /// Testet parallele Kosten-Kombination
    #[test]
    fn test_parallel_cost() {
        let c1 = Cost::new(100, 50, 0.1);
        let c2 = Cost::new(200, 100, 0.2);

        let parallel = c1.par(c2);

        // par: max(gas), sum(mana), max(risk)
        assert_eq!(parallel.gas, 200);
        assert_eq!(parallel.mana, 150); // 50 + 100
        assert!((parallel.trust_risk - 0.2).abs() < 0.0001);
    }
}

// ============================================================================
// Saga Tests
// ============================================================================

mod saga_tests {
    use super::*;

    /// Testet Saga-Erstellung aus Intent
    #[test]
    fn test_saga_from_intent() {
        let source = UniversalId::new(UniversalId::TAG_DID, 1, b"user-alice");
        let realm_id = realm_id_from_name("test-realm");

        let intent = Intent::new(
            source,
            Goal::Transfer {
                to: UniversalId::new(UniversalId::TAG_DID, 1, b"user-bob"),
                amount: 100,
                asset_type: "credits".to_string(),
            },
            realm_id,
            1,
        );

        let steps = vec![SagaStep::new(
            0,
            "Debit Alice",
            SagaAction::Transfer {
                from: source,
                to: UniversalId::new(UniversalId::TAG_DID, 1, b"escrow"),
                amount: 100,
                asset_type: "credits".to_string(),
            },
        )];

        let saga = Saga::from_intent(&intent, steps, 2);

        assert_eq!(saga.steps.len(), 1);
        assert!(matches!(saga.status, SagaStatus::Pending));
        assert!(!saga.is_completed());
    }

    /// Testet Saga-Compensation
    #[test]
    fn test_saga_with_compensation() {
        let source = UniversalId::new(UniversalId::TAG_DID, 1, b"user");

        let step = SagaStep::new(
            0,
            "Create Asset",
            SagaAction::Mint {
                asset_type: "token".to_string(),
                amount: 100,
                to: source,
                authorization: None,
            },
        )
        .with_compensation(SagaCompensation::new(
            "Burn minted tokens",
            SagaAction::Burn {
                asset_type: "token".to_string(),
                amount: 100,
                from: source,
                authorization: None,
            },
        ));

        assert!(step.compensation.is_some());
    }
}

// ============================================================================
// End-to-End Flow
// ============================================================================

mod end_to_end {
    use super::*;

    /// Vollständiger User-Flow
    #[test]
    fn test_complete_user_flow() {
        // 1. User erstellt DID
        let user_did = DID::new(DIDNamespace::Self_, b"alice-key");
        let mut user_doc = DIDDocument::new(user_did.clone());

        // 2. User ID erstellen
        let user_id = UniversalId::new(UniversalId::TAG_DID, 1, user_did.id.as_bytes());

        // 3. Trust-Vector starten als Newcomer
        let trust = TrustVector6D::NEWCOMER;
        assert_eq!(trust.r, 0.1);

        // 4. Nach vielen Interaktionen → Default Trust
        let improved_trust = TrustVector6D::DEFAULT;
        assert!(improved_trust.r > trust.r);

        // 5. User kann jetzt delegieren
        let delegate_did = DID::new(DIDNamespace::Self_, b"bob-key");
        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, delegate_did.id.as_bytes());

        let delegation = Delegation::new(
            user_id,
            delegate_id,
            0.7,
            vec![Capability::Read {
                resource: "realm:*".to_string(),
            }],
        );

        user_doc.delegations.push(delegation);
        assert_eq!(user_doc.delegations.len(), 1);

        // 6. Delegation ist gültig
        let now = TemporalCoord::now(100, &user_id);
        assert!(user_doc.delegations[0].is_valid(&now));
    }
}
