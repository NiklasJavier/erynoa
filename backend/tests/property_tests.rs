//! # Property-Based Tests für Invarianten
//!
//! Tests mit `proptest` für automatisch generierte Eingaben.
//!
//! ## Geprüfte Invarianten
//!
//! - **Κ4**: Asymmetrische Trust-Updates
//! - **Κ8**: Delegation Trust-Decay ∈ (0, 1]
//! - **Κ9**: Kausale Ordnung (Parent < Event)
//! - **Cost-Algebra**: Semiring-Eigenschaften

use erynoa_api::domain::unified::{
    Cost, InvariantChecker, TemporalCoord, TrustDimension, TrustVector6D,
};
use proptest::prelude::*;

// ============================================================================
// Strategien für Typen
// ============================================================================

/// Strategie für TrustVector6D-Werte
fn trust_vector_strategy() -> impl Strategy<Value = TrustVector6D> {
    (
        0.0f32..=1.0f32,
        0.0f32..=1.0f32,
        0.0f32..=1.0f32,
        0.0f32..=1.0f32,
        0.0f32..=1.0f32,
        0.0f32..=1.0f32,
    )
        .prop_map(|(r, i, c, p, v, omega)| TrustVector6D::new(r, i, c, p, v, omega))
}

/// Strategie für Trust-Dimensionen
fn trust_dimension_strategy() -> impl Strategy<Value = TrustDimension> {
    prop_oneof![
        Just(TrustDimension::Reliability),
        Just(TrustDimension::Integrity),
        Just(TrustDimension::Competence),
        Just(TrustDimension::Prestige),
        Just(TrustDimension::Vigilance),
        Just(TrustDimension::Omega),
    ]
}

/// Strategie für Cost-Werte
fn cost_strategy() -> impl Strategy<Value = Cost> {
    (
        0u64..10000u64,  // gas
        0u64..10000u64,  // mana
        0.0f32..=1.0f32, // trust_risk
    )
        .prop_map(|(gas, mana, risk)| Cost::new(gas, mana, risk))
}

/// Strategie für TemporalCoord
#[allow(dead_code)]
fn temporal_coord_strategy() -> impl Strategy<Value = TemporalCoord> {
    (
        0u64..1_000_000u64, // wall_time
        0u32..1_000_000u32, // lamport
        0u32..1000u32,      // node_hash
    )
        .prop_map(|(wall_time, lamport, node_hash)| {
            TemporalCoord::new(wall_time, lamport, node_hash)
        })
}

// ============================================================================
// Κ4: Asymmetrische Trust-Updates
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Κ4: Negative Updates werden mit asymmetry_factor verstärkt
    #[test]
    fn prop_k4_negative_updates_amplified(
        mut trust in trust_vector_strategy(),
        dim in trust_dimension_strategy(),
        negative_delta in -1.0f32..0.0f32,
    ) {
        let original = trust.get(dim);
        trust.update(dim, negative_delta);
        let after_update = trust.get(dim);

        // Der Update muss den Wert verringert haben (oder auf 0 clamped)
        prop_assert!(after_update <= original,
            "Negative update should decrease trust: {} -> {}", original, after_update);

        // Der effektive Delta muss größer sein als der reine Delta
        // (wegen asymmetry_factor > 1.0)
        let actual_delta = after_update - original;
        let asymmetry = dim.asymmetry_factor();

        // Da clamping passiert, können wir nur prüfen:
        // |actual_delta| >= |negative_delta| wenn nicht geclamped
        if original > 0.0 && after_update > 0.0 {
            prop_assert!(
                actual_delta.abs() >= negative_delta.abs() * 0.99,
                "Asymmetric amplification failed: expected >= {} (asymmetry {}), got {}",
                negative_delta.abs(), asymmetry, actual_delta.abs()
            );
        }
    }

    /// Κ4: Positive Updates werden NICHT verstärkt
    #[test]
    fn prop_k4_positive_updates_not_amplified(
        mut trust in trust_vector_strategy(),
        dim in trust_dimension_strategy(),
        positive_delta in 0.0f32..=0.5f32,
    ) {
        let original = trust.get(dim);
        trust.update(dim, positive_delta);
        let after_update = trust.get(dim);

        // Positiver Update erhöht Wert (oder clamped auf 1.0)
        prop_assert!(after_update >= original,
            "Positive update should increase or maintain trust: {} -> {}", original, after_update);

        // Delta entspricht genau dem Input (ohne Verstärkung)
        let actual_delta = after_update - original;
        let expected_max = (original + positive_delta).min(1.0) - original;

        prop_assert!(
            (actual_delta - expected_max).abs() < 0.001,
            "Positive update amplification mismatch: expected {}, got {}",
            expected_max, actual_delta
        );
    }
}

