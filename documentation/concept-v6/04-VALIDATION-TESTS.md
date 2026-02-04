# Validierungstests – V6 Optimierungen

> **Version:** 6.0
> **Datum:** Februar 2026
> **Fokus:** Testspezifikationen und Erwartungswerte

---

## 1. Testübersicht

### 1.1 Neue Tests

| Test                                 | Datei      | Ziel                          |
| ------------------------------------ | ---------- | ----------------------------- |
| `test_sigmoid_scaling_fix`           | formula.rs | Sigmoid nicht saturiert       |
| `test_ln_offset_fix`                 | formula.rs | Neue Entitäten haben Einfluss |
| `test_chain_trust_corrected_formula` | trust.rs   | Korrekte Dämpfung             |

### 1.2 Teststrategie

```
┌─────────────────────────────────────────────────────────────────────┐
│                    V6 Validierungs-Matrix                           │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Unit Tests          Integration Tests       Property-Based Tests   │
│  ───────────         ─────────────────       ───────────────────    │
│  • Grenzwerte        • E2E Contribution      • Monotonie            │
│  • Spezialfälle      • Trust-Propagation     • Beschränktheit       │
│  • Numerik           • Formula-Engine        • Kommutativität       │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. Unit-Tests

### 2.1 Test: Sigmoid-Skalierung

**Datei:** `backend/src/domain/unified/formula.rs`

```rust
#[test]
fn test_sigmoid_scaling_fix() {
    // Scenario: Typische Contribution mit hohen Werten
    // Vorher (V5): Inner-Term ~12 → σ ≈ 0.99999 (saturiert)
    // Nachher (V6): Inner-Term ~0.8 → σ ≈ 0.69 (differenzierend)

    let contribution = WorldFormulaContribution::new(subject.clone(), 100)
        .with_activity(Activity::new(50, 100))   // activity ≈ 1.0
        .with_trust(&high_trust_vector())        // trust_norm ≈ 0.8
        .with_causal_history(150)                // ln(151) ≈ 5.0
        .with_surprisal(Surprisal::new(1.0))     // dampened ≈ 3.0
        .with_human_factor(HumanFactor::new_verified(human))
        .with_temporal_weight(1.0)
        .build();

    let value = contribution.compute();

    // ASSERTION 1: Nicht saturiert (< 0.95)
    assert!(value < 0.95,
        "Sigmoid should not saturate. Got: {}", value);

    // ASSERTION 2: Aber auch nicht zu niedrig (> 0.3)
    assert!(value > 0.3,
        "Sigmoid should still be meaningful. Got: {}", value);

    // ASSERTION 3: Typischer Bereich
    assert!(value > 0.5 && value < 0.85,
        "Typical high-contribution should be in [0.5, 0.85]. Got: {}", value);
}
```

**Erwartete Werte:**

| Szenario              | V5-Wert | V6-Wert | Status        |
| --------------------- | ------- | ------- | ------------- |
| Hohe Contribution     | ~0.99   | ~0.69   | ✅ Verbessert |
| Mittlere Contribution | ~0.95   | ~0.55   | ✅ Verbessert |
| Niedrige Contribution | ~0.80   | ~0.40   | ✅ Verbessert |

### 2.2 Test: ln(+1) Offset

**Datei:** `backend/src/domain/unified/formula.rs`

```rust
#[test]
fn test_ln_offset_fix() {
    // Scenario: Neue Entität mit nur 1 kausaler Verbindung
    // Vorher (V5): ln(1) = 0 → Contribution = 0 (unfair!)
    // Nachher (V6): ln(1+1) = ln(2) ≈ 0.693 → Contribution > 0

    let new_entity = WorldFormulaContribution::new(subject.clone(), 1)
        .with_activity(Activity::new(1, 1))
        .with_trust(&TrustVector6D::new(0.7, 0.7, 0.7, 0.5, 0.6, 0.7))
        .with_causal_history(1)  // Nur 1 Event!
        .with_surprisal(Surprisal::new(2.0))  // Neue Entitäten sind überraschend
        .with_human_factor(HumanFactor::new_verified(human))
        .with_temporal_weight(1.0)
        .build();

    let value = new_entity.compute();

    // ASSERTION: Contribution ist NICHT null
    assert!(value > 0.01,
        "New entity with 1 event should have non-zero contribution. Got: {}", value);

    // ASSERTION 2: Aber auch nicht unfair hoch
    assert!(value < 0.8,
        "New entity should not have too high contribution. Got: {}", value);
}
```

**Mathematische Begründung:**

```
V5:  ln(max(1, 1)) = ln(1) = 0
     inner = trust_norm × 0 × surprisal = 0
     σ(0) = 0.5
     contribution = activity × 0.5 × human × temporal ≈ 0.5

     ABER: Der Trust-Einfluss ist vollständig eliminiert!

