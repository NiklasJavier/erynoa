# ECHO – WebAssembly Architektur

> **Komponente:** ECHO (Emergent Swarm + ECLVM)
> **Schicht:** ◐ IMPULS (Layer 4 – Handlung)
> **Ziel:** WASM-basierte Runtime für autonome Agenten
> **Version:** 1.0

---

## Präambel: Von Logik zu WASM

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║   WELTFORMEL → ECHO → WASM MAPPING                                                                                                       ║
║                                                                                                                                           ║
║       σ(s)           →  AttentionScore       →  wasm: attention_score(entity_id) -> f64                                                 ║
║       𝕋(s)           →  TrustVector          →  wasm: get_trust(entity_id) -> [f64; 4]                                                  ║
║       s : α          →  Action               →  wasm: perform_action(agent_id, action) -> Result                                        ║
║       ◇α             →  Possibility          →  wasm: can_perform(agent_id, action) -> bool                                             ║
║       α ⊛ β          →  Exchange             →  wasm: atomic_exchange(a, b, offer_a, offer_b) -> Result                                 ║
║                                                                                                                                           ║
║       Policy         →  PolicyEngine         →  wasm: evaluate_policy(policy_id, context) -> Decision                                   ║
║       Intent         →  IntentMatcher        →  wasm: match_intent(intent_id) -> Vec<Offer>                                             ║
║       Negotiation    →  NegotiationEngine    →  wasm: negotiate(session_id, round) -> NegotiationState                                  ║
║                                                                                                                                           ║
║   ECLVM              →  VirtualMachine       →  wasm: execute_bytecode(bytecode, gas_limit) -> ExecutionResult                          ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

# Teil I: WASM-Module-Struktur

## 1.1 Module-Übersicht

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║                                           ECHO WASM MODULE HIERARCHY                                                                     ║
║                                                                                                                                           ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ ║
║   │                                                                                                                                     │ ║
║   │   📦 echo-wasm/                                                                                                                     │ ║
║   │   ├── 🔧 echo-core-wasm           # Basistypen + Weltformel-Bindungen                                                              │ ║
║   │   ├── 🧠 echo-eclvm-wasm          # ECLVM Runtime in WASM                                                                          │ ║
║   │   ├── 🤖 echo-agent-wasm          # Agent-Runtime                                                                                  │ ║
║   │   ├── 🎯 echo-intent-wasm         # Intent-Matching-Engine                                                                         │ ║
║   │   ├── 📋 echo-policy-wasm         # Policy-Evaluation                                                                              │ ║
║   │   ├── 🤝 echo-negotiation-wasm    # Verhandlungs-Engine                                                                            │ ║
║   │   ├── 💰 echo-wallet-wasm         # Wallet-Funktionen                                                                              │ ║
║   │   └── 🔌 echo-host-wasm           # Host-Interface zu ERY/NOA                                                                      │ ║
║   │                                                                                                                                     │ ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

## 1.2 Abhängigkeits-Graph

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                                                                             │
│   ABHÄNGIGKEITEN                                                                                                                           │
│   ══════════════                                                                                                                            │
│                                                                                                                                             │
│                                      ┌─────────────────┐                                                                                   │
│                                      │                 │                                                                                   │
│                                      │  echo-host-wasm │ ◀────── Host-Funktionen (ERY, NOA)                                               │
│                                      │                 │                                                                                   │
│                                      └────────┬────────┘                                                                                   │
│                                               │                                                                                            │
│                                               ▼                                                                                            │
│                                      ┌─────────────────┐                                                                                   │
│                                      │                 │                                                                                   │
│                                      │  echo-core-wasm │ ◀────── Basistypen (DID, Trust, etc.)                                            │
│                                      │                 │                                                                                   │
│                                      └────────┬────────┘                                                                                   │
│                                               │                                                                                            │
│                         ┌─────────────────────┼─────────────────────┐                                                                      │
│                         │                     │                     │                                                                      │
│                         ▼                     ▼                     ▼                                                                      │
│                ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐                                                             │
│                │                 │   │                 │   │                 │                                                             │
│                │ echo-eclvm-wasm │   │ echo-wallet-wasm│   │ echo-policy-wasm│                                                             │
│                │                 │   │                 │   │                 │                                                             │
│                └────────┬────────┘   └────────┬────────┘   └────────┬────────┘                                                             │
│                         │                     │                     │                                                                      │
│                         └─────────────────────┼─────────────────────┘                                                                      │
│                                               │                                                                                            │
│                                               ▼                                                                                            │
│                                      ┌─────────────────┐                                                                                   │
│                                      │                 │                                                                                   │
│                                      │ echo-agent-wasm │ ◀────── Agent-Logik                                                              │
│                                      │                 │                                                                                   │
│                                      └────────┬────────┘                                                                                   │
│                                               │                                                                                            │
│                         ┌─────────────────────┴─────────────────────┐                                                                      │
│                         │                                           │                                                                      │
│                         ▼                                           ▼                                                                      │
│                ┌─────────────────┐                         ┌─────────────────┐                                                             │
│                │                 │                         │                 │                                                             │
│                │echo-intent-wasm │                         │echo-negotiation │                                                             │
│                │                 │                         │     -wasm       │                                                             │
│                └─────────────────┘                         └─────────────────┘                                                             │
│                                                                                                                                             │
└─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘
```

---

# Teil II: Host-Interface (echo-host-wasm)

## 2.1 Host-Funktionen

```rust
// crates/echo-host-wasm/src/lib.rs

//! Host-Funktionen für WASM-Module
//! Diese Funktionen werden vom Host (Browser/Node/Native) bereitgestellt

use wasm_bindgen::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
// ERY-INTERFACE: Identität + Trust
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
extern "C" {
    /// Typ für Host-Errors
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // IDENTITY-OPERATIONEN (A1-A4)
    // ─────────────────────────────────────────────────────────────────────────

    /// Prüft ob Entity existiert: ⟨s⟩
    #[wasm_bindgen(js_name = "ery_exists")]
    pub fn host_exists(did: &str) -> bool;

    /// Löst DID auf und gibt Entity-Daten zurück
    #[wasm_bindgen(js_name = "ery_resolve")]
    pub fn host_resolve_did(did: &str) -> JsValue;  // -> Entity | null

    /// Prüft Delegation: s ⊳ s'
    #[wasm_bindgen(js_name = "ery_derives_from")]
    pub fn host_derives_from(child: &str, parent: &str) -> bool;

    /// Gibt Parent-DID zurück
    #[wasm_bindgen(js_name = "ery_get_parent")]
    pub fn host_get_parent(did: &str) -> Option<String>;

    // ─────────────────────────────────────────────────────────────────────────
    // TRUST-OPERATIONEN (A5-A10)
    // ─────────────────────────────────────────────────────────────────────────

    /// Gibt Trust-Vector zurück: 𝕋(s)
    #[wasm_bindgen(js_name = "ery_get_trust")]
    pub fn host_get_trust(did: &str) -> Box<[f64]>;  // [R, I, C, P]

    /// Gibt aggregiertes Trust zurück: 𝕋̄(s)
    #[wasm_bindgen(js_name = "ery_get_trust_aggregate")]
    pub fn host_get_trust_aggregate(did: &str) -> f64;

    /// Berechnet Aufmerksamkeit: σ(s)
    #[wasm_bindgen(js_name = "ery_attention")]
    pub fn host_attention(did: &str) -> f64;

    // ─────────────────────────────────────────────────────────────────────────
    // CAUSALITY-OPERATIONEN (A11-A16)
    // ─────────────────────────────────────────────────────────────────────────

    /// Gibt kausale Tiefe zurück: |ℂ(s)|
    #[wasm_bindgen(js_name = "ery_causal_depth")]
    pub fn host_causal_depth(did: &str) -> u32;

    /// Prüft ob Event bezeugt: ⟦e⟧
    #[wasm_bindgen(js_name = "ery_is_witnessed")]
    pub fn host_is_witnessed(event_hash: &str) -> bool;

    /// Prüft ob Event final: ∎e
    #[wasm_bindgen(js_name = "ery_is_final")]
    pub fn host_is_final(event_hash: &str) -> bool;

    // ─────────────────────────────────────────────────────────────────────────
    // REALM-OPERATIONEN (A17-A20)
    // ─────────────────────────────────────────────────────────────────────────

    /// Prüft Realm-Mitgliedschaft: s ∈ R
    #[wasm_bindgen(js_name = "ery_is_member")]
    pub fn host_is_member(entity: &str, realm: &str) -> bool;

    /// Prüft ob Aktion im Realm erlaubt: [R]◇(s : α)
    #[wasm_bindgen(js_name = "ery_action_allowed")]
    pub fn host_action_allowed(realm: &str, entity: &str, action: &str) -> bool;

    /// Gibt Realm-Regeln zurück
    #[wasm_bindgen(js_name = "ery_get_realm_rules")]
    pub fn host_get_realm_rules(realm: &str) -> JsValue;

    // ─────────────────────────────────────────────────────────────────────────
    // VALUE-OPERATIONEN
    // ─────────────────────────────────────────────────────────────────────────

    /// Gibt Asset-Wert zurück: 𝕍(x)
    #[wasm_bindgen(js_name = "ery_asset_value")]
    pub fn host_asset_value(asset_id: &str) -> f64;

    /// Prüft Eigentum: x ↝ s
    #[wasm_bindgen(js_name = "ery_owns")]
    pub fn host_owns(owner: &str, asset: &str) -> bool;

    /// Gibt Balance für Entity zurück
    #[wasm_bindgen(js_name = "ery_get_balance")]
    pub fn host_get_balance(did: &str, currency: &str) -> f64;
}

