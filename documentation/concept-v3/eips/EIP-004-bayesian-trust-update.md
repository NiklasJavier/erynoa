# EIP-004: Bayesian Trust Update Algorithm

> **EIP:** 004
> **Titel:** Bayesian Trust Update Algorithm
> **Status:** Draft
> **Version:** 0.2
> **Typ:** Standard
> **Ebene:** E2 (Emergenz)
> **Erstellt:** Januar 2026
> **Aktualisiert:** Januar 2026
> **Abhängigkeiten:** EIP-001 (DID:erynoa), EIP-002 (Trust Vector 6D), EIP-003 (Event-DAG)
> **Verwandt:** EIP-006 (Slashing & Dispute Resolution) [Planned]

---

## Abstract

Diese Spezifikation definiert den **Bayesian Trust Update Algorithm** für das Erynoa-Protokoll. Der Algorithmus beschreibt, wie der 6-dimensionale Trust-Vektor W(s) basierend auf Events, Attestationen und **Staked Guardianship** aktualisiert wird.

Kernkonzepte:
- **Bayessche Inferenz** – Trust als Wahrscheinlichkeitsverteilung, nicht als Zahl
- **Prior-Posterior-Update** – Jedes Event aktualisiert den Prior zum Posterior
- **Konfidenzintervalle** – Explizite Unsicherheitsmodellierung
- **Staked Guardianship** – Trust-Vererbung durch institutionelle Bürgschaft
- **Asymmetrisches Update** – Verlust wiegt schwerer als Gewinn
- **Trust-Decay** – Inaktivität führt zu Verfall Richtung Neutral
- **Trust-Floor** – Niemand fällt unter 0.3 (Rehabilitation möglich)

---

## Motivation

Ein dezentrales Vertrauenssystem benötigt:

1. **Mathematische Fundierung** – Keine ad-hoc Formeln, sondern Wahrscheinlichkeitstheorie
2. **Unsicherheitsmodellierung** – Unterscheidung zwischen "Trust 0.7 sicher" und "Trust 0.7 unsicher"
3. **Cold-Start-Lösung** – Neue Nutzer müssen sofort nutzbar sein
4. **Incentive-Alignment** – Bürgen müssen Anreize haben, sorgfältig zu prüfen
5. **Slashing** – Fehlverhalten muss Konsequenzen für Bürgen haben
6. **Fairness** – Vergangene Fehler dürfen nicht ewig bestrafen

---

## Spezifikation

### 1. Trust als Wahrscheinlichkeitsverteilung

#### 1.1 Beta-Verteilung für jede Dimension

Jede Dimension des Trust-Vektors wird als Beta-Verteilung modelliert:

```
Wᵢ(s) ~ Beta(αᵢ, βᵢ)
```

Die Beta-Verteilung hat zwei Parameter:
- **α** ("Erfolge" + Prior): Je höher, desto mehr positive Evidenz
- **β** ("Misserfolge" + Prior): Je höher, desto mehr negative Evidenz

```rust
/// Trust-Dimension als Beta-Verteilung
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustDimension {
    /// "Erfolge" + Prior
    pub alpha: f64,
    
    /// "Misserfolge" + Prior
    pub beta: f64,
    
    /// Abgeleitete Werte (cached)
    pub expected_value: f64,      // α / (α + β)
    pub variance: f64,            // αβ / ((α+β)²(α+β+1))
    pub confidence: f64,          // 1 - CI_width
}

impl TrustDimension {
    /// Neutraler Prior: Beta(2, 2)
    pub fn neutral() -> Self {
        Self::from_params(2.0, 2.0)
    }
    
    /// Erstellt aus α und β
    pub fn from_params(alpha: f64, beta: f64) -> Self {
        let sum = alpha + beta;
        let expected = alpha / sum;
        let variance = (alpha * beta) / (sum * sum * (sum + 1.0));
        let ci_width = Self::confidence_interval_width(alpha, beta);
        
        Self {
            alpha,
            beta,
            expected_value: expected,
            variance,
            confidence: 1.0 - ci_width,
        }
    }
    
    /// 95% Konfidenzintervall-Breite
    fn confidence_interval_width(alpha: f64, beta: f64) -> f64 {
        let dist = Beta::new(alpha, beta).unwrap();
        let lower = dist.inverse_cdf(0.025);
        let upper = dist.inverse_cdf(0.975);
        upper - lower
    }
    
    /// Bayessches Update
    pub fn update(&mut self, success: f64, weight: f64) {
        self.alpha += success * weight;
        self.beta += (1.0 - success) * weight;
        self.recalculate();
    }
    
    fn recalculate(&mut self) {
        let sum = self.alpha + self.beta;
        self.expected_value = self.alpha / sum;
        self.variance = (self.alpha * self.beta) / (sum * sum * (sum + 1.0));
        self.confidence = 1.0 - Self::confidence_interval_width(self.alpha, self.beta);
    }
}
```

#### 1.2 Der vollständige Trust-State

