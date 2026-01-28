# Erynoa – Glossar

> **Dokumenttyp:** Referenz
> **Version:** 1.2
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
│   │  🔐 IDENTITÄT   │  │  🏛️ SPHÄREN     │  │  📦 OBJEKTE     │            │
│   │  DACS, DID, VC  │  │  ERY, ECHO, NOA │  │  AMO, Blueprint │            │
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

## 🔐 Identität & DACS

Begriffe rund um das dezentrale Identitätssystem.

| Begriff                        | Definition                                                                                                                                                                                                     |
| :----------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **DACS**                       | **Decentralized Anchor Control System** – Identity-Modul innerhalb von ERY. Dezentrales System zur Verwaltung von DIDs. DACS Nodes koordinieren DID-Operationen via BFT-Konsens und verankern sie Multi-Chain. |
| **DACS Node**                  | Validierender Knoten im DACS-Netzwerk (Teil der ERY-Infrastruktur). Nimmt an BFT-Konsens teil, speichert DID Documents lokal, signiert Operationen mit BLS Threshold Signatures.                               |
| **did:erynoa**                 | Eigene W3C-konforme DID-Methode von Erynoa. Syntax: `did:erynoa:<namespace>:<unique-identifier>`. Beispiel: `did:erynoa:vehicle:vin-1234567890`.                                                               |
| **DID Document**               | W3C-konformes JSON-LD-Dokument mit Verification Methods, Service Endpoints und Controller-Informationen eines DID-Subjekts.                                                                                    |
| **Multi-Chain Anchoring**      | Strategie zur Verankerung von DIDs auf mehreren Blockchains gleichzeitig. IOTA als Primary Chain (vollständiges DID Doc), Ethereum/Solana als Secondary (Hash + Timestamp).                                    |
| **Primary Chain**              | IOTA Rebased – speichert vollständige DID Documents, führt MoveVM-Logik aus. Hauptquelle für DID-Resolution.                                                                                                   |
| **Secondary Chain**            | Ethereum L2, Solana etc. – speichern nur Hash-Anker und Timestamps. Dienen als Redundanz und Interoperabilitäts-Brücke.                                                                                        |
| **Self-Anchoring**             | Mechanismus, bei dem das DACS seine eigene Registry-DID (`did:erynoa:dacs-registry`) auf allen unterstützten Chains verankert. Bootstrapping ohne externe Abhängigkeit.                                        |
| **Threshold Signatures**       | BLS-basierte t-of-n Signaturen. Mindestens t von n DACS Nodes müssen zustimmen, um eine DID-Operation zu autorisieren (typisch: 67% Threshold).                                                                |
| **Verifiable Credential (VC)** | W3C-konforme kryptografisch signierte Aussage eines Issuers über ein Subjekt. Wird vom DACS ausgestellt und kann von Dritten verifiziert werden.                                                               |
| **BFT Consensus**              | **Byzantine Fault Tolerant Consensus** – PBFT oder HotStuff-basierter Konsens zwischen DACS Nodes. Toleriert bis zu f byzantinische Fehler bei 3f+1 Nodes.                                                     |
| **Chain Adapter**              | Modul im DACS Node, das die Kommunikation mit einer spezifischen Blockchain implementiert (IOTA Adapter, Ethereum Adapter, Solana Adapter).                                                                    |

### Universal DID Namespaces (Identity-First Architecture)

Das Erynoa Identity-First Paradigma definiert, dass **jede Entität** im Ökosystem eine DID besitzt:

| Namespace            | Beschreibung                  | Beispiel-DID                                    |
| :------------------- | :---------------------------- | :---------------------------------------------- |
| **agent:seeker**     | Suchende Agenten              | `did:erynoa:agent:seeker:fleet-agent-001`       |
| **agent:provider**   | Anbietende Agenten            | `did:erynoa:agent:provider:swm-charging`        |
| **agent:autonomous** | Autonome AI-Agenten           | `did:erynoa:agent:autonomous:optimizer-1`       |
| **org**              | Organisationen                | `did:erynoa:org:stadtwerke-munich`              |
| **user**             | Natürliche Personen           | `did:erynoa:user:max-mueller-abc123`            |
| **vehicle**          | Fahrzeuge (Real World Assets) | `did:erynoa:vehicle:vin-WVWZZZ3CZWE123456`      |
| **amo:material**     | Physische Objekte (AMO)       | `did:erynoa:amo:material:station-munich-001`    |
| **amo:credential**   | Soulbound Credentials (AMO)   | `did:erynoa:amo:credential:kyc-verified`        |
| **amo:service**      | Dienstleistungen (AMO)        | `did:erynoa:amo:service:charging-session-xyz`   |
| **blueprint**        | Objekt-Schemata               | `did:erynoa:blueprint:ev-charging-station:v1.2` |
| **standard**         | Normen (ISO, OCPP, Eichrecht) | `did:erynoa:standard:iso:15118:2`               |
| **env:domain**       | Domänen-Umgebungen            | `did:erynoa:env:domain:ev-charging-de`          |
| **env:network**      | Netzwerk-Umgebungen           | `did:erynoa:env:network:hubject-intercharge`    |
| **env:regulatory**   | Regulierte Umgebungen         | `did:erynoa:env:regulatory:eichrecht-de`        |
| **vc**               | Verifiable Credentials        | `did:erynoa:vc:license:fleet-operator-fleetco`  |
| **attestation**      | Trust-Attestationen           | `did:erynoa:attestation:rating-2025-001`        |
| **wallet**           | Krypto-Wallets                | `did:erynoa:wallet:fleetco-main`                |
| **intent**           | Agenten-Intents               | `did:erynoa:intent:i-20250128-abc123`           |
| **policy**           | Agenten-Policies              | `did:erynoa:policy:swm-charging-v2`             |
| **proposal**         | Governance-Vorschläge         | `did:erynoa:proposal:gp-upgrade-v3`             |
| **dao**              | DAO-Organisationen            | `did:erynoa:dao:ev-charging-governance`         |
| **node:dacs**        | DACS-Netzwerk-Knoten          | `did:erynoa:node:dacs:eu-central-1`             |
| **bridge**           | Cross-Chain Bridges           | `did:erynoa:bridge:iota-ethereum`               |
| **sub:avatar**       | Sub-ID: Umgebungs-Avatar      | `did:erynoa:sub:avatar:a1b2c3d4:hubject`        |
| **sub:delegate**     | Sub-ID: Delegierte Befugnis   | `did:erynoa:sub:delegate:e5f6g7h8:negotiator`   |
| **sub:ownership**    | Sub-ID: Besitz-Anker          | `did:erynoa:sub:ownership:i9j0k1l2:vehicle-123` |
| **sub:session**      | Sub-ID: Session-Identity      | `did:erynoa:sub:session:m3n4o5p6:charge-001`    |
| **sub:bundle**       | Sub-ID: Asset-Bündel          | `did:erynoa:sub:bundle:q7r8s9t0:fleet-north`    |
| **sub:proxy**        | Sub-ID: Temporärer Proxy      | `did:erynoa:sub:proxy:u1v2w3x4:emergency`       |
| **sub:capability**   | Sub-ID: Capability-Träger     | `did:erynoa:sub:capability:y5z6a7b8:payment`    |
| **sub:persona**      | Sub-ID: Kontext-Rolle         | `did:erynoa:sub:persona:c9d0e1f2:business`      |
| **sub:guardian**     | Sub-ID: Treuhänder            | `did:erynoa:sub:guardian:g3h4i5j6:iot-custody`  |
| **sub:custodian**    | Sub-ID: Asset-Verwahrer       | `did:erynoa:sub:custodian:k7l8m9n0:cold-store`  |
| **test**             | Test-Entitäten                | `did:erynoa:test:suite:ev-charging-integration` |
| **mock**             | Mock-Entitäten (Testing)      | `did:erynoa:mock:agent:test-seeker`             |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   IDENTITY-FIRST ARCHITECTURE                                │
│   ═════════════════════════════                              │
│                                                              │
│   "Alles existiert, weil es identifizierbar ist"            │
│                                                              │
│   ┌────────────────────────────────────────────────────┐    │
│   │                                                    │    │
│   │   🤖 Agents      → did:erynoa:agent:*             │    │
│   │   📦 Objects     → did:erynoa:amo:*               │    │
│   │   📋 Blueprints  → did:erynoa:blueprint:*         │    │
│   │   🌍 Environments→ did:erynoa:env:*               │    │
│   │   📜 Standards   → did:erynoa:standard:*          │    │
│   │   🎫 Credentials → did:erynoa:vc:*                │    │
│   │   💳 Wallets     → did:erynoa:wallet:*            │    │
│   │   📄 Intents     → did:erynoa:intent:*            │    │
│   │   📜 Policies    → did:erynoa:policy:*            │    │
│   │   🗳️ Proposals   → did:erynoa:proposal:*          │    │
│   │   🔗 Sub-IDs     → did:erynoa:sub:*               │    │
│   │   ⚖️ Legislative → did:erynoa:legislative:*       │    │
│   │   🏛️ Executive   → did:erynoa:executive:*         │    │
│   │   ⚠️ Warning     → did:erynoa:warning:*           │    │
│   │   📋 Complaint   → did:erynoa:complaint:*         │    │
│   │   ⚖️ Dispute     → did:erynoa:dispute:*           │    │
│   │                                                    │    │
│   └────────────────────────────────────────────────────┘    │
│                                                              │
│   Jede Entität hat eigene DID → Universal referenzierbar    │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Governance Namespaces (Legislative/Executive System)

