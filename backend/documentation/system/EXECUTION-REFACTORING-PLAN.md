# Execution-Layer Refactoring-Plan

Detaillierter Refactoring-Plan für die **Execution-Schicht** basierend auf der `state.rs` Referenz-Architektur.

Basis-Dokumente:
- [STATE-RS-REFERENCE.md](./STATE-RS-REFERENCE.md) – Architektur-Referenz
- [ECL-STATE-RS-GAP-ANALYSIS.md](./ECL-STATE-RS-GAP-ANALYSIS.md) – Gap-Analyse
- [ECL-ECLVM-REFACTORING-PLAN.md](./ECL-ECLVM-REFACTORING-PLAN.md) – ECL/ECLVM-Phasen

---

## 1. Architektur-Übersicht (state.rs Referenz)

### 1.1 Execution-Layer Hierarchie

```text
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              UNIFIED STATE                                       │
│                                                                                  │
│  ┌───────────────────────────────────────────────────────────────────────────┐  │
│  │                        ExecutionState (IPS ℳ)                              │  │
│  │  ┌────────────────┐   ┌────────────────┐   ┌────────────────┐             │  │
│  │  │   GasState     │───│   ManaState    │───│ExecutionsState │             │  │
│  │  │  - consumed    │   │  - consumed    │   │ - active_ctx   │             │  │
│  │  │  - refunded    │   │  - regenerated │   │ - total/succ   │             │  │
│  │  │  - limit_hits  │   │  - limit_hits  │   │ - avg_time_ms  │             │  │
│  │  └────────────────┘   └────────────────┘   └────────────────┘             │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
│                                      │                                           │
│  ┌───────────────────────────────────┼───────────────────────────────────────┐  │
│  │                        ECLVMState (Policy-Engine)                          │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │  │
│  │  │ Policy-Exec  │  │ Blueprint    │  │ Saga/Intent  │  │ Per-Realm    │   │  │
│  │  │ - compiled   │  │ - published  │  │ - processed  │  │ - policies   │   │  │
│  │  │ - executed   │  │ - deployed   │  │ - steps      │  │ - gas/mana   │   │  │
│  │  │ - passed/den │  │ - instantiat │  │ - cross-realm│  │ - crossings  │   │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘   │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
│                                      │                                           │
│  ┌───────────────────────────────────┼───────────────────────────────────────┐  │
│  │                     ECLVMStateContext (Orchestration)                      │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                     │  │
│  │  │  StateView   │  │ StateHandle  │  │ ECLVMBudget  │                     │  │
│  │  │  (Read-Only) │  │ (Realm-Write)│  │ (Gas/Mana/TO)│                     │  │
│  │  └──────────────┘  └──────────────┘  └──────────────┘                     │  │
│  └───────────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Aktueller Zustand vs. state.rs Ziel

| Komponente | state.rs Ziel | Aktueller Zustand | Gap |
|------------|---------------|-------------------|-----|
| ExecutionState | Gas/Mana/Contexts zentral | Vorhanden, aber isoliert von ECLVM | Mittel |
| ECLVMState | Policy-Metriken aus realen Läufen | Observer-basiert befüllt, duration_us fehlt | Klein |
| ECLVMStateContext | Host-Anbindung für ECLVM | Nicht an Runtime angebunden | Mittel |
| StateView | Read-Snapshot für ECL | Vorhanden, nicht in Produktion genutzt | Dokumentation |
| StateHandle | Realm-scoped Write | Vorhanden, nicht in Produktion genutzt | Dokumentation |
| ECLVMBudget | Zentrale Budget-Verwaltung | Separate GasMeter + ManaManager | Mittel |
| MultiGas | 4-Layer Gas (Network/Compute/Storage/Realm) | Nur einzelnes Gas-System | Groß |

---

## 2. Refactoring-Phasen

### 2.1 Phase-Übersicht

| Phase | Inhalt | Priorität | Aufwand |
|-------|--------|-----------|---------|
| **E1** | ExecutionState ↔ ECLVM-Integration | Hoch | Mittel |
| **E2** | ECLVMBudget als zentrale Ressourcen-Verwaltung | Hoch | Mittel |
| **E3** | ECLVMStateContext Runtime-Anbindung | Mittel | Groß |
| **E4** | StateHandle für Realm-scoped Writes | Niedrig | Mittel |
| **E5** | MultiGas (4-Layer) | Niedrig | Groß |
| **E6** | StateEvent-Integration für ECL | Niedrig | Klein |

---

## Phase E1: ExecutionState ↔ ECLVM-Integration

**Ziel:** ExecutionState (Gas/Mana/Contexts) wird konsistent mit ECLVM-Ausführungen synchronisiert.

### E1.1 duration_us korrekt erfassen und propagieren

**Problem:** `ExecutionResult.duration_us` wird berechnet, aber nicht an Observer/StateIntegrator übergeben.

**Lösung:**

1. **runner.rs erweitern:**

```rust
// eclvm/runtime/runner.rs
pub fn run_policy(...) -> Result<ExecutionResult> {
    let start = Instant::now();
    // ... VM run ...
    result.duration_us = start.elapsed().as_micros() as u64;
    Ok(result)
}
```

2. **PolicyExecutionObserver erweitern:**

```rust
// eclvm/programmable_gateway.rs
pub trait PolicyExecutionObserver: Send + Sync {
    fn on_policy_executed(
        &self,
        policy_id: &str,
        policy_type: &str,
        passed: bool,
        gas_used: u64,
        mana_used: u64,
        duration_us: u64,  // ← Neu: echte Dauer
        realm_id: Option<&str>,
    );

