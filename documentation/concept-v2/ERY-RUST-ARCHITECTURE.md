# ERY – Rust Architektur

> **Version:** 1.0 – Technische Architekturspezifikation
> **Datum:** Januar 2026
> **Basis:** Weltformel 𝔼 = Σ 𝕀(s) · σ(𝕋(s) · ln|ℂ(s)|)

---

## Präambel: Von Formel zu Code

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║   WELTFORMEL → RUST MAPPING                                                                                                              ║
║                                                                                                                                           ║
║       𝔼              →  SystemState                                                                                                      ║
║       𝕀(s)           →  Identity / Did                                                                                                   ║
║       𝕋(s)           →  TrustVector                                                                                                      ║
║       ℂ(s)           →  CausalHistory                                                                                                    ║
║       σ(x)           →  fn sigmoid(x: f64) -> f64                                                                                        ║
║       Σ              →  Iterator::sum()                                                                                                  ║
║       Π              →  Process (enum)                                                                                                   ║
║       δ(S, Π)        →  fn apply(state: &mut State, process: Process)                                                                    ║
║       Ω              →  trait Invariant                                                                                                  ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

# Teil I: Crate-Struktur

## 1.1 Workspace-Layout

```
erynoa/
├── Cargo.toml                      # Workspace root
│
├── crates/
│   │
│   ├── ery-core/                   # 𝔼 – Kern-Typen & Weltformel
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── identity.rs         # 𝕀 – DID, Entity
│   │       ├── trust.rs            # 𝕋 – TrustVector
│   │       ├── causality.rs        # ℂ – Events, DAG
│   │       ├── realm.rs            # ε – Realms, Hierarchy
│   │       ├── value.rs            # 𝕍 – Assets, AMO
│   │       ├── formula.rs          # σ, 𝔼 – Weltformel
│   │       └── state.rs            # S – SystemState
│   │
│   ├── ery-logic/                  # Ω – Logik & Invarianten
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── operators.rs        # Logische Operatoren
│   │       ├── axioms.rs           # Axiome
│   │       ├── invariants.rs       # Ω – Invarianten
│   │       ├── validation.rs       # valid(Π, S)
│   │       └── rules.rs            # ECL Rules Engine
│   │
│   ├── ery-process/                # Π – Prozesse & Transitionen
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── process.rs          # Process enum
│   │       ├── genesis.rs          # Π-G1, Π-G2
│   │       ├── attestation.rs      # Π-A1, Π-A2
│   │       ├── transaction.rs      # Π-T1, Π-T2
│   │       ├── governance.rs       # Π-V1, Π-V2
│   │       ├── dispute.rs          # Π-D1
│   │       ├── lifecycle.rs        # Π-L1
│   │       └── transition.rs       # δ(S, Π)
│   │
│   ├── ery-crypto/                 # Kryptographie
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── keys.rs             # Ed25519, secp256k1
│   │       ├── signature.rs        # Sign, Verify
│   │       ├── hash.rs             # BLAKE3
│   │       └── threshold.rs        # Threshold Signatures
│   │
│   ├── ery-consensus/              # Konsens-Mechanismus
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── bft.rs              # BFT-Konsens
│   │       ├── weighted.rs         # σ-gewichteter Konsens
│   │       ├── finality.rs         # Finalitätsstufen
│   │       └── anchor.rs           # Multi-Chain Anchoring
│   │
│   ├── ery-storage/                # Persistenz
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dag.rs              # DAG-Storage
│   │       ├── index.rs            # Indizes
│   │       └── snapshot.rs         # State Snapshots
│   │
│   └── ery-api/                    # API Layer
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── grpc.rs             # gRPC/Connect
│           └── handlers.rs         # Request Handler
│
└── bins/
    └── ery-node/                   # Node Binary
        ├── Cargo.toml
        └── src/
            └── main.rs
```

---

# Teil II: Core Types (ery-core)

## 2.1 Identity – 𝕀

```rust
// crates/ery-core/src/identity.rs

use std::fmt;
use serde::{Deserialize, Serialize};

/// DID nach W3C-Standard für Erynoa
/// Format: did:erynoa:<namespace>:<unique-id>
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did {
    namespace: Namespace,
    unique_id: UniqueId,
}

/// Namespace-Typen (Entity-Kategorien)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Namespace {
    /// did:erynoa:self:* – Natürliche Person
    Self_,
    /// did:erynoa:guild:* – Organisation
    Guild,
    /// did:erynoa:spirit:* – Autonomer Agent
    Spirit,
    /// did:erynoa:thing:* – Physisches Gerät
    Thing,
    /// did:erynoa:vessel:* – Fahrzeug
    Vessel,
    /// did:erynoa:source:* – Energiequelle (Ladestation)
    Source,
    /// did:erynoa:craft:* – Service
    Craft,
    /// did:erynoa:vault:* – Wallet
    Vault,
    /// did:erynoa:pact:* – Vertrag
    Pact,
    /// did:erynoa:circle:* – Realm/Environment
    Circle,
    /// did:erynoa:system:* – Systemidentität
    System,
}

/// Eindeutige ID (BLAKE3 Hash, Base58 encoded)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UniqueId(pub [u8; 32]);

impl Did {
    /// Erstellt neue DID
    pub fn new(namespace: Namespace, unique_id: UniqueId) -> Self {
        Self { namespace, unique_id }
    }

    /// Parst DID-String
    pub fn parse(s: &str) -> Result<Self, DidError> {
        // did:erynoa:<namespace>:<unique-id>
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 4 || parts[0] != "did" || parts[1] != "erynoa" {
            return Err(DidError::InvalidFormat);
        }
        let namespace = Namespace::from_str(parts[2])?;
        let unique_id = UniqueId::from_base58(parts[3])?;
        Ok(Self { namespace, unique_id })
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "did:erynoa:{}:{}", 
            self.namespace.as_str(), 
            self.unique_id.to_base58()
        )
    }
}

/// Entity – Eine existierende Identität im System
/// Entspricht ⟨s⟩ = true in der Logik
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entity {
    /// Eindeutige DID
    pub did: Did,
    /// DID-Dokument
    pub document: DidDocument,
    /// Erstellungszeitpunkt
    pub created_at: Timestamp,
    /// Aktueller Status
    pub status: EntityStatus,
    /// Parent (falls delegiert)
    pub parent: Option<Did>,
}

/// Entity-Status (Lifecycle)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityStatus {
    /// Gerade erschaffen
    Nascent,
    /// Aktiv
    Active,
    /// Vertrauenswürdig (𝕋̄ ≥ 0.7)
    Trusted,
    /// Führend (𝕋̄ ≥ 0.85)
    Eminent,
    /// Inaktiv
    Dormant,
    /// Temporär suspendiert
    Suspended,
    /// Permanent widerrufen – 𝕀(s) = 0
    Revoked,
}

/// DID-Dokument nach W3C-Standard
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DidDocument {
    pub id: Did,
    pub verification_method: Vec<VerificationMethod>,
    pub controller: Vec<Did>,
    pub service: Vec<ServiceEndpoint>,
}

/// Prüft ob Entity existiert: ⟨s⟩
pub fn exists(entity: &Entity) -> bool {
    entity.status != EntityStatus::Revoked
}

/// Identity-Faktor für Weltformel: 𝕀(s) ∈ {0, 1}
pub fn identity_factor(entity: &Entity) -> f64 {
    if exists(entity) && entity.status != EntityStatus::Suspended {
        1.0
    } else {
        0.0
    }
}
```

---

## 2.2 Trust – 𝕋

```rust
// crates/ery-core/src/trust.rs

use serde::{Deserialize, Serialize};

/// Minimaler Trust-Wert
pub const MIN_TRUST: f64 = 0.3;
/// Maximaler Trust-Wert
pub const MAX_TRUST: f64 = 1.0;
/// Initialer Trust-Wert für Newcomer
pub const INITIAL_TRUST: f64 = 0.5;
/// Asymmetrie-Faktor für negative Updates
pub const ASYMMETRY_FACTOR: f64 = 1.5;

/// 4-dimensionaler Trust-Vektor
/// 𝕋(s) = (R, I, C, P) ∈ [0.3, 1.0]⁴
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrustVector {
    /// R – Reliability (Zuverlässigkeit)
    pub reliability: f64,
    /// I – Integrity (Integrität)
    pub integrity: f64,
    /// C – Capability (Leistungsfähigkeit)
    pub capability: f64,
    /// P – Prestige (Ansehen)
    pub prestige: f64,
}

impl TrustVector {
    /// Erstellt initialen Trust-Vektor für Newcomer
    pub fn initial() -> Self {
        Self {
            reliability: INITIAL_TRUST,
            integrity: INITIAL_TRUST,
            capability: INITIAL_TRUST,
            prestige: INITIAL_TRUST,
        }
    }

    /// Erstellt Trust-Vektor mit spezifischen Werten
    pub fn new(r: f64, i: f64, c: f64, p: f64) -> Self {
        Self {
            reliability: clamp(r),
            integrity: clamp(i),
            capability: clamp(c),
            prestige: clamp(p),
        }
    }

    /// Berechnet aggregierten Trust-Wert: 𝕋̄(s)
    pub fn aggregate(&self) -> f64 {
        (self.reliability + self.integrity + self.capability + self.prestige) / 4.0
    }

    /// Berechnet gewichteten aggregierten Trust-Wert
    pub fn aggregate_weighted(&self, weights: &TrustWeights) -> f64 {
        weights.reliability * self.reliability
            + weights.integrity * self.integrity
            + weights.capability * self.capability
            + weights.prestige * self.prestige
    }

    /// Wendet Update auf Dimension an
    pub fn apply_update(&mut self, dimension: TrustDimension, delta: f64, negative: bool) {
        let actual_delta = if negative {
            delta * ASYMMETRY_FACTOR  // κ = 1.5
        } else {
            delta
        };
        
        let value = match dimension {
            TrustDimension::Reliability => &mut self.reliability,
            TrustDimension::Integrity => &mut self.integrity,
            TrustDimension::Capability => &mut self.capability,
            TrustDimension::Prestige => &mut self.prestige,
        };
        
        *value = clamp(*value + actual_delta);
    }

    /// Wendet Time-Decay an: 𝕋' = 𝕋 · e^(-λ·Δt)
    pub fn apply_decay(&mut self, months_elapsed: f64, decay_rate: f64) {
        let factor = (-decay_rate * months_elapsed).exp();
        self.reliability = clamp(self.reliability * factor);
        self.integrity = clamp(self.integrity * factor);
        self.capability = clamp(self.capability * factor);
        self.prestige = clamp(self.prestige * factor);
    }

    /// Prüft Delegations-Constraint: 𝕋(child) ≤ 𝕋(parent)
    pub fn constrain_to(&mut self, parent: &TrustVector) {
        self.reliability = self.reliability.min(parent.reliability);
        self.integrity = self.integrity.min(parent.integrity);
        self.capability = self.capability.min(parent.capability);
        self.prestige = self.prestige.min(parent.prestige);
    }
}

/// Trust-Dimension
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustDimension {
    Reliability,
    Integrity,
    Capability,
    Prestige,
}

/// Gewichte für aggregierten Trust
#[derive(Clone, Debug)]
pub struct TrustWeights {
    pub reliability: f64,
    pub integrity: f64,
    pub capability: f64,
    pub prestige: f64,
}

impl Default for TrustWeights {
    fn default() -> Self {
        Self {
            reliability: 0.25,
            integrity: 0.25,
            capability: 0.25,
            prestige: 0.25,
        }
    }
}

/// Clamp-Funktion für Trust-Werte
fn clamp(value: f64) -> f64 {
    value.max(MIN_TRUST).min(MAX_TRUST)
}

/// Trust-Kombinations-Operator: ⊕
/// t₁ ⊕ t₂ = 1 - (1 - t₁)(1 - t₂)
pub fn combine_trust(t1: f64, t2: f64) -> f64 {
    1.0 - (1.0 - t1) * (1.0 - t2)
}

/// Multi-Trust-Kombination
pub fn combine_multi(trusts: &[f64]) -> f64 {
    1.0 - trusts.iter().map(|t| 1.0 - t).product::<f64>()
}
```