V6:  ln(1 + 1) = ln(2) ≈ 0.693
     inner = trust_norm × 0.693 × surprisal / 15
           ≈ 0.7 × 0.693 × 2.0 / 15 ≈ 0.065
     σ(0.065) ≈ 0.516
     contribution = activity × 0.516 × human × temporal

     Trust-Einfluss ist erhalten!
```

### 2.3 Test: Chain-Trust Korrektur

**Datei:** `backend/src/domain/unified/trust.rs`

```rust
#[test]
fn test_chain_trust_corrected_formula() {
    // Property 1: Identity für n=1
    let single = TrustCombination::chain_trust(&[0.8]);
    assert!((single - 0.8).abs() < 0.001,
        "Single element should be unchanged. Got: {}", single);

    // Property 2: Dämpfung für n>1
    let chain_2 = TrustCombination::chain_trust(&[0.8, 0.8]);
    let chain_4 = TrustCombination::chain_trust(&[0.8, 0.8, 0.8, 0.8]);

    // Erwartung V6:
    // n=2: (0.8 × 0.8)^(1/√2) = 0.64^0.707 ≈ 0.715
    // n=4: (0.8^4)^(1/√4) = 0.4096^0.5 = 0.64

    assert!((chain_2 - 0.715).abs() < 0.02,
        "Chain of 2 should be ~0.715. Got: {}", chain_2);

    assert!((chain_4 - 0.64).abs() < 0.02,
        "Chain of 4 should be ~0.64. Got: {}", chain_4);

    // Property 3: Strenge Monotonie
    assert!(chain_4 < chain_2,
        "Longer chains should have lower trust");
    assert!(chain_2 < single,
        "Chain of 2 should be less than single");

    // Property 4: Nicht zu extrem gedämpft (V5 Bug)
    // V5 gab für chain_4 etwa 0.25 – viel zu streng!
    assert!(chain_4 > 0.5,
        "Chain of 4 with 0.8s should not be below 0.5. Got: {}", chain_4);
}
```

**Vergleichstabelle:**

| Kette                | V5-Wert | V6-Wert | V5-Formel        | V6-Formel   |
| -------------------- | ------- | ------- | ---------------- | ----------- |
| [0.8]                | 0.800   | 0.800   | exp(ln(0.8)/1)   | 0.8^1       |
| [0.8, 0.8]           | ~0.569  | ~0.715  | exp(-0.446/1.41) | 0.64^0.707  |
| [0.8, 0.8, 0.8, 0.8] | ~0.256  | ~0.640  | exp(-0.891/2.0)  | 0.4096^0.5  |
| [0.9]×10             | ~0.348  | ~0.713  | exp(-1.054/3.16) | 0.349^0.316 |

---

## 3. Integration-Tests

### 3.1 End-to-End Contribution-Test

```rust
#[tokio::test]
async fn test_full_contribution_pipeline_v6() {
    let engine = WorldFormulaEngine::new();

    // Simuliere verschiedene Entitäten
    let entities = vec![
        ("Alice", 1000, 0.9, 0.8, 500, 1.5),   // Erfahrene Nutzerin
        ("Bob", 100, 0.7, 0.7, 50, 2.0),       // Aktiver Neuling
        ("Carol", 5000, 0.95, 0.85, 2000, 0.5), // Veteran, wenig überraschend
        ("Dave", 1, 0.5, 0.6, 1, 3.0),          // Komplett neu
    ];

    let contributions: Vec<_> = entities.iter().map(|(name, events, trust, activity, history, surprisal)| {
        let contrib = engine.compute_contribution(
            name,
            *events,
            *trust as f32,
            *activity,
            *history,
            *surprisal,
        );
        (name, contrib)
    }).collect();

    // ASSERTION 1: Alle haben positive Contributions
    for (name, contrib) in &contributions {
        assert!(contrib > 0.0, "{} should have positive contribution", name);
    }

    // ASSERTION 2: Keine Saturation
    for (name, contrib) in &contributions {
        assert!(contrib < 0.95, "{} should not saturate", name);
    }

    // ASSERTION 3: Differentiation (Unterschiede sind messbar)
    let values: Vec<f64> = contributions.iter().map(|(_, c)| *c).collect();
    let variance = statistical_variance(&values);
    assert!(variance > 0.01, "Contributions should be well-distributed, variance: {}", variance);

    // ASSERTION 4: Dave (neu) hat trotzdem Einfluss
    let dave_contrib = contributions.iter().find(|(n, _)| *n == &"Dave").unwrap().1;
    assert!(dave_contrib > 0.1, "New entity Dave should have meaningful contribution: {}", dave_contrib);
}
```

### 3.2 Trust-Propagation-Test

```rust
#[tokio::test]
async fn test_trust_propagation_v6() {
    let trust_engine = TrustEngine::new();

    // Kette: A → B → C → D → E
    let chain_trust_levels = [0.9, 0.85, 0.8, 0.75, 0.7];

    // V6 Chain-Trust berechnen
    let propagated = trust_engine.chain_propagate(&chain_trust_levels);

    // ASSERTION 1: Propagierter Trust ist positiv
    assert!(propagated > 0.0);

    // ASSERTION 2: Propagierter Trust ist niedriger als Minimum
    let min_direct = chain_trust_levels.iter().cloned().fold(1.0, f32::min);
    assert!(propagated < min_direct,
        "Propagated trust should be less than minimum direct trust");

    // ASSERTION 3: Aber nicht zu streng (V5 Bug)
    // V5 gab für diese Kette etwa 0.28
    // V6 sollte etwa 0.58 geben
    assert!(propagated > 0.5,
        "Trust propagation should not be too strict. Got: {}", propagated);

    // Erwartung V6: (0.9 × 0.85 × 0.8 × 0.75 × 0.7)^(1/√5)
    //             = 0.321^0.447 ≈ 0.608
    assert!((propagated - 0.608).abs() < 0.05,
        "Expected ~0.608, got: {}", propagated);
}
```

---

## 4. Property-Based Tests

### 4.1 Monotonie-Properties

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn sigmoid_monotonic(inner1 in 0.0..10.0f64, inner2 in 0.0..10.0f64) {
        const SIGMOID_SCALE: f64 = 15.0;
        let s1 = 1.0 / (1.0 + (-inner1 / SIGMOID_SCALE).exp());
        let s2 = 1.0 / (1.0 + (-inner2 / SIGMOID_SCALE).exp());

        if inner1 < inner2 {
            prop_assert!(s1 < s2, "Sigmoid should be monotonically increasing");
        }
    }

    #[test]
    fn chain_trust_monotonic_length(
        base_trust in 0.5..0.99f32,
        n1 in 1usize..10,
        n2 in 1usize..10
    ) {
        let chain1: Vec<f32> = vec![base_trust; n1];
        let chain2: Vec<f32> = vec![base_trust; n2];

        let t1 = TrustCombination::chain_trust(&chain1);
        let t2 = TrustCombination::chain_trust(&chain2);

        if n1 < n2 && base_trust < 1.0 {
            prop_assert!(t1 >= t2, "Longer chains should have equal or lower trust");
        }
    }
}
```

