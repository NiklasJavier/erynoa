# Erynoa – Roadmap

> **Dokumenttyp:** Strategic Roadmap
> **Version:** 4.1 (basierend auf Fachkonzept V6.2 + Backend V4.1)
> **Status:** Active Development
> **Letzte Aktualisierung:** Januar 2026
> **Zielgruppe:** Gründer:innen, Entwickler:innen, Investoren, Partner
> **Referenz:** [Fachkonzept V6.2](./concept-v3/FACHKONZEPT.md)

---

## Executive Summary

Diese Roadmap beschreibt den Implementierungsplan für **Erynoa** – das probabilistische kybernetische Protokoll für vertrauensbasierte Interaktionen. Der Plan basiert auf der **3-Schichten-Architektur** mit **126 Axiomen** (inkl. 6 Peer-Axiome) und ist in **5 Hauptphasen** strukturiert.

**🎯 Aktueller Status:** Phase 1.2 – Core Logic Layer ✅ COMPLETE

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║                     ERYNOA DEVELOPMENT ROADMAP v4.1                           ║
║                                                                               ║
║   ══════════════════════════════════════════════════════════════════════════  ║
║                                                                               ║
║   PHASE 0       PHASE 1        PHASE 2         PHASE 3        PHASE 4        ║
║   RESEARCH      FOUNDATION     PROTOCOL        ROBUSTNESS     NETWORK        ║
║   ────────      ──────────     ────────        ──────────     ───────        ║
║   3 Mo.  ✅     9-12 Mo.       12-15 Mo.       6-9 Mo.        Ongoing        ║
║                                                                               ║
║   ┌───────┐    ┌───────┐      ┌───────┐       ┌───────┐      ┌───────┐      ║
║   │ 🔬    │───▶│ 🔧    │─────▶│ ⚙️    │──────▶│ 🛡️    │─────▶│ 🌐    │      ║
║   │ Specs │    │Domain │      │Protocol│      │Robust │      │Testnet│      ║
║   │ & EIP │    │Core   │      │API     │      │Security│     │& Main │      ║
║   │ ✅    │    │ ▶60%  │      │        │      │       │      │       │      ║
║   └───────┘    └───────┘      └───────┘       └───────┘      └───────┘      ║
║                                                                               ║
║   Q1 2026      Q2-Q4 2026     2027            Q1-Q2 2028     2028+           ║
║                                                                               ║
║   ═══════════════════════════════════════════════════════════════════════    ║
║                                                                               ║
║   DIE 4 SCHICHTEN (126 Axiome):                                              ║
║   ┌─────────────────────────────────────────────────────────────────────┐    ║
║   │  Peer Layer         (IntentParser, SagaComposer, GatewayGuard) ✅   │    ║
║   ├─────────────────────────────────────────────────────────────────────┤    ║
║   │  Protection Layer   (AntiCalcification, Diversity, Quadratic) ✅    │    ║
║   ├─────────────────────────────────────────────────────────────────────┤    ║
║   │  Core Logic Layer   (EventEngine, TrustEngine, Consensus) ✅        │    ║
║   ├─────────────────────────────────────────────────────────────────────┤    ║
║   │  Domain Layer       (DID, Trust6D, Event, Realm, Saga) ✅           │    ║
║   └─────────────────────────────────────────────────────────────────────┘    ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

---

## 📊 Implementation Status Dashboard

### Backend Module Overview (73 Tests ✅)

| Schicht        | Module                                                                                   | Status | Tests | Axiome  |
| -------------- | ---------------------------------------------------------------------------------------- | ------ | ----- | ------- |
| **Domain**     | `did.rs`, `trust.rs`, `event.rs`, `realm.rs`, `saga.rs`, `formula.rs`                    | ✅     | 23    | Κ1-Κ5   |
| **Core Logic** | `event_engine.rs`, `trust_engine.rs`, `surprisal.rs`, `world_formula.rs`, `consensus.rs` | ✅     | 23    | Κ6-Κ18  |
| **Protection** | `anti_calcification.rs`, `diversity.rs`, `quadratic.rs`, `anomaly.rs`                    | ✅     | 17    | Κ19-Κ21 |
| **Peer**       | `intent_parser.rs`, `saga_composer.rs`, `gateway.rs`                                     | ✅     | 14    | PR1-PR6 |

### API Layer Status

| Service              | Proto | Rust | TypeScript | Status        |
| -------------------- | ----- | ---- | ---------- | ------------- |
| `PeerService`        | ✅    | 📋   | 📋         | Proto defined |
| `IntentService`      | ✅    | 📋   | 📋         | Proto defined |
| `SagaService`        | ✅    | 📋   | 📋         | Proto defined |
| `EnvironmentService` | ✅    | 📋   | 📋         | Proto defined |
| `StorageService`     | ✅    | ✅   | ✅         | Complete      |
| `HealthService`      | ✅    | ✅   | ✅         | Complete      |
| `InfoService`        | ✅    | ✅   | ✅         | Complete      |
| `UserService`        | ✅    | ✅   | ✅         | Complete      |

---

