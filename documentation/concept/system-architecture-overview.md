# Erynoa – System Architecture Overview

> **Zielgruppe:** Software-/Systemarchitekt:innen, Senior Developers, Protokoll-Designer
> **Lesezeit:** ca. 15 Minuten
> **Version:** ECL v2.1 – Identity-First + ECLVM
> **Voraussetzung:** [Kernkonzept](./kernkonzept.md) gelesen
> **Verwandte Dokumente:** [DACS Identity](./dacs-identity.md) · [Cybernetic Loop](./cybernetic-loop.md) · [ECL Spezifikation](./erynoa-configuration-language.md) · [Glossar](./glossary.md)

---

## Architektur auf einen Blick

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              LAYER 2 (Off-Chain)                            │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                           🔮 ERY                                    │    │
│  │               Semantic & Identity Lattice (inkl. DACS)             │    │
│  │                                                                     │    │
│  │   ┌───────────────────────────────────────────────────────────┐   │    │
│  │   │  🔐 DACS MODULE (Identity-First)                          │   │    │
│  │   │  BFT Consensus · DIDs · VCs · Sub-Identities (16 Typen)   │   │    │
│  │   │  IOTA (Primary) · ETH L2 · SOL (Secondary)                │   │    │
│  │   └───────────────────────────────────────────────────────────┘   │    │
│  │   ┌───────────────┐ ┌───────────────┐ ┌───────────────────────┐   │    │
│  │   │ 📚 Semantic   │ │ ⚖️ Karmic    │ │ 🌍 Discovery         │   │    │
│  │   │    Index      │ │    Engine    │ │    (DHT+Geo)          │   │    │
│  │   │ Qdrant-based  │ │ Karma Tiers  │ │ libp2p Kademlia       │   │    │
│  │   │               │ │ Asymmetrie   │ │                       │   │    │
│  │   └───────────────┘ └───────────────┘ └───────────────────────┘   │    │
│  │                                                                     │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                         🖥️ ECLVM (Layer 0.5)                        │    │
│  │                    Erynoa Configuration VM                          │    │
│  │                                                                     │    │
│  │   ┌──────────────┐  ┌──────────────┐  ┌─────────────────────────┐  │    │
│  │   │  Bytecode    │  │  Templates   │  │    Sandbox              │  │    │
│  │   │  (~100 ops)  │  │  (3 Typen)   │  │    Execution            │  │    │
│  │   └──────────────┘  └──────────────┘  └─────────────────────────┘  │    │
│  │                                                                     │    │
│  │   Hot-Code-Reload · Live-Patching · Resource Metering              │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐    │
│  │                           🤖 ECHO                                   │    │
│  │                        Emergent Swarm                               │    │
│  │                                                                     │    │
│  │   ┌──────────────┐  ┌──────────────┐  ┌─────────────────────────┐  │    │
│  │   │   Seeker     │  │  Provider    │  │    Consensus            │  │    │
│  │   │   Agents     │  │   Agents     │  │    Bubbles (XMTP)       │  │    │
│  │   └──────────────┘  └──────────────┘  └─────────────────────────┘  │    │
│  │                                                                     │    │
│  │   ECLVM Runtime · ADL · Multi-Chain Wallet · libp2p                │    │
│  └─────────────────────────────────────────────────────────────────────┘    │
│                                                                             │
│                  ERY Query ↕ │ │ ↕ DID Resolution (DACS)                   │
│                              ▼ ▼                                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       │ Events ↑↓ Transaktionen
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              LAYER 0 (On-Chain)                             │
│  ┌───────────────────────────────────────────────────────────────────┐      │
│  │                          ⚡ NOA                                    │      │
│  │                      Causal Ledger                                 │      │
│  │                                                                    │      │
│  │   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐   │      │
│  │   │  AMOs    │    │  Logic   │    │  MoveVM  │    │ Starfish │   │      │
│  │   │ (Assets) │    │  Guards  │    │          │    │   BFT    │   │      │
│  │   └──────────┘    └──────────┘    └──────────┘    └──────────┘   │      │
│  │                                                                    │      │
│  │   IOTA Rebased · DAG · < 2s Finalität                             │      │
│  └───────────────────────────────────────────────────────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Layer-Übersicht

| Layer         | Sphären    | Zweck                                          | Konsens       |
| ------------- | ---------- | ---------------------------------------------- | ------------- |
| **Layer 2**   | ERY + ECHO | Identität (DACS), Semantik, Denken, Verhandeln | PBFT für DACS |
| **Layer 0.5** | ECLVM      | Dynamische Konfiguration, Templates, Sandbox   | Keiner        |
| **Layer 0**   | NOA        | Finalisieren, Beweisen                         | Starfish BFT  |

**Designprinzip:** Identity-First, Execution mit ECLVM Off-Chain, Wahrheit On-Chain.

---

