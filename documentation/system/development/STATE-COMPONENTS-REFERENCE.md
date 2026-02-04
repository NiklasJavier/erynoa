# StateComponent & StateGraph – Detaillierte Referenz

> **Quelldateien:**
> - `domain/unified/component.rs` (643 Zeilen)
> - `core/state.rs` (StateGraph: Zeilen 4080-4450)
> **Letzte Analyse:** 2026-02-04

---

## 1. StateComponent Übersicht

Die `StateComponent` Enum definiert **37 Komponenten** in **8 Layern**. Jede Komponente repräsentiert einen isolierten Teil des System-States.

### 1.1 Component-Hierarchie

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ENGINE-LAYER (6)                                   │
│  ┌─────┐ ┌───────────┐ ┌─────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐ │
│  │ UI  │ │ DataLogic │ │ API │ │ Governance │ │ Controller │ │ BPComposer │ │
│  └─────┘ └───────────┘ └─────┘ └────────────┘ └────────────┘ └────────────┘ │
├─────────────────────────────────────────────────────────────────────────────┤
│                            PEER-LAYER (6)                                    │
│  ┌─────────┐ ┌─────────────┐ ┌──────────────┐ ┌───────┐ ┌──────┐ ┌─────────┐│
│  │ Gateway │ │ SagaComposer│ │ IntentParser │ │ Realm │ │ Room │ │Partition││
│  └─────────┘ └─────────────┘ └──────────────┘ └───────┘ └──────┘ └─────────┘│
├─────────────────────────────────────────────────────────────────────────────┤
│                            P2P-LAYER (6)                                     │
│  ┌───────┐ ┌────────┐ ┌──────────┐ ┌───────┐ ┌───────────────┐ ┌─────────┐  │
│  │ Swarm │ │ Gossip │ │ Kademlia │ │ Relay │ │ NatTraversal  │ │ Privacy │  │
│  └───────┘ └────────┘ └──────────┘ └───────┘ └───────────────┘ └─────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│                         STORAGE-LAYER (4)                                    │
│  ┌─────────┐ ┌────────────┐ ┌─────────┐ ┌───────────┐                       │
│  │ KvStore │ │ EventStore │ │ Archive │ │ Blueprint │                       │
│  └─────────┘ └────────────┘ └─────────┘ └───────────┘                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                       PROTECTION-LAYER (5)                                   │
│  ┌─────────┐ ┌───────────┐ ┌───────────┐ ┌───────────────────┐ ┌───────────┐│
│  │ Anomaly │ │ Diversity │ │ Quadratic │ │ AntiCalcification │ │Calibration││
│  └─────────┘ └───────────┘ └───────────┘ └───────────────────┘ └───────────┘│
├─────────────────────────────────────────────────────────────────────────────┤
│                       EXECUTION-LAYER (6)                                    │
│  ┌─────┐ ┌──────┐ ┌───────────┐ ┌───────┐ ┌───────────┐ ┌─────────────┐     │
│  │ Gas │ │ Mana │ │ Execution │ │ ECLVM │ │ ECLPolicy │ │ ECLBlueprint│     │
│  └─────┘ └──────┘ └───────────┘ └───────┘ └───────────┘ └─────────────┘     │
├─────────────────────────────────────────────────────────────────────────────┤
│                          CORE-LAYER (4)                                      │
│  ┌───────┐ ┌───────┐ ┌──────────────┐ ┌───────────┐                         │
│  │ Trust │ │ Event │ │ WorldFormula │ │ Consensus │                         │
│  └───────┘ └───────┘ └──────────────┘ └───────────┘                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                        IDENTITY-LAYER (3)                                    │
│  ┌──────────┐ ┌────────────┐ ┌───────────────┐                              │
│  │ Identity │ │ Credential │ │ KeyManagement │                              │
│  └──────────┘ └────────────┘ └───────────────┘                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Alle 37 StateComponents

### 2.1 Identity-Layer (Κ6-Κ8)

| Component | Axiom | Beschreibung | Kritisch? |
|-----------|-------|--------------|-----------|
| `Identity` | Κ6-Κ8 | DID-Management, Root-DIDs, Sub-DIDs | ✅ |
| `Credential` | - | Verifiable Credentials, Attestations | ❌ |
| `KeyManagement` | - | Key-Rotation, Recovery, Hardware-Security | ❌ |