## Die Systemgleichung

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║   𝔼 = Σ A(s) · σ( W(s) · ln|C(s)| · N(s) / E(s) ) · H(s) · w(s,t)           ║
║       s                                                                       ║
║                                                                               ║
║   IMPLEMENTIERTE KOMPONENTEN:                                                 ║
║   ──────────────────────────────────────────────────────────────────────────  ║
║                                                                               ║
║   ✅ A(s)     = Aktivitätspräsenz [0,1]           → domain/formula.rs        ║
║   ✅ W(s)     = Wächter-Metrik 6D (R,I,C,P,V,Ω)   → domain/trust.rs          ║
║   ✅ C(s)     = Kausale Geschichte (Event-DAG)    → domain/event.rs          ║
║   ✅ N(s)     = Novelty-Score (Surprisal)         → core/surprisal.rs        ║
║   ✅ E(s)     = Erwartungswert                    → core/surprisal.rs        ║
║   ✅ σ(x)     = Sigmoid-Funktion                  → domain/formula.rs        ║
║   ✅ H(s)     = Human-Alignment (2.0|1.5|1.0)     → domain/formula.rs        ║
║   ✅ w(s,t)   = Temporale Gewichtung              → core/world_formula.rs    ║
║                                                                               ║
║   ✅ Trust-Combination (trust_combine)            → domain/trust.rs          ║
║   ✅ Chain-Trust (chain_trust)                    → domain/trust.rs          ║
║   ✅ Trust-Dampening-Matrix 6×6                   → domain/trust.rs          ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

---

## Phase 0: Research & Proof of Concept ✅ COMPLETE

> **Ziel:** Technische Machbarkeit validieren, Architekturentscheidungen treffen
> **Dauer:** 3 Monate (Q1 2026)
> **Status:** ✅ ABGESCHLOSSEN

### 0.1 Erynoa Improvement Proposals (EIPs)

| ID       | EIP | Beschreibung                                                   | Status |
| -------- | --- | -------------------------------------------------------------- | ------ |
| **R0.1** | 001 | DID:erynoa Method Specification (V0.3 mit Staked Guardianship) | ✅     |
| **R0.2** | 002 | Trust Vector 6D Specification (R,I,C,P,V,Ω)                    | ✅     |
| **R0.3** | 003 | Event-DAG & Finality Specification (V0.2 mit MMR)              | ✅     |
| **R0.4** | 004 | Bayesian Trust Update Algorithm (V0.2 mit Loop Detection)      | ✅     |
| **R0.5** | 005 | Virtualized Environment Architecture (Root-Env/Virt-Env)       | ✅     |
| **R0.6** | 006 | Slashing & Dispute Resolution                                  | ✅     |
| **R0.7** | 007 | EigenTrust Topology Specification                              | ✅     |
| **R0.8** | 008 | TAT-Lifecycle (Seek→Close)                                     | ✅     |
| **R0.9** | 009 | Peer Architecture (Gateway, Composer, Saga)                    | ✅     |

### 0.2 Technologie-Stack (Final)

| Technologie             | Beschreibung                             | Status |
| ----------------------- | ---------------------------------------- | ------ |
| **Rust + Axum**         | High-Performance Backend mit async/await | ✅     |
| **Connect-RPC**         | Type-safe gRPC/gRPC-Web (Proto3)         | ✅     |
| **PostgreSQL/OrioleDB** | Primärer Datenspeicher                   | ✅     |
| **DragonflyDB**         | Redis-kompatibles Caching                | ✅     |
| **ZITADEL**             | Identity & Access Management             | ✅     |
| **SvelteKit**           | Frontend (Console, Platform, Docs)       | ✅     |
| **Buf**                 | Protobuf Management & Code Generation    | ✅     |

---

## Phase 1: Foundation Infrastructure (Q2-Q4 2026) – IN PROGRESS

> **Ziel:** Domain-Layer + Core-Logic-Layer + Protection-Layer + Peer-Layer
> **Dauer:** 9-12 Monate
> **Status:** 🔧 ~60% Complete

### 1.1 Domain Layer ✅ COMPLETE

#### 1.1.1 DID-System (`domain/did.rs`)

| ID        | Milestone        | Beschreibung                                                           | Status | Test |
| --------- | ---------------- | ---------------------------------------------------------------------- | ------ | ---- |
| **D1.01** | DID Structure    | `did:erynoa:<namespace>:<unique_id>`                                   | ✅     | ✅   |
| **D1.02** | 10 Namespaces    | self, guild, spirit, thing, vessel, source, craft, vault, pact, circle | ✅     | ✅   |
| **D1.03** | DID Parsing      | `FromStr` implementation                                               | ✅     | ✅   |
| **D1.04** | Controller-Chain | Delegation mit Trust-Faktoren                                          | ✅     | ✅   |
| **D1.05** | DID Equality     | Hash/Eq ohne `created_at` (Κ1)                                         | ✅     | ✅   |

#### 1.1.2 Trust Vector 6D (`domain/trust.rs`)

