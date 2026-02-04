# 🧬 Synergistische System-Integration

> **Teil von:** Projekt Pluto
> **Version:** 2.0.0
> **Fokus:** Nahtlose Kopplung aller Module über das Nervensystem

---

## 1. Vision: Das Lebendige Backend

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                         ERYNOA NERVENSYSTEM                                  │
│                                                                              │
│   Ein lebendiger Organismus, in dem jedes Modul wie ein Organ arbeitet:    │
│                                                                              │
│   🧠 UnifiedState     = Gehirn (Zentrale Koordination)                      │
│   🔌 SynapseHub       = Synapsen (Signalübertragung)                        │
│   ⚙️  Engines          = Muskeln (Ausführung)                                │
│   🛡️  Protection       = Immunsystem (Schutz)                                │
│   💾 Storage          = Gedächtnis (Persistenz)                             │
│   🌐 P2P              = Nervenbahnen (Kommunikation)                        │
│   🏛️  Realm            = Organe (Isolation & Spezialisierung)               │
│   📜 Domain           = DNA (Typen & Invarianten)                           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Kern-Architektur: Dynamische Event-Kopplung

### 2.1 Das Nervensystem im Detail

```text
                                    ┌─────────────────────┐
                                    │   UnifiedState      │
                                    │   (21.495 Zeilen)   │
                                    │                     │
                                    │ • TrustState        │
                                    │ • EventState        │
                                    │ • IdentityState     │
                                    │ • ECLVMState        │
                                    │ • ProtectionState   │
                                    │ • RealmState        │
                                    │ • P2PState          │
                                    │ • StorageState      │
                                    └──────────┬──────────┘
                                               │
                                               │ StateEvent
                                               │ (42 Varianten)
                                               ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│                              SYNAPSE HUB                                      │
│                                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ EventBus    │  │ StateGraph  │  │ Observer    │  │ Adapter     │         │
│  │ (Ingress/   │  │ (Dependency │  │ Registry    │  │ Factory     │         │
│  │  Egress)    │  │  Graph)     │  │ (30+ Traits)│  │             │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                │                 │
│         └────────────────┴────────────────┴────────────────┘                 │
│                                    │                                         │
│                           dispatch(event)                                    │
│                                    │                                         │
│         ┌──────────────────────────┼──────────────────────────┐              │
│         │                          │                          │              │
│         ▼                          ▼                          ▼              │
│  ┌─────────────┐           ┌─────────────┐           ┌─────────────┐        │
│  │ Direct      │           │ Transitive  │           │ Aggregate   │        │
│  │ Observers   │           │ Observers   │           │ Observers   │        │
│  │ (via DependsOn)         │ (via Triggers)          │ (via Aggregates)     │
│  └─────────────┘           └─────────────┘           └─────────────┘        │
└──────────────────────────────────────────────────────────────────────────────┘
                                    │
         ┌──────────────────────────┼──────────────────────────┐
         │                          │                          │
         ▼                          ▼                          ▼
┌─────────────────┐        ┌─────────────────┐        ┌─────────────────┐
│    ENGINES      │        │    SERVICES     │        │   PROTECTION    │
│                 │        │                 │        │                 │
│ • TrustEngine   │        │ • Gateway       │        │ • AnomalyDetect │
│ • EventEngine   │        │ • SagaComposer  │        │ • Diversity     │
│ • FormulaEngine │        │ • DecStorage    │        │ • Calibration   │
│ • ConsensusEng  │        │ • SwarmManager  │        │ • AntiCalc      │
└─────────────────┘        └─────────────────┘        └─────────────────┘
```

---

## 3. StateComponent → Modul Mapping

### 3.1 Vollständige Komponenten-Tabelle (37 Komponenten)

