# Erynoa Backend - System-Initialisierungskonzept

## Übersicht

Dieses Dokument beschreibt den exakten Startablauf des Erynoa-Backends.

## Einheitliche Initialisierung (NEU)

Mit der integrierten P2P-Unterstützung startet **alles über `main.rs`**:

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                  ERYNOA UNIFIED STARTUP (mit P2P)                                │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌────────────────────────────────────────────────────────────────────────┐     │
│  │                    erynoa (main.rs)                                    │     │
│  │              Vollständiger Node: API + P2P + Storage                    │     │
│  ├────────────────────────────────────────────────────────────────────────┤     │
│  │ ✓ Telemetry                     → Logging/Tracing                      │     │
│  │ ✓ Settings                      → Config laden (inkl. p2p section)     │     │
│  │ ✓ DecentralizedStorage (Fjall)  → Local Storage                        │     │
│  │ ✓ UnifiedState (alle Layer)     → Core, Execution, Protection, Peer... │     │
│  │ ✓ StateCoordinator              → Health + Invarianten                 │     │
│  │ ✓ P2P Network (optional)        → libp2p Swarm, Gossip, Kademlia...   │     │
│  │ ✓ HTTP Router (Axum)            → Vollständige API                     │     │
│  └────────────────────────────────────────────────────────────────────────┘     │
│                                                                                  │
│  Verwendung OHNE P2P:                                                           │
│  cargo run                                                                      │
│                                                                                  │
│  Verwendung MIT P2P:                                                            │
│  cargo run --features p2p                                                       │
│  + Config: features.p2p_enabled = true                                          │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### P2P über Konfiguration aktivieren

**Option 1: Config-Datei (`config/base.toml`)**

```toml
[features]
p2p_enabled = true
privacy_enabled = false  # Optional: Onion-Routing

[p2p]
port = 4001
node_name = "my-erynoa-node"
enable_mdns = true
enable_relay_server = false
enable_autonat = true
enable_upnp = true
min_incoming_trust = 0.1
bootstrap_peers = [
    # "/ip4/51.159.23.74/tcp/4001/p2p/12D3KooW..."
]
```

**Option 2: Environment-Variablen**

```bash
APP_FEATURES__P2P_ENABLED=true \
APP_P2P__PORT=4001 \
APP_P2P__NODE_NAME=my-node \
cargo run --features p2p
```

### Was startet mit P2P?

| Komponente | p2p_enabled=false | p2p_enabled=true |
|------------|:-----------------:|:----------------:|
| **Core** (Trust, Events, Formula, Consensus) | ✓ State | ✓ State + Events |
| **Execution** (Gas, Mana, ECLVM) | ✓ State | ✓ State |
| **Protection** (Anomaly, Diversity) | ✓ State | ✓ State |
| **Local/Storage** (Fjall) | ✓ Aktiv | ✓ Aktiv |
| **Domain** (Typen, Primitives) | ✓ Verfügbar | ✓ Verfügbar |
| **Peer/P2P** (libp2p Swarm) | ✗ Nur State | ✓ **Aktives Netzwerk** |
| **HTTP API** | ✓ Aktiv | ✓ Aktiv |

---

## Legacy: Separates Testnet-Binary

Das separate Binary `erynoa-testnet-node` existiert noch für spezielle Testnet-Szenarien:

```bash
cargo run --features p2p --bin erynoa-testnet-node -- \
    --node-name relay1 \
    --p2p-port 4001 \
    --api-port 9000 \
    --mode relay
```

---

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         SYSTEM-INITIALISIERUNG                                   │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  1. main.rs                    Entry Point                                      │
│     │                                                                            │
│     ├─▶ init_subscriber()      Telemetry/Logging initialisieren                 │
│     ├─▶ Settings::load()       Konfiguration laden                              │
│     │                                                                            │
│  2. Server::build_with_static()                                                 │
│     │                                                                            │
│     ├─▶ DecentralizedStorage::open()    Fjall Storage öffnen                   │
│     ├─▶ AppState::new()                 Application State erstellen            │
│     │   │                                                                        │
│     │   ├─▶ create_unified_state()      UnifiedState mit allen Layern          │
│     │   └─▶ StateCoordinator::new()     Health + Invarianten                   │
│     │                                                                            │
│     ├─▶ create_router()                 API-Router mit Endpunkten              │
│     └─▶ TcpListener::bind()             Server-Socket binden                   │
│                                                                                  │
│  3. server.run()                                                                │
│     │                                                                            │
│     └─▶ axum::serve() + shutdown_signal()   HTTP-Server starten               │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Entry Point (`main.rs`)

