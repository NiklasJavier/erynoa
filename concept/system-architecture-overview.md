# Erynoa – System Architecture Overview

> **Zielgruppe:** Software-/Systemarchitekt:innen, Senior Developers, Protokoll-Designer
> **Kontext:** Übergang von Konzept zu technischer Architektur
> **Verwandte Dokumente:** [Kernkonzept](./kernkonzept.md), [Cybernetic Loop](./cybernetic-loop.md), [Glossar](./glossary.md)

---

## 1. Ziel dieses Dokuments

Dieses Dokument beschreibt die Systemarchitektur von Erynoa auf hohem Abstraktionsniveau.
Es bildet die Brücke zwischen dem **Kernkonzept** (`kernkonzept.md`) und den detaillierten
Architektur- und Implementierungsdokumenten unter `docs/`.

Fokus:

- klare Rollen der drei Sphären **ERY**, **ECHO**, **NOA**
- Aufteilung in **Layer 2 (Off-Chain)** und **Layer 0 (On-Chain)**
- Zusammenspiel im **kybernetischen Regelkreis (Cybernetic Loop)**

---

## 2. High-Level Architektur

Erynoa besteht aus drei spezialisierten Sphären, die gemeinsam eine kybernetische Triade bilden:

- **ERY – Semantic Lattice (Semantik & Gedächtnis)**
- **ECHO – Emergent Swarm (Intelligenz & Agenten)**
- **NOA – Causal Ledger (Wahrheit & Exekution)**

Diese Sphären verteilen sich auf zwei technologische Ebenen:

- **Layer 2 – Off-Chain Intelligence & Semantics**
  - ERY und ECHO
  - Hohe Rechenlast, flexible Entwicklung, keine globale Konsenspflicht

- **Layer 0 – Causal Ledger**
  - NOA
  - Deterministische Finalität, formale Sicherheit, minimale aber belastbare Zustände

Gedankliches Diagramm (vereinfacht):

- Nutzer / Maschinen / Unternehmen
  ↓ (Intents in ADL)
- **ECHO (Agenten, Verhandlung)**
  ↔ **ERY (Semantik, Trust, Index)**
  ↓ (finalisiertes Ergebnis)
- **NOA (Ledger, AMOs, MoveVM)**
  → Events → zurück zu **ERY** (Karmic Feedback)

---

## 3. ERY – Semantic Lattice

**Rolle:** Semantisches Rückgrat und Gedächtnis des Netzwerks.

ERY beantwortet die Fragen:

- _Was_ bedeutet ein Objekt oder Ereignis im Kontext von Normen und Domänen?
- _Wie vertrauenswürdig_ sind Akteure, Objekte und Prozesse im Zeitverlauf?

**Zentrale Komponenten:**

- **Erynoa Node (Verifiable Oracle)**
  - Rust-basierte Binary auf Tokio-Runtime.
  - Nimmt Events aus NOA entgegen, reichert sie mit Kontext an und signiert Ergebnisse (Ed25519).

- **Event Ingestor**
  - Konsumiert „Raw Events“ aus NOA in Echtzeit.
  - Normalisiert, filtert und routet Ereignisse an die Karmic Engine und den Semantic Index.

- **Karmic Engine**
  - Berechnet **Trust Vectors** für Akteure und Objekte.
  - Nutzt den **Ripple Effect**:
    - \( R*\text{new}(t) = R*\text{old}(t-1) + \eta (F\_\text{Event} - E[F]) \)
  - Implementiert **Trust Inheritance**:
    - Vertrauen propagiert entlang hierarchischer Strukturen (z. B. Hersteller → Betreiber → Asset).

- **Semantic Index (Qdrant-basiert)**
  - Vektorbasierte Wissensverwaltung für:
    - Normative Standards und Blueprints (Static Knowledge)
    - Trust Vectors, Attestations, Fluid Extensions (Dynamic State)
  - Skalierung über horizontales Sharding (DHT + Geohashing).

**Architekturprinzipien:**

- Trennung von **unveränderlichen Normen** und **dynamischen Zuständen**.
- Biologisch inspirierte **Synapsen-Architektur**:
  - Synapsen als inhaltsadressierte, kontextuelle Speichereinheiten (CIDs).
  - TTL-Mechanismen für flüchtige Daten (Fluid Persistence, Vermeidung von State Bloat).

---

## 4. ECHO – Emergent Swarm

**Rolle:** Operative Intelligenz und Durchführung von Intents.

ECHO beantwortet die Fragen:

- _Wer_ interagiert mit wem, um einen Intent zu erfüllen?
- _Unter welchen Bedingungen_ (Kosten, Normen, Vertrauen, Geografie) wird ein Deal geschlossen?

**Zentrale Konzepte:**