// ═══════════════════════════════════════════════════════════════════════════════
// NOA-INTERFACE: Ledger + Finality
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
extern "C" {
    /// Sendet Event an NOA-Ledger
    #[wasm_bindgen(js_name = "noa_submit_event")]
    pub fn host_submit_event(event_json: &str) -> String;  // -> event_hash

    /// Wartet auf Finalisierung
    #[wasm_bindgen(js_name = "noa_await_finality")]
    pub fn host_await_finality(event_hash: &str, level: u8) -> js_sys::Promise;

    /// Liest AMO-State
    #[wasm_bindgen(js_name = "noa_get_amo")]
    pub fn host_get_amo(amo_id: &str) -> JsValue;

    /// Führt AMO-Transition aus
    #[wasm_bindgen(js_name = "noa_transition_amo")]
    pub fn host_transition_amo(amo_id: &str, transition: &str) -> js_sys::Promise;
}

// ═══════════════════════════════════════════════════════════════════════════════
// CRYPTO-INTERFACE
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
extern "C" {
    /// Signiert Daten mit Agent-Key
    #[wasm_bindgen(js_name = "crypto_sign")]
    pub fn host_sign(data: &[u8], key_id: &str) -> Box<[u8]>;

    /// Verifiziert Signatur
    #[wasm_bindgen(js_name = "crypto_verify")]
    pub fn host_verify(data: &[u8], signature: &[u8], public_key: &[u8]) -> bool;

    /// Generiert Zufallszahlen
    #[wasm_bindgen(js_name = "crypto_random")]
    pub fn host_random(len: usize) -> Box<[u8]>;

    /// Hash-Funktion
    #[wasm_bindgen(js_name = "crypto_hash")]
    pub fn host_hash(data: &[u8]) -> Box<[u8]>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// STORAGE-INTERFACE
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
extern "C" {
    /// Speichert Wert (Key-Value)
    #[wasm_bindgen(js_name = "storage_set")]
    pub fn host_storage_set(key: &str, value: &[u8]);

    /// Liest Wert
    #[wasm_bindgen(js_name = "storage_get")]
    pub fn host_storage_get(key: &str) -> Option<Box<[u8]>>;

    /// Löscht Wert
    #[wasm_bindgen(js_name = "storage_delete")]
    pub fn host_storage_delete(key: &str);
}
```

---

# Teil III: Core-Typen (echo-core-wasm)

## 3.1 Basis-Typen

```rust
// crates/echo-core-wasm/src/types.rs

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
// DID + ENTITY
// ═══════════════════════════════════════════════════════════════════════════════

/// DID (Decentralized Identifier)
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Did {
    inner: String,
}

#[wasm_bindgen]
impl Did {
    #[wasm_bindgen(constructor)]
    pub fn new(s: &str) -> Result<Did, JsValue> {
        if s.starts_with("did:erynoa:") {
            Ok(Did { inner: s.to_string() })
        } else {
            Err(JsValue::from_str("Invalid DID format"))
        }
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> String {
        self.inner.clone()
    }

    /// Extrahiert Namespace
    #[wasm_bindgen]
    pub fn namespace(&self) -> String {
        self.inner
            .strip_prefix("did:erynoa:")
            .and_then(|s| s.split(':').next())
            .unwrap_or("")
            .to_string()
    }

    /// Prüft ob Agent-DID
    #[wasm_bindgen]
    pub fn is_agent(&self) -> bool {
        self.namespace() == "agent"
    }

    /// Prüft Existenz via Host: ⟨s⟩
    #[wasm_bindgen]
    pub fn exists(&self) -> bool {
        crate::host::host_exists(&self.inner)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRUST VECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// 4-dimensionaler Trust-Vektor: 𝕋(s) = (R, I, C, P)
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrustVector {
    reliability: f64,
    integrity: f64,
    capability: f64,
    prestige: f64,
}

#[wasm_bindgen]
impl TrustVector {
    #[wasm_bindgen(constructor)]
    pub fn new(r: f64, i: f64, c: f64, p: f64) -> TrustVector {
        TrustVector {
            reliability: r.clamp(0.3, 1.0),
            integrity: i.clamp(0.3, 1.0),
            capability: c.clamp(0.3, 1.0),
            prestige: p.clamp(0.3, 1.0),
        }
    }

    #[wasm_bindgen]
    pub fn initial() -> TrustVector {
        TrustVector::new(0.5, 0.5, 0.5, 0.5)
    }

    /// 𝕋̄(s) = (R + I + C + P) / 4
    #[wasm_bindgen]
    pub fn aggregate(&self) -> f64 {
        (self.reliability + self.integrity + self.capability + self.prestige) / 4.0
    }

    #[wasm_bindgen(getter)]
    pub fn reliability(&self) -> f64 { self.reliability }
    
    #[wasm_bindgen(getter)]
    pub fn integrity(&self) -> f64 { self.integrity }
    
    #[wasm_bindgen(getter)]
    pub fn capability(&self) -> f64 { self.capability }
    
    #[wasm_bindgen(getter)]
    pub fn prestige(&self) -> f64 { self.prestige }

    /// Von Host laden
    #[wasm_bindgen]
    pub fn from_host(did: &Did) -> TrustVector {
        let arr = crate::host::host_get_trust(&did.value());
        TrustVector::new(arr[0], arr[1], arr[2], arr[3])
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ATTENTION FUNCTION
// ═══════════════════════════════════════════════════════════════════════════════

/// σ(x) = 1 / (1 + e^(-x))
#[wasm_bindgen]
pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Aufmerksamkeits-Score: σ(s) = σ(𝕋̄(s) · ln|ℂ(s)|)
#[wasm_bindgen]
pub fn attention_score(trust: &TrustVector, causal_depth: u32) -> f64 {
    let ln_c = (causal_depth.max(1) as f64).ln();
    sigmoid(trust.aggregate() * ln_c)
}

/// Lädt Attention von Host
#[wasm_bindgen]
pub fn get_attention(did: &Did) -> f64 {
    crate::host::host_attention(&did.value())
}
```

## 3.2 Agent-Typen

```rust
// crates/echo-core-wasm/src/agent.rs

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use crate::types::{Did, TrustVector};

/// Agent-Typen (A21: Handlungsfähigkeit erfordert Existenz)
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    /// Sucht Ressourcen/Dienste
    Seeker,
    /// Bietet Ressourcen/Dienste
    Provider,
    /// Vermittelt zwischen Parteien
    Broker,
    /// Liefert externe Daten
    Oracle,
    /// Prüft und bestätigt
    Validator,
}

/// Agent-Status
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    /// Erstellt, nicht aktiv
    Created,
    /// Konfiguriert
    Configured,
    /// Aktiv und handelnd
    Active,
    /// Pausiert
    Suspended,
    /// Beendet
    Terminated,
}

/// Agent-Repräsentation in WASM
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    did: Did,
    agent_type: AgentType,
    owner: Did,
    status: AgentStatus,
    policy_id: Option<String>,
    wallet_balance: f64,
}

#[wasm_bindgen]
impl Agent {
    /// Lädt Agent vom Host
    #[wasm_bindgen]
    pub fn load(did: &Did) -> Result<Agent, JsValue> {
        let entity_js = crate::host::host_resolve_did(&did.value());
        
        if entity_js.is_null() || entity_js.is_undefined() {
            return Err(JsValue::from_str("Agent not found"));
        }
        
        // Parse Entity JSON
        let agent: Agent = serde_wasm_bindgen::from_value(entity_js)?;
        Ok(agent)
    }

    #[wasm_bindgen(getter)]
    pub fn did(&self) -> Did { self.did.clone() }

    #[wasm_bindgen(getter)]
    pub fn agent_type(&self) -> AgentType { self.agent_type.clone() }

    #[wasm_bindgen(getter)]
    pub fn owner(&self) -> Did { self.owner.clone() }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> AgentStatus { self.status.clone() }

    /// Prüft Handlungsfähigkeit: ⟨s⟩ ∧ status = Active (A21)
    #[wasm_bindgen]
    pub fn can_act(&self) -> bool {
        self.did.exists() && self.status == AgentStatus::Active
    }

    /// Gibt Trust des Agents zurück
    #[wasm_bindgen]
    pub fn trust(&self) -> TrustVector {
        TrustVector::from_host(&self.did)
    }

    /// Gibt Attention zurück: σ(agent)
    #[wasm_bindgen]
    pub fn attention(&self) -> f64 {
        crate::host::host_attention(&self.did.value())
    }

    /// Prüft ob Aktion möglich: ◇α (A22, A23)
    #[wasm_bindgen]
    pub fn can_perform(&self, action: &str, realm: &str) -> bool {
        if !self.can_act() {
            return false;
        }
        
        // A22: Realm-Erlaubnis
        // A23: Trust-Threshold
        crate::host::host_action_allowed(realm, &self.did.value(), action)
    }
}
```

---

# Teil IV: ECLVM Runtime (echo-eclvm-wasm)

## 4.1 Bytecode-Interpreter

```rust
// crates/echo-eclvm-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use std::collections::HashMap;

/// Gas-Kosten für Operationen
pub mod gas {
    pub const ARITHMETIC: u64 = 1;
    pub const COMPARISON: u64 = 1;
    pub const LOGICAL: u64 = 1;
    pub const VAR_ACCESS: u64 = 2;
    pub const FUNCTION_CALL: u64 = 10;
    pub const DID_RESOLUTION: u64 = 50;
    pub const CREDENTIAL_VERIFY: u64 = 100;
    pub const TRUST_LOOKUP: u64 = 30;
    pub const BLUEPRINT_VALIDATE: u64 = 200;
}

/// Opcodes für ECLVM
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opcode {
    // Stack-Operationen
    Push = 0x01,
    Pop = 0x02,
    Dup = 0x03,
    Swap = 0x04,
    
    // Arithmetik
    Add = 0x10,
    Sub = 0x11,
    Mul = 0x12,
    Div = 0x13,
    Mod = 0x14,
    
    // Vergleiche
    Eq = 0x20,
    Neq = 0x21,
    Lt = 0x22,
    Lte = 0x23,
    Gt = 0x24,
    Gte = 0x25,
    
    // Logik
    And = 0x30,
    Or = 0x31,
    Not = 0x32,
    
    // Variablen
    Load = 0x40,
    Store = 0x41,
    
    // Kontrolle
    Jump = 0x50,
    JumpIf = 0x51,
    Call = 0x52,
    Return = 0x53,
    
    // Host-Calls (ERY/NOA)
    HostExists = 0x60,
    HostTrust = 0x61,
    HostAttention = 0x62,
    HostRealmCheck = 0x63,
    HostCredential = 0x64,
    
    // Beenden
    Halt = 0xFF,
}

/// Wert auf dem Stack
#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

impl Value {
    pub fn as_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
            _ => true,
        }
    }
    
    pub fn as_f64(&self) -> f64 {
        match self {
            Value::Float(f) => *f,
            Value::Int(i) => *i as f64,
            _ => 0.0,
        }
    }
}

/// Execution-Context
#[wasm_bindgen]
pub struct ExecutionContext {
    stack: Vec<Value>,
    variables: HashMap<String, Value>,
    gas_used: u64,
    gas_limit: u64,
    pc: usize,  // Program Counter
    halted: bool,
    error: Option<String>,
}

#[wasm_bindgen]
impl ExecutionContext {
    #[wasm_bindgen(constructor)]
    pub fn new(gas_limit: u64) -> ExecutionContext {
        ExecutionContext {
            stack: Vec::with_capacity(256),
            variables: HashMap::new(),
            gas_used: 0,
            gas_limit,
            pc: 0,
            halted: false,
            error: None,
        }
    }

    /// Verbraucht Gas
    fn consume_gas(&mut self, amount: u64) -> Result<(), String> {
        self.gas_used += amount;
        if self.gas_used > self.gas_limit {
            self.halted = true;
            self.error = Some("Gas exhausted".to_string());
            Err("Gas exhausted".to_string())
        } else {
            Ok(())
        }
    }

    /// Push auf Stack
    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Pop vom Stack
    fn pop(&mut self) -> Result<Value, String> {
        self.stack.pop().ok_or_else(|| "Stack underflow".to_string())
    }

    #[wasm_bindgen(getter)]
    pub fn gas_used(&self) -> u64 { self.gas_used }

    #[wasm_bindgen(getter)]
    pub fn gas_remaining(&self) -> u64 { self.gas_limit.saturating_sub(self.gas_used) }

    #[wasm_bindgen(getter)]
    pub fn is_halted(&self) -> bool { self.halted }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> { self.error.clone() }
}

/// ECLVM Virtual Machine
#[wasm_bindgen]
pub struct EclVm {
    bytecode: Vec<u8>,
    context: ExecutionContext,
}

#[wasm_bindgen]
impl EclVm {
    /// Erstellt neue VM mit Bytecode
    #[wasm_bindgen(constructor)]
    pub fn new(bytecode: Vec<u8>, gas_limit: u64) -> EclVm {
        EclVm {
            bytecode,
            context: ExecutionContext::new(gas_limit),
        }
    }

    /// Führt Bytecode aus
    #[wasm_bindgen]
    pub fn execute(&mut self) -> Result<JsValue, JsValue> {
        while !self.context.halted && self.context.pc < self.bytecode.len() {
            let opcode = self.bytecode[self.context.pc];
            self.context.pc += 1;
            
            match self.execute_opcode(opcode) {
                Ok(()) => {},
                Err(e) => {
                    self.context.halted = true;
                    self.context.error = Some(e.clone());
                    return Err(JsValue::from_str(&e));
                }
            }
        }
        
        // Ergebnis vom Stack
        let result = self.context.stack.last()
            .map(|v| self.value_to_js(v))
            .unwrap_or(JsValue::NULL);
        
        Ok(result)
    }

    /// Führt einzelnen Opcode aus
    fn execute_opcode(&mut self, opcode: u8) -> Result<(), String> {
        match opcode {
            // Push konstanten Wert
            x if x == Opcode::Push as u8 => {
                // Nächste Bytes lesen für Wert
                let value = self.read_value()?;
                self.context.push(value);
                Ok(())
            }
            
            // Pop
            x if x == Opcode::Pop as u8 => {
                self.context.consume_gas(gas::ARITHMETIC)?;
                self.context.pop()?;
                Ok(())
            }
            
            // Arithmetik
            x if x == Opcode::Add as u8 => {
                self.context.consume_gas(gas::ARITHMETIC)?;
                let b = self.context.pop()?.as_f64();
                let a = self.context.pop()?.as_f64();
                self.context.push(Value::Float(a + b));
                Ok(())
            }
            
            x if x == Opcode::Sub as u8 => {
                self.context.consume_gas(gas::ARITHMETIC)?;
                let b = self.context.pop()?.as_f64();
                let a = self.context.pop()?.as_f64();
                self.context.push(Value::Float(a - b));
                Ok(())
            }
            
            x if x == Opcode::Mul as u8 => {
                self.context.consume_gas(gas::ARITHMETIC)?;
                let b = self.context.pop()?.as_f64();
                let a = self.context.pop()?.as_f64();
                self.context.push(Value::Float(a * b));
                Ok(())
            }
            
            x if x == Opcode::Div as u8 => {
                self.context.consume_gas(gas::ARITHMETIC)?;
                let b = self.context.pop()?.as_f64();
                let a = self.context.pop()?.as_f64();
                if b == 0.0 {
                    return Err("Division by zero".to_string());
                }
                self.context.push(Value::Float(a / b));
                Ok(())
            }
            
            // Vergleiche
            x if x == Opcode::Eq as u8 => {
                self.context.consume_gas(gas::COMPARISON)?;
                let b = self.context.pop()?.as_f64();
                let a = self.context.pop()?.as_f64();
                self.context.push(Value::Bool((a - b).abs() < f64::EPSILON));
                Ok(())
            }
            
            x if x == Opcode::Lt as u8 => {
                self.context.consume_gas(gas::COMPARISON)?;
                let b = self.context.pop()?.as_f64();
                let a = self.context.pop()?.as_f64();
                self.context.push(Value::Bool(a < b));
                Ok(())
            }
            
            x if x == Opcode::Lte as u8 => {
                self.context.consume_gas(gas::COMPARISON)?;
                let b = self.context.pop()?.as_f64();
                let a = self.context.pop()?.as_f64();
                self.context.push(Value::Bool(a <= b));
                Ok(())
            }
            
            x if x == Opcode::Gt as u8 => {
                self.context.consume_gas(gas::COMPARISON)?;
                let b = self.context.pop()?.as_f64();
                let a = self.context.pop()?.as_f64();
                self.context.push(Value::Bool(a > b));
                Ok(())
            }
            
            x if x == Opcode::Gte as u8 => {
                self.context.consume_gas(gas::COMPARISON)?;
                let b = self.context.pop()?.as_f64();
                let a = self.context.pop()?.as_f64();
                self.context.push(Value::Bool(a >= b));
                Ok(())
            }
            
            // Logik
            x if x == Opcode::And as u8 => {
                self.context.consume_gas(gas::LOGICAL)?;
                let b = self.context.pop()?.as_bool();
                let a = self.context.pop()?.as_bool();
                self.context.push(Value::Bool(a && b));
                Ok(())
            }
            
            x if x == Opcode::Or as u8 => {
                self.context.consume_gas(gas::LOGICAL)?;
                let b = self.context.pop()?.as_bool();
                let a = self.context.pop()?.as_bool();
                self.context.push(Value::Bool(a || b));
                Ok(())
            }
            
            x if x == Opcode::Not as u8 => {
                self.context.consume_gas(gas::LOGICAL)?;
                let a = self.context.pop()?.as_bool();
                self.context.push(Value::Bool(!a));
                Ok(())
            }
            
            // Host-Calls
            x if x == Opcode::HostExists as u8 => {
                self.context.consume_gas(gas::DID_RESOLUTION)?;
                let did = match self.context.pop()? {
                    Value::String(s) => s,
                    _ => return Err("Expected string DID".to_string()),
                };
                let exists = echo_host_wasm::host_exists(&did);
                self.context.push(Value::Bool(exists));
                Ok(())
            }
            
            x if x == Opcode::HostTrust as u8 => {
                self.context.consume_gas(gas::TRUST_LOOKUP)?;
                let did = match self.context.pop()? {
                    Value::String(s) => s,
                    _ => return Err("Expected string DID".to_string()),
                };
                let trust = echo_host_wasm::host_get_trust_aggregate(&did);
                self.context.push(Value::Float(trust));
                Ok(())
            }
            
            x if x == Opcode::HostAttention as u8 => {
                self.context.consume_gas(gas::TRUST_LOOKUP)?;
                let did = match self.context.pop()? {
                    Value::String(s) => s,
                    _ => return Err("Expected string DID".to_string()),
                };
                let attention = echo_host_wasm::host_attention(&did);
                self.context.push(Value::Float(attention));
                Ok(())
            }
            
            x if x == Opcode::HostRealmCheck as u8 => {
                self.context.consume_gas(gas::TRUST_LOOKUP)?;
                let action = match self.context.pop()? {
                    Value::String(s) => s,
                    _ => return Err("Expected action string".to_string()),
                };
                let entity = match self.context.pop()? {
                    Value::String(s) => s,
                    _ => return Err("Expected entity DID".to_string()),
                };
                let realm = match self.context.pop()? {
                    Value::String(s) => s,
                    _ => return Err("Expected realm DID".to_string()),
                };
                let allowed = echo_host_wasm::host_action_allowed(&realm, &entity, &action);
                self.context.push(Value::Bool(allowed));
                Ok(())
            }
            
            // Halt
            x if x == Opcode::Halt as u8 => {
                self.context.halted = true;
                Ok(())
            }
            
            _ => Err(format!("Unknown opcode: 0x{:02x}", opcode)),
        }
    }

    /// Liest Wert aus Bytecode
    fn read_value(&mut self) -> Result<Value, String> {
        // Typ-Byte lesen
        if self.context.pc >= self.bytecode.len() {
            return Err("Unexpected end of bytecode".to_string());
        }
        
        let type_byte = self.bytecode[self.context.pc];
        self.context.pc += 1;
        
        match type_byte {
            0x00 => Ok(Value::Null),
            0x01 => {
                // Bool
                let b = self.bytecode.get(self.context.pc).copied().unwrap_or(0);
                self.context.pc += 1;
                Ok(Value::Bool(b != 0))
            }
            0x02 => {
                // f64 (8 bytes)
                let bytes: [u8; 8] = self.bytecode[self.context.pc..self.context.pc + 8]
                    .try_into()
                    .map_err(|_| "Invalid f64")?;
                self.context.pc += 8;
                Ok(Value::Float(f64::from_le_bytes(bytes)))
            }
            0x03 => {
                // String (length + bytes)
                let len = self.bytecode[self.context.pc] as usize;
                self.context.pc += 1;
                let s = String::from_utf8_lossy(
                    &self.bytecode[self.context.pc..self.context.pc + len]
                ).to_string();
                self.context.pc += len;
                Ok(Value::String(s))
            }
            _ => Err(format!("Unknown value type: 0x{:02x}", type_byte)),
        }
    }

    fn value_to_js(&self, value: &Value) -> JsValue {
        match value {
            Value::Null => JsValue::NULL,
            Value::Bool(b) => JsValue::from_bool(*b),
            Value::Int(i) => JsValue::from_f64(*i as f64),
            Value::Float(f) => JsValue::from_f64(*f),
            Value::String(s) => JsValue::from_str(s),
            _ => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn gas_used(&self) -> u64 { self.context.gas_used }

    #[wasm_bindgen(getter)]
    pub fn gas_remaining(&self) -> u64 { self.context.gas_remaining() }
}
```

---

# Teil V: Policy-Engine (echo-policy-wasm)

## 5.1 Policy-Evaluator

```rust
// crates/echo-policy-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use echo_core_wasm::types::{Did, TrustVector};

/// Policy-Entscheidung
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Decision {
    /// Automatisch akzeptieren
    Accept,
    /// Automatisch ablehnen
    Reject,
    /// An Owner eskalieren
    Escalate,
    /// Weiter evaluieren (keine Entscheidung)
    Continue,
}

/// Evaluierungs-Kontext
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyContext {
    /// Agent, der die Policy ausführt
    agent_did: String,
    /// Trust des Agents
    agent_trust: f64,
    /// Verbleibendes Budget
    budget_remaining: f64,
    /// Aktueller Zeitstempel (Unix ms)
    current_time: u64,
    /// Angebotsdaten (JSON)
    offer_data: String,
}

#[wasm_bindgen]
impl PolicyContext {
    #[wasm_bindgen(constructor)]
    pub fn new(
        agent_did: &str,
        agent_trust: f64,
        budget_remaining: f64,
        current_time: u64,
        offer_data: &str,
    ) -> PolicyContext {
        PolicyContext {
            agent_did: agent_did.to_string(),
            agent_trust,
            budget_remaining,
            current_time,
            offer_data: offer_data.to_string(),
        }
    }
}

/// Bedingung für Policy-Regel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: String,  // eq, neq, lt, lte, gt, gte, in, contains
    pub value: serde_json::Value,
}

impl Condition {
    /// Evaluiert Bedingung gegen Kontext
    pub fn evaluate(&self, context: &PolicyContext) -> bool {
        let offer: serde_json::Value = serde_json::from_str(&context.offer_data)
            .unwrap_or(serde_json::Value::Null);
        
        // Wert aus Offer extrahieren
        let actual_value = match self.field.as_str() {
            "agent_trust" => serde_json::Value::from(context.agent_trust),
            "budget_remaining" => serde_json::Value::from(context.budget_remaining),
            field => offer.get(field).cloned().unwrap_or(serde_json::Value::Null),
        };
        
        // Vergleichen
        match self.operator.as_str() {
            "eq" => actual_value == self.value,
            "neq" => actual_value != self.value,
            "lt" => {
                let a = actual_value.as_f64().unwrap_or(0.0);
                let b = self.value.as_f64().unwrap_or(0.0);
                a < b
            }
            "lte" => {
                let a = actual_value.as_f64().unwrap_or(0.0);
                let b = self.value.as_f64().unwrap_or(0.0);
                a <= b
            }
            "gt" => {
                let a = actual_value.as_f64().unwrap_or(0.0);
                let b = self.value.as_f64().unwrap_or(0.0);
                a > b
            }
            "gte" => {
                let a = actual_value.as_f64().unwrap_or(0.0);
                let b = self.value.as_f64().unwrap_or(0.0);
                a >= b
            }
            "in" => {
                if let Some(arr) = self.value.as_array() {
                    arr.contains(&actual_value)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

/// Policy-Regel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyRule {
    pub conditions: Vec<Condition>,
    pub combine: String,  // AND, OR
    pub decision: Decision,
}

impl PolicyRule {
    pub fn matches(&self, context: &PolicyContext) -> bool {
        match self.combine.as_str() {
            "AND" => self.conditions.iter().all(|c| c.evaluate(context)),
            "OR" => self.conditions.iter().any(|c| c.evaluate(context)),
            _ => false,
        }
    }
}

/// Policy-Definition
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    id: String,
    name: String,
    auto_reject: Vec<PolicyRule>,
    auto_accept: Vec<PolicyRule>,
    escalate: Vec<PolicyRule>,
}

#[wasm_bindgen]
impl Policy {
    /// Parst Policy aus JSON
    #[wasm_bindgen]
    pub fn from_json(json: &str) -> Result<Policy, JsValue> {
        serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))
    }

    /// Evaluiert Policy
    #[wasm_bindgen]
    pub fn evaluate(&self, context: &PolicyContext) -> PolicyResult {
        let mut reasoning = Vec::new();
        
        // 1. Check auto_reject (höchste Priorität)
        for rule in &self.auto_reject {
            if rule.matches(context) {
                reasoning.push("auto_reject matched".to_string());
                return PolicyResult {
                    decision: Decision::Reject,
                    reasoning,
                    gas_used: 0,
                };
            }
        }
        reasoning.push("auto_reject: no match".to_string());
        
        // 2. Check auto_accept
        for rule in &self.auto_accept {
            if rule.matches(context) {
                reasoning.push("auto_accept matched".to_string());
                return PolicyResult {
                    decision: Decision::Accept,
                    reasoning,
                    gas_used: 0,
                };
            }
        }
        reasoning.push("auto_accept: no match".to_string());
        
        // 3. Check escalate
        for rule in &self.escalate {
            if rule.matches(context) {
                reasoning.push("escalate matched".to_string());
                return PolicyResult {
                    decision: Decision::Escalate,
                    reasoning,
                    gas_used: 0,
                };
            }
        }
        
        // Default: Reject
        reasoning.push("no rules matched, default reject".to_string());
        PolicyResult {
            decision: Decision::Reject,
            reasoning,
            gas_used: 0,
        }
    }
}

/// Ergebnis der Policy-Evaluation
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyResult {
    decision: Decision,
    reasoning: Vec<String>,
    gas_used: u64,
}

#[wasm_bindgen]
impl PolicyResult {
    #[wasm_bindgen(getter)]
    pub fn decision(&self) -> Decision { self.decision.clone() }

    #[wasm_bindgen(getter)]
    pub fn reasoning(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.reasoning).unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter)]
    pub fn gas_used(&self) -> u64 { self.gas_used }
}
```

---

# Teil VI: Intent-Matching (echo-intent-wasm)

## 6.1 Intent-Matcher

```rust
// crates/echo-intent-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Intent-Status
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentStatus {
    Created,
    Matching,
    Matched,
    Negotiating,
    Agreed,
    Executing,
    Completed,
    Cancelled,
    Expired,
    Failed,
}

/// Intent-Constraint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constraint {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
}

