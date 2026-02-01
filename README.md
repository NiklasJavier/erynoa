<div align="center">

# Erynoa

**Dezentrales Protokoll für vertrauensbasierte Zusammenarbeit**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/Tests-409%20passed-brightgreen?style=flat-square)](backend/src/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0-FF3E00?style=flat-square&logo=svelte)](https://kit.svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![libp2p](https://img.shields.io/badge/libp2p-0.54-blue?style=flat-square)](https://libp2p.io/)
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

Erynoa ist ein **dezentrales kybernetisches Protokoll** für vertrauensbasierte Interaktionen zwischen Menschen, Organisationen und KI-Agenten. Es basiert auf **28 formal definierten Axiomen (Κ1-Κ28)**, einem **Unified Data Model (UDM)** und dem **Integrated Processing System (IPS)**.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│                         ERYNOA V4.1 – PRODUCTION READY                          │
│                                                                                 │
│   ┌─────────────────────────────────────────────────────────────────────────┐   │
│   │  P2P Layer          (libp2p, Gossipsub, NAT-Traversal, Kademlia) ✅     │   │
│   ├─────────────────────────────────────────────────────────────────────────┤   │
│   │  Peer Layer         (IntentParser, SagaComposer, GatewayGuard) ✅       │   │
│   ├─────────────────────────────────────────────────────────────────────────┤   │
│   │  Protection Layer   (AntiCalcification, Diversity, AdaptiveCalib) ✅    │   │
│   ├─────────────────────────────────────────────────────────────────────────┤   │
│   │  Core Logic Layer   (EventEngine, TrustEngine, WorldFormula) ✅         │   │
│   ├─────────────────────────────────────────────────────────────────────────┤   │
│   │  Domain Layer       (UDM: UniversalId, Event, Trust6D, Saga) ✅         │   │
│   ├─────────────────────────────────────────────────────────────────────────┤   │
│   │  Storage Layer      (Fjall KV, Cold Archive, Merkle-Proofs) ✅          │   │
│   └─────────────────────────────────────────────────────────────────────────┘   │
│            │                                    │                               │
│            └──────────── FEEDBACK LOOP ◀────────┘                               │
│                                                                                 │
│   KERN-FEATURES:                                                                │
│   ✅ 6D Trust-Vektor (R,I,C,P,V,Ω)     ✅ Bayessche Trust-Evolution            │
│   ✅ Event-DAG mit Finalität           ✅ Anti-Calcification + PID-Controller  │
│   ✅ Human-Alignment (H = 2.0/1.5/1.0) ✅ libp2p mit NAT-Traversal             │
│   ✅ Intent → Saga Resolution          ✅ Cold Storage mit Merkle-Proofs       │
│   ✅ Realm-Hierarchie (Root/Virtual)   ✅ Blueprint Marketplace                │
│                                                                                 │
│   409 TESTS ✅ · 28 AXIOME · 6 SCHICHTEN · DEZENTRALE ARCHITEKTUR             │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Die Weltformel (Systemgleichung)

| Symbol     | Bedeutung                           | Implementation              |
| ---------- | ----------------------------------- | --------------------------- |
| **𝔼**      | Systemwert (kollektive Intelligenz) | `core/world_formula.rs`     |
| **A(s)**   | Aktivitätspräsenz [0,1]             | `domain/unified/formula.rs` |
| **W(s)**   | Wächter-Metrik 6D (R,I,C,P,V,Ω)     | `domain/unified/trust.rs`   |
| **C(s)**   | Kausale Geschichte (Event-DAG)      | `domain/unified/event.rs`   |
| **N(s)**   | Novelty-Score (Surprisal)           | `core/surprisal.rs`         |
| **E(s)**   | Erwartungswert                      | `core/surprisal.rs`         |
| **σ(x)**   | Sigmoid 1/(1+e^(-x))                | `domain/unified/formula.rs` |
| **H(s)**   | Human-Alignment (2.0\|1.5\|1.0)     | `domain/unified/formula.rs` |
| **w(s,t)** | Temporale Gewichtung                | `core/world_formula.rs`     |

> 📖 **Mehr erfahren:** [Fachkonzept V6.2](documentation/concept-v4/FACHKONZEPT.md) · [IPS Logik-Modell](documentation/system/development/IPS-01-imp.md) · [Unified Data Model](documentation/system/development/UNIFIED-DATA-MODEL.md)

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

### 6-Schichten Backend (Rust)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ERYNOA BACKEND ARCHITEKTUR                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │
│   │    API      │  │    PEER     │  │ PROTECTION  │  │     ECLVM       │   │
│   │  (gRPC)     │  │  (P2P+UI)   │  │  (Schutz)   │  │   (Policies)    │   │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └───────┬─────────┘   │
│          │                │                │                 │             │
│   ┌──────┴────────────────┴────────────────┴─────────────────┴──────┐      │
│   │                         CORE LOGIC                               │      │
│   │       (Event Engine, Trust Engine, World Formula, Consensus)     │      │
│   └──────────────────────────────┬───────────────────────────────────┘      │
│                                  │                                          │
│   ┌──────────────────────────────┴───────────────────────────────────┐      │
│   │                         DOMAIN (UDM)                              │      │
│   │   (UniversalId, Event, Trust, Realm, Saga, Cost, Message)        │      │
│   └──────────────────────────────┬───────────────────────────────────┘      │
│                                  │                                          │
│   ┌──────────────────────────────┴───────────────────────────────────┐      │
│   │                     LOCAL STORAGE (Fjall)                         │      │
│   │   (Events, Trust, Identities, Content, Realms, Archive)          │      │
│   └──────────────────────────────────────────────────────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Verzeichnisstruktur

```
backend/src/
├── main.rs              # Einstiegspunkt
├── lib.rs               # Library-Root
├── server.rs            # gRPC/HTTP Server
├── error.rs             # Globale Fehlertypen
├── telemetry.rs         # Observability (Tracing)
│
├── api/                 # 🌐 API-Schicht (gRPC, Connect)
│
├── core/                # 🧠 Business-Logik (Axiome Κ2-Κ18)
│   ├── consensus.rs     # Konsensus-Mechanismus (Κ18)
│   ├── engine.rs        # ExecutionContext-Wrapper
│   ├── event_engine.rs  # Event-Verarbeitung (Κ9-Κ12)
│   ├── surprisal.rs     # Surprisal-Berechnung (Κ15a)
│   ├── trust_engine.rs  # Trust-Berechnung (Κ2-Κ5)
│   └── world_formula.rs # Weltformel-Engine (Κ15b-d)
│
├── domain/              # 📦 Unified Data Model (UDM)
│   └── unified/
│       ├── primitives.rs# UniversalId, TemporalCoord
│       ├── identity.rs  # DID, Delegation (Κ6-Κ8)
│       ├── event.rs     # Events, Finality (Κ9-Κ12)
│       ├── trust.rs     # TrustVector6D (Κ2-Κ5)
│       ├── realm.rs     # Realm-Hierarchie (Κ1)
│       ├── saga.rs      # Sagas (Κ22-Κ24)
│       ├── formula.rs   # Weltformel-Komponenten
│       ├── cost.rs      # Kosten-Algebra (Gas × Mana × Trust)
│       └── message.rs   # P2P-Nachrichtentypen
│
├── eclvm/               # ⚙️ Policy-VM
│   ├── parser.rs        # ECL → AST
│   ├── compiler.rs      # AST → Bytecode
│   ├── bytecode.rs      # OpCode, Value
│   ├── runtime/         # Stack-basierte VM
│   └── mana.rs          # Mana-Management
│
├── local/               # 💾 Persistenz (Fjall KV)
│   ├── kv_store.rs      # Basis KV-Abstraktion
│   ├── event_store.rs   # Event-DAG Persistenz
│   ├── trust_store.rs   # Trust-Vektoren
│   ├── identity_store.rs# DID-Speicher
│   ├── content_store.rs # Content-Addressed Storage
│   ├── realm_storage.rs # Dynamische Realm-Stores
│   ├── archive.rs       # Cold Storage (Merkle-Proofs)
│   └── blueprint_marketplace.rs
│
├── peer/                # 🌍 P2P & Client-Facing
│   ├── gateway.rs       # Cross-Realm Gateway (Κ23)
│   ├── intent_parser.rs # Intent-Parsing (Κ22)
│   ├── saga_composer.rs # Saga-Komposition (Κ22)
│   └── p2p/             # libp2p Netzwerk
│       ├── behaviour.rs # ErynoaBehaviour
│       ├── config.rs    # P2PConfig, NatConfig
│       ├── swarm.rs     # SwarmManager
│       ├── sync.rs      # Delta-Sync Protokoll
│       └── trust_gate.rs# Trust-basierte Verbindungen
│
└── protection/          # 🛡️ Systemschutz (Κ19-Κ28)
    ├── adaptive_calibration.rs  # PID-Controller
    ├── anomaly.rs       # Anomalie-Erkennung
    ├── anti_calcification.rs   # Macht-Dezentralisierung
    ├── diversity.rs     # System-Diversität
    └── quadratic.rs     # Quadratisches Voting
```

### P2P Network Layer (libp2p 0.54)

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          P2P NETWORK LAYER                              │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │   GOSSIPSUB  │  │   KADEMLIA   │  │   IDENTIFY   │                  │
│  │   (PubSub)   │  │   (DHT)      │  │   (Handshake)│                  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │
│         │                 │                 │                           │
│  ┌──────┴─────────────────┴─────────────────┴──────┐                   │
│  │              ERYNOA BEHAVIOUR                    │                   │
│  │  • Realm-Topics (/erynoa/realm/{id}/events/v1)  │                   │
│  │  • DID-based PeerID (Ed25519)                   │                   │
│  │  • Trust-gated Connections                      │                   │
│  └──────────────────────────────────────────────────┘                   │
│                            │                                            │
│  ┌────────────────────────┴────────────────────────┐                   │
│  │    NAT TRAVERSAL (AutoNAT + DCUTR + Relay)      │                   │
│  └──────────────────────────────────────────────────┘                   │
│                            │                                            │
│  ┌────────────────────────┴────────────────────────┐                   │
│  │    TRANSPORT (TCP + Noise + Yamux)              │                   │
│  └──────────────────────────────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Unified Data Model (UDM)

#### UniversalId (32 Bytes)

Content-addressed Identifier mit Type-Tag:

```
┌──────────┬────────────┬─────────────────────────────────────────┐
│ Type Tag │  Version   │            BLAKE3 Hash (28 bytes)       │
│ (2 bytes)│  (2 bytes) │                                         │
└──────────┴────────────┴─────────────────────────────────────────┘
```

#### TrustVector6D (24 Bytes)

```rust
pub struct TrustVector6D {
    pub r: f32,     // Reliability (Verhaltens-Historie)
    pub i: f32,     // Integrity (Aussage-Konsistenz)
    pub c: f32,     // Competence (Fähigkeits-Nachweis)
    pub p: f32,     // Prestige (Externe Attestation)
    pub v: f32,     // Vigilance (Anomalie-Erkennung)
    pub omega: f32, // Omega (Axiom-Treue)
}
```

#### Kosten-Algebra

```rust
pub struct Cost {
    pub gas: u64,        // Computation
    pub mana: u64,       // Storage/Network
    pub trust_risk: f32, // Trust-Kosten [0, 1]
}
```

---

## 📊 Implementation Status

### Backend Module (409 Tests ✅)

| Schicht        | Module                                                         | Tests | Status |
| -------------- | -------------------------------------------------------------- | ----- | ------ |
| **Domain**     | unified (primitives, identity, event, trust, realm, saga)      | 89    | ✅     |
| **Core**       | event_engine, trust_engine, surprisal, world_formula           | 124   | ✅     |
| **ECLVM**      | parser, compiler, runtime, bridge, mana                        | 67    | ✅     |
| **Protection** | anti_calcification, diversity, quadratic, anomaly, calibration | 39    | ✅     |
| **Peer**       | intent_parser, saga_composer, gateway, p2p                     | 38    | ✅     |
| **Local**      | kv_store, event_store, trust_store, archive, marketplace       | 52    | ✅     |

### Prioritäten-Status

| Priorität | Beschreibung                                             | Status           |
| --------- | -------------------------------------------------------- | ---------------- |
| **P1**    | IPS-01-imp Grundgerüst, UDM-Strukturen                   | ✅ Abgeschlossen |
| **P2**    | Core-Engines, ExecutionContext, Invarianten              | ✅ Abgeschlossen |
| **P3**    | libp2p NAT-Traversal, Cold Storage, Adaptive Calibration | ✅ Abgeschlossen |

### Axiom Coverage

| Kategorie   | Axiome  | Status   |
| ----------- | ------- | -------- |
| Kern-Axiome | Κ1-Κ28  | ✅ 28/28 |
| Peer-Axiome | PR1-PR6 | ✅ 6/6   |

```bash
# Tests ausführen
cd backend && cargo test

# Ergebnis:
# test result: ok. 409 passed; 0 failed
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
| P2P        | **libp2p 0.54**         |
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
│   │   ├── domain/unified/    # UDM: UniversalId, Event, Trust, Realm, Saga
│   │   ├── core/              # Engines (Event, Trust, Surprisal, Consensus)
│   │   ├── protection/        # Anti-Gaming, Diversity, Calibration
│   │   ├── peer/              # Intent, Saga, Gateway, P2P
│   │   ├── eclvm/             # Policy-VM (Parser, Compiler, Runtime)
│   │   ├── local/             # Dezentrale Storage (Fjall, Archive)
│   │   └── api/               # HTTP/gRPC Handlers
│   ├── proto/erynoa/v1/       # Protobuf Definitions
│   ├── config/                # TOML Konfiguration
│   └── data/                  # Fjall Datenbank (gitignored)
│
├── frontend/                  # 🎨 SvelteKit Apps
│   ├── console/               # Admin Console
│   ├── platform/              # Main Platform
│   └── docs/                  # Documentation Site
│
├── documentation/             # 📖 Dokumentation
│   ├── ROADMAP.md             # Strategischer Plan
│   ├── concept-v4/            # Fachkonzept V6.2
│   │   ├── FACHKONZEPT.md     # Vollständiges Konzept
│   │   ├── CLI-REFERENCE.md   # CLI Commands
│   │   ├── LOGIC.md           # Logik & Axiome
│   │   └── SYSTEM-ARCHITECTURE.md
│   └── system/                # System-Dokumentation
│       ├── essential_guide.md # Konsolidierter Guide
│       ├── navigation.md      # Dokumentations-Navigation
│       ├── reference/
│       │   ├── architecture.md
│       │   └── BACKEND-ARCHITECTURE.md  # ⭐ Backend-Details
│       └── development/
│           ├── IPS-01-imp.md            # ⭐ IPS Logik-Modell
│           ├── UNIFIED-DATA-MODEL.md    # ⭐ UDM Spezifikation
│           ├── P2P-IMPLEMENTATION.md    # ⭐ P2P Details
│           └── IPS-UDM-GAP-ANALYSIS.md  # Implementierungs-Status
│
├── infra/                     # 🏗 Infrastructure
│   ├── docker/
│   └── proxy/
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
| `just test`  | Tests ausführen (409 Tests) |
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

| Dokument                                                                             | Beschreibung                      |
| ------------------------------------------------------------------------------------ | --------------------------------- |
| **[📋 Fachkonzept V6.2](documentation/concept-v4/FACHKONZEPT.md)**                   | Vollständiges technisches Konzept |
| **[🧮 IPS Logik-Modell](documentation/system/development/IPS-01-imp.md)**            | Kategorialtheoretische Grundlagen |
| **[📦 Unified Data Model](documentation/system/development/UNIFIED-DATA-MODEL.md)**  | Datenstruktur-Spezifikation       |
| **[🏗️ Backend Architektur](documentation/system/reference/BACKEND-ARCHITECTURE.md)** | Backend-Schichten & Module        |
| **[🌐 P2P Implementation](documentation/system/development/P2P-IMPLEMENTATION.md)**  | libp2p Netzwerk-Details           |

### System-Dokumentation

| Dokument                                                       | Beschreibung             |
| -------------------------------------------------------------- | ------------------------ |
| [Essential Guide](documentation/system/essential_guide.md)     | Konsolidierter Guide     |
| [Navigation](documentation/system/navigation.md)               | Dokumentations-Übersicht |
| [Architecture](documentation/system/reference/architecture.md) | System-Architektur       |
| [Setup Guide](documentation/system/setup/setup.md)             | Entwicklungsumgebung     |
| [Style Guide](documentation/system/development/style-guide.md) | Code-Stil                |

### Axiom-Übersicht

<details>
<summary><strong>Κ1-Κ28 Kern-Axiome</strong></summary>

| Axiom   | Name                 | Modul                                                                    |
| ------- | -------------------- | ------------------------------------------------------------------------ |
| Κ1      | Realm-Hierarchie     | `domain/unified/realm.rs`                                                |
| Κ2-Κ5   | Trust-System 6D      | `domain/unified/trust.rs`, `core/trust_engine.rs`                        |
| Κ6-Κ8   | DID & Delegation     | `domain/unified/identity.rs`                                             |
| Κ9-Κ12  | Event-DAG & Finality | `domain/unified/event.rs`, `core/event_engine.rs`                        |
| Κ15a-d  | Weltformel           | `core/world_formula.rs`, `core/surprisal.rs`                             |
| Κ18     | Konsensus            | `core/consensus.rs`                                                      |
| Κ19     | Anti-Calcification   | `protection/anti_calcification.rs`, `protection/adaptive_calibration.rs` |
| Κ20     | Diversity            | `protection/diversity.rs`                                                |
| Κ21     | Quadratic Voting     | `protection/quadratic.rs`                                                |
| Κ22-Κ24 | Saga-System          | `domain/unified/saga.rs`, `peer/saga_composer.rs`                        |
| Κ23     | Gateway Guard        | `peer/gateway.rs`, `eclvm/programmable_gateway.rs`                       |
| Κ26     | Anomalie-Erkennung   | `protection/anomaly.rs`                                                  |

</details>

<details>
<summary><strong>PR1-PR6 Peer-Axiome</strong></summary>

| Axiom | Name                    | Modul                                            |
| ----- | ----------------------- | ------------------------------------------------ |
| PR1   | Intent-Auflösung        | `peer/intent_parser.rs`, `peer/saga_composer.rs` |
| PR2   | Saga-Atomarität         | `domain/unified/saga.rs`                         |
| PR3   | Gateway-Vollständigkeit | `peer/gateway.rs`                                |
| PR4   | Funktor-Eigenschaften   | `domain/unified/trust.rs`                        |
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
┌─────────────────────────────────────────────────────────────────────────┐
│  P2P Layer          (libp2p, NAT-Traversal, Gossipsub, DHT) ✅          │
├─────────────────────────────────────────────────────────────────────────┤
│  Peer Layer         (IntentParser, SagaComposer, GatewayGuard) ✅       │
├─────────────────────────────────────────────────────────────────────────┤
│  Protection Layer   (AntiCalcification, Diversity, Calibration) ✅      │
├─────────────────────────────────────────────────────────────────────────┤
│  Core Logic Layer   (EventEngine, TrustEngine, WorldFormula) ✅         │
├─────────────────────────────────────────────────────────────────────────┤
│  Domain Layer       (UDM: UniversalId, Trust6D, Event, Saga) ✅         │
├─────────────────────────────────────────────────────────────────────────┤
│  Storage Layer      (Fjall, Cold Archive, Blueprint Marketplace) ✅     │
└─────────────────────────────────────────────────────────────────────────┘
```

**28 Axiome (Κ1-Κ28) · 6 Schichten · 409 Tests ✅**

_„Intelligenz im Dienste des Lebens."_

</div>