### 2.2 Core-Layer

| Component | Axiom | Beschreibung | Kritisch? |
|-----------|-------|--------------|-----------|
| `Trust` | Κ2-Κ5 | Trust-Vektoren, Reputation | ✅ |
| `Event` | Κ9-Κ12 | Kausale Events, DAG | ❌ |
| `WorldFormula` | Κ15a-d | Berechnungen nach Weltformel | ❌ |
| `Consensus` | Κ18 | BFT-Konsens, Finalisierung | ✅ |

### 2.3 Execution-Layer

| Component | Axiom | Beschreibung | Kritisch? |
|-----------|-------|--------------|-----------|
| `Gas` | - | Multi-Layer Gas-Tracking | ❌ |
| `Mana` | - | Regeneratives Resource-Budget | ❌ |
| `Execution` | - | ExecutionContext-Management | ❌ |
| `ECLVM` | - | Cost-limited Execution Environment | ❌ |
| `ECLPolicy` | - | Rules, Crossing-Policies | ❌ |
| `ECLBlueprint` | - | Templates, Instantiation | ❌ |

### 2.4 Protection-Layer (Κ19-Κ21)

| Component | Axiom | Beschreibung | Kritisch? |
|-----------|-------|--------------|-----------|
| `Anomaly` | Κ26-Κ28 | Verhaltensanalyse, Outlier-Detection | ✅ |
| `Diversity` | Κ19 | Gini-Koeffizient, Dezentralisierung | ❌ |
| `Quadratic` | Κ21 | Quadratic Voting/Funding | ❌ |
| `AntiCalcification` | Κ21 | Aktivitäts-Decay, Rotation | ❌ |
| `Calibration` | - | Parameter-Tuning, Self-Healing | ❌ |

### 2.5 Storage-Layer

| Component | Axiom | Beschreibung | Kritisch? |
|-----------|-------|--------------|-----------|
| `KvStore` | - | Persistenter Key-Value Store | ❌ |
| `EventStore` | - | Event-Sourcing Backend | ❌ |
| `Archive` | - | Langzeit-Archivierung | ❌ |
| `Blueprint` | - | Blueprint-Templates und Instanzen | ❌ |

### 2.6 Peer-Layer (Κ22-Κ24)

| Component | Axiom | Beschreibung | Kritisch? |
|-----------|-------|--------------|-----------|
| `Gateway` | Κ23 | Realm-Crossing-Koordination | ✅ |
| `SagaComposer` | Κ24 | Multi-Step-Transaction-Orchestrierung | ❌ |
| `IntentParser` | Κ22 | Intent-zu-Saga-Transformation | ❌ |
| `Realm` | Κ1 | Realm-Isolation und per-Realm State | ❌ |
| `Room` | Κ22 | Sub-Realm-Isolation mit Controller-Scope | ❌ |
| `Partition` | - | Trust-basierte Berechtigungspartition | ❌ |

### 2.7 P2P-Layer

| Component | Axiom | Beschreibung | Kritisch? |
|-----------|-------|--------------|-----------|
| `Swarm` | - | libp2p Swarm-Management | ❌ |
| `Gossip` | - | GossipSub Protokoll | ❌ |
| `Kademlia` | - | DHT für Peer-Discovery | ❌ |
| `Relay` | - | Circuit-Relay für NAT-Traversal | ❌ |
| `NatTraversal` | - | NAT-Hole-Punching | ❌ |
| `Privacy` | Κ25 | Onion-Routing, Cover-Traffic | ❌ |

### 2.8 Engine-Layer

| Component | Axiom | Beschreibung | Kritisch? |
|-----------|-------|--------------|-----------|
| `UI` | Κ22 | Deklaratives, Trust-basiertes Interface-Rendering | ❌ |
| `DataLogic` | Κ9-Κ12 | Reaktive Event-Verarbeitung und Aggregation | ❌ |
| `API` | Κ23 | Dynamische REST-API-Definition per ECL | ❌ |
| `Governance` | Κ19, Κ21 | DAO-Prinzipien und Abstimmungsmechanismen | ❌ |
| `Controller` | Κ5 | Berechtigungsverwaltung mit Delegation | ❌ |
| `BlueprintComposer` | - | Template-Komposition und Vererbung | ❌ |