/// Intent-Definition
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intent {
    id: String,
    intent_type: String,
    seeker: String,
    environment: String,
    constraints: Vec<Constraint>,
    budget_max: f64,
    currency: String,
    trust_min: f64,
    priorities: serde_json::Value,
    status: IntentStatus,
    expires_at: u64,
}

#[wasm_bindgen]
impl Intent {
    /// Parst Intent aus JSON
    #[wasm_bindgen]
    pub fn from_json(json: &str) -> Result<Intent, JsValue> {
        serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String { self.id.clone() }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> IntentStatus { self.status.clone() }

    /// Prüft ob Intent abgelaufen
    #[wasm_bindgen]
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time > self.expires_at
    }
}

/// Provider-Angebot
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Offer {
    id: String,
    provider: String,
    price: f64,
    currency: String,
    trust: f64,
    attributes: serde_json::Value,
    valid_until: u64,
}

#[wasm_bindgen]
impl Offer {
    #[wasm_bindgen]
    pub fn from_json(json: &str) -> Result<Offer, JsValue> {
        serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String { self.id.clone() }

    #[wasm_bindgen(getter)]
    pub fn price(&self) -> f64 { self.price }

    #[wasm_bindgen(getter)]
    pub fn trust(&self) -> f64 { self.trust }
}

/// Match-Score für ein Angebot
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchScore {
    offer_id: String,
    total_score: f64,
    constraint_matches: Vec<bool>,
    dimension_scores: serde_json::Value,
}

#[wasm_bindgen]
impl MatchScore {
    #[wasm_bindgen(getter)]
    pub fn offer_id(&self) -> String { self.offer_id.clone() }