| ID        | Milestone         | Beschreibung                    | Status | Test |
| --------- | ----------------- | ------------------------------- | ------ | ---- |
| **T1.01** | TrustVector6D     | (R, I, C, P, V, Ω) ∈ [0,1]⁶     | ✅     | ✅   |
| **T1.02** | Weighted Norm     | Standard-Gewichte implementiert | ✅     | ✅   |
| **T1.03** | Trust Combination | `trust_combine(a, b)` Operator  | ✅     | ✅   |
| **T1.04** | Chain Trust       | A→B→C mit Trust-Decay           | ✅     | ✅   |
| **T1.05** | Dampening Matrix  | 6×6 für Realm-Crossing          | ✅     | ✅   |
| **T1.06** | Asymmetric Update | k_neg > k_pos                   | ✅     | ✅   |

#### 1.1.3 Event-DAG (`domain/event.rs`)

| ID        | Milestone       | Beschreibung                                                           | Status | Test |
| --------- | --------------- | ---------------------------------------------------------------------- | ------ | ---- |
| **E1.01** | Event Structure | id, event_type, actor, timestamp, parents, payload, signature          | ✅     | ✅   |
| **E1.02** | Event Types     | Transfer, TrustAttestation, RealmCrossing, Delegation, Genesis, Custom | ✅     | ✅   |
| **E1.03** | Finality Levels | Pending < Attested < Anchored < Final                                  | ✅     | ✅   |
| **E1.04** | Genesis Event   | Spezial-Event ohne Parents                                             | ✅     | ✅   |
| **E1.05** | Event Creation  | Builder Pattern                                                        | ✅     | ✅   |

#### 1.1.4 Realm-System (`domain/realm.rs`)

| ID        | Milestone        | Beschreibung                                 | Status | Test |
| --------- | ---------------- | -------------------------------------------- | ------ | ---- |
| **R1.01** | RealmId          | String-basierte Identifikation               | ✅     | ✅   |
| **R1.02** | RealmType        | Root, Virtual, Partition                     | ✅     | ✅   |
| **R1.03** | Rule Inheritance | Virtual erbt von Root, Partition von Virtual | ✅     | ✅   |
| **R1.04** | Local Rules      | Additive lokale Regeln                       | ✅     | ✅   |
| **R1.05** | Root-Env         | 112 Kern-Axiome als unveränderliche Basis    | ✅     | ✅   |

#### 1.1.5 Saga-System (`domain/saga.rs`)

| ID        | Milestone      | Beschreibung                           | Status | Test |
| --------- | -------------- | -------------------------------------- | ------ | ---- |
| **S1.01** | Intent         | Abstract Goal mit Constraints          | ✅     | ✅   |
| **S1.02** | Saga           | Atomare Schritt-Sequenz                | ✅     | ✅   |
| **S1.03** | SagaStep       | Chain, Action, Status, Compensation    | ✅     | ✅   |
| **S1.04** | SagaState      | Pending → Executing → Completed/Failed | ✅     | ✅   |
| **S1.05** | Realm Crossing | Steps mit `crosses_realm` Flag         | ✅     | ✅   |

#### 1.1.6 World Formula (`domain/formula.rs`)

| ID        | Milestone            | Beschreibung                    | Status | Test |
| --------- | -------------------- | ------------------------------- | ------ | ---- |
| **F1.01** | AgentContribution    | A(s) · σ(inner) · H(s) · w(s,t) | ✅     | ✅   |
| **F1.02** | Activity Calculation | κ = 10, τ = 24h                 | ✅     | ✅   |
| **F1.03** | Human Factor         | 2.0 / 1.5 / 1.0                 | ✅     | ✅   |
| **F1.04** | Sigmoid Function     | σ(x) = 1/(1+e^(-x))             | ✅     | ✅   |
| **F1.05** | Surprisal Dampening  | σ clamps to (0,1)               | ✅     | ✅   |

---

### 1.2 Core Logic Layer ✅ COMPLETE

#### 1.2.1 Event Engine (`core/event_engine.rs`)

| ID         | Milestone        | Beschreibung           | Status | Test |
| ---------- | ---------------- | ---------------------- | ------ | ---- |
| **CE1.01** | Event Store      | In-Memory DAG Storage  | ✅     | ✅   |
| **CE1.02** | Add Event        | Parent-Validierung     | ✅     | ✅   |
| **CE1.03** | Genesis Event    | Spezial-Handling       | ✅     | ✅   |
| **CE1.04** | Causal History   | Rekursive Berechnung   | ✅     | ✅   |
| **CE1.05** | Parent Rejection | Missing Parent → Error | ✅     | ✅   |

#### 1.2.2 Trust Engine (`core/trust_engine.rs`)

| ID         | Milestone          | Beschreibung                | Status | Test |
| ---------- | ------------------ | --------------------------- | ------ | ---- |
| **CT1.01** | Trust Store        | DID → TrustVector6D Mapping | ✅     | ✅   |
| **CT1.02** | Initialize Trust   | Default für neue DIDs       | ✅     | ✅   |
| **CT1.03** | Process Event      | Positive/Negative Updates   | ✅     | ✅   |
| **CT1.04** | Self-Attestation   | Rejection (Κ4)              | ✅     | ✅   |
| **CT1.05** | Asymmetric Updates | k_neg = 0.10, k_pos = 0.05  | ✅     | ✅   |
| **CT1.06** | Combine Trust      | Multi-Source Aggregation    | ✅     | ✅   |
| **CT1.07** | Chain Trust        | A→B→C Propagation           | ✅     | ✅   |