Jede Umgebung hat ein eigenes Governance-System mit identifizierbaren Komponenten:

| Namespace                    | Beschreibung             | Beispiel-DID                                                       |
| :--------------------------- | :----------------------- | :----------------------------------------------------------------- |
| **legislative:**             | Regelwerk einer Umgebung | `did:erynoa:legislative:env:domain:ev-charging-de`                 |
| **executive:**               | Durchsetzungsorgan       | `did:erynoa:executive:env:domain:ev-charging-de`                   |
| **warning:**                 | Formelle Warnung         | `did:erynoa:warning:env:ev-charging:warn-2025-001`                 |
| **complaint:**               | Formelle Beschwerde      | `did:erynoa:complaint:env:ev-charging:compl-2025-042`              |
| **dispute:**                 | Streitfall-Verfahren     | `did:erynoa:dispute:env:ev-charging:disp-2025-007`                 |
| **proposal:minor:**          | Kleiner Änderungsantrag  | `did:erynoa:proposal:minor:env:ev-charging:prop-2025-015`          |
| **proposal:major:**          | Größerer Änderungsantrag | `did:erynoa:proposal:major:env:ev-charging:prop-2025-003`          |
| **proposal:constitutional:** | Verfassungsänderung      | `did:erynoa:proposal:constitutional:env:ev-charging:prop-2025-001` |

### Sub-Identity Namespaces (Hierarchische Identitäten)

Agenten können **Sub-Identitäten** erzeugen – abgeleitete Identitäten mit eingeschränktem Gültigkeitsbereich:

| Namespace          | Beschreibung               | Beispiel-DID                                          |
| :----------------- | :------------------------- | :---------------------------------------------------- |
| **sub:avatar**     | Umgebungs-Repräsentation   | `did:erynoa:sub:avatar:a1b2c3d4:hubject-network`      |
| **sub:delegate**   | Delegierte Befugnisse      | `did:erynoa:sub:delegate:e5f6g7h8:night-negotiator`   |
| **sub:ownership**  | Besitz-Anker für Assets    | `did:erynoa:sub:ownership:i9j0k1l2:vehicle-vin-123`   |
| **sub:session**    | Session-gebundene Identity | `did:erynoa:sub:session:m3n4o5p6:charging-20250128`   |
| **sub:bundle**     | Asset-Bündel               | `did:erynoa:sub:bundle:q7r8s9t0:fleet-north`          |
| **sub:proxy**      | Temporärer Stellvertreter  | `did:erynoa:sub:proxy:u1v2w3x4:emergency-handler`     |
| **sub:capability** | Capability-Träger          | `did:erynoa:sub:capability:y5z6a7b8:payment-auth`     |
| **sub:persona**    | Kontext-spezifische Rolle  | `did:erynoa:sub:persona:c9d0e1f2:business-context`    |
| **sub:guardian**   | Treuhänder/Vormund         | `did:erynoa:sub:guardian:g3h4i5j6:iot-device-custody` |
| **sub:custodian**  | Verwahrer für Assets       | `did:erynoa:sub:custodian:k7l8m9n0:cold-storage`      |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   SUB-IDENTITY HIERARCHIE                                    │
│   ═══════════════════════════                                │
│                                                              │
│   Root-Identity (Agent/User/Org)                            │
│      │                                                       │
│      ├── Avatar (Umgebung A)                                │
│      │      └── Session (Transaktion)                       │
│      │                                                       │
│      ├── Avatar (Umgebung B)                                │
│      │      └── Session (Transaktion)                       │
│      │                                                       │
│      ├── Delegate (Autonome Aufgabe)                        │
│      │      └── Session (Verhandlung)                       │
│      │                                                       │
│      ├── Ownership Anchor (Asset)                           │
│      │                                                       │
│      └── Bundle (Asset-Sammlung)                            │
│             ├── Ownership Anchor → Asset A                  │
│             ├── Ownership Anchor → Asset B                  │
│             └── Ownership Anchor → Asset C                  │
│                                                              │
│   💡 Scope verengt sich mit jeder Ebene                     │
│   💡 Trust wird anteilig vererbt (70%)                      │
│   💡 Vollständige Audit-Trail                               │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   DACS MODULE (Teil von ERY)                                │
│                                                              │
│   ┌─────────────────────────────────────────────────────┐   │
│   │           DACS Node Network (BFT)                    │   │
│   │       Threshold Signatures (BLS t-of-n)              │   │
│   └──────────────────────┬──────────────────────────────┘   │
│                          │                                   │
│            ┌─────────────┼─────────────┐                    │
│            ▼             ▼             ▼                     │
│       ┌────────┐    ┌────────┐    ┌────────┐                │
│       │  IOTA  │    │  ETH   │    │  SOL   │                │
│       │Primary │    │Second. │    │Second. │                │
│       │Full Doc│    │ Hash   │    │ Hash   │                │
│       └────────┘    └────────┘    └────────┘                │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 🏛️ Sphären