    fn on_crossing_policy_evaluated(...);
}
```

3. **StateIntegrator anpassen:**

```rust
// core/state_integration.rs
impl ECLVMObserver for StateIntegrator {
    fn on_policy_executed(..., duration_us: u64, ...) {
        // duration_us an ECLVMState.policy_executed übergeben
        self.state.eclvm.policy_executed(
            passed,
            policy_type,
            gas_used,
            mana_used,
            duration_us,  // ← Durchgereicht
            realm_id,
        );
    }
}
```

### E1.2 ExecutionState.executions mit ECLVM-Contexts synchronisieren

**Problem:** `ExecutionState.executions.active_contexts` wird nicht aus ECLVM befüllt.

**Lösung:**

```rust
// core/state.rs - ExecutionsState erweitern
impl ExecutionsState {
    /// Von ECLVM-Start aufgerufen (via StateIntegrator)
    pub fn eclvm_context_started(&self, context_id: &str) {
        self.active_contexts.fetch_add(1, Ordering::Relaxed);
        self.start();
    }

    /// Von ECLVM-Ende aufgerufen
    pub fn eclvm_context_completed(
        &self,
        success: bool,
        gas: u64,
        mana: u64,
        events: u64,
        duration_ms: u64,
    ) {
        self.active_contexts.fetch_sub(1, Ordering::Relaxed);
        self.complete(success, events, duration_ms);
    }
}
```

**StateIntegrator:**

```rust
// core/state_integration.rs
fn on_policy_executed(...) {
    // 1. ECLVMState aktualisieren (wie bisher)
    self.state.eclvm.policy_executed(...);

    // 2. ExecutionState synchronisieren
    self.state.execution.executions.eclvm_context_completed(
        passed,
        gas_used,
        mana_used,
        1,  // events
        duration_us / 1000,  // zu ms
    );
}
```

### E1.3 Abnahmekriterien Phase E1

- [ ] `duration_us` korrekt in `ExecutionResult` gesetzt
- [ ] Observer `on_policy_executed` mit `duration_us` aufgerufen
- [ ] `ECLVMState.avg_evaluation_time_us` reflektiert echte Werte
- [ ] `ExecutionState.executions` wird bei ECLVM-Läufen aktualisiert
- [ ] Gas/Mana in `ExecutionState.gas`/`mana` aggregiert

---

## Phase E2: ECLVMBudget als zentrale Ressourcen-Verwaltung

**Ziel:** Ein einheitliches Budget-System für alle ECL-Ausführungen (ersetzt parallele GasMeter + ManaManager).

### E2.1 PolicyRunContext mit ECLVMBudget erweitern

**Aktuell:**

```rust
pub struct PolicyRunContext {
    pub caller_did: String,
    pub realm_id: String,
    pub gas_limit: u64,  // Nur Gas-Limit
    ...
}
```

**Neu:**

```rust
// eclvm/runtime/runner.rs
pub struct PolicyRunContext {
    pub caller_did: String,
    pub realm_id: String,
    pub budget: Arc<ECLVMBudget>,  // ← Vollständiges Budget
    pub policy_id: Option<String>,
    pub policy_type: Option<String>,
}