#### 1.2.3 Surprisal Engine (`core/surprisal.rs`)

| ID         | Milestone           | Beschreibung                     | Status    | Test |
| ---------- | ------------------- | -------------------------------- | --------- | ---- | --- |
| **CS1.01** | SurprisalEngine     | -log₂(P(e                        | history)) | ✅   | ✅  |
| **CS1.02** | Count-Min Sketch    | Probabilistic Frequency Counting | ✅        | ✅   |
| **CS1.03** | Dampened Surprisal  | Sigmoid Normalization            | ✅        | ✅   |
| **CS1.04** | Expectation         | E(s) aus History                 | ✅        | ✅   |
| **CS1.05** | Overflow Protection | Saturating Arithmetic            | ✅        | ✅   |

#### 1.2.4 World Formula Engine (`core/world_formula.rs`)

| ID         | Milestone          | Beschreibung                       | Status | Test |
| ---------- | ------------------ | ---------------------------------- | ------ | ---- |
| **CW1.01** | Global Computation | 𝔼 = Σ contributions                | ✅     | ✅   |
| **CW1.02** | Temporal Weight    | exp(-γ · age)                      | ✅     | ✅   |
| **CW1.03** | Top Contributors   | Sorted by contribution             | ✅     | ✅   |
| **CW1.04** | Decay Constants    | γ_neg = 0.000633, γ_pos = 0.000380 | ✅     | ✅   |

#### 1.2.5 Consensus Engine (`core/consensus.rs`)

| ID         | Milestone              | Beschreibung            | Status | Test |
| ---------- | ---------------------- | ----------------------- | ------ | ---- |
| **CC1.01** | ConsensusEngine        | Finality Tracking       | ✅     | ✅   |
| **CC1.02** | Witness Registration   | DID-basierte Zeugen     | ✅     | ✅   |
| **CC1.03** | Add Attestation        | Witness-Validierung     | ✅     | ✅   |
| **CC1.04** | Finality Check         | ≥3 Witnesses → Final    | ✅     | ✅   |
| **CC1.05** | Revert Probability     | Sinkt mit Attestations  | ✅     | ✅   |
| **CC1.06** | Unauthorized Rejection | Nicht-Witness abgelehnt | ✅     | ✅   |

---

### 1.3 Protection Layer ✅ COMPLETE

#### 1.3.1 Anti-Calcification (`protection/anti_calcification.rs`)

| ID         | Milestone               | Beschreibung                  | Status | Test |
| ---------- | ----------------------- | ----------------------------- | ------ | ---- |
| **PA1.01** | CalcificationMonitor    | Novelty-Bonus Tracking        | ✅     | ✅   |
| **PA1.02** | Decay                   | Alter Partner verlieren Bonus | ✅     | ✅   |
| **PA1.03** | Gini Coefficient        | Ungleichheits-Metrik          | ✅     | ✅   |
| **PA1.04** | Power Cap               | Max Einfluss-Grenze           | ✅     | ✅   |
| **PA1.05** | Calcification Detection | Pattern Recognition           | ✅     | ✅   |

#### 1.3.2 Diversity Monitor (`protection/diversity.rs`)

| ID         | Milestone             | Beschreibung         | Status | Test |
| ---------- | --------------------- | -------------------- | ------ | ---- |
| **PD1.01** | DiversityMonitor      | Verteilungs-Tracking | ✅     | ✅   |
| **PD1.02** | Shannon Entropy       | H = -Σ p·log(p)      | ✅     | ✅   |
| **PD1.03** | Uniform Distribution  | Hohe Entropie        | ✅     | ✅   |
| **PD1.04** | Skewed Distribution   | Niedrige Entropie    | ✅     | ✅   |
| **PD1.05** | Monoculture Detection | Threshold < 0.5      | ✅     | ✅   |

#### 1.3.3 Quadratic Voting (`protection/quadratic.rs`)

| ID         | Milestone           | Beschreibung               | Status | Test |
| ---------- | ------------------- | -------------------------- | ------ | ---- |
| **PQ1.01** | QuadraticVoting     | √n Stimmen für n² Credits  | ✅     | ✅   |
| **PQ1.02** | Vote Cost           | cost = votes²              | ✅     | ✅   |
| **PQ1.03** | Credit Management   | Insufficient Credits Error | ✅     | ✅   |
| **PQ1.04** | Quadratic Advantage | Diminishing Returns        | ✅     | ✅   |
| **PQ1.05** | Max Votes           | √credits Berechnung        | ✅     | ✅   |

#### 1.3.4 Anomaly Detection (`protection/anomaly.rs`)

