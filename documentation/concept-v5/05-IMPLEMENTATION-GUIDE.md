# Implementation Guide

> **Version:** V5.0 – Konsolidiert
> **Status:** Referenz

---

## Übersicht

Dieser Guide beschreibt die technische Implementierung des Erynoa-Systems in Rust.

---

## I. Technologie-Stack

| Komponente     | Technologie         | Begründung                         |
| -------------- | ------------------- | ---------------------------------- |
| Sprache        | Rust                | Sicherheit, Performance, Ownership |
| Async Runtime  | Tokio               | Performantes async I/O             |
| P2P Networking | libp2p              | Dezentrales Networking             |
| Storage        | Fjall (embedded KV) | Single-Binary, LSM-Tree, ACID      |
| Serialisierung | CBOR / MessagePack  | Kompakt, schema-less               |
| Kryptographie  | Ed25519, BLS12-381  | Signaturen, Aggregation            |
| CLI Framework  | Clap                | Ergonomische CLI                   |
| Content Hash   | BLAKE3              | Content-addressable Storage        |

---

## II. Projekt-Struktur

```
backend/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library root
│   ├── server.rs               # HTTP/gRPC server
│   ├── telemetry.rs            # Tracing/Metrics
│   ├── error.rs                # Error types
│   │
│   ├── core/                   # Core Layer (Κ2-Κ18)
│   │   ├── mod.rs              # Re-exports & Modul-Doku
│   │   ├── trust_engine.rs     # TrustEngine (Κ2-Κ5), 755 Zeilen
│   │   ├── event_engine.rs     # EventEngine (Κ9-Κ12), 733 Zeilen
│   │   ├── world_formula.rs    # WorldFormulaEngine (Κ15b-d), 727 Zeilen
│   │   ├── surprisal.rs        # SurprisalCalculator (Κ15a), 334 Zeilen
│   │   ├── consensus.rs        # ConsensusEngine (Κ18)
│   │   ├── state.rs            # UnifiedState + StateGraph, 4389 Zeilen
│   │   ├── state_integration.rs # StateIntegrator (Observer)
│   │   ├── state_coordination.rs # StateCoordinator
│   │   └── engine.rs           # ExecutionContext Wrapper
│   │
│   ├── domain/                 # Domain Models
│   │   ├── mod.rs
│   │   └── unified/            # Unified Domain Models
│   │       ├── identity.rs     # DID, Credentials (Κ6-Κ8)
│   │       ├── realm.rs        # Realm, Partition (Κ1)
│   │       ├── trust.rs        # TrustVector6D, ContextType
│   │       └── ...             # Weitere Domain-Typen
│   │
│   ├── protection/             # Protection Layer (Κ19-Κ21)
│   │   ├── mod.rs
│   │   ├── anti_calcification.rs  # AntiCalcification (Κ19)
│   │   ├── adaptive_calibration.rs # Dynamische Parameter (Κ19, §IX)
│   │   ├── diversity.rs           # DiversityMonitor (Κ20)
│   │   ├── quadratic.rs           # QuadraticGovernance (Κ21)
│   │   └── anomaly.rs             # AnomalyDetector
│   │
│   ├── peer/                   # Peer Layer (Κ22-Κ24)
│   │   ├── mod.rs
│   │   ├── gateway.rs          # GatewayGuard (Κ23)
│   │   ├── saga_composer.rs    # SagaComposer (Κ22)
│   │   ├── intent_parser.rs    # IntentParser
│   │   └── p2p/                # libp2p Integration
│   │       ├── mod.rs
│   │       ├── swarm.rs        # SwarmManager
│   │       ├── gossip.rs       # GossipSub
│   │       ├── sync.rs         # SyncProtocol
│   │       └── diagnostics.rs  # SystemState Observer
│   │
│   ├── execution/              # Execution Layer
│   │   ├── mod.rs
│   │   ├── context.rs          # ExecutionContext (Gas+Mana+Events)
│   │   ├── tracked.rs          # Tracked Execution
│   │   └── information_loss.rs # Informationsverlust-Tracking
│   │
│   ├── eclvm/                  # ECLVM - Erynoa Configuration Language VM
│   │   ├── mod.rs              # Re-exports & Pipeline-Doku
│   │   ├── ast.rs              # Abstract Syntax Tree
│   │   ├── parser.rs           # Lexer & Parser
│   │   ├── compiler.rs         # AST → Bytecode
│   │   ├── bytecode.rs         # OpCode & Value Definitionen
│   │   ├── optimizer.rs        # Bytecode-Optimierung
│   │   ├── runtime/            # VM Runtime
│   │   │   ├── vm.rs           # ECLVM Stack-Maschine
│   │   │   ├── gas.rs          # GasMeter
│   │   │   └── host.rs         # HostInterface
│   │   ├── stdlib.rs           # Standard Library & PolicyBuilder
│   │   ├── mana.rs             # ManaManager, BandwidthTier
│   │   ├── bridge.rs           # CoreToEclvm, EclvmToCore Bridge
│   │   ├── erynoa_host.rs      # ErynoaHost, PolicyContext
│   │   ├── programmable_gateway.rs # ProgrammableGateway mit Policies
│   │   └── cli.rs              # ECLVM CLI (feature-gated)
│   │
│   ├── local/                  # Dezentraler Storage Layer
│   │   ├── mod.rs              # DecentralizedStorage Manager
│   │   ├── kv_store.rs         # Generic Key-Value Store
│   │   ├── event_store.rs      # Event-DAG Persistence
│   │   ├── identity_store.rs   # DID & Key Storage
│   │   ├── trust_store.rs      # Trust-Vektor Persistence
│   │   ├── content_store.rs    # Content-Addressable (BLAKE3)
│   │   ├── realm_storage.rs    # Per-Realm dynamische Stores
│   │   ├── archive.rs          # Cold Storage (ψ_archive Morphismus)
│   │   └── blueprint_marketplace.rs # Blueprint Store & Marketplace
│   │
│   ├── api/                    # API Layer
│   │   ├── mod.rs
│   │   ├── grpc.rs
│   │   └── rest.rs
│   │
│   └── gen/                    # Generated Code (Protobuf)
│       └── erynoa/
│
├── config/
│   ├── base.toml
│   ├── local.toml
│   └── production.toml
│
├── proto/                      # Protobuf definitions
│   └── erynoa/
│
└── tests/
    ├── api.rs
    ├── property_tests.rs
    └── unified_integration.rs
```

