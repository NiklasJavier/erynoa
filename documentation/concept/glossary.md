# Erynoa – Glossar

> **Dokumenttyp:** Referenz
> **Version:** 1.0
> **Status:** Living Document
> **Lesezeit:** Nachschlagewerk

---

## Übersicht

Dieses Glossar definiert die zentralen Begriffe des Erynoa-Protokolls. Es dient als gemeinsame Sprachbasis für alle Konzept-, Architektur- und Implementierungsdokumente.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   📖 GLOSSAR-STRUKTUR                                                       │
│                                                                             │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│   │  🏛️ SPHÄREN     │  │  📦 OBJEKTE     │  │  🔧 PROZESSE    │            │
│   │  ERY, ECHO, NOA │  │  AMO, Blueprint │  │  Loop, Streaming│            │
│   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│                                                                             │
│   ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐            │
│   │  🤝 TRUST       │  │  🤖 AGENTEN     │  │  🔌 TECHNOLOGIE │            │
│   │  Karmic, Vector │  │  Seeker, ADL    │  │  WASM, Move     │            │
│   └─────────────────┘  └─────────────────┘  └─────────────────┘            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 🏛️ Sphären

Die drei Sphären bilden das Fundament der Erynoa-Architektur.

| Begriff  | Definition                                                                                                                                                                       |
| :------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **ERY**  | **Semantic Lattice** – Das Gedächtnis des Netzwerks. Speichert Blueprints, Trust Vectors, Attestations und Fluid Extensions. Ermöglicht semantische Suchen und Trust-Berechnung. |
| **ECHO** | **Emergent Swarm** – Die operative Intelligenz. Führt Agentenlogik aus, wickelt Discovery, Verhandlung und Ausführung von Intents ab.                                            |
| **NOA**  | **Causal Ledger** – Die Quelle der Wahrheit. On-Chain-Ledger, der Transaktionen finalisiert und Zustandsänderungen an AMOs vollzieht.                                            |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   ERY ←──────→ ECHO ←──────→ NOA                            │
│   Semantik      Agenten      Finalität                       │
│   Trust         Verhandlung  Settlement                      │
│   Kontext       P2P          Wahrheit                        │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 📦 Objekte & Datenmodell

Begriffe rund um das liquide Datenmodell und seine Bausteine.

| Begriff                 | Definition                                                                                                                                          |
| :---------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------- |
| **AMO**                 | **Atomic Market Object** – Zentrale On-Chain-Entität in NOA. Digitaler Container, dessen Verhalten durch Blueprints und die MoveVM definiert ist.   |
| **Material AMO**        | AMO-Typ für physische Güter und Real World Assets (z.B. Ladesäulen, Maschinen, Sensoren). Transferierbar.                                           |
| **Credential AMO**      | AMO-Typ für immaterielle Nachweise (z.B. KYC, Zertifikate). Soulbound an eine DID – nicht transferierbar, nur verifizierbar.                        |
| **Service AMO**         | AMO-Typ für zeitgebundene Dienstleistungen (z.B. Ladevorgänge, API-Nutzung). Unterstützt Continuous Value Streaming.                                |
| **Blueprint**           | Semantische und technische Schablone für Objekte und Prozesse. Definiert Struktur, Constraints und Validierungslogik.                               |
| **Normative Standards** | Etablierte Industriestandards (ISO, eCl@ss, OCPP) als unveränderliche Grundlagen für Domain Blueprints.                                             |
| **Domain Blueprint**    | Anwendungsspezifische Definitionen mit Validierungsregeln, die auf Normative Standards aufbauen.                                                    |
| **Fluid Extensions**    | Temporäre Attribut-Erweiterungen von AMOs für flüchtige Daten (Geo-Position, Sensorwerte). Besitzen ein TTL.                                        |
| **EOS**                 | **Erynoa Object Standard** – Architektonische Grundlage des liquiden Datenmodells. Definiert die Beziehung zwischen Standards, Blueprints und AMOs. |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   NORMATIVE STANDARDS                                        │
│   (ISO, eCl@ss, OCPP)                                       │
│            │                                                 │
│            ▼                                                 │
│   DOMAIN BLUEPRINTS                                          │
│   (EV-Charging, KYC-Credential)                             │
│            │                                                 │
│            ▼                                                 │
│   AMO INSTANZEN                                              │
│   (Material │ Credential │ Service)                         │
│            │                                                 │
│            ├── Fluid Extensions (TTL)                       │
│            └── Trust Vector                                  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 🤝 Trust & Reputation

