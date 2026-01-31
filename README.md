<div align="center">

# Erynoa

**Probabilistisches Protokoll für vertrauensbasierte Interaktionen**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/Tests-109%20passed-brightgreen?style=flat-square)](backend/src/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0-FF3E00?style=flat-square&logo=svelte)](https://kit.svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![Nix](https://img.shields.io/badge/Nix-Flakes-5277C3?style=flat-square&logo=nixos)](https://nixos.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Axioms](https://img.shields.io/badge/Axioms-28-blueviolet?style=flat-square)](documentation/concept-v4/FACHKONZEPT.md)

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║   𝔼 = Σ A(s) · σ( W(s) · ln|C(s)| · N(s) / E(s) ) · H(s) · w(s,t)             ║
║       s                                                                       ║
║                                                                               ║
║   "Intelligenz im Dienste des Lebens."                                        ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

[Schnellstart](#-schnellstart) · [Architektur](#-architektur) · [Status](#-implementation-status) · [Dokumentation](#-dokumentation)

</div>

---

## 🧠 Was ist Erynoa?

Erynoa ist ein **probabilistisches kybernetisches Protokoll** für vertrauensbasierte Interaktionen zwischen Menschen, Organisationen und KI-Agenten. Es basiert auf **28 formal definierten Axiomen (Κ1-Κ28)** und einer mathematisch fundierten **Systemgleichung (Weltformel)**.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│                         ERYNOA V4.1 – OVERVIEW                                  │
│                                                                                 │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │  Peer Layer         (IntentParser, SagaComposer, GatewayGuard) ✅       │   │
│   ├─────────────────────────────────────────────────────────────────────────┤   │
│   │  Protection Layer   (AntiCalcification, Diversity, Anomaly) ✅          │   │
│   ├─────────────────────────────────────────────────────────────────────────┤   │
│   │  Core Logic Layer   (EventEngine, TrustEngine, Consensus) ✅            │   │
│   ├─────────────────────────────────────────────────────────────────────────┤   │
│   │  Domain Layer       (DID, Trust6D, Event, Realm, Saga, Formula) ✅      │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
│            │                                    │                               │
│            └──────────── FEEDBACK LOOP ◀────────┘                               │
│                                                                                 │
│   KERN-FEATURES:                                                                │
│   ✅ 6D Trust-Vektor (R,I,C,P,V,Ω)     ✅ Bayessche Trust-Evolution            │
│   ✅ Event-DAG mit Finalität           ✅ Anti-Calcification (Gini, Power-Cap) │
│   ✅ Human-Alignment (H = 2.0/1.5/1.0) ✅ Quadratic Voting                     │
│   ✅ Intent → Saga Resolution          ✅ Gateway Trust-Dampening              │
│   ✅ Realm-Hierarchie (Root/Virtual)   ✅ Anomaly Detection                    │
│                                                                                 │
│   109 TESTS ✅ · 28 AXIOME · 4 SCHICHTEN · DEZENTRALE ARCHITEKTUR             │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Die Systemgleichung

| Symbol     | Bedeutung                           | Implementation          |
| ---------- | ----------------------------------- | ----------------------- |
| **𝔼**      | Systemwert (kollektive Intelligenz) | `core/world_formula.rs` |
| **A(s)**   | Aktivitätspräsenz [0,1]             | `domain/formula.rs`     |
| **W(s)**   | Wächter-Metrik 6D (R,I,C,P,V,Ω)     | `domain/trust.rs`       |
| **C(s)**   | Kausale Geschichte (Event-DAG)      | `domain/event.rs`       |
| **N(s)**   | Novelty-Score (Surprisal)           | `core/surprisal.rs`     |
| **E(s)**   | Erwartungswert                      | `core/surprisal.rs`     |
| **σ(x)**   | Sigmoid 1/(1+e^(-x))                | `domain/formula.rs`     |
| **H(s)**   | Human-Alignment (2.0\|1.5\|1.0)     | `domain/formula.rs`     |
| **w(s,t)** | Temporale Gewichtung                | `core/world_formula.rs` |

> 📖 **Mehr erfahren:** [Fachkonzept V6.2](documentation/concept-v4/FACHKONZEPT.md) · [CLI-Referenz](documentation/concept-v4/CLI-REFERENCE.md) · [Roadmap](documentation/ROADMAP.md)

---

## ⚡ Schnellstart

> **Voraussetzungen:** [Nix](https://nixos.org/) (für Frontend zusätzlich: [Docker Desktop](https://www.docker.com/products/docker-desktop/))

```bash
# 1. Repository klonen
git clone git@github.com:NiklasJavier/erynoa.git && cd erynoa

# 2. Nix Dev-Shell betreten
nix develop

# 3. Backend starten (keine externen Services nötig!)
cd backend && cargo run

# ODER: Vollständige Entwicklungsumgebung mit Frontend
just dev
```

**Backend Single-Binary** 🚀 → Keine externen Datenbanken nötig!

<details>
<summary><strong>🔗 Alle URLs</strong></summary>

| Service                 | URL                              |
| ----------------------- | -------------------------------- |
| **Backend API**         | <http://localhost:8000>          |
| **Hauptzugang (Proxy)** | <http://localhost:3001>          |
| Console                 | <http://localhost:3001/console>  |
| Platform                | <http://localhost:3001/platform> |
| Docs                    | <http://localhost:3001/docs>     |

**Auth:** DID-basiert mit Ed25519-Signaturen (kein externer Auth-Service nötig)

</details>

---

## 🏗 Architektur

### Dezentrale Speicherarchitektur (Fjall)

```
backend/src/local/              # 🗄️ Embedded Storage Layer
├── mod.rs                      # DecentralizedStorage Manager
├── kv_store.rs                 # Generic Type-Safe KV Store
├── identity_store.rs           # DID + Ed25519 Keypairs
├── event_store.rs              # Event-DAG mit Finality
├── trust_store.rs              # TrustVector6D per DID
└── content_store.rs            # Content-Addressable Storage (BLAKE3)

┌─────────────────────────────────────────────────────────────────────┐
│                    DEZENTRALE SPEICHER-ARCHITEKTUR                  │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌───────────────────────────────────────────────────────────┐     │
│   │              DecentralizedStorage (Manager)               │     │
│   │   • open(path) → Keyspace                                 │     │
│   │   • open_temporary() → Temp-Keyspace                      │     │
│   │   • flush() → persist(SyncAll)                            │     │
│   └───────────────┬───────────────────────────────────────────┘     │
│                   │                                                 │
│   ┌───────────────┼───────────────────────────────────────────┐     │
│   │               ▼  Fjall Keyspace (LSM-Tree)                │     │
│   │  ┌─────────────────────────────────────────────────────┐  │     │
│   │  │  identities   │  events  │  trust  │  content  │ kv │  │     │
│   │  │  (Partition)  │ (Part.)  │ (Part.) │  (Part.)  │(P.)│  │     │
│   │  └─────────────────────────────────────────────────────┘  │     │
│   └───────────────────────────────────────────────────────────┘     │
│                                                                     │
│   FEATURES:                                                         │
│   ✅ Embedded (keine externen Services)                             │
│   ✅ ACID mit Sync-Persistenz                                       │
│   ✅ LSM-Tree Architektur                                           │
│   ✅ Type-Safe mit serde                                            │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 4-Schichten Backend (Rust)

```
backend/src/
├── domain/                    # 🎯 Domain Layer (Κ1-Κ5)
│   ├── did.rs                 # DID:erynoa mit 10 Namespaces
│   ├── trust.rs               # TrustVector6D, Dampening Matrix
│   ├── event.rs               # Event-DAG, Finality Levels
│   ├── realm.rs               # Realm-Hierarchie (Root/Virtual/Partition)
│   ├── saga.rs                # Intent, Saga, SagaStep
│   └── formula.rs             # Weltformel-Komponenten
│
├── core/                      # ⚙️ Core Logic Layer (Κ6-Κ18)
│   ├── event_engine.rs        # DAG Storage, Parent-Validation
│   ├── trust_engine.rs        # Bayessche Updates, Self-Attestation-Verbot
│   ├── surprisal.rs           # Count-Min Sketch, -log₂(P)
│   ├── world_formula.rs       # 𝔼 = Σ contributions
│   └── consensus.rs           # Witness-basierte Finality
│
├── protection/                # 🛡️ Protection Layer (Κ19-Κ21, Κ26-Κ28)
│   ├── anti_calcification.rs  # Novelty-Bonus, Gini, Power-Cap
│   ├── diversity.rs           # Shannon-Entropie, Monoculture-Detection
│   ├── quadratic.rs           # √n Stimmen für n² Credits
│   └── anomaly.rs             # Velocity/Amount Alerts
│
├── peer/                      # 🌐 Peer Layer (PR1-PR6)
│   ├── intent_parser.rs       # Natural Language → Intent
│   ├── saga_composer.rs       # Intent → Saga (< 5% Cost)
│   └── gateway.rs             # Realm-Crossing, Trust-Dampening
│
└── api/                       # 🔌 API Layer (gRPC/Connect-RPC)
    └── ...                    # PeerService, IntentService, SagaService
```

### Proto-Services

```protobuf
// backend/proto/erynoa/v1/peer.proto

service PeerService {
  rpc GetStatus(...)           // Peer-Status
  rpc GetInfo(...)             // Capabilities, Config
  rpc DeriveKey(...)           // BIP44 Key Derivation
  rpc EvaluateGateway(...)     // Realm-Crossing Check
}

service IntentService {
  rpc SubmitIntent(...)        // Natural Language Goal
  rpc ResolveIntent(...)       // Intent → Saga
  rpc SimulateIntent(...)      // Dry-Run
}

service SagaService {
  rpc ExecuteSaga(...)         // HTLC Cross-Chain
  rpc StreamSagaUpdates(...)   // Real-time Progress
}

service EnvironmentService {
  rpc CreateEnvironment(...)   // Virtual Realm
  rpc JoinEnvironment(...)     // Membership
}
```

---

## 📊 Implementation Status

### Backend Module (109 Tests ✅)

| Schicht        | Module                                                            | Tests | Status |
| -------------- | ----------------------------------------------------------------- | ----- | ------ |
| **Domain**     | did, trust, event, realm, saga, formula                           | 23    | ✅     |
| **Core**       | event_engine, trust_engine, surprisal, world_formula, consensus   | 23    | ✅     |
| **Protection** | anti_calcification, diversity, quadratic, anomaly                 | 17    | ✅     |
| **Peer**       | intent_parser, saga_composer, gateway                             | 14    | ✅     |
| **Local**      | kv_store, identity_store, event_store, trust_store, content_store | 32    | ✅     |

### Axiom Coverage

| Kategorie    | Axiome     | Status   |
| ------------ | ---------- | -------- |
| Kern-Axiome  | Κ1-Κ28     | ✅ 28/28 |
| Peer-Axiome  | PR1-PR6    | ✅ 6/6   |
| API-Services | peer.proto | ✅ Proto |

### API Layer

| Service            | Proto | Handler | Status        |
| ------------------ | ----- | ------- | ------------- |
| PeerService        | ✅    | 📋      | Proto defined |
| IntentService      | ✅    | 📋      | Proto defined |
| SagaService        | ✅    | 📋      | Proto defined |
| EnvironmentService | ✅    | 📋      | Proto defined |
| StorageService     | ✅    | ✅      | Complete      |
| HealthService      | ✅    | ✅      | Complete      |

```bash
# Tests ausführen
cd backend && cargo test

# Ergebnis:
# running 94 tests (unit) + 13 tests (integration) + 2 tests (doc)
# test result: ok. 109 passed; 0 failed
```

---

## 🛠 Tech Stack

<table>
<tr>
<td width="50%">

### Backend (Dezentral)

| Komponente | Technologie             |
| ---------- | ----------------------- |
| Runtime    | **Rust**, Tokio         |
| Framework  | Axum                    |
| API        | Connect-RPC (Protobuf)  |
| Database   | **Fjall** (Embedded KV) |
| Auth       | **DID + Ed25519**       |
| Storage    | **CAS** (Content-Hash)  |
| Crypto     | ed25519-dalek, blake3   |

</td>
<td width="50%">

### Frontend

| Komponente      | Technologie              |
| --------------- | ------------------------ |
| Framework       | **SvelteKit** (Svelte 5) |
| Styling         | Tailwind CSS             |
| Build           | Vite, Turborepo          |
| Package Manager | pnpm                     |
| Linting         | Biome                    |
| Types           | TypeScript               |

</td>
</tr>
</table>

### Infrastructure

| Komponente       | Technologie                     |
| ---------------- | ------------------------------- |
| Dev Environment  | Nix Flakes                      |
| Containerization | Docker Compose (nur Frontend)   |
| Reverse Proxy    | Caddy                           |
| Task Runner      | just                            |
| Code Generation  | buf (Protobuf)                  |
| Backend Storage  | **Embedded** (keine Container!) |

---

## 📁 Projektstruktur

```
erynoa/
├── backend/                   # 🦀 Rust API Server
│   ├── src/
│   │   ├── domain/            # DID, Trust, Event, Realm, Saga
│   │   ├── core/              # Engines (Event, Trust, Surprisal, Consensus)
│   │   ├── protection/        # Anti-Gaming, Diversity, Anomaly
│   │   ├── peer/              # Intent, Saga, Gateway
│   │   ├── local/             # 🆕 Dezentrale Storage (Fjall)
│   │   │   ├── kv_store.rs        # Generic KV Store
│   │   │   ├── identity_store.rs  # DID + Ed25519
│   │   │   ├── event_store.rs     # Event-DAG
│   │   │   ├── trust_store.rs     # TrustVector6D
│   │   │   └── content_store.rs   # CAS (BLAKE3)
│   │   └── api/               # HTTP/gRPC Handlers
│   ├── proto/erynoa/v1/       # Protobuf Definitions
│   │   ├── peer.proto         # ⭐ Peer/Intent/Saga/Environment Services
│   │   ├── storage.proto
│   │   ├── health.proto
│   │   └── user.proto
│   ├── config/                # TOML Konfiguration
│   └── data/                  # Fjall Datenbank (gitignored)
│
├── frontend/                  # 🎨 SvelteKit Apps
│   ├── console/               # Admin Console
│   ├── platform/              # Main Platform
│   └── docs/                  # Documentation Site
│
├── documentation/             # 📖 Dokumentation
│   ├── ROADMAP.md             # ⭐ Strategic Roadmap V4.1
│   ├── concept-v4/            # ⭐ Aktuell: Fachkonzept V6.2
│   │   ├── FACHKONZEPT.md     # Vollständiges Konzept
│   │   ├── CLI-REFERENCE.md   # CLI Commands
│   │   ├── LOGIC.md           # Logik & Axiome
│   │   └── SYSTEM-ARCHITECTURE.md # System-Architektur
│   └── system/                # Setup, Guides, Reference
│
├── infra/                     # 🏗 Infrastructure
│   ├── docker/
│   ├── proxy/
│   └── auth/
│
├── flake.nix                  # Nix Dev Environment
├── justfile                   # Task Runner
├── buf.yaml                   # Protobuf Config
└── turbo.json                 # Turborepo Config
```

---

## 🔧 Befehle

### Entwicklung

| Befehl        | Beschreibung                                       |
| ------------- | -------------------------------------------------- |
| `just dev`    | **Startet alles** (Frontends + Backend + Services) |
| `just status` | Status aller Services                              |
| `just logs`   | Logs anzeigen                                      |
| `just stop`   | Alle Container stoppen                             |
| `just reset`  | Alles löschen und neu starten                      |

### Backend

| Befehl       | Beschreibung                |
| ------------ | --------------------------- |
| `just check` | Cargo check                 |
| `just lint`  | Clippy Linter               |
| `just fmt`   | Code formatieren            |
| `just test`  | Tests ausführen (109 Tests) |
| `just ci`    | fmt + lint + test           |

### Protobuf

| Befehl         | Beschreibung                    |
| -------------- | ------------------------------- |
| `buf lint`     | Proto-Dateien validieren        |
| `buf generate` | TypeScript-Code generieren      |
| `cargo build`  | Rust-Code generieren (build.rs) |

---

## 📖 Dokumentation

### Kern-Dokumente

| Dokument                                                                     | Beschreibung                      |
| ---------------------------------------------------------------------------- | --------------------------------- |
| **[📋 Fachkonzept V6.2](documentation/concept-v4/FACHKONZEPT.md)**           | Vollständiges technisches Konzept |
| **[🗺️ Roadmap V4.1](documentation/ROADMAP.md)**                              | Strategischer Entwicklungsplan    |
| **[💻 CLI-Referenz](documentation/concept-v4/CLI-REFERENCE.md)**             | CLI Commands                      |
| **[📐 Logik & Axiome](documentation/concept-v4/LOGIC.md)**                   | Κ1-Κ28 Axiome, Formalisierung     |
| **[🏗️ System-Architektur](documentation/concept-v4/SYSTEM-ARCHITECTURE.md)** | V4.1 Architektur                  |

### System-Dokumentation

| Dokument                                                       | Beschreibung         |
| -------------------------------------------------------------- | -------------------- |
| [Setup Guide](documentation/system/setup/setup.md)             | Entwicklungsumgebung |
| [Essential Guide](documentation/system/essential_guide.md)     | Troubleshooting      |
| [Architecture](documentation/system/reference/architecture.md) | System-Architektur   |
| [Style Guide](documentation/system/development/style-guide.md) | Code-Stil            |

### Axiom-Übersicht

<details>
<summary><strong>Κ1-Κ28 Kern-Axiome</strong></summary>

| Axiom | Name                    | Modul                              |
| ----- | ----------------------- | ---------------------------------- |
| Κ1    | DID-Identität           | `domain/did.rs`                    |
| Κ2    | Trust-Vektor 6D         | `domain/trust.rs`                  |
| Κ3    | Event-Kausalität        | `domain/event.rs`                  |
| Κ4    | Self-Attestation-Verbot | `core/trust_engine.rs`             |
| Κ5    | Realm-Hierarchie        | `domain/realm.rs`                  |
| Κ6    | Trust-Kombination       | `domain/trust.rs`                  |
| Κ7    | Chain-Trust             | `domain/trust.rs`                  |
| Κ8    | Asymmetric Update       | `core/trust_engine.rs`             |
| Κ9    | Surprisal               | `core/surprisal.rs`                |
| Κ10   | World Formula           | `core/world_formula.rs`            |
| Κ11   | Human Factor            | `domain/formula.rs`                |
| Κ12   | Temporal Decay          | `core/world_formula.rs`            |
| Κ13   | Activity Presence       | `domain/formula.rs`                |
| Κ14   | Sigmoid Normalization   | `domain/formula.rs`                |
| Κ15   | Consensus Finality      | `core/consensus.rs`                |
| Κ16   | Witness Requirement     | `core/consensus.rs`                |
| Κ17   | Revert Probability      | `core/consensus.rs`                |
| Κ18   | Event-Engine            | `core/event_engine.rs`             |
| Κ19   | Anti-Calcification      | `protection/anti_calcification.rs` |
| Κ20   | Diversity               | `protection/diversity.rs`          |
| Κ21   | Quadratic Voting        | `protection/quadratic.rs`          |
| Κ22   | Intent-Parsing          | `peer/intent_parser.rs`            |
| Κ23   | Cost-Constraint (5%)    | `peer/saga_composer.rs`            |
| Κ24   | Gateway-Predicates      | `peer/gateway.rs`                  |
| Κ25   | Trust-Dampening         | `peer/gateway.rs`                  |
| Κ26   | Anomaly-Detection       | `protection/anomaly.rs`            |
| Κ27   | Gini-Coefficient        | `protection/anti_calcification.rs` |
| Κ28   | Power-Cap               | `protection/anti_calcification.rs` |

</details>

<details>
<summary><strong>PR1-PR6 Peer-Axiome</strong></summary>

| Axiom | Name                    | Modul                                            |
| ----- | ----------------------- | ------------------------------------------------ |
| PR1   | Intent-Auflösung        | `peer/intent_parser.rs`, `peer/saga_composer.rs` |
| PR2   | Saga-Atomarität         | `domain/saga.rs`                                 |
| PR3   | Gateway-Vollständigkeit | `peer/gateway.rs`                                |
| PR4   | Funktor-Eigenschaften   | `domain/trust.rs`                                |
| PR5   | Schlüssel-Isolation     | `peer.proto`                                     |
| PR6   | Trust-Dämpfung          | `peer/gateway.rs`                                |

</details>

---

## 🤝 Contributing

1. Prüfe [Roadmap](documentation/ROADMAP.md) für offene Aufgaben
2. Folge dem [Style Guide](documentation/system/development/style-guide.md)
3. Schreibe Tests ([Testing Guide](documentation/system/development/testing.md))
4. Validiere Axiom-Konsistenz

---

<div align="center">

**[MIT License](LICENSE)**

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
```

**28 Axiome (Κ1-Κ28) · 4 Schichten · 73 Tests ✅**

_„Intelligenz im Dienste des Lebens."_

</div>
