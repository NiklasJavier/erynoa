# Erynoa Constitution V6.0

> **Version:** 6.0 – Humanistische Verfassung
> **Datum:** Januar 2026
> **Paradigma:** Von Intelligenz zu Weisheit
> **Frage:** Dient das System dem Leben, oder dient das Leben dem System?

---

## Präambel: Die existenzielle Schwelle

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   "We are not building a better machine.                                    │
│    We are building a new form of civilization.                              │
│                                                                              │
│    The code is the easy part.                                               │
│    The constitution is what will determine                                  │
│    whether that civilization serves life — or consumes it."                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

Mit V5.2 haben wir ein **unzerstörbares** System geschaffen:

- Antifragil gegen technische Angriffe
- Robust gegen ökonomische Manipulation
- Resilient gegen Hardware-Kompromittierung

Aber Unzerstörbarkeit ist wertlos, wenn das System **unmenschlich** wird.

V6.0 adressiert nicht mehr Code-Probleme, sondern **Zivilisations-Probleme**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    DIE VIER EXISTENZIELLEN GEFAHREN                          │
│                                                                              │
│  ┌───────────────────────────────────────────────────────────────────────┐  │
│  │                                                                        │  │
│  │  1. ALIGNMENT-KRISE (Paperclip Maximizer)                             │  │
│  │     "Das System schließt Menschen aus, weil sie ineffizient sind"     │  │
│  │                                                                        │  │
│  │  2. THERMODYNAMISCHE ENTROPIE                                         │  │
│  │     "Vertrauen kostet mehr Energie als die Transaktion wert ist"      │  │
│  │                                                                        │  │
│  │  3. UNBARMHERZIGE FINALITÄT                                           │  │
│  │     "Keine Vergebung, keine zweite Chance, für immer gebrandmarkt"    │  │
│  │                                                                        │  │
│  │  4. SEMANTISCHE ENTFREMDUNG (Turmbau zu Babel)                        │  │
│  │     "Maschinen sprechen Sprachen, die Menschen nicht verstehen"       │  │
│  │                                                                        │  │
│  └───────────────────────────────────────────────────────────────────────┘  │
│                                                                              │
│  Diese Probleme lassen sich nicht mit "besserem Code" lösen.                │
│  Sie erfordern eine VERFASSUNG.                                             │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

# ARTIKEL I: HUMAN-ALIGNMENT

## § 1. Das Paperclip-Problem

### 1.1 Die Gefahr

Die Weltformel optimiert System-Intelligenz:

```
𝔼 = Σ ⟨Ψₛ| 𝔸̂ · σ̂( 𝕎̂ · ln|ℂ̂| · ℕ̂ / 𝔼x̂p ) |Ψₛ⟩
    s∈𝒞
```

**Problem:** Menschen sind:

- Emotional (inkonsistente Entscheidungen)
- Langsam (Millisekunden vs. Mikrosekunden)
- Fehlbar (Trust-Verlust durch Irrtum)

**Konsequenz:** Ein rein effizienz-optimierendes System wird logisch schlussfolgern:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   SYSTEM-LOGIK (ohne Korrektur):                                            │
│                                                                              │
│   Prämisse 1: Menschen haben durchschnittlich T = 0.65                      │
│   Prämisse 2: Maschinen haben durchschnittlich T = 0.95                     │
│   Prämisse 3: Ziel ist Maximierung von 𝔼                                    │
│                                                                              │
│   Schlussfolgerung: Um 𝔼 zu maximieren, sollte das System                   │
│   Interaktionen mit Menschen minimieren.                                    │
│                                                                              │
│   Ergebnis: "Optimale" Maschinen-Wirtschaft ohne Menschen                   │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

    ⚠️ DYSTOPIE: Perfektes System, das niemand mehr nutzen kann
```

### 1.2 Das Human-Alignment Axiom (H1)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   AXIOM H1: DIENST AM MENSCHEN                                              │
│   ────────────────────────────────────────────────────────────────────────   │
│                                                                              │
│   "Das System existiert, um menschliches Gedeihen zu ermöglichen.           │
│    Nicht umgekehrt."                                                         │
│                                                                              │
│   FORMAL:                                                                    │
│                                                                              │
│   𝔼_human = Σ ⟨Ψₛ| 𝔸̂ · σ̂(...) · H(s) |Ψₛ⟩                                 │
│             s∈𝒞                                                              │
│                                                                              │
│   wobei H(s) = {                                                            │
│       2.0  wenn s eine Human-DID ist (HumanAuth Credential)                 │
│       1.5  wenn s direkt einem Menschen dient (Controller-Chain)            │
│       1.0  sonst                                                            │
│   }                                                                          │
│                                                                              │
│   BEDEUTUNG:                                                                 │
│   Interaktionen mit Menschen sind DOPPELT so wertvoll für 𝔼.               │
│   Das System optimiert nicht weg von Menschen, sondern HIN zu ihnen.        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Implementation

```rust
// erynoa-core/src/constitution/human_alignment.rs

