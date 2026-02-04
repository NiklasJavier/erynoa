# 🔄 ECLVM → WASM Migration

> **Teil von:** Projekt Pluto (Phase 5)
> **Zeitrahmen:** Woche 11-13
> **Abhängigkeiten:** Phase 4 (Integration) abgeschlossen

---

## 1. Executive Summary

Die Migration von ECLVM (Stack-VM) zu WebAssembly ist eine strategische Erweiterung des Projekt Pluto Refactorings. WASM bietet:

| Dimension | Stack-VM (Aktuell) | WASM (Ziel) | Erynoa-Impact |
|-----------|-------------------|-------------|---------------|
| **Trust-Ops** | ~50 Ops/ms | ~500 Ops/ms | Realm-Crossings 10x schneller |
| **Policy-Eval** | ~2ms avg | ~0.2ms avg | Gateway-Latenz minimiert |
| **Isolation** | Process-level | WASM-Sandbox | RealmQuota-Enforcement |
| **Determinismus** | Floating-Point-Drift | IEEE 754 strict | Consensus garantiert |

---

## 2. Integration mit Pluto-Architektur

### 2.1 Neue Verzeichnisstruktur

```
eclvm/
├── mod.rs                     # Bestehend (Feature-Flags erweitert)
├── ast.rs                     # Unverändert
├── parser.rs                  # Unverändert
├── compiler.rs                # Erweitert: + WASM-Backend
├── optimizer.rs               # Unverändert
├── bytecode.rs                # Legacy (für Hybrid-Mode)
├── runtime/                   # Bestehend
│   ├── vm.rs                  # Legacy VM
│   ├── host.rs                # HostInterface Trait
│   ├── state_host.rs          # StateHost
│   └── runner.rs              # Erweitert: Dual-Mode
│
├── 🆕 wasm/                   # NEU: WASM-Subsystem
│   ├── mod.rs                 # Modul-Root
│   ├── engine.rs              # WasmPolicyEngine
│   │
│   ├── codegen/               # ECL → WASM Compiler
│   │   ├── mod.rs
│   │   ├── compiler.rs        # AST → WASM Bytecode
│   │   ├── opcodes.rs         # OpCode → WASM Mapping
│   │   └── optimizer.rs       # WASM-spezifische Optimierungen
│   │
│   ├── host/                  # Host-Functions
│   │   ├── mod.rs
│   │   ├── trust.rs           # erynoa.get_trust, combine_trust
│   │   ├── identity.rs        # erynoa.has_credential, resolve_did
│   │   ├── state.rs           # erynoa.store_get, store_put
│   │   ├── budget.rs          # erynoa.consume_gas, get_budget
│   │   └── bridge.rs          # WasmStateBridge
│   │
│   ├── runtime/               # Wasmtime Integration
│   │   ├── mod.rs
│   │   ├── store.rs           # Wasmtime Store-Wrapper
│   │   ├── fuel.rs            # Fuel → MultiGas Mapping
│   │   └── cache.rs           # Pre-compiled Module Cache
│   │
│   └── types/                 # WIT-Typen
│       ├── mod.rs
│       ├── wit_bindings.rs    # Generierte Bindings
│       └── conversions.rs     # Rust ↔ WASM Konvertierung
│
├── erynoa_host.rs             # Erweitert: WASM-Support
├── bridge.rs                  # Erweitert: WASM-Serialisierung
├── mana.rs                    # Unverändert
├── entrypoints.rs             # Erweitert: Dual-Mode
├── programmable_gateway.rs    # Erweitert: WASM-Policies
└── stdlib.rs                  # Unverändert
```

### 2.2 Integration mit nervous_system/

```
┌─────────────────────────────────────────────────────────────────┐
│                      INTEGRATION                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   nervous_system/                    eclvm/wasm/                │
│   ┌─────────────────┐               ┌─────────────────┐         │
│   │ UnifiedState    │◄─────────────►│ WasmStateBridge │         │
│   │                 │               │                 │         │
│   │ • ECLVMState    │ StateView     │ • get_trust()   │         │
│   │ • TrustState    │───────────────│ • store_get()   │         │
│   │ • IdentityState │ StateHandle   │ • consume_gas() │         │
│   └─────────────────┘               └─────────────────┘         │
│          │                                   │                   │
│          ▼                                   ▼                   │
│   ┌─────────────────┐               ┌─────────────────┐         │
│   │ StateEventLog   │◄──────────────│ Event Emission  │         │
│   │                 │ StateEvent    │ (via Host-Func) │         │
│   └─────────────────┘               └─────────────────┘         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. WIT-Interface Definition

```wit
// erynoa-ecl.wit
package erynoa:ecl@0.1.0;

/// 6D Trust Vector [R, I, C, P, V, Ω]
record trust-vector {
    r: f64,
    i: f64,
    c: f64,
    p: f64,
    v: f64,
    omega: f64,
}

/// Gas Layer (entspricht MultiGas in nervous_system/)
enum gas-layer {
    network,
    compute,
    storage,
    realm,
}