```rust
/// Vollständiger Trust-Zustand eines Agenten
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustState {
    /// Die 6 Dimensionen als Beta-Verteilungen
    pub r: TrustDimension,  // Reliability
    pub i: TrustDimension,  // Integrity
    pub c: TrustDimension,  // Competence
    pub p: TrustDimension,  // Predictability
    pub v: TrustDimension,  // Vigilance
    pub omega: TrustDimension, // Omega-Alignment
    
    /// Staked Guardianship (V0.3)
    pub derived_trust: Option<DerivedTrust>,
    
    /// Letzte Aktualisierung
    pub last_updated: u64,
    
    /// Anzahl Events in der Historie
    pub event_count: u64,
}

/// Trust-Vererbung durch Guardians (V0.3)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DerivedTrust {
    /// Quelle des abgeleiteten Trusts
    pub source: DID,
    
    /// Boost-Faktor (typisch 0.2-0.3)
    pub boost_factor: f64,
    
    /// Gestakter Betrag (ERY Tokens oder Reputation Points)
    pub stake: StakeAmount,
    
    /// Zeitpunkt des Stakings
    pub staked_at: u64,
    
    /// Liability-Level
    pub liability: LiabilityLevel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StakeAmount {
    Tokens(u64),           // ERY Tokens
    ReputationPoints(f64), // Prozent der eigenen Reputation
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LiabilityLevel {
    None,       // Keine Haftung (nur Recovery)
    Partial,    // Teilweise Haftung (25% Trust-Loss bei Betrug)
    Full,       // Volle Haftung (100% Trust-Loss bei Betrug)
}
```

### 2. Staked Guardianship (Cold-Start-Lösung)

#### 2.1 Das Konzept

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      STAKED GUARDIANSHIP                                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   PROBLEM: Neue Nutzer haben Trust = 0.5 (neutral).                    │
│            Sie können keine High-Value-Transaktionen durchführen.       │
│                                                                         │
│   LÖSUNG: Institutionelle Guardians (Banken, Notare) bürgen für sie.   │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   MECHANIK:                                                             │
│                                                                         │
│   1. LINK: Alice trägt Bank als Guardian ein                           │
│   2. VOUCH: Bank signiert UND "staked" Reputation auf Alice            │
│   3. INHERIT: Alice erbt sofort Teil der Reputation                    │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   FORMEL:                                                               │
│                                                                         │
│   T_user(t) = T_base + β × (T_guardian × S_stake)                      │
│                                                                         │
│   Wobei:                                                                │
│   - T_base = Eigene Reputation (startet bei 0.5)                       │
│   - β = Dämpfungsfaktor (0.3)                                          │
│   - T_guardian = Trust des Guardians (z.B. 0.9 für Bank)               │
│   - S_stake = Stake-Faktor (0-1, je nach Einsatz)                      │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   BEISPIEL:                                                             │
│                                                                         │
│   Bank (T=0.9) staked 500 ERY auf Alice                                │
│   S_stake = 0.5 (mittlerer Einsatz)                                    │
│                                                                         │
│   T_alice = 0.5 + 0.3 × (0.9 × 0.5)                                    │
│           = 0.5 + 0.135                                                │
│           = 0.635                                                       │
│                                                                         │
│   Alice startet nicht bei 0.5 (Neutral), sondern bei 0.635 (Verified)! │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 2.2 Trust-Vererbungs-Algorithmus