## 🔮 ERY – Semantic & Identity Lattice

> _Das Gedächtnis und die Identität des Netzwerks_

ERY ist das semantische Rückgrat von Erynoa und besteht aus **vier Modulen**:

| Modul            | Funktion                                   |
| ---------------- | ------------------------------------------ |
| 🔐 **DACS**      | Multi-Chain Identity: DIDs, VCs, Anchoring |
| 📚 **Semantic**  | Blueprints, Normen, Ontologien (Qdrant)    |
| ⚖️ **Karmic**    | Trust-Berechnung aus Events & Attestations |
| 🌍 **Discovery** | DHT + Geohashing für dezentrale Suche      |

---

## 🔐 DACS – Identity Module (Teil von ERY)

> _Die selbst-souveräne Identitätsschicht innerhalb ERY_

### Was DACS macht

| Frage                             | DACS liefert                               |
| --------------------------------- | ------------------------------------------ |
| Wer bin ich?                      | DID-Dokument, W3C-konform                  |
| Auf welchen Chains existiere ich? | Multi-Chain Anchors (IOTA, ETH, SOL)       |
| Welche Credentials habe ich?      | Verifiable Credentials, verifizierbar      |
| Welche Sub-Identities habe ich?   | 16 spezialisierte Typen (Capability-based) |
| Wer autorisiert Änderungen?       | BFT-Konsens der DACS Nodes                 |

### Identity-First Paradigma (v2.1)

Sub-Identities sind spezialisierte Identitäten für unterschiedliche Zwecke:

| Sub-Identity-Typ | Zweck                         | Capabilities                   |
| ---------------- | ----------------------------- | ------------------------------ |
| `Trading`        | Finanzielle Transaktionen     | Transfer, Receive, Stake       |
| `Voting`         | Abstimmungen in Environments  | Vote, Delegate, Propose        |
| `Recovery`       | Wiederherstellung bei Verlust | Recover, Reset (eingeschränkt) |
| `Social`         | Soziale Interaktionen         | Connect, Message, Endorse      |
| `Device`         | IoT-Geräte-Binding            | Sensor, Actuate, Report        |
| `Service`        | Service-spezifische Aktionen  | Provide, Consume, Subscribe    |
| ...              | 10 weitere Typen              | Siehe ECL Spezifikation        |

**Vorteile:**

- **Minimale Exposition:** Nur benötigte Capabilities werden offengelegt
- **Einzelne Revocation:** Kompromittierte Sub-IDs widerrufbar ohne Hauptidentität
- **Audit Trail:** Jede Sub-ID führt eigenes Event-Log

### Architektur

```
┌─────────────────────────────────────────────────────────────────┐
│                      DACS Node Network                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│   │  DACS Node  │◀──▶│  DACS Node  │◀──▶│  DACS Node  │        │
│   │   (BFT)     │    │   (BFT)     │    │   (BFT)     │        │
│   └──────┬──────┘    └──────┬──────┘    └──────┬──────┘        │
│          │                  │                  │                │
│          └──────────────────┼──────────────────┘                │
│                             │                                   │
│              ┌──────────────┼──────────────┐                   │
│              ▼              ▼              ▼                    │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│   │    IOTA      │  │   Ethereum   │  │    Solana    │         │
│   │  (Primary)   │  │    L2        │  │              │         │
│   │              │  │ (Secondary)  │  │ (Secondary)  │         │
│   │  Full DID    │  │  Hash Only   │  │  Hash Only   │         │
│   │  Document    │  │  + Timestamp │  │  + Timestamp │         │
│   └──────────────┘  └──────────────┘  └──────────────┘         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

| Komponente         | Technologie    | Funktion                                       |
| ------------------ | -------------- | ---------------------------------------------- |
| **DACS Node**      | Rust, libp2p   | Validatoren im BFT-Netzwerk                    |
| **BFT Consensus**  | PBFT/HotStuff  | Finalisierung von DID-Operationen              |
| **Threshold Sigs** | BLS t-of-n     | Kollektive Signaturen (67% Threshold)          |
| **Chain Adapters** | Multi-Chain    | IOTA, Ethereum L2, Solana Anbindung            |
| **DID Registry**   | Self-Anchoring | did:erynoa:dacs-registry verankert sich selbst |

### DID-Syntax

```
did:erynoa:<namespace>:<unique-identifier>
```

| Beispiel                              | Bedeutung                 |
| ------------------------------------- | ------------------------- |
| `did:erynoa:vehicle:vin-WVW123456789` | Fahrzeug mit VIN          |
| `did:erynoa:charger:loc-munich-001`   | Ladesäule in München      |
| `did:erynoa:org:erynoa-gmbh`          | Organisation              |
| `did:erynoa:agent:seeker-abc123`      | ECHO Agent                |
| `did:erynoa:dacs-registry`            | Self-Anchoring System-DID |

### Multi-Chain Anchoring Strategie

| Chain        | Rolle     | Speicherung       | Zweck                         |
| ------------ | --------- | ----------------- | ----------------------------- |
| **IOTA**     | Primary   | Full DID Document | Haupt-Identitätsspeicher      |
| **Ethereum** | Secondary | Hash + Timestamp  | Redundanz, Interoperabilität  |
| **Solana**   | Secondary | Hash + Timestamp  | Performance, Ecosystem-Access |

> 📖 **Detaillierte Spezifikation:** [DACS Identity](./dacs-identity.md)

---

## � ERY Semantic Module

> _Das Wissens-Modul innerhalb ERY_

### Was das Semantic-Modul macht

| Frage                          | Semantic liefert                 |
| ------------------------------ | -------------------------------- |
| Was bedeutet dieses Objekt?    | Blueprint-Referenz, Norm-Kontext |
| Wem kann ich vertrauen?        | Trust Vectors, Attestations      |
| Wo finde ich passende Partner? | Semantic Search, Geohashing      |

### ERY Komponenten (alle Module)

```
┌─────────────────────────────────────────────────────────────────┐
│                         ERY Node                                │
│                    (Verifiable Oracle)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│   │   Event     │───▶│   Karmic    │───▶│  Semantic   │        │
│   │  Ingestor   │    │   Engine    │    │   Index     │        │
│   └─────────────┘    └─────────────┘    └─────────────┘        │
│         ▲                   │                   │               │
│         │                   │                   │               │
│    Events von NOA      Trust Vectors      Vektor-Suche          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

