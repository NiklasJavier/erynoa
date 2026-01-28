<div align="center">

<br>

# 🌊 Erynoa

### Kybernetisches Protokoll für die Maschinenökonomie

<br>

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0-FF3E00?style=for-the-badge&logo=svelte&logoColor=white)](https://kit.svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?style=for-the-badge&logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)](LICENSE)

<br>

**Maschinen verhandeln autonom · Vertrauen ist berechenbar · Keine Vermittler**

<br>

[**Schnellstart**](#-schnellstart) · [**Konzept**](#-das-protokoll) · [**Dokumentation**](#-dokumentation) · [**Befehle**](#-befehle)

<br>

</div>

---

<br>

## 🎯 Das Protokoll

> **Erynoa** ermöglicht autonomen Agenten, vertrauensbasierte Transaktionen ohne zentrale Vermittler durchzuführen.

<br>

<div align="center">

```
                              ╭─────────────╮
                              │    ECHO     │
                              │   ───────   │
                              │   Agenten   │
                              │  Verhandlung│
                              ╰──────┬──────╯
                                     │
                    ╭────────────────┼────────────────╮
                    │                │                │
                    ▼                │                ▼
             ╭─────────────╮         │         ╭─────────────╮
             │     ERY     │◀────────┴────────▶│     NOA     │
             │   ───────   │                   │   ───────   │
             │  Semantik   │                   │  Finalität  │
             │    Trust    │                   │  Settlement │
             ╰─────────────╯                   ╰─────────────╯

                        E R Y   +   N O A
                      ═══════════════════════
                            E R Y N O A
```

</div>

<br>

### Die drei Sphären

|  Sphäre  | Aufgabe                           | Kerntechnologie        |
| :------: | :-------------------------------- | :--------------------- |
| **ERY**  | Semantik · Trust · Discovery      | Qdrant · Karmic Engine |
| **ECHO** | Agenten · P2P · Verhandlung       | WASM · libp2p · XMTP   |
| **NOA**  | Finalität · Settlement · Wahrheit | MoveVM · Starfish BFT  |

<br>

---

<br>

## ⚡ Schnellstart

<table>
<tr>
<td>

**Voraussetzungen**

</td>
<td>

[Nix](https://nixos.org/) · [Docker Desktop](https://www.docker.com/products/docker-desktop/)

</td>
</tr>
</table>

```bash
# Repository klonen
git clone git@github.com:NiklasJavier/erynoa.git && cd erynoa

# Dev-Shell betreten (lädt alle Tools)
nix develop

# Starten
just dev
```

<br>

<div align="center">

⏳ **~2 Minuten warten** → 🌐 **http://localhost:3001** öffnen

</div>

<br>

<details>
<summary><strong>🔗 Alle Services & Test-Zugänge</strong></summary>

<br>

| Service      | URL                            | Beschreibung   |
| :----------- | :----------------------------- | :------------- |
| 🌐 **Proxy** | http://localhost:3001          | Hauptzugang    |
| 📊 Console   | http://localhost:3001/console  | Admin-Bereich  |
| 🖥️ Platform  | http://localhost:3001/platform | Hauptplattform |
| 📖 Docs      | http://localhost:3001/docs     | Dokumentation  |
| 🔌 API       | http://localhost:3001/api      | Backend-API    |
| 🔐 ZITADEL   | http://localhost:8080          | Auth-Server    |
| 📦 MinIO     | http://localhost:9001          | Object Storage |

<br>

**Test-Zugänge:**

| Rolle | User            | Passwort     |
| :---- | :-------------- | :----------- |
| User  | `testuser`      | `Test123!`   |
| Admin | `zitadel-admin` | `Password1!` |

</details>

<br>

---

<br>

## 📖 Dokumentation

<br>

<div align="center">

|     | Konzept                                                                    |     | System                                                            |
| :-: | :------------------------------------------------------------------------- | :-: | :---------------------------------------------------------------- |
| 📋  | [**Fachkonzept**](documentation/concept/fachkonzept.md) ⭐                 | 📚  | [**System-Übersicht**](documentation/system/readme.md)            |
| 🎯  | [Kernkonzept](documentation/concept/kernkonzept.md)                        | ⚡  | [Essential Guide](documentation/system/essential_guide.md)        |
| 🏗️  | [Systemarchitektur](documentation/concept/system-architecture-overview.md) | 🚀  | [Getting Started](documentation/system/guides/getting-started.md) |
| 💧  | [Liquides Datenmodell](documentation/concept/liquides-datenmodell.md)      | 🔧  | [Setup](documentation/system/setup/setup.md)                      |
| 🤝  | [Trust & Reputation](documentation/concept/trust-and-reputation.md)        | 🏛️  | [Architecture](documentation/system/reference/architecture.md)    |
| 🔄  | [Cybernetic Loop](documentation/concept/cybernetic-loop.md)                | ⚙️  | [Configuration](documentation/system/reference/config.md)         |
| 🤖  | [Agents & ADL](documentation/concept/agents-and-adl.md)                    | 📝  | [Style Guide](documentation/system/development/style-guide.md)    |
| 💡  | [Use Cases](documentation/concept/use-cases.md)                            | 🧪  | [Testing](documentation/system/development/testing.md)            |
| 📖  | [Glossar](documentation/concept/glossary.md)                               | ✅  | [TODOs](documentation/system/development/todos.md)                |

</div>

<br>

> 💡 **Empfehlung:** Starte mit dem [Fachkonzept](documentation/concept/fachkonzept.md) für einen vollständigen Überblick.

<br>

---

<br>

## 🛠️ Tech Stack

<br>

<table>
<tr>
<td width="50%" valign="top">

### Backend

|              | Technologie            |
| :----------- | :--------------------- |
| 🦀 Runtime   | **Rust** · Tokio       |
| 🌐 Framework | Axum                   |
| 📡 API       | Connect-RPC (Protobuf) |
| 🗄️ Database  | PostgreSQL (OrioleDB)  |
| ⚡ Cache     | DragonflyDB            |
| 📦 Storage   | MinIO (S3)             |
| 🔐 Auth      | ZITADEL (OIDC)         |

</td>
<td width="50%" valign="top">

### Frontend

|              | Technologie              |
| :----------- | :----------------------- |
| 🎨 Framework | **SvelteKit** (Svelte 5) |
| 💅 Styling   | Tailwind CSS             |
| ⚡ Build     | Vite · Turborepo         |
| 📦 Packages  | pnpm                     |
| ✨ Linting   | Biome                    |
| 📘 Types     | TypeScript               |

</td>
</tr>
</table>

<br>

### Infrastruktur

|     | Technologie    | Zweck                        |
| :-- | :------------- | :--------------------------- |
| ❄️  | Nix Flakes     | Reproduzierbare Dev-Umgebung |
| 🐳  | Docker Compose | Container-Orchestrierung     |
| 🔀  | Caddy          | Reverse Proxy                |
| ⚙️  | just           | Task Runner                  |
| 📋  | buf            | Protobuf Code-Gen            |

<br>

---

<br>

## 📁 Projektstruktur

```
erynoa/
│
├── 🦀 backend/                    Rust API Server
│   ├── src/                       Source Code
│   ├── config/                    TOML Konfiguration
│   ├── migrations/                SQL Migrations
│   └── proto/                     Protobuf Definitionen
│
├── 🎨 frontend/                   SvelteKit Apps
│   ├── console/                   Admin Console
│   ├── platform/                  Hauptplattform
│   └── docs/                      Dokumentations-Site
│
├── 📖 documentation/              Dokumentation
│   ├── concept/                   Protokoll & Konzept
│   └── system/                    Plattform & Entwicklung
│
├── 🏗️ infra/                      Infrastruktur
│   ├── docker/                    Docker Compose & Images
│   ├── proxy/                     Caddy Config
│   └── auth/                      ZITADEL Setup
│
├── 🔧 scripts/                    Build & Dev Scripts
│
├── ❄️ flake.nix                   Nix Environment
├── ⚙️ justfile                    Task Commands
└── 📋 buf.yaml                    Protobuf Config
```

<br>

---

<br>

## 🔧 Befehle

<br>

### Entwicklung

| Befehl             | Aktion                |
| :----------------- | :-------------------- |
| `just dev`         | 🚀 **Alles starten**  |
| `just dev console` | Console starten       |
| `just status`      | Service-Status        |
| `just logs`        | Logs anzeigen         |
| `just stop`        | Stoppen               |
| `just reset`       | Komplett zurücksetzen |

<br>

### Backend

| Befehl       | Aktion            |
| :----------- | :---------------- |
| `just check` | Cargo check       |
| `just lint`  | Clippy            |
| `just fmt`   | Formatieren       |
| `just test`  | Tests             |
| `just ci`    | CI-Pipeline lokal |

<br>

### Setup

| Befehl               | Aktion             |
| :------------------- | :----------------- |
| `just init`          | Initialisieren     |
| `just init-env`      | .env erstellen     |
| `just zitadel-setup` | Auth konfigurieren |
| `just proto-gen`     | Types generieren   |

<br>

<details>
<summary><strong>📋 Alle Befehle</strong></summary>

```bash
just --list
```

</details>

<br>

---

<br>

## 📊 Status

<br>

<table>
<tr>
<td width="50%" valign="top">

### ✅ Implementiert

- Connect-RPC API (Protobuf)
- Monorepo (pnpm + Turborepo)
- SvelteKit Frontends (Svelte 5)
- ZITADEL Auth (auto-setup)
- Caddy Reverse Proxy
- DevContainer Support
- GitHub Actions CI/CD
- Nix Flakes Environment

</td>
<td width="50%" valign="top">

### 🔄 In Arbeit

- Frontend Tests
- E2E Testing
- Performance Monitoring

<br>

→ Details: [TODOs](documentation/system/development/todos.md)

</td>
</tr>
</table>

<br>

---

<br>

## 🤝 Contributing

1. **[TODOs](documentation/system/development/todos.md)** prüfen
2. **[Style Guide](documentation/system/development/style-guide.md)** befolgen
3. **[Tests](documentation/system/development/testing.md)** schreiben

<br>

---

<br>

## 📞 Hilfe

| Problem         | Lösung                                                       |
| :-------------- | :----------------------------------------------------------- |
| Allgemein       | [Essential Guide](documentation/system/essential_guide.md)   |
| Bekannte Issues | [TODOs](documentation/system/development/todos.md)           |
| Services        | [Connections](documentation/system/reference/connections.md) |

<br>

---

<br>

<div align="center">

<br>

```
    ╭───────────────────────────────────────────╮
    │                                           │
    │         E R Y   +   N O A                 │
    │      ═════════════════════════            │
    │           E R Y N O A                     │
    │                                           │
    │    Semantic Lattice + Causal Ledger       │
    │       Wissen + Wahrheit = Vertrauen       │
    │                                           │
    ╰───────────────────────────────────────────╯
```

<br>

**[MIT License](LICENSE)**

Made with ❤️ and 🦀

<br>

</div>