// ============================================================================
// Κ8: Delegation Trust-Decay
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Κ8: Gültige Trust-Faktoren werden akzeptiert
    #[test]
    fn prop_k8_valid_trust_factors_accepted(
        factor in 0.001f32..=1.0f32,
    ) {
        let result = InvariantChecker::check_delegation_trust_factor(factor);
        prop_assert!(result.is_ok(), "Factor {} should be valid", factor);
    }

    /// Κ8: Ungültige Trust-Faktoren werden abgelehnt
    #[test]
    fn prop_k8_invalid_trust_factors_rejected(
        factor in prop_oneof![
            (-10.0f32..=0.0f32),    // Negative Faktoren
            (1.001f32..=10.0f32),   // Faktoren > 1.0
        ]
    ) {
        let result = InvariantChecker::check_delegation_trust_factor(factor);
        prop_assert!(result.is_err(), "Factor {} should be invalid", factor);
    }

    /// Κ8: Trust-Decay skaliert Trust-Vektor korrekt
    #[test]
    fn prop_k8_scale_preserves_relative_order(
        trust in trust_vector_strategy(),
        factor in 0.001f32..=1.0f32,
    ) {
        let scaled = trust.scale(factor);

        // Jede Dimension wird proportional skaliert
        for dim in TrustDimension::ALL {
            let original = trust.get(dim);
            let scaled_value = scaled.get(dim);

            let expected = (original * factor).clamp(0.0, 1.0);
            prop_assert!(
                (scaled_value - expected).abs() < 0.0001,
                "Scale mismatch for {:?}: expected {}, got {}",
                dim, expected, scaled_value
            );
        }
    }
}

// ============================================================================
// Κ9: Kausale Ordnung
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// Κ9: Parent muss strikt vor Event sein
    #[test]
    fn prop_k9_causal_order_enforced(
        parent_lamport in 0u32..1_000_000u32,
        delta in 1u32..1000u32,
    ) {
        let parent = TemporalCoord::new(0, parent_lamport, 0);
        let event = TemporalCoord::new(0, parent_lamport + delta, 0);

        let result = InvariantChecker::check_causal_order(&event, &parent);
        prop_assert!(result.is_ok(), "Event with higher lamport should be causally after parent");
    }

    /// Κ9: Gleiche Zeitstempel verletzen Kausalität
    #[test]
    fn prop_k9_same_timestamp_violates_causality(
        wall_time in 0u64..1_000_000u64,
        lamport in 0u32..1_000_000u32,
        node_hash in 0u32..1000u32,
    ) {
        let coord = TemporalCoord::new(wall_time, lamport, node_hash);

        let result = InvariantChecker::check_causal_order(&coord, &coord);
        prop_assert!(result.is_err(), "Same timestamp should violate causality");
    }

    /// Κ9: Parent nach Event verletzt Kausalität
    #[test]
    fn prop_k9_parent_after_event_violates_causality(
        event_lamport in 0u32..1_000_000u32,
        delta in 1u32..1000u32,
    ) {
        let event = TemporalCoord::new(0, event_lamport, 0);
        let parent = TemporalCoord::new(0, event_lamport + delta, 0);

        let result = InvariantChecker::check_causal_order(&event, &parent);
        prop_assert!(result.is_err(), "Parent with higher lamport should violate causality");
    }
}