| StateComponent | Modul (IST) | Modul (SOLL) | Layer | Kritikalität |
|----------------|-------------|--------------|-------|--------------|
| **Identity** | `core/state.rs` | `nervous_system/components/identity.rs` | Core | 🔴 Kritisch |
| **Trust** | `core/state.rs` | `nervous_system/components/core.rs` | Core | 🔴 Kritisch |
| **Event** | `core/state.rs` | `nervous_system/components/core.rs` | Core | 🔴 Kritisch |
| **Formula** | `core/state.rs` | `nervous_system/components/core.rs` | Core | 🟡 Hoch |
| **Consensus** | `core/state.rs` | `nervous_system/components/core.rs` | Core | 🔴 Kritisch |
| **Gas** | `core/state.rs` | `execution/gas/mod.rs` | Execution | 🟡 Hoch |
| **Mana** | `core/state.rs` | `execution/mana/mod.rs` | Execution | 🟡 Hoch |
| **Execution** | `execution/` | `execution/mod.rs` | Execution | 🟡 Hoch |
| **ECLVM** | `eclvm/` | `eclvm/mod.rs` | Engine | 🟡 Hoch |
| **ECLPolicy** | `eclvm/` | `eclvm/mod.rs` | Engine | 🟢 Mittel |
| **ECLBlueprint** | `eclvm/` | `eclvm/mod.rs` | Engine | 🟢 Mittel |
| **Anomaly** | `protection/` | `protection/anomaly/mod.rs` | Protection | 🔴 Kritisch |
| **Diversity** | `protection/` | `protection/diversity/mod.rs` | Protection | 🟡 Hoch |
| **Quadratic** | `protection/` | `protection/quadratic/mod.rs` | Protection | 🟢 Mittel |
| **AntiCalcification** | `protection/` | `protection/anti_calc/mod.rs` | Protection | 🟡 Hoch |
| **Calibration** | `protection/` | `protection/calibration/mod.rs` | Protection | 🟢 Mittel |
| **Realm** | `peer/` | `realm/mod.rs` | Peer | 🔴 Kritisch |
| **Gateway** | `peer/` | `realm/gateway/mod.rs` | Peer | 🔴 Kritisch |
| **SagaComposer** | `peer/` | `realm/saga/mod.rs` | Peer | 🟡 Hoch |
| **IntentParser** | `peer/` | `realm/intent/mod.rs` | Peer | 🟢 Mittel |
| **Room** | `peer/` | `realm/room/mod.rs` | Peer | 🟢 Mittel |
| **Swarm** | `peer/p2p/` | `p2p/swarm/mod.rs` | P2P | 🔴 Kritisch |
| **Gossip** | `peer/p2p/` | `p2p/gossip/mod.rs` | P2P | 🟡 Hoch |
| **DHT** | `peer/p2p/` | `p2p/dht/mod.rs` | P2P | 🟡 Hoch |
| **Relay** | `peer/p2p/` | `p2p/relay/mod.rs` | P2P | 🟢 Mittel |
| **Privacy** | `peer/p2p/` | `p2p/privacy/mod.rs` | P2P | 🟡 Hoch |
| **TrustGate** | `peer/p2p/` | `p2p/trust_gate/mod.rs` | P2P | 🔴 Kritisch |
| **Storage** | `local/` | `storage/mod.rs` | Storage | 🔴 Kritisch |
| **EventStore** | `local/` | `storage/event_store/mod.rs` | Storage | 🟡 Hoch |
| **IdentityStore** | `local/` | `storage/identity_store/mod.rs` | Storage | 🔴 Kritisch |
| **TrustStore** | `local/` | `storage/trust_store/mod.rs` | Storage | 🟡 Hoch |
| **ContentStore** | `local/` | `storage/content_store/mod.rs` | Storage | 🟢 Mittel |
| **Archive** | `local/` | `storage/archive/mod.rs` | Storage | 🟢 Mittel |
| **UI** | - (NEU) | `engines/ui/mod.rs` | Engine | 🟢 Mittel |
| **API** | - (NEU) | `engines/api/mod.rs` | Engine | 🟡 Hoch |
| **Governance** | - (NEU) | `engines/governance/mod.rs` | Engine | 🟡 Hoch |
| **Controller** | - (NEU) | `engines/controller/mod.rs` | Engine | 🔴 Kritisch |

---

## 4. Observer-Traits → StateEvent Mapping

### 4.1 Trait-zu-Event Korrelation

