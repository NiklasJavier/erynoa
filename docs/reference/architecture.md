# 🏗️ System-Architektur

**Technische Architektur-Dokumentation für das Erynoa-Projekt (Plattform-Ebene)**

> 💡 **Hinweis:** Diese Datei beschreibt die **konkrete System- und Plattformarchitektur** (Frontend, Backend, Infrastruktur).  
> Die **Protokoll- und Konzeptarchitektur** von Erynoa (Triade ERY/ECHO/NOA, liquides Datenmodell, Cybernetic Loop) ist im Verzeichnis `concept/` dokumentiert, insbesondere in:
> - `concept/kernkonzept.md`
> - `concept/system-architecture-overview.md`

**Letzte Aktualisierung**: 2026-01-28

---

## 📋 Inhaltsverzeichnis

- [Übersicht](#-übersicht)
- [System-Diagramm](#-system-diagramm)
- [Frontend-Architektur](#-frontend-architektur)
- [Backend-Architektur](#-backend-architektur)
- [Infrastruktur](#-infrastruktur)
- [API-Kommunikation](#-api-kommunikation)
- [Verzeichnisstruktur](#-verzeichnisstruktur)

---

## 🎯 Übersicht

Erynoa basiert auf einem **performanten, typsicheren und skalierbaren** Fundament für die Umsetzung der Plattform, auf der das Protokoll aufsetzt:

| Schicht      | Technologie            | Beschreibung               |
| ------------ | ---------------------- | -------------------------- |
| **Frontend** | SvelteKit, TypeScript  | 3 Apps im Monorepo         |
| **Backend**  | Rust, Axum             | High-Performance API       |
| **API**      | Connect-RPC (Protobuf) | End-to-End Typsicherheit   |
| **Workflows**| Restate                | Durable Orchestrierung     |
| **Auth**     | ZITADEL                | OIDC/JWT Authentication    |
| **Database** | PostgreSQL (OrioleDB)  | Persistenz                 |
| **Cache**    | DragonflyDB            | Redis-kompatibel           |
| **Storage**  | MinIO                  | S3-kompatibel              |
| **Proxy**    | Caddy                  | Reverse Proxy, Auto-SSL    |

---

## 🖼 System-Diagramm

```
┌─────────────────────────────────────────────────────────────────┐
│                         Browser                                  │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Caddy Reverse Proxy                          │
│                      (Port 3001)                                 │
│  ┌──────────┬──────────┬──────────┬──────────┐                  │
│  │ /console │ /platform│  /docs   │   /api   │                  │
│  └────┬─────┴────┬─────┴────┬─────┴────┬─────┘                  │
└───────┼──────────┼──────────┼──────────┼────────────────────────┘
        │          │          │          │
        ▼          ▼          ▼          ▼
┌───────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────────────┐
│  Console  │ │ Platform  │ │   Docs    │ │    Backend (Rust)     │
│   :5173   │ │   :5174   │ │   :5175   │ │        :3000          │
│ SvelteKit │ │ SvelteKit │ │ SvelteKit │ │    Axum + SQLx        │
└───────────┘ └───────────┘ └───────────┘ └───────────┬───────────┘
                                                      │
                    ┌─────────────────────────────────┼─────────────┐
                    │                                 │             │
                    ▼                                 ▼             ▼
           ┌───────────────┐               ┌──────────────┐ ┌─────────────┐
           │   ZITADEL     │               │  PostgreSQL  │ │ DragonflyDB │
           │    :8080      │               │    :5432     │ │    :6379    │
           │   (Auth)      │               │  (OrioleDB)  │ │   (Cache)   │
           └───────────────┘               └──────────────┘ └─────────────┘
                                                      │
                                                      ▼
                                           ┌──────────────┐
                                           │    MinIO     │
                                           │  :9000/9001  │
                                           │  (Storage)   │
                                           └──────────────┘
```

---

## 🎨 Frontend-Architektur

### Monorepo-Strategie

Das Frontend nutzt einen **pnpm Workspace** mit **Turborepo** für optimierte Builds:

```
frontend/
├── console/      # Admin Console
├── platform/     # Main Platform
└── docs/         # Documentation
```

### Vorteile

| Feature                  | Beschreibung                                 |
| ------------------------ | -------------------------------------------- |
| **Shared Dependencies**  | Hardlinked via pnpm (Platz- & Zeitersparnis) |
| **Shared Types**         | Generierte Protobuf-Types in `src/gen/`      |
| **Parallele Builds**     | Turborepo mit Caching                        |
| **Konsistente Struktur** | Gleiche Verzeichnisstruktur in allen Apps    |

### Tech Stack

| Komponente       | Version | Beschreibung         |
| ---------------- | ------- | -------------------- |
| **SvelteKit**    | 2.x     | Meta-Framework       |
| **Svelte**       | 5.x     | UI Framework (Runes) |
| **TypeScript**   | 5.x     | Type Safety          |
| **Tailwind CSS** | 3.x     | Styling              |
| **Vite**         | 5.x     | Build Tool           |
| **Biome**        | 1.x     | Linting & Formatting |

---

## 🦀 Backend-Architektur

### High-Performance Rust Stack

| Komponente      | Version | Beschreibung               |
| --------------- | ------- | -------------------------- |
| **Axum**        | 0.8     | Web Framework              |
| **Tokio**       | 1.x     | Async Runtime              |
| **SQLx**        | 0.8     | DB mit Compile-Time Checks |
| **Connect-RPC** | -       | gRPC-Web API               |
| **Jemalloc**    | -       | Memory Allocator           |

### Backend-Fähigkeiten im Überblick

- **Transaktionale API**: Rust/Axum + SQLx für typsichere, performante Geschäftsvorgänge.
- **Workflows**: Restate für langlebige, fehlertolerante Orchestrierung (Retries, Timer, State).
- **Dokumente**: Typst für reproduzierbare PDF‑Generierung aus Templates.
- **E-Mails**: Lettre (tokio‑1) + Rinja für templatisierte, asynchrone Mailzustellung.

### E-Mail & Templating (Lettre + Rinja)

Für transaktionale E-Mails nutzt Erynoa:

- **Lettre (tokio‑1 kompatibel)** als asynchronen SMTP‑Client:
  - Versand von Mails aus Rust‑Handlern (z. B. Bestätigungen, Benachrichtigungen)
  - Integration in bestehende Tokio‑Runtime (kein separater Mail‑Prozess nötig)
- **Rinja** als Template‑Engine für E-Mail‑Inhalte:
  - Trennung von Layout (HTML/Text‑Templates) und Logik
  - Wiederverwendbare Templates im Repository (z. B. `templates/emails/*.html.rinja`)
  - Lokalisierung und Branding über gemeinsame Basis‑Layouts

Typischer Ablauf:

1. Backend füllt ein Rinja‑Template mit Daten (z. B. Benutzername, Links).
2. Gerendertes HTML/Text wird über Lettre über einen konfigurierten SMTP‑Server versendet.
3. Fehler und Retries können bei Bedarf über Restate‑Workflows robust orchestriert werden.

### Dokument-/PDF-Generierung mit Typst

Für serverseitige Dokument‑ und PDF‑Generierung wird **Typst** eingesetzt:

- **Deklarative Dokumente**: Layout und Inhalt werden in einer typst‑basierten DSL beschrieben.
- **Reproduzierbare PDFs**: Templates im Repository sorgen für konsistente, versionierte PDF‑Ausgaben (z. B. Reports, Exporte).
- **Backend‑Integration**: Das Rust‑Backend rendert PDFs automatisiert über Typst‑Aufrufe (z. B. für Download‑Endpunkte).

### Workflows & Orchestrierung mit Restate

Für langlebige, fehlertolerante Abläufe setzt Erynoa auf **Restate** als dedizierte Orchestrierungsschicht:

- **Durable Execution**: Handler werden so ausgeführt, dass jeder abgeschlossene Schritt persistent gespeichert wird und bei Fehlern nahtlos wiederaufgenommen werden kann.
- **Genau-einmal Semantik**: Externe Aufrufe (z. B. ans Rust‑Backend oder andere Services) werden zuverlässig und idempotent koordiniert.
- **Stateful Workflows**: Komplexe Business-Prozesse (Sagas, mehrstufige Provisioning-Flows, Reminder) besitzen eigenen, stark konsistenten Zustand.
- **Zeitbasierte Events**: Timer, Delays und Deadlines sind first‑class (z. B. „warte 24h auf Bestätigung, sonst storniere Auftrag“).

Restate ergänzt das Rust‑Backend:

- Rust/Axum implementiert die **fachliche Logik** und APIs.
- Restate orchestriert **Ablauf, Retries, State und Wiederaufnahmen** über Service‑ und Prozessgrenzen hinweg.

### Optimierungen

```toml
# Cargo.toml [profile.release]
strip = true      # Debug-Symbole entfernen
lto = "fat"       # Aggressive Link-Time Optimization
```

**Ergebnis:** Kleine Binaries, maximale Performance

### API-Struktur

```
backend/src/api/
├── v1/                    # API Version 1
│   ├── health/            # Health Check
│   ├── info/              # Info & Status
│   ├── users/             # User Management
│   └── storage/           # Storage Operations
├── middleware/            # Middleware Layer
│   ├── auth.rs            # JWT Validation
│   ├── cors.rs            # CORS Config
│   └── logging.rs         # Request Logging
└── shared/                # Shared Utilities
    └── pagination.rs
```

---

## 🏗 Infrastruktur

### Development Environment

| Tool               | Beschreibung                                      |
| ------------------ | ------------------------------------------------- |
| **Nix Flakes**     | Reproduzierbare Toolchain (Rust, Node, buf, etc.) |
| **DevContainer**   | VS Code Container-Entwicklung                     |
| **Docker Compose** | Service-Orchestrierung                            |
| **just**           | Task Runner                                       |

### Services

| Service      | Port      | Technologie | Beschreibung           |
| ------------ | --------- | ----------- | ---------------------- |
| **Proxy**    | 3001      | Caddy       | Reverse Proxy, Routing |
| **Backend**  | 3000      | Rust/Axum   | API Server             |
| **Workflows**| –         | Restate     | Orchestrierungs-Engine |
| **Console**  | 5173      | SvelteKit   | Admin UI               |
| **Platform** | 5174      | SvelteKit   | Main App               |
| **Docs**     | 5175      | SvelteKit   | Documentation          |
| **Database** | 5432      | PostgreSQL  | OrioleDB Engine        |
| **Cache**    | 6379      | DragonflyDB | Redis-kompatibel       |
| **Storage**  | 9000/9001 | MinIO       | S3-kompatibel          |
| **Auth**     | 8080      | ZITADEL     | OIDC/JWT               |

### Caddy Proxy Routing

```
localhost:3001/
├── /console   → localhost:5173
├── /platform  → localhost:5174
├── /docs      → localhost:5175
└── /api       → localhost:3000
```

---

## 🔌 API-Kommunikation

### Connect-RPC (Protobuf)

**End-to-End Typsicherheit** zwischen Frontend und Backend:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   .proto    │ ──▶ │  buf gen    │ ──▶ │ TypeScript  │
│ Definitionen│     │             │     │   Types     │
└─────────────┘     └─────────────┘     └─────────────┘
       │
       ▼
┌─────────────┐
│    Rust     │
│   Server    │
└─────────────┘
```

### Vorteile

| Feature                    | Beschreibung                               |
| -------------------------- | ------------------------------------------ |
| **Single Source of Truth** | `.proto` Dateien definieren API            |
| **Auto-Generated Types**   | TypeScript-Clients automatisch generiert   |
| **Compile-Time Safety**    | Keine Runtime-Fehler durch Type-Mismatches |
| **gRPC-Web**               | Browser-kompatibel                         |

### Frontend API-Struktur

```
frontend/*/src/lib/api/
├── health/               # Health Service
│   ├── types.ts          # Protobuf types
│   └── index.ts          # Public API
├── users/                # User Service
│   ├── connect-client.ts # Connect-RPC client
│   ├── types.ts
│   └── index.ts
└── connect/              # Transport Layer
    ├── transport.ts      # Transport config
    └── services.ts       # Service clients
```

---

## 📁 Verzeichnisstruktur

```
erynoa/
│
├── backend/                 # 🦀 Rust Backend
│   ├── src/
│   │   ├── api/             # API Layer
│   │   ├── auth/            # Auth Logic
│   │   ├── cache/           # Cache Layer
│   │   ├── config/          # Configuration
│   │   ├── db/              # Database Layer
│   │   ├── gen/             # Generated Protobuf
│   │   └── storage/         # Storage Layer
│   ├── config/              # TOML Config Files
│   ├── migrations/          # SQL Migrations
│   └── proto/               # Protobuf Definitions
│
├── frontend/                # 🎨 SvelteKit Apps
│   ├── console/             # Admin Console
│   ├── platform/            # Main Platform
│   └── docs/                # Documentation
│
├── infra/                   # 🏗 Infrastructure
│   ├── docker/              # Docker Compose & Dockerfiles
│   ├── proxy/               # Caddy Configuration
│   ├── auth/                # ZITADEL Config
│   └── static/              # Static Files
│
├── docs/                    # 📚 Documentation
├── scripts/                 # 🔧 Build & Dev Scripts
│
├── flake.nix                # Nix Dev Environment
├── justfile                 # Task Runner
├── buf.yaml                 # Protobuf Config
├── turbo.json               # Turborepo Config
└── pnpm-workspace.yaml      # pnpm Workspace
```

---

## 🔮 Design-Entscheidungen

### Was wurde bewusst weggelassen

Diese Features können bei Bedarf später hinzugefügt werden:

| Feature              | Grund                |
| -------------------- | -------------------- |
| Python Microservices | Fokus auf Rust-Kern  |
| Prometheus           | Observability später |
| RAM-Datenbanken      | PostgreSQL reicht    |

### Prinzipien

- **Modular**: Monorepo + Shared-Core
- **Schnell**: Rust + Svelte
- **Robust**: Typsicherheit + Nix
- **Skalierbar**: Klare Architektur

---

## 📚 Weiterführende Dokumentation

| Dokument                                     | Beschreibung                            |
| -------------------------------------------- | --------------------------------------- |
| [Configuration](config.md)                   | Service-Konfiguration                   |
| [Connections](connections.md)                | API-Verbindungen                        |
| [Style Guide](../development/style-guide.md) | Code-Standards                          |
| [Testing](../development/testing.md)         | Test-Strategien                         |
| [TODOs](../development/todos.md)             | Offene Aufgaben                         |
| [Kernkonzept](../concept/kernkonzept.md)     | Protokollidee & Problemraum             |
| [System Architecture Overview](../concept/system-architecture-overview.md) | Triade ERY/ECHO/NOA & Layer-Modell |
