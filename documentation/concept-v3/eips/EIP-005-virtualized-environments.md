# EIP-005: Virtualized Environment Architecture

> **EIP:** 005
> **Titel:** Virtualized Environment Architecture (Root-Env / Virt-Env / Shards)
> **Status:** Draft
> **Version:** 0.4 (FACHKONZEPT V6.2 Sync)
> **Typ:** Standard
> **Ebene:** E2 (Emergenz) / E5 (Schutz) / E6 (Kybernetik)
> **Erstellt:** Januar 2026
> **Aktualisiert:** Februar 2026
> **Abhängigkeiten:** EIP-001 (DID), EIP-002 (Trust), EIP-003 (Event-DAG), EIP-004 (Bayesian Trust)
> **Axiom-Referenz:** A18-A22 (Realms), Q6-Q8 (Kategorientheorie), E4 (Shards), **L1-L3 (Logic Guards)**

---

## Abstract

Diese Spezifikation definiert die **Virtualized Environment Architecture** für Erynoa und integriert sie mit dem bestehenden **Realm/Shard-Modell** (Axiome A18-A22) und der **Kategorientheorie** (Axiome Q6-Q8).

**Hierarchie:**

```
ROOT-ENV (𝒞_Root)
    │
    ├── VIRT-ENV: EU (𝒞_EU ⊂ 𝒞_Root)         ← Circle-Namespace Realm
    │       │
    │       ├── SHARD: Finance (𝒞_EU_Finance ⊂ 𝒞_EU)
    │       │       └── CBDC: Digital Euro
    │       │
    │       ├── SHARD: Energy (𝒞_EU_Energy ⊂ 𝒞_EU)
    │       │
    │       └── SUB-VIRT-ENV: Germany (𝒞_DE ⊂ 𝒞_EU)
    │               └── SHARD: Healthcare (𝒞_DE_Health ⊂ 𝒞_DE)
    │
    └── VIRT-ENV: ASEAN (𝒞_ASEAN ⊂ 𝒞_Root)
            │
            └── SHARD: Trade (𝒞_ASEAN_Trade ⊂ 𝒞_ASEAN)
                    └── CBDC: Multi-FX Bridge
```

**Kernkonzepte:**

1. **Root-Environment (Root-Env)**: Die globale Kategorie 𝒞_Root mit den 112 Axiomen
2. **Virtual Environments (Virt-Env)**: Sub-Kategorien im `circle`-Namespace (Axiom A18)
3. **Shards**: Spezialisierte Sub-Kategorien innerhalb einer Virt-Env (Axiom E4)
4. **CBDC-Shards**: Spezielle Shard-Typen mit Bridge-Funktoren zu externen Währungssystemen
5. **Funktoren**: Strukturerhaltende Abbildungen zwischen Kategorien (Axiom Q7)

Dieses Modell erlaubt es **souveränen Entitäten** (Staaten, Unionen, Konzernen), eigene Governance-Strukturen und Währungssysteme zu betreiben, während sie vom gemeinsamen Erynoa-Identitätssystem und Vertrauensprotokoll profitieren.

---

## V0.3 Refinements

### A. Unified Identity & Multi-Chain Onboarding

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    UNIFIED IDENTITY ARCHITECTURE                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   EINMALIGE ANMELDUNG (Seed/Passkey)                                       │
│   ══════════════════════════════════                                       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                    MASTER SECRET                                     │  │
│   │         BIP39 Mnemonic (24 Wörter) ODER WebAuthn Passkey            │  │
│   └────────────────────────────┬────────────────────────────────────────┘  │
│                                │                                            │
│                    ┌───────────┴───────────┐                               │
│                    │  DETERMINISTIC KDF    │                               │
│                    │  (HD-Derivation)      │                               │
│                    └───────────┬───────────┘                               │
│                                │                                            │
│          ┌─────────────────────┼─────────────────────┐                     │
│          │                     │                     │                     │
│          ▼                     ▼                     ▼                     │
│   ┌─────────────┐       ┌─────────────┐       ┌─────────────┐             │
│   │ Ed25519 Key │       │ secp256k1   │       │ Ed25519 Key │             │
│   │ (Primary)   │       │ (EVM-Chains)│       │ (MoveVM)    │             │
│   └──────┬──────┘       └──────┬──────┘       └──────┬──────┘             │
│          │                     │                     │                     │
│          ▼                     ▼                     ▼                     │
│   ┌─────────────┐       ┌─────────────┐       ┌─────────────┐             │
│   │ did:erynoa: │       │ 0x...       │       │ 0x...       │             │
│   │ self:alice  │       │ (Ethereum)  │       │ (IOTA/Sui)  │             │
│   └─────────────┘       └─────────────┘       └─────────────┘             │
│          │                     │                     │                     │
│          └─────────────────────┼─────────────────────┘                     │
│                                │                                            │
│                    ┌───────────┴───────────┐                               │
│                    │    DID-DOCUMENT       │                               │
│                    │  (Multi-Chain Links)  │                               │
│                    └───────────────────────┘                               │
│                                                                             │
│   PRINZIP: Eine Anmeldung → Alle Wallets → Eine Identität                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### B. Dynamic Trust Dampening (Kybernetik E6)

In V0.2 war `trust_factor` statisch. Das widerspricht dem kybernetischen Ansatz.

**V0.3: Dynamischer Dämpfungsfaktor basierend auf historischer Entropie:**

```
β_dynamic(s₁, s₂) = β_base × exp(-λ × failure_rate(s₁, s₂))
```

- Wenn viele Transaktionen zwischen EU und ASEAN scheitern, sinkt β automatisch
- Erfolgreiche Transaktionen erhöhen β (bis zum Maximum)

### C. Contextual Trust Rotation (Vektor-Transformation)

Ein hoher "Competence"-Wert im Gaming-Shard bedeutet nicht "Competence" im Medical-Shard.

**V0.3: Trust-Matrix statt skalarer Dämpfung:**

```
T_target = M × T_source

Wobei M die Transformations-Matrix ist:
- Gaming→Finance: C wird stark gedämpft (0.1), I bleibt (0.8)
- Energy→Finance: R wird übertragen (0.9), C neutral (0.5)
```

### D. Boundary Guards (Logic Guards L1-L3)

Kategorientheorie definiert Struktur, aber **Logic Guards** bewachen die Übergänge.

**V0.3: Jeder Funktor hat einen Boundary Guard (Smart Contract in ECL):**

```ecl
guard boundary_eu_to_asean {
  // GDPR-Äquivalenz prüfen
  require(target_env.has_compliance("GDPR-equivalent"))

  // Minimaler Trust
  require(source_trust.scalar() >= 0.6)

  // Sanktions-Check
  require(!sanctions_list.contains(user.did))
}
```

---

## Motivation

### Das Problem der Digitalen Souveränität

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    DAS TRILEMMA DER DIGITALEN SOUVERÄNITÄT              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   OPTION A: Globales System (z.B. Bitcoin, Ethereum)                   │
│   ✓ Interoperabilität                                                  │
│   ✗ Keine Souveränität (Staaten haben keine Kontrolle)                 │
│   ✗ Regulatorische Konflikte                                           │
│                                                                         │
│   OPTION B: Nationale Silos (z.B. separate CBDC-Systeme)               │
│   ✓ Volle Souveränität                                                 │
│   ✗ Keine Interoperabilität                                            │
│   ✗ Doppelte Infrastruktur-Kosten                                      │
│                                                                         │
│   OPTION C: ERYNOA VIRT-ENV ARCHITEKTUR                                │
│   ✓ Souveränität (eigene Governance, eigene CBDC)                      │
│   ✓ Interoperabilität (gemeinsames Identitäts- & Vertrauens-Layer)     │
│   ✓ Shared Infrastructure (Root-Env als Common Good)                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Use Cases

1. **Europäische Union**: Virt-Env mit Digital Euro als CBDC, EU-weite Governance
2. **Deutschland**: Sub-Virt-Env unter EU, mit lokalen Anpassungen (z.B. Datenschutz)
3. **BRICS**: Alternatives Virt-Env mit eigenem Settlement-Token
4. **Multinationale Konzerne**: Private Virt-Envs für Supply-Chain-Management
5. **Städte/Regionen**: Lokale Virt-Envs für Bürgerdienste

---

## Spezifikation

### 1. Architektur-Überblick

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         ROOT-ENVIRONMENT                                │
│                    (Erynoa Protocol Foundation)                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌───────────────┐    ┌───────────────┐    ┌───────────────┐          │
│   │  CORE AXIOMS  │    │  DID STANDARD │    │ TRUST VECTOR  │          │
│   │   (112 Ax.)   │    │   (EIP-001)   │    │   (EIP-002)   │          │
│   └───────────────┘    └───────────────┘    └───────────────┘          │
│                                                                         │
│   ┌───────────────┐    ┌───────────────┐    ┌───────────────┐          │
│   │   EVENT-DAG   │    │ BAYESIAN ALGO │    │   ECLVM CORE  │          │
│   │   (EIP-003)   │    │   (EIP-004)   │    │  (Sandboxed)  │          │
│   └───────────────┘    └───────────────┘    └───────────────┘          │
│                                                                         │
│   ════════════════════════════════════════════════════════════════     │
│                         VIRT-ENV LAYER                                  │
│   ════════════════════════════════════════════════════════════════     │
│                                                                         │
│   ┌──────────────────────────────────────────────────────────────┐     │
│   │  VIRT-ENV: EU (did:erynoa:circle:eu-2026)                    │     │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │     │
│   │  │ GOVERNANCE  │  │ CBDC BRIDGE │  │  LOCAL ID   │           │     │
│   │  │  (EU-DAO)   │  │(Digital EUR)│  │  (eIDAS 2)  │           │     │
│   │  └─────────────┘  └─────────────┘  └─────────────┘           │     │
│   │                                                               │     │
│   │  ┌──────────────────────────────────────────────────────┐    │     │
│   │  │  SUB-VIRT-ENV: Germany (did:erynoa:circle:de-2026)   │    │     │
│   │  │  ┌──────────┐  ┌──────────┐  ┌──────────┐            │    │     │
│   │  │  │ DE-GOV   │  │ DE-RULES │  │ DE-AUTH  │            │    │     │
│   │  │  └──────────┘  └──────────┘  └──────────┘            │    │     │
│   │  └──────────────────────────────────────────────────────┘    │     │
│   └──────────────────────────────────────────────────────────────┘     │
│                                                                         │
│   ┌──────────────────────────────────────────────────────────────┐     │
│   │  VIRT-ENV: ASEAN (did:erynoa:circle:asean-2026)              │     │
│   │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │     │
│   │  │ GOVERNANCE  │  │ CBDC BRIDGE │  │  LOCAL ID   │           │     │
│   │  │ (ASEAN-DAO) │  │ (Multi-FX)  │  │  (Local)    │           │     │
│   │  └─────────────┘  └─────────────┘  └─────────────┘           │     │
│   └──────────────────────────────────────────────────────────────┘     │
│                                                                         │
│   ═══════════════ INTER-ENV PROTOCOL (IEP) ════════════════════════    │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### 2. Integration mit Realm/Shard-Modell (Axiome A18-A22, E4, Q6-Q8)

#### 2.1 Kategorientheorie-Grundlagen

Das gesamte Erynoa-System ist als **geschachtelte Kategorien-Hierarchie** modelliert:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      KATEGORIEN-HIERARCHIE                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   MATHEMATISCHE STRUKTUR:                                                   │
│                                                                             │
│   𝒞_Root                                    ← Globale Kategorie             │
│       │                                                                     │
│       ├── 𝒞_EU  ⊂ 𝒞_Root                   ← Virt-Env (circle:eu)          │
│       │       │                                                             │
│       │       ├── 𝒞_EU_Finance ⊂ 𝒞_EU      ← Shard (finance)               │
│       │       │       │                                                     │
│       │       │       └── CBDC(EUR) ∈ Ob(𝒞_EU_Finance)                     │
│       │       │                                                             │
│       │       ├── 𝒞_EU_Energy ⊂ 𝒞_EU       ← Shard (energy)                │
│       │       │                                                             │
│       │       └── 𝒞_DE ⊂ 𝒞_EU              ← Sub-Virt-Env (circle:de)      │
│       │               │                                                     │
│       │               └── 𝒞_DE_Health ⊂ 𝒞_DE  ← Shard (healthcare)         │
│       │                                                                     │
│       └── 𝒞_ASEAN ⊂ 𝒞_Root                 ← Virt-Env (circle:asean)       │
│               │                                                             │
│               └── 𝒞_ASEAN_Trade ⊂ 𝒞_ASEAN   ← Shard (trade)                │
│                                                                             │
│   ═══════════════════════════════════════════════════════════════════════  │
│                                                                             │
│   AXIOM-MAPPING:                                                            │
│                                                                             │
│   A18 (Schachtelung):    (R ⊑ R') ∧ [R]φ → [R']φ                           │
│                          → Was im Sub-Realm gilt, gilt im Parent            │
│                                                                             │
│   A19 (Monotonie):       rules(Parent) ⊆ rules(Child)                       │
│                          → Child kann nur Regeln hinzufügen                 │
│                                                                             │
│   E4 (Shards):           Shards sind kognitive Subräume mit Spezialisierung │
│                                                                             │
│   Q6 (Kategorien):       Jedes Realm/Shard ist eine Kategorie               │
│                                                                             │
│   Q7 (Funktoren):        F: 𝒞₁ → 𝒞₂ erhält Struktur bei Cross-Transfers    │
│                                                                             │
│   Q8 (Morphismen):       Transaktionen sind Morphismen s₁ → s₂              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 2.2 Formale Definition

