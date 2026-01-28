# Erynoa – System-Dokumentation

> **Dokumenttyp:** Übersicht
> **Bereich:** Plattform & Entwicklung
> **Status:** Aktiv
> **Lesezeit:** ca. 5 Minuten

---

## Willkommen

Diese Dokumentation beschreibt die **technische Implementierung** der Erynoa-Plattform – das Rust-Backend, die SvelteKit-Frontends und die Infrastruktur.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📖 DOKUMENTATIONS-LANDKARTE                                              │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   🚀 GUIDES              ⚙️ SETUP               📗 REFERENCE        │  │
│   │   ──────────            ─────────              ───────────          │  │
│   │   Getting Started       Entwicklungs-          Architektur          │  │
│   │   ZITADEL Auth          umgebung               Konfiguration        │  │
│   │   Deployment            Docker                 Verbindungen         │  │
│   │                                                                     │  │
│   │   ─────────────────────────────────────────────────────────────    │  │
│   │                                                                     │  │
│   │   📙 DEVELOPMENT                                                    │  │
│   │   ──────────────                                                    │  │
│   │   Style Guide · Testing · TODOs                                    │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

> 💡 **Protokoll-Konzepte** (ERY, ECHO, NOA, Trust) findest du unter [documentation/concept/](../concept/fachkonzept.md)

---

## ⚡ Quick Start

```bash
# 1. Klonen
git clone git@github.com:NiklasJavier/erynoa.git && cd erynoa

# 2. Nix Dev-Shell
nix develop

# 3. Starten
just dev
```

<div align="center">

⏳ **~2 Minuten warten** → 🌐 **http://localhost:3001**

</div>

<details>
<summary><strong>🔗 Alle Services</strong></summary>

| Service      | URL                            | Beschreibung   |
| :----------- | :----------------------------- | :------------- |
| 🌐 **Proxy** | http://localhost:3001          | Hauptzugang    |
| 📊 Console   | http://localhost:3001/console  | Admin          |
| 🖥️ Platform  | http://localhost:3001/platform | Hauptplattform |
| 📖 Docs      | http://localhost:3001/docs     | Dokumentation  |
| 🔌 API       | http://localhost:3001/api      | Backend        |
| 🔐 ZITADEL   | http://localhost:8080          | Auth           |
| 📦 MinIO     | http://localhost:9001          | Storage        |

**Test-Login:** `testuser` / `Test123!`

</details>

---

## 📁 Struktur

```
documentation/
│
├── 📖 concept/                    Protokoll & Konzept
│   ├── fachkonzept.md             ⭐ Master-Dokument
│   ├── kernkonzept.md             High-Level Überblick
│   └── ...                        Trust, Agents, Use Cases
│
└── 🛠️ system/                     Plattform & Entwicklung (← Du bist hier)
    │
    ├── 🚀 guides/                 Schritt-für-Schritt
    │   ├── getting-started.md     Erste Schritte
    │   └── zitadel.md             Auth-Setup
    │
    ├── ⚙️ setup/                  Entwicklungsumgebung
    │   ├── setup.md               Vollständiges Setup
    │   ├── dev_setup.md           Dev-Container
    │   └── docker.md              Docker-Konfiguration
    │
    ├── 📗 reference/              Technische Referenz
    │   ├── architecture.md        Systemarchitektur
    │   ├── config.md              Konfiguration
    │   └── connections.md         Service-Verbindungen
    │
    └── 📙 development/            Standards & Workflows
        ├── style-guide.md         Code-Stil
        ├── testing.md             Test-Strategien
        └── todos.md               Offene Aufgaben
```

---

## 🚀 Guides

Schritt-für-Schritt Anleitungen für häufige Aufgaben.

| Guide                                                  | Beschreibung                    | Dauer  |
| :----------------------------------------------------- | :------------------------------ | :----- |
| [**Getting Started**](guides/getting-started.md)       | Erste Schritte mit Erynoa       | 10 min |
| [**ZITADEL Setup**](guides/zitadel.md)                 | Authentifizierung konfigurieren | 15 min |
| [**Unified Deployment**](guides/unified-deployment.md) | Deployment-Anleitung            | 20 min |

---

## ⚙️ Setup

Einrichtung der Entwicklungsumgebung.

