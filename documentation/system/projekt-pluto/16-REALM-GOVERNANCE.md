# 🗳️ Realm-Governance: Souveräne Entscheidungsfindung

> **Teil von:** Projekt Pluto
> **Kategorie:** Kernarchitektur – Governance
> **Status:** Spezifikation
> **Datum:** 2026-02-04

---

## 1. Fundamentale Prinzipien

### 1.1 Governance ist Realm-exklusiv

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    AXIOM: GOVERNANCE IST REALM-GEBUNDEN                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   ❌ Identitäten haben KEINE Governance                                     ║
║   ❌ Packages haben KEINE Governance                                        ║
║   ❌ Globale System-Ebene hat KEINE direktdemokratische Governance          ║
║                                                                              ║
║   ✅ NUR REALMS können Governance haben                                     ║
║                                                                              ║
║   Begründung:                                                                ║
║   - Realms sind souveräne Einheiten mit definierten Grenzen                ║
║   - Governance erfordert eine geschlossene Member-Menge                     ║
║   - Abstimmungen brauchen einen Kontext (Realm-Regeln, -Policies)          ║
║   - Trust ist Realm-lokal (Κ24) → Stimmgewicht muss es auch sein           ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 1.2 Die Governance-Formel

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    GOVERNANCE STIMMGEWICHT-FORMEL                            ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   vote_weight(member) = f(membership_weight, relative_trust)                ║
║                                                                              ║
║   Wobei:                                                                     ║
║   - membership_weight: Definiert durch Governance-Typ (Token, Reputation...)║
║   - relative_trust: Optionaler Trust-Modifikator (Realm-lokaler Trust)      ║
║                                                                              ║
║   Konkret:                                                                   ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                     │   ║
║   │   W(m) = G(m) × (1 + α × T_rel(m))                                  │   ║
║   │                                                                     │   ║
║   │   Wobei:                                                            │   ║
║   │   - W(m)     = Finales Stimmgewicht des Members m                   │   ║
║   │   - G(m)     = Governance-Basis-Gewicht (aus GovernanceType)        │   ║
║   │   - α        = Trust-Einfluss-Faktor ∈ [0, 1] (Realm-konfiguriert)  │   ║
║   │   - T_rel(m) = Relativer Trust im Realm ∈ [-1, 1]                   │   ║
║   │                                                                     │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   T_rel(m) = (T(m) - T_avg) / T_avg                                         ║
║                                                                              ║
║   → Trust über Durchschnitt = Bonus                                         ║
║   → Trust unter Durchschnitt = Malus                                        ║
║   → Trust = Durchschnitt = neutral (×1)                                     ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 2. Governance-Typen (Erweitert)

### 2.1 Die fünf Governance-Modi

```rust
/// Governance-Typen für Realms (erweitert von Κ21)
pub enum GovernanceType {
    /// Quadratisches Voting (Κ21): √tokens = votes
    /// Reduziert Plutokratie, bevorzugt breite Partizipation
    Quadratic {
        token_symbol: String,
        trust_influence: f64,  // α ∈ [0, 1]
    },

    /// Token-basiert: 1 token = 1 vote
    /// Klassisches DAO-Modell
    Token {
        token_symbol: String,
        trust_influence: f64,
    },

    /// Reputation-basiert: Trust = Vote
    /// Meritokratisch, keine Token nötig
    Reputation {
        trust_dimensions: Vec<TrustDimension>,  // Welche Dimensionen zählen
        dimension_weights: Vec<f64>,            // Gewichtung
    },

    /// Delegated (Liquid Democracy)
    /// Members können ihre Stimme delegieren
    Delegated {
        base_type: Box<GovernanceType>,  // Underlying Governance
        max_delegation_depth: u8,        // Max. Kette (default: 5)
        delegation_decay: f64,           // Trust-Decay pro Hop (Κ8)
    },

    /// Member-Equal: 1 member = 1 vote
    /// Basisdemokratie ohne Gewichtung
    MemberEqual {
        trust_influence: f64,  // Kann Trust trotzdem berücksichtigen
    },
}
```

### 2.2 Vergleichsmatrix

```text
╔═══════════════════════════════════════════════════════════════════════════════════╗
║                         GOVERNANCE-TYPEN VERGLEICH                                 ║
╠═══════════════════╦═════════════════╦═════════════════╦═══════════════════════════╣
║  Typ              ║ Basis-Gewicht   ║ Trust-Einfluss  ║ Anwendungsfall            ║
╠═══════════════════╬═════════════════╬═════════════════╬═══════════════════════════╣
║  Quadratic        ║ √(tokens)       ║ Optional (α)    ║ DAOs, faire Token-Voting  ║
╠═══════════════════╬═════════════════╬═════════════════╬═══════════════════════════╣
║  Token            ║ tokens          ║ Optional (α)    ║ Investment-DAOs           ║
╠═══════════════════╬═════════════════╬═════════════════╬═══════════════════════════╣
║  Reputation       ║ T(m)            ║ Immer (100%)    ║ Merit-basierte Guilds     ║
╠═══════════════════╬═════════════════╬═════════════════╬═══════════════════════════╣
║  Delegated        ║ delegated_sum   ║ Via Base        ║ Große Communities         ║
╠═══════════════════╬═════════════════╬═════════════════╬═══════════════════════════╣
║  MemberEqual      ║ 1               ║ Optional (α)    ║ Kleine Teams, Cooperatives║
╚═══════════════════╩═════════════════╩═════════════════╩═══════════════════════════╝
```

