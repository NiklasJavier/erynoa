# Erynoa Backend Architektur

> **Version:** 1.0.0
> **Datum:** Februar 2026
> **Status:** Production-Ready (Priorität 3 abgeschlossen)
> **Basis:** IPS-01-imp.md v1.2.0, UNIFIED-DATA-MODEL.md v1.1.0
> **Tests:** 409 Backend-Tests bestanden

---

## Executive Summary

Das Erynoa Backend implementiert das **Integrated Processing System (IPS)** – ein kategorialtheoretisch fundiertes, dezentrales System für vertrauensbasierte Zusammenarbeit. Die Architektur folgt dem **Fünf-Schichten-Modell**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           ERYNOA BACKEND ARCHITEKTUR                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │
│   │    API      │  │    PEER     │  │ PROTECTION  │  │     ECLVM       │   │
│   │  (gRPC)     │  │  (P2P+UI)   │  │  (Schutz)   │  │   (Policies)    │   │
│   └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └───────┬─────────┘   │
│          │                │                │                 │             │
│   ┌──────┴────────────────┴────────────────┴─────────────────┴──────┐      │
│   │                         CORE LOGIC                               │      │
│   │       (Event Engine, Trust Engine, World Formula, Consensus)     │      │
│   └──────────────────────────────┬───────────────────────────────────┘      │
│                                  │                                          │
│   ┌──────────────────────────────┴───────────────────────────────────┐      │
│   │                         DOMAIN (UDM)                              │      │
│   │   (UniversalId, Event, Trust, Realm, Saga, Cost, Message)        │      │
│   └──────────────────────────────┬───────────────────────────────────┘      │
│                                  │                                          │
│   ┌──────────────────────────────┴───────────────────────────────────┐      │
│   │                     LOCAL STORAGE (Fjall)                         │      │
│   │   (Events, Trust, Identities, Content, Realms, Archive)          │      │
│   └──────────────────────────────────────────────────────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## I. Verzeichnisstruktur (`backend/src/`)