```rust
/// Eine Kategorie im Erynoa-System (Axiom Q6)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Category {
    /// Kategorie-ID (entspricht Realm/Shard/Virt-Env DID)
    pub id: DID,

    /// Typ der Kategorie
    pub category_type: CategoryType,

    /// Parent-Kategorie (None für Root)
    pub parent: Option<DID>,

    /// Objekte (Agenten in dieser Kategorie)
    pub objects: HashSet<DID>,

    /// Morphismen (Transaktionen zwischen Agenten)
    pub morphisms: HashMap<TransactionId, Morphism>,

    /// Identitäts-Morphismen (Axiom Q6: id ∘ f = f = f ∘ id)
    pub identity_morphisms: HashMap<DID, MorphismId>,

    /// Lokale Axiome (A19: Monotonie - nur Erweiterungen)
    pub local_axioms: Vec<LocalAxiom>,

    /// Funktoren zu anderen Kategorien
    pub functors: HashMap<DID, Functor>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CategoryType {
    /// Root-Environment (𝒞_Root)
    RootEnv,

    /// Virtual Environment (𝒞_VirtEnv ⊂ 𝒞_Parent)
    VirtEnv {
        governance: GovernanceConfig,
        cbdc_bridge: Option<CbdcBridge>,
    },

    /// Shard (𝒞_Shard ⊂ 𝒞_VirtEnv)
    Shard {
        shard_type: ShardType,
        preset: Option<ShardPreset>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ShardType {
    /// Finanz-Shard (CBDC, Trading, etc.)
    Finance {
        cbdc_token: Option<CbdcToken>,
        settlement_currency: String,
    },

    /// Energie-Shard (Energy Trading, Grid Management)
    Energy {
        grid_type: GridType,
        metering_protocol: String,
    },

    /// Healthcare-Shard (Medizinische Daten, HIPAA/GDPR)
    Healthcare {
        compliance_level: ComplianceLevel,
        data_classification: DataClassification,
    },

    /// Supply-Chain-Shard (Tracking, Provenance)
    SupplyChain {
        stages: Vec<String>,
        attestation_required: bool,
    },

    /// Gaming-Shard (Assets, Achievements)
    Gaming {
        asset_types: Vec<String>,
        interoperability: bool,
    },

    /// Custom-Shard (benutzerdefiniert)
    Custom {
        schema: BlueprintId,
    },
}
```

#### 2.2.1 Trust-Gewichtung nach Shard-Typ

Jeder Shard-Typ hat spezifische Gewichtungen für die 6D-Trust-Vektor-Komponenten:

| Shard-Typ       | R    | I    | C    | P    | V    | Ω    | Fokus                              |
| --------------- | ---- | ---- | ---- | ---- | ---- | ---- | ---------------------------------- |
| **Finance**     | 0.15 | 0.25 | 0.10 | 0.10 | 0.15 | 0.25 | Integrität & Compliance            |
| **Energy**      | 0.25 | 0.10 | 0.15 | 0.25 | 0.15 | 0.10 | Zuverlässigkeit & Vorhersagbarkeit |
| **Healthcare**  | 0.15 | 0.30 | 0.20 | 0.10 | 0.10 | 0.15 | Integrität & Kompetenz             |
| **Gaming**      | 0.10 | 0.10 | 0.35 | 0.15 | 0.15 | 0.15 | Kompetenz                          |
| **SupplyChain** | 0.20 | 0.20 | 0.15 | 0.20 | 0.15 | 0.10 | Zuverlässigkeit & Integrität       |

**Legende:** R = Reliability, I = Integrity, C = Competence, P = Predictability, V = Vigilance, Ω = Omega-Alignment

#### 2.3 Morphismen und Transaktionen

```rust
/// Ein Morphismus (Transaktion) zwischen Objekten (Agenten)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Morphism {
    /// Morphismus-ID (= Transaktions-ID)
    pub id: MorphismId,

    /// Quell-Objekt (Sender)
    pub source: DID,

    /// Ziel-Objekt (Empfänger)
    pub target: DID,

    /// Kategorie, in der dieser Morphismus existiert
    pub category: DID,

    /// Komposition mit anderen Morphismen möglich?
    /// (Axiom Q6: Kompositions-Gesetz f ∘ g ∘ h = (f ∘ g) ∘ h = f ∘ (g ∘ h))
    pub composable: bool,

    /// Event-Referenz im DAG
    pub event_id: EventId,
}

/// Komposition von Morphismen (Axiom Q6)
pub fn compose(f: &Morphism, g: &Morphism) -> Result<Morphism, CategoryError> {
    // Prüfe: target(g) = source(f)
    if g.target != f.source {
        return Err(CategoryError::CompositionMismatch);
    }

    // Prüfe: Gleiche Kategorie
    if g.category != f.category {
        // Cross-Shard → Funktor erforderlich
        return Err(CategoryError::CrossCategoryComposition);
    }

    Ok(Morphism {
        id: generate_composition_id(&g.id, &f.id),
        source: g.source.clone(),
        target: f.target.clone(),
        category: f.category.clone(),
        composable: true,
        event_id: create_composition_event(&g, &f)?,
    })
}

/// Identitäts-Morphismus (Axiom Q6: id ∘ f = f = f ∘ id)
pub fn identity(agent: &DID, category: &DID) -> Morphism {
    Morphism {
        id: format!("id_{}", agent).into(),
        source: agent.clone(),
        target: agent.clone(),
        category: category.clone(),
        composable: true,
        event_id: EventId::genesis(),
    }
}
```

#### 2.4 Funktoren für Cross-Shard/Cross-Env Operationen (V0.3 Refined)

```rust
/// Ein Funktor F: 𝒞₁ → 𝒞₂ (Axiom Q7) mit Trust-Matrix (V0.3)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Functor {
    /// Funktor-ID
    pub id: FunctorId,

    /// Quell-Kategorie
    pub source_category: DID,

    /// Ziel-Kategorie
    pub target_category: DID,

    /// Objekt-Abbildung: F(s) für jeden Agenten s
    pub object_mapping: ObjectMapping,

    /// Morphismus-Abbildung: F(f) für jede Transaktion f
    pub morphism_mapping: MorphismMapping,

    /// V0.3: Trust-Transformations-Matrix (6x6)
    /// Definiert, wie R, I, C, P, V, Ω transformiert werden
    /// Beispiel: Gaming→Finance: C wird stark gedämpft, I bleibt
    pub trust_matrix: TrustMatrix,

    /// V0.3: Dynamischer Dämpfungsfaktor basierend auf Erfolgsrate
    pub dynamic_dampening: DynamicDampening,

    /// Wert-Konversionsregel
    pub value_conversion: ValueConversion,

    /// V0.3: Boundary Guard (Logic Guard L1-L3)
    pub boundary_guard: BoundaryGuard,

    /// Funktor-Eigenschaften (Q7: Identität und Komposition erhalten)
    pub properties: FunctorProperties,
}

/// Trust-Transformations-Matrix (6x6) für kontextuelle Rotation
/// T_target = M × T_source
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustMatrix {
    /// Die 6x6 Matrix [R, I, C, P, V, Ω] → [R', I', C', P', V', Ω']
    pub matrix: [[f32; 6]; 6],
}

impl TrustMatrix {
    /// Identitäts-Matrix (Trust 1:1 übertragen)
    pub fn identity() -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0, 0.0, 0.0, 0.0], // R
                [0.0, 1.0, 0.0, 0.0, 0.0, 0.0], // I
                [0.0, 0.0, 1.0, 0.0, 0.0, 0.0], // C
                [0.0, 0.0, 0.0, 1.0, 0.0, 0.0], // P
                [0.0, 0.0, 0.0, 0.0, 1.0, 0.0], // V
                [0.0, 0.0, 0.0, 0.0, 0.0, 1.0], // Ω
            ],
        }
    }

    /// Gaming → Finance: Competence gedämpft, Integrity bleibt
    pub fn gaming_to_finance() -> Self {
        Self {
            matrix: [
                [0.5, 0.0, 0.0, 0.0, 0.0, 0.0], // R: 50%
                [0.0, 0.8, 0.0, 0.0, 0.0, 0.0], // I: 80% (Ehrlichkeit übertragbar)
                [0.0, 0.0, 0.1, 0.0, 0.0, 0.0], // C: 10% (Gaming-Skill ≠ Finance-Skill)
                [0.0, 0.0, 0.0, 0.6, 0.0, 0.0], // P: 60%
                [0.0, 0.0, 0.0, 0.0, 0.7, 0.0], // V: 70%
                [0.0, 0.0, 0.0, 0.0, 0.0, 0.9], // Ω: 90%
            ],
        }
    }

    /// Energy → Finance: Reliability bleibt, Competence neutral
    pub fn energy_to_finance() -> Self {
        Self {
            matrix: [
                [0.9, 0.0, 0.0, 0.0, 0.0, 0.0], // R: 90%
                [0.0, 0.8, 0.0, 0.0, 0.0, 0.0], // I: 80%
                [0.0, 0.0, 0.5, 0.0, 0.0, 0.0], // C: 50%
                [0.0, 0.0, 0.0, 0.8, 0.0, 0.0], // P: 80%
                [0.0, 0.0, 0.0, 0.0, 0.6, 0.0], // V: 60%
                [0.0, 0.0, 0.0, 0.0, 0.0, 0.9], // Ω: 90%
            ],
        }
    }

    /// Wendet die Matrix auf einen Trust-Vektor an
    pub fn transform(&self, vector: &TrustVector) -> TrustVector {
        let components = vector.to_array();
        let mut result = [0.0f32; 6];

        for i in 0..6 {
            for j in 0..6 {
                result[i] += self.matrix[i][j] * components[j];
            }
        }

        TrustVector::from_array(result).normalize()
    }
}

/// Dynamischer Dämpfungsfaktor basierend auf Erfolgsrate (Kybernetik E6)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DynamicDampening {
    /// Basis-Dämpfungsfaktor
    pub base_factor: f64,

    /// Decay-Rate für Failures (λ)
    pub failure_decay: f64,

    /// Aktuelle Statistik
    pub stats: DampeningStats,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DampeningStats {
    pub total_transfers: u64,
    pub successful_transfers: u64,
    pub failed_transfers: u64,
}

impl DynamicDampening {
    /// Berechnet aktuellen Dämpfungsfaktor
    /// β_dynamic = β_base × exp(-λ × failure_rate)
    pub fn current_factor(&self) -> f64 {
        let failure_rate = if self.stats.total_transfers > 0 {
            self.stats.failed_transfers as f64 / self.stats.total_transfers as f64
        } else {
            0.0
        };

        self.base_factor * (-self.failure_decay * failure_rate).exp()
    }

    /// Aktualisiert nach Transfer
    pub fn record_transfer(&mut self, success: bool) {
        self.stats.total_transfers += 1;
        if success {
            self.stats.successful_transfers += 1;
        } else {
            self.stats.failed_transfers += 1;
        }
    }
}

/// Boundary Guard (Logic Guard L1-L3) für Funktor-Übergänge
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BoundaryGuard {
    /// Guard-ID
    pub id: LogicGuardId,

    /// ECL-Code für Validierung
    pub ecl_code: String,

    /// Erforderliche Credentials
    pub required_credentials: Vec<CredentialType>,

    /// Minimaler Trust-Level
    pub min_trust: f64,

    /// Compliance-Anforderungen
    pub compliance_requirements: Vec<ComplianceRequirement>,
}

impl BoundaryGuard {
    /// EU → Drittstaaten: GDPR-Äquivalenz prüfen
    pub fn eu_external() -> Self {
        Self {
            id: "guard_eu_external".into(),
            ecl_code: r#"
                // GDPR-Äquivalenz prüfen
                require(target_env.has_compliance("GDPR-equivalent") ||
                        target_env.has_compliance("GDPR"))

                // Minimaler Trust
                require(source_trust.scalar() >= 0.6)

                // Sanktions-Check
                require(!sanctions_list.contains(user.did))

                // Data Classification
                require(data.classification != "RESTRICTED" ||
                        target_env.has_compliance("EU-adequacy"))
            "#.into(),
            required_credentials: vec![],
            min_trust: 0.6,
            compliance_requirements: vec![
                ComplianceRequirement::GdprEquivalent,
            ],
        }
    }

    /// Healthcare-Shard: Medizinische Lizenz erforderlich
    pub fn healthcare_entry() -> Self {
        Self {
            id: "guard_healthcare_entry".into(),
            ecl_code: r#"
                // Medizinische Lizenz prüfen
                require(user.has_credential("medical-license") ||
                        user.has_credential("healthcare-professional"))

                // HIPAA/GDPR-Compliance
                require(user.env.has_compliance("HIPAA") ||
                        user.env.has_compliance("GDPR"))
            "#.into(),
            required_credentials: vec![
                CredentialType::MedicalLicense,
            ],
            min_trust: 0.7,
            compliance_requirements: vec![
                ComplianceRequirement::Hipaa,
                ComplianceRequirement::Gdpr,
            ],
        }
    }

    /// Validiert einen Übergang
    pub async fn validate(
        &self,
        user: &DID,
        source_trust: &TrustVector,
        target_env: &VirtEnv,
        context: &GuardContext,
    ) -> Result<(), GuardError> {
        // 1. Trust-Level prüfen
        if source_trust.scalar() < self.min_trust {
            return Err(GuardError::InsufficientTrust {
                required: self.min_trust,
                actual: source_trust.scalar(),
            });
        }

        // 2. Credentials prüfen
        for cred_type in &self.required_credentials {
            if !context.has_credential(user, cred_type).await? {
                return Err(GuardError::MissingCredential(cred_type.clone()));
            }
        }

        // 3. Compliance prüfen
        for req in &self.compliance_requirements {
            if !target_env.has_compliance(req) {
                return Err(GuardError::ComplianceMismatch(req.clone()));
            }
        }

        // 4. ECL-Code ausführen
        let result = execute_ecl(&self.ecl_code, context).await?;
        if !result.success {
            return Err(GuardError::EclValidationFailed(result.error));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctorProperties {
    /// F(id_A) = id_F(A) (Identität erhalten)
    pub preserves_identity: bool,

    /// F(g ∘ f) = F(g) ∘ F(f) (Komposition erhalten)
    pub preserves_composition: bool,

    /// Ist dieser Funktor injektiv?
    pub injective: bool,

    /// Ist dieser Funktor surjektiv?
    pub surjective: bool,
}

/// Cross-Shard Transfer mittels Funktor (V0.3 mit Trust-Matrix & Boundary Guard)
pub async fn cross_shard_transfer(
    source_shard: &Category,
    target_shard: &Category,
    agent: &DID,
    asset: &Asset,
    functor: &mut Functor,
    context: &GuardContext,
) -> Result<CrossShardResult, ShardError> {
    // 0. V0.3: Boundary Guard validieren
    let source_trust = source_shard.get_trust(agent)?;
    let target_env = get_virt_env(&target_shard.id)?;

    functor.boundary_guard.validate(
        agent,
        &source_trust,
        &target_env,
        context,
    ).await.map_err(|e| ShardError::BoundaryGuardFailed(e))?;

    // 1. Validiere Funktor-Eigenschaften (Axiom Q7)
    if !functor.properties.preserves_identity {
        return Err(ShardError::FunctorViolation("Identity not preserved"));
    }
    if !functor.properties.preserves_composition {
        return Err(ShardError::FunctorViolation("Composition not preserved"));
    }

    // 2. Objekt-Abbildung: Agent in Ziel-Kategorie
    let target_agent = funktor.object_mapping.map(agent)?;

    // 3. Asset-Konversion
    let converted_asset = functor.value_conversion.convert(asset)?;

    // 4. V0.3: Trust-Transformation mittels Matrix
    let transformed_trust = functor.trust_matrix.transform(&source_trust);

    // 5. V0.3: Dynamische Dämpfung anwenden
    let dampening = functor.dynamic_dampening.current_factor();
    let propagated_trust = transformed_trust.scale(dampening);

    // 6. HTLC-basierter Atomic Swap (V0.3: Saga Pattern)
    let lock_result = atomic_cross_bridge_swap(
        &source_shard.id,
        &target_shard.id,
        agent,
        asset.amount,
        converted_asset.amount,
    ).await;

    match lock_result {
        Ok(swap_receipt) => {
            // Success: Statistik aktualisieren
            functor.dynamic_dampening.record_transfer(true);

            // Trust im Ziel-Shard setzen
            target_shard.update_trust(&target_agent, propagated_trust).await?;

            Ok(CrossShardResult {
                event_id: swap_receipt.event_id,
                source_agent: agent.clone(),
                target_agent,
                original_asset: asset.clone(),
                converted_asset,
                trust_propagated: propagated_trust,
                dampening_factor: dampening,
            })
        },
        Err(e) => {
            // Failure: Statistik aktualisieren (erhöht future dampening)
            functor.dynamic_dampening.record_transfer(false);
            Err(ShardError::TransferFailed(e))
        }
    }
}

/// HTLC-basierter Atomic Swap für Cross-Bridge Konsistenz (V0.3)
pub async fn atomic_cross_bridge_swap(
    source_bridge: &DID,
    target_bridge: &DID,
    user: &DID,
    amount_source: u128,
    amount_target: u128,
) -> Result<SwapReceipt, SwapError> {
    // 1. Phase: LOCK (Source Env)
    // Asset wird im Quell-Shard in HTLC gesperrt
    let lock_proof = lock_asset(source_bridge, user, amount_source).await?;

    // 2. Phase: MINT/UNLOCK (Target Env)
    // Mit Lock-Proof wird im Ziel-Shard das Gegen-Asset freigegeben
    let mint_result = mint_asset(
        target_bridge,
        user,
        amount_target,
        &lock_proof,
    ).await;

    match mint_result {
        Ok(receipt) => {
            // 3. Phase: COMMIT (Source Env)
            // Swap erfolgreich → Assets im Source Shard endgültig verbrannt
            commit_burn(source_bridge, &lock_proof.id).await?;
            Ok(receipt)
        },
        Err(e) => {
            // ROLLBACK (Source Env)
            // Swap fehlgeschlagen → Assets werden entsperrt
            rollback_lock(source_bridge, &lock_proof.id).await?;
            Err(e)
        }
    }
}

/// Legacy: Zwei-Phasen-Commit (für intra-Env Transfers)
pub async fn cross_shard_transfer_legacy(
    source_shard: &Category,
    target_shard: &Category,
    agent: &DID,
    asset: &Asset,
    functor: &Functor,
) -> Result<CrossShardResult, ShardError> {
    // 1. Validiere Funktor-Eigenschaften (Axiom Q7)
    if !functor.properties.preserves_identity {
        return Err(ShardError::FunctorViolation("Identity not preserved"));
    }

    // 2. Objekt-Abbildung
    let target_agent = functor.object_mapping.map(agent)?;

    // 3. Asset-Konversion
    let converted_asset = functor.value_conversion.convert(asset)?;

    // 4. Trust-Transformation
    let source_trust = source_shard.get_trust(agent)?;
    let propagated_trust = functor.trust_matrix.transform(&source_trust);

    // 5. Zwei-Phasen-Commit (Atomic Cross-Shard)
    let phase1_source = source_shard.prepare_debit(agent, asset).await?;
    let phase1_target = target_shard.prepare_credit(&target_agent, &converted_asset).await?;

    if !phase1_source.ready || !phase1_target.ready {
        rollback(&phase1_source, &phase1_target).await?;
        return Err(ShardError::PrepareFailed);
    }

    // Commit
    let commit_source = source_shard.commit_debit(&phase1_source).await?;
    let commit_target = target_shard.commit_credit(&phase1_target).await?;

    // 6. Trust im Ziel-Shard setzen
    target_shard.update_trust(&target_agent, propagated_trust).await?;

    // 7. Cross-Shard Event erstellen
    let event = Event {
        event_type: EventType::CrossShardTransfer,
        actor: agent.clone(),
        parents: vec![commit_source.event_id, commit_target.event_id],
        payload: EventPayload::CrossShardTransfer {
            source_shard: source_shard.id.clone(),
            target_shard: target_shard.id.clone(),
            functor_id: functor.id.clone(),
            asset: asset.clone(),
            converted_asset: converted_asset.clone(),
        },
        realm: Some(source_shard.id.clone()),
        ..Default::default()
    };

    Ok(CrossShardResult {
        event_id: submit_event(event).await?,
        source_agent: agent.clone(),
        target_agent,
        original_asset: asset.clone(),
        converted_asset,
        trust_propagated: propagated_trust,
    })
}
```