| Komponente         | Technologie       | Funktion                                          |
| ------------------ | ----------------- | ------------------------------------------------- |
| **ERY Node**       | Rust, Tokio       | Verifiable Oracle – signiert Ergebnisse (Ed25519) |
| **Event Ingestor** | Stream Processing | Konsumiert NOA-Events in Echtzeit                 |
| **Karmic Engine**  | Custom            | Berechnet & propagiert Trust Vectors              |
| **Semantic Index** | Qdrant            | Vektorbasierte Wissens- & Trust-Suche             |

### Datenmodell

```
┌─────────────────────────────────────────────────────────────────┐
│                     Semantic Index                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Static Knowledge          │    Dynamic State                  │
│   ─────────────────         │    ─────────────                  │
│   • Blueprints              │    • Trust Vectors                │
│   • Normative Standards     │    • Attestations                 │
│   • Domain Ontologies       │    • Fluid Extensions (TTL)       │
│                             │                                   │
│   Immutable                 │    Mutable, TTL-gesteuert         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Karmic Engine – Trust-Berechnung

```
Event (z.B. erfolgreiche Lieferung)
           │
           ▼
┌──────────────────────────────────────┐
│  R_new = R_old + η(F_event - E[F])   │
│                                      │
│  η = Lernrate                        │
│  F_event = Event-Beitrag             │
│  E[F] = Erwartungswert               │
└──────────────────────────────────────┘
           │
           ▼
Trust Inheritance (fraktal)
           │
           ├── Hersteller (+0.8)
           ├── Betreiber  (+0.5)
           └── Asset      (+0.3)
```

### Skalierung

| Mechanismus              | Beschreibung                                       |
| ------------------------ | -------------------------------------------------- |
| **DHT**                  | Distributed Hash Table für dezentrale Datenhaltung |
| **Geohashing**           | Räumliche Partitionierung für lokale Queries       |
| **Synapsen-Architektur** | CID-adressierte Speichereinheiten mit TTL          |

---

## 🌍 Discovery Module – Hierarchische Suchumgebungen

> _Kontextbewusste Suche in beliebig verschachtelten Abstraktionsebenen_

### Konzept: Search Environments

Das Discovery-Modul in ERY unterstützt **hierarchische Suchumgebungen (Search Environments)**, die kontextabhängige Suchen mit unterschiedlichen Ordnungsprinzipien ermöglichen.

```
┌─────────────────────────────────────────────────────────────────┐
│                 ENVIRONMENT HIERARCHY                           │
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │                🌍 REALE WELT (ROOT)                     │  │
│   │         Physical Space · Geohashing · BFS/DFS           │  │
│   │                                                         │  │
│   │   ┌─────────────────────────────────────────────────┐  │  │
│   │   │ 🏭 INDUSTRY      🔋 ENERGY       🚗 MOBILITY    │  │  │
│   │   │ eCl@ss, ISO     Grid Codes      MaaS, GTFS     │  │  │
│   │   │                                                 │  │  │
│   │   │   ┌───────────┐ ┌───────────┐ ┌───────────┐    │  │  │
│   │   │   │ EV-Charg. │ │ Prosumer  │ │ Fleet Mgmt│    │  │  │
│   │   │   │ OCPP      │ │ Trading   │ │           │    │  │  │
│   │   │   └─────┬─────┘ └───────────┘ └───────────┘    │  │  │
│   │   │         │                                       │  │  │
│   │   │   ┌─────┴─────┐                                 │  │  │
│   │   │   │  Hubject  │  ...∞ weitere Ebenen           │  │  │
│   │   │   │  Girö-E   │                                 │  │  │
│   │   │   └───────────┘                                 │  │  │
│   │   └─────────────────────────────────────────────────┘  │  │
│   └─────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Environment-Typen

