# 📋 Phasenplan: Konkrete Umsetzungsschritte

> **Teil von:** Projekt Pluto
> **Zeitrahmen:** 14 Wochen (inkl. ECLVM→WASM Integration)

---

## Phasen-Übersicht

| Phase | Woche | Fokus | Abhängigkeiten |
|-------|-------|-------|----------------|
| **1. Foundation** | 1-2 | Basis-Infrastruktur | - |
| **2. Decomposition** | 3-5 | state.rs aufteilen | Phase 1 |
| **3. Synapse Hub** | 6-7 | Observer konsolidieren | Phase 2 |
| **4. Integration** | 8-10 | P2P, Storage, Engines | Phase 3 |
| **5. ECLVM→WASM** | 11-13 | WASM-Runtime | Phase 4 |
| **6. Optimization** | 14 | Performance, Polish | Phase 5 |

---

## Phase 1: Foundation (Woche 1-2)

### Tag 1: Verzeichnisse erstellen

```bash
cd backend/src

# Nervensystem
mkdir -p nervous_system/{event_sourcing,merkle,components,graph,infrastructure}

# Synapses
mkdir -p synapses/adapters

# Realm (aus peer/ extrahiert)
mkdir -p realm/{sharding,quota,gateway,saga}

# Storage (aus local/ umbenannt)
mkdir -p storage/{kv,event_store,identity_store,trust_store,content_store,archive,realm,blueprint,metrics}

# Identity
mkdir -p identity

# Engines
mkdir -p engines

# ECLVM WASM (NEU für Phase 5)
mkdir -p eclvm/wasm/{codegen,host,runtime,types}
```

### Tag 2-3: Unified Error Types

**Datei erstellen:** `domain/unified/error.rs`

### Tag 4-5: Base Traits

**Datei erstellen:** `nervous_system/traits.rs`

### Tag 6-7: EventBus extrahieren

**Von:** `core/state.rs` (Zeilen 39-400)
**Nach:** `nervous_system/infrastructure/event_bus.rs`

---

## Phase 2: Decomposition (Woche 3-5)

### Woche 3: StateEvent Extraktion

| Tag | Aufgabe | Zeilen |
|-----|---------|--------|
| Mo | StateEvent enum → `event_sourcing/state_event.rs` | 800-1900 |
| Di | WrappedStateEvent → `event_sourcing/wrapped_event.rs` | 1900-2100 |
| Mi | StateEventLog → `event_sourcing/event_log.rs` | 2100-2400 |
| Do | Tests migrieren | - |
| Fr | Re-Exports + Compilation-Check | - |

### Woche 4: Merkle & Graph Extraktion

| Tag | Aufgabe | Zeilen |
|-----|---------|--------|
| Mo | MerkleStateTracker → `merkle/tracker.rs` | 2500-2800 |
| Di | MerkleDelta → `merkle/delta.rs` | 2800-2950 |
| Mi | StateGraph → `graph/analysis.rs` | 4080-4450 |
| Do | StateComponent ins domain/ verschieben | - |
| Fr | Tests + Compilation | - |

### Woche 5: Component-States Extraktion

| Tag | Aufgabe |
|-----|---------|
| Mo | TrustState, EventState → `components/core.rs` |
| Di | ProtectionState → `components/protection.rs` |
| Mi | RealmState → `components/peer.rs` |
| Do | ECLVMState → `components/eclvm.rs` |
| Fr | UnifiedState bereinigen → `unified_state.rs` |

---

## Phase 3: Synapse Hub (Woche 6-7)

### Woche 6: Observer-Konsolidierung

**Von:** `core/state_integration.rs` (6.427 Zeilen)
**Nach:** `synapses/`

### Woche 7: Hub Implementation

```rust
// synapses/hub.rs
pub struct SynapseHub {
    observers: DashMap<StateComponent, Vec<Arc<dyn StateObserver>>>,
}
```

---

## Phase 4: Integration (Woche 8-10)

### Woche 8: P2P Konsolidierung

**Von:** `peer/p2p/` (38 Dateien)
**Nach:** `p2p/` (15 Dateien)

### Woche 9: Storage Refactoring

**Von:** `local/` → **Nach:** `storage/`

### Woche 10: Engines Konsolidierung

**Von:** `core/` → **Nach:** `engines/`

---

## Phase 5: ECLVM→WASM Integration (Woche 11-13)

> **Referenz:** Siehe `06-ECLVM-WASM-MIGRATION.md` für Details

### Woche 11: WASM Infrastructure

| Tag | Aufgabe |
|-----|---------|
| Mo | Wasmtime Dependency + Feature-Flags |
| Di | `eclvm/wasm/mod.rs` Struktur |
| Mi | WIT-Interface definieren (`erynoa-ecl.wit`) |
| Do | Basic Host-Functions implementieren |
| Fr | Compilation + Tests |

**Neue Dateien:**

