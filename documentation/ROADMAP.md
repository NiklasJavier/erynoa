# Erynoa – Roadmap

> **Dokumenttyp:** Strategic Roadmap
> **Version:** 4.0 (basierend auf Fachkonzept V6.1)
> **Status:** Draft
> **Letzte Aktualisierung:** Januar 2026
> **Zielgruppe:** Gründer:innen, Entwickler:innen, Investoren, Partner
> **Referenz:** [Fachkonzept V6.1](./concept-v3/FACHKONZEPT.md)

---

## Executive Summary

Diese Roadmap beschreibt den Implementierungsplan für **Erynoa** – das probabilistische kybernetische Protokoll für vertrauensbasierte Interaktionen. Der Plan basiert auf der **7-Ebenen-Architektur** (concept-v3) mit **112 Axiomen** und ist in **5 Hauptphasen** strukturiert.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                        ERYNOA DEVELOPMENT ROADMAP v4                        │
│                                                                             │
│   ════════════════════════════════════════════════════════════════════════  │
│                                                                             │
│   PHASE 0       PHASE 1        PHASE 2         PHASE 3        PHASE 4      │
│   RESEARCH      FOUNDATION     PROTOCOL        ROBUSTNESS     NETWORK      │
│   ────────      ──────────     ────────        ──────────     ───────      │
│   3-6 Mo.       9-12 Mo.       12-15 Mo.       6-9 Mo.        Ongoing      │
│                                                                             │
│   ┌───────┐    ┌───────┐      ┌───────┐       ┌───────┐      ┌───────┐    │
│   │ 🔬    │───▶│ E1-E2 │─────▶│ E3-E5 │──────▶│ E6-E7 │─────▶│ 🌐    │    │
│   │ Specs │    │Fundamt│      │Prozess│       │Robust │      │Testnet│    │
│   │ & PoC │    │Emergz │      │Objekt │       │Humanis│      │& Main │    │
│   │       │    │       │      │Schutz │       │       │      │       │    │
│   └───────┘    └───────┘      └───────┘       └───────┘      └───────┘    │
│                                                                             │
│   Q1-Q2 2026   Q2-Q4 2026     2027            Q1-Q2 2028     2028+         │
│                                                                             │
│   ─────────────────────────────────────────────────────────────────────    │
│                                                                             │
│   DIE 7 EBENEN (112 Axiome):                                               │
│   E1 Fundament → E2 Emergenz → E3 Prozess → E4 Objekt →                    │
│   E5 Schutz → E6 Kybernetik → E7 Humanismus                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Kernprinzip:** Die Ebenen bauen kausal aufeinander auf. Keine Ebene kann ohne ihre Vorgänger funktionieren.

---

## Die Sieben Ebenen im Überblick

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   EBENE    AXIOME   PHASE    FOKUS                                         │
│   ═══════════════════════════════════════════════════════════════════════  │
│                                                                             │
│   E1 FUNDAMENT    30       1       Identität, Kausalität, Basis-Trust      │
│   E2 EMERGENZ     15       1       Kollektive Intelligenz, Konsens         │
│   E3 PROZESS      13       2       TAT-Lifecycle, Streaming                │
│   E4 OBJEKT        9       2       Assets, Credentials, Blueprints         │
│   E5 SCHUTZ       18       2       Anti-Gaming, Anti-Calcification         │
│   E6 KYBERNETIK   23       3       Feedback, Circuit Breakers, Antifragil  │
│   E7 HUMANISMUS    4       3       Human-Alignment, LoD, Amnesty, Semantic │
│                                                                             │
│   ─────────────────────────────────────────────────────────────────────    │
│   GESAMT:        112 Axiome                                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Die Systemgleichung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   𝔼 = Σ A(s) · σ( W(s) · ln|C(s)| · N(s) / E(s) ) · H(s) · w(s,t)         │
│       s                                                                     │
│                                                                             │
│   KOMPONENTEN:                                                              │
│   ─────────────                                                             │
│   𝔼        = Systemwert (kollektive Intelligenz)                           │
│   A(s)     = Aktivitätspräsenz [0,1]                                       │
│   W(s)     = Wächter-Metrik (6D: R,I,C,P,V,Ω)                              │
│   C(s)     = Kausale Geschichte (Event-DAG)                                │
│   N(s)     = Novelty-Score (Informationsgewinn)                            │
│   E(s)     = Erwartungswert (Vorhersagbarkeit)                             │
│   σ(x)     = Sigmoid-Funktion σ(x) = 1/(1+e^(-x))                          │
│   H(s)     = Human-Alignment (2.0|1.5|1.0)                                 │
│   w(s,t)   = Temporale Gewichtung (Vergebungs-Faktor)                      │
│                                                                             │
│   MATHEMATIK: Klassische Wahrscheinlichkeitstheorie, Bayessche Inferenz    │
│   HARDWARE:   Standard-Server, keine Spezial-Hardware erforderlich         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 0: Research & Proof of Concept (Q1-Q2 2026)