```rust
const INHERITANCE_DAMPING: f64 = 0.3;  // β - Max 30% des Guardian-Trusts
const MAX_GUARDIANS_FOR_BOOST: usize = 3;  // Max 3 Guardians zählen
const MAX_TRUST_CHAIN_DEPTH: usize = 5;   // Max Tiefe für Trust-Ketten

/// Berechnet effektiven Trust unter Berücksichtigung von Guardians
/// MIT LOOP-DETECTION (verhindert zirkuläre Trust-Pumping)
pub fn calculate_effective_trust(
    subject: &DID,
    base_trust: &TrustState,
    guardians: &[StakedGuardian],
    guardian_trust_cache: &HashMap<DID, TrustState>,
    visited: &mut HashSet<DID>,  // V0.2: Loop Detection
) -> TrustState {
    let mut effective = base_trust.clone();
    
    // V0.2: LOOP DETECTION - Verhindert A→B→A Zyklen
    if visited.contains(subject) {
        // Zyklus erkannt! Kein Trust-Boost für zirkuläre Referenzen
        return effective;
    }
    visited.insert(subject.clone());
    
    // V0.2: DEPTH LIMIT - Verhindert zu lange Ketten
    if visited.len() > MAX_TRUST_CHAIN_DEPTH {
        return effective;
    }
    
    // Sortiere Guardians nach Trust (beste zuerst)
    let mut sorted_guardians: Vec<_> = guardians.iter()
        .filter(|g| g.endorsement.is_some())
        // V0.2: Filtere Guardians die bereits in der Kette sind
        .filter(|g| !visited.contains(&g.did))
        .collect();
    sorted_guardians.sort_by(|a, b| {
        let ta = guardian_trust_cache.get(&a.did).map(|t| t.scalar()).unwrap_or(0.0);
        let tb = guardian_trust_cache.get(&b.did).map(|t| t.scalar()).unwrap_or(0.0);
        tb.partial_cmp(&ta).unwrap_or(Ordering::Equal)
    });
    
    // Nimm nur die Top-3
    let top_guardians = sorted_guardians.iter().take(MAX_GUARDIANS_FOR_BOOST);
    
    let mut total_boost = TrustBoost::zero();
    
    for guardian in top_guardians {
        if let Some(endorsement) = &guardian.endorsement {
            let guardian_trust = guardian_trust_cache.get(&guardian.did)
                .expect("Guardian trust must be cached");
            
            let stake_factor = match &endorsement.stake {
                StakeAmount::Tokens(amount) => calculate_token_stake_factor(*amount),
                StakeAmount::ReputationPoints(pct) => *pct,
            };
            
            // Boost für jede Dimension
            let boost = TrustBoost {
                r: INHERITANCE_DAMPING * guardian_trust.r.expected_value * stake_factor,
                i: INHERITANCE_DAMPING * guardian_trust.i.expected_value * stake_factor,
                c: INHERITANCE_DAMPING * guardian_trust.c.expected_value * stake_factor,
                p: INHERITANCE_DAMPING * guardian_trust.p.expected_value * stake_factor,
                v: INHERITANCE_DAMPING * guardian_trust.v.expected_value * stake_factor,
                omega: INHERITANCE_DAMPING * guardian_trust.omega.expected_value * stake_factor,
            };
            
            total_boost = total_boost.add(&boost);
        }
    }
    
    // Addiere Boost (mit Ceiling bei 0.95)
    effective.r.expected_value = (base_trust.r.expected_value + total_boost.r).min(0.95);
    effective.i.expected_value = (base_trust.i.expected_value + total_boost.i).min(0.95);
    effective.c.expected_value = (base_trust.c.expected_value + total_boost.c).min(0.95);
    effective.p.expected_value = (base_trust.p.expected_value + total_boost.p).min(0.95);
    effective.v.expected_value = (base_trust.v.expected_value + total_boost.v).min(0.95);
    effective.omega.expected_value = (base_trust.omega.expected_value + total_boost.omega).min(0.95);
    
    // Markiere abgeleiteten Trust
    if !sorted_guardians.is_empty() {
        effective.derived_trust = Some(DerivedTrust {
            source: sorted_guardians[0].did.clone(),
            boost_factor: total_boost.scalar(),
            stake: sorted_guardians[0].endorsement.as_ref().unwrap().stake.clone(),
            staked_at: now_ms(),
            liability: sorted_guardians[0].endorsement.as_ref().unwrap().liability.clone(),
        });
    }
    
    effective
}

fn calculate_token_stake_factor(tokens: u64) -> f64 {
    // Logarithmische Skalierung: 100 ERY = 0.3, 1000 ERY = 0.6, 10000 ERY = 0.9
    let log_tokens = (tokens as f64).ln();
    let log_100 = 100_f64.ln();
    let log_10000 = 10000_f64.ln();
    
    ((log_tokens - log_100) / (log_10000 - log_100) * 0.6 + 0.3)
        .clamp(0.1, 1.0)
}
```

#### 2.3 Loop Detection & Circular Reference Protection (V0.2)

**Problem:** Bank A bürgt für Bank B, Bank B bürgt für Bank A. Beide pumpen ihren Trust künstlich hoch.

**Lösung:** PageRank-ähnliche Loop-Detection:

```rust
/// Detektiert zirkuläre Bürgschafts-Ketten
pub fn detect_circular_endorsement(
    subject: &DID,
    endorsement_graph: &EndorsementGraph,
) -> Option<CircularChain> {
    let mut visited = HashSet::new();
    let mut path = Vec::new();
    
    fn dfs(
        current: &DID,
        target: &DID,
        graph: &EndorsementGraph,
        visited: &mut HashSet<DID>,
        path: &mut Vec<DID>,
    ) -> Option<CircularChain> {
        if current == target && !path.is_empty() {
            return Some(CircularChain { dids: path.clone() });
        }
        
        if visited.contains(current) {
            return None;
        }
        
        visited.insert(current.clone());
        path.push(current.clone());
        
        for guardian in graph.get_guardians(current) {
            if let Some(chain) = dfs(&guardian, target, graph, visited, path) {
                return Some(chain);
            }
        }
        
        path.pop();
        None
    }
    
    dfs(subject, subject, endorsement_graph, &mut visited, &mut path)
}

/// Wird bei Guardian-Registrierung aufgerufen
pub fn validate_endorsement(
    new_guardian: &DID,
    ward: &DID,
    graph: &EndorsementGraph,
) -> Result<(), EndorsementError> {
    // 1. Prüfe, ob der Guardian selbst vom Ward gebürgt wird
    if graph.get_guardians(new_guardian).contains(ward) {
        return Err(EndorsementError::DirectCircularReference);
    }
    
    // 2. Prüfe auf indirekte Zyklen
    let mut test_graph = graph.clone();
    test_graph.add_edge(new_guardian, ward);
    
    if detect_circular_endorsement(ward, &test_graph).is_some() {
        return Err(EndorsementError::IndirectCircularReference);
    }
    
    Ok(())
}
```

**Regeln:**

1. **Direkte Zyklen verboten:** A→B UND B→A ist nicht erlaubt
2. **Indirekte Zyklen verboten:** A→B→C→A ist nicht erlaubt
3. **Maximale Ketten-Tiefe:** 5 Stufen (verhindert "Trust-Laundering")
4. **Periodischer Audit:** Background-Job prüft alle 24h auf neue Zyklen

#### 2.4 Privacy-Preserving Guardianship (V0.2)

**Problem:** Wenn `did:erynoa:guild:sparkasse` öffentlich im DID-Dokument steht, weiß jeder, dass Alice bei der Sparkasse ist.

**Lösung:** Selective Disclosure mit ZK-Proofs

