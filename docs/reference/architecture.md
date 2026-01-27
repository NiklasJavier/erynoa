# 🏗️ System-Architektur

## Übersicht

Architektur-Dokumentation für das Erynoa-Projekt.

---

## 🎯 Aktueller Stack-Status (Enterprise-Grade Fundament)

**Letzte Aktualisierung**: 2026-01-27

**Status**: Aktuell und vollständig dokumentiert ✅

Das Erynoa-Projekt basiert auf einem extrem performanten, typsicheren und skalierbaren Fundament. Die Komplexität von Python-Microservices und Observability-Tools (Prometheus) wurde bewusst zunächst weggelassen, um sich auf die Kernarchitektur zu konzentrieren.

### 1. Frontend: Monorepo & "Shared Core" Strategie

Die redundanten Frontend-Apps (console, platform, docs) wurden in einen effizienten **pnpm Workspace** umgewandelt.

#### Struktur
- **pnpm Workspace**: Alle Frontend-Apps (console, platform, docs) sind im selben Workspace
  - Dependencies werden zwischen Apps gehardlinkt (Platz- und Zeitersparnis)
- **Geteilte Protobuf-Types**: Alle Apps nutzen die gleichen generierten TypeScript-Types aus `src/gen/`
  - Single Source of Truth für API-Definitionen
- **Konsistente Struktur**: Alle Apps haben die gleiche Verzeichnisstruktur (`src/lib/api/`, `src/lib/components/`, etc.)
  - Einfacheres Wartung und Code-Sharing durch Copy-Paste (später kann eine `@erynoa/shared` Library hinzugefügt werden)
- **Build-System**: Nutzung von **Turborepo (turbo)**, um Builds und Lints parallel und gecached auszuführen
  - Drastische Reduzierung der CI-Zeiten
  - Parallele Frontend-Builds (console, platform, docs)
  - Optimiertes Caching für schnellere Builds

#### Vorteile
- Keine Code-Duplikation zwischen Frontend-Apps
- Konsistente UI/UX über alle Apps hinweg
- Schnellere Build-Zeiten durch Caching
- Einfacheres Wartung und Updates

### 2. Backend: High-Performance Rust

Das Backend wurde auf maximale Effizienz und Typsicherheit getrimmt.

#### Modernster Stack
- **Axum 0.8**: Webserver-Framework
- **SQLx 0.8**: Datenbank-Abstraktion mit Compile-Time Query Checking
- **Connect-RPC**: End-to-End Typsicherheit
  - API wird über `.proto` Dateien definiert
  - TypeScript-Clients für das Frontend werden automatisch generiert
  - Frontend und Backend können sich so nicht "missverstehen"

#### Memory Management
- **Jemalloc**: Integration von `tikv-jemallocator` im Code
  - Verhindert Speicherfragmentierung bei Langzeitbetrieb
  - Optimiert für Server-Workloads

#### Release-Optimierung
- **Extrem kleine und schnelle Binaries**:
  - `strip = true`: Entfernt Debug-Symbole
  - `lto = "fat"`: Aggressive Link-Time Optimization
  - Minimale Binary-Größe bei maximaler Performance

#### Vorteile
- Höchste Performance durch Rust
- Compile-Time Typsicherheit
- Automatische API-Synchronisation zwischen Frontend und Backend
- Optimierte Memory-Nutzung

### 3. Developer Experience (DX) & Infrastruktur

Die Entwicklungsumgebung wurde professionalisiert, um "Works on my machine"-Probleme zu eliminieren.

#### Nix-Integration
- **flake.nix**: Hermetische Abriegelung der gesamten Toolchain
  - Rust, Node, Protobuf-Tools werden reproduzierbar bereitgestellt
  - Garantiert identische Entwicklungsumgebung für alle Entwickler
  - Keine Versionskonflikte mehr

#### DevContainer
- **Container-Setup**: Bündelt die gesamte Infrastruktur
  - Datenbank (PostgreSQL)
  - Cache (Redis)
  - Auth (Zitadel)
  - Alle notwendigen Tools