### 3. Unified Identity & Multi-Chain Onboarding (V0.3)

#### 3.1 Master-Secret und Key-Derivation

```rust
/// Master-Secret für deterministische Key-Ableitung
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub enum MasterSecret {
    /// BIP39 Mnemonic (24 Wörter)
    Mnemonic {
        entropy: [u8; 32],
        words: [String; 24],
    },

    /// WebAuthn Passkey
    Passkey {
        credential_id: Vec<u8>,
        public_key: Vec<u8>,
        attestation: PasskeyAttestation,
    },
}

/// HD-Derivation Pfade für verschiedene Chains
pub struct DerivationPaths {
    /// Erynoa Primary: m/44'/9999'/0'/0/0
    pub erynoa_primary: &'static str,

    /// Ethereum/EVM: m/44'/60'/0'/0/0 (secp256k1)
    pub ethereum: &'static str,

    /// Solana: m/44'/501'/0'/0' (Ed25519)
    pub solana: &'static str,

    /// IOTA/MoveVM: m/44'/4218'/0'/0/0 (Ed25519)
    pub iota: &'static str,
}

impl Default for DerivationPaths {
    fn default() -> Self {
        Self {
            erynoa_primary: "m/44'/9999'/0'/0/0",
            ethereum: "m/44'/60'/0'/0/0",
            solana: "m/44'/501'/0'/0'",
            iota: "m/44'/4218'/0'/0/0",
        }
    }
}

/// Erzeugt alle Keys aus einem Master-Secret
pub struct KeyDerivation {
    master_secret: MasterSecret,
    paths: DerivationPaths,
}

impl KeyDerivation {
    /// Leitet den primären Ed25519-Key für did:erynoa:self ab
    pub fn derive_erynoa_primary(&self) -> Ed25519Keypair {
        let seed = self.derive_seed(&self.paths.erynoa_primary);
        Ed25519Keypair::from_seed(&seed)
    }

    /// Leitet secp256k1 Key für EVM-Chains ab
    pub fn derive_ethereum(&self) -> Secp256k1Keypair {
        let seed = self.derive_seed(&self.paths.ethereum);
        Secp256k1Keypair::from_seed(&seed)
    }

    /// Leitet Ed25519 Key für Solana ab
    pub fn derive_solana(&self) -> Ed25519Keypair {
        let seed = self.derive_seed(&self.paths.solana);
        Ed25519Keypair::from_seed(&seed)
    }

    /// Leitet Ed25519 Key für IOTA/MoveVM ab
    pub fn derive_iota(&self) -> Ed25519Keypair {
        let seed = self.derive_seed(&self.paths.iota);
        Ed25519Keypair::from_seed(&seed)
    }

    /// Generiert DID aus primärem Key
    pub fn generate_did(&self) -> DID {
        let primary_key = self.derive_erynoa_primary();
        let public_key_hash = sha256(&primary_key.public_key())[..16];
        DID::from_str(&format!(
            "did:erynoa:self:{}",
            base58_encode(&public_key_hash)
        )).unwrap()
    }
}
```

#### 3.2 Multi-Chain Wallet Creation

```rust
/// Multi-Chain Wallet-Set, verknüpft mit einer Erynoa-DID
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultiChainWalletSet {
    /// Die primäre Erynoa DID
    pub erynoa_did: DID,

    /// Chain-spezifische Wallet-Adressen
    pub wallets: HashMap<ChainId, ChainWallet>,

    /// Aktueller Recovery-Status
    pub recovery_status: RecoveryStatus,

    /// Erstellungszeitpunkt
    pub created_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainWallet {
    /// Chain-Identifier
    pub chain: ChainId,

    /// Wallet-Adresse (native Format)
    pub address: String,

    /// Public Key (für Signatur-Verifikation)
    pub public_key: Vec<u8>,

    /// Key-Typ
    pub key_type: KeyType,

    /// Metadaten-Link zur Erynoa-DID
    pub did_link: DidLinkMethod,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChainId {
    ErynoaRoot,
    Ethereum { chain_id: u64 },
    Polygon,
    Arbitrum,
    Solana,
    Iota { network: String },
    Sui,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum KeyType {
    Ed25519,
    Secp256k1,
    Dilithium3,  // Post-Quantum
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DidLinkMethod {
    /// On-Chain Storage (z.B. Account-Metadaten)
    OnChainStorage { slot: u64 },

    /// ENS/Naming System
    NamingSystem { name: String, resolver: String },

    /// Signierte Attestation
    SignedAttestation { attestation: Vec<u8> },
}

/// Erstellt alle Wallets aus Master-Secret
pub async fn create_multi_chain_wallets(
    derivation: &KeyDerivation,
    chains: &[ChainConfig],
) -> Result<MultiChainWalletSet, WalletError> {
    let erynoa_did = derivation.generate_did();
    let mut wallets = HashMap::new();

    for chain in chains {
        let wallet = match chain.chain_id {
            ChainId::ErynoaRoot => {
                // Erynoa-internes Wallet (primärer Key)
                let keypair = derivation.derive_erynoa_primary();
                ChainWallet {
                    chain: ChainId::ErynoaRoot,
                    address: erynoa_did.to_string(),
                    public_key: keypair.public_key().to_vec(),
                    key_type: KeyType::Ed25519,
                    did_link: DidLinkMethod::OnChainStorage { slot: 0 },
                }
            },

            ChainId::Ethereum { chain_id } => {
                let keypair = derivation.derive_ethereum();
                let address = ethereum_address_from_pubkey(&keypair.public_key());
                ChainWallet {
                    chain: ChainId::Ethereum { chain_id },
                    address: format!("0x{}", hex::encode(&address)),
                    public_key: keypair.public_key().to_vec(),
                    key_type: KeyType::Secp256k1,
                    did_link: DidLinkMethod::OnChainStorage { slot: 0 },
                }
            },

            ChainId::Iota { ref network } => {
                let keypair = derivation.derive_iota();
                let address = iota_address_from_pubkey(&keypair.public_key());
                ChainWallet {
                    chain: ChainId::Iota { network: network.clone() },
                    address,
                    public_key: keypair.public_key().to_vec(),
                    key_type: KeyType::Ed25519,
                    did_link: DidLinkMethod::OnChainStorage { slot: 0 },
                }
            },

            // ... weitere Chains
            _ => continue,
        };

        wallets.insert(chain.chain_id.clone(), wallet);
    }

    Ok(MultiChainWalletSet {
        erynoa_did,
        wallets,
        recovery_status: RecoveryStatus::None,
        created_at: now_ms(),
    })
}
```

#### 3.3 DID-Document mit Multi-Chain Links

```json
{
  "id": "did:erynoa:self:alice-2026-xyz",
  "verificationMethod": [
    {
      "id": "did:erynoa:self:alice-2026-xyz#primary",
      "type": "Ed25519VerificationKey2020",
      "controller": "did:erynoa:self:alice-2026-xyz",
      "publicKeyMultibase": "z6Mkf5rGMoatrSj1f..."
    }
  ],
  "erynoa": {
    "namespace": "self",
    "multiChainWallets": [
      {
        "chain": "erynoa-root",
        "address": "did:erynoa:self:alice-2026-xyz",
        "keyType": "Ed25519",
        "derivationPath": "m/44'/9999'/0'/0/0"
      },
      {
        "chain": "ethereum-mainnet",
        "chainId": 1,
        "address": "0x1234...abcd",
        "keyType": "secp256k1",
        "derivationPath": "m/44'/60'/0'/0/0"
      },
      {
        "chain": "iota-mainnet",
        "address": "iota1qr...xyz",
        "keyType": "Ed25519",
        "derivationPath": "m/44'/4218'/0'/0/0"
      },
      {
        "chain": "solana-mainnet",
        "address": "ABC...XYZ",
        "keyType": "Ed25519",
        "derivationPath": "m/44'/501'/0'/0'"
      }
    ],
    "recovery": {
      "status": "none"
    }
  }
}
```