- **Agenten**
  - **Seeker Agents**: repräsentieren Nachfrager (Nutzer, Unternehmen, IoT-Geräte).
  - **Provider Agents**: repräsentieren Anbieter von Gütern und Services.
  - Beide laufen als **zustandsloser Code („Agent as Code“) in einer WASM-Sandbox**.

- **Agent Definition Language (ADL)**
  - Deklarative Beschreibung von Intents und Constraints:
    - Funktionale Anforderungen (z. B. Leistung, Qualität).
    - Normative Anforderungen (Blueprints, Standards).
    - Vertrauen (MinTrust).
    - Geografie (Geohashing-Regionen).

- **Netzwerk & Kommunikation**
  - P2P-Kommunikation über **libp2p**.
  - Discovery von Handelspartnern über DHT + Geohashing.
  - Verhandlung in verschlüsselten **XMTP Secure Tunnels** („Consensus Bubbles“):
    - Off-Chain, privat, mit **Progressive Disclosure** sensibler Daten.

**Architekturprinzipien:**

- **Ephemeral Intelligence**:
  - Agenten sind kurzlebig und zustandslos, Zustand liegt in ERY und NOA.
- **Privacy-by-Design**:
  - Geschäftsgeheimnisse bleiben Off-Chain; On-Chain landen nur minimale, notwendige Fakten.

---

## 5. NOA – Causal Ledger

**Rolle:** Ebene der Wahrheit und exekutiven Finalität.

NOA beantwortet die Fragen:

- _Was ist tatsächlich passiert?_ (kausale Historie)
- _Wem gehört was?_ (Zustand von Assets, Credentials und Services)

**Technologische Basis:**

- DAG-basierter Ledger auf **IOTA Rebased**.
- Konsensmechanismus: **Starfish BFT** (leaderless, deterministische Finalität, < 2 Sekunden).

**Ausführungsumgebung: MoveVM**

- Programmiersprache **Move** mit Fokus auf Resource Safety:
  - Assets können nicht dupliziert oder implizit gelöscht werden.
- **Logic Guards** als Smart Contracts:
  - Prüfen Invarianten vor jeder Zustandsänderung.
  - Erzwingen z. B. Soulbound-Eigenschaften, Besitzwechsel-Regeln, Domain-spezifische Policy.

**Datenmodell: Atomic Market Objects (AMOs)**

- Zentrale On-Chain-Entität in NOA.
- Verhält sich gemäß Blueprint-Definition in ERY.
- Archetypen:
  - **Material AMOs**: transferierbare Real World Assets (RWA, IoT).
  - **Credential AMOs**: Soulbound-Credentials, an eine DID gebunden.
  - **Service AMOs**: zeitgebundene Services mit Continuous Value Streaming.

---

## 6. Zusammenspiel: Der kybernetische Regelkreis

Die drei Sphären bilden gemeinsam einen geschlossenen **Cybernetic Loop**:
Intents werden in **ECHO** formuliert und verhandelt, durch **NOA** kausal finalisiert und fließen als Events zurück in **ERY**, wo die Karmic Engine Vertrauen und Kontext aktualisiert.
Eine ausführliche, phasenweise Beschreibung (inkl. Inputs/Outputs) findet sich in `cybernetic-loop.md`.

---

## 7. Abgrenzung zu klassischen Blockchain-Architekturen

Im Vergleich zu herkömmlichen Blockchains:

- trennt Erynoa konsequent **Semantik**, **Intelligenz** und **Exekution**,
- nutzt Off-Chain-Komponenten (ERY, ECHO) für komplexe Logik und Suche,
- reduziert On-Chain-Logik (NOA) auf formale, kausale Wahrheiten,
- macht **Vertrauen** zu einem erstklassigen Konzept (Trust Vectors, Karmic Engine),
- und behandelt reale Domänen über ein liquides, normbasiertes Datenmodell (Blueprints + AMOs).

Damit ist Erynoa keine „noch eine Blockchain“, sondern ein kybernetisches Protokoll für skalierbare, vertrauensbasierte Maschinenökonomien.

---

## 8. Fazit

Die Systemarchitektur von Erynoa trennt bewusst Semantik (ERY), Intelligenz (ECHO) und Exekution (NOA) in spezialisierte Sphären. Diese Trennung ermöglicht Skalierbarkeit ohne Kompromisse bei Sicherheit und Finalität.

---

**Weiterführende Dokumente:**

- [Cybernetic Loop](./cybernetic-loop.md) – Detaillierte Workflow-Beschreibung
- [Liquides Datenmodell](./liquides-datenmodell.md) – Blueprints und AMOs im Detail
- [Agents & ADL](./agents-and-adl.md) – Agentenmodell und Agent Definition Language