```
backend/src/
├── main.rs              # Einstiegspunkt
├── lib.rs               # Library-Root
├── server.rs            # gRPC/HTTP Server
├── error.rs             # Globale Fehlertypen
├── telemetry.rs         # Observability (Tracing)
│
├── api/                 # 🌐 API-Schicht (gRPC, Connect)
│   └── ...
│
├── core/                # 🧠 Business-Logik (Axiome Κ2-Κ18)
│   ├── mod.rs
│   ├── consensus.rs     # Konsensus-Mechanismus (Κ18)
│   ├── engine.rs        # ExecutionContext-Wrapper
│   ├── event_engine.rs  # Event-Verarbeitung (Κ9-Κ12)
│   ├── surprisal.rs     # Surprisal-Berechnung (Κ15a)
│   ├── trust_engine.rs  # Trust-Berechnung (Κ2-Κ5)
│   └── world_formula.rs # Weltformel-Engine (Κ15b-d)
│
├── domain/              # 📦 Unified Data Model (UDM)
│   ├── mod.rs
│   └── unified/
│       ├── config.rs    # Globale Konfiguration
│       ├── cost.rs      # Kosten-Algebra (Gas × Mana × Trust)
│       ├── event.rs     # Events, Finality (Κ9-Κ12)
│       ├── formula.rs   # Weltformel-Komponenten
│       ├── identity.rs  # DID, Delegation (Κ6-Κ8)
│       ├── message.rs   # P2P-Nachrichtentypen
│       ├── primitives.rs# UniversalId, TemporalCoord
│       ├── realm.rs     # Realm-Hierarchie (Κ1)
│       ├── saga.rs      # Sagas (Κ22-Κ24)
│       ├── schema.rs    # Schema-Evolution
│       └── trust.rs     # TrustVector6D (Κ2-Κ5)
│
├── eclvm/               # ⚙️ Policy-VM
│   ├── mod.rs
│   ├── ast.rs           # Abstract Syntax Tree
│   ├── bridge.rs        # Core ↔ ECLVM Adjunktion
│   ├── bytecode.rs      # OpCode, Value
│   ├── compiler.rs      # AST → Bytecode
│   ├── erynoa_host.rs   # Host-Interface
│   ├── mana.rs          # Mana-Management
│   ├── optimizer.rs     # Bytecode-Optimierer
│   ├── parser.rs        # ECL → AST
│   ├── programmable_gateway.rs
│   ├── runtime/         # VM-Kern
│   │   ├── gas.rs       # Gas-Metering
│   │   ├── host.rs      # HostInterface Trait
│   │   └── vm.rs        # Stack-basierte VM
│   └── stdlib.rs        # Standard-Bibliothek
│
├── local/               # 💾 Persistenz (Fjall KV)
│   ├── mod.rs
│   ├── archive.rs       # Cold Storage (Merkle-Proofs)
│   ├── blueprint_marketplace.rs
│   ├── content_store.rs # Content-Addressed Storage
│   ├── event_store.rs   # Event-DAG Persistenz
│   ├── identity_store.rs# DID-Speicher
│   ├── kv_store.rs      # Basis KV-Abstraktion
│   ├── realm_storage.rs # Dynamische Realm-Stores
│   └── trust_store.rs   # Trust-Vektoren
│
├── peer/                # 🌍 P2P & Client-Facing
│   ├── mod.rs
│   ├── gateway.rs       # Cross-Realm Gateway (Κ23)
│   ├── intent_parser.rs # Intent-Parsing (Κ22)
│   ├── saga_composer.rs # Saga-Komposition (Κ22)
│   └── p2p/             # libp2p Netzwerk
│       ├── behaviour.rs # ErynoaBehaviour
│       ├── config.rs    # P2PConfig, NatConfig
│       ├── identity.rs  # PeerIdentity
│       ├── swarm.rs     # SwarmManager
│       ├── sync.rs      # Delta-Sync Protokoll
│       ├── topic.rs     # Realm-Topics
│       └── trust_gate.rs# Trust-basierte Verbindungen
│
├── protection/          # 🛡️ Systemschutz (Κ19-Κ28)
│   ├── mod.rs
│   ├── adaptive_calibration.rs  # PID-Controller
│   ├── anomaly.rs       # Anomalie-Erkennung
│   ├── anti_calcification.rs   # Macht-Dezentralisierung
│   ├── diversity.rs     # System-Diversität
│   └── quadratic.rs     # Quadratisches Voting
│
├── execution/           # 🔄 Saga-Execution
│   └── ...
│
└── gen/                 # 🤖 Generierter Protobuf-Code
    └── ...
```

---

## II. Domain Layer (Unified Data Model)

Das **Unified Data Model (UDM)** definiert alle Kerntypen als Single Source of Truth.

### 2.1 Kern-Primitive

#### UniversalId (32 Bytes)

Content-addressed Identifier mit eingebettetem Type-Tag:

```
┌──────────┬────────────┬─────────────────────────────────────────┐
│ Type Tag │  Version   │            BLAKE3 Hash (28 bytes)       │
│ (2 bytes)│  (2 bytes) │                                         │
└──────────┴────────────┴─────────────────────────────────────────┘
```

**Type Tags:**

| Tag    | Typ     | Beschreibung           |
| ------ | ------- | ---------------------- |
| 0x0001 | DID     | Dezentrale Identität   |
| 0x0002 | Event   | Kausales Event         |
| 0x0003 | Realm   | Realm-Instanz          |
| 0x0004 | Trust   | Trust-Record           |
| 0x0005 | Saga    | Multi-Step Transaktion |
| 0x0006 | Schema  | Daten-Schema           |
| 0x0030 | Program | ECLVM-Programm         |

#### TemporalCoord (16 Bytes)

Hybride logisch-physische Zeitkoordinate:

```
┌─────────────────────┬──────────────────┬─────────────────────┐
│   Wall-Clock (8B)   │  Lamport (4B)    │   Node-Hash (4B)    │
│   Mikrosekunden     │  Logische Zeit   │   Tie-Breaker       │
└─────────────────────┴──────────────────┴─────────────────────┘
```

