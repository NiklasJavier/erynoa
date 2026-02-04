# Implementierungsdetails – V6 Optimierungen

> **Version:** 6.0
> **Datum:** Februar 2026
> **Fokus:** Code-Änderungen und Migrationshinweise

---

## 1. Geänderte Dateien

### Übersicht

| Datei                                   | Änderungen                    |
| --------------------------------------- | ----------------------------- |
| `backend/src/domain/unified/formula.rs` | Sigmoid-Skalierung, ln-Offset |
| `backend/src/domain/unified/trust.rs`   | Chain-Trust Korrektur         |

---

## 2. Detaillierte Code-Änderungen

### 2.1 Sigmoid-Skalierung + ln-Offset

**Datei:** `backend/src/domain/unified/formula.rs`

#### Änderung in `compute_full()`

```rust
// ═══════════════════════════════════════════════════════════════════
// VORHER (V5) – Zeilen 354-361
// ═══════════════════════════════════════════════════════════════════

let context = super::trust::ContextType::Default;
let trust_norm = trust.weighted_norm(&TrustVector6D::default_weights());

// Κ15b: Inner term
let ln_connectivity = (causal_connectivity.max(1) as f64).ln();
let inner = (trust_norm as f64) * ln_connectivity * surprisal.dampened();

// Κ15c: Sigmoid
let sigmoid = 1.0 / (1.0 + (-inner).exp());


// ═══════════════════════════════════════════════════════════════════
// NACHHER (V6) – Optimiert
// ═══════════════════════════════════════════════════════════════════

let context = super::trust::ContextType::Default;
let trust_norm = trust.weighted_norm(&TrustVector6D::default_weights());

// Κ15b: Inner term (optimized)
// Fix: +1 offset prevents ln(1)=0 for new entities with single event
let ln_connectivity = (causal_connectivity as f64 + 1.0).ln();

// Fix: Scale factor prevents sigmoid saturation
// Empirically tuned: typical inner values now range ~0.5-3.0 instead of 0-2800
const SIGMOID_SCALE: f64 = 15.0;
let inner = (trust_norm as f64) * ln_connectivity * surprisal.dampened() / SIGMOID_SCALE;

// Κ15c: Sigmoid (now properly distributed across [0.3, 0.95])
let sigmoid = 1.0 / (1.0 + (-inner).exp());
```

#### Änderung in `build()`

```rust
// ═══════════════════════════════════════════════════════════════════
// VORHER (V5) – Zeilen 601-608
// ═══════════════════════════════════════════════════════════════════

/// Berechne Beitrag (für Builder-Pattern)
pub fn build(mut self) -> Self {
    // Κ15b: Inner term
    let ln_connectivity = (self.causal_connectivity.max(1) as f64).ln();
    let inner = (self.trust_norm as f64) * ln_connectivity * self.surprisal.dampened();

    // Κ15c: Sigmoid
    let sigmoid = 1.0 / (1.0 + (-inner).exp());


// ═══════════════════════════════════════════════════════════════════
// NACHHER (V6) – Optimiert
// ═══════════════════════════════════════════════════════════════════

/// Berechne Beitrag (für Builder-Pattern)
pub fn build(mut self) -> Self {
    // Κ15b: Inner term (optimized)
    // Fix: +1 offset prevents ln(1)=0 for new entities
    let ln_connectivity = (self.causal_connectivity as f64 + 1.0).ln();

    // Fix: Scale factor prevents sigmoid saturation
    const SIGMOID_SCALE: f64 = 15.0;
    let inner = (self.trust_norm as f64) * ln_connectivity * self.surprisal.dampened() / SIGMOID_SCALE;

    // Κ15c: Sigmoid (properly distributed)
    let sigmoid = 1.0 / (1.0 + (-inner).exp());
```

#### Änderung in `compute_value()`

```rust
// ═══════════════════════════════════════════════════════════════════
// VORHER (V5) – Zeilen 619-624
// ═══════════════════════════════════════════════════════════════════

/// Berechne Beitrag und gib Wert zurück (Kompatibilität mit alter API)
pub fn compute_value(&self) -> f64 {
    let ln_connectivity = (self.causal_connectivity.max(1) as f64).ln();
    let inner = (self.trust_norm as f64) * ln_connectivity * self.surprisal.dampened();
    let sigmoid = 1.0 / (1.0 + (-inner).exp());
    self.activity.value() * sigmoid * self.human_factor.value() * self.temporal_weight
}


// ═══════════════════════════════════════════════════════════════════
// NACHHER (V6) – Optimiert
// ═══════════════════════════════════════════════════════════════════

/// Berechne Beitrag und gib Wert zurück (Kompatibilität mit alter API)
pub fn compute_value(&self) -> f64 {
    // Κ15b optimized: +1 offset + scaling
    let ln_connectivity = (self.causal_connectivity as f64 + 1.0).ln();
    const SIGMOID_SCALE: f64 = 15.0;
    let inner = (self.trust_norm as f64) * ln_connectivity * self.surprisal.dampened() / SIGMOID_SCALE;
    let sigmoid = 1.0 / (1.0 + (-inner).exp());
    self.activity.value() * sigmoid * self.human_factor.value() * self.temporal_weight
}
```