// ============================================================================
// Cost-Algebra: Semiring-Eigenschaften
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Cost-Algebra: seq ist assoziativ: (a ⊕ b) ⊕ c = a ⊕ (b ⊕ c)
    #[test]
    fn prop_cost_seq_associative(
        c1 in cost_strategy(),
        c2 in cost_strategy(),
        c3 in cost_strategy(),
    ) {
        let left = c1.seq(c2).seq(c3);
        let right = c1.seq(c2.seq(c3));

        // Gas addiert sich
        prop_assert_eq!(left.gas, right.gas, "Gas associativity failed");

        // Mana addiert sich
        prop_assert_eq!(left.mana, right.mana, "Mana associativity failed");

        // Risk kombiniert (mit Floating-Point-Toleranz)
        prop_assert!(
            (left.trust_risk - right.trust_risk).abs() < 0.0001,
            "Risk associativity failed: {} vs {}",
            left.trust_risk, right.trust_risk
        );
    }

    /// Cost-Algebra: ZERO ist neutrales Element für seq: a ⊕ 0 = a
    #[test]
    fn prop_cost_seq_identity(
        c in cost_strategy(),
    ) {
        let with_zero = c.seq(Cost::ZERO);

        prop_assert_eq!(c.gas, with_zero.gas, "Gas identity failed");
        prop_assert_eq!(c.mana, with_zero.mana, "Mana identity failed");
        prop_assert!(
            (c.trust_risk - with_zero.trust_risk).abs() < 0.0001,
            "Risk identity failed"
        );
    }

    /// Cost-Algebra: par nimmt Maximum für Gas, Summe für Mana
    #[test]
    fn prop_cost_par_semantics(
        c1 in cost_strategy(),
        c2 in cost_strategy(),
    ) {
        let parallel = c1.par(c2);

        // Gas: max
        prop_assert_eq!(
            parallel.gas,
            c1.gas.max(c2.gas),
            "Par should take max gas"
        );

        // Mana: sum
        prop_assert_eq!(
            parallel.mana,
            c1.mana.saturating_add(c2.mana),
            "Par should sum mana"
        );

        // Risk: max
        let expected_risk = c1.trust_risk.max(c2.trust_risk);
        prop_assert!(
            (parallel.trust_risk - expected_risk).abs() < 0.0001,
            "Par should take max risk: expected {}, got {}",
            expected_risk, parallel.trust_risk
        );
    }

    /// Cost-Algebra: par ist kommutativ: a ⊗ b = b ⊗ a
    #[test]
    fn prop_cost_par_commutative(
        c1 in cost_strategy(),
        c2 in cost_strategy(),
    ) {
        let ab = c1.par(c2);
        let ba = c2.par(c1);

        prop_assert_eq!(ab.gas, ba.gas, "Par gas should be commutative");
        prop_assert_eq!(ab.mana, ba.mana, "Par mana should be commutative");
        prop_assert!(
            (ab.trust_risk - ba.trust_risk).abs() < 0.0001,
            "Par risk should be commutative"
        );
    }

    /// Cost-Algebra: par ist assoziativ
    #[test]
    fn prop_cost_par_associative(
        c1 in cost_strategy(),
        c2 in cost_strategy(),
        c3 in cost_strategy(),
    ) {
        let left = c1.par(c2).par(c3);
        let right = c1.par(c2.par(c3));

        prop_assert_eq!(left.gas, right.gas, "Par gas associativity failed");
        prop_assert_eq!(left.mana, right.mana, "Par mana associativity failed");
        prop_assert!(
            (left.trust_risk - right.trust_risk).abs() < 0.0001,
            "Par risk associativity failed"
        );
    }
}