**Garantie:** `happens_before(a, b) ⟹ a.coord < b.coord`

### 2.2 Trust-System (Κ2-Κ5)

#### TrustVector6D (24 Bytes)

```rust
pub struct TrustVector6D {
    pub r: f32,     // Reliability (Verhaltens-Historie)
    pub i: f32,     // Integrity (Aussage-Konsistenz)
    pub c: f32,     // Competence (Fähigkeits-Nachweis)
    pub p: f32,     // Prestige (Externe Attestation)
    pub v: f32,     // Vigilance (Anomalie-Erkennung)
    pub omega: f32, // Omega (Axiom-Treue)
}
```

**Kontext-Gewichtung:**

- **Finance:** `[0.25, 0.25, 0.15, 0.15, 0.10, 0.10]` (R, I hoch)
- **Social:** `[0.10, 0.15, 0.10, 0.30, 0.25, 0.10]` (P, V hoch)
- **Govern:** `[0.15, 0.20, 0.10, 0.10, 0.10, 0.35]` (Ω hoch)

### 2.3 Event-System (Κ9-Κ12)

```rust
pub struct Event {
    pub id: EventId,           // Content-Hash (32 Bytes)
    pub creator: DID,          // Ersteller
    pub realm_id: RealmId,     // Zugehöriger Realm
    pub payload: EventPayload, // Typ-spezifische Daten
    pub parents: Vec<EventId>, // DAG-Kanten
    pub timestamp: TemporalCoord,
    pub signature: Signature64,
    pub finality: FinalityState,
}
```

**Finality-Level:**

| Level       | Bedeutung              | Bedingung               |
| ----------- | ---------------------- | ----------------------- |
| `Pending`   | Noch nicht finalisiert | Initiale Events         |
| `Witnessed` | Von Witness bestätigt  | ≥1 Attestation          |
| `Confirmed` | Stark bezeugt          | ≥3 Attestations + 10min |
| `Anchored`  | Unveränderlich         | Checkpointed            |

### 2.4 Kosten-Algebra

```rust
pub struct Cost {
    pub gas: u64,        // Computation
    pub mana: u64,       // Storage/Network
    pub trust_risk: f32, // Trust-Kosten [0, 1]
}
```

**Operationen:**

- **Sequentiell (⊕):** `(g₁+g₂, m₁+m₂, 1-(1-t₁)(1-t₂))`
- **Parallel (⊗):** `(max(g₁,g₂), m₁+m₂, max(t₁,t₂))`

---

## III. Core Logic Layer

### 3.1 Event Engine (Κ9-Κ12)

Verarbeitet kausale Events im DAG:

```rust
pub struct EventEngine {
    event_store: Arc<EventStore>,
    trust_engine: Arc<TrustEngine>,
    formula_engine: Arc<WorldFormulaEngine>,
}

impl EventEngine {
    pub async fn process_event(&self, event: Event) -> Result<ProcessedEvent>;
    pub async fn validate_causality(&self, event: &Event) -> Result<()>;
    pub async fn compute_finality(&self, event_id: &EventId) -> FinalityState;
}
```

### 3.2 Trust Engine (Κ2-Κ5)

Berechnet und aktualisiert Trust-Vektoren:

```rust
pub struct TrustEngine {
    trust_store: Arc<TrustStore>,
    dampening_matrix: TrustDampeningMatrix,
}

impl TrustEngine {
    pub fn compute_trust(&self, subject: &DID, context: ContextType) -> TrustVector6D;
    pub fn update_from_attestation(&mut self, attestation: &Attestation) -> Result<()>;
    pub fn combine_vectors(&self, a: &TrustVector6D, b: &TrustVector6D) -> TrustVector6D;
}
```

### 3.3 World Formula Engine (Κ15a-d)

Berechnet den systemweiten Optimum-Wert 𝔼:

