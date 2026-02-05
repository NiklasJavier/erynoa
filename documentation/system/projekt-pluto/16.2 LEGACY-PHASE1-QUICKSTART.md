# 🚀 Quick-Start: Phase 1 Umsetzung

> **Dieser Guide** zeigt die ersten konkreten Schritte zur Umsetzung des MEGA-REFACTORING-PLAN.md

---

## Tag 1: Verzeichnisstruktur erstellen

```bash
cd /Users/niklas/Development/30_Development_Code/31_git_ownbusiness/erynoa/erynoa-main/backend

# Nervensystem-Struktur
mkdir -p src/nervous_system/event_sourcing
mkdir -p src/nervous_system/merkle
mkdir -p src/nervous_system/components
mkdir -p src/nervous_system/coordination
mkdir -p src/nervous_system/graph
mkdir -p src/nervous_system/infrastructure

# Synapses (Observer-Hub)
mkdir -p src/synapses/adapters

# Realm-Layer (aus peer/)
mkdir -p src/realm/sharding
mkdir -p src/realm/quota
mkdir -p src/realm/gateway
mkdir -p src/realm/saga

# Storage-Layer (aus local/)
mkdir -p src/storage/kv
mkdir -p src/storage/event_store
mkdir -p src/storage/archive
mkdir -p src/storage/identity_store
mkdir -p src/storage/blueprint
```

---

## Tag 2-3: Unified Error Types

### Schritt 1: Neue Error-Datei erstellen

**Datei:** `src/domain/unified/error.rs`

```rust
//! # Unified Error Types
//!
//! Zentrale Fehlerdefinitionen für das gesamte Erynoa-System.

use thiserror::Error;

/// Root-Error für alle Erynoa-Operationen
#[derive(Debug, Error)]
pub enum ErynoaError {
    #[error("Identity error: {0}")]
    Identity(#[from] IdentityError),

    #[error("Execution error: {0}")]
    Execution(#[from] ExecutionError),

    #[error("Realm error: {0}")]
    Realm(#[from] RealmError),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("P2P error: {0}")]
    P2P(#[from] P2PError),

    #[error("ECLVM error: {0}")]
    ECLVM(#[from] ECLVMError),

    #[error("State error: {0}")]
    State(#[from] StateError),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("Not bootstrapped")]
    NotBootstrapped,

    #[error("Key store not initialized")]
    KeyStoreNotInitialized,

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Unknown identity: {0}")]
    UnknownIdentity(String),

    #[error("Lock error")]
    LockError,
}

#[derive(Debug, Error)]
pub enum ExecutionError {
    #[error("Gas exhausted: required {required}, available {available}")]
    GasExhausted { required: u64, available: u64 },

    #[error("Mana exhausted: required {required}, available {available}")]
    ManaExhausted { required: u64, available: u64 },

    #[error("Execution timeout after {0}ms")]
    Timeout(u64),
}

#[derive(Debug, Error)]
pub enum RealmError {
    #[error("Realm not found: {0}")]
    NotFound(String),

    #[error("Realm quarantined: {0}")]
    Quarantined(String),

    #[error("Quota exceeded for {resource}")]
    QuotaExceeded { resource: String, requested: u64, limit: u64 },
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

#[derive(Debug, Error)]
pub enum P2PError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),
}

#[derive(Debug, Error)]
pub enum ECLVMError {
    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("Runtime error: {0}")]
    Runtime(String),
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Component not found: {0:?}")]
    ComponentNotFound(String),

    #[error("Invalid state transition")]
    InvalidTransition,
}

/// Result-Alias für Erynoa-Operationen
pub type ErynoaResult<T> = Result<T, ErynoaError>;
```

### Schritt 2: In mod.rs exportieren

**Datei:** `src/domain/unified/mod.rs` – Am Ende hinzufügen:

```rust
pub mod error;
pub use error::*;
```

---

## Tag 4-5: StateLayer-Trait definieren

### Schritt 1: Traits-Datei erstellen

**Datei:** `src/nervous_system/traits.rs`