```rust
/// Guardianship kann privat oder öffentlich sein
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GuardianshipVisibility {
    /// Vollständig öffentlich (Guardian-DID sichtbar)
    Public,
    
    /// Privat mit ZK-Proof (nur "ist gebürgt von Tier-1 Bank" sichtbar)
    Private {
        /// ZK-Proof: "Guardian ist in Set {Tier1Banks}" ohne zu verraten welche
        zk_proof: ZkProof,
        /// Tier-Level (ohne konkreten Guardian)
        tier_level: GuardianTier,
        /// Commitment (für spätere selektive Enthüllung)
        commitment: [u8; 32],
    },
}

#[derive(Clone, Debug)]
pub enum GuardianTier {
    Tier1Institutional,  // Zentralbanken, Staatsorgane
    Tier2Institutional,  // Große Privatbanken, Notare
    Tier3Institutional,  // Regionale Banken, Anwälte
    Personal,            // Freunde, Familie
}

/// Generiert ZK-Proof für Guardianship
pub fn generate_guardianship_proof(
    guardian: &DID,
    tier: GuardianTier,
    tier_membership_list: &[DID],
) -> Result<ZkProof, ZkError> {
    // ZK-Proof: "Ich kenne ein DID X, sodass:"
    // 1. X ist in tier_membership_list
    // 2. X hat für mich gebürgt
    // OHNE X zu verraten
    
    let circuit = GuardianshipMembershipCircuit::new(
        guardian,
        tier_membership_list,
    );
    
    circuit.prove()
}

/// Selektive Enthüllung (z.B. für Autovermietung)
pub fn reveal_guardianship(
    commitment: &[u8; 32],
    guardian: &DID,
    reveal_to: &DID,
) -> RevealProof {
    // Signierte Aussage: "Für DID {reveal_to} enthülle ich: Mein Guardian ist {guardian}"
    // Nur der Empfänger kann das verifizieren
    RevealProof {
        commitment: *commitment,
        guardian: guardian.clone(),
        revealed_to: reveal_to.clone(),
        timestamp: now_ms(),
        signature: vec![],  // Signiert vom Ward
    }
}
```

**DID-Dokument mit Privacy:**

```json
{
  "erynoa": {
    "recovery": {
      "method": "social-staked",
      "threshold": 2,
      "guardians": [
        {
          "visibility": "private",
          "tier": "Tier1Institutional",
          "zkProof": "z8aGdRnI...",
          "commitment": "0xabc123..."
        },
        {
          "did": "did:erynoa:self:bob-friend",
          "visibility": "public",
          "role": "personal"
        }
      ]
    },
    "trustDerived": {
      "level": "Verified",
      "source": "private",
      "proof": "z9bHeSnJ..."
    }
  }
}
```

#### 2.5 DID-Dokument Erweiterung

```json
{
  "id": "did:erynoa:self:alice-2024-abc123",
  "erynoa": {
    "recovery": {
      "method": "social-staked",
      "threshold": 2,
      "guardians": [
        {
          "did": "did:erynoa:guild:sparkasse-berlin",
          "role": "institutional",
          "endorsement": {
            "level": "kyc-level-3",
            "stake": {
              "type": "tokens",
              "amount": 500
            },
            "liability": "full",
            "signature": "z4GdRnI..."
          }
        },
        {
          "did": "did:erynoa:self:bob-friend",
          "role": "personal",
          "endorsement": null
        },
        {
          "did": "did:erynoa:guild:notar-office-muc",
          "role": "institutional",
          "endorsement": {
            "level": "notarized",
            "stake": {
              "type": "reputation",
              "percentage": 0.1
            },
            "liability": "partial",
            "signature": "z5HeSnJ..."
          }
        }
      ]
    },
    "trustDerived": {
      "sources": [
        {
          "guardian": "did:erynoa:guild:sparkasse-berlin",
          "boost": 0.135,
          "since": "2026-01-15T10:30:00Z"
        }
      ],
      "totalBoost": 0.15,
      "effectiveLevel": "Verified"
    }
  }
}
```

### 3. Slashing (Guardian-Haftung)

#### 3.1 Slashing-Mechanik

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         SLASHING-MECHANIK                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   TRIGGER: User (gebürgt von Guardian) wird beim Betrug erwischt       │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   SLASHING-BERECHNUNG:                                                  │
│                                                                         │
│   Δ_guardian = -liability_factor × severity × stake_factor             │
│                                                                         │
│   Wobei:                                                                │
│   - liability_factor: 0 (none), 0.25 (partial), 1.0 (full)             │
│   - severity: Schwere des Vergehens (0.1 minor, 0.5 major, 1.0 fraud)  │
│   - stake_factor: Anteil des Stakes am Gesamt-Trust                    │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   BEISPIEL:                                                             │
│                                                                         │
│   Bank bürgt für Scammer mit 500 ERY (full liability)                  │
│   Scammer begeht Major Fraud (severity = 0.8)                          │
│                                                                         │
│   Δ_bank = -1.0 × 0.8 × 0.1 = -0.08                                    │
│                                                                         │
│   Bank verliert 8% ihrer Integrity-Dimension!                          │
│                                                                         │
│   ──────────────────────────────────────────────────────────────────    │
│                                                                         │
│   CHAIN REACTION:                                                       │
│                                                                         │
│   Wenn Bank viele User gebürgt hat, sinkt deren Trust minimal,         │
│   weil der abgeleitete Boost von der Bank abhängt.                     │
│                                                                         │
│   User verliert Trust-Boost:                                            │
│   T_user_new = T_base + β × (T_guardian_new × S_stake)                 │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 3.2 Slashing-Implementierung

