<div align="center">

# Erynoa

**Kybernetisches Protokoll für die Maschinenökonomie**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0-FF3E00?style=flat-square&logo=svelte)](https://kit.svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![Nix](https://img.shields.io/badge/Nix-Flakes-5277C3?style=flat-square&logo=nixos)](https://nixos.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   "Ein Protokoll, das Maschinen befähigt, eigenständig zu       │
│    handeln, zu verhandeln und voneinander zu lernen –           │
│    mit mathematisch fundiertem Vertrauen."                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

[Schnellstart](#-schnellstart) · [Konzept](#-was-ist-erynoa) · [Dokumentation](#-dokumentation) · [Befehle](#-befehle)

</div>

---

## 🧠 Was ist Erynoa?

Erynoa ist ein **dezentrales Protokoll**, das autonomen Agenten ermöglicht, vertrauensbasierte Transaktionen ohne zentrale Vermittler durchzuführen.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                         DIE DREI SPHÄREN                                    │
│                                                                             │
│                              ┌─────────┐                                    │
│                              │  ECHO   │                                    │
│                              │ Emergent│  ← Agenten, Verhandlung            │
│                              │  Swarm  │                                    │
│                              └────┬────┘                                    │
│                                   │                                         │
│               ┌───────────────────┼───────────────────┐                     │
│               │                   │                   │                     │
│               ▼                   │                   ▼                     │
│        ┌─────────────┐            │            ┌─────────────┐              │
│        │     ERY     │◀───────────┴───────────▶│     NOA     │              │
│        │  Semantic   │                         │   Causal    │              │
│        │   Lattice   │                         │   Ledger    │              │
│        └─────────────┘                         └─────────────┘              │
│              ↑                                        ↑                     │
│       Semantik, Trust                         Finalität, Wahrheit           │
│                                                                             │
│                        ERY + NOA = ERYNOA                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

| Sphäre   | Funktion                   | Technologie           |
| -------- | -------------------------- | --------------------- |
| **ERY**  | Semantik, Trust, Discovery | Qdrant, Karmic Engine |
| **ECHO** | Agenten, Verhandlung, P2P  | WASM, libp2p, XMTP    |
| **NOA**  | Finalität, Settlement      | MoveVM, Starfish BFT  |

> 📖 **Mehr erfahren:** [Fachkonzept](documentation/concept/fachkonzept.md) · [Kernkonzept](documentation/concept/kernkonzept.md)

---

## ⚡ Schnellstart

> **Voraussetzungen:** [Nix](https://nixos.org/) und [Docker Desktop](https://www.docker.com/products/docker-desktop/)
>
> 📖 **Detaillierte Anleitung:** [Setup Guide](documentation/system/setup/setup.md)

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
| Backend direkt          | <http://localhost:3000>          |
| ZITADEL (Auth)          | <http://localhost:8080>          |
| MinIO (Storage)         | <http://localhost:9001>          |

**Test-Login:**

- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

</details>

---

## 📖 Dokumentation

Die Dokumentation ist in zwei Bereiche unterteilt:

```
documentation/
├── concept/          # 🧠 Protokoll & Konzept
│   ├── fachkonzept.md           # ⭐ Master-Dokument (Start hier)
│   ├── kernkonzept.md           # High-Level Überblick
│   ├── system-architecture-overview.md
│   ├── liquides-datenmodell.md
│   ├── trust-and-reputation.md
│   ├── cybernetic-loop.md
│   ├── agents-and-adl.md
│   ├── use-cases.md
│   └── glossary.md
│
└── system/           # 🛠️ Plattform & Entwicklung
    ├── readme.md                # System-Übersicht
    ├── essential_guide.md       # Alles auf einen Blick
    ├── guides/                  # Getting Started, ZITADEL
    ├── setup/                   # Entwicklungsumgebung
    ├── reference/               # Architektur, Config
    └── development/             # Style Guide, Testing, TODOs
```

### 🧠 Konzept-Dokumentation

| Dokument                                                                   | Beschreibung                                     |
| -------------------------------------------------------------------------- | ------------------------------------------------ |
| **[📋 Fachkonzept](documentation/concept/fachkonzept.md)**                 | **⭐ Hier starten** – Vollständige Spezifikation |
| [Kernkonzept](documentation/concept/kernkonzept.md)                        | Kompakter High-Level-Überblick                   |
| [Systemarchitektur](documentation/concept/system-architecture-overview.md) | Technische Architektur-Details                   |
| [Liquides Datenmodell](documentation/concept/liquides-datenmodell.md)      | Blueprints, AMOs, Fluid Extensions               |
| [Trust & Reputation](documentation/concept/trust-and-reputation.md)        | Karmic Engine, Trust Vectors                     |
| [Cybernetic Loop](documentation/concept/cybernetic-loop.md)                | Der 6-Phasen-Workflow                            |
| [Agents & ADL](documentation/concept/agents-and-adl.md)                    | Agentenmodell und Intent-Sprache                 |
| [Use Cases](documentation/concept/use-cases.md)                            | EV-Charging, Industrie, Prosumer                 |
| [Glossar](documentation/concept/glossary.md)                               | Begriffsdefinitionen                             |

### 🛠️ System-Dokumentation

| Dokument                                                          | Beschreibung                    |
| ----------------------------------------------------------------- | ------------------------------- |
| **[📚 Übersicht](documentation/system/readme.md)**                | Plattform-Dokumentation         |
| [Essential Guide](documentation/system/essential_guide.md)        | Alles Wichtige auf einen Blick  |
| [Getting Started](documentation/system/guides/getting-started.md) | Erste Schritte                  |
| [Setup](documentation/system/setup/setup.md)                      | Entwicklungsumgebung einrichten |
| [Architecture](documentation/system/reference/architecture.md)    | System-Architektur              |
| [Configuration](documentation/system/reference/config.md)         | Service-Konfiguration           |
| [Style Guide](documentation/system/development/style-guide.md)    | Code-Stil                       |
| [Testing](documentation/system/development/testing.md)            | Test-Strategien                 |
| [TODOs](documentation/system/development/todos.md)                | Offene Aufgaben                 |

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
├── backend/              # 🦀 Rust API Server
│   ├── src/              # Source Code
│   ├── config/           # TOML Konfiguration
│   ├── migrations/       # SQL Migrations
│   └── proto/            # Protobuf Definitionen
│
├── frontend/             # 🎨 SvelteKit Apps (pnpm Workspace)
│   ├── console/          # Admin Console
│   ├── platform/         # Main Platform
│   └── docs/             # Documentation Site
│
├── documentation/        # 📖 Dokumentation
│   ├── concept/          # 🧠 Protokoll & Konzept
│   └── system/           # 🛠️ Plattform & Entwicklung
│
├── infra/                # 🏗 Infrastructure
│   ├── docker/           # Docker Compose & Dockerfiles
│   ├── proxy/            # Caddy Reverse Proxy
│   ├── auth/             # ZITADEL Config
│   └── static/           # Static Files
│
├── scripts/              # 🔧 Build & Dev Scripts
│
├── flake.nix             # Nix Dev Environment
├── justfile              # Task Runner Commands
├── buf.yaml              # Protobuf Config
└── turbo.json            # Turborepo Config
```

---

## 🔧 Befehle

### Entwicklung

| Befehl             | Beschreibung                                       |
| ------------------ | -------------------------------------------------- |
| `just dev`         | **Startet alles** (Frontends + Backend + Services) |
| `just dev console` | Nur Console starten                                |
| `just status`      | Status aller Services                              |
| `just logs`        | Logs anzeigen                                      |
| `just stop`        | Alle Container stoppen                             |
| `just restart`     | Schneller Neustart                                 |
| `just reset`       | Alles löschen und neu starten                      |

### Backend

| Befehl       | Beschreibung      |
| ------------ | ----------------- |
| `just check` | Cargo check       |
| `just lint`  | Clippy Linter     |
| `just fmt`   | Code formatieren  |
| `just test`  | Tests ausführen   |
| `just ci`    | fmt + lint + test |

### Setup

| Befehl               | Beschreibung                    |
| -------------------- | ------------------------------- |
| `just init`          | Initialisierung ohne Dev-Server |
| `just init-env`      | `.env` erstellen                |
| `just zitadel-setup` | ZITADEL neu konfigurieren       |
| `just proto-gen`     | Protobuf Types generieren       |

<details>
<summary><strong>📋 Alle Befehle anzeigen</strong></summary>

```bash
just --list
```

</details>

---

## 📊 Status

### ✅ Implementiert

- ✅ Connect-RPC API (Protobuf)
- ✅ Monorepo mit pnpm & Turborepo
- ✅ SvelteKit Frontends (Svelte 5)
- ✅ ZITADEL Auth mit automatischem Setup
- ✅ Caddy Reverse Proxy
- ✅ DevContainer Support
- ✅ GitHub Actions CI/CD
- ✅ Nix Flakes Dev Environment

### 🔄 In Arbeit

- Frontend Tests
- Weitere Details: [TODOs](documentation/system/development/todos.md)

---

## 🤝 Contributing

1. Prüfe [TODOs](documentation/system/development/todos.md) für offene Aufgaben
2. Folge dem [Style Guide](documentation/system/development/style-guide.md)
3. Schreibe Tests ([Testing Guide](documentation/system/development/testing.md))

---

## 📞 Support

Bei Problemen:

1. [Essential Guide](documentation/system/essential_guide.md) – Troubleshooting
2. [TODOs](documentation/system/development/todos.md) – Bekannte Issues
3. [Connections](documentation/system/reference/connections.md) – Service-Probleme

---

<div align="center">

**[MIT License](LICENSE)**

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│          ERY + NOA = ERYNOA                            │
│     Semantic Lattice + Causal Ledger                    │
│          Wissen + Wahrheit = Vertrauen                  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

Made with ❤️ and 🦀

</div>