```rust
//! # State-Layer Traits
//!
//! Unified Traits für alle State-Komponenten.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::sync::Arc;

use crate::core::state::{StateComponent, WrappedStateEvent};

/// Trait für alle State-Komponenten im Nervensystem
pub trait StateLayer: Send + Sync + 'static {
    /// Snapshot-Typ für Serialisierung
    type Snapshot: Clone + Serialize + DeserializeOwned + Send + Sync;

    /// Erstelle Snapshot des aktuellen States
    fn snapshot(&self) -> Self::Snapshot;

    /// Health-Score (0.0-1.0)
    fn health_score(&self) -> f64;

    /// Wende Event an (für Replay)
    fn apply_event(&self, event: &WrappedStateEvent);

    /// Zugehörige Komponente
    fn component(&self) -> StateComponent;
}

/// Trait für resettbare Komponenten (Tests)
pub trait Resettable {
    fn reset(&self);
}

/// Trait für metrische Komponenten
pub trait Metered {
    fn record(&self, metric: &str, value: f64);
    fn counter(&self, metric: &str) -> u64;
}

/// Observer-Priorität
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObserverPriority {
    Critical = 0,  // Anomaly, CircuitBreaker
    High = 1,      // Trust, Consensus
    Normal = 2,    // Default
    Low = 3,       // Metrics, Logging
}

/// Universeller State-Observer
pub trait StateObserver: Send + Sync + 'static {
    /// Reagiere auf Event
    fn on_event(&self, event: &WrappedStateEvent);

    /// Zuständige Komponente
    fn target_component(&self) -> StateComponent;

    /// Priorität des Observers
    fn priority(&self) -> ObserverPriority {
        ObserverPriority::Normal
    }

    /// Ist Observer aktiv?
    fn is_active(&self) -> bool {
        true
    }
}

/// Observer-Handle für Deregistrierung
pub struct ObserverHandle {
    pub id: u64,
    pub component: StateComponent,
}
```

### Schritt 2: mod.rs für nervous_system

**Datei:** `src/nervous_system/mod.rs`

```rust
//! # Nervous System
//!
//! Zentrales State-Management für Erynoa.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      UNIFIED STATE                              │
//! │                  (Zentrales Nervensystem)                       │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Event-Sourcing │ Merkle-Tracking │ Circuit Breaker │ Health   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

pub mod traits;

// Re-exports
pub use traits::*;

// Diese Module werden schrittweise hinzugefügt:
// pub mod event_sourcing;
// pub mod merkle;
// pub mod components;
// pub mod coordination;
// pub mod graph;
// pub mod infrastructure;
```

---

## Tag 6-7: Erste Extraktion (EventBus)

### Schritt 1: EventBus extrahieren

**Aus:** `src/core/state.rs` (Zeilen 39-400)
**Nach:** `src/nervous_system/infrastructure/event_bus.rs`