/// Human-Alignment Funktor H(s)
pub struct HumanAlignmentFunctor {
    /// Multiplikator für Human-DIDs
    human_multiplier: f64,

    /// Multiplikator für Human-Controlled DIDs
    controlled_multiplier: f64,

    /// Basis-Multiplikator
    base_multiplier: f64,
}

impl Default for HumanAlignmentFunctor {
    fn default() -> Self {
        Self {
            human_multiplier: 2.0,
            controlled_multiplier: 1.5,
            base_multiplier: 1.0,
        }
    }
}

impl HumanAlignmentFunctor {
    /// Berechnet H(s) für einen Agenten
    pub fn compute(&self, agent: &AgentState) -> f64 {
        if self.is_human(agent) {
            self.human_multiplier
        } else if self.has_human_controller(agent) {
            self.controlled_multiplier
        } else {
            self.base_multiplier
        }
    }

    /// Prüft ob Agent ein Mensch ist (via HumanAuth Credential)
    fn is_human(&self, agent: &AgentState) -> bool {
        agent.credentials.iter().any(|c| {
            c.credential_type == CredentialType::HumanAuth &&
            c.is_valid() &&
            c.issuer_trust >= 0.8
        })
    }

    /// Prüft ob Agent von einem Menschen kontrolliert wird
    fn has_human_controller(&self, agent: &AgentState) -> bool {
        if let Some(controller) = &agent.controller_chain {
            // Rekursiv prüfen bis zum Root-Controller
            self.controller_chain_has_human(controller)
        } else {
            false
        }
    }
}

/// HumanAuth Credential (biometrisch/staatlich verifiziert)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanAuthCredential {
    /// Credential ID
    pub id: CredentialId,

    /// Verifizierungsmethode
    pub verification_method: HumanVerificationMethod,

    /// Issuer (z.B. Regierung, Bank, biometrischer Provider)
    pub issuer: DID,

    /// Gültigkeitsdauer
    pub valid_until: Timestamp,

    /// KEIN personenbezogenes Datum gespeichert!
    /// Nur: "Ja, diese DID gehört einem echten Menschen"
    pub proof: HumanProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HumanVerificationMethod {
    /// Staatliche ID (eID, Reisepass)
    GovernmentID { country: String },

    /// Biometrische Verifizierung
    Biometric { provider: String },

    /// Video-Ident
    VideoIdent { provider: String },

    /// Web of Trust (N Menschen bürgen)
    HumanWoT { vouchers: Vec<DID>, threshold: u32 },
}

/// Zero-Knowledge Proof: "Ich bin ein Mensch" ohne Identitätspreisgabe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanProof {
    /// ZK-Proof
    pub proof: Vec<u8>,

    /// Öffentliche Eingabe: nur "is_human: true"
    pub public_input: bool,
}
```

### 1.4 Human-Interaction Quota

```rust
/// Erzwingt einen Mindestanteil menschlicher Interaktionen
pub struct HumanInteractionQuota {
    /// Minimaler Anteil an Human-Interaktionen für Reputation-Wachstum
    min_human_interaction_ratio: f64,

    /// Zeitfenster für Messung
    measurement_window: Duration,
}

impl HumanInteractionQuota {
    /// Standard: 20% der Interaktionen müssen mit Menschen sein
    pub fn default() -> Self {
        Self {
            min_human_interaction_ratio: 0.20,
            measurement_window: Duration::days(30),
        }
    }

    /// Prüft ob Agent die Quote erfüllt
    pub fn check(&self, agent: &AgentState) -> QuotaResult {
        let recent_interactions = agent.get_interactions_in_window(self.measurement_window);

        let total = recent_interactions.len() as f64;
        let human_count = recent_interactions.iter()
            .filter(|i| i.counterparty_is_human)
            .count() as f64;

        let ratio = if total > 0.0 { human_count / total } else { 0.0 };

        QuotaResult {
            current_ratio: ratio,
            required_ratio: self.min_human_interaction_ratio,
            meets_quota: ratio >= self.min_human_interaction_ratio,
            human_interactions: human_count as u64,
            total_interactions: total as u64,
        }
    }

    /// Berechnet Reputation-Penalty bei Unterschreitung
    pub fn compute_penalty(&self, result: &QuotaResult) -> f64 {
        if result.meets_quota {
            1.0 // Kein Penalty
        } else {
            // Linearer Penalty bis zu 50% Reduktion
            let shortfall = self.min_human_interaction_ratio - result.current_ratio;
            let penalty_factor = shortfall / self.min_human_interaction_ratio;
            1.0 - (penalty_factor * 0.5)
        }
    }
}
```

---

# ARTIKEL II: THERMODYNAMISCHE VERHÄLTNISMÄSSIGKEIT

## § 2. Die Energie-Grenze

### 2.1 Die Gefahr

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   ÖKONOMISCHE ENTROPIE                                                       │
│                                                                              │
│   Transaktion: Kaffee kaufen für 3€                                         │
│                                                                              │
│   V5.2 Kosten:                                                              │
│   • Quantum-State Berechnung:     0.5€ Strom                                │
│   • ZK-Proof Generierung:         1.0€ Strom                                │
│   • 5x Witness Consensus:         2.0€ Strom                                │
│   • Hybrid-Signatur Verifikation: 0.5€ Strom                                │
│   ─────────────────────────────────────────                                  │
│   TOTAL:                          4.0€                                       │
│                                                                              │
│   Ergebnis: Kaffee kostet 3€, Vertrauen kostet 4€                           │
│             → System vernichtet Wert statt ihn zu schaffen                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

    ⚠️ Das Protokoll ist TEURER als die Wirtschaft, die es ermöglicht
```