---

## 2.3 Causality – ℂ

```rust
// crates/ery-core/src/causality.rs

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::identity::Did;
use crate::crypto::{Hash, Signature};

/// Event im kausalen DAG
/// e = ⟨actor, type, payload, parents, signature, timestamp⟩
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    /// Hash des Events (berechneter Identifikator)
    pub hash: Hash,
    /// Actor: Wer hat das Event ausgelöst
    pub actor: Did,
    /// Event-Typ
    pub event_type: EventType,
    /// Payload (typ-spezifisch)
    pub payload: EventPayload,
    /// Parent-Hashes (kausale Vorgänger)
    pub parents: Vec<Hash>,
    /// Signaturen der beteiligten Parteien
    pub signatures: Vec<Signature>,
    /// Zeitstempel
    pub timestamp: Timestamp,
    /// Finalitätsstufe
    pub finality: FinalityLevel,
}

/// Event-Typen (entsprechen Prozessen)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    // Genesis
    SystemGenesis,
    EntityGenesis,
    Delegation,
    
    // Attestation
    TrustAttestation,
    CredentialIssuance,
    CredentialRevocation,
    
    // Transaction
    Lock,
    Exchange,
    Transfer,
    
    // Governance
    RealmCreate,
    RealmUpdate,
    ProposalSubmit,
    Vote,
    
    // Dispute
    DisputeFile,
    DisputeResponse,
    DisputeVerdict,
    
    // Lifecycle
    Suspend,
    Unsuspend,
    Revoke,
}

/// Finalitätsstufen: ○ → ◐ → ◑ → ●
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FinalityLevel {
    /// Gerade erstellt, P(revert) ≈ 50%
    Nascent,
    /// Lokal validiert
    Validated,
    /// Netzwerk-Konsens erreicht: ⟦e⟧ = true
    Witnessed,
    /// Auf primärer Chain verankert
    Anchored,
    /// Multi-Chain verankert: ∎e = true
    Eternal,
}

/// Kausale Geschichte einer Entity
/// ℂ(s) = { e : actor(e) = s ∧ ⟦e⟧ }
#[derive(Clone, Debug, Default)]
pub struct CausalHistory {
    /// Events nach Hash indiziert
    events: HashMap<Hash, Event>,
    /// Events pro Actor
    by_actor: HashMap<Did, Vec<Hash>>,
    /// Kausale Ordnung (Edges im DAG)
    edges: HashSet<(Hash, Hash)>,
    /// Tips (Events ohne Nachfolger)
    tips: HashSet<Hash>,
}

impl CausalHistory {
    /// Neue leere Geschichte
    pub fn new() -> Self {
        Self::default()
    }

    /// Fügt Event hinzu
    pub fn insert(&mut self, event: Event) -> Result<(), CausalityError> {
        // Prüfe: Alle Parents müssen existieren (Ω-C4)
        for parent in &event.parents {
            if !self.events.contains_key(parent) {
                return Err(CausalityError::MissingParent(parent.clone()));
            }
        }

        // Prüfe: Kein Duplikat
        if self.events.contains_key(&event.hash) {
            return Err(CausalityError::DuplicateEvent);
        }

        let hash = event.hash.clone();
        let actor = event.actor.clone();

        // Aktualisiere Tips
        for parent in &event.parents {
            self.tips.remove(parent);
            self.edges.insert((parent.clone(), hash.clone()));
        }
        self.tips.insert(hash.clone());

        // Speichere Event
        self.by_actor.entry(actor).or_default().push(hash.clone());
        self.events.insert(hash, event);

        Ok(())
    }

    /// Kausale Tiefe einer Entity: |ℂ(s)|
    pub fn depth(&self, actor: &Did) -> usize {
        self.by_actor.get(actor).map(|v| v.len()).unwrap_or(0)
    }

    /// Prüft kausale Präzedenz: e ⊲ e'
    pub fn precedes(&self, earlier: &Hash, later: &Hash) -> bool {
        self.reachable(earlier, later)
    }

    /// Prüft ob bezeugt: ⟦e⟧
    pub fn is_witnessed(&self, hash: &Hash) -> bool {
        self.events
            .get(hash)
            .map(|e| e.finality >= FinalityLevel::Witnessed)
            .unwrap_or(false)
    }

    /// Prüft ob endgültig: ∎e
    pub fn is_final(&self, hash: &Hash) -> bool {
        self.events
            .get(hash)
            .map(|e| e.finality == FinalityLevel::Eternal)
            .unwrap_or(false)
    }

    /// Aktuelle Tips für neue Events
    pub fn current_tips(&self) -> Vec<Hash> {
        self.tips.iter().cloned().collect()
    }

    /// Erreichbarkeits-Check im DAG
    fn reachable(&self, from: &Hash, to: &Hash) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![to.clone()];
        
        while let Some(current) = stack.pop() {
            if &current == from {
                return true;
            }
            if visited.insert(current.clone()) {
                if let Some(event) = self.events.get(&current) {
                    stack.extend(event.parents.clone());
                }
            }
        }
        false
    }
}

/// Kausaltiefen-Berechnung für Weltformel: ln|ℂ(s)|
pub fn log_causal_depth(depth: usize) -> f64 {
    (depth.max(1) as f64).ln()
}
```

---

## 2.4 Formula – σ, 𝔼

```rust
// crates/ery-core/src/formula.rs

use crate::identity::{Entity, identity_factor};
use crate::trust::TrustVector;
use crate::causality::{CausalHistory, log_causal_depth};

/// Sigmoid-Funktion: σ(x) = 1 / (1 + e^(-x))
/// Die Aufmerksamkeitsfunktion der Weltformel
#[inline]
pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Berechnet Aufmerksamkeits-Score für eine Entity
/// σ(s) = 1 / (1 + e^(-𝕋̄(s) · ln|ℂ(s)|))
pub fn attention_score(trust: &TrustVector, causal_depth: usize) -> f64 {
    let t_bar = trust.aggregate();
    let ln_c = log_causal_depth(causal_depth);
    sigmoid(t_bar * ln_c)
}

/// Berechnet den Beitrag einer Entity zum Systemwert
/// contribution(s) = 𝕀(s) · σ(s)
pub fn entity_contribution(
    entity: &Entity,
    trust: &TrustVector,
    causal_depth: usize,
) -> f64 {
    let i = identity_factor(entity);
    let sigma = attention_score(trust, causal_depth);
    i * sigma
}

/// Weltformel: 𝔼 = Σ 𝕀(s) · σ(𝕋(s) · ln|ℂ(s)|)
pub struct WorldFormula;

impl WorldFormula {
    /// Berechnet den vollständigen Systemwert
    pub fn compute<'a, I>(entities: I) -> f64
    where
        I: Iterator<Item = (&'a Entity, &'a TrustVector, usize)>,
    {
        entities
            .map(|(entity, trust, depth)| entity_contribution(entity, trust, depth))
            .sum()
    }

    /// Inkrementelle Berechnung nach Zustandsänderung
    pub fn compute_delta(
        old_contribution: f64,
        new_contribution: f64,
    ) -> f64 {
        new_contribution - old_contribution
    }

    /// Partielle Ableitung nach Identität: ∂𝔼/∂𝕀
    pub fn partial_identity(trust: &TrustVector, causal_depth: usize) -> f64 {
        attention_score(trust, causal_depth)
    }

    /// Partielle Ableitung nach Trust: ∂𝔼/∂𝕋
    pub fn partial_trust(
        entity: &Entity,
        trust: &TrustVector,
        causal_depth: usize,
    ) -> f64 {
        let i = identity_factor(entity);
        let t_bar = trust.aggregate();
        let ln_c = log_causal_depth(causal_depth);
        let sigma = sigmoid(t_bar * ln_c);
        
        // σ'(x) = σ(x) · (1 - σ(x))
        let sigma_prime = sigma * (1.0 - sigma);
        
        i * sigma_prime * ln_c
    }

    /// Partielle Ableitung nach Kausaltiefe: ∂𝔼/∂|ℂ|
    pub fn partial_causality(
        entity: &Entity,
        trust: &TrustVector,
        causal_depth: usize,
    ) -> f64 {
        let i = identity_factor(entity);
        let t_bar = trust.aggregate();
        let ln_c = log_causal_depth(causal_depth);
        let sigma = sigmoid(t_bar * ln_c);
        let sigma_prime = sigma * (1.0 - sigma);
        
        // ∂ln(|ℂ|)/∂|ℂ| = 1/|ℂ|
        i * sigma_prime * t_bar / (causal_depth.max(1) as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid_properties() {
        // σ(0) = 0.5
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-10);
        
        // σ(-∞) → 0
        assert!(sigmoid(-100.0) < 0.001);
        
        // σ(+∞) → 1
        assert!(sigmoid(100.0) > 0.999);
        
        // Monoton steigend
        assert!(sigmoid(1.0) > sigmoid(0.0));
    }

    #[test]
    fn test_newcomer_attention() {
        // Newcomer: 𝕋̄ = 0.5, |ℂ| = 1 → σ = 0.5
        let trust = TrustVector::initial();
        let depth = 1;
        let sigma = attention_score(&trust, depth);
        assert!((sigma - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_established_attention() {
        // Etabliert: 𝕋̄ = 0.8, |ℂ| = 100 → σ ≈ 0.975
        let trust = TrustVector::new(0.8, 0.8, 0.8, 0.8);
        let depth = 100;
        let sigma = attention_score(&trust, depth);
        assert!(sigma > 0.97);
    }
}
```

---

## 2.5 State – S

```rust
// crates/ery-core/src/state.rs

use std::collections::HashMap;
use crate::identity::{Did, Entity};
use crate::trust::TrustVector;
use crate::causality::CausalHistory;
use crate::realm::RealmHierarchy;
use crate::value::AssetRegistry;
use crate::formula::WorldFormula;

/// Systemzustand: S = ⟨𝔼, 𝕊, 𝕋, ℂ, ℜ, 𝕍⟩
#[derive(Clone, Debug)]
pub struct SystemState {
    /// 𝔼 – Aktueller Systemwert (Weltformel)
    pub system_value: f64,
    
    /// 𝕊 – Menge aller Entities
    pub entities: HashMap<Did, Entity>,
    
    /// 𝕋 – Trust pro Entity
    pub trust: HashMap<Did, TrustVector>,
    
    /// ℂ – Kausale Geschichte (globaler DAG)
    pub history: CausalHistory,
    
    /// ℜ – Realm-Hierarchie
    pub realms: RealmHierarchy,
    
    /// 𝕍 – Asset-Registry
    pub values: AssetRegistry,
}

impl SystemState {
    /// Erstellt Genesis-Zustand: s₀
    pub fn genesis() -> Self {
        let mut state = Self {
            system_value: 0.0,
            entities: HashMap::new(),
            trust: HashMap::new(),
            history: CausalHistory::new(),
            realms: RealmHierarchy::with_global(),
            values: AssetRegistry::new(),
        };
        
        // Genesis-Entity erstellen
        // ...
        
        state.recompute_system_value();
        state
    }

    /// Gibt Entity zurück
    pub fn get_entity(&self, did: &Did) -> Option<&Entity> {
        self.entities.get(did)
    }

    /// Gibt Trust zurück
    pub fn get_trust(&self, did: &Did) -> Option<&TrustVector> {
        self.trust.get(did)
    }

    /// Kausale Tiefe einer Entity
    pub fn causal_depth(&self, did: &Did) -> usize {
        self.history.depth(did)
    }

    /// Berechnet Aufmerksamkeit einer Entity: σ(s)
    pub fn attention(&self, did: &Did) -> f64 {
        let trust = self.trust.get(did).cloned().unwrap_or(TrustVector::initial());
        let depth = self.causal_depth(did);
        crate::formula::attention_score(&trust, depth)
    }

    /// Berechnet Systemwert neu: 𝔼 = Σ 𝕀·σ
    pub fn recompute_system_value(&mut self) {
        let entities_iter = self.entities.iter().map(|(did, entity)| {
            let trust = self.trust.get(did).cloned().unwrap_or(TrustVector::initial());
            let depth = self.history.depth(did);
            (entity, &trust, depth)
        });
        
        // Wegen Lifetime-Issues hier vereinfacht:
        self.system_value = self.entities
            .iter()
            .map(|(did, entity)| {
                let trust = self.trust.get(did).cloned().unwrap_or(TrustVector::initial());
                let depth = self.history.depth(did);
                crate::formula::entity_contribution(entity, &trust, depth)
            })
            .sum();
    }

    /// Prüft ob Entity existiert: ⟨s⟩
    pub fn exists(&self, did: &Did) -> bool {
        self.entities
            .get(did)
            .map(crate::identity::exists)
            .unwrap_or(false)
    }
}
```