| Typ            | Symbol | Beschreibung                                      |
| -------------- | ------ | ------------------------------------------------- |
| **REAL**       | 🌍     | Root-Umgebung, physische Welt mit Geohashing      |
| **VIRTUAL**    | 🔮     | Abstraktionsebene mit eigenen Ordnungsprinzipien  |
| **DOMAIN**     | 🏭     | Standard-verknüpfte Fachdomäne (ISO, eCl@ss)      |
| **NETWORK**    | 🌐     | Netzwerk-basiert (z.B. Roaming-Verbund)           |
| **REGULATORY** | ⚖️     | Regulatorisch definiert (z.B. Eichrecht-Konform)  |
| **CUSTOM**     | 🎨     | Benutzerdefiniert für spezifische Anwendungsfälle |

### Suchstrategien

| Strategie             | Typ          | Beschreibung                                      |
| --------------------- | ------------ | ------------------------------------------------- |
| **BFS**               | Uninformiert | Breitensuche – systematisch alle Nachbarn         |
| **DFS**               | Uninformiert | Tiefensuche – verfolgt einen Pfad bis zum Ende    |
| **A\***               | Informiert   | Optimal mit Heuristik (z.B. geo_distance + trust) |
| **Greedy Best-First** | Informiert   | Schnell aber nicht optimal, folgt Heuristik       |
| **Beam Search**       | Informiert   | Begrenzte Parallelität für Effizienz              |

### Environment-spezifische Heuristiken

Jede Umgebung kann eigene Heuristiken definieren:

```
// Reale Welt: Geospatiale Heuristik
heuristic geo_distance(current, goal) {
  return haversine(current.geohash, goal.geohash)
}

// EV-Charging: Kombinierte Heuristik
heuristic ev_charging_score(candidate, intent) {
  0.3 * geo_distance(candidate, intent.location) +
  0.25 * normalize(candidate.price_kwh) +
  0.25 * candidate.trust_vector.reliability +
  0.2 * candidate.fluid_ext.available
}
```

### Cross-Environment Queries

Intents können mehrere Umgebungen referenzieren:

```yaml
intent:
  environments:
    primary: "env:erynoa:ev-charging:germany"
    intersect:
      - "env:erynoa:roaming:hubject"
      - "env:erynoa:energy:renewable"
    exclude:
      - "env:erynoa:operator:blacklisted"
    fallback:
      - "env:erynoa:real_world"
```

### Object Placement & Chain-Anchoring (v2.1)

Objekte haben einen definierten Platz in der Environment-Hierarchie:

```
┌─────────────────────────────────────────────────────────────────┐
│                 ERY PLACEMENT SERVICES                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ery_get_object_placement(did) → PlacementInfo                │
│   ────────────────────────────────────────────                  │
│   • current_environment   →  Aktuelle Umgebung des Objekts     │
│   • environment_hierarchy →  Pfad von ROOT bis current          │
│   • chain_branch          →  DLT für diese Umgebung            │
│   • anchored              →  Wurde auf Chain geankert?         │
│   • scoring_active        →  Ist Scoring aktiviert?            │
│                                                                 │
│   ery_get_environment_network(env_id) → NetworkInfo            │
│   ─────────────────────────────────────────────────             │
│   • chain_type    →  "iota" | "ethereum" | "solana" | ...      │
│   • network_id    →  Spezifisches Netzwerk (mainnet, testnet)  │
│   • anchoring_endpoint → URL für Anchoring-Transaktionen       │
│                                                                 │
│   ery_get_hierarchy_path(from_env, to_env) → [env_id, ...]     │
│   ─────────────────────────────────────────────────             │
│   • Navigiert zwischen Umgebungen in der Hierarchie            │
│   • Ermittelt Fallback-Kette bei Deaktivierung                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key Concepts:**

| Konzept           | Beschreibung                                           |
| ----------------- | ------------------------------------------------------ |
| **Default: ROOT** | Alle Objekte starten in `env:erynoa:real_world`        |
| **Chain-Branch**  | Jede virtuelle Umgebung definiert ihre DLT             |
| **Anchoring**     | Pflicht bevor Scoring in virtueller Umgebung aktiviert |
| **Fallback**      | Bei Deaktivierung: Abstieg zur Parent-Umgebung         |

**Lifecycle:**

```
CREATED → PLANNED → ANCHORED → ACTIVE
   │         │          │          │
   │         │          │          └── Scoring & Discovery aktiv
   │         │          └── Auf Environment-Chain geankert
   │         └── Für Umgebung geplant, Membership geprüft
   └── Default in ROOT, Basis-Scoring