Die drei Sphären bilden das Fundament der Erynoa-Architektur. DACS ist ein Modul innerhalb von ERY.

| Begriff  | Definition                                                                                                                                                                                                 |
| :------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **ERY**  | **Semantic & Identity Lattice** – Das Gedächtnis und die Identität des Netzwerks. Enthält vier Module: DACS (Identität), Semantic Index (Wissen), Karmic Engine (Vertrauen), Discovery (DHT + Geohashing). |
| **ECHO** | **Emergent Swarm** – Die operative Intelligenz. Führt Agentenlogik aus, wickelt Discovery, Verhandlung und Ausführung von Intents ab.                                                                      |
| **NOA**  | **Causal Ledger** – Die Quelle der Wahrheit. On-Chain-Ledger, der Transaktionen finalisiert und Zustandsänderungen an AMOs vollzieht.                                                                      |

### ERY-Module

| Modul                 | Funktion                                                                                                                     |
| :-------------------- | :--------------------------------------------------------------------------------------------------------------------------- |
| **🔐 DACS**           | **Decentralized Anchor Control System** – Identity-Modul. Verwaltet DIDs, stellt VCs aus, verankert Identitäten Multi-Chain. |
| **📚 Semantic Index** | Speichert Blueprints, Normen, Ontologien. Qdrant-basierte Vektorsuche für semantische Queries.                               |
| **⚖️ Karmic Engine**  | Berechnet Trust Vectors aus Events und Attestations. Implementiert Ripple Effect für Trust-Propagation.                      |
| **🌍 Discovery**      | DHT (libp2p Kademlia) + Geohashing für dezentrale, privacy-schonende Agent-Discovery.                                        |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│                          ERY                                 │
│   ┌────────────────────────────────────────────────────┐    │
│   │  🔐 DACS    📚 Semantic   ⚖️ Karmic   🌍 Discovery │    │
│   │  Identity   Index        Engine      DHT+Geo       │    │
│   └────────────────────────────────────────────────────┘    │
│                          │                                   │
│            ┌─────────────┼─────────────┐                    │
│            ▼             ▼             ▼                     │
│         ECHO           NOA       Multi-Chain                │
│        Agenten      Settlement    Anchoring                 │
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

| Begriff                     | Definition                                                                                                                      |
| :-------------------------- | :------------------------------------------------------------------------------------------------------------------------------ |
| **Trust Vector**            | Mehrdimensionaler Vektor, der das Vertrauen in ein Subjekt beschreibt. Wird von der Karmic Engine berechnet.                    |
| **Karmic Engine**           | Komponente in ERY, die aus Events und Attestations Trust Vectors berechnet. Nutzt den Ripple-Effekt für dynamische Updates.     |
| **Attestation**             | Signierte Aussage einer externen oder internen Instanz über ein Subjekt (z.B. DNS-Bindung, Zertifikate, Konformitätsnachweise). |
| **Trust Gating**            | Mechanismus, bei dem minimale Trust-Schwellen als Zugangskriterium für Interaktionen dienen.                                    |
| **Trust Inheritance**       | Fraktale Vererbung von Trust entlang hierarchischer Beziehungen (Hersteller → Betreiber → Asset).                               |
| **Ripple Effect**           | Algorithmus zur Propagation von Trust-Änderungen durch das Netzwerk mit Dämpfungsfaktor λ.                                      |
| **Event**                   | Abstraktion eines finalisierten Vorgangs in NOA (z.B. erfolgreiche Lieferung, SLA-Verstoß). Input für die Karmic Engine.        |
| **Environment Trust**       | Trust Vector einer Search Environment. Quantifiziert Zuverlässigkeit, Qualität, Aktualität und Governance einer Umgebung.       |
| **Bidirectional Trust**     | Wechselseitige Trust-Propagation zwischen Environments und ihren Members. Members beeinflussen Env-Trust und umgekehrt.         |
| **Unified Trust Model**     | Ganzheitliches Bewertungssystem, das alle Entitätstypen (Agents, AMOs, Orgs, Envs, Blueprints, Standards) kohärent bewertet.    |
| **Trust Propagation Graph** | Netzwerk aller Trust-Beziehungen zwischen Entitäten mit bidirektionalen Kanten und Dämpfungsfaktoren.                           |
| **Member Trust Bonus**      | Trust-Aufschlag, den Mitglieder von hochrangigen Environments erhalten. Gewichtet mit `env_bonus_weight`.                       |
| **Environment Governance**  | Trust-Dimension, die misst, wie streng Constraints und Standards in einer Umgebung durchgesetzt werden.                         |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   UNIFIED TRUST MODEL                                        │
│   ═══════════════════                                        │
│                                                              │
│   Bewertbare Entitäten:                                      │
│   ┌────────────────────────────────────────────────────┐    │
│   │  🤖 Agents    📦 AMOs     🏢 Organizations         │    │
│   │  🌍 Envs      📋 Blueprints  🔗 Standards          │    │
│   └────────────────────────────────────────────────────┘    │
│                          │                                   │
│                          ▼                                   │
│   ┌────────────────────────────────────────────────────┐    │
│   │               KARMIC ENGINE                        │    │
│   │  Events + Attestations + Inheritance → Trust       │    │
│   └────────────────────────────────────────────────────┘    │
│                          │                                   │
│                          ▼                                   │
│            Bidirektionale Propagation                        │
│            zwischen allen Entitäten                          │
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
| **Intent**                 | Maschinenlesbarer Wunsch eines Seekers, bestehend aus funktionalen, normativen und Trust-Anforderungen. Definiert in ECL.    |
| **Policy**                 | Deklarative Regel eines Providers, die definiert, unter welchen Bedingungen Anfragen akzeptiert werden. Definiert in ECL.    |
| **Consensus Bubble**       | Verschlüsselte Off-Chain-Kommunikationsumgebung (XMTP), in der Agenten privat verhandeln.                                    |
| **Progressive Disclosure** | Prinzip, nach dem sensible Informationen nur schrittweise offengelegt werden, wenn Vertrauen gegeben ist.                    |

---

## 🧬 Erynoa Configuration Language (ECL)

Die einheitliche, modulare Beschreibungssprache für das gesamte Erynoa-System.

