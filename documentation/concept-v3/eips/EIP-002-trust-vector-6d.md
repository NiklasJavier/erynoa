# EIP-002: Trust Vector 6D Specification

> **EIP:** 002
> **Titel:** Trust Vector 6D Specification (R, I, C, P, V, Ω)
> **Status:** Draft
> **Version:** 0.2
> **Typ:** Standard
> **Ebene:** E1 (Fundament) / E2 (Emergenz)
> **Erstellt:** Januar 2026
> **Aktualisiert:** Januar 2026
> **Abhängigkeiten:** EIP-001 (DID:erynoa), EIP-004 (Bayesian Trust Update)

---

## Abstract

Diese Spezifikation definiert den 6-dimensionalen Trust-Vektor W(s), der die Vertrauenswürdigkeit eines Agenten im Erynoa-Netzwerk repräsentiert. Der Vektor besteht aus sechs orthogonalen Dimensionen:

- **R** – Reliability (Zuverlässigkeit)
- **I** – Integrity (Integrität)
- **C** – Competence (Kompetenz)
- **P** – Predictability (Vorhersagbarkeit)
- **V** – Vigilance (Wachsamkeit)
- **Ω** – Omega-Alignment (Regelkonformität)

Jede Dimension wird unabhängig berechnet, ermöglicht kontextspezifische Gewichtung und aggregiert zu einem skalaren Trust-Wert für die Systemgleichung.

---

## Motivation

Ein eindimensionaler Trust-Score (z.B. "Trust = 0.7") verliert wichtige Nuancen:

- Ein Agent kann zuverlässig (R hoch), aber inkompetent (C niedrig) sein
- Ein Agent kann integer (I hoch), aber unvorhersagbar (P niedrig) sein
- Ein Agent kann wachsam (V hoch), aber regelfeindlich (Ω niedrig) sein

Der 6D-Vektor ermöglicht:

1. **Kontextuelle Entscheidungen** – Ein Finanz-Realm gewichtet I und Ω höher, ein Kreativ-Realm gewichtet C höher
2. **Differenzierte Analyse** – Schwächen können identifiziert und adressiert werden
3. **Faire Bewertung** – Neue Agenten werden in Dimensionen bewertet, für die Daten existieren
4. **Sybil-Resistenz** – Dimensionen wie V und Ω sind schwer zu faken

---

## Spezifikation

### 1. Der Trust-Vektor

#### 1.1 Definition

Der Trust-Vektor eines Agenten s ist ein 6-Tupel:

```
W(s) = (R, I, C, P, V, Ω) ∈ [0,1]⁶
```

Jede Komponente ist ein Wert zwischen 0 (kein Vertrauen) und 1 (volles Vertrauen).

#### 1.2 Datenstruktur

```rust
/// 6D Trust-Vektor
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustVector {
    /// Reliability: Anteil erfüllter Verpflichtungen
    pub r: f64,

    /// Integrity: Konsistenz zwischen Aussagen und Fakten
    pub i: f64,

    /// Competence: Qualität der Leistungen
    pub c: f64,

    /// Predictability: Verhaltenskonsistenz über Zeit
    pub p: f64,

    /// Vigilance: Korrektheit bei Anomalie-Meldungen
    pub v: f64,

    /// Omega-Alignment: Regelkonformität
    pub omega: f64,
}

impl TrustVector {
    /// Neutral-Prior für neue Agenten (Beta(2,2))
    pub fn neutral() -> Self {
        Self {
            r: 0.5,
            i: 0.5,
            c: 0.5,
            p: 0.5,
            v: 0.5,
            omega: 0.5,
        }
    }
}
```

#### 1.3 JSON-Repräsentation

```json
{
  "trust_vector": {
    "R": 0.82,
    "I": 0.91,
    "C": 0.75,
    "P": 0.88,
    "V": 0.67,
    "Ω": 0.94
  },
  "confidence": {
    "R": 0.85,
    "I": 0.92,
    "C": 0.71,
    "P": 0.89,
    "V": 0.45,
    "Ω": 0.96
  },
  "observation_count": {
    "R": 423,
    "I": 512,
    "C": 287,
    "P": 445,
    "V": 34,
    "Ω": 589
  }
}
```

### 2. Die sechs Dimensionen

#### 2.1 R – Reliability (Zuverlässigkeit)