impl PolicyRunContext {
    pub fn new(
        caller_did: impl Into<String>,
        realm_id: impl Into<String>,
        limits: ECLVMBudgetLimits,
    ) -> Self {
        Self {
            caller_did: caller_did.into(),
            realm_id: realm_id.into(),
            budget: Arc::new(ECLVMBudget::new(limits)),
            policy_id: None,
            policy_type: None,
        }
    }

    /// Legacy-Kompatibilität
    pub fn with_gas_limit(caller_did: impl Into<String>, realm_id: impl Into<String>, gas_limit: u64) -> Self {
        Self::new(
            caller_did,
            realm_id,
            ECLVMBudgetLimits {
                gas_limit,
                ..Default::default()
            },
        )
    }
}
```

### E2.2 GasMeter → ECLVMBudget in ECLVM

**Aktuell (vm.rs):**

```rust
pub struct ECLVM<'a> {
    gas: GasMeter,  // Separater Meter
    ...
}
```

**Neu (Option A – Wrapper):**

```rust
pub struct ECLVM<'a> {
    budget: Arc<ECLVMBudget>,  // Nutzt state.rs Budget
    ...
}

impl<'a> ECLVM<'a> {
    pub fn new(program: Vec<OpCode>, budget: Arc<ECLVMBudget>, host: &'a dyn HostInterface) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            ip: 0,
            program,
            budget,
            host,
            ...
        }
    }

    fn execute_instruction(&mut self, op: OpCode) -> Result<ControlFlow> {
        // Gas über Budget konsumieren
        if !self.budget.consume_gas(op.gas_cost()) {
            return Err(ApiError::Internal(anyhow!("Out of gas")));
        }
        ...
    }
}
```

### E2.3 ManaManager → ECLVMBudget

**Aktuell (ProgrammableGateway):**

```rust
if let Some(mana) = &self.mana {
    mana.deduct_mana(...);  // Separater Manager
}
```

**Neu:**

```rust
// In run_policy oder ProgrammableGateway:
context.budget.consume_mana(CROSSING_MANA_COST)?;
```

### E2.4 Timeout-Prüfung integrieren

ECLVMBudget prüft bereits bei `consume_gas()`:

```rust
if self.started_at.elapsed().as_millis() as u64 > self.limits.timeout_ms {
    self.mark_exhausted(BudgetExhaustionReason::Timeout);
    return false;
}
```

→ Automatisches Timeout ohne separaten Watchdog.

### E2.5 Abnahmekriterien Phase E2

- [ ] `PolicyRunContext` enthält `ECLVMBudget`
- [ ] ECLVM nutzt `budget.consume_gas()` statt `GasMeter`
- [ ] Mana-Verbrauch über `budget.consume_mana()`
- [ ] Timeout automatisch bei Gas-Consume geprüft
- [ ] `ECLVMBudgetSnapshot` liefert vollständige Metriken
- [ ] Rückwärts-kompatible Konstruktoren für Legacy-Code

---

## Phase E3: ECLVMStateContext Runtime-Anbindung

**Ziel:** ECLVMStateContext als optionaler Kontext für „State-backed ECL" (Lesen aus StateView, Schreiben über StateHandle).

### E3.1 StateHost implementieren

**Neuer Host-Adapter:**

```rust
// eclvm/runtime/state_host.rs
pub struct StateHost<'a> {
    context: &'a ECLVMStateContext,
}

impl<'a> StateHost<'a> {
    pub fn new(context: &'a ECLVMStateContext) -> Self {
        Self { context }
    }
}