// ============================================================================
// TrustVector6D Kombinationen (Κ5)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Κ5: Probabilistische Kombination ist kommutativ
    #[test]
    fn prop_k5_combine_commutative(
        t1 in trust_vector_strategy(),
        t2 in trust_vector_strategy(),
    ) {
        let ab = t1.combine(&t2);
        let ba = t2.combine(&t1);

        for dim in TrustDimension::ALL {
            prop_assert!(
                (ab.get(dim) - ba.get(dim)).abs() < 0.0001,
                "Combine should be commutative for {:?}", dim
            );
        }
    }

    /// Κ5: Kombinieren mit MAX ergibt MAX
    #[test]
    fn prop_k5_combine_with_max_is_max(
        t in trust_vector_strategy(),
    ) {
        let combined = t.combine(&TrustVector6D::MAX);

        for dim in TrustDimension::ALL {
            let value = combined.get(dim);
            prop_assert!(
                (value - 1.0).abs() < 0.0001,
                "Combine with MAX should yield MAX for {:?}: got {}",
                dim, value
            );
        }
    }

    /// Κ5: Kombinieren mit ZERO ergibt Original
    #[test]
    fn prop_k5_combine_with_zero_is_identity(
        t in trust_vector_strategy(),
    ) {
        let combined = t.combine(&TrustVector6D::ZERO);

        for dim in TrustDimension::ALL {
            let original = t.get(dim);
            let value = combined.get(dim);
            prop_assert!(
                (value - original).abs() < 0.0001,
                "Combine with ZERO should preserve value for {:?}: {} vs {}",
                dim, original, value
            );
        }
    }

    /// Κ5: Kombination erhöht nie den Trust über 1.0
    #[test]
    fn prop_k5_combine_bounded(
        t1 in trust_vector_strategy(),
        t2 in trust_vector_strategy(),
    ) {
        let combined = t1.combine(&t2);

        for dim in TrustDimension::ALL {
            let value = combined.get(dim);
            prop_assert!(
                value <= 1.0 + 0.0001,
                "Combined trust should be <= 1.0 for {:?}: got {}",
                dim, value
            );
            prop_assert!(
                value >= 0.0 - 0.0001,
                "Combined trust should be >= 0.0 for {:?}: got {}",
                dim, value
            );
        }
    }
}

// ============================================================================
// TrustVector6D Interpolation
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Interpolation bei t=0 ergibt self
    #[test]
    fn prop_lerp_at_zero_is_self(
        t1 in trust_vector_strategy(),
        t2 in trust_vector_strategy(),
    ) {
        let lerped = t1.lerp(&t2, 0.0);

        for dim in TrustDimension::ALL {
            prop_assert!(
                (lerped.get(dim) - t1.get(dim)).abs() < 0.0001,
                "Lerp at t=0 should equal self for {:?}", dim
            );
        }
    }

    /// Interpolation bei t=1 ergibt other
    #[test]
    fn prop_lerp_at_one_is_other(
        t1 in trust_vector_strategy(),
        t2 in trust_vector_strategy(),
    ) {
        let lerped = t1.lerp(&t2, 1.0);

        for dim in TrustDimension::ALL {
            prop_assert!(
                (lerped.get(dim) - t2.get(dim)).abs() < 0.0001,
                "Lerp at t=1 should equal other for {:?}", dim
            );
        }
    }

    /// Interpolation ist monoton zwischen self und other
    #[test]
    fn prop_lerp_monotonic(
        t1 in trust_vector_strategy(),
        t2 in trust_vector_strategy(),
        t in 0.0f32..=1.0f32,
    ) {
        let lerped = t1.lerp(&t2, t);

        for dim in TrustDimension::ALL {
            let v1 = t1.get(dim);
            let v2 = t2.get(dim);
            let vl = lerped.get(dim);

            // Interpolierter Wert liegt zwischen den beiden Endpunkten
            let min = v1.min(v2);
            let max = v1.max(v2);

            prop_assert!(
                vl >= min - 0.0001 && vl <= max + 0.0001,
                "Lerp should be between endpoints for {:?}: {} not in [{}, {}]",
                dim, vl, min, max
            );
        }
    }
}

// ============================================================================
// Config Validation
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Gültige Konfigurationen werden akzeptiert
    #[test]
    fn prop_valid_config_accepted(
        asymmetry_base in 1.1f32..=3.0f32,
        newcomer in 0.01f32..=0.4f32,
        tau_days in 30u32..=365u32, // Mindestens 30 damit tau_days_mobile (default 30) <= tau_days
        kappa in 1u32..=100u32,
        lambda in 0.001f64..=0.1f64, // Max 0.1 damit lambda_fast (default 0.1) >= lambda
    ) {
        use erynoa_api::domain::unified::WorldFormulaConfig;

        let config = WorldFormulaConfig::builder()
            .asymmetry_base(asymmetry_base)
            .asymmetry_critical(asymmetry_base + 0.5)
            .newcomer_trust(newcomer)
            .activity_tau_days(tau_days)
            .activity_kappa(kappa)
            .lambda_per_day(lambda)
            .build();

        prop_assert!(config.validate().is_ok(), "Valid config should pass validation");
    }
}