```rust
// Jeder Observer-Trait korrespondiert mit StateEvent-Varianten

trait TrustObserver {
    fn on_trust_update(...) → StateEvent::TrustUpdate { ... }
    fn on_entity_registered(...) → StateEvent::IdentityBootstrapped { ... }
    fn on_violation_detected(...) → StateEvent::TrustViolation { ... }
}

trait EventObserver {
    fn on_event_added(...) → StateEvent::EventAdded { ... }
    fn on_event_finalized(...) → StateEvent::EventFinalized { ... }
}

trait ProtectionObserver {
    fn on_anomaly_detected(...) → StateEvent::AnomalyDetected { ... }
    fn on_entropy_update(...) → StateEvent::EntropyUpdate { ... }
}

trait RealmObserver {
    fn on_crossing_succeeded(...) → StateEvent::CrossingEvaluated { ... }
    fn on_realm_registered(...) → StateEvent::RealmRegistered { ... }
}

trait StorageObserver {
    fn on_event_persisted(...) → StateEvent::EventPersisted { ... }
    fn on_archived(...) → StateEvent::ArchiveCompleted { ... }
}

trait P2PObserver {
    fn on_peer_connected(...) → StateEvent::PeerConnectionChange { ... }
    fn on_gossip_received(...) → StateEvent::NetworkMetricUpdate { ... }
}
```

---

## 5. Dynamische Abhängigkeitsketten

### 5.1 StateGraph Relationen

```rust
// Aus domain/unified/component.rs - StateR

pub enum StateRelation {
    /// A hängt von B ab (B muss vor A initialisiert sein)
    DependsOn,
    /// A löst Updates in B aus
    Triggers,
    /// A enthält/aggregiert B
    Aggregates,
    /// A validiert B
    Validates,
    /// Bidirektionale Abhängigkeit
    Bidirectional,
}
```

### 5.2 Beispiel: Trust-Update Kaskade

```text
TrustEngine.update_trust(entity, +0.1)
    │
    ▼
┌─────────────────┐
│ StateEvent::    │
│ TrustUpdate     │
│ {               │
│   entity,       │
│   old: 0.5,     │
│   new: 0.6,     │
│   reason        │
│ }               │
└────────┬────────┘
         │
         │ SynapseHub.dispatch()
         │
         ├─────────────────────────────────────┐
         │                                     │
         ▼                                     ▼
┌─────────────────┐                   ┌─────────────────┐
│ TrustState      │                   │ RealmState      │
│ .apply_event()  │                   │ .apply_event()  │
│                 │                   │                 │
│ Relation:       │                   │ Relation:       │
│ Direct Owner    │                   │ DependsOn       │
└────────┬────────┘                   └────────┬────────┘
         │                                     │
         │ Triggers                            │ Triggers
         │                                     │
         ▼                                     ▼
┌─────────────────┐                   ┌─────────────────┐
│ GatewayState    │                   │ QuotaEnforcer   │
│ .recalc_access()│                   │ .update_limits()│
│                 │                   │                 │
│ → Entity kann   │                   │ → Entity erhält │
│   neue Realms   │                   │   mehr Quota    │
│   betreten      │                   │                 │
└────────┬────────┘                   └─────────────────┘
         │
         │ Triggers
         │
         ▼
┌─────────────────┐
│ ECLVMState      │
│ .update_budget()│
│                 │
│ → Entity erhält │
│   mehr Gas/Mana │
└─────────────────┘
```

---

## 6. Modul-spezifische Integration

### 6.1 P2P ↔ Nervensystem

```rust
// p2p/swarm/manager.rs

impl SwarmManager {
    /// StateEventEmitter für Integration mit UnifiedState
    state_event_emitter: Arc<dyn StateEventEmitter>,

    async fn handle_swarm_event(&self, event: SwarmEvent) {
        match event {
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                // 1. Trust-Gate prüfen
                let decision = self.trust_gate.check_connection(&peer_id);

                // 2. StateEvent emittieren
                self.state_event_emitter.emit(StateEvent::PeerConnectionChange {
                    peer_id: peer_id.to_string(),
                    peer_universal_id: self.trust_gate.get_universal_id(&peer_id),
                    connected: true,
                    addr: Some(addr),
                    connection_level: Some(format!("{:?}", decision.level)),
                });

                // 3. Metrik-Update
                self.state_event_emitter.emit(StateEvent::NetworkMetricUpdate {
                    metric: NetworkMetric::ConnectedPeers,
                    value: self.connected_peers.load(Ordering::Relaxed),
                    delta: 1,
                });
            }
            // ...
        }
    }
}
```

### 6.2 Storage ↔ Nervensystem

