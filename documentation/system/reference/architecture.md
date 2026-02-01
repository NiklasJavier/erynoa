# Erynoa – System-Architektur

> **Dokumenttyp:** Referenz
> **Bereich:** Plattform-Architektur
> **Status:** Aktiv
> **Lesezeit:** ca. 15 Minuten

---

## Übersicht

Dieses Dokument beschreibt die **technische Plattform-Architektur** – Frontend, Backend, Infrastruktur und deren Zusammenspiel.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📐 ARCHITEKTUR-EBENEN                                                     │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   🎨 FRONTEND              🦀 BACKEND              🏗️ INFRA         │  │
│   │   ───────────              ──────────              ─────────        │  │
│   │   SvelteKit                Rust/Axum              Docker            │  │
│   │   TypeScript               Connect-RPC            PostgreSQL        │  │
│   │   Tailwind                 SQLx                   ZITADEL           │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   💡 Protokoll-Konzepte (ERY, ECHO, NOA) → documentation/concept/          │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

> 💡 **Hinweis:** Für die **Protokoll-Architektur** (ERY/ECHO/NOA, Cybernetic Loop) siehe [Fachkonzept](../../concept/fachkonzept.md).

---

## Tech Stack

| Schicht       | Technologie            | Beschreibung             |
| :------------ | :--------------------- | :----------------------- |
| **Frontend**  | SvelteKit · TypeScript | 3 Apps im Monorepo       |
| **Backend**   | Rust · Axum            | High-Performance API     |
| **API**       | Connect-RPC (Protobuf) | End-to-End Typsicherheit |
| **Workflows** | Restate                | Durable Orchestrierung   |
| **Auth**      | ZITADEL                | OIDC/JWT                 |
| **Database**  | PostgreSQL (OrioleDB)  | Persistenz               |
| **Cache**     | DragonflyDB            | Redis-kompatibel         |
| **Storage**   | MinIO                  | S3-kompatibel            |
| **Proxy**     | Caddy                  | Reverse Proxy            |

---

## System-Diagramm

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                              🌐 BROWSER                                     │
│                                   │                                         │
│                                   ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                      🔀 CADDY PROXY (:3001)                         │  │
│   │                                                                     │  │
│   │    /console ─────▶ Console (:5173)                                 │  │
│   │    /platform ────▶ Platform (:5174)                                │  │
│   │    /docs ────────▶ Docs (:5175)                                    │  │
│   │    /api ─────────▶ Backend (:3000)                                 │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                   │                                         │
│         ┌─────────────────────────┼─────────────────────────┐              │
│         │                         │                         │              │
│         ▼                         ▼                         ▼              │
│   ┌───────────┐            ┌───────────┐            ┌───────────┐         │
│   │  Console  │            │ Platform  │            │   Docs    │         │
│   │  :5173    │            │  :5174    │            │  :5175    │         │
│   │ SvelteKit │            │ SvelteKit │            │ SvelteKit │         │
│   └───────────┘            └───────────┘            └───────────┘         │
│                                   │                                         │
│                                   ▼                                         │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                      🦀 BACKEND (:3000)                             │  │
│   │                        Rust · Axum · SQLx                           │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                   │                                         │
│         ┌─────────────────────────┼─────────────────────────┐              │
│         │                         │                         │              │
│         ▼                         ▼                         ▼              │
│   ┌───────────┐            ┌───────────┐            ┌───────────┐         │
│   │  ZITADEL  │            │ PostgreSQL│            │DragonflyDB│         │
│   │  :8080    │            │  :5432    │            │  :6379    │         │
│   │   Auth    │            │ OrioleDB  │            │  Cache    │         │
│   └───────────┘            └───────────┘            └───────────┘         │
│                                   │                                         │
│                                   ▼                                         │
│                            ┌───────────┐                                   │
│                            │   MinIO   │                                   │
│                            │ :9000/9001│                                   │
│                            │  Storage  │                                   │
│                            └───────────┘                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🎨 Frontend-Architektur

### Monorepo-Strategie

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📦 FRONTEND MONOREPO (pnpm Workspace + Turborepo)                        │
│                                                                             │
│   frontend/                                                                 │
│   ├── console/          📊 Admin Console                                   │
│   ├── platform/         🖥️ Hauptplattform                                  │
│   └── docs/             📖 Dokumentation                                   │
│                                                                             │
│   Shared:                                                                   │
│   • Dependencies (hardlinked via pnpm)                                     │
│   • Protobuf Types (src/gen/)                                              │
│   • Build Cache (Turborepo)                                                │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Tech Stack

| Komponente       | Version | Beschreibung         |
| :--------------- | :------ | :------------------- |
| **SvelteKit**    | 2.x     | Meta-Framework       |
| **Svelte**       | 5.x     | UI Framework (Runes) |
| **TypeScript**   | 5.x     | Type Safety          |
| **Tailwind CSS** | 3.x     | Styling              |
| **Vite**         | 5.x     | Build Tool           |
| **Biome**        | 1.x     | Linting & Formatting |