| Begriff            | Definition                                                                                                                    |
| :----------------- | :---------------------------------------------------------------------------------------------------------------------------- |
| **ECL**            | **Erynoa Configuration Language** – Einheitliche modulare Sprache für das gesamte Erynoa-Ökosystem. Definiert alles als Code. |
| **ecl/core**       | Basismodul mit primitiven Typen, Collections, Constraints und Referenz-Mechanismen. Grundlage für alle anderen Module.        |
| **ecl/identity**   | Modul für DIDs, Verifiable Credentials und DACS-Konfiguration. Ersetzt separate Identity-Definitionen.                        |
| **ecl/object**     | Modul für Blueprints, AMOs und Fluid Extensions. Definiert das Liquide Datenmodell in ECL-Syntax.                             |
| **ecl/environ**    | Modul für Search Environments, Hierarchien, Heuristiken und Constraints. Ex-EDL vollständig integriert.                       |
| **ecl/agent**      | Modul für Seeker Intents und Provider Policies. Ex-ADL vollständig integriert als ECL-Submodul.                               |
| **ecl/trust**      | Modul für Trust-Vektoren, Attestations, Karmic Rules und Trust-Konfiguration.                                                 |
| **ecl/economic**   | Modul für Preismodelle, Multi-Chain Wallets, Payment-Flows und Network Selection.                                             |
| **ecl/network**    | Modul für Chain-Konfigurationen, Bridge-Definitionen und Cross-Chain-Routing.                                                 |
| **ecl/governance** | Modul für Voting-Mechanismen, DAO-Rules und Protokoll-Upgrade-Pfade.                                                          |
| **ecl/test**       | Modul für Test-Spezifikationen, Mocks und Simulationen.                                                                       |
| **ADL**            | **Agent Definition Language** – Historischer Name, jetzt Teil von ecl/agent. Deklarative Sprache für Intents und Policies.    |
| **EDL**            | **Environment Definition Language** – Historischer Name, jetzt Teil von ecl/environ. Sprache für Search Environments.         |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│               ECL MODULE HIERARCHY                           │
│                                                              │
│                    ┌───────────┐                            │
│                    │ ecl/core  │                            │
│                    │  Types &  │                            │
│                    │Constraints│                            │
│                    └─────┬─────┘                            │
│                          │                                   │
│   ┌──────────┬───────────┼───────────┬──────────┐           │
│   │          │           │           │          │           │
│   ▼          ▼           ▼           ▼          ▼           │
│ ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐           │
│ │iden- │  │object│  │envir-│  │agent │  │trust │           │
│ │tity  │  │      │  │on    │  │      │  │      │           │
│ └──────┘  └──────┘  └──────┘  └──────┘  └──────┘           │
│                                                              │
│   ┌──────┐  ┌──────┐  ┌──────┐                              │
│   │econo-│  │net-  │  │gover-│                              │
│   │mic   │  │work  │  │nance │                              │
│   └──────┘  └──────┘  └──────┘                              │
│                                                              │
│   "Everything as Code"                                       │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 💰 Multi-Chain Wallet & Network Selection

Begriffe rund um das Multi-Chain Zahlungssystem der Agenten.

| Begriff                      | Definition                                                                                                                                  |
| :--------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------ |
| **Multi-Chain Wallet**       | Wallet-System eines Agenten, das Guthaben auf mehreren Blockchains (IOTA, ETH L2, Solana) gleichzeitig verwaltet. Verknüpft mit did:erynoa. |
| **Network Selection Engine** | Komponente in ECHO, die automatisch das optimale Netzwerk für eine Transaktion wählt basierend auf Gebühren, Latenz und Counterparty.       |
| **Chain Priority**           | Priorisierte Liste der bevorzugten Chains eines Agenten. Höhere Priorität = bevorzugt bei Gleichstand anderer Kriterien.                    |
| **Counterparty-Match**       | Optimierungsstrategie: Wähle die Chain, auf der beide Parteien (Seeker & Provider) Guthaben haben → keine Bridge-Kosten.                    |
| **Cross-Chain Bridge**       | Mechanismus zum atomaren Transfer von Assets zwischen verschiedenen Blockchains via DACS-koordiniertem Settlement.                          |
| **Atomic Cross-Chain**       | Settlement, bei dem Transaktionen auf zwei Chains gleichzeitig finalisiert werden (alle oder keine). Verhindert Verluste.                   |
| **Fee Oracle**               | Dienst, der aktuelle Transaktionsgebühren aller unterstützten Chains bereitstellt. Input für die Network Selection Engine.                  |
| **Latency Preference**       | Konfigurationsoption: max_latency_seconds definiert, wie schnell eine Transaktion finalisiert sein muss.                                    |
| **Bridge Fee Threshold**     | Konfigurierbare Obergrenze (max_bridge_fee_eur), ab der ein Cross-Chain-Bridge wirtschaftlich vertretbar ist.                               |
| **wallet_balance()**         | Host API zum Abfragen des Guthabens auf einer bestimmten Chain. Gibt Token-Typ und Menge zurück.                                            |
| **wallet_transfer()**        | Host API zum Initiieren eines Transfers. Akzeptiert chain, recipient_did, amount, optional: bridge_if_needed.                               |
| **network_select()**         | Host API zum Ermitteln der optimalen Chain basierend auf Betrag, Empfänger und konfigurierten Präferenzen.                                  |
| **network_fees()**           | Host API zum Abfragen aktueller Gebühren aller unterstützten Chains für Kostenvergleich.                                                    |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   MULTI-CHAIN WALLET                                         │
│   ══════════════════                                         │
│                                                              │
│   ┌─────────┐   ┌─────────┐   ┌─────────┐                   │
│   │  IOTA   │   │  ETH L2 │   │ SOLANA  │                   │
│   │ Wallet  │   │ Wallet  │   │ Wallet  │                   │
│   └────┬────┘   └────┬────┘   └────┬────┘                   │
│        │             │             │                         │
│        └─────────────┼─────────────┘                         │
│                      ▼                                       │
│   ┌──────────────────────────────────────────────────────┐  │
│   │            NETWORK SELECTION ENGINE                  │  │
│   │  ─────────────────────────────────────────────────   │  │
│   │  • Counterparty-Chain ermitteln                      │  │
│   │  • Gemeinsame Chains identifizieren                  │  │
│   │  • Gebühren & Latenz vergleichen                     │  │
│   │  • Optimale Chain wählen (oder Bridge)               │  │
│   └──────────────────────────────────────────────────────┘  │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## 🌍 Search Environments (Suchumgebungen)

Begriffe rund um hierarchische Suchordnungen und virtuelle Umgebungen.