```
𝔼(s) = α · Activity(s) + β · Surprisal(s) + γ · HumanFactor(s) + δ · Temporal(s)
```

```rust
pub struct WorldFormulaEngine {
    config: WorldFormulaConfig,
    calibration: CalibrationEngine,
}

impl WorldFormulaEngine {
    pub fn compute(&self, state: &WorldState) -> f64;
    pub fn compute_contribution(&self, event: &Event) -> WorldFormulaContribution;
}
```

### 3.4 Consensus Engine (Κ18)

Witness-basierter Konsensus ohne globalen Leader:

```rust
pub struct ConsensusEngine {
    witness_threshold: u32,
    time_threshold: Duration,
}

impl ConsensusEngine {
    pub fn check_finality(&self, event: &Event, attestations: &[Attestation]) -> FinalityLevel;
    pub fn select_witnesses(&self, realm: &Realm) -> Vec<DID>;
}
```

---

## IV. ECLVM (Policy Engine)

### 4.1 Pipeline

```
┌─────────┐    ┌──────────┐    ┌──────────┐    ┌─────────┐
│   ECL   │───▶│  Parser  │───▶│ Compiler │───▶│Bytecode │
│  Text   │    │ (Lexer)  │    │(AST→Op)  │    │(OpCode) │
└─────────┘    └──────────┘    └──────────┘    └────┬────┘
                                                    │
                    ┌───────────────────────────────┘
                    ▼
              ┌─────────────────────────────────┐
              │           ECLVM Runtime          │
              │  [Stack] [IP] [Gas] [Host]       │
              └─────────────────────────────────┘
```

### 4.2 OpCodes

| OpCode            | Gas        | Beschreibung       |
| ----------------- | ---------- | ------------------ |
| `PushConst`       | 1          | Wert auf Stack     |
| `Add/Sub/Mul/Div` | 2          | Arithmetik         |
| `Call`            | 10+arity×2 | Funktionsaufruf    |
| `HostCall`        | 50         | Erynoa-API Zugriff |
| `Return`          | 1          | Rückgabe           |

### 4.3 Core ↔ ECLVM Adjunktion

Verlustfreie Übersetzung zwischen Domain-Typen und VM-Werten:

```rust
// Linker Adjunkt F: Core → ECLVM (Embedding)
pub trait CoreToEclvm {
    fn embed(&self) -> EclvmValue;
}

// Rechter Adjunkt G: ECLVM → Core (Interpretation)
pub trait EclvmToCore: Sized {
    fn interpret(value: &EclvmValue) -> Result<Self, InterpretError>;
}
```

**Zig-Zag Identity:** `interpret(embed(x)) ≅ x`

---

## V. Local Storage (Fjall)

### 5.1 Partitionen

```
┌─────────────┬─────────────┬─────────────┬─────────────┬─────────────────────┐
│ identities  │   events    │    trust    │   content   │    realm_storage    │
│ (DIDs,Keys) │  (DAG)      │ (Vektoren)  │  (CAS)      │  (Dynamische Stores)│
└─────────────┴─────────────┴─────────────┴─────────────┴─────────────────────┘
```

### 5.2 Cold Storage Archive

Langzeit-Archivierung mit Merkle-Root-Preservation:

```rust
pub struct Archive {
    fjall: Arc<Keyspace>,
    config: ArchiveConfig,
}

impl Archive {
    pub async fn archive_epoch(&self, epoch: u64, events: &[Event]) -> ArchiveResult<EpochMetadata>;
    pub fn get_merkle_proof(&self, epoch: u64, event_id: &EventId) -> Option<MerkleProof>;
    pub fn verify_proof(&self, proof: &MerkleProof) -> bool;
}
```

### 5.3 Blueprint Marketplace

Dezentraler Template-Store:

```rust
pub struct BlueprintMarketplace {
    store: BlueprintStore,
    novelty_calculator: NoveltyCalculator,
}

impl BlueprintMarketplace {
    pub async fn publish(&self, blueprint: Blueprint) -> PublishResult;
    pub async fn deploy(&self, id: &BlueprintId, realm: &RealmId) -> DeploymentResult;
    pub async fn rate(&self, id: &BlueprintId, rating: u8) -> RatingResult;
}
```