| ID          | Milestone          | Beschreibung               | Status | Test |
| ----------- | ------------------ | -------------------------- | ------ | ---- |
| **PAN1.01** | AnomalyDetector    | Multi-Dimension Detection  | ✅     | ✅   |
| **PAN1.02** | Velocity Detection | Trust-Änderung > Threshold | ✅     | ✅   |
| **PAN1.03** | Amount Detection   | Transfer > Threshold       | ✅     | ✅   |
| **PAN1.04** | Alert System       | Alert mit Severity         | ✅     | ✅   |

---

### 1.4 Peer Layer ✅ COMPLETE

#### 1.4.1 Intent Parser (`peer/intent_parser.rs`)

| ID         | Milestone         | Beschreibung              | Status | Test |
| ---------- | ----------------- | ------------------------- | ------ | ---- |
| **PI1.01** | IntentParser      | Natural Language → Intent | ✅     | ✅   |
| **PI1.02** | Transfer Intent   | "send X to Y" Pattern     | ✅     | ✅   |
| **PI1.03** | Delegation Intent | "delegate to Y" Pattern   | ✅     | ✅   |
| **PI1.04** | Amount Validation | > 0 Check                 | ✅     | ✅   |
| **PI1.05** | Error Handling    | ParseError Types          | ✅     | ✅   |

#### 1.4.2 Saga Composer (`peer/saga_composer.rs`)

| ID         | Milestone       | Beschreibung             | Status | Test |
| ---------- | --------------- | ------------------------ | ------ | ---- |
| **PS1.01** | SagaComposer    | Intent → Saga Resolution | ✅     | ✅   |
| **PS1.02** | Transfer Saga   | Multi-Step Composition   | ✅     | ✅   |
| **PS1.03** | Delegation Saga | Authorization Steps      | ✅     | ✅   |
| **PS1.04** | Cost Estimation | < 5% Constraint (Κ23)    | ✅     | ✅   |
| **PS1.05** | Realm Crossing  | Crossing Detection       | ✅     | ✅   |

#### 1.4.3 Gateway Guard (`peer/gateway.rs`)

| ID         | Milestone            | Beschreibung              | Status | Test |
| ---------- | -------------------- | ------------------------- | ------ | ---- |
| **PG1.01** | GatewayGuard         | Realm-Crossing Validation | ✅     | ✅   |
| **PG1.02** | Trust Check          | min_trust Threshold       | ✅     | ✅   |
| **PG1.03** | Credential Check     | Required Credentials      | ✅     | ✅   |
| **PG1.04** | Trust Dampening      | Matrix Application        | ✅     | ✅   |
| **PG1.05** | Predicate Evaluation | Gateway Rules             | ✅     | ✅   |

---

### 1.5 API Layer – IN PROGRESS 🔧

#### 1.5.1 Proto Definitions

| ID          | Proto           | Status | Services                                                    |
| ----------- | --------------- | ------ | ----------------------------------------------------------- |
| **API1.01** | `peer.proto`    | ✅     | PeerService, IntentService, SagaService, EnvironmentService |
| **API1.02** | `storage.proto` | ✅     | StorageService                                              |
| **API1.03** | `health.proto`  | ✅     | HealthService                                               |
| **API1.04** | `info.proto`    | ✅     | InfoService                                                 |
| **API1.05** | `user.proto`    | ✅     | UserService                                                 |

#### 1.5.2 Service Implementation

| ID          | Service                       | Proto | Rust Handler | Status  |
| ----------- | ----------------------------- | ----- | ------------ | ------- |
| **SVC1.01** | PeerService.GetStatus         | ✅    | 📋           | Pending |
| **SVC1.02** | PeerService.GetInfo           | ✅    | 📋           | Pending |
| **SVC1.03** | PeerService.DeriveKey         | ✅    | 📋           | Pending |
| **SVC1.04** | PeerService.EvaluateGateway   | ✅    | 📋           | Pending |
| **SVC1.05** | IntentService.SubmitIntent    | ✅    | 📋           | Pending |
| **SVC1.06** | IntentService.ResolveIntent   | ✅    | 📋           | Pending |
| **SVC1.07** | IntentService.SimulateIntent  | ✅    | 📋           | Pending |
| **SVC1.08** | SagaService.ListSagas         | ✅    | 📋           | Pending |
| **SVC1.09** | SagaService.GetSagaStatus     | ✅    | 📋           | Pending |
| **SVC1.10** | SagaService.ExecuteSaga       | ✅    | 📋           | Pending |
| **SVC1.11** | SagaService.StreamSagaUpdates | ✅    | 📋           | Pending |
| **SVC1.12** | EnvironmentService.\*         | ✅    | 📋           | Pending |

#### 1.5.3 Developer Platform

| ID         | Milestone     | Beschreibung                        | Status |
| ---------- | ------------- | ----------------------------------- | ------ |
| **DP1.01** | erynoa-cli    | CLI-Tool (126 Befehle spezifiziert) | 📋     |
| **DP1.02** | erynoa-sdk-rs | Rust SDK                            | 📋     |
| **DP1.03** | erynoa-sdk-ts | TypeScript SDK (WASM)               | 📋     |
| **DP1.04** | Local Devnet  | Single-Node Test Environment        | 📋     |