Begriffe rund um das Vertrauenssystem.

| Begriff               | Definition                                                                                                                      |
| :-------------------- | :------------------------------------------------------------------------------------------------------------------------------ |
| **Trust Vector**      | Mehrdimensionaler Vektor, der das Vertrauen in ein Subjekt beschreibt. Wird von der Karmic Engine berechnet.                    |
| **Karmic Engine**     | Komponente in ERY, die aus Events und Attestations Trust Vectors berechnet. Nutzt den Ripple-Effekt für dynamische Updates.     |
| **Attestation**       | Signierte Aussage einer externen oder internen Instanz über ein Subjekt (z.B. DNS-Bindung, Zertifikate, Konformitätsnachweise). |
| **Trust Gating**      | Mechanismus, bei dem minimale Trust-Schwellen als Zugangskriterium für Interaktionen dienen.                                    |
| **Trust Inheritance** | Fraktale Vererbung von Trust entlang hierarchischer Beziehungen (Hersteller → Betreiber → Asset).                               |
| **Ripple Effect**     | Algorithmus zur Propagation von Trust-Änderungen durch das Netzwerk mit Dämpfungsfaktor λ.                                      |
| **Event**             | Abstraktion eines finalisierten Vorgangs in NOA (z.B. erfolgreiche Lieferung, SLA-Verstoß). Input für die Karmic Engine.        |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   TRUST INPUTS                    TRUST OUTPUT               │
│   ═══════════                     ════════════               │
│                                                              │
│   📈 Events (aus NOA)     ─┐                                 │
│   🎫 Attestations         ─┼──▶  KARMIC ENGINE  ──▶  Trust   │
│   🧬 Inheritance          ─┘                         Vector  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 🤖 Agenten & ADL

Begriffe rund um autonome Agenten und ihre Sprache.

| Begriff                    | Definition                                                                                                                   |
| :------------------------- | :--------------------------------------------------------------------------------------------------------------------------- |
| **Agent**                  | Autonome Software-Entität, die im Namen eines Nutzers oder einer Organisation handelt. Läuft isoliert in einer WASM-Sandbox. |
| **Seeker Agent**           | Agentenrolle für Nachfrager. Formuliert Intents, führt Discovery durch, wählt Provider aus.                                  |
| **Provider Agent**         | Agentenrolle für Anbieter. Publiziert Capabilities, empfängt Anfragen, führt Services aus.                                   |
| **ADL**                    | **Agent Definition Language** – Deklarative Sprache zur Beschreibung von Intents, Constraints und Policies.                  |
| **Intent**                 | Maschinenlesbarer Wunsch eines Seekers, bestehend aus funktionalen, normativen und Trust-Anforderungen.                      |
| **Policy**                 | Deklarative Regel eines Providers, die definiert, unter welchen Bedingungen Anfragen akzeptiert werden.                      |
| **Consensus Bubble**       | Verschlüsselte Off-Chain-Kommunikationsumgebung (XMTP), in der Agenten privat verhandeln.                                    |
| **Progressive Disclosure** | Prinzip, nach dem sensible Informationen nur schrittweise offengelegt werden, wenn Vertrauen gegeben ist.                    |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   SEEKER                          PROVIDER                   │
│   ══════                          ════════                   │
│                                                              │
│   ┌──────────┐                    ┌──────────┐              │
│   │  Intent  │   ──Negotiate──▶   │  Policy  │              │
│   │  (ADL)   │   ◀────────────    │  (ADL)   │              │
│   └──────────┘                    └──────────┘              │
│        │                               │                     │
│        └───────── Consensus ───────────┘                     │
│                    Bubble                                    │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 🔄 Prozesse & Workflows