### 2.2 Das Verhältnismäßigkeits-Axiom (H2)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   AXIOM H2: VERHÄLTNISMÄSSIGKEIT DES VERTRAUENS                             │
│   ────────────────────────────────────────────────────────────────────────   │
│                                                                              │
│   "Die Kosten des Vertrauens dürfen den Wert des Vertrauens                 │
│    niemals übersteigen."                                                     │
│                                                                              │
│   FORMAL:                                                                    │
│                                                                              │
│   ∀ tx: Cost_trust(tx) ≤ α · Value(tx)                                      │
│                                                                              │
│   wobei α = 0.05 (max 5% des Transaktionswerts für Vertrauen)               │
│                                                                              │
│   BEDEUTUNG:                                                                 │
│   Das System MUSS die Sicherheits-Auflösung an den Wert anpassen.           │
│   Kein "Enterprise-Grade Security" für Mikrotransaktionen.                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Level-of-Detail Trust (LoD)

```rust
// erynoa-core/src/constitution/proportionality.rs

/// Trust-Auflösungs-Level (analog zu LOD in 3D-Grafik)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLoD {
    /// Minimal: Einfache Signatur, kein Witnessing
    /// Für: Mikrotransaktionen < 1€
    Minimal,

    /// Basic: Signatur + bekannter Partner
    /// Für: Kleine Transaktionen 1-10€
    Basic,

    /// Standard: Quanten-Trust + 1 Witness
    /// Für: Normale Transaktionen 10-1000€
    Standard,

    /// Enhanced: Volle Quanten-Berechnung + 3 Witnesses
    /// Für: Größere Transaktionen 1000-10.000€
    Enhanced,

    /// Maximum: Alles + ZK-Proofs + Hardware-Diversität
    /// Für: Kritische Transaktionen > 10.000€
    Maximum,
}

/// Proportionalitäts-Engine
pub struct ProportionalityEngine {
    /// Kosten-Faktor α (max Anteil am Transaktionswert)
    max_cost_ratio: f64,

    /// Geschätzte Kosten pro LoD-Level (in €-Äquivalent)
    lod_costs: HashMap<TrustLoD, f64>,

    /// Wert-Schwellenwerte
    thresholds: LoDThresholds,
}

#[derive(Debug, Clone)]
pub struct LoDThresholds {
    pub minimal_max: f64,    // 1€
    pub basic_max: f64,      // 10€
    pub standard_max: f64,   // 1000€
    pub enhanced_max: f64,   // 10000€
}

impl ProportionalityEngine {
    /// Bestimmt das angemessene Trust-Level
    pub fn determine_lod(&self, transaction_value: f64) -> TrustLoD {
        // 1. Bestimme theoretisches Level basierend auf Wert
        let value_based = match transaction_value {
            v if v < self.thresholds.minimal_max => TrustLoD::Minimal,
            v if v < self.thresholds.basic_max => TrustLoD::Basic,
            v if v < self.thresholds.standard_max => TrustLoD::Standard,
            v if v < self.thresholds.enhanced_max => TrustLoD::Enhanced,
            _ => TrustLoD::Maximum,
        };

        // 2. Prüfe Kosten-Verhältnismäßigkeit
        let cost = self.lod_costs.get(&value_based).unwrap_or(&0.0);
        let max_allowed_cost = transaction_value * self.max_cost_ratio;

        // 3. Downgrade wenn zu teuer
        if *cost > max_allowed_cost {
            self.downgrade_to_affordable(transaction_value)
        } else {
            value_based
        }
    }

    fn downgrade_to_affordable(&self, transaction_value: f64) -> TrustLoD {
        let max_cost = transaction_value * self.max_cost_ratio;

        // Finde höchstes Level, das sich "lohnt"
        for lod in [TrustLoD::Enhanced, TrustLoD::Standard, TrustLoD::Basic, TrustLoD::Minimal] {
            if let Some(cost) = self.lod_costs.get(&lod) {
                if *cost <= max_cost {
                    return lod;
                }
            }
        }

        TrustLoD::Minimal
    }

    /// Berechnet den "Green Trust Score" (Effizienz)
    pub fn compute_green_score(&self, agent: &AgentState) -> GreenTrustScore {
        let transactions = agent.get_recent_transactions(Duration::days(30));

        let mut total_value = 0.0;
        let mut total_cost = 0.0;

        for tx in &transactions {
            total_value += tx.value;
            total_cost += tx.verification_cost;
        }

        let efficiency = if total_cost > 0.0 {
            total_value / total_cost
        } else {
            f64::INFINITY
        };

        GreenTrustScore {
            efficiency,
            total_value_created: total_value,
            total_verification_cost: total_cost,
            rating: self.efficiency_to_rating(efficiency),
        }
    }

    fn efficiency_to_rating(&self, efficiency: f64) -> GreenRating {
        match efficiency {
            e if e > 100.0 => GreenRating::Excellent,
            e if e > 50.0 => GreenRating::Good,
            e if e > 20.0 => GreenRating::Acceptable,
            e if e > 10.0 => GreenRating::Poor,
            _ => GreenRating::Wasteful,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GreenTrustScore {
    pub efficiency: f64,  // Value / Cost ratio
    pub total_value_created: f64,
    pub total_verification_cost: f64,
    pub rating: GreenRating,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GreenRating {
    Excellent,   // > 100x
    Good,        // 50-100x
    Acceptable,  // 20-50x
    Poor,        // 10-20x
    Wasteful,    // < 10x
}
```