**Definition:** Der Anteil erfüllter Verpflichtungen an zugesagten Verpflichtungen.

**Formel:**

```
R(s) = (fulfilled_commitments + α) / (total_commitments + 2α)
```

Mit α = 2 (Beta-Prior-Parameter).

**Beobachtbare Events:**

| Event-Typ               | Impact auf R             |
| ----------------------- | ------------------------ |
| TAT CLOSE (erfolgreich) | +1 fulfilled             |
| TAT ABORT (durch Agent) | +1 total, 0 fulfilled    |
| Deadline eingehalten    | +1 fulfilled             |
| Deadline verpasst       | +1 total, 0 fulfilled    |
| Service online          | +0.1 fulfilled (pro Tag) |
| Service offline         | +0.1 total, 0 fulfilled  |

**Beispiel:**

```
Agent hat 95 von 100 Transaktionen erfolgreich abgeschlossen.

R = (95 + 2) / (100 + 4) = 97 / 104 ≈ 0.933
```

**Konfidenz:**

```
confidence_R = 1 - sqrt(R × (1-R) / (total_commitments + 2α))
```

---

#### 2.2 I – Integrity (Integrität)

**Definition:** Konsistenz zwischen Aussagen und verifizierten Fakten.

**Formel:**

```
I(s) = (verified_true_statements + α) / (total_verifiable_statements + 2α)
```

**Beobachtbare Events:**

| Event-Typ                      | Impact auf I         |
| ------------------------------ | -------------------- |
| Attestation bestätigt          | +1 verified_true     |
| Attestation widerlegt          | +1 total, 0 verified |
| Self-Report korrekt            | +1 verified_true     |
| Self-Report falsch             | +1 total, 0 verified |
| Credential verifiziert         | +0.5 verified_true   |
| Credential widerrufen (Betrug) | +2 total, 0 verified |

**Gewichtung nach Schwere:**

```rust
enum IntegrityEventWeight {
    MinorMisstatement = 1,      // Kleiner Fehler
    SignificantFalsehood = 5,   // Signifikante Falschaussage
    FraudulentClaim = 20,       // Betrügerische Behauptung
}
```

**Beispiel:**

```
Agent hat 200 Aussagen gemacht, 190 wurden verifiziert,
8 waren Minor-Fehler, 2 waren Significant.

verified_true = 190
total = 200 + (8 × 1) + (2 × 5) = 218

I = (190 + 2) / (218 + 4) = 192 / 222 ≈ 0.865
```

---

#### 2.3 C – Competence (Kompetenz)

**Definition:** Qualitätsmetrik basierend auf Peer-Reviews und Outcome-Bewertungen.

**Formel:**

```
C(s) = Σ(rating_i × weight_i) / Σ(weight_i)
```

Wobei `weight_i = trust(reviewer_i)` – gewichtet nach Vertrauen in den Reviewer.

**Beobachtbare Events:**

| Event-Typ                     | Rating-Range | Gewicht         |
| ----------------------------- | ------------ | --------------- |
| Peer-Review (5-Sterne)        | [0, 1]       | T(reviewer)     |
| TAT-Outcome-Rating            | [0, 1]       | T(counterparty) |
| Automatischer Qualitäts-Check | [0, 1]       | 0.5 (fixed)     |
| Credential von Experte        | 0.9 (fixed)  | T(issuer)       |

**Bayessche Modellierung:**

```
Prior: C ~ Beta(2, 2)
Posterior: C ~ Beta(α + Σ(rating × weight), β + Σ((1-rating) × weight))
```

**Beispiel:**

```
3 Bewertungen:
- Reviewer A (Trust 0.9) gibt 4/5 Sterne → rating = 0.8
- Reviewer B (Trust 0.6) gibt 5/5 Sterne → rating = 1.0
- Reviewer C (Trust 0.3) gibt 3/5 Sterne → rating = 0.6

C = (0.8×0.9 + 1.0×0.6 + 0.6×0.3) / (0.9 + 0.6 + 0.3)
  = (0.72 + 0.6 + 0.18) / 1.8
  = 1.5 / 1.8
  ≈ 0.833
```

---

#### 2.4 P – Predictability (Vorhersagbarkeit)

**Definition:** Konsistenz des Verhaltens über Zeit, gemessen als inverse Varianz.

**Formel:**