---

## 3. Trust-Einfluss: Der Charakter-Faktor

### 3.1 Warum relativer Trust?

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    RELATIVER TRUST ALS CHARAKTER-INDIKATOR                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Der relative Trust zeigt, wie vertrauenswürdig ein Member                 ║
║   IM VERGLEICH zu anderen Members ist.                                      ║
║                                                                              ║
║   Warum relativ und nicht absolut?                                          ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                     │   ║
║   │   Realm A: Alle Members haben Trust 0.9                             │   ║
║   │   → Absoluter Trust würde ALLE gleich stark boosten                │   ║
║   │   → Kein differenzierender Effekt                                  │   ║
║   │                                                                     │   ║
║   │   Realm B: Trust-Verteilung von 0.3 bis 0.95                       │   ║
║   │   → Relativer Trust differenziert zwischen Members                 │   ║
║   │   → Wer sich mehr engagiert, bekommt mehr Einfluss                 │   ║
║   │                                                                     │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   Relativer Trust ist FAIR:                                                  ║
║   - Newcomer in High-Trust-Realm → Malus (noch nicht bewiesen)              ║
║   - Veteran in neuem Realm → Bonus (hat sich woanders bewährt)              ║
║   - Durchschnittlicher Member → Neutral (weder Bonus noch Malus)            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 3.2 Berechnung des relativen Trust

```rust
/// Berechnet den relativen Trust eines Members im Realm
pub fn calculate_relative_trust(
    member_trust: &TrustVector6D,
    realm_trust_stats: &RealmTrustStatistics,
    governance_config: &GovernanceConfig,
) -> f64 {
    // Aggregierten Trust-Wert berechnen (gewichtet nach Konfiguration)
    let member_score = aggregate_trust(member_trust, &governance_config.trust_dimensions);
    let realm_average = realm_trust_stats.average_trust;

    // Relativer Trust: (T - T_avg) / T_avg
    // Ergebnis: ∈ [-1, ∞) aber praktisch [-1, 1] bei normalem Trust
    if realm_average > 0.0 {
        (member_score - realm_average) / realm_average
    } else {
        0.0  // Fallback bei leerem Realm
    }
}

/// Aggregiert Trust-Dimensionen gemäß Konfiguration
fn aggregate_trust(trust: &TrustVector6D, config: &TrustDimensionConfig) -> f64 {
    let weighted_sum =
        trust.r * config.weights.reliability +
        trust.i * config.weights.integrity +
        trust.c * config.weights.competence +
        trust.p * config.weights.prestige +
        trust.v * config.weights.vigilance +
        trust.omega * config.weights.omega;

    let total_weight = config.weights.total();
    if total_weight > 0.0 {
        weighted_sum / total_weight
    } else {
        trust.average()  // Fallback: Gleichgewichtung
    }
}
```

### 3.3 Trust-Einfluss-Faktor (α)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    TRUST-EINFLUSS-FAKTOR α                                   ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   α = 0.0: Trust hat KEINEN Einfluss                                        ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   W(m) = G(m) × (1 + 0 × T_rel) = G(m)                                      ║
║   → Reines Token/Reputation-Voting                                          ║
║   → Anwendung: DAOs wo nur Stake zählt                                      ║
║                                                                              ║
║   α = 0.5: Moderater Trust-Einfluss                                         ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   W(m) = G(m) × (1 + 0.5 × T_rel)                                           ║
║   → Trust kann ±50% Bonus/Malus geben                                       ║
║   → Member mit T_rel = +1: 150% Gewicht                                     ║
║   → Member mit T_rel = -0.5: 75% Gewicht                                    ║
║   → Anwendung: Balanced DAOs                                                ║
║                                                                              ║
║   α = 1.0: Maximaler Trust-Einfluss                                         ║
║   ─────────────────────────────────────────────────────────────────────────  ║
║   W(m) = G(m) × (1 + 1.0 × T_rel)                                           ║
║   → Trust kann ±100% Bonus/Malus geben                                      ║
║   → Member mit T_rel = +1: 200% Gewicht (verdoppelt)                        ║
║   → Member mit T_rel = -0.5: 50% Gewicht (halbiert)                         ║
║   → Anwendung: Trust-zentrierte Communities                                 ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 4. Governance-Konfiguration pro Realm

### 4.1 ECL-Policy Definition