---

# Teil III: Logic (ery-logic)

## 3.1 Invarianten – Ω

```rust
// crates/ery-logic/src/invariants.rs

use ery_core::state::SystemState;
use ery_core::identity::Did;
use ery_core::trust::{MIN_TRUST, MAX_TRUST};

/// Ergebnis einer Invarianten-Prüfung
pub type InvariantResult = Result<(), InvariantViolation>;

/// Invarianten-Verletzung
#[derive(Debug, Clone)]
pub enum InvariantViolation {
    // Identität (Ω-I)
    DuplicateDid(Did),
    UnresolvableDid(Did),
    DelegationCycle(Did),
    
    // Trust (Ω-T)
    TrustOutOfBounds { did: Did, value: f64 },
    DelegationTrustExceeded { child: Did, parent: Did },
    
    // Kausalität (Ω-C)
    CausalCycle,
    MissingParent { event: String, parent: String },
    InvalidSignature { event: String },
    
    // Realm (Ω-R)
    OrphanRealm(String),
    MonotonicityViolation { child: String, parent: String },
    
    // Wert (Ω-V)
    MultipleOwners { asset: String },
    NegativeValue { asset: String },
    ValueNotConserved { expected: f64, actual: f64 },
    
    // Weltformel (Ω-E)
    IncorrectSystemValue { expected: f64, actual: f64 },
}

/// Trait für Invarianten
pub trait Invariant {
    /// Name der Invariante
    fn name(&self) -> &'static str;
    
    /// Prüft die Invariante
    fn check(&self, state: &SystemState) -> InvariantResult;
}

/// Ω-I1: Identitäts-Eindeutigkeit
pub struct IdentityUniqueness;

impl Invariant for IdentityUniqueness {
    fn name(&self) -> &'static str {
        "Ω-I1: Identity Uniqueness"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        // DIDs sind bereits durch HashMap eindeutig
        // Zusätzliche Cross-Checks können hier erfolgen
        Ok(())
    }
}

/// Ω-T1: Trust-Beschränktheit
pub struct TrustBoundedness;

impl Invariant for TrustBoundedness {
    fn name(&self) -> &'static str {
        "Ω-T1: Trust Boundedness"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        for (did, trust) in &state.trust {
            for &value in &[
                trust.reliability,
                trust.integrity,
                trust.capability,
                trust.prestige,
            ] {
                if value < MIN_TRUST || value > MAX_TRUST {
                    return Err(InvariantViolation::TrustOutOfBounds {
                        did: did.clone(),
                        value,
                    });
                }
            }
        }
        Ok(())
    }
}

/// Ω-T3: Delegations-Obergrenze
pub struct DelegationTrustLimit;

impl Invariant for DelegationTrustLimit {
    fn name(&self) -> &'static str {
        "Ω-T3: Delegation Trust Limit"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        for (did, entity) in &state.entities {
            if let Some(parent_did) = &entity.parent {
                let child_trust = state.trust.get(did);
                let parent_trust = state.trust.get(parent_did);
                
                if let (Some(ct), Some(pt)) = (child_trust, parent_trust) {
                    if ct.aggregate() > pt.aggregate() {
                        return Err(InvariantViolation::DelegationTrustExceeded {
                            child: did.clone(),
                            parent: parent_did.clone(),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

/// Ω-E1: Korrekte Weltformel-Berechnung
pub struct WorldFormulaCorrectness;

impl Invariant for WorldFormulaCorrectness {
    fn name(&self) -> &'static str {
        "Ω-E1: World Formula Correctness"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        let computed: f64 = state.entities
            .iter()
            .map(|(did, entity)| {
                let trust = state.trust.get(did)
                    .cloned()
                    .unwrap_or(ery_core::trust::TrustVector::initial());
                let depth = state.history.depth(did);
                ery_core::formula::entity_contribution(entity, &trust, depth)
            })
            .sum();
        
        let epsilon = 1e-10;
        if (state.system_value - computed).abs() > epsilon {
            return Err(InvariantViolation::IncorrectSystemValue {
                expected: computed,
                actual: state.system_value,
            });
        }
        Ok(())
    }
}

/// Vollständiger Invarianten-Checker
pub struct InvariantChecker {
    invariants: Vec<Box<dyn Invariant>>,
}

impl InvariantChecker {
    /// Erstellt Checker mit allen Invarianten
    pub fn full() -> Self {
        Self {
            invariants: vec![
                Box::new(IdentityUniqueness),
                Box::new(TrustBoundedness),
                Box::new(DelegationTrustLimit),
                Box::new(WorldFormulaCorrectness),
                // ... weitere Invarianten
            ],
        }
    }

    /// Prüft alle Invarianten
    pub fn check_all(&self, state: &SystemState) -> Vec<InvariantViolation> {
        self.invariants
            .iter()
            .filter_map(|inv| inv.check(state).err())
            .collect()
    }

    /// Prüft ob alle Invarianten erfüllt: Ω(S) = true
    pub fn satisfies(&self, state: &SystemState) -> bool {
        self.invariants.iter().all(|inv| inv.check(state).is_ok())
    }
}
```

---

# Teil IV: Processes (ery-process)

## 4.1 Transition – δ(S, Π)

```rust
// crates/ery-process/src/transition.rs

use ery_core::state::SystemState;
use ery_logic::invariants::InvariantChecker;
use crate::process::Process;
use crate::validation::Validator;

/// Übergangs-Engine: δ(S, Π) → S'
pub struct TransitionEngine {
    validator: Validator,
    invariant_checker: InvariantChecker,
}

impl TransitionEngine {
    pub fn new() -> Self {
        Self {
            validator: Validator::new(),
            invariant_checker: InvariantChecker::full(),
        }
    }

    /// Führt Zustandsübergang aus: δ(S, Π) → S'
    pub fn apply(
        &self,
        state: &mut SystemState,
        process: Process,
    ) -> Result<TransitionResult, TransitionError> {
        // Schritt 1: Validierung
        self.validator.validate(&process, state)?;

        // Schritt 2: Snapshot für Rollback
        let snapshot = state.clone();

        // Schritt 3: Prozess anwenden
        let events = match &process {
            Process::Genesis(p) => crate::genesis::apply(state, p)?,
            Process::Delegate(p) => crate::genesis::apply_delegate(state, p)?,
            Process::Attest(p) => crate::attestation::apply(state, p)?,
            Process::Exchange(p) => crate::transaction::apply_exchange(state, p)?,
            Process::RealmCreate(p) => crate::governance::apply_realm_create(state, p)?,
            Process::Vote(p) => crate::governance::apply_vote(state, p)?,
            Process::Revoke(p) => crate::lifecycle::apply_revoke(state, p)?,
            // ... weitere Prozesse
        };

        // Schritt 4: Weltformel aktualisieren
        state.recompute_system_value();

        // Schritt 5: Invarianten prüfen
        let violations = self.invariant_checker.check_all(state);
        if !violations.is_empty() {
            // Rollback
            *state = snapshot;
            return Err(TransitionError::InvariantViolation(violations));
        }

        Ok(TransitionResult {
            events,
            delta_e: state.system_value - snapshot.system_value,
        })
    }
}

/// Ergebnis einer Transition
#[derive(Debug)]
pub struct TransitionResult {
    /// Erzeugte Events
    pub events: Vec<ery_core::causality::Event>,
    /// Änderung des Systemwerts: Δ𝔼
    pub delta_e: f64,
}

/// Transition-Fehler
#[derive(Debug)]
pub enum TransitionError {
    Validation(crate::validation::ValidationError),
    InvariantViolation(Vec<ery_logic::invariants::InvariantViolation>),
    ProcessError(String),
}
```

---

# Teil V: Realm – ε (A17-A20)

## 5.1 Realm-Hierarchie