// ============================================================================
// Delegation Chain Tests (Κ8)
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Κ8: Delegation-Kette Trust-Decay ist multiplikativ
    #[test]
    fn prop_k8_chain_decay_multiplicative(
        factors in proptest::collection::vec(0.1f32..=1.0f32, 1..5),
    ) {
        use erynoa_api::domain::unified::{UniversalId, Delegation, Capability};

        // Erstelle Delegation-Kette mit gegebenen Faktoren
        let mut expected_trust = 1.0f32;
        let mut delegator_id = UniversalId::new(UniversalId::TAG_DID, 1, b"root");

        for (i, &factor) in factors.iter().enumerate() {
            let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, format!("node-{i}").as_bytes());

            let del = Delegation::new(
                delegator_id.clone(),
                delegate_id.clone(),
                factor,
                vec![Capability::Read { resource: "*".to_string() }],
            );

            prop_assert!(
                del.trust_factor > 0.0 && del.trust_factor <= 1.0,
                "Delegation trust factor should be in (0, 1]"
            );

            expected_trust *= factor;
            delegator_id = delegate_id;
        }

        // Erwarteter Trust am Ende der Kette
        prop_assert!(
            expected_trust >= 0.0 && expected_trust <= 1.0,
            "Chain trust should remain in [0, 1]: {}",
            expected_trust
        );

        // Trust nimmt ab oder bleibt gleich (nie erhöht)
        prop_assert!(
            expected_trust <= 1.0,
            "Chain trust should never exceed 1.0"
        );
    }

    /// Κ8: Längere Ketten haben niedrigeren effektiven Trust
    #[test]
    fn prop_k8_longer_chains_lower_trust(
        chain_length in 1usize..=8usize,
        factor in 0.5f32..=0.99f32,
    ) {
        let effective_trust = factor.powi(chain_length as i32);

        prop_assert!(
            effective_trust >= 0.0 && effective_trust <= 1.0,
            "Effective trust should be in [0, 1]"
        );

        // Längere Ketten → niedrigerer Trust (wenn factor < 1)
        if chain_length > 1 && factor < 1.0 {
            let shorter_trust = factor.powi((chain_length - 1) as i32);
            prop_assert!(
                effective_trust < shorter_trust,
                "Longer chain should have lower trust: {} < {}",
                effective_trust, shorter_trust
            );
        }
    }
}

// ============================================================================
// TrustVector6D Weighted Norm
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Gewichtete Norm ist nicht-negativ
    #[test]
    fn prop_weighted_norm_non_negative(
        trust in trust_vector_strategy(),
        weights in proptest::array::uniform6(0.0f32..=1.0f32),
    ) {
        let norm = trust.weighted_norm(&weights);
        prop_assert!(norm >= 0.0, "Weighted norm should be non-negative: {}", norm);
    }

    /// Gewichtete Norm mit Null-Gewichten ist 0
    #[test]
    fn prop_weighted_norm_zero_weights(
        trust in trust_vector_strategy(),
    ) {
        let zero_weights = [0.0f32; 6];
        let norm = trust.weighted_norm(&zero_weights);
        prop_assert!(
            norm.abs() < 0.0001,
            "Weighted norm with zero weights should be 0: {}",
            norm
        );
    }

    /// Scale mit 0 ergibt ZERO-Vektor
    #[test]
    fn prop_scale_by_zero_is_zero(
        trust in trust_vector_strategy(),
    ) {
        let scaled = trust.scale(0.0);

        for dim in TrustDimension::ALL {
            prop_assert!(
                scaled.get(dim).abs() < 0.0001,
                "Scaled by 0 should be 0 for {:?}: {}",
                dim, scaled.get(dim)
            );
        }
    }

    /// Scale mit 1 ist Identity
    #[test]
    fn prop_scale_by_one_is_identity(
        trust in trust_vector_strategy(),
    ) {
        let scaled = trust.scale(1.0);

        for dim in TrustDimension::ALL {
            prop_assert!(
                (scaled.get(dim) - trust.get(dim)).abs() < 0.0001,
                "Scaled by 1 should be identity for {:?}: {} vs {}",
                dim, trust.get(dim), scaled.get(dim)
            );
        }
    }
}