```ecl
// Vollständige Governance-Konfiguration für ein Realm
governance_config "dao-governance-v1" {
    // ═══════════════════════════════════════════════════════════════════════
    // GOVERNANCE-TYP
    // ═══════════════════════════════════════════════════════════════════════
    governance_type: quadratic {
        token_symbol: "REALM_GOV",
        trust_influence: 0.5,  // α = 0.5 → moderater Trust-Einfluss
    },

    // ═══════════════════════════════════════════════════════════════════════
    // TRUST-DIMENSION-GEWICHTUNG
    // ═══════════════════════════════════════════════════════════════════════
    trust_dimensions: {
        reliability: 1.0,    // R: Verhaltens-Historie
        integrity: 1.5,      // I: Aussage-Konsistenz (höher gewichtet)
        competence: 0.5,     // C: Fähigkeits-Nachweis
        prestige: 0.5,       // P: Externe Attestation
        vigilance: 1.0,      // V: Anomalie-Erkennung
        omega: 2.0,          // Ω: Axiom-Treue (höchste Gewichtung!)
    },

    // ═══════════════════════════════════════════════════════════════════════
    // PROPOSAL-ANFORDERUNGEN
    // ═══════════════════════════════════════════════════════════════════════
    proposal_requirements: {
        // Wer darf Proposals erstellen?
        min_membership_days: 30,         // Mindestens 30 Tage Member
        min_trust_to_propose: 0.5,       // Mindest-Trust
        min_tokens_to_propose: 100,      // Mindest-Token-Balance

        // Zeitliche Grenzen
        min_discussion_period: "48h",    // Diskussion vor Voting
        min_voting_period: "72h",        // Voting-Dauer
        max_voting_period: "14d",        // Max. Voting-Dauer

        // Proposal-Kosten (Anti-Spam)
        proposal_mana_cost: 500,         // Mana-Kosten für Proposal
        proposal_deposit: 50,            // Token-Deposit (refundable)
    },

    // ═══════════════════════════════════════════════════════════════════════
    // QUORUM & APPROVAL
    // ═══════════════════════════════════════════════════════════════════════
    quorum: {
        // Mindest-Beteiligung (in % der vote_power)
        min_participation: 0.10,  // 10% aller Stimmgewichte

        // Zustimmungs-Schwelle
        min_approval: 0.50,       // 50% der abgegebenen Stimmen

        // Dynamisches Quorum (optional)
        dynamic_quorum: {
            enabled: true,
            base_quorum: 0.10,
            participation_boost: 0.05,  // +5% pro 10% Participation
            max_quorum: 0.30,           // Nie mehr als 30%
        },
    },

    // ═══════════════════════════════════════════════════════════════════════
    // EXECUTION
    // ═══════════════════════════════════════════════════════════════════════
    execution: {
        // Timelock: Verzögerung zwischen Approval und Execution
        timelock: "24h",

        // Veto-Mechanismus
        veto_threshold: 0.33,     // 33% können blockieren
        veto_period: "12h",       // Veto-Window nach Approval

        // Automatische Execution via ECLVM
        auto_execute: true,
        execution_gas_limit: 100000,
        execution_mana_limit: 10000,

        // Notfall-Pause (für kritische Proposals)
        emergency_pause_threshold: 0.25,  // 25% können Pause triggern
    },

    // ═══════════════════════════════════════════════════════════════════════
    // PROPOSAL-KATEGORIEN
    // ═══════════════════════════════════════════════════════════════════════
    proposal_categories: {
        // Verschiedene Kategorien mit unterschiedlichen Anforderungen
        "parameter_change": {
            min_approval: 0.50,
            timelock: "24h",
        },
        "treasury_spend": {
            min_approval: 0.60,
            timelock: "48h",
            max_amount_without_supermajority: 1000,
        },
        "rule_change": {
            min_approval: 0.67,    // Supermajority
            timelock: "72h",
        },
        "member_ban": {
            min_approval: 0.75,    // Hohe Schwelle
            timelock: "24h",
            require_evidence: true,
        },
        "governance_change": {
            min_approval: 0.80,    // Höchste Schwelle
            timelock: "7d",        // Längste Verzögerung
        },
    },
}
```

### 4.2 Rust-Strukturen

