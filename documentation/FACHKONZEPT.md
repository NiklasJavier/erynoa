# Erynoa – Fachkonzept

> **Dokumenttyp:** Fachkonzept (Business Requirements Specification)
> **Version:** 2.0
> **Status:** Konsolidiert
> **Datum:** Januar 2026
> **Zielgruppe:** Produktmanagement, Stakeholder, Investoren, Business Analysten
> **Referenz:** [Konzept-Navigator](./concept-v2/00-navigator.md) | [Roadmap](./ROADMAP.md)

---

## Inhaltsverzeichnis

1. [Zusammenfassung](#1-zusammenfassung)
2. [Problemstellung](#2-problemstellung)
3. [Lösungsansatz](#3-lösungsansatz)
4. [Systemarchitektur](#4-systemarchitektur)
5. [Fachliche Domänen](#5-fachliche-domänen)
6. [Geschäftsobjekte](#6-geschäftsobjekte)
7. [Prozesse und Abläufe](#7-prozesse-und-abläufe)
8. [Anwendungsfälle](#8-anwendungsfälle)
9. [Nicht-funktionale Anforderungen](#9-nicht-funktionale-anforderungen)
10. [Governance und Compliance](#10-governance-und-compliance)
11. [Wirtschaftsmodell](#11-wirtschaftsmodell)
12. [Risiken und Mitigationen](#12-risiken-und-mitigationen)
13. [Glossar](#13-glossar)

---

## 1. Zusammenfassung

### 1.1 Vision

**Erynoa** ist ein kybernetisches Protokoll für die Maschinenökonomie – ein System, das Maschinen befähigt, eigenständig zu handeln, zu verhandeln und voneinander zu lernen, mit mathematisch fundiertem Vertrauen statt zentraler Autoritäten.

### 1.2 Kernaussage

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   "Erynoa ermöglicht autonome Machine-to-Machine-Transaktionen             │
│    durch dezentrale Identität, semantische Interoperabilität               │
│    und kybernetische Feedbackschleifen."                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Wertversprechen

| Stakeholder     | Nutzen                                                      |
| --------------- | ----------------------------------------------------------- |
| **Unternehmen** | Automatisierung komplexer B2B-Prozesse, Kostensenkung       |
| **Entwickler**  | Standardisierte APIs und Protokolle für M2M-Kommunikation   |
| **Endanwender** | Nahtlose, autonome Services ohne manuelle Intervention      |
| **Regulatoren** | Auditierbare, compliance-fähige Transaktionen               |
| **Investoren**  | Skalierbare Plattform für die aufkommende Maschinenökonomie |

### 1.4 Differenzierung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   VERGLEICH: TRADITIONELL vs. ERYNOA                                       │
│                                                                             │
│   ┌─────────────────────────────────┬─────────────────────────────────────┐│
│   │        TRADITIONELL             │           ERYNOA                    ││
│   ├─────────────────────────────────┼─────────────────────────────────────┤│
│   │ Zentralisierte Identität        │ Dezentrale DIDs (Self-Sovereign)    ││
│   │ Silobasierte Daten              │ Semantisch vernetzte Ontologie      ││
│   │ Manuelles Vertrauen             │ Mathematisch berechnete Trust-Werte ││
│   │ Statische Verträge              │ Dynamische Smart Policies           ││
│   │ Batch-Transaktionen             │ Streaming-Zahlungen                 ││
│   │ Nachträgliche Audits            │ Echtzeitbeweis (Causal Ledger)      ││
│   │ Monolinguale Systeme            │ Multi-Chain-Interoperabilität       ││
│   └─────────────────────────────────┴─────────────────────────────────────┘│
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Problemstellung

### 2.1 Marktsituation

Die Maschinenökonomie (Machine Economy) wächst exponentiell. IoT-Geräte, autonome Fahrzeuge, intelligente Infrastruktur und KI-Agenten erzeugen zunehmend wirtschaftliche Transaktionen ohne menschliche Intervention.

**Aktuelle Marktgrößen:**

| Segment              | 2024      | 2030 (progn.) | CAGR  |
| -------------------- | --------- | ------------- | ----- |
| IoT-Markt            | $714 Mrd. | $1.5 Bio.     | 13.5% |
| Autonome Fahrzeuge   | $54 Mrd.  | $556 Mrd.     | 39.1% |
| Smart Infrastructure | $89 Mrd.  | $265 Mrd.     | 19.5% |
| M2M-Payments         | $12 Mrd.  | $89 Mrd.      | 39.8% |

### 2.2 Kernprobleme

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   PROBLEM 1: FRAGMENTIERTE IDENTITÄT                                       │
│   ═══════════════════════════════════                                       │
│                                                                             │
│   • Maschinen haben keine souveräne digitale Identität                     │
│   • Jedes System verwendet eigene ID-Schemata (keine Interoperabilität)    │
│   • Keine kryptografische Verifikation von Maschinen-Identitäten           │
│   • Identitäts-Silos verhindern Cross-Platform-Transaktionen               │
│                                                                             │
│   Konsequenz: Maschinen können nicht vertrauenswürdig miteinander          │
│               interagieren.                                                 │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   PROBLEM 2: FEHLENDES MASCHINENVERTRAUEN                                  │
│   ════════════════════════════════════════                                  │
│                                                                             │
│   • Kein Mechanismus für Trust zwischen unbekannten Maschinen              │
│   • Reputation ist nicht portabel (gilt nur in einem System)               │
│   • Keine Abstraktion für "Zuverlässigkeit", "Ehrlichkeit", "Fähigkeit"   │
│   • Vertrauen basiert auf zentralen Autoritäten (Single Point of Failure)  │
│                                                                             │
│   Konsequenz: Autonome Transaktionen erfordern immer noch menschliche      │
│               Aufsicht oder zentrale Intermediäre.                         │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   PROBLEM 3: SEMANTISCHE INKOMPATIBILITÄT                                  │
│   ═══════════════════════════════════════                                   │
│                                                                             │
│   • Verschiedene Systeme verstehen Objekte unterschiedlich                 │
│   • Keine gemeinsame Ontologie für "Ladesäule", "Fahrzeug", "Service"     │
│   • Standards (ISO, OCPP, etc.) sind nicht maschinenlesbar verknüpft       │
│   • Semantik geht bei Systemübergängen verloren                            │
│                                                                             │
│   Konsequenz: Jede Integration erfordert Custom-Mappings und Adapter.      │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   PROBLEM 4: TRANSAKTIONSINEFFIZIENZ                                       │
│   ══════════════════════════════════                                        │
│                                                                             │
│   • Zahlungen sind batch-basiert, nicht streaming-fähig                    │
│   • Hohe Transaktionskosten für Mikrotransaktionen                         │
│   • Keine echtzeitfähige Wertstromverrechnung                              │
│   • Settlement dauert Tage (nicht Sekunden)                                │
│                                                                             │
│   Konsequenz: Viele M2M-Geschäftsmodelle sind wirtschaftlich nicht tragbar.│
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   PROBLEM 5: MANGELNDE NACHWEISBARKEIT                                     │
│   ════════════════════════════════════                                      │
│                                                                             │
│   • Keine unveränderliche Protokollierung von M2M-Transaktionen            │
│   • Kausalität von Events ist nicht nachvollziehbar                        │
│   • Compliance-Audits erfordern manuelle Rekonstruktion                    │
│   • Streitfälle haben keine objektive Beweisgrundlage                      │
│                                                                             │
│   Konsequenz: Regulatoren und Versicherer können M2M nicht bewerten.       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Betroffene Branchen

| Branche           | Spezifisches Problem                      |
| ----------------- | ----------------------------------------- |
| **E-Mobilität**   | Fragmentierte Lade-Roaming-Netze          |
| **Energie**       | Keine P2P-Energiehandels-Infrastruktur    |
| **Logistik**      | Autonome Flotten ohne Trust-Framework     |
| **Smart City**    | Keine interoperable IoT-Governance        |
| **Finanzwesen**   | KYC nicht portabel zwischen Institutionen |
| **Industrie 4.0** | Keine M2M-Vertragsautomatisierung         |

---

## 3. Lösungsansatz

### 3.1 Grundprinzipien

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ERYNOA DESIGN PRINCIPLES                                                 │
│                                                                             │
│   1️⃣ IDENTITY-FIRST                                                         │
│      "Alles existiert, weil es identifizierbar ist."                       │
│      → Jede Entität (Agent, Objekt, Regel) hat eine eindeutige DID.        │
│                                                                             │
│   2️⃣ KAUSALITÄT                                                             │
│      "Etwas kann nur handeln, wenn es existiert."                          │
│      → Schichten bauen aufeinander auf (keine Shortcuts).                  │
│                                                                             │
│   3️⃣ KYBERNETISCHE REGELKREISE                                              │
│      "Das System lernt aus seinen Ergebnissen."                            │
│      → Feedback-Loops aktualisieren Trust und Wissen kontinuierlich.       │
│                                                                             │
│   4️⃣ DEZENTRALE AUTONOMIE                                                   │
│      "Keine zentrale Autorität kontrolliert das System."                   │
│      → Self-Anchoring, Self-Governance, Self-Sovereignty.                  │
│                                                                             │
│   5️⃣ PROGRESSIVE DEZENTRALISIERUNG                                          │
│      "Vom einfachen Start zur vollständigen Dezentralität."                │
│      → System kann graduell dezentralisiert werden.                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Die Kybernetische Triade

Erynoa besteht aus drei komplementären Sphären:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   DIE KYBERNETISCHE TRIADE: ERY · ECHO · NOA                               │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │                           ╔═══════════╗                            │  │
│   │                           ║    ERY    ║                            │  │
│   │                           ║ ═════════ ║                            │  │
│   │                           ║  Wissen   ║                            │  │
│   │                           ║ Identität ║                            │  │
│   │                           ║ Vertrauen ║                            │  │
│   │                           ╚═════╤═════╝                            │  │
│   │                                 │                                   │  │
│   │                          KONTEXT                                   │  │
│   │                                 │                                   │  │
│   │              ┌──────────────────┴──────────────────┐               │  │
│   │              │                                     │               │  │
│   │              ▼                                     ▼               │  │
│   │       ╔═══════════╗                         ╔═══════════╗         │  │
│   │       ║   ECHO    ║                         ║    NOA    ║         │  │
│   │       ║ ═════════ ║                         ║ ═════════ ║         │  │
│   │       ║  Agenten  ║                         ║  Ledger   ║         │  │
│   │       ║   Schwarm ║                         ║  Beweis   ║         │  │
│   │       ║  Emergenz ║                         ║  Kausal   ║         │  │
│   │       ╚═════╤═════╝                         ╚═════╤═════╝         │  │
│   │             │                                     │               │  │
│   │        HANDLUNG                              FINALITÄT             │  │
│   │             │                                     │               │  │
│   │             └──────────────────┬──────────────────┘               │  │
│   │                                │                                   │  │
│   │                           FEEDBACK                                 │  │
│   │                                │                                   │  │
│   │                                ▼                                   │  │
│   │                    ┌───────────────────────┐                      │  │
│   │                    │   Trust-Update        │                      │  │
│   │                    │   Karma-Update        │                      │  │
│   │                    │   Wissens-Update      │                      │  │
│   │                    └───────────────────────┘                      │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   ERY = Semantisches Gitter (Identität, Wissen, Vertrauen, Räume)         │
│   ECHO = Emergenter Schwarm (Agenten, Intents, Verhandlung)               │
│   NOA = Kausales Hauptbuch (Objekte, Transaktionen, Beweis)               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 3.3 Die Sieben Schichten

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                    E R Y N O A   S C H I C H T E N                         │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   SCHICHT │ SYMBOL │ NAME    │ FRAGE          │ SPHÄRE │ FUNKTION  │  │
│   │   ════════════════════════════════════════════════════════════════  │  │
│   │                                                                     │  │
│   │      0    │   ◉    │ ANKER   │ "Wer existiert?"│  ERY  │ Identität │  │
│   │           │        │         │                 │       │           │  │
│   │      1    │   ◈    │ SCHEMA  │ "Was ist das?"  │  ERY  │ Wissen    │  │
│   │           │        │         │                 │       │           │  │
│   │      2    │   ◊    │ METRIK  │ "Wie gut?"      │  ERY  │ Vertrauen │  │
│   │           │        │         │                 │       │           │  │
│   │      3    │   ▣    │ SPHÄRE  │ "Wo gilt was?"  │  ERY  │ Räume     │  │
│   │           │        │         │                 │       │           │  │
│   │      4    │   ◐    │ IMPULS  │ "Was geschieht?"│ ECHO  │ Handlung  │  │
│   │           │        │         │                 │       │           │  │
│   │      5    │   ◆    │ CHRONIK │ "Was ist wahr?" │  NOA  │ Beweis    │  │
│   │           │        │         │                 │       │           │  │
│   │      6    │   ◇    │ NEXUS   │ "Wie verbunden?"│  NOA  │ Vernetzung│  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   Kausalitätsprinzip:                                                       │
│   ◉ ANKER → ◈ SCHEMA → ◊ METRIK → ▣ SPHÄRE → ◐ IMPULS → ◆ CHRONIK → ◇ NEXUS│
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Systemarchitektur

### 4.1 Schicht 0: ◉ ANKER – Identität

**Kernfrage:** _„Wer existiert?"_

**Fachlicher Zweck:**
Die ANKER-Schicht etabliert die fundamentale Identitätsgrundlage des Systems. Jede Entität – ob Maschine, Organisation, Objekt oder Regelwerk – erhält eine kryptografisch verifizierbare, dezentrale Identität (DID).

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   IDENTITY-FIRST PARADIGMA                                                 │
│                                                                             │
│   AXIOM: "Existenz durch Identifizierbarkeit"                              │
│                                                                             │
│   1. Eine Entität EXISTIERT, weil sie eine Identität HAT                   │
│   2. Ohne Identität ist keine Interaktion möglich                          │
│   3. Identität ist nicht optional – sie ist konstitutiv                    │
│   4. Alle Beziehungen sind Identitäts-Beziehungen                          │
│                                                                             │
│   ENTITÄTS-TYPEN MIT DID                                                   │
│   ════════════════════════                                                  │
│                                                                             │
│   👤 Agent (Seeker)           →  did:erynoa:agent:seeker:...               │
│   👤 Agent (Provider)         →  did:erynoa:agent:provider:...             │
│   🏢 Organisation             →  did:erynoa:org:...                        │
│   🚗 Fahrzeug                 →  did:erynoa:vehicle:...                    │
│   📦 AMO (Objekt-Instanz)     →  did:erynoa:amo:...                        │
│   📋 Blueprint                →  did:erynoa:blueprint:...                  │
│   🌍 Environment              →  did:erynoa:env:...                        │
│   📜 Standard/Norm            →  did:erynoa:standard:...                   │
│   🎫 Credential               →  did:erynoa:vc:...                         │
│   💳 Wallet                   →  did:erynoa:wallet:...                     │
│   🗳️ Governance Proposal      →  did:erynoa:proposal:...                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Kernkomponenten:**

| Komponente           | Beschreibung                            |
| -------------------- | --------------------------------------- |
| **DID-Resolver**     | Auflösung von DIDs zu DID-Dokumenten    |
| **Credential-Store** | Verwaltung von Verifiable Credentials   |
| **DACS-Modul**       | Multi-Chain-Verankerung von Identitäten |
| **Key-Management**   | Ed25519-Schlüsselverwaltung             |

**Fachliche Anforderungen:**

| ID       | Anforderung                                             | Priorität |
| -------- | ------------------------------------------------------- | --------- |
| FA-A-001 | Jede Entität MUSS eine eindeutige DID erhalten          | MUSS      |
| FA-A-002 | DIDs MÜSSEN kryptografisch verifizierbar sein           | MUSS      |
| FA-A-003 | DIDs MÜSSEN auf mehreren Chains verankert werden        | SOLL      |
| FA-A-004 | Sub-Identities MÜSSEN hierarchisch ableitbar sein       | SOLL      |
| FA-A-005 | Credentials MÜSSEN nach W3C-VC-Standard formatiert sein | MUSS      |

---

### 4.2 Schicht 1: ◈ SCHEMA – Wissen

**Kernfrage:** _„Was ist das?"_

**Fachlicher Zweck:**
Die SCHEMA-Schicht definiert die semantische Grundlage des Systems. Sie legt fest, welche Objekttypen existieren, welche Attribute sie haben und wie sie sich zueinander verhalten.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   WISSENSPYRAMIDE                                                          │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   Normative Standards (Ebene 1)                                     │  │
│   │   ─────────────────────────────                                     │  │
│   │   ISO 19112 · eCl@ss · OCPP · DIN · ETSI                           │  │
│   │   (Externe, akkreditierte Normen)                                   │  │
│   │        │                                                            │  │
│   │        │ referenziert von                                           │  │
│   │        ▼                                                            │  │
│   │   Generische Blueprints (Ebene 2a)                                  │  │
│   │   ────────────────────────────────                                  │  │
│   │   ev-charging-station:v1 (abstrakt)                                │  │
│   │   (Plattformweite Schablonen)                                       │  │
│   │        │                                                            │  │
│   │        │ spezialisiert zu                                           │  │
│   │        ▼                                                            │  │
│   │   Domain Blueprints (Ebene 2b)                                      │  │
│   │   ────────────────────────────                                      │  │
│   │   ev-charging-station-de:v1                                        │  │
│   │   + Eichrecht-Anforderungen                                        │  │
│   │   + PTB-Zertifizierung                                             │  │
│   │   (Markt-/regulierungsspezifisch)                                   │  │
│   │        │                                                            │  │
│   │        │ instanziiert zu                                            │  │
│   │        ▼                                                            │  │
│   │   AMO-Instanzen (Ebene 3)                                           │  │
│   │   ───────────────────────                                           │  │
│   │   station-munich-001                                               │  │
│   │   (Konkrete Objekte)                                                │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Kernkomponenten:**

| Komponente             | Beschreibung                              |
| ---------------------- | ----------------------------------------- |
| **Blueprint-Engine**   | Verwaltung und Validierung von Blueprints |
| **Semantic Index**     | Vektorbasierte semantische Suche (Qdrant) |
| **Standards-Registry** | Mapping zu externen Normen (ISO, OCPP)    |
| **Version Manager**    | Versionierung ohne Breaking Changes       |

**Fachliche Anforderungen:**

| ID       | Anforderung                                             | Priorität |
| -------- | ------------------------------------------------------- | --------- |
| FA-S-001 | Blueprints MÜSSEN auf normative Standards referenzieren | SOLL      |
| FA-S-002 | Blueprints MÜSSEN versioniert und immutabel sein        | MUSS      |
| FA-S-003 | Attribute MÜSSEN typisiert und validierbar sein         | MUSS      |
| FA-S-004 | Semantic Search MUSS Blueprints nach Bedeutung finden   | SOLL      |
| FA-S-005 | Blueprint-Evolution MUSS Migration unterstützen         | SOLL      |

---

### 4.3 Schicht 2: ◊ METRIK – Vertrauen

**Kernfrage:** _„Wie vertrauenswürdig?"_

**Fachlicher Zweck:**
Die METRIK-Schicht quantifiziert Vertrauen maschinenlesbar. Sie ermöglicht automatisiertes Trust-Gating – die Filterung von Interaktionspartnern basierend auf mathematisch berechneten Vertrauenswerten.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   TRUST VECTOR – 4 DIMENSIONEN                                             │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   DIMENSION       │ SYMBOL │ BESCHREIBUNG                          │  │
│   │   ════════════════════════════════════════════════════════════════  │  │
│   │                                                                     │  │
│   │   Reliability     │   🎯   │ Liefert pünktlich, wie vereinbart     │  │
│   │                   │        │ (Uptime, Termintreue)                  │  │
│   │                                                                     │  │
│   │   Integrity       │   🛡️   │ Macht keine Falschangaben             │  │
│   │                   │        │ (Ehrlichkeit, Datenqualität)           │  │
│   │                                                                     │  │
│   │   Capability      │   ⚡   │ Technisch in der Lage                  │  │
│   │                   │        │ (Leistungsfähigkeit, Kapazität)        │  │
│   │                                                                     │  │
│   │   Reputation      │   🌟   │ Von anderen positiv bewertet          │  │
│   │                   │        │ (Attestations, Endorsements)           │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   BEISPIEL TRUST VECTOR                                                    │
│   ═════════════════════                                                     │
│                                                                             │
│   did:erynoa:agent:provider:swm-charging                                   │
│   Trust: [Reliability: 0.92, Integrity: 0.87, Capability: 0.78, Rep: 0.95]│
│   Aggregate: 0.88                                                          │
│                                                                             │
│   TRUST-GATING                                                             │
│   ════════════                                                              │
│                                                                             │
│   Intent verlangt: min_trust = 0.7, min_reliability = 0.8                  │
│                                                                             │
│   Provider A: [0.92, 0.87, 0.78, 0.95] → ✅ PASS                           │
│   Provider B: [0.65, 0.90, 0.70, 0.80] → ❌ FAIL (Reliability < 0.8)       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Kernkomponenten:**

| Komponente              | Beschreibung                                    |
| ----------------------- | ----------------------------------------------- |
| **Trust Engine**        | Berechnung und Aktualisierung von Trust Vectors |
| **Karma Engine**        | Tier-System und asymmetrische Updates           |
| **Attestation Service** | Externe Trust-Bestätigungen                     |
| **Decay Service**       | Zeitbasierter Verfall alter Events              |

**Karma-Tiers:**

| Tier            | Karma-Punkte | Privilegien                          |
| --------------- | ------------ | ------------------------------------ |
| **Newcomer**    | 0 - 99       | Basis-Funktionen, eingeschränkte API |
| **Established** | 100 - 499    | Erweiterte Funktionen, höhere Limits |
| **Veteran**     | 500 - 999    | Volle Funktionen, Governance-Rechte  |
| **Elder**       | 1000+        | Validator-Berechtigung, DAO-Council  |

**Fachliche Anforderungen:**

| ID       | Anforderung                                                    | Priorität |
| -------- | -------------------------------------------------------------- | --------- |
| FA-M-001 | Trust Vectors MÜSSEN aus verifizierten Events berechnet werden | MUSS      |
| FA-M-002 | Trust MUSS Environment-spezifisch sein                         | SOLL      |
| FA-M-003 | Negatives Verhalten MUSS asymmetrisch stärker wirken           | SOLL      |
| FA-M-004 | Trust-Updates MÜSSEN in Echtzeit erfolgen                      | SOLL      |
| FA-M-005 | Attestations MÜSSEN signiert und verifizierbar sein            | MUSS      |

---

### 4.4 Schicht 3: ▣ SPHÄRE – Räume

**Kernfrage:** _„Wo gilt was?"_

**Fachlicher Zweck:**
Die SPHÄRE-Schicht definiert abgegrenzte Kontextblasen (Environments), in denen spezifische Regeln, Standards und Governance gelten. Sie ist das "Spielfeld" für Agenten-Interaktionen.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ENVIRONMENT-HIERARCHIE                                                   │
│                                                                             │
│   env:global (Erynoa Protocol)                                             │
│   ║                                                                        │
│   ╠══ env:geo:europe                                                       │
│   ║   ║                                                                    │
│   ║   ╠══ env:domain:ev-charging-eu                                       │
│   ║   ║   ║                                                                │
│   ║   ║   ╠══ env:domain:ev-charging-de                                   │
│   ║   ║   ║   └── + Eichrecht                                             │
│   ║   ║   ║   └── + PTB-Zertifizierung                                    │
│   ║   ║   ║                                                                │
│   ║   ║   ╠══ env:domain:ev-charging-fr                                   │
│   ║   ║   ║   └── + AFIR-FR                                               │
│   ║   ║   ║                                                                │
│   ║   ║   └══ env:domain:ev-charging-nl                                   │
│   ║   ║                                                                    │
│   ║   └══ env:domain:energy-trading-eu                                    │
│   ║       └── + MiFID II, REMIT                                           │
│   ║                                                                        │
│   ╠══ env:geo:north-america                                               │
│   ║   └══ ...                                                              │
│   ║                                                                        │
│   └══ env:private:stadtwerke-munich                                       │
│       └══ env:private:swm-internal-fleet                                  │
│                                                                             │
│   Regeln vererben sich nach unten, können aber überschrieben werden.       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Environment-Typen:**

| Typ         | Scope             | Beispiel                    | Governance      |
| ----------- | ----------------- | --------------------------- | --------------- |
| **Domain**  | Branche/Industrie | `env:domain:ev-charging-de` | Branchenverband |
| **Geo**     | Geographisch      | `env:geo:europe`            | Regulatoren     |
| **Private** | Unternehmen       | `env:private:swm`           | Eigentümer      |
| **Testnet** | Entwicklung       | `env:test:dev-staging`      | Entwickler      |

**Fachliche Anforderungen:**

| ID       | Anforderung                                            | Priorität |
| -------- | ------------------------------------------------------ | --------- |
| FA-P-001 | Environments MÜSSEN hierarchisch verschachtelbar sein  | MUSS      |
| FA-P-002 | Regeln MÜSSEN sich von Parent-Environments vererben    | SOLL      |
| FA-P-003 | Membership MUSS durch Credentials gesteuert werden     | SOLL      |
| FA-P-004 | Cross-Environment-Operationen MÜSSEN möglich sein      | SOLL      |
| FA-P-005 | Governance MUSS durch DAO-ähnliche Strukturen erfolgen | KANN      |

---

### 4.5 Schicht 4: ◐ IMPULS – Handlung

**Kernfrage:** _„Was geschieht?"_

**Fachlicher Zweck:**
Die IMPULS-Schicht ermöglicht autonome Handlungen. Agenten definieren Intents (Absichten), Policies (Entscheidungsregeln) und führen Verhandlungen durch. Die ECLVM (Erynoa Configuration Language VM) ist die Runtime für deterministische Policy-Evaluation.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AGENT-MODELL                                                             │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │      🤖                                                             │  │
│   │      AGENT                                                          │  │
│   │      did:erynoa:agent:*                                            │  │
│   │                                                                     │  │
│   │      ┌────────────────────────────────────────────────────────┐    │  │
│   │      │                                                        │    │  │
│   │      │  • Eigene DID (Identität)                              │    │  │
│   │      │  • Eigener Trust Vector (Reputation)                   │    │  │
│   │      │  • Eigene Credentials (Berechtigungen)                 │    │  │
│   │      │  • Eigener Wallet (Vermögen)                           │    │  │
│   │      │  • Eigene Policies (Entscheidungsregeln)               │    │  │
│   │      │  • Eigene Intents (Ziele)                              │    │  │
│   │      │                                                        │    │  │
│   │      └────────────────────────────────────────────────────────┘    │  │
│   │                                                                     │  │
│   │   Agent handelt autonom innerhalb seiner Policy-Grenzen.           │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│   AGENT-TYPEN                                                              │
│   ═══════════                                                               │
│                                                                             │
│   Seeker     │ Sucht Ressourcen/Dienste    │ Fahrzeug sucht Ladestation   │
│   Provider   │ Bietet Ressourcen/Dienste   │ Ladesäulen-Betreiber         │
│   Broker     │ Vermittelt zwischen Parteien│ Roaming-Plattform            │
│   Oracle     │ Liefert externe Daten       │ Wetter-Service, Preisfeed    │
│   Validator  │ Prüft und bestätigt         │ Eichamt, Zertifizierer       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Der Cybernetic Loop:**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   CYBERNETIC LOOP – DAS HERZSTÜCK VON ERYNOA                               │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   1. PERCEPTION (ERY)                                               │  │
│   │      Agent nimmt Umgebung wahr                                      │  │
│   │      → Discovery, Trust-Check, Constraints                          │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   2. DECISION (ECHO)                                                │  │
│   │      Agent entscheidet                                              │  │
│   │      → Policy-Evaluation, Offer-Ranking, Selection                  │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   3. ACTION (ECHO → NOA)                                            │  │
│   │      Agent handelt                                                  │  │
│   │      → Transaction, State Change, Commitment                        │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   4. OBSERVATION (NOA)                                              │  │
│   │      System beobachtet                                              │  │
│   │      → Success/Failure, Metrics, Evidence                           │  │
│   │           │                                                         │  │
│   │           ▼                                                         │  │
│   │   5. FEEDBACK (NOA → ERY)                                           │  │
│   │      System lernt                                                   │  │
│   │      → Trust Update, Karma Update, Knowledge Update                 │  │
│   │           │                                                         │  │
│   │           └──────────────────▶ zurück zu 1. PERCEPTION              │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Fachliche Anforderungen:**

| ID       | Anforderung                                                   | Priorität |
| -------- | ------------------------------------------------------------- | --------- |
| FA-I-001 | Intents MÜSSEN Constraints und Budget enthalten               | MUSS      |
| FA-I-002 | Policies MÜSSEN auto_accept/auto_reject/escalate definieren   | SOLL      |
| FA-I-003 | Negotiation MUSS Direct, Auction und Multi-Round unterstützen | SOLL      |
| FA-I-004 | ECLVM MUSS deterministisch und sandboxed sein                 | MUSS      |
| FA-I-005 | Agent-Lifecycle MUSS vollständig definiert sein               | MUSS      |

---

### 4.6 Schicht 5: ◆ CHRONIK – Beweis

**Kernfrage:** _„Was ist wahr?"_

**Fachlicher Zweck:**
Die CHRONIK-Schicht speichert nicht nur _was_ passiert ist, sondern _warum_ und _in welcher Reihenfolge_. Der NOA Ledger ist ein kausales Beweissystem mit DAG-Struktur.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   AMO (ATOMIC MANAGED OBJECT)                                              │
│                                                                             │
│   AMO = Universelle Repräsentation von Assets, Services, Credentials       │
│                                                                             │
│   AMO-TYPEN                                                                │
│   ═════════                                                                 │
│                                                                             │
│   Material   │ amo:material:*   │ Ladesäule, Fahrzeug, Sensor              │
│   Service    │ amo:service:*    │ Ladevorgang, Wartung, Transport          │
│   Credential │ amo:credential:* │ Zertifikat, Lizenz, KYC                  │
│   Data       │ amo:data:*       │ Messwert, Report, Log                    │
│   Contract   │ amo:contract:*   │ Vertrag, SLA, Agreement                  │
│                                                                             │
│   AMO LIFECYCLE                                                            │
│   ═════════════                                                             │
│                                                                             │
│   PENDING → ACTIVE → SUSPENDED → ACTIVE → DECOMMISSIONED                   │
│                                                                             │
│   • activate()      → PENDING → ACTIVE (requires credentials)              │
│   • suspend()       → ACTIVE → SUSPENDED                                   │
│   • resume()        → SUSPENDED → ACTIVE                                   │
│   • decommission()  → * → DECOMMISSIONED (permanent)                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Finality-Levels:**

| Level       | Status      | Beschreibung                                |
| ----------- | ----------- | ------------------------------------------- |
| **Level 0** | PENDING     | Erstellt, noch nicht verteilt               |
| **Level 1** | DISTRIBUTED | An Netzwerk verteilt, wird validiert        |
| **Level 2** | ANCHORED    | Auf IOTA/Ethereum verankert                 |
| **Level 3** | FINAL       | Genug Confirmations, praktisch irreversibel |

**Fachliche Anforderungen:**

| ID       | Anforderung                                                      | Priorität |
| -------- | ---------------------------------------------------------------- | --------- |
| FA-C-001 | Events MÜSSEN kausale Referenzen (causes) enthalten              | MUSS      |
| FA-C-002 | AMOs MÜSSEN gegen ihr Blueprint validiert werden                 | MUSS      |
| FA-C-003 | Logic Guards MÜSSEN in ECLVM ausführbar sein                     | SOLL      |
| FA-C-004 | Streaming Payments MÜSSEN während laufender Dienste möglich sein | SOLL      |
| FA-C-005 | Finality MUSS durch Multi-Chain-Anchoring erreicht werden        | SOLL      |

---

### 4.7 Schicht 6: ◇ NEXUS – Vernetzung

**Kernfrage:** _„Wie verbunden?"_

**Fachlicher Zweck:**
Die NEXUS-Schicht verbindet Erynoa mit externen Systemen und Blockchains. Sie implementiert Multi-Chain-Anchoring für Redundanz und Interoperabilität.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   MULTI-CHAIN STRATEGIE                                                    │
│                                                                             │
│   PRIMARY CHAIN: IOTA                                                      │
│   ═══════════════════════                                                   │
│   • Feeless Transactions (ideal für Mikrotransaktionen)                    │
│   • DAG-basiert (Tangle)                                                   │
│   • Schnelle Finality (~10s)                                               │
│   • MoveVM-Integration (IOTA Rebased)                                      │
│                                                                             │
│   SECONDARY CHAINS (optional, für spezifische Use Cases)                   │
│   ════════════════════════════════════════════════════════                  │
│                                                                             │
│   Ethereum   │ High-Value Anchoring, DeFi-Bridge │ $1-50 pro Tx           │
│   Solana     │ High-Frequency Trading            │ $0.01 pro Tx           │
│   Polygon    │ Volume Scaling                    │ $0.001 pro Tx          │
│                                                                             │
│   CHAIN-SELECTION LOGIC                                                    │
│   ═════════════════════                                                     │
│                                                                             │
│   if transaction.value > 10000 EUR  → IOTA + Ethereum                      │
│   if transaction.type == streaming  → IOTA only (feeless)                  │
│   if transaction.defi_enabled       → IOTA + Ethereum                      │
│   default                           → IOTA only                            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Fachliche Anforderungen:**

| ID       | Anforderung                                         | Priorität |
| -------- | --------------------------------------------------- | --------- |
| FA-N-001 | IOTA MUSS als Primary Chain verwendet werden        | MUSS      |
| FA-N-002 | High-Value-Transaktionen SOLLEN multi-anchored sein | SOLL      |
| FA-N-003 | Bridges zu externen Systemen MÜSSEN definiert sein  | SOLL      |
| FA-N-004 | P2P-Kommunikation MUSS über libp2p erfolgen         | SOLL      |
| FA-N-005 | Cross-Chain-Verification MUSS möglich sein          | SOLL      |

---

## 5. Fachliche Domänen

### 5.1 Domain: E-Mobilität

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   E-MOBILITÄT – FACHLICHE OBJEKTE                                          │
│                                                                             │
│   ENTITÄTEN                                                                │
│   ══════════                                                                │
│   • Elektrofahrzeug (did:erynoa:vehicle:vin-*)                             │
│   • Ladesäule (did:erynoa:amo:material:charger-*)                          │
│   • Ladepunkt-Betreiber (did:erynoa:org:cpo-*)                             │
│   • E-Mobility-Provider (did:erynoa:org:emp-*)                             │
│   • Roaming-Hub (did:erynoa:org:hub-*)                                     │
│                                                                             │
│   BLUEPRINTS                                                               │
│   ══════════                                                                │
│   • ev-charging-station (ISO 15118, OCPP 2.0.1)                            │
│   • ev-charging-station-de (+Eichrecht, +PTB)                              │
│   • charging-session (Ladevorgang)                                         │
│   • roaming-contract (Roaming-Vertrag)                                     │
│                                                                             │
│   ENVIRONMENTS                                                             │
│   ════════════                                                              │
│   • env:domain:ev-charging-eu (AFIR, EU-Regulierung)                       │
│   • env:domain:ev-charging-de (Eichrecht, LSV)                             │
│   • env:network:hubject (Intercharge)                                      │
│   • env:network:gireve (französisches Roaming)                             │
│                                                                             │
│   STANDARDS                                                                │
│   ═════════                                                                 │
│   • ISO 15118 (Vehicle-to-Grid-Kommunikation)                              │
│   • OCPP 2.0.1 (Open Charge Point Protocol)                                │
│   • OCPI 2.2.1 (Open Charge Point Interface)                               │
│   • Eichrecht (Mess- und Eichverordnung)                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 Domain: Energiehandel

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ENERGIEHANDEL – FACHLICHE OBJEKTE                                        │
│                                                                             │
│   ENTITÄTEN                                                                │
│   ══════════                                                                │
│   • Prosumer (did:erynoa:agent:provider:prosumer-*)                        │
│   • Consumer (did:erynoa:agent:seeker:consumer-*)                          │
│   • Grid Operator (did:erynoa:agent:validator:grid-*)                      │
│   • Smart Meter (did:erynoa:amo:material:meter-*)                          │
│   • Battery Storage (did:erynoa:amo:material:battery-*)                    │
│                                                                             │
│   BLUEPRINTS                                                               │
│   ══════════                                                                │
│   • energy-certificate (Herkunftsnachweis)                                 │
│   • power-purchase-agreement (PPA)                                         │
│   • grid-feed-in (Einspeisung)                                             │
│   • demand-response (Lastmanagement)                                       │
│                                                                             │
│   ENVIRONMENTS                                                             │
│   ════════════                                                              │
│   • env:domain:energy-trading-eu (MiFID II, REMIT)                         │
│   • env:domain:p2p-energy-de (EEG, EnWG)                                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 5.3 Domain: Fleet Management

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   FLEET MANAGEMENT – FACHLICHE OBJEKTE                                     │
│                                                                             │
│   ENTITÄTEN                                                                │
│   ══════════                                                                │
│   • Fleet Owner (did:erynoa:org:fleet-*)                                   │
│   • Fleet Manager Agent (did:erynoa:agent:broker:fleet-mgr-*)              │
│   • Vehicle Agent (did:erynoa:agent:seeker:vehicle-*)                      │
│   • Maintenance Provider (did:erynoa:agent:provider:maintenance-*)         │
│                                                                             │
│   HIERARCHIE                                                               │
│   ══════════                                                                │
│   Fleet Owner                                                              │
│   └── Fleet Manager Agent (koordiniert)                                    │
│       ├── Vehicle Agent 001 → AMO: EV-001                                  │
│       ├── Vehicle Agent 002 → AMO: EV-002                                  │
│       └── ... (N Fahrzeuge)                                                │
│                                                                             │
│   FEATURES                                                                 │
│   ════════                                                                  │
│   • Zentrale Fleet-Policy für alle Fahrzeuge                               │
│   • Aggregiertes Budget-Management                                         │
│   • Trust-Aggregation über Fleet-Durchschnitt                              │
│   • Zentrales Reporting aller Events                                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 6. Geschäftsobjekte

### 6.1 Objekt-Übersicht

| Objekt           | Typ         | Beschreibung                     | Schicht   |
| ---------------- | ----------- | -------------------------------- | --------- |
| **DID**          | Identifier  | Dezentraler Identifier           | ◉ ANKER   |
| **Credential**   | Nachweis    | Verifiable Credential            | ◉ ANKER   |
| **Blueprint**    | Schablone   | Objektdefinition                 | ◈ SCHEMA  |
| **Standard**     | Norm        | Externe Normreferenz             | ◈ SCHEMA  |
| **Trust Vector** | Metrik      | Mehrdimensionaler Vertrauenswert | ◊ METRIK  |
| **Attestation**  | Bestätigung | Externe Trust-Bestätigung        | ◊ METRIK  |
| **Environment**  | Kontext     | Abgegrenzter Regelraum           | ▣ SPHÄRE  |
| **Agent**        | Akteur      | Handelnde Einheit                | ◐ IMPULS  |
| **Intent**       | Absicht     | Formalisiertes Ziel              | ◐ IMPULS  |
| **Policy**       | Regel       | Entscheidungslogik               | ◐ IMPULS  |
| **AMO**          | Objekt      | Atomic Managed Object            | ◆ CHRONIK |
| **Event**        | Ereignis    | Kausales Event auf NOA           | ◆ CHRONIK |
| **Anchor**       | Verankerung | Chain-Proof                      | ◇ NEXUS   |

### 6.2 Objekt-Beziehungen

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   OBJEKT-BEZIEHUNGEN                                                       │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │        DID ─────────┐                                               │  │
│   │         │           │                                               │  │
│   │    identifies    has                                                │  │
│   │         │           │                                               │  │
│   │         ▼           ▼                                               │  │
│   │      Agent ───── Credential                                         │  │
│   │         │           │                                               │  │
│   │    has │           │ verifies                                       │  │
│   │         │           │                                               │  │
│   │         ▼           ▼                                               │  │
│   │      Policy     Trust Vector ──────┐                                │  │
│   │         │           │              │                                │  │
│   │    defines     affects        context-of                            │  │
│   │         │           │              │                                │  │
│   │         ▼           ▼              ▼                                │  │
│   │      Intent ───▶ AMO ◀──────── Environment                         │  │
│   │         │           │              │                                │  │
│   │    creates     based-on      governed-by                            │  │
│   │         │           │              │                                │  │
│   │         ▼           ▼              ▼                                │  │
│   │      Event     Blueprint      Governance                            │  │
│   │         │           │                                               │  │
│   │    anchored  references                                             │  │
│   │         │           │                                               │  │
│   │         ▼           ▼                                               │  │
│   │      Anchor     Standard                                            │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Prozesse und Abläufe

### 7.1 Prozess: Agent-Registrierung

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   PROZESS: AGENT-REGISTRIERUNG                                             │
│                                                                             │
│   1. DID-ERSTELLUNG                                                        │
│      Owner generiert Schlüsselpaar (Ed25519)                               │
│      System erstellt DID: did:erynoa:agent:seeker:<id>                     │
│                                                                             │
│   2. DID-DOKUMENT                                                          │
│      Erstellung des DID-Dokuments mit:                                     │
│      • Public Key                                                          │
│      • Authentication Methods                                              │
│      • Service Endpoints                                                   │
│                                                                             │
│   3. INITIAL TRUST                                                         │
│      Agent erhält Initial Trust = Owner Trust × 0.5                        │
│      Karma-Tier = Newcomer                                                 │
│                                                                             │
│   4. ANCHORING                                                             │
│      DID wird auf IOTA verankert                                           │
│      (Optional: Ethereum für High-Value-Agents)                            │
│                                                                             │
│   5. ENVIRONMENT-REGISTRIERUNG                                             │
│      Agent tritt Environments bei                                          │
│      Membership-Credentials werden geprüft                                 │
│                                                                             │
│   6. CONFIGURATION                                                         │
│      Policy zuweisen                                                       │
│      Wallet konfigurieren                                                  │
│      Credentials delegieren                                                │
│                                                                             │
│   7. ACTIVATION                                                            │
│      Agent ist operativ                                                    │
│      Kann Intents erstellen und verhandeln                                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.2 Prozess: Transaktion (EV-Charging)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   PROZESS: EV-LADEVORGANG (VOLLSTÄNDIG)                                    │
│                                                                             │
│   ┌─────────────────────────────────────────────────────────────────────┐  │
│   │                                                                     │  │
│   │   PHASE 1: PERCEPTION (ERY)                                         │  │
│   │   ════════════════════════════                                      │  │
│   │   • Fahrzeug-Agent erkennt: Batterie bei 20%                       │  │
│   │   • Agent startet Discovery im Environment                          │  │
│   │   • Query: 50kW+, CCS, 5km Radius, Trust > 0.7                     │  │
│   │   • Ergebnis: 5 Stationen gefunden                                  │  │
│   │                                                                     │  │
│   │   PHASE 2: DECISION (ECHO)                                          │  │
│   │   ═══════════════════════════                                       │  │
│   │   • Agent erstellt Intent:                                          │  │
│   │     - type: charge_vehicle                                          │  │
│   │     - constraints: power_min: 50kW, connector: CCS                  │  │
│   │     - budget: max 30€                                               │  │
│   │   • Policy evaluiert Angebote:                                      │  │
│   │     - Station A: 0.42€/kWh, Trust 0.92, 1.2km → Score: 0.94        │  │
│   │     - Station B: 0.38€/kWh, Trust 0.71, 3.5km → Score: 0.81        │  │
│   │   • Auto-Accept: Station A (unter 0.50€, Trust > 0.7)              │  │
│   │                                                                     │  │
│   │   PHASE 3: ACTION (ECHO → NOA)                                      │  │
│   │   ══════════════════════════════                                    │  │
│   │   • Agreement wird erstellt                                         │  │
│   │   • Beide Parteien signieren                                        │  │
│   │   • Ladevorgang startet                                             │  │
│   │   • Streaming Payment beginnt: 0.42€ pro kWh                       │  │
│   │                                                                     │  │
│   │   PHASE 4: OBSERVATION (NOA)                                        │  │
│   │   ═════════════════════════════                                     │  │
│   │   • Ladevorgang abgeschlossen                                       │  │
│   │   • Messwerte: 45 kWh in 28 Minuten                                │  │
│   │   • Zahlung: 18.90€                                                │  │
│   │   • Event wird finalisiert (Level 3: FINAL)                        │  │
│   │                                                                     │  │
│   │   PHASE 5: FEEDBACK (NOA → ERY)                                     │  │
│   │   ════════════════════════════════                                  │  │
│   │   • Station A: Reliability +0.02                                    │  │
│   │   • Fahrzeug: Integrity +0.02 (korrekte Zahlung)                   │  │
│   │   • Beide: Karma-Punkte akkumuliert                                │  │
│   │   • Feedback fließt in nächste Perception                          │  │
│   │                                                                     │  │
│   └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 7.3 Prozess: Trust-Update

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   PROZESS: TRUST-UPDATE                                                    │
│                                                                             │
│   TRIGGER                                                                  │
│   ═══════                                                                   │
│   Transaktion wird auf NOA finalisiert                                     │
│                                                                             │
│   BERECHNUNG                                                               │
│   ══════════                                                                │
│                                                                             │
│   trust[dim] = Σ(event_weight × event_value × decay(age)) / normalization  │
│                                                                             │
│   Faktoren:                                                                │
│   • event_weight: Gewicht des Event-Typs (0.0 - 1.0)                       │
│   • event_value: Positiv (+) oder Negativ (-)                              │
│   • decay(age): Zeitlicher Verfall (ältere Events zählen weniger)          │
│   • normalization: Normierung auf [0, 1]                                   │
│                                                                             │
│   ASYMMETRIE                                                               │
│   ══════════                                                                │
│   Positiv:  +0.02 pro erfolgreichem Event                                  │
│   Negativ:  -0.10 pro fehlgeschlagenem Event (5× stärker)                  │
│                                                                             │
│   PERSISTIERUNG                                                            │
│   ══════════════                                                            │
│   • Trust Vector wird aktualisiert                                         │
│   • Karma-Punkte werden akkumuliert                                        │
│   • Bei Tier-Wechsel: Benachrichtigung                                     │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 8. Anwendungsfälle

### 8.1 Use Case: Autonomes EV-Charging

| Attribut          | Wert                                                  |
| ----------------- | ----------------------------------------------------- |
| **ID**            | UC-001                                                |
| **Name**          | Autonomes EV-Charging                                 |
| **Akteure**       | Fahrzeug-Agent, Ladesäulen-Agent, Betreiber           |
| **Vorbedingung**  | Fahrzeug hat Agent mit Wallet und Credentials         |
| **Trigger**       | Batterie unter Schwellwert                            |
| **Hauptszenario** | Siehe Prozess 7.2                                     |
| **Nachbedingung** | Fahrzeug geladen, Zahlung erfolgt, Trust aktualisiert |

### 8.2 Use Case: P2P Energy Trading

| Attribut          | Wert                                                                                                                                   |
| ----------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| **ID**            | UC-002                                                                                                                                 |
| **Name**          | Peer-to-Peer Energiehandel                                                                                                             |
| **Akteure**       | Prosumer-Agent, Consumer-Agent, Grid Validator                                                                                         |
| **Vorbedingung**  | Beide haben Agenten im env:domain:energy-trading                                                                                       |
| **Trigger**       | Prosumer hat Überschuss, Consumer hat Bedarf                                                                                           |
| **Hauptszenario** | 1. Prosumer erstellt Angebot<br>2. Consumer akzeptiert<br>3. Physische Lieferung<br>4. Smart Meter verifiziert<br>5. Streaming Payment |
| **Nachbedingung** | Energie übertragen, Zahlung erfolgt, HKN ausgestellt                                                                                   |

### 8.3 Use Case: Fleet Management

| Attribut          | Wert                                                                                                                         |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| **ID**            | UC-003                                                                                                                       |
| **Name**          | Autonomes Fleet Management                                                                                                   |
| **Akteure**       | Fleet Manager, Vehicle Agents, Maintenance Provider                                                                          |
| **Vorbedingung**  | Fleet Owner hat Fleet Manager Agent deployed                                                                                 |
| **Trigger**       | Fahrzeug benötigt Ladung/Wartung                                                                                             |
| **Hauptszenario** | 1. Vehicle Agent meldet Bedarf<br>2. Fleet Manager koordiniert<br>3. Budget wird allokiert<br>4. Transaktion erfolgt autonom |
| **Nachbedingung** | Service erfolgt, Budget aktualisiert, Reporting                                                                              |

### 8.4 Use Case: KYC Credential Sharing

| Attribut          | Wert                                                                                                                                        |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| **ID**            | UC-004                                                                                                                                      |
| **Name**          | Portabler KYC-Nachweis                                                                                                                      |
| **Akteure**       | Nutzer, Bank (Issuer), Service (Verifier)                                                                                                   |
| **Vorbedingung**  | Bank hat KYC durchgeführt                                                                                                                   |
| **Trigger**       | Nutzer will Service nutzen, der KYC erfordert                                                                                               |
| **Hauptszenario** | 1. Bank issued Credential<br>2. Nutzer präsentiert Credential<br>3. Service verifiziert Signatur<br>4. Selective Disclosure (nur "über 18") |
| **Nachbedingung** | Service verifiziert ohne vollständige Daten                                                                                                 |

---

## 9. Nicht-funktionale Anforderungen

### 9.1 Performance

| ID        | Anforderung                | Zielwert |
| --------- | -------------------------- | -------- |
| NFA-P-001 | Discovery-Latenz           | < 500ms  |
| NFA-P-002 | Intent-to-Agreement-Zeit   | < 2s     |
| NFA-P-003 | Trust-Update-Latenz        | < 100ms  |
| NFA-P-004 | Event-Finality auf IOTA    | < 15s    |
| NFA-P-005 | Streaming-Payment-Interval | ≥ 1/s    |

### 9.2 Skalierbarkeit

| ID        | Anforderung               | Zielwert     |
| --------- | ------------------------- | ------------ |
| NFA-S-001 | Gleichzeitige Agenten     | > 1 Million  |
| NFA-S-002 | Transaktionen pro Sekunde | > 10.000 TPS |
| NFA-S-003 | Events pro Tag            | > 100 Mio.   |
| NFA-S-004 | Environments pro Netzwerk | > 10.000     |

### 9.3 Sicherheit

| ID        | Anforderung                           | Priorität |
| --------- | ------------------------------------- | --------- |
| NFA-X-001 | Alle Signaturen Ed25519               | MUSS      |
| NFA-X-002 | Transport über TLS 1.3                | MUSS      |
| NFA-X-003 | ECLVM vollständig sandboxed           | MUSS      |
| NFA-X-004 | Key Rotation unterstützt              | SOLL      |
| NFA-X-005 | Zero-Knowledge-Proofs für Credentials | KANN      |

### 9.4 Verfügbarkeit

| ID        | Anforderung                          | Zielwert |
| --------- | ------------------------------------ | -------- |
| NFA-V-001 | System-Uptime                        | 99.9%    |
| NFA-V-002 | Graceful Degradation bei Partitionen | MUSS     |
| NFA-V-003 | Multi-Region-Deployment              | SOLL     |

---

## 10. Governance und Compliance

### 10.1 Governance-Struktur

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ERYNOA GOVERNANCE                                                        │
│                                                                             │
│   GLOBAL LEVEL                                                             │
│   ════════════                                                              │
│   Erynoa Foundation                                                        │
│   ├── Technical Council (Protokoll-Entwicklung)                            │
│   ├── Standards Committee (EIP-Prozess)                                    │
│   └── Community DAO (Abstimmungen)                                         │
│                                                                             │
│   ENVIRONMENT LEVEL                                                        │
│   ═════════════════                                                         │
│   Environment-spezifische Governance                                       │
│   ├── Legislative (Regelwerk definieren)                                   │
│   ├── Executive (Regeln durchsetzen)                                       │
│   └── Judicial (Streitfälle entscheiden)                                   │
│                                                                             │
│   PROPOSAL-PROZESS                                                         │
│   ════════════════                                                          │
│   1. Draft → 2. Review → 3. Vote → 4. Implementation                       │
│                                                                             │
│   Voting-Power basiert auf Karma-Tier:                                     │
│   • Newcomer: 0 Votes                                                      │
│   • Established: 1 Vote                                                    │
│   • Veteran: 2 Votes                                                       │
│   • Elder: 3 Votes                                                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 10.2 Compliance-Anforderungen

| Regulierung   | Bereich           | Erynoa-Lösung                             |
| ------------- | ----------------- | ----------------------------------------- |
| **GDPR**      | Datenschutz       | DIDs sind pseudonym, Selective Disclosure |
| **eIDAS**     | Elektronische ID  | W3C-konforme Verifiable Credentials       |
| **MiFID II**  | Finanzinstrumente | Auditierbare Transaktionen auf NOA        |
| **Eichrecht** | Messwesen (DE)    | Eichrechts-konforme Blueprints            |
| **AFIR**      | E-Mobilität (EU)  | OCPI/OCPP-kompatible Blueprints           |

---

## 11. Wirtschaftsmodell

### 11.1 Wertschöpfungsströme

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│   ERYNOA WIRTSCHAFTSMODELL                                                 │
│                                                                             │
│   WERTSTRÖME                                                               │
│   ══════════                                                                │
│                                                                             │
│   1. TRANSAKTIONSGEBÜHREN (minimal)                                        │
│      • Basis: Feeless auf IOTA                                             │
│      • Premium: 0.01% für High-Value-Anchoring auf Ethereum                │
│                                                                             │
│   2. ENVIRONMENT-GEBÜHREN                                                  │
│      • Environment-Betreiber erheben Membership-Fees                       │
│      • Typisch: 10-100 EUR/Monat pro Agent                                 │
│                                                                             │
│   3. PREMIUM-SERVICES                                                      │
│      • Advanced Analytics                                                  │
│      • Priority Matching                                                   │
│      • Enterprise SLAs                                                     │
│                                                                             │
│   4. VALIDATOR-REWARDS                                                     │
│      • Validatoren erhalten Anteile an Transaktionsgebühren                │
│      • Requires: Elder Karma-Tier                                          │
│                                                                             │
│   KARMA-ÖKONOMIE                                                           │
│   ═══════════════                                                           │
│   • Karma ist NICHT handelbar (Sybil-Schutz)                               │
│   • Karma beeinflusst Privilegien, nicht Zahlungen                         │
│   • Positives Verhalten wird intrinsisch belohnt                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 11.2 Marktpotenzial

| Segment            | TAM (2030)    | SAM (realistisch) | SOM (Jahr 3)  |
| ------------------ | ------------- | ----------------- | ------------- |
| M2M-Payments       | $89 Mrd.      | $5 Mrd.           | $50 Mio.      |
| E-Mobility Roaming | $12 Mrd.      | $2 Mrd.           | $20 Mio.      |
| P2P Energy Trading | $8 Mrd.       | $1 Mrd.           | $10 Mio.      |
| Fleet Management   | $25 Mrd.      | $3 Mrd.           | $30 Mio.      |
| **Gesamt**         | **$134 Mrd.** | **$11 Mrd.**      | **$110 Mio.** |

---

## 12. Risiken und Mitigationen

### 12.1 Technische Risiken

| Risiko                     | Wahrscheinlichkeit | Impact | Mitigation                    |
| -------------------------- | ------------------ | ------ | ----------------------------- |
| IOTA-Netzwerk-Instabilität | Mittel             | Hoch   | Multi-Chain-Fallback          |
| ECLVM-Sicherheitslücken    | Niedrig            | Hoch   | Formale Verifikation, Audits  |
| Skalierungsprobleme        | Mittel             | Mittel | Sharding, Layer-2-Lösungen    |
| Key-Compromise             | Niedrig            | Hoch   | Key Rotation, HSM-Integration |

### 12.2 Marktrisiken

| Risiko                   | Wahrscheinlichkeit | Impact | Mitigation                    |
| ------------------------ | ------------------ | ------ | ----------------------------- |
| Langsame Adoption        | Mittel             | Hoch   | Fokus auf einzelne Domains    |
| Regulatorische Hürden    | Mittel             | Mittel | Proaktive Compliance-Arbeit   |
| Konkurrierende Standards | Hoch               | Mittel | Interoperabilität als Feature |
| Lock-in-Befürchtungen    | Mittel             | Mittel | Open-Source, offene Standards |

### 12.3 Operationelle Risiken

| Risiko               | Wahrscheinlichkeit | Impact | Mitigation                       |
| -------------------- | ------------------ | ------ | -------------------------------- |
| Team-Abhängigkeit    | Mittel             | Mittel | Dokumentation, Knowledge Sharing |
| Finanzierungslücke   | Mittel             | Hoch   | Diversifizierte Funding-Quellen  |
| Governance-Konflikte | Niedrig            | Mittel | Klare Governance-Strukturen      |

---

## 13. Glossar

### Kernbegriffe

| Begriff          | Definition                                                  |
| ---------------- | ----------------------------------------------------------- |
| **AMO**          | Atomic Managed Object – Universelle Objektrepräsentation    |
| **Blueprint**    | Schablone für AMO-Struktur und Validierung                  |
| **DACS**         | Decentralized Anchor Control System – Multi-Chain-Anchoring |
| **DID**          | Decentralized Identifier – Dezentrale Identität             |
| **ECLVM**        | Erynoa Configuration Language VM – Deterministische Runtime |
| **Environment**  | Abgegrenzter Kontext mit spezifischen Regeln                |
| **Intent**       | Formalisierte Absichtserklärung eines Agenten               |
| **Karma**        | Akkumuliertes Vertrauenskapital (nicht handelbar)           |
| **NOA**          | Causal Ledger – Kausales Beweissystem                       |
| **Policy**       | Entscheidungsregeln für autonome Agent-Aktionen             |
| **Trust Vector** | Mehrdimensionaler Vertrauenswert [Rel, Int, Cap, Rep]       |

### Akronyme

| Kürzel | Bedeutung                           |
| ------ | ----------------------------------- |
| DID    | Decentralized Identifier            |
| VC     | Verifiable Credential               |
| AMO    | Atomic Managed Object               |
| ECL    | Erynoa Configuration Language       |
| DACS   | Decentralized Anchor Control System |
| DAG    | Directed Acyclic Graph              |
| DHT    | Distributed Hash Table              |
| OCPP   | Open Charge Point Protocol          |
| OCPI   | Open Charge Point Interface         |
| EIP    | Erynoa Improvement Proposal         |

---

## Dokumentenhistorie

| Version | Datum   | Autor       | Änderungen                                                               |
| ------- | ------- | ----------- | ------------------------------------------------------------------------ |
| 1.0     | 2024-06 | Erynoa Team | Initiale Version                                                         |
| 2.0     | 2026-01 | Erynoa Team | Vollständige Überarbeitung auf Basis concept-v2, 7-Schichten-Architektur |

---

## Referenzen

| Dokument          | Pfad                                                                           |
| ----------------- | ------------------------------------------------------------------------------ |
| Konzept-Navigator | [concept-v2/00-navigator.md](./concept-v2/00-navigator.md)                     |
| Roadmap           | [ROADMAP.md](./ROADMAP.md)                                                     |
| Identity-First    | [concept-v2/anker/identity-first.md](./concept-v2/anker/identity-first.md)     |
| Blueprints        | [concept-v2/schema/blueprints.md](./concept-v2/schema/blueprints.md)           |
| Trust Vectors     | [concept-v2/metrik/trust-vectors.md](./concept-v2/metrik/trust-vectors.md)     |
| Environments      | [concept-v2/sphaere/environments.md](./concept-v2/sphaere/environments.md)     |
| Agent-Modell      | [concept-v2/impuls/agent-modell.md](./concept-v2/impuls/agent-modell.md)       |
| Cybernetic Loop   | [concept-v2/impuls/cybernetic-loop.md](./concept-v2/impuls/cybernetic-loop.md) |
| AMO               | [concept-v2/chronik/amo.md](./concept-v2/chronik/amo.md)                       |
| NOA Ledger        | [concept-v2/chronik/noa-ledger.md](./concept-v2/chronik/noa-ledger.md)         |
| Multi-Chain       | [concept-v2/nexus/multi-chain.md](./concept-v2/nexus/multi-chain.md)           |
| Glossar           | [concept-v2/appendix/glossar.md](./concept-v2/appendix/glossar.md)             |
| Anwendungen       | [concept-v2/appendix/anwendungen.md](./concept-v2/appendix/anwendungen.md)     |