| Begriff                     | Definition                                                                                                                                                      |
| :-------------------------- | :-------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Search Environment**      | Strukturierte Abstraktionsebene innerhalb des Discovery-Moduls mit eigenen Ordnungsrelationen, Heuristiken und Constraints. Hierarchisch verschachtelbar.       |
| **Real World Environment**  | Root-Umgebung (🌍) der Hierarchie. Repräsentiert die physische Welt mit Geohashing als primärer Ordnungsrelation. Uninformierte Suche möglich.                  |
| **Virtual Environment**     | Abstraktionsebene (🔮) unterhalb der realen Welt mit eigenen Ordnungsprinzipien, Standards und Heuristiken. Kann beliebig tief verschachtelt werden (∞ Ebenen). |
| **Domain Environment**      | Umgebungstyp (🏭) für standard-verknüpfte Fachdomänen (z.B. ISO, eCl@ss). Erbt Constraints von Parent und definiert domänenspezifische Regeln.                  |
| **Network Environment**     | Umgebungstyp (🌐) für netzwerkbasierte Strukturen (z.B. Roaming-Verbünde wie Hubject). Membership-basierte Filterung.                                           |
| **Regulatory Environment**  | Umgebungstyp (⚖️) für regulatorisch definierte Kontexte (z.B. Eichrecht-Konformität). Automatische Constraint-Ableitung aus Regulierungen.                      |
| **Custom Environment**      | Benutzerdefinierter Umgebungstyp (🎨) für spezifische Anwendungsfälle. Kann von Agents oder Organisationen dynamisch erstellt werden.                           |
| **Environment Hierarchy**   | Baumstruktur der Umgebungen mit Real World als Root. Sub-Umgebungen erben Constraints ihrer Parents und können zusätzliche definieren.                          |
| **Informed Search**         | Suchstrategie mit domänenspezifischer Heuristik (A\*, Greedy, Beam). Nutzt Wissen über die Domäne für effiziente Traversierung.                                 |
| **Uninformed Search**       | Suchstrategie ohne domänenspezifisches Wissen (BFS, DFS). Exploriert systematisch alle Nachbarn ohne Präferenz.                                                 |
| **Search Heuristic**        | Bewertungsfunktion innerhalb einer Umgebung, die Kandidaten nach domänenspezifischen Kriterien scored (z.B. `ev_charging_score`, `merit_order`).                |
| **Environment Constraint**  | Bedingung, die alle Mitglieder einer Umgebung erfüllen müssen. Hard Constraints werden erzwungen, Soft Constraints beeinflussen Ranking.                        |
| **Cross-Environment Query** | Suchanfrage, die mehrere Umgebungen referenziert (primary, intersect, exclude, fallback). Ermöglicht komplexe, kontextbewusste Discovery.                       |
| **EDL**                     | **Environment Definition Language** – Deklaratives Format zur Spezifikation von Umgebungen inkl. Ordnung, Heuristiken, Standards und Constraints.               |
| **Environment Membership**  | Zugehörigkeit eines AMO oder Agents zu einer Umgebung. Kann automatisch (regelbasiert) oder manuell erfolgen.                                                   |
| **Standard Linkage**        | Verknüpfung einer Umgebung mit Normen, Blueprints oder Zertifizierungen im Semantic Index. Ermöglicht automatische Constraint-Ableitung.                        |
| **Constraint Inheritance**  | Mechanismus, bei dem Sub-Umgebungen automatisch alle Constraints ihrer Parent-Umgebungen erben. Constraints können hinzugefügt, aber nicht entfernt werden.     |
| **Geospatial Ordering**     | Ordnungsrelation basierend auf Geohashing. Primäre Ordnung in der Real World Environment.                                                                       |
| **Topological Ordering**    | Ordnungsrelation basierend auf Graph-Distanz (z.B. Grid-Topologie, Supply Chain Hops). Verwendet in virtuellen Umgebungen.                                      |
| **Semantic Ordering**       | Ordnungsrelation basierend auf Blueprint-Hierarchie oder Vektor-Ähnlichkeit. Für konzeptuelle Nähe zwischen Objekten.                                           |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   ENVIRONMENT HIERARCHY                                      │
│   ═════════════════════                                      │
│                                                              │
│   🌍 REAL_WORLD (ROOT)                                      │
│   │   Geohashing · BFS/DFS · Uninformiert                   │
│   │                                                          │
│   ├── 🏭 DOMAIN: Industry (eCl@ss, ISO)                     │
│   │       │                                                  │
│   │       └── 🌐 NETWORK: Specific Vendor                   │
│   │                                                          │
│   ├── 🔋 DOMAIN: Energy (Grid Codes)                        │
│   │       │                                                  │
│   │       ├── 🔮 VIRTUAL: EV-Charging (OCPP)                │
│   │       │       │                                          │
│   │       │       ├── 🌐 Hubject                            │
│   │       │       └── 🌐 Girö-E                             │
│   │       │                                                  │
│   │       └── ⚖️ REGULATORY: Eichrecht                      │
│   │                                                          │
│   └── 🎨 CUSTOM: Organization-specific                      │
│                                                              │
│   ∞ Levels: Beliebig tiefe Verschachtelung möglich          │
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

## ⚡ ECLVM – Erynoa Virtual Machine

Die ECLVM ermöglicht **dynamisches Programmieren** durch Agenten. Agenten können Code zur Laufzeit ausführen, Templates instanziieren und neue Entitäten erzeugen.