```rust
pub struct SlashingEvent {
    /// Der Übeltäter
    pub offender: DID,
    
    /// Der Tatbestand
    pub offense: OffenseType,
    
    /// Schwere (0-1)
    pub severity: f64,
    
    /// Betroffene Trust-Dimension
    pub dimension: TrustDimension,
    
    /// Beweis-Events
    pub evidence: Vec<EventId>,
}

#[derive(Clone, Debug)]
pub enum OffenseType {
    MinorViolation,      // severity ~ 0.1
    PolicyBreach,        // severity ~ 0.3
    MajorViolation,      // severity ~ 0.5
    Fraud,               // severity ~ 0.8
    CriticalFraud,       // severity = 1.0
}

pub fn process_slashing(
    slashing: &SlashingEvent,
    trust_db: &mut TrustDatabase,
) -> Result<SlashingResult, SlashingError> {
    // 1. Offender bestrafen
    let offender_trust = trust_db.get_mut(&slashing.offender)?;
    let offender_delta = apply_offense_to_trust(offender_trust, slashing);
    
    // 2. Guardians identifizieren und slashen
    let guardians = get_staked_guardians(&slashing.offender)?;
    let mut guardian_deltas = Vec::new();
    
    for guardian in guardians {
        if let Some(endorsement) = &guardian.endorsement {
            let liability_factor = match endorsement.liability {
                LiabilityLevel::None => 0.0,
                LiabilityLevel::Partial => 0.25,
                LiabilityLevel::Full => 1.0,
            };
            
            if liability_factor > 0.0 {
                let guardian_trust = trust_db.get_mut(&guardian.did)?;
                
                let stake_factor = match &endorsement.stake {
                    StakeAmount::Tokens(amount) => calculate_token_stake_factor(*amount),
                    StakeAmount::ReputationPoints(pct) => *pct,
                };
                
                // Slashing-Betrag berechnen
                let slash_amount = liability_factor * slashing.severity * stake_factor * 0.1;
                
                // Primär Integrity betroffen (Guardian hat falsch gebürgt)
                guardian_trust.i.update(0.0, slash_amount * guardian_trust.i.alpha);
                
                // Sekundär Reliability (Guardian ist weniger zuverlässig)
                guardian_trust.r.update(0.0, slash_amount * 0.5 * guardian_trust.r.alpha);
                
                guardian_deltas.push(GuardianSlashResult {
                    guardian: guardian.did.clone(),
                    delta_i: -slash_amount,
                    delta_r: -slash_amount * 0.5,
                });
                
                // Token-Stake verbrennen (falls applicable)
                if let StakeAmount::Tokens(amount) = &endorsement.stake {
                    let burn_amount = (*amount as f64 * slashing.severity) as u64;
                    burn_tokens(&guardian.did, burn_amount)?;
                }
            }
        }
    }
    
    // 3. Chain Reaction: Alle von diesem Guardian gebürgten User
    for guardian_delta in &guardian_deltas {
        propagate_trust_change(&guardian_delta.guardian, trust_db)?;
    }
    
    Ok(SlashingResult {
        offender: slashing.offender.clone(),
        offender_delta,
        guardian_deltas,
    })
}

fn propagate_trust_change(guardian: &DID, trust_db: &mut TrustDatabase) -> Result<(), Error> {
    // Finde alle Users, die von diesem Guardian gebürgt wurden
    let dependents = find_dependents(guardian)?;
    
    for dependent in dependents {
        // Recalculate effective trust
        let base_trust = trust_db.get(&dependent)?;
        let guardians = get_staked_guardians(&dependent)?;
        let guardian_cache = build_guardian_cache(&guardians, trust_db)?;
        
        let new_effective = calculate_effective_trust(&base_trust, &guardians, &guardian_cache);
        
        // Update derived trust
        trust_db.update_derived(&dependent, new_effective.derived_trust)?;
    }
    
    Ok(())
}
```

### 4. Event-basiertes Bayesian Update

#### 4.1 Update-Regeln pro Event-Typ