---

## VI. P2P Network Layer

### 6.1 libp2p Stack

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          P2P NETWORK LAYER                              │
├─────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │   GOSSIPSUB  │  │   KADEMLIA   │  │   IDENTIFY   │                  │
│  │   (PubSub)   │  │   (DHT)      │  │   (Handshake)│                  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │
│         │                 │                 │                           │
│  ┌──────┴─────────────────┴─────────────────┴──────┐                   │
│  │              ERYNOA BEHAVIOUR                    │                   │
│  │  • Realm-Topics (/erynoa/realm/{id}/events/v1)  │                   │
│  │  • DID-based PeerID (Ed25519)                   │                   │
│  │  • Trust-gated Connections                      │                   │
│  └──────────────────────────────────────────────────┘                   │
│                            │                                            │
│  ┌────────────────────────┴────────────────────────┐                   │
│  │    NAT TRAVERSAL (AutoNAT + DCUTR + Relay)      │                   │
│  └──────────────────────────────────────────────────┘                   │
│                            │                                            │
│  ┌────────────────────────┴────────────────────────┐                   │
│  │    TRANSPORT (TCP + Noise + Yamux)              │                   │
│  └──────────────────────────────────────────────────┘                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### 6.2 NAT-Traversal (Priorität 3)

```rust
pub struct NatConfig {
    pub enable_autonat: bool,    // NAT-Typ Erkennung
    pub enable_dcutr: bool,      // Direct Connection Upgrade
    pub enable_relay: bool,      // Relay für symmetric NAT
    pub enable_upnp: bool,       // UPnP Port-Mapping
    pub relay_servers: Vec<Multiaddr>,
}
```

### 6.3 Trust Gate

Verbindungen nur mit Trust.R > 0.5:

```rust
pub struct TrustGate {
    min_trust: f32,
    trust_store: Arc<TrustStore>,
}

impl TrustGate {
    pub fn allow_connection(&self, peer: &DID) -> bool {
        self.trust_store.get(peer)
            .map(|t| t.r >= self.min_trust)
            .unwrap_or(false)
    }
}
```

---

## VII. Protection Layer

### 7.1 Adaptive Calibration (Κ19, §IX)

PID-Controller für dynamische Weltformel-Parameter:

```rust
pub struct CalibrationEngine {
    config: CalibrationConfig,
    history: VecDeque<NetworkMetrics>,
    ema_alpha: f64,
}

impl CalibrationEngine {
    pub fn calibrate(&mut self, metrics: NetworkMetrics) -> CalibratedParameters;
    pub fn compute_pid_adjustment(&self, error: f64, dimension: &str) -> f64;
}
```

**Parameter-Grenzen:**

| Parameter     | Min  | Max  | Default |
| ------------- | ---- | ---- | ------- |
| α (Activity)  | 0.05 | 0.50 | 0.25    |
| β (Surprisal) | 0.10 | 0.60 | 0.35    |
| γ (Human)     | 0.05 | 0.40 | 0.20    |
| δ (Temporal)  | 0.05 | 0.40 | 0.20    |

### 7.2 Anti-Calcification (Κ19)

Verhindert Macht-Konzentration:

```rust
pub struct AntiCalcification {
    config: AntiCalcificationConfig,
    power_tracker: PowerDistribution,
}

impl AntiCalcification {
    pub fn check_power_concentration(&self) -> Option<PowerAlert>;
    pub fn apply_decay(&mut self, time_delta: Duration);
}
```

### 7.3 Anomaly Detection (Κ26)

```rust
pub struct AnomalyDetector {
    baseline: BehaviorBaseline,
    z_threshold: f64,
}

impl AnomalyDetector {
    pub fn detect(&self, behavior: &BehaviorSample) -> Option<Anomaly>;
    pub fn update_baseline(&mut self, sample: &BehaviorSample);
}
```