| Begriff                    | Definition                                                                                                                                            |
| :------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------- |
| **ECLVM**                  | **Erynoa Configuration Language Virtual Machine** – Stack-basierte VM, die ECL-Bytecode ausführt. Ermöglicht dynamische Programmierung durch Agenten. |
| **ECL Bytecode**           | Kompilierte, portable Zwischenrepräsentation von ECL-Code. Stack-basiertes Instruction Set mit ~100 Opcodes.                                          |
| **Template**               | Parametrisierte Schablone für Entitäten (Blueprints, Environments, Agents). Kann zur Laufzeit instanziiert werden.                                    |
| **Template Instantiation** | Prozess der Erzeugung einer neuen Entität aus einem Template mit konkreten Parametern. Generiert automatisch eine neue DID.                           |
| **Sandbox**                | Isolierte Ausführungsumgebung für Agent-Code. Kein direkter Zugriff auf Dateisystem oder Netzwerk – nur Host-API-Calls.                               |
| **Gas Metering**           | Ressourcenbegrenzung durch Instruction Counting. Jede Operation kostet "Gas" – bei Erschöpfung wird Ausführung abgebrochen.                           |
| **Host API**               | Kontrollierte Schnittstelle zwischen Sandbox und Host-System. Ermöglicht Network Discovery, Storage, Crypto-Operationen, etc.                         |
| **Hot-Code-Reload**        | Aktualisierung von Funktionen zur Laufzeit ohne Neustart. Erfordert Signatur-Kompatibilität.                                                          |
| **Live-Patching**          | Ersetzen einzelner Funktionen im laufenden Betrieb. Agent kann eigene Logik dynamisch anpassen.                                                       |
| **Agent Program**          | Von einem Agenten geschriebener ECL-Code, der in der ECLVM ausgeführt wird. Kombiniert deklarative (Templates) und imperative (Funktionen) Elemente.  |
| **Schema Evolution**       | Kontrollierte Erweiterung von Schemas ohne Datenverlust. Unterstützt Add-Only-Changes und Deprecations.                                               |
| **State Migration**        | Automatische Transformation von internem State bei Version-Updates. Definiert Rename/Transform-Regeln.                                                |
| **Checkpoint**             | Snapshot des Ausführungszustands für möglichen Rollback. Ermöglicht transaktionale Semantik.                                                          |
| **Resource Limits**        | Konfigurierbare Grenzen für Gas, Memory, Time, Host-Calls pro Ausführung. Verhindert Ressourcen-Erschöpfung.                                          |
| **DID_GENERATE**           | ECLVM-Opcode zur dynamischen Generierung neuer DIDs. Erzeugt `did:erynoa:<namespace>:<unique>`.                                                       |
| **TEMPLATE_LOAD**          | ECLVM-Opcode zum Laden eines Templates aus der Registry anhand seiner DID.                                                                            |
| **TEMPLATE_BIND**          | ECLVM-Opcode zum Binden von Parameterwerten an ein geladenes Template.                                                                                |
| **TEMPLATE_INSTANTIATE**   | ECLVM-Opcode zur Instanziierung eines Templates mit gebundenen Parametern. Erzeugt neue Entität mit generierter DID.                                  |
| **ENV_CREATE**             | ECLVM-Opcode zur dynamischen Erstellung einer neuen Umgebung aus einem Environment-Template.                                                          |
| **Compile-Time**           | Phase, in der ECL-Sourcecode zu Bytecode kompiliert wird. Type-Checking und Optimierung finden hier statt.                                            |
| **Runtime**                | Phase, in der ECL-Bytecode in der ECLVM ausgeführt wird. Sandbox-Enforcement und Gas-Metering aktiv.                                                  |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   ECLVM EXECUTION PIPELINE                                  │
│   ════════════════════════                                   │
│                                                              │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│   │ ECL Source  │ →  │  Compiler   │ →  │  Bytecode   │     │
│   │    Code     │    │  (Parser,   │    │   (.eclb)   │     │
│   │             │    │  TypeCheck) │    │             │     │
│   └─────────────┘    └─────────────┘    └─────────────┘     │
│                                                │              │
│                                                ▼              │
│   ┌────────────────────────────────────────────────────┐    │
│   │                   ECLVM Runtime                     │    │
│   │  ┌──────────┐ ┌──────────┐ ┌──────────┐            │    │
│   │  │  Stack   │ │  Heap    │ │ Sandbox  │            │    │
│   │  │ Machine  │ │ Manager  │ │ Enforcer │            │    │
│   │  └──────────┘ └──────────┘ └──────────┘            │    │
│   │                     │                               │    │
│   │                     ▼                               │    │
│   │  ┌──────────────────────────────────────────┐      │    │
│   │  │           Host API Bridge                 │      │    │
│   │  │  (network, storage, crypto, governance)   │      │    │
│   │  └──────────────────────────────────────────┘      │    │
│   └────────────────────────────────────────────────────┘    │
│                                                              │
│   💡 Agenten programmieren – nicht nur konfigurieren        │
│   💡 Templates zur Laufzeit instanziieren                   │
│   💡 Alles sandboxed mit Resource Limits                    │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### ECLVM Namespace-Erweiterungen

| Namespace           | Beschreibung          | Beispiel-DID                                        |
| :------------------ | :-------------------- | :-------------------------------------------------- |
| **template:**       | Template-Definitionen | `did:erynoa:template:blueprint:charging-station:v1` |
| **template:env:**   | Environment-Templates | `did:erynoa:template:env:roaming-network:v1`        |
| **template:agent:** | Agent-Templates       | `did:erynoa:template:agent:fleet-manager:v1`        |
| **program:**        | Agent-Programme       | `did:erynoa:program:fleet-optimizer:v1`             |
| **bytecode:**       | Kompilierter Code     | `did:erynoa:bytecode:fleet-optimizer:v1:abc123`     |

---

## 📚 Schichtmodell

Zusammenfassung der Architektur-Layer.

| Layer         | Sphären    | Aufgaben                                                   |
| :------------ | :--------- | :--------------------------------------------------------- |
| **Layer 0**   | NOA        | On-Chain: Finalität, AMOs, Move-Execution, Starfish BFT    |
| **Layer 0.5** | **ECLVM**  | Execution Engine: Bytecode-Interpreter, Templates, Sandbox |
| **Layer 2**   | ERY + ECHO | Off-Chain: Semantik, Trust, Agenten, Verhandlung           |

```
┌──────────────────────────────────────────────────────────────┐
│                                                              │
│   LAYER 2 (Off-Chain)                                       │
│   ┌─────────────────────┐  ┌─────────────────────┐          │
│   │        ERY          │  │        ECHO         │          │
│   │  Semantic Lattice   │  │   Emergent Swarm    │          │
│   └─────────────────────┘  └─────────────────────┘          │
│                    │                    │                    │
│   ─────────────────┴────────────────────┴─────────────────   │
│                             │                                │
│   LAYER 0.5 (Execution)     ▼                                │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                      ECLVM                          │   │
│   │        Bytecode Interpreter, Template Engine        │   │
│   └─────────────────────────────────────────────────────┘   │
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

│ │ │
│ ───────────────────────────────────────────────────────── │
│ │ │
│ LAYER 0 (On-Chain) ▼ │
│ ┌─────────────────────────────────────────────────────┐ │
│ │ NOA │ │
│ │ Causal Ledger │ │
│ └─────────────────────────────────────────────────────┘ │
│ │
└──────────────────────────────────────────────────────────────┘

```

---

## Kurzreferenz (Alphabetisch)