**Datei:** `backend/src/main.rs`

### 1.1 Allocator-Setup (Optional)

```rust
#[cfg(all(not(target_env = "msvc"), feature = "jemalloc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
```

**Zweck:** Optimierter Memory-Allocator für bessere Performance unter Linux.

### 1.2 Tokio-Runtime starten

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
```

**Zweck:** Multi-threaded async Runtime initialisieren.

### 1.3 Telemetry initialisieren

```rust
let subscriber = get_subscriber("erynoa-backend".into(), "info".into(), std::io::stdout);
init_subscriber(subscriber);
```

**Was passiert:**
- Tracing-Subscriber wird konfiguriert
- Log-Level auf "info" gesetzt
- Output auf stdout

### 1.4 Konfiguration laden

```rust
let settings = Settings::load().expect("Failed to load configuration");
```

**Was passiert:**
- Lädt Konfiguration aus Environment-Variablen und/oder Config-Dateien
- Beinhaltet: Host, Port, Storage-Pfad, Environment (dev/prod)

### 1.5 CLI-Argumente parsen

```rust
let static_dir = parse_static_dir(&args);
```

**Unterstützte Argumente:**
- `--static-dir <path>` oder `ERYNOA_STATIC_DIR` Environment-Variable

---

## Phase 2: Server Build (`server.rs`)

**Datei:** `backend/src/server.rs`

### 2.1 Storage initialisieren

```rust
let storage = DecentralizedStorage::open(data_dir)?;
```

**Was passiert:**
- Fjall LSM-Tree Storage wird geöffnet
- Pfad aus `settings.storage.data_dir`
- Erstellt Verzeichnis falls nicht vorhanden

### 2.2 AppState erstellen

```rust
let state = AppState::new(storage, settings.clone());
```

**AppState-Struktur:**

```rust
pub struct AppState {
    pub unified_state: SharedUnifiedState,  // Arc<UnifiedState>
    pub coordinator: Arc<StateCoordinator>, // Health + Invarianten
    pub storage: DecentralizedStorage,       // Fjall Storage
    pub config: Arc<Settings>,               // Konfiguration
    pub started_at: Option<Instant>,         // Startzeit
    pub gateway: Option<Arc<GatewayGuard>>,  // Phase 2
}
```

### 2.3 UnifiedState initialisieren (Kern des Systems)

```rust
pub fn create_unified_state() -> SharedUnifiedState {
    Arc::new(UnifiedState::new())
}
```

---

## Phase 3: UnifiedState - Hierarchische Initialisierung

**Datei:** `backend/src/core/state.rs` (Zeile 10718-10866)

### 3.1 UnifiedState-Struktur

```rust
pub struct UnifiedState {
    pub started_at: Instant,

    // ═══════════════════════════════════════════════════════════════
    // LAYER 1: IDENTITY (Κ6-Κ8 DID Management)
    // ═══════════════════════════════════════════════════════════════
    pub identity: IdentityState,

    // ═══════════════════════════════════════════════════════════════
    // LAYER 2: CORE (Κ2-Κ18)
    // ═══════════════════════════════════════════════════════════════
    pub core: CoreState,          // Trust, Events, Formula, Consensus

    // ═══════════════════════════════════════════════════════════════
    // LAYER 3: EXECUTION (IPS ℳ)
    // ═══════════════════════════════════════════════════════════════
    pub execution: ExecutionState, // Gas, Mana, Context-Tracking
    pub eclvm: ECLVMState,         // ECL Policies, Blueprints, Sagas

    // ═══════════════════════════════════════════════════════════════
    // LAYER 4: PROTECTION (Κ19-Κ21)
    // ═══════════════════════════════════════════════════════════════
    pub protection: ProtectionState, // Anomaly, Diversity, Calibration

    // ═══════════════════════════════════════════════════════════════
    // LAYER 5: STORAGE
    // ═══════════════════════════════════════════════════════════════
    pub storage: StorageState,

    // ═══════════════════════════════════════════════════════════════
    // LAYER 6: PEER (Κ22-Κ24)
    // ═══════════════════════════════════════════════════════════════
    pub peer: PeerState,           // Gateway, Saga, Realm
    pub p2p: P2PState,             // Swarm, Gossip, Kademlia