---

## VIII. Axiom-Mapping

| Axiom   | Modul                                                                    | Implementierung                  |
| ------- | ------------------------------------------------------------------------ | -------------------------------- |
| Κ1      | `domain/unified/realm.rs`                                                | Realm-Hierarchie, Rule-Vererbung |
| Κ2-Κ5   | `domain/unified/trust.rs`, `core/trust_engine.rs`                        | TrustVector6D, Bayesian Update   |
| Κ6-Κ8   | `domain/unified/identity.rs`                                             | DID, Delegation, Capabilities    |
| Κ9-Κ12  | `domain/unified/event.rs`, `core/event_engine.rs`                        | Event-DAG, Finality              |
| Κ15a-d  | `core/world_formula.rs`, `core/surprisal.rs`                             | Weltformel, Surprisal            |
| Κ18     | `core/consensus.rs`                                                      | Witness-Konsensus                |
| Κ19     | `protection/anti_calcification.rs`, `protection/adaptive_calibration.rs` | Power-Decay, PID                 |
| Κ20     | `protection/diversity.rs`                                                | Diversity-Monitoring             |
| Κ21     | `protection/quadratic.rs`                                                | Quadratic Voting                 |
| Κ22-Κ24 | `domain/unified/saga.rs`, `peer/saga_composer.rs`                        | Saga-System                      |
| Κ23     | `peer/gateway.rs`, `eclvm/programmable_gateway.rs`                       | Gateway Guard                    |
| Κ26     | `protection/anomaly.rs`                                                  | Anomalie-Erkennung               |

---

## IX. Konfiguration

### 9.1 Umgebungsvariablen

| Variable           | Beschreibung     | Default  |
| ------------------ | ---------------- | -------- |
| `ERYNOA_DATA_DIR`  | Datenverzeichnis | `./data` |
| `ERYNOA_LOG_LEVEL` | Log-Level        | `info`   |
| `ERYNOA_P2P_PORT`  | P2P Port         | `4001`   |
| `ERYNOA_GRPC_PORT` | gRPC Port        | `50051`  |

### 9.2 Config-Dateien

```toml
# config/base.toml
[storage]
path = "./data"
max_size_gb = 100

[p2p]
enable = true
bootstrap_peers = []

[world_formula]
alpha = 0.25
beta = 0.35
gamma = 0.20
delta = 0.20
```

---

## X. Test-Status

| Bereich       | Tests   | Status |
| ------------- | ------- | ------ |
| Domain (UDM)  | 89      | ✅     |
| Core Engines  | 124     | ✅     |
| ECLVM         | 67      | ✅     |
| Local Storage | 52      | ✅     |
| P2P           | 38      | ✅     |
| Protection    | 39      | ✅     |
| **Gesamt**    | **409** | ✅     |

---

## XI. Performance-Charakteristiken

| Operation             | Latenz (p99) | Durchsatz |
| --------------------- | ------------ | --------- |
| Event erstellen       | < 5ms        | 10k/s     |
| Trust-Lookup          | < 1ms        | 100k/s    |
| Weltformel-Berechnung | < 10ms       | 1k/s      |
| Storage Read          | < 1ms        | 50k/s     |
| Storage Write         | < 5ms        | 10k/s     |
| P2P Gossip            | < 100ms      | 1k msg/s  |

---

## XII. Weiterführende Dokumentation

- [IPS-01-imp.md](../development/IPS-01-imp.md) – Mathematisches Logik-Modell
- [UNIFIED-DATA-MODEL.md](../development/UNIFIED-DATA-MODEL.md) – Datenstruktur-Spezifikation
- [P2P-IMPLEMENTATION.md](../development/P2P-IMPLEMENTATION.md) – P2P-Details
- [IPS-UDM-GAP-ANALYSIS.md](../development/IPS-UDM-GAP-ANALYSIS.md) – Implementierungs-Status

---

_Letzte Aktualisierung: Februar 2026 (Priorität 3 abgeschlossen)_