```rust
/// Wendet Event auf Trust-State an
pub fn apply_event(
    trust: &mut TrustState,
    event: &Event,
    context: &EventContext,
) -> TrustDelta {
    let mut delta = TrustDelta::zero();
    
    match &event.payload {
        // === Transaktionen ===
        EventPayload::TransactionClose { outcome, .. } => {
            match outcome {
                TransactionOutcome::Success => {
                    // Reliability: Verpflichtung erfüllt
                    trust.r.update(1.0, 1.0);
                    delta.r = trust.r.expected_value - delta.r;
                    
                    // Omega: Regelkonform
                    trust.omega.update(1.0, 0.5);
                }
                TransactionOutcome::PartialSuccess { completion_pct } => {
                    trust.r.update(*completion_pct, 1.0);
                }
                TransactionOutcome::Failure { blamed } => {
                    if *blamed {
                        // Asymmetrisch: Negativ wiegt 4x schwerer
                        trust.r.update(0.0, 4.0);
                        trust.omega.update(0.0, 2.0);
                    }
                }
            }
        }
        
        EventPayload::TransactionAbort { reason, blamed, .. } => {
            if *blamed {
                trust.r.update(0.0, 2.0);
                trust.p.update(0.0, 1.0);  // Unvorhersagbar
            }
        }
        
        // === Attestationen ===
        EventPayload::TrustAttestation { subject, dimension, value, .. } => {
            // Attester muss vertrauenswürdig sein
            let attester_trust = context.attester_trust.unwrap_or(0.5);
            let weight = attester_trust * 0.5;  // Gedämpft
            
            match dimension {
                TrustDimensionType::Reliability => trust.r.update(*value, weight),
                TrustDimensionType::Integrity => trust.i.update(*value, weight),
                TrustDimensionType::Competence => trust.c.update(*value, weight),
                TrustDimensionType::Predictability => trust.p.update(*value, weight),
                TrustDimensionType::Vigilance => trust.v.update(*value, weight),
                TrustDimensionType::OmegaAlignment => trust.omega.update(*value, weight),
            }
        }
        
        // === Integrität ===
        EventPayload::CredentialVerified { valid, .. } => {
            if *valid {
                trust.i.update(1.0, 1.0);
            } else {
                // Falsche Credential: schwere Integritätsverletzung
                trust.i.update(0.0, 10.0);
            }
        }
        
        // === Vigilance ===
        EventPayload::AnomalyReport { confirmed, severity, .. } => {
            if *confirmed {
                trust.v.update(1.0, severity.weight());
            } else {
                // False Positive
                trust.v.update(0.0, 0.5);
            }
        }
        
        // === Governance ===
        EventPayload::GovernanceVote { .. } => {
            // Teilnahme an Governance ist positiv
            trust.omega.update(1.0, 0.3);
        }
        
        EventPayload::GovernancePropose { accepted, .. } => {
            if *accepted {
                trust.c.update(1.0, 1.0);  // Kompetenter Vorschlag
                trust.omega.update(1.0, 0.5);
            }
        }
        
        _ => {}
    }
    
    // Trust-Floor anwenden
    apply_trust_floor(trust, TRUST_FLOOR);
    
    trust.last_updated = event.timestamp;
    trust.event_count += 1;
    
    delta
}

const TRUST_FLOOR: f64 = 0.3;

fn apply_trust_floor(trust: &mut TrustState, floor: f64) {
    // Floor gilt für Expected Value
    trust.r.expected_value = trust.r.expected_value.max(floor);
    trust.i.expected_value = trust.i.expected_value.max(floor);
    trust.c.expected_value = trust.c.expected_value.max(floor);
    trust.p.expected_value = trust.p.expected_value.max(floor);
    trust.v.expected_value = trust.v.expected_value.max(floor);
    trust.omega.expected_value = trust.omega.expected_value.max(floor);
}
```

#### 4.2 Asymmetrie-Konstanten

```rust
/// Asymmetrie: Negativ wiegt N-mal schwerer als Positiv
pub const ASYMMETRY_FACTOR: f64 = 4.0;

/// Gewichtung pro Event-Typ
pub struct EventWeights {
    pub transaction_success: f64,      // 1.0
    pub transaction_failure: f64,      // 4.0 (asymmetrisch)
    pub attestation_positive: f64,     // 0.5 (gedämpft)
    pub attestation_negative: f64,     // 2.0
    pub credential_valid: f64,         // 1.0
    pub credential_invalid: f64,       // 10.0 (schwer)
    pub anomaly_correct: f64,          // severity-abhängig
    pub anomaly_false: f64,            // 0.5
    pub governance_participation: f64, // 0.3
}

impl Default for EventWeights {
    fn default() -> Self {
        Self {
            transaction_success: 1.0,
            transaction_failure: 4.0,
            attestation_positive: 0.5,
            attestation_negative: 2.0,
            credential_valid: 1.0,
            credential_invalid: 10.0,
            anomaly_correct: 1.0,
            anomaly_false: 0.5,
            governance_participation: 0.3,
        }
    }
}
```

### 5. Trust-Decay

#### 5.1 Decay-Mechanik

Inaktive Agenten verlieren Trust Richtung Neutral (0.5):

```
T(t) = 0.5 + (T(0) - 0.5) × λ^t
```

Mit λ = 0.9997 pro Tag (Halbwertszeit ≈ 6 Jahre).

```rust
const DECAY_FACTOR_PER_DAY: f64 = 0.9997;
const MS_PER_DAY: u64 = 24 * 60 * 60 * 1000;

pub fn apply_decay(trust: &mut TrustState, now: u64) {
    let days_since_update = (now - trust.last_updated) / MS_PER_DAY;
    
    if days_since_update == 0 {
        return;
    }
    
    let decay = DECAY_FACTOR_PER_DAY.powi(days_since_update as i32);
    
    // Decay Richtung 0.5 (Neutral)
    trust.r.expected_value = 0.5 + (trust.r.expected_value - 0.5) * decay;
    trust.i.expected_value = 0.5 + (trust.i.expected_value - 0.5) * decay;
    trust.c.expected_value = 0.5 + (trust.c.expected_value - 0.5) * decay;
    trust.p.expected_value = 0.5 + (trust.p.expected_value - 0.5) * decay;
    trust.v.expected_value = 0.5 + (trust.v.expected_value - 0.5) * decay;
    trust.omega.expected_value = 0.5 + (trust.omega.expected_value - 0.5) * decay;
    
    // Auch Konfidenz sinkt (Unsicherheit steigt)
    let confidence_decay = decay.sqrt();
    trust.r.confidence *= confidence_decay;
    trust.i.confidence *= confidence_decay;
    trust.c.confidence *= confidence_decay;
    trust.p.confidence *= confidence_decay;
    trust.v.confidence *= confidence_decay;
    trust.omega.confidence *= confidence_decay;
    
    trust.last_updated = now;
}
```

