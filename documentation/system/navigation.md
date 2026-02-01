# 🧭 Dokumentations-Navigation

**Letzte Aktualisierung**: 2026-02-01

**Status**: Aktuell und vollständig ✅ (inkl. Priorität 3)

Diese Datei bietet eine Übersicht über die gesamte Dokumentationsstruktur und hilft dabei, schnell die richtige Dokumentation zu finden.

---

## 📚 Dokumentationsstruktur

```
documentation/
├── concept/                     # Protokoll-Konzepte (ERY/ECHO/NOA, etc.)
│   ├── README.md                # Concept Navigation
│   ├── kernkonzept.md           # Kernkonzept
│   ├── system-architecture-overview.md
│   └── ...
│
└── system/                      # System-/Plattform-Dokumentation
    ├── readme.md                # Hauptübersicht
    ├── essential_guide.md       # Konsolidierter Guide (alles Wichtige)
    ├── navigation.md            # Diese Datei
    ├── documentation_status.md # Dokumentations-Status & Übersicht
    │
    ├── guides/                  # Schritt-für-Schritt Anleitungen
    ├── setup/                   # Setup-Anleitungen
    ├── reference/               # Technische Referenz
    ├── development/             # Development-spezifisch
    └── archive/                 # Historische Dokumentation
```

> 💡 **Hinweis:** Höher-level **Konzept- und Protokoll-Dokumente** (Erynoa Triade, liquides Datenmodell, Cybernetic Loop etc.) befinden sich im Verzeichnis `../concept/`.
> Einstieg: `../concept/README.md`.

---

## 🎯 Schnellzugriff nach Zielgruppe

### 🆕 Für neue Entwickler

1. **[Getting Started](guides/getting-started.md)** - Erste Schritte
2. **[Setup (macOS)](setup/setup.md)** - Entwicklungsumgebung einrichten
3. **[Architecture](reference/architecture.md)** - System-Überblick
4. **[Style Guide](development/style-guide.md)** - Code-Standards

### 👨‍💻 Für erfahrene Entwickler

- **[Architecture](reference/architecture.md)** - System-Design & Entscheidungen
- **[Configuration](reference/config.md)** - Service-Konfiguration
- **[Connections](reference/connections.md)** - API-Verbindungen
- **[Style Guide](development/style-guide.md)** - Code-Standards
- **[Testing](development/testing.md)** - Test-Strategien
- **[todos](development/todos.md)** - Offene Aufgaben

### 🐳 Für DevOps/Infrastructure

- **[Configuration](reference/config.md)** - Service-Konfiguration
- **[Connections](reference/connections.md)** - Netzwerk-Verbindungen
- **[Docker Setup](setup/docker.md)** - Docker Development
- **[Dev Setup](setup/dev_setup.md)** - Container-in-Container

### 🔐 Für Auth/Backend-Entwickler

- **[ZITADEL Setup](guides/zitadel.md)** - Authentifizierung konfigurieren
- **[Architecture](reference/architecture.md)** - Connect-RPC & API-Design
- **[Configuration](reference/config.md)** - Backend-Konfiguration

### 🎨 Für Frontend-Entwickler

- **[Architecture](reference/architecture.md)** - Frontend Monorepo & Shared Core
- **[Connections](reference/connections.md)** - API-Verbindungen
- **[Style Guide](development/style-guide.md)** - Code-Standards

### 🧠 Für Konzept/Protokoll-Interessierte

- **[Kernkonzept](../concept/kernkonzept.md)** – Problemraum, Triade, Cybernetic Loop
- **[Concept README](../concept/README.md)** – Übersicht über alle Protokoll- und Architekturkonzepte

---

## 📖 Dokumentations-Kategorien

### 📘 Guides (Schritt-für-Schritt)

**Ziel**: Praktische Anleitungen für häufige Aufgaben

- **[Getting Started](guides/getting-started.md)** - Schnellstart
- **[Setup (macOS)](setup/setup.md)** - Vollständige macOS Setup-Anleitung
- **[Dev Setup](setup/dev_setup.md)** - Container-in-Container Entwicklung
- **[Docker Setup](setup/docker.md)** - Docker Development Setup
- **[ZITADEL Setup](guides/zitadel.md)** - Authentifizierung konfigurieren

### 📗 Reference (Technische Referenz)

**Ziel**: Detaillierte technische Informationen

