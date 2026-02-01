# Erynoa – Essential Guide

> **Dokumenttyp:** Referenz
> **Version:** 2.0
> **Status:** Aktiv
> **Lesezeit:** ca. 10 Minuten

---

## Auf einen Blick

Dieser Guide enthält **alles Wichtige** für die tägliche Entwicklung mit Erynoa – Quick Start, Befehle, Konfiguration, Troubleshooting.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📋 ESSENTIAL GUIDE – INHALTSÜBERSICHT                                    │
│                                                                             │
│   ┌───────────────┐  ┌───────────────┐  ┌───────────────┐                  │
│   │  ⚡ START     │  │  🔧 BEFEHLE   │  │  🔗 SERVICES  │                  │
│   │  Quick Start  │  │  just dev     │  │  URLs & Ports │                  │
│   │  3 Schritte   │  │  Alle Befehle │  │  Verbindungen │                  │
│   └───────────────┘  └───────────────┘  └───────────────┘                  │
│                                                                             │
│   ┌───────────────┐  ┌───────────────┐  ┌───────────────┐                  │
│   │  ⚙️ CONFIG    │  │  🐛 DEBUG     │  │  📊 STATUS    │                  │
│   │  Backend      │  │  Trouble-     │  │  Features     │                  │
│   │  Frontend     │  │  shooting     │  │  Roadmap      │                  │
│   └───────────────┘  └───────────────┘  └───────────────┘                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## ⚡ Quick Start

### Voraussetzungen