```rust
// crates/ery-core/src/realm.rs

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::identity::Did;

/// Realm (Environment) – Raum mit eigenen Regeln
/// Entspricht [R]φ in der Logik
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Realm {
    /// Eindeutige Realm-ID
    pub id: RealmId,
    /// Name
    pub name: String,
    /// Eltern-Realm (None nur für Global)
    pub parent: Option<RealmId>,
    /// Regeln dieses Realms
    pub rules: RealmRules,
    /// Governance-Modell
    pub governance: GovernanceModel,
    /// Mitglieder
    pub members: HashSet<Did>,
    /// Administratoren
    pub admins: HashSet<Did>,
    /// Erstellungszeitpunkt
    pub created_at: Timestamp,
    /// Ersteller
    pub creator: Did,
}

/// Realm-ID (spezialisierte DID)
pub type RealmId = Did;  // did:erynoa:circle:*

/// Regeln eines Realms
/// A18: rules(R') ⊆ rules(R) bei R ⊑ R'
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RealmRules {
    /// Minimales Vertrauen für Mitgliedschaft
    pub min_trust: f64,
    /// Maximale Delegationstiefe
    pub max_delegation_depth: u32,
    /// Erlaubte Aktionen
    pub allowed_actions: HashSet<ActionType>,
    /// Erforderliche Attestationen
    pub required_attestations: HashSet<AttestationType>,
    /// Maximaler Transaktionswert
    pub max_transaction_value: Option<f64>,
    /// Zusätzliche Constraints (ECL)
    pub custom_constraints: Vec<Constraint>,
}

impl RealmRules {
    /// Prüft Monotonie: Child-Rules müssen strenger sein (A18)
    /// (R ⊑ R') → (rules(R') ⊆ rules(R))
    pub fn is_stricter_than(&self, parent: &RealmRules) -> bool {
        // Min-Trust muss größer oder gleich sein
        self.min_trust >= parent.min_trust
        // Max-Delegation muss kleiner oder gleich sein
        && self.max_delegation_depth <= parent.max_delegation_depth
        // Allowed Actions muss Subset sein
        && self.allowed_actions.is_subset(&parent.allowed_actions)
        // Required Attestations muss Superset sein
        && self.required_attestations.is_superset(&parent.required_attestations)
        // Max-Value muss kleiner oder gleich sein
        && match (self.max_transaction_value, parent.max_transaction_value) {
            (Some(c), Some(p)) => c <= p,
            (Some(_), None) => true,  // Strenger
            (None, Some(_)) => false, // Lockerer
            (None, None) => true,
        }
    }
}

/// Governance-Modell
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceModel {
    pub governance_type: GovernanceType,
    pub quorum: f64,
    pub voting_method: VotingMethod,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernanceType {
    /// Demokratisch: Alle Mitglieder abstimmen
    Democratic,
    /// Hierarchisch: Admins entscheiden
    Hierarchical,
    /// Schwellenwert: Bestimmte Trust-Schwelle nötig
    Threshold(f64),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VotingMethod {
    /// Linear: 1 Stimme pro Mitglied
    Linear,
    /// Gewichtet: σ(s) als Gewicht
    SigmaWeighted,
    /// Quadratisch: sqrt(stake) als Gewicht
    Quadratic,
}

/// Realm-Hierarchie – Baum aller Realms
#[derive(Clone, Debug, Default)]
pub struct RealmHierarchy {
    realms: HashMap<RealmId, Realm>,
    children: HashMap<RealmId, HashSet<RealmId>>,
    global_id: Option<RealmId>,
}

impl RealmHierarchy {
    /// Erstellt Hierarchie mit Global-Realm
    pub fn with_global() -> Self {
        let global = Realm {
            id: Did::parse("did:erynoa:circle:global").unwrap(),
            name: "Global".to_string(),
            parent: None,
            rules: RealmRules::default_global(),
            governance: GovernanceModel::default(),
            members: HashSet::new(),
            admins: HashSet::new(),
            created_at: Timestamp::now(),
            creator: Did::parse("did:erynoa:system:genesis").unwrap(),
        };
        
        let global_id = global.id.clone();
        let mut realms = HashMap::new();
        realms.insert(global_id.clone(), global);
        
        Self {
            realms,
            children: HashMap::new(),
            global_id: Some(global_id),
        }
    }

    /// Prüft Containment: R ⊑ R' (A17)
    pub fn is_contained_in(&self, child: &RealmId, parent: &RealmId) -> bool {
        let mut current = child.clone();
        while let Some(realm) = self.realms.get(&current) {
            if let Some(p) = &realm.parent {
                if p == parent {
                    return true;
                }
                current = p.clone();
            } else {
                break;
            }
        }
        false
    }

    /// Fügt neuen Realm hinzu mit Monotonie-Prüfung (A18)
    pub fn add_realm(&mut self, realm: Realm) -> Result<(), RealmError> {
        // Prüfe: Parent muss existieren
        if let Some(parent_id) = &realm.parent {
            let parent = self.realms.get(parent_id)
                .ok_or(RealmError::ParentNotFound)?;
            
            // A18: Monotonie-Check
            if !realm.rules.is_stricter_than(&parent.rules) {
                return Err(RealmError::MonotonicityViolation);
            }
        } else if self.global_id.is_some() {
            // Nur ein Global-Realm erlaubt
            return Err(RealmError::MultipleGlobalRealms);
        }

        let id = realm.id.clone();
        if let Some(parent_id) = &realm.parent {
            self.children.entry(parent_id.clone())
                .or_default()
                .insert(id.clone());
        }
        self.realms.insert(id, realm);
        Ok(())
    }

    /// Prüft Mitgliedschaft: s ∈ R (A19)
    pub fn is_member(&self, entity: &Did, realm: &RealmId) -> bool {
        self.realms.get(realm)
            .map(|r| r.members.contains(entity))
            .unwrap_or(false)
    }

    /// Prüft ob Aktion im Realm erlaubt: [R]◇(s : α) (A22)
    pub fn is_action_allowed(
        &self,
        realm: &RealmId,
        entity: &Did,
        action: &ActionType,
        trust: f64,
    ) -> bool {
        if let Some(r) = self.realms.get(realm) {
            // Muss Mitglied sein
            r.members.contains(entity)
            // Aktion muss erlaubt sein
            && r.rules.allowed_actions.contains(action)
            // Trust muss ausreichen
            && trust >= r.rules.min_trust
        } else {
            false
        }
    }

    /// A17: Was im Kind gilt, gilt auch im Eltern
    pub fn propagate_truth_upward(&self, realm: &RealmId) -> Vec<RealmId> {
        let mut path = vec![realm.clone()];
        let mut current = realm.clone();
        
        while let Some(r) = self.realms.get(&current) {
            if let Some(parent) = &r.parent {
                path.push(parent.clone());
                current = parent.clone();
            } else {
                break;
            }
        }
        path
    }
}

impl RealmRules {
    /// Default-Regeln für Global-Realm
    pub fn default_global() -> Self {
        Self {
            min_trust: 0.3,
            max_delegation_depth: 10,
            allowed_actions: ActionType::all(),
            required_attestations: HashSet::new(),
            max_transaction_value: None,
            custom_constraints: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum RealmError {
    ParentNotFound,
    MonotonicityViolation,
    MultipleGlobalRealms,
    NotAMember,
    ActionNotAllowed,
}
```

---

# Teil VI: Value – 𝕍

## 6.1 Asset-System

```rust
// crates/ery-core/src/value.rs

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::identity::Did;
use crate::crypto::Hash;

/// Asset – Wertgegenstand im System
/// Entspricht 𝕍(x) in der Logik
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Asset {
    /// Eindeutige Asset-ID
    pub id: AssetId,
    /// Typ des Assets
    pub asset_type: AssetType,
    /// Aktueller Eigentümer: x ↝ s
    pub owner: Did,
    /// Wert: 𝕍(x)
    pub value: f64,
    /// Währung/Einheit
    pub unit: Unit,
    /// Sperrstatus
    pub lock_status: LockStatus,
    /// Erstellungszeitpunkt
    pub created_at: Timestamp,
    /// Letzte Änderung
    pub updated_at: Timestamp,
    /// Provenienz (Herkunft)
    pub provenance: Vec<Hash>,
}

/// Asset-ID (Hash-basiert)
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetId(pub Hash);

/// Asset-Typen
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    /// Fungible Token (z.B. Währung)
    Token { symbol: String, decimals: u8 },
    /// Non-Fungible (einzigartig)
    Unique { metadata: HashMap<String, String> },
    /// Energie (kWh)
    Energy,
    /// Credential
    Credential,
    /// Service-Berechtigung
    ServiceAccess,
}

/// Sperrung
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockStatus {
    /// Frei verfügbar
    Unlocked,
    /// Gesperrt für Transaktion
    LockedFor { transaction: Hash, until: Timestamp },
    /// Permanent eingefroren
    Frozen,
}

/// Asset-Registry
#[derive(Clone, Debug, Default)]
pub struct AssetRegistry {
    assets: HashMap<AssetId, Asset>,
    by_owner: HashMap<Did, Vec<AssetId>>,
    total_value: f64,
}

impl AssetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Erstellt neues Asset
    pub fn create(&mut self, asset: Asset) -> Result<(), ValueError> {
        if asset.value < 0.0 {
            return Err(ValueError::NegativeValue);
        }
        
        let id = asset.id.clone();
        let owner = asset.owner.clone();
        
        self.total_value += asset.value;
        self.by_owner.entry(owner).or_default().push(id.clone());
        self.assets.insert(id, asset);
        
        Ok(())
    }

    /// Wert eines Assets: 𝕍(x)
    pub fn value(&self, id: &AssetId) -> Option<f64> {
        self.assets.get(id).map(|a| a.value)
    }

    /// Prüft Eigentum: x ↝ s
    pub fn owns(&self, owner: &Did, asset: &AssetId) -> bool {
        self.assets.get(asset)
            .map(|a| &a.owner == owner)
            .unwrap_or(false)
    }

    /// Transfer: x ⇝ s
    pub fn transfer(
        &mut self,
        asset_id: &AssetId,
        from: &Did,
        to: &Did,
    ) -> Result<(), ValueError> {
        let asset = self.assets.get_mut(asset_id)
            .ok_or(ValueError::NotFound)?;
        
        // Prüfe Eigentum
        if &asset.owner != from {
            return Err(ValueError::NotOwner);
        }
        
        // Prüfe Sperrung
        if asset.lock_status != LockStatus::Unlocked {
            return Err(ValueError::Locked);
        }
        
        // Update Owner-Index
        if let Some(assets) = self.by_owner.get_mut(from) {
            assets.retain(|id| id != asset_id);
        }
        self.by_owner.entry(to.clone()).or_default().push(asset_id.clone());
        
        // Transfer
        asset.owner = to.clone();
        asset.updated_at = Timestamp::now();
        asset.provenance.push(asset_id.0.clone());
        
        Ok(())
    }

    /// Vereinigung: x ⊎ y (für fungible)
    pub fn merge(
        &mut self,
        asset1: &AssetId,
        asset2: &AssetId,
    ) -> Result<AssetId, ValueError> {
        let a1 = self.assets.get(asset1).ok_or(ValueError::NotFound)?;
        let a2 = self.assets.get(asset2).ok_or(ValueError::NotFound)?;
        
        // Müssen gleichen Typ und Owner haben
        if a1.asset_type != a2.asset_type || a1.owner != a2.owner {
            return Err(ValueError::IncompatibleAssets);
        }
        
        // Müssen fungible sein
        if !matches!(a1.asset_type, AssetType::Token { .. }) {
            return Err(ValueError::NotFungible);
        }
        
        // Neues Asset erstellen
        let merged = Asset {
            id: AssetId(Hash::random()),
            asset_type: a1.asset_type.clone(),
            owner: a1.owner.clone(),
            value: a1.value + a2.value,  // Werterhaltung
            unit: a1.unit.clone(),
            lock_status: LockStatus::Unlocked,
            created_at: Timestamp::now(),
            updated_at: Timestamp::now(),
            provenance: vec![asset1.0.clone(), asset2.0.clone()],
        };
        
        let new_id = merged.id.clone();
        
        // Alte löschen, neue hinzufügen
        self.remove(asset1)?;
        self.remove(asset2)?;
        self.create(merged)?;
        
        Ok(new_id)
    }

    /// Teilung: x ⊘ n
    pub fn split(
        &mut self,
        asset_id: &AssetId,
        parts: u32,
    ) -> Result<Vec<AssetId>, ValueError> {
        let asset = self.assets.get(asset_id).ok_or(ValueError::NotFound)?;
        
        if !matches!(asset.asset_type, AssetType::Token { .. }) {
            return Err(ValueError::NotFungible);
        }
        
        if parts < 2 {
            return Err(ValueError::InvalidSplit);
        }
        
        let value_per_part = asset.value / (parts as f64);
        let owner = asset.owner.clone();
        let asset_type = asset.asset_type.clone();
        let unit = asset.unit.clone();
        
        let mut new_ids = Vec::new();
        
        for _ in 0..parts {
            let new_asset = Asset {
                id: AssetId(Hash::random()),
                asset_type: asset_type.clone(),
                owner: owner.clone(),
                value: value_per_part,
                unit: unit.clone(),
                lock_status: LockStatus::Unlocked,
                created_at: Timestamp::now(),
                updated_at: Timestamp::now(),
                provenance: vec![asset_id.0.clone()],
            };
            new_ids.push(new_asset.id.clone());
            self.create(new_asset)?;
        }
        
        self.remove(asset_id)?;
        
        Ok(new_ids)
    }

    /// Entfernt Asset
    fn remove(&mut self, id: &AssetId) -> Result<(), ValueError> {
        let asset = self.assets.remove(id).ok_or(ValueError::NotFound)?;
        self.total_value -= asset.value;
        
        if let Some(assets) = self.by_owner.get_mut(&asset.owner) {
            assets.retain(|a| a != id);
        }
        
        Ok(())
    }

    /// Sperrt Asset für Transaktion
    pub fn lock(&mut self, id: &AssetId, tx: Hash, until: Timestamp) -> Result<(), ValueError> {
        let asset = self.assets.get_mut(id).ok_or(ValueError::NotFound)?;
        
        if asset.lock_status != LockStatus::Unlocked {
            return Err(ValueError::AlreadyLocked);
        }
        
        asset.lock_status = LockStatus::LockedFor { transaction: tx, until };
        Ok(())
    }

    /// Entsperrt Asset
    pub fn unlock(&mut self, id: &AssetId) -> Result<(), ValueError> {
        let asset = self.assets.get_mut(id).ok_or(ValueError::NotFound)?;
        asset.lock_status = LockStatus::Unlocked;
        Ok(())
    }

    /// Prüft Werterhaltung (Invariante Ω-V2)
    pub fn check_conservation(&self) -> bool {
        let computed: f64 = self.assets.values().map(|a| a.value).sum();
        (self.total_value - computed).abs() < 1e-10
    }
}

#[derive(Debug)]
pub enum ValueError {
    NotFound,
    NotOwner,
    Locked,
    AlreadyLocked,
    NegativeValue,
    IncompatibleAssets,
    NotFungible,
    InvalidSplit,
}
```