#### 3.4 Optional Recovery (Aktivierbar)

```rust
/// Recovery-Status (initial: None)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecoveryStatus {
    /// Keine Recovery konfiguriert (reine User-Kontrolle)
    None,

    /// Recovery konfiguriert
    Configured {
        method: RecoveryMethod,
        guardians: Vec<Guardian>,
        threshold: usize,
        timelock: Duration,
        activated_at: u64,
    },

    /// Recovery im Gange
    InProgress {
        initiated_by: DID,
        initiated_at: u64,
        confirmations: Vec<GuardianConfirmation>,
        timelock_expires: u64,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RecoveryMethod {
    /// Social Recovery (Freunde/Familie)
    Social,

    /// Staked Guardianship (Institutionen)
    Staked,

    /// Multi-Sig (technisch)
    MultiSig,
}

/// Aktiviert Recovery nachträglich
pub async fn activate_recovery(
    wallet_set: &mut MultiChainWalletSet,
    config: RecoveryConfig,
    user_signature: &Signature,
) -> Result<(), RecoveryError> {
    // Nur wenn aktuell keine Recovery konfiguriert
    if !matches!(wallet_set.recovery_status, RecoveryStatus::None) {
        return Err(RecoveryError::AlreadyConfigured);
    }

    // Validiere Guardians
    for guardian in &config.guardians {
        validate_guardian(guardian).await?;
    }

    // Update-Event erstellen
    let event = Event {
        event_type: EventType::RecoveryActivation,
        actor: wallet_set.erynoa_did.clone(),
        payload: EventPayload::RecoveryActivation {
            method: config.method.clone(),
            guardians: config.guardians.clone(),
            threshold: config.threshold,
            timelock: config.timelock,
        },
        signature: user_signature.clone(),
        ..Default::default()
    };

    submit_event(event).await?;

    wallet_set.recovery_status = RecoveryStatus::Configured {
        method: config.method,
        guardians: config.guardians,
        threshold: config.threshold,
        timelock: config.timelock,
        activated_at: now_ms(),
    };

    Ok(())
}

/// Recovery-Prozess durchführen
pub async fn execute_recovery(
    wallet_set: &MultiChainWalletSet,
    new_master_secret: &MasterSecret,
    guardian_confirmations: Vec<GuardianConfirmation>,
) -> Result<RecoveryResult, RecoveryError> {
    let config = match &wallet_set.recovery_status {
        RecoveryStatus::Configured { threshold, timelock, guardians, .. } => {
            (threshold, timelock, guardians)
        },
        _ => return Err(RecoveryError::NotConfigured),
    };

    // 1. Threshold prüfen
    let valid_confirmations = guardian_confirmations.iter()
        .filter(|c| verify_guardian_confirmation(c, config.2))
        .count();

    if valid_confirmations < *config.0 {
        return Err(RecoveryError::ThresholdNotMet);
    }

    // 2. Timelock starten
    let recovery_event = Event {
        event_type: EventType::RecoveryInitiated,
        actor: wallet_set.erynoa_did.clone(),
        payload: EventPayload::RecoveryInitiated {
            confirmations: guardian_confirmations.clone(),
            timelock_expires: now_ms() + config.1.as_millis() as u64,
        },
        ..Default::default()
    };

    submit_event(recovery_event).await?;

    // 3. Nach Timelock: Key-Rotation durchführen
    // (Async: Wird von separatem Job nach Ablauf ausgeführt)

    // 4. RightsTransfer-Event erstellen (nach Timelock)
    let new_derivation = KeyDerivation::from_secret(new_master_secret);
    let new_wallets = create_multi_chain_wallets(&new_derivation, &get_chain_configs()).await?;

    let transfer_event = Event {
        event_type: EventType::RightsTransfer,
        actor: wallet_set.erynoa_did.clone(),
        payload: EventPayload::RightsTransfer {
            old_did: wallet_set.erynoa_did.clone(),
            new_did: new_wallets.erynoa_did.clone(),
            old_wallets: wallet_set.wallets.values().cloned().collect(),
            new_wallets: new_wallets.wallets.values().cloned().collect(),
        },
        ..Default::default()
    };

    submit_event(transfer_event).await?;

    Ok(RecoveryResult {
        old_did: wallet_set.erynoa_did.clone(),
        new_did: new_wallets.erynoa_did,
        new_wallets,
        assets_to_transfer: collect_assets(&wallet_set.wallets).await?,
    })
}
```

### 4. Root-Environment (Root-Env) als 𝒞_Root

#### 4.1 Definition

Die **Root-Environment** ist die globale Kategorie 𝒞_Root – das unveränderliche Fundament von Erynoa. Sie enthält:

| Komponente    | Beschreibung                     | Änderbar?                   |
| ------------- | -------------------------------- | --------------------------- |
| Core Axioms   | Die 112 Axiome des Fachkonzepts  | Nein (nur durch H4-Prozess) |
| DID Standard  | EIP-001 Spezifikation            | Nein (Append-Only Updates)  |
| Trust Vector  | EIP-002 Spezifikation            | Nein                        |
| Event-DAG     | EIP-003 Spezifikation            | Nein                        |
| Bayesian Algo | EIP-004 Spezifikation            | Nein                        |
| ECLVM Core    | Deterministische VM für Policies | Nein                        |
| Genesis State | Initiale Konfiguration           | Nein                        |

#### 2.2 Root-Env DID

```json
{
  "id": "did:erynoa:circle:root",
  "controller": ["did:erynoa:circle:root"],
  "erynoa": {
    "type": "root-environment",
    "genesisBlock": "0x0000...genesis...",
    "protocolVersion": "1.0.0",
    "axiomsHash": "sha256:abc123...",
    "immutable": true,
    "childEnvs": [
      "did:erynoa:circle:eu-2026",
      "did:erynoa:circle:asean-2026",
      "did:erynoa:circle:china-2026"
    ]
  }
}
```

#### 2.3 Unveränderlichkeits-Garantie

```rust
/// Root-Env State ist immutable nach Genesis
pub struct RootEnvState {
    /// Genesis Block Hash (Identität der Root-Env)
    pub genesis_hash: [u8; 32],

    /// Axiom-Set (112 Axiome)
    pub axioms: AxiomSet,

    /// EIP Registry
    pub eips: HashMap<u8, EipSpec>,

    /// Diese Felder sind UNVERÄNDERBAR
    pub frozen: bool,
}

impl RootEnvState {
    /// Root-Env kann NICHT modifiziert werden
    pub fn update(&self, _update: RootUpdate) -> Result<(), RootEnvError> {
        Err(RootEnvError::Immutable)
    }

    /// Einzige Ausnahme: H4-Prozess (Human Override)
    pub fn h4_override(&mut self, override_req: H4Override, signatures: &[H4Signature]) -> Result<(), RootEnvError> {
        // Erfordert 2/3 Supermajority aller Virt-Env Governances
        // UND Beweis menschlicher Deliberation (physische Konferenz, etc.)
        if !verify_h4_threshold(signatures) {
            return Err(RootEnvError::InsufficientH4Consensus);
        }

        // Timelock: 1 Jahr Wartezeit
        if !verify_h4_timelock(&override_req) {
            return Err(RootEnvError::H4TimelockNotExpired);
        }

        // Anwendung des Override
        self.apply_h4(override_req)
    }
}
```

### 3. Virtual Environment (Virt-Env)

#### 3.1 Definition

Eine **Virtual Environment** ist eine souveräne Sub-Umgebung mit:

- **Eigener Governance**: DAO, Parlament, Zentralbank, etc.
- **Eigener Währung/CBDC**: Integration bestehender Finanzsysteme
- **Eigenen Regeln**: Zusätzliche Axiome, Policies, Compliance
- **Eigener Identitäts-Authority**: Wer darf DIDs in dieser Env erstellen?

#### 3.2 Virt-Env Struktur

```rust
/// Virtual Environment Konfiguration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtEnv {
    /// Eindeutige Identität (circle-Namespace)
    pub did: DID, // z.B. did:erynoa:circle:eu-2026

    /// Übergeordnete Umgebung (Root oder Parent-Virt-Env)
    pub parent_env: DID,

    /// Governance-Konfiguration
    pub governance: GovernanceConfig,

    /// CBDC/Währungs-Bridge
    pub currency_bridge: Option<CurrencyBridge>,

    /// Identitäts-Authorität
    pub identity_authority: IdentityAuthorityConfig,

    /// Lokale Axiom-Erweiterungen (dürfen Root nicht widersprechen)
    pub local_axioms: Vec<LocalAxiom>,

    /// Child-Envs (z.B. Bundesländer unter Deutschland)
    pub child_envs: Vec<DID>,

    /// Inter-Env Agreements (Kooperationsverträge)
    pub agreements: Vec<InterEnvAgreement>,

    /// Status
    pub status: VirtEnvStatus,

    /// Bootstrap-Zeitpunkt
    pub bootstrapped_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VirtEnvStatus {
    Bootstrapping,  // Initialisierung läuft
    Active,         // Voll operativ
    Suspended,      // Temporär pausiert
    Deprecated,     // Auslaufend
}
```

#### 3.3 Virt-Env DID Document

```json
{
  "id": "did:erynoa:circle:eu-2026",
  "controller": ["did:erynoa:guild:eu-commission"],
  "verificationMethod": [
    {
      "id": "did:erynoa:circle:eu-2026#gov-key-1",
      "type": "Ed25519VerificationKey2020",
      "controller": "did:erynoa:guild:eu-commission",
      "publicKeyMultibase": "z6Mkf5rGMo..."
    }
  ],
  "service": [
    {
      "id": "did:erynoa:circle:eu-2026#governance",
      "type": "ErynoaGovernance",
      "serviceEndpoint": "https://gov.erynoa.eu/api/v1"
    },
    {
      "id": "did:erynoa:circle:eu-2026#cbdc-bridge",
      "type": "CbdcBridge",
      "serviceEndpoint": "https://bridge.erynoa.eu/digital-euro"
    }
  ],
  "erynoa": {
    "type": "virtual-environment",
    "parentEnv": "did:erynoa:circle:root",
    "governance": {
      "type": "multi-sig-dao",
      "threshold": "2/3",
      "members": [
        "did:erynoa:guild:eu-commission",
        "did:erynoa:guild:ecb",
        "did:erynoa:guild:eu-parliament"
      ]
    },
    "currencyBridge": {
      "type": "cbdc",
      "currency": "EUR",
      "issuer": "did:erynoa:guild:ecb",
      "bridgeContract": "0xabc123..."
    },
    "identityAuthority": {
      "type": "federated",
      "trustedIssuers": [
        "did:erynoa:guild:eidas-authority",
        "did:erynoa:guild:de-bundesdruckerei",
        "did:erynoa:guild:fr-ants"
      ],
      "minKycLevel": 2
    },
    "localAxioms": [
      {
        "id": "EU-A1",
        "description": "GDPR Compliance Requirement",
        "eclCode": "require(data.retention_days <= 365)"
      },
      {
        "id": "EU-A2",
        "description": "Right to be Forgotten",
        "eclCode": "allow(user.request_deletion())"
      }
    ],
    "childEnvs": [
      "did:erynoa:circle:de-2026",
      "did:erynoa:circle:fr-2026",
      "did:erynoa:circle:it-2026"
    ],
    "bootstrappedAt": "2026-01-01T00:00:00Z",
    "status": "active"
  }
}
```

### 4. Bootstrapping-Prozess

#### 4.0 Bootstrapping-Modi

| Modus     | Dauer      | Anwendung                              | Anforderungen                        |
| --------- | ---------- | -------------------------------------- | ------------------------------------ |
| **Short** | 30–60 Tage | Persönliche/kleine Envs (Family-Realm) | Basis-Governance                     |
| **Long**  | 120 Tage   | Große Envs (EU, ASEAN)                 | CBDC-Integration, strenge Governance |

**Short-Modus:** Für persönliche oder kleine Gruppen-Umgebungen, die keine CBDC-Integration benötigen. Vereinfachte Governance (z.B. Multi-Sig mit 2-3 Personen).

**Long-Modus:** Für souveräne Entitäten (Staaten, Unionen) mit komplexer Governance, CBDC-Bridges und Identity-Authority-Integration.

#### 4.1 Phasen (Long-Modus)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      VIRT-ENV BOOTSTRAPPING (Long-Modus)                │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   PHASE 1: INTENTION (Tag 0)                                           │
│   ─────────────────────────────                                        │
│   • Initiator (z.B. EU-Kommission) signiert Intent                     │
│   • Parent-Env (Root) empfängt Request                                 │
│   • Prüfung: Hat Initiator Berechtigung? (Trust-Level?)                │
│                                                                         │
│   PHASE 2: GOVERNANCE SETUP (Tag 1-30)                                 │
│   ────────────────────────────────────                                 │
│   • Governance-Struktur definieren (DAO, Multi-Sig, etc.)              │
│   • Initiale Controller festlegen                                       │
│   • Voting-Regeln konfigurieren                                         │
│                                                                         │
│   PHASE 3: CBDC BRIDGE (Tag 30-60)                                     │
│   ────────────────────────────────                                      │
│   • Zentralbank-Integration konfigurieren                              │
│   • Bridge-Contract deployen                                            │
│   • Liquiditäts-Pools initialisieren                                   │
│   • Compliance-Layer aktivieren                                         │
│                                                                         │
│   PHASE 4: IDENTITY AUTHORITY (Tag 60-90)                              │
│   ──────────────────────────────────────                               │
│   • Trusted Issuers definieren (z.B. eIDAS-Stellen)                    │
│   • KYC-Level Mapping festlegen                                         │
│   • Staked Guardianship für institutionelle Bürgen                     │
│                                                                         │
│   PHASE 5: LOCAL AXIOMS (Tag 90-120)                                   │
│   ──────────────────────────────────                                   │
│   • Lokale Regeln in ECL kodieren                                       │
│   • Konsistenz-Check gegen Root-Axiome                                 │
│   • Deployment in ECLVM                                                 │
│                                                                         │
│   PHASE 6: ACTIVATION (Tag 120)                                        │
│   ─────────────────────────────                                        │
│   • Genesis-Event für Virt-Env                                          │
│   • Status: Active                                                      │
│   • Erste DIDs können erstellt werden                                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 4.2 Bootstrap-Implementierung