```rust
/// Vollständige Governance-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Governance-Typ (definiert Basis-Gewichtung)
    pub governance_type: GovernanceType,

    /// Trust-Dimension-Gewichtung
    pub trust_dimension_weights: TrustDimensionWeights,

    /// Proposal-Anforderungen
    pub proposal_requirements: ProposalRequirements,

    /// Quorum-Konfiguration
    pub quorum: QuorumConfig,

    /// Execution-Konfiguration
    pub execution: ExecutionConfig,

    /// Kategorie-spezifische Overrides
    pub category_overrides: HashMap<ProposalCategory, CategoryConfig>,
}

/// Trust-Dimension-Gewichtung für Governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustDimensionWeights {
    pub reliability: f64,    // R
    pub integrity: f64,      // I
    pub competence: f64,     // C
    pub prestige: f64,       // P
    pub vigilance: f64,      // V
    pub omega: f64,          // Ω
}

impl TrustDimensionWeights {
    pub fn total(&self) -> f64 {
        self.reliability + self.integrity + self.competence +
        self.prestige + self.vigilance + self.omega
    }

    /// Default: Alle Dimensionen gleich, Omega doppelt
    pub fn default() -> Self {
        Self {
            reliability: 1.0,
            integrity: 1.0,
            competence: 1.0,
            prestige: 1.0,
            vigilance: 1.0,
            omega: 2.0,  // Axiom-Treue ist wichtiger
        }
    }
}
```

---

## 5. Stimmgewicht-Berechnung

### 5.1 Algorithmus

```rust
/// Berechnet das finale Stimmgewicht eines Members
pub fn calculate_vote_weight(
    member: &RealmMember,
    realm_state: &RealmSpecificState,
    governance_config: &GovernanceConfig,
) -> Result<VoteWeight, GovernanceError> {
    // 1. Basis-Gewicht aus Governance-Typ
    let base_weight = match &governance_config.governance_type {
        GovernanceType::Quadratic { token_symbol, .. } => {
            let tokens = member.token_balance(token_symbol)?;
            (tokens as f64).sqrt()
        },
        GovernanceType::Token { token_symbol, .. } => {
            member.token_balance(token_symbol)? as f64
        },
        GovernanceType::Reputation { trust_dimensions, dimension_weights } => {
            aggregate_trust_weighted(&member.trust, trust_dimensions, dimension_weights)
        },
        GovernanceType::MemberEqual { .. } => {
            1.0  // Jeder Member = 1
        },
        GovernanceType::Delegated { base_type, .. } => {
            // Rekursiv: Basis-Gewicht + delegierte Gewichte
            calculate_delegated_weight(member, realm_state, base_type)?
        },
    };

    // 2. Trust-Einfluss (α)
    let trust_influence = governance_config.governance_type.trust_influence();

    // 3. Relativer Trust berechnen
    let relative_trust = if trust_influence > 0.0 {
        calculate_relative_trust(
            &member.trust,
            &realm_state.trust_statistics,
            governance_config,
        )
    } else {
        0.0  // Kein Trust-Einfluss
    };

    // 4. Finale Formel: W(m) = G(m) × (1 + α × T_rel(m))
    let trust_multiplier = 1.0 + (trust_influence * relative_trust);

    // 5. Sicherheits-Clamp: Stimmgewicht kann nie negativ sein
    let final_weight = (base_weight * trust_multiplier).max(0.0);

    Ok(VoteWeight {
        base_weight,
        trust_multiplier,
        final_weight,
        relative_trust,
    })
}

/// Ergebnis der Stimmgewicht-Berechnung
#[derive(Debug, Clone)]
pub struct VoteWeight {
    pub base_weight: f64,       // G(m)
    pub trust_multiplier: f64,  // (1 + α × T_rel)
    pub final_weight: f64,      // W(m) = G(m) × multiplier
    pub relative_trust: f64,    // T_rel(m) für Transparenz
}
```

### 5.2 Beispielrechnungen

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    BEISPIEL: QUADRATIC VOTING MIT TRUST                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Governance-Typ: Quadratic, α = 0.5                                        ║
║   Realm Trust-Durchschnitt: T_avg = 0.6                                     ║
║                                                                              ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │  MEMBER A: 1000 Tokens, Trust = 0.9                                 │   ║
║   │  ─────────────────────────────────────────────────────────────────  │   ║
║   │  G(A) = √1000 = 31.62                                               │   ║
║   │  T_rel(A) = (0.9 - 0.6) / 0.6 = +0.5                               │   ║
║   │  Trust-Multiplier = 1 + (0.5 × 0.5) = 1.25                         │   ║
║   │  W(A) = 31.62 × 1.25 = 39.53 ✓                                     │   ║
║   │                                                                     │   ║
║   │  → 25% Bonus durch überdurchschnittlichen Trust                    │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │  MEMBER B: 2500 Tokens, Trust = 0.6                                 │   ║
║   │  ─────────────────────────────────────────────────────────────────  │   ║
║   │  G(B) = √2500 = 50.0                                                │   ║
║   │  T_rel(B) = (0.6 - 0.6) / 0.6 = 0.0                                │   ║
║   │  Trust-Multiplier = 1 + (0.5 × 0.0) = 1.0                          │   ║
║   │  W(B) = 50.0 × 1.0 = 50.0                                          │   ║
║   │                                                                     │   ║
║   │  → Neutral: Durchschnittlicher Trust                               │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │  MEMBER C: 4000 Tokens, Trust = 0.3                                 │   ║
║   │  ─────────────────────────────────────────────────────────────────  │   ║
║   │  G(C) = √4000 = 63.25                                               │   ║
║   │  T_rel(C) = (0.3 - 0.6) / 0.6 = -0.5                               │   ║
║   │  Trust-Multiplier = 1 + (0.5 × -0.5) = 0.75                        │   ║
║   │  W(C) = 63.25 × 0.75 = 47.44                                       │   ║
║   │                                                                     │   ║
║   │  → 25% Malus durch unterdurchschnittlichen Trust                   │   ║
║   │  → Trotz 4× mehr Tokens weniger Einfluss als B!                    │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   Fazit:                                                                     ║
║   - Token-Menge allein reicht nicht                                         ║
║   - Vertrauenswürdiges Verhalten wird belohnt                               ║
║   - System ist Sybil-resistent (neue Accounts haben niedrigen Trust)        ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 6. Delegated Governance (Liquid Democracy)