impl<'a> HostInterface for StateHost<'a> {
    fn get_trust_vector(&self, did: &str) -> Option<TrustVector6D> {
        // Aus StateView lesen (Gas wird intern konsumiert)
        self.context.get_trust(did).map(|t| {
            // Konvertiere f64 → TrustVector6D (vereinfacht)
            TrustVector6D::uniform(t as f32)
        })
    }

    fn has_credential(&self, did: &str, credential_type: &str) -> bool {
        // Aus StateView / Identity-Cache
        self.context.view.get_identity(did)
            .map(|id| id.has_credential(credential_type))
            .unwrap_or(false)
    }

    fn resolve_did(&self, did: &str) -> Option<String> {
        self.context.view.get_identity(did).map(|id| id.did.clone())
    }

    fn get_realm_config(&self, realm_id: &str) -> Option<RealmConfig> {
        self.context.view.get_realm(realm_id).map(|r| r.into())
    }

    fn store_get(&self, _key: &str) -> Option<Vec<u8>> {
        // StateView ist Read-Only → None oder panic
        None
    }

    fn store_put(&self, key: &str, value: Vec<u8>) -> bool {
        // Über StateHandle schreiben
        if let Ok(handle) = self.context.create_write_handle() {
            handle.store_put(key, value);
            handle.commit().is_success()
        } else {
            false
        }
    }

    fn get_metric(&self, name: &str) -> Option<f64> {
        // Aus UnifiedState Snapshot
        match name {
            "trust.avg" => Some(self.context.view.trust_cache.read().ok()?.values().sum::<f64>() / ...),
            "realm.member_count" => self.context.view.get_realm(&self.context.realm())
                .map(|r| r.member_count as f64),
            _ => None,
        }
    }
}
```

### E3.2 run_policy mit StateContext

```rust
// eclvm/runtime/runner.rs
pub fn run_policy_with_state_context(
    bytecode: &[OpCode],
    context: &ECLVMStateContext,
) -> Result<ExecutionResult> {
    let host = StateHost::new(context);
    run_policy_internal(bytecode, &host, context.budget.clone())
}
```

### E3.3 EclEntrypoints mit State-Option

```rust
// eclvm/entrypoints.rs
impl<H: HostInterface + Send + Sync> EclEntrypoints<H> {
    /// Führe mit State-backed Context aus (kein Storage)
    pub fn run_api_with_state(
        &self,
        route_id: &str,
        state_context: &ECLVMStateContext,
        gas_limit: Option<u64>,
    ) -> Result<Value> {
        let bytecode = self.api_handlers.get(route_id).ok_or_else(...)?;
        run_policy_with_state_context(bytecode, state_context)
            .map(|r| r.value)
    }
}
```

### E3.4 Abnahmekriterien Phase E3

- [ ] `StateHost` implementiert `HostInterface`
- [ ] Lesen aus `StateView` (get_trust, get_realm, get_identity)
- [ ] Schreiben über `StateHandle.store_put()` → `commit()`
- [ ] `run_policy_with_state_context()` verfügbar
- [ ] EclEntrypoints optional mit StateContext nutzbar
- [ ] Dokumentation: „ErynoaHost = Produktion, StateHost = Tests/State-only"

---

## Phase E4: StateHandle für Realm-scoped Writes

**Ziel:** ECL-Policies können über StateHandle ephemere State-Änderungen vornehmen (nur UnifiedState, nicht Storage).

### E4.1 StateHandle-Operationen für ECL

```rust
// core/state.rs - StateHandle erweitern
impl<'a> StateHandle<'a> {
    /// Trust-Update im Realm-Kontext
    pub fn update_trust(&self, entity_id: &str, delta: f64) -> bool {
        if !self.valid.load(Ordering::Relaxed) {
            return false;
        }
        // Gas konsumieren
        if !self.budget.consume_gas(50) {
            return false;
        }
        // Dirty-Key merken
        if let Ok(mut keys) = self.dirty_keys.write() {
            keys.insert(format!("trust:{}", entity_id));
        }
        // Event emittieren
        self.emit_event(&StateEvent::TrustUpdate {
            entity_id: entity_id.to_string(),
            delta,
            reason: TrustReason::ECLPolicy,
            from_realm: Some(self.realm_id.clone()),
            triggered_events: 0,
            new_trust: delta,  // Vereinfacht
        });
        true
    }