```rust
/// Bootstrapping-Prozess für neue Virt-Env
pub struct BootstrapProcess {
    /// Initiator (muss high-trust sein)
    pub initiator: DID,

    /// Ziel-Parent (meist Root)
    pub parent_env: DID,

    /// Konfiguration
    pub config: VirtEnvConfig,

    /// Aktuelle Phase
    pub phase: BootstrapPhase,

    /// Checkpoint-Signaturen
    pub checkpoints: Vec<BootstrapCheckpoint>,
}

#[derive(Clone, Debug)]
pub enum BootstrapPhase {
    Intention { intent_hash: [u8; 32] },
    GovernanceSetup { governance: GovernanceConfig },
    CbdcBridge { bridge: Option<CurrencyBridge> },
    IdentityAuthority { authority: IdentityAuthorityConfig },
    LocalAxioms { axioms: Vec<LocalAxiom>, consistency_proof: Vec<u8> },
    Activation { genesis_event: EventId },
}

impl BootstrapProcess {
    /// Starte Bootstrapping
    pub async fn initiate(
        initiator: DID,
        parent_env: DID,
        config: VirtEnvConfig,
        context: &EnvContext,
    ) -> Result<Self, BootstrapError> {
        // Prüfe Initiator-Trust
        let trust = context.get_trust(&initiator).await?;
        if trust.scalar() < BOOTSTRAP_TRUST_THRESHOLD {
            return Err(BootstrapError::InsufficientTrust);
        }

        // Prüfe, ob Parent-Env existiert
        if !context.env_exists(&parent_env).await? {
            return Err(BootstrapError::ParentNotFound);
        }

        // Erstelle Intent-Event
        let intent = BootstrapIntent {
            initiator: initiator.clone(),
            parent_env: parent_env.clone(),
            config: config.clone(),
            timestamp: now_ms(),
        };

        let intent_hash = hash_intent(&intent);

        Ok(Self {
            initiator,
            parent_env,
            config,
            phase: BootstrapPhase::Intention { intent_hash },
            checkpoints: vec![],
        })
    }

    /// Governance Phase
    pub async fn setup_governance(
        &mut self,
        governance: GovernanceConfig,
        signatures: Vec<Signature>,
    ) -> Result<(), BootstrapError> {
        // Validiere Governance-Struktur
        validate_governance(&governance)?;

        // Multi-Sig Threshold erreicht?
        if signatures.len() < governance.initial_threshold {
            return Err(BootstrapError::InsufficientSignatures);
        }

        self.phase = BootstrapPhase::GovernanceSetup { governance };
        self.add_checkpoint("governance_setup")?;

        Ok(())
    }

    /// CBDC Bridge Phase
    pub async fn setup_cbdc_bridge(
        &mut self,
        bridge_config: CurrencyBridgeConfig,
        central_bank_signature: Signature,
    ) -> Result<(), BootstrapError> {
        // Validiere Zentralbank-Signatur
        let central_bank_did = &bridge_config.issuer;
        verify_signature(central_bank_did, &bridge_config, &central_bank_signature)?;

        // Deploy Bridge Contract
        let bridge_contract = deploy_bridge_contract(&bridge_config).await?;

        let bridge = CurrencyBridge {
            currency: bridge_config.currency,
            issuer: central_bank_did.clone(),
            bridge_contract,
            exchange_rate_oracle: bridge_config.oracle,
            compliance_layer: bridge_config.compliance,
        };

        self.phase = BootstrapPhase::CbdcBridge { bridge: Some(bridge) };
        self.add_checkpoint("cbdc_bridge_setup")?;

        Ok(())
    }

    /// Local Axioms Phase
    pub async fn setup_local_axioms(
        &mut self,
        axioms: Vec<LocalAxiom>,
        context: &EnvContext,
    ) -> Result<(), BootstrapError> {
        // Konsistenz-Check gegen Root-Axiome
        let root_axioms = context.get_root_axioms().await?;
        let consistency_proof = prove_axiom_consistency(&root_axioms, &axioms)?;

        if consistency_proof.is_none() {
            return Err(BootstrapError::AxiomConflict);
        }

        self.phase = BootstrapPhase::LocalAxioms {
            axioms,
            consistency_proof: consistency_proof.unwrap()
        };
        self.add_checkpoint("local_axioms_setup")?;

        Ok(())
    }

    /// Activation Phase
    pub async fn activate(
        &mut self,
        context: &mut EnvContext,
    ) -> Result<VirtEnv, BootstrapError> {
        // Erstelle Genesis-Event
        let genesis_event = Event {
            event_type: EventType::VirtEnvGenesis,
            actor: self.initiator.clone(),
            payload: EventPayload::VirtEnvGenesis {
                env_did: self.config.did.clone(),
                parent_env: self.parent_env.clone(),
                governance: self.extract_governance()?,
                currency_bridge: self.extract_bridge()?,
                local_axioms: self.extract_axioms()?,
            },
            timestamp: now_ms(),
            signature: vec![], // Wird später signiert
            parents: vec![context.get_latest_root_event().await?],
        };

        let genesis_id = context.submit_event(genesis_event).await?;

        self.phase = BootstrapPhase::Activation { genesis_event: genesis_id };

        // Erstelle Virt-Env
        let virt_env = VirtEnv {
            did: self.config.did.clone(),
            parent_env: self.parent_env.clone(),
            governance: self.extract_governance()?,
            currency_bridge: self.extract_bridge()?,
            identity_authority: self.config.identity_authority.clone(),
            local_axioms: self.extract_axioms()?,
            child_envs: vec![],
            agreements: vec![],
            status: VirtEnvStatus::Active,
            bootstrapped_at: now_ms(),
        };

        // Registriere bei Parent
        context.register_child_env(&self.parent_env, &virt_env).await?;

        Ok(virt_env)
    }
}
```

### 5. CBDC Bridge

#### 5.1 Konzept

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         CBDC BRIDGE ARCHITEKTUR                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   EXTERNES SYSTEM                    ERYNOA VIRT-ENV                   │
│   (z.B. TARGET2)                     (z.B. EU)                          │
│                                                                         │
│   ┌───────────────┐                  ┌───────────────┐                 │
│   │ Zentralbank   │                  │ CBDC Bridge   │                 │
│   │ Ledger (EUR)  │◄────────────────►│ Contract      │                 │
│   └───────────────┘                  └───────┬───────┘                 │
│         │                                    │                          │
│         │ Mint/Burn                          │ Wrapped CBDC             │
│         ▼                                    ▼                          │
│   ┌───────────────┐                  ┌───────────────┐                 │
│   │ Reserve Pool  │                  │ wEUR Token    │                 │
│   │ (1:1 Backed)  │◄═══ Audit ══════►│ (on Erynoa)   │                 │
│   └───────────────┘                  └───────────────┘                 │
│                                                                         │
│   ════════════════════════════════════════════════════════════════     │
│                                                                         │
│   FLOW: User will 100 EUR ins System bringen                           │
│                                                                         │
│   1. User → Zentralbank: "Transferiere 100 EUR an Reserve"             │
│   2. Zentralbank → Bridge: "Mint Bestätigung (Proof)"                  │
│   3. Bridge → User: "100 wEUR gutgeschrieben"                          │
│                                                                         │
│   FLOW: User will 50 wEUR auszahlen                                    │
│                                                                         │
│   1. User → Bridge: "Burn 50 wEUR"                                     │
│   2. Bridge → Zentralbank: "Release 50 EUR aus Reserve"                │
│   3. Zentralbank → User: "50 EUR auf Konto"                            │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 5.2 Bridge-Implementierung

```rust
/// CBDC Bridge Konfiguration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CurrencyBridge {
    /// Währung (ISO 4217)
    pub currency: String, // "EUR", "CNY", "USD", etc.

    /// Issuer (Zentralbank DID)
    pub issuer: DID,

    /// Bridge Contract Address
    pub bridge_contract: ContractAddress,

    /// Exchange Rate Oracle (für Cross-CBDC)
    pub exchange_rate_oracle: Option<OracleConfig>,

    /// Compliance Layer (AML/KYC)
    pub compliance_layer: ComplianceConfig,
}

/// Bridge Contract (ECL)
pub struct BridgeContract {
    /// Wrapped Token Symbol
    pub token_symbol: String,  // "wEUR"

    /// Total Supply (muss = Reserve sein)
    pub total_supply: u128,

    /// Mint-Authority (nur Zentralbank)
    pub mint_authority: DID,

    /// User Balances
    pub balances: HashMap<DID, u128>,

    /// Pending Withdrawals
    pub pending_withdrawals: Vec<Withdrawal>,

    /// Audit Trail
    pub audit_log: Vec<BridgeEvent>,
}

impl BridgeContract {
    /// Mint (nur durch Zentralbank)
    pub fn mint(
        &mut self,
        caller: &DID,
        recipient: &DID,
        amount: u128,
        proof: MintProof,
    ) -> Result<(), BridgeError> {
        // Nur Mint-Authority darf minten
        if caller != &self.mint_authority {
            return Err(BridgeError::Unauthorized);
        }

        // Validiere Proof (Zentralbank-Signatur über Einzahlung)
        if !verify_mint_proof(&proof, &self.mint_authority) {
            return Err(BridgeError::InvalidProof);
        }

        // Compliance-Check
        if !self.compliance_check(recipient, amount)? {
            return Err(BridgeError::ComplianceFailed);
        }

        // Mint
        *self.balances.entry(recipient.clone()).or_insert(0) += amount;
        self.total_supply += amount;

        self.audit_log.push(BridgeEvent::Mint {
            recipient: recipient.clone(),
            amount,
            proof_hash: hash(&proof),
            timestamp: now_ms(),
        });

        Ok(())
    }

    /// Burn (User initiiert Auszahlung)
    pub fn burn(
        &mut self,
        caller: &DID,
        amount: u128,
        target_iban: String,
    ) -> Result<WithdrawalId, BridgeError> {
        // Balance prüfen
        let balance = self.balances.get(caller).copied().unwrap_or(0);
        if balance < amount {
            return Err(BridgeError::InsufficientBalance);
        }

        // Compliance-Check (AML)
        if !self.aml_check(caller, amount, &target_iban)? {
            return Err(BridgeError::AmlCheckFailed);
        }

        // Burn
        *self.balances.get_mut(caller).unwrap() -= amount;
        self.total_supply -= amount;

        // Pending Withdrawal erstellen
        let withdrawal = Withdrawal {
            id: generate_withdrawal_id(),
            user: caller.clone(),
            amount,
            target_iban,
            status: WithdrawalStatus::Pending,
            created_at: now_ms(),
        };

        self.pending_withdrawals.push(withdrawal.clone());

        self.audit_log.push(BridgeEvent::Burn {
            user: caller.clone(),
            amount,
            withdrawal_id: withdrawal.id,
            timestamp: now_ms(),
        });

        Ok(withdrawal.id)
    }

    /// Transfer (innerhalb Erynoa)
    pub fn transfer(
        &mut self,
        caller: &DID,
        recipient: &DID,
        amount: u128,
    ) -> Result<(), BridgeError> {
        let balance = self.balances.get(caller).copied().unwrap_or(0);
        if balance < amount {
            return Err(BridgeError::InsufficientBalance);
        }

        *self.balances.get_mut(caller).unwrap() -= amount;
        *self.balances.entry(recipient.clone()).or_insert(0) += amount;

        Ok(())
    }
}
```

#### 5.3 Dynamic Exchange Model

Für Cross-CBDC Transaktionen zwischen verschiedenen Virt-Envs:

```rust
/// Cross-CBDC Exchange
pub struct CrossCbdcExchange {
    /// Quell-Bridge (z.B. EU wEUR)
    pub source_bridge: DID,

    /// Ziel-Bridge (z.B. China wCNY)
    pub target_bridge: DID,

    /// Oracle für Exchange Rate
    pub rate_oracle: OracleConfig,

    /// Liquidity Providers
    pub liquidity_pools: HashMap<(String, String), LiquidityPool>,
}

impl CrossCbdcExchange {
    /// Exchange zwischen zwei CBDCs
    pub async fn exchange(
        &mut self,
        user: &DID,
        source_amount: u128,
        source_currency: &str,
        target_currency: &str,
        min_target_amount: u128,  // Slippage Protection
    ) -> Result<ExchangeResult, ExchangeError> {
        // 1. Aktuellen Exchange Rate holen
        let rate = self.rate_oracle.get_rate(source_currency, target_currency).await?;

        // 2. Ziel-Betrag berechnen
        let target_amount = (source_amount as f64 * rate) as u128;

        // 3. Slippage prüfen
        if target_amount < min_target_amount {
            return Err(ExchangeError::SlippageExceeded);
        }

        // 4. Liquidity prüfen
        let pool_key = (source_currency.to_string(), target_currency.to_string());
        let pool = self.liquidity_pools.get_mut(&pool_key)
            .ok_or(ExchangeError::NoLiquidity)?;

        if pool.target_reserve < target_amount {
            return Err(ExchangeError::InsufficientLiquidity);
        }

        // 5. Atomic Swap ausführen
        // Source Bridge: Burn source_amount wEUR
        // Target Bridge: Mint target_amount wCNY

        let swap_result = atomic_cross_bridge_swap(
            &self.source_bridge,
            &self.target_bridge,
            user,
            source_amount,
            target_amount,
        ).await?;

        Ok(ExchangeResult {
            source_amount,
            target_amount,
            rate,
            swap_id: swap_result.id,
        })
    }
}
```

