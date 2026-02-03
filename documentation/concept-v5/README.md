# Erynoa Unified Specification V5.0

> **Version:** 5.0 – Konsolidierte & Vollständige Spezifikation
> **Datum:** Januar 2026
> **Status:** Produktionsreif
> **Basiert auf:** concept-v4 (LOGIC, SYSTEM-ARCHITECTURE, STATE-MANAGEMENT, FACHKONZEPT, CLI-REFERENCE)

---

## Übersicht

Diese konsolidierte Spezifikation vereint alle Aspekte des Erynoa-Systems in einer kohärenten, logisch abgestimmten Dokumentation. Concept-v5 eliminiert Redundanzen, schließt logische Lücken und bietet eine vollständige Referenz für Entwickler, Architekten und Stakeholder.

### Dokumentenstruktur

```
concept-v5/
├── README.md                     # Diese Übersicht
├── 01-VISION-AND-FOUNDATIONS.md  # Vision, Kernkonzepte, Grundlagen
├── 02-AXIOM-SYSTEM.md            # 28 Kern-Axiome + 4 Unter-Axiome
├── 03-SYSTEM-ARCHITECTURE.md     # 4-Schichten-Architektur
├── 04-STATE-MANAGEMENT.md        # StateGraph, Propagation, Thread-Safety
├── 05-IMPLEMENTATION-GUIDE.md    # Technologie-Stack, Code-Beispiele
├── 06-CLI-REFERENCE.md           # Vollständige Befehlsreferenz
└── 07-APPENDIX.md                # Glossar, Referenzen, Changelog
```

### Schnellreferenz: Die 28 Kern-Axiome

| Kategorie               | Axiome  | Beschreibung                                  |
| ----------------------- | ------- | --------------------------------------------- |
| Kategorische Fundierung | Κ1-Κ2   | Regelvererbung, Trust-Funktor                 |
| Trust-Algebra           | Κ3-Κ5   | 6D-Vektor, Asymmetrie, Kombination            |
| Identitäts-Algebra      | Κ6-Κ8   | DID-Eindeutigkeit, Permanenz, Delegation      |
| Kausale Algebra         | Κ9-Κ10  | DAG-Struktur, Bezeugung-Finalität             |
| Prozess-Algebra         | Κ11-Κ14 | Hoare-Korrektheit, Atomarität, Fairness       |
| Weltformel              | Κ15a-d  | Surprisal, Trust-Norm, Parameter, Skalierung  |
| Humanismus              | Κ16-Κ17 | Human-Alignment, Temporale Vergebung          |
| Konsens                 | Κ18     | Gewichteter Partition-Konsens                 |
| Schutz                  | Κ19-Κ21 | Anti-Calcification, Diversity, Quadratic      |
| Peer-Logik              | Κ22-Κ24 | Intent-Auflösung, Gateway, Funktor            |
| System-Garantien        | Κ25-Κ28 | Determinismus, Offenheit, Verhältnismäßigkeit |

### Die Weltformel V2.0

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                                                                               ║
║   𝔼 = Σ  𝔸(s) · σ⃗( ‖𝕎(s)‖_w · ln|ℂ(s)| · 𝒮(s) ) · Ĥ(s) · w(s,t)             ║
║       s∈𝒞                                                                     ║
║                                                                               ║
║   wobei 𝒮(s) = ‖𝕎(s)‖² · ℐ(s)     [Trust-gedämpfte Surprisal]                ║
║         ℐ(s) = −log₂ P(e | ℂ(s))  [Shannon-Surprisal]                         ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### 4-Schichten-Architektur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Client / Peer Layer (Κ22-Κ24)                        │
│  ┌─────────────┐  ┌──────────────────┐  ┌──────────────────────────────┐   │
│  │Intent Parser│──│  Saga Composer   │──│       Gateway Guard          │   │
│  └─────────────┘  └──────────────────┘  └──────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│                         Core Logic Layer (Κ2-Κ18)                           │
│  ┌───────────┐  ┌────────────┐  ┌────────────┐  ┌─────────────────────┐   │
│  │Event (DAG)│──│Trust Engine│──│World Formula│──│  Consensus Engine  │   │
│  └───────────┘  └────────────┘  └────────────┘  └─────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│                       Storage / Realm Layer (Κ1)                            │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────────────────────────┐   │
│  │Event Store │──│Identity Store│──│   Realm Hierarchy (Root→Part)   │   │
│  └────────────┘  └──────────────┘  └──────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│                       Protection Layer (Κ19-Κ21)                            │
│  ┌──────────────────┐  ┌────────────────┐  ┌────────────────────────┐    │
│  │Anti-Calcification│──│Diversity Monitor│──│ Quadratic Governance │    │
│  └──────────────────┘  └────────────────┘  └────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Navigationshinweise

### Für Einsteiger


1. Beginne mit [01-VISION-AND-FOUNDATIONS](01-VISION-AND-FOUNDATIONS.md) für das Gesamtverständnis
2. Lies [02-AXIOM-SYSTEM](02-AXIOM-SYSTEM.md) für die mathematischen Grundlagen
3. Studiere [06-CLI-REFERENCE](06-CLI-REFERENCE.md) für praktische Anwendung


### Für Entwickler

1. [03-SYSTEM-ARCHITECTURE](03-SYSTEM-ARCHITECTURE.md) für Architektur-Übersicht
2. [04-STATE-MANAGEMENT](04-STATE-MANAGEMENT.md) für interne Strukturen
3. [05-IMPLEMENTATION-GUIDE](05-IMPLEMENTATION-GUIDE.md) für Code-Beispiele


### Für Architekten

1. Alle Dokumente in Reihenfolge
2. Besonderer Fokus auf Axiom-Mapping in jedem Abschnitt

---

## Änderungen gegenüber V4

| Aspekt                   | V4                     | V5                                  |
| ------------------------ | ---------------------- | ----------------------------------- |
| Dokumentstruktur         | 8 separate Dateien     | 7 konsolidierte, logisch verbundene |
| Axiom-Darstellung        | Verstreut              | Zentral in 02-AXIOM-SYSTEM.md       |
| State-Management         | Eigenständig           | Integriert mit Architektur          |
| Redundanzen              | Mehrfache Definitionen | Single Source of Truth              |
| Querverweise             | Implizit               | Explizite Axiom-Tags überall        |
| Implementierungs-Details | In SYSTEM-ARCHITECTURE | Separates IMPLEMENTATION-GUIDE      |

---

_Erynoa Unified Specification V5.0 – Dezentrales Vertrauen, mathematisch fundiert._