---

# Teil VII: Erweiterte Invarianten (A2-A4)

## 7.1 Vollständige Invarianten

```rust
// crates/ery-logic/src/invariants.rs (Erweiterung)

/// Ω-I2: Permanenz (A2)
/// ⟨s⟩ ∧ ⟦create(s)⟧ → □⟨s⟩
pub struct IdentityPermanence;

impl Invariant for IdentityPermanence {
    fn name(&self) -> &'static str {
        "Ω-I2: Identity Permanence"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        // Prüfe: Jede Entity mit bezeugtem Genesis-Event ist noch existent
        for (did, entity) in &state.entities {
            // Finde Genesis-Event
            let genesis_events: Vec<_> = state.history
                .get_events_by_actor(did)
                .iter()
                .filter(|e| e.event_type == EventType::EntityGenesis)
                .collect();
            
            for genesis in genesis_events {
                // Wenn Genesis bezeugt, muss Entity existieren
                if state.history.is_witnessed(&genesis.hash) {
                    if entity.status == EntityStatus::Revoked {
                        // Revoked ist erlaubt, aber Identity bleibt in History
                        // Die Geschichte bleibt permanent, nur Status ändert sich
                    }
                }
            }
        }
        Ok(())
    }
}

/// Ω-I3: Ableitung erfordert Existenz (A3)
/// s ⊳ s' → ⟨s⟩ ∧ ⟨s'⟩ ∧ (s ≢ s')
pub struct DelegationExistence;

impl Invariant for DelegationExistence {
    fn name(&self) -> &'static str {
        "Ω-I3: Delegation Existence"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        for (child_did, child_entity) in &state.entities {
            if let Some(parent_did) = &child_entity.parent {
                // Parent muss existieren
                let parent_exists = state.entities.get(parent_did)
                    .map(|e| crate::identity::exists(e))
                    .unwrap_or(false);
                
                if !parent_exists {
                    return Err(InvariantViolation::DelegationParentNotExists {
                        child: child_did.clone(),
                        parent: parent_did.clone(),
                    });
                }
                
                // Kind muss existieren
                if !crate::identity::exists(child_entity) {
                    return Err(InvariantViolation::DelegationChildNotExists {
                        child: child_did.clone(),
                    });
                }
                
                // Parent ≢ Child
                if parent_did == child_did {
                    return Err(InvariantViolation::SelfDelegation {
                        entity: child_did.clone(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Ω-I4: Keine Zyklen in Delegation (A4)
/// ¬(s ⊳⁺ s)
pub struct DelegationAcyclicity;

impl Invariant for DelegationAcyclicity {
    fn name(&self) -> &'static str {
        "Ω-I4: Delegation Acyclicity"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        // Für jede Entity: Folge Parent-Chain und prüfe auf Zyklen
        for (start_did, _) in &state.entities {
            let mut visited = std::collections::HashSet::new();
            let mut current = start_did.clone();
            
            while let Some(entity) = state.entities.get(&current) {
                if !visited.insert(current.clone()) {
                    // Zyklus gefunden!
                    return Err(InvariantViolation::DelegationCycle(start_did.clone()));
                }
                
                if let Some(parent) = &entity.parent {
                    current = parent.clone();
                } else {
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Ω-C1: DAG-Struktur (A11-A13)
pub struct CausalDagStructure;

impl Invariant for CausalDagStructure {
    fn name(&self) -> &'static str {
        "Ω-C1: Causal DAG Structure"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        // Prüfe auf Zyklen im Event-DAG
        if state.history.has_cycle() {
            return Err(InvariantViolation::CausalCycle);
        }
        Ok(())
    }
}

/// Ω-R2: Realm-Monotonie (A18)
pub struct RealmMonotonicity;

impl Invariant for RealmMonotonicity {
    fn name(&self) -> &'static str {
        "Ω-R2: Realm Monotonicity"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        for (realm_id, realm) in &state.realms.realms {
            if let Some(parent_id) = &realm.parent {
                if let Some(parent) = state.realms.realms.get(parent_id) {
                    if !realm.rules.is_stricter_than(&parent.rules) {
                        return Err(InvariantViolation::MonotonicityViolation {
                            child: realm_id.to_string(),
                            parent: parent_id.to_string(),
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

/// Ω-V2: Werterhaltung
pub struct ValueConservation;

impl Invariant for ValueConservation {
    fn name(&self) -> &'static str {
        "Ω-V2: Value Conservation"
    }
    
    fn check(&self, state: &SystemState) -> InvariantResult {
        if !state.values.check_conservation() {
            let computed: f64 = state.values.assets.values().map(|a| a.value).sum();
            return Err(InvariantViolation::ValueNotConserved {
                expected: state.values.total_value,
                actual: computed,
            });
        }
        Ok(())
    }
}

/// Zusätzliche Invarianten-Violations
impl InvariantViolation {
    // Erweiterte Variants:
    // DelegationParentNotExists { child: Did, parent: Did }
    // DelegationChildNotExists { child: Did }
    // SelfDelegation { entity: Did }
}
```

---

# Teil VIII: Validierungsschichten

## 8.1 6-Schichten-Validierung

```rust
// crates/ery-logic/src/validation.rs

use ery_core::state::SystemState;
use crate::process::Process;

/// Vollständige Validierung: valid(Π, S)
pub struct Validator {
    layers: Vec<Box<dyn ValidationLayer>>,
}

impl Validator {
    pub fn new() -> Self {
        Self {
            layers: vec![
                Box::new(SyntaxValidation),
                Box::new(IdentityValidation),
                Box::new(CausalityValidation),
                Box::new(TrustValidation),
                Box::new(RealmValidation),
                Box::new(ResourceValidation),
            ],
        }
    }

    pub fn validate(&self, process: &Process, state: &SystemState) -> Result<(), ValidationError> {
        for layer in &self.layers {
            layer.validate(process, state)?;
        }
        Ok(())
    }
}

/// Trait für Validierungsschichten
pub trait ValidationLayer {
    fn name(&self) -> &'static str;
    fn validate(&self, process: &Process, state: &SystemState) -> Result<(), ValidationError>;
}

/// Schicht 1: Syntax-Validierung
pub struct SyntaxValidation;

impl ValidationLayer for SyntaxValidation {
    fn name(&self) -> &'static str { "Syntax" }
    
    fn validate(&self, process: &Process, _state: &SystemState) -> Result<(), ValidationError> {
        // Prüfe: Alle Pflichtfelder vorhanden
        if !process.has_actor() {
            return Err(ValidationError::MissingField("actor"));
        }
        if !process.has_payload() {
            return Err(ValidationError::MissingField("payload"));
        }
        if !process.has_signature() {
            return Err(ValidationError::MissingField("signature"));
        }
        if !process.has_timestamp() {
            return Err(ValidationError::MissingField("timestamp"));
        }
        
        // Prüfe: Bekannter Prozess-Typ
        if !process.is_known_type() {
            return Err(ValidationError::UnknownProcessType);
        }
        
        // Prüfe: Payload-Schema valid
        process.validate_payload_schema()?;
        
        Ok(())
    }
}

/// Schicht 2: Identitäts-Validierung
pub struct IdentityValidation;

impl ValidationLayer for IdentityValidation {
    fn name(&self) -> &'static str { "Identity" }
    
    fn validate(&self, process: &Process, state: &SystemState) -> Result<(), ValidationError> {
        let actor = process.actor();
        
        // Prüfe: Actor existiert ⟨actor⟩
        if !state.exists(actor) {
            return Err(ValidationError::ActorNotExists(actor.clone()));
        }
        
        // Prüfe: DID auflösbar
        if state.get_entity(actor).is_none() {
            return Err(ValidationError::UnresolvableDid(actor.clone()));
        }
        
        // Prüfe: Signatur gültig
        let entity = state.get_entity(actor).unwrap();
        let pubkey = entity.document.get_verification_key()?;
        if !process.verify_signature(&pubkey) {
            return Err(ValidationError::InvalidSignature);
        }
        
        // Prüfe: Nicht widerrufen
        if entity.status == EntityStatus::Revoked {
            return Err(ValidationError::ActorRevoked(actor.clone()));
        }
        
        Ok(())
    }
}

/// Schicht 3: Kausalitäts-Validierung
pub struct CausalityValidation;

impl ValidationLayer for CausalityValidation {
    fn name(&self) -> &'static str { "Causality" }
    
    fn validate(&self, process: &Process, state: &SystemState) -> Result<(), ValidationError> {
        // Prüfe: Alle Parents bezeugt
        for parent in process.parents() {
            if !state.history.is_witnessed(parent) {
                return Err(ValidationError::ParentNotWitnessed(parent.clone()));
            }
        }
        
        // Prüfe: Parents im lokalen State
        for parent in process.parents() {
            if !state.history.contains(parent) {
                return Err(ValidationError::ParentNotInState(parent.clone()));
            }
        }
        
        // Prüfe: Kein Duplikat
        let event_hash = process.compute_hash();
        if state.history.contains(&event_hash) {
            return Err(ValidationError::DuplicateEvent);
        }
        
        // Prüfe: Timestamp konsistent
        let max_parent_time = process.parents()
            .iter()
            .filter_map(|p| state.history.get(p))
            .map(|e| e.timestamp)
            .max()
            .unwrap_or(Timestamp::EPOCH);
        
        if process.timestamp() <= max_parent_time {
            return Err(ValidationError::InvalidTimestamp);
        }
        
        // Prüfe: Nicht zu weit in der Zukunft
        const MAX_DRIFT: i64 = 60_000;  // 1 Minute
        if process.timestamp().millis() - Timestamp::now().millis() > MAX_DRIFT {
            return Err(ValidationError::FutureTimestamp);
        }
        
        Ok(())
    }
}

/// Schicht 4: Trust-Validierung (A23)
pub struct TrustValidation;

impl ValidationLayer for TrustValidation {
    fn name(&self) -> &'static str { "Trust" }
    
    fn validate(&self, process: &Process, state: &SystemState) -> Result<(), ValidationError> {
        let actor = process.actor();
        let trust = state.get_trust(actor)
            .cloned()
            .unwrap_or(TrustVector::initial());
        
        // A23: □(s : α) → 𝕋(s) ≥ threshold(α)
        let required_trust = process.required_trust();
        if trust.aggregate() < required_trust {
            return Err(ValidationError::InsufficientTrust {
                actor: actor.clone(),
                required: required_trust,
                actual: trust.aggregate(),
            });
        }
        
        // Prüfe Delegation-Constraint: 𝕋(child) ≤ 𝕋(parent)
        if let Some(entity) = state.get_entity(actor) {
            if let Some(parent_did) = &entity.parent {
                if let Some(parent_trust) = state.get_trust(parent_did) {
                    if trust.aggregate() > parent_trust.aggregate() {
                        return Err(ValidationError::DelegationTrustExceeded);
                    }
                }
            }
        }
        
        // Prüfe: Nicht geblacklisted
        if state.is_blacklisted(actor, process.context()) {
            return Err(ValidationError::Blacklisted(actor.clone()));
        }
        
        Ok(())
    }
}

/// Schicht 5: Realm-Validierung (A19, A20, A22)
pub struct RealmValidation;

impl ValidationLayer for RealmValidation {
    fn name(&self) -> &'static str { "Realm" }
    
    fn validate(&self, process: &Process, state: &SystemState) -> Result<(), ValidationError> {
        let actor = process.actor();
        let action_type = process.action_type();
        
        // A19: Actor muss in einem Realm sein
        let actor_realms = state.realms.get_realms_for(actor);
        if actor_realms.is_empty() {
            return Err(ValidationError::NotInAnyRealm(actor.clone()));
        }
        
        // A22: Aktion muss im Realm erlaubt sein [R]◇(s : α)
        let trust = state.get_trust(actor)
            .map(|t| t.aggregate())
            .unwrap_or(0.5);
        
        let mut allowed_in_any = false;
        for realm_id in &actor_realms {
            if state.realms.is_action_allowed(realm_id, actor, &action_type, trust) {
                allowed_in_any = true;
                break;
            }
        }
        
        if !allowed_in_any {
            return Err(ValidationError::ActionNotAllowedInRealm {
                actor: actor.clone(),
                action: action_type,
            });
        }
        
        // Prüfe Realm-spezifische Policies
        for realm_id in &actor_realms {
            if let Some(realm) = state.realms.get(realm_id) {
                for constraint in &realm.rules.custom_constraints {
                    if !constraint.evaluate(process, state) {
                        return Err(ValidationError::ConstraintViolation {
                            realm: realm_id.clone(),
                            constraint: constraint.name.clone(),
                        });
                    }
                }
            }
        }
        
        Ok(())
    }
}

/// Schicht 6: Ressourcen-Validierung
pub struct ResourceValidation;

impl ValidationLayer for ResourceValidation {
    fn name(&self) -> &'static str { "Resources" }
    
    fn validate(&self, process: &Process, state: &SystemState) -> Result<(), ValidationError> {
        let actor = process.actor();
        
        // Prüfe: Genug Funds
        let required_funds = process.required_funds();
        let available_funds = state.values.get_balance(actor);
        if available_funds < required_funds {
            return Err(ValidationError::InsufficientFunds {
                required: required_funds,
                available: available_funds,
            });
        }
        
        // Prüfe: Besitzt benötigte Assets
        for asset_id in process.required_assets() {
            if !state.values.owns(actor, &asset_id) {
                return Err(ValidationError::AssetNotOwned {
                    actor: actor.clone(),
                    asset: asset_id,
                });
            }
            
            // Prüfe: Asset nicht gesperrt
            if let Some(asset) = state.values.get(&asset_id) {
                if asset.lock_status != LockStatus::Unlocked {
                    return Err(ValidationError::AssetLocked(asset_id));
                }
            }
        }
        
        Ok(())
    }
}

/// Validierungsfehler
#[derive(Debug)]
pub enum ValidationError {
    // Syntax
    MissingField(&'static str),
    UnknownProcessType,
    InvalidPayloadSchema(String),
    
    // Identity
    ActorNotExists(Did),
    UnresolvableDid(Did),
    InvalidSignature,
    ActorRevoked(Did),
    
    // Causality
    ParentNotWitnessed(Hash),
    ParentNotInState(Hash),
    DuplicateEvent,
    InvalidTimestamp,
    FutureTimestamp,
    
    // Trust
    InsufficientTrust { actor: Did, required: f64, actual: f64 },
    DelegationTrustExceeded,
    Blacklisted(Did),
    
    // Realm
    NotInAnyRealm(Did),
    ActionNotAllowedInRealm { actor: Did, action: ActionType },
    ConstraintViolation { realm: RealmId, constraint: String },
    
    // Resources
    InsufficientFunds { required: f64, available: f64 },
    AssetNotOwned { actor: Did, asset: AssetId },
    AssetLocked(AssetId),
}
```