---

## 3. StateRelation Typen

```rust
pub enum StateRelation {
    DependsOn,     // A ← B : A hängt kausal von B ab
    Triggers,      // A → B : A triggert Updates in B
    Bidirectional, // A ↔ B : A und B sind bidirektional
    Aggregates,    // A ⊃ B : A aggregiert Daten aus B
    Validates,     // A ✓ B : A validiert B
}
```

### 3.1 Semantik

| Relation | Symbol | Bedeutung | Transitiv? |
|----------|--------|-----------|------------|
| `DependsOn` | ← | A benötigt Daten von B für Berechnung | ✅ |
| `Triggers` | → | Änderung in A löst Update in B aus | ✅ |
| `Bidirectional` | ↔ | Gegenseitige Abhängigkeit | ❌ |
| `Aggregates` | ⊃ | A sammelt/aggregiert B-Daten | ✅ |
| `Validates` | ✓ | A validiert Konsistenz von B | ❌ |

### 3.2 Inverse Relationen

```
DependsOn ⟷ Triggers
Bidirectional ⟷ Bidirectional
Aggregates → (keine Inverse)
Validates → (keine Inverse)
```

---

## 4. StateGraph Beziehungen

Der `StateGraph` in `state.rs` (Zeilen 4080-4300) definiert **~100 Beziehungen**:

### 4.1 Identity-Layer Beziehungen

```
Trust ──DependsOn──▶ Identity    # Trust basiert auf Identity-Verifikation
Identity ──Triggers──▶ Trust     # Neue Identities erhalten initialen Trust

Event ──DependsOn──▶ Identity    # Events müssen Signatur der Identity haben
Identity ──Triggers──▶ Event     # Identity-Operationen erzeugen Events

Consensus ──DependsOn──▶ Identity # Validator-Identifikation via DID

Execution ──DependsOn──▶ Identity # ExecutionContext hat Identity
Identity ──DependsOn──▶ Execution # Identity-Ops verbrauchen Execution-Budget
Identity ──DependsOn──▶ Gas       # Sub-DID Derivation verbraucht Gas
Identity ──DependsOn──▶ Mana      # Identity-Events verbrauchen Mana

Realm ──DependsOn──▶ Identity     # Realm-Membership basiert auf Identity
Identity ──Triggers──▶ Realm      # Identity-Join/Leave triggert Realm-Updates
Room ──DependsOn──▶ Identity      # Room-Access basiert auf Identity
Partition ──DependsOn──▶ Identity # Partition-Zugehörigkeit basiert auf Identity

Controller ──DependsOn──▶ Identity # AuthZ basiert auf Identity
Identity ──Validates──▶ Controller # Identity validiert Delegation-Chain
Controller ──Aggregates──▶ Identity # Controller trackt Identities

Gateway ──DependsOn──▶ Identity   # Crossing erfordert Identity-Verifikation
Gateway ──Validates──▶ Identity   # Gateway validiert Cross-Realm Identity

ECLVM ──DependsOn──▶ Identity     # ECLVM prüft Caller-Identity
ECLPolicy ──DependsOn──▶ Identity # Policies können Identity-basierte Rules haben

Swarm ──DependsOn──▶ Identity     # Peer-ID ist Device-Sub-DID
Swarm ──Validates──▶ Identity     # Peer-Authentifizierung via Identity
Gossip ──DependsOn──▶ Identity    # Gossip-Messages sind signiert
Privacy ──DependsOn──▶ Identity   # Privacy-Level basiert auf Identity-Mode

Anomaly ──Validates──▶ Identity   # Anomalie-Detection für Identity-Ops
Identity ──Triggers──▶ Anomaly    # Suspicious Activity triggert Anomaly
AntiCalcification ──Validates──▶ Identity # Power-Konzentration durch Delegationen

Credential ──DependsOn──▶ Identity # Credentials gehören zu Identity
Credential ──Validates──▶ Identity # Credential-Verifikation validiert Identity
Identity ──Aggregates──▶ Credential # Identity aggregiert ihre Credentials

KeyManagement ──DependsOn──▶ Identity # Keys gehören zu Identity
Identity ──Aggregates──▶ KeyManagement # Identity aggregiert Key-Material
KeyManagement ──Triggers──▶ Event     # Key-Rotation erzeugt Events

KvStore ──Aggregates──▶ Identity  # KvStore persistiert Identity-Daten
Identity ──DependsOn──▶ KvStore   # Identity lädt State aus KvStore

UI ──DependsOn──▶ Identity        # UI zeigt Identity-basierte Inhalte
API ──DependsOn──▶ Identity       # API-AuthN basiert auf Identity
Governance ──DependsOn──▶ Identity # Voting-Power basiert auf Identity
```