---

## Phase 2: Protocol Implementation (2027)

> **Ziel:** Persistenz, Chain-Integration, TAT-Lifecycle, Credentials
> **Dauer:** 12-15 Monate

### 2.1 Persistence Layer

| ID         | Milestone                 | Beschreibung         | Status |
| ---------- | ------------------------- | -------------------- | ------ |
| **PL2.01** | Event Store (PostgreSQL)  | DAG-Persistenz       | 📋     |
| **PL2.02** | Trust Store (PostgreSQL)  | Trust-Snapshots      | 📋     |
| **PL2.03** | Identity Store            | DID Registry         | 📋     |
| **PL2.04** | Realm Hierarchy Store     | Environment-Struktur | 📋     |
| **PL2.05** | Cache Layer (DragonflyDB) | Hot Trust Values     | 📋     |

### 2.2 Chain Integration

| ID         | Milestone           | Beschreibung           | Status |
| ---------- | ------------------- | ---------------------- | ------ |
| **CI2.01** | Erynoa DAG          | Native Event Storage   | 📋     |
| **CI2.02** | IOTA/Shimmer Anchor | External Finality      | 📋     |
| **CI2.03** | Ethereum Bridge     | HTLC für ERC-20        | 📋     |
| **CI2.04** | Polygon Bridge      | L2 Integration         | 📋     |
| **CI2.05** | Key Vault (BIP44)   | Multi-Chain Derivation | 📋     |

### 2.3 TAT-Lifecycle

| ID          | Milestone     | Beschreibung                     | Status |
| ----------- | ------------- | -------------------------------- | ------ |
| **TAT2.01** | SEEK Phase    | Discovery mit Trust-Ranking      | 📋     |
| **TAT2.02** | PROPOSE Phase | Signiertes Angebot               | 📋     |
| **TAT2.03** | AGREE Phase   | Matching, Escrow-Setup           | 📋     |
| **TAT2.04** | STREAM Phase  | Kontinuierliche Mikro-Payments   | 📋     |
| **TAT2.05** | CLOSE Phase   | Finale Attestation, Trust-Update | 📋     |
| **TAT2.06** | ABORT Phase   | Proportionale Erstattung         | 📋     |
| **TAT2.07** | DISPUTE Phase | Schiedsverfahren                 | 📋     |

### 2.4 Credential System

| ID         | Milestone            | Beschreibung              | Status |
| ---------- | -------------------- | ------------------------- | ------ |
| **CR2.01** | VC Issuance          | W3C-konforme Credentials  | 📋     |
| **CR2.02** | VC Verification      | Multi-Chain Anchor Check  | 📋     |
| **CR2.03** | HumanAuth Credential | Mensch-Verifizierung (H1) | 📋     |
| **CR2.04** | Revocation           | Widerruf mit Anchor Proof | 📋     |

---

## Phase 3: Robustness & Humanismus (Q1-Q2 2028)

> **Ziel:** Security Hardening, Circuit Breakers, H1-H4 Axiome
> **Dauer:** 6-9 Monate

### 3.1 Circuit Breakers

| ID         | Milestone              | Beschreibung                  | Status |
| ---------- | ---------------------- | ----------------------------- | ------ |
| **CB3.01** | Trust Velocity Limiter | Max ±10% pro Stunde           | 📋     |
| **CB3.02** | Volatility Monitor     | Abort-Rate Überwachung        | 📋     |
| **CB3.03** | Automatic Cooldown     | 10min Freeze bei Kritisch     | 📋     |
| **CB3.04** | Dampening              | Glättung schneller Änderungen | 📋     |

### 3.2 Human Alignment (H1-H4)

| ID        | Axiom                       | Beschreibung                     | Status                |
| --------- | --------------------------- | -------------------------------- | --------------------- |
| **H3.01** | H1: Human-Alignment         | H(s) = 2.0/1.5/1.0 Multiplikator | ✅ (formula.rs)       |
| **H3.02** | H2: Verhältnismäßigkeit     | Cost ≤ 5% of Value               | ✅ (saga_composer.rs) |
| **H3.03** | H3: Temporale Gnade         | Asymmetric Decay                 | ✅ (world_formula.rs) |
| **H3.04** | H4: Semantische Verankerung | NLD + FormalSpec                 | 📋                    |

### 3.3 Security Hardening

| ID         | Milestone              | Beschreibung             | Status |
| ---------- | ---------------------- | ------------------------ | ------ |
| **SH3.01** | Security Audit Phase 1 | Domain + Core Review     | 📋     |
| **SH3.02** | Security Audit Phase 2 | Protection + Peer Review | 📋     |
| **SH3.03** | Penetration Testing    | Full Stack               | 📋     |
| **SH3.04** | Bug Bounty (Private)   | Closed Beta              | 📋     |

### 3.4 Post-Quantum Readiness