    #[wasm_bindgen(getter)]
    pub fn total_score(&self) -> f64 { self.total_score }

    #[wasm_bindgen(getter)]
    pub fn matches_all(&self) -> bool {
        self.constraint_matches.iter().all(|&m| m)
    }
}

/// Intent-Matching-Engine
#[wasm_bindgen]
pub struct IntentMatcher {
    intent: Intent,
}

#[wasm_bindgen]
impl IntentMatcher {
    #[wasm_bindgen(constructor)]
    pub fn new(intent: Intent) -> IntentMatcher {
        IntentMatcher { intent }
    }

    /// Matched ein einzelnes Angebot
    #[wasm_bindgen]
    pub fn match_offer(&self, offer: &Offer) -> MatchScore {
        let mut constraint_matches = Vec::new();
        let mut dimension_scores = serde_json::Map::new();
        
        // Prüfe alle Constraints
        for constraint in &self.intent.constraints {
            let matches = self.evaluate_constraint(constraint, offer);
            constraint_matches.push(matches);
        }
        
        // Prüfe Budget
        let budget_ok = offer.price <= self.intent.budget_max;
        constraint_matches.push(budget_ok);
        
        // Prüfe Trust (A23)
        let trust_ok = offer.trust >= self.intent.trust_min;
        constraint_matches.push(trust_ok);
        
        // Berechne dimensionsbezogene Scores
        let priorities = self.intent.priorities.as_object()
            .cloned()
            .unwrap_or_default();
        
        // Preis-Score (niedriger ist besser)
        let price_weight = priorities.get("price")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25);
        let price_score = if self.intent.budget_max > 0.0 {
            1.0 - (offer.price / self.intent.budget_max).min(1.0)
        } else {
            0.0
        };
        dimension_scores.insert("price".to_string(), serde_json::Value::from(price_score * price_weight));
        