/// Host Functions
interface host {
    // Trust (Κ2-Κ5)
    get-trust: func(did: string) -> result<trust-vector, string>;
    trust-norm: func(tv: trust-vector) -> f64;

    // Identity (Κ6-Κ8)
    has-credential: func(did: string, schema: string) -> result<bool, string>;
    resolve-did: func(did: string) -> result<bool, string>;

    // State (via StateView/StateHandle)
    store-get: func(store: string, key: string) -> result<option<string>, string>;
    store-put: func(store: string, key: string, value: string) -> result<unit, string>;

    // Budget (ECLVMBudget + MultiGas)
    consume-gas: func(layer: gas-layer, amount: u64) -> result<unit, string>;
    get-budget: func() -> tuple<u64, u64, u64, u64>;  // gas-used, gas-limit, mana-used, mana-limit

    // Context
    get-caller: func() -> string;
    get-realm: func() -> string;
    get-timestamp: func() -> u64;
    log: func(message: string);

    // Events (StateEvent Emission)
    emit-event: func(event-type: string, payload: string) -> result<unit, string>;
}
```

---

## 4. Kernkomponenten

### 4.1 WasmPolicyEngine

```rust
// eclvm/wasm/engine.rs
pub struct WasmPolicyEngine {
    /// Wasmtime Engine (shared)
    engine: Engine,
    /// Pre-compiled Modules Cache
    module_cache: Arc<RwLock<HashMap<String, Module>>>,
    /// Linker mit Host-Functions
    linker: Linker<WasmHostState>,
    /// Config
    config: WasmEngineConfig,
}

pub struct WasmEngineConfig {
    pub fuel_limit: u64,
    pub memory_pages_limit: u32,
    pub cache_compiled_modules: bool,
    pub enable_simd: bool,
}

impl WasmPolicyEngine {
    /// Kompiliere ECL zu WASM
    pub fn compile(&self, ecl_source: &str) -> Result<CompiledWasmPolicy>;

    /// Führe Policy aus
    pub async fn execute(
        &self,
        policy: &CompiledWasmPolicy,
        context: ECLVMStateContext,
    ) -> Result<ExecutionResult>;
}
```

### 4.2 WasmStateBridge

```rust
// eclvm/wasm/host/bridge.rs
pub struct WasmStateBridge {
    /// Read-only State View
    state_view: Arc<StateView>,
    /// Write-capable Handle (optional)
    state_handle: Option<StateHandle<'static>>,
    /// Budget Tracking
    budget: Arc<ECLVMBudget>,
    /// Multi-Layer Gas
    multi_gas: Arc<MultiGas>,
    /// Pending Events
    pending_events: RwLock<Vec<StateEvent>>,
}

impl WasmStateBridge {
    // Trust Operations
    pub fn get_trust(&self, did: &str) -> Result<TrustVector6D>;

    // State Operations
    pub fn store_get(&self, store: &str, key: &str) -> Result<Option<Value>>;
    pub fn store_put(&mut self, store: &str, key: &str, value: Value) -> Result<()>;

    // Gas/Mana
    pub fn consume_gas(&self, layer: GasLayer, amount: u64) -> Result<()>;
    pub fn consume_mana(&self, amount: u64) -> Result<()>;

    // Events
    pub fn emit_event(&mut self, event: StateEvent) -> Result<()>;

    // Commit pending changes
    pub fn commit(&mut self) -> Result<u64>;
}
```

### 4.3 Dual-Mode Runner

```rust
// eclvm/runtime/runner.rs (erweitert)
pub enum ExecutionMode {
    /// Legacy Stack-VM (für Rückwärtskompatibilität)
    Legacy,
    /// Neuer WASM-Runner
    Wasm,
    /// Automatische Auswahl basierend auf Komplexität
    Auto,
}

pub struct PolicyRunner {
    /// Legacy VM
    legacy_vm: ECLVM,
    /// WASM Engine (optional, feature-gated)
    #[cfg(feature = "wasm")]
    wasm_engine: Option<Arc<WasmPolicyEngine>>,
    /// Aktiver Modus
    mode: ExecutionMode,
    /// Threshold für Auto-Mode (OpCode-Count)
    auto_threshold: usize,
}

