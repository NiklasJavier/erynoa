# 💫 Trust-Gas-Mana Dreieinigkeit: Die Lebensenergie des Nervensystems

> **Teil von:** Projekt Pluto
> **Kategorie:** Kernphilosophie
> **Status:** Synthetisch abgestimmt

---

## 1. Fundamentales Verständnis: KEINE Token!

### 1.1 Was Trust, Gas und Mana NICHT sind

```text
❌ Trust ist KEIN fungible Token (nicht handelbar)
❌ Gas ist KEIN Coin (man kauft es nicht)
❌ Mana ist KEIN Prepaid-Guthaben (man ladet es nicht auf)

Sie sind KEINE wirtschaftlichen Assets!
```

### 1.2 Was sie WIRKLICH sind: System-Vitalzeichen

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                    DAS NERVENSYSTEM ATMET DURCH SIE                         │
│                                                                              │
│   🫀 TRUST  = Reputation / Immunsystem                                      │
│              → Bestimmt die "Gesundheit" einer Identity im System          │
│              → Entscheidet WER was tun darf                                 │
│              → Wächst langsam, fällt schnell (asymmetrisch, Κ4)            │
│                                                                              │
│   ⚡ GAS    = Rechenenergie / Muskelkraft                                   │
│              → Limitiert WIEVIEL Computation eine Aktion kostet            │
│              → Schützt vor DoS/Endlosschleifen                             │
│              → Verbraucht bei jeder Operation, nicht regenerierend         │
│                                                                              │
│   🌊 MANA   = Bandbreite / Ausdauer                                         │
│              → Limitiert WIE OFT jemand Aktionen ausführen darf            │
│              → Regeneriert über Zeit (wie Energie)                         │
│              → Basiert auf Trust – mehr Trust = mehr Kapazität             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Die Synergistische Kopplung

### 2.1 Die Dreiecks-Beziehung

```text
                         🫀 TRUST
                        ╱        ╲
                       ╱          ╲
              DependsOn            DependsOn
                     ╱              ╲
                    ╱                ╲
                  ⚡ GAS ◄─Triggers─► 🌊 MANA
                        ╲          ╱
                         ╲        ╱
                          Calibration
                              ▲
                              │
                         🛡️ Protection
```

### 2.2 Konkrete Abhängigkeiten (aus StateGraph)

Aus `state.rs` Zeilen 4158-4172:

```rust
// Gas ← Trust (DependsOn)
(Gas, DependsOn, Trust),      // Gas-Budget basiert auf Trust

// Mana ← Trust (DependsOn)
(Mana, DependsOn, Trust),     // Mana basiert auf Trust

// Execution ⊃ Gas (Aggregates)
(Execution, Aggregates, Gas), // Execution trackt Gas

// Execution ⊃ Mana (Aggregates)
(Execution, Aggregates, Mana),// Execution trackt Mana

// Calibration → Gas/Mana (Triggers)
(Calibration, Triggers, Gas), // Calibration passt Gas-Preise an
(Calibration, Triggers, Mana),// Calibration passt Mana-Regen an
```

---

## 3. Trust: Das Immunsystem

### 3.1 TrustVector6D – Die 6 Dimensionen

Aus `domain/unified/trust.rs`:

```rust
pub struct TrustVector6D {
    pub r: f32,     // R - Reliability (Verhaltens-Historie)
    pub i: f32,     // I - Integrity (Aussage-Konsistenz)
    pub c: f32,     // C - Competence (Fähigkeits-Nachweis)
    pub p: f32,     // P - Prestige (Externe Attestation)
    pub v: f32,     // V - Vigilance (Anomalie-Erkennung)
    pub omega: f32, // Ω - Omega (Axiom-Treue)
}
```

### 3.2 Vordefinierte Trust-Levels

```text
NEWCOMER = [0.1, 0.1, 0.1, 0.1, 0.1, 0.1]  → Sybil-Schutz
DEFAULT  = [0.5, 0.5, 0.5, 0.5, 0.5, 0.5]  → Etablierte Entität
MAX      = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0]  → Perfektes Trust
```

### 3.3 Asymmetrische Evolution (Κ4)

```rust
// Negative Updates wirken STÄRKER als positive!
fn update(&mut self, dim: TrustDimension, delta: f32) {
    let asymmetry = dim.asymmetry_factor(); // 1.5 oder 2.0

    let new_value = if delta < 0.0 {
        // Negative Updates stärker gewichtet
        (current + delta * asymmetry).clamp(0.0, 1.0)
    } else {
        // Positive Updates normal
        (current + delta).clamp(0.0, 1.0)
    };
}
```