| Begriff                   | Kurzdefinition                                  |
| :------------------------ | :---------------------------------------------- |
| ADL                       | Agent Definition Language (→ ecl/agent)         |
| AMO                       | Atomic Market Object                            |
| Attestation               | Signierte Aussage über ein Subjekt              |
| Atomic Cross-Chain        | Gleichzeitiges Settlement auf zwei Chains       |
| Bidirectional Trust       | Wechselseitige Trust-Propagation                |
| Blueprint                 | Semantische Schablone für Objekte               |
| Chain Priority            | Priorisierte Chain-Liste eines Agenten          |
| Consensus Bubble          | Verschlüsselte Verhandlungsumgebung             |
| Constraint Inheritance    | Vererbung von Constraints in Sub-Umgebungen     |
| Counterparty-Match        | Chain-Wahl basierend auf Partner-Guthaben       |
| Cross-Chain Bridge        | Atomarer Asset-Transfer zwischen Chains         |
| Cross-Environment Query   | Suchanfrage über mehrere Umgebungen             |
| Custom Environment        | Benutzerdefinierte Suchumgebung                 |
| DID                       | Decentralized Identifier                        |
| DHT                       | Distributed Hash Table                          |
| Domain Environment        | Standard-verknüpfte Fachdomänen-Umgebung        |
| ECHO                      | Emergent Swarm (Agenten-Sphäre)                 |
| **ECL**                   | **Erynoa Configuration Language**               |
| ecl/agent                 | ECL-Modul für Intents & Policies                |
| ecl/core                  | ECL-Basismodul (Typen, Constraints)             |
| ecl/economic              | ECL-Modul für Preise & Wallets                  |
| ecl/environ               | ECL-Modul für Search Environments               |
| ecl/governance            | ECL-Modul für DAO & Voting                      |
| ecl/identity              | ECL-Fundament für DIDs & Credentials            |
| ecl/network               | ECL-Modul für Multi-Chain & Bridges             |
| ecl/object                | ECL-Modul für Blueprints & AMOs                 |
| ecl/test                  | ECL-Modul für Tests & Mocks                     |
| ecl/trust                 | ECL-Modul für Trust-Vektoren                    |
| EDL                       | Environment Definition Language (→ ecl/environ) |
| Environment Governance    | Trust-Dimension für Constraint-Durchsetzung     |
| Environment Hierarchy     | Baumstruktur der Suchumgebungen                 |
| Environment Membership    | Zugehörigkeit zu einer Suchumgebung             |
| **Identifiable**          | ECL-Trait: Jede Entität muss DID haben          |
| **Identity-First**        | Paradigma: Alles existiert durch Identität      |
| **Identity Layer**        | ECL Foundation Layer (ecl/identity)             |
| **Identity Relationship** | DID-basierte Verknüpfung zwischen Entitäten     |
| **Identity Scope**        | Gültigkeitsbereich einer (Sub-)Identity         |
| **Identity-Trait**        | Pflicht-Interface für alle ECL-Entitäten        |
| Environment Trust         | Trust Vector einer Suchumgebung                 |
| EOS                       | Erynoa Object Standard                          |
| ERY                       | Semantic Lattice (Wissens-Sphäre)               |
| Fee Oracle                | Gebühren-Abfrage aller Chains                   |
| Fluid Extensions          | Temporäre AMO-Attribute mit TTL                 |
| Geohashing                | Geo-Koordinaten als kompakte Strings            |
| Geospatial Ordering       | Räumliche Ordnung via Geohash                   |
| Informed Search           | Heuristik-basierte Suchstrategie                |
| Intent                    | Maschinenlesbarer Wunsch eines Seekers          |
| Karmic Engine             | Trust-Berechnungskomponente                     |
| **Karmic Ledger**         | Power-Konten aller Umgebungs-Mitglieder         |
| **Karmic Power**          | Akkumulierte Governance-Stärke eines Mitglieds  |
| **Karma Asymmetry**       | Negatives Karma wiegt stärker als positives     |
| **Karma Flow**            | Karma-Transfer von Assets zu Owner              |
| **Karma Tier**            | Stufe basierend auf akkumulierter Karma         |
| Logic Guards              | Invarianten-Prüfung vor Zustandsänderungen      |
| **Legislative**           | Regelgebungsorgan einer Umgebung (Karma-Rules)  |
| **Executive**             | Durchsetzungsorgan (Warning Stack, Enforcement) |
| **ECL Bytecode**          | Kompilierter Stack-basierter VM-Code            |
| **ECLVM**                 | Erynoa VM – Dynamische Programmierung           |
| ecl/vm                    | ECL-Modul für VM, Templates, Sandbox            |
| **Gas Metering**          | Ressourcenbegrenzung via Instruction Counting   |
| **Host API**              | Sandbox-Bridge zu Network/Storage/Crypto        |
| **Hot-Code-Reload**       | Funktions-Update zur Laufzeit                   |
| Member Trust Bonus        | Trust-Aufschlag von hochrangigen Environments   |
| Move/MoveVM               | Sprache und VM für Smart Contracts              |
| Multi-Chain Wallet        | Wallet mit Guthaben auf mehreren Chains         |
| Network Environment       | Netzwerk-basierte Suchumgebung                  |
| Network Selection Engine  | Automatische optimale Netzwerkwahl              |
| **Network Warning**       | Formelle Warnung im Executive Warning Stack     |
| network_select()          | Host API für Netzwerkwahl                       |
| NOA                       | Causal Ledger (Finalitäts-Sphäre)               |
| **Ownership Anchor**      | Sub-Identity die Besitz an Asset repräsentiert  |
| Policy                    | Deklarative Annahme-Regeln eines Providers      |
| **Power Delegation**      | Übertragung von Governance-Power an andere      |
| Progressive Disclosure    | Schrittweise Informationsfreigabe               |
| Qdrant                    | Vektor-Datenbank für ERY                        |
| Real World Environment    | Root-Umgebung der physischen Welt               |
| Regulatory Environment    | Regulatorisch definierte Suchumgebung           |
| **Reputation Event**      | Protokolliertes Karma-änderndes Ereignis        |
| **Resource Limits**       | Sandbox-Grenzen: Gas, Memory, Time, Host-Calls  |
| Ripple Effect             | Trust-Propagation mit Dämpfung                  |
| **Root Identity**         | Ursprungs-Identity ohne Parent (depth=0)        |
| **Sandbox**               | Isolierte Ausführungsumgebung für Agent-Code    |
| Search Environment        | Hierarchische Abstraktionsebene für Discovery   |
| Search Heuristic          | Bewertungsfunktion für Suchkandidaten           |
| Semantic Ordering         | Konzeptuelle Ordnung via Blueprints/Vektoren    |
| Standard Linkage          | Verknüpfung mit Normen im Semantic Index        |
| Starfish BFT              | Leaderloser Konsens in NOA                      |
| **Sub-Identity**          | Abgeleitete Identity mit eingeschränktem Scope  |
| **Sub-Identity Avatar**   | Umgebungs-spezifische Repräsentation            |
| **Sub-Identity Bundle**   | Zusammenfassung mehrerer Assets                 |
| **Sub-Identity Delegate** | Delegierte Befugnisse für Aufgaben              |
| **Sub-Identity Session**  | Temporäre, transaktionsgebundene Identity       |
| **SubIdentityKind**       | ECL-Enum der Sub-Identity-Arten                 |
| Synapse                   | Elementare Speichereinheit in ERY               |
| **Template**              | Parametrisierte Schablone für Entitäten         |
| **Template Instantiation**| Erzeugung neuer Entität aus Template            |
| TEMPLATE_INSTANTIATE      | ECLVM-Opcode: Template zu Entität               |
| TEMPLATE_LOAD             | ECLVM-Opcode: Template aus Registry laden       |
| Topological Ordering      | Graph-basierte Ordnung (Grid, Supply Chain)     |
| Trust Gating              | Trust-Schwellen als Zugangskriterium            |
| **Trust Inheritance**     | Anteilige Trust-Vererbung an Sub-Identities     |
| Trust Propagation Graph   | Netzwerk aller Trust-Beziehungen                |
| Trust Vector              | Mehrdimensionale Trust-Repräsentation           |
| TTL                       | Time-To-Live                                    |
| Unified Trust Model       | Ganzheitliches Bewertungssystem für alle        |
| Uninformed Search         | Systematische Suche ohne Heuristik              |
| **Verification Chain**    | Kryptografische Beweiskette für Sub-IDs         |
| Virtual Environment       | Abstrakte Sub-Umgebung unter Real World         |
| wallet_balance()          | Host API für Chain-Guthaben                     |
| wallet_transfer()         | Host API für Cross-Chain Transfer               |
| **Warning Escalation**    | Stufenweise Verschärfung bei wiederholten Warns |
| **Warning Stack**         | Akkumulierte Warnungen pro Mitglied             |
| WASM                      | WebAssembly Runtime für Agenten                 |
| XMTP                      | Verschlüsseltes Messaging-Protokoll             |

---

## 🏛️ Environment Governance