### 4.2 Beschränktheits-Properties

```rust
proptest! {
    #[test]
    fn sigmoid_bounded(
        trust_norm in 0.0..1.0f64,
        connectivity in 1u32..10000,
        surprisal in 0.0..10.0f64
    ) {
        let ln_connectivity = (connectivity as f64 + 1.0).ln();
        const SIGMOID_SCALE: f64 = 15.0;
        let inner = trust_norm * ln_connectivity * surprisal / SIGMOID_SCALE;
        let sigmoid = 1.0 / (1.0 + (-inner).exp());

        // Sigmoid ist immer im Bereich (0, 1)
        prop_assert!(sigmoid > 0.0 && sigmoid < 1.0,
            "Sigmoid must be in (0,1). Got: {}", sigmoid);
    }

    #[test]
    fn chain_trust_bounded(trusts in prop::collection::vec(0.0f32..1.0, 1..20)) {
        let result = TrustCombination::chain_trust(&trusts);

        prop_assert!(result >= 0.0 && result <= 1.0,
            "Chain trust must be in [0,1]. Got: {}", result);

        // Sollte auch nicht größer als das Maximum sein
        let max_trust = trusts.iter().cloned().fold(0.0f32, f32::max);
        prop_assert!(result <= max_trust + 0.001,
            "Chain trust should not exceed max individual trust");
    }
}
```