### 4.2 Core-Layer Beziehungen

```
Trust ──Triggers──▶ Event         # Trust-Updates erzeugen Events
Event ──Triggers──▶ Trust         # Events können Trust beeinflussen
Trust ──DependsOn──▶ WorldFormula # Trust fließt in 𝔼
Event ──DependsOn──▶ WorldFormula # Events fließen in 𝔼

WorldFormula ──Triggers──▶ Consensus # 𝔼 beeinflusst Konsens
Consensus ──Validates──▶ Event    # Konsens validiert Events
```

### 4.3 Execution-Layer Beziehungen

```
Gas ──DependsOn──▶ Trust          # Gas-Budget basiert auf Trust
Mana ──DependsOn──▶ Trust         # Mana basiert auf Trust
Execution ──Aggregates──▶ Gas     # Execution trackt Gas
Execution ──Aggregates──▶ Mana    # Execution trackt Mana
Execution ──Triggers──▶ Event     # Execution emittiert Events

ECLVM ──DependsOn──▶ Gas          # ECLVM verbraucht Gas (Compute)
ECLVM ──DependsOn──▶ Mana         # ECLVM verbraucht Mana
ECLVM ──Triggers──▶ Event         # ECL-Ausführung emittiert Events
ECLVM ──Aggregates──▶ Execution   # ECLVM aggregiert Execution-Metriken
ECLVM ──DependsOn──▶ Trust        # ECL-Budget basiert auf Trust

ECLPolicy ──Validates──▶ Gateway  # Policies validieren Crossings
ECLPolicy ──Validates──▶ Realm    # Policies definieren Realm-Regeln
ECLPolicy ──DependsOn──▶ ECLVM    # Policies werden von ECLVM ausgeführt
ECLPolicy ──Triggers──▶ Event     # Policy-Evaluationen erzeugen Events

ECLBlueprint ──DependsOn──▶ ECLVM # Blueprints werden von ECLVM instanziiert
ECLBlueprint ──Aggregates──▶ Blueprint # Blueprint-Marketplace nutzt Storage
ECLBlueprint ──Triggers──▶ Event  # Blueprint-Instanziierung erzeugt Events
```

### 4.4 Protection-Layer Beziehungen

```
Anomaly ──Validates──▶ Event      # Anomaly prüft Events
Anomaly ──Validates──▶ Trust      # Anomaly prüft Trust-Patterns
Diversity ──Validates──▶ Trust    # Diversity prüft Trust-Verteilung
Diversity ──Validates──▶ Consensus # Diversity prüft Validator-Mix
Quadratic ──DependsOn──▶ Trust    # Voting-Power hängt von Trust ab

AntiCalcification ──Validates──▶ Trust # Anti-Calc überwacht Power
AntiCalcification ──Triggers──▶ Trust  # Anti-Calc kann Trust limitieren
Calibration ──Triggers──▶ Gas     # Calibration passt Gas-Preise an
Calibration ──Triggers──▶ Mana    # Calibration passt Mana-Regen an
```

### 4.5 Storage-Layer Beziehungen

```
EventStore ──Aggregates──▶ Event  # EventStore persistiert Events
Archive ──Aggregates──▶ EventStore # Archive komprimiert EventStore
KvStore ──DependsOn──▶ Trust      # KV-Access prüft Trust
Blueprint ──DependsOn──▶ Trust    # Blueprint-Publish prüft Trust
```

### 4.6 Peer-Layer Beziehungen (Κ22-Κ24)