Begriffe rund um das Legislative/Executive-System für Umgebungen.

| Begriff                     | Definition                                                                                                                             |
| :-------------------------- | :------------------------------------------------------------------------------------------------------------------------------------- |
| **Legislative**             | Regelgebungsorgan einer Umgebung. Definiert Karma-Regeln, Schwellenwerte, Eskalationsstufen und Belohnungen.                           |
| **Executive**               | Durchsetzungsorgan einer Umgebung. Verwaltet Karmic Ledger, Warning Stack, Enforcement Actions und Governance Proposals.               |
| **Karmic Power**            | Akkumulierte Governance-Stärke eines Mitglieds. Berechnet aus positiven und negativen Karma-Events. Basis für Stimmgewicht.            |
| **Karmic Ledger**           | Verteiltes Konten-System aller Mitglieder einer Umgebung. Speichert Karma, Warnings, Tier und Governance-Power pro Mitglied.           |
| **Karma Tier**              | Stufe basierend auf akkumulierter Karmic Power: Gebannt, Suspendiert, Unter Beobachtung, Neuling, Etabliert, Vertrauenswürdig, etc.    |
| **Karma Asymmetry**         | Prinzip, dass negative Karma-Events stärker gewichtet werden als positive. Typischer Faktor: 1.5x.                                     |
| **Warning Stack**           | Akkumulierte Liste formeller Warnungen pro Mitglied. Warnungen haben Decay (Halbwertszeit) und lösen bei Schwellenwerten Aktionen aus. |
| **Network Warning**         | Formelle Warnung, die in den Warning Stack eines Mitglieds eingetragen wird. Kann von Guardians oder dem Governance Committee kommen.  |
| **Warning Escalation**      | Automatische Verschärfung der Konsequenzen bei steigender Warning-Anzahl: Notice → Restricted → Probation → Suspension → Ban.          |
| **Reputation Event**        | Protokolliertes Ereignis, das Karma verändert: karma_earned, karma_deducted, warning_issued, tier_upgrade/downgrade, etc.              |
| **Connected Objects Karma** | Karma-Beiträge, die von registrierten Assets (via Ownership-Anchor Sub-IDs) zum Eigentümer fließen. Performance-basiert.               |
| **Karma Flow**              | Mechanismus, durch den Asset-Performance (Uptime, Reviews, Complaints) in die Karmic Power des Eigentümers einfließt.                  |
| **Complaint System**        | Strukturiertes Beschwerde-Verfahren mit Typen, Validierung, Lifecycle und asymmetrischer Karma-Auswirkung.                             |
| **Power Delegation**        | Übertragung von Governance-Power (Stimmrecht) an ein anderes Mitglied. Max 50% delegierbar, zeitlich gebunden.                         |
| **Governance Proposal**     | Formeller Änderungsantrag an Legislative-Regeln. Erfordert Quorum und Approval-Threshold. Typen: minor, major, constitutional.         |
| **Entry Rights**            | Berechtigung, Einträge (Complaints, Attestations, Proposals) im Executive-System vorzunehmen. Für agent-assoziierte Identitäten.       |
| **Enforcement Engine**      | Automatisches System zur Durchsetzung von Konsequenzen bei Karma-Schwellen oder Warning-Counts.                                        |
| **Dispute Resolution**      | Schiedsverfahren für Streitfälle zwischen Mitgliedern. Panel aus Guardian-Tier-Mitgliedern, formaler Prozess, Karma-Adjustments.       |
| **Arbitration Panel**       | Dreiköpfiges Schiedsgericht aus zufällig gewählten Guardian-Tier-Mitgliedern ohne Verbindung zu den Parteien.                          |
| **Governance Committee**    | Gruppe von Architect-Tier-Mitgliedern mit besonderen Befugnissen (Legislative-Änderungen, Critical Enforcement).                       |
| **Voting Power**            | Effektives Stimmgewicht bei Governance-Abstimmungen. Berechnet als √(Karmic Power) für fairere Verteilung.                             |
| **Quorum**                  | Mindest-Anteil der Governance-Power, der abstimmen muss, damit eine Abstimmung gültig ist. Typisch: 15%.                               |
| **Approval Threshold**      | Mindest-Anteil der Ja-Stimmen für Annahme eines Proposals. Typisch: 67% (major), 75% (constitutional).                                 |

```

┌──────────────────────────────────────────────────────────────┐
│ │
│ ENVIRONMENT GOVERNANCE SYSTEM │
│ ══════════════════════════════ │
│ │
│ ┌───────────────────────────────────────────────────┐ │
│ │ LEGISLATIVE │ │
│ │ 📜 Karma Rules 📊 Tiers ⚖️ Escalation │ │
│ │ 🎁 Rewards 🔄 Amendment Process │ │
│ └───────────────────────────────────────────────────┘ │
│ │ │
│ definiert │
│ ▼ │
│ ┌───────────────────────────────────────────────────┐ │
│ │ EXECUTIVE │ │
│ │ 📒 Karmic Ledger ⚠️ Warning Stack │ │
│ │ 📋 Event Log 🤖 Enforcement │ │
│ │ 🗳️ Proposals 🤝 Delegation │ │
│ └───────────────────────────────────────────────────┘ │
│ │ │
│ beeinflusst │
│ ▼ │
│ ┌───────────────────────────────────────────────────┐ │
│ │ MEMBERS (Agents + ihre Connected Objects) │ │
│ │ 🏆 Guardians → ⭐ Trusted → ✅ Est. → 🌱 New │ │
│ │ ⚠️ Observed → 🚫 Suspended → ⛔ Banned │ │
│ └───────────────────────────────────────────────────┘ │
│ │
│ 💡 Asymmetrie: Beschwerden -15, Lob +5 │
│ 💡 Power = √Karma (fairere Verteilung) │
│ 💡 Assets bauen Owner-Karma auf (Connected Objects) │
│ │
└──────────────────────────────────────────────────────────────┘

```

---

## Weiterführende Dokumente

| Dokument                                                | Inhalt                             |
| :------------------------------------------------------ | :--------------------------------- |
| [Fachkonzept](./fachkonzept.md)                         | Vollständige Spezifikation         |
| [Kernkonzept](./kernkonzept.md)                         | High-Level-Überblick               |
| [ECL Spezifikation](./erynoa-configuration-language.md) | **Die einheitliche Systemsprache** |
| [Systemarchitektur](./system-architecture-overview.md)  | Technische Architektur             |
| [Search Environments](./search-environments.md)         | Hierarchische Suchordnungen        |
| [Liquides Datenmodell](./liquides-datenmodell.md)       | Blueprints, AMOs, Extensions       |
| [Trust & Reputation](./trust-and-reputation.md)         | Karmic Engine, Trust Vectors       |
| [Cybernetic Loop](./cybernetic-loop.md)                 | Der 6-Phasen-Workflow              |
| [Agents & ADL](./agents-and-adl.md)                     | Agentenmodell (ecl/agent)          |
| [Use Cases](./use-cases.md)                             | Praktische Anwendungsszenarien     |
```
