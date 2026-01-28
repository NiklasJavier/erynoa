# 📖 APPENDIX – Glossar

> **Typ:** Referenz
> **Zweck:** Begriffsdefinitionen

---

## Kernbegriffe

### AMO (Atomic Managed Object)

Universelle Repräsentation von Assets, Services und Credentials. Jedes handelbare oder referenzierbare "Ding" ist ein AMO mit eigener DID.

### Blueprint

Anwendungsspezifische Schablone, die definiert, wie ein AMO strukturiert sein soll. Basiert auf normativen Standards.

### DACS (Decentralized Anchor Control System)

Multi-Chain-System zur DID-Verankerung. Ermöglicht Self-Anchoring nach progressiver Dezentralisierung.

### DID (Decentralized Identifier)

Eindeutige, dezentrale Identifikation im Format `did:erynoa:<namespace>:<id>`. Grundlage für Identity-First.

### ECLVM (Erynoa Configuration Language Virtual Machine)

Layer-0.5-Runtime zur Ausführung von ECL-Code. Interpretiert Policies, Constraints und Logic Guards.

### Environment

Abgegrenzte Kontextblase mit spezifischen Regeln, Standards und Governance. "Spielfeld" für Agenten.

---

## Kybernetische Triade

### ERY (Semantic & Identity Lattice)

Erste Komponente der Triade. Verantwortlich für:

- Identität (DIDs, Credentials)
- Semantik (Blueprints, Standards, Ontologie)
- Vertrauen (Trust Vectors, Karma Engine)

### ECHO (Emergent Swarm)

Zweite Komponente der Triade. Verantwortlich für:

- Agenten (Seeker, Provider, Broker)
- Intents und Policies
- Verhandlung und Matching

### NOA (Causal Ledger)

Dritte Komponente der Triade. Verantwortlich für:

- Events und Transaktionen
- AMO-Lifecycle
- Finality und Anchoring

---

## Agent-Begriffe

### Intent

Formalisierte Absichtserklärung eines Agenten: "Ich möchte X unter Bedingungen Y."

### Policy

Entscheidungsregeln eines Agenten. Definiert Autonomie-Grenzen für automatische Entscheidungen.

### Negotiation

Prozess von Intent zu Agreement. Unterstützt Direct, Auction und Multi-Round-Modelle.

### Wallet

Vermögensspeicher eines Agenten. Verwaltet Guthaben, Zahlungsmethoden und Budget-Limits.

---

## Trust-Begriffe

### Trust Vector

Mehrdimensionaler Vertrauenswert mit vier Dimensionen: Reliability, Integrity, Capability, Reputation.

### Karma Tier

Gestaffelte Vertrauensstufen: Newcomer → Established → Veteran → Elder. Basiert auf akkumuliertem Karma.

### Attestation

Signierte Aussage über ein Subjekt von vertrauenswürdigen Dritten. Erhöht Trust.

---

## Ledger-Begriffe

### Event

Kausales Ereignis auf NOA. Zeigt auf seine Ursachen (DAG-Struktur).

### Logic Guard

Deterministisches Programm zur Validierung von AMO-Transitionen. Läuft in ECLVM.

### Finality

Zustand der Unveränderlichkeit. Erreicht durch Multi-Chain-Anchoring.

### Streaming

Kontinuierlicher Werttransfer während laufender Dienste.

---

## Netzwerk-Begriffe

### Anchor

Merkle-Root von Events, die auf externe Chains (IOTA, Ethereum) geschrieben wird.

### Bridge

Kommunikationsbrücke zwischen Erynoa und externen Systemen.

### Node

Netzwerk-Teilnehmer. Typen: Full Node, Light Node, Edge Node, Validator.

---

## ECL-Module

| Modul            | Funktion                           |
| ---------------- | ---------------------------------- |
| `ecl/core`       | Basistypen, Operatoren, Funktionen |
| `ecl/identity`   | DID, Credential, Delegation        |
| `ecl/trust`      | Trust Vector, Karma, Gating        |
| `ecl/agent`      | Policy, Intent, Negotiation        |
| `ecl/object`     | Blueprint, Constraint, Transition  |
| `ecl/environ`    | Environment, Governance            |
| `ecl/economic`   | Wallet, Streaming, Settlement      |
| `ecl/network`    | Routing, Gossip, DHT               |
| `ecl/governance` | Proposals, Voting, Council         |
| `ecl/test`       | Testing, Simulation                |

---

## Abkürzungen

| Kürzel | Bedeutung                           |
| ------ | ----------------------------------- |
| AMO    | Atomic Managed Object               |
| DACS   | Decentralized Anchor Control System |
| DID    | Decentralized Identifier            |
| ECL    | Erynoa Configuration Language       |
| ECLVM  | ECL Virtual Machine                 |
| VC     | Verifiable Credential               |
| DAG    | Directed Acyclic Graph              |
| DHT    | Distributed Hash Table              |
| CID    | Content Identifier                  |