> **Ziel:** Technische Machbarkeit validieren, Architekturentscheidungen treffen
> **Dauer:** 3-6 Monate

### 0.1 Erynoa Improvement Proposals (EIPs)

| ID | EIP | Beschreibung | Ebene | Prio | Status |
|----|-----|--------------|-------|------|--------|
| **R0.1** | 001 | DID:erynoa Method Specification | E1 | 🔴 | 📋 |
| **R0.2** | 002 | Trust Vector 6D Specification (R,I,C,P,V,Ω) | E1 | 🔴 | 📋 |
| **R0.3** | 003 | Event-DAG & Finality Specification | E1 | 🔴 | 📋 |
| **R0.4** | 004 | Bayesian Trust Update Algorithm | E2 | 🔴 | 📋 |
| **R0.5** | 005 | EigenTrust Topology Specification | E2 | 🔴 | 📋 |
| **R0.6** | 006 | TAT-Lifecycle (Seek→Close) | E3 | 🔴 | 📋 |
| **R0.7** | 007 | Value Streaming Protocol | E3 | 🔴 | 📋 |
| **R0.8** | 008 | Blueprint Schema Standard | E4 | 🔴 | 📋 |
| **R0.9** | 009 | Credential Issuance & Verification | E4 | 🔴 | 📋 |
| **R0.10** | 010 | Anti-Gaming Measures (Stake, Slashing) | E5 | 🟡 | 📋 |
| **R0.11** | 011 | Circuit Breaker Specification | E6 | 🟡 | 📋 |
| **R0.12** | 012 | Human-Auth Credential (H1) | E7 | 🟡 | 📋 |
| **R0.13** | 013 | Level-of-Detail Trust (H2) | E7 | 🟡 | 📋 |
| **R0.14** | 014 | Temporal Forgiveness / Amnesty (H3) | E7 | 🟡 | 📋 |
| **R0.15** | 015 | Semantic Anchoring (H4) | E7 | 🟡 | 📋 |

### 0.2 Technologie-Evaluation

| ID | Technologie | Beschreibung | Ebene | Prio | Status |
|----|-------------|--------------|-------|------|--------|
| **T1.1** | IOTA Rebased | Event-DAG, MoveVM, Starfish BFT | E1 | 🔴 | 📋 |
| **T1.2** | Qdrant | Vector Search für Semantic Index | E4 | 🔴 | 📋 |
| **T1.3** | libp2p | P2P Networking, Kademlia DHT | E6 | 🟡 | 📋 |
| **T1.4** | Ed25519 + Dilithium | Hybrid-Signaturen (Post-Quantum ready) | E6 | 🟡 | 📋 |
| **T1.5** | Wasmtime | ECL Runtime (deterministisch, sandboxed) | E3 | 🔴 | 📋 |

### 0.3 Proof of Concepts

| ID | PoC | Beschreibung | Ebene | Prio | Status |
|----|-----|--------------|-------|------|--------|
| **P1.1** | DID Resolution | 10 Namespace-Patterns, Controller-Chain | E1 | 🔴 | 📋 |
| **P1.2** | Bayesian Trust | Trust-Update mit Konfidenzintervallen | E2 | 🔴 | 📋 |
| **P1.3** | EigenTrust | Globales Ranking, Sybil-Resistenz | E2 | 🔴 | 📋 |
| **P1.4** | TAT Streaming | Mikro-Payments mit Escrow | E3 | 🔴 | 📋 |
| **P1.5** | LoD Calculator | Automatische Trust-Level-Auswahl | E7 | 🟡 | 📋 |
| **P1.6** | Human-Auth Flow | Biometric/Gov-ID Verifizierung | E7 | 🟡 | 📋 |

---

## Phase 1: Foundation Infrastructure (Q2-Q4 2026)