```
Gateway ──Validates──▶ Trust      # Gateway prüft Trust für Crossing
Gateway ──DependsOn──▶ Trust      # Gateway-Entscheidung basiert auf Trust
Gateway ──Triggers──▶ Event       # Crossing erzeugt Events
Gateway ──DependsOn──▶ Realm      # Gateway prüft Realm-Crossing-Rules
Gateway ──DependsOn──▶ ECLPolicy  # Gateway führt Crossing-Policies aus

SagaComposer ──DependsOn──▶ Trust # Saga-Budget basiert auf Trust
SagaComposer ──Triggers──▶ Execution # Sagas erzeugen Executions
SagaComposer ──Aggregates──▶ IntentParser # Composer nutzt Parser
SagaComposer ──DependsOn──▶ ECLVM # Sagas werden durch ECLVM orchestriert

IntentParser ──Validates──▶ Event # Parser validiert Intent-Events
IntentParser ──DependsOn──▶ ECLPolicy # Intents werden gegen Policies validiert

Realm ──DependsOn──▶ Trust        # Realm-Trust basiert auf Global-Trust
Realm ──Triggers──▶ Trust         # Realm-Verhalten beeinflusst Global-Trust
Realm ──Aggregates──▶ Gateway     # Realm trackt Crossings
Realm ──DependsOn──▶ Gateway      # Realm nutzt Gateway für Crossing-Kontrolle
Realm ──Triggers──▶ SagaComposer  # Realm kann Cross-Realm-Sagas auslösen
Realm ──Triggers──▶ Event         # Realm-Events
Realm ──Validates──▶ Event        # Realm validiert Events gegen Policies
Realm ──DependsOn──▶ ECLPolicy    # Realm-Regeln definiert durch ECL
Realm ──Aggregates──▶ ECLPolicy   # Realm trackt aktive Policies

Room ──DependsOn──▶ Realm         # Room ist Sub-Einheit eines Realms
Room ──DependsOn──▶ Trust         # Room-Access prüft Trust
Room ──Triggers──▶ Event          # Room-Aktionen erzeugen Events
Room ──Aggregates──▶ Controller   # Room trackt Controller-Permissions

Partition ──DependsOn──▶ Room     # Partition ist Sub-Einheit eines Rooms
Partition ──DependsOn──▶ Trust    # Partition-Access prüft Trust
Partition ──Validates──▶ Controller # Partition validiert Controller-Scope
```

### 4.7 P2P-Layer Beziehungen

```
Swarm ──Triggers──▶ Event         # Swarm propagiert Events
Gossip ──DependsOn──▶ Trust       # Gossip-Scoring nutzt Trust
Gossip ──Triggers──▶ Event        # Gossip verteilt Events
Kademlia ──Aggregates──▶ Swarm    # DHT aggregiert Peer-Info
Relay ──DependsOn──▶ Trust        # Relay-Auswahl basiert auf Trust
Relay ──Triggers──▶ Swarm         # Relay beeinflusst Connections
NatTraversal ──Triggers──▶ Swarm  # NAT-Status beeinflusst Erreichbarkeit
Privacy ──DependsOn──▶ Trust      # Privacy-Level basiert auf Trust
Privacy ──Validates──▶ Gossip     # Privacy validiert Routing
```

### 4.8 Engine-Layer Beziehungen