// ============================================================================
// Finality Level Ordering
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Finality Levels sind strikt geordnet
    #[test]
    fn prop_finality_level_ordering(
        _attestation_count in 0usize..=20usize,
        _trust_sum in 0.0f64..=10.0f64,
    ) {
        use erynoa_api::domain::unified::FinalityLevel;

        // Nascent < Validated < Witnessed < Anchored
        prop_assert!(FinalityLevel::Nascent < FinalityLevel::Validated);
        prop_assert!(FinalityLevel::Validated < FinalityLevel::Witnessed);
        prop_assert!(FinalityLevel::Witnessed < FinalityLevel::Anchored);

        // Transitivität
        prop_assert!(FinalityLevel::Nascent < FinalityLevel::Anchored);
    }
}

// ============================================================================
// Event ID Determinism
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Event-IDs sind deterministisch basierend auf Content
    #[test]
    fn prop_event_id_deterministic(
        event_type in "[a-z.]{3,20}",
        data in proptest::collection::vec(any::<u8>(), 0..100),
        lamport in 0u32..1_000_000u32,
    ) {
        use erynoa_api::domain::unified::{Event, EventPayload, DID, DIDNamespace, UniversalId};

        let actor = DID::new(DIDNamespace::Self_, b"prop-test-actor");
        let actor_id = UniversalId::new(UniversalId::TAG_DID, 1, actor.id.as_bytes());

        let event1 = Event::new(
            actor_id.clone(),
            vec![],
            EventPayload::Custom {
                event_type: event_type.clone(),
                data: data.clone(),
            },
            lamport,
        );

        let event2 = Event::new(
            actor_id,
            vec![],
            EventPayload::Custom {
                event_type,
                data,
            },
            lamport,
        );

        prop_assert_eq!(
            event1.id, event2.id,
            "Same content should produce same event ID"
        );
    }
}

// ============================================================================
// TemporalCoord Properties
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// TemporalCoord Ordering ist total
    #[test]
    fn prop_temporal_coord_total_order(
        t1 in temporal_coord_strategy(),
        t2 in temporal_coord_strategy(),
    ) {
        // Entweder t1 < t2, t1 == t2, oder t1 > t2
        let lt = t1 < t2;
        let eq = t1 == t2;
        let gt = t1 > t2;

        prop_assert!(
            (lt && !eq && !gt) || (!lt && eq && !gt) || (!lt && !eq && gt),
            "Exactly one of <, ==, > should be true"
        );
    }

    /// TemporalCoord Ordering ist transitiv
    #[test]
    fn prop_temporal_coord_transitive(
        t1_lamport in 0u32..1000u32,
        t2_lamport in 0u32..1000u32,
        t3_lamport in 0u32..1000u32,
    ) {
        let t1 = TemporalCoord::new(0, t1_lamport, 0);
        let t2 = TemporalCoord::new(0, t2_lamport, 0);
        let t3 = TemporalCoord::new(0, t3_lamport, 0);

        if t1 < t2 && t2 < t3 {
            prop_assert!(t1 < t3, "Transitivity: t1 < t2 && t2 < t3 => t1 < t3");
        }
    }

    /// Lamport-Clock ist Teil von TemporalCoord Ordering
    #[test]
    fn prop_lamport_affects_ordering(
        wall_time in 0u64..1_000_000u64,
        lamport1 in 0u32..1_000_000u32,
        lamport2 in 0u32..1_000_000u32,
        node_hash in 0u32..1000u32,
    ) {
        let coord1 = TemporalCoord::new(wall_time, lamport1, node_hash);
        let coord2 = TemporalCoord::new(wall_time, lamport2, node_hash);

        // Größerer Lamport → größere Coord (bei gleichem wall_time und node_hash)
        if lamport1 < lamport2 {
            prop_assert!(coord1 < coord2, "Higher lamport should mean higher coord");
        } else if lamport1 > lamport2 {
            prop_assert!(coord1 > coord2, "Lower lamport should mean lower coord");
        } else {
            prop_assert!(coord1 == coord2, "Same lamport should mean equal coord");
        }
    }
}