| Tool       | Installation                                                               |
| :--------- | :------------------------------------------------------------------------- |
| **Nix**    | `curl -sSf -L https://install.determinate.systems/nix \| sh -s -- install` |
| **Docker** | [Docker Desktop](https://www.docker.com/products/docker-desktop/)          |

### 3 Schritte

```bash
# 1. Klonen
git clone git@github.com:NiklasJavier/erynoa.git && cd erynoa

# 2. Dev-Shell (lädt alle Tools)
nix develop

# 3. Starten
just dev
```

<div align="center">

⏳ **~2 Minuten warten** → 🌐 **http://localhost:3001**

</div>

### Test-Zugänge

| Rolle | User            | Passwort     |
| :---- | :-------------- | :----------- |
| User  | `testuser`      | `Test123!`   |
| Admin | `zitadel-admin` | `Password1!` |

---

## 🔗 Services & URLs

### Entwicklungs-URLs

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🌐 HAUPTZUGANG: http://localhost:3001                                    │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   /console   ───▶  Admin Console                                   │  │
│   │   /platform  ───▶  Hauptplattform                                  │  │
│   │   /docs      ───▶  Dokumentation                                   │  │
│   │   /api       ───▶  Backend API                                     │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

| Service      | URL                            | Beschreibung        |
| :----------- | :----------------------------- | :------------------ |
| 🌐 **Proxy** | http://localhost:3001          | Caddy Reverse Proxy |
| 📊 Console   | http://localhost:3001/console  | Admin-Bereich       |
| 🖥️ Platform  | http://localhost:3001/platform | Hauptplattform      |
| 📖 Docs      | http://localhost:3001/docs     | Dokumentation       |
| 🔌 API       | http://localhost:3001/api      | Backend API         |
| 🦀 Backend   | http://localhost:3000          | Direkt (für Tests)  |
| 🔐 ZITADEL   | http://localhost:8080          | Auth Server         |
| 📦 MinIO     | http://localhost:9001          | Storage Console     |

### Interne Docker-Namen

| Service  | Name      | Connection String                           |
| :------- | :-------- | :------------------------------------------ |
| Database | `db`      | `postgresql://erynoa:erynoa@db:5432/erynoa` |
| Cache    | `cache`   | `redis://cache:6379`                        |
| Storage  | `minio`   | `http://minio:9000`                         |
| Auth     | `zitadel` | `http://zitadel:8080`                       |

---

## 🔧 Befehle

### Entwicklung

| Befehl              | Beschreibung          |
| :------------------ | :-------------------- |
| `just dev`          | 🚀 **Startet alles**  |
| `just dev console`  | Nur Console           |
| `just dev platform` | Nur Platform          |
| `just status`       | Service-Status        |
| `just logs`         | Alle Logs             |
| `just logs backend` | Backend-Logs          |
| `just stop`         | Container stoppen     |
| `just restart`      | Neustart              |
| `just reset`        | Komplett zurücksetzen |

### Backend

| Befehl       | Beschreibung          |
| :----------- | :-------------------- |
| `just check` | Cargo check           |
| `just lint`  | Clippy                |
| `just fmt`   | Formatieren           |
| `just test`  | Tests (cargo-nextest) |
| `just ci`    | fmt + lint + test     |

### Frontend

| Befehl                | Beschreibung     |
| :-------------------- | :--------------- |
| `just frontend-lint`  | Biome Lint       |
| `just frontend-check` | TypeScript Check |

### Setup & Tools

| Befehl               | Beschreibung                     |
| :------------------- | :------------------------------- |
| `just init`          | Initialisieren (ohne Dev-Server) |
| `just init-env`      | .env erstellen                   |
| `just zitadel-setup` | Auth konfigurieren               |
| `just zitadel-reset` | Auth zurücksetzen                |
| `just proto-gen`     | Protobuf Types generieren        |
| `just services`      | Nur Hintergrund-Services         |

<details>
<summary><strong>📋 Alle Befehle</strong></summary>

```bash
just --list
```

</details>

---

## 🏗️ Architektur

### Projektstruktur

```
erynoa/
│
├── 🦀 backend/                    Rust API Server
│   ├── src/api/v1/                Feature-basierte API
│   │   ├── health/                Health Checks
│   │   ├── info/                  Info & Status
│   │   ├── users/                 User Management
│   │   └── storage/               Storage Operations
│   ├── config/                    TOML Konfiguration
│   └── proto/                     Protobuf Definitionen
│
├── 🎨 frontend/                   SvelteKit Apps
│   ├── console/                   Admin Console
│   ├── platform/                  Hauptplattform
│   └── docs/                      Dokumentation
│
├── 📖 documentation/              Dokumentation
│   ├── concept/                   Protokoll & Konzept
│   └── system/                    Plattform & Entwicklung
│
└── 🏗️ infra/                      Infrastruktur
    ├── docker/                    Docker Compose
    ├── proxy/                     Caddy Config
    └── auth/                      ZITADEL Setup
```

### Tech Stack

| Bereich            | Technologie                     |
| :----------------- | :------------------------------ |
| **Backend**        | Rust · Axum · Tokio · SQLx      |
| **API**            | Connect-RPC (Protobuf)          |
| **Frontend**       | SvelteKit · Svelte 5 · Tailwind |
| **Database**       | PostgreSQL (OrioleDB)           |
| **Cache**          | DragonflyDB (Redis)             |
| **Storage**        | MinIO (S3)                      |
| **Auth**           | ZITADEL (OIDC/JWT)              |
| **Orchestrierung** | Restate                         |

---

## ⚙️ Konfiguration

### Backend (`backend/config/base.toml`)

```toml
[application]
api_url = "http://localhost:3000"
console_url = "http://localhost:3001/console"

[database]
host = "db"
port = 5432
username = "erynoa"
password = "erynoa"
database = "erynoa"

[cache]
url = "redis://cache:6379"

[storage]
endpoint = "http://minio:9000"
access_key_id = "erynoa"
secret_access_key = "erynoa123"
default_bucket = "erynoa"

[auth]
issuer = "http://localhost:8080"
internal_issuer = "http://zitadel:8080"
```

### Konfigurationspriorität

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   1. Umgebungsvariablen    APP_DATABASE__HOST=db            │
│   2. local.toml            (auto-generated, gitignored)      │
│   3. base.toml             (Standard-Werte)                  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 🔌 Connect-RPC

### Status: ✅ Aktiv

| Service          | Beschreibung       |
| :--------------- | :----------------- |
| `HealthService`  | Health Checks      |
| `InfoService`    | Info & Status      |
| `UserService`    | User Management    |
| `StorageService` | Storage Operations |

### Protobuf generieren

```bash
just proto-gen
```

---

## 🔐 ZITADEL Auth

### Automatisches Setup

ZITADEL wird beim ersten `just dev` automatisch konfiguriert:

- Projekt `erynoa` erstellt
- OIDC Applications für alle Frontends
- Test-User `testuser` / `Test123!`
- `backend/config/local.toml` aktualisiert

### Manuelles Setup

```bash
just zitadel-setup    # Neu konfigurieren
just zitadel-reset    # Zurücksetzen
```

---

## 🐛 Troubleshooting

### Services starten nicht

```bash
just reset
just dev
```

### ZITADEL Client-ID ungültig

```bash
just zitadel-reset
```

### Backend kompiliert nicht

```bash
just shell backend
cargo check
```

### Port bereits belegt

```bash
just stop
lsof -i :3000
```

### Logs prüfen

```bash
just logs              # Alle
just logs backend      # Backend
just logs console      # Console
```

### Häufige Probleme

| Problem                | Lösung                       |
| :--------------------- | :--------------------------- |
| Services starten nicht | `just reset && just dev`     |
| Auth-Fehler            | `just zitadel-reset`         |
| Port belegt            | `just stop && lsof -i :PORT` |
| Build-Fehler           | `cargo check` / `pnpm check` |
| Docker-Probleme        | Docker Desktop neustarten    |

---

## 📝 Code Standards

### Naming Conventions

| Sprache        | Functions    | Types        | Files           |
| :------------- | :----------- | :----------- | :-------------- |
| **Rust**       | `snake_case` | `PascalCase` | `snake_case.rs` |
| **TypeScript** | `camelCase`  | `PascalCase` | `kebab-case.ts` |

### Backend API Struktur

```
api/v1/{feature}/
├── handler.rs       # REST handlers
├── connect.rs       # Connect-RPC handlers
├── models.rs        # Request/Response types
├── routes.rs        # Route definitions
└── mod.rs           # Module exports
```

→ Vollständig: [Style Guide](development/style-guide.md)

---

## 📊 Status

### ✅ Implementiert

| Feature                        | Status |
| :----------------------------- | :----- |
| Connect-RPC API                | ✅     |
| Monorepo (pnpm + Turborepo)    | ✅     |
| SvelteKit Frontends (Svelte 5) | ✅     |
| ZITADEL Auth (auto-setup)      | ✅     |
| Caddy Reverse Proxy            | ✅     |
| DevContainer Support           | ✅     |
| GitHub Actions CI/CD           | ✅     |
| Nix Flakes Environment         | ✅     |
| Protobuf Code-Gen              | ✅     |
| **libp2p NAT-Traversal**       | ✅     |
| **Cold Storage / Archive**     | ✅     |
| **Adaptive Kalibrierung**      | ✅     |
| **409 Backend-Tests**          | ✅     |

### 🔄 In Arbeit

- Frontend Tests
- Performance Monitoring
- Extended Error-Tracking

### 📅 Geplant

- REST Endpoints deprecaten
- API Documentation
- Type Definitions Cleanup

→ Details: [TODOs](development/todos.md)

---

## 🧠 Protokoll-Konzepte

Die System-Dokumentation fokussiert auf die **Implementierung**. Für das **Protokoll-Design** siehe:

| Dokument                                                           | Inhalt                        |
| :----------------------------------------------------------------- | :---------------------------- |
| [📋 Fachkonzept](../concept/fachkonzept.md)                        | Vollständige Spezifikation    |
| [🎯 Kernkonzept](../concept/kernkonzept.md)                        | High-Level Überblick          |
| [🏗️ Systemarchitektur](../concept/system-architecture-overview.md) | Drei-Sphären (ERY, ECHO, NOA) |
| [📖 Glossar](../concept/glossary.md)                               | Begriffsdefinitionen          |

---

## 📚 Weiterführende Dokumente

| Dokument                                  | Beschreibung                 |
| :---------------------------------------- | :--------------------------- |
| [Setup Guide](setup/setup.md)             | Vollständige Setup-Anleitung |
| [Architecture](reference/architecture.md) | System-Architektur           |
| [Configuration](reference/config.md)      | Service-Konfiguration        |
| [Style Guide](development/style-guide.md) | Code Standards               |
| [Testing](development/testing.md)         | Test-Strategien              |
| [TODOs](development/todos.md)             | Offene Aufgaben              |
| [ZITADEL Guide](guides/zitadel.md)        | Auth-Setup                   |

---

<div align="center">

```
┌─────────────────────────────────────────────┐
│                                             │
│   Probleme?                                 │
│                                             │
│   1. Troubleshooting (oben)                 │
│   2. just logs prüfen                       │
│   3. TODOs durchsuchen                      │
│   4. Issue erstellen                        │
│                                             │
└─────────────────────────────────────────────┘
```

**Letzte Aktualisierung:** Februar 2026 (Priorität 3 abgeschlossen)

</div>