### 6.1 Konzept

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    LIQUID DEMOCRACY: DELEGATION                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Members können ihre Stimmkraft an andere delegieren:                      ║
║                                                                              ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                     │   ║
║   │   Alice (100 votes) ──delegate──► Bob                              │   ║
║   │                                                                     │   ║
║   │   Bob (50 votes) + Alice's (100) = 150 effektive Stimmen           │   ║
║   │                                                                     │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   Eigenschaften:                                                             ║
║   - Transitiv: Alice → Bob → Carol (Ketten erlaubt)                        ║
║   - Widerrufbar: Alice kann jederzeit zurücknehmen                         ║
║   - Kategorie-spezifisch: Alice delegiert nur für "treasury_spend"         ║
║   - Trust-basiert: Delegation-Decay basiert auf Trust (Κ8)                 ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 6.2 Delegation mit Trust-Decay (Κ8)

```rust
/// Berechnet delegiertes Stimmgewicht mit Trust-Decay
pub fn calculate_delegated_weight(
    delegate: &RealmMember,
    realm_state: &RealmSpecificState,
    config: &DelegatedGovernanceConfig,
) -> Result<f64, GovernanceError> {
    // Eigenes Basis-Gewicht
    let own_weight = calculate_base_weight(delegate, &config.base_type)?;

    // Delegationen zu diesem Member finden
    let delegations = realm_state
        .governance
        .get_delegations_to(&delegate.did)?;

    let mut delegated_sum = 0.0;

    for delegation in delegations {
        // Maximale Tiefe prüfen
        if delegation.depth > config.max_delegation_depth {
            continue;
        }

        // Κ8: Trust-Decay pro Delegation-Hop
        // decay_factor = trust(delegator) ^ depth
        let decay_factor = delegation
            .delegator_trust
            .powf(delegation.depth as f64);

        // Minimum-Decay (verhindert zu starke Konzentration)
        let effective_decay = decay_factor.max(config.min_decay_factor);

        // Delegiertes Gewicht = Original-Gewicht × Decay
        let delegated_weight = delegation.original_weight * effective_decay;

        delegated_sum += delegated_weight;
    }

    Ok(own_weight + delegated_sum)
}

/// Eine Delegation
#[derive(Debug, Clone)]
pub struct Delegation {
    pub delegator_did: UniversalId,
    pub delegate_did: UniversalId,
    pub original_weight: f64,
    pub depth: u8,                      // Wie viele Hops
    pub delegator_trust: f64,           // Trust des Delegierenden
    pub categories: Option<Vec<ProposalCategory>>,  // Nur für bestimmte Kategorien
    pub expires_at: Option<DateTime>,   // Optional: Ablaufdatum
}
```

---

## 7. Proposal-Lifecycle

### 7.1 Phasen

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    PROPOSAL-LIFECYCLE                                         ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐           ║
║   │  DRAFT   │────►│DISCUSSION│────►│  VOTING  │────►│ TIMELOCK │           ║
║   └──────────┘     └──────────┘     └──────────┘     └──────────┘           ║
║        │                │                │                │                  ║
║        │                │                │                ▼                  ║
║        │                │                │          ┌──────────┐            ║
║        │                │                │          │ EXECUTED │            ║
║        │                │                │          └──────────┘            ║
║        │                │                │                                  ║
║        ▼                ▼                ▼                                  ║
║   ┌──────────┐     ┌──────────┐     ┌──────────┐                           ║
║   │WITHDRAWN │     │ REJECTED │     │ DEFEATED │                           ║
║   └──────────┘     └──────────┘     └──────────┘                           ║
║                                          │                                  ║
║                                          ▼                                  ║
║                                    ┌──────────┐                            ║
║                                    │  VETOED  │                            ║
║                                    └──────────┘                            ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝

Phasen:
─────────────────────────────────────────────────────────────────────────────

1. DRAFT
   - Proposal wird erstellt aber noch nicht eingereicht
   - Autor kann bearbeiten
   - Kein Mana-Kosten