```

> 📖 **Detaillierte Spezifikation:** [Search Environments](./search-environments.md#6-object-placement--chain-anchoring-v21)

---

## 🖥️ ECLVM – Erynoa Configuration Virtual Machine (Layer 0.5)

> _Die dynamische Execution Engine für ECL_

### Was ECLVM macht

| Frage                                | ECLVM liefert                              |
| ------------------------------------ | ------------------------------------------ |
| Wie führe ich ECL-Konfiguration aus? | Bytecode-Interpretation (~100 Opcodes)     |
| Wie erstelle ich Objekte dynamisch?  | Template-Instantiierung (3 Template-Typen) |
| Wie programmiere ich Agenten?        | Agent-Routinen mit Hot-Code-Reload         |
| Wie sichere ich die Ausführung?      | Sandboxed Execution mit Resource Limits    |

### Architektur

```
┌─────────────────────────────────────────────────────────────────┐
│                          ECLVM                                  │
│                    (Layer 0.5 Execution)                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│   │  Bytecode   │───▶│  Template   │───▶│  Sandbox    │        │
│   │  Compiler   │    │   Engine    │    │  Runtime    │        │
│   └─────────────┘    └─────────────┘    └─────────────┘        │
│         │                   │                   │               │
│    ~100 Opcodes       3 Template-Typen    Resource Limits       │
│    Stack-based        Blueprint, Env,     CPU, Memory, I/O      │
│                       Agent Templates                           │
│                                                                 │
│   ┌───────────────────────────────────────────────────────┐    │
│   │              Hot-Code-Reload & Live-Patching           │    │
│   │     Funktionen live aktualisieren ohne Neustart        │    │
│   └───────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Opcode-Kategorien

| Kategorie      | Opcodes | Beschreibung                       |
| -------------- | ------- | ---------------------------------- |
| **Stack**      | ~15     | PUSH, POP, DUP, SWAP, ROT          |
| **Arithmetik** | ~10     | ADD, SUB, MUL, DIV, MOD, NEG       |
| **Vergleich**  | ~10     | EQ, NE, LT, GT, LE, GE             |
| **Kontrolle**  | ~15     | JMP, JZ, JNZ, CALL, RET, LOOP      |
| **Objekte**    | ~20     | LOAD, STORE, CREATE, BIND, RESOLVE |
| **Trust**      | ~10     | CHECK_TRUST, GET_KARMA, ATTEST     |
| **System**     | ~20     | EMIT_EVENT, CALL_EXTERNAL, SANDBOX |

### Template-System

| Template-Typ    | Zweck                                     | Beispiel                           |
| --------------- | ----------------------------------------- | ---------------------------------- |
| **Blueprint**   | Objekt-Schemata mit Parametern            | `ChargingService<power_kw, price>` |
| **Environment** | Vorkonfigurierte Umgebungen               | `EVChargingEnv<region, standards>` |
| **Agent**       | Wiederverwendbare Agenten-Konfigurationen | `SeekerAgent<intent_type, budget>` |

### Sandbox & Resource Limits

```
sandbox_limits {
  cpu_cycles:     1_000_000       // Max CPU-Zyklen pro Aufruf
  memory_bytes:   10_485_760      // 10 MB RAM-Limit
  io_operations:  100             // Max I/O-Operationen
  network_calls:  10              // Max externe Aufrufe
}
```