**Mathematik:**
```text
Κ4: Δ⁻(dim) = λ_asym · Δ⁺(dim)

Für R, I, C, P: λ_asym = 1.5
Für V, Ω:      λ_asym = 2.0

→ Vertrauen ist schwer zu gewinnen, leicht zu verlieren!
```

### 3.4 Trust → Mana Formel

Aus `eclvm/mana.rs`:

```rust
fn calculate_max_mana(reliability: f64, config: &ManaConfig) -> u64 {
    let multiplier = 1.0 + (reliability * config.max_multiplier);
    (config.base_allowance as f64 * multiplier) as u64
}

// Default-Werte:
// base_allowance: 10_000
// max_multiplier: 100.0
```

**Beispiele:**
```text
Trust 0.0 → Mana 10.000   (Basis)
Trust 0.1 → Mana 110.000  (11x)
Trust 0.5 → Mana 510.000  (51x)
Trust 0.9 → Mana 910.000  (91x)
Trust 1.0 → Mana 1.010.000 (101x)
```

---

## 4. Gas: Die Rechenenergie

### 4.1 GasMeter – Compute-Limitierung

Aus `eclvm/runtime/gas.rs`:

```rust
pub struct GasMeter {
    remaining: u64,    // Verbleibendes Gas
    limit: u64,        // Ursprüngliches Limit
    consumed: u64,     // Verbrauchtes Gas
}

impl GasMeter {
    pub fn consume(&mut self, amount: u64) -> Result<()> {
        if amount > self.remaining {
            return Err("Out of gas");
        }
        self.remaining -= amount;
        self.consumed += amount;
        Ok(())
    }
}
```

### 4.2 Was kostet Gas?

Aus `domain/unified/cost.rs` CostTable:

```text
ECLVM OpCodes:
├── push/const          → 1 Gas
├── add/sub             → 2 Gas
├── mul                 → 3 Gas
├── div/mod             → 5 Gas
├── call (base)         → 10 Gas
├── call (per arg)      → +2 Gas
├── host_call           → 50 Gas + 10 Mana
├── branch              → 3 Gas
├── load                → 5 Gas
└── store               → 10 Gas

Storage:
├── storage_get         → 5 Mana
├── storage_put (base)  → 10 Mana
└── storage_put (/KB)   → +10 Mana

P2P:
├── p2p_publish         → 10 Mana
├── p2p_connect         → 20 Mana + 0.1 trust_risk
└── p2p_dht_put         → 20 Mana
```

### 4.3 Gas vs. Mana Zuordnung

```text
┌────────────────────────────────────────────────────────────────┐
│                    KOSTEN-ALGEBRA κ                            │
│                                                                │
│   κ = (gas, mana, trust_risk)                                 │
│                                                                │
│   ⚡ GAS   → ECLVM Compute (CPU-intensive)                    │
│   🌊 MANA  → Storage + P2P (I/O, Network)                     │
│   🛡️ RISK  → Vertrauens-Risiko der Operation                  │
│                                                                │
│   Sequentiell: κ₁ ⊕ κ₂ = (g₁+g₂, m₁+m₂, 1-(1-t₁)(1-t₂))     │
│   Parallel:    κ₁ ⊗ κ₂ = (max(g₁,g₂), m₁+m₂, max(t₁,t₂))    │
└────────────────────────────────────────────────────────────────┘
```

---

## 5. Mana: Die Ausdauer

### 5.1 ManaAccount – Per-Identity Bandbreite

Aus `eclvm/mana.rs`:

```rust
pub struct ManaAccount {
    current: u64,         // Aktuelles Guthaben
    max: u64,             // Maximum (basierend auf Trust)
    regen_rate: u64,      // Regeneration pro Sekunde
    last_update: Instant, // Letztes Update
    trust_snapshot: f32,  // Trust bei letzter Berechnung
}
```

### 5.2 Regeneration-Formel

```rust
fn calculate_regen_rate(reliability: f64, config: &ManaConfig) -> u64 {
    let multiplier = 1.0 + (reliability * config.regen_trust_factor);
    (config.base_regen_per_sec as f64 * multiplier) as u64
}

// Default:
// base_regen_per_sec: 100
// regen_trust_factor: 10.0
```

**Beispiele:**
```text
Trust 0.0 → Regen 100/sec  (1x)
Trust 0.5 → Regen 600/sec  (6x)
Trust 1.0 → Regen 1.100/sec (11x)
```