2. DISCUSSION
   - Proposal ist öffentlich sichtbar
   - Community kann kommentieren
   - Mindestdauer: min_discussion_period
   - Mana-Kosten werden abgezogen

3. VOTING
   - Members stimmen ab
   - Dauer: voting_period
   - Keine Änderungen am Proposal

4. TIMELOCK
   - Proposal wurde angenommen
   - Warte-Zeit vor Execution
   - Veto-Window ist offen

5. EXECUTED
   - Proposal wurde ausgeführt
   - Änderungen sind aktiv

Abbruch-Zustände:
─────────────────────────────────────────────────────────────────────────────

- WITHDRAWN: Autor hat zurückgezogen (vor Voting)
- REJECTED: Discussion-Phase nicht bestanden (zu wenig Support)
- DEFEATED: Quorum oder Approval nicht erreicht
- VETOED: Veto-Threshold erreicht in Timelock-Phase
```

### 7.2 Proposal-Typen und ihre Anforderungen

```rust
/// Proposal-Kategorien mit unterschiedlichen Anforderungen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProposalCategory {
    /// Parameter-Änderung (Quorum, Timelock, etc.)
    ParameterChange,

    /// Treasury-Ausgabe
    TreasurySpend,

    /// Regel-Änderung (ECL-Policies)
    RuleChange,

    /// Member-Ban oder -Unban
    MemberAction,

    /// Governance-Änderung (meta!)
    GovernanceChange,

    /// Package-Installation/-Deinstallation
    PackageAction,

    /// Realm-Einstellungen (Name, Description, etc.)
    RealmSettings,

    /// Custom (für erweiterbare Governance)
    Custom(String),
}

impl ProposalCategory {
    /// Default-Anforderungen pro Kategorie
    pub fn default_requirements(&self) -> CategoryRequirements {
        match self {
            Self::ParameterChange => CategoryRequirements {
                min_approval: 0.50,
                timelock: Duration::hours(24),
                supermajority_required: false,
            },
            Self::TreasurySpend => CategoryRequirements {
                min_approval: 0.60,
                timelock: Duration::hours(48),
                supermajority_required: false,  // Außer bei großen Beträgen
            },
            Self::RuleChange => CategoryRequirements {
                min_approval: 0.67,
                timelock: Duration::hours(72),
                supermajority_required: true,
            },
            Self::MemberAction => CategoryRequirements {
                min_approval: 0.75,
                timelock: Duration::hours(24),
                supermajority_required: true,
            },
            Self::GovernanceChange => CategoryRequirements {
                min_approval: 0.80,
                timelock: Duration::days(7),
                supermajority_required: true,
            },
            Self::PackageAction => CategoryRequirements {
                min_approval: 0.50,
                timelock: Duration::hours(24),
                supermajority_required: false,
            },
            Self::RealmSettings => CategoryRequirements {
                min_approval: 0.50,
                timelock: Duration::hours(12),
                supermajority_required: false,
            },
            Self::Custom(_) => CategoryRequirements {
                min_approval: 0.50,
                timelock: Duration::hours(24),
                supermajority_required: false,
            },
        }
    }
}
```

---

## 8. Integration mit Pluto-Komponenten

### 8.1 Governance × Trust (Κ24)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   GOVERNANCE × TRUST: BIDIREKTIONALE KOPPLUNG                                ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Trust → Governance:                                                        ║
║   - Relativer Trust beeinflusst Stimmgewicht                                ║
║   - Min-Trust für Proposal-Erstellung                                       ║
║   - Trust-Dimensionen gewichtet nach Realm-Policy                           ║
║                                                                              ║
║   Governance → Trust:                                                        ║
║   - Erfolgreiche Proposals → Trust ↑ (für Autor)                            ║
║   - Abgelehnte Proposals → Trust ↓ (leichter Malus)                         ║
║   - Spam-Proposals → Trust ↓↓ (starker Malus)                               ║
║   - Voting-Participation → Trust ↑ (Engagement belohnt)                     ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 8.2 Governance × Gas/Mana

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   GOVERNANCE × GAS/MANA: RESOURCE-KOSTEN                                     ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Proposal erstellen:                                                        ║
║   - proposal_mana_cost (Anti-Spam)                                          ║
║   - Token-Deposit (refundable)                                              ║
║                                                                              ║
║   Voting:                                                                    ║
║   - vote_mana_cost (minimal, fördert Participation)                         ║
║                                                                              ║
║   Execution:                                                                 ║
║   - execution_gas_limit (ECLVM-Limit)                                       ║
║   - execution_mana_limit (I/O-Limit)                                        ║
║                                                                              ║
║   Beispiel:                                                                  ║
║   - Proposal-Erstellung: 500 Mana + 50 Token Deposit                        ║
║   - Vote: 10 Mana                                                           ║
║   - Execution: bis zu 100.000 Gas, 10.000 Mana                              ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 8.3 Governance × Identity (Multi-DID)

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║   GOVERNANCE × IDENTITY: REALM-SUB-DID                                       ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Jeder Member hat eine Realm-spezifische Sub-DID:                          ║
║                                                                              ║
║   Root-DID: did:erynoa:self:alice123...                                     ║
║       │                                                                      ║
║       └── Realm-DID: did:erynoa:circle:dao-alice...                         ║
║           - Governance-Aktionen werden mit Realm-DID signiert               ║
║           - Sybil-Schutz: Eine Root-DID = Eine Realm-DID                    ║
║           - Privacy: Voting-Verhalten nicht über Realms korrelierbar        ║
║                                                                              ║
║   Vorteile:                                                                  ║
║   - Isolierte Governance-Historie pro Realm                                 ║
║   - Root-DID bleibt privat                                                  ║
║   - Revocation einer Realm-DID ≠ Verlust der Root-Identity                  ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 9. StateEvents für Governance

```rust
pub enum StateEvent {
    // ═══════════════════════════════════════════════════════════════════════
    // PROPOSAL LIFECYCLE
    // ═══════════════════════════════════════════════════════════════════════
    ProposalCreated {
        realm_id: UniversalId,
        proposal_id: String,
        author_did: UniversalId,
        category: ProposalCategory,
        title: String,
        mana_spent: u64,
        deposit_locked: u64,
    },