```
P(s) = 1 / (1 + k × variance(behavior))
```

Mit k = 10 als Skalierungsfaktor.

**Verhaltensmetriken:**

| Metrik           | Beschreibung                | Erwarteter Bereich |
| ---------------- | --------------------------- | ------------------ |
| Response Time    | Antwortzeit auf Anfragen    | [0s, ∞)            |
| Transaction Rate | Transaktionen pro Tag       | [0, ∞)             |
| Online Hours     | Stunden online pro Tag      | [0, 24]            |
| Rejection Rate   | Ablehnungsrate von Anfragen | [0, 1]             |

**Berechnung der Varianz:**

```rust
fn calculate_predictability(observations: &[BehaviorSnapshot]) -> f64 {
    let n = observations.len() as f64;
    if n < 10.0 {
        return 0.5; // Nicht genug Daten
    }

    let metrics = [
        normalized_variance(&observations, |o| o.response_time),
        normalized_variance(&observations, |o| o.transaction_rate),
        normalized_variance(&observations, |o| o.online_hours),
        normalized_variance(&observations, |o| o.rejection_rate),
    ];

    let avg_variance = metrics.iter().sum::<f64>() / 4.0;
    1.0 / (1.0 + 10.0 * avg_variance)
}
```

**Beispiel:**

```
Agent hat sehr konstantes Verhalten:
- Response Time: μ=100ms, σ=10ms → variance_norm ≈ 0.01
- Online Hours: μ=16h, σ=2h → variance_norm ≈ 0.016

avg_variance ≈ 0.013
P = 1 / (1 + 10 × 0.013) = 1 / 1.13 ≈ 0.885
```

---

#### 2.5 V – Vigilance (Wachsamkeit)

**Definition:** Anteil korrekt gemeldeter Anomalien an allen Meldungen.

**Formel:**

```
V(s) = (correct_reports + α) / (total_reports + 2α)
```

**Beobachtbare Events:**

| Event-Typ                          | Impact auf V        |
| ---------------------------------- | ------------------- |
| Anomalie gemeldet → bestätigt      | +1 correct          |
| Anomalie gemeldet → widerlegt      | +1 total, 0 correct |
| Anomalie nicht gemeldet → entdeckt | -0.5 (Penalty)      |
| False Positive                     | +1 total, 0 correct |
| True Positive (kritisch)           | +2 correct          |

**Wichtung nach Schweregrad:**

```rust
enum AnomalySeverity {
    Low = 1,       // Kleine Auffälligkeit
    Medium = 2,    // Regelverstoß
    High = 5,      // Sicherheitsrisiko
    Critical = 10, // Systembedrohung
}
```

**Beispiel:**

```
Agent hat 50 Anomalien gemeldet:
- 40 waren korrekt (davon 5 kritisch)
- 10 waren False Positives

correct = 35 + (5 × 2) = 45  // Kritische zählen doppelt
total = 40 + 10 = 50

V = (45 + 2) / (50 + 4) = 47 / 54 ≈ 0.870
```

**Hinweis:** V ist die am schwersten zu manipulierende Dimension, da sie aktive Netzwerkbeteiligung erfordert.

---

#### 2.6 Ω – Omega-Alignment (Regelkonformität)

**Definition:** Anteil regelkonformer Aktionen an Gesamtaktionen.

**Formel:**

```
Ω(s) = (compliant_actions + α) / (total_actions + 2α)
```

**Beobachtbare Events:**

| Event-Typ               | Impact auf Ω           |
| ----------------------- | ---------------------- |
| Aktion innerhalb Policy | +1 compliant           |
| Policy-Verstoß (minor)  | +1 total, 0 compliant  |
| Policy-Verstoß (major)  | +5 total, 0 compliant  |
| Governance-Teilnahme    | +0.5 compliant         |
| Governance-Sabotage     | +10 total, 0 compliant |

**Ebenen der Regelkonformität:**

```rust
enum ComplianceLevel {
    /// Globale Erynoa-Axiome (E1-E7)
    GlobalAxioms,

    /// Realm-spezifische Regeln
    RealmRules,

    /// Bilaterale Vereinbarungen
    ContractTerms,

    /// Empfohlene Best Practices
    BestPractices,
}

fn compliance_weight(level: ComplianceLevel) -> f64 {
    match level {
        ComplianceLevel::GlobalAxioms => 10.0,   // Kritisch
        ComplianceLevel::RealmRules => 3.0,     // Wichtig
        ComplianceLevel::ContractTerms => 2.0,  // Standard
        ComplianceLevel::BestPractices => 0.5,  // Optional
    }
}
```