### 5.3 Sybil-Schutz durch Mana

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                         SYBIL-ATTACKE BEISPIEL                              │
│                                                                              │
│   Angreifer: 10 Fake-Accounts mit Trust 0.0                                │
│                                                                              │
│   Jeder Account: max_mana = 10.000                                          │
│   Gesamt:        10 × 10.000 = 100.000 Mana                                │
│                                                                              │
│   Legitimer User mit Trust 0.8:                                             │
│   max_mana = 10.000 × (1 + 0.8 × 100) = 810.000 Mana                       │
│                                                                              │
│   → Ein legitimer User hat 8x mehr Mana als ALLE 10 Sybils zusammen!       │
│   → Nach wenigen Aktionen ist Sybil-Mana leer                              │
│   → Regeneration langsam → Spam wird unökonomisch                          │
│   → Bei Spam: Trust sinkt → Mana sinkt → negative Feedback-Loop            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Die Nahtlose Nervensystem-Integration

### 6.1 Flow: Request → Response

```text
User-Request kommt an
        │
        ▼
┌───────────────────┐
│ 1. Trust-Check    │  ← TrustState: Ist der User bekannt?
│    Identity laden │    Welchen Trust-Level hat er?
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 2. Mana-Preflight │  ← ManaManager.preflight_check()
│    Check          │    Hat der User genug Mana für die Operation?
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 3. Gas-Budget     │  ← Budget basierend auf Trust:
│    berechnen      │    higher_trust → higher_gas_budget
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 4. ECLVM          │  ← GasMeter trackt Verbrauch
│    Execution      │    Bei Out-of-Gas: Abort
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 5. Mana-Deduct    │  ← ManaManager.deduct()
│    (bei Erfolg)   │    Tatsächlicher Verbrauch abziehen
└────────┬──────────┘
         │
         ▼
┌───────────────────┐
│ 6. Trust-Update   │  ← Erfolg = +Trust, Fehler = −Trust
│    (Event)        │    Asymmetrisch (Κ4)
└───────────────────┘
```

### 6.2 StateEvents für Trust-Gas-Mana

```rust
// Trust-Update mit Audit-Trail
StateEvent::TrustUpdate {
    entity_id: String,
    delta: f64,
    reason: TrustReason,
    from_realm: Option<String>,
    triggered_events: u64,
    new_trust: f64,
}

// Execution mit Gas/Mana Tracking
StateEvent::ExecutionCompleted {
    context_id: String,
    success: bool,
    gas_consumed: u64,
    mana_consumed: u64,
    events_emitted: u64,
    duration_ms: u64,
    error: Option<String>,
}

// System-Modus bei kritischen Anomalien (CircuitBreaker)
StateEvent::SystemModeChanged {
    old_mode: SystemMode,
    new_mode: SystemMode,  // Normal → Degraded → Emergency
    trigger_event_id: String,
    automatic: bool,
}
```

### 6.3 GasState und ManaState im UnifiedState

```rust
pub struct ExecutionState {
    pub gas: GasState,     // Relationship-Tracking für Calibration
    pub mana: ManaState,   // Relationship-Tracking für Trust
    pub executions: ExecutionsState,
}

pub struct GasState {
    pub consumed: AtomicU64,
    pub refunded: AtomicU64,
    pub out_of_gas_count: AtomicU64,
    pub current_price: RwLock<f64>,

    // Relationship-Tracking
    pub calibration_adjustments: AtomicU64,  // Calibration → Gas
    pub trust_dependency_updates: AtomicU64, // Gas ← Trust
}

pub struct ManaState {
    pub consumed: AtomicU64,
    pub regenerated: AtomicU64,
    pub rate_limited_count: AtomicU64,
    pub regen_rate: RwLock<f64>,

    // Relationship-Tracking
    pub calibration_adjustments: AtomicU64,  // Calibration → Mana
    pub trust_dependency_updates: AtomicU64, // Mana ← Trust
}
```

---

## 7. Protection-Layer: Das Immunsystem

### 7.1 CircuitBreaker: Automatische Degradation

```rust
pub struct CircuitBreaker {
    mode: AtomicU8,  // Normal, Degraded, EmergencyShutdown

    // Κ19: Anti-Calcification
    pub gini_threshold: RwLock<f64>,  // Default: 0.8

    // Automatische Modus-Wechsel
    pub degraded_threshold: AtomicU64,    // Default: 10 Anomalien/min
    pub emergency_threshold: AtomicU64,   // Default: 50 Anomalien/min
}
```

### 7.2 Modus → Erlaubte Aktionen

```text
NORMAL:
├── ECLVM Execution:   ✅
├── Crossings:         ✅
├── P2P:               ✅
└── Full Trust-Updates ✅

DEGRADED:
├── ECLVM Execution:   ❌ (keine neuen)
├── Crossings:         ❌
├── P2P:               ✅ (read-only)
└── Trust-Updates:     ⚠️ (nur negativ)

EMERGENCY_SHUTDOWN:
├── ECLVM Execution:   ❌
├── Crossings:         ❌
├── P2P:               ❌
└── Trust-Updates:     ❌
```

---

## 8. Pluto-Integration