> **Ziel:** Ebene 1 (Fundament) + Ebene 2 (Emergenz) produktionsreif
> **Dauer:** 9-12 Monate
> **Referenz:** [FACHKONZEPT.md Teil II+III](./concept-v3/FACHKONZEPT.md)

### 1.1 E1 FUNDAMENT – Identität & Kausalität

#### 1.1.1 DID-System

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **F1.01** | did:erynoa Resolver | W3C DID-konforme Resolution | 🔴 | 📋 |
| **F1.02** | 10 Namespaces | self, guild, spirit, thing, vessel, source, craft, vault, pact, circle | 🔴 | 📋 |
| **F1.03** | Controller-Chain | Haftungskette für autonome Agenten | 🔴 | 📋 |
| **F1.04** | Sub-Identity | Delegation mit can_operate, can_revoke | 🟡 | 📋 |
| **F1.05** | DID Permanence | Deaktivierung, keine Löschung | 🔴 | 📋 |

#### 1.1.2 Event-DAG

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **F2.01** | Event Schema | id, type, actor, timestamp, parents, payload, signature | 🔴 | 📋 |
| **F2.02** | DAG Storage | Content-addressable, Merkle-Trees | 🔴 | 📋 |
| **F2.03** | Finality Levels | Pending → Attested → Anchored → Final | 🔴 | 📋 |
| **F2.04** | IOTA Anchoring | Primary Chain Integration | 🔴 | 📋 |
| **F2.05** | Merkle Proofs | Externe Verifizierbarkeit | 🟡 | 📋 |

#### 1.1.3 Basis-Trust

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **F3.01** | Trust Vector 6D | (R, I, C, P, V, Ω) Struktur | 🔴 | 📋 |
| **F3.02** | Trust Store | Persistenz im Semantic Index | 🔴 | 📋 |
| **F3.03** | Trust Decay | λ = 0.9997/Tag (6 Jahre Halbwertszeit) | 🔴 | 📋 |
| **F3.04** | Trust Floor | Minimum 0.3 (Rehabilitation möglich) | 🔴 | 📋 |
| **F3.05** | Asymmetrie | k_neg / k_pos ≈ 3-5 | 🔴 | 📋 |

### 1.2 E2 EMERGENZ – Kollektive Intelligenz

#### 1.2.1 Bayessche Trust-Evolution

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **E1.01** | Prior Distribution | Beta(2,2) für neue Agenten | 🔴 | 📋 |
| **E1.02** | Likelihood Update | Event-basierte Posterior-Berechnung | 🔴 | 📋 |
| **E1.03** | Konfidenzintervalle | 95%-CI für alle Trust-Werte | 🔴 | 📋 |
| **E1.04** | Qualitative Buckets | Unknown, Caution, Neutral, Verified, HighTrust | 🔴 | 📋 |
| **E1.05** | Hysterese | Anti-Oszillation an Schwellwerten | 🟡 | 📋 |

#### 1.2.2 EigenTrust & Globales Ranking

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **E2.01** | EigenTrust Algorithm | Iterative globale Trust-Berechnung | 🔴 | 📋 |
| **E2.02** | Sybil Detection | Isolierte Cluster erhalten keinen globalen Trust | 🔴 | 📋 |
| **E2.03** | Trust Propagation | A→B→C mit decay < 1 | 🟡 | 📋 |

#### 1.2.3 Witness-System

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **E3.01** | Witness Protocol | k-of-n unabhängige Zeugen | 🔴 | 📋 |
| **E3.02** | Geographic Diversity | Min. 2 Regionen für Enhanced+ | 🟡 | 📋 |
| **E3.03** | Hardware Diversity | Min. 2 Hersteller für Maximum | 🟢 | 📋 |

### 1.3 Developer Platform (Phase 1)

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **D1.01** | erynoa-core (Rust) | Systemgleichung-Engine, Crypto, Storage | 🔴 | 📋 |
| **D1.02** | erynoa-sdk (Rust) | High-Level API | 🔴 | 📋 |
| **D1.03** | erynoa-sdk-ts | TypeScript/WASM Binding | 🔴 | 📋 |
| **D1.04** | erynoa-cli | Kommandozeilen-Tool | 🟡 | 📋 |
| **D1.05** | Local Devnet | Single-Node Test Environment | 🟡 | 📋 |

---

## Phase 2: Protocol Implementation (2027)