```rust
// storage/mod.rs

impl DecentralizedStorage {
    /// Aktualisiert StorageState in UnifiedState
    pub fn update_storage_state(&self, state: &UnifiedState) {
        state.storage.with_write(|s| {
            s.identity_count.store(self.identity_count(), Ordering::Relaxed);
            s.event_count.store(self.event_count(), Ordering::Relaxed);
            s.trust_count.store(self.trust_count(), Ordering::Relaxed);
            s.content_count.store(self.content_count(), Ordering::Relaxed);
            s.health_score.store(
                (self.health_score() * 1000.0) as u64,
                Ordering::Relaxed
            );
        });
    }

    /// Event persistieren mit StateEvent-Emission
    pub async fn persist_event(&self, event: &Event, emitter: &dyn StateEventEmitter) {
        let size = self.event_store.put(event)?;

        emitter.emit(StateEvent::EventPersisted {
            event_id: event.id.clone(),
            size_bytes: size,
            timestamp: event.temporal.unix_timestamp,
        });
    }
}
```

### 6.3 Protection ↔ Nervensystem

```rust
// protection/anomaly/detector.rs

impl AnomalyDetector {
    /// Analysiere Event und emittiere StateEvents bei Anomalien
    pub fn analyze_with_state(
        &mut self,
        event: &Event,
        emitter: &dyn StateEventEmitter
    ) -> Vec<Anomaly> {
        let anomalies = self.analyze_event(event);

        for anomaly in &anomalies {
            emitter.emit(StateEvent::AnomalyDetected {
                anomaly_type: format!("{:?}", anomaly.anomaly_type),
                severity: format!("{:?}", anomaly.severity),
                subject: anomaly.subject.clone(),
                description: anomaly.description.clone(),
                suggested_action: self.recommend_action(anomaly),
            });

            // Bei kritischen Anomalien: CircuitBreaker aktivieren
            if anomaly.severity == Severity::Critical {
                emitter.emit(StateEvent::SystemModeChange {
                    old_mode: SystemMode::Normal,
                    new_mode: SystemMode::Degraded,
                    reason: anomaly.description.clone(),
                });
            }
        }

        anomalies
    }
}
```

### 6.4 Execution ↔ Nervensystem

```rust
// execution/context.rs

impl ExecutionContext {
    /// Observer für State-Integration
    observer: Option<Arc<dyn ExecutionObserver>>,

    /// Gas verbrauchen mit StateEvent
    pub fn consume_gas(&mut self, amount: u64) -> ExecutionResult<()> {
        if self.gas_used + amount > self.gas_budget {
            if let Some(ref obs) = self.observer {
                obs.on_out_of_gas(amount, self.gas_budget - self.gas_used);
            }
            return Err(ExecutionError::OutOfGas {
                required: amount,
                available: self.gas_budget - self.gas_used
            });
        }

        self.gas_used += amount;

        if let Some(ref obs) = self.observer {
            obs.on_gas_consumed(amount);
        }

        Ok(())
    }

    /// Finalisiere und emittiere Summary
    pub fn finalize(self) -> ExecutionSummary {
        let summary = ExecutionSummary {
            gas_used: self.gas_used,
            mana_used: self.mana_used,
            events_emitted: self.events.len() as u64,
            duration: self.started_at.elapsed(),
            success: !self.is_timed_out(),
        };

        if let Some(ref obs) = self.observer {
            obs.on_execution_complete(
                self.context_id,
                summary.success,
                summary.gas_used,
                summary.mana_used,
                summary.events_emitted,
                summary.duration.as_millis() as u64,
            );
        }

        summary
    }
}
```

### 6.5 Realm ↔ Nervensystem

```rust
// realm/gateway/guard.rs

impl GatewayGuard {
    /// Evaluiere Crossing mit vollständiger State-Integration
    pub async fn evaluate_crossing(
        &self,
        ctx: &mut ExecutionContext,
        identity: &UniversalId,
        from_realm: &RealmId,
        to_realm: &RealmId,
        observer: &dyn RealmObserver,
    ) -> ErynoaResult<CrossingDecision> {
        // 1. Identity aus State laden
        let trust = self.state.trust.get_trust(identity)?;

        // 2. Realm-Policies aus ECLVM laden
        let policy = self.eclvm.get_crossing_policy(to_realm)?;

        // 3. Policy evaluieren (Gas-metered)
        let result = self.eclvm.evaluate_policy(
            &policy,
            PolicyContext {
                identity: identity.clone(),
                from_realm: from_realm.clone(),
                to_realm: to_realm.clone(),
                trust: trust.clone(),
            },
            ctx,
        ).await?;

        // 4. StateEvent emittieren
        let decision = if result.allowed {
            observer.on_crossing_succeeded(
                &from_realm.to_string(),
                &to_realm.to_string(),
            );
            CrossingDecision::Allowed { trust_at_crossing: trust.omega() }
        } else {
            observer.on_crossing_failed(
                &from_realm.to_string(),
                &to_realm.to_string(),
                &result.reason,
            );
            CrossingDecision::Denied { reason: result.reason }
        };

        Ok(decision)
    }
}
```