Begriffe rund um den Cybernetic Loop und Abrechnungsmodelle.

| Begriff                        | Definition                                                                                                |
| :----------------------------- | :-------------------------------------------------------------------------------------------------------- |
| **Cybernetic Loop**            | Der universelle 6-Phasen-Workflow: Sensing → Discovery → Validation → Negotiation → Execution → Feedback. |
| **Discovery**                  | Phase, in der ERY nach passenden Blueprints, AMOs und Providern gesucht wird.                             |
| **Validation**                 | Prüfung von Trust-Schwellen und Attestations vor der Verhandlung (Trust Gating).                          |
| **Negotiation**                | Bilaterale Abstimmung in Consensus Bubbles über Preis, Konditionen und Details.                           |
| **Execution**                  | Ausführung des vereinbarten Services mit atomarer Finalisierung auf NOA.                                  |
| **Feedback**                   | Emission von Events nach Abschluss, die in die Karmic Engine fließen.                                     |
| **Continuous Value Streaming** | Abrechnungsmodell mit kontinuierlichem, fein granularem Werttransfer (z.B. €/kWh in Echtzeit).            |
| **Logic Guards**               | Smart-Contract-artige Prüfmechanismen in NOA, die vor jeder Zustandsänderung Invarianten sicherstellen.   |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│              ┌─────────┐                                     │
│        ┌────▶│1 INTENT │────┐                                │
│        │     └─────────┘    │                                │
│        │                    ▼                                │
│   ┌─────────┐          ┌─────────┐                          │
│   │6 FEED-  │          │2 DISCO- │                          │
│   │  BACK   │          │  VERY   │                          │
│   └─────────┘          └─────────┘                          │
│        ▲                    │                                │
│        │                    ▼                                │
│   ┌─────────┐          ┌─────────┐                          │
│   │5 EXECU- │◀─────────│3 TRUST  │                          │
│   │  TION   │          │ GATING  │                          │
│   └─────────┘          └─────────┘                          │
│        ▲     ┌─────────┐    │                                │
│        └─────│4 NEGOTI-│◀───┘                                │
│              │  ATION  │                                     │
│              └─────────┘                                     │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 🔌 Technologie & Infrastruktur

Technische Begriffe und Protokolle.

| Begriff          | Definition                                                                                                   |
| :--------------- | :----------------------------------------------------------------------------------------------------------- |
| **MoveVM**       | Virtuelle Maschine in NOA. Optimiert auf Resource Safety und formale Kontrolle über Assets.                  |
| **Move**         | Programmiersprache für Smart Contracts in NOA. Garantiert lineare Typen und verhindert Asset-Duplikation.    |
| **Starfish BFT** | Leaderloser Konsensmechanismus in NOA. Deterministische Finalität in unter 2 Sekunden.                       |
| **WASM**         | **WebAssembly** – Portable, sichere Runtime für Agenten in ECHO. Ermöglicht sprachagnostische Entwicklung.   |
| **XMTP**         | **Extensible Message Transport Protocol** – Protokoll für verschlüsselte Nachrichtenkanäle zwischen Agenten. |
| **Qdrant**       | Vektor-Datenbank für den Semantic Index in ERY. Ermöglicht semantische Ähnlichkeitssuchen.                   |
| **DHT**          | **Distributed Hash Table** – Verteilte Datenstruktur zur Partitionierung und Auffindbarkeit von Daten.       |
| **Geohashing**   | Kodierung geographischer Regionen in kompakte Strings. Für räumliche Partitionierung und Geo-Constraints.    |
| **DID**          | **Decentralized Identifier** – Dezentraler, kryptografisch gesicherter Identifikator für Akteure.            |
| **Synapse**      | Elementare, inhaltsadressierte Speichereinheit in ERY. Grundlage der synaptischen Sharding-Architektur.      |
| **TTL**          | **Time-To-Live** – Lebensdauer eines flüchtigen Dateneintrags. Nach Ablauf automatische Entfernung.          |

---