#### 5.2 Asymmetrischer Decay (H3)

Negative Events zerfallen schneller als positive (Axiom H3: Vergebung):

```rust
const DECAY_HALF_LIFE_NEGATIVE_YEARS: f64 = 3.0;
const DECAY_HALF_LIFE_POSITIVE_YEARS: f64 = 5.0;

/// Temporale Gewichtung für ein Event
pub fn temporal_weight(event_age_days: u64, is_negative: bool) -> f64 {
    let half_life_days = if is_negative {
        DECAY_HALF_LIFE_NEGATIVE_YEARS * 365.0
    } else {
        DECAY_HALF_LIFE_POSITIVE_YEARS * 365.0
    };
    
    let gamma = (0.5_f64).ln() / half_life_days;
    (gamma * event_age_days as f64).exp()
}
```

### 6. Konfidenz-basierte Interpretation

#### 6.1 Qualitative Buckets

```rust
pub fn interpret_trust(trust: &TrustState) -> TrustInterpretation {
    let scalar = trust.scalar();
    let confidence = trust.average_confidence();
    
    // Niedrige Konfidenz überschreibt Score
    if confidence < 0.5 {
        return TrustInterpretation {
            level: TrustLevel::Unknown,
            scalar,
            confidence,
            explanation: "Nicht genug Daten für verlässliche Einschätzung".into(),
        };
    }
    
    let level = match scalar {
        s if s < 0.4 => TrustLevel::Caution,
        s if s < 0.6 => TrustLevel::Neutral,
        s if s < 0.8 => TrustLevel::Verified,
        _ => TrustLevel::HighTrust,
    };
    
    let explanation = generate_explanation(trust, level);
    
    TrustInterpretation {
        level,
        scalar,
        confidence,
        explanation,
    }
}

fn generate_explanation(trust: &TrustState, level: TrustLevel) -> String {
    let mut parts = Vec::new();
    
    // Herausragende Dimensionen
    if trust.r.expected_value > 0.85 {
        parts.push("Sehr zuverlässig bei Zusagen");
    }
    if trust.i.expected_value > 0.9 {
        parts.push("Höchste Integrität");
    }
    if trust.v.expected_value > 0.7 {
        parts.push("Aktiver Wächter");
    }
    
    // Schwächen
    if trust.p.expected_value < 0.5 {
        parts.push("Unvorhersagbares Verhalten");
    }
    if trust.omega.expected_value < 0.7 {
        parts.push("Gelegentliche Regelverstöße");
    }
    
    // Guardian-Boost
    if let Some(derived) = &trust.derived_trust {
        parts.push(&format!("Gebürgt von {}", derived.source));
    }
    
    parts.join(". ")
}
```

### 7. API

#### 7.1 Trust berechnen

```
POST /v1/trust/{did}/calculate
```

**Request:**

```json
{
  "include_derived": true,
  "include_confidence": true,
  "weights": {
    "R": 0.15, "I": 0.15, "C": 0.15,
    "P": 0.10, "V": 0.20, "Ω": 0.25
  }
}
```

**Response:**

```json
{
  "did": "did:erynoa:self:alice-2024-abc123",
  "trust": {
    "base": {
      "R": { "value": 0.65, "alpha": 45, "beta": 25, "confidence": 0.72 },
      "I": { "value": 0.78, "alpha": 82, "beta": 23, "confidence": 0.85 },
      "C": { "value": 0.55, "alpha": 28, "beta": 23, "confidence": 0.61 },
      "P": { "value": 0.72, "alpha": 36, "beta": 14, "confidence": 0.68 },
      "V": { "value": 0.50, "alpha": 5, "beta": 5, "confidence": 0.35 },
      "Ω": { "value": 0.82, "alpha": 95, "beta": 21, "confidence": 0.88 }
    },
    "derived": {
      "sources": [
        { "guardian": "did:erynoa:guild:sparkasse-berlin", "boost": 0.135 }
      ],
      "totalBoost": 0.135
    },
    "effective": {
      "R": 0.785, "I": 0.905, "C": 0.685,
      "P": 0.855, "V": 0.635, "Ω": 0.935
    },
    "scalar": 0.812,
    "level": "HighTrust",
    "confidence": 0.68,
    "confidence_interval_95": [0.72, 0.90]
  },
  "interpretation": {
    "level": "HighTrust",
    "explanation": "Höchste Integrität. Gebürgt von Sparkasse Berlin."
  }
}
```

#### 7.2 Guardian-Endorsement erstellen

```
POST /v1/guardianship/endorse
```

**Request:**

```json
{
  "guardian": "did:erynoa:guild:sparkasse-berlin",
  "subject": "did:erynoa:self:alice-2024-abc123",
  "endorsement": {
    "level": "kyc-level-3",
    "stake": { "type": "tokens", "amount": 500 },
    "liability": "full"
  },
  "signature": "z4GdRnI..."
}
```

### 8. CLI-Nutzung