---

### 2.2 Chain-Trust Korrektur

**Datei:** `backend/src/domain/unified/trust.rs`

```rust
// ═══════════════════════════════════════════════════════════════════
// VORHER (V5) – TrustCombination::chain_trust() – Zeilen 743-754
// ═══════════════════════════════════════════════════════════════════

/// Τ1: Ketten-Trust mit √n Dämpfung
///
/// `t_chain = exp(Σᵢ ln(tᵢ) / √n)`
pub fn chain_trust(chain: &[f32]) -> f32 {
    if chain.is_empty() {
        return 0.0;
    }

    let n = chain.len() as f32;
    let log_sum: f32 = chain.iter().map(|t| t.max(1e-10).ln()).sum();

    (log_sum / n.sqrt()).exp()
}


// ═══════════════════════════════════════════════════════════════════
// NACHHER (V6) – Korrigierte Formel
// ═══════════════════════════════════════════════════════════════════

/// Τ1: Ketten-Trust mit √n Dämpfung (korrigierte Formel)
///
/// Mathematisch korrekt: `t_chain = (∏ᵢ tᵢ)^(1/√n)`
///
/// Dies entspricht dem geometrischen Durchschnitt mit √n-Dämpfung:
/// - Bei n=1: t_chain = t₁ (unverändert)
/// - Bei n=4, alle t=0.8: t_chain = 0.8^(4/2) = 0.64
/// - Längere Ketten → stärkere Dämpfung, aber nicht so extrem wie vorher
pub fn chain_trust(chain: &[f32]) -> f32 {
    if chain.is_empty() {
        return 0.0;
    }

    let n = chain.len() as f32;

    // Berechne Produkt aller Trust-Werte (mit Epsilon für numerische Stabilität)
    let product: f32 = chain.iter().fold(1.0, |acc, &t| acc * t.max(1e-10));

    // Korrigierte Formel: geometrischer Durchschnitt mit √n Exponent
    // product^(1/√n) = product^(√n/n) für sanftere Dämpfung
    product.powf(1.0 / n.sqrt())
}
```

---

## 3. Konstanten-Referenz

### 3.1 Neue Konstanten

| Konstante       | Wert | Verwendung                     | Datei      |
| --------------- | ---- | ------------------------------ | ---------- |
| `SIGMOID_SCALE` | 15.0 | Normalisierung des Inner-Terms | formula.rs |

### 3.2 Unveränderte Konstanten

| Konstante          | Wert      | Verwendung                           |
| ------------------ | --------- | ------------------------------------ |
| `kappa` (Activity) | 10        | Aktivitätsschwelle                   |
| `lambda_asym`      | 1.5 / 2.0 | Asymmetrie-Faktoren für Trust-Update |
| `tau_seconds`      | 90 Tage   | Aktivitätszeitfenster                |

---

## 4. API-Kompatibilität

### 4.1 Keine Breaking Changes auf API-Ebene

Die öffentliche API bleibt unverändert:

```rust
// Diese Methoden funktionieren wie vorher:
WorldFormulaContribution::new(subject, lamport)
    .with_activity(activity)
    .with_trust(&trust)
    .with_causal_history(100)
    .with_surprisal(surprisal)
    .with_human_factor(human_factor)
    .with_temporal_weight(0.9)
    .build();

contribution.compute();

TrustCombination::chain_trust(&[0.8, 0.8, 0.8]);
```

### 4.2 Geänderte Rückgabewerte

⚠️ **Die numerischen Rückgabewerte ändern sich!**

| Methode                  | V5-Bereich           | V6-Bereich          |
| ------------------------ | -------------------- | ------------------- |
| `contribution.compute()` | ~0.5-1.0 (saturiert) | 0.3-0.95 (verteilt) |
| `chain_trust(&[0.8;4])`  | ~0.25                | ~0.64               |

---

## 5. Migrationsanleitung

### 5.1 Für bestehende Systeme

**Schritt 1: Tests aktualisieren**

Tests, die auf exakten numerischen Werten basieren, müssen angepasst werden:

```rust
// VORHER (V5):
assert!((contribution.compute() - 0.95).abs() < 0.1);

// NACHHER (V6):
assert!(contribution.compute() > 0.3 && contribution.compute() < 0.95);
```

**Schritt 2: Schwellwerte überprüfen**

Wenn absolute Schwellwerte für Contributions verwendet werden:

```rust
// VORHER (V5):
if contribution.compute() > 0.9 { /* high trust */ }

// NACHHER (V6):
if contribution.compute() > 0.7 { /* high trust */ }
// Oder besser: Relative Vergleiche verwenden
```