```
# UI-Engine
UI ──DependsOn──▶ Trust           # UI-Sichtbarkeit basiert auf Trust
UI ──DependsOn──▶ Realm           # UI ist per-Realm isoliert
UI ──DependsOn──▶ Room            # UI-Scoping auf Room-Ebene
UI ──DependsOn──▶ Controller      # UI nutzt Controller für Permissions
UI ──Triggers──▶ Event            # UI-Actions erzeugen Events
UI ──Aggregates──▶ DataLogic      # UI nutzt DataLogic für Bindings
UI ──DependsOn──▶ ECLVM           # UI-Logik läuft in ECLVM
UI ──DependsOn──▶ Gas             # UI-Rendering verbraucht Gas
UI ──DependsOn──▶ Mana            # UI-Events verbrauchen Mana

# DataLogic-Engine
DataLogic ──DependsOn──▶ Event    # DataLogic verarbeitet Events
DataLogic ──Aggregates──▶ Event   # DataLogic aggregiert Event-Streams
DataLogic ──Triggers──▶ Event     # Aggregationen emittieren Events
DataLogic ──DependsOn──▶ Trust    # DataAccess prüft Trust
DataLogic ──DependsOn──▶ ECLVM    # DataLogic-Funktionen in ECLVM
DataLogic ──DependsOn──▶ Gas      # Compute verbraucht Gas
DataLogic ──Validates──▶ UI       # DataLogic validiert UI-Bindings

# API-Engine
API ──DependsOn──▶ Trust          # API-Access basiert auf Trust
API ──DependsOn──▶ Controller     # API nutzt Controller für AuthZ
API ──Validates──▶ Gateway        # API validiert External-Gateway
API ──Triggers──▶ Event           # API-Calls erzeugen Events
API ──DependsOn──▶ ECLVM          # API-Handler laufen in ECLVM
API ──DependsOn──▶ Gas            # API-Processing verbraucht Gas
API ──DependsOn──▶ Mana           # API-Responses verbrauchen Mana
API ──Aggregates──▶ DataLogic     # API nutzt DataLogic für Queries

# Governance-Engine
Governance ──DependsOn──▶ Trust   # Voting-Power basiert auf Trust
Governance ──DependsOn──▶ Quadratic # Governance nutzt Quadratic-Voting
Governance ──Validates──▶ Controller # Governance validiert Controller-Changes
Governance ──Triggers──▶ Controller # Governance-Votes ändern Controller
Governance ──Triggers──▶ Event    # Proposals/Votes erzeugen Events
Governance ──DependsOn──▶ ECLVM   # Governance-Regeln in ECLVM
Governance ──DependsOn──▶ Realm   # Governance ist per-Realm
Governance ──Validates──▶ AntiCalcification # Governance prüft Machtkonzentration

# Controller-Engine
Controller ──DependsOn──▶ Trust   # Permissions basieren auf Trust
Controller ──Triggers──▶ Event    # Permission-Changes erzeugen Events
Controller ──Validates──▶ Gateway # Controller validiert Crossings
Controller ──Validates──▶ API     # Controller validiert API-Access
Controller ──Validates──▶ UI      # Controller validiert UI-Access
Controller ──DependsOn──▶ Realm   # Controller-Scope ist per-Realm
Controller ──DependsOn──▶ Room    # Controller-Scope ist per-Room
Controller ──DependsOn──▶ Partition # Controller-Scope ist per-Partition
Controller ──Aggregates──▶ Governance # Controller trackt Gov-Delegations
Controller ──DependsOn──▶ ECLVM   # Permission-Rules in ECLVM

# BlueprintComposer-Engine
BlueprintComposer ──DependsOn──▶ Blueprint # Composer nutzt Blueprint-Storage
BlueprintComposer ──Aggregates──▶ ECLBlueprint # Composer aggregiert Instanzen
BlueprintComposer ──Triggers──▶ Event # Composition erzeugt Events
BlueprintComposer ──DependsOn──▶ ECLVM # Composition läuft in ECLVM
BlueprintComposer ──DependsOn──▶ Trust # Blueprint-Publish prüft Trust
BlueprintComposer ──Validates──▶ Realm # Composer validiert Realm-Kompatibilität
BlueprintComposer ──DependsOn──▶ Gas  # Composition verbraucht Gas
```

---

## 5. Graph-Analyse-Methoden

Der `StateGraph` bietet folgende Analyse-Funktionen:

### 5.1 Abhängigkeits-Analyse

```rust
// Alle Komponenten die von `component` abhängen
fn dependents(&self, component: StateComponent) -> Vec<StateComponent>

// Alle Komponenten von denen `component` abhängt
fn dependencies_of(&self, component: StateComponent) -> Vec<StateComponent>

// Transitive Abhängigkeiten (rekursiv)
fn transitive_dependencies(&self, component: StateComponent) -> HashSet<StateComponent>
```

**Beispiel:**
```rust
let graph = StateGraph::erynoa_graph();
let trust_deps = graph.transitive_dependencies(StateComponent::Trust);
// → {Identity, WorldFormula, ...}
```

### 5.2 Trigger-Analyse

```rust
// Alle Komponenten die `component` triggert
fn triggered_by(&self, component: StateComponent) -> Vec<StateComponent>

// Transitive Trigger-Kette
fn transitive_triggers(&self, component: StateComponent) -> HashSet<StateComponent>
```

**Beispiel:**
```rust
let triggered = graph.transitive_triggers(StateComponent::Identity);
// → {Trust, Event, Realm, Anomaly, ...}
```

### 5.3 Validierungs-Analyse

