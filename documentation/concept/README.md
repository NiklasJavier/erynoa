# Erynoa – Concept Navigation

> **Zielgruppe:** Alle Leser:innen der Erynoa-Konzeptdokumente
> **Kontext:** Einstiegskarte und Orientierungshilfe für das Konzeptverzeichnis
> **ECL Version:** 2.1 – Identity-First + ECLVM

---

## Empfohlene Lesereihenfolge

| Schritt | Dokument                                                            | Fokus                                    |
| ------- | ------------------------------------------------------------------- | ---------------------------------------- |
| 1       | [Kernkonzept](./kernkonzept.md)                                     | Überblick & Motivation                   |
| 2       | [System Architecture Overview](./system-architecture-overview.md)   | Technische Systemebene                   |
| 3       | [Erynoa Configuration Language](./erynoa-configuration-language.md) | **ECL v2.1 – Die Systemsprache**         |
| 4       | [Liquides Datenmodell](./liquides-datenmodell.md)                   | Blueprints & AMOs (ecl/object)           |
| 5       | [Trust & Reputation](./trust-and-reputation.md)                     | Vertrauensmodell (ecl/trust)             |
| 6       | [Search Environments](./search-environments.md)                     | Suchordnungen + Governance (ecl/environ) |
| 7       | [Cybernetic Loop](./cybernetic-loop.md)                             | Universeller Workflow                    |
| 8       | [Agents & ADL](./agents-and-adl.md)                                 | Agentenmodell + ECLVM (ecl/agent)        |
| 9       | [Use Cases](./use-cases.md)                                         | Praktische Beispiele                     |

---

## Dokumentenstruktur

### Einstieg & Überblick

- [Kernkonzept](./kernkonzept.md) – High-Level-Einführung für Business/Product
- [System Architecture Overview](./system-architecture-overview.md) – Brücke zu technischen Details
- [Glossar](./glossary.md) – Zentrale Begriffsdefinitionen (v1.2)

### Systemsprache: ECL v2.1

- [Erynoa Configuration Language](./erynoa-configuration-language.md) – **Die einheitliche Sprache für alles**
  - **ecl/core** – Basis-Typen, Syntax, Validierung
  - **ecl/vm** – ECLVM: Dynamische Programmierung, Templates, Sandbox ⚡ NEU
  - **ecl/identity** – DIDs, Sub-Identities, Verifiable Credentials
  - **ecl/object** – Blueprints, AMOs, Standards
  - **ecl/environ** – Environments mit Legislative/Executive Governance
  - **ecl/agent** – Intents, Policies, ECLVM-Programme
  - **ecl/trust** – Trust Vectors, Karmic Engine
  - **ecl/economic** – Wallets, Payment, Streaming
  - **ecl/network** – Multi-Chain, Bridges
  - **ecl/governance** – DAOs, Voting, Proposals
  - **ecl/test** – Test Suites, Mocks

### Daten- & Vertrauensmodell (ecl/object, ecl/trust)

- [Liquides Datenmodell](./liquides-datenmodell.md) – Identity-First Blueprints, AMOs, Fluid Extensions
- [Trust & Reputation](./trust-and-reputation.md) – Karmic Engine, Trust Vectors, Karma-Tiers, Trust-Gating

### Discovery & Suchumgebungen (ecl/environ)

- [Search Environments](./search-environments.md) – Hierarchische Suchordnungen, **Environment Governance** (Legislative/Executive)

### Prozesse & Agenten (ecl/agent, ecl/vm)

- [Cybernetic Loop](./cybernetic-loop.md) – Der universelle Workflow von Intent bis Feedback
- [Agents & ADL](./agents-and-adl.md) – Seeker/Provider-Agenten, **ECLVM-Programme**, Sub-Identities

### Use Cases & Narrative

- [Use Cases](./use-cases.md) – Konkrete Anwendungsszenarien (E-Mobility, Wartung, Energie)

---

## Verknüpfung mit der Dokumentation

| Konzeptdokument     | Relevante Docs                                               |
| ------------------- | ------------------------------------------------------------ |
| System Architecture | [Backend-Architektur](../system/reference/architecture.md)   |
| Alle Konzepte       | [Essential Guide](../system/essential_guide.md)              |
| Deployment          | [Unified Deployment](../system/guides/unified-deployment.md) |
| Erste Schritte      | [Getting Started](../system/guides/getting-started.md)       |

---

## Legende: Die kybernetische Triade + ECLVM

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      ERYNOA PROTOKOLL v2.1                              │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                  │
│   │     ERY     │   │    ECHO     │   │     NOA     │                  │
│   │  Semantic   │◄──│  Emergent   │──►│   Causal    │                  │
│   │  Lattice    │   │   Swarm     │   │   Ledger    │                  │
│   └─────────────┘   └─────────────┘   └─────────────┘                  │
│         ▲                 │                  │                          │
│         │           ┌─────┴─────┐            │                          │
│         │           │   ECLVM   │            │                          │
│         │           │  Sandbox  │            │                          │
│         │           └───────────┘            │                          │
│         │         Feedback Loop              │                          │
│         └────────────────────────────────────┘                          │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

- **ERY:** Semantik & Gedächtnis (Identity, Blueprints, Trust Vectors, Governance)
- **ECHO:** Intelligenz & Agenten (Seeker/Provider, ECLVM-Programme, Verhandlung)
- **NOA:** Wahrheit & Exekution (AMOs, MoveVM, Starfish BFT)
- **ECLVM:** Execution Engine – Agenten programmieren dynamisch (Templates, Hot-Reload)

**ECL v2.1** verbindet alle Sphären: Identity-First + Dynamische Programmierung.

---

## Was ist neu in v2.1?

| Feature                    | Beschreibung                                               |
| -------------------------- | ---------------------------------------------------------- |
| **ECLVM**                  | Layer 0.5 Execution Engine mit ~100 Opcodes, Templates     |
| **Sub-Identities**         | 16 spezialisierte Identity-Typen für granulare Kontrolle   |
| **Karma Tiers**            | Gestaffelte Trust-Level (Newcomer → Elder)                 |
| **Asymmetrie**             | Negative Events wiegen 1.5× stärker                        |
| **Environment Governance** | Legislative (Regeln) + Executive (Councils)                |
| **Hot-Code-Reload**        | Agenten aktualisieren Funktionen live ohne Neustart        |
| **Template System**        | Blueprint, Environment, Agent Templates für Instantiierung |

**ECL** (Erynoa Configuration Language) verbindet alle drei Sphären als einheitliche Konfigurationssprache.
