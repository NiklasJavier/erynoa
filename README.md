<div align="center">

# Erynoa

**Probabilistisches Protokoll für vertrauensbasierte Interaktionen**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/Tests-73%20passed-brightgreen?style=flat-square)](backend/src/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0-FF3E00?style=flat-square&logo=svelte)](https://kit.svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![Nix](https://img.shields.io/badge/Nix-Flakes-5277C3?style=flat-square&logo=nixos)](https://nixos.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Axioms](https://img.shields.io/badge/Axioms-126-blueviolet?style=flat-square)](documentation/concept-v3/FACHKONZEPT.md)

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║   𝔼 = Σ A(s) · σ( W(s) · ln|C(s)| · N(s) / E(s) ) · H(s) · w(s,t)           ║
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

Erynoa ist ein **probabilistisches kybernetisches Protokoll** für vertrauensbasierte Interaktionen zwischen Menschen, Organisationen und KI-Agenten. Es basiert auf **126 formal definierten Axiomen** und einer mathematisch fundierten **Systemgleichung (Weltformel)**.

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
│   73 TESTS ✅ · 126 AXIOME · 4 SCHICHTEN · KLASSISCHE WAHRSCHEINLICHKEIT       │
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

> 📖 **Mehr erfahren:** [Fachkonzept V6.2](documentation/concept-v3/FACHKONZEPT.md) · [CLI-Referenz](documentation/concept-v3/CLI-REFERENCE.md) · [Roadmap](documentation/ROADMAP.md)

---

## ⚡ Schnellstart

> **Voraussetzungen:** [Nix](https://nixos.org/) und [Docker Desktop](https://www.docker.com/products/docker-desktop/)

```bash
# 1. Repository klonen
git clone git@github.com:NiklasJavier/erynoa.git && cd erynoa

# 2. Nix Dev-Shell betreten
nix develop

# 3. Projekt starten
just dev
```

**Warte ~2 Minuten** ⏳ → Öffne **<http://localhost:3001>**

<details>
<summary><strong>🔗 Alle URLs & Test-Login</strong></summary>

| Service                 | URL                              |
| ----------------------- | -------------------------------- |
| **Hauptzugang (Proxy)** | <http://localhost:3001>          |
| Console                 | <http://localhost:3001/console>  |
| Platform                | <http://localhost:3001/platform> |
| Docs                    | <http://localhost:3001/docs>     |
| Backend API             | <http://localhost:3001/api>      |
| ZITADEL (Auth)          | <http://localhost:8080>          |
| MinIO (Storage)         | <http://localhost:9001>          |

**Test-Login:**

- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

</details>

---

## 🏗 Architektur

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

### Backend Module (73 Tests ✅)

| Schicht        | Module                                                          | Tests | Status |
| -------------- | --------------------------------------------------------------- | ----- | ------ |
| **Domain**     | did, trust, event, realm, saga, formula                         | 23    | ✅     |
| **Core**       | event_engine, trust_engine, surprisal, world_formula, consensus | 23    | ✅     |
| **Protection** | anti_calcification, diversity, quadratic, anomaly               | 17    | ✅     |
| **Peer**       | intent_parser, saga_composer, gateway                           | 14    | ✅     |

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
# running 73 tests
# test result: ok. 73 passed; 0 failed
```

---

## 🛠 Tech Stack

<table>
<tr>
<td width="50%">

### Backend

| Komponente | Technologie            |
| ---------- | ---------------------- |
| Runtime    | **Rust**, Tokio        |
| Framework  | Axum                   |
| API        | Connect-RPC (Protobuf) |
| Database   | PostgreSQL (OrioleDB)  |
| Cache      | DragonflyDB (Redis)    |
| Storage    | MinIO (S3)             |
| Auth       | ZITADEL (OIDC/JWT)     |

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

| Komponente       | Technologie    |
| ---------------- | -------------- |
| Dev Environment  | Nix Flakes     |
| Containerization | Docker Compose |
| Reverse Proxy    | Caddy          |
| Task Runner      | just           |
| Code Generation  | buf (Protobuf) |

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
│   │   └── api/               # HTTP/gRPC Handlers
│   ├── proto/erynoa/v1/       # Protobuf Definitions
│   │   ├── peer.proto         # ⭐ Peer/Intent/Saga/Environment Services
│   │   ├── storage.proto
│   │   ├── health.proto
│   │   └── user.proto
│   ├── config/                # TOML Konfiguration
│   └── migrations/            # SQL Migrations
│
├── frontend/                  # 🎨 SvelteKit Apps
│   ├── console/               # Admin Console
│   ├── platform/              # Main Platform
│   └── docs/                  # Documentation Site
│
├── documentation/             # 📖 Dokumentation
│   ├── ROADMAP.md             # ⭐ Strategic Roadmap V4.1
│   ├── concept-v3/            # ⭐ Aktuell: Fachkonzept V6.2
│   │   ├── FACHKONZEPT.md     # Vollständiges Konzept
│   │   ├── CLI-REFERENCE.md   # 126 CLI Commands
│   │   ├── WORLD-FORMULA.md   # Systemgleichung
│   │   └── PROTOCOL.md        # Protokoll-Details
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

| Befehl       | Beschreibung               |
| ------------ | -------------------------- |
| `just check` | Cargo check                |
| `just lint`  | Clippy Linter              |
| `just fmt`   | Code formatieren           |
| `just test`  | Tests ausführen (73 Tests) |
| `just ci`    | fmt + lint + test          |

### Protobuf

| Befehl         | Beschreibung                    |
| -------------- | ------------------------------- |
| `buf lint`     | Proto-Dateien validieren        |
| `buf generate` | TypeScript-Code generieren      |
| `cargo build`  | Rust-Code generieren (build.rs) |

---

## 📖 Dokumentation

### Kern-Dokumente

| Dokument                                                           | Beschreibung                      |
| ------------------------------------------------------------------ | --------------------------------- |
| **[📋 Fachkonzept V6.2](documentation/concept-v3/FACHKONZEPT.md)** | Vollständiges technisches Konzept |
| **[🗺️ Roadmap V4.1](documentation/ROADMAP.md)**                    | Strategischer Entwicklungsplan    |
| **[💻 CLI-Referenz](documentation/concept-v3/CLI-REFERENCE.md)**   | 126 CLI Commands                  |
| **[🔢 Weltformel](documentation/concept-v3/WORLD-FORMULA.md)**     | Systemgleichung, Axiome           |
| **[⚖️ Verfassung](documentation/concept-v3/CONSTITUTION.md)**      | Human-Alignment (H1-H4)           |

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

**126 Axiome · 4 Schichten · 73 Tests ✅**

_„Intelligenz im Dienste des Lebens."_

</div>