### 6. Inter-Env Protocol (IEP)

#### 6.1 Konzept

Das **Inter-Env Protocol** ermöglicht Zusammenarbeit zwischen verschiedenen Virt-Envs.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      INTER-ENV PROTOCOL (IEP)                           │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   VIRT-ENV: EU                       VIRT-ENV: ASEAN                   │
│   ┌───────────────┐                  ┌───────────────┐                 │
│   │ Governance    │                  │ Governance    │                 │
│   │ (EU-DAO)      │                  │ (ASEAN-DAO)   │                 │
│   └───────┬───────┘                  └───────┬───────┘                 │
│           │                                  │                          │
│           └──────────┬───────────────────────┘                          │
│                      │                                                  │
│                      ▼                                                  │
│           ┌───────────────────────┐                                    │
│           │  INTER-ENV AGREEMENT  │                                    │
│           │  (Bilateral Treaty)   │                                    │
│           └───────────┬───────────┘                                    │
│                       │                                                 │
│   ┌───────────────────┼───────────────────┐                            │
│   │                   │                   │                            │
│   ▼                   ▼                   ▼                            │
│ ┌─────────┐    ┌─────────────┐    ┌───────────────┐                   │
│ │ Trust   │    │ Currency    │    │ Identity      │                   │
│ │ Bridge  │    │ Exchange    │    │ Recognition   │                   │
│ └─────────┘    └─────────────┘    └───────────────┘                   │
│                                                                         │
│ "EU vertraut       "wEUR ↔ wSGD"      "EU erkennt               │
│  ASEAN-DIDs"       "Cross-CBDC"        ASEAN-KYC an"                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 6.2 Agreement-Struktur

```rust
/// Inter-Env Agreement (Bilateraler Vertrag)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InterEnvAgreement {
    /// Agreement ID
    pub id: AgreementId,

    /// Parteien
    pub parties: [DID; 2],  // Beide Virt-Envs

    /// Typ des Agreements
    pub agreement_type: AgreementType,

    /// Bedingungen
    pub terms: AgreementTerms,

    /// Signaturen beider Governances
    pub signatures: [Signature; 2],

    /// Status
    pub status: AgreementStatus,

    /// Gültigkeit
    pub valid_from: u64,
    pub valid_until: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgreementType {
    /// Trust-Anerkennung (DIDs aus Env A sind in Env B trusted)
    TrustRecognition {
        /// Trust-Mapping (z.B. ASEAN-Verified → EU-Neutral)
        trust_mapping: HashMap<TrustLevel, TrustLevel>,
        /// Minimaler Trust für Anerkennung
        min_trust: f64,
    },

    /// Währungs-Exchange
    CurrencyExchange {
        /// Erlaubte Währungspaare
        pairs: Vec<(String, String)>,
        /// Exchange-Limits pro Tag
        daily_limit: u128,
        /// Fee-Struktur
        fees: FeeStructure,
    },

    /// Identity Recognition
    IdentityRecognition {
        /// Welche Credential-Typen werden anerkannt?
        recognized_credentials: Vec<String>,
        /// KYC-Level Mapping
        kyc_mapping: HashMap<u8, u8>,
    },

    /// Full Association (alle oben)
    FullAssociation,
}
```

#### 6.3 Cross-Env Trust

```rust
/// Trust-Berechnung für Cross-Env Interaktionen
pub fn calculate_cross_env_trust(
    source_did: &DID,
    source_env: &VirtEnv,
    target_env: &VirtEnv,
    agreement: Option<&InterEnvAgreement>,
) -> CrossEnvTrust {
    // Basis-Trust in Source-Env
    let source_trust = source_env.get_trust(source_did);

    match agreement {
        Some(agr) if matches!(agr.agreement_type, AgreementType::TrustRecognition { .. }) => {
            // Trust-Mapping anwenden
            let mapping = match &agr.agreement_type {
                AgreementType::TrustRecognition { trust_mapping, min_trust } => {
                    if source_trust.scalar() < *min_trust {
                        return CrossEnvTrust::NotRecognized;
                    }
                    trust_mapping
                },
                _ => unreachable!(),
            };

            let source_level = source_trust.to_level();
            let target_level = mapping.get(&source_level)
                .unwrap_or(&TrustLevel::Unknown);

            CrossEnvTrust::Recognized {
                source_env: source_env.did.clone(),
                source_trust: source_trust.scalar(),
                mapped_level: target_level.clone(),
                agreement_id: agr.id,
            }
        },

        Some(agr) if matches!(agr.agreement_type, AgreementType::FullAssociation) => {
            // Full Association: Trust wird 1:1 übernommen (mit Dämpfung)
            CrossEnvTrust::Recognized {
                source_env: source_env.did.clone(),
                source_trust: source_trust.scalar() * 0.8,  // 20% Dämpfung
                mapped_level: source_trust.to_level(),
                agreement_id: agr.id,
            }
        },

        _ => {
            // Kein Agreement: Trust wird nicht anerkannt
            CrossEnvTrust::NotRecognized
        }
    }
}
```

### 7. Hierarchie & Vererbung

#### 7.1 Axiom-Hierarchie

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      AXIOM HIERARCHIE                                   │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   EBENE 0: ROOT-AXIOME (112 Axiome)                                    │
│   ═══════════════════════════════════                                  │
│   • Unveränderlich                                                      │
│   • Gelten überall                                                      │
│   • Können NICHT durch Local Axioms überschrieben werden               │
│                                                                         │
│   EBENE 1: VIRT-ENV AXIOME (Additiv)                                   │
│   ═══════════════════════════════════                                  │
│   • Erweitern Root-Axiome                                               │
│   • Dürfen Root NICHT widersprechen                                    │
│   • Konsistenz-Check bei Bootstrapping                                 │
│   • Beispiel: EU-A1 (GDPR), EU-A2 (Right to Forget)                    │
│                                                                         │
│   EBENE 2: SUB-VIRT-ENV AXIOME (Additiv)                               │
│   ════════════════════════════════════════                             │
│   • Erweitern Parent + Root                                             │
│   • Dürfen weder Root noch Parent widersprechen                        │
│   • Beispiel: DE-A1 (Datenschutz), DE-A2 (TMG)                         │
│                                                                         │
│   KONFLIKT-AUFLÖSUNG:                                                  │
│   • Root > Virt-Env > Sub-Virt-Env                                     │
│   • Bei Widerspruch: Höhere Ebene gewinnt IMMER                        │
│   • Widersprüche werden bei Bootstrapping verhindert                   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

#### 7.2 Konsistenz-Prüfung

```rust
/// Prüft, ob lokale Axiome mit Parent/Root konsistent sind
pub fn prove_axiom_consistency(
    root_axioms: &AxiomSet,
    parent_axioms: &[LocalAxiom],
    new_axioms: &[LocalAxiom],
) -> Result<ConsistencyProof, ConsistencyError> {
    // 1. Prüfe gegen Root
    for axiom in new_axioms {
        if contradicts_root(root_axioms, axiom) {
            return Err(ConsistencyError::ContradictsRoot {
                local: axiom.id.clone(),
                root: find_conflicting_root(root_axioms, axiom)?,
            });
        }
    }

    // 2. Prüfe gegen Parent
    for axiom in new_axioms {
        if let Some(conflict) = find_contradiction(parent_axioms, axiom) {
            return Err(ConsistencyError::ContradictsParent {
                local: axiom.id.clone(),
                parent: conflict.id.clone(),
            });
        }
    }

    // 3. Prüfe interne Konsistenz
    for (i, a1) in new_axioms.iter().enumerate() {
        for a2 in new_axioms.iter().skip(i + 1) {
            if contradicts(a1, a2) {
                return Err(ConsistencyError::InternalContradiction {
                    axiom1: a1.id.clone(),
                    axiom2: a2.id.clone(),
                });
            }
        }
    }

    // 4. Generiere Proof (für Audit)
    Ok(ConsistencyProof {
        root_hash: root_axioms.hash(),
        parent_hash: hash_axioms(parent_axioms),
        new_hash: hash_axioms(new_axioms),
        verified_at: now_ms(),
        prover_signature: vec![],
    })
}
```

### 8. Shard-Typen und CBDC-Integration

#### 8.1 Shard als Sub-Kategorie

Ein **Shard** ist eine spezialisierte Sub-Kategorie 𝒞_Shard ⊂ 𝒞_VirtEnv mit eigenem Fokus:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      SHARD-HIERARCHIE                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   VIRT-ENV: EU (𝒞_EU)                                                      │
│       │                                                                     │
│       ├── SHARD: Finance (𝒞_EU_Finance)                                    │
│       │       │   └── Trust-Weights: I=0.25, Ω=0.20                        │
│       │       │   └── CBDC: wEUR (Digital Euro)                            │
│       │       │   └── Compliance: MiCA, PSD2                               │
│       │       │                                                             │
│       │       ├── SUB-SHARD: Banking                                       │
│       │       ├── SUB-SHARD: Trading                                       │
│       │       └── SUB-SHARD: Insurance                                     │
│       │                                                                     │
│       ├── SHARD: Energy (𝒞_EU_Energy)                                      │
│       │       │   └── Trust-Weights: R=0.25, P=0.20                        │
│       │       │   └── Settlement: wEUR                                      │
│       │       │   └── Compliance: EU Energy Directive                      │
│       │       │                                                             │
│       │       ├── SUB-SHARD: Solar                                         │
│       │       ├── SUB-SHARD: Wind                                          │
│       │       └── SUB-SHARD: Grid                                          │
│       │                                                                     │
│       └── SHARD: Mobility (𝒞_EU_Mobility)                                  │
│               │   └── Trust-Weights: R=0.30, V=0.15                        │
│               │   └── Settlement: wEUR                                      │
│               │                                                             │
│               ├── SUB-SHARD: EV-Charging                                   │
│               └── SUB-SHARD: Car-Sharing                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### 8.2 Shard-Definition

```rust
/// Shard innerhalb einer Virt-Env
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shard {
    /// Shard-DID (im circle-Namespace)
    pub id: DID,

    /// Parent (Virt-Env oder übergeordneter Shard)
    pub parent: DID,

    /// Kategorie-Struktur
    pub category: Category,

    /// Shard-Typ
    pub shard_type: ShardType,

    /// Kontextuelle Trust-Gewichte (Axiom Q4)
    pub trust_weights: TrustWeights,

    /// Settlement-Währung (CBDC-Token oder Standard)
    pub settlement: SettlementConfig,

    /// Compliance-Anforderungen
    pub compliance: Vec<ComplianceRequirement>,

    /// Lokale Axiome (Erweiterung von Parent)
    pub local_axioms: Vec<LocalAxiom>,

    /// Funktoren zu anderen Shards
    pub functors: HashMap<DID, Functor>,

    /// Objekte (Agenten in diesem Shard)
    pub members: HashSet<DID>,
}

/// Kontextuelle Trust-Gewichte (pro Shard unterschiedlich)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustWeights {
    pub r: f64,     // Reliability
    pub i: f64,     // Integrity
    pub c: f64,     // Competence
    pub p: f64,     // Predictability
    pub v: f64,     // Vigilance
    pub omega: f64, // Omega-Alignment
}

impl Default for TrustWeights {
    fn default() -> Self {
        // Globale Default-Gewichte
        Self { r: 0.15, i: 0.15, c: 0.15, p: 0.10, v: 0.20, omega: 0.25 }
    }
}

impl TrustWeights {
    /// Finanz-Shard: Integrität und Compliance wichtiger
    pub fn finance() -> Self {
        Self { r: 0.15, i: 0.25, c: 0.10, p: 0.15, v: 0.10, omega: 0.25 }
    }

    /// Energie-Shard: Zuverlässigkeit und Vorhersagbarkeit wichtiger
    pub fn energy() -> Self {
        Self { r: 0.25, i: 0.15, c: 0.10, p: 0.20, v: 0.15, omega: 0.15 }
    }

    /// Healthcare-Shard: Integrität kritisch
    pub fn healthcare() -> Self {
        Self { r: 0.10, i: 0.30, c: 0.15, p: 0.10, v: 0.15, omega: 0.20 }
    }

    /// Gaming-Shard: Kompetenz wichtiger
    pub fn gaming() -> Self {
        Self { r: 0.10, i: 0.10, c: 0.30, p: 0.10, v: 0.25, omega: 0.15 }
    }
}
```

#### 8.3 CBDC-Shard (Finance-Spezialisierung)

Ein **CBDC-Shard** ist ein spezialisierter Finance-Shard mit Bridge zu einer Zentralbank:

```rust
/// CBDC-Shard Konfiguration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CbdcShard {
    /// Basis-Shard
    pub shard: Shard,

    /// CBDC-Token Konfiguration
    pub token: CbdcToken,

    /// Bridge zu externer Zentralbank
    pub bridge: CbdcBridge,

    /// Mint-Authority (nur Zentralbank)
    pub mint_authority: DID,

    /// Compliance-Layer (AML/KYC)
    pub compliance: CbdcCompliance,
}

/// CBDC-Token (Wrapped Central Bank Digital Currency)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CbdcToken {
    /// Token-Symbol (z.B. "wEUR", "wCNY")
    pub symbol: String,

    /// ISO 4217 Währungscode
    pub currency_code: String,

    /// Dezimalstellen
    pub decimals: u8,

    /// Total Supply (muss = Reserve bei Zentralbank sein)
    pub total_supply: u128,

    /// Ist dieses Token 1:1 backed?
    pub fully_backed: bool,

    /// Audit-Zyklus (wie oft wird Reserve geprüft?)
    pub audit_interval: Duration,
}

/// CBDC-Bridge Mechanik
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CbdcBridge {
    /// Bridge-Contract Adresse
    pub contract: ContractAddress,

    /// Zentralbank-DID
    pub central_bank: DID,

    /// Reserve-Proof Methode
    pub reserve_proof: ReserveProofMethod,

    /// Exchange Rate Oracle (für Cross-CBDC)
    pub oracle: Option<OracleConfig>,

    /// Compliance-Level
    pub compliance_level: ComplianceLevel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReserveProofMethod {
    /// Merkle-Proof über Reserve-Bestand
    MerkleProof {
        root_url: String,
        update_frequency: Duration,
    },

    /// Attestation durch vertrauenswürdige Wirtschaftsprüfer
    AuditorAttestation {
        auditors: Vec<DID>,
        min_auditors: usize,
    },

    /// On-Chain Reserve (z.B. bei Stablecoin)
    OnChainReserve {
        contract: ContractAddress,
    },
}
```