```rust
//! # Event Bus
//!
//! P2P ↔ Core Kommunikation über bounded Queues.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};

use crate::domain::unified::primitives::UniversalId;
use crate::domain::unified::system::EventPriority;

/// Network-Event für P2P-Kommunikation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub id: u64,
    pub event_type: String,
    pub payload: Vec<u8>,
    pub priority: EventPriority,
    pub peer_id: Option<String>,
    pub realm_id: Option<String>,
    pub timestamp_ms: u64,
    pub peer_universal_id: Option<UniversalId>,
    pub signature: Option<[u8; 64]>,
    pub signature_verified: Option<bool>,
}

impl NetworkEvent {
    pub fn new(event_type: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0),
            event_type: event_type.into(),
            payload,
            priority: EventPriority::Normal,
            peer_id: None,
            realm_id: None,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            peer_universal_id: None,
            signature: None,
            signature_verified: None,
        }
    }

    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_peer(mut self, peer_id: impl Into<String>) -> Self {
        self.peer_id = Some(peer_id.into());
        self
    }

    pub fn with_realm(mut self, realm_id: impl Into<String>) -> Self {
        self.realm_id = Some(realm_id.into());
        self
    }
}

/// Kapazitäten
const INGRESS_CAPACITY: usize = 10_000;
const EGRESS_CAPACITY: usize = 10_000;
const PRIORITY_CAPACITY: usize = 1_000;

/// Event-Bus für P2P ↔ Core Kommunikation
#[derive(Debug)]
pub struct EventBus {
    // Ingress: P2P → Core
    pub ingress_tx: mpsc::Sender<NetworkEvent>,
    ingress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,

    // Egress: Core → P2P
    pub egress_tx: mpsc::Sender<NetworkEvent>,
    egress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,

    // High-Priority
    pub priority_ingress_tx: mpsc::Sender<NetworkEvent>,
    priority_ingress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,

    // Metrics
    pub ingress_count: AtomicU64,
    pub egress_count: AtomicU64,
    pub dropped_count: AtomicU64,
    pub processed_count: AtomicU64,
    pub priority_processed: AtomicU64,
}

impl EventBus {
    pub fn new() -> Self {
        let (ingress_tx, ingress_rx) = mpsc::channel(INGRESS_CAPACITY);
        let (egress_tx, egress_rx) = mpsc::channel(EGRESS_CAPACITY);
        let (priority_tx, priority_rx) = mpsc::channel(PRIORITY_CAPACITY);

        Self {
            ingress_tx,
            ingress_rx: RwLock::new(Some(ingress_rx)),
            egress_tx,
            egress_rx: RwLock::new(Some(egress_rx)),
            priority_ingress_tx: priority_tx,
            priority_ingress_rx: RwLock::new(Some(priority_rx)),
            ingress_count: AtomicU64::new(0),
            egress_count: AtomicU64::new(0),
            dropped_count: AtomicU64::new(0),
            processed_count: AtomicU64::new(0),
            priority_processed: AtomicU64::new(0),
        }
    }

    pub fn try_send_ingress(&self, event: NetworkEvent) -> bool {
        let is_priority = matches!(event.priority, EventPriority::Critical | EventPriority::High);

        if is_priority {
            if self.priority_ingress_tx.try_send(event).is_ok() {
                self.ingress_count.fetch_add(1, Ordering::Relaxed);
                return true;
            }
        }

        if self.ingress_tx.try_send(event).is_ok() {
            self.ingress_count.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            self.dropped_count.fetch_add(1, Ordering::Relaxed);
            false
        }
    }

    pub fn take_ingress_receiver(&self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.ingress_rx.write().ok()?.take()
    }

    pub fn take_priority_receiver(&self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.priority_ingress_rx.write().ok()?.take()
    }

    pub fn take_egress_receiver(&self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.egress_rx.write().ok()?.take()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot für Metriken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusSnapshot {
    pub ingress_count: u64,
    pub egress_count: u64,
    pub dropped_count: u64,
    pub processed_count: u64,
    pub priority_processed: u64,
}

impl EventBus {
    pub fn snapshot(&self) -> EventBusSnapshot {
        EventBusSnapshot {
            ingress_count: self.ingress_count.load(Ordering::Relaxed),
            egress_count: self.egress_count.load(Ordering::Relaxed),
            dropped_count: self.dropped_count.load(Ordering::Relaxed),
            processed_count: self.processed_count.load(Ordering::Relaxed),
            priority_processed: self.priority_processed.load(Ordering::Relaxed),
        }
    }
}
```

### Schritt 2: infrastructure/mod.rs

**Datei:** `src/nervous_system/infrastructure/mod.rs`

```rust
//! # Infrastructure
//!
//! Infrastruktur-Komponenten des Nervensystems.

pub mod event_bus;

pub use event_bus::*;
```

### Schritt 3: Re-Export in state.rs (temporär)

**In `src/core/state.rs` am Anfang:**

```rust
// Temporärer Re-Export für Rückwärtskompatibilität
pub use crate::nervous_system::infrastructure::event_bus::*;
```

---

## Checkliste Phase 1

- [ ] Verzeichnisstruktur erstellt
- [ ] `domain/unified/error.rs` erstellt
- [ ] `nervous_system/traits.rs` erstellt
- [ ] `nervous_system/mod.rs` erstellt
- [ ] `infrastructure/event_bus.rs` extrahiert
- [ ] Re-Exports für Rückwärtskompatibilität
- [ ] `cargo build` erfolgreich
- [ ] `cargo test` erfolgreich
- [ ] `cargo clippy` bestanden

---

## Nächste Schritte (Phase 2)

Nach Phase 1:
1. **CircuitBreaker** nach `infrastructure/circuit_breaker.rs` extrahieren
2. **StateBroadcaster** nach `infrastructure/broadcaster.rs` extrahieren
3. **StateEvent** enum nach `event_sourcing/state_event.rs` extrahieren
4. **MerkleStateTracker** nach `merkle/tracker.rs` extrahieren

---

> **Tipp:** Nach jeder Extraktion `cargo build && cargo test` ausführen!