### 4.3 Spezialfall-Properties

```rust
proptest! {
    #[test]
    fn chain_trust_identity(t in 0.0f32..1.0) {
        // Kette mit einem Element = das Element selbst
        let single = TrustCombination::chain_trust(&[t]);
        prop_assert!((single - t).abs() < 0.0001,
            "Single element chain should equal the element. {} != {}", single, t);
    }

    #[test]
    fn chain_trust_zeros(n in 1usize..10) {
        // Kette mit einer 0 = 0
        let mut chain = vec![0.8f32; n];
        chain[n/2] = 0.0;  // Füge eine 0 ein

        let result = TrustCombination::chain_trust(&chain);
        prop_assert!(result < 0.0001,
            "Chain with zero should be near zero. Got: {}", result);
    }

    #[test]
    fn ln_offset_positive(connectivity in 0u32..1000000) {
        // ln(n+1) ist immer positiv für n >= 0
        let ln_val = (connectivity as f64 + 1.0).ln();
        prop_assert!(ln_val >= 0.0,
            "ln(n+1) should always be non-negative. Got: {}", ln_val);

        // Und für n >= 1 ist es strikt positiv
        if connectivity >= 1 {
            prop_assert!(ln_val > 0.0,
                "ln(n+1) should be positive for n >= 1. Got: {}", ln_val);
        }
    }
}
```

---

## 5. Regressions-Tests

### 5.1 Bekannte V5-Bugs