#### 8.4 Shard-Bootstrapping

```rust
/// Shard innerhalb einer Virt-Env bootstrappen
pub async fn bootstrap_shard(
    virt_env: &VirtEnv,
    config: ShardConfig,
    governance_signatures: &[Signature],
) -> Result<Shard, ShardError> {
    // 1. Validiere, dass Virt-Env existiert und aktiv ist
    if virt_env.status != VirtEnvStatus::Active {
        return Err(ShardError::ParentEnvNotActive);
    }

    // 2. Validiere Governance-Signaturen
    if !virt_env.governance.verify_threshold(governance_signatures) {
        return Err(ShardError::InsufficientGovernance);
    }

    // 3. Generiere Shard-Kategorie (Axiom Q6)
    let category = Category {
        id: config.did.clone(),
        category_type: CategoryType::Shard {
            shard_type: config.shard_type.clone(),
            preset: config.preset.clone(),
        },
        parent: Some(virt_env.did.clone()),
        objects: HashSet::new(),
        morphisms: HashMap::new(),
        identity_morphisms: HashMap::new(),
        local_axioms: config.local_axioms.clone(),
        functors: HashMap::new(),
    };

    // 4. Validiere Axiom-Konsistenz (A19: Monotonie)
    prove_axiom_consistency(
        &virt_env.local_axioms,
        &config.local_axioms,
    )?;

    // 5. Generiere Inklusions-Funktor (𝒞_Shard → 𝒞_VirtEnv)
    let inclusion_functor = Functor {
        id: format!("incl_{}_{}", config.did, virt_env.did).into(),
        source_category: config.did.clone(),
        target_category: virt_env.did.clone(),
        object_mapping: ObjectMapping::Identity,
        morphism_mapping: MorphismMapping::Identity,
        trust_factor: 1.0,  // Trust wird 1:1 nach oben propagiert
        value_conversion: ValueConversion::Identity,
        properties: FunctorProperties {
            preserves_identity: true,
            preserves_composition: true,
            injective: true,
            surjective: false,
        },
    };

    // 6. Settlement-Währung konfigurieren
    let settlement = match &config.shard_type {
        ShardType::Finance { cbdc_token, .. } => {
            if let Some(token) = cbdc_token {
                SettlementConfig::Cbdc(token.clone())
            } else {
                SettlementConfig::ParentCurrency(virt_env.currency_bridge.clone())
            }
        },
        _ => SettlementConfig::ParentCurrency(virt_env.currency_bridge.clone()),
    };

    // 7. Erstelle Shard
    let shard = Shard {
        id: config.did,
        parent: virt_env.did.clone(),
        category,
        shard_type: config.shard_type,
        trust_weights: config.trust_weights.unwrap_or_else(||
            TrustWeights::for_shard_type(&config.shard_type)
        ),
        settlement,
        compliance: config.compliance,
        local_axioms: config.local_axioms,
        functors: hashmap! { virt_env.did.clone() => inclusion_functor },
        members: HashSet::new(),
    };

    // 8. Genesis-Event
    let event = Event {
        event_type: EventType::ShardGenesis,
        actor: virt_env.governance.primary_controller(),
        parents: vec![virt_env.latest_event()],
        payload: EventPayload::ShardGenesis {
            shard_id: shard.id.clone(),
            parent_env: virt_env.did.clone(),
            shard_type: shard.shard_type.clone(),
        },
        realm: Some(virt_env.did.clone()),
        ..Default::default()
    };

    submit_event(event).await?;

    Ok(shard)
}
```

#### 8.5 Cross-Shard Funktoren

```rust
/// Definiert einen Funktor zwischen zwei Shards
pub async fn create_cross_shard_functor(
    source_shard: &Shard,
    target_shard: &Shard,
    config: FunctorConfig,
    governance_signatures: &[Signature],
) -> Result<Functor, FunctorError> {
    // 1. Validiere, dass beide Shards in derselben Virt-Env sind
    //    ODER ein Inter-Env Agreement existiert
    let same_env = source_shard.parent == target_shard.parent;

    if !same_env {
        let agreement = find_inter_env_agreement(
            &source_shard.parent,
            &target_shard.parent,
        ).await?;

        if agreement.is_none() {
            return Err(FunctorError::NoInterEnvAgreement);
        }
    }

    // 2. Generiere Objekt-Mapping
    let object_mapping = match &config.object_mapping {
        ObjectMappingConfig::Identity => ObjectMapping::Identity,
        ObjectMappingConfig::Project(fields) => ObjectMapping::Projection(fields.clone()),
        ObjectMappingConfig::Custom(fn_id) => ObjectMapping::Custom(fn_id.clone()),
    };

    // 3. Generiere Wert-Konversion (für CBDC-Shards)
    let value_conversion = if let (
        SettlementConfig::Cbdc(source_token),
        SettlementConfig::Cbdc(target_token),
    ) = (&source_shard.settlement, &target_shard.settlement) {
        // Cross-CBDC: Exchange Rate nötig
        let oracle = config.exchange_oracle.ok_or(FunctorError::OracleRequired)?;
        ValueConversion::ExchangeRate {
            source_currency: source_token.currency_code.clone(),
            target_currency: target_token.currency_code.clone(),
            oracle,
        }
    } else {
        ValueConversion::Identity
    };

    // 4. Trust-Faktor berechnen
    let trust_factor = if same_env {
        config.trust_factor.unwrap_or(0.9)  // 10% Verlust bei Cross-Shard
    } else {
        config.trust_factor.unwrap_or(0.7)  // 30% Verlust bei Cross-Env
    };

    // 5. Erstelle Funktor
    let functor = Functor {
        id: generate_functor_id(&source_shard.id, &target_shard.id),
        source_category: source_shard.id.clone(),
        target_category: target_shard.id.clone(),
        object_mapping,
        morphism_mapping: MorphismMapping::Preserve,
        trust_factor,
        value_conversion,
        properties: FunctorProperties {
            preserves_identity: true,
            preserves_composition: true,
            injective: config.injective.unwrap_or(false),
            surjective: config.surjective.unwrap_or(false),
        },
    };

    // 6. Registriere Funktor in beiden Shards
    register_functor(&source_shard.id, &target_shard.id, &functor).await?;

    Ok(functor)
}
```

#### 8.6 Shard-DID-Dokument

```json
{
  "id": "did:erynoa:circle:eu-finance-2026",
  "controller": ["did:erynoa:guild:ecb", "did:erynoa:guild:eu-commission"],
  "service": [
    {
      "id": "did:erynoa:circle:eu-finance-2026#cbdc",
      "type": "CbdcBridge",
      "serviceEndpoint": "https://bridge.ecb.europa.eu/weur"
    }
  ],
  "erynoa": {
    "type": "shard",
    "shardType": "finance",
    "parent": "did:erynoa:circle:eu-2026",
    "category": {
      "objects": 150000,
      "morphisms": 12000000,
      "functors": [
        "did:erynoa:circle:eu-energy-2026",
        "did:erynoa:circle:asean-trade-2026"
      ]
    },
    "trustWeights": {
      "R": 0.15,
      "I": 0.25,
      "C": 0.1,
      "P": 0.15,
      "V": 0.1,
      "Ω": 0.25
    },
    "settlement": {
      "type": "cbdc",
      "token": {
        "symbol": "wEUR",
        "currency": "EUR",
        "decimals": 18,
        "totalSupply": "1000000000000000000000000000"
      },
      "bridge": {
        "centralBank": "did:erynoa:guild:ecb",
        "reserveProof": "merkle",
        "auditInterval": "24h"
      }
    },
    "compliance": [
      { "type": "MiCA", "version": "1.0" },
      { "type": "PSD2", "version": "2.0" },
      { "type": "GDPR", "version": "2016/679" }
    ],
    "localAxioms": [
      {
        "id": "FIN-A1",
        "description": "Mindest-KYC für Transaktionen > 1000 EUR",
        "ecl": "require(tx.value <= 1000 || tx.sender.kyc_level >= 2)"
      },
      {
        "id": "FIN-A2",
        "description": "AML-Check für Transaktionen > 10000 EUR",
        "ecl": "require(tx.value <= 10000 || aml_check(tx.sender, tx.recipient))"
      }
    ],
    "bootstrappedAt": "2026-03-15T00:00:00Z",
    "status": "active"
  }
}
```

### 9. CLI-Nutzung

```bash
# === V0.3: UNIFIED ONBOARDING ===

# Neue Identität mit Mnemonic erstellen
erynoa identity create \
  --method mnemonic \
  --words 24

# Output:
# Your Mnemonic (SAVE SECURELY):
# abandon ability able about above absent absorb abstract absurd abuse access accident
# ...
#
# Created:
# - Erynoa DID: did:erynoa:self:abc123xyz
# - Ethereum:   0x1234...abcd
# - IOTA:       iota1qr...xyz
# - Solana:     ABC...XYZ
#
# Recovery Status: NONE (aktivierbar mit 'erynoa recovery setup')

# Identität mit Passkey erstellen (WebAuthn)
erynoa identity create \
  --method passkey \
  --device "YubiKey 5"

# Multi-Chain Wallets anzeigen
erynoa identity wallets

# Recovery nachträglich aktivieren
erynoa recovery setup \
  --method social-staked \
  --threshold 3 \
  --guardian did:erynoa:guild:sparkasse-berlin \
  --guardian did:erynoa:self:bob-friend \
  --guardian did:erynoa:self:carol-friend \
  --guardian did:erynoa:guild:notar-office-muc \
  --guardian did:erynoa:self:dave-family \
  --timelock 7d

# Recovery initiieren (bei Verlust)
erynoa recovery initiate \
  --old-did did:erynoa:self:alice-2026 \
  --new-mnemonic \
  --guardian-confirmations ./confirmations.json

# === V0.3: TRUST MATRIX CONFIGURATION ===

# Trust-Matrix für Funktor konfigurieren
erynoa functor configure-matrix \
  --functor f_gaming_finance \
  --map "R -> R * 0.5" \
  --map "I -> I * 0.8" \
  --map "C -> C * 0.1" \
  --map "P -> P * 0.6" \
  --map "V -> V * 0.7" \
  --map "Omega -> Omega * 0.9"

# Vordefinierte Matrix verwenden
erynoa functor configure-matrix \
  --functor f_energy_finance \
  --preset energy-to-finance

# Aktuelle Matrix anzeigen
erynoa functor show-matrix f_gaming_finance

# === V0.3: BOUNDARY GUARDS ===

# Boundary Guard deployen
erynoa guard deploy \
  --env did:erynoa:circle:de-health \
  --type boundary \
  --ecl-file ./guards/healthcare-entry.ecl

# Guard-ECL Beispiel (healthcare-entry.ecl):
# guard healthcare_entry {
#   require(user.has_credential("medical-license"))
#   require(user.env.has_compliance("HIPAA") || user.env.has_compliance("GDPR"))
#   require(source_trust.scalar() >= 0.7)
# }

# Guard testen (ohne echten Transfer)
erynoa guard test \
  --guard guard_healthcare_entry \
  --user did:erynoa:self:alice \
  --source-env did:erynoa:circle:eu-2026 \
  --target-env did:erynoa:circle:de-health

# Guards auflisten
erynoa guard list --env did:erynoa:circle:de-health

# === V0.3: DYNAMIC DAMPENING ===

# Dämpfungs-Statistik anzeigen
erynoa functor dampening-stats f_eu_asean

# Output:
# Functor: f_eu_asean
# Base Factor: 0.9
# Current Factor: 0.847 (dynamic)
# Statistics:
#   Total Transfers: 15,234
#   Successful: 14,892 (97.8%)
#   Failed: 342 (2.2%)
# Failure Decay (λ): 0.5

# Dämpfung manuell anpassen
erynoa functor set-dampening \
  --functor f_eu_asean \
  --base 0.85 \
  --decay 0.3

# === VIRT-ENV MANAGEMENT ===

# Neue Virt-Env bootstrappen (als EU-Kommission)
erynoa env bootstrap \
  --name "European Union" \
  --did "did:erynoa:circle:eu-2026" \
  --parent "did:erynoa:circle:root" \
  --governance-type "multi-sig-dao" \
  --governance-threshold "2/3" \
  --governance-members "did:erynoa:guild:eu-commission,did:erynoa:guild:ecb"

# Bootstrap-Status prüfen
erynoa env bootstrap-status did:erynoa:circle:eu-2026

# CBDC Bridge konfigurieren
erynoa env cbdc-bridge setup \
  --env did:erynoa:circle:eu-2026 \
  --currency EUR \
  --issuer did:erynoa:guild:ecb \
  --oracle "https://ecb.europa.eu/rates/api"

# Local Axioms hinzufügen
erynoa env axiom add \
  --env did:erynoa:circle:eu-2026 \
  --id "EU-A1" \
  --description "GDPR Compliance" \
  --ecl "require(data.retention_days <= 365)"

# Sub-Env erstellen (Deutschland unter EU)
erynoa env bootstrap \
  --name "Germany" \
  --did "did:erynoa:circle:de-2026" \
  --parent "did:erynoa:circle:eu-2026"

# === INTER-ENV AGREEMENTS ===

# Agreement zwischen EU und ASEAN erstellen
erynoa env agreement create \
  --party1 "did:erynoa:circle:eu-2026" \
  --party2 "did:erynoa:circle:asean-2026" \
  --type "trust-recognition" \
  --trust-mapping "Verified→Neutral,HighTrust→Verified"

# Currency Exchange Agreement
erynoa env agreement create \
  --party1 "did:erynoa:circle:eu-2026" \
  --party2 "did:erynoa:circle:china-2026" \
  --type "currency-exchange" \
  --pairs "EUR/CNY" \
  --daily-limit 1000000

# === CBDC OPERATIONS ===

# CBDC minten (als Zentralbank)
erynoa cbdc mint \
  --bridge did:erynoa:circle:eu-2026 \
  --recipient did:erynoa:self:alice \
  --amount 1000 \
  --proof-file ./bank-receipt.json

# CBDC auszahlen
erynoa cbdc withdraw \
  --bridge did:erynoa:circle:eu-2026 \
  --amount 500 \
  --iban DE89370400440532013000

# Cross-CBDC Exchange
erynoa cbdc exchange \
  --from-env did:erynoa:circle:eu-2026 \
  --to-env did:erynoa:circle:asean-2026 \
  --amount 1000 \
  --from-currency EUR \
  --to-currency SGD \
  --max-slippage 0.02

# === SHARD MANAGEMENT ===

# Shard innerhalb einer Virt-Env erstellen
erynoa shard create \
  --name "EU Finance" \
  --did "did:erynoa:circle:eu-finance-2026" \
  --parent "did:erynoa:circle:eu-2026" \
  --type finance \
  --cbdc-token wEUR \
  --compliance MiCA,PSD2

# Shard mit Preset erstellen
erynoa shard create \
  --name "EU Energy Trading" \
  --parent "did:erynoa:circle:eu-2026" \
  --preset energy \
  --settlement wEUR

# Trust-Gewichte für Shard konfigurieren
erynoa shard weights set did:erynoa:circle:eu-finance-2026 \
  --R 0.15 --I 0.25 --C 0.10 --P 0.15 --V 0.10 --Omega 0.25

# Shard-Details anzeigen
erynoa shard show did:erynoa:circle:eu-finance-2026

# Shards einer Virt-Env auflisten
erynoa shard list --env did:erynoa:circle:eu-2026

# === FUNKTOR MANAGEMENT ===

# Funktor zwischen Shards erstellen (Cross-Shard Transfer)
erynoa functor create \
  --source did:erynoa:circle:eu-finance-2026 \
  --target did:erynoa:circle:eu-energy-2026 \
  --trust-factor 0.9

# Cross-CBDC Funktor erstellen
erynoa functor create \
  --source did:erynoa:circle:eu-finance-2026 \
  --target did:erynoa:circle:asean-trade-2026 \
  --trust-factor 0.7 \
  --exchange-oracle "https://ecb.europa.eu/rates" \
  --value-conversion "EUR→SGD"

# Funktor-Details anzeigen
erynoa functor show f_eu-finance_eu-energy

# Cross-Shard Transfer ausführen
erynoa shard transfer \
  --from did:erynoa:circle:eu-finance-2026 \
  --to did:erynoa:circle:eu-energy-2026 \
  --asset 1000wEUR \
  --functor auto

# === KATEGORIE-OPERATIONEN (für Experten) ===

# Kategorie-Struktur anzeigen
erynoa category show did:erynoa:circle:eu-finance-2026

# Morphismen auflisten
erynoa category morphisms did:erynoa:circle:eu-finance-2026 --limit 100

# Objekte (Agenten) im Shard auflisten
erynoa category objects did:erynoa:circle:eu-finance-2026 --limit 100

# Identitäts-Morphismus prüfen
erynoa category verify-identity did:erynoa:self:alice did:erynoa:circle:eu-finance-2026

# Komposition prüfen (f ∘ g)
erynoa category verify-composition --morphism1 tx_123 --morphism2 tx_456

# === MONITORING ===

# Alle Virt-Envs anzeigen
erynoa env list

# Env-Details anzeigen (inkl. Shards)
erynoa env show did:erynoa:circle:eu-2026 --include-shards

# Shard-Hierarchie visualisieren
erynoa env tree did:erynoa:circle:eu-2026

# Axiom-Hierarchie anzeigen
erynoa env axioms did:erynoa:circle:de-2026 --include-inherited

# Cross-Env Trust prüfen
erynoa trust cross-env \
  --did did:erynoa:self:alice \
  --source-env did:erynoa:circle:eu-2026 \
  --target-env did:erynoa:circle:asean-2026

# Cross-Shard Trust prüfen
erynoa trust cross-shard \
  --did did:erynoa:self:alice \
  --source-shard did:erynoa:circle:eu-finance-2026 \
  --target-shard did:erynoa:circle:eu-energy-2026
```