| Dokument                            | Beschreibung             | Voraussetzungen |
| :---------------------------------- | :----------------------- | :-------------- |
| [**Setup Guide**](setup/setup.md)   | Vollständige Anleitung   | Nix, Docker     |
| [**Dev Setup**](setup/dev_setup.md) | DevContainer-Entwicklung | VS Code         |
| [**Docker Setup**](setup/docker.md) | Docker-Konfiguration     | Docker          |

---

## 📗 Reference

Technische Referenz-Dokumentation.

| Dokument                                                        | Beschreibung               |
| :-------------------------------------------------------------- | :------------------------- |
| [**Architecture**](reference/architecture.md)                   | Systemarchitektur & Design |
| [**Configuration**](reference/config.md)                        | Service-Konfiguration      |
| [**Connections**](reference/connections.md)                     | API-Verbindungen & Ports   |
| [**Platform Architecture**](reference/platform-architecture.md) | Plattform-Komponenten      |

---

## 📙 Development

Standards, Workflows und offene Aufgaben.

| Dokument                                                     | Beschreibung               |
| :----------------------------------------------------------- | :------------------------- |
| [**Style Guide**](development/style-guide.md)                | Code-Stil & Best Practices |
| [**Testing**](development/testing.md)                        | Test-Strategien & Muster   |
| [**TODOs**](development/todos.md)                            | Offene Aufgaben & Roadmap  |
| [**REST Deprecation**](development/rest_deprecation_plan.md) | Migration zu Connect-RPC   |

---

## 🎯 Schnellzugriff nach Rolle

<table>
<tr>
<td width="33%" valign="top">

### 🆕 Neue Entwickler

1. [Getting Started](guides/getting-started.md)
2. [Setup Guide](setup/setup.md)
3. [Architecture](reference/architecture.md)
4. [Style Guide](development/style-guide.md)

</td>
<td width="33%" valign="top">

### 👨‍💻 Erfahrene Entwickler

- [Architecture](reference/architecture.md)
- [Configuration](reference/config.md)
- [TODOs](development/todos.md)
- [Testing](development/testing.md)

</td>
<td width="33%" valign="top">

### 🔧 DevOps

- [Docker Setup](setup/docker.md)
- [Configuration](reference/config.md)
- [Connections](reference/connections.md)
- [Unified Deployment](guides/unified-deployment.md)

</td>
</tr>
</table>

---

## 📋 Wichtige Dokumente

| Dokument                                        | Beschreibung                      |
| :---------------------------------------------- | :-------------------------------- |
| ⭐ [**Essential Guide**](essential_guide.md)    | Alles Wichtige auf einen Blick    |
| [Documentation Status](documentation_status.md) | Status-Übersicht aller Dokumente  |
| [Navigation](navigation.md)                     | Vollständige Navigationsübersicht |

---

## 🔗 Verbindung zum Konzept

Die System-Dokumentation beschreibt die **Implementierung** – für das **Protokoll-Design** siehe:

| Konzept-Dokument                                                   | Inhalt                     |
| :----------------------------------------------------------------- | :------------------------- |
| [📋 Fachkonzept](../concept/fachkonzept.md)                        | Vollständige Spezifikation |
| [🎯 Kernkonzept](../concept/kernkonzept.md)                        | High-Level Überblick       |
| [🏗️ Systemarchitektur](../concept/system-architecture-overview.md) | Drei-Sphären-Architektur   |
| [📖 Glossar](../concept/glossary.md)                               | Begriffsdefinitionen       |

---

## 🛠️ Tech Stack (Kurzübersicht)

| Bereich      | Technologie                                  |
| :----------- | :------------------------------------------- |
| **Backend**  | Rust · Axum · Connect-RPC · PostgreSQL       |
| **Frontend** | SvelteKit · Svelte 5 · Tailwind · TypeScript |
| **Auth**     | ZITADEL (OIDC/JWT)                           |
| **Infra**    | Nix · Docker Compose · Caddy                 |

---

<div align="center">

```
┌─────────────────────────────────────────────┐
│                                             │
│   Bei Fragen:                               │
│   1. Essential Guide prüfen                 │
│   2. TODOs durchsuchen                      │
│   3. Issue erstellen                        │
│                                             │
└─────────────────────────────────────────────┘
```

**Letzte Aktualisierung:** Januar 2026

</div>