```rust
#[test]
fn regression_v5_sigmoid_saturation() {
    // V5 Bug: High-value entities all got ~0.99
    let high_entity = create_high_value_entity();
    let medium_entity = create_medium_value_entity();

    let high_contrib = high_entity.compute();
    let medium_contrib = medium_entity.compute();

    // V5: both were ~0.99, undistinguishable
    // V6: should be clearly different
    let difference = (high_contrib - medium_contrib).abs();
    assert!(difference > 0.1,
        "V5 Bug: High and medium should be distinguishable. Diff: {}", difference);
}

#[test]
fn regression_v5_new_entity_invisible() {
    // V5 Bug: Entities with connectivity=1 had effectively zero trust influence
    let new_entity = WorldFormulaContribution::new(subject.clone(), 1)
        .with_activity(Activity::new(1, 1))
        .with_trust(&TrustVector6D::new(0.9, 0.9, 0.9, 0.9, 0.9, 0.9))  // Sehr vertrauenswürdig!
        .with_causal_history(1)
        .with_surprisal(Surprisal::new(1.0))
        .with_human_factor(HumanFactor::new_verified(human))
        .with_temporal_weight(1.0)
        .build();

    // V5: Trust-Einfluss war 0, weil ln(1)=0
    // V6: Trust-Einfluss sollte messbar sein

    // Vergleich mit Entität ohne Trust
    let no_trust_entity = WorldFormulaContribution::new(subject.clone(), 1)
        .with_activity(Activity::new(1, 1))
        .with_trust(&TrustVector6D::new(0.1, 0.1, 0.1, 0.1, 0.1, 0.1))  // Kaum vertrauenswürdig
        .with_causal_history(1)
        .with_surprisal(Surprisal::new(1.0))
        .with_human_factor(HumanFactor::new_verified(human))
        .with_temporal_weight(1.0)
        .build();

    let trusted_value = new_entity.compute();
    let untrusted_value = no_trust_entity.compute();

    // V5: both were equal (Trust had no effect)
    // V6: trusted should be higher
    assert!(trusted_value > untrusted_value + 0.05,
        "V5 Bug: Trust should matter for new entities. Trusted: {}, Untrusted: {}",
        trusted_value, untrusted_value);
}

#[test]
fn regression_v5_chain_trust_too_strict() {
    // V5 Bug: Chain of 4 with 0.8 trust gave only 0.25
    let chain = vec![0.8f32; 4];
    let result = TrustCombination::chain_trust(&chain);

    // V5: ~0.256 (zu streng)
    // V6: ~0.64 (angemessen)
    assert!(result > 0.5,
        "V5 Bug: Chain trust was too strict. Got: {}", result);

    // Sollte aber auch nicht zu lax sein
    assert!(result < 0.85,
        "Chain trust should still apply dampening. Got: {}", result);
}
```

---

## 6. Performance-Tests

### 6.1 Benchmark: Sigmoid-Berechnung

```rust
#[bench]
fn bench_sigmoid_v6(b: &mut Bencher) {
    let test_cases: Vec<(f64, f64, f64, u32)> = (0..1000)
        .map(|i| {
            let trust = (i % 100) as f64 / 100.0;
            let surprisal = (i % 50) as f64 / 10.0;
            let connectivity = (i % 10000) as u32 + 1;
            (trust, surprisal, (connectivity as f64 + 1.0).ln(), connectivity)
        })
        .collect();

    b.iter(|| {
        let mut sum = 0.0;
        for (trust, surprisal, ln_conn, _) in &test_cases {
            const SIGMOID_SCALE: f64 = 15.0;
            let inner = trust * ln_conn * surprisal / SIGMOID_SCALE;
            let sigmoid = 1.0 / (1.0 + (-inner).exp());
            sum += sigmoid;
        }
        sum
    });
}
```

### 6.2 Benchmark: Chain-Trust

```rust
#[bench]
fn bench_chain_trust_v6(b: &mut Bencher) {
    let chains: Vec<Vec<f32>> = (1..100)
        .map(|n| vec![0.8f32; n])
        .collect();

    b.iter(|| {
        let mut sum = 0.0f32;
        for chain in &chains {
            sum += TrustCombination::chain_trust(chain);
        }
        sum
    });
}
```

---

## 7. Test-Daten-Generator