    // ═══════════════════════════════════════════════════════════════
    // LAYER 7: ENGINES
    // ═══════════════════════════════════════════════════════════════
    pub ui: UIState,
    pub api: APIState,
    pub governance: GovernanceState,
    pub controller: ControllerState,
    pub data_logic: DataLogicState,
    pub blueprint_composer: BlueprintComposerState,

    // ═══════════════════════════════════════════════════════════════
    // INFRASTRUCTURE
    // ═══════════════════════════════════════════════════════════════
    pub graph: StateGraph,          // Beziehungs-Graph
    pub event_bus: EventBus,        // P2P/Core Entkopplung
    pub circuit_breaker: CircuitBreaker, // Degradation
    pub broadcaster: StateBroadcaster,   // CQRS Deltas
    pub storage_handle: StorageHandle,   // Pluggable Storage
    pub merkle_tracker: MerkleStateTracker, // Differential Snapshots
    pub multi_gas: MultiGas,        // Multi-Level Gas Metering
    pub event_log: StateEventLog,   // Event-Sourcing
}
```

### 3.2 Initialisierungs-Reihenfolge im Detail

```rust
impl UnifiedState {
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),

            // 1. Identity Layer (Basis für alle anderen)
            identity: IdentityState::new(),

            // 2. Core Logic Layer
            core: CoreState::new(),
            //    └── TrustState::new()
            //    └── EventState::new()
            //    └── FormulaState::new()
            //    └── ConsensusState::new()

            // 3. Execution Layer
            execution: ExecutionState::new(),
            //    └── GasState::new()
            //    └── ManaState::new()
            //    └── ExecutionContextState::new()

            eclvm: ECLVMState::new(),
            //    └── ECLPolicyState::new()
            //    └── ECLBlueprintState::new()
            //    └── ECLCrossingState::new()

            // 4. Protection Layer
            protection: ProtectionState::new(),
            //    └── AnomalyState::new()
            //    └── DiversityState::new()
            //    └── CalibrationState::new()

            // 5. Storage Layer
            storage: StorageState::new(),

            // 6. Peer/P2P Layer
            peer: PeerState::new(),
            //    └── GatewayState::new()
            //    └── SagaComposerState::new()
            //    └── RealmState::new()

            p2p: P2PState::new(),
            //    └── SwarmState::new()
            //    └── GossipState::new()
            //    └── KademliaState::new()
            //    └── PrivacyState::new()

            // 7. Engine Layer
            ui: UIState::new(),
            api: APIState::new(),
            governance: GovernanceState::new(),
            controller: ControllerState::new(),
            data_logic: DataLogicState::new(),
            blueprint_composer: BlueprintComposerState::new(),

            // 8. State-Graph (Beziehungen zwischen Komponenten)
            graph: StateGraph::erynoa_graph(),

            // 9. Architektur-Komponenten
            event_bus: EventBus::new(),
            circuit_breaker: CircuitBreaker::new(),
            broadcaster: StateBroadcaster::new(),
            storage_handle: StorageHandle::new(StorageBackend::RocksDB),
            merkle_tracker: MerkleStateTracker::new(),
            multi_gas: MultiGas::new(),
            event_log: StateEventLog::new(),
        }
    }
}
```

---

## Phase 4: Sub-State Initialisierungen

### 4.1 IdentityState (Κ6-Κ8)

```rust
pub struct IdentityState {
    // Atomics (Lock-free)
    pub bootstrap_completed: AtomicBool,    // false → true nach Bootstrap
    pub root_created_at_ms: AtomicU64,
    pub mode: AtomicU8,                      // 0=Interactive, 1=Agent, 2=Ephemeral, 3=Test
    pub sub_dids_total: AtomicU64,
    pub active_delegations_count: AtomicU64,

    // Complex State (RwLock)
    pub root_did: RwLock<Option<DID>>,
    pub delegations: RwLock<HashMap<UniversalId, Delegation>>,
    pub realm_memberships: RwLock<HashMap<UniversalId, RealmMembership>>,

