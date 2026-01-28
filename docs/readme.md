# 📚 Erynoa Dokumentation

<div align="center">

**Vollständige Dokumentation für das Erynoa-Projekt**

[Quick Start](#-quick-start) •
[Guides](#-guides) •
[Reference](#-reference) •
[Development](#-development)

</div>

---

## ⚡ Quick Start

> **Voraussetzungen:** [Nix](https://nixos.org/) und [Docker](https://www.docker.com/) → Details: [Setup Guide](setup/setup.md)

```bash
git clone git@github.com:NiklasJavier/erynoa.git && cd erynoa
nix develop
just dev
```

**Warte ~2 Minuten** → Öffne **http://localhost:3001**

<details>
<summary><strong>🔗 URLs & Login</strong></summary>

| Service         | URL                            |
| --------------- | ------------------------------ |
| **Hauptzugang** | http://localhost:3001          |
| Console         | http://localhost:3001/console  |
| Platform        | http://localhost:3001/platform |
| Docs            | http://localhost:3001/docs     |
| Backend API     | http://localhost:3001/api      |
| ZITADEL         | http://localhost:8080          |
| MinIO           | http://localhost:9001          |

**Login:** `testuser` / `Test123!`

</details>

---

## 🗂 Dokumentationsübersicht

```
docs/
├── 📘 guides/        # Schritt-für-Schritt Anleitungen
├── ⚙️ setup/         # Setup & Installation
├── 📗 reference/     # Technische Referenz
└── 📙 development/   # Development-Standards
```

---

## 📘 Guides

Schritt-für-Schritt Anleitungen für häufige Aufgaben:

| Guide                                            | Beschreibung                            |
| ------------------------------------------------ | --------------------------------------- |
| **[Getting Started](guides/getting-started.md)** | Erste Schritte (3-Schritte Quick Start) |
| **[ZITADEL Setup](guides/zitadel.md)**           | Authentifizierung konfigurieren         |

---

## ⚙️ Setup

Anleitungen zur Einrichtung der Entwicklungsumgebung:

| Guide                               | Beschreibung                                    |
| ----------------------------------- | ----------------------------------------------- |
| **[Setup Guide](setup/setup.md)**   | Vollständige Setup-Anleitung (Nix, Docker, Git) |
| **[Dev Setup](setup/dev_setup.md)** | Container-in-Container Entwicklung              |
| **[Docker Setup](setup/docker.md)** | Docker Development Setup                        |

---

## 📗 Reference

Technische Referenz-Dokumentation:

| Dokument                                      | Beschreibung               |
| --------------------------------------------- | -------------------------- |
| **[Architecture](reference/architecture.md)** | Systemarchitektur & Design |
| **[Configuration](reference/config.md)**      | Service-Konfiguration      |
| **[Connections](reference/connections.md)**   | API-Verbindungen           |

---

## 📙 Development

Development-Standards und Workflows:

| Dokument                                                     | Beschreibung               |
| ------------------------------------------------------------ | -------------------------- |
| **[Style Guide](development/style-guide.md)**                | Code-Stil & Best Practices |
| **[Testing](development/testing.md)**                        | Test-Strategien            |
| **[TODOs](development/todos.md)**                            | Offene Aufgaben            |
| **[REST Deprecation](development/rest_deprecation_plan.md)** | REST-API Entfernung        |

---

## 🎯 Schnellzugriff

<table>
<tr>
<td width="33%">

### 🆕 Neue Entwickler

1. [Getting Started](guides/getting-started.md)
2. [Architecture](reference/architecture.md)
3. [Setup Guide](setup/setup.md)

</td>
<td width="33%">

### 👨‍💻 Erfahrene Entwickler

- [Architecture](reference/architecture.md)
- [Style Guide](development/style-guide.md)
- [TODOs](development/todos.md)

</td>
<td width="33%">

### 🔧 DevOps

- [Configuration](reference/config.md)
- [Connections](reference/connections.md)
- [Docker Setup](setup/docker.md)

</td>
</tr>
</table>

---

## 📋 Wichtige Dokumente

| Dokument                                            | Beschreibung                      |
| --------------------------------------------------- | --------------------------------- |
| **[Essential Guide](essential_guide.md)**           | Alles Wichtige auf einen Blick    |
| **[Documentation Status](documentation_status.md)** | Status-Übersicht                  |
| **[Navigation](navigation.md)**                     | Vollständige Navigationsübersicht |

---

<div align="center">

**Letzte Aktualisierung**: 2026-01-28

Bei Fragen → [TODOs](development/todos.md) prüfen oder Issue erstellen

</div>