| ID         | Milestone         | Beschreibung                       | Status |
| ---------- | ----------------- | ---------------------------------- | ------ |
| **PQ3.01** | Hybrid Signatures | Ed25519 + Dilithium-3              | 📋     |
| **PQ3.02** | Key Rotation      | Trust-erhaltende Migration         | 📋     |
| **PQ3.03** | Crypto Agility    | Algorithmus-Wechsel ohne Hard Fork | 📋     |

---

## Phase 4: Network Launch (2028+)

> **Ziel:** Testnet, Piloten, Mainnet Launch
> **Dauer:** Ongoing

### 4.1 Testnet

| ID         | Milestone            | Beschreibung         | Status |
| ---------- | -------------------- | -------------------- | ------ |
| **TN4.01** | Testnet Alpha        | Private, 10-20 Nodes | 📋     |
| **TN4.02** | Testnet Beta         | Public, 50+ Nodes    | 📋     |
| **TN4.03** | Incentivized Testnet | Rewards              | 📋     |

### 4.2 Pilot: EV-Charging

| ID         | Milestone                 | Beschreibung        | Status |
| ---------- | ------------------------- | ------------------- | ------ |
| **EV4.01** | OCPP Bridge               | OCPP 2.0.1 ↔ Erynoa | 📋     |
| **EV4.02** | 5 Operators, 100 Chargers | Onboarding          | 📋     |
| **EV4.03** | 500+ Vehicle Agents       | Mobile App          | 📋     |
| **EV4.04** | 1000+ Charging Sessions   | Live Test           | 📋     |

### 4.3 Mainnet

| ID         | Milestone           | Beschreibung     | Status |
| ---------- | ------------------- | ---------------- | ------ |
| **MN4.01** | Genesis Preparation | Validator Setup  | 📋     |
| **MN4.02** | Mainnet Launch      | Go-Live          | 📋     |
| **MN4.03** | 50+ Validators      | Decentralization | 📋     |

---

## Success Metrics

### Technical KPIs (aktuelle Implementation)

| Modul      | Metric         | Target        | Aktuell      |
| ---------- | -------------- | ------------- | ------------ |
| Domain     | Unit Tests     | 100% pass     | ✅ 23/23     |
| Core       | Unit Tests     | 100% pass     | ✅ 23/23     |
| Protection | Unit Tests     | 100% pass     | ✅ 17/17     |
| Peer       | Unit Tests     | 100% pass     | ✅ 14/14     |
| **Gesamt** | **Unit Tests** | **100% pass** | **✅ 73/73** |

### Technical KPIs (Ziel)

| Phase   | Metric                     | Target       |
| ------- | -------------------------- | ------------ |
| Phase 1 | DID Resolution             | < 50ms (p95) |
| Phase 1 | Trust Calculation          | < 10ms       |
| Phase 2 | TAT Full Cycle             | < 10s        |
| Phase 2 | Blueprint Validation       | < 100ms      |
| Phase 3 | Circuit Breaker Activation | < 100ms      |
| Phase 4 | Testnet Uptime             | > 99.5%      |
| Phase 4 | Mainnet Uptime             | > 99.9%      |

### Business KPIs

| Phase   | Metric                 | Target               |
| ------- | ---------------------- | -------------------- |
| Phase 1 | Core Logic Complete    | ✅                   |
| Phase 1 | Proto Definitions      | ✅                   |
| Phase 2 | SDKs Released          | 3 (Rust, TS, Python) |
| Phase 3 | Security Audits Passed | 3                    |
| Phase 4 | Active DIDs (Year 1)   | 50.000+              |

---

## Axiom Implementation Tracking

### Kern-Axiome Κ1-Κ28

| Axiom | Name                    | Modul                              | Status |
| ----- | ----------------------- | ---------------------------------- | ------ |
| Κ1    | DID-Identität           | `domain/did.rs`                    | ✅     |
| Κ2    | Trust-Vektor 6D         | `domain/trust.rs`                  | ✅     |
| Κ3    | Event-Kausalität        | `domain/event.rs`                  | ✅     |
| Κ4    | Self-Attestation-Verbot | `core/trust_engine.rs`             | ✅     |
| Κ5    | Realm-Hierarchie        | `domain/realm.rs`                  | ✅     |
| Κ6    | Trust-Kombination       | `domain/trust.rs`                  | ✅     |
| Κ7    | Chain-Trust             | `domain/trust.rs`                  | ✅     |
| Κ8    | Asymmetric Update       | `core/trust_engine.rs`             | ✅     |
| Κ9    | Surprisal               | `core/surprisal.rs`                | ✅     |
| Κ10   | World Formula           | `core/world_formula.rs`            | ✅     |
| Κ11   | Human Factor            | `domain/formula.rs`                | ✅     |
| Κ12   | Temporal Decay          | `core/world_formula.rs`            | ✅     |
| Κ13   | Activity Presence       | `domain/formula.rs`                | ✅     |
| Κ14   | Sigmoid Normalization   | `domain/formula.rs`                | ✅     |
| Κ15   | Consensus Finality      | `core/consensus.rs`                | ✅     |
| Κ16   | Witness Requirement     | `core/consensus.rs`                | ✅     |
| Κ17   | Revert Probability      | `core/consensus.rs`                | ✅     |
| Κ18   | Event-Engine            | `core/event_engine.rs`             | ✅     |
| Κ19   | Anti-Calcification      | `protection/anti_calcification.rs` | ✅     |
| Κ20   | Diversity               | `protection/diversity.rs`          | ✅     |
| Κ21   | Quadratic Voting        | `protection/quadratic.rs`          | ✅     |
| Κ22   | Intent-Parsing          | `peer/intent_parser.rs`            | ✅     |
| Κ23   | Cost-Constraint (5%)    | `peer/saga_composer.rs`            | ✅     |
| Κ24   | Gateway-Predicates      | `peer/gateway.rs`                  | ✅     |
| Κ25   | Trust-Dampening         | `peer/gateway.rs`                  | ✅     |
| Κ26   | Anomaly-Detection       | `protection/anomaly.rs`            | ✅     |
| Κ27   | Gini-Coefficient        | `protection/anti_calcification.rs` | ✅     |
| Κ28   | Power-Cap               | `protection/anti_calcification.rs` | ✅     |