        // Trust-Score (höher ist besser)
        let trust_weight = priorities.get("trust")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.25);
        let trust_score = offer.trust;
        dimension_scores.insert("trust".to_string(), serde_json::Value::from(trust_score * trust_weight));
        
        // Gesamt-Score
        let total_score = if constraint_matches.iter().all(|&m| m) {
            dimension_scores.values()
                .filter_map(|v| v.as_f64())
                .sum()
        } else {
            0.0
        };
        
        MatchScore {
            offer_id: offer.id.clone(),
            total_score,
            constraint_matches,
            dimension_scores: serde_json::Value::Object(dimension_scores),
        }
    }

    /// Matched mehrere Angebote und sortiert nach Score
    #[wasm_bindgen]
    pub fn match_offers(&self, offers_json: &str) -> Result<JsValue, JsValue> {
        let offers: Vec<Offer> = serde_json::from_str(offers_json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        let mut scores: Vec<MatchScore> = offers.iter()
            .map(|o| self.match_offer(o))
            .filter(|s| s.matches_all())
            .collect();
        
        // Sortiere nach Score (absteigend)
        scores.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap());
        
        serde_wasm_bindgen::to_value(&scores)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }

    fn evaluate_constraint(&self, constraint: &Constraint, offer: &Offer) -> bool {
        let actual = offer.attributes.get(&constraint.field)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        
        match constraint.operator.as_str() {
            "eq" => actual == constraint.value,
            "gte" => {
                let a = actual.as_f64().unwrap_or(0.0);
                let b = constraint.value.as_f64().unwrap_or(0.0);
                a >= b
            }
            "lte" => {
                let a = actual.as_f64().unwrap_or(0.0);
                let b = constraint.value.as_f64().unwrap_or(0.0);
                a <= b
            }
            "in" => {
                if let Some(arr) = constraint.value.as_array() {
                    arr.contains(&actual)
                } else {
                    false
                }
            }
            "contains" => {
                if let (Some(haystack), Some(needle)) = (actual.as_str(), constraint.value.as_str()) {
                    haystack.contains(needle)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
```

---

# Teil VII: Negotiation-Engine (echo-negotiation-wasm)

## 7.1 Verhandlungs-Logik

```rust
// crates/echo-negotiation-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Verhandlungsmodelle
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NegotiationModel {
    /// Take-it-or-leave-it
    Direct,
    /// Mehrere Bieter
    Auction,
    /// Mehrere Runden
    MultiRound,
}

/// Verhandlungsstatus
#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NegotiationStatus {
    Open,
    CounterOffered,
    Accepted,
    Rejected,
    Timeout,
    Cancelled,
}

/// Verhandlungsrunde
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Round {
    pub number: u32,
    pub from: String,  // seeker | provider
    pub terms: serde_json::Value,
    pub status: NegotiationStatus,
    pub timestamp: u64,
}

/// Verhandlungs-Session
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NegotiationSession {
    id: String,
    intent_id: String,
    seeker: String,
    provider: String,
    model: NegotiationModel,
    rounds: Vec<Round>,
    max_rounds: u32,
    round_timeout: u64,  // Sekunden
    current_status: NegotiationStatus,
}

#[wasm_bindgen]
impl NegotiationSession {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: &str,
        intent_id: &str,
        seeker: &str,
        provider: &str,
        model: NegotiationModel,
        max_rounds: u32,
        round_timeout: u64,
    ) -> NegotiationSession {
        NegotiationSession {
            id: id.to_string(),
            intent_id: intent_id.to_string(),
            seeker: seeker.to_string(),
            provider: provider.to_string(),
            model,
            rounds: Vec::new(),
            max_rounds,
            round_timeout,
            current_status: NegotiationStatus::Open,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String { self.id.clone() }

    #[wasm_bindgen(getter)]
    pub fn current_round(&self) -> u32 { self.rounds.len() as u32 }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> NegotiationStatus { self.current_status.clone() }

    /// Fügt neue Runde hinzu
    #[wasm_bindgen]
    pub fn add_round(
        &mut self,
        from: &str,
        terms_json: &str,
        action: &str,  // offer | counter | accept | reject
        timestamp: u64,
    ) -> Result<(), JsValue> {
        // Prüfe ob Verhandlung noch offen
        if self.current_status != NegotiationStatus::Open 
            && self.current_status != NegotiationStatus::CounterOffered {
            return Err(JsValue::from_str("Negotiation not open"));
        }
        
        // Prüfe Max-Runden
        if self.rounds.len() as u32 >= self.max_rounds {
            self.current_status = NegotiationStatus::Timeout;
            return Err(JsValue::from_str("Max rounds exceeded"));
        }
        
        let terms: serde_json::Value = serde_json::from_str(terms_json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        let status = match action {
            "offer" | "counter" => NegotiationStatus::CounterOffered,
            "accept" => NegotiationStatus::Accepted,
            "reject" => NegotiationStatus::Rejected,
            _ => return Err(JsValue::from_str("Invalid action")),
        };
        
        self.rounds.push(Round {
            number: self.rounds.len() as u32 + 1,
            from: from.to_string(),
            terms,
            status: status.clone(),
            timestamp,
        });
        
        self.current_status = status;
        
        Ok(())
    }

    /// Prüft ob Runde timeout hat
    #[wasm_bindgen]
    pub fn check_timeout(&mut self, current_time: u64) -> bool {
        if let Some(last) = self.rounds.last() {
            if current_time - last.timestamp > self.round_timeout * 1000 {
                self.current_status = NegotiationStatus::Timeout;
                return true;
            }
        }
        false
    }

    /// Gibt finale Bedingungen zurück (wenn Accepted)
    #[wasm_bindgen]
    pub fn final_terms(&self) -> JsValue {
        if self.current_status != NegotiationStatus::Accepted {
            return JsValue::NULL;
        }
        
        self.rounds.last()
            .map(|r| serde_wasm_bindgen::to_value(&r.terms).unwrap_or(JsValue::NULL))
            .unwrap_or(JsValue::NULL)
    }

    /// Exportiert Session als JSON
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(self)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }
}

/// Agreement (nach erfolgreicher Verhandlung)
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agreement {
    id: String,
    negotiation_id: String,
    seeker: String,
    provider: String,
    terms: serde_json::Value,
    created_at: u64,
    status: String,
}

#[wasm_bindgen]
impl Agreement {
    /// Erstellt Agreement aus abgeschlossener Verhandlung
    #[wasm_bindgen]
    pub fn from_negotiation(
        agreement_id: &str,
        session: &NegotiationSession,
        created_at: u64,
    ) -> Result<Agreement, JsValue> {
        if session.status() != NegotiationStatus::Accepted {
            return Err(JsValue::from_str("Negotiation not accepted"));
        }
        
        let terms = session.final_terms();
        let terms_value: serde_json::Value = serde_wasm_bindgen::from_value(terms)?;
        
        Ok(Agreement {
            id: agreement_id.to_string(),
            negotiation_id: session.id(),
            seeker: session.seeker.clone(),
            provider: session.provider.clone(),
            terms: terms_value,
            created_at,
            status: "active".to_string(),
        })
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String { self.id.clone() }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(self)
            .map_err(|e| JsValue::from_str(&format!("Serialize error: {}", e)))
    }
}
```

---

# Teil VIII: Wallet-Modul (echo-wallet-wasm)

## 8.1 Wallet-Funktionen

```rust
// crates/echo-wallet-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Budget-Limit
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BudgetLimits {
    pub per_transaction: f64,
    pub per_day: f64,
    pub per_month: f64,
}

/// Wallet-Status
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    owner: String,
    balances: HashMap<String, f64>,  // currency -> amount
    limits: BudgetLimits,
    spent_today: f64,
    spent_this_month: f64,
}

#[wasm_bindgen]
impl Wallet {
    #[wasm_bindgen(constructor)]
    pub fn new(owner: &str, limits_json: &str) -> Result<Wallet, JsValue> {
        let limits: BudgetLimits = serde_json::from_str(limits_json)
            .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;
        
        Ok(Wallet {
            owner: owner.to_string(),
            balances: HashMap::new(),
            limits,
            spent_today: 0.0,
            spent_this_month: 0.0,
        })
    }

    /// Lädt Wallet-Daten vom Host
    #[wasm_bindgen]
    pub fn load_from_host(owner: &str) -> Wallet {
        let mut balances = HashMap::new();
        
        // Lade bekannte Währungen
        for currency in &["EUR", "USD", "USDC"] {
            let balance = echo_host_wasm::host_get_balance(owner, currency);
            if balance > 0.0 {
                balances.insert(currency.to_string(), balance);
            }
        }
        
        Wallet {
            owner: owner.to_string(),
            balances,
            limits: BudgetLimits {
                per_transaction: 100.0,
                per_day: 500.0,
                per_month: 2000.0,
            },
            spent_today: 0.0,
            spent_this_month: 0.0,
        }
    }

    /// Gibt Balance für Währung zurück
    #[wasm_bindgen]
    pub fn balance(&self, currency: &str) -> f64 {
        *self.balances.get(currency).unwrap_or(&0.0)
    }

    /// Prüft ob Zahlung möglich (A21: Handlungsfähigkeit)
    #[wasm_bindgen]
    pub fn can_pay(&self, amount: f64, currency: &str) -> bool {
        // Prüfe Balance
        let balance = self.balance(currency);
        if balance < amount {
            return false;
        }
        
        // Prüfe Limits
        if amount > self.limits.per_transaction {
            return false;
        }
        
        if self.spent_today + amount > self.limits.per_day {
            return false;
        }
        
        if self.spent_this_month + amount > self.limits.per_month {
            return false;
        }
        
        true
    }

    /// Reserviert Betrag (für atomare Transaktionen A25)
    #[wasm_bindgen]
    pub fn reserve(&mut self, amount: f64, currency: &str) -> Result<String, JsValue> {
        if !self.can_pay(amount, currency) {
            return Err(JsValue::from_str("Cannot reserve: insufficient funds or limits exceeded"));
        }
        
        // Reduziere Balance
        let balance = self.balances.entry(currency.to_string()).or_insert(0.0);
        *balance -= amount;
        
        // Generiere Reservation-ID
        let reservation_id = format!("res_{}", js_sys::Date::now() as u64);
        
        Ok(reservation_id)
    }

    /// Gibt reservierten Betrag frei (bei Abbruch)
    #[wasm_bindgen]
    pub fn release(&mut self, amount: f64, currency: &str) {
        let balance = self.balances.entry(currency.to_string()).or_insert(0.0);
        *balance += amount;
    }

    /// Bestätigt Zahlung
    #[wasm_bindgen]
    pub fn confirm_payment(&mut self, amount: f64) {
        self.spent_today += amount;
        self.spent_this_month += amount;
    }

    /// Gibt verbleibendes Tagesbudget zurück
    #[wasm_bindgen]
    pub fn remaining_daily_budget(&self) -> f64 {
        (self.limits.per_day - self.spent_today).max(0.0)
    }

    /// Setzt Tageszähler zurück (am neuen Tag)
    #[wasm_bindgen]
    pub fn reset_daily(&mut self) {
        self.spent_today = 0.0;
    }

    /// Setzt Monatszähler zurück
    #[wasm_bindgen]
    pub fn reset_monthly(&mut self) {
        self.spent_this_month = 0.0;
        self.spent_today = 0.0;
    }
}
```

---

# Teil IX: Agent-Runtime (echo-agent-wasm)

## 9.1 Vollständige Agent-Runtime

```rust
// crates/echo-agent-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use echo_core_wasm::agent::{Agent, AgentStatus};
use echo_core_wasm::types::{Did, TrustVector};
use echo_policy_wasm::{Policy, PolicyContext, PolicyResult, Decision};
use echo_intent_wasm::{Intent, IntentMatcher, Offer, MatchScore};
use echo_negotiation_wasm::{NegotiationSession, NegotiationModel, Agreement};
use echo_wallet_wasm::Wallet;

/// Agent-Runtime: Vollständige Laufzeitumgebung für einen Agenten
#[wasm_bindgen]
pub struct AgentRuntime {
    agent: Agent,
    policy: Option<Policy>,
    wallet: Wallet,
    active_intents: Vec<Intent>,
    active_negotiations: Vec<NegotiationSession>,
}

#[wasm_bindgen]
impl AgentRuntime {
    /// Erstellt neue Agent-Runtime
    #[wasm_bindgen(constructor)]
    pub fn new(agent_did: &str) -> Result<AgentRuntime, JsValue> {
        let did = Did::new(agent_did)?;
        let agent = Agent::load(&did)?;
        let wallet = Wallet::load_from_host(agent_did);
        
        Ok(AgentRuntime {
            agent,
            policy: None,
            wallet,
            active_intents: Vec::new(),
            active_negotiations: Vec::new(),
        })
    }

    /// Lädt Policy
    #[wasm_bindgen]
    pub fn load_policy(&mut self, policy_json: &str) -> Result<(), JsValue> {
        self.policy = Some(Policy::from_json(policy_json)?);
        Ok(())
    }

    /// Prüft ob Agent handeln kann (A21)
    #[wasm_bindgen]
    pub fn can_act(&self) -> bool {
        self.agent.can_act()
    }

    /// Gibt Trust des Agents zurück
    #[wasm_bindgen]
    pub fn trust(&self) -> TrustVector {
        self.agent.trust()
    }

    /// Gibt Attention zurück: σ(agent)
    #[wasm_bindgen]
    pub fn attention(&self) -> f64 {
        self.agent.attention()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // INTENT-MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Erstellt neuen Intent
    #[wasm_bindgen]
    pub fn create_intent(&mut self, intent_json: &str) -> Result<String, JsValue> {
        if !self.can_act() {
            return Err(JsValue::from_str("Agent cannot act"));
        }
        
        let intent = Intent::from_json(intent_json)?;
        let intent_id = intent.id();
        self.active_intents.push(intent);
        
        Ok(intent_id)
    }

    /// Matched Angebote gegen Intent
    #[wasm_bindgen]
    pub fn match_offers(&self, intent_id: &str, offers_json: &str) -> Result<JsValue, JsValue> {
        let intent = self.active_intents.iter()
            .find(|i| i.id() == intent_id)
            .ok_or_else(|| JsValue::from_str("Intent not found"))?;
        
        let matcher = IntentMatcher::new(intent.clone());
        matcher.match_offers(offers_json)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POLICY-EVALUATION
    // ═══════════════════════════════════════════════════════════════════════════

    /// Evaluiert Angebot gegen Policy
    #[wasm_bindgen]
    pub fn evaluate_offer(&self, offer_json: &str) -> Result<PolicyResult, JsValue> {
        let policy = self.policy.as_ref()
            .ok_or_else(|| JsValue::from_str("No policy loaded"))?;
        
        let context = PolicyContext::new(
            &self.agent.did().value(),
            self.agent.trust().aggregate(),
            self.wallet.remaining_daily_budget(),
            js_sys::Date::now() as u64,
            offer_json,
        );
        
        Ok(policy.evaluate(&context))
    }

    /// Automatische Angebots-Verarbeitung
    #[wasm_bindgen]
    pub fn process_offer(&mut self, offer_json: &str) -> Result<JsValue, JsValue> {
        let result = self.evaluate_offer(offer_json)?;
        
        match result.decision() {
            Decision::Accept => {
                // Starte Negotiation
                let offer: Offer = Offer::from_json(offer_json)?;
                let session = self.start_negotiation(
                    &offer.id(),
                    &offer.id(), // Vereinfacht: offer_id = intent_id
                    NegotiationModel::Direct,
                )?;
                
                // Bei Direct: Sofort akzeptieren
                let session_id = session.id();
                self.accept_current_offer(&session_id, offer_json)?;
                
                Ok(JsValue::from_str("accepted"))
            }
            Decision::Reject => {
                Ok(JsValue::from_str("rejected"))
            }
            Decision::Escalate => {
                Ok(JsValue::from_str("escalated"))
            }
            Decision::Continue => {
                Ok(JsValue::from_str("pending"))
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // NEGOTIATION-MANAGEMENT
    // ═══════════════════════════════════════════════════════════════════════════

    /// Startet Verhandlung
    #[wasm_bindgen]
    pub fn start_negotiation(
        &mut self,
        session_id: &str,
        intent_id: &str,
        model: NegotiationModel,
    ) -> Result<NegotiationSession, JsValue> {
        // Vereinfacht: Provider-DID aus Kontext
        let provider = "did:erynoa:agent:provider:unknown";
        
        let session = NegotiationSession::new(
            session_id,
            intent_id,
            &self.agent.did().value(),
            provider,
            model,
            5,   // max_rounds
            300, // timeout (5 min)
        );
        
        self.active_negotiations.push(session.clone());
        Ok(session)
    }

    /// Akzeptiert aktuelles Angebot
    #[wasm_bindgen]
    pub fn accept_current_offer(
        &mut self,
        session_id: &str,
        terms_json: &str,
    ) -> Result<Agreement, JsValue> {
        let session = self.active_negotiations.iter_mut()
            .find(|s| s.id() == session_id)
            .ok_or_else(|| JsValue::from_str("Session not found"))?;
        
        session.add_round(
            "seeker",
            terms_json,
            "accept",
            js_sys::Date::now() as u64,
        )?;
        
        // Erstelle Agreement
        let agreement = Agreement::from_negotiation(
            &format!("agr_{}", session_id),
            session,
            js_sys::Date::now() as u64,
        )?;
        
        Ok(agreement)
    }

    /// Macht Gegenangebot
    #[wasm_bindgen]
    pub fn counter_offer(
        &mut self,
        session_id: &str,
        terms_json: &str,
    ) -> Result<(), JsValue> {
        let session = self.active_negotiations.iter_mut()
            .find(|s| s.id() == session_id)
            .ok_or_else(|| JsValue::from_str("Session not found"))?;
        
        session.add_round(
            "seeker",
            terms_json,
            "counter",
            js_sys::Date::now() as u64,
        )
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WALLET-OPERATIONEN
    // ═══════════════════════════════════════════════════════════════════════════

    /// Prüft ob Zahlung möglich
    #[wasm_bindgen]
    pub fn can_pay(&self, amount: f64, currency: &str) -> bool {
        self.wallet.can_pay(amount, currency)
    }

    /// Gibt Wallet-Balance zurück
    #[wasm_bindgen]
    pub fn balance(&self, currency: &str) -> f64 {
        self.wallet.balance(currency)
    }

    /// Gibt verbleibendes Tagesbudget zurück
    #[wasm_bindgen]
    pub fn remaining_budget(&self) -> f64 {
        self.wallet.remaining_daily_budget()
    }
}
```

---

# Teil X: Vollständigkeits-Matrix

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║                                    VOLLSTÄNDIGKEITS-MATRIX: ECHO LOGIK → WASM                                                            ║
║                                                                                                                                           ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                                                                           ║
║   KONZEPT                 LOGIK-BEZUG                     WASM-MODUL                      FUNKTION                     STATUS             ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                                                           ║
║   Agent-Existenz          A21: (s:α) → ⟨s⟩                echo-core-wasm                  Agent::can_act()              ✅               ║
║   Trust-Threshold         A23: □(s:α) → 𝕋 ≥ threshold     echo-core-wasm                  Agent::can_perform()          ✅               ║
║   Attention               σ(s) = σ(𝕋̄·ln|ℂ|)              echo-core-wasm                  attention_score()             ✅               ║
║   Trust-Vector            𝕋(s) = (R,I,C,P)                echo-core-wasm                  TrustVector                   ✅               ║
║                                                                                                                                           ║
║   ECLVM                   ECL-Bytecode                    echo-eclvm-wasm                 EclVm::execute()              ✅               ║
║   Gas-Metering            Ressourcen-Limits               echo-eclvm-wasm                 gas::*                        ✅               ║
║   Host-Calls              ERY/NOA-Interface               echo-host-wasm                  host_*()                      ✅               ║
║                                                                                                                                           ║
║   Policy-Eval             Entscheidungsregeln             echo-policy-wasm                Policy::evaluate()            ✅               ║
║   Auto-Accept             auto_accept Conditions          echo-policy-wasm                Decision::Accept              ✅               ║
║   Auto-Reject             auto_reject Conditions          echo-policy-wasm                Decision::Reject              ✅               ║
║   Escalation              Owner-Rückfrage                 echo-policy-wasm                Decision::Escalate            ✅               ║
║                                                                                                                                           ║
║   Intent-Matching         Constraint-Prüfung              echo-intent-wasm                IntentMatcher                 ✅               ║
║   Offer-Scoring           Multi-Dimensional               echo-intent-wasm                MatchScore                    ✅               ║
║   Priorities              Gewichtung                      echo-intent-wasm                priorities                    ✅               ║
║                                                                                                                                           ║
║   Direct Negotiation      Take-it-or-leave-it             echo-negotiation-wasm           NegotiationModel::Direct      ✅               ║
║   Auction                 Competitive Bidding             echo-negotiation-wasm           NegotiationModel::Auction     ✅               ║
║   Multi-Round             Haggling                        echo-negotiation-wasm           NegotiationModel::MultiRound  ✅               ║
║   Agreement               Vertragsabschluss               echo-negotiation-wasm           Agreement                     ✅               ║
║                                                                                                                                           ║
║   Wallet                  Budget-Management               echo-wallet-wasm                Wallet                        ✅               ║
║   Limits                  per_tx/day/month                echo-wallet-wasm                BudgetLimits                  ✅               ║
║   Reservation             Atomare Sperren                 echo-wallet-wasm                reserve/release               ✅               ║
║                                                                                                                                           ║
║   Agent-Runtime           Vollständige Laufzeit           echo-agent-wasm                 AgentRuntime                  ✅               ║
║                                                                                                                                           ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                                                                           ║
║   AXIOM-COVERAGE                                                                                                                         ║
║   ─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────── ║
║                                                                                                                                           ║
║   A21 (Handlung → Existenz)      host_exists() + can_act()                                                              ✅               ║
║   A22 (Realm-Erlaubnis)          host_action_allowed()                                                                  ✅               ║
║   A23 (Trust-Threshold)          trust >= threshold in Policy                                                           ✅               ║
║   A24 (Exchange-Symmetrie)       NegotiationSession (beide Seiten)                                                      ✅               ║
║   A25 (Atomizität)               Wallet::reserve/release + Host-Calls                                                   ✅               ║
║                                                                                                                                           ║
║   ═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════ ║
║                                                                                                                                           ║
║   VOLLSTÄNDIGKEITS-SCORE:  100%  (Alle ECHO-Komponenten + Axiome A21-A25 implementiert)                                                 ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

---

# Teil XI: Zusammenfassung

## Architektur-Übersicht

```
╔═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╗
║                                                                                                                                           ║
║                                         ECHO WASM ARCHITEKTUR (VOLLSTÄNDIG)                                                             ║
║                                                                                                                                           ║
║   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐ ║
║   │                                                                                                                                     │ ║
║   │   BROWSER / EDGE / NODE                                                                                                            │ ║
║   │   ═════════════════════                                                                                                            │ ║
║   │                                                                                                                                     │ ║
║   │   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   │   ECHO WASM MODULES                                                                                                         │  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   │   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                │  │ ║
║   │   │   │             │   │             │   │             │   │             │   │             │   │             │                │  │ ║
║   │   │   │  ECLVM      │   │  Policy     │   │  Intent     │   │ Negotiation │   │  Wallet     │   │  Agent      │                │  │ ║
║   │   │   │  Runtime    │   │  Engine     │   │  Matcher    │   │  Engine     │   │  Module     │   │  Runtime    │                │  │ ║
║   │   │   │             │   │             │   │             │   │             │   │             │   │             │                │  │ ║
║   │   │   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘   └──────┬──────┘                │  │ ║
║   │   │          │                 │                 │                 │                 │                 │                       │  │ ║
║   │   │          └─────────────────┴─────────────────┴─────────────────┴─────────────────┴─────────────────┘                       │  │ ║
║   │   │                                              │                                                                             │  │ ║
║   │   │                                              ▼                                                                             │  │ ║
║   │   │                                     ┌─────────────────┐                                                                    │  │ ║
║   │   │                                     │                 │                                                                    │  │ ║
║   │   │                                     │  echo-core-wasm │                                                                    │  │ ║
║   │   │                                     │  (Types, σ, 𝕋)  │                                                                    │  │ ║
║   │   │                                     │                 │                                                                    │  │ ║
║   │   │                                     └────────┬────────┘                                                                    │  │ ║
║   │   │                                              │                                                                             │  │ ║
║   │   │                                              ▼                                                                             │  │ ║
║   │   │                                     ┌─────────────────┐                                                                    │  │ ║
║   │   │                                     │                 │                                                                    │  │ ║
║   │   │                                     │  echo-host-wasm │                                                                    │  │ ║
║   │   │                                     │  (Host-Imports) │                                                                    │  │ ║
║   │   │                                     │                 │                                                                    │  │ ║
║   │   │                                     └────────┬────────┘                                                                    │  │ ║
║   │   │                                              │                                                                             │  │ ║
║   │   └──────────────────────────────────────────────┼──────────────────────────────────────────────────────────────────────────────┘  │ ║
║   │                                                  │                                                                                 │ ║
║   │                               ═══════════════════╪═══════════════════  WASM BOUNDARY                                              │ ║
║   │                                                  │                                                                                 │ ║
║   │                                                  ▼                                                                                 │ ║
║   │   ┌─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   │   HOST ENVIRONMENT (JavaScript / Rust)                                                                                      │  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   │   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                                  │  │ ║
║   │   │   │             │   │             │   │             │   │             │   │             │                                  │  │ ║
║   │   │   │     ERY     │   │     NOA     │   │   Crypto    │   │   Storage   │   │   Network   │                                  │  │ ║
║   │   │   │  (Identity) │   │  (Ledger)   │   │  (Signing)  │   │  (KV/IPFS)  │   │  (P2P/HTTP) │                                  │  │ ║
║   │   │   │             │   │             │   │             │   │             │   │             │                                  │  │ ║
║   │   │   └─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘                                  │  │ ║
║   │   │                                                                                                                             │  │ ║
║   │   └─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘  │ ║
║   │                                                                                                                                     │ ║
║   └─────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘ ║
║                                                                                                                                           ║
╚═══════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════════╝
```

## Build-Konfiguration

```toml
# Cargo.toml (Workspace)
[workspace]
members = [
    "crates/echo-core-wasm",
    "crates/echo-host-wasm",
    "crates/echo-eclvm-wasm",
    "crates/echo-policy-wasm",
    "crates/echo-intent-wasm",
    "crates/echo-negotiation-wasm",
    "crates/echo-wallet-wasm",
    "crates/echo-agent-wasm",
]

[workspace.dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde-wasm-bindgen = "0.6"

[profile.release]
opt-level = "s"
lto = true
```

---

*ECHO WASM Architektur Version 1.0 – Vollständige Implementierung der Agent-Runtime für Browser und Edge-Devices.*