### 10. SDK-Nutzung

#### 9.1 Rust

```rust
use erynoa_sdk::{VirtEnv, BootstrapProcess, CurrencyBridge, InterEnvAgreement};

// Virt-Env bootstrappen
let bootstrap = BootstrapProcess::initiate(
    eu_commission_did,
    root_env_did,
    VirtEnvConfig {
        name: "European Union".into(),
        did: "did:erynoa:circle:eu-2026".parse()?,
        governance: GovernanceConfig::MultiSigDao {
            threshold: (2, 3),
            members: vec![eu_commission, ecb, eu_parliament],
        },
        ..Default::default()
    },
    &context,
).await?;

// CBDC Bridge aufsetzen
bootstrap.setup_cbdc_bridge(
    CurrencyBridgeConfig {
        currency: "EUR".into(),
        issuer: ecb_did.clone(),
        oracle: OracleConfig::url("https://ecb.europa.eu/rates"),
        compliance: ComplianceConfig::eu_aml(),
    },
    ecb_signature,
).await?;

// Aktivieren
let eu_env = bootstrap.activate(&mut context).await?;

// Inter-Env Agreement erstellen
let agreement = InterEnvAgreement::new(
    eu_env.did.clone(),
    asean_env.did.clone(),
    AgreementType::TrustRecognition {
        trust_mapping: hashmap! {
            TrustLevel::Verified => TrustLevel::Neutral,
            TrustLevel::HighTrust => TrustLevel::Verified,
        },
        min_trust: 0.6,
    },
);

context.propose_agreement(agreement, &eu_governance_keys).await?;
```

#### 9.2 TypeScript

```typescript
import { VirtEnv, BootstrapProcess, CurrencyBridge } from "@erynoa/sdk";

// Virt-Env bootstrappen
const bootstrap = await BootstrapProcess.initiate({
  initiator: euCommissionDid,
  parentEnv: "did:erynoa:circle:root",
  config: {
    name: "European Union",
    did: "did:erynoa:circle:eu-2026",
    governance: {
      type: "multi-sig-dao",
      threshold: { numerator: 2, denominator: 3 },
      members: [euCommission, ecb, euParliament],
    },
  },
});

// CBDC Bridge
await bootstrap.setupCbdcBridge(
  {
    currency: "EUR",
    issuer: ecbDid,
    oracle: "https://ecb.europa.eu/rates",
  },
  ecbSignature,
);

// Aktivieren
const euEnv = await bootstrap.activate();

// Cross-CBDC Exchange
const result = await cbdcExchange.exchange({
  user: aliceDid,
  sourceAmount: 1000n,
  sourceCurrency: "EUR",
  targetCurrency: "SGD",
  minTargetAmount: 1450n, // Slippage protection
});
```

---

## Praktischer Ablauf: IoT-Gerät in Shard

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    GERÄT-ONBOARDING IN ENERGY-SHARD                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. QR-SCAN                                                                 │
│     ─────────                                                               │
│     • User scannt QR-Code am Gerät (z.B. Solar-Panel)                      │
│     • Master-DID erzeugt Sub-DID im `thing`-Namespace                      │
│     • did:erynoa:thing:solar-panel-roof-a1                                 │
│                                                                             │
│  2. SHARD-JOIN                                                              │
│     ──────────                                                              │
│     • Join-Event: EU-Energy/Siemens-Shard                                  │
│     • Shard-Regeln greifen: allowed_chains = ["iota"]                      │
│     • Virtuelle IOTA-Adresse wird deterministisch erzeugt                  │
│                                                                             │
│  3. AUTONOME OPERATION                                                      │
│     ────────────────────                                                    │
│     • Gerät subscribed Shard-Events (z.B. Strompreise)                     │
│     • Autonom: Einspeisen wenn Preis > Threshold                           │
│     • Settlement: wEUR-Mikrozahlungen im Shard                             │
│                                                                             │
│  4. INTER-DEVICE KOOPERATION                                                │
│     ──────────────────────────                                              │
│     • Mutual Auth mit Batterie-System                                       │
│     • Gemeinsamer Optimierungs-Algorithmus                                  │
│     • Trust-Attestation bei erfolgreicher Kooperation                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Zusammenfassung der 3-Schichten-Architektur

| Schicht      | Scope                   | Governance                   | Regeln                | Beispiel                       |
| ------------ | ----------------------- | ---------------------------- | --------------------- | ------------------------------ |
| **Root-Env** | Global                  | Unveränderlich (H4-Override) | 112 Axiome            | `did:erynoa:*:*`               |
| **Virt-Env** | Souveräne Gruppe        | DAO / Multi-Sig              | Root + Local Axioms   | `did:erynoa:circle:eu-2026`    |
| **Shard**    | Spezialisierter Bereich | Virt-Env-delegiert           | Parent + Shard-Axioms | `did:erynoa:circle:eu-finance` |

**Kernprinzip:** Globale Neutralität durch Root, lokale Souveränität durch Virt-Envs, kontextuelle Spezialisierung durch Shards – verbunden durch kategorientheoretisch fundierte Funktoren mit Trust-Dämpfung und Trust-Rotation.

---

## Test-Vektoren

### TV-1: Successful Bootstrap

**Input:**

- Initiator Trust: 0.9
- Parent: Root-Env
- Governance: 3-of-5 Multi-Sig

**Expected:**

- Bootstrap: Success
- Status: Active nach allen Phasen

### TV-2: Failed Bootstrap (Low Trust)

**Input:**

- Initiator Trust: 0.3
- Parent: Root-Env

**Expected:**

- Error: InsufficientTrust

### TV-3: Axiom Conflict

**Input:**

- Root Axiom: "Trust kann nie unter 0.3 fallen"
- Local Axiom: "Trust kann auf 0 fallen bei Fraud"

**Expected:**

- Error: ContradictsRoot

### TV-4: Cross-Env Trust Recognition

**Input:**

- Source DID Trust: 0.85 (HighTrust in EU)
- Agreement: TrustRecognition (HighTrust → Verified)

**Expected:**

- Target Trust Level: Verified
- Source Trust Visible: 0.85

### TV-5: Cross-Shard Funktor (V0.2)

**Input:**

- Source Shard: EU-Finance (𝒞_EU_Finance)
- Target Shard: EU-Energy (𝒞_EU_Energy)
- Asset: 1000 wEUR
- Funktor trust_factor: 0.9

**Expected:**

- Transfer: Success
- Trust in Target: 0.9 × Source Trust
- Asset: 1000 wEUR (Identity-Konversion)

### TV-6: Cross-CBDC Funktor (V0.2)

**Input:**

- Source Shard: EU-Finance (wEUR)
- Target Shard: ASEAN-Trade (wSGD)
- Asset: 1000 wEUR
- Exchange Rate: 1 EUR = 1.45 SGD
- Funktor trust_factor: 0.7

**Expected:**

- Transfer: Success
- Trust in Target: 0.7 × Source Trust
- Asset: 1450 wSGD (Exchange-Konversion)

---

## Referenzen

- [EIP-001: DID:erynoa](./EIP-001-did-erynoa.md)
- [EIP-002: Trust Vector 6D](./EIP-002-trust-vector-6d.md)
- [EIP-003: Event-DAG](./EIP-003-event-dag-finality.md)
- [EIP-004: Bayesian Trust](./EIP-004-bayesian-trust-update.md)
- [Erynoa Fachkonzept V6.2](../FACHKONZEPT.md)
- [Erynoa LOGIC.md – Realm-Axiome A18-A22](../LOGIC.md)
- [Erynoa LOGIC.md – Quanten-Axiome Q6-Q8](../LOGIC.md)
- [BIS CBDC Principles](https://www.bis.org/publ/othp33.htm)
- [Digital Euro Project](https://www.ecb.europa.eu/paym/digital_euro/)
- [Category Theory (nLab)](https://ncatlab.org/nlab/show/category+theory)

---

## Changelog

| Version | Datum      | Änderung                                                                                                                                                                                                                                                                                       |
| ------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0.1     | 2026-01-29 | Initial Draft: Root-Env/Virt-Env Architecture, CBDC Bridges, Bootstrapping, Inter-Env Protocol                                                                                                                                                                                                 |
| 0.2     | 2026-01-29 | **Shard-Integration**: Kategorientheorie (Axiome Q6-Q8), Realm-Axiome (A18-A22), ShardTypes, CBDC-Shards, Cross-Shard Funktoren, Trust-Weights pro Shard, Shard-Bootstrapping, CLI-Erweiterungen                                                                                               |
| 0.3     | 2026-02-01 | **Refined**: Unified Identity (BIP39/Passkey → Multi-Chain Wallets), Trust-Matrix (6x6 Transformation statt skalarer Dämpfung), Dynamic Dampening (Kybernetik E6), Boundary Guards (Logic Guards L1-L3), HTLC Atomic Swaps, Optional Recovery (nachträglich aktivierbar), RightsTransfer-Event |
| 0.4     | 2026-01-31 | **FACHKONZEPT V6.2 Sync**: Bootstrapping-Modi (Short/Long), Trust-Gewichtungs-Tabelle nach Shard-Typ, IoT-Gerät-Onboarding-Diagramm, 3-Schichten-Zusammenfassungs-Tabelle                                                                                                                      |

---

_EIP-005: Virtualized Environment Architecture_
_Version: 0.4 (FACHKONZEPT V6.2 Sync)_
_Status: Draft_
_Ebene: E2 (Emergenz) / E5 (Schutz) / E6 (Kybernetik)_