### Vorteile

| Feature                  | Beschreibung                |
| :----------------------- | :-------------------------- |
| **Shared Dependencies**  | Hardlinked via pnpm         |
| **Shared Types**         | Generierte Protobuf-Types   |
| **Parallele Builds**     | Turborepo mit Caching       |
| **Konsistente Struktur** | Gleiche Verzeichnisstruktur |

---

## 🦀 Backend-Architektur

### Tech Stack

| Komponente      | Version | Beschreibung             |
| :-------------- | :------ | :----------------------- |
| **Axum**        | 0.8     | Web Framework            |
| **Tokio**       | 1.x     | Async Runtime            |
| **SQLx**        | 0.8     | DB (Compile-Time Checks) |
| **Connect-RPC** | -       | gRPC-Web API             |
| **libp2p**      | 0.54    | P2P Networking           |
| **Fjall**       | -       | Embedded Key-Value Store |
| **Jemalloc**    | -       | Memory Allocator         |

### Fähigkeiten

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   🦀 BACKEND CAPABILITIES                                                   │
│                                                                             │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│   │  📡 API         │  │  🔄 Workflows   │  │  📄 Dokumente   │            │
│   │  ────────────   │  │  ────────────   │  │  ────────────   │            │
│   │  Rust/Axum      │  │  Restate        │  │  Typst          │            │
│   │  Connect-RPC    │  │  Durable Exec   │  │  PDF-Gen        │            │
│   │  SQLx           │  │  Retries/Timer  │  │  Templates      │            │
│   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│                                                                             │
│   ┌─────────────────┐  ┌─────────────────┐                                 │
│   │  📧 E-Mails     │  │  🔐 Auth        │                                 │
│   │  ────────────   │  │  ────────────   │                                 │
│   │  Lettre (SMTP)  │  │  ZITADEL       │                                 │
│   │  Rinja          │  │  JWT/OIDC      │                                 │
│   │  Templates      │  │  Token Valid.  │                                 │
│   └─────────────────┘  └─────────────────┘                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### API-Struktur

```
backend/src/api/
│
├── v1/                        API Version 1
│   ├── health/                Health Check
│   │   ├── handler.rs         REST handlers
│   │   ├── connect.rs         Connect-RPC handlers
│   │   ├── models.rs          Request/Response types
│   │   └── routes.rs          Route definitions
│   ├── info/                  Info & Status
│   ├── users/                 User Management
│   └── storage/               Storage Operations
│
├── middleware/                Middleware Layer
│   ├── auth.rs                JWT Validation
│   ├── cors.rs                CORS Config
│   └── logging.rs             Request Logging
│
└── shared/                    Shared Utilities
    └── pagination.rs
```

### Performance-Optimierungen

```toml
# Cargo.toml [profile.release]
strip = true      # Debug-Symbole entfernen
lto = "fat"       # Aggressive Link-Time Optimization
```

**Ergebnis:** Kleine Binaries, maximale Performance

---

## 🔄 Workflows & Orchestrierung

**Restate** für langlebige, fehlertolerante Abläufe:

| Feature                   | Beschreibung                         |
| :------------------------ | :----------------------------------- |
| **Durable Execution**     | Jeder Schritt persistent gespeichert |
| **Genau-einmal Semantik** | Idempotente externe Aufrufe          |
| **Stateful Workflows**    | Eigener, stark konsistenter Zustand  |
| **Zeitbasierte Events**   | Timer, Delays, Deadlines             |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   RESTATE WORKFLOW BEISPIEL                                  │
│                                                              │
│   User Request ──▶ [Step 1] ──▶ [Step 2] ──▶ [Step 3]       │
│                        │            │            │           │
│                        ▼            ▼            ▼           │
│                    Persistent   Persistent   Persistent      │
│                    State        State        State           │
│                                                              │
│   Bei Fehler: Wiederaufnahme ab letztem erfolgreichen Step  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 🔌 API-Kommunikation

### Connect-RPC (Protobuf)

End-to-End Typsicherheit zwischen Frontend und Backend:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📋 .proto                    ⚙️ buf gen                   📦 Output       │
│   ──────────                   ──────────                   ────────        │
│                                                                             │
│   service UserService {        ──────────▶                 TypeScript      │
│     rpc GetUser(...);                                       Types +        │
│     rpc CreateUser(...);                                    Clients        │
│   }                                                                         │
│         │                                                                   │
│         │                                                                   │
│         ▼                                                                   │
│   Rust Server Implementation                                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Vorteile

| Feature                    | Beschreibung                   |
| :------------------------- | :----------------------------- |
| **Single Source of Truth** | `.proto` definiert API         |
| **Auto-Generated Types**   | TypeScript-Clients automatisch |
| **Compile-Time Safety**    | Keine Runtime Type-Fehler      |
| **gRPC-Web**               | Browser-kompatibel             |

### Services

| Service          | Beschreibung       |
| :--------------- | :----------------- |
| `HealthService`  | Health Checks      |
| `InfoService`    | Info & Status      |
| `UserService`    | User Management    |
| `StorageService` | Storage Operations |