- **Sofort startklar**: Neue Entwickler können sofort mit der Entwicklung beginnen
- **Konsistente Umgebung**: Gleiche Bedingungen für alle

#### Proxy-Server
- **Caddy**: Reverse Proxy für alle Services
  - Bündelt alle Frontend-Apps und das Backend unter einem Port
  - Übernimmt Routing und SSL automatisch
  - Einfache Konfiguration durch Caddyfile

#### Vorteile
- Reproduzierbare Entwicklungsumgebung
- Schneller Onboarding für neue Entwickler
- Einfaches Routing und SSL-Management
- Keine lokalen Konfigurationsprobleme mehr

### Fazit: Enterprise-Grade Fundament

Das System ist:

- **Modular**: Durch das Monorepo und die Shared-Core-Strategie
- **Schnell**: Durch Rust & Svelte
- **Robust**: Durch Typsicherheit und Nix
- **Skalierbar**: Durch klare Architektur und moderne Patterns

**Bewusst weggelassen** (können später bei Bedarf hinzugefügt werden):
- Python-Microservices
- Prometheus (Observability)
- RAM-Datenbanken

Diese "Add-ons" können später problemlos hinzugefügt werden, ohne die Architektur umwerfen zu müssen.

---

## Backend API-Struktur

### Feature-basierte Organisation

```
backend/src/api/
├── v1/                       # API Version 1
│   ├── health/               # Health Check
│   ├── info/                 # Info & Status
│   ├── users/                # User Management
│   └── storage/              # Storage Operations
├── middleware/               # Middleware Layer
│   ├── auth.rs
│   ├── cors.rs
│   ├── logging.rs
│   └── error_handler.rs
└── shared/                   # Shared Utilities
    └── pagination.rs
```

**Vorteile:**
- Klare Feature-Trennung
- Einfacheres Testing
- Bessere Skalierbarkeit
- API-Versionierung vorbereitet

---

## Console API-Struktur

### Feature-basierte Organisation (Phase 2)

```
frontend/console/src/api/
├── health/                   # Health Service
│   ├── types.ts              # Protobuf types + helpers
│   └── index.ts              # Public API
├── info/                     # Info Service
│   ├── types.ts
│   └── index.ts
├── users/                    # User Service
│   ├── connect-client.ts     # Connect-RPC client
│   ├── types.ts              # Protobuf types + helpers
│   └── index.ts
├── storage/                   # Storage Service
│   ├── connect-client.ts
│   ├── types.ts
│   └── index.ts
├── connect/                  # Connect-RPC Transport
│   ├── transport.ts          # Transport configuration
│   └── services.ts           # Service clients
└── rest/                     # REST Client (deprecated)
    ├── client.ts
    └── endpoints.ts
```

**Vorteile:**
- Konsistente Struktur mit Backend
- Protobuf-Types als Single Source of Truth
- Klare Feature-Trennung
- Einfacheres Testing

---

## 🔌 Connect-RPC/gRPC-Web

Das Projekt verwendet Connect-RPC für die Console-Backend-Kommunikation:

- **Protobuf** für Type-Safe Serialisierung
- **gRPC-Web** für Browser-Kompatibilität
- **Feature-basierte** Service-Organisation
- **Automatische** Code-Generierung

Siehe [Connect-RPC Guide](CONNECT_RPC_GUIDE.md) für Details.

---

## 📚 Weitere Informationen

- [API Restrukturierung](../changelog/API_RESTRUCTURE_COMPLETE.md)
- [Console API Konsolidierung](../changelog/CONSOLE_API_RESTRUCTURE_COMPLETE.md)
- [Connect-RPC Guide](CONNECT_RPC_GUIDE.md)
- [Style Guide](STYLE_GUIDE.md)
- [Testing Guide](testing.md)
- [Harmonization Roadmap](HARMONIZATION_ROADMAP.md)
- [TODO Management](../development/todos.md)