```rust
// Alle Komponenten die `component` validiert
fn validated_by(&self, component: StateComponent) -> Vec<StateComponent>

// Alle Validatoren für `component`
fn validators_of(&self, component: StateComponent) -> Vec<StateComponent>

// Validierungs-Kette
fn validation_chain(&self, component: StateComponent) -> Vec<StateComponent>
```

### 5.4 Kritikalitäts-Analyse

```rust
// Wie viele andere Komponenten abhängen + triggern + aggregieren
fn criticality_score(&self, component: StateComponent) -> usize
```

**Top 5 Kritische Komponenten:**

| Component | Score | Begründung |
|-----------|-------|------------|
| `Trust` | ~25 | Fast alle Komponenten nutzen Trust |
| `Identity` | ~22 | Basis für AuthN/AuthZ |
| `Event` | ~18 | Zentrales Kommunikationsmedium |
| `ECLVM` | ~15 | Alle Policies/Blueprints laufen hier |
| `Gateway` | ~12 | Realm-Isolation hängt davon ab |

---

## 6. Visualisierung: Trust-Zentrale Abhängigkeiten

```
                                    ┌──────────────┐
                                    │   Identity   │
                                    └───────┬──────┘
                                            │
                    ┌───────────────────────┼───────────────────────┐
                    │                       │                       │
                    ▼                       ▼                       ▼
            ┌──────────────┐        ┌──────────────┐        ┌──────────────┐
            │    Trust     │        │    Event     │        │   Gateway    │
            └──────┬───────┘        └──────────────┘        └──────────────┘
                   │
       ┌───────────┼───────────┬───────────┬───────────┐
       │           │           │           │           │
       ▼           ▼           ▼           ▼           ▼
   ┌───────┐  ┌────────┐  ┌─────────┐  ┌────────┐  ┌───────────┐
   │  Gas  │  │  Mana  │  │  Realm  │  │ Gossip │  │ Governance│
   └───────┘  └────────┘  └─────────┘  └────────┘  └───────────┘
```

---

## 7. Layer-Kommunikation

### 7.1 Erlaubte Zugriffe

| Von Layer | Darf zugreifen auf |
|-----------|-------------------|
| Engine | Alle unteren Layer |
| Peer | Execution, Protection, Storage, P2P, Core, Identity |
| P2P | Storage, Core, Identity |
| Storage | Core, Identity |
| Protection | Execution, Core, Identity |
| Execution | Core, Identity |
| Core | Identity |
| Identity | - (lowest layer) |

### 7.2 Verbotene Zugriffe

- Identity darf NICHT auf Core zugreifen
- Core darf NICHT auf Protection zugreifen
- P2P darf NICHT auf Engine zugreifen

---

## 8. Compile-Time Garantien

```rust
const _: () = {
    // StateComponent sollte klein bleiben (1-2 bytes)
    assert!(std::mem::size_of::<StateComponent>() <= 2);
    assert!(std::mem::size_of::<ComponentLayer>() == 1);
    assert!(std::mem::size_of::<StateRelation>() == 1);
};
```

---

## 9. Best Practices

### 9.1 Neue Beziehung hinzufügen

1. Prüfe ob Layer-Hierarchie eingehalten wird
2. Füge Edge in `StateGraph::erynoa_graph()` hinzu
3. Implementiere Observer-Methode in `state_integration.rs`
4. Aktualisiere diese Dokumentation

### 9.2 Neue Component hinzufügen

1. Erweitere `StateComponent` enum
2. Füge zu `ComponentLayer::components()` hinzu
3. Implementiere `layer()`, `description()` Match-Arms
4. Füge Beziehungen in `StateGraph` hinzu
5. Erstelle Observer-Trait
6. Erweitere `UnifiedState`

### 9.3 Kritikalitäts-Check

Vor jeder Änderung an kritischen Komponenten:
```rust
let component = StateComponent::Trust;
if component.is_critical() {
    // Erfordert zusätzliche Review
    // Circuit Breaker beachten
}
```

---

## 10. Referenzen

- **Haupt-Dokumentation:** [STATE-RS-ARCHITECTURE-DEEP-DIVE.md](./STATE-RS-ARCHITECTURE-DEEP-DIVE.md)
- **Quelldatei:** `backend/src/domain/unified/component.rs`
- **StateGraph:** `backend/src/core/state.rs` (Zeilen 4080-4450)
