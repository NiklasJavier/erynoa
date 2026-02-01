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
    Cost, InvariantChecker, TemporalCoord, TrustDimension, TrustVector6D, UniversalId,
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