    ProposalStateChanged {
        realm_id: UniversalId,
        proposal_id: String,
        old_state: ProposalState,
        new_state: ProposalState,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // VOTING
    // ═══════════════════════════════════════════════════════════════════════
    VoteCast {
        realm_id: UniversalId,
        proposal_id: String,
        voter_did: UniversalId,
        vote: Vote,  // For, Against, Abstain
        weight: VoteWeight,
        mana_spent: u64,
    },

    VoteChanged {
        realm_id: UniversalId,
        proposal_id: String,
        voter_did: UniversalId,
        old_vote: Vote,
        new_vote: Vote,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // DELEGATION
    // ═══════════════════════════════════════════════════════════════════════
    DelegationCreated {
        realm_id: UniversalId,
        delegator_did: UniversalId,
        delegate_did: UniversalId,
        categories: Option<Vec<ProposalCategory>>,
        expires_at: Option<DateTime>,
    },

    DelegationRevoked {
        realm_id: UniversalId,
        delegator_did: UniversalId,
        delegate_did: UniversalId,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // EXECUTION
    // ═══════════════════════════════════════════════════════════════════════
    ProposalExecuted {
        realm_id: UniversalId,
        proposal_id: String,
        gas_used: u64,
        mana_used: u64,
        success: bool,
        error: Option<String>,
    },

    ProposalVetoed {
        realm_id: UniversalId,
        proposal_id: String,
        veto_votes: f64,
        veto_threshold: f64,
    },

    // ═══════════════════════════════════════════════════════════════════════
    // TRUST UPDATES (als Folge von Governance-Aktionen)
    // ═══════════════════════════════════════════════════════════════════════
    GovernanceTrustUpdate {
        realm_id: UniversalId,
        member_did: UniversalId,
        reason: GovernanceTrustReason,
        delta: f64,
    },
}

/// Gründe für Trust-Updates durch Governance
#[derive(Debug, Clone)]
pub enum GovernanceTrustReason {
    ProposalAccepted,        // +0.02
    ProposalRejected,        // -0.01
    ProposalSpam,            // -0.1
    VotingParticipation,     // +0.005
    DelegationReceived,      // +0.01 (Vertrauensbeweis)
    SuccessfulVeto,          // +0.02 (schützte Community)
}
```

---

## 10. Sicherheits-Mechanismen

### 10.1 Anti-Sybil durch Trust

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    SYBIL-SCHUTZ DURCH TRUST                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Problem: Angreifer erstellt viele Fake-Accounts                           ║
║                                                                              ║
║   Lösung durch Trust-Mechanismen:                                           ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                     │   ║
║   │   1. Newcomer-Trust: Neue Accounts starten mit Trust 0.1            │   ║
║   │      → T_rel = (0.1 - 0.6) / 0.6 = -0.83                           │   ║
║   │      → Mit α=0.5: Multiplier = 0.58                                │   ║
║   │      → Stark reduziertes Stimmgewicht                              │   ║
║   │                                                                     │   ║
║   │   2. Min-Trust für Proposals: z.B. 0.5                             │   ║
║   │      → Sybils können keine Proposals erstellen                     │   ║
║   │                                                                     │   ║
║   │   3. Mana-Limit: Niedrig-Trust-Accounts haben weniger Mana         │   ║
║   │      → Weniger Voting-Aktionen möglich                             │   ║
║   │                                                                     │   ║
║   │   4. Asymmetrische Trust-Evolution (Κ4):                           │   ║
║   │      → Trust aufzubauen dauert lange                               │   ║
║   │      → Trust-Farming ist teuer (Zeit + Engagement)                 │   ║
║   │                                                                     │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### 10.2 Veto-Mechanismus

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    VETO: MINDERHEITENSCHUTZ                                  ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   Nach Approval eines Proposals gibt es ein Veto-Window:                    ║
║                                                                              ║
║   ┌─────────────────────────────────────────────────────────────────────┐   ║
║   │                                                                     │   ║
║   │   Timelock-Periode (z.B. 24h)                                       │   ║
║   │   │                                                                 │   ║
║   │   ├── Veto-Window (z.B. 12h)                                        │   ║
║   │   │   - Members können VETO stimmen                                │   ║
║   │   │   - Veto-Threshold: 33% der vote_power                         │   ║
║   │   │   - Bei Erreichen: Proposal wird VETOED                        │   ║
║   │   │                                                                 │   ║
║   │   └── Execution-Window (restliche 12h)                             │   ║
║   │       - Wenn kein Veto: Execution startet                          │   ║
║   │                                                                     │   ║
║   └─────────────────────────────────────────────────────────────────────┘   ║
║                                                                              ║
║   Veto schützt gegen:                                                        ║
║   - Übereilte Mehrheits-Entscheidungen                                      ║
║   - Last-Minute-Manipulationen                                              ║
║   - Proposals die Minderheiten schaden                                      ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

---

## 11. Verzeichnisstruktur

```text
backend/src/nervous_system/realm/governance/
│
├── mod.rs                    # Re-exports
│
├── types/
│   ├── governance_type.rs    # GovernanceType enum
│   ├── proposal.rs           # Proposal, ProposalState
│   ├── vote.rs               # Vote, VoteWeight
│   └── delegation.rs         # Delegation
│
├── config/
│   ├── governance_config.rs  # GovernanceConfig
│   ├── trust_weights.rs      # TrustDimensionWeights
│   └── category_config.rs    # CategoryConfig
│
├── calculation/
│   ├── vote_weight.rs        # calculate_vote_weight()
│   ├── relative_trust.rs     # calculate_relative_trust()
│   ├── quorum.rs             # check_quorum()
│   └── delegation.rs         # calculate_delegated_weight()
│
├── lifecycle/
│   ├── create.rs             # create_proposal()
│   ├── vote.rs               # cast_vote(), change_vote()
│   ├── execute.rs            # execute_proposal()
│   └── veto.rs               # cast_veto(), check_veto()
│
├── events/
│   └── governance_events.rs  # StateEvents
│
└── policies/
    ├── governance_policy.rs  # ECL-Policy-Parser
    └── builtin_policies.rs   # Default-Policies
```

---

## 12. Zusammenfassung

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║                    REALM-GOVERNANCE: KERNPUNKTE                              ║
╠══════════════════════════════════════════════════════════════════════════════╣
║                                                                              ║
║   🏛️ REALM-EXKLUSIV                                                         ║
║      → Nur Realms können Governance haben                                   ║
║      → Keine globale oder Identity-Governance                               ║
║                                                                              ║
║   📊 STIMMGEWICHT-FORMEL                                                     ║
║      → W(m) = G(m) × (1 + α × T_rel(m))                                     ║
║      → Basis-Gewicht × Trust-Modifikator                                    ║
║                                                                              ║
║   🔄 RELATIVER TRUST                                                         ║
║      → T_rel = (T - T_avg) / T_avg                                          ║
║      → Differenziert zwischen Members                                       ║
║      → Fair: Über Durchschnitt = Bonus, unter = Malus                       ║
║                                                                              ║
║   🎛️ KONFIGURIERBARER TRUST-EINFLUSS (α)                                    ║
║      → α = 0: Kein Trust-Einfluss                                           ║
║      → α = 0.5: Moderater Einfluss (±50%)                                   ║
║      → α = 1.0: Maximaler Einfluss (±100%)                                  ║
║                                                                              ║
║   🗳️ GOVERNANCE-TYPEN                                                        ║
║      → Quadratic: √tokens × Trust-Multiplier                                ║
║      → Token: tokens × Trust-Multiplier                                     ║
║      → Reputation: Trust = Stimme                                           ║
║      → Delegated: Liquid Democracy mit Trust-Decay                          ║
║      → MemberEqual: 1 Member = 1 Vote                                       ║
║                                                                              ║
║   🛡️ SICHERHEIT                                                              ║
║      → Anti-Sybil durch Trust-Mechanismen                                   ║
║      → Veto-Mechanismus für Minderheitenschutz                              ║
║      → Mana-Kosten gegen Spam                                               ║
║                                                                              ║
║   🔗 PLUTO-INTEGRATION                                                       ║
║      → Trust: Bidirektionale Kopplung                                       ║
║      → Gas/Mana: Resource-Limits für Governance                             ║
║      → Identity: Realm-Sub-DIDs für Privacy                                 ║
║      → Events: Vollständiges Event-Sourcing                                 ║
║                                                                              ║
╚══════════════════════════════════════════════════════════════════════════════╝
```