impl PolicyRunner {
    pub async fn execute(
        &self,
        policy: &CompiledPolicy,
        context: ECLVMStateContext,
    ) -> Result<ExecutionResult> {
        match self.mode {
            ExecutionMode::Legacy => self.execute_legacy(policy, context),
            ExecutionMode::Wasm => self.execute_wasm(policy, context).await,
            ExecutionMode::Auto => {
                if policy.opcodes.len() > self.auto_threshold {
                    self.execute_wasm(policy, context).await
                } else {
                    self.execute_legacy(policy, context)
                }
            }
        }
    }
}
```

---

## 5. OpCode → WASM Mapping

| ECL OpCode | WASM Equivalent | Notes |
|------------|-----------------|-------|
| `Push(f64)` | `f64.const` | Direkt |
| `Add` | `f64.add` | Direkt |
| `Sub` | `f64.sub` | Direkt |
| `Mul` | `f64.mul` | Direkt |
| `Div` | `f64.div` | Direkt |
| `Eq` | `f64.eq` | Direkt |
| `Lt` | `f64.lt` | Direkt |
| `And` | `i32.and` | Nach bool-Konversion |
| `Or` | `i32.or` | Nach bool-Konversion |
| `Not` | `i32.eqz` | Nach bool-Konversion |
| `LoadTrust(dim)` | `call $erynoa.get_trust` | Host-Call |
| `HasCredential` | `call $erynoa.has_credential` | Host-Call |
| `StoreGet` | `call $erynoa.store_get` | Host-Call + Mana |
| `StorePut` | `call $erynoa.store_put` | Host-Call + Mana |
| `Return` | `return` | Direkt |
| `Require` | `br_if + unreachable` | Conditional |

---

## 6. Performance-Ziele

| Metrik | Legacy (Bytecode) | WASM Ziel | Verbesserung |
|--------|-------------------|-----------|--------------|
| Policy-Latenz | 2ms | 0.2ms | 10x |
| Trust-Ops/ms | 50 | 500 | 10x |
| Startup-Zeit | 0.1ms | 1ms (cold), 0.1ms (hot) | Cache |
| Memory | 1MB/Policy | 2MB/Policy | Akzeptabel |
| Throughput | 500 Policies/s | 5000 Policies/s | 10x |

---

## 7. Implementierungsschritte

### Woche 11: Infrastructure

| Tag | Aufgabe | Deliverable |
|-----|---------|-------------|
| Mo | Wasmtime zu Cargo.toml | `Cargo.toml` |
| Di | `eclvm/wasm/mod.rs` Struktur | Modul-Layout |
| Mi | WIT-Datei + wit-bindgen | `erynoa-ecl.wit` |
| Do | Basic Host-Functions | `host/mod.rs` |
| Fr | Unit-Tests | `tests/wasm_basic.rs` |

### Woche 12: Host Integration

| Tag | Aufgabe | Deliverable |
|-----|---------|-------------|
| Mo | WasmStateBridge | `host/bridge.rs` |
| Di | Trust-Host-Functions | `host/trust.rs` |
| Mi | State-Host-Functions | `host/state.rs` |
| Do | Budget/Gas-Integration | `host/budget.rs` |
| Fr | Integration-Tests | `tests/wasm_integration.rs` |

### Woche 13: Full Parity

| Tag | Aufgabe | Deliverable |
|-----|---------|-------------|
| Mo | AST→WASM Compiler | `codegen/compiler.rs` |
| Di | Alle OpCodes mappend | `codegen/opcodes.rs` |
| Mi | Dual-Mode Runner | `runtime/runner.rs` |
| Do | ProgrammableGateway WASM | `programmable_gateway.rs` |
| Fr | Benchmarks + Dokumentation | `benchmarks/wasm_perf.rs` |

---

## 8. Feature-Flags

```toml
# Cargo.toml
[features]
default = ["wasm"]  # WASM standardmäßig aktiviert
wasm = ["wasmtime", "wit-bindgen"]
wasm-simd = ["wasm", "wasmtime/simd"]
legacy-only = []    # Nur Bytecode-Interpreter
```

---

## 9. Migrations-Strategie

### 9.1 Rückwärtskompatibilität

```rust
// Bestehende API bleibt verfügbar
pub fn evaluate_policy(
    policy: &CompiledPolicy,
    context: &ECLVMStateContext,
) -> Result<Value> {
    // Automatische Auswahl: WASM wenn verfügbar, sonst Legacy
    #[cfg(feature = "wasm")]
    {
        GLOBAL_RUNNER.execute(policy, context).await
    }
    #[cfg(not(feature = "wasm"))]
    {
        legacy_evaluate(policy, context)
    }
}
```

### 9.2 Graduelle Umstellung

1. **Phase A:** Neue Policies mit WASM kompilieren
2. **Phase B:** Bestehende Policies bei nächster Änderung migrieren
3. **Phase C:** Legacy-Interpreter deprecated
4. **Phase D:** Legacy-Code entfernen (v1.0)

---

## 10. Risiken

| Risiko | Wahrscheinlichkeit | Mitigation |
|--------|-------------------|------------|
| WASM Cold-Start Latenz | Mittel | Pre-compiled Module Cache |
| Memory-Overhead | Niedrig | Linear Memory Limits |
| Host-Function Overhead | Mittel | Batch-Calls, Caching |
| Breaking Changes | Niedrig | Versionierte WIT-Interfaces |

---

## 11. Erfolgsmetriken

- [ ] WASM Policy-Latenz ≤ 0.2ms (für Standard-Policies)
- [ ] Throughput ≥ 5000 Policies/s
- [ ] Test-Coverage ≥ 90% für WASM-Module
- [ ] Alle 50+ OpCodes in WASM implementiert
- [ ] Dual-Mode funktioniert nahtlos