    /// Governance-Vote im Realm
    pub fn cast_vote(&self, proposal_id: &str, vote: bool) -> bool {
        if !self.valid.load(Ordering::Relaxed) {
            return false;
        }
        if !self.budget.consume_gas(100) {
            return false;
        }
        // ... Vote-Logik ...
        true
    }
}
```

### E4.2 Commit mit Event-Log

```rust
impl<'a> StateHandle<'a> {
    pub fn commit(self) -> CommitResult {
        if !self.valid.swap(false, Ordering::Relaxed) {
            return CommitResult::AlreadyCommitted;
        }

        let events_applied = self.events_emitted.load(Ordering::Relaxed);
        let keys_written = self.dirty_keys.read()
            .map(|k| k.len())
            .unwrap_or(0);

        // Events in UnifiedState.event_log schreiben
        for event in self.pending_events.read().unwrap().iter() {
            self.state.apply_state_event(event.clone());
        }

        CommitResult::Success {
            events_applied,
            keys_written,
        }
    }
}
```

### E4.3 Abnahmekriterien Phase E4

- [ ] `StateHandle.update_trust()` verfügbar
- [ ] `StateHandle.store_put()` für Key-Value
- [ ] `StateHandle.emit_event()` für benutzerdefinierte Events
- [ ] `commit()` wendet Events auf UnifiedState an
- [ ] `rollback()` verwirft alle Änderungen
- [ ] TransactionGuard für RAII

---

## Phase E5: MultiGas (4-Layer)

**Ziel:** Differenzierte Kosten-Abrechnung nach Layer (Network, Compute, Storage, Realm).

### E5.1 GasLayer Enum

```rust
// core/state.rs (existiert bereits)
pub enum GasLayer {
    Network,   // L1: P2P-Bandbreite
    Compute,   // L2: CPU/Instructions
    Storage,   // L3: Persistence
    Realm,     // L4: Per-Realm Quotas
}
```

### E5.2 MultiGas Struktur

```rust
pub struct MultiGas {
    pub network: AtomicU64,
    pub compute: AtomicU64,
    pub storage: AtomicU64,
    pub realm: DashMap<String, AtomicU64>,
    pub prices: RwLock<HashMap<GasLayer, u64>>,  // Dynamic pricing
}

impl MultiGas {
    pub fn consume(&self, layer: GasLayer, amount: u64, realm_id: Option<&str>) {
        match layer {
            GasLayer::Network => self.network.fetch_add(amount, Ordering::Relaxed),
            GasLayer::Compute => self.compute.fetch_add(amount, Ordering::Relaxed),
            GasLayer::Storage => self.storage.fetch_add(amount, Ordering::Relaxed),
            GasLayer::Realm => {
                if let Some(rid) = realm_id {
                    self.realm.entry(rid.to_string())
                        .or_insert(AtomicU64::new(0))
                        .fetch_add(amount, Ordering::Relaxed);
                }
            }
        };
    }

    pub fn total_cost(&self) -> u64 {
        let prices = self.prices.read().unwrap();
        let network_cost = self.network.load(Ordering::Relaxed) * prices.get(&GasLayer::Network).unwrap_or(&1);
        let compute_cost = self.compute.load(Ordering::Relaxed) * prices.get(&GasLayer::Compute).unwrap_or(&1);
        let storage_cost = self.storage.load(Ordering::Relaxed) * prices.get(&GasLayer::Storage).unwrap_or(&1);
        network_cost + compute_cost + storage_cost
    }
}
```

### E5.3 ECLVM mit MultiGas

```rust
impl<'a> ECLVM<'a> {
    fn execute_instruction(&mut self, op: OpCode) -> Result<ControlFlow> {
        let (layer, cost) = op.gas_layer_cost();
        self.multi_gas.consume(layer, cost, self.realm_id.as_deref());

        if self.multi_gas.total_cost() > self.budget.limits.gas_limit {
            return Err(ApiError::Internal(anyhow!("Multi-layer gas exceeded")));
        }
        ...
    }
}