---

## III. Kern-Implementierungen

### 3.1 TrustEngine (Κ2-Κ5)

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct TrustEngine {
    vectors: RwLock<HashMap<DID, TrustVector6D>>,
    context_weights: HashMap<ContextType, [f64; 6]>,
    asymmetry_factors: [f64; 6],
}

#[derive(Clone, Debug, Default)]
pub struct TrustVector6D {
    pub r: f64,      // Reliability
    pub i: f64,      // Integrity
    pub c: f64,      // Competence
    pub p: f64,      // Prestige
    pub v: f64,      // Vigilance
    pub omega: f64,  // Omega
}

impl TrustEngine {
    pub fn new() -> Self {
        Self {
            vectors: RwLock::new(HashMap::new()),
            context_weights: Self::default_context_weights(),
            asymmetry_factors: [1.5, 1.5, 1.5, 1.5, 2.0, 2.0],
        }
    }

    /// Κ4: Asymmetrische Evolution
    pub fn update(&self, did: &DID, dimension: TrustDimension, delta: f64, is_positive: bool) {
        let mut vectors = self.vectors.write().unwrap();
        let vector = vectors.entry(did.clone()).or_default();

        let effective_delta = if is_positive {
            delta
        } else {
            delta * self.asymmetry_factors[dimension as usize]
        };

        let current = vector.get_dimension(dimension);
        let new_value = if is_positive {
            (current + effective_delta).min(1.0)
        } else {
            (current - effective_delta).max(0.01)
        };

        vector.set_dimension(dimension, new_value);
    }

    /// Κ5: Probabilistische Kombination
    pub fn combine(t1: f64, t2: f64) -> f64 {
        1.0 - (1.0 - t1) * (1.0 - t2)
    }

    /// Κ15b: Gewichtete Norm
    pub fn weighted_norm(&self, vector: &TrustVector6D, context: ContextType) -> f64 {
        let weights = self.context_weights.get(&context)
            .unwrap_or(&[1.0/6.0; 6]);

        let components = [vector.r, vector.i, vector.c, vector.p, vector.v, vector.omega];
        let sum: f64 = components.iter()
            .zip(weights.iter())
            .map(|(v, w)| w * v * v)
            .sum();

        sum.sqrt()
    }