---

## 7. Domain-Typen als DNA

### 7.1 Unified Primitives

```rust
// domain/unified/primitives.rs

/// Universelle ID für alle Entitäten
pub struct UniversalId {
    tag: u8,           // Typ-Tag (Event, DID, Realm, etc.)
    version: u8,       // Schema-Version
    hash: [u8; 32],    // BLAKE3-Hash
}

/// Zeitliche Koordinate für Kausalität
pub struct TemporalCoord {
    unix_timestamp: u128,
    lamport_clock: u64,
    node_id: UniversalId,
}
```

### 7.2 Trust-Typen (Κ2-Κ5)

```rust
// domain/unified/trust.rs

/// 6-dimensionaler Trust-Vektor
pub struct TrustVector6D {
    pub r: f64,     // Reliability
    pub i: f64,     // Integrity
    pub c: f64,     // Competence
    pub p: f64,     // Predictability
    pub v: f64,     // Verification
    pub omega: f64, // Aggregated (Ω)
}

impl TrustVector6D {
    pub const NEWCOMER: Self = Self { r: 0.1, ... };
    pub const TRUSTED: Self = Self { r: 0.7, ... };
    pub const VERIFIED: Self = Self { r: 0.9, ... };
}
```

### 7.3 Invarianten-Checker

```rust
// domain/unified/mod.rs

pub struct InvariantChecker;

impl InvariantChecker {
    /// Κ1: Realm-Regel-Vererbung
    pub fn check_realm_rule_inheritance(...) -> Result<(), InvariantViolation>;

    /// Κ4: Asymmetrische Trust-Updates
    pub fn check_asymmetric_update(...) -> Result<(), InvariantViolation>;

    /// Κ8: Delegation Trust-Decay
    pub fn check_delegation_trust_factor(...) -> Result<(), InvariantViolation>;

    /// Κ9: Event-Kausalität
    pub fn check_event_causality(...) -> Result<(), InvariantViolation>;
}
```

---

## 8. SynapseHub Implementation

### 8.1 Kern-Struktur

```rust
// synapses/hub.rs

pub struct SynapseHub {
    /// Registrierte Observer pro StateComponent
    observers: DashMap<StateComponent, Vec<Arc<dyn StateObserver>>>,

    /// StateGraph für Dependency-Tracking
    graph: StateGraph,

    /// Event-Queue für async Dispatch
    event_queue: mpsc::Sender<WrappedStateEvent>,

    /// Metriken
    events_dispatched: AtomicU64,
    observers_notified: AtomicU64,
}

impl SynapseHub {
    /// Registriere Observer für eine Komponente
    pub fn register<O: StateObserver + 'static>(&self, observer: Arc<O>) {
        let component = observer.target_component();
        self.observers
            .entry(component)
            .or_default()
            .push(observer);
    }

    /// Dispatch Event an alle relevanten Observer
    pub async fn dispatch(&self, event: WrappedStateEvent) {
        let component = event.component;

        // 1. Direkte Observer
        if let Some(observers) = self.observers.get(&component) {
            for obs in observers.value() {
                obs.on_event(&event);
            }
        }

        // 2. Transitive Observer (via StateGraph)
        let triggered = self.graph.get_triggered_by(component);
        for triggered_component in triggered {
            if let Some(observers) = self.observers.get(&triggered_component) {
                for obs in observers.value() {
                    obs.on_event(&event);
                }
            }
        }

        self.events_dispatched.fetch_add(1, Ordering::Relaxed);
    }
}
```

### 8.2 StateIntegrator Facade