### Peer-Axiome PR1-PR6

| Axiom | Name                    | Modul                                            | Status   |
| ----- | ----------------------- | ------------------------------------------------ | -------- |
| PR1   | Intent-Auflösung        | `peer/intent_parser.rs`, `peer/saga_composer.rs` | ✅       |
| PR2   | Saga-Atomarität         | `domain/saga.rs`, `peer/saga_composer.rs`        | ✅       |
| PR3   | Gateway-Vollständigkeit | `peer/gateway.rs`                                | ✅       |
| PR4   | Funktor-Eigenschaften   | `domain/trust.rs` (dampening matrix)             | ✅       |
| PR5   | Schlüssel-Isolation     | `peer.proto` (DeriveKey)                         | ✅ Proto |
| PR6   | Trust-Dämpfung          | `peer/gateway.rs`                                | ✅       |

---

## Risk Matrix

| Risk                     | Wahrscheinlichkeit | Impact    | Mitigation                                       |
| ------------------------ | ------------------ | --------- | ------------------------------------------------ |
| IOTA Rebased Verzögerung | 🟡 Mittel          | 🔴 Hoch   | Alternative L1, modularer Ansatz                 |
| Trust Gaming             | 🟢 Niedrig         | 🔴 Hoch   | ✅ Implementiert: Asymmetrie, Power-Cap, Anomaly |
| Complexity Overload      | 🟡 Mittel          | 🟡 Mittel | ✅ 73 Tests, klare Schichtung                    |
| Regulatory Changes       | 🟡 Mittel          | 🟡 Mittel | Legal Advisory, Compliance-First                 |

---

## Verwandte Dokumente

| Dokument                                                | Beschreibung                           |
| ------------------------------------------------------- | -------------------------------------- |
| [FACHKONZEPT.md](./concept-v3/FACHKONZEPT.md)           | Vollständiges technisches Konzept V6.2 |
| [CLI-REFERENCE.md](./concept-v3/CLI-REFERENCE.md)       | CLI-Befehle (126 Commands)             |
| [WORLD-FORMULA.md](./concept-v3/WORLD-FORMULA.md)       | Systemgleichung, Axiome                |
| [LOGIC.md](./concept-v3/LOGIC.md)                       | Formale Logik, Beweise                 |
| [CONSTITUTION.md](./concept-v3/CONSTITUTION.md)         | Humanistische Verfassung (H1-H4)       |
| [ROBUSTNESS-LAYER.md](./concept-v3/ROBUSTNESS-LAYER.md) | Antifragilitäts-Architektur            |
| [SDK-ARCHITECTURE.md](./concept-v3/SDK-ARCHITECTURE.md) | SDK-Spezifikation                      |
| [PROTOCOL.md](./concept-v3/PROTOCOL.md)                 | Protokoll-Details                      |
| [peer.proto](../backend/proto/erynoa/v1/peer.proto)     | gRPC API Definition                    |

---

<div align="center">

**Erynoa V4.1 – Probabilistisches Protokoll für vertrauensbasierte Interaktionen**

_„Intelligenz im Dienste des Lebens."_

```
┌─────────────────────────────────────────────────────────────────────┐
│  Peer Layer         (IntentParser, SagaComposer, GatewayGuard) ✅   │
├─────────────────────────────────────────────────────────────────────┤
│  Protection Layer   (AntiCalcification, Diversity, Anomaly) ✅      │
├─────────────────────────────────────────────────────────────────────┤
│  Core Logic Layer   (EventEngine, TrustEngine, Consensus) ✅        │
├─────────────────────────────────────────────────────────────────────┤
│  Domain Layer       (DID, Trust6D, Event, Realm, Saga, Formula) ✅  │
└─────────────────────────────────────────────────────────────────────┘
         │                                    │
         └──────────── FEEDBACK LOOP ◀────────┘
```

**126 Axiome · 4 Schichten · 73 Tests ✅ · Klassische Wahrscheinlichkeitstheorie**

</div>