    // Handles
    pub key_store: Option<SharedKeyStore>,
    pub passkey_manager: Option<SharedPasskeyManager>,
}
```

**Initialer Zustand:** Nicht bootstrapped, keine Root-DID, leere Maps.

### 4.2 CoreState (Κ2-Κ18)

```rust
pub struct CoreState {
    pub trust: TrustState,      // Trust-Updates, Asymmetrie
    pub events: EventState,     // DAG-Events, Depths
    pub formula: FormulaState,  // Weltformel E-Wert
    pub consensus: ConsensusState, // Rounds, Success-Rate
}
```

**Jede Sub-State initialisiert:**
- Atomare Counter auf 0
- RwLocks mit Default-Werten
- Keine laufenden Operationen

### 4.3 ExecutionState (IPS ℳ)

```rust
pub struct ExecutionState {
    pub gas: GasState,          // Gas-Tracking, Preise
    pub mana: ManaState,        // Mana-Regeneration
    pub contexts: ExecutionContextState, // Laufende Executions
}
```

**Initialer Zustand:**
- Gas consumed: 0, Preis: 1.0
- Mana consumed: 0, Regen-Rate: 1.0
- Keine aktiven Execution-Contexts

### 4.4 ProtectionState (Κ19-Κ21)

```rust
pub struct ProtectionState {
    pub anomaly: AnomalyState,      // Anomalie-Erkennung
    pub diversity: DiversityState,  // Gini, Entropie
    pub calibration: CalibrationState, // Anti-Calcification
}
```

**Initialer Zustand:**
- Anomaly-Counter: 0
- Diversity-Metriken: Neutral (Entropie ~1.0, Gini ~0.0)
- Calibration: Aktiv, keine Anpassungen

### 4.5 EventBus (P2P/Core Entkopplung)

```rust
pub struct EventBus {
    // Ingress: P2P → Core
    pub ingress_tx: mpsc::Sender<NetworkEvent>,       // Kapazität: 10.000
    pub ingress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,

    // Egress: Core → P2P
    pub egress_tx: mpsc::Sender<NetworkEvent>,
    pub egress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,

    // High-Priority (Consensus, Trust-Critical)
    pub priority_ingress_tx: mpsc::Sender<NetworkEvent>, // Kapazität: 1.000
    pub priority_ingress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,
}
```

### 4.6 CircuitBreaker

```rust
pub struct CircuitBreaker {
    pub mode: AtomicU8,                 // SystemMode::Normal (0)
    pub degraded_threshold: AtomicU64,  // 10 kritische Anomalien/Minute
    pub emergency_threshold: AtomicU64, // 50 kritische Anomalien/Minute
    pub gini_threshold: RwLock<f64>,    // 0.8
}
```

**Initialer Modus:** `SystemMode::Normal`

---

## Phase 5: StateCoordinator

**Datei:** `backend/src/core/state_coordination.rs`

```rust
pub struct StateCoordinator {
    unified_state: SharedUnifiedState,
    integrator: StateIntegrator,  // Observer-Pattern
}