> **Ziel:** Ebene 3 (Prozess) + Ebene 4 (Objekt) + Ebene 5 (Schutz)
> **Dauer:** 12-15 Monate
> **Referenz:** [FACHKONZEPT.md Teil IV+V](./concept-v3/FACHKONZEPT.md)

### 2.1 E3 PROZESS – Transaktionen

#### 2.1.1 TAT-Lifecycle

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **P1.01** | SEEK Phase | Discovery mit Trust-basiertem Ranking | 🔴 | 📋 |
| **P1.02** | PROPOSE Phase | Signiertes Angebot mit Ricardian Contract | 🔴 | 📋 |
| **P1.03** | AGREE Phase | Matching, Escrow-Setup | 🔴 | 📋 |
| **P1.04** | STREAM Phase | Kontinuierliche Mikro-Payments | 🔴 | 📋 |
| **P1.05** | CLOSE Phase | Finale Attestation, Trust-Update | 🔴 | 📋 |
| **P1.06** | ABORT Phase | Proportionale Erstattung | 🟡 | 📋 |
| **P1.07** | DISPUTE Phase | Schiedsverfahren | 🟡 | 📋 |

#### 2.1.2 Level-of-Detail (LoD)

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **P2.01** | LoD Calculator | Automatische Level-Auswahl basierend auf Wert | 🔴 | 📋 |
| **P2.02** | 5 LoD-Levels | Minimal, Basic, Standard, Enhanced, Maximum | 🔴 | 📋 |
| **P2.03** | Cost Constraint | Cost ≤ 5% of Value | 🔴 | 📋 |
| **P2.04** | Green-Score | Effizienz-Metrik | 🟡 | 📋 |

### 2.2 E4 OBJEKT – Substanz

#### 2.2.1 Blueprint-System

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **O1.01** | Blueprint Schema | ECL-basierte Definition | 🔴 | 📋 |
| **O1.02** | Blueprint Registry | CRUD mit DID-Adressierung | 🔴 | 📋 |
| **O1.03** | Blueprint Versioning | Immutable Versions | 🔴 | 📋 |
| **O1.04** | NLD Requirement | Natural Language Description (H4) | 🔴 | 📋 |
| **O1.05** | LLM Equivalence Check | NLD ↔ FormalSpec Validierung | 🟡 | 📋 |

#### 2.2.2 Credential-System

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **O2.01** | VC Issuance | W3C-konforme Credentials | 🔴 | 📋 |
| **O2.02** | VC Verification | Multi-Chain Anchor Check | 🔴 | 📋 |
| **O2.03** | HumanAuth Credential | Mensch-Verifizierung (H1) | 🔴 | 📋 |
| **O2.04** | Revocation | Widerruf mit Anchor Proof | 🟡 | 📋 |

#### 2.2.3 Realm-System

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **O3.01** | Realm Schema | ECL-basierte Realm-Definition | 🔴 | 📋 |
| **O3.02** | Realm Hierarchy | Global → Domain → Private | 🔴 | 📋 |
| **O3.03** | Cross-Realm Bridges | Konversionsregeln zwischen Realms | 🟡 | 📋 |
| **O3.04** | Realm Governance | Council, Proposals, Voting | 🟡 | 📋 |

### 2.3 E5 SCHUTZ – Anti-Gaming

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **S1.01** | Stake-at-Risk | Bonding für hohe Reputation | 🔴 | 📋 |
| **S1.02** | Slashing | Automatische Bestrafung bei Betrug | 🔴 | 📋 |
| **S1.03** | Collusion Detection | Cluster-Analyse | 🟡 | 📋 |
| **S1.04** | Novelty Bonus | 3x für neue Partner (Anti-Calcification) | 🟡 | 📋 |
| **S1.05** | Trust Decay Enforcement | Aktive ≠ Passive Reputation | 🔴 | 📋 |

---

## Phase 3: Robustness & Humanismus (Q1-Q2 2028)

> **Ziel:** Ebene 6 (Kybernetik) + Ebene 7 (Humanismus) + Security Hardening
> **Dauer:** 6-9 Monate
> **Referenz:** [ROBUSTNESS-LAYER.md](./concept-v3/ROBUSTNESS-LAYER.md), [CONSTITUTION.md](./concept-v3/CONSTITUTION.md)

### 3.1 E6 KYBERNETIK – Antifragilität

