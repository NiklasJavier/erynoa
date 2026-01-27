# 📚 Dokumentation

**Letzte Aktualisierung**: 2026-01-27 (23:40)

**Status**: Aktuell und vollständig organisiert ✅

Willkommen zur Dokumentation des Erynoa-Projekts. Diese Dokumentation ist in drei Hauptkategorien organisiert:

---

## 🚀 Quick Start

**Voraussetzungen:** (siehe [Setup Guide](setup/setup.md))
- Nix installiert
- Docker Desktop installiert und gestartet

**3 Schritte zum laufenden Projekt:**

```bash
# 1. Repository klonen
git clone git@github.com:NiklasJavier/erynoa.git
cd erynoa

# 2. Nix Dev-Shell betreten (lädt alle Tools automatisch)
nix develop

# 3. Projekt starten
just dev
```

**4. Warte 2 Minuten** ⏳

Die Services starten und ZITADEL wird automatisch konfiguriert. Nach ca. 2 Minuten kannst du im Browser öffnen:

```
http://localhost:3001
```

**Fertig!** 🎉

Das startet alles:
- **Proxy** auf http://localhost:3001 (Caddy Reverse Proxy)
  - **Console** auf http://localhost:3001/console
  - **Platform** auf http://localhost:3001/platform
  - **Docs** auf http://localhost:3001/docs
  - **Backend API** auf http://localhost:3001/api
- **Backend** direkt auf http://localhost:3000 (für Tests)
- **ZITADEL** auf http://localhost:8080 (Auth)
- **MinIO** auf http://localhost:9001 (S3 Storage Console)

**Test Login:**
- User: `testuser` / `Test123!`
- Admin: `zitadel-admin` / `Password1!`

---

## 📖 Dokumentationsstruktur

### 📘 Guides
Schritt-für-Schritt Anleitungen für häufige Aufgaben:

- **[Getting Started](guides/getting-started.md)** - Erste Schritte mit dem Projekt (3-Schritte Quick Start)
- **[Setup](setup/setup.md)** - Entwicklungsumgebung einrichten
  - ⚡ **Quick Start** - Nix + Docker installieren, dann `nix develop` → `just dev`
  - 📋 **Vollständige Anleitung** - Detaillierte Setup-Anleitung für macOS und Ubuntu
- **[Dev Setup](setup/dev_setup.md)** - Container-in-Container Entwicklung
- **[Docker Setup](setup/docker.md)** - Docker Development Setup
- **[ZITADEL Setup](guides/zitadel.md)** - Authentifizierung konfigurieren (automatisch via `just zitadel-setup`)

### 📗 Reference
Referenz-Dokumentation für Architektur und Konfiguration:

- **[Architecture](reference/architecture.md)** - Systemarchitektur und Design-Entscheidungen
- **[Configuration](reference/config.md)** - Service-Konfiguration und Verbindungen
- **[Connections](reference/connections.md)** - API-Verbindungen und Harmonisierung

### 📙 Development
Development-spezifische Dokumentation:

- **[Style Guide](development/style-guide.md)** - Code-Stil und Best Practices
- **[Testing](development/testing.md)** - Test-Strategien und -Tools
- **[TODOs](development/todos.md)** - Offene Aufgaben und Prioritäten
- **[REST Deprecation Plan](development/rest_deprecation_plan.md)** - Plan zur REST-API Entfernung

---

## 🎯 Schnellzugriff

### Für neue Entwickler
1. Starte mit [Getting Started](guides/getting-started.md)
2. Lese [Architecture](reference/architecture.md) für Überblick
3. Folge [Setup (macOS)](setup/setup.md) oder [Dev Setup](setup/dev_setup.md) für Entwicklungsumgebung

### Für erfahrene Entwickler
- [Architecture](reference/architecture.md) - System-Design
- [Configuration](reference/config.md) - Service-Konfiguration
- [Style Guide](development/style-guide.md) - Code-Standards
- [TODOs](development/todos.md) - Offene Aufgaben

### Für DevOps
- [Configuration](reference/config.md) - Service-Konfiguration
- [Connections](reference/connections.md) - Netzwerk-Verbindungen
- [Setup](setup/setup.md) - Deployment-Konfiguration

---

## 📋 Wichtige Dokumente

- **[essential_guide.md](essential_guide.md)** - Konsolidierter Guide mit allen wichtigen Informationen
- **[documentation_status.md](documentation_status.md)** - Dokumentations-Status und Übersicht
- **[todos](development/todos.md)** - Aktuelle Aufgaben und Prioritäten

---

## 🔍 Navigation

- **Guides** (`guides/`) - Schritt-für-Schritt Anleitungen
- **Setup** (`setup/`) - Setup-Anleitungen (macOS, Docker, DevContainer)
- **Reference** (`reference/`) - Technische Referenz
- **Development** (`development/`) - Development-spezifisch

**Vollständige Navigation**: Siehe [navigation.md](navigation.md) für detaillierte Übersicht

---

**Hinweis**: Diese Dokumentation wird kontinuierlich aktualisiert. Bei Fragen oder Verbesserungsvorschlägen, bitte ein Issue erstellen.