    /// Τ1: Ketten-Trust
    pub fn chain_trust(chain: &[f64]) -> f64 {
        if chain.is_empty() {
            return 0.0;
        }
        let n = chain.len() as f64;
        let log_sum: f64 = chain.iter().map(|t| t.ln()).sum();
        (log_sum / n.sqrt()).exp()
    }
}
```

### 3.2 EventEngine (Κ9-Κ12)

```rust
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct EventEngine {
    dag: RwLock<CausalDAG>,
    finality_tracker: FinalityTracker,
    witness_manager: WitnessManager,
    lamport_clock: AtomicU64,
}

pub struct CausalDAG {
    events: HashMap<EventId, Event>,
    children: HashMap<EventId, Vec<EventId>>,
}

impl EventEngine {
    /// Κ12: Event-Erzeugung
    pub fn create_event(
        &self,
        author: &DID,
        payload: Payload,
        parents: Vec<EventId>,
    ) -> Result<Event, EventError> {
        let dag = self.dag.read().unwrap();

        // Κ9: Validate no cycles
        for parent in &parents {
            if !dag.events.contains_key(parent) {
                return Err(EventError::UnknownParent(*parent));
            }
        }

        let timestamp = self.lamport_clock.fetch_add(1, Ordering::SeqCst);

        let event = Event {
            id: EventId::from_content(&payload, &parents),
            parents,
            author: author.clone(),
            payload,
            timestamp,
            finality: FinalityLevel::Nascent,
        };

        drop(dag);

        let mut dag = self.dag.write().unwrap();
        dag.insert(event.clone());

        Ok(event)
    }

    /// Κ10: Finalität erhöhen
    pub fn witness_event(
        &self,
        event_id: &EventId,
        witness: &DID,
    ) -> Result<FinalityLevel, WitnessError> {
        self.witness_manager.add_witness(event_id, witness);

        let count = self.witness_manager.count(event_id);
        let threshold = self.finality_tracker.threshold();

        if count >= threshold {
            let mut dag = self.dag.write().unwrap();
            if let Some(event) = dag.events.get_mut(event_id) {
                event.finality = event.finality.promote();
                return Ok(event.finality);
            }
        }

        Ok(FinalityLevel::Nascent)
    }

    /// Κ9: Ancestor check
    pub fn is_ancestor(&self, ancestor: &EventId, descendant: &EventId) -> bool {
        let dag = self.dag.read().unwrap();
        self.dfs_ancestor(&dag, ancestor, descendant, &mut HashSet::new())
    }

    fn dfs_ancestor(
        &self,
        dag: &CausalDAG,
        target: &EventId,
        current: &EventId,
        visited: &mut HashSet<EventId>,
    ) -> bool {
        if current == target {
            return true;
        }
        if visited.contains(current) {
            return false;
        }
        visited.insert(*current);

        if let Some(event) = dag.events.get(current) {
            for parent in &event.parents {
                if self.dfs_ancestor(dag, target, parent, visited) {
                    return true;
                }
            }
        }
        false
    }
}
```

### 3.3 WorldFormulaEngine (Κ15)

```rust
pub struct WorldFormulaEngine {
    trust_engine: Arc<TrustEngine>,
    surprisal_calc: Arc<SurprisalCalculator>,
    event_engine: Arc<EventEngine>,
    current_e: AtomicF64,
    tau_update: Duration,
}

impl WorldFormulaEngine {
    /// Κ15b: Vollständige Formel
    pub fn compute_contribution(
        &self,
        s: &DID,
        context: ContextType,
        t: Timestamp,
    ) -> f64 {
        // 𝔸(s)
        let activity = self.compute_activity(s);

        // ‖𝕎(s)‖_w
        let trust = self.trust_engine.get_vector(s);
        let trust_norm = self.trust_engine.weighted_norm(&trust, context);

        // |ℂ(s)|
        let causal_size = self.event_engine.causal_history_size(s) as f64;

        // 𝒮(s) = ‖𝕎‖² · ℐ  (Κ15a)
        let surprisal = self.surprisal_calc.dampened_surprisal(s, trust_norm);

        // Ĥ(s) (Κ16)
        let human_factor = self.get_human_factor(s);

        // w(s,t) (Κ17)
        let temporal = self.compute_temporal_weight(s, t);

        // σ⃗(x) (Κ15c)
        let inner = trust_norm * causal_size.ln().max(1.0) * surprisal;
        let sigmoid = 1.0 / (1.0 + (-inner).exp());

        activity * sigmoid * human_factor * temporal
    }

