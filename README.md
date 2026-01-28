<div align="center">

# Erynoa EU inc

**Full-Stack Application mit Rust Backend und SvelteKit Frontends**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![SvelteKit](https://img.shields.io/badge/SvelteKit-2.0-FF3E00?style=flat-square&logo=svelte)](https://kit.svelte.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![Nix](https://img.shields.io/badge/Nix-Flakes-5277C3?style=flat-square&logo=nixos)](https://nixos.org/)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

[Schnellstart](#-schnellstart) •
[Dokumentation](docs/readme.md) •
[Tech Stack](#-tech-stack) •
[Befehle](#-befehle)

</div>

---

## ⚡ Schnellstart

> **Voraussetzungen:** [Nix](https://nixos.org/) und [Docker Desktop](https://www.docker.com/products/docker-desktop/) installiert
>
> → Detaillierte Installationsanleitung: [Setup Guide](docs/setup/setup.md)

```bash
# 1. Repository klonen
git clone git@github.com:NiklasJavier/erynoa.git && cd erynoa

# 2. Nix Dev-Shell betreten (lädt alle Tools automatisch)
nix develop

# 3. Projekt starten
just dev
```

**Warte ~2 Minuten** ⏳ → Dann öffne **<http://localhost:3001>**

<details>
<summary><strong>🔗 Alle URLs & Test-Login</strong></summary>

| Service                 | URL                            |
| ----------------------- | ------------------------------ |
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
├── infra/                # 🏗 Infrastructure
│   ├── docker/           # Docker Compose & Dockerfiles
│   ├── proxy/            # Caddy Reverse Proxy
│   ├── auth/             # ZITADEL Config
│   └── static/           # Static Files
│
├── docs/                 # 📚 Dokumentation
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

## 📖 Dokumentation

| Dokument                                          | Beschreibung                   |
| ------------------------------------------------- | ------------------------------ |
| **[📚 Docs Overview](docs/readme.md)**            | Dokumentations-Übersicht       |
| **[⚡ Essential Guide](docs/essential_guide.md)** | Alles Wichtige auf einen Blick |

### Guides

| Guide                                             | Beschreibung                    |
| ------------------------------------------------- | ------------------------------- |
| [Getting Started](docs/guides/getting-started.md) | Erste Schritte                  |
| [Setup](docs/setup/setup.md)                      | Entwicklungsumgebung einrichten |
| [ZITADEL](docs/guides/zitadel.md)                 | Authentifizierung               |

### Reference

| Dokument                                       | Beschreibung          |
| ---------------------------------------------- | --------------------- |
| [Architecture](docs/reference/architecture.md) | Systemarchitektur     |
| [Configuration](docs/reference/config.md)      | Service-Konfiguration |
| [Connections](docs/reference/connections.md)   | API-Verbindungen      |

### Development

| Dokument                                       | Beschreibung    |
| ---------------------------------------------- | --------------- |
| [Style Guide](docs/development/style-guide.md) | Code-Stil       |
| [Testing](docs/development/testing.md)         | Test-Strategien |
| [TODOs](docs/development/todos.md)             | Offene Aufgaben |

---

## 📊 Status

### ✅ Features

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
- Siehe [TODOs](docs/development/todos.md)

---

## 🤝 Contributing

1. Prüfe [TODOs](docs/development/todos.md) für offene Aufgaben
2. Folge dem [Style Guide](docs/development/style-guide.md)
3. Schreibe Tests ([Testing Guide](docs/development/testing.md))

---

## 📞 Support

Bei Problemen:

1. [Essential Guide](docs/essential_guide.md) - Troubleshooting
2. [TODOs](docs/development/todos.md) - Bekannte Issues
3. [Connections](docs/reference/connections.md) - Service-Probleme

---

<div align="center">

**[MIT License](LICENSE)**

Made with ❤️ and 🦀

</div>