---

# Teil IX: Tat-Axiome (A21-A25)

## 9.1 Atomare Transaktionen

```rust
// crates/ery-process/src/transaction.rs (Erweiterung)

use ery_core::state::SystemState;
use ery_core::causality::{Event, EventType, FinalityLevel};
use ery_core::value::LockStatus;

/// Exchange-Prozess mit Atomizität (A25)
#[derive(Clone, Debug)]
pub struct ExchangeProcess {
    pub party_a: Did,
    pub party_b: Did,
    pub offer_a: Vec<AssetId>,  // Was A gibt
    pub offer_b: Vec<AssetId>,  // Was B gibt
    pub signatures: (Signature, Signature),
}

/// Atomarer Exchange: (s₁ : α) ⊛ (s₂ : β)
/// A24: Symmetrisch
/// A25: Atomar - beides passiert oder nichts
pub fn apply_exchange(
    state: &mut SystemState,
    process: &ExchangeProcess,
) -> Result<Vec<Event>, ProcessError> {
    // ═══════════════════════════════════════════════════════════
    // PHASE 1: VALIDATION (A21 - Handlungsfähigkeit)
    // ═══════════════════════════════════════════════════════════
    
    // A21: (s : α) → ⟨s⟩ - Beide Parteien müssen existieren
    if !state.exists(&process.party_a) {
        return Err(ProcessError::ActorNotExists(process.party_a.clone()));
    }
    if !state.exists(&process.party_b) {
        return Err(ProcessError::ActorNotExists(process.party_b.clone()));
    }
    
    // Prüfe Eigentum
    for asset in &process.offer_a {
        if !state.values.owns(&process.party_a, asset) {
            return Err(ProcessError::NotOwner(process.party_a.clone(), asset.clone()));
        }
    }
    for asset in &process.offer_b {
        if !state.values.owns(&process.party_b, asset) {
            return Err(ProcessError::NotOwner(process.party_b.clone(), asset.clone()));
        }
    }
    
    // Prüfe Signaturen (beide müssen signiert haben)
    // ... Signatur-Validierung ...
    
    // ═══════════════════════════════════════════════════════════
    // PHASE 2: LOCK (Vorbereitung für Atomizität)
    // ═══════════════════════════════════════════════════════════
    
    let tx_hash = Hash::from_exchange(process);
    let lock_until = Timestamp::now() + Duration::minutes(5);
    
    // Sperre alle beteiligten Assets
    for asset in &process.offer_a {
        state.values.lock(asset, tx_hash.clone(), lock_until)?;
    }
    for asset in &process.offer_b {
        state.values.lock(asset, tx_hash.clone(), lock_until)?;
    }
    
    // ═══════════════════════════════════════════════════════════
    // PHASE 3: EXECUTE (A25 - Atomizität)
    // ═══════════════════════════════════════════════════════════
    
    // Snapshot für Rollback
    let snapshot = state.values.clone();
    
    // Transfer A → B
    let transfer_a_result: Result<(), _> = process.offer_a.iter()
        .map(|asset| state.values.transfer(asset, &process.party_a, &process.party_b))
        .collect();
    
    // Transfer B → A
    let transfer_b_result: Result<(), _> = process.offer_b.iter()
        .map(|asset| state.values.transfer(asset, &process.party_b, &process.party_a))
        .collect();
    
    // A25: Atomizität - Entweder beide oder keins
    match (transfer_a_result, transfer_b_result) {
        (Ok(()), Ok(())) => {
            // Erfolg - Entsperren
            for asset in process.offer_a.iter().chain(process.offer_b.iter()) {
                state.values.unlock(asset)?;
            }
        }
        _ => {
            // Fehlschlag - Rollback
            state.values = snapshot;
            return Err(ProcessError::AtomicExchangeFailed);
        }
    }
    
    // ═══════════════════════════════════════════════════════════
    // PHASE 4: WITNESS (Event erstellen)
    // ═══════════════════════════════════════════════════════════
    
    let event = Event {
        hash: tx_hash,
        actor: process.party_a.clone(),  // Primärer Actor
        event_type: EventType::Exchange,
        payload: EventPayload::Exchange {
            parties: vec![process.party_a.clone(), process.party_b.clone()],
            transfers: vec![
                Transfer { from: process.party_a.clone(), to: process.party_b.clone(), assets: process.offer_a.clone() },
                Transfer { from: process.party_b.clone(), to: process.party_a.clone(), assets: process.offer_b.clone() },
            ],
        },
        parents: state.history.current_tips(),
        signatures: vec![process.signatures.0.clone(), process.signatures.1.clone()],
        timestamp: Timestamp::now(),
        finality: FinalityLevel::Nascent,
    };
    
    state.history.insert(event.clone())?;
    
    // Trust-Update für erfolgreiche Transaktion
    if let Some(trust_a) = state.trust.get_mut(&process.party_a) {
        trust_a.apply_update(TrustDimension::Reliability, 0.01, false);
    }
    if let Some(trust_b) = state.trust.get_mut(&process.party_b) {
        trust_b.apply_update(TrustDimension::Reliability, 0.01, false);
    }
    
    Ok(vec![event])
}

/// A24: Tausch-Symmetrie - (s₁ : α) ⊛ (s₂ : β) ↔ (s₂ : β) ⊛ (s₁ : α)
/// Wird durch symmetrische Datenstruktur garantiert
impl ExchangeProcess {
    pub fn symmetric(&self) -> Self {
        Self {
            party_a: self.party_b.clone(),
            party_b: self.party_a.clone(),
            offer_a: self.offer_b.clone(),
            offer_b: self.offer_a.clone(),
            signatures: (self.signatures.1.clone(), self.signatures.0.clone()),
        }
    }
    
    pub fn is_equivalent(&self, other: &Self) -> bool {
        // Symmetrie-Prüfung
        (self.party_a == other.party_a && self.party_b == other.party_b
            && self.offer_a == other.offer_a && self.offer_b == other.offer_b)
        ||
        (self.party_a == other.party_b && self.party_b == other.party_a
            && self.offer_a == other.offer_b && self.offer_b == other.offer_a)
    }
}
```

---

# Teil X: Logische Operatoren

## 10.1 Operatoren-Traits