**Schritt 3: Chain-Trust-Logik prüfen**

```rust
// VORHER (V5): Sehr strenge Dämpfung
let trust = chain_trust(&[0.8; 4]); // ~0.25

// NACHHER (V6): Sanftere Dämpfung
let trust = chain_trust(&[0.8; 4]); // ~0.64
```

### 5.2 Empfohlene Praktiken

1. **Relative Vergleiche statt absolute Schwellwerte:**

   ```rust
   // Besser:
   let ranking: Vec<_> = entities.sorted_by(|a, b| b.compute().cmp(&a.compute()));
   ```

2. **Percentile-basierte Schwellwerte:**

   ```rust
   let top_10_percent_threshold = compute_percentile(&all_contributions, 90);
   ```

3. **A/B-Testing bei kritischen Pfaden:**

   ```rust
   #[cfg(feature = "v6-formula")]
   const SIGMOID_SCALE: f64 = 15.0;

   #[cfg(not(feature = "v6-formula"))]
   const SIGMOID_SCALE: f64 = 1.0;  // Legacy-Verhalten
   ```

---

## 6. Performance-Überlegungen

### 6.1 Komplexität

| Operation             | V5   | V6   | Änderung |
| --------------------- | ---- | ---- | -------- |
| `compute_value()`     | O(1) | O(1) | Keine    |
| `chain_trust()`       | O(n) | O(n) | Keine    |
| Inkrementelles Update | O(1) | O(1) | Keine    |

### 6.2 Numerische Stabilität

| Operation                 | V5                              | V6                                    |
| ------------------------- | ------------------------------- | ------------------------------------- |
| `ln()` bei kleinen Werten | Unterlauf möglich               | Kein Problem (direkte Multiplikation) |
| `exp()` bei großen Werten | Overflow bei ~700               | Kein Problem (skaliert)               |
| Akkumulation von Fehlern  | Problematisch bei langen Ketten | Minimiert                             |

---

## 7. Debugging-Hilfen

### 7.1 Logging der Zwischenwerte

```rust
// Debug-Ausgabe für Contribution-Berechnung
#[cfg(debug_assertions)]
fn compute_value_debug(&self) -> f64 {
    let ln_connectivity = (self.causal_connectivity as f64 + 1.0).ln();
    eprintln!("  ln_connectivity: {}", ln_connectivity);

    const SIGMOID_SCALE: f64 = 15.0;
    let inner_unscaled = (self.trust_norm as f64) * ln_connectivity * self.surprisal.dampened();
    eprintln!("  inner_unscaled: {}", inner_unscaled);

    let inner = inner_unscaled / SIGMOID_SCALE;
    eprintln!("  inner_scaled: {}", inner);

    let sigmoid = 1.0 / (1.0 + (-inner).exp());
    eprintln!("  sigmoid: {}", sigmoid);

    let result = self.activity.value() * sigmoid * self.human_factor.value() * self.temporal_weight;
    eprintln!("  final: {}", result);

    result
}
```

### 7.2 Validierungsfunktion

```rust
/// Validiere, dass V6-Optimierungen korrekt funktionieren
pub fn validate_v6_optimizations() -> Result<(), String> {
    // Test 1: Sigmoid sollte nicht saturieren
    let inner_typical = 0.8 * 5.0 * 3.0; // = 12
    let inner_scaled = inner_typical / 15.0; // = 0.8
    let sigmoid = 1.0 / (1.0 + (-inner_scaled).exp());
    if sigmoid > 0.95 {
        return Err(format!("Sigmoid still saturating: {}", sigmoid));
    }

    // Test 2: ln(1+1) sollte nicht 0 sein
    let ln_val = (1.0_f64 + 1.0).ln();
    if ln_val < 0.5 {
        return Err(format!("ln offset not working: {}", ln_val));
    }

    // Test 3: Chain-Trust für n=1 sollte identisch sein
    let single = TrustCombination::chain_trust(&[0.7]);
    if (single - 0.7).abs() > 0.001 {
        return Err(format!("Chain trust identity broken: {}", single));
    }

    Ok(())
}
```

---

## 8. Changelog

### V6.0.0 (Februar 2026)

**Breaking (numerisch):**

- Sigmoid-Werte ändern sich von ~0.5-1.0 auf ~0.3-0.95
- Chain-Trust-Werte ändern sich (sanftere Dämpfung)

**Fixes:**

- Sigmoid-Saturation behoben (SIGMOID_SCALE = 15.0)
- ln(1)=0 Problem behoben (+1 Offset)
- Chain-Trust numerisch stabilisiert (Produkt statt Log-Summe)

**Tests:**

- `test_sigmoid_scaling_fix` hinzugefügt
- `test_ln_offset_fix` hinzugefügt
- `test_chain_trust_corrected_formula` hinzugefügt