**Beispiel:**

```
Agent hat 1000 Aktionen durchgeführt:
- 990 waren compliant
- 8 minor Policy-Verstöße (RealmRules)
- 2 major Verstöße (GlobalAxioms)

compliant = 990
total = 1000 + (8 × 3) + (2 × 10) = 1044

Ω = (990 + 2) / (1044 + 4) = 992 / 1048 ≈ 0.947
```

---

### 3. Aggregation zum Skalar

#### 3.1 Gewichtete Summe

Der Trust-Vektor wird zu einem Skalar aggregiert:

```
W_scalar(s) = Σ(wᵢ × Wᵢ(s))
```

#### 3.2 Default-Gewichte

| Dimension | Gewicht | Begründung                   |
| --------- | ------- | ---------------------------- |
| R         | 0.15    | Grundlegende Zuverlässigkeit |
| I         | 0.15    | Kernvertrauen                |
| C         | 0.15    | Leistungsqualität            |
| P         | 0.10    | Stabilität                   |
| V         | 0.20    | Aktive Netzwerkbeteiligung   |
| Ω         | 0.25    | Systemgesundheit             |

**Summe:** 1.0

#### 3.3 Kontextuelle Gewichte

Verschiedene Realms können unterschiedliche Gewichte definieren:

```json
{
  "realm": "did:erynoa:circle:finance-trading",
  "trust_weights": {
    "R": 0.2,
    "I": 0.25,
    "C": 0.15,
    "P": 0.15,
    "V": 0.1,
    "Ω": 0.15
  }
}
```

```json
{
  "realm": "did:erynoa:circle:creative-arts",
  "trust_weights": {
    "R": 0.1,
    "I": 0.1,
    "C": 0.35,
    "P": 0.05,
    "V": 0.15,
    "Ω": 0.25
  }
}
```

#### 3.4 Implementierung

```rust
pub struct TrustWeights {
    pub r: f64,
    pub i: f64,
    pub c: f64,
    pub p: f64,
    pub v: f64,
    pub omega: f64,
}

impl TrustWeights {
    pub fn default() -> Self {
        Self {
            r: 0.15,
            i: 0.15,
            c: 0.15,
            p: 0.10,
            v: 0.20,
            omega: 0.25,
        }
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        let sum = self.r + self.i + self.c + self.p + self.v + self.omega;
        if (sum - 1.0).abs() > 0.001 {
            return Err(ValidationError::WeightsSumNotOne(sum));
        }
        Ok(())
    }
}

impl TrustVector {
    pub fn to_scalar(&self, weights: &TrustWeights) -> f64 {
        weights.r * self.r
            + weights.i * self.i
            + weights.c * self.c
            + weights.p * self.p
            + weights.v * self.v
            + weights.omega * self.omega
    }
}
```

---

### 4. Bayessche Trust-Evolution

#### 4.1 Prior-Verteilung

Jede Dimension startet mit einem Beta(α₀, β₀) Prior:

```
Prior: Wᵢ ~ Beta(2, 2)
E[Wᵢ] = 2 / (2 + 2) = 0.5
Var[Wᵢ] = 4 / (4 × 5) = 0.2
```

#### 4.2 Likelihood-Update

Bei einem neuen Event e wird der Prior zum Posterior aktualisiert:

```
Posterior: Wᵢ ~ Beta(α₀ + success_weight, β₀ + failure_weight)
```

**Beispiel für Reliability:**

```
Vor Event: R ~ Beta(10, 5)  → E[R] = 10/15 ≈ 0.667
Event: TAT erfolgreich abgeschlossen (weight = 1)
Nach Event: R ~ Beta(11, 5) → E[R] = 11/16 ≈ 0.688
```

#### 4.3 Konfidenzintervall

Das 95%-Konfidenzintervall wird aus der Beta-Verteilung berechnet:

```rust
fn confidence_interval_95(alpha: f64, beta: f64) -> (f64, f64) {
    let dist = Beta::new(alpha, beta).unwrap();
    let lower = dist.inverse_cdf(0.025);
    let upper = dist.inverse_cdf(0.975);
    (lower, upper)
}
```