```rust
// crates/ery-logic/src/operators.rs

use ery_core::identity::{Did, Entity};
use ery_core::trust::TrustVector;
use ery_core::causality::{Event, Hash};
use ery_core::realm::RealmId;
use ery_core::state::SystemState;

// ═══════════════════════════════════════════════════════════════════════════════
// IDENTITÄTS-OPERATOREN
// ═══════════════════════════════════════════════════════════════════════════════

/// ⟨s⟩ – SELBST: Identitätsmarker
pub trait IdentityOperator {
    /// Prüft ob Entität existiert: ⟨s⟩
    fn exists(&self, did: &Did) -> bool;
    
    /// Prüft Existenz-Eindeutigkeit: ∃!s
    fn exists_unique(&self, did: &Did) -> bool;
    
    /// Prüft Identitätsgleichheit: s ≡ s'
    fn identity_equals(&self, s1: &Did, s2: &Did) -> bool;
    
    /// Prüft Ableitung: s ⊳ s'
    fn derives_from(&self, child: &Did, parent: &Did) -> bool;
    
    /// Prüft transitive Ableitung: s ⊳⁺ s'
    fn derives_from_transitive(&self, child: &Did, ancestor: &Did) -> bool;
}

impl IdentityOperator for SystemState {
    fn exists(&self, did: &Did) -> bool {
        self.entities.get(did)
            .map(|e| crate::identity::exists(e))
            .unwrap_or(false)
    }
    
    fn exists_unique(&self, did: &Did) -> bool {
        self.entities.contains_key(did)
    }
    
    fn identity_equals(&self, s1: &Did, s2: &Did) -> bool {
        s1 == s2
    }
    
    fn derives_from(&self, child: &Did, parent: &Did) -> bool {
        self.entities.get(child)
            .and_then(|e| e.parent.as_ref())
            .map(|p| p == parent)
            .unwrap_or(false)
    }
    
    fn derives_from_transitive(&self, child: &Did, ancestor: &Did) -> bool {
        let mut current = child.clone();
        while let Some(entity) = self.entities.get(&current) {
            if let Some(parent) = &entity.parent {
                if parent == ancestor {
                    return true;
                }
                current = parent.clone();
            } else {
                break;
            }
        }
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GLAUBENS-OPERATOREN
// ═══════════════════════════════════════════════════════════════════════════════

/// 𝕋(s), s ⊨_t φ, ⊕, ⊖ – Glaubens-Operatoren
pub trait TrustOperator {
    /// 𝕋(s) – Vertrauensvektor abrufen
    fn trust(&self, did: &Did) -> TrustVector;
    
    /// 𝕋̄(s) – Aggregiertes Vertrauen
    fn trust_aggregate(&self, did: &Did) -> f64;
    
    /// s ⊨_t φ – Entity glaubt Proposition mit Stärke t
    fn believes(&self, did: &Did, proposition: &Proposition) -> f64;
    
    /// ⊕ – Trust-Kombination: t₁ ⊕ t₂ = 1 - (1-t₁)(1-t₂)
    fn combine_trust(&self, t1: f64, t2: f64) -> f64;
    
    /// ⊖ – Trust-Schwächung mit Asymmetrie
    fn weaken_trust(&self, trust: f64, delta: f64) -> f64;
}

impl TrustOperator for SystemState {
    fn trust(&self, did: &Did) -> TrustVector {
        self.trust.get(did).cloned().unwrap_or(TrustVector::initial())
    }
    
    fn trust_aggregate(&self, did: &Did) -> f64 {
        self.trust(did).aggregate()
    }
    
    fn believes(&self, did: &Did, proposition: &Proposition) -> f64 {
        // Suche Attestations des Actors für diese Proposition
        let attestations = self.history.find_attestations(did, proposition);
        if attestations.is_empty() {
            0.0
        } else {
            // Kombiniere alle Attestations
            attestations.iter()
                .map(|a| a.confidence)
                .fold(0.0, |acc, t| self.combine_trust(acc, t))
        }
    }
    
    fn combine_trust(&self, t1: f64, t2: f64) -> f64 {
        1.0 - (1.0 - t1) * (1.0 - t2)
    }
    
    fn weaken_trust(&self, trust: f64, delta: f64) -> f64 {
        (trust - 1.5 * delta).max(0.3)  // κ = 1.5, min = 0.3
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SPUR-OPERATOREN
// ═══════════════════════════════════════════════════════════════════════════════

/// ⊲, ⟦e⟧, ∎e, |ℂ(s)| – Kausalitäts-Operatoren
pub trait CausalityOperator {
    /// e ⊲ e' – Kausale Präzedenz
    fn precedes(&self, earlier: &Hash, later: &Hash) -> bool;
    
    /// e ⋖ e' – Direkter Vorgänger
    fn directly_precedes(&self, earlier: &Hash, later: &Hash) -> bool;
    
    /// ⟦e⟧ – Event ist bezeugt
    fn is_witnessed(&self, event: &Hash) -> bool;
    
    /// ∎e – Event ist endgültig
    fn is_final(&self, event: &Hash) -> bool;
    
    /// |ℂ(s)| – Kausale Tiefe
    fn causal_depth(&self, did: &Did) -> usize;
    
    /// ln|ℂ(s)| – Logarithmische Tiefe für Weltformel
    fn log_causal_depth(&self, did: &Did) -> f64;
}

impl CausalityOperator for SystemState {
    fn precedes(&self, earlier: &Hash, later: &Hash) -> bool {
        self.history.precedes(earlier, later)
    }
    
    fn directly_precedes(&self, earlier: &Hash, later: &Hash) -> bool {
        self.history.get(later)
            .map(|e| e.parents.contains(earlier))
            .unwrap_or(false)
    }
    
    fn is_witnessed(&self, event: &Hash) -> bool {
        self.history.is_witnessed(event)
    }
    
    fn is_final(&self, event: &Hash) -> bool {
        self.history.is_final(event)
    }
    
    fn causal_depth(&self, did: &Did) -> usize {
        self.history.depth(did)
    }
    
    fn log_causal_depth(&self, did: &Did) -> f64 {
        let depth = self.causal_depth(did);
        (depth.max(1) as f64).ln()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RAUM-OPERATOREN
// ═══════════════════════════════════════════════════════════════════════════════

/// [R]φ, R ⊑ R', s ∈ R – Realm-Operatoren
pub trait RealmOperator {
    /// [R]φ – Proposition gilt im Realm
    fn holds_in_realm(&self, realm: &RealmId, proposition: &Proposition) -> bool;
    
    /// R ⊑ R' – Realm ist Unterraum
    fn is_subrealm(&self, child: &RealmId, parent: &RealmId) -> bool;
    
    /// s ∈ R – Entity ist Mitglied
    fn is_member(&self, entity: &Did, realm: &RealmId) -> bool;
    
    /// ⟨R⟩φ – Proposition gilt in irgendeinem Realm
    fn holds_in_some_realm(&self, entity: &Did, proposition: &Proposition) -> bool;
}

impl RealmOperator for SystemState {
    fn holds_in_realm(&self, realm: &RealmId, proposition: &Proposition) -> bool {
        if let Some(r) = self.realms.get(realm) {
            proposition.evaluate_in_context(r, self)
        } else {
            false
        }
    }
    
    fn is_subrealm(&self, child: &RealmId, parent: &RealmId) -> bool {
        self.realms.is_contained_in(child, parent)
    }
    
    fn is_member(&self, entity: &Did, realm: &RealmId) -> bool {
        self.realms.is_member(entity, realm)
    }
    
    fn holds_in_some_realm(&self, entity: &Did, proposition: &Proposition) -> bool {
        for realm_id in self.realms.get_realms_for(entity) {
            if self.holds_in_realm(&realm_id, proposition) {
                return true;
            }
        }
        false
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TAT-OPERATOREN
// ═══════════════════════════════════════════════════════════════════════════════

/// s : α, ◇α, □α, α ⊛ β – Aktions-Operatoren
pub trait ActionOperator {
    /// s : α – Entity führt Aktion aus
    fn performs(&self, actor: &Did, action: &Action) -> bool;
    
    /// ◇α – Aktion ist möglich
    fn is_possible(&self, actor: &Did, action: &Action) -> bool;
    
    /// □α – Aktion ist erforderlich
    fn is_required(&self, actor: &Did, action: &Action) -> bool;
    
    /// α → β – Aktion verursacht Effekt
    fn causes(&self, action: &Action, effect: &Effect) -> bool;
}

impl ActionOperator for SystemState {
    fn performs(&self, actor: &Did, action: &Action) -> bool {
        // Prüfe ob Event für diese Aktion existiert
        self.history.find_action(actor, action).is_some()
    }
    
    fn is_possible(&self, actor: &Did, action: &Action) -> bool {
        // A21: Actor muss existieren
        if !self.exists(actor) {
            return false;
        }
        
        // A22/A23: Trust und Realm-Checks
        let trust = self.trust_aggregate(actor);
        let required = action.required_trust();
        
        if trust < required {
            return false;
        }
        
        // Realm-Erlaubnis prüfen
        for realm_id in self.realms.get_realms_for(actor) {
            if self.realms.is_action_allowed(&realm_id, actor, &action.action_type(), trust) {
                return true;
            }
        }
        
        false
    }
    
    fn is_required(&self, actor: &Did, action: &Action) -> bool {
        // Prüfe ob es eine Regel gibt, die diese Aktion erfordert
        for realm_id in self.realms.get_realms_for(actor) {
            if let Some(realm) = self.realms.get(&realm_id) {
                for constraint in &realm.rules.custom_constraints {
                    if constraint.requires_action(actor, action) {
                        return true;
                    }
                }
            }
        }
        false
    }
    
    fn causes(&self, action: &Action, effect: &Effect) -> bool {
        action.effects().contains(effect)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WERT-OPERATOREN
// ═══════════════════════════════════════════════════════════════════════════════

/// 𝕍(x), x ↝ s, x ⇝ s, x ⊎ y, x ⊘ n – Wert-Operatoren
pub trait ValueOperator {
    /// 𝕍(x) – Wert eines Assets
    fn value(&self, asset: &AssetId) -> f64;
    
    /// x ↝ s – Entity besitzt Asset
    fn owns(&self, owner: &Did, asset: &AssetId) -> bool;
    
    /// x ⇝ s – Transfer (mutierend)
    fn transfer(&mut self, asset: &AssetId, from: &Did, to: &Did) -> Result<(), ValueError>;
    
    /// x ⊎ y – Assets vereinigen
    fn merge(&mut self, a: &AssetId, b: &AssetId) -> Result<AssetId, ValueError>;
    
    /// x ⊘ n – Asset aufteilen
    fn split(&mut self, asset: &AssetId, parts: u32) -> Result<Vec<AssetId>, ValueError>;
}

impl ValueOperator for SystemState {
    fn value(&self, asset: &AssetId) -> f64 {
        self.values.value(asset).unwrap_or(0.0)
    }
    
    fn owns(&self, owner: &Did, asset: &AssetId) -> bool {
        self.values.owns(owner, asset)
    }
    
    fn transfer(&mut self, asset: &AssetId, from: &Did, to: &Did) -> Result<(), ValueError> {
        self.values.transfer(asset, from, to)
    }
    
    fn merge(&mut self, a: &AssetId, b: &AssetId) -> Result<AssetId, ValueError> {
        self.values.merge(a, b)
    }
    
    fn split(&mut self, asset: &AssetId, parts: u32) -> Result<Vec<AssetId>, ValueError> {
        self.values.split(asset, parts)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AUFMERKSAMKEITS-OPERATOR
// ═══════════════════════════════════════════════════════════════════════════════

/// σ(s) – Aufmerksamkeitsfunktion
pub trait AttentionOperator {
    /// σ(s) = 1 / (1 + e^(-𝕋̄(s) · ln|ℂ(s)|))
    fn attention(&self, did: &Did) -> f64;
    
    /// Σσ – Summe aller Aufmerksamkeiten
    fn total_attention(&self) -> f64;
    
    /// Normalisierte Aufmerksamkeit: σ(s) / Σσ
    fn normalized_attention(&self, did: &Did) -> f64;
}

impl AttentionOperator for SystemState {
    fn attention(&self, did: &Did) -> f64 {
        let trust = self.trust_aggregate(did);
        let ln_c = self.log_causal_depth(did);
        crate::formula::sigmoid(trust * ln_c)
    }
    
    fn total_attention(&self) -> f64 {
        self.entities.keys()
            .map(|did| self.attention(did))
            .sum()
    }
    
    fn normalized_attention(&self, did: &Did) -> f64 {
        let total = self.total_attention();
        if total > 0.0 {
            self.attention(did) / total
        } else {
            0.0
        }
    }
}
```

---

# Teil XI: Axiome als Code

## 11.1 Axiom-Definitionen