    fn compute_activity(&self, s: &DID) -> f64 {
        let recent = self.event_engine.recent_event_count(s);
        let kappa = 10.0;
        recent as f64 / (recent as f64 + kappa)
    }

    fn get_human_factor(&self, s: &DID) -> f64 {
        match self.get_human_status(s) {
            HumanStatus::VerifiedHuman => 1.5,
            HumanStatus::HumanControlled => 1.2,
            HumanStatus::Unknown => 1.0,
        }
    }

    /// Κ15d: Streaming Approximation
    pub fn update_streaming(&self, new_events: &[Event]) {
        let now = Timestamp::now();
        let dt = (now - self.last_update()).as_secs_f64();
        let alpha = (-dt / self.tau_update.as_secs_f64()).exp();

        let new_contribution: f64 = new_events.iter()
            .map(|e| self.compute_contribution(&e.author, ContextType::Default, now))
            .sum();

        let current = self.current_e.load(Ordering::Relaxed);
        let updated = alpha * current + (1.0 - alpha) * new_contribution;
        self.current_e.store(updated, Ordering::Relaxed);
    }
}
```

### 3.4 SurprisalCalculator (Κ15a)

```rust
pub struct SurprisalCalculator {
    count_min_sketch: CountMinSketch,
    minhash: MinHash,
    recent_count: AtomicU64,
}

impl SurprisalCalculator {
    /// Κ15a: Shannon-Surprisal mit Trust-Dämpfung
    pub fn calculate_surprisal(&self, event: &Event) -> f64 {
        let fingerprint = self.minhash.hash(&EventFeatures::from(event));
        let count = self.count_min_sketch.query(&fingerprint) as f64;
        let total = self.recent_count.load(Ordering::Relaxed) as f64;

        // Laplace smoothing
        let probability = (count + 1.0) / (total + 1.0);

        // ℐ = −log₂ P
        -probability.log2()
    }

    /// Trust-gedämpft (Anti-Hype)
    pub fn dampened_surprisal(&self, s: &DID, trust_norm: f64) -> f64 {
        let surprisal = self.calculate_surprisal_for_subject(s);
        // 𝒮 = ‖𝕎‖² · ℐ
        trust_norm.powi(2) * surprisal
    }
}

/// Count-Min Sketch für effiziente Frequenz-Schätzung
pub struct CountMinSketch {
    tables: Vec<Vec<u64>>,
    width: usize,
    depth: usize,
    hash_seeds: Vec<u64>,
}

impl CountMinSketch {
    pub fn new(width: usize, depth: usize) -> Self {
        Self {
            tables: vec![vec![0; width]; depth],
            width,
            depth,
            hash_seeds: (0..depth).map(|i| i as u64 * 0x9e3779b97f4a7c15).collect(),
        }
    }

    pub fn insert(&mut self, item: &[u8]) {
        for (i, table) in self.tables.iter_mut().enumerate() {
            let hash = self.hash(item, self.hash_seeds[i]) as usize % self.width;
            table[hash] = table[hash].saturating_add(1);
        }
    }

    pub fn query(&self, item: &[u8]) -> u64 {
        self.tables.iter().enumerate()
            .map(|(i, table)| {
                let hash = self.hash(item, self.hash_seeds[i]) as usize % self.width;
                table[hash]
            })
            .min()
            .unwrap_or(0)
    }

    fn hash(&self, item: &[u8], seed: u64) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        item.hash(&mut hasher);
        hasher.finish()
    }
}
```

---

## IV. Deployment-Konfiguration

### 4.1 Cargo.toml

```toml
[package]
name = "erynoa-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
axum = "0.7"
tonic = "0.11"
prost = "0.12"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ciborium = "0.2"  # CBOR
libp2p = { version = "0.53", features = ["tokio", "gossipsub", "kad", "mdns"] }
ed25519-dalek = "2.1"
sha3 = "0.10"
rocksdb = "0.21"
tracing = "0.1"
tracing-subscriber = "0.3"
config = "0.13"

