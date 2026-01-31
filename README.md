<div align="center">

# Erynoa

**Dezentrales Vertrauen für Menschen, Organisationen und KI-Agenten**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0-FF3E00?style=flat-square&logo=svelte)](https://kit.svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![Nix](https://img.shields.io/badge/Nix-Flakes-5277C3?style=flat-square&logo=nixos)](https://nixos.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Logic](https://img.shields.io/badge/Axioms-28%20Kern--Axiome-blueviolet?style=flat-square)](documentation/concept-v4/LOGIC.md)

<pre>
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│   "Ein axiomatisch fundiertes System für dezentrales            │
│    Vertrauen – mathematisch garantiert, human-aligned,          │
│    manipulationsresistent."                                     │
│                                                                 │
│   𝔼 = Σ 𝔸(s) · σ⃗( ‖𝕎(s)‖ · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
</pre>

[Schnellstart](#-schnellstart) · [Konzept](#-was-ist-erynoa) · [Dokumentation](#-dokumentation) · [Befehle](#-befehle)

</div>

---

## 🧠 Was ist Erynoa?

Erynoa ist ein **axiomatisch fundiertes, dezentrales System** für Vertrauen zwischen Menschen, Organisationen und KI-Agenten. Es basiert auf **28 formal definierten Kern-Axiomen**, die zusammen eine vollständige und widerspruchsfreie Logik für dezentrale Kooperation bilden.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                      ERYNOA V4.1 – KERNKONZEPTE                             │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐   │
│   │                        REALM-HIERARCHIE (Κ1)                        │   │
│   │                                                                     │   │
│   │              ROOT-REALM (Universelle Axiome Κ1-Κ28)                 │   │
│   │                           │                                         │   │
│   │            ┌──────────────┼──────────────┐                          │   │
│   │            ▼              ▼              ▼                          │   │
│   │      VIRTUAL-REALM  VIRTUAL-REALM  VIRTUAL-REALM                    │   │
│   │      (Knowledge)    (Finance)      (Governance)                     │   │
│   │            │              │              │                          │   │
│   │         ┌──┴──┐        ┌──┴──┐        ┌──┴──┐                       │   │
│   │         ▼     ▼        ▼     ▼        ▼     ▼                       │   │
│   │     Partition  ...  Partition  ...  Partition  ...                  │   │
│   │                                                                     │   │
│   └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│   ┌───────────────────────┐  ┌───────────────────────┐                      │
│   │   TRUST-VEKTOR 𝕎      │  │   WELTFORMEL 𝔼        │                      │
│   │   6-dimensional:      │  │                       │                      │
│   │   • Reliability       │  │   𝔼 = Σ 𝔸·σ⃗(𝕎·𝒮)·Ĥ·w  │                      │
│   │   • Integrity         │  │                       │                      │
│   │   • Competence        │  │   Trust-gedämpfte     │                      │
│   │   • Prestige          │  │   Surprisal + Human-  │                      │
│   │   • Vigilance         │  │   Alignment-Faktor    │                      │
│   │   • Omega (Axiom-Treue)│  │                       │                      │
│   └───────────────────────┘  └───────────────────────┘                      │
│                                                                             │
│   EIGENSCHAFTEN:                                                            │
│   ✓ Dezentral (P2P)        ✓ Skalierbar (Milliarden Entitäten)             │
│   ✓ Human-Aligned (Ĥ)      ✓ Formal Verifiziert (TLA+)                     │
│   ✓ Anti-Gaming            ✓ Asymmetrische Trust-Dynamik                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

> 📖 **Mehr erfahren:** [Fachkonzept](documentation/concept-v4/FACHKONZEPT.md) · [Logic V4.1](documentation/concept-v4/LOGIC.md) · [System-Architektur](documentation/concept-v4/SYSTEM-ARCHITECTURE.md)

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

### 🧠 Konzept & Protokoll (V4.1)

| Dokument                                                                     | Beschreibung                                              |
| ---------------------------------------------------------------------------- | --------------------------------------------------------- |
| **[📋 Fachkonzept](documentation/concept-v4/FACHKONZEPT.md)**                | **Fließtext-Übersicht** – Vision, Konzepte, Use Cases     |
| **[🔢 LOGIC.md](documentation/concept-v4/LOGIC.md)**                         | **28 Kern-Axiome + 4 Unter-Axiome** – Formale Logik       |
| **[🏗️ System-Architektur](documentation/concept-v4/SYSTEM-ARCHITECTURE.md)** | **Implementierbare Architektur** – 4 Schichten, Rust-Code |

<details>
<summary><strong>📊 Axiom-Übersicht (Κ1-Κ28)</strong></summary>

| Kategorie                  | Axiome          | Beschreibung                                  |
| -------------------------- | --------------- | --------------------------------------------- |
| **Kategorien-Algebra**     | Κ1              | Regelvererbung in Realm-Hierarchie            |
| **Trust-Algebra**          | Κ2-Κ5           | 6D-Vektor, Asymmetrie, Kombination            |
| **Identität & Delegation** | Κ6-Κ8           | DIDs, Capability-basierte Delegation          |
| **Kausalität & Finalität** | Κ9-Κ12          | DAG-Struktur, Finalitätsspektrum              |
| **Wert & Atomizität**      | Κ13-Κ14         | Wert-Äquivalenz, Saga-Pattern                 |
| **Weltformel**             | Κ15a-d, Κ16-Κ17 | 𝔼-Berechnung, Human-Alignment, Temporal       |
| **Konsens & Schutz**       | Κ18-Κ21         | Partition-Wahrheit, Anti-Degeneration         |
| **Peer-Logik**             | Κ22-Κ24         | Intent→Saga, Gateway Guards                   |
| **System-Garantien**       | Κ25-Κ28         | Determinismus, Offenheit, Verhältnismäßigkeit |

</details>

### 🗺️ Roadmap & Archiv

| Dokument                                   | Beschreibung                   |
| ------------------------------------------ | ------------------------------ |
| **[🗺️ Roadmap](documentation/ROADMAP.md)** | Strategischer Entwicklungsplan |
| [Concept V2](documentation/concept-v2/)    | Archiv: 7-Schichten-Navigator  |
| [Concept V3](documentation/concept-v3/)    | Archiv: EIPs, Protocol Spec    |

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

### 📁 Dokumentations-Struktur

```
documentation/
├── ROADMAP.md                    # 🗺️ Strategische Roadmap
│
├── concept-v4/                   # 🧠 AKTUELL: Unified Logic Framework V4.1
│   ├── FACHKONZEPT.md           # ⭐ Fließtext für Stakeholder
│   ├── LOGIC.md                  # ⭐ 28 Kern-Axiome + Weltformel
│   └── SYSTEM-ARCHITECTURE.md    # ⭐ Implementierbare Architektur
│
├── concept-v3/                   # 📦 Archiv: EIPs, Protocol Spec
├── concept-v2/                   # 📦 Archiv: 7-Schichten-Navigator
│
└── system/                       # 🛠️ Plattform & Entwicklung
    ├── readme.md
    ├── essential_guide.md
    ├── guides/
    ├── setup/
    ├── reference/
    └── development/
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
│   ├── concept-v4/       # 🧠 Unified Logic Framework V4.1
│   ├── concept-v3/       # 📦 Archiv
│   ├── concept-v2/       # 📦 Archiv
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

- ✅ **Unified Logic Framework V4.1** (28 Kern-Axiome)
- ✅ **Weltformel V2.0** mit Trust-gedämpfter Surprisal
- ✅ **System-Architektur** (4-Schichten, formal verifiziert)
- ✅ Connect-RPC API (Protobuf)
- ✅ Monorepo mit pnpm & Turborepo
- ✅ SvelteKit Frontends (Svelte 5)
- ✅ ZITADEL Auth mit automatischem Setup
- ✅ Caddy Reverse Proxy
- ✅ DevContainer Support
- ✅ GitHub Actions CI/CD
- ✅ Nix Flakes Dev Environment

### 🔄 In Arbeit

- Event Engine (DAG-Struktur)
- Trust Engine (6D-Vektor)
- P2P Networking (libp2p)
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

</div>