**Beispiel:**

```
R ~ Beta(50, 10)
E[R] = 50/60 ≈ 0.833
95%-CI: [0.72, 0.92]
```

#### 4.4 Konfidenz-Score

Der Konfidenz-Score misst, wie sicher wir uns über den Trust-Wert sind:

```
confidence = 1 - (upper_95 - lower_95) / 2
```

| Beobachtungen | 95%-CI Breite | Konfidenz |
| ------------- | ------------- | --------- |
| 10            | 0.45          | 0.78      |
| 50            | 0.20          | 0.90      |
| 200           | 0.10          | 0.95      |
| 1000          | 0.05          | 0.98      |

---

### 5. Update-Algorithmus

#### 5.1 Event-basiertes Update

```rust
pub fn update_trust_vector(
    current: &TrustVector,
    event: &TrustEvent,
    config: &TrustConfig,
) -> TrustVector {
    let mut updated = current.clone();

    match event.event_type {
        EventType::TransactionClose { success } => {
            if success {
                updated.r = bayesian_update(current.r, 1.0, config.r_params);
            } else {
                updated.r = bayesian_update(current.r, 0.0, config.r_params);
            }
        }

        EventType::AttestationVerified { correct } => {
            if correct {
                updated.i = bayesian_update(current.i, 1.0, config.i_params);
            } else {
                updated.i = bayesian_update(current.i, 0.0, config.i_params);
            }
        }

        EventType::PeerReview { rating, reviewer_trust } => {
            updated.c = weighted_bayesian_update(
                current.c,
                rating,
                reviewer_trust,
                config.c_params
            );
        }

        EventType::BehaviorObserved { snapshot } => {
            updated.p = update_predictability(current.p, snapshot);
        }

        EventType::AnomalyReport { correct, severity } => {
            let weight = severity.weight();
            if correct {
                updated.v = bayesian_update(current.v, 1.0 * weight, config.v_params);
            } else {
                updated.v = bayesian_update(current.v, 0.0, config.v_params);
            }
        }

        EventType::PolicyAction { compliant, level } => {
            let weight = level.weight();
            if compliant {
                updated.omega = bayesian_update(current.omega, 1.0, config.omega_params);
            } else {
                updated.omega = bayesian_update(current.omega, 0.0, config.omega_params);
                // Zusätzlicher Penalty basierend auf Schwere
                updated.omega -= weight * config.violation_penalty;
                updated.omega = updated.omega.max(0.0);
            }
        }

        _ => {}
    }

    // Floor-Garantie (niemand fällt unter 0.3)
    updated.apply_floor(config.trust_floor);

    updated
}

fn bayesian_update(current: f64, observation: f64, params: &BayesParams) -> f64 {
    let alpha = current * params.pseudo_count;
    let beta = (1.0 - current) * params.pseudo_count;

    let new_alpha = alpha + observation * params.observation_weight;
    let new_beta = beta + (1.0 - observation) * params.observation_weight;

    new_alpha / (new_alpha + new_beta)
}
```

#### 5.2 Asymmetrie (Gain vs. Loss)

Vertrauen ist asymmetrisch: Verlust ist stärker als Gewinn.

```rust
const K_POS: f64 = 0.1;  // Gain-Faktor
const K_NEG: f64 = 0.4;  // Loss-Faktor (4x stärker)

fn asymmetric_update(current: f64, outcome: f64, significance: f64) -> f64 {
    if outcome > 0.0 {
        // Positives Event: Gain ist proportional zum "Headroom"
        let headroom = 1.0 - current;
        current + K_POS * significance * headroom
    } else {
        // Negatives Event: Loss ist proportional zum aktuellen Trust
        current - K_NEG * significance * current
    }
}
```

**Beispiel:**

```
Agent mit R = 0.8:
- Positives Event (sig=1): R = 0.8 + 0.1 × 1 × 0.2 = 0.82
- Negatives Event (sig=1): R = 0.8 - 0.4 × 1 × 0.8 = 0.48

Ein negatives Event zerstört Jahre positiver Arbeit.
```

#### 5.3 Trust-Decay

Inaktive Agenten verlieren langsam Trust:

```
W(t) = W(0) × λ^t
```