[dev-dependencies]
proptest = "1.4"
criterion = "0.5"
```

### 4.2 Konfiguration (base.toml)

```toml
[server]
host = "0.0.0.0"
port = 8080
grpc_port = 50051

[storage]
type = "rocksdb"
path = "./data"

[p2p]
listen = "/ip4/0.0.0.0/tcp/4001"
bootstrap = [
    "/dnsaddr/bootstrap.erynoa.network/p2p/..."
]

[trust]
default_vector = [0.5, 0.5, 0.5, 0.5, 0.5, 0.5]
asymmetry_factors = [1.5, 1.5, 1.5, 1.5, 2.0, 2.0]

[formula]
tau_update = "1h"
approximation = "cms"  # count-min-sketch

[protection]
gini_threshold = 0.5
diversity_min_entropy = 0.7
anti_calc_gamma = 0.7
```

---

## V. Testing

### 5.1 Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn trust_asymmetry_holds(
        delta in 0.01..0.5f64,
        initial in 0.1..0.9f64,
    ) {
        let engine = TrustEngine::new();
        let did = DID::random();

        // Set initial
        engine.set_trust(&did, TrustDimension::Reliability, initial);

        // Positive update
        engine.update(&did, TrustDimension::Reliability, delta, true);
        let after_positive = engine.get_trust(&did, TrustDimension::Reliability);

        // Reset
        engine.set_trust(&did, TrustDimension::Reliability, initial);

        // Negative update
        engine.update(&did, TrustDimension::Reliability, delta, false);
        let after_negative = engine.get_trust(&did, TrustDimension::Reliability);

        // Κ4: Negative impact should be larger
        let positive_delta = (after_positive - initial).abs();
        let negative_delta = (initial - after_negative).abs();

        prop_assert!(negative_delta >= positive_delta * 1.4);
    }

    #[test]
    fn trust_combination_increases(t1 in 0.01..0.99f64, t2 in 0.01..0.99f64) {
        let combined = TrustEngine::combine(t1, t2);
        // Κ5: Combined trust should be >= max of inputs
        prop_assert!(combined >= t1.max(t2));
        prop_assert!(combined <= 1.0);
    }

    #[test]
    fn dag_no_cycles(events in prop::collection::vec(any::<EventPayload>(), 1..100)) {
        let engine = EventEngine::new();
        let author = DID::random();
        let mut ids = vec![];

        for (i, payload) in events.into_iter().enumerate() {
            let parents = if i == 0 {
                vec![]
            } else {
                vec![ids[i - 1]]
            };

            let event = engine.create_event(&author, payload, parents).unwrap();
            ids.push(event.id);
        }

        // Κ9: No self-ancestry
        for id in &ids {
            prop_assert!(!engine.is_ancestor(id, id));
        }
    }
}
```

### 5.2 Integration Tests

```rust
#[tokio::test]
async fn full_transaction_lifecycle() {
    let system = TestSystem::new().await;

    // Create identities
    let alice = system.create_identity("alice").await;
    let bob = system.create_identity("bob").await;

    // Build trust
    system.attest(&alice, &bob, TrustDimension::Reliability, 0.1).await;

    // TAT Lifecycle (Κ13)
    let proposal = system.propose(&alice, &bob, Amount::new(100)).await;
    system.agree(&bob, &proposal, true).await;

    // Verify trust increased
    let trust = system.get_trust(&alice, &bob).await;
    assert!(trust.r > 0.5);
}
```

---

## VI. Monitoring & Telemetry

```rust
use tracing::{info, instrument, span, Level};
use metrics::{counter, gauge, histogram};

#[instrument(skip(self))]
pub fn update_trust(&self, did: &DID, dimension: TrustDimension, delta: f64, positive: bool) {
    let span = span!(Level::DEBUG, "trust_update", %did, ?dimension);
    let _enter = span.enter();

    // Update metrics
    if positive {
        counter!("trust.updates.positive").increment(1);
    } else {
        counter!("trust.updates.negative").increment(1);
    }

    // Actual update
    self.do_update(did, dimension, delta, positive);

    // Record latency
    histogram!("trust.update.duration").record(elapsed);

    info!("Trust updated");
}
```

---

_Weiter zu [06-CLI-REFERENCE.md](06-CLI-REFERENCE.md) für die Kommando-Referenz._