```rust
// synapses/integrator.rs

/// Zentrale Facade für State-Integration
pub struct StateIntegrator {
    state: Arc<UnifiedState>,
    hub: Arc<SynapseHub>,
    adapters: HashMap<StateComponent, Box<dyn EngineAdapter>>,
}

impl StateIntegrator {
    /// Verbinde alle Engines mit dem State
    pub fn connect_engines(&mut self) {
        // Trust Engine
        self.adapters.insert(
            StateComponent::Trust,
            Box::new(TrustEngineAdapter::new(self.state.clone())),
        );

        // Event Engine
        self.adapters.insert(
            StateComponent::Event,
            Box::new(EventEngineAdapter::new(self.state.clone())),
        );

        // Protection Layer
        self.adapters.insert(
            StateComponent::Anomaly,
            Box::new(AnomalyAdapter::new(self.state.clone())),
        );

        // ... weitere Adapter
    }

    /// Propagiere StateEvent durch das System
    pub async fn propagate(&self, event: StateEvent) {
        // 1. Log & Apply im UnifiedState
        let wrapped = self.state.log_and_apply(event).await;

        // 2. Dispatch an Observer
        self.hub.dispatch(wrapped).await;

        // 3. Persistiere wenn nötig
        if wrapped.requires_persistence() {
            self.state.persist_checkpoint().await;
        }
    }
}
```

---

## 9. Adapter-Pattern für Engine-Integration

### 9.1 Engine-Adapter Trait

```rust
// synapses/adapters/mod.rs

pub trait EngineAdapter: Send + Sync {
    /// Welche Komponente wird adaptiert?
    fn component(&self) -> StateComponent;

    /// Initialisiere Adapter mit State
    fn init(&mut self, state: Arc<UnifiedState>);

    /// Verarbeite eingehendes StateEvent
    fn on_event(&self, event: &WrappedStateEvent);

    /// Health-Check
    fn health_score(&self) -> f64;
}
```

### 9.2 Beispiel: TrustEngineAdapter

```rust
// synapses/adapters/trust.rs

pub struct TrustEngineAdapter {
    state: Arc<UnifiedState>,
    engine: TrustEngine,
}

impl EngineAdapter for TrustEngineAdapter {
    fn component(&self) -> StateComponent {
        StateComponent::Trust
    }

    fn on_event(&self, event: &WrappedStateEvent) {
        match &event.event {
            StateEvent::TrustUpdate { entity_id, new_trust, .. } => {
                // Update TrustState
                self.state.trust.update_trust(entity_id, *new_trust);
            }
            StateEvent::IdentityBootstrapped { did, .. } => {
                // Initialisiere Trust für neue Identity
                self.state.trust.init_trust(did, TrustVector6D::NEWCOMER);
            }
            _ => {}
        }
    }

    fn health_score(&self) -> f64 {
        self.engine.health_score()
    }
}
```

---

## 10. Dynamische Konfiguration

### 10.1 Runtime-Konfiguration

```rust
// config/integration.rs

pub struct IntegrationConfig {
    /// Event-Batch-Size für Dispatch
    pub dispatch_batch_size: usize,

    /// Timeout für Observer-Callbacks
    pub observer_timeout_ms: u64,

    /// Aktivierte Module
    pub enabled_modules: HashSet<StateComponent>,

    /// Observer-Prioritäten
    pub observer_priorities: HashMap<StateComponent, ObserverPriority>,
}

impl IntegrationConfig {
    /// Lade aus Environment
    pub fn from_env() -> Self {
        Self {
            dispatch_batch_size: env::var("DISPATCH_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(100),
            observer_timeout_ms: 1000,
            enabled_modules: StateComponent::all().into_iter().collect(),
            observer_priorities: Self::default_priorities(),
        }
    }
}
```

### 10.2 Feature-Flags

```toml
# Cargo.toml
[features]
default = ["p2p", "storage", "protection"]
p2p = ["libp2p"]
privacy = ["p2p", "onion-routing"]
storage = ["fjall"]
protection = []
full = ["p2p", "privacy", "storage", "protection", "wasm"]
wasm = ["wasmtime"]
```

---

## 11. Erfolgsmetriken

| Metrik | Aktuell | Phase 4 | Phase 6 |
|--------|---------|---------|---------|
| Event-Dispatch Latenz | 100 µs | 50 µs | 30 µs |
| Observer-Notification | sync | async | batch-async |
| StateGraph Traversal | O(n) | O(log n) | O(1) cached |
| Memory Footprint | 100 MB | 80 MB | 60 MB |
| Modul-Kopplung | Tight | Loose | Event-driven |

---

## 12. Nächste Schritte

1. **SynapseHub implementieren** (Woche 6)
2. **Adapter für alle Engines** (Woche 7)
3. **Event-Batching optimieren** (Woche 10)
4. **Performance-Benchmarks** (Woche 14)