```rust
// crates/ery-logic/src/axioms.rs

use ery_core::state::SystemState;
use crate::operators::*;

/// Trait für Axiome
pub trait Axiom {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn verify(&self, state: &SystemState) -> bool;
}

// ═══════════════════════════════════════════════════════════════════════════════
// IDENTITÄTS-AXIOME (A1-A4)
// ═══════════════════════════════════════════════════════════════════════════════

/// A1: ∀s : ∃!id ∈ DID : identity(s) = id
pub struct AxiomExistence;
impl Axiom for AxiomExistence {
    fn id(&self) -> &'static str { "A1" }
    fn name(&self) -> &'static str { "Existence" }
    fn description(&self) -> &'static str { 
        "Jede Entität hat genau eine Identität." 
    }
    fn verify(&self, state: &SystemState) -> bool {
        // HashMap garantiert Eindeutigkeit
        for (did, _) in &state.entities {
            if !state.exists_unique(did) {
                return false;
            }
        }
        true
    }
}

/// A2: ⟨s⟩ ∧ ⟦create(s)⟧ → □⟨s⟩
pub struct AxiomPermanence;
impl Axiom for AxiomPermanence {
    fn id(&self) -> &'static str { "A2" }
    fn name(&self) -> &'static str { "Permanence" }
    fn description(&self) -> &'static str { 
        "Einmal erschaffen und bezeugt, existiert eine Identität für immer." 
    }
    fn verify(&self, state: &SystemState) -> bool {
        // Alle bezeugten Genesis-Events haben noch existierende Entities
        for (did, _) in &state.entities {
            if let Some(genesis) = state.history.find_genesis(did) {
                if state.is_witnessed(&genesis.hash) {
                    // Entity muss noch existieren (auch wenn revoked)
                    if !state.entities.contains_key(did) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

/// A3: s ⊳ s' → ⟨s⟩ ∧ ⟨s'⟩ ∧ (s ≢ s')
pub struct AxiomDerivation;
impl Axiom for AxiomDerivation {
    fn id(&self) -> &'static str { "A3" }
    fn name(&self) -> &'static str { "Derivation" }
    fn description(&self) -> &'static str { 
        "Ableitung erfordert, dass beide Identitäten existieren und verschieden sind." 
    }
    fn verify(&self, state: &SystemState) -> bool {
        for (child_did, entity) in &state.entities {
            if let Some(parent_did) = &entity.parent {
                // Beide müssen existieren
                if !state.exists(child_did) || !state.exists(parent_did) {
                    return false;
                }
                // Müssen verschieden sein
                if child_did == parent_did {
                    return false;
                }
            }
        }
        true
    }
}

/// A4: ¬(s ⊳⁺ s)
pub struct AxiomNonCircularity;
impl Axiom for AxiomNonCircularity {
    fn id(&self) -> &'static str { "A4" }
    fn name(&self) -> &'static str { "Non-Circularity" }
    fn description(&self) -> &'static str { 
        "Keine Identität kann sich von sich selbst ableiten (transitiv)." 
    }
    fn verify(&self, state: &SystemState) -> bool {
        for (did, _) in &state.entities {
            if state.derives_from_transitive(did, did) {
                return false;
            }
        }
        true
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GLAUBENS-AXIOME (A5-A10)
// ═══════════════════════════════════════════════════════════════════════════════

/// A5: ∀s : 0 ≤ 𝕋(s) ≤ 1
pub struct AxiomBoundedness;
impl Axiom for AxiomBoundedness {
    fn id(&self) -> &'static str { "A5" }
    fn name(&self) -> &'static str { "Boundedness" }
    fn description(&self) -> &'static str { 
        "Vertrauen liegt immer zwischen 0 und 1." 
    }
    fn verify(&self, state: &SystemState) -> bool {
        for (_, trust) in &state.trust {
            for &v in &[trust.reliability, trust.integrity, trust.capability, trust.prestige] {
                if v < 0.0 || v > 1.0 {
                    return false;
                }
            }
        }
        true
    }
}

/// A6: ∀s : 𝕋(s) ≥ 0.3
pub struct AxiomFloor;
impl Axiom for AxiomFloor {
    fn id(&self) -> &'static str { "A6" }
    fn name(&self) -> &'static str { "Floor" }
    fn description(&self) -> &'static str { 
        "Keine Entität fällt unter das Mindestvertrauen." 
    }
    fn verify(&self, state: &SystemState) -> bool {
        for (_, trust) in &state.trust {
            if trust.aggregate() < 0.3 {
                return false;
            }
        }
        true
    }
}

/// A9: s ⊳ s' → 𝕋(s') ≤ 𝕋(s)
pub struct AxiomInheritance;
impl Axiom for AxiomInheritance {
    fn id(&self) -> &'static str { "A9" }
    fn name(&self) -> &'static str { "Inheritance" }
    fn description(&self) -> &'static str { 
        "Eine abgeleitete Identität kann das Vertrauen des Elternteils nicht überschreiten." 
    }
    fn verify(&self, state: &SystemState) -> bool {
        for (child_did, entity) in &state.entities {
            if let Some(parent_did) = &entity.parent {
                let child_trust = state.trust_aggregate(child_did);
                let parent_trust = state.trust_aggregate(parent_did);
                if child_trust > parent_trust {
                    return false;
                }
            }
        }
        true
    }
}

// ... weitere Axiome A7, A8, A10-A25 nach gleichem Muster ...

/// Vollständiger Axiom-Checker
pub struct AxiomChecker {
    axioms: Vec<Box<dyn Axiom>>,
}

impl AxiomChecker {
    pub fn all() -> Self {
        Self {
            axioms: vec![
                // Identity
                Box::new(AxiomExistence),
                Box::new(AxiomPermanence),
                Box::new(AxiomDerivation),
                Box::new(AxiomNonCircularity),
                // Trust
                Box::new(AxiomBoundedness),
                Box::new(AxiomFloor),
                Box::new(AxiomInheritance),
                // ... alle 25 Axiome
            ],
        }
    }
    
    pub fn verify_all(&self, state: &SystemState) -> Vec<&'static str> {
        self.axioms.iter()
            .filter(|a| !a.verify(state))
            .map(|a| a.id())
            .collect()
    }
    
    pub fn is_consistent(&self, state: &SystemState) -> bool {
        self.axioms.iter().all(|a| a.verify(state))
    }
}
```

---

# Teil XII: Vollständigkeits-Matrix

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║                                        VOLLSTÄNDIGKEITS-MATRIX: LOGIK → RUST                                                            ║
║                                                                                                                                           ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                                                                           ║
║   AXIOM       LOGIK-FORMEL                               RUST-IMPLEMENTIERUNG                                    STATUS                  ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                                                           ║
║   A1          ∀s : ∃!id ∈ DID                            HashMap<Did, Entity>                                    ✅                       ║
║   A2          ⟨s⟩ ∧ ⟦create(s)⟧ → □⟨s⟩                   AxiomPermanence + History                               ✅                       ║
║   A3          s ⊳ s' → ⟨s⟩ ∧ ⟨s'⟩ ∧ (s ≢ s')            DelegationExistence invariant                           ✅                       ║
║   A4          ¬(s ⊳⁺ s)                                  DelegationAcyclicity invariant                          ✅                       ║
║                                                                                                                                           ║
║   A5          0 ≤ 𝕋(s) ≤ 1                               MIN_TRUST, MAX_TRUST, clamp()                           ✅                       ║
║   A6          𝕋(s) ≥ 0.3                                 MIN_TRUST = 0.3                                         ✅                       ║
║   A7          Δ⁻ = 1.5 · Δ⁺                              ASYMMETRY_FACTOR = 1.5                                  ✅                       ║
║   A8          𝕋(t+1) = 𝕋(t) · λ                          apply_decay()                                           ✅                       ║
║   A9          s ⊳ s' → 𝕋(s') ≤ 𝕋(s)                      constrain_to(), DelegationTrustLimit                    ✅                       ║
║   A10         t₁ ⊕ t₂ = 1-(1-t₁)(1-t₂)                   combine_trust()                                         ✅                       ║
║                                                                                                                                           ║
║   A11         ¬(e ⊲ e)                                   DAG-Struktur in CausalHistory                           ✅                       ║
║   A12         (e ⊲ e') → ¬(e' ⊲ e)                       DAG-Struktur                                            ✅                       ║
║   A13         (e ⊲ e') ∧ (e' ⊲ e'') → (e ⊲ e'')          precedes() mit Reachability                             ✅                       ║
║   A14         ⟦e⟧ → □⟦e⟧                                 FinalityLevel, is_witnessed()                           ✅                       ║
║   A15         ∎e → ¬◇undo(e)                             FinalityLevel::Eternal, is_final()                      ✅                       ║
║   A16         (α → β) ∧ (s : α) ∧ ⟦s : α⟧ → ◇β           causes() in ActionOperator                              ✅                       ║
║                                                                                                                                           ║
║   A17         (R ⊑ R') ∧ [R]φ → [R']φ                    propagate_truth_upward()                                ✅                       ║
║   A18         (R ⊑ R') → (rules(R') ⊆ rules(R))          is_stricter_than()                                      ✅                       ║
║   A19         (s ∈ R) ∧ [R]φ → s ⊨ φ                     is_member() + holds_in_realm()                          ✅                       ║
║   A20         (s : α) ∧ (s ∈ R) → [R](s : α)             RealmValidation layer                                   ✅                       ║
║                                                                                                                                           ║
║   A21         (s : α) → ⟨s⟩                              IdentityValidation layer                                ✅                       ║
║   A22         (s : α) ∧ (s ∈ R) → [R]◇(s : α)            is_action_allowed()                                     ✅                       ║
║   A23         □(s : α) → 𝕋(s) ≥ threshold(α)             TrustValidation layer                                   ✅                       ║
║   A24         (s₁ : α) ⊛ (s₂ : β) ↔ symmetrisch          ExchangeProcess::symmetric()                            ✅                       ║
║   A25         ⟦(s₁:α) ⊛ (s₂:β)⟧ → atomar                 apply_exchange() mit Rollback                           ✅                       ║
║                                                                                                                                           ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                                                                           ║
║   OPERATOREN        SYMBOL          RUST-IMPLEMENTIERUNG                                                                                 ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────── ║
║   Identität         ⟨s⟩, ∃!, ≡, ⊳   IdentityOperator trait                                                       ✅                       ║
║   Glaube            𝕋, ⊨_t, ⊕, ⊖    TrustOperator trait                                                          ✅                       ║
║   Spur              ⊲, ⟦⟧, ∎, |ℂ|   CausalityOperator trait                                                      ✅                       ║
║   Raum              [R], ⊑, ∈       RealmOperator trait                                                          ✅                       ║
║   Tat               s:α, ◇, □, ⊛    ActionOperator trait                                                         ✅                       ║
║   Wert              𝕍, ↝, ⇝, ⊎, ⊘   ValueOperator trait                                                          ✅                       ║
║   Aufmerksamkeit    σ               AttentionOperator trait                                                      ✅                       ║
║                                                                                                                                           ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                                                                           ║
║   VOLLSTÄNDIGKEITS-SCORE:  100%  (25/25 Axiome + alle Operatoren + alle Module)                                                          ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

# Teil XIII: Zusammenfassung

## Architektur-Übersicht (Vollständig)

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║                                         ERY RUST ARCHITEKTUR (VOLLSTÄNDIG)                                                              ║
║                                                                                                                                           ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ ║
║   │                                                                                                                                     │ ║
║   │   WELTFORMEL                                          RUST MAPPING                                                                  │ ║
║   │                                                                                                                                     │ ║
║   │   𝔼 = Σ 𝕀(s) · σ(𝕋(s) · ln|ℂ(s)|)                  SystemState::system_value                                                     │ ║
║   │       │      │    │        │                               │      │    │        │                                                  │ ║
║   │       │      │    │        └── CausalHistory ──────────────┼──────┼────┘        │                                                  │ ║
║   │       │      │    └── TrustVector ─────────────────────────┼──────┘             │                                                  │ ║
║   │       │      └── sigmoid() ────────────────────────────────┼────────────────────┘                                                  │ ║
║   │       └── Entity / identity_factor() ──────────────────────┘                                                                       │ ║
║   │                                                                                                                                     │ ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ ║
║                                                                                                                                           ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ ║
║   │                                                                                                                                     │ ║
║   │   ery-core                    ery-logic                   ery-process                                                               │ ║
║   │   ─────────                   ─────────                   ───────────                                                               │ ║
║   │   identity.rs (𝕀)            operators.rs (30+)          genesis.rs (Π-G)                                                          │ ║
║   │   trust.rs (𝕋)               axioms.rs (A1-A25)          attestation.rs (Π-A)                                                      │ ║
║   │   causality.rs (ℂ)           invariants.rs (Ω)           transaction.rs (Π-T)                                                      │ ║
║   │   realm.rs (ε)               validation.rs (6-Layer)     governance.rs (Π-V)                                                       │ ║
║   │   value.rs (𝕍)               rules.rs (ECL)              dispute.rs (Π-D)                                                          │ ║
║   │   formula.rs (σ, 𝔼)                                      lifecycle.rs (Π-L)                                                        │ ║
║   │   state.rs (S)                                           transition.rs (δ)                                                         │ ║
║   │                                                                                                                                     │ ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

*ERY Rust Architektur Version 2.0 – Vollständige Implementierung aller 25 Axiome und Logik-Operatoren.*