Mit λ = 0.9997 pro Tag (Halbwertszeit ≈ 6 Jahre).

```rust
fn apply_decay(trust: &mut TrustVector, days_inactive: u64) {
    let decay_factor = 0.9997_f64.powi(days_inactive as i32);

    trust.r = 0.5 + (trust.r - 0.5) * decay_factor;
    trust.i = 0.5 + (trust.i - 0.5) * decay_factor;
    trust.c = 0.5 + (trust.c - 0.5) * decay_factor;
    trust.p = 0.5 + (trust.p - 0.5) * decay_factor;
    trust.v = 0.5 + (trust.v - 0.5) * decay_factor;
    trust.omega = 0.5 + (trust.omega - 0.5) * decay_factor;
}
```

#### 5.4 Trust-Floor

Niemand fällt unter 0.3 (ermöglicht Rehabilitation):

```rust
impl TrustVector {
    pub fn apply_floor(&mut self, floor: f64) {
        self.r = self.r.max(floor);
        self.i = self.i.max(floor);
        self.c = self.c.max(floor);
        self.p = self.p.max(floor);
        self.v = self.v.max(floor);
        self.omega = self.omega.max(floor);
    }
}
```

---

### 6. Qualitative Interpretation

#### 6.1 Trust-Level

Der skalare Trust-Wert wird in qualitative Level übersetzt:

| Level     | Score-Range     | Beschreibung          |
| --------- | --------------- | --------------------- |
| Unknown   | Konfidenz < 0.5 | Nicht genug Daten     |
| Caution   | [0.0, 0.4)      | Vorsicht empfohlen    |
| Neutral   | [0.4, 0.6)      | Durchschnittlich      |
| Verified  | [0.6, 0.8)      | Gute Reputation       |
| HighTrust | [0.8, 1.0]      | Exzellente Reputation |

#### 6.2 Hysterese

Um Oszillationen an Schwellwerten zu verhindern:

```rust
fn determine_level(score: f64, previous_level: TrustLevel) -> TrustLevel {
    const HYSTERESIS: f64 = 0.05;

    let thresholds = match previous_level {
        TrustLevel::Caution => (0.0, 0.4 + HYSTERESIS),
        TrustLevel::Neutral => (0.4 - HYSTERESIS, 0.6 + HYSTERESIS),
        TrustLevel::Verified => (0.6 - HYSTERESIS, 0.8 + HYSTERESIS),
        TrustLevel::HighTrust => (0.8 - HYSTERESIS, 1.0),
        TrustLevel::Unknown => (0.0, 1.0),
    };

    if score < thresholds.0 {
        previous_level.downgrade()
    } else if score > thresholds.1 {
        previous_level.upgrade()
    } else {
        previous_level
    }
}
```

#### 6.3 Menschenlesbare Erklärung

```rust
pub fn explain_trust(vector: &TrustVector, confidence: &TrustConfidence) -> String {
    let mut explanations = vec![];

    if vector.r > 0.8 && confidence.r > 0.7 {
        explanations.push("Sehr zuverlässig bei der Erfüllung von Zusagen");
    } else if vector.r < 0.5 {
        explanations.push("Hat Schwierigkeiten, Zusagen einzuhalten");
    }

    if vector.i > 0.9 {
        explanations.push("Höchste Integrität – Aussagen werden stets verifiziert");
    } else if vector.i < 0.6 {
        explanations.push("Einige Aussagen konnten nicht verifiziert werden");
    }

    if vector.c > 0.8 {
        explanations.push("Überdurchschnittliche Kompetenz in seinem Bereich");
    }

    if vector.p < 0.5 {
        explanations.push("Unvorhersagbares Verhalten – Vorsicht empfohlen");
    }

    if vector.v > 0.7 {
        explanations.push("Aktiver Wächter – meldet Anomalien zuverlässig");
    }

    if vector.omega > 0.95 {
        explanations.push("Vorbildliche Regelkonformität");
    } else if vector.omega < 0.7 {
        explanations.push("Hat wiederholt gegen Regeln verstoßen");
    }

    explanations.join(". ")
}
```

---

### 7. Integration mit Systemgleichung

Der Trust-Vektor W(s) fließt in die Systemgleichung ein:

```
𝔼 = Σ A(s) · σ( W(s) · ln|C(s)| · N(s) / E(s) ) · H(s) · w(s,t)
       s
```