## 📚 Schichtmodell

Zusammenfassung der Architektur-Layer.

| Layer       | Sphären    | Aufgaben                                                |
| :---------- | :--------- | :------------------------------------------------------ |
| **Layer 0** | NOA        | On-Chain: Finalität, AMOs, Move-Execution, Starfish BFT |
| **Layer 2** | ERY + ECHO | Off-Chain: Semantik, Trust, Agenten, Verhandlung        |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   LAYER 2 (Off-Chain)                                       │
│   ┌─────────────────────┐  ┌─────────────────────┐          │
│   │        ERY          │  │        ECHO         │          │
│   │  Semantic Lattice   │  │   Emergent Swarm    │          │
│   └─────────────────────┘  └─────────────────────┘          │
│                    │                    │                    │
│                    └────────┬───────────┘                    │
│                             │                                │
│   ─────────────────────────────────────────────────────────  │
│                             │                                │
│   LAYER 0 (On-Chain)        ▼                                │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                        NOA                          │   │
│   │                   Causal Ledger                     │   │
│   └─────────────────────────────────────────────────────┘   │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## Kurzreferenz (Alphabetisch)

| Begriff                | Kurzdefinition                             |
| :--------------------- | :----------------------------------------- |
| ADL                    | Agent Definition Language                  |
| AMO                    | Atomic Market Object                       |
| Attestation            | Signierte Aussage über ein Subjekt         |
| Blueprint              | Semantische Schablone für Objekte          |
| Consensus Bubble       | Verschlüsselte Verhandlungsumgebung        |
| DID                    | Decentralized Identifier                   |
| DHT                    | Distributed Hash Table                     |
| ECHO                   | Emergent Swarm (Agenten-Sphäre)            |
| EOS                    | Erynoa Object Standard                     |
| ERY                    | Semantic Lattice (Wissens-Sphäre)          |
| Fluid Extensions       | Temporäre AMO-Attribute mit TTL            |
| Geohashing             | Geo-Koordinaten als kompakte Strings       |
| Intent                 | Maschinenlesbarer Wunsch eines Seekers     |
| Karmic Engine          | Trust-Berechnungskomponente                |
| Logic Guards           | Invarianten-Prüfung vor Zustandsänderungen |
| Move/MoveVM            | Sprache und VM für Smart Contracts         |
| NOA                    | Causal Ledger (Finalitäts-Sphäre)          |
| Policy                 | Deklarative Annahme-Regeln eines Providers |
| Progressive Disclosure | Schrittweise Informationsfreigabe          |
| Qdrant                 | Vektor-Datenbank für ERY                   |
| Ripple Effect          | Trust-Propagation mit Dämpfung             |
| Starfish BFT           | Leaderloser Konsens in NOA                 |
| Synapse                | Elementare Speichereinheit in ERY          |
| Trust Gating           | Trust-Schwellen als Zugangskriterium       |
| Trust Vector           | Mehrdimensionale Trust-Repräsentation      |
| TTL                    | Time-To-Live                               |
| WASM                   | WebAssembly Runtime für Agenten            |
| XMTP                   | Verschlüsseltes Messaging-Protokoll        |

---

## Weiterführende Dokumente

| Dokument                                               | Inhalt                           |
| :----------------------------------------------------- | :------------------------------- |
| [Fachkonzept](./fachkonzept.md)                        | Vollständige Spezifikation       |
| [Kernkonzept](./kernkonzept.md)                        | High-Level-Überblick             |
| [Systemarchitektur](./system-architecture-overview.md) | Technische Architektur           |
| [Liquides Datenmodell](./liquides-datenmodell.md)      | Blueprints, AMOs, Extensions     |
| [Trust & Reputation](./trust-and-reputation.md)        | Karmic Engine, Trust Vectors     |
| [Cybernetic Loop](./cybernetic-loop.md)                | Der 6-Phasen-Workflow            |
| [Agents & ADL](./agents-and-adl.md)                    | Agentenmodell und Intent-Sprache |
| [Use Cases](./use-cases.md)                            | Praktische Anwendungsszenarien   |