#### 3.1.1 Circuit Breakers

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **K1.01** | Trust Velocity Limiter | Max ±10% pro Stunde | 🔴 | 📋 |
| **K1.02** | Volatility Monitor | Abort-Rate Überwachung | 🔴 | 📋 |
| **K1.03** | Automatic Cooldown | 10min Freeze bei Kritisch | 🔴 | 📋 |
| **K1.04** | Dampening | Glättung schneller Änderungen | 🟡 | 📋 |

#### 3.1.2 Hardware Diversity

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **K2.01** | Manufacturer Registry | Tracking von Hardware-Herstellern | 🟡 | 📋 |
| **K2.02** | Diversity Constraints | Min. k Witnesses, m Hersteller, r Regionen | 🟡 | 📋 |

#### 3.1.3 Post-Quantum Readiness

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **K3.01** | Hybrid Signatures | Ed25519 + Dilithium-3 | 🟡 | 📋 |
| **K3.02** | Key Rotation | Trust-erhaltende Migration | 🟢 | 📋 |
| **K3.03** | Crypto Agility | Algorithmus-Wechsel ohne Hard Fork | 🟢 | 📋 |

### 3.2 E7 HUMANISMUS – Verfassung

#### 3.2.1 H1: Human-Alignment

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **H1.01** | H(s) Funktor | 2.0/1.5/1.0 Multiplikator | 🔴 | 📋 |
| **H1.02** | HumanAuth Verification | Biometric, Gov-ID, Video, Web-of-Trust | 🔴 | 📋 |
| **H1.03** | Human-Interaction Quota | Min. 20% der Wertschöpfung | 🟡 | 📋 |

#### 3.2.2 H2: Verhältnismäßigkeit

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **H2.01** | LoD Integration | Automatische Level-Wahl | 🔴 | 📋 |
| **H2.02** | Cost Constraint | ≤5% Enforcement | 🔴 | 📋 |
| **H2.03** | Green-Trust Score | Effizienz-Ranking | 🟡 | 📋 |

#### 3.2.3 H3: Temporale Gnade

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **H3.01** | Temporal Weighting | w(e,t) = e^(-γ·age) | 🔴 | 📋 |
| **H3.02** | Asymmetric Decay | γ_neg=0.000633, γ_pos=0.000380 | 🔴 | 📋 |
| **H3.03** | Automatic Amnesty | Nach 7 Jahren ohne negative Events | 🟡 | 📋 |
| **H3.04** | Fresh-Start | Neue DID mit positivem Trust-Transfer | 🟢 | 📋 |

#### 3.2.4 H4: Semantische Verankerung

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **H4.01** | NLD Requirement | Menschenlesbare Beschreibung | 🔴 | 📋 |
| **H4.02** | FormalSpec Requirement | Maschinenprüfbare Spezifikation | 🔴 | 📋 |
| **H4.03** | LLM Equivalence Auditor | Automatische Äquivalenz-Prüfung | 🟡 | 📋 |
| **H4.04** | Glossary Enforcement | Technische Begriffe erklärt | 🟢 | 📋 |

### 3.3 Security Hardening

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **S1.01** | Security Audit Phase 1 | E1+E2 Code Review | 🔴 | 📋 |
| **S1.02** | Security Audit Phase 2 | E3+E4+E5 Review | 🔴 | 📋 |
| **S1.03** | Security Audit Phase 3 | E6+E7 Review | 🔴 | 📋 |
| **S1.04** | Penetration Testing | Full Stack | 🔴 | 📋 |
| **S1.05** | Bug Bounty (Private) | Closed Beta | 🟡 | 📋 |

---

## Phase 4: Network Launch (2028+)

> **Ziel:** Testnet, Piloten, Mainnet Launch
> **Dauer:** Ongoing

### 4.1 Testnet

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **T1.01** | Testnet Alpha | Private, 10-20 Nodes | 📋 | 📋 |
| **T1.02** | Testnet Beta | Public, 50+ Nodes | 📋 | 📋 |
| **T1.03** | Incentivized Testnet | Rewards | 📋 | 📋 |

### 4.2 Pilot: EV-Charging

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **EV1.01** | OCPP Bridge | OCPP 2.0.1 ↔ Erynoa | 📋 | 📋 |
| **EV1.02** | 5 Operators, 100 Chargers | Onboarding | 📋 | 📋 |
| **EV1.03** | 500+ Vehicle Agents | Mobile App | 📋 | 📋 |
| **EV1.04** | 1000+ Charging Sessions | Live Test | 📋 | 📋 |