### 8.1 Aktualisierte Ziel-Architektur

```text
backend/src/nervous_system/
│
├── unified_cost/                    # Cost-Algebra κ
│   ├── mod.rs                       # Cost, Budget, CostTable
│   ├── cost.rs                      # κ = (gas, mana, risk)
│   └── budget.rs                    # Intent-Budgets
│
├── gas/                             # ⚡ Gas-Layer
│   ├── mod.rs                       # GasState
│   ├── meter.rs                     # GasMeter (ECLVM)
│   └── pricing.rs                   # Dynamische Preise
│
├── mana/                            # 🌊 Mana-Layer
│   ├── mod.rs                       # ManaState
│   ├── account.rs                   # ManaAccount
│   ├── manager.rs                   # ManaManager
│   └── bandwidth_tier.rs            # BandwidthTier enum
│
├── trust/                           # 🫀 Trust-Layer
│   ├── mod.rs                       # TrustState
│   ├── vector6d.rs                  # TrustVector6D
│   ├── record.rs                    # TrustRecord + History
│   ├── combination.rs               # Κ5 probabilistische Kombination
│   ├── dampening.rs                 # Κ24 Realm-Crossing Dämpfung
│   └── context.rs                   # ContextType-Gewichtung
│
└── protection/                      # 🛡️ Protection-Layer
    ├── mod.rs
    ├── circuit_breaker.rs           # CircuitBreaker
    ├── calibration.rs               # Self-Healing
    └── anti_calc.rs                 # Κ19 Gini-Monitoring
```

### 8.2 Aktualisierte StateGraph-Relationen

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                   TRUST-GAS-MANA BEZIEHUNGEN                                │
│                                                                              │
│   Trust ──────────────────────────────────────────────────────────────────┐ │
│     │                                                                     │ │
│     ├── DependsOn ──► Identity      (Trust basiert auf Identity)         │ │
│     ├── Triggers ───► Event         (Trust-Updates erzeugen Events)      │ │
│     ├── DependsOn ──► WorldFormula  (Trust fließt in 𝔼)                   │ │
│     └──────────────────────────────────────────►──────────────────────────┘ │
│                                     │                                        │
│   Gas ◄──────── DependsOn ◄─────────┤                                        │
│     │                               │                                        │
│     ├── DependsOn ◄─ Calibration    (Calibration passt Gas an)             │
│     ├── Aggregates ◄─ Execution     (Execution trackt Gas)                 │
│     └── DependsOn ◄─ ECLVM          (ECLVM verbraucht Gas)                 │
│                                     │                                        │
│   Mana ◄─────── DependsOn ◄─────────┘                                        │
│     │                                                                        │
│     ├── DependsOn ◄─ Calibration    (Calibration passt Mana an)            │
│     ├── Aggregates ◄─ Execution     (Execution trackt Mana)                │
│     └── DependsOn ◄─ ECLVM          (ECLVM verbraucht Mana)                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 8.3 Invarianten-Checkliste

| Invariante | Beschreibung | Implementierung |
|------------|--------------|-----------------|
| **Κ2** | Trust ∈ [0, 1] | `value.clamp(0.0, 1.0)` |
| **Κ4** | Δ⁻ = λ · Δ⁺ (Asymmetrie) | `dim.asymmetry_factor()` |
| **Κ5** | t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂) | `TrustCombination::combine()` |
| **Κ8** | Delegation Trust-Decay | `TrustEntry.decay_factor` |
| **Κ11** | Gas monoton erschöpfend | `GasMeter.consume()` bricht ab |
| **Κ13** | Mana regeneriert positiv | `ManaAccount.update()` |
| **Κ19** | Gini < 0.8 | `CircuitBreaker.check_gini()` |

---

## 9. Fazit: Die Lebensenergie

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   Trust, Gas und Mana sind die LEBENSENERGIE des Erynoa-Nervensystems.     │
│                                                                              │
│   Sie sind:                                                                  │
│   ✓ Intrinsisch verbunden (nicht isolierbar)                               │
│   ✓ Gegenseitig abhängig (Trust → Mana → Execution)                        │
│   ✓ Selbstregulierend (Calibration, CircuitBreaker)                        │
│   ✓ Sybil-resistent (Trust-basierte Kapazitäten)                           │
│   ✓ Asymmetrisch fair (leichter zu verlieren als zu gewinnen)              │
│                                                                              │
│   Ohne Trust gibt es keine Mana-Kapazität.                                  │
│   Ohne Mana kann keine Aktion gestartet werden.                             │
│   Ohne Gas kann keine Computation abgeschlossen werden.                     │
│                                                                              │
│   Das Nervensystem atmet nur, wenn alle drei im Gleichgewicht sind.        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```