> 📖 **Detaillierte Spezifikation:** [ECL Spezifikation – VM Module](./erynoa-configuration-language.md#4-vm-module-eclvm)

---

## 🤖 ECHO – Emergent Swarm

> _Die operative Intelligenz_

### Was ECHO macht

| Frage                       | ECHO liefert                    |
| --------------------------- | ------------------------------- |
| Wer braucht was?            | Intent-Parsing via ADL          |
| Wer kann liefern?           | Discovery via ERY               |
| Zu welchen Konditionen?     | Private Verhandlung             |
| Auf welcher Chain zahlen?   | Network Selection (Multi-Chain) |
| Wie viel Guthaben habe ich? | Multi-Chain Wallet Abfrage      |

### Agentenmodell

```
┌─────────────────────────────────────────────────────────────────┐
│                         ECHO Sphäre                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Seeker Agent                    Provider Agent                │
│   ─────────────                   ──────────────                │
│                                                                 │
│   ┌─────────────┐                 ┌─────────────┐              │
│   │   Intent    │                 │   Offer     │              │
│   │   (ADL)     │                 │   (ADL)     │              │
│   └──────┬──────┘                 └──────┬──────┘              │
│          │                               │                      │
│   ┌──────┴──────┐                 ┌──────┴──────┐              │
│   │Multi-Chain  │                 │Multi-Chain  │              │
│   │  Wallet     │                 │  Wallet     │              │
│   │ IOTA│ETH│SOL│                 │ IOTA│ETH│SOL│              │
│   └──────┬──────┘                 └──────┬──────┘              │
│          │                               │                      │
│          │      ┌───────────────┐        │                      │
│          └─────▶│  Consensus    │◀───────┘                      │
│                 │    Bubble     │                               │
│                 │   (XMTP)      │                               │
│                 └───────┬───────┘                               │
│                         │                                       │
│                         ▼                                       │
│          ┌──────────────────────────────┐                       │
│          │  Network Selection Engine    │                       │
│          │  • Counterparty-Chains       │                       │
│          │  • Gebührenvergleich         │                       │
│          │  • Latenz/Finalität          │                       │
│          │  • Cross-Chain Bridge        │                       │
│          └──────────────┬───────────────┘                       │
│                         │                                       │
│                         ▼                                       │
│                 Verhandlungsergebnis                            │
│                         │                                       │
│                         ▼                                       │
│               NOA Transaktion (Multi-Chain)                     │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

| Agentenrolle | Repräsentiert | Beispiele                                     |
| ------------ | ------------- | --------------------------------------------- |
| **Seeker**   | Nachfrage     | Fahrzeug sucht Ladesäule, Firma sucht Wartung |
| **Provider** | Angebot       | Ladesäulen-Betreiber, Wartungsdienstleister   |

### Multi-Chain Wallet & Network Selection

Jeder Agent verwaltet Guthaben auf mehreren Blockchains gleichzeitig und wählt automatisch das optimale Netzwerk:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                     MULTI-CHAIN WALLET ENGINE                            │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   Agent Wallet (verknüpft mit did:erynoa)                                │
│   ══════════════════════════════════════                                 │
│                                                                          │
│   ┌────────────────┐  ┌────────────────┐  ┌────────────────┐            │
│   │   IOTA Wallet  │  │  ETH L2 Wallet │  │  Solana Wallet │            │
│   │   (Priority 1) │  │   (Priority 2) │  │   (Priority 3) │            │
│   │   ──────────── │  │   ──────────── │  │   ──────────── │            │
│   │   1500 IOTA    │  │   0.5 ETH      │  │   25 SOL       │            │
│   │    500 ERY     │  │   200 USDC     │  │   100 USDC     │            │
│   └───────┬────────┘  └───────┬────────┘  └───────┬────────┘            │
│           │                   │                   │                      │
│           └───────────────────┼───────────────────┘                      │
│                               ▼                                          │
│   ┌────────────────────────────────────────────────────────────────┐    │
│   │               NETWORK SELECTION ENGINE                         │    │
│   │                                                                │    │
│   │   1. Counterparty-Analyse:                                     │    │
│   │      → Auf welchen Chains hat der Partner Guthaben?            │    │
│   │                                                                │    │
│   │   2. Gemeinsame Chains identifizieren:                         │    │
│   │      Seeker: [IOTA, ETH, SOL]  ∩  Provider: [IOTA, ETH]        │    │
│   │      = Gemeinsam: [IOTA, ETH]                                  │    │
│   │                                                                │    │
│   │   3. Kosten-Nutzen-Analyse:                                    │    │
│   │      • IOTA: 0.001€ Gebühr, 2s Finalität                       │    │
│   │      • ETH:  0.50€ Gebühr, 12s Finalität                       │    │
│   │                                                                │    │
│   │   4. Entscheidung: IOTA (günstigste gemeinsame Chain)          │    │
│   │                                                                │    │
│   └────────────────────────────────────────────────────────────────┘    │
│                                                                          │
│   Falls keine gemeinsame Chain: Cross-Chain Bridge via DACS             │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

| Host API            | Funktion                                        |
| ------------------- | ----------------------------------------------- |
| `wallet_balance()`  | Guthaben auf einer Chain abfragen               |
| `wallet_transfer()` | Transfer initiieren (inkl. optionalem Bridge)   |
| `network_select()`  | Optimale Chain ermitteln                        |
| `network_fees()`    | Aktuelle Gebühren aller Chains                  |
| `dacs_resolve()`    | DID auflösen (inkl. Wallet-Chains des Partners) |

### Agent Definition Language (ADL)

```yaml
# Beispiel: Intent eines Seeker-Agenten
intent:
  type: "ev-charging"
  constraints:
    functional:
      power_min: 50kW
      energy_source: renewable
    geographic:
      geohash: "u0v9*" # München-Region
      radius: 5km
    trust:
      min_trust: 0.8
      required_attestations:
        - type: "energy-certificate"
        - type: "operator-license"
    economic:
      max_price: 0.40 EUR/kWh
      payment: streaming
```

### Technologie-Stack

| Komponente      | Technologie  | Funktion                                      |
| --------------- | ------------ | --------------------------------------------- |
| **Runtime**     | WASM Sandbox | Isolierte, sichere Agentenausführung          |
| **Netzwerk**    | libp2p       | P2P-Kommunikation, Discovery                  |
| **Verhandlung** | XMTP         | E2E-verschlüsselte Secure Tunnels             |
| **State**       | Stateless    | Zustand liegt in ERY (Trust) und NOA (Assets) |

### Sicherheitsmodell

```
┌─────────────────────────────────────────────────────────────────┐
│                    WASM Sandbox                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Agent Code                                                    │
│       │                                                         │
│       ▼                                                         │
│   Host APIs (kontrolliert)                                      │
│       │                                                         │
│       ├── ERY Query API (read-only)                             │
│       ├── libp2p Network API (rate-limited)                     │
│       └── NOA Transaction API (Trust-gated)                     │
│                                                                 │
│   Kein direkter System-Zugriff                                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## ⚡ NOA – Causal Ledger

> _Die Quelle der Wahrheit_

### Was NOA macht

| Frage             | NOA liefert                       |
| ----------------- | --------------------------------- |
| Was ist passiert? | Kausale, unveränderliche Historie |
| Wem gehört was?   | Aktueller Zustand aller Assets    |
| Ist das erlaubt?  | Logic Guards prüfen Invarianten   |

### Technologische Basis

```
┌─────────────────────────────────────────────────────────────────┐
│                         NOA Ledger                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌───────────────────────────────────────────────────────┐    │
│   │                    IOTA Rebased                        │    │
│   │                    (DAG-Struktur)                      │    │
│   └───────────────────────────────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌───────────────────────────────────────────────────────┐    │
│   │                   Starfish BFT                         │    │
│   │         (Leaderless, < 2 Sekunden Finalität)          │    │
│   └───────────────────────────────────────────────────────┘    │
│                            │                                    │
│                            ▼                                    │
│   ┌───────────────────────────────────────────────────────┐    │
│   │                      MoveVM                            │    │
│   │              (Resource Safety, Logic Guards)           │    │
│   └───────────────────────────────────────────────────────┘    │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

| Komponente       | Beschreibung                                      |
| ---------------- | ------------------------------------------------- |
| **IOTA Rebased** | DAG-basierter Ledger, keine klassische Blockchain |
| **Starfish BFT** | Leaderloser Konsens, deterministische Finalität   |
| **MoveVM**       | Sichere Ausführung mit Resource Safety            |

### Atomic Market Objects (AMOs)

```
┌─────────────────────────────────────────────────────────────────┐
│                           AMO                                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────┐                                               │
│   │  Blueprint  │ ◀── Referenz zu ERY (Validierungsregeln)      │
│   │  Reference  │                                               │
│   └─────────────┘                                               │
│                                                                 │
│   ┌─────────────┐                                               │
│   │    State    │ ◀── Aktueller Zustand (Owner, Werte, etc.)    │
│   └─────────────┘                                               │
│                                                                 │
│   ┌─────────────┐                                               │
│   │   Logic     │ ◀── Invarianten (via Logic Guards)            │
│   │   Guards    │                                               │
│   └─────────────┘                                               │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

| AMO-Typ           | Transfer | Bindung         | Beispiel               |
| ----------------- | -------- | --------------- | ---------------------- |
| 🏭 **Material**   | ✅ Ja    | Asset           | Ladesäule, Sensor      |
| 🎫 **Credential** | ❌ Nein  | DID (Soulbound) | KYC, Zertifikat        |
| ⏱️ **Service**    | ❌ Nein  | Zeit (TTL)      | Ladevorgang, Streaming |

### Logic Guards

```move
// Beispiel: Logic Guard für Soulbound Credential
module credential {
    struct Credential has key {
        id: ID,
        owner: address,
        attestations: vector<Attestation>,
    }

    // Transfer ist nicht erlaubt
    public fun transfer(_cred: Credential, _new_owner: address) {
        abort(ERR_SOULBOUND) // Immer fehlschlagen
    }

    // Nur Verifizierung ist möglich
    public fun verify(cred: &Credential, claim: Claim): bool {
        // Prüfe Attestations gegen Claim
    }
}
```

---

## Zusammenspiel: Der kybernetische Regelkreis

```
┌────────────────────────────────────────────────────────────────────────────┐
│                                                                            │
│    1. INTENT                    2. DISCOVERY                               │
│    ┌──────────┐                 ┌──────────┐                               │
│    │  Seeker  │────────────────▶│   ERY    │                               │
│    │  (ECHO)  │ ADL Query       │  Index   │                               │
│    └──────────┘                 └────┬─────┘                               │
│                                      │                                     │
│                                      │ Kandidaten + Trust Vectors          │
│                                      ▼                                     │
│    3. TRUST-GATING              4. VERHANDLUNG                             │
│    ┌──────────┐                 ┌──────────┐                               │
│    │  Karmic  │────────────────▶│Consensus │                               │
│    │  Engine  │ Filter          │  Bubble  │                               │
│    └──────────┘                 └────┬─────┘                               │
│                                      │                                     │
│                                      │ Verhandlungsergebnis                │
│                                      ▼                                     │
│    6. FEEDBACK                  5. EXEKUTION                               │
│    ┌──────────┐                 ┌──────────┐                               │
│    │   ERY    │◀────────────────│   NOA    │                               │
│    │  Update  │ Events          │  Ledger  │                               │
│    └──────────┘                 └──────────┘                               │
│         │                                                                  │
│         │                                                                  │
│         └─────────── beeinflusst nächste Discovery ────────────────────────│
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

| Phase           | Ort          | Input        | Output              |
| --------------- | ------------ | ------------ | ------------------- |
| 1. Intent       | ECHO         | Nutzerbedarf | ADL-Spezifikation   |
| 2. Discovery    | ERY          | ADL-Query    | Kandidatenliste     |
| 3. Trust-Gating | ERY (Karmic) | Kandidaten   | Gefilterte Liste    |
| 4. Verhandlung  | ECHO         | Partner      | Vertragsbedingungen |
| 5. Exekution    | NOA          | Transaktion  | Finalisierter State |
| 6. Feedback     | ERY (Karmic) | Events       | Trust-Update        |

---

## Vergleich: Erynoa vs. klassische Blockchain

| Aspekt          | Klassische Blockchain    | Erynoa                      |
| --------------- | ------------------------ | --------------------------- |
| **Architektur** | Alles auf einer Ebene    | Drei spezialisierte Sphären |
| **Semantik**    | Nicht vorhanden          | ERY: Blueprints, Ontologien |
| **Intelligenz** | On-Chain Smart Contracts | ECHO: Off-Chain Agenten     |
| **Konsens**     | Für alles                | Nur für Zustandsänderungen  |
| **Vertrauen**   | Implizit (oder nicht)    | Explizit: Trust Vectors     |
| **Skalierung**  | Schwierig                | Off-Chain Entlastung        |
| **Privacy**     | Alles öffentlich         | Progressive Disclosure      |

---

## Deployment-Perspektive

```
┌─────────────────────────────────────────────────────────────────┐
│                     Production Deployment                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │                    ERY Cluster                           │  │
│   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │  │
│   │  │  Node   │  │  Node   │  │  Node   │  │  Node   │    │  │
│   │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │  │
│   │                    Qdrant Cluster                        │  │
│   └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │                   ECHO Network                           │  │
│   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │  │
│   │  │ Agent   │  │ Agent   │  │ Agent   │  │ Agent   │    │  │
│   │  │ Runtime │  │ Runtime │  │ Runtime │  │ Runtime │    │  │
│   │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │  │
│   │                    libp2p Mesh                           │  │
│   └─────────────────────────────────────────────────────────┘  │
│                                                                 │
│   ┌─────────────────────────────────────────────────────────┐  │
│   │                    NOA Network                           │  │
│   │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │  │
│   │  │Validator│  │Validator│  │Validator│  │Validator│    │  │
│   │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │  │
│   │                  IOTA Rebased Network                    │  │
│   └─────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Zusammenfassung

| Sphäre      | Rolle                                | Technologie                   | Skalierung            |
| ----------- | ------------------------------------ | ----------------------------- | --------------------- |
| 🔮 **ERY**  | Identität (DACS), Wissen & Vertrauen | Qdrant, DHT, BFT, Multi-Chain | Horizontal (Sharding) |
| 🤖 **ECHO** | Intelligenz & Verhandlung            | WASM, libp2p                  | Horizontal (Agents)   |
| ⚡ **NOA**  | Wahrheit & Finalität                 | MoveVM, Starfish              | Durch Entlastung      |

**Das Designprinzip:** Jede Sphäre macht genau das, was sie am besten kann – nicht mehr, nicht weniger. ERY vereint Identität und Semantik, ECHO koordiniert Agenten, NOA finalisiert Transaktionen. Zusammen bilden sie einen lernenden, kybernetischen Organismus.

---

## Weiterführend

| Dokument                                                   | Fokus                              |
| ---------------------------------------------------------- | ---------------------------------- |
| [Cybernetic Loop](./cybernetic-loop.md)                    | Workflow im Detail (6 Phasen)      |
| [Liquides Datenmodell](./liquides-datenmodell.md)          | Blueprints, AMOs, Fluid Extensions |
| [Trust & Reputation](./trust-and-reputation.md)            | Karmic Engine, Trust Vectors       |
| [Agents & ADL](./agents-and-adl.md)                        | Agentenmodell, ADL-Syntax          |
| [Backend-Architektur](../system/reference/architecture.md) | Implementierungsdetails            |