---

## 🏗️ Infrastruktur

### Services

| Service      | Port      | Technologie | Beschreibung     |
| :----------- | :-------- | :---------- | :--------------- |
| **Proxy**    | 3001      | Caddy       | Reverse Proxy    |
| **Backend**  | 3000      | Rust/Axum   | API Server       |
| **Console**  | 5173      | SvelteKit   | Admin UI         |
| **Platform** | 5174      | SvelteKit   | Main App         |
| **Docs**     | 5175      | SvelteKit   | Documentation    |
| **Database** | 5432      | PostgreSQL  | OrioleDB Engine  |
| **Cache**    | 6379      | DragonflyDB | Redis-kompatibel |
| **Storage**  | 9000/9001 | MinIO       | S3-kompatibel    |
| **Auth**     | 8080      | ZITADEL     | OIDC/JWT         |

### Development Tools

| Tool               | Beschreibung                  |
| :----------------- | :---------------------------- |
| **Nix Flakes**     | Reproduzierbare Toolchain     |
| **DevContainer**   | VS Code Container-Entwicklung |
| **Docker Compose** | Service-Orchestrierung        |
| **just**           | Task Runner                   |
| **Turborepo**      | Build-Caching                 |

### Caddy Routing

```
localhost:3001/
├── /console   → :5173
├── /platform  → :5174
├── /docs      → :5175
└── /api       → :3000
```

---

## 📁 Projektstruktur

```
erynoa/
│
├── 🦀 backend/                    Rust Backend
│   ├── src/
│   │   ├── api/                   API Layer
│   │   ├── core/                  Weltformel, Consensus, Engine
│   │   ├── domain/                Domain-Typen (UniversalId, Trust, Event)
│   │   ├── execution/             ExecutionContext (Monade ℳ)
│   │   ├── local/                 Fjall Storage, Archive, Realm-Storage
│   │   ├── peer/                  P2P Layer (libp2p, NAT-Traversal)
│   │   ├── protection/            Anti-Calcification, Adaptive Calibration
│   │   ├── auth/                  Auth Logic
│   │   ├── config/                Configuration
│   │   └── gen/                   Generated Protobuf
│   ├── config/                    TOML Config Files
│   ├── migrations/                SQL Migrations
│   └── proto/                     Protobuf Definitions
│
├── 🎨 frontend/                   SvelteKit Apps
│   ├── console/                   Admin Console
│   ├── platform/                  Main Platform
│   └── docs/                      Documentation
│
├── 📖 documentation/              Dokumentation
│   ├── concept/                   Protokoll & Konzept
│   └── system/                    Plattform & Entwicklung
│
├── 🏗️ infra/                      Infrastruktur
│   ├── docker/                    Docker Compose
│   ├── proxy/                     Caddy Config
│   └── auth/                      ZITADEL Setup
│
├── 🔧 scripts/                    Build & Dev Scripts
│
├── flake.nix                      Nix Environment
├── justfile                       Task Runner
├── buf.yaml                       Protobuf Config
└── turbo.json                     Turborepo Config
```

---

## 🎯 Design-Entscheidungen

### Prinzipien

| Prinzip        | Umsetzung                       |
| :------------- | :------------------------------ |
| **Modular**    | Monorepo + Feature-basierte API |
| **Schnell**    | Rust + Svelte + Turborepo       |
| **Robust**     | Typsicherheit + Nix + Restate   |
| **Skalierbar** | Klare Schichten, lose Kopplung  |

### Bewusst weggelassen (vorerst)

| Feature              | Grund                         |
| :------------------- | :---------------------------- |
| Python Microservices | Fokus auf Rust-Kern           |
| Prometheus           | Observability später          |
| Kubernetes           | Docker Compose reicht für Dev |

---

## 📚 Weiterführende Dokumente

### System-Dokumentation

| Dokument                                     | Beschreibung          |
| :------------------------------------------- | :-------------------- |
| [Configuration](config.md)                   | Service-Konfiguration |
| [Connections](connections.md)                | API-Verbindungen      |
| [Style Guide](../development/style-guide.md) | Code-Standards        |
| [Testing](../development/testing.md)         | Test-Strategien       |

### Protokoll-Konzepte

| Dokument                                                           | Beschreibung               |
| :----------------------------------------------------------------- | :------------------------- |
| [Fachkonzept](../../concept/fachkonzept.md)                        | Vollständige Spezifikation |
| [Systemarchitektur](../../concept/system-architecture-overview.md) | ERY/ECHO/NOA               |
| [Glossar](../../concept/glossary.md)                               | Begriffsdefinitionen       |

---

<div align="center">

```
┌─────────────────────────────────────────────┐
│                                             │
│   🎨 Frontend   →   🔌 API   →   🦀 Backend │
│   SvelteKit        Connect       Rust/Axum  │
│                    -RPC                     │
│                                             │
└─────────────────────────────────────────────┘
```

</div>
