# 🧭 Dokumentations-Navigation

**Letzte Aktualisierung**: 2026-01-27 (20:57)

**Status**: Aktuell und vollständig ✅

Diese Datei bietet eine Übersicht über die gesamte Dokumentationsstruktur und hilft dabei, schnell die richtige Dokumentation zu finden.

---

## 📚 Dokumentationsstruktur

```
docs/
├── readme.md                    # Hauptübersicht
├── essential_guide.md           # Konsolidierter Guide (alles Wichtige)
├── navigation.md                # Diese Datei
├── documentation_status.md      # Dokumentations-Status & Übersicht
│
├── guides/                      # Schritt-für-Schritt Anleitungen
│   ├── getting-started.md       # Schnellstart für neue Entwickler
│   ├── setup.md                 # Setup-Übersicht (verweist auf setup/)
│   └── zitadel.md               # ZITADEL Authentifizierung (automatisch)
│
├── setup/                       # Setup-Anleitungen
│   ├── setup.md                 # Vollständige macOS Setup-Anleitung
│   ├── dev_setup.md             # Container-in-Container Entwicklung
│   └── docker.md                # Docker Development Setup
│
├── reference/                   # Technische Referenz
│   ├── architecture.md          # Systemarchitektur & Design-Entscheidungen
│   ├── config.md                # Service-Konfiguration & Verbindungen
│   └── connections.md          # API-Verbindungen & Harmonisierung
│
├── development/                 # Development-spezifisch
│   ├── style-guide.md           # Code-Stil & Best Practices
│   ├── testing.md               # Test-Strategien & Tools (cargo-nextest)
│   ├── todos.md                 # Offene Aufgaben & Prioritäten
│   ├── rest_deprecation_plan.md # Plan zur REST-API Entfernung
│   └── folder_structure_analysis.md # Folder Structure Analysis & Optimization
│
└── archive/                     # Historische Dokumentation
    └── structure_improvements.md # Strukturverbesserungen (2026-01-25, archiviert)
```

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
- **[Configuration](reference/config.md)** - Service-Konfiguration, Ports, Verbindungen
- **[Connections](reference/connections.md)** - API-Verbindungen, Error-Handling Harmonisierung

### 📙 Development (Development-spezifisch)

**Ziel**: Code-Standards, Testing, TODOs

- **[Style Guide](development/style-guide.md)** - Code-Stil, Naming Conventions, File Organization
- **[Testing](development/testing.md)** - Test-Strategien, Tools, Best Practices
- **[todos](development/todos.md)** - Offene Aufgaben, Prioritäten, bekannte Issues
- **[REST Deprecation Plan](development/rest_deprecation_plan.md)** - Plan zur REST-API Entfernung
- **[Folder Structure Analysis](development/folder_structure_analysis.md)** - Struktur-Analyse & Optimierungsvorschläge
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