### 4.3 Mainnet

| ID | Milestone | Beschreibung | Prio | Status |
|----|-----------|--------------|------|--------|
| **M1.01** | Genesis Preparation | Validator Setup | 📋 | 📋 |
| **M1.02** | Mainnet Launch | Go-Live | 📋 | 📋 |
| **M1.03** | 50+ Validators | Decentralization | 📋 | 📋 |

---

## Success Metrics

### Technical KPIs

| Phase | Ebene | Metric | Target |
|-------|-------|--------|--------|
| Phase 1 | E1 | DID Resolution | < 50ms (p95) |
| Phase 1 | E2 | Trust Calculation | < 10ms |
| Phase 1 | E2 | Konfidenzintervall-Berechnung | < 5ms |
| Phase 2 | E3 | TAT Full Cycle (Seek→Close) | < 10s |
| Phase 2 | E4 | Blueprint Validation | < 100ms |
| Phase 2 | E5 | Sybil Detection | < 1s |
| Phase 3 | E6 | Circuit Breaker Activation | < 100ms |
| Phase 3 | E7 | HumanAuth Verification | < 30s |
| Phase 3 | E7 | LLM Equivalence Check | < 5s |
| Phase 4 | ALL | Testnet Uptime | > 99.5% |
| Phase 4 | ALL | Mainnet Uptime | > 99.9% |

### Business KPIs

| Phase | Metric | Target |
|-------|--------|--------|
| Phase 1 | SDKs Released | 3 (Rust, TS, Python) |
| Phase 2 | Blueprints Published | 10+ |
| Phase 3 | Security Audits Passed | 3 |
| Phase 4 | Active DIDs (Year 1) | 50.000+ |
| Phase 4 | Daily Transactions (Year 1) | 10.000+ |

---

## Risk Matrix

| Risk | Wahrscheinlichkeit | Impact | Mitigation |
|------|-------------------|--------|------------|
| IOTA Rebased Verzögerung | 🟡 Mittel | 🔴 Hoch | Alternative L1, modularer Ansatz |
| Trust Gaming | 🟢 Niedrig | 🔴 Hoch | EigenTrust, Stake-at-Risk, Asymmetrie |
| Humanismus-Akzeptanz | 🟡 Mittel | 🟡 Mittel | Opt-in für Enterprises, klare Kommunikation |
| LLM Equivalence Accuracy | 🟡 Mittel | 🟡 Mittel | Fallback auf manuellen Review |
| Regulatory Changes | 🟡 Mittel | 🟡 Mittel | Legal Advisory, Compliance-First |

---

## Verwandte Dokumente (concept-v3)

| Dokument | Beschreibung |
|----------|--------------|
| [FACHKONZEPT.md](./concept-v3/FACHKONZEPT.md) | Vollständiges technisches Konzept |
| [WORLD-FORMULA.md](./concept-v3/WORLD-FORMULA.md) | Systemgleichung, Axiome |
| [LOGIC.md](./concept-v3/LOGIC.md) | Formale Logik, Beweise |
| [CONSTITUTION.md](./concept-v3/CONSTITUTION.md) | Humanistische Verfassung (H1-H4) |
| [ROBUSTNESS-LAYER.md](./concept-v3/ROBUSTNESS-LAYER.md) | Antifragilitäts-Architektur |
| [SDK-ARCHITECTURE.md](./concept-v3/SDK-ARCHITECTURE.md) | SDK-Spezifikation |
| [PROTOCOL.md](./concept-v3/PROTOCOL.md) | Protokoll-Details |
| [CLI-REFERENCE.md](./concept-v3/CLI-REFERENCE.md) | CLI-Befehle |

---

<div align="center">

**Erynoa – Probabilistisches Protokoll für vertrauensbasierte Interaktionen**

_„Intelligenz im Dienste des Lebens."_

```
E1 FUNDAMENT → E2 EMERGENZ → E3 PROZESS → E4 OBJEKT →
E5 SCHUTZ → E6 KYBERNETIK → E7 HUMANISMUS
         │                                    │
         └──────────── FEEDBACK LOOP ◀────────┘
```

**112 Axiome · 7 Ebenen · Klassische Wahrscheinlichkeitstheorie**

</div>