```
eclvm/wasm/
├── mod.rs              (100 Zeilen)
├── engine.rs           (WasmPolicyEngine)
├── codegen/
│   ├── mod.rs
│   └── compiler.rs     (ECL AST → WASM)
├── host/
│   ├── mod.rs
│   ├── trust.rs        (Trust-Host-Functions)
│   ├── identity.rs     (Identity-Host-Functions)
│   ├── state.rs        (State-Host-Functions)
│   └── budget.rs       (Budget-Host-Functions)
├── runtime/
│   ├── mod.rs
│   ├── store.rs        (Wasmtime Store-Wrapper)
│   └── fuel.rs         (Fuel→MultiGas Mapping)
└── types/
    ├── mod.rs
    └── wit_bindings.rs (WIT-generierte Typen)
```

### Woche 12: Host Integration

| Tag | Aufgabe |
|-----|---------|
| Mo | `WasmStateBridge` implementieren |
| Di | StateView → WASM Serialisierung |
| Mi | StateHandle Transaktionen |
| Do | MultiGas → Fuel Mapping |
| Fr | Integration Tests |

**Kernstück: Host-State-Bridge**

```rust
// eclvm/wasm/host/state.rs
pub struct WasmStateBridge {
    state_view: Arc<StateView>,
    budget: Arc<ECLVMBudget>,
    pending_mutations: Vec<StateMutation>,
}

impl WasmStateBridge {
    pub fn get_trust(&self, did: &str) -> Result<TrustVector6D>;
    pub fn store_get(&self, store: &str, key: &str) -> Result<Option<Value>>;
    pub fn consume_gas_layered(&self, layer: GasLayer, amount: u64) -> Result<()>;
}
```

### Woche 13: Full Feature Parity

| Tag | Aufgabe |
|-----|---------|
| Mo | Alle OpCodes → WASM kompilierbar |
| Di | Store-Operationen mit Schema-Evolution |
| Mi | Event-Emission (StateEvent) |
| Do | Dual-Mode (Legacy Bytecode + WASM) |
| Fr | Performance-Benchmarks |

**Dual-Mode Implementierung:**

```rust
// eclvm/runtime/runner.rs (erweitert)
pub enum ExecutionMode {
    Legacy,       // Bestehendes Bytecode-Interpreter
    Wasm,         // Neuer Wasmtime-Runner
    Auto,         // Wählt basierend auf Policy-Komplexität
}

pub struct PolicyRunner {
    legacy_vm: ECLVM,
    wasm_engine: Option<WasmPolicyEngine>,
    mode: ExecutionMode,
}
```

---

## Phase 6: Optimization (Woche 14)

### Performance-Tuning

| Bereich | Aktion | Ziel |
|---------|--------|------|
| Trust-Lookup | Bereits O(1) | Beibehalten |
| Event-Dispatch | IndexMap statt Vec | -30% |
| WASM-Startup | Pre-compiled Modules | <1ms |
| Policy-Eval | WASM vs Bytecode | 5-10x schneller |

### Polish

- [ ] Dokumentation aktualisieren
- [ ] CHANGELOG.md schreiben
- [ ] Version auf 0.5.0 erhöhen
- [ ] Deprecated-Warnings hinzufügen
- [ ] Performance-Benchmarks dokumentieren

---

## Checkliste pro Phase

### Phase 1 ✅

- [ ] Verzeichnisse erstellt (inkl. `eclvm/wasm/`)
- [ ] `error.rs` implementiert
- [ ] `traits.rs` implementiert
- [ ] EventBus extrahiert

### Phase 2 ✅

- [ ] StateEvent extrahiert
- [ ] Merkle extrahiert
- [ ] StateGraph extrahiert
- [ ] Components extrahiert

### Phase 3 ✅

- [ ] Observer-Traits konsolidiert
- [ ] SynapseHub implementiert
- [ ] Adapter erstellt

### Phase 4 ✅

- [ ] P2P konsolidiert
- [ ] Storage refactored
- [ ] Engines konsolidiert

### Phase 5: ECLVM→WASM ✅

- [ ] Wasmtime integriert
- [ ] WIT-Interface definiert
- [ ] Host-Functions implementiert
- [ ] AST→WASM Compiler
- [ ] Dual-Mode Runner
- [ ] Benchmarks: WASM ≤1.5x Bytecode Latenz

### Phase 6 ✅

- [ ] Performance-Ziele erreicht
- [ ] Dokumentation vollständig
- [ ] Release-Ready

---

## Risiken & Mitigationen

| Risiko | Wahrscheinlichkeit | Impact | Mitigation |
|--------|-------------------|--------|------------|
| Breaking Changes | Hoch | Mittel | Re-Exports |
| WASM-Overhead | Mittel | Niedrig | Hybrid-Mode |
| Memory-Safety | Mittel | Hoch | Bounds-Checks |
| Performance-Regression | Niedrig | Mittel | Benchmarks |

---

## Erfolgskriterien

| Metrik | Aktuell | Phase 4 | Phase 6 |
|--------|---------|---------|---------|
| `state.rs` Zeilen | 21.495 | 5.000 | 2.000 |
| WASM Policy-Latenz | N/A | N/A | ≤0.2ms |
| Policy Throughput | 50 Ops/ms | 50 Ops/ms | 500 Ops/ms |
| Test-Coverage | ~60% | 75% | >85% |