```rust
/// Generiere realistische Test-Szenarien
pub mod test_fixtures {
    use super::*;

    pub fn new_user() -> WorldFormulaContribution {
        WorldFormulaContribution::new(Subject::random(), 1)
            .with_activity(Activity::new(1, 1))
            .with_trust(&TrustVector6D::default())
            .with_causal_history(1)
            .with_surprisal(Surprisal::new(3.0))  // Sehr überraschend
            .with_human_factor(HumanFactor::new_verified(Human::default()))
            .with_temporal_weight(1.0)
            .build()
    }

    pub fn active_contributor() -> WorldFormulaContribution {
        WorldFormulaContribution::new(Subject::random(), 500)
            .with_activity(Activity::new(50, 90))
            .with_trust(&TrustVector6D::new(0.8, 0.75, 0.85, 0.7, 0.8, 0.75))
            .with_causal_history(200)
            .with_surprisal(Surprisal::new(1.5))
            .with_human_factor(HumanFactor::new_verified(Human::default()))
            .with_temporal_weight(0.95)
            .build()
    }

    pub fn veteran_expert() -> WorldFormulaContribution {
        WorldFormulaContribution::new(Subject::random(), 10000)
            .with_activity(Activity::new(100, 365))
            .with_trust(&TrustVector6D::new(0.95, 0.9, 0.95, 0.85, 0.9, 0.92))
            .with_causal_history(5000)
            .with_surprisal(Surprisal::new(0.3))  // Wenig überraschend
            .with_human_factor(HumanFactor::new_verified(Human::default()))
            .with_temporal_weight(0.85)
            .build()
    }

    pub fn suspicious_actor() -> WorldFormulaContribution {
        WorldFormulaContribution::new(Subject::random(), 50)
            .with_activity(Activity::new(50, 2))  // Plötzliche Aktivität
            .with_trust(&TrustVector6D::new(0.3, 0.4, 0.5, 0.2, 0.7, 0.3))
            .with_causal_history(10)
            .with_surprisal(Surprisal::new(4.5))  // Sehr überraschend
            .with_human_factor(HumanFactor::new_unverified())
            .with_temporal_weight(1.0)
            .build()
    }

    pub fn typical_trust_chain() -> Vec<f32> {
        vec![0.9, 0.85, 0.8, 0.75, 0.7]  // 5 Stufen
    }

    pub fn long_trust_chain() -> Vec<f32> {
        vec![0.8; 10]  // 10 Stufen
    }

    pub fn weak_link_chain() -> Vec<f32> {
        vec![0.9, 0.9, 0.3, 0.9, 0.9]  // Schwaches Glied in der Mitte
    }
}
```

---

## 8. Validierungs-Checkliste

### Vor dem Release

- [ ] Alle Unit-Tests bestehen
- [ ] Alle Integration-Tests bestehen
- [ ] Property-Tests mit 10.000 Iterationen bestanden
- [ ] Regressions-Tests für V5-Bugs bestehen
- [ ] Performance-Benchmarks zeigen keine Regression (< 5% Unterschied)
- [ ] Numerische Stabilität bei Extremwerten validiert
- [ ] Documentation stimmt mit Implementation überein

### Manuelle Überprüfung

- [ ] Sigmoid-Verteilung in Produktionsdaten visualisiert
- [ ] Chain-Trust-Verhalten mit realen Ketten getestet
- [ ] Neue Entitäten haben messbare Contributions
- [ ] Keine Division durch Null möglich
- [ ] Keine Overflow/Underflow-Szenarien

---

## 9. Kontinuierliche Validierung

```yaml
# .github/workflows/formula-validation.yml
name: World Formula V6 Validation

on:
  push:
    paths:
      - "backend/src/domain/unified/formula.rs"
      - "backend/src/domain/unified/trust.rs"

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Unit Tests
        run: cargo test --package erynoa-api -- formula trust

      - name: Run Property Tests
        run: cargo test --package erynoa-api -- --ignored proptest
        env:
          PROPTEST_CASES: 10000

      - name: Run Benchmarks
        run: cargo bench --package erynoa-api -- sigmoid chain_trust

      - name: Validate Numerical Stability
        run: cargo run --package erynoa-api --bin validate-v6
```