- **[Architecture](reference/architecture.md)** - Systemarchitektur, Tech Stack, Design-Entscheidungen
- **[Backend Architecture](reference/BACKEND-ARCHITECTURE.md)** - Backend-Schichten, UDM, Axiom-Mapping
- **[CLI Tool](reference/CLI-TOOL.md)** - ECL CLI Referenz, REPL, Bytecode-Kompilierung
- **[Configuration](reference/config.md)** - Service-Konfiguration, Ports, Verbindungen
- **[Connections](reference/connections.md)** - API-Verbindungen, Error-Handling Harmonisierung

### 📙 Development (Development-spezifisch)

**Ziel**: Code-Standards, Testing, TODOs

- **[Style Guide](development/style-guide.md)** - Code-Stil, Naming Conventions, File Organization
- **[Testing](development/testing.md)** - Test-Strategien, Tools, Best Practices
- **[todos](development/todos.md)** - Offene Aufgaben, Prioritäten, bekannte Issues
- **[IPS Implementation](development/IPS-01-imp.md)** - Mathematisches Logik-Modell (Kategorialtheorie)
- **[Unified Data Model](development/UNIFIED-DATA-MODEL.md)** - UDM Datenstruktur-Spezifikation
- **[P2P Implementation](development/P2P-IMPLEMENTATION.md)** - libp2p Netzwerk-Details
- **[IPS-UDM Gap Analysis](development/IPS-UDM-GAP-ANALYSIS.md)** - Implementierungs-Status
- **[REST Deprecation Plan](development/rest_deprecation_plan.md)** - Plan zur REST-API Entfernung
- **[Folder Structure Analysis](development/folder_structure_analysis.md)** - Struktur-Analyse & Optimierungsvorschläge

### 📦 Archive (Historische Dokumentation)

**Ziel**: Historische Dokumente, die nicht mehr aktiv verwendet werden

- **[Structure Improvements](archive/structure_improvements.md)** - Strukturverbesserungen (2026-01-25, archiviert)

---

## 🔍 Häufige Fragen

### "Wie starte ich das Projekt?"

→ **[Getting Started](guides/getting-started.md)** oder **[Dev Setup](setup/dev_setup.md)**

### "Wie richte ich die Entwicklungsumgebung ein?"

→ **[Setup (macOS)](setup/setup.md)** für Host-Setup oder **[Dev Setup](setup/dev_setup.md)** für DevContainer

### "Wie funktioniert die Architektur?"

→ **[Architecture](reference/architecture.md)**

### "Wie funktioniert das Erynoa-Protokoll konzeptionell?"

→ **[Kernkonzept](../concept/kernkonzept.md)** und **[Concept README](../concept/README.md)**

### "Welche Ports werden verwendet?"

→ **[Configuration](reference/config.md)** - Service-Konfiguration Sektion

### "Wie konfiguriere ich ZITADEL?"

→ **[ZITADEL Setup](guides/zitadel.md)**

### "Was sind die Code-Standards?"

→ **[Style Guide](development/style-guide.md)**

### "Was muss noch gemacht werden?"

→ **[todos](development/todos.md)**

### "Wie teste ich?"

→ **[Testing](development/testing.md)**

---

## 📋 Wichtige Dokumente

### 🚀 Quick Reference

- **[essential_guide.md](essential_guide.md)** - Konsolidierter Guide mit allen wichtigen Informationen
- **[readme.md](readme.md)** - Dokumentations-Übersicht
- **[documentation_status.md](documentation_status.md)** - Dokumentations-Status & Übersicht

### 🧩 Konzept & Protokoll

- **[Kernkonzept](../concept/kernkonzept.md)** – High-Level Protokollbeschreibung
- **[System Architecture Overview](../concept/system-architecture-overview.md)** – Triade ERY/ECHO/NOA & Layer-Modell

### 📝 Aktuelle Aufgaben

- **[todos](development/todos.md)** - Offene Aufgaben und Prioritäten

### 🏗️ Architektur

- **[Architecture](reference/architecture.md)** - Vollständige Systemarchitektur

---

## 🔄 Dokumentation aktualisieren

Wenn du Dokumentation aktualisierst:

1. **Datum aktualisieren**: `**Letzte Aktualisierung**: YYYY-MM-DD`
2. **Links prüfen**: Alle relativen Links sollten funktionieren
3. **Konsistenz**: Verwende die gleiche Terminologie wie in anderen Dokumenten
4. **Navigation**: Aktualisiere diese Datei, wenn neue Dokumente hinzugefügt werden

---

**Hinweis**: Diese Dokumentation wird kontinuierlich aktualisiert. Bei Fragen oder Verbesserungsvorschlägen, bitte ein Issue erstellen.