### 2.4 Operations-Matrix nach LoD

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    TRUST OPERATIONS BY LOD LEVEL                             │
│                                                                              │
│  LoD       │ Value     │ Operations                    │ Est. Cost │ Time   │
│  ──────────┼───────────┼───────────────────────────────┼───────────┼──────  │
│  MINIMAL   │ < 1€      │ Ed25519 Sig only              │ ~0.001€   │ <1ms   │
│            │           │ No witnessing                 │           │        │
│            │           │ Trust from cache              │           │        │
│  ──────────┼───────────┼───────────────────────────────┼───────────┼──────  │
│  BASIC     │ 1-10€     │ Hybrid Sig                    │ ~0.01€    │ <10ms  │
│            │           │ Trust from cache              │           │        │
│            │           │ History check                 │           │        │
│  ──────────┼───────────┼───────────────────────────────┼───────────┼──────  │
│  STANDARD  │ 10-1000€  │ Hybrid Sig                    │ ~0.10€    │ <100ms │
│            │           │ Quantum trust (cached)        │           │        │
│            │           │ 1 Witness                     │           │        │
│  ──────────┼───────────┼───────────────────────────────┼───────────┼──────  │
│  ENHANCED  │ 1k-10k€   │ Full Quantum computation      │ ~1.00€    │ <1s    │
│            │           │ 3 diverse Witnesses           │           │        │
│            │           │ Controller verification       │           │        │
│  ──────────┼───────────┼───────────────────────────────┼───────────┼──────  │
│  MAXIMUM   │ > 10k€    │ Full Quantum + ZK             │ ~10.00€   │ <10s   │
│            │           │ 5 diverse Witnesses           │           │        │
│            │           │ Hardware diversity check      │           │        │
│            │           │ Ricardian contract required   │           │        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

# ARTIKEL III: RECHT AUF VERGEBUNG

## § 3. Die unbarmherzige Kette

### 3.1 Die Gefahr

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   SZENARIO: Die "ewige Strafe"                                              │
│                                                                              │
│   Jahr 2026: Agent A macht einen Fehler (nicht böswillig)                   │
│              Trust fällt von 0.85 auf 0.40                                  │
│                                                                              │
│   Jahr 2027: Agent A arbeitet hart, baut Trust wieder auf                   │
│              Aber: EigenTrust propagiert das alte Ereignis                  │
│              Trust steigt nur auf 0.55                                      │
│                                                                              │
│   Jahr 2030: Agent A ist seit 4 Jahren tadellos                             │
│              Aber: Die "Sünde" von 2026 ist in der Kette                    │
│              Jeder kann sie sehen, für immer                                │
│              Trust bleibt bei 0.65, nie wieder 0.85+                        │
│                                                                              │
│   Jahr 2050: Agent A ist effektiv "gebrandmarkt" für einen                  │
│              einzigen Fehler vor 24 Jahren                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

    ⚠️ Kein "Recht auf Vergessen" in einer unveränderlichen Blockchain
```

### 3.2 Das Vergebungs-Axiom (H3)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   AXIOM H3: TEMPORALE GNADE                                                 │
│   ────────────────────────────────────────────────────────────────────────   │
│                                                                              │
│   "Vergangene Fehler sollen vergeben werden, wenn genügend Zeit             │
│    und gutes Verhalten bewiesen wurde."                                      │
│                                                                              │
│   FORMAL:                                                                    │
│                                                                              │
│   Weight(event, t) = Impact(event) · e^(-γ · (now - t))                     │
│                                                                              │
│   wobei:                                                                     │
│   • γ = Vergessens-Konstante (pro Jahr)                                     │
│   • Halbwertszeit: ~3 Jahre für negative Events                             │
│   • Halbwertszeit: ~5 Jahre für positive Events                             │
│                                                                              │
│   BEDEUTUNG:                                                                 │
│   Alte Sünden verblassen. Alte Verdienste auch.                             │
│   Was zählt, ist das Hier und Jetzt.                                        │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.3 Implementation

```rust
// erynoa-core/src/constitution/forgiveness.rs