Dabei ist W(s) der skalare Trust-Wert:

```
W(s) = W_scalar(s) = Σ(wᵢ × Wᵢ(s))
```

---

### 8. Speicherung

#### 8.1 On-Chain (Commitment)

Auf der Blockchain wird nur ein Hash des Trust-Vektors gespeichert:

```rust
struct TrustCommitment {
    did: DID,
    vector_hash: [u8; 32],
    timestamp: u64,
    block_height: u64,
}

fn commit_trust(vector: &TrustVector) -> [u8; 32] {
    let serialized = bincode::serialize(vector).unwrap();
    sha256(&serialized)
}
```

#### 8.2 Off-Chain (Full Data)

Die vollständigen Daten werden im Semantic Index gespeichert:

```json
{
  "did": "did:erynoa:spirit:trading-bot-7",
  "trust_vector": {
    "R": { "value": 0.82, "alpha": 95, "beta": 15 },
    "I": { "value": 0.91, "alpha": 102, "beta": 10 },
    "C": { "value": 0.75, "alpha": 45, "beta": 15 },
    "P": { "value": 0.88, "variance": 0.012 },
    "V": { "value": 0.67, "alpha": 34, "beta": 16 },
    "Ω": { "value": 0.94, "alpha": 520, "beta": 35 }
  },
  "last_updated": "2026-01-29T14:30:00Z",
  "observation_history": [ ... ]
}
```

---

### 9. API

#### 9.1 Query Trust

```
GET /v1/trust/{did}
```

**Response:**

```json
{
  "did": "did:erynoa:spirit:trading-bot-7",
  "trust_vector": {
    "R": 0.82,
    "I": 0.91,
    "C": 0.75,
    "P": 0.88,
    "V": 0.67,
    "Ω": 0.94
  },
  "scalar": 0.847,
  "level": "HighTrust",
  "confidence": 0.89,
  "confidence_interval_95": [0.79, 0.9],
  "last_updated": "2026-01-29T14:30:00Z"
}
```

#### 9.2 Query Trust with Weights

```
POST /v1/trust/{did}/calculate
Content-Type: application/json

{
  "weights": {
    "R": 0.20,
    "I": 0.25,
    "C": 0.15,
    "P": 0.15,
    "V": 0.10,
    "Ω": 0.15
  }
}
```

---

### 10. SDK-Nutzung

#### 10.1 Rust

```rust
use erynoa_sdk::trust::{TrustVector, TrustWeights, TrustLevel};

// Trust abfragen
let did = DID::parse("did:erynoa:spirit:trading-bot-7")?;
let trust = client.get_trust(&did).await?;

println!("Trust-Vektor: {:?}", trust.vector);
println!("Skalar: {}", trust.scalar);
println!("Level: {:?}", trust.level);

// Mit Custom-Gewichten
let weights = TrustWeights {
    r: 0.20,
    i: 0.25,
    c: 0.15,
    p: 0.15,
    v: 0.10,
    omega: 0.15,
};

let custom_scalar = trust.vector.to_scalar(&weights);
println!("Custom Skalar: {}", custom_scalar);

// Erklärung generieren
let explanation = trust.explain();
println!("Erklärung: {}", explanation);
```

#### 10.2 TypeScript

```typescript
import { TrustVector, TrustWeights, TrustLevel } from "@erynoa/sdk";

// Trust abfragen
const did = "did:erynoa:spirit:trading-bot-7";
const trust = await client.getTrust(did);

console.log("Trust-Vektor:", trust.vector);
console.log("Skalar:", trust.scalar);
console.log("Level:", TrustLevel[trust.level]);

// Dimensionen einzeln prüfen
if (trust.vector.I > 0.9 && trust.confidence.I > 0.8) {
  console.log("Hohe Integrität mit hoher Konfidenz");
}

// Mit Realm-spezifischen Gewichten
const financeWeights: TrustWeights = {
  R: 0.2,
  I: 0.25,
  C: 0.15,
  P: 0.15,
  V: 0.1,
  Ω: 0.15,
};

const financeScore = trust.vector.toScalar(financeWeights);
```

#### 10.3 CLI