// OpCode erweitern
impl OpCode {
    pub fn gas_layer_cost(&self) -> (GasLayer, u64) {
        match self {
            OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Div => (GasLayer::Compute, 1),
            OpCode::LoadTrust | OpCode::HasCredential => (GasLayer::Network, 5),
            OpCode::StoreGet | OpCode::StorePut => (GasLayer::Storage, 10),
            OpCode::CrossRealm(_) => (GasLayer::Realm, 20),
            _ => (GasLayer::Compute, 1),
        }
    }
}
```

### E5.4 Abnahmekriterien Phase E5

- [ ] `MultiGas` struktur mit 4 Layern
- [ ] OpCodes haben `gas_layer_cost()`
- [ ] ECLVM aggregiert per Layer
- [ ] Dynamic Pricing konfigurierbar
- [ ] Per-Realm Gas-Tracking
- [ ] Snapshot mit Layer-Breakdown

---

## Phase E6: StateEvent-Integration für ECL

**Ziel:** ECL-Ausführungen emittieren `StateEvent::PolicyEvaluated` für konsistenten Event-Pfad.

### E6.1 StateEvent::PolicyEvaluated nach Runner

```rust
// core/state_integration.rs
impl ECLVMObserver for StateIntegrator {
    fn on_policy_executed(
        &self,
        policy_id: &str,
        policy_type: &str,
        passed: bool,
        gas_used: u64,
        mana_used: u64,
        duration_us: u64,
        realm_id: Option<&str>,
    ) {
        // 1. ECLVMState aktualisieren
        self.state.eclvm.policy_executed(...);

        // 2. StateEvent emittieren
        let event = StateEvent::PolicyEvaluated {
            policy_id: policy_id.to_string(),
            realm_id: realm_id.map(|s| s.to_string()),
            passed,
            policy_type: Self::map_policy_type(policy_type),
            gas_used,
            mana_used,
            duration_us,
        };

        // In Event-Log + Broadcaster
        self.state.event_log.log(event.clone());
        self.state.broadcaster.broadcast(StateDelta::new(
            StateComponent::ECLVM,
            DeltaType::Update,
            serde_json::to_vec(&event).unwrap_or_default(),
        ));
    }
}
```

### E6.2 Subscriber reagieren auf PolicyEvaluated

```rust
// Beispiel: DataLogic-Engine
async fn process_state_deltas(mut rx: broadcast::Receiver<StateDelta>) {
    while let Ok(delta) = rx.recv().await {
        if delta.component == StateComponent::ECLVM {
            if let Ok(event) = serde_json::from_slice::<StateEvent>(&delta.data) {
                match event {
                    StateEvent::PolicyEvaluated { policy_id, passed, .. } => {
                        // z.B. Alerting bei denied Policies
                        if !passed {
                            log::warn!("Policy denied: {}", policy_id);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
```

### E6.3 Abnahmekriterien Phase E6

- [ ] `StateEvent::PolicyEvaluated` nach jeder ECL-Ausführung
- [ ] Event in `state.event_log` geloggt
- [ ] Delta via `broadcaster` gesendet
- [ ] Subscriber (DataLogic, Monitoring) können reagieren

---

## 3. Abhängigkeiten und Reihenfolge

```
Phase E1 (ExecutionState ↔ ECLVM)
    ↓
Phase E2 (ECLVMBudget zentral)
    ↓
Phase E3 (StateContext Runtime) ← Parallel zu E4 möglich
    ↓                           ↘
Phase E4 (StateHandle Writes)    Phase E5 (MultiGas) ← Unabhängig
    ↓
Phase E6 (StateEvent-Integration) ← Parallel möglich
```

---

## 4. Migrations-Strategie

### 4.1 Rückwärts-Kompatibilität

Alle Phasen sollen bestehenden Code nicht brechen:

```rust
// Legacy-Konstruktor bleibt erhalten
impl PolicyRunContext {
    pub fn new(caller: &str, realm: &str, gas_limit: u64) -> Self {
        Self::with_budget(
            caller.into(),
            realm.into(),
            ECLVMBudgetLimits { gas_limit, ..Default::default() }.into(),
        )
    }
}

// Legacy GasMeter als Wrapper
impl GasMeter {
    pub fn from_budget(budget: Arc<ECLVMBudget>) -> Self {
        Self { budget_ref: Some(budget), ..Default::default() }
    }
}
```

### 4.2 Feature-Flags

```rust
// Cargo.toml
[features]
legacy-gas = []  # Alte GasMeter behalten
multi-gas = []   # Phase E5 aktivieren
state-host = []  # Phase E3 aktivieren
```

---

## 5. Test-Strategie

### 5.1 Unit-Tests pro Phase

| Phase | Test-Fokus |
|-------|------------|
| E1 | `duration_us` korrekt propagiert; ExecutionState Sync |
| E2 | Budget-Erschöpfung (Gas, Mana, Timeout); Snapshot |
| E3 | StateHost liest StateView; StateHandle commit/rollback |
| E4 | update_trust/store_put mit Budget; TransactionGuard |
| E5 | MultiGas per Layer; total_cost; Dynamic Pricing |
| E6 | StateEvent::PolicyEvaluated geloggt; Broadcaster |

### 5.2 Integration-Tests

```rust
#[tokio::test]
async fn test_e2e_policy_with_unified_budget() {
    let state = Arc::new(UnifiedState::new());
    let limits = ECLVMBudgetLimits::default();
    let context = PolicyRunContext::with_budget("did:test:alice", "realm:main", limits);

    let bytecode = compile_ecl("trust.r >= 0.5")?;
    let host = ErynoaHost::new(storage);

    let result = run_policy(&bytecode, &host, &context)?;

    // Budget korrekt konsumiert
    assert!(context.budget.gas_used() > 0);

    // ECLVMState aktualisiert
    let snapshot = state.eclvm.snapshot();
    assert!(snapshot.policies_executed > 0);
    assert!(snapshot.avg_evaluation_time_us > 0.0);
}
```

---

## 6. Metriken und Observability

### 6.1 Prometheus-Exposition

```rust
// Nach jeder Phase: Metriken exportieren
lazy_static! {
    static ref ECLVM_GAS_CONSUMED: IntCounterVec = register_int_counter_vec!(
        "erynoa_eclvm_gas_consumed_total",
        "Total gas consumed by ECLVM",
        &["layer", "realm"]
    ).unwrap();

    static ref ECLVM_BUDGET_EXHAUSTED: IntCounterVec = register_int_counter_vec!(
        "erynoa_eclvm_budget_exhausted_total",
        "Budget exhaustions by reason",
        &["reason"]  // OutOfGas, OutOfMana, Timeout
    ).unwrap();
}
```

### 6.2 Tracing-Integration

```rust
#[tracing::instrument(skip(bytecode, host))]
pub fn run_policy(
    bytecode: &[OpCode],
    host: &dyn HostInterface,
    context: &PolicyRunContext,
) -> Result<ExecutionResult> {
    let span = tracing::info_span!(
        "eclvm_policy_run",
        policy_id = %context.policy_id.as_deref().unwrap_or("unknown"),
        realm = %context.realm_id,
        gas_limit = context.budget.limits.gas_limit,
    );
    let _enter = span.enter();

    // ... run ...

    tracing::info!(
        gas_used = result.gas_used,
        duration_us = result.duration_us,
        "Policy execution completed"
    );

    Ok(result)
}
```

---

## 7. Zusammenfassung

| Phase | Aufwand | Wichtigste Änderung |
|-------|---------|---------------------|
| **E1** | 2-3 Tage | `duration_us` propagieren; ExecutionState Sync |
| **E2** | 3-5 Tage | ECLVMBudget als zentrale Ressourcen-Verwaltung |
| **E3** | 5-7 Tage | StateHost für State-backed ECL |
| **E4** | 3-4 Tage | StateHandle Writes mit Budget |
| **E5** | 5-7 Tage | MultiGas 4-Layer System |
| **E6** | 1-2 Tage | StateEvent::PolicyEvaluated Emission |

**Empfohlene Reihenfolge:** E1 → E2 → E6 → E3 → E4 → E5

---

**Stand:** Basierend auf state.rs, ECL-STATE-RS-GAP-ANALYSIS.md, aktueller eclvm-Implementierung.