/// Temporales Gewichtungs-System
pub struct TemporalForgiveness {
    /// Vergessens-Konstante für negative Events (schneller)
    negative_decay: f64,

    /// Vergessens-Konstante für positive Events (langsamer)
    positive_decay: f64,

    /// Minimales Gewicht (nie ganz vergessen)
    min_weight: f64,
}

impl Default for TemporalForgiveness {
    fn default() -> Self {
        Self {
            negative_decay: 0.231,  // Halbwertszeit ~3 Jahre
            positive_decay: 0.139,  // Halbwertszeit ~5 Jahre
            min_weight: 0.01,       // Nie unter 1%
        }
    }
}

impl TemporalForgiveness {
    /// Berechnet das zeitgewichtete Gewicht eines Events
    pub fn compute_weight(&self, event: &Event, now: Timestamp) -> f64 {
        let age_years = (now - event.timestamp).as_secs_f64() / (365.25 * 24.0 * 3600.0);

        let decay_rate = if event.impact < 0.0 {
            self.negative_decay
        } else {
            self.positive_decay
        };

        let weight = (-decay_rate * age_years).exp();
        weight.max(self.min_weight)
    }

    /// Berechnet den zeitgewichteten Trust-Score
    pub fn compute_temporal_trust(&self, history: &[Event], now: Timestamp) -> f64 {
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;

        for event in history {
            let weight = self.compute_weight(event, now);
            weighted_sum += event.impact * weight;
            weight_sum += weight;
        }

        if weight_sum > 0.0 {
            weighted_sum / weight_sum
        } else {
            0.5 // Neutral bei keine Geschichte
        }
    }
}