```bash
# Trust abfragen
erynoa trust did:erynoa:spirit:trading-bot-7

# Output:
# Trust Vector for did:erynoa:spirit:trading-bot-7
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# R (Reliability):     0.82  ████████░░  [High]
# I (Integrity):       0.91  █████████░  [Excellent]
# C (Competence):      0.75  ███████░░░  [Good]
# P (Predictability):  0.88  ████████░░  [High]
# V (Vigilance):       0.67  ██████░░░░  [Moderate]
# Ω (Omega-Alignment): 0.94  █████████░  [Excellent]
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Scalar (default weights): 0.847
# Level: HighTrust
# Confidence: 89%
# 95%-CI: [0.79, 0.90]

# Detaillierte Ansicht
erynoa trust did:erynoa:spirit:trading-bot-7 --detailed

# Mit Custom-Gewichten
erynoa trust did:erynoa:spirit:trading-bot-7 \
  --weights "R=0.2,I=0.25,C=0.15,P=0.15,V=0.1,Ω=0.15"

# Nur eine Dimension
erynoa trust did:erynoa:spirit:trading-bot-7 --dimension I

# Erklärung
erynoa trust did:erynoa:spirit:trading-bot-7 --explain
```

---

## Test-Vektoren

### TV-1: Neutral-Prior

**Input:** Neuer Agent, keine Events

**Expected:**

```json
{
  "R": 0.5,
  "I": 0.5,
  "C": 0.5,
  "P": 0.5,
  "V": 0.5,
  "Ω": 0.5,
  "scalar": 0.5,
  "level": "Neutral",
  "confidence": 0.2
}
```

### TV-2: Nach 100 erfolgreichen Transaktionen

**Input:** 100 TAT CLOSE (alle erfolgreich)

**Expected:**

```json
{
  "R": 0.962,
  "confidence_R": 0.91,
  "confidence_interval_95_R": [0.91, 0.99]
}
```

### TV-3: Nach einem schweren Integritätsverstoß

**Input:** 50 verifizierte Aussagen, dann 1 Betrugsfall (weight=20)

**Expected:**

```json
{
  "I_before": 0.963,
  "I_after": 0.714,
  "delta": -0.249
}
```

### TV-4: Staked Guardian Trust-Boost (V0.2)

**Input:**

- Neuer Agent (alle Dimensionen = 0.5)
- Guardian: did:erynoa:guild:sparkasse (T = 0.9)
- Stake: 500 ERY (stake_factor ≈ 0.3)

**Berechnung:**

```
boost = β × T_guardian × stake_factor
      = 0.3 × 0.9 × 0.3
      = 0.081

T_effective = T_base + boost
            = 0.5 + 0.081
            = 0.581
```

**Expected:**

```json
{
  "base_trust": 0.5,
  "guardian_boost": 0.081,
  "effective_trust": 0.581,
  "level": "Neutral → Verified"
}
```

### TV-5: Guardian Slashing (V0.2)

**Input:**

- Guardian bürgt für Scammer (Full Liability, 500 ERY)
- Scammer begeht Major Fraud (severity = 0.8)

**Berechnung:**

```
Δ_guardian_I = -liability × severity × stake_factor × 0.1
             = -1.0 × 0.8 × 0.3 × 0.1
             = -0.024
```

**Expected:**

```json
{
  "guardian_integrity_before": 0.9,
  "guardian_integrity_after": 0.876,
  "delta": -0.024
}
```

---

## Referenzen

- [Erynoa Fachkonzept V6.2](../FACHKONZEPT.md)
- [EIP-001: DID:erynoa](./EIP-001-did-erynoa.md)
- [EIP-004: Bayesian Trust Update](./EIP-004-bayesian-trust-update.md)
- [EigenTrust Paper](https://nlp.stanford.edu/pubs/eigentrust.pdf)
- [Bayesian Trust Models](https://www.cs.ox.ac.uk/people/audun.josang/trust/)

---

## Changelog

| Version | Datum      | Änderung                                                                                                                           |
| ------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| 0.1     | 2026-01-29 | Initial Draft                                                                                                                      |
| 0.2     | 2026-01-29 | **Staked Guardianship Integration**: Trust-Vererbung durch institutionelle Guardians, Slashing-Test-Vektoren, Referenz auf EIP-004 |

---

_EIP-002: Trust Vector 6D Specification_
_Version: 0.2_
_Status: Draft_
_Ebene: E1/E2 (Fundament/Emergenz)_