```bash
# Trust abfragen (mit Guardians)
erynoa trust did:erynoa:self:alice-2024 --include-derived

# Output:
# Trust for did:erynoa:self:alice-2024
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# BASE TRUST:
# R (Reliability):     0.65  ██████░░░░  [Moderate]  conf: 72%
# I (Integrity):       0.78  ███████░░░  [Good]      conf: 85%
# ...
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# GUARDIAN BOOST:
# Sparkasse Berlin     +0.135  (500 ERY staked, full liability)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# EFFECTIVE TRUST:     0.812   Level: HighTrust   Conf: 68%

# Als Guardian für jemanden bürgen
erynoa guardianship endorse did:erynoa:self:alice \
  --stake 500 \
  --liability full \
  --kyc-level 3

# Guardianship-Status anzeigen
erynoa guardianship list --guardian did:erynoa:guild:sparkasse-berlin

# Slashing-Historie
erynoa guardianship slashing-history did:erynoa:guild:sparkasse-berlin
```

### 9. SDK-Nutzung

#### 9.1 Rust

```rust
use erynoa_sdk::trust::{TrustState, StakedGuardian, Endorsement};

// Trust berechnen mit Guardians
let base_trust = client.get_base_trust(&alice_did).await?;
let guardians = client.get_staked_guardians(&alice_did).await?;

let effective_trust = calculate_effective_trust(
    &base_trust,
    &guardians,
    &guardian_cache,
);

println!("Base: {}", base_trust.scalar());
println!("Effective: {}", effective_trust.scalar());
println!("Boost: +{}", effective_trust.derived_trust.unwrap().boost_factor);

// Als Guardian bürgen
let endorsement = Endorsement {
    level: "kyc-level-3".into(),
    stake: StakeAmount::Tokens(500),
    liability: LiabilityLevel::Full,
};

client.endorse(&my_guild_did, &alice_did, endorsement, &keypair).await?;
```

#### 9.2 TypeScript

```typescript
import { TrustState, Endorsement, LiabilityLevel } from '@erynoa/sdk';

// Trust berechnen
const trust = await client.getTrust(aliceDid, { includeDerived: true });

console.log(`Base: ${trust.base.scalar}`);
console.log(`Effective: ${trust.effective.scalar}`);
console.log(`Boost: +${trust.derived.totalBoost}`);

// Als Guardian bürgen
await client.endorse(myGuildDid, aliceDid, {
  level: 'kyc-level-3',
  stake: { type: 'tokens', amount: 500 },
  liability: LiabilityLevel.Full,
}, keypair);
```

---

## Test-Vektoren

### TV-1: Neutral-Prior

**Input:** Neuer Agent, keine Events, keine Guardians

**Expected:**

```json
{
  "R": 0.5, "I": 0.5, "C": 0.5, "P": 0.5, "V": 0.5, "Ω": 0.5,
  "scalar": 0.5,
  "confidence": 0.2
}
```

### TV-2: Guardian-Boost

**Input:**
- Base Trust: all 0.5
- Guardian: T = 0.9, Stake = 500 ERY (stake_factor ≈ 0.3)

**Expected:**

```
boost = 0.3 × 0.9 × 0.3 = 0.081

T_effective = 0.5 + 0.081 = 0.581
```

### TV-3: Slashing

**Input:**
- Offender begeht Fraud (severity = 0.8)
- Guardian hat Full Liability, 500 ERY staked

**Expected:**

```
Δ_guardian_I = -1.0 × 0.8 × 0.3 × 0.1 = -0.024
```

Guardian-Integrity sinkt um 2.4%.

### TV-4: Circular Reference Detection (V0.2)

**Input:**
- Bank A bürgt für Bank B
- Bank B versucht, für Bank A zu bürgen

**Expected:**

```
Error: DirectCircularReference
Message: "Cannot create endorsement: A↔B cycle detected"
```

### TV-5: Private Guardianship (V0.2)

**Input:**
- Guardian: Sparkasse (Tier2Institutional)
- Visibility: Private

**Expected DID Document:**

```json
{
  "guardians": [{
    "visibility": "private",
    "tier": "Tier2Institutional",
    "zkProof": "z8aGdRnI..."
  }]
}
```

Guardian-DID ist NICHT öffentlich sichtbar.

---

## Referenzen

- [EIP-001: DID:erynoa](./EIP-001-did-erynoa.md)
- [EIP-002: Trust Vector 6D](./EIP-002-trust-vector-6d.md)
- [EIP-003: Event-DAG & Finality](./EIP-003-event-dag-finality.md)
- [EIP-006: Slashing & Dispute Resolution](./EIP-006-slashing-dispute.md) [Planned]
- [Bayesian Inference (Wikipedia)](https://en.wikipedia.org/wiki/Bayesian_inference)
- [Beta Distribution](https://en.wikipedia.org/wiki/Beta_distribution)
- [Erynoa Fachkonzept V6.1](../FACHKONZEPT.md)

---

## Changelog

| Version | Datum | Änderung |
|---------|-------|----------|
| 0.1 | 2026-01-29 | Initial Draft mit Staked Guardianship |
| 0.2 | 2026-01-29 | **Loop Detection**: Circular Reference Protection (PageRank-ähnlich), **Privacy**: ZK-Proofs für Guardianship, Selective Disclosure, **Trust-Chain Depth Limit**: Max 5 Stufen |

---

*EIP-004: Bayesian Trust Update Algorithm*
*Version: 0.2*
*Status: Draft*
*Ebene: E2 (Emergenz)*