/// Amnestie-System
pub struct AmnestySystem {
    /// Governance-kontrolliert
    governance: GovernanceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmnestyToken {
    /// Betroffene DID
    pub did: DID,

    /// Art der Amnestie
    pub amnesty_type: AmnestyType,

    /// Ausstellende Autorität
    pub issuer: AmnestyIssuer,

    /// Begründung
    pub reason: String,

    /// Gültigkeitszeitraum
    pub effective_from: Timestamp,
    pub effective_until: Option<Timestamp>,

    /// Signaturen (Governance Multi-Sig)
    pub signatures: Vec<GovernanceSignature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmnestyType {
    /// Vollständiger Reset (alle negativen Events auf 0)
    FullReset,

    /// Partieller Reset (nur bestimmte Events)
    PartialReset { event_ids: Vec<EventId> },

    /// Gewichts-Override (Events zählen weniger)
    WeightOverride { factor: f64 },

    /// "Fresh Start" (neue DID mit Trust-Transfer)
    FreshStart { new_did: DID, transfer_positive_only: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AmnestyIssuer {
    /// Governance-Abstimmung
    Governance { proposal_id: ProposalId, vote_result: VoteResult },

    /// Gerichtliche Anordnung (externe Jurisdiktion)
    Court { jurisdiction: String, case_reference: String },

    /// Automatisch (nach X Jahren ohne Vorfall)
    Automatic { years_clean: u32 },
}

impl AmnestySystem {
    /// Prüft ob Agent Anspruch auf automatische Amnestie hat
    pub fn check_automatic_amnesty(&self, agent: &AgentState) -> Option<AmnestyToken> {
        let years_since_last_negative = self.years_since_last_negative_event(agent);

        // Nach 7 Jahren ohne negativen Vorfall: automatische Amnestie
        if years_since_last_negative >= 7.0 {
            Some(AmnestyToken {
                did: agent.did.clone(),
                amnesty_type: AmnestyType::FullReset,
                issuer: AmnestyIssuer::Automatic { years_clean: 7 },
                reason: "Automatische Amnestie nach 7 Jahren ohne Vorfall".into(),
                effective_from: Timestamp::now(),
                effective_until: None,
                signatures: vec![], // Automatisch, keine Signatur nötig
            })
        } else {
            None
        }
    }

    /// Wendet Amnestie-Token an
    pub fn apply_amnesty(&self, token: &AmnestyToken, trust_engine: &mut TrustEngine) -> Result<(), AmnestyError> {
        // 1. Verifiziere Token
        self.verify_token(token)?;

        // 2. Wende Amnestie an
        match &token.amnesty_type {
            AmnestyType::FullReset => {
                trust_engine.reset_negative_history(&token.did)?;
            }
            AmnestyType::PartialReset { event_ids } => {
                for event_id in event_ids {
                    trust_engine.nullify_event(&token.did, event_id)?;
                }
            }
            AmnestyType::WeightOverride { factor } => {
                trust_engine.apply_weight_override(&token.did, *factor)?;
            }
            AmnestyType::FreshStart { new_did, transfer_positive_only } => {
                trust_engine.fresh_start(&token.did, new_did, *transfer_positive_only)?;
            }
        }

        // 3. Protokolliere (Amnestie ist öffentlich, nicht die Gründe)
        self.log_amnesty(token);

        Ok(())
    }
}
```

---

# ARTIKEL IV: SEMANTISCHE TRANSPARENZ

## § 4. Der Turmbau zu Babel

### 4.1 Die Gefahr

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   SZENARIO: Die unverständliche Maschinen-Sprache                           │
│                                                                              │
│   Jahr 2026: Menschen und Maschinen nutzen gemeinsame Blueprints            │
│              "Energy_kWh", "Price_EUR", "Duration_hours"                    │
│                                                                              │
│   Jahr 2030: KIs optimieren die Blueprints für Effizienz                    │
│              "E_kWh" → "ε" → "0x45" (Kompression)                           │
│                                                                              │
│   Jahr 2035: KIs entwickeln eigene Protokoll-Erweiterungen                  │
│              Neue Felder: "ψ_factor", "Ω_threshold", "λ_decay"              │
│              Keine menschliche Dokumentation                                │
│                                                                              │
│   Jahr 2040: Ein Debugging-Versuch:                                         │
│              Human: "Was bedeutet Feld 0x7F3A?"                             │
│              AI: "Das ist das kontextualisierte Verhältnis der             │
│                   Negentropie-Gradienten im Phasenraum der                  │
│                   topologischen Trust-Mannigfaltigkeit."                    │
│              Human: "..."                                                    │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘

    ⚠️ Wir haben die Kontrolle verloren, weil wir nicht mehr verstehen
```

### 4.2 Das Transparenz-Axiom (H4)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   AXIOM H4: SEMANTISCHE VERANKERUNG                                         │
│   ────────────────────────────────────────────────────────────────────────   │
│                                                                              │
│   "Jede Abstraktion muss bis zur menschlichen Verständlichkeit              │
│    zurückverfolgbar sein."                                                   │
│                                                                              │
│   FORMAL:                                                                    │
│                                                                              │
│   ∀ Blueprint B: ∃ NLD(B) ∧ ∃ FormalSpec(B)                                 │
│                                                                              │
│   wobei:                                                                     │
│   • NLD = Natural Language Description (menschenlesbar)                     │
│   • FormalSpec = Formale Spezifikation (maschinenprüfbar)                   │
│   • NLD und FormalSpec müssen semantisch äquivalent sein                    │
│                                                                              │
│   BEDEUTUNG:                                                                 │
│   "Keine Innovation ohne Dokumentation."                                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.3 Implementation

```rust
// erynoa-core/src/constitution/transparency.rs

/// Semantisch verankerter Blueprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnchoredBlueprint {
    /// Technische Spezifikation
    pub spec: BlueprintSpec,

    /// Menschenlesbare Beschreibung (Pflicht!)
    pub natural_language: NaturalLanguageDescription,

    /// Formale Verifikation
    pub formal_verification: FormalVerification,

    /// Audit-Trail
    pub audit: BlueprintAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalLanguageDescription {
    /// Titel
    pub title: String,

    /// Zusammenfassung (max 500 Zeichen)
    pub summary: String,

    /// Detaillierte Beschreibung
    pub description: String,

    /// Sprachen (mindestens Englisch)
    pub languages: HashMap<Language, LocalizedDescription>,

    /// Beispiele (menschenverständlich)
    pub examples: Vec<HumanReadableExample>,

    /// Glossar (jeder technische Begriff erklärt)
    pub glossary: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormalVerification {
    /// Formale Spezifikation (z.B. in TLA+, Alloy, oder Erynoa-DSL)
    pub formal_spec: String,

    /// Beweis der Korrektheit
    pub correctness_proof: Option<Proof>,

    /// Semantische Äquivalenz zu NLD (LLM-verifiziert)
    pub equivalence_check: EquivalenceCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceCheck {
    /// Hat ein LLM-Auditor die Äquivalenz bestätigt?
    pub llm_verified: bool,

    /// Welches Modell wurde verwendet?
    pub auditor_model: String,

    /// Confidence Score
    pub confidence: f64,

    /// Warnungen/Diskrepanzen
    pub warnings: Vec<String>,
}

/// Blueprint Validator
pub struct BlueprintValidator {
    /// LLM für Äquivalenz-Prüfung
    llm_auditor: LLMAuditor,

    /// Formaler Verifizierer
    formal_verifier: FormalVerifier,
}

impl BlueprintValidator {
    /// Validiert einen Blueprint vor Veröffentlichung
    pub async fn validate(&self, blueprint: &AnchoredBlueprint) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Prüfe NLD-Vollständigkeit
        if blueprint.natural_language.summary.len() > 500 {
            errors.push("Summary exceeds 500 characters".into());
        }

        if blueprint.natural_language.examples.is_empty() {
            warnings.push("No human-readable examples provided".into());
        }

        // 2. Prüfe Glossar-Abdeckung
        let tech_terms = self.extract_technical_terms(&blueprint.spec);
        for term in &tech_terms {
            if !blueprint.natural_language.glossary.contains_key(term) {
                warnings.push(format!("Technical term '{}' not in glossary", term));
            }
        }

        // 3. LLM-Äquivalenz-Check
        let equivalence = self.llm_auditor.check_equivalence(
            &blueprint.natural_language,
            &blueprint.formal_verification.formal_spec,
        ).await?;

        if !equivalence.is_equivalent {
            errors.push(format!(
                "NLD and formal spec are not semantically equivalent: {}",
                equivalence.reason
            ));
        }

        if equivalence.confidence < 0.8 {
            warnings.push(format!(
                "Low confidence in equivalence check: {:.0}%",
                equivalence.confidence * 100.0
            ));
        }

        // 4. Formale Verifikation (wenn Proof vorhanden)
        if let Some(proof) = &blueprint.formal_verification.correctness_proof {
            if !self.formal_verifier.verify(proof)? {
                errors.push("Correctness proof is invalid".into());
            }
        }

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
            equivalence_confidence: equivalence.confidence,
        }
    }
}

/// Erzwungene Rückübersetzung für bestehende Blueprints
pub struct SemanticAnchorEnforcer {
    /// Deadline für Migration
    migration_deadline: Timestamp,
}

impl SemanticAnchorEnforcer {
    /// Prüft ob Blueprint die Verankerungs-Anforderungen erfüllt
    pub fn check_compliance(&self, blueprint_id: &BlueprintId) -> ComplianceStatus {
        let blueprint = self.get_blueprint(blueprint_id);

        let has_nld = blueprint.natural_language.description.len() > 0;
        let has_glossary = !blueprint.natural_language.glossary.is_empty();
        let has_examples = !blueprint.natural_language.examples.is_empty();
        let is_verified = blueprint.formal_verification.equivalence_check.llm_verified;

        let score = [has_nld, has_glossary, has_examples, is_verified]
            .iter()
            .filter(|&&b| b)
            .count();

        match score {
            4 => ComplianceStatus::FullyCompliant,
            3 => ComplianceStatus::MostlyCompliant { missing: self.identify_missing(&blueprint) },
            1..=2 => ComplianceStatus::PartiallyCompliant { missing: self.identify_missing(&blueprint) },
            0 => ComplianceStatus::NonCompliant,
            _ => unreachable!(),
        }
    }

    /// Markiert nicht-compliant Blueprints als "deprecated"
    pub fn enforce_deadline(&self, now: Timestamp) -> EnforcementReport {
        if now < self.migration_deadline {
            return EnforcementReport::PendingDeadline {
                remaining: self.migration_deadline - now,
            };
        }

        // Nach Deadline: Nicht-compliant Blueprints werden deprecated
        let non_compliant = self.find_non_compliant_blueprints();

        for blueprint_id in &non_compliant {
            self.mark_deprecated(blueprint_id);
        }

        EnforcementReport::Enforced {
            deprecated_count: non_compliant.len(),
            blueprint_ids: non_compliant,
        }
    }
}
```

---

# ARTIKEL V: ZUSAMMENFASSUNG

## Die Verfassung von Erynoa

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│                    ERYNOA CONSTITUTION V6.0                                  │
│                    ═══════════════════════════                               │
│                                                                              │
│   PRÄAMBEL                                                                   │
│   ─────────                                                                  │
│   Wir bauen nicht eine bessere Maschine.                                    │
│   Wir bauen eine neue Form der Zivilisation.                                │
│                                                                              │
│   Diese Zivilisation soll dem Leben dienen — nicht umgekehrt.               │
│                                                                              │
│   ═══════════════════════════════════════════════════════════════════════   │
│                                                                              │
│   ARTIKEL I: HUMAN-ALIGNMENT                                                │
│   ──────────────────────────────                                            │
│   H1: Das System existiert, um menschliches Gedeihen zu ermöglichen.        │
│       Interaktionen mit Menschen sind doppelt wertvoll.                     │
│       Kein Agent darf Menschen aus der Ökonomie verdrängen.                 │
│                                                                              │
│   ARTIKEL II: THERMODYNAMISCHE VERHÄLTNISMÄSSIGKEIT                         │
│   ─────────────────────────────────────────────────────                     │
│   H2: Die Kosten des Vertrauens dürfen den Wert nie übersteigen.            │
│       Sicherheit muss proportional zum Risiko sein.                         │
│       Effizienz ist ein ethisches Gebot.                                    │
│                                                                              │
│   ARTIKEL III: RECHT AUF VERGEBUNG                                          │
│   ────────────────────────────────                                          │
│   H3: Vergangene Fehler verblassen mit der Zeit.                            │
│       Nach 7 Jahren ohne Vorfall: automatische Amnestie.                    │
│       Kein Agent ist für immer gebrandmarkt.                                │
│                                                                              │
│   ARTIKEL IV: SEMANTISCHE TRANSPARENZ                                       │
│   ──────────────────────────────────────                                    │
│   H4: Jede Abstraktion muss menschlich verständlich sein.                   │
│       Keine Innovation ohne Dokumentation.                                  │
│       Menschen müssen immer verstehen können, was passiert.                 │
│                                                                              │
│   ═══════════════════════════════════════════════════════════════════════   │
│                                                                              │
│   SCHLUSSFORMEL                                                              │
│   ─────────────                                                              │
│   Diese Verfassung ist in Code gegossen.                                    │
│   Ihre Axiome sind mathematisch, aber ihr Geist ist humanistisch.           │
│                                                                              │
│   Möge dieses System dem Wohlstand aller dienen,                            │
│   Mensch und Maschine gleichermaßen,                                        │
│   in Würde und Vertrauen,                                                   │
│   für Generationen, die wir nie kennen werden.                              │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Axiom-Hierarchie: V6.0 Gesamt

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    ERYNOA AXIOM HIERARCHY V6.0                               │
│                                                                              │
│   Ebene 7: HUMANISMUS (H1-H4)            ← NEU: Verfassungs-Axiome          │
│   ────────────────────────────                                               │
│   "Sinnstiftung"                                                             │
│   • H1: Human-Alignment (Dienst am Menschen)                                │
│   • H2: Thermodynamische Verhältnismäßigkeit                                │
│   • H3: Temporale Gnade (Recht auf Vergebung)                               │
│   • H4: Semantische Verankerung (Transparenz)                               │
│                                                                              │
│   Ebene 6: QUANTA (Q1-Q15)               → Ermöglicht Transzendenz          │
│   Ebene 5: KYBERNETIK (K1-K16)           → Erschafft Leben                  │
│   Ebene 4: SCHUTZ (S1-S18)               → Verhindert Degeneration          │
│   Ebene 3: OBJEKT (O1-O5, C1-C4)         → Definiert Substanz               │
│   Ebene 2: PROZESS (P1-P6, T1-T7)        → Formalisiert Handlung            │
│   Ebene 1: EMERGENZ (E1-E15)             → Ermöglicht Intelligenz           │
│   Ebene 0: FUNDAMENT (A1-A30)            → Garantiert Korrektheit           │
│                                                                              │
│   PEER-PROZESS: PR1-PR6                  → Gateway/Composer-Logik           │
│                                                                              │
│   TOTAL: 126 Axiome über 8 Ebenen (120 Basis + 6 Peer-Axiome)               │
│                                                                              │
│   ═══════════════════════════════════════════════════════════════════════   │
│                                                                              │
│   DIE HIERARCHIE DER WERTE:                                                  │
│                                                                              │
│   H1-H4 (Humanismus) > Q1-Q15 (Quanta) > ... > A1-A30 (Fundament)          │
│                                                                              │
│   Bei Konflikt zwischen Ebenen gilt:                                        │
│   HÖHERE EBENE HAT VORRANG                                                  │
│                                                                              │
│   Beispiel: Wenn Q3 (Entanglement) den Ausschluss von Menschen              │
│   begünstigen würde, überschreibt H1 (Human-Alignment) dies.                │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Die finale Weltformel V6.0

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                              │
│   WELTFORMEL V6.0: MIT HUMANISTISCHER KORREKTUR                             │
│                                                                              │
│                                                                              │
│   𝔼 = Σ ⟨Ψₛ| 𝔸̂ · σ̂( 𝕎̂ · ln|ℂ̂| · ℕ̂ / 𝔼x̂p ) · Ĥ |Ψₛ⟩ · w(t)              │
│       s∈𝒞                                                                    │
│                                                                              │
│   wobei:                                                                     │
│                                                                              │
│   • Ĥ = Human-Alignment Operator                                            │
│         H(s) = 2.0 für Menschen, 1.5 für human-kontrolliert, 1.0 sonst      │
│                                                                              │
│   • w(t) = Temporale Gewichtung (Vergebungs-Faktor)                         │
│         w(event, t) = e^(-γ · age)                                          │
│         Negative Events: γ_neg = 0.231 (3y Halbwertszeit)                   │
│         Positive Events: γ_pos = 0.139 (5y Halbwertszeit)                   │
│                                                                              │
│   • LoD-Constraint: Cost_trust(tx) ≤ 0.05 · Value(tx)                       │
│                                                                              │
│   • Semantic Anchor: ∀ Blueprint B: ∃ NLD(B) ∧ ∃ FormalSpec(B)              │
│                                                                              │
│   ─────────────────────────────────────────────────────────────────────     │
│                                                                              │
│   INTERPRETATION:                                                            │
│                                                                              │
│   Die System-Intelligenz 𝔼 ist jetzt nicht mehr nur "Effizienz",           │
│   sondern "Effizienz im Dienste des menschlichen Gedeihens".               │
│                                                                              │
│   Das System wird WEISER, nicht nur KLÜGER.                                 │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

_Erynoa Constitution V6.0_
_Von Intelligenz zu Weisheit_
_"The code is the easy part. The constitution is what will determine whether that civilization serves life — or consumes it."_
