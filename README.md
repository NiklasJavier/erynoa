<div align="center">

# Erynoa

**Dezentrales Protokoll für vertrauensbasierte Zusammenarbeit**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0-FF3E00?style=flat-square&logo=svelte)](https://kit.svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![libp2p](https://img.shields.io/badge/libp2p-0.54-blue?style=flat-square)](https://libp2p.io/)
[![Nix](https://img.shields.io/badge/Nix-Flakes-5277C3?style=flat-square&logo=nixos)](https://nixos.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Axioms](https://img.shields.io/badge/Axioms-28-blueviolet?style=flat-square)](documentation/concept-v5/02-AXIOM-SYSTEM.md)

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

[Schnellstart](#-schnellstart) · [Architektur](#-architektur) · [API](#-api) · [Dokumentation](#-dokumentation)

</div>

---

## Was ist Erynoa?

Erynoa ist ein **dezentrales kybernetisches Protokoll** für vertrauensbasierte Interaktionen zwischen Menschen, Organisationen und KI-Agenten. Es basiert auf **28 formal definierten Axiomen (Κ1–Κ28)**, einem **Unified Data Model (UDM)** und einem **Unified State** als Single Source of Truth.

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         ERYNOA – STATE-GETRIEBENE ARCHITEKTUR                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│   ┌─────────────────────────────────────────────────────────────────────────┐    │
│   │  API Layer           REST /api/v1 (State, Health, Crossing, Trust, …) │    │
│   │  Connect-RPC         Health, Info, Peer (optional)                     │    │
│   │  Auth                Passkey/WebAuthn                                   │    │
│   ├─────────────────────────────────────────────────────────────────────────┤    │
│   │  Unified State       Snapshots, Event-Log, Merkle/Delta, CQRS-Stream   │    │
│   │  StateCoordinator    Health, Invarianten, Circuit Breaker               │    │
│   ├─────────────────────────────────────────────────────────────────────────┤    │
│   │  Peer Layer          GatewayGuard, IntentParser, SagaComposer           │    │
│   │  P2P (optional)      libp2p, Gossipsub, NAT-Traversal, Privacy         │    │
│   ├─────────────────────────────────────────────────────────────────────────┤    │
│   │  Core                EventEngine, TrustEngine, WorldFormula, Consensus  │    │
│   │  ECLVM               Policy-VM, Programmable Gateway                    │    │
│   ├─────────────────────────────────────────────────────────────────────────┤    │
│   │  Domain (UDM)        UniversalId, Event, Trust6D, Realm, Saga          │    │
│   ├─────────────────────────────────────────────────────────────────────────┤    │
│   │  Storage             Fjall KV, Event-Store, Archive, Blueprint         │    │
│   └─────────────────────────────────────────────────────────────────────────┘    │
│                                                                                  │
│   KERN-FEATURES:                                                                 │
│   • UnifiedState + StateEvent (log_and_apply)                                    │
│   • REST State-API: Snapshots, Health, Events, Merkle/Delta, SSE-Stream          │
│   • 6D Trust, Event-DAG, Realm-Hierarchie, Intent → Saga                         │
│   • ECLVM Policy-VM, Gateway-Crossing, Protection (Anti-Calc, Diversity)        │
│   • P2P Testnet (libp2p), Dev Container, Nix Dev-Shell                           │
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

> **Mehr:** [Concept V5](documentation/concept-v5/README.md) · [Fachkonzept](documentation/concept-v5/02-FACHKONZEPT.md) · [State-Management](documentation/concept-v5/04-STATE-MANAGEMENT.md) · [API-Beschreibung](documentation/system/reference/API-BESCHREIBUNG.md)

---

## Schnellstart

**Voraussetzungen:** [Nix](https://nixos.org/) (optional für Dev-Shell), [Docker](https://www.docker.com/) für den vollen Stack.

```bash
# Repository klonen
git clone <repo-url> && cd erynoa-main

# Option A: Backend lokal (ohne Docker)
nix develop
cd backend && cargo run
# → API: http://localhost:8000  (REST /api/v1/*)

# Option B: Vollständiger Dev-Stack (Docker)
just dev
# → Proxy: http://localhost:3001
# → Console: http://localhost:3001/console
# → Platform: http://localhost:3001/platform
# → Docs:     http://localhost:3001/docs
# → API:      http://localhost:3001/api/v1
```

**Backend Single-Binary** – keine externe Datenbank nötig; State in-memory bzw. Fjall-embedded.

<details>
<summary><strong>URLs im Überblick</strong></summary>

| Service           | URL (mit Proxy)              | Lokal (cargo run)   |
| ----------------- | ---------------------------- | ------------------- |
| **API (REST)**    | http://localhost:3001/api/v1 | http://localhost:8000/api/v1 |
| **Health**       | GET /api/v1/health           | GET /api/v1/health  |
| **State-Snapshot** | GET /api/v1/state/snapshot | GET /api/v1/state/snapshot |
| **Console**      | http://localhost:3001/console | –                  |
| **Platform**     | http://localhost:3001/platform | –                 |
| **Docs**         | http://localhost:3001/docs   | –                  |

**Auth:** Passkey/WebAuthn + optional DID-basiert (kein externer Auth-Service nötig).

</details>

---

## Architektur

### Backend (Rust)

- **Unified State** (`core/state.rs`): Single Source of Truth, Snapshots, Event-Log, Merkle-Tracker, Circuit Breaker, CQRS-Broadcaster.
- **State-Coordination** (`core/state_coordination.rs`, `state_integration.rs`): Health, Invarianten, Observer-Integration.
- **API** (`api/`): REST unter `/api/v1` (State, Health, Events, Crossing, Trust, Identity, Realms, ECL, Governance, Controller, Intent, Saga, Debug, Merkle/Delta/Stream), optional Connect-RPC, Passkey-Auth.
- **Core** (`core/`): EventEngine, TrustEngine, WorldFormula, Consensus, ECLVM-State-Host.
- **Domain** (`domain/unified/`): UDM (UniversalId, Event, Trust, Realm, Saga, Formula, Cost).
- **ECLVM** (`eclvm/`): Parser, Compiler, Bytecode, Runtime, Programmable Gateway, Mana.
- **Peer** (`peer/`): GatewayGuard, IntentParser, SagaComposer, P2P (libp2p: Swarm, Gossip, Transport, Privacy).
- **Local Storage** (`local/`): Fjall KV, Event-Store, Trust-Store, Identity-Store, Archive, Blueprint-Marketplace.
- **Protection** (`protection/`): Anomaly, Diversity, Quadratic, Anti-Calcification, Adaptive Calibration.

### Binaries

| Binary              | Beschreibung                    |
| ------------------- | ------------------------------- |
| `erynoa-api`        | Haupt-API-Server (REST + optional Connect) |
| `ecl`               | ECL-CLI (mit Feature `cli`)     |
| `erynoa-testnet-node` | P2P-Testnet-Node (Feature `p2p`) |
| `erynoa-debug`      | Egui-Debugger (Feature `debug`) |

### Frontend (SvelteKit)

| App       | Zweck                    |
| --------- | ------------------------ |
| **console** | Admin-Console          |
| **platform** | Haupt-Plattform       |
| **docs**     | Dokumentations-Site   |
| **ui**       | Gemeinsame UI-Bibliothek (@erynoa/ui) |

---

## API

Die **REST-API** unter `/api/v1` ist state-getrieben und in fünf Phasen umgesetzt:

| Bereich        | Beispiele |
| -------------- | --------- |
| **State**     | Snapshots (voll/komponenten), Metriken, Warnings, Mode, Circuit Breaker, State-Event (Mutation), **Merkle** (root, component, delta, proof), **Stream** (SSE) |
| **Health**    | /health/state, /health/state/detail, /health/aggregate |
| **Events**    | Liste, Einzelevent, log/snapshot, checkpoints |
| **Invariants**| Liste (optional nach Severity) |
| **Crossing**  | POST /crossing/validate, GET /crossing/stats |
| **Trust**     | GET /trust/:did, POST /trust/update |
| **Identity**  | GET /identity/root, GET /identity/:did |
| **Realms**    | CRUD, members, ecl |
| **ECL**       | run, api/:route_id, ui/:component_id, controller/:key (Stubs) |
| **Governance**| proposals (create, list), proposals/:id/vote |
| **Controller**| check, permissions |
| **Intent/Saga** | parse, compose, execute, stats (teils Stubs) |
| **Debug**     | replay, replay/checkpoint, checkpoint |

Vollständige Beschreibung: **[API-BESCHREIBUNG.md](documentation/system/reference/API-BESCHREIBUNG.md)**.
Connect-RPC/Proto: [API-REFERENCE.md](documentation/system/reference/API-REFERENCE.md).

---

## Projektstruktur

```
erynoa-main/
├── backend/                    # Rust API & Core
│   ├── src/
│   │   ├── api/                # REST (state_handlers, production_handlers, debug_handlers), Auth, Middleware
│   │   ├── core/               # state.rs, state_coordination, state_integration, event_engine, trust_engine, …
│   │   ├── domain/unified/     # UDM (primitives, identity, event, trust, realm, saga, formula, cost)
│   │   ├── eclvm/              # Parser, Compiler, Runtime, Programmable Gateway, Mana
│   │   ├── local/              # Fjall KV, event_store, trust_store, archive, blueprint_marketplace
│   │   ├── peer/               # gateway, intent_parser, saga_composer, p2p/
│   │   ├── protection/         # anomaly, diversity, quadratic, anti_calcification, adaptive_calibration
│   │   └── execution/          # context, tracked
│   ├── documentation/system/   # API-PLAN-STATE-DRIVEN, ECL-ECLVM, CORE-DOMAIN, …
│   └── Cargo.toml
│
├── frontend/
│   ├── console/                # SvelteKit Admin Console
│   ├── platform/               # SvelteKit Platform
│   ├── docs/                   # SvelteKit Docs
│   └── ui/                     # Gemeinsame UI-Komponenten
│
├── documentation/
│   ├── concept-v5/             # Spezifikation V5 (Vision, Axiome, Architektur, State, CLI)
│   ├── concept-v4/             # Concept V4
│   ├── system/
│   │   ├── reference/          # API-BESCHREIBUNG, API-REFERENCE, BACKEND-ARCHITECTURE, …
│   │   ├── development/       # IPS-01-imp, UNIFIED-DATA-MODEL, STATE-RS-*, style-guide, testing
│   │   └── setup/              # setup, docker, devcontainer
│   └── ROADMAP.md
│
├── infra/docker/               # docker-compose, Dockerfiles, Testnet
├── scripts/                    # build, dev, test
├── flake.nix                   # Nix Dev-Shell
├── justfile                    # Tasks (dev, backend-*, testnet, proto-gen, …)
├── pnpm-workspace.yaml         # Monorepo (console, platform, docs, ui)
└── turbo.json
```

---

## Befehle (just)

| Kategorie   | Befehl            | Beschreibung |
| ----------- | ----------------- | ------------ |
| **Dev**     | `just dev`        | Vollständiger Stack (Docker: console, platform, docs, backend, proxy) |
|             | `just backend-run`| Backend lokal starten (cargo run) |
| **Backend** | `just backend-check`  | cargo check |
|             | `just backend-test`   | cargo test |
|             | `just backend-build`  | cargo build |
|             | `just backend-fmt`    | cargo fmt |
|             | `just backend-clippy` | clippy |
| **Docker**  | `just stop`       | Container stoppen |
|             | `just status`     | Service-Status + Health-Checks |
|             | `just logs [service]` | Logs (optional service) |
| **Testnet** | `just testnet run`   | P2P-Testnet starten |
|             | `just testnet-dev run`| P2P-Testnet Dev-Mode (Hot-Reload) |
| **Proto**   | `just proto-gen`  | buf generate |
| **Clean**   | `just clean`      | Docker down -v |
|             | `just clean-all`  | clean + backend clean + node_modules |

---

## Dokumentation

| Dokument | Inhalt |
| -------- | ------ |
| [Concept V5](documentation/concept-v5/README.md) | Spezifikation V5 (Vision, Axiome, Architektur, State, CLI) |
| [API-Beschreibung](documentation/system/reference/API-BESCHREIBUNG.md) | **REST-API** (State, Health, Events, Crossing, Trust, Identity, Realms, ECL, Governance, Debug, Merkle/Stream) |
| [API-REFERENCE](documentation/system/reference/API-REFERENCE.md) | Connect-RPC, Proto, WebAuthn |
| [BACKEND-ARCHITECTURE](documentation/system/reference/BACKEND-ARCHITECTURE.md) | Backend-Schichten & Module |
| [UNIFIED-DATA-MODEL](documentation/system/development/UNIFIED-DATA-MODEL.md) | UDM-Spezifikation |
| [IPS-01-imp](documentation/system/development/IPS-01-imp.md) | IPS Logik-Modell |
| [API-PLAN-STATE-DRIVEN](backend/documentation/system/API-PLAN-STATE-DRIVEN.md) | State-API-Plan (Phasen 1–5) |
| [Essential Guide](documentation/system/essential_guide.md) | Konsolidierter System-Guide |
| [Navigation](documentation/system/navigation.md) | Dokumentations-Übersicht |
| [Setup](documentation/system/setup/setup.md) | Entwicklungsumgebung |
| [Style Guide](documentation/system/development/style-guide.md) | Code-Stil |
| [Testing](documentation/system/development/testing.md) | Tests |

---

## Tech Stack

| Schicht     | Technologie |
| ----------- | ----------- |
| **Backend** | Rust, Tokio, Axum, Connect-RPC (optional), Fjall (KV), blake3, ed25519-dalek |
| **API**     | REST JSON, SSE (State-Stream), Passkey/WebAuthn |
| **P2P**     | libp2p 0.54 (optional), QUIC/TCP, Gossipsub, Kademlia, Privacy |
| **Frontend**| SvelteKit, Vite, TypeScript, Tailwind, Biome, Turborepo, pnpm |
| **Infra**   | Docker Compose, Caddy (Proxy), Nix Flakes, just |

---

## Entwicklungsumgebung

- **Nix:** `nix develop` für Rust-Toolchain und Umgebung.
- **Dev Container:** `.devcontainer/` für VS Code / Cursor (Docker-basiert).
- **Docker:** `just dev` startet Backend + Frontends + Proxy; Health-Check nutzt Connect-RPC oder REST `/api/v1/health`.

---

## Contributing

1. [Roadmap](documentation/ROADMAP.md) und offene Aufgaben prüfen.
2. [Style Guide](documentation/system/development/style-guide.md) beachten.
3. Tests ergänzen ([Testing](documentation/system/development/testing.md)).
4. Axiom-Konsistenz wahren (Concept V5, UDM).

---

<div align="center">

**[MIT License](LICENSE)**

_„Intelligenz im Dienste des Lebens."_

</div>