impl StateCoordinator {
    pub fn new(unified_state: SharedUnifiedState) -> Self {
        Self {
            unified_state: unified_state.clone(),
            integrator: StateIntegrator::new(unified_state),
        }
    }
}
```

**Funktionen:**
- `aggregate_health()` → HealthReport
- `check_invariants()` → Vec<InvariantViolation>
- `integrator()` → StateIntegrator (für Observer-Pattern)

---

## Phase 6: Router + Server

### 6.1 API-Router erstellen

```rust
let api_router = create_router(state);
```

**Enthält:**
- Health-Endpoints (`/health`, `/ready`)
- State-Endpoints (`/api/v1/state/*`)
- Governance-Endpoints
- CORS, Tracing, Compression Middleware

### 6.2 Static File Router (Optional)

```rust
if let Some(dir) = static_dir {
    let static_router = create_static_router(&static_config);
    api_router.merge(static_router)
}
```

### 6.3 TCP-Listener binden

```rust
let addr = format!("{}:{}", settings.application.host, settings.application.port);
let listener = TcpListener::bind(&addr).await?;
```

### 6.4 Server starten

```rust
pub async fn run(self) -> Result<(), std::io::Error> {
    axum::serve(
        self.listener,
        self.router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
}
```

**Shutdown-Signale:**
- `Ctrl+C` (SIGINT)
- `SIGTERM` (Unix)

---

## Vollständiger Initialisierungsfluss

```text
┌────────────────────────────────────────────────────────────────────────────┐
│  START: cargo run / ./erynoa-api                                           │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  1. RUNTIME SETUP                                                          │
│     • jemalloc (falls aktiviert)                                           │
│     • tokio multi-thread runtime                                           │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  2. TELEMETRY                                                              │
│     • tracing-subscriber initialisieren                                    │
│     • Log-Level: info                                                      │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  3. CONFIGURATION                                                          │
│     • Settings::load() (Environment + Config)                              │
│     • CLI-Args parsen (--static-dir)                                       │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  4. STORAGE                                                                │
│     • DecentralizedStorage::open(data_dir)                                 │
│     • Fjall LSM-Tree initialisieren                                        │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  5. UNIFIED STATE                                                          │
│     • create_unified_state()                                               │
│     │                                                                      │
│     ├── IdentityState::new()                                              │
│     ├── CoreState::new()                                                  │
│     │   ├── TrustState::new()                                             │
│     │   ├── EventState::new()                                             │
│     │   ├── FormulaState::new()                                           │
│     │   └── ConsensusState::new()                                         │
│     ├── ExecutionState::new()                                             │
│     ├── ECLVMState::new()                                                 │
│     ├── ProtectionState::new()                                            │
│     ├── StorageState::new()                                               │
│     ├── PeerState::new()                                                  │
│     ├── P2PState::new()                                                   │
│     ├── UIState::new() ... BlueprintComposerState::new()                  │
│     ├── StateGraph::erynoa_graph()                                        │
│     ├── EventBus::new()                                                   │
│     ├── CircuitBreaker::new()                                             │
│     ├── StateBroadcaster::new()                                           │
│     ├── MerkleStateTracker::new()                                         │
│     ├── MultiGas::new()                                                   │
│     └── StateEventLog::new()                                              │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  6. STATE COORDINATOR                                                      │
│     • StateCoordinator::new(unified_state)                                │
│     • StateIntegrator erstellen (Observer-Pattern)                        │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  7. APP STATE                                                              │
│     • AppState { unified_state, coordinator, storage, config }            │
│     • started_at = Instant::now()                                         │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  8. ROUTER                                                                 │
│     • create_router(state)                                                │
│     • Middleware: CORS, Tracing, Compression                              │
│     • Optional: static file serving                                        │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  9. NETWORK                                                                │
│     • TcpListener::bind(host:port)                                        │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  10. SERVER RUN                                                            │
│     • axum::serve(listener, router)                                       │
│     • with_graceful_shutdown(SIGINT/SIGTERM)                              │
│     • 🚀 Server läuft!                                                     │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Log-Ausgaben beim Start

```
INFO  🚀 Starting Erynoa API
      version: 0.1.0
      env: development
      port: 8080
      static_dir: None

INFO  🏗️  Building server...
      env: development

INFO  ✅ Decentralized storage ready
      path: ./data

INFO  ✅ Unified state management initialized

INFO  📁 Static file serving enabled (falls aktiviert)
      path: ./static

INFO  🚀 Server ready
      addr: 0.0.0.0:8080
```

---

## Post-Init: Identity Bootstrap

Nach dem Start muss die Identity-Layer explizit gebootstrapped werden:

```rust
// Option 1: Interactive (mit Passkey)
state.identity.bootstrap_interactive(&public_key)?;

// Option 2: Agent-Managed
state.identity.bootstrap_agent(&public_key)?;

// Option 3: Ephemeral (Session)
state.identity.bootstrap_ephemeral(&public_key)?;

// Option 4: Test
state.identity.bootstrap_test(&public_key)?;
```

**Nach Bootstrap:**
- `identity.bootstrap_completed = true`
- Root-DID ist gesetzt
- Device/Agent Sub-DIDs können abgeleitet werden
- Realm-Memberships können erstellt werden

---

## Fehlerbehandlung

| Phase | Fehler | Reaktion |
|-------|--------|----------|
| Config | `Settings::load()` failed | Panic mit Fehlermeldung |
| Storage | Fjall open failed | `Result::Err` propagieren |
| Network | Port bereits belegt | `Result::Err` propagieren |
| Runtime | Panic in async Task | Tokio loggt + Task stirbt |

---

## Shutdown-Sequenz

```text
┌─────────────────────────────────────────────────────────────────┐
│  SIGNAL: Ctrl+C oder SIGTERM                                    │
└────────────────────────────────────┬────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────┐
│  1. Log: "Ctrl+C received" oder "SIGTERM received"              │
└────────────────────────────────────┬────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────┐
│  2. Log: "🛑 Shutting down gracefully..."                       │
└────────────────────────────────────┬────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────┐
│  3. Axum graceful shutdown                                      │
│     • Keine neuen Connections                                   │
│     • Laufende Requests abschließen                             │
└────────────────────────────────────┬────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────┐
│  4. Drop-Reihenfolge (Rust LIFO)                                │
│     • Server dropped                                            │
│     • AppState dropped                                          │
│     • UnifiedState dropped                                      │
│     • DecentralizedStorage dropped → Fjall flush                │
└────────────────────────────────────┬────────────────────────────┘
                                     │
                                     ▼
┌─────────────────────────────────────────────────────────────────┐
│  5. Prozess-Exit mit Code 0                                     │
└─────────────────────────────────────────────────────────────────┘
```

---

---

## P2P-Node Initialisierung (erynoa-testnet-node)

**Datei:** `backend/src/bin/testnet_node.rs`

Das P2P-Binary hat einen **komplett separaten** Initialisierungspfad:

### P2P Startsequenz

```text
┌────────────────────────────────────────────────────────────────────────────┐
│  START: cargo run --features p2p --bin erynoa-testnet-node                 │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  1. LOGGING                                                                │
│     • tracing-subscriber initialisieren                                    │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  2. CLI-ARGS PARSEN                                                        │
│     • --node-name, --p2p-port, --api-port, --mode                         │
│     • --bootstrap-peers, --enable-mdns, --genesis-node                    │
│     • --data-dir                                                           │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  3. STORAGE DIRECTORY                                                      │
│     • std::fs::create_dir_all(&args.data_dir)                             │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  4. KEYPAIR GENERIEREN                                                     │
│     • Keypair::generate_ed25519()                                         │
│     • PeerId = Hash(PublicKey)                                            │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  5. P2P-KONFIGURATION                                                      │
│     • Listen-Adressen: /ip4/0.0.0.0/tcp/{port}                            │
│     • Bootstrap-Peers setzen                                               │
│     • mDNS aktivieren/deaktivieren                                        │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  6. TESTNET SWARM ERSTELLEN                                                │
│     • TestnetSwarm::new(keypair, &config)                                 │
│     │                                                                      │
│     ├── libp2p Transport (TCP + Noise + Yamux)                            │
│     ├── ErynoaBehaviour (Combined NetworkBehaviour):                      │
│     │   ├── identify::Behaviour (Peer-Identifikation)                    │
│     │   ├── kademlia::Behaviour (DHT für Peer-Discovery)                 │
│     │   ├── mdns::Behaviour (lokale Peer-Discovery)                      │
│     │   ├── gossipsub::Behaviour (Pub/Sub Messaging)                     │
│     │   ├── ping::Behaviour (Latenz-Messung)                             │
│     │   ├── relay::Behaviour (NAT-Traversal Server)                      │
│     │   ├── relay::client::Behaviour (NAT-Traversal Client)              │
│     │   ├── dcutr::Behaviour (Direct Connection Upgrade)                 │
│     │   ├── autonat::Behaviour (NAT-Typ-Erkennung)                       │
│     │   ├── upnp::Behaviour (Router Port-Mapping)                        │
│     │   └── request_response::Behaviour (Sync-Protokoll)                 │
│     └── Event-Channel (TestnetEvent → Receiver)                           │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  7. TOKIO TASKS SPAWNEN                                                    │
│     │                                                                      │
│     ├── event_task: Event-Handler (Peers, Gossip, Kad, Relay...)         │
│     ├── api_task: Mini HTTP-Server (/health, /status, /peers)            │
│     └── swarm_task: swarm.run() - Haupt-Event-Loop                        │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  8. SWARM STARTEN                                                          │
│     • Auf konfigurierten Adressen lauschen                                │
│     • Bootstrap-Peers verbinden (falls konfiguriert)                      │
│     • Kademlia-Bootstrap initiieren                                       │
│     • Gossip-Topics subscriben                                            │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      │
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│  9. EVENT-LOOP (tokio::select!)                                            │
│     • Ctrl+C → Shutdown                                                   │
│     • SwarmEvent → Verarbeiten (Connect, Disconnect, Messages...)        │
│     • 🌐 P2P-Node läuft!                                                   │
└────────────────────────────────────────────────────────────────────────────┘
```

### libp2p Behaviour Stack

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         ERYNOA NETWORK BEHAVIOUR                                 │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                          NAT-TRAVERSAL STACK                             │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐      │   │
│  │  │  AutoNAT │ │   UPnP   │ │  Relay   │ │ Relay/   │ │  DCUTR   │      │   │
│  │  │ (Detect) │ │ (Router) │ │ (Server) │ │ Client   │ │(Holepunch│      │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘      │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                          PEER DISCOVERY                                  │   │
│  │  ┌──────────────────────┐ ┌──────────────────────┐ ┌────────────┐      │   │
│  │  │       Kademlia       │ │        mDNS          │ │  Identify  │      │   │
│  │  │   (DHT für Routing)  │ │   (Lokale Discovery) │ │ (PeerInfo) │      │   │
│  │  └──────────────────────┘ └──────────────────────┘ └────────────┘      │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                          MESSAGING                                       │   │
│  │  ┌──────────────────────────────────┐ ┌─────────────────────────────┐   │   │
│  │  │           GossipSub              │ │     Request/Response        │   │   │
│  │  │   (Pub/Sub für Broadcasts)       │ │    (Sync-Protokoll)         │   │   │
│  │  │   Topics: trust, consensus,      │ │    ErynoaProtocol           │   │   │
│  │  │   events, realm_*, privacy       │ │                             │   │   │
│  │  └──────────────────────────────────┘ └─────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
│  ┌─────────────────────────────────────────────────────────────────────────┐   │
│  │                          UTILITIES                                       │   │
│  │  ┌──────────────────────────────────┐                                   │   │
│  │  │             Ping                  │                                   │   │
│  │  │      (Latenz-Messung)             │                                   │   │
│  │  └──────────────────────────────────┘                                   │   │
│  └─────────────────────────────────────────────────────────────────────────┘   │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### P2P Events

Der Event-Handler verarbeitet diese Event-Typen:

| Event | Beschreibung |
|-------|-------------|
| `PeerConnected` | Neuer Peer verbunden (inbound/outbound) |
| `PeerDisconnected` | Peer getrennt |
| `MdnsDiscovered` | Lokaler Peer via mDNS gefunden |
| `KademliaBootstrapComplete` | DHT-Bootstrap abgeschlossen |
| `GossipMessage` | Nachricht über Topic empfangen |
| `GossipMeshPeerAdded/Removed` | Peer joined/left Gossip-Mesh |
| `AutoNatStatus` | NAT-Typ erkannt (Public/Private/Unknown) |
| `ExternalAddressConfirmed` | Öffentliche IP bestätigt |
| `RelayReservation` | Relay-Reservierung (als Client) |
| `RelayCircuitOpened/Closed` | Relay-Circuit (als Server) |
| `DirectConnectionEstablished` | Holepunching erfolgreich |
| `UpnpMapped` | Router Port-Mapping via UPnP |
| `PingResult` | Latenz-Messung zu Peer |

---

## Local Storage Layer

**Datei:** `backend/src/local/mod.rs`

Der dezentrale Storage wird von **beiden Binaries** verwendet:

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         DECENTRALIZED STORAGE                                    │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  DecentralizedStorage::open(data_dir)                                           │
│  │                                                                               │
│  ├── keyspace: Arc<Keyspace>     ← Fjall LSM-Tree Engine                        │
│  │                                                                               │
│  ├── identities: IdentityStore   ← DIDs, Public Keys, DID-Documents            │
│  │                                                                               │
│  ├── events: EventStore          ← Kausaler Event-DAG                           │
│  │                                                                               │
│  ├── trust: TrustStore           ← Trust-Vektoren zwischen Entities            │
│  │                                                                               │
│  ├── content: ContentStore       ← Content Addressable Storage (BLAKE3)        │
│  │                                                                               │
│  └── realm: RealmStorage         ← Dynamische Realm-spezifische Stores         │
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Storage-Initialisierung

```rust
pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
    // 1. Fjall Keyspace öffnen (LSM-Tree Engine)
    let keyspace = Arc::new(fjall::Config::new(path.as_ref().join("data")).open()?);

    // 2. Alle Partitionen initialisieren
    let identities = IdentityStore::new(&keyspace)?;  // Partition: "identities"
    let events = EventStore::new(&keyspace)?;          // Partition: "events"
    let trust = TrustStore::new(&keyspace)?;           // Partition: "trust"
    let content = ContentStore::new(&keyspace)?;       // Partition: "content"
    let realm = RealmStorage::new(&keyspace, config)?; // Partition: "realm_storage"

    Ok(Self { keyspace, identities, events, trust, content, realm })
}
```

---

## Domain Layer

**Datei:** `backend/src/domain/unified/`

Das Domain-Modul enthält **reine Typen und Primitives** - keine Initialisierung nötig:

```text
domain/unified/
├── primitives.rs    ← UniversalId, TemporalCoord, Coordinates
├── identity.rs      ← DID, DIDDocument, Delegation, Capability
├── trust.rs         ← TrustVector, TrustReason, TrustLevel
├── event.rs         ← EventType, EventPayload
├── realm.rs         ← RealmId, RealmConfig, MemberRole
├── saga.rs          ← Saga, SagaStep, SagaAction
├── formula.rs       ← WorldFormula, FormulaParams
├── system.rs        ← SystemMode, EventPriority, AnomalySeverity
├── component.rs     ← StateComponent, StateRelation, ComponentLayer
├── action.rs        ← BlueprintAction, RealmAction, MembershipAction
├── config.rs        ← Konfigurationsstrukturen
├── cost.rs          ← GasCost, ManaCost
├── message.rs       ← NetworkMessage, MessageType
└── schema.rs        ← StoreSchema, SchemaField
```

Diese Typen werden **lazy** bei Bedarf verwendet - keine explizite Initialisierung.

---

## Vollständige Modul-Hierarchie

```text
backend/src/
│
├── main.rs                    ← Entry Point (API-Server)
├── server.rs                  ← AppState, Server::build()
│
├── bin/
│   └── testnet_node.rs        ← Entry Point (P2P Full-Node)
│
├── core/                      ← State + Engines
│   ├── state.rs               ← UnifiedState (21.000+ Zeilen)
│   ├── state_coordination.rs  ← StateCoordinator, Health
│   ├── state_integration.rs   ← StateIntegrator, Observer
│   ├── trust_engine.rs        ← Trust-Berechnungen
│   ├── event_engine.rs        ← Event-DAG-Logik
│   ├── world_formula.rs       ← Weltformel-Berechnungen
│   ├── consensus.rs           ← Konsensus-Logik
│   └── identity_types.rs      ← Identity-Support-Typen
│
├── domain/unified/            ← Reine Typen (kein State)
│
├── local/                     ← Dezentraler Storage (Fjall)
│   ├── mod.rs                 ← DecentralizedStorage
│   ├── identity_store.rs      ← DID-Persistenz
│   ├── event_store.rs         ← Event-DAG-Persistenz
│   ├── trust_store.rs         ← Trust-Persistenz
│   ├── content_store.rs       ← CAS-Storage
│   ├── realm_storage.rs       ← Realm-spezifische Stores
│   └── archive.rs             ← Cold Storage (ψ_archive)
│
├── peer/                      ← P2P + Gateway
│   ├── gateway.rs             ← GatewayGuard, Crossing-Validierung
│   ├── saga_composer.rs       ← Saga-Orchestration
│   ├── intent_parser.rs       ← Intent → Saga
│   └── p2p/                   ← libp2p-Schicht
│       ├── swarm.rs           ← SwarmManager
│       ├── behaviour.rs       ← ErynoaBehaviour
│       ├── testnet.rs         ← TestnetSwarm
│       ├── identity.rs        ← PeerId ↔ DID
│       ├── trust_gate.rs      ← Trust-basierte Peer-Filterung
│       ├── topics.rs          ← GossipSub Topics
│       ├── protocol.rs        ← Sync-Protokoll
│       └── privacy/           ← Onion-Routing, Mixing
│
├── execution/                 ← Execution Context
│   ├── context.rs             ← ExecutionContext
│   ├── tracked.rs             ← TrackedExecution
│   └── error.rs               ← ExecutionError
│
├── eclvm/                     ← Erynoa Core Language VM
│   ├── runtime/               ← VM, Gas, Host
│   ├── compiler.rs            ← ECL → Bytecode
│   └── ...
│
├── protection/                ← Κ19-Κ21
│   ├── anomaly.rs
│   ├── diversity.rs
│   └── anti_calcification.rs
│
└── api/                       ← HTTP-Endpoints
    ├── routes.rs              ← Router-Setup
    └── v1/                    ← API-Handler
```

---

## Referenzen

- **Entry Point API:** `backend/src/main.rs`
- **Entry Point P2P:** `backend/src/bin/testnet_node.rs`
- **Server:** `backend/src/server.rs`
- **State:** `backend/src/core/state.rs` (21.573 Zeilen)
- **Coordinator:** `backend/src/core/state_coordination.rs`
- **Storage:** `backend/src/local/mod.rs`
- **P2P Swarm:** `backend/src/peer/p2p/testnet.rs`
- **P2P Behaviour:** `backend/src/peer/p2p/behaviour.rs`
- **Domain Types:** `backend/src/domain/unified/`
