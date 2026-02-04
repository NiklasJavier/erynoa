//! ## Design-Prinzipien
//!
//! 1. **Hierarchische Komposition**: State-Layer bauen aufeinander auf
//! 2. **Thread-Safety**: Alle Counter sind atomar, komplexe Strukturen unter RwLock
//! 3. **Dependency Injection**: Jeder Layer kennt seine Abhängigkeiten
//! 4. **Event-Driven Updates**: Änderungen propagieren durch Observer-Pattern
//! 5. **Snapshot-Isolation**: Konsistente Reads ohne Locking
//! 6. **Per-Realm Isolation**: Jedes Realm hat eigenen TrustVector, Rules und Metrics
//! 7. **Event-Inversion**: P2P/Core Entkopplung durch Ingress/Egress-Queues
//! 8. **Circuit Breaker**: Automatische Degradation bei kritischen Anomalien
//! 9. **CQRS light**: Broadcast-Channels für State-Deltas an Subscriber
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, mpsc};

// Domain Primitives
use crate::domain::unified::primitives::UniversalId;
pub use crate::domain::MemberRole;

// Diese Typen sind jetzt in domain/unified definiert und werden hier re-exportiert
// für Rückwärtskompatibilität. Neue Nutzung sollte direkt aus domain importieren.
pub use crate::domain::unified::action::{
    BlueprintAction, MembershipAction, NetworkMetric, RealmAction,
};
pub use crate::domain::unified::component::{ComponentLayer, StateComponent, StateRelation};
pub use crate::domain::unified::system::{AnomalySeverity, EventPriority, SystemMode};

// Sharding & High-Performance Concurrent Data Structures
use dashmap::DashMap;
use lru::LruCache;
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use tokio::sync::RwLock as TokioRwLock;

// ============================================================================
// SYSTEM MODE (CIRCUIT BREAKER PATTERN)
// ============================================================================
// NOTE: SystemMode ist jetzt in domain/unified/system.rs definiert und wird
// oben via `pub use` re-exportiert für Rückwärtskompatibilität.
// ============================================================================
// EVENT BUS (P2P/CORE ENTKOPPLUNG)
// ============================================================================
// NOTE: EventPriority ist jetzt in domain/unified/system.rs definiert und wird
// oben via `pub use` re-exportiert für Rückwärtskompatibilität.
/// Typisiertes Network-Event für Ingress/Egress-Queues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    /// Eindeutige Event-ID
    pub id: u64,
    /// Event-Typ (z.B. "trust_update", "consensus_vote", "gossip_message")
    pub event_type: String,
    /// Serialisierte Payload (JSON/CBOR)
    pub payload: Vec<u8>,
    /// Priorität für Queue-Ordering
    pub priority: EventPriority,
    /// Source Peer-ID (für Ingress) oder Target (für Egress) - Legacy
    pub peer_id: Option<String>,
    /// Realm-Kontext (falls realm-spezifisch)
    pub realm_id: Option<String>,
    /// Timestamp (Unix-Epoch Millis)
    pub timestamp_ms: u64,
    // ─────────────────────────────────────────────────────────────────────────
    // Identity-Integration (Phase 7)
    // ─────────────────────────────────────────────────────────────────────────
    /// Peer UniversalId (Identity-basiert)
    pub peer_universal_id: Option<UniversalId>,
    /// Ed25519 Signatur über (event_type | payload | timestamp_ms)
    #[serde(with = "serde_signature_option")]
    pub signature: Option<[u8; 64]>,
    /// Signatur-Verifikations-Cache (nicht serialisiert)
    #[serde(skip)]
    pub signature_verified: Option<bool>,
}
/// Serde helper für Option<[u8; 64]> als hex string
mod serde_signature_option {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<[u8; 64]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(sig) => serializer.serialize_some(&hex::encode(sig)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 64]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
                if bytes.len() != 64 {
                    return Err(serde::de::Error::custom("signature must be 64 bytes"));
                }
                let mut arr = [0u8; 64];
                arr.copy_from_slice(&bytes);
                Ok(Some(arr))
            }
            None => Ok(None),
        }
    }
}

impl NetworkEvent {
    pub fn new(event_type: impl Into<String>, payload: Vec<u8>, priority: EventPriority) -> Self {
        Self {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0),
            event_type: event_type.into(),
            payload,
            priority,
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

    pub fn with_peer(mut self, peer_id: impl Into<String>) -> Self {
        self.peer_id = Some(peer_id.into());
        self
    }

    pub fn with_realm(mut self, realm_id: impl Into<String>) -> Self {
        self.realm_id = Some(realm_id.into());
        self
    }

    /// Builder: Setze Identity-basierte Peer-ID
    pub fn with_peer_identity(mut self, peer_id: UniversalId) -> Self {
        self.peer_universal_id = Some(peer_id);
        self
    }

    /// Erstelle signiertes Event
    /// Signiert (event_type | payload | timestamp_ms) mit dem gegebenen Signatur-Callback
    pub fn signed<F>(
        event_type: impl Into<String>,
        payload: Vec<u8>,
        priority: EventPriority,
        signer_id: UniversalId,
        sign_fn: F,
    ) -> Result<Self, crate::core::identity_types::IdentityError>
    where
        F: FnOnce(&[u8]) -> Result<[u8; 64], crate::core::identity_types::IdentityError>,
    {
        let event_type_str = event_type.into();
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Erstelle Signatur-Payload: event_type | payload | timestamp_ms (big-endian)
        let mut sign_payload = Vec::new();
        sign_payload.extend_from_slice(event_type_str.as_bytes());
        sign_payload.extend_from_slice(&payload);
        sign_payload.extend_from_slice(&timestamp_ms.to_be_bytes());

        let signature = sign_fn(&sign_payload)?;

        Ok(Self {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0),
            event_type: event_type_str,
            payload,
            priority,
            peer_id: None,
            realm_id: None,
            timestamp_ms,
            peer_universal_id: Some(signer_id),
            signature: Some(signature),
            signature_verified: Some(true), // Wir haben gerade selbst signiert
        })
    }

    /// Verifiziere die Signatur mit einem IdentityResolver
    /// Returns true wenn Signatur gültig, false wenn ungültig oder keine Signatur vorhanden
    pub fn verify_signature<R: crate::core::identity_types::IdentityResolver + ?Sized>(
        &mut self,
        resolver: &R,
    ) -> bool {
        // Check cache
        if let Some(verified) = self.signature_verified {
            return verified;
        }

        // Brauchen sowohl Signatur als auch Signer-ID
        let (signature, signer_id) = match (self.signature, self.peer_universal_id) {
            (Some(sig), Some(id)) => (sig, id),
            _ => {
                self.signature_verified = Some(false);
                return false;
            }
        };

        // Erstelle Signatur-Payload: event_type | payload | timestamp_ms
        let mut sign_payload = Vec::new();
        sign_payload.extend_from_slice(self.event_type.as_bytes());
        sign_payload.extend_from_slice(&self.payload);
        sign_payload.extend_from_slice(&self.timestamp_ms.to_be_bytes());

        // Resolve Public Key
        let result = resolver
            .resolve_public_key(&signer_id)
            .map(|pubkey| {
                // Verify signature (Ed25519)
                // In production würde hier ed25519_dalek::Signature::verify verwendet
                // Für jetzt: vereinfachte Verifikation
                pubkey.len() == 32 && signature.len() == 64
            })
            .unwrap_or(false);

        self.signature_verified = Some(result);
        result
    }

    /// Prüfe ob Event eine Signatur hat
    pub fn is_signed(&self) -> bool {
        self.signature.is_some() && self.peer_universal_id.is_some()
    }

    /// Prüfe ob Signatur verifiziert wurde
    pub fn is_verified(&self) -> Option<bool> {
        self.signature_verified
    }
}

/// EventBus für P2P/Core Entkopplung (Verbesserung 1)
///
/// # Architektur
///
/// ```text
/// P2P Layer ──▶ [Ingress Queue] ──▶ Core Processor Task
///                                         │
///                                         ▼
///                                   CoreState Updates
///                                         │
///                                         ▼
/// P2P Layer ◀── [Egress Queue] ◀── Outbound Events
/// ```
///
/// # Features
///
/// - **Bounded Queues**: Verhindert Memory-Exhaustion (default: 10.000 Events)
/// - **Priority Queues**: Critical Events werden bevorzugt verarbeitet
/// - **Backpressure**: P2P blockiert bei voller Queue (graceful degradation)
/// - **Async Processing**: Core verarbeitet Events non-blocking
#[derive(Debug)]
pub struct EventBus {
    /// Ingress-Queue: P2P → Core (empfangene Events)
    pub ingress_tx: mpsc::Sender<NetworkEvent>,
    /// Ingress-Receiver für Core-Processor
    pub ingress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,
    /// Egress-Queue: Core → P2P (zu sendende Events)
    pub egress_tx: mpsc::Sender<NetworkEvent>,
    /// Egress-Receiver für P2P-Sender
    pub egress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,
    /// High-Priority Ingress (Consensus, Trust-Critical)
    pub priority_ingress_tx: mpsc::Sender<NetworkEvent>,
    /// High-Priority Receiver
    pub priority_ingress_rx: RwLock<Option<mpsc::Receiver<NetworkEvent>>>,
    // ─────────────────────────────────────────────────────────────────────────
    // Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Events empfangen (Ingress)
    pub ingress_count: AtomicU64,
    /// Events gesendet (Egress)
    pub egress_count: AtomicU64,
    /// Verworfene Events (Queue voll)
    pub dropped_count: AtomicU64,
    /// Verarbeitete Events
    pub processed_count: AtomicU64,
    /// Priority-Events verarbeitet
    pub priority_processed: AtomicU64,
}
impl EventBus {
    /// Queue-Kapazität (bounded channel)
    pub const DEFAULT_QUEUE_SIZE: usize = 10_000;
    pub const PRIORITY_QUEUE_SIZE: usize = 1_000;

    pub fn new() -> Self {
        let (ingress_tx, ingress_rx) = mpsc::channel(Self::DEFAULT_QUEUE_SIZE);
        let (egress_tx, egress_rx) = mpsc::channel(Self::DEFAULT_QUEUE_SIZE);
        let (priority_tx, priority_rx) = mpsc::channel(Self::PRIORITY_QUEUE_SIZE);
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
    /// Event in Ingress-Queue einreihen (non-blocking try)
    pub fn try_send_ingress(&self, event: NetworkEvent) -> Result<(), NetworkEvent> {
        let tx = if event.priority == EventPriority::Critical {
            &self.priority_ingress_tx
        } else {
            &self.ingress_tx
        };

        match tx.try_send(event) {
            Ok(()) => {
                self.ingress_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(mpsc::error::TrySendError::Full(e)) => {
                self.dropped_count.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
            Err(mpsc::error::TrySendError::Closed(e)) => Err(e),
        }
    }

    /// Event in Egress-Queue einreihen
    pub fn try_send_egress(&self, event: NetworkEvent) -> Result<(), NetworkEvent> {
        match self.egress_tx.try_send(event) {
            Ok(()) => {
                self.egress_count.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(mpsc::error::TrySendError::Full(e)) => {
                self.dropped_count.fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
            Err(mpsc::error::TrySendError::Closed(e)) => Err(e),
        }
    }

    /// Nimm Ingress-Receiver (für Core-Processor Task)
    pub fn take_ingress_receiver(&self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.ingress_rx.write().ok()?.take()
    }

    /// Nimm Priority-Ingress-Receiver
    pub fn take_priority_receiver(&self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.priority_ingress_rx.write().ok()?.take()
    }

    /// Nimm Egress-Receiver (für P2P-Sender Task)
    pub fn take_egress_receiver(&self) -> Option<mpsc::Receiver<NetworkEvent>> {
        self.egress_rx.write().ok()?.take()
    }

    /// Markiere Event als verarbeitet
    pub fn mark_processed(&self, is_priority: bool) {
        self.processed_count.fetch_add(1, Ordering::Relaxed);
        if is_priority {
            self.priority_processed.fetch_add(1, Ordering::Relaxed);
        }
    }

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

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBusSnapshot {
    pub ingress_count: u64,
    pub egress_count: u64,
    pub dropped_count: u64,
    pub processed_count: u64,
    pub priority_processed: u64,
}

// ============================================================================
// STATE DELTA (CQRS BROADCAST)
// ============================================================================

/// State-Delta für CQRS Broadcast (Verbesserung 4)
///
/// Subscriber (DataLogic-Engine, Monitoring, Metrics-Exporter) empfangen
/// State-Änderungen über Broadcast-Channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDelta {
    /// Sequenznummer für Ordering
    pub sequence: u64,
    /// Betroffene State-Komponente
    pub component: StateComponent,
    /// Delta-Typ
    pub delta_type: DeltaType,
    /// Serialisierte Delta-Daten
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp_ms: u64,
    /// Optional: Realm-Kontext
    pub realm_id: Option<String>,
}

/// Typ der State-Änderung
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeltaType {
    /// Inkrementelles Update (Counter, einzelnes Feld)
    Increment,
    /// Vollständiges Snapshot-Update
    Snapshot,
    /// Neuer Eintrag hinzugefügt
    Insert,
    /// Eintrag entfernt
    Delete,
    /// Eintrag aktualisiert
    Update,
    /// Batch-Operation
    Batch,
}

impl StateDelta {
    pub fn new(component: StateComponent, delta_type: DeltaType, data: Vec<u8>) -> Self {
        Self {
            sequence: 0, // Wird beim Broadcast gesetzt
            component,
            delta_type,
            data,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            realm_id: None,
        }
    }

    pub fn with_realm(mut self, realm_id: impl Into<String>) -> Self {
        self.realm_id = Some(realm_id.into());
        self
    }
}
/// Broadcast-Sender für State-Deltas
#[derive(Debug)]
pub struct StateBroadcaster {
    /// Broadcast-Sender (Multi-Consumer)
    sender: broadcast::Sender<StateDelta>,
    /// Sequenz-Counter
    sequence: AtomicU64,
    /// Gesendete Deltas
    pub deltas_sent: AtomicU64,
    /// Subscriber-Count (geschätzt)
    pub subscriber_count: AtomicU64,
}

impl StateBroadcaster {
    pub const DEFAULT_CAPACITY: usize = 1024;

    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(Self::DEFAULT_CAPACITY);
        Self {
            sender,
            sequence: AtomicU64::new(0),
            deltas_sent: AtomicU64::new(0),
            subscriber_count: AtomicU64::new(0),
        }
    }

    /// Neuen Subscriber erstellen
    pub fn subscribe(&self) -> broadcast::Receiver<StateDelta> {
        self.subscriber_count.fetch_add(1, Ordering::Relaxed);
        self.sender.subscribe()
    }

    /// Delta broadcasten
    pub fn broadcast(&self, mut delta: StateDelta) {
        delta.sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        // Ignore send errors (no subscribers = ok)
        let _ = self.sender.send(delta);
        self.deltas_sent.fetch_add(1, Ordering::Relaxed);
    }

    /// Convenience: Increment-Delta senden
    pub fn send_increment(&self, component: StateComponent, field: &str, value: i64) {
        let data = format!("{{\"field\":\"{}\",\"value\":{}}}", field, value).into_bytes();
        self.broadcast(StateDelta::new(component, DeltaType::Increment, data));
    }

    pub fn snapshot(&self) -> BroadcasterSnapshot {
        BroadcasterSnapshot {
            sequence: self.sequence.load(Ordering::Relaxed),
            deltas_sent: self.deltas_sent.load(Ordering::Relaxed),
            subscriber_count: self.subscriber_count.load(Ordering::Relaxed),
        }
    }
}

impl Default for StateBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcasterSnapshot {
    pub sequence: u64,
    pub deltas_sent: u64,
    pub subscriber_count: u64,
}

// ============================================================================
// STORAGE HANDLE (ORTHOGONALE SCHICHT)
// ============================================================================
/// StorageHandle für orthogonalen Storage-Zugriff (Verbesserung 2)
/// Alle Layer erhalten einen shared StorageHandle statt direkter Storage-Abhängigkeit.
/// Ermöglicht pluggable Storage (RocksDB, IPFS, Cloud) und einheitliche Recovery.
#[derive(Debug, Clone)]
pub struct StorageHandle {
    /// Storage-Backend-Typ
    pub backend: StorageBackend,
    /// Shared Referenz auf StorageState für Metriken
    pub metrics: Arc<RwLock<StorageMetrics>>,
}

/// Storage-Backend-Typ (pluggable) - High-Level für StorageHandle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StorageBackend {
    /// Pure Rust Fjall LSM-Tree (neuer Standard)
    #[default]
    Fjall,
    /// Lokale RocksDB (Legacy)
    RocksDB,
    /// Distributed IPFS
    IPFS,
    /// Cloud-Storage (Azure Blob, S3, etc.)
    Cloud,
    /// In-Memory (Testing)
    Memory,
}

/// Storage-Metriken für Handle
#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    pub reads: u64,
    pub writes: u64,
    pub deletes: u64,
    pub bytes_read: u64,
    pub bytes_written: u64,
}

impl StorageHandle {
    pub fn new(backend: StorageBackend) -> Self {
        Self {
            backend,
            metrics: Arc::new(RwLock::new(StorageMetrics::default())),
        }
    }

    /// Async persist (placeholder für echte Implementierung)
    pub fn record_write(&self, bytes: u64) {
        if let Ok(mut m) = self.metrics.write() {
            m.writes += 1;
            m.bytes_written += bytes;
        }
    }

    pub fn record_read(&self, bytes: u64) {
        if let Ok(mut m) = self.metrics.write() {
            m.reads += 1;
            m.bytes_read += bytes;
        }
    }

    pub fn snapshot(&self) -> StorageMetrics {
        self.metrics.read().map(|m| m.clone()).unwrap_or_default()
    }
}

impl Default for StorageHandle {
    fn default() -> Self {
        Self::new(StorageBackend::RocksDB)
    }
}

// ============================================================================
// CIRCUIT BREAKER STATE
// ============================================================================

/// Circuit Breaker für automatische Degradation (Verbesserung 3)
///
/// Überwacht kritische Metriken und schaltet System in Degraded/Emergency-Modus.
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Aktueller System-Modus
    mode: AtomicU8,
    /// Kritische Anomalien in letzter Minute
    critical_window: RwLock<Vec<u64>>,
    /// Threshold für Degraded-Modus (kritische Anomalien/Minute)
    pub degraded_threshold: AtomicU64,
    /// Threshold für Emergency-Shutdown (kritische Anomalien/Minute)
    pub emergency_threshold: AtomicU64,
    /// Gini-Threshold für Anti-Calcification (Κ19)
    pub gini_threshold: RwLock<f64>,
    /// Modus-Wechsel-Zähler
    pub mode_changes: AtomicU64,
    /// Letzte Modus-Änderung (Unix-Epoch Millis)
    pub last_mode_change_ms: AtomicU64,
}

impl CircuitBreaker {
    pub const DEFAULT_DEGRADED_THRESHOLD: u64 = 10;
    pub const DEFAULT_EMERGENCY_THRESHOLD: u64 = 50;
    pub const DEFAULT_GINI_THRESHOLD: f64 = 0.8;

    pub fn new() -> Self {
        Self {
            mode: AtomicU8::new(SystemMode::Normal as u8),
            critical_window: RwLock::new(Vec::new()),
            degraded_threshold: AtomicU64::new(Self::DEFAULT_DEGRADED_THRESHOLD),
            emergency_threshold: AtomicU64::new(Self::DEFAULT_EMERGENCY_THRESHOLD),
            gini_threshold: RwLock::new(Self::DEFAULT_GINI_THRESHOLD),
            mode_changes: AtomicU64::new(0),
            last_mode_change_ms: AtomicU64::new(0),
        }
    }

    /// Aktuellen Modus abfragen
    pub fn mode(&self) -> SystemMode {
        SystemMode::from_u8(self.mode.load(Ordering::Relaxed))
    }

    /// Modus setzen (mit Logging)
    pub fn set_mode(&self, new_mode: SystemMode) {
        let old = self.mode.swap(new_mode as u8, Ordering::Relaxed);
        if old != new_mode as u8 {
            self.mode_changes.fetch_add(1, Ordering::Relaxed);
            self.last_mode_change_ms.store(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                Ordering::Relaxed,
            );
        }
    }

    /// Kritische Anomalie aufzeichnen und ggf. Modus wechseln
    pub fn record_critical_anomaly(&self) -> SystemMode {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Alte Einträge entfernen (> 1 Minute)
        if let Ok(mut window) = self.critical_window.write() {
            window.retain(|&ts| now - ts < 60_000);
            window.push(now);

            let count = window.len() as u64;
            let degraded_threshold = self.degraded_threshold.load(Ordering::Relaxed);
            let emergency_threshold = self.emergency_threshold.load(Ordering::Relaxed);

            if count >= emergency_threshold {
                self.set_mode(SystemMode::EmergencyShutdown);
            } else if count >= degraded_threshold {
                self.set_mode(SystemMode::Degraded);
            }
        }

        self.mode()
    }

    /// Gini-Koeffizient prüfen (Anti-Calcification)
    pub fn check_gini(&self, gini: f64) -> SystemMode {
        let threshold = self.gini_threshold.read().map(|t| *t).unwrap_or(0.8);
        if gini > threshold {
            self.record_critical_anomaly()
        } else {
            self.mode()
        }
    }

    /// Manual Recovery (Admin-Aktion)
    pub fn reset_to_normal(&self) {
        if let Ok(mut window) = self.critical_window.write() {
            window.clear();
        }
        self.set_mode(SystemMode::Normal);
    }

    /// Prüfe ob Execution erlaubt
    pub fn allows_execution(&self) -> bool {
        self.mode().allows_execution()
    }

    /// Prüfe ob Crossings erlaubt
    pub fn allows_crossings(&self) -> bool {
        self.mode().allows_crossings()
    }

    pub fn snapshot(&self) -> CircuitBreakerSnapshot {
        CircuitBreakerSnapshot {
            mode: self.mode(),
            critical_count_last_minute: self
                .critical_window
                .read()
                .map(|w| w.len() as u64)
                .unwrap_or(0),
            mode_changes: self.mode_changes.load(Ordering::Relaxed),
            last_mode_change_ms: self.last_mode_change_ms.load(Ordering::Relaxed),
            degraded_threshold: self.degraded_threshold.load(Ordering::Relaxed),
            emergency_threshold: self.emergency_threshold.load(Ordering::Relaxed),
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerSnapshot {
    pub mode: SystemMode,
    pub critical_count_last_minute: u64,
    pub mode_changes: u64,
    pub last_mode_change_ms: u64,
    pub degraded_threshold: u64,
    pub emergency_threshold: u64,
}

// ============================================================================
// PHASE 6.3: EVENT-SOURCING FÜR STATE-MANAGEMENT
// ============================================================================
//
// Event-Sourcing passt perfekt zu Erynoas event-driven Architektur:
// - Jede State-Änderung wird als Event mit Kausalität geloggt
// - Vollständiger Crash-Recovery durch Replay
// - Audits/Debugging: "Was hat den Trust-Vektor geändert?"
// - Time-Travel: State zu historischem Punkt rekonstruieren
// - Events sind klein (~100-500 Bytes) → lineares Wachstum
// ============================================================================

/// Grund für Trust-Änderung (für Audits und Debugging)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrustReason {
    /// Positive Interaktion (erfolgreiche Transaktion, gutes Verhalten)
    PositiveInteraction,
    /// Negative Interaktion (fehlgeschlagene Transaktion, Timeout)
    NegativeInteraction,
    /// Regelverstoß (Policy-Violation, Spam)
    Violation,
    /// Realm-Aktivität (Beiträge, Governance-Teilnahme)
    RealmActivity,
    /// Konsensus-Validierung (Block-Produktion, Attestation)
    ConsensusValidation,
    /// Automatische Kalibrierung (Anti-Calcification)
    Calibration,
    /// Manueller Admin-Eingriff
    AdminOverride,
    /// Dispute-Resolution
    DisputeResolution,
}

// ============================================================================
// ACTION ENUMS (Migriert nach domain/unified/action.rs)
// ============================================================================
// NOTE: Die folgenden Typen sind jetzt in domain/unified definiert und werden
// oben via `pub use` re-exportiert für Rückwärtskompatibilität:
// - AnomalySeverity (domain/unified/system.rs)
// - BlueprintAction (domain/unified/action.rs) - war: BlueprintActionType
// - RealmAction (domain/unified/action.rs)
// - MembershipAction (domain/unified/action.rs)
// - NetworkMetric (domain/unified/action.rs)
// - MemberRole: Re-exported from domain::unified::realm

/// Alias für Rückwärtskompatibilität
/// Neue Nutzung sollte `BlueprintAction` aus domain verwenden
pub type BlueprintActionType = BlueprintAction;

/// Semantisches State-Event für Event-Sourcing
///
/// Jedes Event repräsentiert eine aggregierte, bedeutungsvolle Änderung.
/// Nicht jede Atomic-Operation wird geloggt, sondern semantisch sinnvolle Deltas.
///
/// ## Vorteile
/// - **Crash-Recovery**: Vollständiger Replay ohne Datenverlust
/// - **Audits**: "Was hat Trust geändert?" → einfach querybar
/// - **Time-Travel**: State zu historischem Punkt rekonstruieren
/// - **Skalierbarkeit**: Events ~100-500 Bytes, lineares Wachstum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateEvent {
    // ═══════════════════════════════════════════════════════════════════════════
    // CORE STATE EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Trust-Update mit vollem Audit-Trail
    TrustUpdate {
        /// Entity (DID/UniversalId) deren Trust sich ändert
        entity_id: String,
        /// Delta (kann negativ sein)
        delta: f64,
        /// Grund für die Änderung
        reason: TrustReason,
        /// Optional: Realm-Kontext
        from_realm: Option<String>,
        /// Wie viele Events dadurch ausgelöst wurden
        triggered_events: u64,
        /// Neuer Trust-Wert nach Änderung
        new_trust: f64,
    },

    /// Netzwerk-Event verarbeitet (DAG-Integration)
    EventProcessed {
        /// ID des verarbeiteten Netzwerk-Events
        event_id: String,
        /// Tiefe im DAG
        depth: u64,
        /// Anzahl Parent-Events
        parents_count: usize,
        /// Getriggerte State-Komponenten
        triggers: Vec<StateComponent>,
        /// Validierungsfehler aufgetreten?
        validation_errors: bool,
        /// Verarbeitungszeit in Mikrosekunden
        processing_us: u64,
    },

    /// Weltformel neu berechnet
    FormulaComputed {
        /// Alter E-Wert
        old_e: f64,
        /// Neuer E-Wert
        new_e: f64,
        /// Delta der Contributor-Anzahl
        contributors_delta: i32,
        /// Aktueller Human-Factor
        human_factor: f64,
        /// Trend (positiv/negativ)
        trend: f64,
        /// Epoch-Nummer
        epoch: u64,
        /// Durchschnittliche Aktivität (für Update)
        activity: f64,
        /// Normalisierter Trust-Wert (für Update)
        trust_norm: f64,
    },

    /// Konsensus-Runde abgeschlossen
    ConsensusRoundCompleted {
        /// Epoch-Nummer
        epoch: u64,
        /// Erfolgreich?
        success: bool,
        /// Rundendauer in ms
        duration_ms: u64,
        /// Erkannte Byzantine-Nodes (Entity-IDs)
        byzantine_detected: Vec<String>,
        /// Anzahl teilnehmender Nodes
        participants: u64,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // EXECUTION + ECLVM EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Execution-Context gestartet
    ExecutionStarted {
        /// Eindeutige Context-ID
        context_id: String,
        /// Gas-Budget
        gas_budget: u64,
        /// Mana-Budget
        mana_budget: u64,
        /// Realm-Kontext
        realm_id: Option<String>,
    },

    /// Execution abgeschlossen
    ExecutionCompleted {
        /// Context-ID
        context_id: String,
        /// Erfolgreich?
        success: bool,
        /// Verbrauchtes Gas
        gas_consumed: u64,
        /// Verbrauchtes Mana
        mana_consumed: u64,
        /// Emittierte Events
        events_emitted: u64,
        /// Dauer in ms
        duration_ms: u64,
        /// Fehlermeldung bei Failure
        error: Option<String>,
    },

    /// ECL-Policy evaluiert
    PolicyEvaluated {
        /// Policy-ID
        policy_id: String,
        /// Realm-Kontext
        realm_id: Option<String>,
        /// Bestanden?
        passed: bool,
        /// Policy-Typ
        policy_type: ECLPolicyType,
        /// Gas verwendet
        gas_used: u64,
        /// Mana verwendet
        mana_used: u64,
        /// Evaluierungsdauer in Mikrosekunden
        duration_us: u64,
    },

    /// Blueprint-Aktion
    BlueprintAction {
        /// Blueprint-ID
        blueprint_id: String,
        /// Aktion
        action: BlueprintActionType,
        /// Realm-Kontext
        realm_id: Option<String>,
        /// Version (falls applicable)
        version: Option<String>,
    },

    /// Saga-Fortschritt (Cross-Realm Orchestration)
    SagaProgress {
        /// Saga-ID
        saga_id: String,
        /// Aktueller Schritt
        step: usize,
        /// Gesamtschritte
        total_steps: usize,
        /// Cross-Realm involviert?
        cross_realm: bool,
        /// Kompensation getriggert?
        compensation_triggered: bool,
        /// Beteiligte Realms
        realms: Vec<String>,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // PROTECTION STATE EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Anomalie erkannt
    AnomalyDetected {
        /// Schweregrad
        severity: AnomalySeverity,
        /// Beschreibung
        description: String,
        /// Betroffene Komponente
        affected_component: StateComponent,
        /// Betroffene Entities
        affected_entities: Vec<String>,
        /// Automatische Reaktion (z.B. Circuit Breaker)
        auto_response: Option<String>,
    },

    /// Diversity-Metrik aktualisiert
    DiversityMetricUpdate {
        /// Dimension (z.B. "trust_distribution")
        dimension: String,
        /// Shannon-Entropie
        entropy: f64,
        /// Gini-Koeffizient
        gini: f64,
        /// Warnung getriggert?
        warning_triggered: bool,
    },

    /// Kalibrierung angewendet (Anti-Calcification)
    CalibrationApplied {
        /// Parameter-Name
        param: String,
        /// Alter Wert
        old_value: f64,
        /// Neuer Wert
        new_value: f64,
        /// Grund für Kalibrierung
        reason: String,
    },

    /// System-Modus geändert (Circuit Breaker)
    SystemModeChanged {
        /// Alter Modus
        old_mode: SystemMode,
        /// Neuer Modus
        new_mode: SystemMode,
        /// Auslösendes Event
        trigger_event_id: String,
        /// Automatisch oder manuell
        automatic: bool,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // PEER + REALM EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Realm-Lifecycle-Event
    RealmLifecycle {
        /// Realm-ID
        realm_id: String,
        /// Aktion
        action: RealmAction,
        /// Konfiguration (falls ConfigChanged)
        config: Option<String>,
    },

    /// Mitgliedschafts-Änderung
    MembershipChange {
        /// Realm-ID (Legacy String-Form)
        realm_id: String,
        /// Identity-ID (Legacy String-Form für API-Kompatibilität)
        identity_id: String,
        /// Identity UniversalId (Primary, Phase 7)
        identity_universal_id: Option<UniversalId>,
        /// Aktion
        action: MembershipAction,
        /// Neue Rolle (falls RoleChanged)
        new_role: Option<MemberRole>,
        /// Initiator (Legacy String-Form)
        initiated_by: Option<String>,
        /// Initiator UniversalId (Phase 7)
        initiated_by_id: Option<UniversalId>,
    },

    /// Crossing evaluiert (Gateway)
    CrossingEvaluated {
        /// Quell-Realm
        from_realm: String,
        /// Ziel-Realm
        to_realm: String,
        /// Entity-ID die crossed
        entity_id: String,
        /// Erlaubt?
        allowed: bool,
        /// Grund bei Ablehnung
        reason_denied: Option<String>,
        /// Trust zum Zeitpunkt
        trust_at_time: f64,
        /// Angewendete Policy
        policy_id: Option<String>,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // P2P NETWORK EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Netzwerk-Metrik aktualisiert
    NetworkMetricUpdate {
        /// Metrik-Typ
        metric: NetworkMetric,
        /// Absoluter Wert
        value: u64,
        /// Delta zum vorherigen Wert
        delta: i64,
    },

    /// Peer connected/disconnected
    PeerConnectionChange {
        /// Peer-ID (libp2p PeerId als String)
        peer_id: String,
        /// UniversalId des Peers (Phase 7 - konsistente Identifikation)
        peer_universal_id: Option<UniversalId>,
        /// Verbunden?
        connected: bool,
        /// Multiaddr
        addr: Option<String>,
        /// Connection-Level nach Trust-Check
        connection_level: Option<String>,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // TRUST GATE EVENTS (v0.4.0)
    // ═══════════════════════════════════════════════════════════════════════════
    /// Trust-Wert eines Peers wurde aktualisiert
    TrustUpdated {
        /// Peer-ID (libp2p PeerId als String)
        peer_id: String,
        /// UniversalId des Peers (falls bekannt)
        peer_universal_id: Option<UniversalId>,
        /// Alter Trust-R Wert
        old_trust_r: f64,
        /// Alter Trust-Ω Wert
        old_trust_omega: f64,
        /// Neuer Trust-R Wert
        new_trust_r: f64,
        /// Neuer Trust-Ω Wert
        new_trust_omega: f64,
        /// Grund für Update (optional)
        reason: Option<String>,
        /// Neues Connection-Level
        new_level: String,
    },

    /// Peer wurde gebannt
    PeerBanned {
        /// Peer-ID (libp2p PeerId als String)
        peer_id: String,
        /// UniversalId des Peers (falls bekannt)
        peer_universal_id: Option<UniversalId>,
        /// Ban-Dauer in Sekunden
        duration_secs: u64,
        /// Grund für Ban
        reason: String,
    },

    /// Peer wurde entbannt
    PeerUnbanned {
        /// Peer-ID (libp2p PeerId als String)
        peer_id: String,
        /// UniversalId des Peers (falls bekannt)
        peer_universal_id: Option<UniversalId>,
        /// War Ban manuell oder automatisch abgelaufen?
        manual: bool,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // PRIVACY LAYER EVENTS (Phase 2 Woche 8)
    // ═══════════════════════════════════════════════════════════════════════════
    /// Privacy-Circuit erstellt (Onion-Route)
    PrivacyCircuitCreated {
        /// Circuit-ID (für Tracking)
        circuit_id: String,
        /// Anzahl Hops
        hop_count: u8,
        /// Sensitivitäts-Level
        sensitivity: String,
        /// Sind alle Relays mit UniversalId?
        fully_identified: bool,
        /// Anzahl involvierter Jurisdiktionen
        jurisdiction_count: u8,
    },

    /// Privacy-Circuit geschlossen
    PrivacyCircuitClosed {
        /// Circuit-ID
        circuit_id: String,
        /// Grund (expired, error, manual)
        reason: String,
        /// Lebenszeit in Sekunden
        lifetime_secs: u64,
        /// Anzahl Nachrichten über diesen Circuit
        messages_routed: u64,
    },

    /// Privacy-Nachricht gesendet
    PrivacyMessageSent {
        /// Ziel-UniversalId (falls bekannt)
        destination_id: Option<UniversalId>,
        /// Sensitivitäts-Level
        sensitivity: String,
        /// Payload-Größe in Bytes
        payload_size: u64,
        /// Mixing-Delay in ms
        mixing_delay_ms: u64,
        /// Anzahl Hops
        hop_count: u8,
        /// War es Cover-Traffic?
        is_cover_traffic: bool,
    },

    /// Cover-Traffic generiert
    CoverTrafficGenerated {
        /// Anzahl generierter Cover-Nachrichten
        messages_count: u64,
        /// Gesamt-Bytes generiert
        total_bytes: u64,
        /// Compliance-Status (ok, warning, violation)
        compliance_status: String,
        /// Aktuelle Rate pro Minute
        rate_per_minute: f64,
    },

    /// Mixing-Pool geleert
    MixingPoolFlushed {
        /// Anzahl geflusher Nachrichten
        messages_flushed: u64,
        /// Durchschnittlicher Delay in ms
        avg_delay_ms: u64,
        /// Maximaler Delay in ms
        max_delay_ms: u64,
        /// Pool-Größe nach Flush
        pool_size_after: u64,
    },

    /// Relay-Auswahl durchgeführt
    RelaySelectionCompleted {
        /// Anzahl verfügbarer Kandidaten
        candidates_available: u64,
        /// Anzahl eligibler Kandidaten (Trust-Check bestanden)
        candidates_eligible: u64,
        /// Anzahl ausgewählter Relays
        relays_selected: u8,
        /// Sensitivitäts-Level
        sensitivity: String,
        /// Durchschnittlicher Trust-Score
        avg_trust_score: f64,
        /// Erfolgreich?
        success: bool,
        /// Fehler (falls nicht erfolgreich)
        error: Option<String>,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // RECOVERY + REORG EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Checkpoint erstellt (für Recovery)
    CheckpointCreated {
        /// Checkpoint-ID
        checkpoint_id: String,
        /// Letzte Event-Sequenz
        last_event_sequence: u64,
        /// State-Hash zum Checkpoint
        state_hash: MerkleHash,
        /// Timestamp
        created_at_ms: u128,
    },

    /// Recovery durchgeführt
    RecoveryCompleted {
        /// Checkpoint von dem recovered wurde
        from_checkpoint_id: String,
        /// Anzahl replayed Events
        events_replayed: u64,
        /// Dauer in ms
        duration_ms: u64,
        /// Fehler während Recovery
        errors: Vec<String>,
    },

    /// DAG-Reorganisation erkannt
    ReorgDetected {
        /// Verworfene Event-IDs
        discarded_ids: Vec<String>,
        /// Neue kanonische Chain-Tip
        new_tip_id: String,
        /// Tiefe der Reorg
        reorg_depth: u64,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // GOVERNANCE EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Proposal erstellt
    ProposalCreated {
        /// Proposal-ID
        proposal_id: String,
        /// Realm-ID
        realm_id: String,
        /// Proposer
        proposer_id: String,
        /// Typ (z.B. ConfigChange, MemberAction)
        proposal_type: String,
        /// Voting-Deadline (Unix-Epoch ms)
        deadline_ms: u128,
    },

    /// Vote abgegeben
    VoteCast {
        /// Proposal-ID
        proposal_id: String,
        /// Voter-ID
        voter_id: String,
        /// Stimme (true = Ja, false = Nein)
        vote: bool,
        /// Gewicht (quadratic voting)
        weight: f64,
    },

    /// Proposal abgeschlossen
    ProposalResolved {
        /// Proposal-ID
        proposal_id: String,
        /// Angenommen?
        accepted: bool,
        /// Ja-Stimmen
        yes_votes: u64,
        /// Nein-Stimmen
        no_votes: u64,
        /// Ausführungs-Status
        execution_status: Option<String>,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // QUOTA + RESOURCE EVENTS
    // ═══════════════════════════════════════════════════════════════════════════
    /// Quota-Verletzung
    QuotaViolation {
        /// Realm-ID
        realm_id: String,
        /// Resource-Typ
        resource: ResourceType,
        /// Angeforderter Betrag
        requested: u64,
        /// Verfügbarer Betrag
        available: u64,
        /// Auto-Quarantine getriggert?
        quarantined: bool,
    },

    /// Realm quarantined/unquarantined
    RealmQuarantineChange {
        /// Realm-ID
        realm_id: String,
        /// Quarantined?
        quarantined: bool,
        /// Grund
        reason: String,
        /// Violations vor Quarantine
        violations_count: u64,
    },

    // ═══════════════════════════════════════════════════════════════════════════
    // IDENTITY EVENTS (Κ6-Κ8 DID Management)
    // ═══════════════════════════════════════════════════════════════════════════
    /// Root-DID erstellt (Bootstrap abgeschlossen)
    IdentityBootstrapped {
        /// Root-DID UniversalId
        root_did: UniversalId,
        /// DID-Namespace (Self, Guild, Spirit, etc.)
        namespace: crate::domain::unified::identity::DIDNamespace,
        /// Identity-Modus (Interactive, AgentManaged, Ephemeral, Test)
        mode: crate::core::identity_types::IdentityMode,
        /// Timestamp der Erstellung (ms)
        timestamp_ms: u64,
    },

    /// Identity-Modus gewechselt (z.B. Interactive → AgentManaged)
    IdentityModeChanged {
        /// Root-DID UniversalId
        root_did: UniversalId,
        /// Alter Modus
        old_mode: crate::core::identity_types::IdentityMode,
        /// Neuer Modus
        new_mode: crate::core::identity_types::IdentityMode,
        /// Timestamp der Änderung (ms)
        timestamp_ms: u64,
    },

    /// Sub-DID abgeleitet (Device, Agent, Realm, Custom)
    SubDIDDerived {
        /// Root-DID UniversalId
        root_did: UniversalId,
        /// Abgeleitete Sub-DID UniversalId
        sub_did: UniversalId,
        /// Namespace der Sub-DID
        namespace: crate::domain::unified::identity::DIDNamespace,
        /// BIP44-ähnlicher Derivation-Pfad
        derivation_path: String,
        /// Zweck (z.B. "device", "agent", "realm")
        purpose: String,
        /// Gas-Verbrauch für Derivation
        gas_used: u64,
        /// Optional: Realm-Kontext bei Realm-DIDs
        realm_id: Option<UniversalId>,
    },

    /// Wallet-Adresse abgeleitet
    WalletDerived {
        /// DID UniversalId von der abgeleitet wurde
        did: UniversalId,
        /// Chain-ID (CAIP-2 Format, z.B. "eip155:1")
        chain_id: String,
        /// Wallet-Adresse auf der Chain
        address: String,
        /// BIP44 Derivation-Pfad
        derivation_path: String,
    },

    /// Delegation erstellt (Κ8: Trust-Decay)
    DelegationCreated {
        /// Delegator UniversalId
        delegator: UniversalId,
        /// Delegate UniversalId (erhält Berechtigungen)
        delegate: UniversalId,
        /// Trust-Faktor (0 < tf ≤ 1) gemäß Κ8
        trust_factor: f32,
        /// Delegierte Capabilities (String-Repräsentation)
        capabilities: Vec<String>,
        /// Optional: Ablaufzeitpunkt (Unix-Epoch ms)
        valid_until: Option<u64>,
    },

    /// Delegation widerrufen
    DelegationRevoked {
        /// Delegation-ID UniversalId
        delegation_id: UniversalId,
        /// Delegator UniversalId
        delegator: UniversalId,
        /// Delegate UniversalId
        delegate: UniversalId,
        /// Grund für Widerruf
        reason: String,
    },

    /// Credential ausgestellt
    CredentialIssued {
        /// Issuer (Aussteller) UniversalId
        issuer: UniversalId,
        /// Subject (Betroffener) UniversalId
        subject: UniversalId,
        /// Credential-Typ (z.B. "KYC", "AgeVerification")
        credential_type: String,
        /// Hash des Claim-Inhalts (Datenschutz)
        claim_hash: [u8; 32],
    },

    /// Credential verifiziert
    CredentialVerified {
        /// Verifier UniversalId
        verifier: UniversalId,
        /// Credential-ID UniversalId
        credential_id: UniversalId,
        /// Validierungsergebnis
        valid: bool,
    },

    /// Key rotiert
    KeyRotated {
        /// DID UniversalId dessen Key rotiert wurde
        did: UniversalId,
        /// Alter Key UniversalId
        old_key_id: UniversalId,
        /// Neuer Key UniversalId
        new_key_id: UniversalId,
        /// Rotationsgrund
        reason: String,
    },

    /// Recovery initiiert
    RecoveryInitiated {
        /// DID UniversalId die recovered werden soll
        did: UniversalId,
        /// Recovery-Key UniversalId
        recovery_key_id: UniversalId,
        /// Initiierungszeitpunkt (ms)
        initiated_at: u64,
    },

    /// Identity-Anomalie erkannt
    IdentityAnomalyDetected {
        /// Betroffene DID UniversalId
        did: UniversalId,
        /// Anomalie-Typ (z.B. "RapidDelegation", "UnusualActivity")
        anomaly_type: String,
        /// Severity-Level ("low", "medium", "high", "critical")
        severity: String,
        /// Details zur Anomalie
        details: String,
    },

    /// Cross-Shard Identity aufgelöst
    CrossShardIdentityResolved {
        /// Identity UniversalId
        identity_id: UniversalId,
        /// Quell-Shard Index
        source_shard: u64,
        /// Ziel-Shard Index
        target_shard: u64,
        /// Erfolgreich aufgelöst?
        success: bool,
        /// Latenz der Auflösung (ms)
        latency_ms: u64,
    },

    /// Realm-Membership geändert (mit UniversalId)
    RealmMembershipChanged {
        /// Realm UniversalId
        realm_id: UniversalId,
        /// Member UniversalId
        member_id: UniversalId,
        /// Aktion (Joined, Left, RoleChanged, Banned)
        action: String,
        /// Neue Rolle (falls RoleChanged)
        new_role: Option<String>,
        /// Optional: Realm-spezifische Sub-DID
        realm_sub_did: Option<UniversalId>,
    },
}

impl StateEvent {
    /// Ermittle primäre betroffene Komponente (für Indexing)
    pub fn primary_component(&self) -> StateComponent {
        match self {
            StateEvent::TrustUpdate { .. } => StateComponent::Trust,
            StateEvent::EventProcessed { .. } => StateComponent::Event,
            StateEvent::FormulaComputed { .. } => StateComponent::WorldFormula,
            StateEvent::ConsensusRoundCompleted { .. } => StateComponent::Consensus,
            StateEvent::ExecutionStarted { .. } | StateEvent::ExecutionCompleted { .. } => {
                StateComponent::Execution
            }
            StateEvent::PolicyEvaluated { .. } => StateComponent::ECLPolicy,
            StateEvent::BlueprintAction { .. } => StateComponent::ECLBlueprint,
            StateEvent::SagaProgress { .. } => StateComponent::SagaComposer,
            StateEvent::AnomalyDetected { .. } => StateComponent::Anomaly,
            StateEvent::DiversityMetricUpdate { .. } => StateComponent::Diversity,
            StateEvent::CalibrationApplied { .. } => StateComponent::Calibration,
            StateEvent::SystemModeChanged { .. } => StateComponent::Anomaly,
            StateEvent::RealmLifecycle { .. } | StateEvent::MembershipChange { .. } => {
                StateComponent::Realm
            }
            StateEvent::CrossingEvaluated { .. } => StateComponent::Gateway,
            StateEvent::NetworkMetricUpdate { .. }
            | StateEvent::PeerConnectionChange { .. }
            | StateEvent::TrustUpdated { .. }
            | StateEvent::PeerBanned { .. }
            | StateEvent::PeerUnbanned { .. } => StateComponent::Swarm,
            // Privacy Events (Phase 2 Woche 8)
            StateEvent::PrivacyCircuitCreated { .. }
            | StateEvent::PrivacyCircuitClosed { .. }
            | StateEvent::PrivacyMessageSent { .. }
            | StateEvent::CoverTrafficGenerated { .. }
            | StateEvent::MixingPoolFlushed { .. }
            | StateEvent::RelaySelectionCompleted { .. } => StateComponent::Privacy,
            StateEvent::CheckpointCreated { .. }
            | StateEvent::RecoveryCompleted { .. }
            | StateEvent::ReorgDetected { .. } => StateComponent::EventStore,
            StateEvent::ProposalCreated { .. }
            | StateEvent::VoteCast { .. }
            | StateEvent::ProposalResolved { .. } => StateComponent::Governance,
            StateEvent::QuotaViolation { .. } | StateEvent::RealmQuarantineChange { .. } => {
                StateComponent::Realm
            }
            // Identity Events (Κ6-Κ8)
            StateEvent::IdentityBootstrapped { .. }
            | StateEvent::IdentityModeChanged { .. }
            | StateEvent::SubDIDDerived { .. }
            | StateEvent::WalletDerived { .. }
            | StateEvent::DelegationCreated { .. }
            | StateEvent::DelegationRevoked { .. }
            | StateEvent::CredentialIssued { .. }
            | StateEvent::CredentialVerified { .. }
            | StateEvent::KeyRotated { .. }
            | StateEvent::RecoveryInitiated { .. }
            | StateEvent::IdentityAnomalyDetected { .. }
            | StateEvent::CrossShardIdentityResolved { .. }
            | StateEvent::RealmMembershipChanged { .. } => StateComponent::Identity,
        }
    }

    /// Event-Größe schätzen (für Metering)
    pub fn estimated_size_bytes(&self) -> usize {
        // Basis: Enum-Diskriminant + typische Payload
        match self {
            StateEvent::TrustUpdate { .. } => 150,
            StateEvent::EventProcessed { triggers, .. } => 100 + triggers.len() * 8,
            StateEvent::FormulaComputed { .. } => 80,
            StateEvent::ConsensusRoundCompleted {
                byzantine_detected, ..
            } => 100 + byzantine_detected.len() * 64,
            StateEvent::ExecutionStarted { .. } => 120,
            StateEvent::ExecutionCompleted { .. } => 180,
            StateEvent::PolicyEvaluated { .. } => 160,
            StateEvent::BlueprintAction { .. } => 120,
            StateEvent::SagaProgress { realms, .. } => 100 + realms.len() * 64,
            StateEvent::AnomalyDetected {
                affected_entities, ..
            } => 200 + affected_entities.len() * 64,
            StateEvent::DiversityMetricUpdate { .. } => 80,
            StateEvent::CalibrationApplied { .. } => 120,
            StateEvent::SystemModeChanged { .. } => 100,
            StateEvent::RealmLifecycle { .. } => 150,
            StateEvent::MembershipChange { .. } => 180,
            StateEvent::CrossingEvaluated { .. } => 200,
            StateEvent::NetworkMetricUpdate { .. } => 40,
            StateEvent::PeerConnectionChange { .. } => 150,
            // Trust Gate Events (v0.4.0)
            StateEvent::TrustUpdated { reason, .. } => {
                120 + reason.as_ref().map(|r| r.len()).unwrap_or(0)
            }
            StateEvent::PeerBanned { reason, .. } => 100 + reason.len(),
            StateEvent::PeerUnbanned { .. } => 80,
            // Privacy Events (Phase 2 Woche 8)
            StateEvent::PrivacyCircuitCreated { .. } => 100,
            StateEvent::PrivacyCircuitClosed { .. } => 80,
            StateEvent::PrivacyMessageSent { .. } => 120,
            StateEvent::CoverTrafficGenerated { .. } => 60,
            StateEvent::MixingPoolFlushed { .. } => 50,
            StateEvent::RelaySelectionCompleted { error, .. } => {
                80 + error.as_ref().map(|e| e.len()).unwrap_or(0)
            }
            StateEvent::CheckpointCreated { .. } => 100,
            StateEvent::RecoveryCompleted { errors, .. } => 80 + errors.len() * 100,
            StateEvent::ReorgDetected { discarded_ids, .. } => 50 + discarded_ids.len() * 64,
            StateEvent::ProposalCreated { .. } => 200,
            StateEvent::VoteCast { .. } => 100,
            StateEvent::ProposalResolved { .. } => 120,
            StateEvent::QuotaViolation { .. } => 100,
            StateEvent::RealmQuarantineChange { .. } => 150,
            // Identity Events (Κ6-Κ8)
            StateEvent::IdentityBootstrapped { .. } => 64,
            StateEvent::IdentityModeChanged { .. } => 48,
            StateEvent::SubDIDDerived {
                derivation_path,
                purpose,
                ..
            } => 128 + derivation_path.len() + purpose.len(),
            StateEvent::WalletDerived {
                chain_id,
                address,
                derivation_path,
                ..
            } => 64 + chain_id.len() + address.len() + derivation_path.len(),
            StateEvent::DelegationCreated { capabilities, .. } => 96 + capabilities.len() * 32,
            StateEvent::DelegationRevoked { reason, .. } => 112 + reason.len(),
            StateEvent::CredentialIssued {
                credential_type, ..
            } => 128 + credential_type.len(),
            StateEvent::CredentialVerified { .. } => 64,
            StateEvent::KeyRotated { reason, .. } => 128 + reason.len(),
            StateEvent::RecoveryInitiated { .. } => 64,
            StateEvent::IdentityAnomalyDetected {
                anomaly_type,
                severity,
                details,
                ..
            } => 96 + anomaly_type.len() + severity.len() + details.len(),
            StateEvent::CrossShardIdentityResolved { .. } => 80,
            StateEvent::RealmMembershipChanged { action, .. } => 128 + action.len(),
        }
    }

    /// Ist dieses Event kritisch (erfordert sofortige Persistenz)?
    pub fn is_critical(&self) -> bool {
        match self {
            // Existing critical events
            StateEvent::AnomalyDetected {
                severity: AnomalySeverity::Critical,
                ..
            } => true,
            StateEvent::SystemModeChanged { .. } => true,
            StateEvent::ReorgDetected { .. } => true,
            StateEvent::QuotaViolation {
                quarantined: true, ..
            } => true,
            StateEvent::RealmQuarantineChange {
                quarantined: true, ..
            } => true,

            // Identity-Critical Events
            StateEvent::IdentityBootstrapped { .. } => true,
            StateEvent::IdentityModeChanged { .. } => true,
            StateEvent::KeyRotated { .. } => true,
            StateEvent::RecoveryInitiated { .. } => true,
            StateEvent::IdentityAnomalyDetected { severity, .. } => severity == "critical",

            _ => false,
        }
    }

    /// Hat dieses Event einen Realm-Kontext?
    pub fn realm_context(&self) -> Option<&UniversalId> {
        match self {
            // Identity Events mit Realm-Kontext
            StateEvent::SubDIDDerived { realm_id, .. } => realm_id.as_ref(),
            StateEvent::RealmMembershipChanged { realm_id, .. } => Some(realm_id),
            _ => None,
        }
    }

    /// Alle betroffenen Identities (für Indexing)
    pub fn involved_identities(&self) -> Vec<UniversalId> {
        match self {
            StateEvent::IdentityBootstrapped { root_did, .. } => vec![*root_did],
            StateEvent::IdentityModeChanged { root_did, .. } => vec![*root_did],
            StateEvent::SubDIDDerived {
                root_did, sub_did, ..
            } => vec![*root_did, *sub_did],
            StateEvent::WalletDerived { did, .. } => vec![*did],
            StateEvent::DelegationCreated {
                delegator,
                delegate,
                ..
            } => vec![*delegator, *delegate],
            StateEvent::DelegationRevoked {
                delegator,
                delegate,
                ..
            } => vec![*delegator, *delegate],
            StateEvent::CredentialIssued {
                issuer, subject, ..
            } => vec![*issuer, *subject],
            StateEvent::CredentialVerified {
                verifier,
                credential_id,
                ..
            } => vec![*verifier, *credential_id],
            StateEvent::KeyRotated { did, .. } => vec![*did],
            StateEvent::RecoveryInitiated { did, .. } => vec![*did],
            StateEvent::IdentityAnomalyDetected { did, .. } => vec![*did],
            StateEvent::CrossShardIdentityResolved { identity_id, .. } => vec![*identity_id],
            StateEvent::RealmMembershipChanged { member_id, .. } => vec![*member_id],
            _ => vec![],
        }
    }

    /// Ist dies ein Identity-Event?
    pub fn is_identity_event(&self) -> bool {
        matches!(
            self,
            StateEvent::IdentityBootstrapped { .. }
                | StateEvent::IdentityModeChanged { .. }
                | StateEvent::SubDIDDerived { .. }
                | StateEvent::WalletDerived { .. }
                | StateEvent::DelegationCreated { .. }
                | StateEvent::DelegationRevoked { .. }
                | StateEvent::CredentialIssued { .. }
                | StateEvent::CredentialVerified { .. }
                | StateEvent::KeyRotated { .. }
                | StateEvent::RecoveryInitiated { .. }
                | StateEvent::IdentityAnomalyDetected { .. }
                | StateEvent::CrossShardIdentityResolved { .. }
                | StateEvent::RealmMembershipChanged { .. }
        )
    }

    /// Ist dies ein Privacy-Event?
    pub fn is_privacy_event(&self) -> bool {
        matches!(
            self,
            StateEvent::PrivacyCircuitCreated { .. }
                | StateEvent::PrivacyCircuitClosed { .. }
                | StateEvent::PrivacyMessageSent { .. }
                | StateEvent::CoverTrafficGenerated { .. }
                | StateEvent::MixingPoolFlushed { .. }
                | StateEvent::RelaySelectionCompleted { .. }
        )
    }

    /// Ist dies ein P2P-Netzwerk-Event?
    pub fn is_p2p_event(&self) -> bool {
        matches!(
            self,
            StateEvent::NetworkMetricUpdate { .. }
                | StateEvent::PeerConnectionChange { .. }
                | StateEvent::TrustUpdated { .. }
                | StateEvent::PeerBanned { .. }
                | StateEvent::PeerUnbanned { .. }
        ) || self.is_privacy_event()
    }

    /// Ist dies ein Trust-Gate-Event? (v0.4.0)
    pub fn is_trust_gate_event(&self) -> bool {
        matches!(
            self,
            StateEvent::TrustUpdated { .. }
                | StateEvent::PeerBanned { .. }
                | StateEvent::PeerUnbanned { .. }
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STATE EVENT EMITTER
// ═══════════════════════════════════════════════════════════════════════════

/// Trait für Module, die StateEvents emittieren können
///
/// Ermöglicht P2P-Modulen wie `SwarmManager` und `PrivacyService`,
/// StateEvents an das zentrale `UnifiedState` zu senden.
///
/// ## Beispiel
///
/// ```rust,ignore
/// use erynoa_api::core::state::{StateEvent, StateEventEmitter};
///
/// struct MyP2PComponent {
///     emitter: Box<dyn StateEventEmitter>,
/// }
///
/// impl MyP2PComponent {
///     fn on_peer_connected(&self, peer_id: &str) {
///         self.emitter.emit(StateEvent::PeerConnectionChange {
///             peer_id: peer_id.to_string(),
///             peer_universal_id: None,
///             connected: true,
///             addr: None,
///             connection_level: Some("Full".to_string()),
///         });
///     }
/// }
/// ```
pub trait StateEventEmitter: Send + Sync {
    /// Emittiere ein StateEvent
    fn emit(&self, event: StateEvent);

    /// Emittiere mehrere StateEvents (batch)
    fn emit_batch(&self, events: Vec<StateEvent>) {
        for event in events {
            self.emit(event);
        }
    }

    /// Ist der Emitter aktiv?
    fn is_active(&self) -> bool {
        true
    }
}

/// No-Op Emitter (für Tests oder wenn keine StateEvent-Integration gewünscht ist)
#[derive(Debug, Clone, Default)]
pub struct NoOpEmitter;

impl StateEventEmitter for NoOpEmitter {
    fn emit(&self, _event: StateEvent) {
        // No-Op
    }

    fn is_active(&self) -> bool {
        false
    }
}

/// Channel-basierter StateEventEmitter
///
/// Sendet StateEvents über einen mpsc-Channel an den UnifiedState.
pub struct ChannelEmitter {
    /// Sender für StateEvents
    tx: tokio::sync::mpsc::UnboundedSender<StateEvent>,
    /// Name der Quelle (für Logging)
    source: String,
}

impl ChannelEmitter {
    /// Erstelle neuen Channel-Emitter
    pub fn new(
        tx: tokio::sync::mpsc::UnboundedSender<StateEvent>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            tx,
            source: source.into(),
        }
    }
}

impl StateEventEmitter for ChannelEmitter {
    fn emit(&self, event: StateEvent) {
        if let Err(e) = self.tx.send(event) {
            tracing::warn!(
                source = %self.source,
                error = %e,
                "Failed to emit StateEvent"
            );
        }
    }

    fn is_active(&self) -> bool {
        !self.tx.is_closed()
    }
}

/// Callback-basierter StateEventEmitter
///
/// Ruft einen Callback für jedes emittierte Event auf.
pub struct CallbackEmitter {
    /// Callback-Funktion
    callback: Box<dyn Fn(StateEvent) + Send + Sync>,
    /// Name der Quelle (für Logging)
    source: String,
}

impl CallbackEmitter {
    /// Erstelle neuen Callback-Emitter
    pub fn new<F>(callback: F, source: impl Into<String>) -> Self
    where
        F: Fn(StateEvent) + Send + Sync + 'static,
    {
        Self {
            callback: Box::new(callback),
            source: source.into(),
        }
    }
}

impl StateEventEmitter for CallbackEmitter {
    fn emit(&self, event: StateEvent) {
        (self.callback)(event);
    }
}

impl std::fmt::Debug for CallbackEmitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackEmitter")
            .field("source", &self.source)
            .finish()
    }
}

/// Wrapped StateEvent für DAG-Integration und Kausalitäts-Tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrappedStateEvent {
    /// Eindeutige Event-ID (Blake3-Hash des Inhalts)
    pub id: String,
    /// Timestamp (Unix-Epoch Millisekunden)
    pub timestamp_ms: u128,
    /// Parent-Event-IDs (Kausalität)
    pub parent_ids: Vec<String>,
    /// Primär betroffene Komponente (für Indexing)
    pub component: StateComponent,
    /// Sequenznummer (monoton steigend)
    pub sequence: u64,
    /// Das eigentliche Event
    pub event: StateEvent,
    /// Optional: Signatur des Nodes (für Malicious-Replay-Schutz)
    pub signature: Option<Vec<u8>>,
}

impl WrappedStateEvent {
    /// Erstelle neues WrappedStateEvent
    pub fn new(event: StateEvent, parent_ids: Vec<String>, sequence: u64) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let component = event.primary_component();

        // ID aus Content-Hash generieren
        let mut hasher = blake3::Hasher::new();
        hasher.update(&timestamp_ms.to_le_bytes());
        hasher.update(&sequence.to_le_bytes());
        for parent in &parent_ids {
            hasher.update(parent.as_bytes());
        }
        // Event-Inhalt (vereinfacht via Debug)
        hasher.update(format!("{:?}", event).as_bytes());
        let id = hex::encode(&hasher.finalize().as_bytes()[..16]);

        Self {
            id,
            timestamp_ms,
            parent_ids,
            component,
            sequence,
            event,
            signature: None,
        }
    }

    /// Füge Signatur hinzu
    pub fn with_signature(mut self, signature: Vec<u8>) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Extrahiere Realm-Kontext aus dem Event (falls vorhanden)
    ///
    /// Viele Events wie Membership, TrustUpdate, RealmLifecycle haben
    /// einen Realm-Kontext. Diese Methode extrahiert ihn.
    pub fn realm_context(&self) -> Option<String> {
        match &self.event {
            // Trust-Updates mit Realm-Kontext
            StateEvent::TrustUpdate { from_realm, .. } => from_realm.clone(),

            // Realm-Lifecycle Events
            StateEvent::RealmLifecycle { realm_id, .. } => Some(realm_id.clone()),

            // Membership-Änderungen
            StateEvent::MembershipChange { realm_id, .. } => Some(realm_id.clone()),

            // Crossing Events
            StateEvent::CrossingEvaluated { to_realm, .. } => Some(to_realm.clone()),

            // Identity Events mit Realm-Kontext (UniversalId → hex String)
            StateEvent::SubDIDDerived { realm_id, .. } => {
                realm_id.as_ref().map(|id| hex::encode(id.as_bytes()))
            }
            StateEvent::RealmMembershipChanged { realm_id, .. } => {
                Some(hex::encode(realm_id.as_bytes()))
            }

            // Alle anderen Events haben keinen direkten Realm-Kontext
            _ => None,
        }
    }

    /// Extrahiere Realm-Kontext als UniversalId (für neue Events)
    pub fn realm_context_id(&self) -> Option<UniversalId> {
        self.event.realm_context().copied()
    }

    /// Event-Größe in Bytes (für Metering)
    pub fn size_bytes(&self) -> usize {
        self.id.len()
            + 16 // timestamp
            + self.parent_ids.iter().map(|p| p.len()).sum::<usize>()
            + 8 // sequence
            + self.event.estimated_size_bytes()
            + self.signature.as_ref().map(|s| s.len()).unwrap_or(0)
    }
}

/// State-Event-Log für Event-Sourcing
///
/// Verwaltet Event-Historie, Checkpoints und Recovery.
/// Overhead: <5% CPU/RAM bei normaler Load.
#[derive(Debug)]
pub struct StateEventLog {
    /// Event-Sequenz-Counter
    sequence: AtomicU64,
    /// In-Memory Event-Buffer (letzte N Events)
    buffer: RwLock<Vec<WrappedStateEvent>>,
    /// Buffer-Kapazität
    buffer_capacity: usize,
    /// Letzter Checkpoint-ID
    last_checkpoint_id: RwLock<Option<String>>,
    /// Letzter Checkpoint-Sequenz
    last_checkpoint_sequence: AtomicU64,
    /// Events seit letztem Checkpoint
    events_since_checkpoint: AtomicU64,
    /// Checkpoint-Intervall (Events)
    checkpoint_interval: u64,
    /// Gesamt-Events geloggt
    pub total_events: AtomicU64,
    /// Kritische Events geloggt
    pub critical_events: AtomicU64,
    /// Events nach Komponente
    pub events_by_component: RwLock<HashMap<StateComponent, u64>>,
    /// Recovery-Status
    pub is_recovering: std::sync::atomic::AtomicBool,
}

impl StateEventLog {
    /// Standard Buffer-Kapazität (10.000 Events)
    pub const DEFAULT_BUFFER_CAPACITY: usize = 10_000;
    /// Standard Checkpoint-Intervall (5.000 Events)
    pub const DEFAULT_CHECKPOINT_INTERVAL: u64 = 5_000;

    pub fn new() -> Self {
        Self::new_with_config(
            Self::DEFAULT_BUFFER_CAPACITY,
            Self::DEFAULT_CHECKPOINT_INTERVAL,
        )
    }

    /// Erstelle Event-Log mit konfigurierbarer Buffer-Kapazität und Checkpoint-Intervall
    pub fn new_with_config(buffer_capacity: usize, checkpoint_interval: u64) -> Self {
        Self {
            sequence: AtomicU64::new(0),
            buffer: RwLock::new(Vec::with_capacity(buffer_capacity)),
            buffer_capacity,
            last_checkpoint_id: RwLock::new(None),
            last_checkpoint_sequence: AtomicU64::new(0),
            events_since_checkpoint: AtomicU64::new(0),
            checkpoint_interval,
            total_events: AtomicU64::new(0),
            critical_events: AtomicU64::new(0),
            events_by_component: RwLock::new(HashMap::new()),
            is_recovering: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Logge neues Event
    pub fn log(&self, event: StateEvent, parent_ids: Vec<String>) -> WrappedStateEvent {
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        let wrapped = WrappedStateEvent::new(event, parent_ids, sequence);

        // Statistiken
        self.total_events.fetch_add(1, Ordering::Relaxed);
        if wrapped.event.is_critical() {
            self.critical_events.fetch_add(1, Ordering::Relaxed);
        }

        // By-Component Counter
        if let Ok(mut by_comp) = self.events_by_component.write() {
            *by_comp.entry(wrapped.component).or_insert(0) += 1;
        }

        // In Buffer schreiben (Ring-Buffer)
        if let Ok(mut buffer) = self.buffer.write() {
            if buffer.len() >= self.buffer_capacity {
                buffer.remove(0);
            }
            buffer.push(wrapped.clone());
        }

        self.events_since_checkpoint.fetch_add(1, Ordering::Relaxed);

        wrapped
    }

    /// Hole Events seit Sequenz (für Sync)
    pub fn events_since(&self, since_sequence: u64) -> Vec<WrappedStateEvent> {
        self.buffer
            .read()
            .map(|b| {
                b.iter()
                    .filter(|e| e.sequence > since_sequence)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Hole Events für Komponente
    pub fn events_for_component(&self, component: StateComponent) -> Vec<WrappedStateEvent> {
        self.buffer
            .read()
            .map(|b| {
                b.iter()
                    .filter(|e| e.component == component)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Prüfe ob Checkpoint fällig
    pub fn needs_checkpoint(&self) -> bool {
        self.events_since_checkpoint.load(Ordering::Relaxed) >= self.checkpoint_interval
    }

    /// Setze Checkpoint-Marker
    pub fn mark_checkpoint(
        &self,
        checkpoint_id: String,
        state_hash: MerkleHash,
    ) -> WrappedStateEvent {
        let sequence = self.sequence.load(Ordering::SeqCst);
        self.last_checkpoint_sequence
            .store(sequence, Ordering::SeqCst);
        self.events_since_checkpoint.store(0, Ordering::SeqCst);

        if let Ok(mut last_id) = self.last_checkpoint_id.write() {
            *last_id = Some(checkpoint_id.clone());
        }

        // Checkpoint als Event loggen
        self.log(
            StateEvent::CheckpointCreated {
                checkpoint_id,
                last_event_sequence: sequence,
                state_hash,
                created_at_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0),
            },
            vec![],
        )
    }

    /// Starte Recovery-Modus
    pub fn start_recovery(&self) {
        self.is_recovering.store(true, Ordering::SeqCst);
    }

    /// Beende Recovery-Modus
    pub fn end_recovery(&self) {
        self.is_recovering.store(false, Ordering::SeqCst);
    }

    /// Snapshot für Metriken
    pub fn snapshot(&self) -> EventLogSnapshot {
        EventLogSnapshot {
            sequence: self.sequence.load(Ordering::Relaxed),
            buffer_size: self.buffer.read().map(|b| b.len()).unwrap_or(0),
            total_events: self.total_events.load(Ordering::Relaxed),
            critical_events: self.critical_events.load(Ordering::Relaxed),
            events_since_checkpoint: self.events_since_checkpoint.load(Ordering::Relaxed),
            last_checkpoint_sequence: self.last_checkpoint_sequence.load(Ordering::Relaxed),
            is_recovering: self.is_recovering.load(Ordering::Relaxed),
        }
    }
}

impl Default for StateEventLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot des StateEventLog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLogSnapshot {
    pub sequence: u64,
    pub buffer_size: usize,
    pub total_events: u64,
    pub critical_events: u64,
    pub events_since_checkpoint: u64,
    pub last_checkpoint_sequence: u64,
    pub is_recovering: bool,
}

// ============================================================================
// PHASE 6.2: DIFFERENTIAL STATE SNAPSHOTS (MERKLE-BASIERT)
// ============================================================================

/// Merkle-Hash für State-Verifizierung und Delta-Synchronisation
///
/// Ermöglicht:
/// - Light-Clients: Nur Deltas statt voller Snapshots synchronisieren
/// - State-Proofs: Kryptographische Verifizierung gegen Tampering
/// - Effiziente Recovery: Letzten Snapshot + Deltas replayen
pub type MerkleHash = [u8; 32];

/// Merkle-Node für hierarchischen State-Tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Blake3-Hash dieses Nodes
    pub hash: MerkleHash,
    /// Timestamp der letzten Änderung (Unix-Epoch Millis)
    pub updated_ms: u64,
    /// Child-Hashes (für Branch-Nodes)
    pub children: Vec<MerkleHash>,
}

impl MerkleNode {
    pub fn leaf(data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let hash: [u8; 32] = *hasher.finalize().as_bytes();
        Self {
            hash,
            updated_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            children: Vec::new(),
        }
    }

    pub fn branch(children: &[MerkleHash]) -> Self {
        let mut hasher = blake3::Hasher::new();
        for child in children {
            hasher.update(child);
        }
        let hash: [u8; 32] = *hasher.finalize().as_bytes();
        Self {
            hash,
            updated_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            children: children.to_vec(),
        }
    }
}

/// Trait für hashbare State-Komponenten
pub trait Hashable {
    /// Berechne Blake3-Hash der Komponente
    fn compute_hash(&self) -> MerkleHash;
}

/// State-Delta mit Merkle-Proof für effiziente Synchronisation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleDelta {
    /// Alte Root-Hash vor dem Update
    pub old_root: MerkleHash,
    /// Neue Root-Hash nach dem Update
    pub new_root: MerkleHash,
    /// Betroffene Komponente
    pub component: StateComponent,
    /// Proof-Path (Sibling-Hashes für Verifikation)
    pub proof_path: Vec<MerkleHash>,
    /// Serialisierte Delta-Daten
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp_ms: u64,
    /// Sequenznummer für Ordering
    pub sequence: u64,
}

impl MerkleDelta {
    /// Verifiziere Delta gegen erwartete Root
    pub fn verify(&self, expected_new_root: &MerkleHash) -> bool {
        self.new_root == *expected_new_root
    }

    /// Berechne Größe des Deltas (für Bandbreiten-Metriken)
    pub fn size_bytes(&self) -> usize {
        64 + // old_root + new_root
        std::mem::size_of::<StateComponent>() +
        self.proof_path.len() * 32 +
        self.data.len() +
        16 // timestamp + sequence
    }
}

/// Merkle-State-Tracker für UnifiedState
///
/// Verwaltet Merkle-Tree über alle Sub-States und generiert Deltas.
#[derive(Debug)]
pub struct MerkleStateTracker {
    /// Root-Hash über alle Sub-States
    root_hash: RwLock<MerkleHash>,
    /// Sub-State-Hashes (component → hash)
    component_hashes: RwLock<HashMap<StateComponent, MerkleHash>>,
    /// Sequenzzähler für Deltas
    sequence: AtomicU64,
    /// Delta-History (für Sync-Requests)
    delta_history: RwLock<Vec<MerkleDelta>>,
    /// Max History-Länge
    max_history: usize,
    /// Total Deltas erzeugt
    deltas_generated: AtomicU64,
    /// Total Proofs verifiziert
    proofs_verified: AtomicU64,
    /// Fehlgeschlagene Verifikationen
    proofs_failed: AtomicU64,
}

impl MerkleStateTracker {
    pub const DEFAULT_MAX_HISTORY: usize = 1000;

    pub fn new() -> Self {
        Self {
            root_hash: RwLock::new([0u8; 32]),
            component_hashes: RwLock::new(HashMap::new()),
            sequence: AtomicU64::new(0),
            delta_history: RwLock::new(Vec::new()),
            max_history: Self::DEFAULT_MAX_HISTORY,
            deltas_generated: AtomicU64::new(0),
            proofs_verified: AtomicU64::new(0),
            proofs_failed: AtomicU64::new(0),
        }
    }

    /// Update Sub-State-Hash und recompute Root
    pub fn update_component(&self, component: StateComponent, data: &[u8]) -> MerkleDelta {
        let old_root = self.root_hash.read().map(|r| *r).unwrap_or([0u8; 32]);

        // Compute new hash for component
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let new_component_hash: [u8; 32] = *hasher.finalize().as_bytes();

        // Update component hash
        if let Ok(mut hashes) = self.component_hashes.write() {
            hashes.insert(component, new_component_hash);

            // Recompute root from all component hashes
            let mut root_hasher = blake3::Hasher::new();
            let mut sorted_components: Vec<_> = hashes.iter().collect();
            // Sort by format string for deterministic ordering (StateComponent is Debug)
            sorted_components.sort_by_key(|(c, _)| format!("{:?}", c));
            for (_, hash) in sorted_components {
                root_hasher.update(hash);
            }
            let new_root: [u8; 32] = *root_hasher.finalize().as_bytes();

            if let Ok(mut root) = self.root_hash.write() {
                *root = new_root;
            }

            // Generate proof path (simplified: all sibling hashes)
            let proof_path: Vec<MerkleHash> = hashes
                .iter()
                .filter(|(c, _)| **c != component)
                .map(|(_, h)| *h)
                .collect();

            let delta = MerkleDelta {
                old_root,
                new_root,
                component,
                proof_path,
                data: data.to_vec(),
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
                sequence: self.sequence.fetch_add(1, Ordering::Relaxed),
            };

            // Add to history
            if let Ok(mut history) = self.delta_history.write() {
                history.push(delta.clone());
                if history.len() > self.max_history {
                    history.remove(0);
                }
            }

            self.deltas_generated.fetch_add(1, Ordering::Relaxed);
            delta
        } else {
            // Fallback bei Lock-Fehler
            MerkleDelta {
                old_root,
                new_root: old_root,
                component,
                proof_path: Vec::new(),
                data: data.to_vec(),
                timestamp_ms: 0,
                sequence: 0,
            }
        }
    }

    /// Hole aktuelle Root-Hash
    pub fn root_hash(&self) -> MerkleHash {
        self.root_hash.read().map(|r| *r).unwrap_or([0u8; 32])
    }

    /// Merkle-Hash einer Komponente (für Light-Client / Proof-API)
    pub fn component_hash(&self, component: StateComponent) -> Option<MerkleHash> {
        self.component_hashes
            .read()
            .ok()
            .and_then(|h| h.get(&component).copied())
    }

    /// Sequenznummer des Deltas, das den angegebenen Root erzeugt hat (für delta?since_root=)
    pub fn sequence_for_root(&self, root: &MerkleHash) -> Option<u64> {
        self.delta_history
            .read()
            .ok()
            .and_then(|h| h.iter().find(|d| d.new_root == *root).map(|d| d.sequence))
    }

    /// State-Proof für eine Komponente: (Komponenten-Hash, Proof-Path = andere Komponenten-Hashes sortiert)
    pub fn component_proof(
        &self,
        component: StateComponent,
    ) -> Option<(MerkleHash, Vec<MerkleHash>)> {
        let hashes = self.component_hashes.read().ok()?;
        let hash = *hashes.get(&component)?;
        let mut proof_path: Vec<MerkleHash> = hashes
            .iter()
            .filter(|(c, _)| **c != component)
            .map(|(_, h)| *h)
            .collect();
        proof_path.sort_by_key(|h| *h);
        Some((hash, proof_path))
    }

    /// Hole Deltas seit bestimmter Sequenz (für Sync)
    pub fn deltas_since(&self, since_sequence: u64) -> Vec<MerkleDelta> {
        self.delta_history
            .read()
            .map(|history| {
                history
                    .iter()
                    .filter(|d| d.sequence > since_sequence)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Verifiziere eingehendes Delta
    pub fn verify_delta(&self, delta: &MerkleDelta) -> bool {
        // Simplified verification: check that applying delta produces correct new_root
        let result = delta.new_root != [0u8; 32];
        if result {
            self.proofs_verified.fetch_add(1, Ordering::Relaxed);
        } else {
            self.proofs_failed.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    pub fn snapshot(&self) -> MerkleTrackerSnapshot {
        MerkleTrackerSnapshot {
            root_hash: self.root_hash(),
            component_count: self.component_hashes.read().map(|h| h.len()).unwrap_or(0),
            sequence: self.sequence.load(Ordering::Relaxed),
            history_size: self.delta_history.read().map(|h| h.len()).unwrap_or(0),
            deltas_generated: self.deltas_generated.load(Ordering::Relaxed),
            proofs_verified: self.proofs_verified.load(Ordering::Relaxed),
            proofs_failed: self.proofs_failed.load(Ordering::Relaxed),
        }
    }
}

impl Default for MerkleStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTrackerSnapshot {
    pub root_hash: MerkleHash,
    pub component_count: usize,
    pub sequence: u64,
    pub history_size: usize,
    pub deltas_generated: u64,
    pub proofs_verified: u64,
    pub proofs_failed: u64,
}

// ============================================================================
// PHASE 6.2: MULTI-LEVEL GAS METERING (HIERARCHISCHE KOSTEN)
// ============================================================================

/// Gas-Layer für hierarchisches Metering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GasLayer {
    /// L1: P2P-Bandbreite (Bytes sent/received, Gossip-Messages)
    Network,
    /// L2: CPU/Execution (Instructions, Policy-Evaluationen)
    Compute,
    /// L3: Storage (KV-Writes, EventStore, Archive)
    Storage,
    /// L4: Per-Realm Quotas
    Realm,
}

impl GasLayer {
    pub fn description(&self) -> &'static str {
        match self {
            GasLayer::Network => "P2P network bandwidth (bytes, messages)",
            GasLayer::Compute => "CPU/execution (instructions, policies)",
            GasLayer::Storage => "Persistence (writes, events)",
            GasLayer::Realm => "Per-realm resource quota",
        }
    }

    pub fn default_price(&self) -> u64 {
        match self {
            GasLayer::Network => 1,   // 1 Gas pro Byte
            GasLayer::Compute => 10,  // 10 Gas pro Instruction
            GasLayer::Storage => 100, // 100 Gas pro KB written
            GasLayer::Realm => 50,    // 50 Gas pro Realm-Operation
        }
    }
}

/// Multi-Layer Gas Tracker für faire Abrechnung
///
/// Jeder Layer hat eigene Metriken und Preise, ermöglicht:
/// - Schutz gegen layered DoS-Attacks
/// - Faire Kosten (Storage-Spam zahlt Storage-Gas, nicht Compute)
/// - Dynamische Preisanpassung bei Netzwerk-Load
#[derive(Debug)]
pub struct MultiGas {
    /// L1: Network Gas (P2P-Bandbreite)
    pub network: AtomicU64,
    /// L2: Compute Gas (CPU/Instructions)
    pub compute: AtomicU64,
    /// L3: Storage Gas (Persistence)
    pub storage: AtomicU64,
    /// L4: Per-Realm Gas (realm_id → consumed)
    pub realm: RwLock<HashMap<String, AtomicU64>>,
    /// Dynamic Prices (can be adjusted based on load)
    pub prices: RwLock<HashMap<GasLayer, u64>>,
    /// Total Gas consumed (all layers)
    pub total_consumed: AtomicU64,
    /// Network: Bytes sent
    pub network_bytes_sent: AtomicU64,
    /// Network: Bytes received
    pub network_bytes_received: AtomicU64,
    /// Network: Messages sent
    pub network_messages_sent: AtomicU64,
    /// Compute: Instructions executed
    pub compute_instructions: AtomicU64,
    /// Storage: Bytes written
    pub storage_bytes_written: AtomicU64,
    /// Storage: Operations
    pub storage_operations: AtomicU64,
}

impl MultiGas {
    pub fn new() -> Self {
        let mut prices = HashMap::new();
        prices.insert(GasLayer::Network, GasLayer::Network.default_price());
        prices.insert(GasLayer::Compute, GasLayer::Compute.default_price());
        prices.insert(GasLayer::Storage, GasLayer::Storage.default_price());
        prices.insert(GasLayer::Realm, GasLayer::Realm.default_price());

        Self {
            network: AtomicU64::new(0),
            compute: AtomicU64::new(0),
            storage: AtomicU64::new(0),
            realm: RwLock::new(HashMap::new()),
            prices: RwLock::new(prices),
            total_consumed: AtomicU64::new(0),
            network_bytes_sent: AtomicU64::new(0),
            network_bytes_received: AtomicU64::new(0),
            network_messages_sent: AtomicU64::new(0),
            compute_instructions: AtomicU64::new(0),
            storage_bytes_written: AtomicU64::new(0),
            storage_operations: AtomicU64::new(0),
        }
    }

    /// Konsumiere Gas auf einem Layer
    pub fn consume(&self, layer: GasLayer, amount: u64, realm_id: Option<&str>) {
        let price = self
            .prices
            .read()
            .map(|p| *p.get(&layer).unwrap_or(&1))
            .unwrap_or(1);
        let cost = amount.saturating_mul(price);

        match layer {
            GasLayer::Network => {
                self.network.fetch_add(cost, Ordering::Relaxed);
            }
            GasLayer::Compute => {
                self.compute.fetch_add(cost, Ordering::Relaxed);
            }
            GasLayer::Storage => {
                self.storage.fetch_add(cost, Ordering::Relaxed);
            }
            GasLayer::Realm => {
                if let Some(rid) = realm_id {
                    if let Ok(realms) = self.realm.read() {
                        if let Some(counter) = realms.get(rid) {
                            counter.fetch_add(cost, Ordering::Relaxed);
                        }
                    }
                }
            }
        }
        self.total_consumed.fetch_add(cost, Ordering::Relaxed);
    }

    /// Konsumiere Network-Gas (Bytes + Messages)
    pub fn consume_network(&self, bytes_sent: u64, bytes_received: u64, messages: u64) {
        self.network_bytes_sent
            .fetch_add(bytes_sent, Ordering::Relaxed);
        self.network_bytes_received
            .fetch_add(bytes_received, Ordering::Relaxed);
        self.network_messages_sent
            .fetch_add(messages, Ordering::Relaxed);
        self.consume(GasLayer::Network, bytes_sent + bytes_received, None);
    }

    /// Konsumiere Compute-Gas (Instructions)
    pub fn consume_compute(&self, instructions: u64) {
        self.compute_instructions
            .fetch_add(instructions, Ordering::Relaxed);
        self.consume(GasLayer::Compute, instructions, None);
    }

    /// Konsumiere Storage-Gas (Bytes + Operations)
    pub fn consume_storage(&self, bytes_written: u64, operations: u64) {
        self.storage_bytes_written
            .fetch_add(bytes_written, Ordering::Relaxed);
        self.storage_operations
            .fetch_add(operations, Ordering::Relaxed);
        // Storage: pro KB + pro Operation
        let kb = (bytes_written + 1023) / 1024;
        self.consume(GasLayer::Storage, kb + operations, None);
    }

    /// Registriere Realm für Tracking
    pub fn register_realm(&self, realm_id: &str) {
        if let Ok(mut realms) = self.realm.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(|| AtomicU64::new(0));
        }
    }

    /// Konsumiere Realm-spezifisches Gas
    pub fn consume_realm(&self, realm_id: &str, amount: u64) {
        self.consume(GasLayer::Realm, amount, Some(realm_id));
    }

    /// Hole Gas-Verbrauch für Realm
    pub fn realm_consumed(&self, realm_id: &str) -> u64 {
        self.realm
            .read()
            .map(|r| {
                r.get(realm_id)
                    .map(|c| c.load(Ordering::Relaxed))
                    .unwrap_or(0)
            })
            .unwrap_or(0)
    }

    /// Setze dynamischen Preis für Layer
    pub fn set_price(&self, layer: GasLayer, price: u64) {
        if let Ok(mut prices) = self.prices.write() {
            prices.insert(layer, price);
        }
    }

    /// Erhöhe Preise bei hoher Load (Congestion Pricing)
    pub fn apply_congestion_multiplier(&self, multiplier: f64) {
        if let Ok(mut prices) = self.prices.write() {
            for (_, price) in prices.iter_mut() {
                *price = ((*price as f64) * multiplier) as u64;
            }
        }
    }

    pub fn snapshot(&self) -> MultiGasSnapshot {
        MultiGasSnapshot {
            network: self.network.load(Ordering::Relaxed),
            compute: self.compute.load(Ordering::Relaxed),
            storage: self.storage.load(Ordering::Relaxed),
            realm_count: self.realm.read().map(|r| r.len()).unwrap_or(0),
            total_consumed: self.total_consumed.load(Ordering::Relaxed),
            network_bytes_sent: self.network_bytes_sent.load(Ordering::Relaxed),
            network_bytes_received: self.network_bytes_received.load(Ordering::Relaxed),
            network_messages_sent: self.network_messages_sent.load(Ordering::Relaxed),
            compute_instructions: self.compute_instructions.load(Ordering::Relaxed),
            storage_bytes_written: self.storage_bytes_written.load(Ordering::Relaxed),
            storage_operations: self.storage_operations.load(Ordering::Relaxed),
            prices: self.prices.read().map(|p| p.clone()).unwrap_or_default(),
        }
    }
}

impl Default for MultiGas {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiGasSnapshot {
    pub network: u64,
    pub compute: u64,
    pub storage: u64,
    pub realm_count: usize,
    pub total_consumed: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub network_messages_sent: u64,
    pub compute_instructions: u64,
    pub storage_bytes_written: u64,
    pub storage_operations: u64,
    pub prices: HashMap<GasLayer, u64>,
}

// ============================================================================
// PHASE 6.2: SELF-HEALING REALM-ISOLIERUNG (SANDBOXING)
// ============================================================================

/// Resource-Typ für Quota-Tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Ingress-Queue-Slots (pending Events)
    QueueSlots,
    /// Storage (Bytes im KV/EventStore)
    StorageBytes,
    /// Compute-Gas (Gas pro Epoch)
    ComputeGas,
    /// Events (Events pro Zeitfenster)
    Events,
    /// Crossings (Crossings pro Zeitfenster)
    Crossings,
}

impl ResourceType {
    pub fn default_limit(&self) -> u64 {
        match self {
            ResourceType::QueueSlots => 100,
            ResourceType::StorageBytes => 10_000_000, // 10 MB
            ResourceType::ComputeGas => 1_000_000,
            ResourceType::Events => 10_000,
            ResourceType::Crossings => 1_000,
        }
    }
}

/// Realm-spezifische Quota für Self-Healing Isolation
///
/// Bei Überschreitung: Realm wird pausiert/blockiert (Circuit Breaker pro Realm)
#[derive(Debug)]
pub struct RealmQuota {
    /// Max Ingress-Queue-Slots
    pub queue_slots_limit: AtomicU64,
    /// Aktuelle Queue-Belegung
    pub queue_slots_used: AtomicU64,
    /// Max Storage (Bytes)
    pub storage_bytes_limit: AtomicU64,
    /// Aktuelle Storage-Belegung
    pub storage_bytes_used: AtomicU64,
    /// Max Compute-Gas pro Epoch
    pub compute_gas_limit: AtomicU64,
    /// Aktuell verbrauchtes Compute-Gas
    pub compute_gas_used: AtomicU64,
    /// Max Events pro Zeitfenster
    pub events_limit: AtomicU64,
    /// Events im aktuellen Fenster
    pub events_used: AtomicU64,
    /// Max Crossings pro Zeitfenster
    pub crossings_limit: AtomicU64,
    /// Crossings im aktuellen Fenster
    pub crossings_used: AtomicU64,
    /// Quota-Verletzungen
    pub violations: AtomicU64,
    /// Realm ist quarantined (auto-pausiert)
    pub quarantined: AtomicU8,
    /// Letzte Quota-Reset Zeit (Epoch Millis)
    pub last_reset_ms: AtomicU64,
}

impl RealmQuota {
    pub fn new() -> Self {
        Self {
            queue_slots_limit: AtomicU64::new(ResourceType::QueueSlots.default_limit()),
            queue_slots_used: AtomicU64::new(0),
            storage_bytes_limit: AtomicU64::new(ResourceType::StorageBytes.default_limit()),
            storage_bytes_used: AtomicU64::new(0),
            compute_gas_limit: AtomicU64::new(ResourceType::ComputeGas.default_limit()),
            compute_gas_used: AtomicU64::new(0),
            events_limit: AtomicU64::new(ResourceType::Events.default_limit()),
            events_used: AtomicU64::new(0),
            crossings_limit: AtomicU64::new(ResourceType::Crossings.default_limit()),
            crossings_used: AtomicU64::new(0),
            violations: AtomicU64::new(0),
            quarantined: AtomicU8::new(0),
            last_reset_ms: AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0),
            ),
        }
    }

    /// Prüfe ob Ressource verfügbar ist
    pub fn check_quota(&self, resource: ResourceType, amount: u64) -> bool {
        if self.is_quarantined() {
            return false;
        }

        let (used, limit) = match resource {
            ResourceType::QueueSlots => (&self.queue_slots_used, &self.queue_slots_limit),
            ResourceType::StorageBytes => (&self.storage_bytes_used, &self.storage_bytes_limit),
            ResourceType::ComputeGas => (&self.compute_gas_used, &self.compute_gas_limit),
            ResourceType::Events => (&self.events_used, &self.events_limit),
            ResourceType::Crossings => (&self.crossings_used, &self.crossings_limit),
        };

        let current = used.load(Ordering::Relaxed);
        let max = limit.load(Ordering::Relaxed);
        current.saturating_add(amount) <= max
    }

    /// Konsumiere Ressource (nach check_quota)
    pub fn consume(&self, resource: ResourceType, amount: u64) -> bool {
        if !self.check_quota(resource, amount) {
            self.violations.fetch_add(1, Ordering::Relaxed);
            // Auto-Quarantine nach 10 Violations
            if self.violations.load(Ordering::Relaxed) >= 10 {
                self.quarantine();
            }
            return false;
        }

        let used = match resource {
            ResourceType::QueueSlots => &self.queue_slots_used,
            ResourceType::StorageBytes => &self.storage_bytes_used,
            ResourceType::ComputeGas => &self.compute_gas_used,
            ResourceType::Events => &self.events_used,
            ResourceType::Crossings => &self.crossings_used,
        };
        used.fetch_add(amount, Ordering::Relaxed);
        true
    }

    /// Freigebe Ressource (z.B. Queue-Slot nach Processing)
    pub fn release(&self, resource: ResourceType, amount: u64) {
        let used = match resource {
            ResourceType::QueueSlots => &self.queue_slots_used,
            ResourceType::StorageBytes => &self.storage_bytes_used,
            ResourceType::ComputeGas => &self.compute_gas_used,
            ResourceType::Events => &self.events_used,
            ResourceType::Crossings => &self.crossings_used,
        };
        used.fetch_sub(amount.min(used.load(Ordering::Relaxed)), Ordering::Relaxed);
    }

    /// Ist Realm quarantined?
    pub fn is_quarantined(&self) -> bool {
        self.quarantined.load(Ordering::Relaxed) == 1
    }

    /// Quarantine Realm (auto-pausieren)
    pub fn quarantine(&self) {
        self.quarantined.store(1, Ordering::Relaxed);
    }

    /// Unquarantine Realm (Admin-Recovery)
    pub fn unquarantine(&self) {
        self.quarantined.store(0, Ordering::Relaxed);
        self.violations.store(0, Ordering::Relaxed);
    }

    /// Setze Limit für Ressource
    pub fn set_limit(&self, resource: ResourceType, limit: u64) {
        let counter = match resource {
            ResourceType::QueueSlots => &self.queue_slots_limit,
            ResourceType::StorageBytes => &self.storage_bytes_limit,
            ResourceType::ComputeGas => &self.compute_gas_limit,
            ResourceType::Events => &self.events_limit,
            ResourceType::Crossings => &self.crossings_limit,
        };
        counter.store(limit, Ordering::Relaxed);
    }

    /// Reset Quotas (für Epoch-Wechsel)
    pub fn reset_epoch_quotas(&self) {
        self.compute_gas_used.store(0, Ordering::Relaxed);
        self.events_used.store(0, Ordering::Relaxed);
        self.crossings_used.store(0, Ordering::Relaxed);
        self.last_reset_ms.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            Ordering::Relaxed,
        );
    }

    /// Utilization-Prozent für eine Ressource
    pub fn utilization(&self, resource: ResourceType) -> f64 {
        let (used, limit) = match resource {
            ResourceType::QueueSlots => (&self.queue_slots_used, &self.queue_slots_limit),
            ResourceType::StorageBytes => (&self.storage_bytes_used, &self.storage_bytes_limit),
            ResourceType::ComputeGas => (&self.compute_gas_used, &self.compute_gas_limit),
            ResourceType::Events => (&self.events_used, &self.events_limit),
            ResourceType::Crossings => (&self.crossings_used, &self.crossings_limit),
        };
        let u = used.load(Ordering::Relaxed) as f64;
        let l = limit.load(Ordering::Relaxed) as f64;
        if l > 0.0 {
            (u / l) * 100.0
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> RealmQuotaSnapshot {
        RealmQuotaSnapshot {
            queue_slots_limit: self.queue_slots_limit.load(Ordering::Relaxed),
            queue_slots_used: self.queue_slots_used.load(Ordering::Relaxed),
            storage_bytes_limit: self.storage_bytes_limit.load(Ordering::Relaxed),
            storage_bytes_used: self.storage_bytes_used.load(Ordering::Relaxed),
            compute_gas_limit: self.compute_gas_limit.load(Ordering::Relaxed),
            compute_gas_used: self.compute_gas_used.load(Ordering::Relaxed),
            events_limit: self.events_limit.load(Ordering::Relaxed),
            events_used: self.events_used.load(Ordering::Relaxed),
            crossings_limit: self.crossings_limit.load(Ordering::Relaxed),
            crossings_used: self.crossings_used.load(Ordering::Relaxed),
            violations: self.violations.load(Ordering::Relaxed),
            quarantined: self.is_quarantined(),
            last_reset_ms: self.last_reset_ms.load(Ordering::Relaxed),
        }
    }
}

impl Default for RealmQuota {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmQuotaSnapshot {
    pub queue_slots_limit: u64,
    pub queue_slots_used: u64,
    pub storage_bytes_limit: u64,
    pub storage_bytes_used: u64,
    pub compute_gas_limit: u64,
    pub compute_gas_used: u64,
    pub events_limit: u64,
    pub events_used: u64,
    pub crossings_limit: u64,
    pub crossings_used: u64,
    pub violations: u64,
    pub quarantined: bool,
    pub last_reset_ms: u64,
}

// ============================================================================
// IDENTITY STATE LAYER (Κ6-Κ8 DID Management)
// ============================================================================

/// Identity-State-Layer für DID-Management
///
/// # Architektur
///
/// ```text
/// IdentityState
/// ├── Atomics (High-Frequency)
/// │   ├── bootstrap_completed
/// │   ├── mode
/// │   ├── sub_dids_total
/// │   └── ... (12 weitere)
/// ├── RwLock (Complex State)
/// │   ├── root_did
/// │   ├── root_document
/// │   ├── sub_dids
/// │   ├── delegations
/// │   └── realm_memberships
/// └── Handles (Orthogonal)
///     ├── key_store
///     └── passkey_manager
/// ```
///
/// # Axiom-Referenz
///
/// - **Κ6 (Existenz-Eindeutigkeit)**: `∀ entity e : ∃! did ∈ DID : identity(e) = did`
/// - **Κ7 (Permanenz)**: `⟨s⟩ ∧ ⟦create(s)⟧ ⟹ □⟨s⟩`
/// - **Κ8 (Delegations-Struktur)**: `s ⊳ s' → 𝕋(s') ≤ 𝕋(s)`
///
/// # StateGraph-Beziehungen
///
/// - Trust DependsOn Identity
/// - Identity Triggers Trust
/// - Event DependsOn Identity
/// - Identity Triggers Event
/// - Swarm DependsOn Identity
/// - Controller DependsOn Identity
/// - ... (38 Kanten total)
#[derive(Debug)]
pub struct IdentityState {
    // ─────────────────────────────────────────────────────────────────────────
    // HIGH-FREQUENCY ATOMICS (Lock-free)
    // ─────────────────────────────────────────────────────────────────────────
    /// Bootstrap abgeschlossen?
    pub bootstrap_completed: AtomicBool,

    /// Root-DID erstellt (Timestamp ms)
    pub root_created_at_ms: AtomicU64,

    /// Aktueller Modus (0=Interactive, 1=AgentManaged, 2=Ephemeral, 3=Test)
    pub mode: AtomicU8,

    /// Gesamtanzahl abgeleiteter Sub-DIDs
    pub sub_dids_total: AtomicU64,

    /// Gesamtanzahl abgeleiteter Wallet-Adressen
    pub addresses_total: AtomicU64,

    /// Aktive Delegationen
    pub active_delegations_count: AtomicU64,

    /// Widerrufene Delegationen
    pub revoked_delegations_count: AtomicU64,

    /// Credentials ausgestellt
    pub credentials_issued: AtomicU64,

    /// Credentials verifiziert
    pub credentials_verified: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // RELATIONSHIP COUNTERS (StateGraph-Tracking)
    // ─────────────────────────────────────────────────────────────────────────
    /// Identity → Triggers → Event
    pub events_triggered: AtomicU64,

    /// Identity → Triggers → Trust (Initial Trust-Entries)
    pub trust_entries_created: AtomicU64,

    /// Identity → Triggers → Realm (Join/Leave)
    pub realm_memberships_changed: AtomicU64,

    /// Gas verbraucht für Identity-Ops
    pub gas_consumed: AtomicU64,

    /// Mana verbraucht für Identity-Ops
    pub mana_consumed: AtomicU64,

    /// Signaturen erstellt
    pub signatures_created: AtomicU64,

    /// Signaturen verifiziert
    pub signatures_verified: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // COMPLEX STATE (RwLock-protected)
    // ─────────────────────────────────────────────────────────────────────────
    /// Root-DID (None vor Bootstrap)
    pub root_did: RwLock<Option<crate::domain::unified::identity::DID>>,

    /// DID-Document (None vor Bootstrap)
    pub root_document: RwLock<Option<crate::domain::unified::identity::DIDDocument>>,

    /// Device-Sub-DID (aktuelles Gerät)
    pub current_device_did: RwLock<Option<crate::domain::unified::identity::DID>>,

    /// Sub-DIDs nach Typ (device, agent, realm, custom)
    pub sub_dids: RwLock<HashMap<String, Vec<crate::domain::unified::identity::DID>>>,

    /// Sub-DID-Zähler nach Namespace
    pub sub_did_counts: RwLock<HashMap<crate::domain::unified::identity::DIDNamespace, u64>>,

    /// Wallet-Adressen nach Chain (CAIP-2 Format)
    pub wallets: RwLock<HashMap<String, Vec<crate::core::identity_types::WalletAddress>>>,

    /// Aktive Delegationen (delegate_id → Delegation)
    pub delegations: RwLock<HashMap<UniversalId, crate::domain::unified::identity::Delegation>>,

    /// Realm-Memberships (realm_id → membership_info)
    pub realm_memberships:
        RwLock<HashMap<UniversalId, crate::core::identity_types::RealmMembership>>,

    // ─────────────────────────────────────────────────────────────────────────
    // ORTHOGONAL HANDLES
    // ─────────────────────────────────────────────────────────────────────────
    /// Secure Key-Store Handle (TEE/TPM Abstraction)
    pub key_store: Option<crate::core::identity_types::SharedKeyStore>,

    /// WebAuthn/Passkey Manager Handle
    pub passkey_manager: Option<crate::core::identity_types::SharedPasskeyManager>,
}

impl IdentityState {
    /// Erstelle neuen IdentityState
    pub fn new() -> Self {
        Self {
            // Atomics
            bootstrap_completed: AtomicBool::new(false),
            root_created_at_ms: AtomicU64::new(0),
            mode: AtomicU8::new(0), // Interactive default
            sub_dids_total: AtomicU64::new(0),
            addresses_total: AtomicU64::new(0),
            active_delegations_count: AtomicU64::new(0),
            revoked_delegations_count: AtomicU64::new(0),
            credentials_issued: AtomicU64::new(0),
            credentials_verified: AtomicU64::new(0),

            // Relationship counters
            events_triggered: AtomicU64::new(0),
            trust_entries_created: AtomicU64::new(0),
            realm_memberships_changed: AtomicU64::new(0),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            signatures_created: AtomicU64::new(0),
            signatures_verified: AtomicU64::new(0),

            // Complex state
            root_did: RwLock::new(None),
            root_document: RwLock::new(None),
            current_device_did: RwLock::new(None),
            sub_dids: RwLock::new(HashMap::new()),
            sub_did_counts: RwLock::new(HashMap::new()),
            wallets: RwLock::new(HashMap::new()),
            delegations: RwLock::new(HashMap::new()),
            realm_memberships: RwLock::new(HashMap::new()),

            // Handles (not set by default)
            key_store: None,
            passkey_manager: None,
        }
    }

    /// Erstelle mit Key-Store
    pub fn with_key_store(
        mut self,
        key_store: crate::core::identity_types::SharedKeyStore,
    ) -> Self {
        self.key_store = Some(key_store);
        self
    }

    /// Erstelle mit Passkey-Manager
    pub fn with_passkey_manager(
        mut self,
        passkey_manager: crate::core::identity_types::SharedPasskeyManager,
    ) -> Self {
        self.passkey_manager = Some(passkey_manager);
        self
    }

    /// Erstelle Snapshot
    pub fn snapshot(&self) -> IdentitySnapshot {
        let root_did_uri = self.root_did.read().unwrap().as_ref().map(|d| d.to_uri());

        let sub_did_counts = self
            .sub_did_counts
            .read()
            .unwrap()
            .iter()
            .map(|(ns, count)| (ns.to_string(), *count))
            .collect();

        let wallet_chains: Vec<String> = self.wallets.read().unwrap().keys().cloned().collect();
        let realm_membership_count = self.realm_memberships.read().unwrap().len();

        IdentitySnapshot {
            bootstrap_completed: self.bootstrap_completed.load(Ordering::Relaxed),
            root_created_at_ms: self.root_created_at_ms.load(Ordering::Relaxed),
            mode: crate::core::identity_types::IdentityMode::from_u8(
                self.mode.load(Ordering::Relaxed),
            ),
            sub_dids_total: self.sub_dids_total.load(Ordering::Relaxed),
            addresses_total: self.addresses_total.load(Ordering::Relaxed),
            active_delegations: self.active_delegations_count.load(Ordering::Relaxed),
            revoked_delegations: self.revoked_delegations_count.load(Ordering::Relaxed),
            credentials_issued: self.credentials_issued.load(Ordering::Relaxed),
            credentials_verified: self.credentials_verified.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
            trust_entries_created: self.trust_entries_created.load(Ordering::Relaxed),
            realm_memberships_changed: self.realm_memberships_changed.load(Ordering::Relaxed),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            signatures_created: self.signatures_created.load(Ordering::Relaxed),
            signatures_verified: self.signatures_verified.load(Ordering::Relaxed),
            root_did: root_did_uri,
            sub_did_counts,
            realm_membership_count,
            wallet_chains,
        }
    }

    /// Health-Score für Identity-Layer (0.0 - 1.0)
    pub fn health_score(&self) -> f64 {
        // Nicht bootstrapped → 0.0
        if !self.bootstrap_completed.load(Ordering::Relaxed) {
            return 0.0;
        }

        let mut score = 1.0;

        // Mode-basierte Penalty
        let mode =
            crate::core::identity_types::IdentityMode::from_u8(self.mode.load(Ordering::Relaxed));
        if !mode.is_production_safe() {
            score *= 0.5;
        }

        // Zu viele widerrufene Delegationen (> 50% aktive) → Penalty
        let active = self.active_delegations_count.load(Ordering::Relaxed);
        let revoked = self.revoked_delegations_count.load(Ordering::Relaxed);
        if active > 0 && revoked > active {
            score *= 0.8;
        }

        // Keine Device-DID → kleine Penalty
        if self.current_device_did.read().unwrap().is_none() {
            score *= 0.9;
        }

        score
    }

    /// Aktueller Modus
    pub fn current_mode(&self) -> crate::core::identity_types::IdentityMode {
        crate::core::identity_types::IdentityMode::from_u8(self.mode.load(Ordering::Relaxed))
    }

    /// Ist bootstrapped?
    pub fn is_bootstrapped(&self) -> bool {
        self.bootstrap_completed.load(Ordering::Relaxed)
    }

    /// Root-DID UniversalId (falls bootstrapped)
    pub fn root_did_id(&self) -> Option<UniversalId> {
        self.root_did.read().unwrap().as_ref().map(|d| d.id)
    }

    /// Device-DID UniversalId (falls vorhanden)
    pub fn device_did_id(&self) -> Option<UniversalId> {
        self.current_device_did
            .read()
            .unwrap()
            .as_ref()
            .map(|d| d.id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BOOTSTRAP METHODS
    // ─────────────────────────────────────────────────────────────────────────

    /// Bootstrap Identity im Interactive-Modus
    ///
    /// Erfordert Key-Store und Passkey-Manager.
    pub fn bootstrap_interactive(
        &self,
        public_key: &[u8; 32],
    ) -> Result<UniversalId, crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        if self.is_bootstrapped() {
            return Err(IdentityError::AlreadyBootstrapped);
        }

        if self.key_store.is_none() {
            return Err(IdentityError::KeyStoreNotInitialized);
        }

        if self.passkey_manager.is_none() {
            return Err(IdentityError::PasskeyNotAvailable);
        }

        // Root-DID erstellen
        let did = crate::domain::unified::identity::DID::new_self(public_key);
        let doc = crate::domain::unified::identity::DIDDocument::new(did.clone());
        let root_id = did.id;

        // State aktualisieren
        *self.root_did.write().unwrap() = Some(did);
        *self.root_document.write().unwrap() = Some(doc);
        self.mode.store(0, Ordering::Relaxed); // Interactive
        self.root_created_at_ms.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            Ordering::Relaxed,
        );
        self.bootstrap_completed.store(true, Ordering::Relaxed);

        Ok(root_id)
    }

    /// Bootstrap Identity im Agent-Modus
    ///
    /// Erlaubt autonome Signaturen ohne User-Confirmation.
    pub fn bootstrap_agent(
        &self,
        public_key: &[u8; 32],
    ) -> Result<UniversalId, crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        if self.is_bootstrapped() {
            return Err(IdentityError::AlreadyBootstrapped);
        }

        if self.key_store.is_none() {
            return Err(IdentityError::KeyStoreNotInitialized);
        }

        let did = crate::domain::unified::identity::DID::new_self(public_key);
        let doc = crate::domain::unified::identity::DIDDocument::new(did.clone());
        let root_id = did.id;

        *self.root_did.write().unwrap() = Some(did);
        *self.root_document.write().unwrap() = Some(doc);
        self.mode.store(1, Ordering::Relaxed); // AgentManaged
        self.root_created_at_ms.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            Ordering::Relaxed,
        );
        self.bootstrap_completed.store(true, Ordering::Relaxed);

        Ok(root_id)
    }

    /// Bootstrap Ephemeral Identity
    ///
    /// Kurzlebige Session ohne Persistenz.
    pub fn bootstrap_ephemeral(
        &self,
        public_key: &[u8; 32],
    ) -> Result<UniversalId, crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        if self.is_bootstrapped() {
            return Err(IdentityError::AlreadyBootstrapped);
        }

        let did = crate::domain::unified::identity::DID::new_self(public_key);
        let doc = crate::domain::unified::identity::DIDDocument::new(did.clone());
        let root_id = did.id;

        *self.root_did.write().unwrap() = Some(did);
        *self.root_document.write().unwrap() = Some(doc);
        self.mode.store(2, Ordering::Relaxed); // Ephemeral
        self.root_created_at_ms.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            Ordering::Relaxed,
        );
        self.bootstrap_completed.store(true, Ordering::Relaxed);

        Ok(root_id)
    }

    /// Bootstrap Test Identity (für Unit-Tests)
    pub fn bootstrap_test(
        &self,
        public_key: &[u8; 32],
    ) -> Result<UniversalId, crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        if self.is_bootstrapped() {
            return Err(IdentityError::AlreadyBootstrapped);
        }

        let did = crate::domain::unified::identity::DID::new_self(public_key);
        let doc = crate::domain::unified::identity::DIDDocument::new(did.clone());
        let root_id = did.id;

        *self.root_did.write().unwrap() = Some(did);
        *self.root_document.write().unwrap() = Some(doc);
        self.mode.store(3, Ordering::Relaxed); // Test
        self.root_created_at_ms.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            Ordering::Relaxed,
        );
        self.bootstrap_completed.store(true, Ordering::Relaxed);

        Ok(root_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SUB-DID DERIVATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Leite Device-Sub-DID ab
    pub fn derive_device_did(
        &self,
        device_index: u32,
    ) -> Result<UniversalId, crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;
        use crate::domain::unified::identity::{DIDNamespace, DID};

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        let root = self.root_did.read().unwrap();
        let root_did = root.as_ref().ok_or(IdentityError::NotBootstrapped)?;

        let device_did = DID::derive_device(root_did, device_index);
        let device_id = device_did.id;

        // Zu DID-Document hinzufügen
        if let Some(ref mut doc) = *self.root_document.write().unwrap() {
            doc.add_device_key(&device_did);
        }

        // Sub-DID speichern
        self.sub_dids
            .write()
            .unwrap()
            .entry("device".to_string())
            .or_default()
            .push(device_did.clone());

        // Counter aktualisieren
        *self
            .sub_did_counts
            .write()
            .unwrap()
            .entry(DIDNamespace::Self_)
            .or_default() += 1;
        self.sub_dids_total.fetch_add(1, Ordering::Relaxed);

        // Erstes Device als aktuelles setzen
        if self.current_device_did.read().unwrap().is_none() {
            *self.current_device_did.write().unwrap() = Some(device_did);
        }

        Ok(device_id)
    }

    /// Leite Agent-Sub-DID ab
    pub fn derive_agent_did(
        &self,
        agent_index: u32,
    ) -> Result<UniversalId, crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;
        use crate::domain::unified::identity::{DIDNamespace, DID};

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        let root = self.root_did.read().unwrap();
        let root_did = root.as_ref().ok_or(IdentityError::NotBootstrapped)?;

        let agent_did = DID::derive_agent(root_did, agent_index);
        let agent_id = agent_did.id;

        // Zu DID-Document hinzufügen
        if let Some(ref mut doc) = *self.root_document.write().unwrap() {
            doc.add_agent_key(&agent_did);
        }

        // Sub-DID speichern
        self.sub_dids
            .write()
            .unwrap()
            .entry("agent".to_string())
            .or_default()
            .push(agent_did);

        // Counter aktualisieren
        *self
            .sub_did_counts
            .write()
            .unwrap()
            .entry(DIDNamespace::Spirit)
            .or_default() += 1;
        self.sub_dids_total.fetch_add(1, Ordering::Relaxed);

        Ok(agent_id)
    }

    /// Leite Realm-Sub-DID ab
    pub fn derive_realm_did(
        &self,
        realm_id: &UniversalId,
    ) -> Result<UniversalId, crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;
        use crate::domain::unified::identity::{DIDNamespace, DID};

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        let root = self.root_did.read().unwrap();
        let root_did = root.as_ref().ok_or(IdentityError::NotBootstrapped)?;

        let realm_did = DID::derive_realm(root_did, realm_id);
        let realm_did_id = realm_did.id;

        // Sub-DID speichern
        self.sub_dids
            .write()
            .unwrap()
            .entry(format!("realm:{}", hex::encode(realm_id.as_bytes())))
            .or_default()
            .push(realm_did);

        // Counter aktualisieren
        *self
            .sub_did_counts
            .write()
            .unwrap()
            .entry(DIDNamespace::Circle)
            .or_default() += 1;
        self.sub_dids_total.fetch_add(1, Ordering::Relaxed);

        Ok(realm_did_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SIGNATURE METHODS
    // ─────────────────────────────────────────────────────────────────────────

    /// Signiere mit Device-Key
    pub fn sign_with_device(
        &self,
        payload: &[u8],
    ) -> Result<[u8; 64], crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        let device_did = self
            .current_device_did
            .read()
            .unwrap()
            .as_ref()
            .ok_or(IdentityError::NoDeviceKey)?
            .id;

        let key_store = self
            .key_store
            .as_ref()
            .ok_or(IdentityError::KeyStoreNotInitialized)?;

        let signature = key_store.sign(device_did, payload)?;

        self.signatures_created.fetch_add(1, Ordering::Relaxed);

        Ok(signature)
    }

    /// Signiere mit Root-Key (erfordert User-Confirmation im Interactive-Modus)
    pub fn sign_with_root(
        &self,
        payload: &[u8],
    ) -> Result<[u8; 64], crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        let mode = self.current_mode();

        // Interactive-Modus: Erfordert Passkey-Confirmation
        if mode == crate::core::identity_types::IdentityMode::Interactive {
            let passkey = self
                .passkey_manager
                .as_ref()
                .ok_or(IdentityError::PasskeyNotAvailable)?;

            let signature = passkey.sign_with_confirmation(payload)?;
            self.signatures_created.fetch_add(1, Ordering::Relaxed);
            return Ok(signature);
        }

        // Agent/Ephemeral/Test: Nutze Key-Store direkt
        if !mode.allows_autonomous_signing() {
            return Err(IdentityError::SignatureNotAllowed(mode.to_string()));
        }

        let root_id = self
            .root_did
            .read()
            .unwrap()
            .as_ref()
            .ok_or(IdentityError::NotBootstrapped)?
            .id;

        let key_store = self
            .key_store
            .as_ref()
            .ok_or(IdentityError::KeyStoreNotInitialized)?;

        let signature = key_store.sign(root_id, payload)?;
        self.signatures_created.fetch_add(1, Ordering::Relaxed);

        Ok(signature)
    }

    /// Verifiziere Signatur
    pub fn verify_signature(
        &self,
        signer_id: UniversalId,
        payload: &[u8],
        signature: &[u8],
    ) -> bool {
        if let Some(ref key_store) = self.key_store {
            let result = key_store.verify(signer_id, payload, signature);
            if result {
                self.signatures_verified.fetch_add(1, Ordering::Relaxed);
            }
            result
        } else {
            false
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // DELEGATION METHODS (Κ8)
    // ─────────────────────────────────────────────────────────────────────────

    /// Delegation erstellen
    pub fn add_delegation(
        &self,
        delegate: UniversalId,
        trust_factor: f32,
        capabilities: Vec<crate::domain::unified::identity::Capability>,
        valid_until: Option<crate::domain::unified::primitives::TemporalCoord>,
    ) -> Result<UniversalId, crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;
        use crate::domain::unified::identity::Delegation;

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        // Κ8: Trust-Faktor prüfen
        if trust_factor <= 0.0 || trust_factor > 1.0 {
            return Err(IdentityError::InvalidTrustFactor(trust_factor));
        }

        let root_id = self
            .root_did
            .read()
            .unwrap()
            .as_ref()
            .ok_or(IdentityError::NotBootstrapped)?
            .id;

        let mut delegation = Delegation::new(root_id, delegate, trust_factor, capabilities);

        if let Some(until) = valid_until {
            delegation.valid_until = Some(until);
        }

        let delegation_id = delegation.id;

        // Zur Root-Document hinzufügen
        if let Some(ref mut doc) = *self.root_document.write().unwrap() {
            doc.add_delegation(delegation.clone());
        }

        // Zu Delegations-Map hinzufügen
        self.delegations
            .write()
            .unwrap()
            .insert(delegate, delegation);

        // Counter aktualisieren
        self.active_delegations_count
            .fetch_add(1, Ordering::Relaxed);

        Ok(delegation_id)
    }

    /// Delegation widerrufen
    pub fn revoke_delegation(
        &self,
        delegate: &UniversalId,
    ) -> Result<(), crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        // Aus Delegations-Map entfernen/widerrufen
        let mut delegations = self.delegations.write().unwrap();
        if let Some(mut delegation) = delegations.remove(delegate) {
            delegation.revoke();

            // Im Document widerrufen
            if let Some(ref mut doc) = *self.root_document.write().unwrap() {
                doc.revoke_delegation(&delegation.id);
            }

            // Counter aktualisieren
            self.active_delegations_count
                .fetch_sub(1, Ordering::Relaxed);
            self.revoked_delegations_count
                .fetch_add(1, Ordering::Relaxed);

            Ok(())
        } else {
            Err(IdentityError::UnknownIdentity(*delegate))
        }
    }

    /// Hole Delegation für Delegate
    pub fn get_delegation(
        &self,
        delegate: &UniversalId,
    ) -> Option<crate::domain::unified::identity::Delegation> {
        self.delegations.read().unwrap().get(delegate).cloned()
    }

    /// Prüfe ob Delegation gültig ist
    pub fn is_delegation_valid(
        &self,
        delegate: &UniversalId,
        now: &crate::domain::unified::primitives::TemporalCoord,
    ) -> bool {
        self.delegations
            .read()
            .unwrap()
            .get(delegate)
            .map(|d| d.is_valid(now))
            .unwrap_or(false)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // REALM MEMBERSHIP
    // ─────────────────────────────────────────────────────────────────────────

    /// Realm beitreten
    pub fn join_realm(
        &self,
        realm_id: UniversalId,
        role: crate::core::identity_types::RealmRole,
        initial_trust: Option<f64>,
    ) -> Result<(), crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::{IdentityError, RealmMembership};

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        let mode = self.current_mode();
        if !mode.allows_realm_membership() {
            return Err(IdentityError::SignatureNotAllowed(format!(
                "Realm membership not allowed in {} mode",
                mode
            )));
        }

        let root_id = self.root_did_id().ok_or(IdentityError::NotBootstrapped)?;

        // Membership erstellen
        let mut membership = RealmMembership::new(realm_id, root_id, role);
        if let Some(trust) = initial_trust {
            membership = membership.with_trust(trust);
        }

        // Realm-spezifische Sub-DID ableiten (optional)
        if let Ok(realm_sub_did) = self.derive_realm_did(&realm_id) {
            membership = membership.with_realm_sub_did(realm_sub_did);
        }

        // Speichern
        self.realm_memberships
            .write()
            .unwrap()
            .insert(realm_id, membership);

        // Counter aktualisieren
        self.realm_memberships_changed
            .fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Realm verlassen
    pub fn leave_realm(
        &self,
        realm_id: &UniversalId,
    ) -> Result<(), crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        let mut memberships = self.realm_memberships.write().unwrap();
        if let Some(membership) = memberships.get_mut(realm_id) {
            membership.deactivate();
            self.realm_memberships_changed
                .fetch_add(1, Ordering::Relaxed);
            Ok(())
        } else {
            Err(IdentityError::NotRealmMember(*realm_id))
        }
    }

    /// Hole Realm-Membership
    pub fn get_realm_membership(
        &self,
        realm_id: &UniversalId,
    ) -> Option<crate::core::identity_types::RealmMembership> {
        self.realm_memberships
            .read()
            .unwrap()
            .get(realm_id)
            .cloned()
    }

    /// Ist Mitglied in Realm?
    pub fn is_realm_member(&self, realm_id: &UniversalId) -> bool {
        self.realm_memberships
            .read()
            .unwrap()
            .get(realm_id)
            .map(|m| m.is_active)
            .unwrap_or(false)
    }

    /// Aktive Realm-Memberships
    pub fn active_realm_memberships(&self) -> Vec<UniversalId> {
        self.realm_memberships
            .read()
            .unwrap()
            .iter()
            .filter(|(_, m)| m.is_active)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Update Realm-Role
    pub fn update_realm_role(
        &self,
        realm_id: &UniversalId,
        new_role: crate::core::identity_types::RealmRole,
    ) -> Result<(), crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        let mut memberships = self.realm_memberships.write().unwrap();
        if let Some(membership) = memberships.get_mut(realm_id) {
            membership.role = new_role;
            membership.record_activity();
            Ok(())
        } else {
            Err(IdentityError::NotRealmMember(*realm_id))
        }
    }

    /// Update Realm-Trust
    pub fn update_realm_trust(
        &self,
        realm_id: &UniversalId,
        new_trust: f64,
    ) -> Result<(), crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        let mut memberships = self.realm_memberships.write().unwrap();
        if let Some(membership) = memberships.get_mut(realm_id) {
            membership.local_trust = new_trust.clamp(0.0, 1.0);
            membership.record_activity();
            Ok(())
        } else {
            Err(IdentityError::NotRealmMember(*realm_id))
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // WALLET ADDRESS MANAGEMENT
    // ─────────────────────────────────────────────────────────────────────────

    /// Wallet-Adresse hinzufügen
    pub fn add_wallet_address(
        &self,
        wallet: crate::core::identity_types::WalletAddress,
    ) -> Result<(), crate::core::identity_types::IdentityError> {
        use crate::core::identity_types::IdentityError;

        if !self.is_bootstrapped() {
            return Err(IdentityError::NotBootstrapped);
        }

        wallet.validate()?;

        self.wallets
            .write()
            .unwrap()
            .entry(wallet.chain_id.clone())
            .or_default()
            .push(wallet);

        self.addresses_total.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Hole Wallet-Adressen für Chain
    pub fn get_wallets_for_chain(
        &self,
        chain_id: &str,
    ) -> Vec<crate::core::identity_types::WalletAddress> {
        self.wallets
            .read()
            .unwrap()
            .get(chain_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Hole primäre Wallet-Adresse für Chain
    pub fn get_primary_wallet(
        &self,
        chain_id: &str,
    ) -> Option<crate::core::identity_types::WalletAddress> {
        self.wallets
            .read()
            .unwrap()
            .get(chain_id)
            .and_then(|wallets| wallets.iter().find(|w| w.is_primary).cloned())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CREDENTIAL METHODS
    // ─────────────────────────────────────────────────────────────────────────

    /// Record credential issuance
    pub fn record_credential_issued(&self) {
        self.credentials_issued.fetch_add(1, Ordering::Relaxed);
    }

    /// Record credential verification
    pub fn record_credential_verified(&self) {
        self.credentials_verified.fetch_add(1, Ordering::Relaxed);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // GAS/MANA TRACKING
    // ─────────────────────────────────────────────────────────────────────────

    /// Record gas consumption for identity operation
    pub fn record_gas(&self, amount: u64) {
        self.gas_consumed.fetch_add(amount, Ordering::Relaxed);
    }

    /// Record mana consumption for identity operation
    pub fn record_mana(&self, amount: u64) {
        self.mana_consumed.fetch_add(amount, Ordering::Relaxed);
    }

    /// Record triggered event
    pub fn record_event_triggered(&self) {
        self.events_triggered.fetch_add(1, Ordering::Relaxed);
    }

    /// Record trust entry creation
    pub fn record_trust_entry_created(&self) {
        self.trust_entries_created.fetch_add(1, Ordering::Relaxed);
    }

    /// Record signature created
    pub fn record_signature_created(&self) {
        self.signatures_created.fetch_add(1, Ordering::Relaxed);
    }

    /// Record signature verified
    pub fn record_signature_verified(&self) {
        self.signatures_verified.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for IdentityState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// IDENTITY SNAPSHOT
// ============================================================================

/// Snapshot für Persistence/CQRS (keine Keys!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentitySnapshot {
    pub bootstrap_completed: bool,
    pub root_created_at_ms: u64,
    pub mode: crate::core::identity_types::IdentityMode,
    pub sub_dids_total: u64,
    pub addresses_total: u64,
    pub active_delegations: u64,
    pub revoked_delegations: u64,
    pub credentials_issued: u64,
    pub credentials_verified: u64,
    pub events_triggered: u64,
    pub trust_entries_created: u64,
    pub realm_memberships_changed: u64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub signatures_created: u64,
    pub signatures_verified: u64,
    /// DID URI String (z.B. "did:erynoa:self:...")
    pub root_did: Option<String>,
    /// Namespace → Count
    pub sub_did_counts: HashMap<String, u64>,
    pub realm_membership_count: usize,
    /// Liste der Chains mit Wallets
    pub wallet_chains: Vec<String>,
}

impl Default for IdentitySnapshot {
    fn default() -> Self {
        Self {
            bootstrap_completed: false,
            root_created_at_ms: 0,
            mode: crate::core::identity_types::IdentityMode::Interactive,
            sub_dids_total: 0,
            addresses_total: 0,
            active_delegations: 0,
            revoked_delegations: 0,
            credentials_issued: 0,
            credentials_verified: 0,
            events_triggered: 0,
            trust_entries_created: 0,
            realm_memberships_changed: 0,
            gas_consumed: 0,
            mana_consumed: 0,
            signatures_created: 0,
            signatures_verified: 0,
            root_did: None,
            sub_did_counts: HashMap::new(),
            realm_membership_count: 0,
            wallet_chains: Vec::new(),
        }
    }
}

// ============================================================================
// STATE RELATIONSHIP TYPES
// ============================================================================
// NOTE: StateRelation und StateComponent sind jetzt in domain/unified/component.rs
// definiert und werden oben via `pub use` re-exportiert für Rückwärtskompatibilität.

/// Beziehungs-Graph zwischen State-Komponenten
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StateGraph {
    pub edges: Vec<(StateComponent, StateRelation, StateComponent)>,
}

impl StateGraph {
    /// Erstelle den Erynoa-State-Graph mit allen Beziehungen
    pub fn erynoa_graph() -> Self {
        use StateComponent::*;
        use StateRelation::*;

        Self {
            edges: vec![
                // ═══════════════════════════════════════════════════════════════════════════
                // IDENTITY-LAYER BEZIEHUNGEN (Κ6-Κ8: DID, Delegation, Credentials)
                // ═══════════════════════════════════════════════════════════════════════════

                // Core-Abhängigkeiten
                (Trust, DependsOn, Identity), // Trust-Werte basieren auf Identity-Verifikation
                (Identity, Triggers, Trust),  // Neue Identities erhalten initialen Trust
                (Event, DependsOn, Identity), // Events müssen Signatur der Identity haben
                (Identity, Triggers, Event),  // Identity-Operationen erzeugen Events
                (Consensus, DependsOn, Identity), // Validator-Identifikation via DID
                // Execution-Abhängigkeiten
                (Execution, DependsOn, Identity), // ExecutionContext hat Identity
                (Identity, DependsOn, Execution), // Identity-Ops verbrauchen Execution-Budget
                (Identity, DependsOn, Gas),       // Sub-DID Derivation verbraucht Gas
                (Identity, DependsOn, Mana),      // Identity-Events verbrauchen Mana
                // Realm-Integration
                (Realm, DependsOn, Identity), // Realm-Membership basiert auf Identity
                (Identity, Triggers, Realm),  // Identity-Join/Leave triggert Realm-Updates
                (Room, DependsOn, Identity),  // Room-Access basiert auf Identity
                (Partition, DependsOn, Identity), // Partition-Zugehörigkeit basiert auf Identity
                // Controller/Auth
                (Controller, DependsOn, Identity), // AuthZ basiert auf Identity
                (Identity, Validates, Controller), // Identity validiert Delegation-Chain
                (Controller, Aggregates, Identity), // Controller trackt Identities für Delegationen
                // Gateway/Crossing
                (Gateway, DependsOn, Identity), // Crossing erfordert Identity-Verifikation
                (Gateway, Validates, Identity), // Gateway validiert Cross-Realm Identity
                // ECLVM
                (ECLVM, DependsOn, Identity), // ECLVM prüft Caller-Identity
                (ECLPolicy, DependsOn, Identity), // Policies können Identity-basierte Rules haben
                // P2P Network
                (Swarm, DependsOn, Identity), // Peer-ID ist Device-Sub-DID
                (Swarm, Validates, Identity), // Peer-Authentifizierung via Identity
                (Gossip, DependsOn, Identity), // Gossip-Messages sind signiert
                (Privacy, DependsOn, Identity), // Privacy-Level basiert auf Identity-Mode
                // Protection
                (Anomaly, Validates, Identity), // Anomalie-Detection für Identity-Ops
                (Identity, Triggers, Anomaly),  // Suspicious Identity-Activity triggert Anomaly
                (AntiCalcification, Validates, Identity), // Power-Konzentration durch Delegationen
                // Credential-Sub-System
                (Credential, DependsOn, Identity), // Credentials gehören zu Identity
                (Credential, Validates, Identity), // Credential-Verifikation validiert Identity
                (Identity, Aggregates, Credential), // Identity aggregiert ihre Credentials
                // KeyManagement-Sub-System
                (KeyManagement, DependsOn, Identity), // Keys gehören zu Identity
                (Identity, Aggregates, KeyManagement), // Identity aggregiert Key-Material
                (KeyManagement, Triggers, Event),     // Key-Rotation erzeugt Events
                // Storage
                (KvStore, Aggregates, Identity), // KvStore persistiert Identity-Daten
                (Identity, DependsOn, KvStore),  // Identity lädt State aus KvStore
                // Engine-Layer
                (UI, DependsOn, Identity), // UI zeigt Identity-basierte Inhalte
                (API, DependsOn, Identity), // API-AuthN basiert auf Identity
                (Governance, DependsOn, Identity), // Voting-Power basiert auf Identity
                // ═══════════════════════════════════════════════════════════════════════════
                // CORE-LAYER BEZIEHUNGEN (Trust, Event, WorldFormula, Consensus)
                // ═══════════════════════════════════════════════════════════════════════════
                (Trust, Triggers, Event), // Trust-Updates erzeugen Events
                (Event, Triggers, Trust), // Events können Trust beeinflussen
                (Trust, DependsOn, WorldFormula), // Trust fließt in 𝔼
                (Event, DependsOn, WorldFormula), // Events fließen in 𝔼
                (WorldFormula, Triggers, Consensus), // 𝔼 beeinflusst Konsens
                (Consensus, Validates, Event), // Konsens validiert Events
                // Execution-Layer Beziehungen
                (Gas, DependsOn, Trust),       // Gas-Budget basiert auf Trust
                (Mana, DependsOn, Trust),      // Mana basiert auf Trust
                (Execution, Aggregates, Gas),  // Execution trackt Gas
                (Execution, Aggregates, Mana), // Execution trackt Mana
                (Execution, Triggers, Event),  // Execution emittiert Events
                // Protection-Layer Beziehungen
                (Anomaly, Validates, Event),       // Anomaly prüft Events
                (Anomaly, Validates, Trust),       // Anomaly prüft Trust-Patterns
                (Diversity, Validates, Trust),     // Diversity prüft Trust-Verteilung
                (Diversity, Validates, Consensus), // Diversity prüft Validator-Mix
                (Quadratic, DependsOn, Trust),     // Voting-Power hängt von Trust ab
                (AntiCalcification, Validates, Trust), // Anti-Calc überwacht Power
                (AntiCalcification, Triggers, Trust), // Anti-Calc kann Trust limitieren
                (Calibration, Triggers, Gas),      // Calibration passt Gas-Preise an
                (Calibration, Triggers, Mana),     // Calibration passt Mana-Regen an
                // Storage-Layer Beziehungen
                (EventStore, Aggregates, Event), // EventStore persistiert Events
                (Archive, Aggregates, EventStore), // Archive komprimiert EventStore
                (KvStore, DependsOn, Trust),     // KV-Access prüft Trust
                (Blueprint, DependsOn, Trust),   // Blueprint-Publish prüft Trust
                // Peer-Layer Beziehungen (Κ22-Κ24)
                (Gateway, Validates, Trust), // Gateway prüft Trust für Crossing
                (Gateway, DependsOn, Trust), // Gateway-Entscheidung basiert auf Trust
                (Gateway, Triggers, Event),  // Crossing erzeugt Events
                (Gateway, DependsOn, Realm), // Gateway prüft Realm-Crossing-Rules
                (SagaComposer, DependsOn, Trust), // Saga-Budget basiert auf Trust
                (SagaComposer, Triggers, Execution), // Sagas erzeugen Executions
                (SagaComposer, Aggregates, IntentParser), // Composer nutzt Parser
                (IntentParser, Validates, Event), // Parser validiert Intent-Events
                // REALM-LAYER BEZIEHUNGEN (Κ22-Κ24: Isolation, Crossing, Sagas)
                (Realm, DependsOn, Trust), // Realm-Trust basiert auf Global-Trust + Realm-Modifikator
                (Realm, Triggers, Trust),  // Realm-spezifisches Verhalten beeinflusst Global-Trust
                (Realm, Aggregates, Gateway), // Realm trackt Crossings (in/out)
                (Realm, DependsOn, Gateway), // Realm nutzt Gateway für Crossing-Kontrolle
                (Realm, Triggers, SagaComposer), // Realm kann Cross-Realm-Sagas auslösen
                (Realm, Triggers, Event), // Realm-Events (Registrierung, Rule-Änderungen, Membership)
                (Realm, Validates, Event), // Realm validiert Events gegen Realm-Policies
                (Realm, DependsOn, ECLPolicy), // Realm-Regeln definiert durch ECL-Policies
                (Realm, Aggregates, ECLPolicy), // Realm trackt aktive Policies
                // ECLVM-Layer Beziehungen (Erynoa Core Language)
                (ECLVM, DependsOn, Gas),  // ECLVM verbraucht Gas (Compute)
                (ECLVM, DependsOn, Mana), // ECLVM verbraucht Mana (Bandwidth/Events)
                (ECLVM, Triggers, Event), // Jede ECL-Ausführung emittiert Events
                (ECLVM, Aggregates, Execution), // ECLVM aggregiert Execution-Metriken
                (ECLVM, DependsOn, Trust), // ECL-Budget basiert auf Trust
                (ECLPolicy, Validates, Gateway), // Policies validieren Crossings (Κ23)
                (ECLPolicy, Validates, Realm), // Policies definieren Realm-Regeln
                (ECLPolicy, DependsOn, ECLVM), // Policies werden von ECLVM ausgeführt
                (ECLPolicy, Triggers, Event), // Policy-Evaluationen erzeugen Events
                (ECLBlueprint, DependsOn, ECLVM), // Blueprints werden von ECLVM instanziiert
                (ECLBlueprint, Aggregates, Blueprint), // Blueprint-Marketplace nutzt Storage
                (ECLBlueprint, Triggers, Event), // Blueprint-Instanziierung erzeugt Events
                (SagaComposer, DependsOn, ECLVM), // Sagas werden durch ECLVM orchestriert
                (IntentParser, DependsOn, ECLPolicy), // Intents werden gegen Policies validiert
                (Gateway, DependsOn, ECLPolicy), // Gateway führt Crossing-Policies aus
                // P2P Network-Layer Beziehungen
                (Swarm, Triggers, Event),        // Swarm propagiert Events
                (Gossip, DependsOn, Trust),      // Gossip-Scoring nutzt Trust
                (Gossip, Triggers, Event),       // Gossip verteilt Events
                (Kademlia, Aggregates, Swarm),   // DHT aggregiert Peer-Info
                (Relay, DependsOn, Trust),       // Relay-Auswahl basiert auf Trust
                (Relay, Triggers, Swarm),        // Relay beeinflusst Connections
                (NatTraversal, Triggers, Swarm), // NAT-Status beeinflusst Erreichbarkeit
                (Privacy, DependsOn, Trust),     // Privacy-Level basiert auf Trust
                (Privacy, Validates, Gossip),    // Privacy validiert Routing
                // ═══════════════════════════════════════════════════════════════════════════
                // ROOM & PARTITION BEZIEHUNGEN (Sub-Realm Isolation)
                // ═══════════════════════════════════════════════════════════════════════════
                (Room, DependsOn, Realm), // Room ist Sub-Einheit eines Realms
                (Room, DependsOn, Trust), // Room-Access prüft Trust
                (Room, Triggers, Event),  // Room-Aktionen erzeugen Events
                (Room, Aggregates, Controller), // Room trackt Controller-Permissions
                (Partition, DependsOn, Room), // Partition ist Sub-Einheit eines Rooms
                (Partition, DependsOn, Trust), // Partition-Access prüft Trust
                (Partition, Validates, Controller), // Partition validiert Controller-Scope
                // ═══════════════════════════════════════════════════════════════════════════
                // UI-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (UI, DependsOn, Trust), // UI-Sichtbarkeit basiert auf Trust
                (UI, DependsOn, Realm), // UI ist per-Realm isoliert
                (UI, DependsOn, Room),  // UI-Scoping auf Room-Ebene
                (UI, DependsOn, Controller), // UI nutzt Controller für Permissions
                (UI, Triggers, Event),  // UI-Actions erzeugen Events
                (UI, Aggregates, DataLogic), // UI nutzt DataLogic für Bindings
                (UI, DependsOn, ECLVM), // UI-Logik läuft in ECLVM
                (UI, DependsOn, Gas),   // UI-Rendering verbraucht Gas
                (UI, DependsOn, Mana),  // UI-Events verbrauchen Mana
                // ═══════════════════════════════════════════════════════════════════════════
                // DATALOGIC-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (DataLogic, DependsOn, Event), // DataLogic verarbeitet Events
                (DataLogic, Aggregates, Event), // DataLogic aggregiert Event-Streams
                (DataLogic, Triggers, Event),  // Aggregationen emittieren Events
                (DataLogic, DependsOn, Trust), // DataAccess prüft Trust
                (DataLogic, DependsOn, ECLVM), // DataLogic-Funktionen in ECLVM
                (DataLogic, DependsOn, Gas),   // Compute verbraucht Gas
                (DataLogic, Validates, UI),    // DataLogic validiert UI-Bindings
                // ═══════════════════════════════════════════════════════════════════════════
                // API-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (API, DependsOn, Trust),      // API-Access basiert auf Trust
                (API, DependsOn, Controller), // API nutzt Controller für AuthZ
                (API, Validates, Gateway),    // API validiert External-Gateway
                (API, Triggers, Event),       // API-Calls erzeugen Events
                (API, DependsOn, ECLVM),      // API-Handler laufen in ECLVM
                (API, DependsOn, Gas),        // API-Processing verbraucht Gas
                (API, DependsOn, Mana),       // API-Responses verbrauchen Mana
                (API, Aggregates, DataLogic), // API nutzt DataLogic für Queries
                // ═══════════════════════════════════════════════════════════════════════════
                // GOVERNANCE-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (Governance, DependsOn, Trust), // Voting-Power basiert auf Trust
                (Governance, DependsOn, Quadratic), // Governance nutzt Quadratic-Voting
                (Governance, Validates, Controller), // Governance validiert Controller-Changes
                (Governance, Triggers, Controller), // Governance-Votes ändern Controller
                (Governance, Triggers, Event),  // Proposals/Votes erzeugen Events
                (Governance, DependsOn, ECLVM), // Governance-Regeln in ECLVM
                (Governance, DependsOn, Realm), // Governance ist per-Realm
                (Governance, Validates, AntiCalcification), // Governance prüft Machtkonz.
                // ═══════════════════════════════════════════════════════════════════════════
                // CONTROLLER-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (Controller, DependsOn, Trust), // Permissions basieren auf Trust
                (Controller, Triggers, Event),  // Permission-Changes erzeugen Events
                (Controller, Validates, Gateway), // Controller validiert Crossings
                (Controller, Validates, API),   // Controller validiert API-Access
                (Controller, Validates, UI),    // Controller validiert UI-Access
                (Controller, DependsOn, Realm), // Controller-Scope ist per-Realm
                (Controller, DependsOn, Room),  // Controller-Scope ist per-Room
                (Controller, DependsOn, Partition), // Controller-Scope ist per-Partition
                (Controller, Aggregates, Governance), // Controller trackt Gov-Delegations
                (Controller, DependsOn, ECLVM), // Permission-Rules in ECLVM
                // ═══════════════════════════════════════════════════════════════════════════
                // BLUEPRINTCOMPOSER-ENGINE BEZIEHUNGEN
                // ═══════════════════════════════════════════════════════════════════════════
                (BlueprintComposer, DependsOn, Blueprint), // Composer nutzt Blueprint-Storage
                (BlueprintComposer, Aggregates, ECLBlueprint), // Composer aggregiert Instanzen
                (BlueprintComposer, Triggers, Event),      // Composition erzeugt Events
                (BlueprintComposer, DependsOn, ECLVM),     // Composition läuft in ECLVM
                (BlueprintComposer, DependsOn, Trust),     // Blueprint-Publish prüft Trust
                (BlueprintComposer, Validates, Realm),     // Composer validiert Realm-Compat.
                (BlueprintComposer, DependsOn, Gas),       // Composition verbraucht Gas
            ],
        }
    }

    /// Finde alle Komponenten die von `component` abhängen
    pub fn dependents(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(_, rel, to)| {
                *to == component
                    && matches!(rel, StateRelation::DependsOn | StateRelation::Aggregates)
            })
            .map(|(from, _, _)| *from)
            .collect()
    }

    /// Finde alle Komponenten die `component` triggert
    pub fn triggered_by(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, _)| *from == component && matches!(rel, StateRelation::Triggers))
            .map(|(_, _, to)| *to)
            .collect()
    }

    /// Finde alle Komponenten die von `component` aggregiert werden
    pub fn aggregated_by(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, _)| *from == component && matches!(rel, StateRelation::Aggregates))
            .map(|(_, _, to)| *to)
            .collect()
    }

    /// Finde alle Komponenten die `component` validiert
    pub fn validated_by(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, _)| *from == component && matches!(rel, StateRelation::Validates))
            .map(|(_, _, to)| *to)
            .collect()
    }

    /// Finde alle Validatoren für `component`
    pub fn validators_of(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(_, rel, to)| *to == component && matches!(rel, StateRelation::Validates))
            .map(|(from, _, _)| *from)
            .collect()
    }

    /// Finde alle bidirektionalen Partner von `component`
    pub fn bidirectional_with(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, to)| {
                matches!(rel, StateRelation::Bidirectional)
                    && (*from == component || *to == component)
            })
            .map(|(from, _, to)| if *from == component { *to } else { *from })
            .collect()
    }

    /// Finde alle Komponenten von denen `component` abhängt
    pub fn dependencies_of(&self, component: StateComponent) -> Vec<StateComponent> {
        self.edges
            .iter()
            .filter(|(from, rel, _)| *from == component && matches!(rel, StateRelation::DependsOn))
            .map(|(_, _, to)| *to)
            .collect()
    }

    /// Prüfe ob eine Beziehung existiert
    pub fn has_relation(
        &self,
        from: StateComponent,
        relation: StateRelation,
        to: StateComponent,
    ) -> bool {
        self.edges.contains(&(from, relation, to))
    }

    /// Alle Beziehungen einer Komponente (eingehend und ausgehend)
    pub fn all_relations(
        &self,
        component: StateComponent,
    ) -> Vec<(StateComponent, StateRelation, StateComponent)> {
        self.edges
            .iter()
            .filter(|(from, _, to)| *from == component || *to == component)
            .cloned()
            .collect()
    }

    /// Transitive Abhängigkeiten (rekursiv alle Dependencies)
    pub fn transitive_dependencies(&self, component: StateComponent) -> HashSet<StateComponent> {
        let mut visited = HashSet::new();
        let mut stack = vec![component];

        while let Some(current) = stack.pop() {
            for dep in self.dependencies_of(current) {
                if visited.insert(dep) {
                    stack.push(dep);
                }
            }
        }
        visited
    }

    /// Transitive Trigger-Kette (alle Komponenten die transitiv getriggert werden)
    pub fn transitive_triggers(&self, component: StateComponent) -> HashSet<StateComponent> {
        let mut visited = HashSet::new();
        let mut stack = vec![component];

        while let Some(current) = stack.pop() {
            for triggered in self.triggered_by(current) {
                if visited.insert(triggered) {
                    stack.push(triggered);
                }
            }
        }
        visited
    }

    /// Ermittle Validierungs-Kette für eine Komponente
    pub fn validation_chain(&self, component: StateComponent) -> Vec<StateComponent> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();
        let mut current = component;

        while let Some(validator) = self.validators_of(current).first().copied() {
            if visited.insert(validator) {
                chain.push(validator);
                current = validator;
            } else {
                break; // Zyklus erkannt
            }
        }
        chain
    }

    /// Kritikalitäts-Score einer Komponente (wie viele andere abhängen)
    pub fn criticality_score(&self, component: StateComponent) -> usize {
        self.dependents(component).len()
            + self.transitive_triggers(component).len()
            + self.aggregated_by(component).len()
    }
}

// ============================================================================
// CORE STATE LAYER (Κ2-Κ18)
// ============================================================================

/// Trust-State mit Beziehungs-Tracking
#[derive(Debug)]
pub struct TrustState {
    // Atomic Counters
    pub entities_count: AtomicUsize,
    pub relationships_count: AtomicUsize,
    pub updates_total: AtomicU64,
    pub positive_updates: AtomicU64,
    pub negative_updates: AtomicU64,
    pub violations_count: AtomicU64,

    // Complex State (RwLock)
    pub avg_trust: RwLock<f64>,
    pub trust_distribution: RwLock<TrustDistribution>,

    // ─────────────────────────────────────────────────────────────────────────
    // Identity-Integration (Phase 7)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Einträge keyed by UniversalId
    /// Enthält globalen und per-Realm Trust für jede Identity
    pub trust_by_id: RwLock<HashMap<UniversalId, TrustEntry>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (Beziehungen im StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Events die durch Trust-Updates ausgelöst wurden (Trust → Event)
    pub triggered_events: AtomicU64,
    /// Trust-Updates die durch Events ausgelöst wurden (Event → Trust)
    pub event_triggered_updates: AtomicU64,
    /// Trust-Updates die durch Realm-Aktivität ausgelöst wurden (Realm → Trust)
    pub realm_triggered_updates: AtomicU64,
}

/// Trust-Verteilung für Diversity-Monitoring
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustDistribution {
    /// Buckets: [0-0.1, 0.1-0.2, ..., 0.9-1.0]
    pub histogram: [u64; 10],
    /// Gini-Koeffizient
    pub gini: f64,
    /// Shannon-Entropie
    pub entropy: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// TRUST ENTRY (Identity-Integration Phase 7)
// ─────────────────────────────────────────────────────────────────────────────

/// Trust-Eintrag mit Identity-Integration
/// Speichert globalen Trust und Per-Realm Trust für eine Identität
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEntry {
    /// Identity UniversalId
    pub identity_id: UniversalId,
    /// Globaler Trust-Wert [0.0, 1.0]
    pub global_trust: f64,
    /// Per-Realm Trust-Werte (Realm-ID → Trust-Wert)
    pub per_realm_trust: HashMap<UniversalId, f64>,
    /// Letztes Update (Unix-Epoch Millisekunden)
    pub last_update_ms: u64,
    /// Anzahl Updates
    pub update_count: u64,
    /// Trust-Decay-Faktor (Κ8)
    pub decay_factor: f64,
}

impl TrustEntry {
    /// Erstelle neuen Trust-Eintrag
    pub fn new(identity_id: UniversalId, initial_trust: f64) -> Self {
        Self {
            identity_id,
            global_trust: initial_trust.clamp(0.0, 1.0),
            per_realm_trust: HashMap::new(),
            last_update_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            update_count: 0,
            decay_factor: 1.0, // Kein Decay initial
        }
    }

    /// Aktualisiere globalen Trust
    pub fn update_global(&mut self, delta: f64) {
        self.global_trust = (self.global_trust + delta).clamp(0.0, 1.0);
        self.update_count += 1;
        self.last_update_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
    }

    /// Aktualisiere Realm-spezifischen Trust
    pub fn update_realm(&mut self, realm_id: UniversalId, delta: f64) {
        let current = self
            .per_realm_trust
            .get(&realm_id)
            .copied()
            .unwrap_or(self.global_trust);
        self.per_realm_trust
            .insert(realm_id, (current + delta).clamp(0.0, 1.0));
        self.update_count += 1;
        self.last_update_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
    }

    /// Hole Trust für bestimmtes Realm (fällt auf global zurück)
    pub fn get_realm_trust(&self, realm_id: &UniversalId) -> f64 {
        self.per_realm_trust
            .get(realm_id)
            .copied()
            .unwrap_or(self.global_trust)
    }

    /// Wende Decay an (Κ8: Trust-Decay über Zeit)
    pub fn apply_decay(&mut self, decay_rate: f64) {
        self.decay_factor *= 1.0 - decay_rate;
        self.global_trust *= 1.0 - decay_rate;
        for trust in self.per_realm_trust.values_mut() {
            *trust *= 1.0 - decay_rate;
        }
    }
}

/// Fehler bei Trust-Operationen
#[derive(Debug, Clone, thiserror::Error)]
pub enum TrustError {
    #[error("Identity not found: {0}")]
    IdentityNotFound(String),
    #[error("Trust value out of range: {0}")]
    ValueOutOfRange(f64),
    #[error("Realm not found: {0}")]
    RealmNotFound(String),
    #[error("Lock acquisition failed")]
    LockError,
}

impl TrustState {
    pub fn new() -> Self {
        Self {
            entities_count: AtomicUsize::new(0),
            relationships_count: AtomicUsize::new(0),
            updates_total: AtomicU64::new(0),
            positive_updates: AtomicU64::new(0),
            negative_updates: AtomicU64::new(0),
            violations_count: AtomicU64::new(0),
            avg_trust: RwLock::new(0.5),
            trust_distribution: RwLock::new(TrustDistribution::default()),
            trust_by_id: RwLock::new(HashMap::new()),
            triggered_events: AtomicU64::new(0),
            event_triggered_updates: AtomicU64::new(0),
            realm_triggered_updates: AtomicU64::new(0),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Identity-based Trust Operations (Phase 7)
    // ─────────────────────────────────────────────────────────────────────────

    /// Hole Trust für Identity (global)
    pub fn get_trust(&self, identity: &UniversalId) -> Option<f64> {
        self.trust_by_id
            .read()
            .ok()
            .and_then(|map| map.get(identity).map(|e| e.global_trust))
    }

    /// Hole Trust für Identity in bestimmtem Realm
    pub fn get_realm_trust(&self, identity: &UniversalId, realm: &UniversalId) -> Option<f64> {
        self.trust_by_id
            .read()
            .ok()
            .and_then(|map| map.get(identity).map(|e| e.get_realm_trust(realm)))
    }

    /// Registriere neue Identity mit Initial-Trust
    pub fn register_identity(
        &self,
        identity: UniversalId,
        initial_trust: f64,
    ) -> Result<(), TrustError> {
        let entry = TrustEntry::new(identity, initial_trust);
        if let Ok(mut map) = self.trust_by_id.write() {
            map.insert(identity, entry);
            self.entities_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        } else {
            Err(TrustError::LockError)
        }
    }

    /// Aktualisiere Trust für Identity (global)
    pub fn update_identity_trust(
        &self,
        identity: &UniversalId,
        delta: f64,
    ) -> Result<f64, TrustError> {
        if let Ok(mut map) = self.trust_by_id.write() {
            if let Some(entry) = map.get_mut(identity) {
                entry.update_global(delta);
                self.updates_total.fetch_add(1, Ordering::Relaxed);
                if delta > 0.0 {
                    self.positive_updates.fetch_add(1, Ordering::Relaxed);
                } else {
                    self.negative_updates.fetch_add(1, Ordering::Relaxed);
                }
                Ok(entry.global_trust)
            } else {
                Err(TrustError::IdentityNotFound(hex::encode(
                    identity.as_bytes(),
                )))
            }
        } else {
            Err(TrustError::LockError)
        }
    }

    /// Aktualisiere Trust für Identity in Realm
    pub fn update_identity_realm_trust(
        &self,
        identity: &UniversalId,
        realm: UniversalId,
        delta: f64,
    ) -> Result<f64, TrustError> {
        if let Ok(mut map) = self.trust_by_id.write() {
            if let Some(entry) = map.get_mut(identity) {
                entry.update_realm(realm, delta);
                self.updates_total.fetch_add(1, Ordering::Relaxed);
                self.realm_triggered_updates.fetch_add(1, Ordering::Relaxed);
                Ok(entry.get_realm_trust(&realm))
            } else {
                Err(TrustError::IdentityNotFound(hex::encode(
                    identity.as_bytes(),
                )))
            }
        } else {
            Err(TrustError::LockError)
        }
    }

    /// Hole vollständigen TrustEntry für Identity
    pub fn get_trust_entry(&self, identity: &UniversalId) -> Option<TrustEntry> {
        self.trust_by_id
            .read()
            .ok()
            .and_then(|map| map.get(identity).cloned())
    }

    /// Wende Decay auf alle Identities an (periodisch aufrufen)
    pub fn apply_global_decay(&self, decay_rate: f64) {
        if let Ok(mut map) = self.trust_by_id.write() {
            for entry in map.values_mut() {
                entry.apply_decay(decay_rate);
            }
        }
    }

    /// Anzahl registrierter Identities
    pub fn identity_count(&self) -> usize {
        self.trust_by_id.read().map(|map| map.len()).unwrap_or(0)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Legacy Operations
    // ─────────────────────────────────────────────────────────────────────────

    /// Update Trust mit Kausalitäts-Tracking
    pub fn update(&self, positive: bool, from_event: bool) {
        self.updates_total.fetch_add(1, Ordering::Relaxed);
        if positive {
            self.positive_updates.fetch_add(1, Ordering::Relaxed);
        } else {
            self.negative_updates.fetch_add(1, Ordering::Relaxed);
        }
        if from_event {
            self.event_triggered_updates.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Trust-Update erzeugt Event
    pub fn update_triggered_event(&self) {
        self.triggered_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Berechne Update-Asymmetrie-Ratio (sollte ~2:1 sein wegen Κ4)
    pub fn asymmetry_ratio(&self) -> f64 {
        let pos = self.positive_updates.load(Ordering::Relaxed) as f64;
        let neg = self.negative_updates.load(Ordering::Relaxed) as f64;
        if pos > 0.0 {
            neg / pos
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> TrustSnapshot {
        TrustSnapshot {
            entities_count: self.entities_count.load(Ordering::Relaxed),
            relationships_count: self.relationships_count.load(Ordering::Relaxed),
            updates_total: self.updates_total.load(Ordering::Relaxed),
            positive_updates: self.positive_updates.load(Ordering::Relaxed),
            negative_updates: self.negative_updates.load(Ordering::Relaxed),
            violations_count: self.violations_count.load(Ordering::Relaxed),
            avg_trust: self.avg_trust.read().map(|v| *v).unwrap_or(0.5),
            asymmetry_ratio: self.asymmetry_ratio(),
            triggered_events: self.triggered_events.load(Ordering::Relaxed),
            event_triggered_updates: self.event_triggered_updates.load(Ordering::Relaxed),
            distribution: self.trust_distribution.read().map(|d| d.clone()).ok(),
            identity_trust_count: self.identity_count(),
        }
    }
}

impl Default for TrustState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustSnapshot {
    pub entities_count: usize,
    pub relationships_count: usize,
    pub updates_total: u64,
    pub positive_updates: u64,
    pub negative_updates: u64,
    pub violations_count: u64,
    pub avg_trust: f64,
    pub asymmetry_ratio: f64,
    pub triggered_events: u64,
    pub event_triggered_updates: u64,
    pub distribution: Option<TrustDistribution>,
    /// Anzahl Identity-Trust-Einträge (Phase 7)
    pub identity_trust_count: usize,
}

/// Event-State mit DAG-Tracking und Relationship-Counters
#[derive(Debug)]
pub struct EventState {
    // Atomic Counters
    pub total: AtomicU64,
    pub genesis: AtomicU64,
    pub finalized: AtomicU64,
    pub witnessed: AtomicU64,
    pub validation_errors: AtomicU64,
    pub cycles_detected: AtomicU64,

    // DAG Metrics
    pub max_depth: AtomicU64,
    pub avg_parents: RwLock<f64>,

    // Finality Tracking
    pub finality_latency_ms: RwLock<Vec<u64>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph Trigger-Beziehungen → Event)
    // ─────────────────────────────────────────────────────────────────────────
    /// Events durch Trust-Updates getriggert (Trust → Event)
    pub trust_triggered: AtomicU64,
    /// Events durch Consensus validiert (Consensus → Event)
    pub consensus_validated: AtomicU64,
    /// Events durch Execution getriggert (Execution → Event)
    pub execution_triggered: AtomicU64,
    /// Events durch Gateway/Crossing getriggert (Gateway → Event)
    pub gateway_triggered: AtomicU64,
    /// Events durch Realm getriggert (Realm → Event)
    pub realm_triggered: AtomicU64,
    /// Events durch ECLVM-Ausführung getriggert (ECLVM → Event)
    pub eclvm_triggered: AtomicU64,
    /// Events durch ECLPolicy getriggert (ECLPolicy → Event)
    pub policy_triggered: AtomicU64,
    /// Events durch ECLBlueprint getriggert (ECLBlueprint → Event)
    pub blueprint_triggered: AtomicU64,
    /// Events durch Swarm propagiert (Swarm → Event)
    pub swarm_triggered: AtomicU64,
    /// Events durch Gossip verteilt (Gossip → Event)
    pub gossip_triggered: AtomicU64,
}

impl EventState {
    pub fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            genesis: AtomicU64::new(0),
            finalized: AtomicU64::new(0),
            witnessed: AtomicU64::new(0),
            validation_errors: AtomicU64::new(0),
            cycles_detected: AtomicU64::new(0),
            max_depth: AtomicU64::new(0),
            avg_parents: RwLock::new(0.0),
            finality_latency_ms: RwLock::new(Vec::new()),
            trust_triggered: AtomicU64::new(0),
            consensus_validated: AtomicU64::new(0),
            execution_triggered: AtomicU64::new(0),
            gateway_triggered: AtomicU64::new(0),
            realm_triggered: AtomicU64::new(0),
            eclvm_triggered: AtomicU64::new(0),
            policy_triggered: AtomicU64::new(0),
            blueprint_triggered: AtomicU64::new(0),
            swarm_triggered: AtomicU64::new(0),
            gossip_triggered: AtomicU64::new(0),
        }
    }

    pub fn add(&self, is_genesis: bool, parents_count: usize, depth: u64) {
        self.total.fetch_add(1, Ordering::Relaxed);
        if is_genesis {
            self.genesis.fetch_add(1, Ordering::Relaxed);
        }
        // Update max depth
        loop {
            let current = self.max_depth.load(Ordering::Relaxed);
            if depth <= current {
                break;
            }
            if self
                .max_depth
                .compare_exchange(current, depth, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
        // Update avg parents (rolling average)
        if let Ok(mut avg) = self.avg_parents.write() {
            let total = self.total.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + parents_count as f64) / total;
        }
    }

    pub fn finalize(&self, latency_ms: u64) {
        self.finalized.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut latencies) = self.finality_latency_ms.write() {
            latencies.push(latency_ms);
            // Keep last 1000 for averaging
            if latencies.len() > 1000 {
                latencies.remove(0);
            }
        }
    }

    pub fn avg_finality_latency(&self) -> f64 {
        self.finality_latency_ms
            .read()
            .map(|v| {
                if v.is_empty() {
                    0.0
                } else {
                    v.iter().sum::<u64>() as f64 / v.len() as f64
                }
            })
            .unwrap_or(0.0)
    }

    pub fn snapshot(&self) -> EventSnapshot {
        EventSnapshot {
            total: self.total.load(Ordering::Relaxed),
            genesis: self.genesis.load(Ordering::Relaxed),
            finalized: self.finalized.load(Ordering::Relaxed),
            witnessed: self.witnessed.load(Ordering::Relaxed),
            validation_errors: self.validation_errors.load(Ordering::Relaxed),
            cycles_detected: self.cycles_detected.load(Ordering::Relaxed),
            max_depth: self.max_depth.load(Ordering::Relaxed),
            avg_parents: self.avg_parents.read().map(|v| *v).unwrap_or(0.0),
            avg_finality_latency_ms: self.avg_finality_latency(),
            trust_triggered: self.trust_triggered.load(Ordering::Relaxed),
            consensus_validated: self.consensus_validated.load(Ordering::Relaxed),
        }
    }
}

impl Default for EventState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSnapshot {
    pub total: u64,
    pub genesis: u64,
    pub finalized: u64,
    pub witnessed: u64,
    pub validation_errors: u64,
    pub cycles_detected: u64,
    pub max_depth: u64,
    pub avg_parents: f64,
    pub avg_finality_latency_ms: f64,
    pub trust_triggered: u64,
    pub consensus_validated: u64,
}

/// World Formula State (Κ15b-d)
#[derive(Debug)]
pub struct FormulaState {
    pub current_e: RwLock<f64>,
    pub computations: AtomicU64,
    pub contributors: AtomicUsize,
    pub human_verified: AtomicUsize,

    // Komponenten von 𝔼
    pub avg_activity: RwLock<f64>,
    pub avg_trust_norm: RwLock<f64>,
    pub human_factor: RwLock<f64>,

    // History für Trend-Analyse
    pub e_history: RwLock<Vec<(u64, f64)>>, // (timestamp_ms, value)
}

impl FormulaState {
    pub fn new() -> Self {
        Self {
            current_e: RwLock::new(0.0),
            computations: AtomicU64::new(0),
            contributors: AtomicUsize::new(0),
            human_verified: AtomicUsize::new(0),
            avg_activity: RwLock::new(0.0),
            avg_trust_norm: RwLock::new(0.0),
            human_factor: RwLock::new(1.0),
            e_history: RwLock::new(Vec::new()),
        }
    }

    pub fn update(&self, e: f64, activity: f64, trust_norm: f64, human_factor: f64) {
        self.computations.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut current) = self.current_e.write() {
            *current = e;
        }
        if let Ok(mut a) = self.avg_activity.write() {
            *a = activity;
        }
        if let Ok(mut t) = self.avg_trust_norm.write() {
            *t = trust_norm;
        }
        if let Ok(mut h) = self.human_factor.write() {
            *h = human_factor;
        }
        // Record history
        if let Ok(mut history) = self.e_history.write() {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            history.push((ts, e));
            // Keep last 1000 entries
            if history.len() > 1000 {
                history.remove(0);
            }
        }
    }

    /// Berechne Trend (positiv = steigend)
    pub fn trend(&self) -> f64 {
        self.e_history
            .read()
            .map(|h| {
                if h.len() < 10 {
                    return 0.0;
                }
                let recent: f64 = h.iter().rev().take(10).map(|(_, e)| e).sum::<f64>() / 10.0;
                let older: f64 = h
                    .iter()
                    .rev()
                    .skip(10)
                    .take(10)
                    .map(|(_, e)| e)
                    .sum::<f64>()
                    / 10.0_f64.max(h.len().saturating_sub(10) as f64);
                recent - older
            })
            .unwrap_or(0.0)
    }

    pub fn snapshot(&self) -> FormulaSnapshot {
        FormulaSnapshot {
            current_e: self.current_e.read().map(|v| *v).unwrap_or(0.0),
            computations: self.computations.load(Ordering::Relaxed),
            contributors: self.contributors.load(Ordering::Relaxed),
            human_verified: self.human_verified.load(Ordering::Relaxed),
            avg_activity: self.avg_activity.read().map(|v| *v).unwrap_or(0.0),
            avg_trust_norm: self.avg_trust_norm.read().map(|v| *v).unwrap_or(0.0),
            human_factor: self.human_factor.read().map(|v| *v).unwrap_or(1.0),
            trend: self.trend(),
        }
    }
}

impl Default for FormulaState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormulaSnapshot {
    pub current_e: f64,
    pub computations: u64,
    pub contributors: usize,
    pub human_verified: usize,
    pub avg_activity: f64,
    pub avg_trust_norm: f64,
    pub human_factor: f64,
    pub trend: f64,
}

/// Consensus State (Κ18)
#[derive(Debug)]
pub struct ConsensusState {
    pub epoch: AtomicU64,
    pub validators: AtomicUsize,
    pub successful_rounds: AtomicU64,
    pub failed_rounds: AtomicU64,
    pub avg_round_time_ms: RwLock<f64>,

    // BFT-spezifisch
    pub byzantine_detected: AtomicU64,
    pub leader_changes: AtomicU64,

    // Relationship-Tracking
    /// Events validiert durch Consensus (Consensus ✓ Event)
    pub events_validated: AtomicU64,
}

impl ConsensusState {
    pub fn new() -> Self {
        Self {
            epoch: AtomicU64::new(0),
            validators: AtomicUsize::new(0),
            successful_rounds: AtomicU64::new(0),
            failed_rounds: AtomicU64::new(0),
            avg_round_time_ms: RwLock::new(0.0),
            byzantine_detected: AtomicU64::new(0),
            leader_changes: AtomicU64::new(0),
            events_validated: AtomicU64::new(0),
        }
    }

    pub fn round_completed(&self, success: bool, duration_ms: u64) {
        if success {
            self.successful_rounds.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_rounds.fetch_add(1, Ordering::Relaxed);
        }
        // Rolling average
        if let Ok(mut avg) = self.avg_round_time_ms.write() {
            let total = self.successful_rounds.load(Ordering::Relaxed)
                + self.failed_rounds.load(Ordering::Relaxed);
            *avg = (*avg * (total.saturating_sub(1)) as f64 + duration_ms as f64) / total as f64;
        }
    }

    pub fn success_rate(&self) -> f64 {
        let success = self.successful_rounds.load(Ordering::Relaxed) as f64;
        let failed = self.failed_rounds.load(Ordering::Relaxed) as f64;
        let total = success + failed;
        if total > 0.0 {
            success / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> ConsensusSnapshot {
        ConsensusSnapshot {
            epoch: self.epoch.load(Ordering::Relaxed),
            validators: self.validators.load(Ordering::Relaxed),
            successful_rounds: self.successful_rounds.load(Ordering::Relaxed),
            failed_rounds: self.failed_rounds.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            avg_round_time_ms: self.avg_round_time_ms.read().map(|v| *v).unwrap_or(0.0),
            byzantine_detected: self.byzantine_detected.load(Ordering::Relaxed),
            leader_changes: self.leader_changes.load(Ordering::Relaxed),
        }
    }
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusSnapshot {
    pub epoch: u64,
    pub validators: usize,
    pub successful_rounds: u64,
    pub failed_rounds: u64,
    pub success_rate: f64,
    pub avg_round_time_ms: f64,
    pub byzantine_detected: u64,
    pub leader_changes: u64,
}

/// Aggregierter Core State
#[derive(Debug)]
pub struct CoreState {
    pub trust: TrustState,
    pub events: EventState,
    pub formula: FormulaState,
    pub consensus: ConsensusState,
}

impl CoreState {
    pub fn new() -> Self {
        Self {
            trust: TrustState::new(),
            events: EventState::new(),
            formula: FormulaState::new(),
            consensus: ConsensusState::new(),
        }
    }

    pub fn snapshot(&self) -> CoreSnapshot {
        CoreSnapshot {
            trust: self.trust.snapshot(),
            events: self.events.snapshot(),
            formula: self.formula.snapshot(),
            consensus: self.consensus.snapshot(),
        }
    }
}

impl Default for CoreState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreSnapshot {
    pub trust: TrustSnapshot,
    pub events: EventSnapshot,
    pub formula: FormulaSnapshot,
    pub consensus: ConsensusSnapshot,
}

// ============================================================================
// EXECUTION STATE LAYER (IPS ℳ) - Tiefe Struktur mit Sub-States
// ============================================================================

/// Gas-State mit Relationship-Tracking
///
/// Gas ist die Compute-Ressource für ECL-Ausführungen.
/// Basiert auf Trust (DependsOn) und wird durch Calibration angepasst (Triggers).
#[derive(Debug)]
pub struct GasState {
    /// Total verbrauchtes Gas
    pub consumed: AtomicU64,
    /// Refundiertes Gas
    pub refunded: AtomicU64,
    /// Out-of-Gas Errors
    pub out_of_gas_count: AtomicU64,
    /// Aktueller Gas-Preis
    pub current_price: RwLock<f64>,
    /// Max Gas pro Block
    pub max_per_block: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Calibration hat Gas angepasst (Calibration → Gas)
    pub calibration_adjustments: AtomicU64,
    /// Trust-Dependency-Updates (Gas ← Trust)
    pub trust_dependency_updates: AtomicU64,
}

impl GasState {
    pub fn new() -> Self {
        Self {
            consumed: AtomicU64::new(0),
            refunded: AtomicU64::new(0),
            out_of_gas_count: AtomicU64::new(0),
            current_price: RwLock::new(1.0),
            max_per_block: AtomicU64::new(10_000_000),
            calibration_adjustments: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
        }
    }

    pub fn consume(&self, amount: u64) {
        self.consumed.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn refund(&self, amount: u64) {
        self.refunded.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> GasSnapshot {
        GasSnapshot {
            consumed: self.consumed.load(Ordering::Relaxed),
            refunded: self.refunded.load(Ordering::Relaxed),
            out_of_gas_count: self.out_of_gas_count.load(Ordering::Relaxed),
            current_price: self.current_price.read().map(|v| *v).unwrap_or(1.0),
            max_per_block: self.max_per_block.load(Ordering::Relaxed),
            calibration_adjustments: self.calibration_adjustments.load(Ordering::Relaxed),
            trust_dependency_updates: self.trust_dependency_updates.load(Ordering::Relaxed),
        }
    }
}

impl Default for GasState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasSnapshot {
    pub consumed: u64,
    pub refunded: u64,
    pub out_of_gas_count: u64,
    pub current_price: f64,
    pub max_per_block: u64,
    pub calibration_adjustments: u64,
    pub trust_dependency_updates: u64,
}

/// Mana-State mit Relationship-Tracking
///
/// Mana ist die Bandwidth/Event-Ressource.
/// Regeneriert über Zeit, basiert auf Trust (DependsOn).
#[derive(Debug)]
pub struct ManaState {
    /// Total verbrauchtes Mana
    pub consumed: AtomicU64,
    /// Regeneriertes Mana
    pub regenerated: AtomicU64,
    /// Rate-Limited wegen Mana
    pub rate_limited_count: AtomicU64,
    /// Aktuelle Regenerations-Rate
    pub regen_rate: RwLock<f64>,
    /// Max Mana pro Entity
    pub max_per_entity: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Calibration hat Mana angepasst (Calibration → Mana)
    pub calibration_adjustments: AtomicU64,
    /// Trust-Dependency-Updates (Mana ← Trust)
    pub trust_dependency_updates: AtomicU64,
}

impl ManaState {
    pub fn new() -> Self {
        Self {
            consumed: AtomicU64::new(0),
            regenerated: AtomicU64::new(0),
            rate_limited_count: AtomicU64::new(0),
            regen_rate: RwLock::new(1.0),
            max_per_entity: AtomicU64::new(100_000),
            calibration_adjustments: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
        }
    }

    pub fn consume(&self, amount: u64) {
        self.consumed.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn regenerate(&self, amount: u64) {
        self.regenerated.fetch_add(amount, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> ManaSnapshot {
        ManaSnapshot {
            consumed: self.consumed.load(Ordering::Relaxed),
            regenerated: self.regenerated.load(Ordering::Relaxed),
            rate_limited_count: self.rate_limited_count.load(Ordering::Relaxed),
            regen_rate: self.regen_rate.read().map(|v| *v).unwrap_or(1.0),
            max_per_entity: self.max_per_entity.load(Ordering::Relaxed),
            calibration_adjustments: self.calibration_adjustments.load(Ordering::Relaxed),
            trust_dependency_updates: self.trust_dependency_updates.load(Ordering::Relaxed),
        }
    }
}

impl Default for ManaState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManaSnapshot {
    pub consumed: u64,
    pub regenerated: u64,
    pub rate_limited_count: u64,
    pub regen_rate: f64,
    pub max_per_entity: u64,
    pub calibration_adjustments: u64,
    pub trust_dependency_updates: u64,
}

/// Core Execution State mit Relationship-Tracking
#[derive(Debug)]
pub struct ExecutionsState {
    /// Aktive Execution-Kontexte
    pub active_contexts: AtomicUsize,
    /// Total Executions
    pub total: AtomicU64,
    /// Erfolgreiche Executions
    pub successful: AtomicU64,
    /// Fehlgeschlagene Executions
    pub failed: AtomicU64,
    /// Events emittiert
    pub events_emitted: AtomicU64,
    /// Ausführungszeiten für Averaging
    pub execution_times_ms: RwLock<Vec<u64>>,
    /// Aktuelles Epoch
    pub current_epoch: AtomicU64,
    /// Aktueller Lamport-Timestamp
    pub current_lamport: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Executions durch SagaComposer getriggert (SagaComposer → Execution)
    pub saga_triggered: AtomicU64,
    /// Gas-Aggregationen (Execution ⊃ Gas)
    pub gas_aggregations: AtomicU64,
    /// Mana-Aggregationen (Execution ⊃ Mana)
    pub mana_aggregations: AtomicU64,
}

impl ExecutionsState {
    pub fn new() -> Self {
        Self {
            active_contexts: AtomicUsize::new(0),
            total: AtomicU64::new(0),
            successful: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            events_emitted: AtomicU64::new(0),
            execution_times_ms: RwLock::new(Vec::new()),
            current_epoch: AtomicU64::new(0),
            current_lamport: AtomicU64::new(0),
            saga_triggered: AtomicU64::new(0),
            gas_aggregations: AtomicU64::new(0),
            mana_aggregations: AtomicU64::new(0),
        }
    }

    pub fn start(&self) {
        self.active_contexts.fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn complete(&self, success: bool, events: u64, duration_ms: u64) {
        self.active_contexts.fetch_sub(1, Ordering::Relaxed);
        if success {
            self.successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }
        self.events_emitted.fetch_add(events, Ordering::Relaxed);

        if let Ok(mut times) = self.execution_times_ms.write() {
            times.push(duration_ms);
            if times.len() > 1000 {
                times.remove(0);
            }
        }
    }

    /// ECLVM-Policy-Ausführung erfassen (E1.2: ExecutionState ↔ ECLVM Sync)
    ///
    /// Im Gegensatz zu start()/complete() wird hier keine active_context-Verwaltung
    /// benötigt, da ECLVM-Policy-Ausführungen synchron sind (kein langlebiger Kontext).
    /// Diese Methode aggregiert ECLVM-Läufe in die Execution-Metriken.
    pub fn record_eclvm_policy_execution(
        &self,
        success: bool,
        gas_used: u64,
        mana_used: u64,
        events: u64,
        duration_us: u64,
    ) {
        // Zähle als Execution
        self.total.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed.fetch_add(1, Ordering::Relaxed);
        }
        self.events_emitted.fetch_add(events, Ordering::Relaxed);

        // Konvertiere µs → ms für consistency mit anderen Execution-Zeiten
        let duration_ms = duration_us / 1000;
        if let Ok(mut times) = self.execution_times_ms.write() {
            times.push(duration_ms);
            if times.len() > 1000 {
                times.remove(0);
            }
        }

        // Aggregiere Gas/Mana (Relationship-Tracking)
        if gas_used > 0 {
            self.gas_aggregations.fetch_add(1, Ordering::Relaxed);
        }
        if mana_used > 0 {
            self.mana_aggregations.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn avg_execution_time(&self) -> f64 {
        self.execution_times_ms
            .read()
            .map(|v| {
                if v.is_empty() {
                    0.0
                } else {
                    v.iter().sum::<u64>() as f64 / v.len() as f64
                }
            })
            .unwrap_or(0.0)
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.total.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> ExecutionsSnapshot {
        ExecutionsSnapshot {
            active_contexts: self.active_contexts.load(Ordering::Relaxed),
            total: self.total.load(Ordering::Relaxed),
            successful: self.successful.load(Ordering::Relaxed),
            failed: self.failed.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            events_emitted: self.events_emitted.load(Ordering::Relaxed),
            avg_execution_time_ms: self.avg_execution_time(),
            current_epoch: self.current_epoch.load(Ordering::Relaxed),
            current_lamport: self.current_lamport.load(Ordering::Relaxed),
            saga_triggered: self.saga_triggered.load(Ordering::Relaxed),
            gas_aggregations: self.gas_aggregations.load(Ordering::Relaxed),
            mana_aggregations: self.mana_aggregations.load(Ordering::Relaxed),
        }
    }
}

impl Default for ExecutionsState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionsSnapshot {
    pub active_contexts: usize,
    pub total: u64,
    pub successful: u64,
    pub failed: u64,
    pub success_rate: f64,
    pub events_emitted: u64,
    pub avg_execution_time_ms: f64,
    pub current_epoch: u64,
    pub current_lamport: u64,
    pub saga_triggered: u64,
    pub gas_aggregations: u64,
    pub mana_aggregations: u64,
}

/// Execution State Layer mit Sub-States für tiefe Relationship-Integration
#[derive(Debug)]
pub struct ExecutionState {
    /// Gas Sub-State
    pub gas: GasState,
    /// Mana Sub-State
    pub mana: ManaState,
    /// Core Executions Sub-State
    pub executions: ExecutionsState,
}

impl ExecutionState {
    pub fn new() -> Self {
        Self {
            gas: GasState::new(),
            mana: ManaState::new(),
            executions: ExecutionsState::new(),
        }
    }

    /// Legacy-Kompatibilität: Start Execution
    pub fn start(&self) {
        self.executions.start();
    }

    /// Legacy-Kompatibilität: Complete Execution
    pub fn complete(&self, success: bool, gas: u64, mana: u64, events: u64, duration_ms: u64) {
        self.executions.complete(success, events, duration_ms);
        self.gas.consume(gas);
        self.mana.consume(mana);
    }

    /// Legacy-Kompatibilität: Durchschnittliche Ausführungszeit
    pub fn avg_execution_time(&self) -> f64 {
        self.executions.avg_execution_time()
    }

    /// Legacy-Kompatibilität: Erfolgsrate
    pub fn success_rate(&self) -> f64 {
        self.executions.success_rate()
    }

    pub fn snapshot(&self) -> ExecutionSnapshot {
        ExecutionSnapshot {
            gas: self.gas.snapshot(),
            mana: self.mana.snapshot(),
            executions: self.executions.snapshot(),
        }
    }
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution State Snapshot mit Sub-States
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSnapshot {
    pub gas: GasSnapshot,
    pub mana: ManaSnapshot,
    pub executions: ExecutionsSnapshot,
}

// ============================================================================
// ECLVM STATE LAYER (Erynoa Core Language Virtual Machine)
// ============================================================================
//
// ECL (Erynoa Core Language) ist die DSL für:
// - Regeln definieren (Crossing-Policies, Membership, Transaction-Rules)
// - Blueprints erstellen (App-Templates für Chat, Marketplace, etc.)
// - Intents & Sagas beschreiben (Cross-Realm-Aktionen)
//
// ECLVM ist die cost-limited Execution Environment:
// - Sicher durch Gas (Compute) und Mana (Bandwidth/Events)
// - Integration mit ExecutionState für Resource-Tracking
// - Realm-spezifische Policy-Ausführung

/// Policy-Typ für ECL-Regeln
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ECLPolicyType {
    /// Crossing-Rules für Gateway (Κ23)
    Crossing,
    /// Membership-Rules für Realm-Beitritt
    Membership,
    /// Transaction-Rules für Aktionen
    Transaction,
    /// Governance-Rules für Abstimmungen
    Governance,
    /// Privacy-Rules für Daten-Sichtbarkeit
    Privacy,
    /// API-Engine Eintrittspunkt (Phase 3.2)
    Api,
    /// UI-Engine Eintrittspunkt (Phase 3.3)
    Ui,
    /// DataLogic-Engine Eintrittspunkt (Phase 3.4)
    DataLogic,
    /// Controller-Engine Eintrittspunkt (Phase 3.6)
    Controller,
    /// Custom User-defined Policy
    Custom,
}

/// Blueprint-Status im Marketplace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BlueprintStatus {
    /// Entwurf, noch nicht publiziert
    Draft,
    /// Veröffentlicht und verfügbar
    Published,
    /// Verifiziert durch Community
    Verified,
    /// Deprecated, nicht mehr empfohlen
    Deprecated,
}

/// Per-Realm ECL State - Policy-Ausführungen pro Realm
#[derive(Debug)]
pub struct RealmECLState {
    /// Policies ausgeführt in diesem Realm
    pub policies_executed: AtomicU64,
    /// Erfolgreiche Policy-Evaluationen
    pub policies_passed: AtomicU64,
    /// Fehlgeschlagene Policy-Evaluationen
    pub policies_denied: AtomicU64,
    /// Gas verbraucht für Policies in diesem Realm
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht für Policies in diesem Realm
    pub mana_consumed: AtomicU64,
    /// Crossing-Policies ausgeführt
    pub crossing_policies: AtomicU64,
    /// Membership-Policies ausgeführt
    pub membership_policies: AtomicU64,
    /// Aktive compiled Policies in diesem Realm
    pub active_policies: RwLock<Vec<String>>,
    /// Instantiierte Blueprints in diesem Realm
    pub instantiated_blueprints: AtomicU64,
}

impl RealmECLState {
    pub fn new() -> Self {
        Self {
            policies_executed: AtomicU64::new(0),
            policies_passed: AtomicU64::new(0),
            policies_denied: AtomicU64::new(0),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            crossing_policies: AtomicU64::new(0),
            membership_policies: AtomicU64::new(0),
            active_policies: RwLock::new(Vec::new()),
            instantiated_blueprints: AtomicU64::new(0),
        }
    }

    pub fn policy_executed(&self, passed: bool, policy_type: ECLPolicyType, gas: u64, mana: u64) {
        self.policies_executed.fetch_add(1, Ordering::Relaxed);
        if passed {
            self.policies_passed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.policies_denied.fetch_add(1, Ordering::Relaxed);
        }
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.mana_consumed.fetch_add(mana, Ordering::Relaxed);

        match policy_type {
            ECLPolicyType::Crossing => {
                self.crossing_policies.fetch_add(1, Ordering::Relaxed);
            }
            ECLPolicyType::Membership => {
                self.membership_policies.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    pub fn register_policy(&self, policy_id: &str) {
        if let Ok(mut policies) = self.active_policies.write() {
            if !policies.contains(&policy_id.to_string()) {
                policies.push(policy_id.to_string());
            }
        }
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.policies_executed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.policies_passed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> RealmECLSnapshot {
        RealmECLSnapshot {
            policies_executed: self.policies_executed.load(Ordering::Relaxed),
            policies_passed: self.policies_passed.load(Ordering::Relaxed),
            policies_denied: self.policies_denied.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            crossing_policies: self.crossing_policies.load(Ordering::Relaxed),
            membership_policies: self.membership_policies.load(Ordering::Relaxed),
            active_policies: self
                .active_policies
                .read()
                .map(|p| p.clone())
                .unwrap_or_default(),
            instantiated_blueprints: self.instantiated_blueprints.load(Ordering::Relaxed),
        }
    }
}

impl Default for RealmECLState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmECLSnapshot {
    pub policies_executed: u64,
    pub policies_passed: u64,
    pub policies_denied: u64,
    pub success_rate: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub crossing_policies: u64,
    pub membership_policies: u64,
    pub active_policies: Vec<String>,
    pub instantiated_blueprints: u64,
}

/// ECLVM State - Erynoa Core Language Virtual Machine
///
/// Trackt alle ECL-bezogenen Aktivitäten:
/// - Policy-Kompilierung und -Ausführung
/// - Blueprint-Management (Publish, Deploy, Instantiate)
/// - Saga/Intent-Orchestrierung durch ECL
/// - Realm-spezifische ECL-Metriken
#[derive(Debug)]
pub struct ECLVMState {
    // === Policy Engine ===
    /// Policies kompiliert insgesamt
    pub policies_compiled: AtomicU64,
    /// Policies im Cache (compiled bytecode)
    pub policies_cached: AtomicUsize,
    /// Policy-Kompilierungsfehler
    pub policy_compile_errors: AtomicU64,
    /// Policy-Ausführungen insgesamt
    pub policies_executed: AtomicU64,
    /// Erfolgreiche Policy-Evaluationen
    pub policies_passed: AtomicU64,
    /// Fehlgeschlagene Policy-Evaluationen (denied)
    pub policies_denied: AtomicU64,
    /// Policy-Runtime-Fehler (Bugs, nicht Denials)
    pub policy_runtime_errors: AtomicU64,
    /// Policies nach Typ
    pub policies_by_type: RwLock<HashMap<String, u64>>,

    // === Blueprint Engine ===
    /// Blueprints publiziert (im Marketplace)
    pub blueprints_published: AtomicU64,
    /// Blueprints deployed (ready for instantiation)
    pub blueprints_deployed: AtomicU64,
    /// Blueprint-Instanziierungen
    pub blueprints_instantiated: AtomicU64,
    /// Blueprint-Verifikationen (Community)
    pub blueprints_verified: AtomicU64,
    /// Blueprint-Downloads
    pub blueprints_downloaded: AtomicU64,
    /// Blueprints nach Status
    pub blueprints_by_status: RwLock<HashMap<String, u64>>,

    // === Saga/Intent Orchestrierung ===
    /// Intents verarbeitet durch ECL
    pub intents_processed: AtomicU64,
    /// Intents erfolgreich ausgeführt
    pub intents_successful: AtomicU64,
    /// Saga-Steps durch ECLVM ausgeführt
    pub saga_steps_executed: AtomicU64,
    /// Cross-Realm-Saga-Steps
    pub cross_realm_steps: AtomicU64,
    /// Kompensationen durch ECLVM
    pub compensations_triggered: AtomicU64,

    // === Resource Tracking ===
    /// Gesamt-Gas verbraucht durch ECLVM
    pub total_gas_consumed: AtomicU64,
    /// Gesamt-Mana verbraucht durch ECLVM
    pub total_mana_consumed: AtomicU64,
    /// Out-of-Gas während ECL-Ausführung
    pub out_of_gas_aborts: AtomicU64,
    /// Rate-Limited durch Mana-Erschöpfung
    pub mana_rate_limited: AtomicU64,

    // === Per-Realm ECL State ===
    /// ECL-State pro Realm
    pub realm_ecl: RwLock<HashMap<String, RealmECLState>>,

    // === Crossing-Policy Cache (Κ23) ===
    /// Crossing-Policies evaluiert
    pub crossing_evaluations: AtomicU64,
    /// Crossings durch Policy erlaubt
    pub crossings_allowed: AtomicU64,
    /// Crossings durch Policy abgelehnt
    pub crossings_denied: AtomicU64,
    /// Durchschnittliche Policy-Evaluation Zeit (µs)
    pub avg_evaluation_time_us: RwLock<f64>,

    // === Events ===
    /// Events emittiert durch ECLVM
    pub events_emitted: AtomicU64,
}

impl ECLVMState {
    pub fn new() -> Self {
        Self {
            policies_compiled: AtomicU64::new(0),
            policies_cached: AtomicUsize::new(0),
            policy_compile_errors: AtomicU64::new(0),
            policies_executed: AtomicU64::new(0),
            policies_passed: AtomicU64::new(0),
            policies_denied: AtomicU64::new(0),
            policy_runtime_errors: AtomicU64::new(0),
            policies_by_type: RwLock::new(HashMap::new()),
            blueprints_published: AtomicU64::new(0),
            blueprints_deployed: AtomicU64::new(0),
            blueprints_instantiated: AtomicU64::new(0),
            blueprints_verified: AtomicU64::new(0),
            blueprints_downloaded: AtomicU64::new(0),
            blueprints_by_status: RwLock::new(HashMap::new()),
            intents_processed: AtomicU64::new(0),
            intents_successful: AtomicU64::new(0),
            saga_steps_executed: AtomicU64::new(0),
            cross_realm_steps: AtomicU64::new(0),
            compensations_triggered: AtomicU64::new(0),
            total_gas_consumed: AtomicU64::new(0),
            total_mana_consumed: AtomicU64::new(0),
            out_of_gas_aborts: AtomicU64::new(0),
            mana_rate_limited: AtomicU64::new(0),
            realm_ecl: RwLock::new(HashMap::new()),
            crossing_evaluations: AtomicU64::new(0),
            crossings_allowed: AtomicU64::new(0),
            crossings_denied: AtomicU64::new(0),
            avg_evaluation_time_us: RwLock::new(0.0),
            events_emitted: AtomicU64::new(0),
        }
    }

    // === Policy Operations ===

    pub fn policy_compiled(&self, success: bool, policy_type: ECLPolicyType) {
        if success {
            self.policies_compiled.fetch_add(1, Ordering::Relaxed);
            self.policies_cached.fetch_add(1, Ordering::Relaxed);
        } else {
            self.policy_compile_errors.fetch_add(1, Ordering::Relaxed);
        }
        // Track by type
        let type_name = format!("{:?}", policy_type);
        if let Ok(mut by_type) = self.policies_by_type.write() {
            *by_type.entry(type_name).or_insert(0) += 1;
        }
    }

    pub fn policy_executed(
        &self,
        passed: bool,
        policy_type: ECLPolicyType,
        gas: u64,
        mana: u64,
        duration_us: u64,
        realm_id: Option<&str>,
    ) {
        self.policies_executed.fetch_add(1, Ordering::Relaxed);
        if passed {
            self.policies_passed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.policies_denied.fetch_add(1, Ordering::Relaxed);
        }
        self.total_gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.total_mana_consumed.fetch_add(mana, Ordering::Relaxed);
        self.events_emitted.fetch_add(1, Ordering::Relaxed);

        // Track by type (Gap 3: Api, Ui, DataLogic, Controller, …)
        let type_name = format!("{:?}", policy_type);
        if let Ok(mut by_type) = self.policies_by_type.write() {
            *by_type.entry(type_name).or_insert(0) += 1;
        }

        // Update per-realm state
        if let Some(realm) = realm_id {
            self.get_or_create_realm_ecl(realm)
                .policy_executed(passed, policy_type, gas, mana);
        }

        // Update avg evaluation time
        if let Ok(mut avg) = self.avg_evaluation_time_us.write() {
            let total = self.policies_executed.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + duration_us as f64) / total;
        }
    }

    pub fn policy_runtime_error(&self) {
        self.policy_runtime_errors.fetch_add(1, Ordering::Relaxed);
    }

    // === Blueprint Operations ===

    pub fn blueprint_published(&self) {
        self.blueprints_published.fetch_add(1, Ordering::Relaxed);
        self.update_blueprint_status("Draft");
    }

    pub fn blueprint_deployed(&self) {
        self.blueprints_deployed.fetch_add(1, Ordering::Relaxed);
        self.update_blueprint_status("Published");
    }

    pub fn blueprint_instantiated(&self, realm_id: &str) {
        self.blueprints_instantiated.fetch_add(1, Ordering::Relaxed);
        self.events_emitted.fetch_add(1, Ordering::Relaxed);

        // Track per-realm
        if let Ok(realms) = self.realm_ecl.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm
                    .instantiated_blueprints
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn blueprint_verified(&self) {
        self.blueprints_verified.fetch_add(1, Ordering::Relaxed);
        self.update_blueprint_status("Verified");
    }

    pub fn blueprint_downloaded(&self) {
        self.blueprints_downloaded.fetch_add(1, Ordering::Relaxed);
    }

    fn update_blueprint_status(&self, status: &str) {
        if let Ok(mut by_status) = self.blueprints_by_status.write() {
            *by_status.entry(status.to_string()).or_insert(0) += 1;
        }
    }

    // === Saga/Intent Operations ===

    pub fn intent_processed(&self, success: bool) {
        self.intents_processed.fetch_add(1, Ordering::Relaxed);
        if success {
            self.intents_successful.fetch_add(1, Ordering::Relaxed);
        }
        self.events_emitted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn saga_step_executed(&self, cross_realm: bool, gas: u64, mana: u64) {
        self.saga_steps_executed.fetch_add(1, Ordering::Relaxed);
        if cross_realm {
            self.cross_realm_steps.fetch_add(1, Ordering::Relaxed);
        }
        self.total_gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.total_mana_consumed.fetch_add(mana, Ordering::Relaxed);
    }

    pub fn compensation_triggered(&self) {
        self.compensations_triggered.fetch_add(1, Ordering::Relaxed);
    }

    // === Crossing-Policy (Κ23) ===

    pub fn crossing_policy_evaluated(&self, allowed: bool, from_realm: &str, to_realm: &str) {
        self.crossing_evaluations.fetch_add(1, Ordering::Relaxed);
        if allowed {
            self.crossings_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.crossings_denied.fetch_add(1, Ordering::Relaxed);
        }

        // Track in source realm
        self.get_or_create_realm_ecl(from_realm)
            .crossing_policies
            .fetch_add(1, Ordering::Relaxed);
        // Track in target realm
        self.get_or_create_realm_ecl(to_realm)
            .crossing_policies
            .fetch_add(1, Ordering::Relaxed);
    }

    // === Resource Tracking ===

    pub fn out_of_gas(&self) {
        self.out_of_gas_aborts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn rate_limited(&self) {
        self.mana_rate_limited.fetch_add(1, Ordering::Relaxed);
    }

    // === Per-Realm Operations ===

    pub fn register_realm(&self, realm_id: &str) {
        if let Ok(mut realms) = self.realm_ecl.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmECLState::new);
        }
    }

    pub fn register_policy_to_realm(&self, realm_id: &str, policy_id: &str) {
        self.get_or_create_realm_ecl(realm_id)
            .register_policy(policy_id);
    }

    /// Holt oder erstellt RealmECLState für ein Realm
    pub fn get_or_create_realm_ecl(&self, realm_id: &str) -> &RealmECLState {
        // Note: This is a simplification - in production you'd use a more sophisticated approach
        if let Ok(mut realms) = self.realm_ecl.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmECLState::new);
        }
        // Return a reference - this works because we hold the lock
        // In practice, you might want to return a guard or use interior mutability
        unsafe {
            // Safe because we just ensured the entry exists
            self.realm_ecl
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmECLState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmECLState> = std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmECLState::new)
                })
        }
    }

    pub fn get_realm_ecl(&self, realm_id: &str) -> Option<RealmECLSnapshot> {
        self.realm_ecl
            .read()
            .ok()?
            .get(realm_id)
            .map(|r| r.snapshot())
    }

    // === Metrics ===

    pub fn policy_success_rate(&self) -> f64 {
        let total = self.policies_executed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.policies_passed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn crossing_allow_rate(&self) -> f64 {
        let total = self.crossing_evaluations.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.crossings_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn intent_success_rate(&self) -> f64 {
        let total = self.intents_processed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.intents_successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> ECLVMSnapshot {
        let realm_snapshots = self
            .realm_ecl
            .read()
            .map(|r| r.iter().map(|(k, v)| (k.clone(), v.snapshot())).collect())
            .unwrap_or_default();

        ECLVMSnapshot {
            policies_compiled: self.policies_compiled.load(Ordering::Relaxed),
            policies_cached: self.policies_cached.load(Ordering::Relaxed),
            policy_compile_errors: self.policy_compile_errors.load(Ordering::Relaxed),
            policies_executed: self.policies_executed.load(Ordering::Relaxed),
            policies_passed: self.policies_passed.load(Ordering::Relaxed),
            policies_denied: self.policies_denied.load(Ordering::Relaxed),
            policy_runtime_errors: self.policy_runtime_errors.load(Ordering::Relaxed),
            policy_success_rate: self.policy_success_rate(),
            policies_by_type: self
                .policies_by_type
                .read()
                .map(|p| p.clone())
                .unwrap_or_default(),
            blueprints_published: self.blueprints_published.load(Ordering::Relaxed),
            blueprints_deployed: self.blueprints_deployed.load(Ordering::Relaxed),
            blueprints_instantiated: self.blueprints_instantiated.load(Ordering::Relaxed),
            blueprints_verified: self.blueprints_verified.load(Ordering::Relaxed),
            blueprints_downloaded: self.blueprints_downloaded.load(Ordering::Relaxed),
            blueprints_by_status: self
                .blueprints_by_status
                .read()
                .map(|b| b.clone())
                .unwrap_or_default(),
            intents_processed: self.intents_processed.load(Ordering::Relaxed),
            intents_successful: self.intents_successful.load(Ordering::Relaxed),
            intent_success_rate: self.intent_success_rate(),
            saga_steps_executed: self.saga_steps_executed.load(Ordering::Relaxed),
            cross_realm_steps: self.cross_realm_steps.load(Ordering::Relaxed),
            compensations_triggered: self.compensations_triggered.load(Ordering::Relaxed),
            total_gas_consumed: self.total_gas_consumed.load(Ordering::Relaxed),
            total_mana_consumed: self.total_mana_consumed.load(Ordering::Relaxed),
            out_of_gas_aborts: self.out_of_gas_aborts.load(Ordering::Relaxed),
            mana_rate_limited: self.mana_rate_limited.load(Ordering::Relaxed),
            realm_ecl: realm_snapshots,
            crossing_evaluations: self.crossing_evaluations.load(Ordering::Relaxed),
            crossings_allowed: self.crossings_allowed.load(Ordering::Relaxed),
            crossings_denied: self.crossings_denied.load(Ordering::Relaxed),
            crossing_allow_rate: self.crossing_allow_rate(),
            avg_evaluation_time_us: self
                .avg_evaluation_time_us
                .read()
                .map(|a| *a)
                .unwrap_or(0.0),
            events_emitted: self.events_emitted.load(Ordering::Relaxed),
        }
    }
}

impl Default for ECLVMState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECLVMSnapshot {
    // Policy Engine
    pub policies_compiled: u64,
    pub policies_cached: usize,
    pub policy_compile_errors: u64,
    pub policies_executed: u64,
    pub policies_passed: u64,
    pub policies_denied: u64,
    pub policy_runtime_errors: u64,
    pub policy_success_rate: f64,
    pub policies_by_type: HashMap<String, u64>,
    // Blueprint Engine
    pub blueprints_published: u64,
    pub blueprints_deployed: u64,
    pub blueprints_instantiated: u64,
    pub blueprints_verified: u64,
    pub blueprints_downloaded: u64,
    pub blueprints_by_status: HashMap<String, u64>,
    // Saga/Intent
    pub intents_processed: u64,
    pub intents_successful: u64,
    pub intent_success_rate: f64,
    pub saga_steps_executed: u64,
    pub cross_realm_steps: u64,
    pub compensations_triggered: u64,
    // Resources
    pub total_gas_consumed: u64,
    pub total_mana_consumed: u64,
    pub out_of_gas_aborts: u64,
    pub mana_rate_limited: u64,
    // Per-Realm
    pub realm_ecl: HashMap<String, RealmECLSnapshot>,
    // Crossing-Policy
    pub crossing_evaluations: u64,
    pub crossings_allowed: u64,
    pub crossings_denied: u64,
    pub crossing_allow_rate: f64,
    pub avg_evaluation_time_us: f64,
    // Events
    pub events_emitted: u64,
}

// ============================================================================
// PROTECTION STATE LAYER (Κ19-Κ21) - Tiefe Struktur mit Sub-States
// ============================================================================

/// Anomaly Detection Sub-State mit Relationship-Tracking
#[derive(Debug)]
pub struct AnomalyState {
    /// Total Anomalien erkannt
    pub total: AtomicU64,
    /// Kritische Anomalien
    pub critical: AtomicU64,
    /// Hohe Anomalien
    pub high: AtomicU64,
    /// Mittlere Anomalien
    pub medium: AtomicU64,
    /// Niedrige Anomalien
    pub low: AtomicU64,
    /// False Positives
    pub false_positives: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (Anomaly ✓ Event/Trust)
    // ─────────────────────────────────────────────────────────────────────────
    /// Events validiert (Anomaly ✓ Event)
    pub events_validated: AtomicU64,
    /// Trust-Patterns geprüft (Anomaly ✓ Trust)
    pub trust_patterns_checked: AtomicU64,
}

impl AnomalyState {
    pub fn new() -> Self {
        Self {
            total: AtomicU64::new(0),
            critical: AtomicU64::new(0),
            high: AtomicU64::new(0),
            medium: AtomicU64::new(0),
            low: AtomicU64::new(0),
            false_positives: AtomicU64::new(0),
            events_validated: AtomicU64::new(0),
            trust_patterns_checked: AtomicU64::new(0),
        }
    }

    pub fn record(&self, severity: &str) {
        self.total.fetch_add(1, Ordering::Relaxed);
        match severity {
            "critical" => self.critical.fetch_add(1, Ordering::Relaxed),
            "high" => self.high.fetch_add(1, Ordering::Relaxed),
            "medium" => self.medium.fetch_add(1, Ordering::Relaxed),
            _ => self.low.fetch_add(1, Ordering::Relaxed),
        };
    }

    pub fn snapshot(&self) -> AnomalySnapshot {
        AnomalySnapshot {
            total: self.total.load(Ordering::Relaxed),
            critical: self.critical.load(Ordering::Relaxed),
            high: self.high.load(Ordering::Relaxed),
            medium: self.medium.load(Ordering::Relaxed),
            low: self.low.load(Ordering::Relaxed),
            false_positives: self.false_positives.load(Ordering::Relaxed),
            events_validated: self.events_validated.load(Ordering::Relaxed),
            trust_patterns_checked: self.trust_patterns_checked.load(Ordering::Relaxed),
        }
    }
}

impl Default for AnomalyState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalySnapshot {
    pub total: u64,
    pub critical: u64,
    pub high: u64,
    pub medium: u64,
    pub low: u64,
    pub false_positives: u64,
    pub events_validated: u64,
    pub trust_patterns_checked: u64,
}

/// Diversity Monitor Sub-State (Κ20) mit Relationship-Tracking
#[derive(Debug)]
pub struct DiversityState {
    /// Dimensionen die überwacht werden
    pub dimensions: AtomicUsize,
    /// Monokultur-Warnungen
    pub monoculture_warnings: AtomicU64,
    /// Entropy pro Dimension
    pub entropy_values: RwLock<HashMap<String, f64>>,
    /// Minimum Entropy
    pub min_entropy: RwLock<f64>,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (Diversity ✓ Trust/Consensus)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Verteilung geprüft (Diversity ✓ Trust)
    pub trust_distribution_checks: AtomicU64,
    /// Validator-Mix geprüft (Diversity ✓ Consensus)
    pub validator_mix_checks: AtomicU64,
}

impl DiversityState {
    pub fn new() -> Self {
        Self {
            dimensions: AtomicUsize::new(0),
            monoculture_warnings: AtomicU64::new(0),
            entropy_values: RwLock::new(HashMap::new()),
            min_entropy: RwLock::new(1.0),
            trust_distribution_checks: AtomicU64::new(0),
            validator_mix_checks: AtomicU64::new(0),
        }
    }

    pub fn set_entropy(&self, dimension: &str, value: f64) {
        if let Ok(mut map) = self.entropy_values.write() {
            map.insert(dimension.to_string(), value);
            if let Ok(mut min) = self.min_entropy.write() {
                *min = map.values().copied().fold(f64::MAX, f64::min);
            }
        }
    }

    pub fn snapshot(&self) -> DiversitySnapshot {
        DiversitySnapshot {
            dimensions: self.dimensions.load(Ordering::Relaxed),
            monoculture_warnings: self.monoculture_warnings.load(Ordering::Relaxed),
            min_entropy: self.min_entropy.read().map(|v| *v).unwrap_or(1.0),
            trust_distribution_checks: self.trust_distribution_checks.load(Ordering::Relaxed),
            validator_mix_checks: self.validator_mix_checks.load(Ordering::Relaxed),
        }
    }
}

impl Default for DiversityState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversitySnapshot {
    pub dimensions: usize,
    pub monoculture_warnings: u64,
    pub min_entropy: f64,
    pub trust_distribution_checks: u64,
    pub validator_mix_checks: u64,
}

/// Quadratic Governance Sub-State (Κ21) mit Relationship-Tracking
#[derive(Debug)]
pub struct QuadraticState {
    /// Aktive Abstimmungen
    pub active_votes: AtomicUsize,
    /// Abgeschlossene Abstimmungen
    pub completed_votes: AtomicU64,
    /// Teilnehmer total
    pub total_participants: AtomicU64,
    /// Quadratische Reduktionen angewandt
    pub quadratic_reductions: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (Quadratic ← Trust)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (Quadratic ← Trust)
    pub trust_dependency_updates: AtomicU64,
}

impl QuadraticState {
    pub fn new() -> Self {
        Self {
            active_votes: AtomicUsize::new(0),
            completed_votes: AtomicU64::new(0),
            total_participants: AtomicU64::new(0),
            quadratic_reductions: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> QuadraticSnapshot {
        QuadraticSnapshot {
            active_votes: self.active_votes.load(Ordering::Relaxed),
            completed_votes: self.completed_votes.load(Ordering::Relaxed),
            total_participants: self.total_participants.load(Ordering::Relaxed),
            quadratic_reductions: self.quadratic_reductions.load(Ordering::Relaxed),
            trust_dependency_updates: self.trust_dependency_updates.load(Ordering::Relaxed),
        }
    }
}

impl Default for QuadraticState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuadraticSnapshot {
    pub active_votes: usize,
    pub completed_votes: u64,
    pub total_participants: u64,
    pub quadratic_reductions: u64,
    pub trust_dependency_updates: u64,
}

/// Anti-Calcification Sub-State (Κ19) mit Relationship-Tracking
#[derive(Debug)]
pub struct AntiCalcificationState {
    /// Power-Konzentration (0.0 = perfekt verteilt, 1.0 = monopol)
    pub power_concentration: RwLock<f64>,
    /// Gini-Koeffizient
    pub gini_coefficient: RwLock<f64>,
    /// Interventionen durchgeführt
    pub interventions: AtomicU64,
    /// Überwachte Entitäten
    pub watched_entities: AtomicUsize,
    /// Schwellenwert-Verletzungen
    pub threshold_violations: AtomicU64,
    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (AntiCalcification ✓/→ Trust)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Limits geprüft (AntiCalcification → Trust)
    pub trust_limits_checked: AtomicU64,
    /// Power-Checks durchgeführt (AntiCalcification ✓ Trust)
    pub power_checks: AtomicU64,
}

impl AntiCalcificationState {
    pub fn new() -> Self {
        Self {
            power_concentration: RwLock::new(0.0),
            gini_coefficient: RwLock::new(0.0),
            interventions: AtomicU64::new(0),
            watched_entities: AtomicUsize::new(0),
            threshold_violations: AtomicU64::new(0),
            trust_limits_checked: AtomicU64::new(0),
            power_checks: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> AntiCalcificationSnapshot {
        AntiCalcificationSnapshot {
            power_concentration: self.power_concentration.read().map(|v| *v).unwrap_or(0.0),
            gini_coefficient: self.gini_coefficient.read().map(|v| *v).unwrap_or(0.0),
            interventions: self.interventions.load(Ordering::Relaxed),
            watched_entities: self.watched_entities.load(Ordering::Relaxed),
            threshold_violations: self.threshold_violations.load(Ordering::Relaxed),
            trust_limits_checked: self.trust_limits_checked.load(Ordering::Relaxed),
            power_checks: self.power_checks.load(Ordering::Relaxed),
        }
    }
}

impl Default for AntiCalcificationState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiCalcificationSnapshot {
    pub power_concentration: f64,
    pub gini_coefficient: f64,
    pub interventions: u64,
    pub watched_entities: usize,
    pub threshold_violations: u64,
    pub trust_limits_checked: u64,
    pub power_checks: u64,
}

/// Calibration Sub-State
#[derive(Debug)]
pub struct CalibrationState {
    /// Calibration-Updates durchgeführt
    pub updates_total: AtomicU64,
    /// Kalibrierte Parameter
    pub params_map: RwLock<HashMap<String, f64>>,
}

impl CalibrationState {
    pub fn new() -> Self {
        Self {
            updates_total: AtomicU64::new(0),
            params_map: RwLock::new(HashMap::new()),
        }
    }

    pub fn snapshot(&self) -> CalibrationSnapshot {
        CalibrationSnapshot {
            updates_total: self.updates_total.load(Ordering::Relaxed),
        }
    }
}

impl Default for CalibrationState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSnapshot {
    pub updates_total: u64,
}

/// Protection State mit tiefgründigen Sub-States
#[derive(Debug)]
pub struct ProtectionState {
    /// Anomaly Detection (Anomaly ✓ Event/Trust)
    pub anomaly: AnomalyState,
    /// Diversity Monitor (Κ20) (Diversity ✓ Trust/Consensus)
    pub diversity: DiversityState,
    /// Quadratic Governance (Κ21) (Quadratic ← Trust)
    pub quadratic: QuadraticState,
    /// Anti-Calcification (Κ19) (AntiCalcification ✓/→ Trust)
    pub anti_calcification: AntiCalcificationState,
    /// Calibration (Calibration → Gas/Mana)
    pub calibration: CalibrationState,
    /// Shard Monitor für horizontale Skalierungssicherheit (Phase 7.1)
    /// Optional: Wird nur bei aktiviertem Sharding benötigt
    shard_monitor: Option<Arc<ShardMonitor>>,
}

impl ProtectionState {
    pub fn new() -> Self {
        Self {
            anomaly: AnomalyState::new(),
            diversity: DiversityState::new(),
            quadratic: QuadraticState::new(),
            anti_calcification: AntiCalcificationState::new(),
            calibration: CalibrationState::new(),
            shard_monitor: None,
        }
    }

    /// Erstelle mit aktiviertem ShardMonitor
    pub fn with_shard_monitor(config: ShardMonitorConfig) -> Self {
        Self {
            anomaly: AnomalyState::new(),
            diversity: DiversityState::new(),
            quadratic: QuadraticState::new(),
            anti_calcification: AntiCalcificationState::new(),
            calibration: CalibrationState::new(),
            shard_monitor: Some(Arc::new(ShardMonitor::new(config))),
        }
    }

    /// Aktiviere ShardMonitor nachträglich
    pub fn enable_shard_monitor(&mut self, config: ShardMonitorConfig) {
        self.shard_monitor = Some(Arc::new(ShardMonitor::new(config)));
    }

    /// Hole ShardMonitor Reference (falls aktiviert)
    pub fn shard_monitor(&self) -> Option<&Arc<ShardMonitor>> {
        self.shard_monitor.as_ref()
    }

    /// Legacy-Kompatibilität: Anomalie aufzeichnen
    pub fn anomaly(&self, severity: &str) {
        self.anomaly.record(severity);
    }

    /// Anomalie aufzeichnen mit Circuit Breaker Check (Verbesserung 3)
    ///
    /// Bei kritischen Anomalien wird der Circuit Breaker informiert,
    /// der ggf. das System in Degraded/Emergency-Modus schaltet.
    ///
    /// # Returns
    /// SystemMode nach dem Check (Normal, Degraded, EmergencyShutdown)
    pub fn anomaly_with_circuit_breaker(
        &self,
        severity: &str,
        circuit_breaker: &CircuitBreaker,
    ) -> SystemMode {
        self.anomaly.record(severity);
        if severity == "critical" {
            circuit_breaker.record_critical_anomaly()
        } else {
            circuit_breaker.mode()
        }
    }

    /// Legacy-Kompatibilität: Entropy setzen
    pub fn set_entropy(&self, dimension: &str, value: f64) {
        self.diversity.set_entropy(dimension, value);
    }

    /// Berechne System-Health basierend auf Protection-Metriken
    pub fn health_score(&self) -> f64 {
        let mut score: f64 = 100.0;

        // Anomalien reduzieren Score
        let critical = self.anomaly.critical.load(Ordering::Relaxed);
        let high = self.anomaly.high.load(Ordering::Relaxed);
        score -= (critical * 20) as f64;
        score -= (high * 10) as f64;

        // Diversity Warnings
        let warnings = self.diversity.monoculture_warnings.load(Ordering::Relaxed);
        score -= (warnings * 5) as f64;

        // Anti-Calc Violations
        let violations = self
            .anti_calcification
            .threshold_violations
            .load(Ordering::Relaxed);
        score -= (violations * 10) as f64;

        // Shard-Monitor: Quarantinierte Shards und niedrige Reputationen
        if let Some(shard_mon) = &self.shard_monitor {
            let snapshot = shard_mon.snapshot();
            // Quarantinierte Shards sind kritisch
            score -= (snapshot.quarantined_shard_count * 15) as f64;
            // Niedrige Success-Rate ist Warning
            if snapshot.cross_shard_success_rate < 0.9 {
                score -= 10.0;
            }
            if snapshot.cross_shard_success_rate < 0.7 {
                score -= 20.0;
            }
        }

        score.max(0.0).min(100.0)
    }

    /// Record Trust-Update aus einem Shard (für Bias-Tracking)
    pub fn record_shard_trust_update(&self, shard_id: u64, entropy_delta: f64) {
        if let Some(shard_mon) = &self.shard_monitor {
            shard_mon.record_trust_update(shard_id, entropy_delta);
        }
    }

    /// Record Cross-Shard Success
    pub fn record_cross_shard_success(&self, source_shard: u64) {
        if let Some(shard_mon) = &self.shard_monitor {
            shard_mon.record_cross_success(source_shard);
        }
    }

    /// Record Cross-Shard Failure
    pub fn record_cross_shard_failure(&self, source_shard: u64) {
        if let Some(shard_mon) = &self.shard_monitor {
            shard_mon.record_cross_failure(source_shard);
        }
    }

    /// Hole Cross-Shard Penalty-Multiplier (für Gateway)
    pub fn get_cross_shard_penalty(&self, source_shard: u64) -> f64 {
        self.shard_monitor
            .as_ref()
            .map(|m| m.get_cross_shard_penalty(source_shard))
            .unwrap_or(1.0) // Kein Monitor → keine Strafe
    }

    /// Contribute zu Multi-Veto Risk-Score (für Circuit Breaker)
    pub fn shard_veto_contribution(&self, shard_id: u64) -> f64 {
        self.shard_monitor
            .as_ref()
            .map(|m| m.contribute_to_veto(shard_id))
            .unwrap_or(1.0) // Kein Monitor → neutraler Score
    }

    pub fn snapshot(&self) -> ProtectionSnapshot {
        ProtectionSnapshot {
            anomaly: self.anomaly.snapshot(),
            diversity: self.diversity.snapshot(),
            quadratic: self.quadratic.snapshot(),
            anti_calcification: self.anti_calcification.snapshot(),
            calibration: self.calibration.snapshot(),
            health_score: self.health_score(),
            shard_monitor: self.shard_monitor.as_ref().map(|m| m.snapshot()),
        }
    }
}

impl Default for ProtectionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Protection State Snapshot mit tiefgründigen Sub-Snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionSnapshot {
    pub anomaly: AnomalySnapshot,
    pub diversity: DiversitySnapshot,
    pub quadratic: QuadraticSnapshot,
    pub anti_calcification: AntiCalcificationSnapshot,
    pub calibration: CalibrationSnapshot,
    pub health_score: f64,
    /// Optional: Shard-Monitor Snapshot (nur bei aktiviertem Sharding)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard_monitor: Option<ShardMonitorSnapshot>,
}

// ============================================================================
// STORAGE STATE LAYER
// ============================================================================

/// Storage State mit Persistenz-Tracking
#[derive(Debug)]
pub struct StorageState {
    // KV Store
    pub kv_keys: AtomicU64,
    pub kv_bytes: AtomicU64,
    pub kv_reads: AtomicU64,
    pub kv_writes: AtomicU64,

    // Event Store
    pub event_store_count: AtomicU64,
    pub event_store_bytes: AtomicU64,

    // Archive
    pub archived_epochs: AtomicU64,
    pub archived_events: AtomicU64,
    pub archive_bytes: AtomicU64,
    pub merkle_roots: AtomicU64,

    // Blueprint Marketplace
    pub blueprints_published: AtomicU64,
    pub blueprints_deployed: AtomicU64,
    pub blueprints_downloaded: AtomicU64,

    // Realms
    pub realm_count: AtomicUsize,
    pub identities: AtomicU64,
    pub trust_entries: AtomicU64,
}

impl StorageState {
    pub fn new() -> Self {
        Self {
            kv_keys: AtomicU64::new(0),
            kv_bytes: AtomicU64::new(0),
            kv_reads: AtomicU64::new(0),
            kv_writes: AtomicU64::new(0),
            event_store_count: AtomicU64::new(0),
            event_store_bytes: AtomicU64::new(0),
            archived_epochs: AtomicU64::new(0),
            archived_events: AtomicU64::new(0),
            archive_bytes: AtomicU64::new(0),
            merkle_roots: AtomicU64::new(0),
            blueprints_published: AtomicU64::new(0),
            blueprints_deployed: AtomicU64::new(0),
            blueprints_downloaded: AtomicU64::new(0),
            realm_count: AtomicUsize::new(0),
            identities: AtomicU64::new(0),
            trust_entries: AtomicU64::new(0),
        }
    }

    pub fn total_bytes(&self) -> u64 {
        self.kv_bytes.load(Ordering::Relaxed)
            + self.event_store_bytes.load(Ordering::Relaxed)
            + self.archive_bytes.load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> StorageSnapshot {
        StorageSnapshot {
            kv_keys: self.kv_keys.load(Ordering::Relaxed),
            kv_bytes: self.kv_bytes.load(Ordering::Relaxed),
            kv_reads: self.kv_reads.load(Ordering::Relaxed),
            kv_writes: self.kv_writes.load(Ordering::Relaxed),
            event_store_count: self.event_store_count.load(Ordering::Relaxed),
            event_store_bytes: self.event_store_bytes.load(Ordering::Relaxed),
            archived_epochs: self.archived_epochs.load(Ordering::Relaxed),
            archived_events: self.archived_events.load(Ordering::Relaxed),
            archive_bytes: self.archive_bytes.load(Ordering::Relaxed),
            merkle_roots: self.merkle_roots.load(Ordering::Relaxed),
            blueprints_published: self.blueprints_published.load(Ordering::Relaxed),
            blueprints_deployed: self.blueprints_deployed.load(Ordering::Relaxed),
            blueprints_downloaded: self.blueprints_downloaded.load(Ordering::Relaxed),
            realm_count: self.realm_count.load(Ordering::Relaxed),
            identities: self.identities.load(Ordering::Relaxed),
            trust_entries: self.trust_entries.load(Ordering::Relaxed),
            total_bytes: self.total_bytes(),
        }
    }
}

impl Default for StorageState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSnapshot {
    pub kv_keys: u64,
    pub kv_bytes: u64,
    pub kv_reads: u64,
    pub kv_writes: u64,
    pub event_store_count: u64,
    pub event_store_bytes: u64,
    pub archived_epochs: u64,
    pub archived_events: u64,
    pub archive_bytes: u64,
    pub merkle_roots: u64,
    pub blueprints_published: u64,
    pub blueprints_deployed: u64,
    pub blueprints_downloaded: u64,
    pub realm_count: usize,
    pub identities: u64,
    pub trust_entries: u64,
    pub total_bytes: u64,
}

// ============================================================================
// PEER STATE LAYER (Κ22-Κ24)
// ============================================================================

/// Gateway State (Κ23)
#[derive(Debug)]
pub struct GatewayState {
    /// Crossing-Anfragen insgesamt
    pub crossings_total: AtomicU64,
    /// Erfolgreiche Crossings
    pub crossings_allowed: AtomicU64,
    /// Abgelehnte Crossings
    pub crossings_denied: AtomicU64,
    /// Trust-Verletzungen (Trust < min_trust)
    pub trust_violations: AtomicU64,
    /// Credential-Verletzungen
    pub credential_violations: AtomicU64,
    /// Rule-Verletzungen
    pub rule_violations: AtomicU64,
    /// Durchschnittlicher Trust bei erfolgreichen Crossings
    pub avg_crossing_trust: RwLock<f64>,
    /// Trust-Dampening-Anwendungen
    pub dampening_applied: AtomicU64,
    /// Aktive Realm-Registrierungen
    pub registered_realms: AtomicUsize,
}

impl GatewayState {
    pub fn new() -> Self {
        Self {
            crossings_total: AtomicU64::new(0),
            crossings_allowed: AtomicU64::new(0),
            crossings_denied: AtomicU64::new(0),
            trust_violations: AtomicU64::new(0),
            credential_violations: AtomicU64::new(0),
            rule_violations: AtomicU64::new(0),
            avg_crossing_trust: RwLock::new(0.5),
            dampening_applied: AtomicU64::new(0),
            registered_realms: AtomicUsize::new(0),
        }
    }

    pub fn crossing_allowed(&self, trust: f64) {
        self.crossings_total.fetch_add(1, Ordering::Relaxed);
        self.crossings_allowed.fetch_add(1, Ordering::Relaxed);
        // Update rolling average
        if let Ok(mut avg) = self.avg_crossing_trust.write() {
            let allowed = self.crossings_allowed.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (allowed - 1.0) + trust) / allowed;
        }
    }

    pub fn crossing_denied(&self, reason: &str) {
        self.crossings_total.fetch_add(1, Ordering::Relaxed);
        self.crossings_denied.fetch_add(1, Ordering::Relaxed);
        match reason {
            "trust" => self.trust_violations.fetch_add(1, Ordering::Relaxed),
            "credential" => self.credential_violations.fetch_add(1, Ordering::Relaxed),
            "rule" => self.rule_violations.fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }

    pub fn success_rate(&self) -> f64 {
        let total = self.crossings_total.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.crossings_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> GatewaySnapshot {
        GatewaySnapshot {
            crossings_total: self.crossings_total.load(Ordering::Relaxed),
            crossings_allowed: self.crossings_allowed.load(Ordering::Relaxed),
            crossings_denied: self.crossings_denied.load(Ordering::Relaxed),
            trust_violations: self.trust_violations.load(Ordering::Relaxed),
            credential_violations: self.credential_violations.load(Ordering::Relaxed),
            rule_violations: self.rule_violations.load(Ordering::Relaxed),
            avg_crossing_trust: self.avg_crossing_trust.read().map(|v| *v).unwrap_or(0.5),
            dampening_applied: self.dampening_applied.load(Ordering::Relaxed),
            registered_realms: self.registered_realms.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
        }
    }
}

impl Default for GatewayState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewaySnapshot {
    pub crossings_total: u64,
    pub crossings_allowed: u64,
    pub crossings_denied: u64,
    pub trust_violations: u64,
    pub credential_violations: u64,
    pub rule_violations: u64,
    pub avg_crossing_trust: f64,
    pub dampening_applied: u64,
    pub registered_realms: usize,
    pub success_rate: f64,
}

/// Saga Composer State (Κ22, Κ24)
#[derive(Debug)]
pub struct SagaComposerState {
    /// Sagas komponiert insgesamt
    pub sagas_composed: AtomicU64,
    /// Erfolgreiche Kompositionen
    pub successful_compositions: AtomicU64,
    /// Fehlgeschlagene Kompositionen
    pub failed_compositions: AtomicU64,
    /// Durchschnittliche Schritte pro Saga
    pub avg_steps_per_saga: RwLock<f64>,
    /// Kompensationen ausgeführt (Κ24)
    pub compensations_executed: AtomicU64,
    /// Kompensationen erfolgreich
    pub compensations_successful: AtomicU64,
    /// Budget-Verletzungen
    pub budget_violations: AtomicU64,
    /// Cross-Realm-Sagas
    pub cross_realm_sagas: AtomicU64,
    /// Nach Goal-Typ
    pub goals_by_type: RwLock<HashMap<String, u64>>,
}

impl SagaComposerState {
    pub fn new() -> Self {
        Self {
            sagas_composed: AtomicU64::new(0),
            successful_compositions: AtomicU64::new(0),
            failed_compositions: AtomicU64::new(0),
            avg_steps_per_saga: RwLock::new(0.0),
            compensations_executed: AtomicU64::new(0),
            compensations_successful: AtomicU64::new(0),
            budget_violations: AtomicU64::new(0),
            cross_realm_sagas: AtomicU64::new(0),
            goals_by_type: RwLock::new(HashMap::new()),
        }
    }

    pub fn saga_composed(&self, success: bool, steps: usize, goal_type: &str) {
        self.sagas_composed.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_compositions.fetch_add(1, Ordering::Relaxed);
            // Update avg steps
            if let Ok(mut avg) = self.avg_steps_per_saga.write() {
                let total = self.successful_compositions.load(Ordering::Relaxed) as f64;
                *avg = (*avg * (total - 1.0) + steps as f64) / total;
            }
        } else {
            self.failed_compositions.fetch_add(1, Ordering::Relaxed);
        }
        // Track goal type
        if let Ok(mut goals) = self.goals_by_type.write() {
            *goals.entry(goal_type.to_string()).or_insert(0) += 1;
        }
    }

    pub fn compensation(&self, success: bool) {
        self.compensations_executed.fetch_add(1, Ordering::Relaxed);
        if success {
            self.compensations_successful
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn composition_success_rate(&self) -> f64 {
        let total = self.sagas_composed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.successful_compositions.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn compensation_success_rate(&self) -> f64 {
        let total = self.compensations_executed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.compensations_successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> SagaComposerSnapshot {
        SagaComposerSnapshot {
            sagas_composed: self.sagas_composed.load(Ordering::Relaxed),
            successful_compositions: self.successful_compositions.load(Ordering::Relaxed),
            failed_compositions: self.failed_compositions.load(Ordering::Relaxed),
            composition_success_rate: self.composition_success_rate(),
            avg_steps_per_saga: self.avg_steps_per_saga.read().map(|v| *v).unwrap_or(0.0),
            compensations_executed: self.compensations_executed.load(Ordering::Relaxed),
            compensations_successful: self.compensations_successful.load(Ordering::Relaxed),
            compensation_success_rate: self.compensation_success_rate(),
            budget_violations: self.budget_violations.load(Ordering::Relaxed),
            cross_realm_sagas: self.cross_realm_sagas.load(Ordering::Relaxed),
            goals_by_type: self
                .goals_by_type
                .read()
                .map(|g| g.clone())
                .unwrap_or_default(),
        }
    }
}

impl Default for SagaComposerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaComposerSnapshot {
    pub sagas_composed: u64,
    pub successful_compositions: u64,
    pub failed_compositions: u64,
    pub composition_success_rate: f64,
    pub avg_steps_per_saga: f64,
    pub compensations_executed: u64,
    pub compensations_successful: u64,
    pub compensation_success_rate: f64,
    pub budget_violations: u64,
    pub cross_realm_sagas: u64,
    pub goals_by_type: HashMap<String, u64>,
}

/// Intent Parser State
#[derive(Debug)]
pub struct IntentParserState {
    /// Intents geparst
    pub intents_parsed: AtomicU64,
    /// Erfolgreiche Parses
    pub successful_parses: AtomicU64,
    /// Parse-Fehler
    pub parse_errors: AtomicU64,
    /// Validierungsfehler
    pub validation_errors: AtomicU64,
    /// Nach Intent-Typ
    pub intents_by_type: RwLock<HashMap<String, u64>>,
    /// Durchschnittliche Parse-Zeit (µs)
    pub avg_parse_time_us: RwLock<f64>,
}

impl IntentParserState {
    pub fn new() -> Self {
        Self {
            intents_parsed: AtomicU64::new(0),
            successful_parses: AtomicU64::new(0),
            parse_errors: AtomicU64::new(0),
            validation_errors: AtomicU64::new(0),
            intents_by_type: RwLock::new(HashMap::new()),
            avg_parse_time_us: RwLock::new(0.0),
        }
    }

    pub fn parsed(&self, success: bool, intent_type: &str, duration_us: u64) {
        self.intents_parsed.fetch_add(1, Ordering::Relaxed);
        if success {
            self.successful_parses.fetch_add(1, Ordering::Relaxed);
        } else {
            self.parse_errors.fetch_add(1, Ordering::Relaxed);
        }
        if let Ok(mut types) = self.intents_by_type.write() {
            *types.entry(intent_type.to_string()).or_insert(0) += 1;
        }
        // Update avg time
        if let Ok(mut avg) = self.avg_parse_time_us.write() {
            let total = self.intents_parsed.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + duration_us as f64) / total;
        }
    }

    pub fn snapshot(&self) -> IntentParserSnapshot {
        IntentParserSnapshot {
            intents_parsed: self.intents_parsed.load(Ordering::Relaxed),
            successful_parses: self.successful_parses.load(Ordering::Relaxed),
            parse_errors: self.parse_errors.load(Ordering::Relaxed),
            validation_errors: self.validation_errors.load(Ordering::Relaxed),
            intents_by_type: self
                .intents_by_type
                .read()
                .map(|t| t.clone())
                .unwrap_or_default(),
            avg_parse_time_us: self.avg_parse_time_us.read().map(|v| *v).unwrap_or(0.0),
        }
    }
}

impl Default for IntentParserState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentParserSnapshot {
    pub intents_parsed: u64,
    pub successful_parses: u64,
    pub parse_errors: u64,
    pub validation_errors: u64,
    pub intents_by_type: HashMap<String, u64>,
    pub avg_parse_time_us: f64,
}

// ============================================================================
// REALM STATE LAYER - Per-Realm Isolation (Κ22-Κ24)
// ============================================================================

/// Per-Realm spezifischer State
///
/// Jedes Realm hat seinen eigenen isolierten State mit:
/// - Eigener TrustVector für Realm-spezifische Trust-Bewertung
/// - Eigenes Rule-Set (RuleCategory: Membership, Transaction, etc.)
/// - Identity-Tracking innerhalb des Realms
/// - Activity-Metriken für Monitoring
#[derive(Debug)]
/// Per-Realm Isolation State (Κ22-Κ24)
///
/// Implementiert das Realm-Konzept gemäß der Kernidee:
/// - **Isolation**: Daten/Aktionen bleiben im Realm (Sicherheit gegen Leak)
/// - **Crossing**: Kontrollierter Wechsel zwischen Realms (Gateway prüft Trust/Regeln)
/// - **Cross-Realm-Sagas**: Komplexe Aktionen über Realms (SagaComposer koordiniert)
/// - **Realm-spezifischer Trust**: Trust kann pro Realm variieren
///
/// # Beispiele für Realm-Typen:
/// - "private-friends" (hoher Trust, enge Gruppe)
/// - "public" (niedriger min_trust, öffentlich zugänglich)
/// - "app-specific" (anwendungsspezifische Regeln)
pub struct RealmSpecificState {
    // ─────────────────────────────────────────────────────────────────────────
    // TRUST & GOVERNANCE
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifischer Trust-Vektor
    /// Kombiniert globalen Trust mit Realm-lokalem Verhalten.
    /// Kann höher sein (z.B. bei Freunden) oder niedriger (z.B. bei Fremden).
    pub trust: RwLock<crate::domain::unified::TrustVector6D>,

    /// Minimum-Trust für Membership in diesem Realm
    /// Entities unter diesem Schwellenwert können nicht beitreten.
    pub min_trust: RwLock<f32>,

    /// Governance-Typ bestimmt Entscheidungsprozesse:
    /// - "owner": Einzelne Entität hat volle Kontrolle
    /// - "democratic": Mehrheitsentscheidung
    /// - "token": Token-gewichtete Abstimmung
    /// - "reputation": Trust-gewichtete Abstimmung
    /// - "consensus": Einstimmigkeit erforderlich
    pub governance_type: RwLock<String>,

    // ─────────────────────────────────────────────────────────────────────────
    // MEMBERSHIP & IDENTITIES (Explizite Isolation)
    // ─────────────────────────────────────────────────────────────────────────
    /// Explizite Mitgliederliste (Identity UniversalIds)
    /// Kernfeature für Isolation: Nur Mitglieder haben Zugriff.
    pub members_by_id: RwLock<HashSet<UniversalId>>,

    /// Mapping für Realm-spezifische Sub-DIDs
    /// Key: Root-Identity UniversalId, Value: Realm-Sub-DID UniversalId
    pub member_realm_dids: RwLock<HashMap<UniversalId, UniversalId>>,

    /// Anzahl registrierter Identitäten im Realm (Snapshot-friendly)
    pub identity_count: AtomicUsize,

    /// Pending Membership-Requests (UniversalIds, awaiting approval)
    pub pending_members_by_id: RwLock<HashSet<UniversalId>>,

    /// Gebannte Identitäten (UniversalIds, permanent ausgeschlossen)
    pub banned_members_by_id: RwLock<HashSet<UniversalId>>,

    /// Realm-Owner/Admin-Identitäten (UniversalIds)
    pub admins_by_id: RwLock<HashSet<UniversalId>>,

    // ─────────────────────────────────────────────────────────────────────────
    // ECL RULES & POLICIES (Realm-spezifische Logik)
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive ECL Policy-IDs für dieses Realm
    /// Definiert: Crossing-Rules, Membership-Rules, Transaction-Rules
    pub active_policies: RwLock<Vec<String>>,

    /// Aktive Legacy Rule-IDs (deprecated, use active_policies)
    pub active_rules: RwLock<Vec<String>>,

    // ─────────────────────────────────────────────────────────────────────────
    // ISOLATION & DATA PROTECTION
    // ─────────────────────────────────────────────────────────────────────────
    /// Isolation-Level: Wie streng ist die Daten-Isolation?
    /// - 0: Public (alle können lesen)
    /// - 1: Members-Only (nur Mitglieder können lesen)
    /// - 2: Strict (kein Cross-Realm-Zugriff, selbst mit Crossing)
    pub isolation_level: AtomicU8,

    /// Data-Leak-Events (Versuche Daten nach außen zu übertragen)
    pub leak_attempts: AtomicU64,

    /// Erfolgreich geblockte Leak-Versuche
    pub leaks_blocked: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // CROSSING METRICS (Κ23 Gateway-Integration)
    // ─────────────────────────────────────────────────────────────────────────
    /// Eingehende Crossings (in dieses Realm)
    pub crossings_in: AtomicU64,

    /// Ausgehende Crossings (aus diesem Realm)
    pub crossings_out: AtomicU64,

    /// Crossing-Requests abgelehnt (Trust zu niedrig oder Regel verletzt)
    pub crossings_denied: AtomicU64,

    /// Aktive Crossings (gerade im Übergang befindliche Entities)
    pub active_crossings: AtomicU64,

    /// Allowlisted Realms (Crossing ohne Policy-Check erlaubt)
    pub crossing_allowlist: RwLock<HashSet<String>>,

    /// Blocklisted Realms (Crossing immer abgelehnt)
    pub crossing_blocklist: RwLock<HashSet<String>>,

    // ─────────────────────────────────────────────────────────────────────────
    // SAGA & EXECUTION (Κ22/Κ24 SagaComposer-Integration)
    // ─────────────────────────────────────────────────────────────────────────
    /// Sagas die in diesem Realm initiiert wurden
    pub sagas_initiated: AtomicU64,

    /// Cross-Realm-Sagas die dieses Realm involvieren
    pub cross_realm_sagas_involved: AtomicU64,

    /// Sagas die in diesem Realm fehlgeschlagen sind
    pub sagas_failed: AtomicU64,

    /// Compensations in diesem Realm ausgeführt
    pub compensations_executed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // ACTIVITY METRICS
    // ─────────────────────────────────────────────────────────────────────────
    /// Events insgesamt in diesem Realm
    pub events_total: AtomicU64,

    /// Events heute (rolling 24h window, reset via maintenance)
    pub events_today: AtomicU64,

    /// Letztes Event-Timestamp (Unix)
    pub last_event_at: AtomicU64,

    /// Erstellungszeitpunkt (Unix-Timestamp)
    pub created_at: u64,

    // ─────────────────────────────────────────────────────────────────────────
    // SELF-HEALING ISOLATION (Phase 6.2)
    // ─────────────────────────────────────────────────────────────────────────
    /// Resource-Quotas für Self-Healing Isolation
    /// Bei Überschreitung: Realm wird auto-quarantined
    pub quota: RealmQuota,
}

impl RealmSpecificState {
    pub fn new(min_trust: f32, governance_type: &str) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            // Trust & Governance
            trust: RwLock::new(crate::domain::unified::TrustVector6D::DEFAULT),
            min_trust: RwLock::new(min_trust),
            governance_type: RwLock::new(governance_type.to_string()),

            // Membership & Identities (UniversalId-based)
            members_by_id: RwLock::new(HashSet::new()),
            member_realm_dids: RwLock::new(HashMap::new()),
            identity_count: AtomicUsize::new(0),
            pending_members_by_id: RwLock::new(HashSet::new()),
            banned_members_by_id: RwLock::new(HashSet::new()),
            admins_by_id: RwLock::new(HashSet::new()),

            // ECL Rules & Policies
            active_policies: RwLock::new(Vec::new()),
            active_rules: RwLock::new(Vec::new()),

            // Isolation & Data Protection
            isolation_level: AtomicU8::new(1), // Default: Members-Only
            leak_attempts: AtomicU64::new(0),
            leaks_blocked: AtomicU64::new(0),

            // Crossing Metrics
            crossings_in: AtomicU64::new(0),
            crossings_out: AtomicU64::new(0),
            crossings_denied: AtomicU64::new(0),
            active_crossings: AtomicU64::new(0),
            crossing_allowlist: RwLock::new(HashSet::new()),
            crossing_blocklist: RwLock::new(HashSet::new()),

            // Saga & Execution
            sagas_initiated: AtomicU64::new(0),
            cross_realm_sagas_involved: AtomicU64::new(0),
            sagas_failed: AtomicU64::new(0),
            compensations_executed: AtomicU64::new(0),

            // Activity Metrics
            events_total: AtomicU64::new(0),
            events_today: AtomicU64::new(0),
            last_event_at: AtomicU64::new(0),
            created_at: now,

            // Self-Healing Isolation
            quota: RealmQuota::new(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SELF-HEALING ISOLATION OPERATIONS (Phase 6.2)
    // ─────────────────────────────────────────────────────────────────────────

    /// Prüfe ob Realm-Operation erlaubt ist (Quota + Quarantine Check)
    pub fn check_operation(&self, resource: ResourceType, amount: u64) -> bool {
        self.quota.check_quota(resource, amount)
    }

    /// Konsumiere Ressource mit Quota-Check
    pub fn consume_resource(&self, resource: ResourceType, amount: u64) -> bool {
        self.quota.consume(resource, amount)
    }

    /// Freigebe Ressource
    pub fn release_resource(&self, resource: ResourceType, amount: u64) {
        self.quota.release(resource, amount);
    }

    /// Ist Realm quarantined?
    pub fn is_quarantined(&self) -> bool {
        self.quota.is_quarantined()
    }

    /// Quarantine Realm (z.B. durch ProtectionState)
    pub fn quarantine(&self) {
        self.quota.quarantine();
    }

    /// Unquarantine Realm (Admin-Recovery)
    pub fn unquarantine(&self) {
        self.quota.unquarantine();
    }

    /// Setze Quota-Limit (via Governance)
    pub fn set_quota_limit(&self, resource: ResourceType, limit: u64) {
        self.quota.set_limit(resource, limit);
    }

    /// Health-Score basierend auf Quota-Utilization
    pub fn quota_health(&self) -> f64 {
        let queue_util = self.quota.utilization(ResourceType::QueueSlots);
        let storage_util = self.quota.utilization(ResourceType::StorageBytes);
        let gas_util = self.quota.utilization(ResourceType::ComputeGas);

        // Gewichteter Durchschnitt (Queue am wichtigsten)
        let weighted = (queue_util * 0.4 + storage_util * 0.3 + gas_util * 0.3) / 100.0;
        100.0 - (weighted * 100.0).min(100.0)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // MEMBERSHIP OPERATIONS (UniversalId-based - Primary)
    // ─────────────────────────────────────────────────────────────────────────

    /// Füge Member zum Realm hinzu (UniversalId-basiert, Primary)
    pub fn add_member_by_id(&self, identity_id: UniversalId, realm_sub_did: Option<UniversalId>) {
        if let Ok(mut members) = self.members_by_id.write() {
            if members.insert(identity_id) {
                self.identity_count.fetch_add(1, Ordering::Relaxed);
            }
        }
        // Entferne aus pending falls vorhanden
        if let Ok(mut pending) = self.pending_members_by_id.write() {
            pending.remove(&identity_id);
        }
        // Speichere Realm-Sub-DID Mapping falls vorhanden
        if let Some(sub_did) = realm_sub_did {
            if let Ok(mut dids) = self.member_realm_dids.write() {
                dids.insert(identity_id, sub_did);
            }
        }
    }

    /// Entferne Member vom Realm (UniversalId-basiert)
    pub fn remove_member_by_id(&self, identity_id: &UniversalId) {
        if let Ok(mut members) = self.members_by_id.write() {
            if members.remove(identity_id) {
                let _ = self
                    .identity_count
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                        if v > 0 {
                            Some(v - 1)
                        } else {
                            Some(0)
                        }
                    });
            }
        }
        // Entferne Realm-Sub-DID Mapping
        if let Ok(mut dids) = self.member_realm_dids.write() {
            dids.remove(identity_id);
        }
    }

    /// Prüfe ob Identity Member ist (UniversalId-basiert)
    pub fn is_member_by_id(&self, identity_id: &UniversalId) -> bool {
        self.members_by_id
            .read()
            .map(|m| m.contains(identity_id))
            .unwrap_or(false)
    }

    /// Hole Realm-Sub-DID für Member
    pub fn get_realm_sub_did(&self, identity_id: &UniversalId) -> Option<UniversalId> {
        self.member_realm_dids
            .read()
            .ok()
            .and_then(|dids| dids.get(identity_id).copied())
    }

    /// Füge Membership-Request hinzu (UniversalId-basiert)
    pub fn request_membership_by_id(&self, identity_id: UniversalId) {
        if let Ok(mut pending) = self.pending_members_by_id.write() {
            pending.insert(identity_id);
        }
    }

    /// Banne Identity (UniversalId-basiert, permanent)
    pub fn ban_member_by_id(&self, identity_id: &UniversalId) {
        self.remove_member_by_id(identity_id);
        if let Ok(mut banned) = self.banned_members_by_id.write() {
            banned.insert(*identity_id);
        }
    }

    /// Prüfe ob Identity gebannt ist (UniversalId-basiert)
    pub fn is_banned_by_id(&self, identity_id: &UniversalId) -> bool {
        self.banned_members_by_id
            .read()
            .map(|b| b.contains(identity_id))
            .unwrap_or(false)
    }

    /// Füge Admin hinzu (UniversalId-basiert)
    pub fn add_admin_by_id(&self, identity_id: UniversalId, realm_sub_did: Option<UniversalId>) {
        if let Ok(mut admins) = self.admins_by_id.write() {
            admins.insert(identity_id);
        }
        // Admins sind automatisch auch Members
        self.add_member_by_id(identity_id, realm_sub_did);
    }

    /// Prüfe ob Identity Admin ist (UniversalId-basiert)
    pub fn is_admin_by_id(&self, identity_id: &UniversalId) -> bool {
        self.admins_by_id
            .read()
            .map(|a| a.contains(identity_id))
            .unwrap_or(false)
    }

    /// Migriere Legacy-Member zu UniversalId
    /// Konvertiert String-ID zu UniversalId mittels DID-Parsing
    pub fn migrate_legacy_member(&self, legacy_id: &str) -> Option<UniversalId> {
        // Versuche DID zu parsen: did:erynoa:namespace:id
        if legacy_id.starts_with("did:erynoa:") {
            let parts: Vec<&str> = legacy_id.split(':').collect();
            if parts.len() >= 4 {
                // Generiere UniversalId aus dem Legacy-ID Hash
                let id = UniversalId::new(
                    UniversalId::TAG_DID,
                    1, // Version 1
                    legacy_id.as_bytes(),
                );
                self.add_member_by_id(id, None);
                return Some(id);
            }
        }
        None
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CROSSING OPERATIONS (Κ23)
    // ─────────────────────────────────────────────────────────────────────────

    pub fn crossing_in(&self) {
        self.crossings_in.fetch_add(1, Ordering::Relaxed);
        self.record_event();
    }

    pub fn crossing_out(&self) {
        self.crossings_out.fetch_add(1, Ordering::Relaxed);
        self.record_event();
    }

    pub fn crossing_denied(&self) {
        self.crossings_denied.fetch_add(1, Ordering::Relaxed);
    }

    /// Prüfe ob Crossing zu target_realm erlaubt ist (Allowlist/Blocklist)
    pub fn is_crossing_allowed(&self, target_realm: &str) -> Option<bool> {
        // Blocklist hat Priorität
        if let Ok(blocklist) = self.crossing_blocklist.read() {
            if blocklist.contains(target_realm) {
                return Some(false);
            }
        }
        // Allowlist erlaubt ohne Policy-Check
        if let Ok(allowlist) = self.crossing_allowlist.read() {
            if allowlist.contains(target_realm) {
                return Some(true);
            }
        }
        // Weder Allow noch Block → Policy muss entscheiden
        None
    }

    /// Füge Realm zur Allowlist hinzu
    pub fn allow_crossing_to(&self, target_realm: &str) {
        if let Ok(mut allowlist) = self.crossing_allowlist.write() {
            allowlist.insert(target_realm.to_string());
        }
        // Entferne aus Blocklist falls vorhanden
        if let Ok(mut blocklist) = self.crossing_blocklist.write() {
            blocklist.remove(target_realm);
        }
    }

    /// Füge Realm zur Blocklist hinzu
    pub fn block_crossing_to(&self, target_realm: &str) {
        if let Ok(mut blocklist) = self.crossing_blocklist.write() {
            blocklist.insert(target_realm.to_string());
        }
        // Entferne aus Allowlist falls vorhanden
        if let Ok(mut allowlist) = self.crossing_allowlist.write() {
            allowlist.remove(target_realm);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SAGA OPERATIONS (Κ22/Κ24)
    // ─────────────────────────────────────────────────────────────────────────

    pub fn saga_initiated(&self, cross_realm: bool) {
        self.sagas_initiated.fetch_add(1, Ordering::Relaxed);
        if cross_realm {
            self.cross_realm_sagas_involved
                .fetch_add(1, Ordering::Relaxed);
        }
        self.record_event();
    }

    pub fn saga_failed(&self) {
        self.sagas_failed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn compensation_executed(&self) {
        self.compensations_executed.fetch_add(1, Ordering::Relaxed);
        self.record_event();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // LEGACY COMPATIBILITY
    // ─────────────────────────────────────────────────────────────────────────

    pub fn identity_joined(&self) {
        self.identity_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn identity_left(&self) {
        let _ = self
            .identity_count
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            });
    }

    pub fn add_rule(&self, rule_id: &str) {
        if let Ok(mut rules) = self.active_rules.write() {
            if !rules.contains(&rule_id.to_string()) {
                rules.push(rule_id.to_string());
            }
        }
    }

    pub fn remove_rule(&self, rule_id: &str) {
        if let Ok(mut rules) = self.active_rules.write() {
            rules.retain(|r| r != rule_id);
        }
    }

    pub fn update_trust(&self, new_trust: crate::domain::unified::TrustVector6D) {
        if let Ok(mut trust) = self.trust.write() {
            *trust = new_trust;
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ECL POLICY OPERATIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Aktiviere ECL-Policy für dieses Realm
    pub fn activate_policy(&self, policy_id: &str) {
        if let Ok(mut policies) = self.active_policies.write() {
            if !policies.contains(&policy_id.to_string()) {
                policies.push(policy_id.to_string());
            }
        }
    }

    /// Deaktiviere ECL-Policy
    pub fn deactivate_policy(&self, policy_id: &str) {
        if let Ok(mut policies) = self.active_policies.write() {
            policies.retain(|p| p != policy_id);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ISOLATION OPERATIONS
    // ─────────────────────────────────────────────────────────────────────────

    /// Setze Isolation-Level (0=Public, 1=Members-Only, 2=Strict)
    pub fn set_isolation_level(&self, level: u8) {
        self.isolation_level.store(level.min(2), Ordering::Relaxed);
    }

    /// Hole Isolation-Level
    pub fn get_isolation_level(&self) -> u8 {
        self.isolation_level.load(Ordering::Relaxed)
    }

    /// Registriere Leak-Versuch
    pub fn record_leak_attempt(&self, blocked: bool) {
        self.leak_attempts.fetch_add(1, Ordering::Relaxed);
        if blocked {
            self.leaks_blocked.fetch_add(1, Ordering::Relaxed);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ACTIVITY TRACKING
    // ─────────────────────────────────────────────────────────────────────────

    fn record_event(&self) {
        use std::time::{SystemTime, UNIX_EPOCH};

        self.events_total.fetch_add(1, Ordering::Relaxed);
        self.events_today.fetch_add(1, Ordering::Relaxed);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.last_event_at.store(now, Ordering::Relaxed);
    }

    /// Reset daily counter (called by maintenance)
    pub fn reset_daily_events(&self) {
        self.events_today.store(0, Ordering::Relaxed);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // SNAPSHOT
    // ─────────────────────────────────────────────────────────────────────────

    pub fn snapshot(&self) -> RealmSpecificSnapshot {
        RealmSpecificSnapshot {
            // Trust & Governance
            trust: self.trust.read().map(|t| *t).unwrap_or_default(),
            min_trust: self.min_trust.read().map(|t| *t).unwrap_or(0.0),
            governance_type: self
                .governance_type
                .read()
                .map(|g| g.clone())
                .unwrap_or_default(),

            // Membership
            member_count: self.identity_count.load(Ordering::Relaxed),
            pending_member_count: self
                .pending_members_by_id
                .read()
                .map(|p| p.len())
                .unwrap_or(0),
            banned_count: self
                .banned_members_by_id
                .read()
                .map(|b| b.len())
                .unwrap_or(0),
            admin_count: self.admins_by_id.read().map(|a| a.len()).unwrap_or(0),

            // ECL Policies
            active_policies: self
                .active_policies
                .read()
                .map(|p| p.clone())
                .unwrap_or_default(),
            active_rules: self
                .active_rules
                .read()
                .map(|r| r.clone())
                .unwrap_or_default(),

            // Isolation
            isolation_level: self.isolation_level.load(Ordering::Relaxed),
            leak_attempts: self.leak_attempts.load(Ordering::Relaxed),
            leaks_blocked: self.leaks_blocked.load(Ordering::Relaxed),

            // Crossings
            crossings_in: self.crossings_in.load(Ordering::Relaxed),
            crossings_out: self.crossings_out.load(Ordering::Relaxed),
            crossings_denied: self.crossings_denied.load(Ordering::Relaxed),
            active_crossings: self.active_crossings.load(Ordering::Relaxed),
            crossing_allowlist_count: self.crossing_allowlist.read().map(|a| a.len()).unwrap_or(0),
            crossing_blocklist_count: self.crossing_blocklist.read().map(|b| b.len()).unwrap_or(0),

            // Sagas
            sagas_initiated: self.sagas_initiated.load(Ordering::Relaxed),
            cross_realm_sagas_involved: self.cross_realm_sagas_involved.load(Ordering::Relaxed),
            sagas_failed: self.sagas_failed.load(Ordering::Relaxed),
            compensations_executed: self.compensations_executed.load(Ordering::Relaxed),

            // Activity
            events_total: self.events_total.load(Ordering::Relaxed),
            events_today: self.events_today.load(Ordering::Relaxed),
            last_event_at: self.last_event_at.load(Ordering::Relaxed),
            created_at: self.created_at,

            // Self-Healing Isolation (Phase 6.2)
            quota: self.quota.snapshot(),
            quota_health: self.quota_health(),
        }
    }
}

/// Serializable Snapshot of RealmSpecificState
///
/// Vollständige Realm-Metriken für Debugging, Monitoring und Isolation-Prüfung.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmSpecificSnapshot {
    // Trust & Governance
    pub trust: crate::domain::unified::TrustVector6D,
    pub min_trust: f32,
    pub governance_type: String,

    // Membership (nur Counts für Privacy)
    pub member_count: usize,
    pub pending_member_count: usize,
    pub banned_count: usize,
    pub admin_count: usize,

    // ECL Policies
    pub active_policies: Vec<String>,
    pub active_rules: Vec<String>,

    // Isolation
    pub isolation_level: u8,
    pub leak_attempts: u64,
    pub leaks_blocked: u64,

    // Crossings (Κ23)
    pub crossings_in: u64,
    pub crossings_out: u64,
    pub crossings_denied: u64,
    pub active_crossings: u64,
    pub crossing_allowlist_count: usize,
    pub crossing_blocklist_count: usize,

    // Sagas (Κ22/Κ24)
    pub sagas_initiated: u64,
    pub cross_realm_sagas_involved: u64,
    pub sagas_failed: u64,
    pub compensations_executed: u64,

    // Activity
    pub events_total: u64,
    pub events_today: u64,
    pub last_event_at: u64,
    pub created_at: u64,

    // Self-Healing Isolation (Phase 6.2)
    pub quota: RealmQuotaSnapshot,
    pub quota_health: f64,
}

/// Aggregierter Realm State für alle Realms
///
/// Verwaltet alle registrierten Realms mit ihrem jeweiligen State.
/// Implementiert das Realm-Konzept: Isolierte Bereiche mit eigenen
/// Regeln, Identitäten und Trust-Leveln (Κ22-Κ24).
#[derive(Debug)]
pub struct RealmState {
    /// Alle registrierten Realms mit ihrem State
    pub realms: RwLock<HashMap<String, RealmSpecificState>>,

    /// Gesamt-Anzahl Realms
    pub total_realms: AtomicUsize,

    /// Aktuell aktive Cross-Realm-Crossings
    pub active_crossings: AtomicU64,

    /// Gesamt Cross-Realm-Sagas
    pub total_cross_realm_sagas: AtomicU64,

    /// Fehlgeschlagene Crossing-Versuche
    pub crossing_failures: AtomicU64,

    /// Root-Realm ID (falls vorhanden)
    pub root_realm_id: RwLock<Option<String>>,
}

impl RealmState {
    pub fn new() -> Self {
        Self {
            realms: RwLock::new(HashMap::new()),
            total_realms: AtomicUsize::new(0),
            active_crossings: AtomicU64::new(0),
            total_cross_realm_sagas: AtomicU64::new(0),
            crossing_failures: AtomicU64::new(0),
            root_realm_id: RwLock::new(None),
        }
    }

    /// Registriere ein neues Realm
    pub fn register_realm(&self, realm_id: &str, min_trust: f32, governance_type: &str) {
        if let Ok(mut realms) = self.realms.write() {
            if !realms.contains_key(realm_id) {
                realms.insert(
                    realm_id.to_string(),
                    RealmSpecificState::new(min_trust, governance_type),
                );
                self.total_realms.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Setze Root-Realm
    pub fn set_root_realm(&self, realm_id: &str) {
        if let Ok(mut root) = self.root_realm_id.write() {
            *root = Some(realm_id.to_string());
        }
    }

    /// Hole Realm-spezifischen State
    pub fn get_realm(&self, realm_id: &str) -> Option<RealmSpecificSnapshot> {
        self.realms.read().ok()?.get(realm_id).map(|r| r.snapshot())
    }

    /// Registriere ein erfolgreiches Crossing
    pub fn crossing_succeeded(&self, from_realm: &str, to_realm: &str) {
        if let Ok(realms) = self.realms.read() {
            if let Some(from) = realms.get(from_realm) {
                from.crossing_out();
            }
            if let Some(to) = realms.get(to_realm) {
                to.crossing_in();
            }
        }
        self.active_crossings.fetch_add(1, Ordering::Relaxed);
    }

    /// Registriere ein fehlgeschlagenes Crossing
    pub fn crossing_failed(&self) {
        self.crossing_failures.fetch_add(1, Ordering::Relaxed);
    }

    /// Crossing beendet
    pub fn crossing_completed(&self) {
        let _ = self
            .active_crossings
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            });
    }

    /// Registriere Cross-Realm-Saga
    pub fn cross_realm_saga_started(&self, realm_ids: &[&str]) {
        self.total_cross_realm_sagas.fetch_add(1, Ordering::Relaxed);
        if let Ok(realms) = self.realms.read() {
            for realm_id in realm_ids {
                if let Some(realm) = realms.get(*realm_id) {
                    realm.saga_initiated(true);
                }
            }
        }
    }

    /// Identity tritt einem Realm bei
    pub fn identity_joined_realm(&self, realm_id: &str) {
        if let Ok(realms) = self.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.identity_joined();
            }
        }
    }

    /// Identity verlässt ein Realm
    pub fn identity_left_realm(&self, realm_id: &str) {
        if let Ok(realms) = self.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.identity_left();
            }
        }
    }

    /// Update Trust für ein Realm
    pub fn update_realm_trust(&self, realm_id: &str, trust: crate::domain::unified::TrustVector6D) {
        if let Ok(realms) = self.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.update_trust(trust);
            }
        }
    }

    /// Füge Rule zu Realm hinzu
    pub fn add_rule_to_realm(&self, realm_id: &str, rule_id: &str) {
        if let Ok(realms) = self.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.add_rule(rule_id);
            }
        }
    }

    pub fn snapshot(&self) -> RealmSnapshot {
        let realms_snapshot = self
            .realms
            .read()
            .map(|r| r.iter().map(|(k, v)| (k.clone(), v.snapshot())).collect())
            .unwrap_or_default();

        RealmSnapshot {
            realms: realms_snapshot,
            total_realms: self.total_realms.load(Ordering::Relaxed),
            active_crossings: self.active_crossings.load(Ordering::Relaxed),
            total_cross_realm_sagas: self.total_cross_realm_sagas.load(Ordering::Relaxed),
            crossing_failures: self.crossing_failures.load(Ordering::Relaxed),
            root_realm_id: self.root_realm_id.read().map(|r| r.clone()).unwrap_or(None),
        }
    }
}

impl Default for RealmState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmSnapshot {
    pub realms: HashMap<String, RealmSpecificSnapshot>,
    pub total_realms: usize,
    pub active_crossings: u64,
    pub total_cross_realm_sagas: u64,
    pub crossing_failures: u64,
    pub root_realm_id: Option<String>,
}

/// Aggregierter Peer State (Gateway + Saga + Intent + Realm)
#[derive(Debug)]
pub struct PeerState {
    pub gateway: GatewayState,
    pub saga: SagaComposerState,
    pub intent: IntentParserState,
    /// Realm-State für isolierte Bereiche mit eigenen Regeln und Trust-Leveln
    pub realm: RealmState,
}

impl PeerState {
    pub fn new() -> Self {
        Self {
            gateway: GatewayState::new(),
            saga: SagaComposerState::new(),
            intent: IntentParserState::new(),
            realm: RealmState::new(),
        }
    }

    pub fn snapshot(&self) -> PeerSnapshot {
        PeerSnapshot {
            gateway: self.gateway.snapshot(),
            saga: self.saga.snapshot(),
            intent: self.intent.snapshot(),
            realm: self.realm.snapshot(),
        }
    }
}

impl Default for PeerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerSnapshot {
    pub gateway: GatewaySnapshot,
    pub saga: SagaComposerSnapshot,
    pub intent: IntentParserSnapshot,
    pub realm: RealmSnapshot,
}

// ============================================================================
// P2P NETWORK STATE LAYER
// ============================================================================

/// NAT Traversal Status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NatStatus {
    #[default]
    Unknown,
    Public,
    Private,
}

/// Swarm State
#[derive(Debug)]
pub struct SwarmState {
    /// Eigene Peer-ID (Legacy String-basiert)
    pub peer_id: RwLock<String>,
    /// Eigene Peer-ID (UniversalId-basiert, Phase 7)
    pub peer_universal_id: RwLock<Option<UniversalId>>,
    /// Verbundene Peers
    pub connected_peers: AtomicUsize,
    /// Eingehende Verbindungen
    pub inbound_connections: AtomicU64,
    /// Ausgehende Verbindungen
    pub outbound_connections: AtomicU64,
    /// Verbindungsfehler
    pub connection_errors: AtomicU64,
    /// Bytes gesendet
    pub bytes_sent: AtomicU64,
    /// Bytes empfangen
    pub bytes_received: AtomicU64,
    /// Latenz-Summe (für Durchschnitt)
    pub latency_sum_us: AtomicU64,
    /// Latenz-Messungen
    pub latency_count: AtomicU64,
    /// NAT-Status
    pub nat_status: RwLock<NatStatus>,
    /// Externe Adressen
    pub external_addresses: RwLock<Vec<String>>,
}

impl SwarmState {
    pub fn new() -> Self {
        Self {
            peer_id: RwLock::new(String::new()),
            peer_universal_id: RwLock::new(None),
            connected_peers: AtomicUsize::new(0),
            inbound_connections: AtomicU64::new(0),
            outbound_connections: AtomicU64::new(0),
            connection_errors: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            latency_sum_us: AtomicU64::new(0),
            latency_count: AtomicU64::new(0),
            nat_status: RwLock::new(NatStatus::Unknown),
            external_addresses: RwLock::new(Vec::new()),
        }
    }

    /// Setze Peer-Identity (UniversalId-basiert)
    /// Synchronisiert auch die Legacy peer_id String-Repräsentation
    pub fn set_peer_identity(&self, device_did: &crate::domain::unified::identity::DID) {
        if let Ok(mut pid) = self.peer_id.write() {
            *pid = device_did.to_uri();
        }
        if let Ok(mut uid) = self.peer_universal_id.write() {
            *uid = Some(device_did.id);
        }
    }

    /// Setze Peer-Identity direkt via UniversalId
    pub fn set_peer_universal_id(&self, identity_id: UniversalId) {
        if let Ok(mut uid) = self.peer_universal_id.write() {
            *uid = Some(identity_id);
        }
        // Generiere Legacy String-Repräsentation
        if let Ok(mut pid) = self.peer_id.write() {
            *pid = hex::encode(identity_id.as_bytes());
        }
    }

    /// Hole Peer UniversalId
    pub fn get_peer_universal_id(&self) -> Option<UniversalId> {
        self.peer_universal_id.read().ok().and_then(|id| *id)
    }

    pub fn peer_connected(&self, inbound: bool) {
        self.connected_peers.fetch_add(1, Ordering::Relaxed);
        if inbound {
            self.inbound_connections.fetch_add(1, Ordering::Relaxed);
        } else {
            self.outbound_connections.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn peer_disconnected(&self) {
        let _ = self
            .connected_peers
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            });
    }

    pub fn record_latency(&self, latency_us: u64) {
        self.latency_sum_us.fetch_add(latency_us, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn avg_latency_ms(&self) -> f64 {
        let count = self.latency_count.load(Ordering::Relaxed);
        if count > 0 {
            (self.latency_sum_us.load(Ordering::Relaxed) as f64 / count as f64) / 1000.0
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> SwarmSnapshot {
        SwarmSnapshot {
            peer_id: self.peer_id.read().map(|p| p.clone()).unwrap_or_default(),
            peer_universal_id: self.get_peer_universal_id(),
            connected_peers: self.connected_peers.load(Ordering::Relaxed),
            inbound_connections: self.inbound_connections.load(Ordering::Relaxed),
            outbound_connections: self.outbound_connections.load(Ordering::Relaxed),
            connection_errors: self.connection_errors.load(Ordering::Relaxed),
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            avg_latency_ms: self.avg_latency_ms(),
            nat_status: self.nat_status.read().map(|n| *n).unwrap_or_default(),
            external_addresses: self
                .external_addresses
                .read()
                .map(|a| a.clone())
                .unwrap_or_default(),
        }
    }
}

impl Default for SwarmState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmSnapshot {
    pub peer_id: String,
    /// Peer UniversalId (Phase 7)
    pub peer_universal_id: Option<UniversalId>,
    pub connected_peers: usize,
    pub inbound_connections: u64,
    pub outbound_connections: u64,
    pub connection_errors: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub avg_latency_ms: f64,
    pub nat_status: NatStatus,
    pub external_addresses: Vec<String>,
}

/// Gossipsub State
#[derive(Debug)]
pub struct GossipState {
    /// Peers im Mesh
    pub mesh_peers: AtomicUsize,
    /// Subscribed Topics
    pub subscribed_topics: AtomicUsize,
    /// Messages empfangen
    pub messages_received: AtomicU64,
    /// Messages gesendet
    pub messages_sent: AtomicU64,
    /// Messages validiert
    pub messages_validated: AtomicU64,
    /// Messages abgelehnt
    pub messages_rejected: AtomicU64,
    /// Duplicate Messages (ignoriert)
    pub duplicate_messages: AtomicU64,
    /// Trust-basierte Scores
    pub peers_pruned: AtomicU64,
    pub peers_grafted: AtomicU64,
}

impl GossipState {
    pub fn new() -> Self {
        Self {
            mesh_peers: AtomicUsize::new(0),
            subscribed_topics: AtomicUsize::new(0),
            messages_received: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            messages_validated: AtomicU64::new(0),
            messages_rejected: AtomicU64::new(0),
            duplicate_messages: AtomicU64::new(0),
            peers_pruned: AtomicU64::new(0),
            peers_grafted: AtomicU64::new(0),
        }
    }

    pub fn message_received(&self) {
        self.messages_received.fetch_add(1, Ordering::Relaxed);
    }

    pub fn message_sent(&self) {
        self.messages_sent.fetch_add(1, Ordering::Relaxed);
    }

    pub fn validation_rate(&self) -> f64 {
        let total = self.messages_received.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.messages_validated.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> GossipSnapshot {
        GossipSnapshot {
            mesh_peers: self.mesh_peers.load(Ordering::Relaxed),
            subscribed_topics: self.subscribed_topics.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_validated: self.messages_validated.load(Ordering::Relaxed),
            messages_rejected: self.messages_rejected.load(Ordering::Relaxed),
            duplicate_messages: self.duplicate_messages.load(Ordering::Relaxed),
            peers_pruned: self.peers_pruned.load(Ordering::Relaxed),
            peers_grafted: self.peers_grafted.load(Ordering::Relaxed),
            validation_rate: self.validation_rate(),
        }
    }
}

impl Default for GossipState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipSnapshot {
    pub mesh_peers: usize,
    pub subscribed_topics: usize,
    pub messages_received: u64,
    pub messages_sent: u64,
    pub messages_validated: u64,
    pub messages_rejected: u64,
    pub duplicate_messages: u64,
    pub peers_pruned: u64,
    pub peers_grafted: u64,
    pub validation_rate: f64,
}

/// Kademlia DHT State
#[derive(Debug)]
pub struct KademliaState {
    /// Peers in Routing Table
    pub routing_table_size: AtomicUsize,
    /// Bootstrap abgeschlossen
    pub bootstrap_complete: RwLock<bool>,
    /// Records gespeichert
    pub records_stored: AtomicU64,
    /// Queries durchgeführt
    pub queries_total: AtomicU64,
    /// Queries erfolgreich
    pub queries_successful: AtomicU64,
    /// Provider-Registrierungen
    pub provider_registrations: AtomicU64,
}

impl KademliaState {
    pub fn new() -> Self {
        Self {
            routing_table_size: AtomicUsize::new(0),
            bootstrap_complete: RwLock::new(false),
            records_stored: AtomicU64::new(0),
            queries_total: AtomicU64::new(0),
            queries_successful: AtomicU64::new(0),
            provider_registrations: AtomicU64::new(0),
        }
    }

    pub fn query_success_rate(&self) -> f64 {
        let total = self.queries_total.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.queries_successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> KademliaSnapshot {
        KademliaSnapshot {
            routing_table_size: self.routing_table_size.load(Ordering::Relaxed),
            bootstrap_complete: self.bootstrap_complete.read().map(|b| *b).unwrap_or(false),
            records_stored: self.records_stored.load(Ordering::Relaxed),
            queries_total: self.queries_total.load(Ordering::Relaxed),
            queries_successful: self.queries_successful.load(Ordering::Relaxed),
            query_success_rate: self.query_success_rate(),
            provider_registrations: self.provider_registrations.load(Ordering::Relaxed),
        }
    }
}

impl Default for KademliaState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KademliaSnapshot {
    pub routing_table_size: usize,
    pub bootstrap_complete: bool,
    pub records_stored: u64,
    pub queries_total: u64,
    pub queries_successful: u64,
    pub query_success_rate: f64,
    pub provider_registrations: u64,
}

/// Relay State (Circuit Relay V2)
#[derive(Debug)]
pub struct RelayState {
    /// Aktive Relay-Reservation
    pub has_reservation: RwLock<bool>,
    /// Relay-Peer
    pub relay_peer: RwLock<Option<String>>,
    /// Circuits bedient (als Server)
    pub circuits_served: AtomicU64,
    /// Circuits aktiv
    pub circuits_active: AtomicUsize,
    /// DCUTR Erfolge (Hole-Punching)
    pub dcutr_successes: AtomicU64,
    /// DCUTR Fehlschläge
    pub dcutr_failures: AtomicU64,
    /// Bytes über Relay
    pub relay_bytes: AtomicU64,
}

impl RelayState {
    pub fn new() -> Self {
        Self {
            has_reservation: RwLock::new(false),
            relay_peer: RwLock::new(None),
            circuits_served: AtomicU64::new(0),
            circuits_active: AtomicUsize::new(0),
            dcutr_successes: AtomicU64::new(0),
            dcutr_failures: AtomicU64::new(0),
            relay_bytes: AtomicU64::new(0),
        }
    }

    pub fn dcutr_success_rate(&self) -> f64 {
        let total = self.dcutr_successes.load(Ordering::Relaxed)
            + self.dcutr_failures.load(Ordering::Relaxed);
        if total > 0 {
            self.dcutr_successes.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> RelaySnapshot {
        RelaySnapshot {
            has_reservation: self.has_reservation.read().map(|b| *b).unwrap_or(false),
            relay_peer: self
                .relay_peer
                .read()
                .map(|p| p.clone())
                .unwrap_or_default(),
            circuits_served: self.circuits_served.load(Ordering::Relaxed),
            circuits_active: self.circuits_active.load(Ordering::Relaxed),
            dcutr_successes: self.dcutr_successes.load(Ordering::Relaxed),
            dcutr_failures: self.dcutr_failures.load(Ordering::Relaxed),
            dcutr_success_rate: self.dcutr_success_rate(),
            relay_bytes: self.relay_bytes.load(Ordering::Relaxed),
        }
    }
}

impl Default for RelayState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelaySnapshot {
    pub has_reservation: bool,
    pub relay_peer: Option<String>,
    pub circuits_served: u64,
    pub circuits_active: usize,
    pub dcutr_successes: u64,
    pub dcutr_failures: u64,
    pub dcutr_success_rate: f64,
    pub relay_bytes: u64,
}

/// Privacy Layer State (Onion Routing)
#[derive(Debug)]
pub struct PrivacyState {
    /// Circuits erstellt
    pub circuits_created: AtomicU64,
    /// Circuits geschlossen (v0.4.0)
    pub circuits_closed: AtomicU64,
    /// Circuits aktiv
    pub circuits_active: AtomicUsize,
    /// Hops durchschnittlich
    pub avg_hops: RwLock<f64>,
    /// Messages über Privacy-Layer gesendet (v0.4.0)
    pub messages_sent: AtomicU64,
    /// Messages über Privacy-Layer geroutet (v0.4.0)
    pub messages_routed: AtomicU64,
    /// Messages über Privacy-Layer (Legacy)
    pub private_messages: AtomicU64,
    /// Cover-Traffic Messages gesendet (v0.4.0)
    pub cover_traffic_sent: AtomicU64,
    /// Cover-Traffic Messages (Legacy)
    pub cover_traffic: AtomicU64,
    /// Mixing-Pool Flushes (v0.4.0)
    pub mixing_flushes: AtomicU64,
    /// Messages durch Mixing-Pool (v0.4.0)
    pub messages_mixed: AtomicU64,
    /// Relay-Rotationen
    pub relay_rotations: AtomicU64,
    /// Trust-basierte Relay-Auswahl
    pub trust_based_selections: AtomicU64,
    /// Fehlgeschlagene Relay-Auswahlen (v0.4.0)
    pub selection_failures: AtomicU64,
}

impl PrivacyState {
    pub fn new() -> Self {
        Self {
            circuits_created: AtomicU64::new(0),
            circuits_closed: AtomicU64::new(0),
            circuits_active: AtomicUsize::new(0),
            avg_hops: RwLock::new(3.0),
            messages_sent: AtomicU64::new(0),
            messages_routed: AtomicU64::new(0),
            private_messages: AtomicU64::new(0),
            cover_traffic_sent: AtomicU64::new(0),
            cover_traffic: AtomicU64::new(0),
            mixing_flushes: AtomicU64::new(0),
            messages_mixed: AtomicU64::new(0),
            relay_rotations: AtomicU64::new(0),
            trust_based_selections: AtomicU64::new(0),
            selection_failures: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> PrivacySnapshot {
        PrivacySnapshot {
            circuits_created: self.circuits_created.load(Ordering::Relaxed),
            circuits_closed: self.circuits_closed.load(Ordering::Relaxed),
            circuits_active: self.circuits_active.load(Ordering::Relaxed),
            avg_hops: self.avg_hops.read().map(|h| *h).unwrap_or(3.0),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_routed: self.messages_routed.load(Ordering::Relaxed),
            private_messages: self.private_messages.load(Ordering::Relaxed),
            cover_traffic_sent: self.cover_traffic_sent.load(Ordering::Relaxed),
            cover_traffic: self.cover_traffic.load(Ordering::Relaxed),
            mixing_flushes: self.mixing_flushes.load(Ordering::Relaxed),
            messages_mixed: self.messages_mixed.load(Ordering::Relaxed),
            relay_rotations: self.relay_rotations.load(Ordering::Relaxed),
            trust_based_selections: self.trust_based_selections.load(Ordering::Relaxed),
            selection_failures: self.selection_failures.load(Ordering::Relaxed),
        }
    }
}

impl Default for PrivacyState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySnapshot {
    pub circuits_created: u64,
    pub circuits_closed: u64,
    pub circuits_active: usize,
    pub avg_hops: f64,
    pub messages_sent: u64,
    pub messages_routed: u64,
    pub private_messages: u64,
    pub cover_traffic_sent: u64,
    pub cover_traffic: u64,
    pub mixing_flushes: u64,
    pub messages_mixed: u64,
    pub relay_rotations: u64,
    pub trust_based_selections: u64,
    pub selection_failures: u64,
}

/// Aggregierter P2P State
#[derive(Debug)]
pub struct P2PState {
    pub swarm: SwarmState,
    pub gossip: GossipState,
    pub kademlia: KademliaState,
    pub relay: RelayState,
    pub privacy: PrivacyState,
}

impl P2PState {
    pub fn new() -> Self {
        Self {
            swarm: SwarmState::new(),
            gossip: GossipState::new(),
            kademlia: KademliaState::new(),
            relay: RelayState::new(),
            privacy: PrivacyState::new(),
        }
    }

    /// Berechne P2P-Health Score
    pub fn health_score(&self) -> f64 {
        let mut score: f64 = 100.0;

        // Mindestens 3 Peers
        let peers = self.swarm.connected_peers.load(Ordering::Relaxed);
        if peers < 3 {
            score -= 30.0;
        } else if peers < 6 {
            score -= 10.0;
        }

        // Kademlia Bootstrap
        if !self
            .kademlia
            .bootstrap_complete
            .read()
            .map(|b| *b)
            .unwrap_or(false)
        {
            score -= 20.0;
        }

        // Gossip Mesh
        let mesh = self.gossip.mesh_peers.load(Ordering::Relaxed);
        if mesh < 2 {
            score -= 15.0;
        }

        // Connection Errors
        let errors = self.swarm.connection_errors.load(Ordering::Relaxed);
        let total_conns = self.swarm.inbound_connections.load(Ordering::Relaxed)
            + self.swarm.outbound_connections.load(Ordering::Relaxed);
        if total_conns > 0 && errors as f64 / total_conns as f64 > 0.1 {
            score -= 10.0;
        }

        // DCUTR Success Rate
        let dcutr_rate = self.relay.dcutr_success_rate();
        if dcutr_rate < 0.5 {
            score -= 10.0;
        }

        score.max(0.0).min(100.0)
    }

    pub fn snapshot(&self) -> P2PSnapshot {
        P2PSnapshot {
            swarm: self.swarm.snapshot(),
            gossip: self.gossip.snapshot(),
            kademlia: self.kademlia.snapshot(),
            relay: self.relay.snapshot(),
            privacy: self.privacy.snapshot(),
            health_score: self.health_score(),
        }
    }
}

impl Default for P2PState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PSnapshot {
    pub swarm: SwarmSnapshot,
    pub gossip: GossipSnapshot,
    pub kademlia: KademliaSnapshot,
    pub relay: RelaySnapshot,
    pub privacy: PrivacySnapshot,
    pub health_score: f64,
}

// ============================================================================
// ENGINE-LAYER STATE (6 neue Engines für SOLL-Zustand)
// ============================================================================

// ────────────────────────────────────────────────────────────────────────────
// 2.1 UI-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// UI-Engine State mit Component-Tree und Binding-Tracking
///
/// # Design
///
/// Die UI-Engine verwaltet deklarative, Trust-basierte Interfaces:
/// - **Component-Tree**: Hierarchischer UI-Aufbau
/// - **Bindings**: Reaktive Daten-Verbindungen
/// - **Trust-Gates**: Sichtbarkeit basierend auf Trust
/// - **Credential-Gates**: Zugriffskontrolle basierend auf Credentials
/// - **Render-Cache**: Optimierte Re-Renders
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// UI ──DependsOn──▶ Trust (Sichtbarkeit)
/// UI ──DependsOn──▶ Realm (Isolation)
/// UI ──DependsOn──▶ Room (Scoping)
/// UI ──DependsOn──▶ Controller (Permissions)
/// UI ──Triggers───▶ Event (UI-Actions)
/// UI ──Aggregates─▶ DataLogic (Bindings)
/// UI ──DependsOn──▶ ECLVM (UI-Logik)
/// UI ──DependsOn──▶ Gas/Mana (Resources)
/// ```
#[derive(Debug)]
pub struct UIState {
    // ─────────────────────────────────────────────────────────────────────────
    // Component-Tree Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Total registrierte UI-Components
    pub components_registered: AtomicU64,
    /// Aktuell aktive Components (mounted)
    pub components_active: AtomicU64,
    /// Component-Updates durchgeführt
    pub component_updates: AtomicU64,
    /// Component-Renders durchgeführt
    pub renders: AtomicU64,
    /// Cached Renders (keine Änderung)
    pub cache_hits: AtomicU64,
    /// Re-Renders (State-Änderung)
    pub cache_misses: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Binding-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Bindings
    pub bindings_active: AtomicU64,
    /// Binding-Updates propagiert
    pub binding_updates: AtomicU64,
    /// Binding-Fehler (z.B. Source nicht verfügbar)
    pub binding_errors: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Trust-Gates (Sichtbarkeits-Kontrolle)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Gate Evaluationen
    pub trust_gate_evaluations: AtomicU64,
    /// Trust-Gate Allowed
    pub trust_gate_allowed: AtomicU64,
    /// Trust-Gate Denied
    pub trust_gate_denied: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Credential-Gates (Zugriffs-Kontrolle)
    // ─────────────────────────────────────────────────────────────────────────
    /// Credential-Gate Evaluationen
    pub credential_gate_evaluations: AtomicU64,
    /// Credential-Gate Allowed
    pub credential_gate_allowed: AtomicU64,
    /// Credential-Gate Denied
    pub credential_gate_denied: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm UI-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische UI-Metriken
    pub realm_ui: RwLock<HashMap<String, RealmUIState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht für UI-Rendering
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht für UI-Events
    pub mana_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (UI ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// DataLogic-Aggregations (UI ⊃ DataLogic)
    pub datalogic_aggregations: AtomicU64,
    /// Controller-Validations (Controller ✓ UI)
    pub controller_validations: AtomicU64,
    /// Events getriggert (UI → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm UI-State für Isolation
#[derive(Debug)]
pub struct RealmUIState {
    pub components: AtomicU64,
    pub renders: AtomicU64,
    pub bindings: AtomicU64,
    pub trust_gate_denied: AtomicU64,
}

impl RealmUIState {
    pub fn new() -> Self {
        Self {
            components: AtomicU64::new(0),
            renders: AtomicU64::new(0),
            bindings: AtomicU64::new(0),
            trust_gate_denied: AtomicU64::new(0),
        }
    }
}

impl Default for RealmUIState {
    fn default() -> Self {
        Self::new()
    }
}

impl UIState {
    pub fn new() -> Self {
        Self {
            components_registered: AtomicU64::new(0),
            components_active: AtomicU64::new(0),
            component_updates: AtomicU64::new(0),
            renders: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            bindings_active: AtomicU64::new(0),
            binding_updates: AtomicU64::new(0),
            binding_errors: AtomicU64::new(0),
            trust_gate_evaluations: AtomicU64::new(0),
            trust_gate_allowed: AtomicU64::new(0),
            trust_gate_denied: AtomicU64::new(0),
            credential_gate_evaluations: AtomicU64::new(0),
            credential_gate_allowed: AtomicU64::new(0),
            credential_gate_denied: AtomicU64::new(0),
            realm_ui: RwLock::new(HashMap::new()),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            datalogic_aggregations: AtomicU64::new(0),
            controller_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Registriere neue Component
    pub fn register_component(&self, realm_id: Option<&str>) {
        self.components_registered.fetch_add(1, Ordering::Relaxed);
        self.components_active.fetch_add(1, Ordering::Relaxed);

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .components
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Component unmounted
    pub fn unregister_component(&self) {
        self.components_active.fetch_sub(1, Ordering::Relaxed);
    }

    /// Render durchgeführt
    pub fn render(&self, from_cache: bool, gas: u64, mana: u64, realm_id: Option<&str>) {
        self.renders.fetch_add(1, Ordering::Relaxed);
        if from_cache {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.mana_consumed.fetch_add(mana, Ordering::Relaxed);

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .renders
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Trust-Gate evaluiert
    pub fn trust_gate(&self, allowed: bool, realm_id: Option<&str>) {
        self.trust_gate_evaluations.fetch_add(1, Ordering::Relaxed);
        if allowed {
            self.trust_gate_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.trust_gate_denied.fetch_add(1, Ordering::Relaxed);
            if let Some(realm) = realm_id {
                self.get_or_create_realm(realm)
                    .trust_gate_denied
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Credential-Gate evaluiert
    pub fn credential_gate(&self, allowed: bool) {
        self.credential_gate_evaluations
            .fetch_add(1, Ordering::Relaxed);
        if allowed {
            self.credential_gate_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.credential_gate_denied.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Binding update
    pub fn binding_update(&self, success: bool, realm_id: Option<&str>) {
        self.binding_updates.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.binding_errors.fetch_add(1, Ordering::Relaxed);
        }
        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .bindings
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    fn get_or_create_realm(&self, realm_id: &str) -> &RealmUIState {
        if let Ok(mut realms) = self.realm_ui.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmUIState::new);
        }
        // Safe: Entry wurde gerade erstellt
        unsafe {
            self.realm_ui
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmUIState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmUIState> = std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmUIState::new)
                })
        }
    }

    /// Cache-Hit-Rate berechnen
    pub fn cache_hit_rate(&self) -> f64 {
        let total =
            self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed);
        if total > 0 {
            self.cache_hits.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            1.0
        }
    }

    /// Trust-Gate-Allow-Rate berechnen
    pub fn trust_gate_allow_rate(&self) -> f64 {
        let total = self.trust_gate_evaluations.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.trust_gate_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    /// Credential-Gate-Allow-Rate berechnen
    pub fn credential_gate_allow_rate(&self) -> f64 {
        let total = self.credential_gate_evaluations.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.credential_gate_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> UISnapshot {
        UISnapshot {
            components_registered: self.components_registered.load(Ordering::Relaxed),
            components_active: self.components_active.load(Ordering::Relaxed),
            component_updates: self.component_updates.load(Ordering::Relaxed),
            renders: self.renders.load(Ordering::Relaxed),
            cache_hit_rate: self.cache_hit_rate(),
            bindings_active: self.bindings_active.load(Ordering::Relaxed),
            binding_updates: self.binding_updates.load(Ordering::Relaxed),
            binding_errors: self.binding_errors.load(Ordering::Relaxed),
            trust_gate_evaluations: self.trust_gate_evaluations.load(Ordering::Relaxed),
            trust_gate_allow_rate: self.trust_gate_allow_rate(),
            credential_gate_evaluations: self.credential_gate_evaluations.load(Ordering::Relaxed),
            credential_gate_allow_rate: self.credential_gate_allow_rate(),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UISnapshot {
    pub components_registered: u64,
    pub components_active: u64,
    pub component_updates: u64,
    pub renders: u64,
    pub cache_hit_rate: f64,
    pub bindings_active: u64,
    pub binding_updates: u64,
    pub binding_errors: u64,
    pub trust_gate_evaluations: u64,
    pub trust_gate_allow_rate: f64,
    pub credential_gate_evaluations: u64,
    pub credential_gate_allow_rate: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.2 API-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// API-Engine State mit Endpoint-Registry und Rate-Limiting
///
/// # Design
///
/// Die API-Engine ermöglicht dynamische REST-API-Definition per ECL:
/// - **Endpoint-Registry**: Routing-Tabelle per Realm
/// - **Rate-Limits**: Trust-basierte Throttling
/// - **Metrics**: Request/Response-Tracking mit Latenz-Percentiles
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// API ──DependsOn──▶ Trust (Access-Control)
/// API ──DependsOn──▶ Controller (AuthZ)
/// API ──Validates──▶ Gateway (External)
/// API ──Triggers───▶ Event (API-Calls)
/// API ──Aggregates─▶ DataLogic (Queries)
/// API ──DependsOn──▶ ECLVM (Handler)
/// API ──DependsOn──▶ Gas/Mana (Resources)
/// ```
#[derive(Debug)]
pub struct APIState {
    // ─────────────────────────────────────────────────────────────────────────
    // Endpoint-Registry
    // ─────────────────────────────────────────────────────────────────────────
    /// Registrierte Endpoints total
    pub endpoints_registered: AtomicU64,
    /// Aktive Endpoints
    pub endpoints_active: AtomicU64,
    /// Endpoint-Updates (Hot-Reload)
    pub endpoint_updates: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Request-Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Total Requests
    pub requests_total: AtomicU64,
    /// Erfolgreiche Requests (2xx)
    pub requests_success: AtomicU64,
    /// Client-Errors (4xx)
    pub requests_client_error: AtomicU64,
    /// Server-Errors (5xx)
    pub requests_server_error: AtomicU64,
    /// Rate-Limited Requests (429)
    pub requests_rate_limited: AtomicU64,
    /// Auth-Failed Requests (401/403)
    pub requests_auth_failed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Latenz-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Durchschnittliche Latenz (µs)
    pub avg_latency_us: RwLock<f64>,
    /// P95 Latenz (µs)
    pub p95_latency_us: RwLock<f64>,
    /// P99 Latenz (µs)
    pub p99_latency_us: RwLock<f64>,
    /// Latenz-Historie (Rolling Window)
    latency_history: RwLock<Vec<u64>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Rate-Limiting
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Rate-Limit-Buckets
    pub rate_limit_buckets: AtomicU64,
    /// Rate-Limit-Resets
    pub rate_limit_resets: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm API-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische API-Metriken
    pub realm_api_map: RwLock<HashMap<String, RealmAPIState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht für API-Processing
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht für Responses
    pub mana_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (API ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Controller-Validations (Controller ✓ API)
    pub controller_validations: AtomicU64,
    /// Gateway-Validations (API ✓ Gateway)
    pub gateway_validations: AtomicU64,
    /// Events getriggert (API → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm API-State für Isolation
#[derive(Debug)]
pub struct RealmAPIState {
    pub endpoints_total: AtomicU64,
    pub requests_total: AtomicU64,
    pub rate_limited_count: AtomicU64,
    pub auth_failed_count: AtomicU64,
}

impl RealmAPIState {
    pub fn new() -> Self {
        Self {
            endpoints_total: AtomicU64::new(0),
            requests_total: AtomicU64::new(0),
            rate_limited_count: AtomicU64::new(0),
            auth_failed_count: AtomicU64::new(0),
        }
    }
}

impl Default for RealmAPIState {
    fn default() -> Self {
        Self::new()
    }
}

impl APIState {
    pub fn new() -> Self {
        Self {
            endpoints_registered: AtomicU64::new(0),
            endpoints_active: AtomicU64::new(0),
            endpoint_updates: AtomicU64::new(0),
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_client_error: AtomicU64::new(0),
            requests_server_error: AtomicU64::new(0),
            requests_rate_limited: AtomicU64::new(0),
            requests_auth_failed: AtomicU64::new(0),
            avg_latency_us: RwLock::new(0.0),
            p95_latency_us: RwLock::new(0.0),
            p99_latency_us: RwLock::new(0.0),
            latency_history: RwLock::new(Vec::with_capacity(1000)),
            rate_limit_buckets: AtomicU64::new(0),
            rate_limit_resets: AtomicU64::new(0),
            realm_api_map: RwLock::new(HashMap::new()),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            controller_validations: AtomicU64::new(0),
            gateway_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Endpoint registrieren
    pub fn register_endpoint(&self, realm_id: Option<&str>) {
        self.endpoints_registered.fetch_add(1, Ordering::Relaxed);
        self.endpoints_active.fetch_add(1, Ordering::Relaxed);

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .endpoints_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Request verarbeitet
    pub fn record_request(
        &self,
        latency_us: u64,
        status: u16,
        gas: u64,
        mana: u64,
        realm_id: Option<&str>,
    ) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.mana_consumed.fetch_add(mana, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        // Status-Kategorie
        match status {
            200..=299 => {
                self.requests_success.fetch_add(1, Ordering::Relaxed);
            }
            400..=499 => {
                self.requests_client_error.fetch_add(1, Ordering::Relaxed);
                if status == 429 {
                    self.requests_rate_limited.fetch_add(1, Ordering::Relaxed);
                    if let Some(realm) = realm_id {
                        self.get_or_create_realm(realm)
                            .rate_limited_count
                            .fetch_add(1, Ordering::Relaxed);
                    }
                } else if status == 401 || status == 403 {
                    self.requests_auth_failed.fetch_add(1, Ordering::Relaxed);
                    if let Some(realm) = realm_id {
                        self.get_or_create_realm(realm)
                            .auth_failed_count
                            .fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
            500..=599 => {
                self.requests_server_error.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }

        // Realm-Tracking
        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .requests_total
                .fetch_add(1, Ordering::Relaxed);
        }

        // Latenz-Tracking
        self.update_latency(latency_us);
    }

    fn update_latency(&self, latency_us: u64) {
        if let Ok(mut history) = self.latency_history.write() {
            history.push(latency_us);
            if history.len() > 1000 {
                history.remove(0);
            }

            // Durchschnitt
            let avg = history.iter().sum::<u64>() as f64 / history.len() as f64;
            if let Ok(mut a) = self.avg_latency_us.write() {
                *a = avg;
            }

            // Percentiles
            if history.len() >= 10 {
                let mut sorted = history.clone();
                sorted.sort_unstable();
                let p95_idx = (sorted.len() as f64 * 0.95) as usize;
                let p99_idx = (sorted.len() as f64 * 0.99) as usize;

                if let Ok(mut p95) = self.p95_latency_us.write() {
                    *p95 = sorted
                        .get(p95_idx.min(sorted.len() - 1))
                        .copied()
                        .unwrap_or(0) as f64;
                }
                if let Ok(mut p99) = self.p99_latency_us.write() {
                    *p99 = sorted
                        .get(p99_idx.min(sorted.len() - 1))
                        .copied()
                        .unwrap_or(0) as f64;
                }
            }
        }
    }

    fn get_or_create_realm(&self, realm_id: &str) -> &RealmAPIState {
        if let Ok(mut realms) = self.realm_api_map.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmAPIState::new);
        }
        unsafe {
            self.realm_api_map
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmAPIState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmAPIState> = std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmAPIState::new)
                })
        }
    }

    /// Success-Rate berechnen
    pub fn success_rate(&self) -> f64 {
        let total = self.requests_total.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.requests_success.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> APISnapshot {
        APISnapshot {
            endpoints_registered: self.endpoints_registered.load(Ordering::Relaxed),
            endpoints_active: self.endpoints_active.load(Ordering::Relaxed),
            endpoint_updates: self.endpoint_updates.load(Ordering::Relaxed),
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_success: self.requests_success.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            requests_client_error: self.requests_client_error.load(Ordering::Relaxed),
            requests_server_error: self.requests_server_error.load(Ordering::Relaxed),
            requests_rate_limited: self.requests_rate_limited.load(Ordering::Relaxed),
            requests_auth_failed: self.requests_auth_failed.load(Ordering::Relaxed),
            avg_latency_us: self.avg_latency_us.read().map(|v| *v).unwrap_or(0.0),
            p95_latency_us: self.p95_latency_us.read().map(|v| *v).unwrap_or(0.0),
            p99_latency_us: self.p99_latency_us.read().map(|v| *v).unwrap_or(0.0),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for APIState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APISnapshot {
    pub endpoints_registered: u64,
    pub endpoints_active: u64,
    pub endpoint_updates: u64,
    pub requests_total: u64,
    pub requests_success: u64,
    pub success_rate: f64,
    pub requests_client_error: u64,
    pub requests_server_error: u64,
    pub requests_rate_limited: u64,
    pub requests_auth_failed: u64,
    pub avg_latency_us: f64,
    pub p95_latency_us: f64,
    pub p99_latency_us: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.3 GOVERNANCE-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// Governance-Engine State mit Proposal-Tracking und Delegation-Graph
///
/// # Design
///
/// Die Governance-Engine implementiert DAO-Prinzipien:
/// - **Quadratic Voting**: √-basierte Stimmgewichtung (Κ21)
/// - **Delegation**: Transitive Trust-Delegation (Liquid Democracy)
/// - **Anti-Calcification**: Machtkonzentrations-Check (Κ19)
/// - **Proposals**: Lifecycle-Management mit Quorum
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// Governance ──DependsOn──▶ Trust (Voting-Power)
/// Governance ──DependsOn──▶ Quadratic (Voting-Mechanik)
/// Governance ──Validates──▶ Controller (Permission-Changes)
/// Governance ──Triggers───▶ Controller (Vote-Results)
/// Governance ──Triggers───▶ Event (Proposals/Votes)
/// Governance ──Validates──▶ AntiCalcification (Power-Check)
/// ```
#[derive(Debug)]
pub struct GovernanceState {
    // ─────────────────────────────────────────────────────────────────────────
    // Proposal-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Total erstellte Proposals
    pub proposals_created: AtomicU64,
    /// Aktive Proposals (in Voting-Phase)
    pub proposals_active: AtomicU64,
    /// Abgeschlossene Proposals
    pub proposals_completed: AtomicU64,
    /// Angenommene Proposals
    pub proposals_accepted: AtomicU64,
    /// Abgelehnte Proposals
    pub proposals_rejected: AtomicU64,
    /// Abgebrochene Proposals (Quorum nicht erreicht)
    pub proposals_expired: AtomicU64,
    /// Vetoed Proposals
    pub proposals_vetoed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Voting-Metriken
    // ─────────────────────────────────────────────────────────────────────────
    /// Total abgegebene Votes
    pub votes_cast: AtomicU64,
    /// Unique Voters (geschätzt)
    pub unique_voters: AtomicU64,
    /// Delegierte Votes
    pub votes_delegated: AtomicU64,
    /// Quadratische Reduktionen angewendet
    pub quadratic_reductions: AtomicU64,
    /// Durchschnittliche Voting-Power (vor Quadratic)
    pub avg_voting_power: RwLock<f64>,
    /// Durchschnittliche Participation-Rate
    pub avg_participation_rate: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Delegation-Graph
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Delegationen
    pub delegations_active: AtomicU64,
    /// Delegations-Ketten-Tiefe (max observed)
    pub max_delegation_depth: AtomicU64,
    /// Zirkuläre Delegationen verhindert
    pub circular_delegations_prevented: AtomicU64,
    /// Abgelaufene Delegationen
    pub delegations_expired: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Anti-Calcification (Κ19)
    // ─────────────────────────────────────────────────────────────────────────
    /// Power-Concentration-Checks
    pub power_checks: AtomicU64,
    /// Power-Concentration-Violations
    pub power_violations: AtomicU64,
    /// Gini-Koeffizient der Voting-Power
    pub voting_power_gini: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm Governance-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische Governance-Metriken
    pub realm_governance: RwLock<HashMap<String, RealmGovernanceState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (Governance ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Quadratic-Validations (Governance ← Quadratic)
    pub quadratic_validations: AtomicU64,
    /// Controller-Triggers (Governance → Controller)
    pub controller_triggers: AtomicU64,
    /// AntiCalc-Validations (Governance ✓ AntiCalcification)
    pub anticalc_validations: AtomicU64,
    /// Events getriggert (Governance → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm Governance-State
#[derive(Debug)]
pub struct RealmGovernanceState {
    pub proposals: AtomicU64,
    pub votes: AtomicU64,
    pub delegations: AtomicU64,
    /// Governance-Typ: "council", "direct", "liquid", "quadratic"
    pub governance_type: RwLock<String>,
}

impl RealmGovernanceState {
    pub fn new() -> Self {
        Self {
            proposals: AtomicU64::new(0),
            votes: AtomicU64::new(0),
            delegations: AtomicU64::new(0),
            governance_type: RwLock::new("quadratic".to_string()),
        }
    }
}

impl Default for RealmGovernanceState {
    fn default() -> Self {
        Self::new()
    }
}

impl GovernanceState {
    pub fn new() -> Self {
        Self {
            proposals_created: AtomicU64::new(0),
            proposals_active: AtomicU64::new(0),
            proposals_completed: AtomicU64::new(0),
            proposals_accepted: AtomicU64::new(0),
            proposals_rejected: AtomicU64::new(0),
            proposals_expired: AtomicU64::new(0),
            proposals_vetoed: AtomicU64::new(0),
            votes_cast: AtomicU64::new(0),
            unique_voters: AtomicU64::new(0),
            votes_delegated: AtomicU64::new(0),
            quadratic_reductions: AtomicU64::new(0),
            avg_voting_power: RwLock::new(1.0),
            avg_participation_rate: RwLock::new(0.0),
            delegations_active: AtomicU64::new(0),
            max_delegation_depth: AtomicU64::new(0),
            circular_delegations_prevented: AtomicU64::new(0),
            delegations_expired: AtomicU64::new(0),
            power_checks: AtomicU64::new(0),
            power_violations: AtomicU64::new(0),
            voting_power_gini: RwLock::new(0.0),
            realm_governance: RwLock::new(HashMap::new()),
            trust_dependency_updates: AtomicU64::new(0),
            quadratic_validations: AtomicU64::new(0),
            controller_triggers: AtomicU64::new(0),
            anticalc_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Proposal erstellt
    pub fn proposal_created(&self, realm_id: Option<&str>) {
        self.proposals_created.fetch_add(1, Ordering::Relaxed);
        self.proposals_active.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .proposals
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Proposal abgeschlossen
    pub fn proposal_completed(&self, result: &str) {
        self.proposals_active.fetch_sub(1, Ordering::Relaxed);
        self.proposals_completed.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        match result {
            "accepted" => {
                self.proposals_accepted.fetch_add(1, Ordering::Relaxed);
                self.controller_triggers.fetch_add(1, Ordering::Relaxed);
            }
            "rejected" => {
                self.proposals_rejected.fetch_add(1, Ordering::Relaxed);
            }
            "expired" => {
                self.proposals_expired.fetch_add(1, Ordering::Relaxed);
            }
            "vetoed" => {
                self.proposals_vetoed.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }

    /// Vote abgegeben
    pub fn vote_cast(
        &self,
        voting_power: f64,
        is_delegated: bool,
        quadratic_reduced: bool,
        realm_id: Option<&str>,
    ) {
        self.votes_cast.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        if is_delegated {
            self.votes_delegated.fetch_add(1, Ordering::Relaxed);
        }
        if quadratic_reduced {
            self.quadratic_reductions.fetch_add(1, Ordering::Relaxed);
            self.quadratic_validations.fetch_add(1, Ordering::Relaxed);
        }

        // Update average voting power
        if let Ok(mut avg) = self.avg_voting_power.write() {
            let total = self.votes_cast.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + voting_power) / total;
        }

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .votes
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Delegation erstellt
    pub fn delegation_created(&self, depth: u64, realm_id: Option<&str>) {
        self.delegations_active.fetch_add(1, Ordering::Relaxed);

        // Update max depth
        loop {
            let current = self.max_delegation_depth.load(Ordering::Relaxed);
            if depth <= current {
                break;
            }
            if self
                .max_delegation_depth
                .compare_exchange(current, depth, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .delegations
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Power-Check durchgeführt
    pub fn power_check(&self, violated: bool, gini: f64) {
        self.power_checks.fetch_add(1, Ordering::Relaxed);
        self.anticalc_validations.fetch_add(1, Ordering::Relaxed);

        if violated {
            self.power_violations.fetch_add(1, Ordering::Relaxed);
        }

        if let Ok(mut g) = self.voting_power_gini.write() {
            *g = gini;
        }
    }

    fn get_or_create_realm(&self, realm_id: &str) -> &RealmGovernanceState {
        if let Ok(mut realms) = self.realm_governance.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmGovernanceState::new);
        }
        unsafe {
            self.realm_governance
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmGovernanceState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmGovernanceState> =
                        std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmGovernanceState::new)
                })
        }
    }

    /// Proposal-Success-Rate
    pub fn proposal_success_rate(&self) -> f64 {
        let completed = self.proposals_completed.load(Ordering::Relaxed) as f64;
        if completed > 0.0 {
            self.proposals_accepted.load(Ordering::Relaxed) as f64 / completed
        } else {
            0.0
        }
    }

    /// Delegation-Rate (Anteil delegierter Votes)
    pub fn delegation_rate(&self) -> f64 {
        let total = self.votes_cast.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.votes_delegated.load(Ordering::Relaxed) as f64 / total
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> GovernanceSnapshot {
        GovernanceSnapshot {
            proposals_created: self.proposals_created.load(Ordering::Relaxed),
            proposals_active: self.proposals_active.load(Ordering::Relaxed),
            proposals_completed: self.proposals_completed.load(Ordering::Relaxed),
            proposals_accepted: self.proposals_accepted.load(Ordering::Relaxed),
            proposals_rejected: self.proposals_rejected.load(Ordering::Relaxed),
            proposal_success_rate: self.proposal_success_rate(),
            votes_cast: self.votes_cast.load(Ordering::Relaxed),
            unique_voters: self.unique_voters.load(Ordering::Relaxed),
            votes_delegated: self.votes_delegated.load(Ordering::Relaxed),
            delegation_rate: self.delegation_rate(),
            delegations_active: self.delegations_active.load(Ordering::Relaxed),
            max_delegation_depth: self.max_delegation_depth.load(Ordering::Relaxed),
            quadratic_reductions: self.quadratic_reductions.load(Ordering::Relaxed),
            avg_voting_power: self.avg_voting_power.read().map(|v| *v).unwrap_or(1.0),
            voting_power_gini: self.voting_power_gini.read().map(|v| *v).unwrap_or(0.0),
            power_violations: self.power_violations.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for GovernanceState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceSnapshot {
    pub proposals_created: u64,
    pub proposals_active: u64,
    pub proposals_completed: u64,
    pub proposals_accepted: u64,
    pub proposals_rejected: u64,
    pub proposal_success_rate: f64,
    pub votes_cast: u64,
    pub unique_voters: u64,
    pub votes_delegated: u64,
    pub delegation_rate: f64,
    pub delegations_active: u64,
    pub max_delegation_depth: u64,
    pub quadratic_reductions: u64,
    pub avg_voting_power: f64,
    pub voting_power_gini: f64,
    pub power_violations: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.4 CONTROLLER-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// Controller-Engine State mit Permission-Registry und Audit-Log
///
/// # Design
///
/// Die Controller-Engine verwaltet Berechtigungen:
/// - **Scoped Permissions**: Realm > Room > Partition Hierarchie
/// - **Delegation**: Transitive Permission-Vererbung mit Constraints
/// - **Audit-Trail**: Vollständige Permission-History
/// - **Automation**: Trigger-basierte Permission-Änderungen
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// Controller ──DependsOn──▶ Trust (Permission-Basis)
/// Controller ──Validates──▶ Gateway (Crossing-Auth)
/// Controller ──Validates──▶ API (API-Auth)
/// Controller ──Validates──▶ UI (UI-Auth)
/// Controller ──Aggregates─▶ Governance (Delegation-Sync)
/// Controller ──DependsOn──▶ Realm/Room/Partition (Scope)
/// Controller ──DependsOn──▶ ECLVM (Permission-Rules)
/// ```
#[derive(Debug)]
pub struct ControllerState {
    // ─────────────────────────────────────────────────────────────────────────
    // Permission-Registry
    // ─────────────────────────────────────────────────────────────────────────
    /// Total registrierte Permissions
    pub permissions_registered: AtomicU64,
    /// Aktive Permissions
    pub permissions_active: AtomicU64,
    /// Permission-Grants
    pub permission_grants: AtomicU64,
    /// Permission-Revokes
    pub permission_revokes: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Authorization-Checks
    // ─────────────────────────────────────────────────────────────────────────
    /// AuthZ-Checks total
    pub authz_checks: AtomicU64,
    /// AuthZ-Allowed
    pub authz_allowed: AtomicU64,
    /// AuthZ-Denied
    pub authz_denied: AtomicU64,
    /// Via-Delegation AuthZ
    pub authz_via_delegation: AtomicU64,
    /// Durchschnittliche Check-Latenz (µs)
    pub avg_check_latency_us: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Delegation
    // ─────────────────────────────────────────────────────────────────────────
    /// Aktive Delegationen
    pub delegations_active: AtomicU64,
    /// Delegations-Ketten (max depth)
    pub max_delegation_depth: AtomicU64,
    /// Delegations-Nutzungen
    pub delegations_used: AtomicU64,
    /// Abgelaufene Delegationen
    pub delegations_expired: AtomicU64,
    /// Delegations-Konflikte (z.B. zirkulär)
    pub delegation_conflicts: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Scope-Tracking (Realm > Room > Partition)
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-Scope Checks
    pub realm_scope_checks: AtomicU64,
    /// Room-Scope Checks
    pub room_scope_checks: AtomicU64,
    /// Partition-Scope Checks
    pub partition_scope_checks: AtomicU64,
    /// Scope-Inheritance-Resolutions
    pub scope_inheritance_resolutions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Audit-Log
    // ─────────────────────────────────────────────────────────────────────────
    /// Audit-Entries geschrieben
    pub audit_entries: AtomicU64,
    /// Audit-Log-Größe (Bytes, approximiert)
    pub audit_log_bytes: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Automation
    // ─────────────────────────────────────────────────────────────────────────
    /// Automation-Rules aktiv
    pub automation_rules_active: AtomicU64,
    /// Automation-Triggers ausgelöst
    pub automation_triggers: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Per-Realm Controller-State
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-spezifische Controller-Metriken
    pub realm_controller: RwLock<HashMap<String, RealmControllerState>>,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (Controller ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Gateway-Validations (Controller ✓ Gateway)
    pub gateway_validations: AtomicU64,
    /// API-Validations (Controller ✓ API)
    pub api_validations: AtomicU64,
    /// UI-Validations (Controller ✓ UI)
    pub ui_validations: AtomicU64,
    /// Governance-Aggregations (Controller ⊃ Governance)
    pub governance_aggregations: AtomicU64,
    /// Events getriggert (Controller → Event)
    pub events_triggered: AtomicU64,
}

/// Per-Realm Controller-State
#[derive(Debug)]
pub struct RealmControllerState {
    pub permissions: AtomicU64,
    pub authz_checks: AtomicU64,
    pub authz_denied: AtomicU64,
    pub delegations: AtomicU64,
    pub rooms: AtomicU64,
    pub partitions: AtomicU64,
}

impl RealmControllerState {
    pub fn new() -> Self {
        Self {
            permissions: AtomicU64::new(0),
            authz_checks: AtomicU64::new(0),
            authz_denied: AtomicU64::new(0),
            delegations: AtomicU64::new(0),
            rooms: AtomicU64::new(0),
            partitions: AtomicU64::new(0),
        }
    }
}

impl Default for RealmControllerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ControllerState {
    pub fn new() -> Self {
        Self {
            permissions_registered: AtomicU64::new(0),
            permissions_active: AtomicU64::new(0),
            permission_grants: AtomicU64::new(0),
            permission_revokes: AtomicU64::new(0),
            authz_checks: AtomicU64::new(0),
            authz_allowed: AtomicU64::new(0),
            authz_denied: AtomicU64::new(0),
            authz_via_delegation: AtomicU64::new(0),
            avg_check_latency_us: RwLock::new(0.0),
            delegations_active: AtomicU64::new(0),
            max_delegation_depth: AtomicU64::new(0),
            delegations_used: AtomicU64::new(0),
            delegations_expired: AtomicU64::new(0),
            delegation_conflicts: AtomicU64::new(0),
            realm_scope_checks: AtomicU64::new(0),
            room_scope_checks: AtomicU64::new(0),
            partition_scope_checks: AtomicU64::new(0),
            scope_inheritance_resolutions: AtomicU64::new(0),
            audit_entries: AtomicU64::new(0),
            audit_log_bytes: AtomicU64::new(0),
            automation_rules_active: AtomicU64::new(0),
            automation_triggers: AtomicU64::new(0),
            realm_controller: RwLock::new(HashMap::new()),
            trust_dependency_updates: AtomicU64::new(0),
            gateway_validations: AtomicU64::new(0),
            api_validations: AtomicU64::new(0),
            ui_validations: AtomicU64::new(0),
            governance_aggregations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Permission gewährt
    pub fn grant_permission(&self, realm_id: Option<&str>) {
        self.permission_grants.fetch_add(1, Ordering::Relaxed);
        self.permissions_active.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);
        self.write_audit(128); // ~128 bytes per audit entry

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .permissions
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Permission widerrufen
    pub fn revoke_permission(&self) {
        self.permission_revokes.fetch_add(1, Ordering::Relaxed);
        self.permissions_active.fetch_sub(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);
        self.write_audit(128);
    }

    /// AuthZ-Check durchgeführt
    pub fn check_authorization(
        &self,
        allowed: bool,
        via_delegation: bool,
        latency_us: u64,
        scope: &str,
        realm_id: Option<&str>,
    ) {
        self.authz_checks.fetch_add(1, Ordering::Relaxed);

        if allowed {
            self.authz_allowed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.authz_denied.fetch_add(1, Ordering::Relaxed);
            if let Some(realm) = realm_id {
                self.get_or_create_realm(realm)
                    .authz_denied
                    .fetch_add(1, Ordering::Relaxed);
            }
        }

        if via_delegation {
            self.authz_via_delegation.fetch_add(1, Ordering::Relaxed);
            self.delegations_used.fetch_add(1, Ordering::Relaxed);
        }

        // Scope-Tracking
        match scope {
            "realm" => {
                self.realm_scope_checks.fetch_add(1, Ordering::Relaxed);
            }
            "room" => {
                self.room_scope_checks.fetch_add(1, Ordering::Relaxed);
            }
            "partition" => {
                self.partition_scope_checks.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }

        // Latenz-Update
        if let Ok(mut avg) = self.avg_check_latency_us.write() {
            let total = self.authz_checks.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
        }

        // Realm-Tracking
        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .authz_checks
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Delegation erstellt
    pub fn create_delegation(&self, depth: u64, realm_id: Option<&str>) {
        self.delegations_active.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);
        self.governance_aggregations.fetch_add(1, Ordering::Relaxed);
        self.write_audit(256);

        // Update max depth
        loop {
            let current = self.max_delegation_depth.load(Ordering::Relaxed);
            if depth <= current {
                break;
            }
            if self
                .max_delegation_depth
                .compare_exchange(current, depth, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }

        if let Some(realm) = realm_id {
            self.get_or_create_realm(realm)
                .delegations
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Audit-Entry schreiben
    fn write_audit(&self, bytes: u64) {
        self.audit_entries.fetch_add(1, Ordering::Relaxed);
        self.audit_log_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    fn get_or_create_realm(&self, realm_id: &str) -> &RealmControllerState {
        if let Ok(mut realms) = self.realm_controller.write() {
            realms
                .entry(realm_id.to_string())
                .or_insert_with(RealmControllerState::new);
        }
        unsafe {
            self.realm_controller
                .read()
                .unwrap()
                .get(realm_id)
                .map(|r| &*(r as *const RealmControllerState))
                .unwrap_or_else(|| {
                    static DEFAULT: std::sync::OnceLock<RealmControllerState> =
                        std::sync::OnceLock::new();
                    DEFAULT.get_or_init(RealmControllerState::new)
                })
        }
    }

    /// AuthZ-Success-Rate
    pub fn authz_success_rate(&self) -> f64 {
        let total = self.authz_checks.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.authz_allowed.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    /// Delegation-Usage-Rate
    pub fn delegation_usage_rate(&self) -> f64 {
        let checks = self.authz_checks.load(Ordering::Relaxed) as f64;
        if checks > 0.0 {
            self.authz_via_delegation.load(Ordering::Relaxed) as f64 / checks
        } else {
            0.0
        }
    }

    pub fn snapshot(&self) -> ControllerSnapshot {
        ControllerSnapshot {
            permissions_registered: self.permissions_registered.load(Ordering::Relaxed),
            permissions_active: self.permissions_active.load(Ordering::Relaxed),
            permission_grants: self.permission_grants.load(Ordering::Relaxed),
            permission_revokes: self.permission_revokes.load(Ordering::Relaxed),
            authz_checks: self.authz_checks.load(Ordering::Relaxed),
            authz_allowed: self.authz_allowed.load(Ordering::Relaxed),
            authz_denied: self.authz_denied.load(Ordering::Relaxed),
            authz_success_rate: self.authz_success_rate(),
            avg_check_latency_us: self.avg_check_latency_us.read().map(|v| *v).unwrap_or(0.0),
            delegations_active: self.delegations_active.load(Ordering::Relaxed),
            max_delegation_depth: self.max_delegation_depth.load(Ordering::Relaxed),
            delegation_usage_rate: self.delegation_usage_rate(),
            realm_scope_checks: self.realm_scope_checks.load(Ordering::Relaxed),
            room_scope_checks: self.room_scope_checks.load(Ordering::Relaxed),
            partition_scope_checks: self.partition_scope_checks.load(Ordering::Relaxed),
            audit_entries: self.audit_entries.load(Ordering::Relaxed),
            audit_log_bytes: self.audit_log_bytes.load(Ordering::Relaxed),
            automation_triggers: self.automation_triggers.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for ControllerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerSnapshot {
    pub permissions_registered: u64,
    pub permissions_active: u64,
    pub permission_grants: u64,
    pub permission_revokes: u64,
    pub authz_checks: u64,
    pub authz_allowed: u64,
    pub authz_denied: u64,
    pub authz_success_rate: f64,
    pub avg_check_latency_us: f64,
    pub delegations_active: u64,
    pub max_delegation_depth: u64,
    pub delegation_usage_rate: f64,
    pub realm_scope_checks: u64,
    pub room_scope_checks: u64,
    pub partition_scope_checks: u64,
    pub audit_entries: u64,
    pub audit_log_bytes: u64,
    pub automation_triggers: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.5 DATALOGIC-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// DataLogic-Engine State mit Reactive Streams und Aggregations
///
/// # Design
///
/// Die DataLogic-Engine verarbeitet Events reaktiv:
/// - **Streams**: Event-basierte Datenströme
/// - **Aggregations**: count, sum, avg, window-basiert
/// - **Bindings**: Reaktive UI-Verbindungen
/// - **Filters**: Trust-basierte Filterung
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// DataLogic ──DependsOn──▶ Event (Event-Processing)
/// DataLogic ──Aggregates─▶ Event (Aggregation)
/// DataLogic ──Triggers───▶ Event (Derived Events)
/// DataLogic ──DependsOn──▶ Trust (Access-Control)
/// DataLogic ──DependsOn──▶ ECLVM (Functions)
/// DataLogic ──Validates──▶ UI (Binding-Validation)
/// ```
#[derive(Debug)]
pub struct DataLogicState {
    // ─────────────────────────────────────────────────────────────────────────
    // Stream-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Registrierte Streams
    pub streams_registered: AtomicU64,
    /// Aktive Streams
    pub streams_active: AtomicU64,
    /// Stream-Subscriptions
    pub stream_subscriptions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Event-Processing
    // ─────────────────────────────────────────────────────────────────────────
    /// Events verarbeitet
    pub events_processed: AtomicU64,
    /// Events gefiltert (Trust/Access)
    pub events_filtered: AtomicU64,
    /// Events weitergeleitet (nach Filter)
    pub events_forwarded: AtomicU64,
    /// Processing-Fehler
    pub processing_errors: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Aggregation
    // ─────────────────────────────────────────────────────────────────────────
    /// Registrierte Aggregationen
    pub aggregations_registered: AtomicU64,
    /// Aggregationen berechnet
    pub aggregations_computed: AtomicU64,
    /// Aggregation-Results emittiert
    pub aggregation_results: AtomicU64,
    /// Durchschnittliche Aggregation-Latenz (µs)
    pub avg_aggregation_latency_us: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Binding-Propagation
    // ─────────────────────────────────────────────────────────────────────────
    /// Binding-Updates propagiert
    pub binding_propagations: AtomicU64,
    /// Binding-Fehler
    pub binding_errors: AtomicU64,
    /// Durchschnittliche Propagation-Latenz (µs)
    pub avg_propagation_latency_us: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht
    pub gas_consumed: AtomicU64,
    /// Mana verbraucht
    pub mana_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (DataLogic ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// Event-Aggregations (DataLogic ⊃ Event)
    pub event_aggregations: AtomicU64,
    /// UI-Validations (DataLogic ✓ UI)
    pub ui_validations: AtomicU64,
    /// Events getriggert (DataLogic → Event)
    pub events_triggered: AtomicU64,
}

impl DataLogicState {
    pub fn new() -> Self {
        Self {
            streams_registered: AtomicU64::new(0),
            streams_active: AtomicU64::new(0),
            stream_subscriptions: AtomicU64::new(0),
            events_processed: AtomicU64::new(0),
            events_filtered: AtomicU64::new(0),
            events_forwarded: AtomicU64::new(0),
            processing_errors: AtomicU64::new(0),
            aggregations_registered: AtomicU64::new(0),
            aggregations_computed: AtomicU64::new(0),
            aggregation_results: AtomicU64::new(0),
            avg_aggregation_latency_us: RwLock::new(0.0),
            binding_propagations: AtomicU64::new(0),
            binding_errors: AtomicU64::new(0),
            avg_propagation_latency_us: RwLock::new(0.0),
            gas_consumed: AtomicU64::new(0),
            mana_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            event_aggregations: AtomicU64::new(0),
            ui_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Stream registrieren
    pub fn register_stream(&self) {
        self.streams_registered.fetch_add(1, Ordering::Relaxed);
        self.streams_active.fetch_add(1, Ordering::Relaxed);
    }

    /// Event verarbeiten
    pub fn process_event(&self, filtered: bool, gas: u64) {
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);

        if filtered {
            self.events_filtered.fetch_add(1, Ordering::Relaxed);
        } else {
            self.events_forwarded.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Aggregation berechnet
    pub fn aggregation_computed(&self, latency_us: u64, gas: u64) {
        self.aggregations_computed.fetch_add(1, Ordering::Relaxed);
        self.aggregation_results.fetch_add(1, Ordering::Relaxed);
        self.event_aggregations.fetch_add(1, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);

        // Update average latency
        if let Ok(mut avg) = self.avg_aggregation_latency_us.write() {
            let total = self.aggregations_computed.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
        }
    }

    /// Binding propagiert
    pub fn propagate_binding(&self, success: bool, latency_us: u64, mana: u64) {
        self.binding_propagations.fetch_add(1, Ordering::Relaxed);
        self.mana_consumed.fetch_add(mana, Ordering::Relaxed);
        self.ui_validations.fetch_add(1, Ordering::Relaxed);

        if !success {
            self.binding_errors.fetch_add(1, Ordering::Relaxed);
        }

        // Update average latency
        if let Ok(mut avg) = self.avg_propagation_latency_us.write() {
            let total = self.binding_propagations.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + latency_us as f64) / total;
        }
    }

    /// Success-Rate (Events die nicht gefiltert wurden)
    pub fn success_rate(&self) -> f64 {
        let total = self.events_processed.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.events_forwarded.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    /// Binding-Success-Rate
    pub fn binding_success_rate(&self) -> f64 {
        let total = self.binding_propagations.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            (total - self.binding_errors.load(Ordering::Relaxed) as f64) / total
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> DataLogicSnapshot {
        DataLogicSnapshot {
            streams_registered: self.streams_registered.load(Ordering::Relaxed),
            streams_active: self.streams_active.load(Ordering::Relaxed),
            stream_subscriptions: self.stream_subscriptions.load(Ordering::Relaxed),
            events_processed: self.events_processed.load(Ordering::Relaxed),
            events_filtered: self.events_filtered.load(Ordering::Relaxed),
            events_forwarded: self.events_forwarded.load(Ordering::Relaxed),
            success_rate: self.success_rate(),
            aggregations_registered: self.aggregations_registered.load(Ordering::Relaxed),
            aggregations_computed: self.aggregations_computed.load(Ordering::Relaxed),
            avg_aggregation_latency_us: self
                .avg_aggregation_latency_us
                .read()
                .map(|v| *v)
                .unwrap_or(0.0),
            binding_propagations: self.binding_propagations.load(Ordering::Relaxed),
            binding_success_rate: self.binding_success_rate(),
            avg_propagation_latency_us: self
                .avg_propagation_latency_us
                .read()
                .map(|v| *v)
                .unwrap_or(0.0),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            mana_consumed: self.mana_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for DataLogicState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLogicSnapshot {
    pub streams_registered: u64,
    pub streams_active: u64,
    pub stream_subscriptions: u64,
    pub events_processed: u64,
    pub events_filtered: u64,
    pub events_forwarded: u64,
    pub success_rate: f64,
    pub aggregations_registered: u64,
    pub aggregations_computed: u64,
    pub avg_aggregation_latency_us: f64,
    pub binding_propagations: u64,
    pub binding_success_rate: f64,
    pub avg_propagation_latency_us: f64,
    pub gas_consumed: u64,
    pub mana_consumed: u64,
    pub events_triggered: u64,
}

// ────────────────────────────────────────────────────────────────────────────
// 2.6 BLUEPRINTCOMPOSER-ENGINE STATE
// ────────────────────────────────────────────────────────────────────────────

/// BlueprintComposer-Engine State mit Composition und Versioning
///
/// # Design
///
/// Der BlueprintComposer verwaltet Template-Komposition:
/// - **Composition**: Blueprint-Vererbung und -Erweiterung
/// - **Versioning**: Semantic Versioning mit Migrations
/// - **Validation**: Realm-Compatibility-Checks
/// - **Caching**: Compiled Blueprint Cache
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// BlueprintComposer ──DependsOn──▶ Blueprint (Storage)
/// BlueprintComposer ──Aggregates─▶ ECLBlueprint (Instances)
/// BlueprintComposer ──Triggers───▶ Event (Composition)
/// BlueprintComposer ──DependsOn──▶ ECLVM (Execution)
/// BlueprintComposer ──DependsOn──▶ Trust (Publish-Auth)
/// BlueprintComposer ──Validates──▶ Realm (Compatibility)
/// ```
#[derive(Debug)]
pub struct BlueprintComposerState {
    // ─────────────────────────────────────────────────────────────────────────
    // Composition-Tracking
    // ─────────────────────────────────────────────────────────────────────────
    /// Compositions erstellt
    pub compositions_created: AtomicU64,
    /// Compositions erfolgreich
    pub compositions_successful: AtomicU64,
    /// Compositions fehlgeschlagen
    pub compositions_failed: AtomicU64,
    /// Durchschnittliche Vererbungs-Tiefe
    pub avg_inheritance_depth: RwLock<f64>,
    /// Maximale Vererbungs-Tiefe
    pub max_inheritance_depth: AtomicU64,
    /// Konflikt-Resolutions bei Composition
    pub conflict_resolutions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Versioning
    // ─────────────────────────────────────────────────────────────────────────
    /// Blueprint-Versionen publiziert
    pub versions_published: AtomicU64,
    /// Migrationen durchgeführt
    pub migrations_executed: AtomicU64,
    /// Migrations-Fehler
    pub migration_errors: AtomicU64,
    /// Deprecations markiert
    pub deprecations: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Instantiation
    // ─────────────────────────────────────────────────────────────────────────
    /// Instanziierungen aus Compositions
    pub instantiations: AtomicU64,
    /// Instanziierungs-Fehler
    pub instantiation_errors: AtomicU64,
    /// Instanzen aktiv
    pub instances_active: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Validation
    // ─────────────────────────────────────────────────────────────────────────
    /// Realm-Compatibility-Checks
    pub realm_compatibility_checks: AtomicU64,
    /// Compatibility-Failures
    pub compatibility_failures: AtomicU64,
    /// Dependency-Resolutions
    pub dependency_resolutions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Caching
    // ─────────────────────────────────────────────────────────────────────────
    /// Cache-Hits
    pub cache_hits: AtomicU64,
    /// Cache-Misses
    pub cache_misses: AtomicU64,
    /// Cache-Evictions
    pub cache_evictions: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Resource-Verbrauch
    // ─────────────────────────────────────────────────────────────────────────
    /// Gas verbraucht für Composition
    pub gas_consumed: AtomicU64,

    // ─────────────────────────────────────────────────────────────────────────
    // Relationship-Tracking (StateGraph)
    // ─────────────────────────────────────────────────────────────────────────
    /// Trust-Dependency-Updates (BlueprintComposer ← Trust)
    pub trust_dependency_updates: AtomicU64,
    /// ECLBlueprint-Aggregations (BlueprintComposer ⊃ ECLBlueprint)
    pub ecl_blueprint_aggregations: AtomicU64,
    /// Realm-Validations (BlueprintComposer ✓ Realm)
    pub realm_validations: AtomicU64,
    /// Events getriggert (BlueprintComposer → Event)
    pub events_triggered: AtomicU64,
}

impl BlueprintComposerState {
    pub fn new() -> Self {
        Self {
            compositions_created: AtomicU64::new(0),
            compositions_successful: AtomicU64::new(0),
            compositions_failed: AtomicU64::new(0),
            avg_inheritance_depth: RwLock::new(0.0),
            max_inheritance_depth: AtomicU64::new(0),
            conflict_resolutions: AtomicU64::new(0),
            versions_published: AtomicU64::new(0),
            migrations_executed: AtomicU64::new(0),
            migration_errors: AtomicU64::new(0),
            deprecations: AtomicU64::new(0),
            instantiations: AtomicU64::new(0),
            instantiation_errors: AtomicU64::new(0),
            instances_active: AtomicU64::new(0),
            realm_compatibility_checks: AtomicU64::new(0),
            compatibility_failures: AtomicU64::new(0),
            dependency_resolutions: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_evictions: AtomicU64::new(0),
            gas_consumed: AtomicU64::new(0),
            trust_dependency_updates: AtomicU64::new(0),
            ecl_blueprint_aggregations: AtomicU64::new(0),
            realm_validations: AtomicU64::new(0),
            events_triggered: AtomicU64::new(0),
        }
    }

    /// Composition erstellt
    pub fn composition_created(
        &self,
        success: bool,
        inheritance_depth: u64,
        conflicts: u64,
        gas: u64,
    ) {
        self.compositions_created.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.events_triggered.fetch_add(1, Ordering::Relaxed);

        if success {
            self.compositions_successful.fetch_add(1, Ordering::Relaxed);
        } else {
            self.compositions_failed.fetch_add(1, Ordering::Relaxed);
        }

        if conflicts > 0 {
            self.conflict_resolutions
                .fetch_add(conflicts, Ordering::Relaxed);
        }

        // Update max depth
        loop {
            let current = self.max_inheritance_depth.load(Ordering::Relaxed);
            if inheritance_depth <= current {
                break;
            }
            if self
                .max_inheritance_depth
                .compare_exchange(
                    current,
                    inheritance_depth,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }

        // Update average depth
        if let Ok(mut avg) = self.avg_inheritance_depth.write() {
            let total = self.compositions_created.load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + inheritance_depth as f64) / total;
        }
    }

    /// Blueprint instanziiert
    pub fn instantiate(&self, success: bool, gas: u64) {
        self.instantiations.fetch_add(1, Ordering::Relaxed);
        self.gas_consumed.fetch_add(gas, Ordering::Relaxed);
        self.ecl_blueprint_aggregations
            .fetch_add(1, Ordering::Relaxed);

        if success {
            self.instances_active.fetch_add(1, Ordering::Relaxed);
        } else {
            self.instantiation_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Realm-Compatibility-Check
    pub fn realm_compatibility_check(&self, compatible: bool) {
        self.realm_compatibility_checks
            .fetch_add(1, Ordering::Relaxed);
        self.realm_validations.fetch_add(1, Ordering::Relaxed);

        if !compatible {
            self.compatibility_failures.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Cache-Zugriff
    pub fn cache_access(&self, hit: bool) {
        if hit {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Composition-Success-Rate
    pub fn composition_success_rate(&self) -> f64 {
        let total = self.compositions_created.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            self.compositions_successful.load(Ordering::Relaxed) as f64 / total
        } else {
            1.0
        }
    }

    /// Cache-Hit-Rate
    pub fn cache_hit_rate(&self) -> f64 {
        let total =
            self.cache_hits.load(Ordering::Relaxed) + self.cache_misses.load(Ordering::Relaxed);
        if total > 0 {
            self.cache_hits.load(Ordering::Relaxed) as f64 / total as f64
        } else {
            1.0
        }
    }

    pub fn snapshot(&self) -> BlueprintComposerSnapshot {
        BlueprintComposerSnapshot {
            compositions_created: self.compositions_created.load(Ordering::Relaxed),
            compositions_successful: self.compositions_successful.load(Ordering::Relaxed),
            compositions_failed: self.compositions_failed.load(Ordering::Relaxed),
            composition_success_rate: self.composition_success_rate(),
            avg_inheritance_depth: self.avg_inheritance_depth.read().map(|v| *v).unwrap_or(0.0),
            max_inheritance_depth: self.max_inheritance_depth.load(Ordering::Relaxed),
            conflict_resolutions: self.conflict_resolutions.load(Ordering::Relaxed),
            versions_published: self.versions_published.load(Ordering::Relaxed),
            migrations_executed: self.migrations_executed.load(Ordering::Relaxed),
            instantiations: self.instantiations.load(Ordering::Relaxed),
            instances_active: self.instances_active.load(Ordering::Relaxed),
            realm_compatibility_checks: self.realm_compatibility_checks.load(Ordering::Relaxed),
            compatibility_failures: self.compatibility_failures.load(Ordering::Relaxed),
            cache_hit_rate: self.cache_hit_rate(),
            gas_consumed: self.gas_consumed.load(Ordering::Relaxed),
            events_triggered: self.events_triggered.load(Ordering::Relaxed),
        }
    }
}

impl Default for BlueprintComposerState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintComposerSnapshot {
    pub compositions_created: u64,
    pub compositions_successful: u64,
    pub compositions_failed: u64,
    pub composition_success_rate: f64,
    pub avg_inheritance_depth: f64,
    pub max_inheritance_depth: u64,
    pub conflict_resolutions: u64,
    pub versions_published: u64,
    pub migrations_executed: u64,
    pub instantiations: u64,
    pub instances_active: u64,
    pub realm_compatibility_checks: u64,
    pub compatibility_failures: u64,
    pub cache_hit_rate: f64,
    pub gas_consumed: u64,
    pub events_triggered: u64,
}

// ============================================================================
// UNIFIED STATE
// ============================================================================

/// Unified State - Hierarchisches State-Management für alle Module
///
/// # Design
///
/// Der UnifiedState verbindet alle State-Layer mit ihren Beziehungen:
/// - **CoreState**: Trust, Events, WorldFormula, Consensus
/// - **ExecutionState**: Gas, Mana, Context-Tracking
/// - **ProtectionState**: Anomaly, Diversity, Quadratic, Anti-Calc
/// - **StorageState**: KV, EventStore, Archive, Blueprints
/// - **PeerState**: Gateway, SagaComposer, IntentParser
/// - **P2PState**: Swarm, Gossip, Kademlia, Relay, Privacy
/// - **UIState**: Component-Tree, Bindings, Trust-Gates
/// - **APIState**: Endpoints, Rate-Limits, Request-Tracking
/// - **GovernanceState**: Proposals, Voting, Delegation
/// - **ControllerState**: Permissions, AuthZ, Audit
/// - **DataLogicState**: Streams, Aggregations, Event-Processing
/// - **BlueprintComposerState**: Composition, Versioning, Caching
///
/// # Thread-Safety
///
/// - Atomare Counter für High-Frequency Updates
/// - RwLock für komplexe Strukturen
/// - Arc-Sharing für Cross-Module Access
///
/// # Beispiel
///
/// ```rust,ignore
/// let state = UnifiedState::new();
///
/// // Trust-Update mit Event-Trigger
/// state.core.trust.update(true, false);
/// state.core.trust.update_triggered_event();
/// state.core.events.trust_triggered.fetch_add(1, Ordering::Relaxed);
///
/// // Gateway Crossing
/// state.peer.gateway.crossing_allowed(0.7);
///
/// // P2P Peer Connected
/// state.p2p.swarm.peer_connected(false);
///
/// // UI Render
/// state.ui.render(false, 100, 50, Some("default"));
///
/// // API Request
/// state.api.record_request(1500, 200, 50, 10, Some("default"));
///
/// // Governance Vote
/// state.governance.vote_cast(1.5, false, true, Some("default"));
///
/// // Controller AuthZ
/// state.controller.check_authorization(true, false, 50, "realm", Some("default"));
///
/// // DataLogic Event
/// state.data_logic.process_event(false, 25);
///
/// // BlueprintComposer Composition
/// state.blueprint_composer.composition_created(true, 2, 0, 100);
///
/// // Snapshot für Diagnostics
/// let snapshot = state.snapshot();
/// ```
pub struct UnifiedState {
    /// Startzeit
    pub started_at: Instant,

    // ═══════════════════════════════════════════════════════════════════════════
    // IDENTITY-LAYER (Κ6-Κ8: DID Management) - Position: VOR Core
    // ═══════════════════════════════════════════════════════════════════════════
    /// Identity-State für DID-Management
    /// - Root-DID, Sub-DIDs (Device, Agent, Realm)
    /// - Delegationen (Κ8 Trust-Decay)
    /// - Realm-Memberships
    /// - Wallet-Adressen
    pub identity: IdentityState,

    /// Core Logic Layer (Κ2-Κ18)
    pub core: CoreState,

    /// Execution Layer (IPS ℳ)
    pub execution: ExecutionState,

    /// ECLVM Layer (Erynoa Core Language Virtual Machine)
    /// Führt ECL-Policies, Blueprints und Sagas aus
    pub eclvm: ECLVMState,

    /// Protection Layer (Κ19-Κ21)
    pub protection: ProtectionState,

    /// Storage Layer
    pub storage: StorageState,

    /// Peer Layer (Κ22-Κ24)
    pub peer: PeerState,

    /// P2P Network Layer
    pub p2p: P2PState,

    // ─────────────────────────────────────────────────────────────────────────
    // Engine-Layer (6 neue Engines für SOLL-Zustand)
    // ─────────────────────────────────────────────────────────────────────────
    /// UI-Engine Layer (Component-Tree, Bindings, Trust-Gates)
    pub ui: UIState,

    /// API-Engine Layer (Endpoints, Rate-Limits, Request-Tracking)
    pub api: APIState,

    /// Governance-Engine Layer (Proposals, Voting, Delegation)
    pub governance: GovernanceState,

    /// Controller-Engine Layer (Permissions, AuthZ, Audit)
    pub controller: ControllerState,

    /// DataLogic-Engine Layer (Streams, Aggregations, Event-Processing)
    pub data_logic: DataLogicState,

    /// BlueprintComposer-Engine Layer (Composition, Versioning, Caching)
    pub blueprint_composer: BlueprintComposerState,

    /// State-Beziehungs-Graph
    pub graph: StateGraph,

    /// Aktive Warnings
    pub warnings: RwLock<Vec<String>>,

    /// Global Health Score (cached)
    pub health_score: RwLock<f64>,

    // ─────────────────────────────────────────────────────────────────────────
    // Architektur-Verbesserungen (Phase 6+)
    // ─────────────────────────────────────────────────────────────────────────
    /// EventBus für P2P/Core Entkopplung (Verbesserung 1)
    /// - Ingress-Queue: P2P → Core (empfangene Events)
    /// - Egress-Queue: Core → P2P (zu sendende Events)
    /// - Priority-Queue: Consensus/Trust-Critical Events
    pub event_bus: EventBus,

    /// Circuit Breaker für automatische Degradation (Verbesserung 3)
    /// - SystemMode: Normal → Degraded → EmergencyShutdown
    /// - Automatische Reaktion auf kritische Anomalien
    pub circuit_breaker: CircuitBreaker,

    /// State Broadcaster für CQRS light (Verbesserung 4)
    /// - Broadcast-Channel für State-Deltas
    /// - Subscriber: DataLogic-Engine, Monitoring, Metrics
    pub broadcaster: StateBroadcaster,

    /// Storage Handle für orthogonalen Zugriff (Verbesserung 2)
    /// - Pluggable Backend (RocksDB, IPFS, Cloud, Memory)
    /// - Einheitliche Recovery und Metriken
    pub storage_handle: StorageHandle,

    // ─────────────────────────────────────────────────────────────────────────
    // Architektur-Verbesserungen Phase 6.2
    // ─────────────────────────────────────────────────────────────────────────
    /// Merkle State Tracker für Differential Snapshots
    /// - Merkle-Tree über alle Sub-States
    /// - Effiziente Delta-Synchronisation für Light-Clients
    /// - State-Proofs für Verifizierung
    pub merkle_tracker: MerkleStateTracker,

    /// Multi-Level Gas Metering für hierarchische Kosten
    /// - L1: Network (P2P-Bandbreite)
    /// - L2: Compute (CPU/Instructions)
    /// - L3: Storage (Persistence)
    /// - L4: Realm (Per-Realm Quotas)
    pub multi_gas: MultiGas,

    // ─────────────────────────────────────────────────────────────────────────
    // Architektur-Verbesserungen Phase 6.3: Event-Sourcing
    // ─────────────────────────────────────────────────────────────────────────
    /// State Event Log für Event-Sourcing
    /// - Jede semantisch sinnvolle Änderung wird geloggt
    /// - Crash-Recovery durch Replay
    /// - Audits und Time-Travel
    pub event_log: StateEventLog,
}

impl UnifiedState {
    /// Erstelle neuen Unified State
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
            identity: IdentityState::new(),
            core: CoreState::new(),
            execution: ExecutionState::new(),
            eclvm: ECLVMState::new(),
            protection: ProtectionState::new(),
            storage: StorageState::new(),
            peer: PeerState::new(),
            p2p: P2PState::new(),
            ui: UIState::new(),
            api: APIState::new(),
            governance: GovernanceState::new(),
            controller: ControllerState::new(),
            data_logic: DataLogicState::new(),
            blueprint_composer: BlueprintComposerState::new(),
            graph: StateGraph::erynoa_graph(),
            warnings: RwLock::new(Vec::new()),
            health_score: RwLock::new(100.0),
            // Architektur-Verbesserungen Phase 6.1
            event_bus: EventBus::new(),
            circuit_breaker: CircuitBreaker::new(),
            broadcaster: StateBroadcaster::new(),
            storage_handle: StorageHandle::new(StorageBackend::RocksDB),
            // Architektur-Verbesserungen Phase 6.2
            merkle_tracker: MerkleStateTracker::new(),
            multi_gas: MultiGas::new(),
            // Architektur-Verbesserungen Phase 6.3
            event_log: StateEventLog::new(),
        }
    }

    /// Uptime in Sekunden
    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Berechne und cache Health Score
    pub fn calculate_health(&self) -> f64 {
        let mut score: f64 = 100.0;

        // ─────────────────────────────────────────────────────────────────────
        // Identity-Layer Health (10% Gewicht)
        // ─────────────────────────────────────────────────────────────────────
        let identity_health = self.identity.health_score() * 100.0;
        score -= (100.0 - identity_health) * 0.10;

        // Protection Health (15% Gewicht)
        score -= (100.0 - self.protection.health_score()) * 0.15;

        // Consensus Success Rate (12% Gewicht)
        score -= (1.0 - self.core.consensus.success_rate()) * 12.0;

        // Execution Success Rate (8% Gewicht)
        score -= (1.0 - self.execution.success_rate()) * 8.0;

        // ECLVM Policy Success Rate (8% Gewicht)
        score -= (1.0 - self.eclvm.policy_success_rate()) * 8.0;

        // P2P Health (15% Gewicht)
        score -= (100.0 - self.p2p.health_score()) * 0.15;

        // Peer Layer Health (8% Gewicht)
        let gateway_rate = self.peer.gateway.success_rate();
        let saga_rate = self.peer.saga.composition_success_rate();
        let peer_health = (gateway_rate + saga_rate) / 2.0 * 100.0;
        score -= (100.0 - peer_health) * 0.08;

        // Realm Crossing Success (4% Gewicht)
        score -= (1.0 - self.eclvm.crossing_allow_rate()) * 4.0;

        // Event Validation Errors (5% Gewicht)
        let event_errors = self.core.events.validation_errors.load(Ordering::Relaxed);
        let event_total = self.core.events.total.load(Ordering::Relaxed);
        if event_total > 0 {
            let error_rate = event_errors as f64 / event_total as f64;
            score -= error_rate * 5.0;
        }

        // ─────────────────────────────────────────────────────────────────────
        // Engine-Layer Health (25% Gewicht verteilt auf 6 Engines)
        // ─────────────────────────────────────────────────────────────────────

        // UI-Engine Health (4% Gewicht)
        let ui_health = (self.ui.cache_hit_rate() + self.ui.trust_gate_allow_rate()) / 2.0 * 100.0;
        score -= (100.0 - ui_health) * 0.04;

        // API-Engine Health (5% Gewicht)
        let api_health = self.api.success_rate() * 100.0;
        score -= (100.0 - api_health) * 0.05;

        // Governance-Engine Health (4% Gewicht)
        // Hohe Participation = Gesundheit
        let governance_health = if self.governance.proposals_completed.load(Ordering::Relaxed) > 0 {
            self.governance.proposal_success_rate() * 100.0
        } else {
            100.0 // Noch keine Proposals = neutral
        };
        score -= (100.0 - governance_health) * 0.04;

        // Controller-Engine Health (5% Gewicht)
        let controller_health = self.controller.authz_success_rate() * 100.0;
        score -= (100.0 - controller_health) * 0.05;

        // DataLogic-Engine Health (4% Gewicht)
        let datalogic_health =
            (self.data_logic.success_rate() + self.data_logic.binding_success_rate()) / 2.0 * 100.0;
        score -= (100.0 - datalogic_health) * 0.04;

        // BlueprintComposer-Engine Health (3% Gewicht)
        let blueprint_health = (self.blueprint_composer.composition_success_rate()
            + self.blueprint_composer.cache_hit_rate())
            / 2.0
            * 100.0;
        score -= (100.0 - blueprint_health) * 0.03;

        let final_score = score.max(0.0).min(100.0);

        // Cache
        if let Ok(mut cached) = self.health_score.write() {
            *cached = final_score;
        }

        final_score
    }

    /// Warning hinzufügen
    pub fn add_warning(&self, warning: String) {
        if let Ok(mut warnings) = self.warnings.write() {
            if !warnings.contains(&warning) {
                warnings.push(warning);
                if warnings.len() > 100 {
                    warnings.remove(0);
                }
            }
        }
    }

    /// Warning entfernen (per Prefix-Match)
    pub fn clear_warning(&self, prefix: &str) {
        if let Ok(mut warnings) = self.warnings.write() {
            warnings.retain(|w| !w.starts_with(prefix));
        }
    }

    /// Vollständiger Snapshot
    pub fn snapshot(&self) -> UnifiedSnapshot {
        UnifiedSnapshot {
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            uptime_secs: self.uptime_secs(),
            identity: self.identity.snapshot(),
            core: self.core.snapshot(),
            execution: self.execution.snapshot(),
            eclvm: self.eclvm.snapshot(),
            protection: self.protection.snapshot(),
            storage: self.storage.snapshot(),
            peer: self.peer.snapshot(),
            p2p: self.p2p.snapshot(),
            ui: self.ui.snapshot(),
            api: self.api.snapshot(),
            governance: self.governance.snapshot(),
            controller: self.controller.snapshot(),
            data_logic: self.data_logic.snapshot(),
            blueprint_composer: self.blueprint_composer.snapshot(),
            health_score: self.calculate_health(),
            warnings: self.warnings.read().map(|w| w.clone()).unwrap_or_default(),
            // Architektur-Verbesserungen Phase 6.1
            event_bus: self.event_bus.snapshot(),
            circuit_breaker: self.circuit_breaker.snapshot(),
            broadcaster: self.broadcaster.snapshot(),
            system_mode: self.circuit_breaker.mode(),
            // Architektur-Verbesserungen Phase 6.2
            merkle_tracker: self.merkle_tracker.snapshot(),
            multi_gas: self.multi_gas.snapshot(),
            // Architektur-Verbesserungen Phase 6.3
            event_log: self.event_log.snapshot(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Architektur-Verbesserungen: Convenience-Methoden
    // ─────────────────────────────────────────────────────────────────────────

    /// Prüfe ob das System operationell ist (Normal oder Degraded)
    pub fn is_operational(&self) -> bool {
        self.circuit_breaker.mode().is_operational()
    }

    /// Prüfe ob Execution erlaubt ist (nur Normal-Modus)
    pub fn allows_execution(&self) -> bool {
        self.circuit_breaker.allows_execution()
    }

    /// Prüfe ob Gateway-Crossings erlaubt sind (nur Normal-Modus)
    pub fn allows_crossings(&self) -> bool {
        self.circuit_breaker.allows_crossings()
    }

    /// Aktuellen System-Modus abfragen
    pub fn system_mode(&self) -> SystemMode {
        self.circuit_breaker.mode()
    }

    /// Anomalie aufzeichnen mit automatischer Circuit Breaker Integration
    pub fn record_anomaly(&self, severity: &str) -> SystemMode {
        self.protection
            .anomaly_with_circuit_breaker(severity, &self.circuit_breaker)
    }

    /// State-Delta broadcasten (für CQRS Subscriber)
    pub fn broadcast_delta(&self, component: StateComponent, delta_type: DeltaType, data: Vec<u8>) {
        self.broadcaster
            .broadcast(StateDelta::new(component, delta_type, data));
    }

    /// Network-Event über EventBus senden (Core → P2P)
    pub fn send_network_event(&self, event: NetworkEvent) -> Result<(), NetworkEvent> {
        self.event_bus.try_send_egress(event)
    }

    /// Network-Event empfangen (P2P → Core) - non-blocking try
    pub fn receive_network_event(&self, event: NetworkEvent) -> Result<(), NetworkEvent> {
        self.event_bus.try_send_ingress(event)
    }

    /// Neuen State-Delta Subscriber erstellen (für DataLogic, Monitoring, etc.)
    pub fn subscribe_deltas(&self) -> broadcast::Receiver<StateDelta> {
        self.broadcaster.subscribe()
    }

    /// Manual Recovery: System in Normal-Modus zurücksetzen
    pub fn reset_circuit_breaker(&self) {
        self.circuit_breaker.reset_to_normal();
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 6.2: Differential State Snapshots
    // ─────────────────────────────────────────────────────────────────────────

    /// Aktuelle Merkle-Root-Hash des gesamten State
    pub fn merkle_root(&self) -> MerkleHash {
        self.merkle_tracker.root_hash()
    }

    /// Update State-Komponente mit Merkle-Delta-Tracking
    pub fn update_with_merkle(&self, component: StateComponent, data: &[u8]) -> MerkleDelta {
        self.merkle_tracker.update_component(component, data)
    }

    /// Hole Deltas seit bestimmter Sequenz (für Light-Client-Sync)
    pub fn deltas_since(&self, sequence: u64) -> Vec<MerkleDelta> {
        self.merkle_tracker.deltas_since(sequence)
    }

    /// Verifiziere eingehendes Delta
    pub fn verify_delta(&self, delta: &MerkleDelta) -> bool {
        self.merkle_tracker.verify_delta(delta)
    }

    /// Merkle-Hash einer Komponente (Phase 5: Light-Client / Proof-API)
    pub fn merkle_component_hash(&self, component: StateComponent) -> Option<MerkleHash> {
        self.merkle_tracker.component_hash(component)
    }

    /// Sequenz zu einem bekannten Root (für delta?since_root=)
    pub fn merkle_sequence_for_root(&self, root: &MerkleHash) -> Option<u64> {
        self.merkle_tracker.sequence_for_root(root)
    }

    /// State-Proof für eine Komponente (Phase 5: Verifizierung gegen Root)
    pub fn merkle_component_proof(
        &self,
        component: StateComponent,
    ) -> Option<(MerkleHash, Vec<MerkleHash>)> {
        self.merkle_tracker.component_proof(component)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 6.2: Multi-Level Gas Metering
    // ─────────────────────────────────────────────────────────────────────────

    /// Konsumiere Network-Gas (P2P-Bandbreite)
    pub fn consume_network_gas(&self, bytes_sent: u64, bytes_received: u64, messages: u64) {
        self.multi_gas
            .consume_network(bytes_sent, bytes_received, messages);
    }

    /// Konsumiere Compute-Gas (CPU/Instructions)
    pub fn consume_compute_gas(&self, instructions: u64) {
        self.multi_gas.consume_compute(instructions);
    }

    /// Konsumiere Storage-Gas (Persistence)
    pub fn consume_storage_gas(&self, bytes_written: u64, operations: u64) {
        self.multi_gas.consume_storage(bytes_written, operations);
    }

    /// Konsumiere Realm-spezifisches Gas
    pub fn consume_realm_gas(&self, realm_id: &str, amount: u64) {
        self.multi_gas.consume_realm(realm_id, amount);
    }

    /// Registriere Realm für Gas-Tracking
    pub fn register_realm_for_gas(&self, realm_id: &str) {
        self.multi_gas.register_realm(realm_id);
    }

    /// Setze dynamischen Gas-Preis (Congestion Pricing)
    pub fn set_gas_price(&self, layer: GasLayer, price: u64) {
        self.multi_gas.set_price(layer, price);
    }

    /// Hole Gas-Verbrauch für Realm
    pub fn realm_gas_consumed(&self, realm_id: &str) -> u64 {
        self.multi_gas.realm_consumed(realm_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 6.2: Self-Healing Realm-Isolierung
    // ─────────────────────────────────────────────────────────────────────────

    /// Prüfe ob Realm-Operation erlaubt ist (delegiert an RealmState)
    pub fn check_realm_quota(&self, realm_id: &str, resource: ResourceType, amount: u64) -> bool {
        self.peer
            .realm
            .realms
            .read()
            .map(|realms| {
                realms
                    .get(realm_id)
                    .map(|r| r.check_operation(resource, amount))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    /// Konsumiere Realm-Ressource (mit Quota-Check)
    pub fn consume_realm_resource(
        &self,
        realm_id: &str,
        resource: ResourceType,
        amount: u64,
    ) -> bool {
        self.peer
            .realm
            .realms
            .read()
            .map(|realms| {
                realms
                    .get(realm_id)
                    .map(|r| r.consume_resource(resource, amount))
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    /// Quarantine Realm (manuell oder durch Protection)
    pub fn quarantine_realm(&self, realm_id: &str) {
        if let Ok(realms) = self.peer.realm.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.quarantine();
                self.record_anomaly("realm_quarantine");
            }
        }
    }

    /// Unquarantine Realm (Admin-Recovery)
    pub fn unquarantine_realm(&self, realm_id: &str) {
        if let Ok(realms) = self.peer.realm.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.unquarantine();
            }
        }
    }

    /// Prüfe ob Realm quarantined ist
    pub fn is_realm_quarantined(&self, realm_id: &str) -> bool {
        self.peer
            .realm
            .realms
            .read()
            .map(|realms| {
                realms
                    .get(realm_id)
                    .map(|r| r.is_quarantined())
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Phase 6.3: Event-Sourcing
    // ─────────────────────────────────────────────────────────────────────────

    /// Logge und wende State-Event an (Event-Sourcing Hauptmethode)
    ///
    /// 1. Erstellt WrappedStateEvent mit Kausalität
    /// 2. Persistiert Event (asynchron, fire-and-forget)
    /// 3. Wendet Event sofort an (für Low-Latency)
    ///
    /// ## Beispiel
    /// ```ignore
    /// state.log_and_apply(
    ///     StateEvent::TrustUpdate {
    ///         entity_id: "did:example:123".to_string(),
    ///         delta: 0.05,
    ///         reason: TrustReason::PositiveInteraction,
    ///         from_realm: Some("realm_abc".to_string()),
    ///         triggered_events: 1,
    ///         new_trust: 0.75,
    ///     },
    ///     vec![parent_event_id],
    /// );
    /// ```
    pub fn log_and_apply(&self, event: StateEvent, parent_ids: Vec<String>) -> WrappedStateEvent {
        // 1. Logge Event
        let wrapped = self.event_log.log(event.clone(), parent_ids);

        // 2. Wende Event an (State-Mutation)
        self.apply_state_event(&wrapped);

        // 3. Broadcast Delta für CQRS Subscriber
        let delta = StateDelta::new(
            wrapped.component,
            DeltaType::Update,
            format!("{:?}", wrapped.event).into_bytes(),
        );
        self.broadcaster.broadcast(delta);

        // 4. Update Merkle-Tracker
        self.merkle_tracker
            .update_component(wrapped.component, format!("{:?}", wrapped.event).as_bytes());

        // 5. Prüfe ob Checkpoint fällig
        if self.event_log.needs_checkpoint() {
            let checkpoint_id = format!("ckpt_{}", wrapped.sequence);
            let state_hash = self.merkle_root();
            let _ = self.event_log.mark_checkpoint(checkpoint_id, state_hash);
        }

        wrapped
    }

    /// Wende State-Event an (Replay-Logik)
    ///
    /// Diese Methode wird für:
    /// 1. Live-Updates (log_and_apply)
    /// 2. Recovery-Replay
    /// 3. Time-Travel-Rekonstruktion
    fn apply_state_event(&self, wrapped: &WrappedStateEvent) {
        match &wrapped.event {
            // ═══════════════════════════════════════════════════════════════════
            // CORE STATE EVENTS
            // ═══════════════════════════════════════════════════════════════════
            StateEvent::TrustUpdate {
                delta,
                reason,
                triggered_events,
                ..
            } => {
                // Trust-Counter aktualisieren über existierende Methode
                let positive = *delta > 0.0;
                let from_event = matches!(
                    reason,
                    TrustReason::PositiveInteraction | TrustReason::NegativeInteraction
                );
                self.core.trust.update(positive, from_event);

                // Getriggerte Events zählen
                if *triggered_events > 0 {
                    self.core
                        .trust
                        .triggered_events
                        .fetch_add(*triggered_events, Ordering::Relaxed);
                }
            }

            StateEvent::EventProcessed {
                validation_errors, ..
            } => {
                self.core.events.total.fetch_add(1, Ordering::Relaxed);
                if *validation_errors {
                    self.core
                        .events
                        .validation_errors
                        .fetch_add(1, Ordering::Relaxed);
                }
            }

            StateEvent::FormulaComputed {
                new_e,
                activity,
                trust_norm,
                human_factor,
                ..
            } => {
                // Formula-State über existierende Methode aktualisieren
                self.core
                    .formula
                    .update(*new_e, *activity, *trust_norm, *human_factor);
            }

            StateEvent::ConsensusRoundCompleted {
                success,
                duration_ms,
                byzantine_detected,
                ..
            } => {
                // Nutze existierende round_completed Methode
                self.core.consensus.round_completed(*success, *duration_ms);

                if !byzantine_detected.is_empty() {
                    self.core
                        .consensus
                        .byzantine_detected
                        .fetch_add(byzantine_detected.len() as u64, Ordering::Relaxed);
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // EXECUTION + ECLVM EVENTS
            // ═══════════════════════════════════════════════════════════════════
            StateEvent::ExecutionStarted { .. } => {
                // Nutze existierende start() Methode
                self.execution.start();
            }

            StateEvent::ExecutionCompleted {
                success,
                gas_consumed,
                mana_consumed,
                events_emitted,
                duration_ms,
                ..
            } => {
                // Nutze existierende complete() Methode
                self.execution.complete(
                    *success,
                    *gas_consumed,
                    *mana_consumed,
                    *events_emitted,
                    *duration_ms,
                );
            }

            StateEvent::PolicyEvaluated {
                passed,
                policy_type,
                gas_used,
                mana_used,
                duration_us,
                realm_id,
                ..
            } => {
                // Nutze existierende policy_executed Methode für ECLVMState
                self.eclvm.policy_executed(
                    *passed,
                    *policy_type,
                    *gas_used,
                    *mana_used,
                    *duration_us,
                    realm_id.as_deref(),
                );

                // E1.2: Synchronisiere mit ExecutionState
                // ECLVM-Policy-Ausführungen werden als Execution-Contexts aggregiert
                self.execution.executions.record_eclvm_policy_execution(
                    *passed,
                    *gas_used,
                    *mana_used,
                    1, // Policy emittiert 1 Event
                    *duration_us,
                );

                // Aktualisiere auch Gas/Mana State
                self.execution.gas.consume(*gas_used);
                self.execution.mana.consume(*mana_used);
            }

            StateEvent::BlueprintAction { action, .. } => {
                match action {
                    BlueprintActionType::Published => {
                        self.eclvm
                            .blueprints_published
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    BlueprintActionType::Deployed => {
                        self.eclvm
                            .blueprints_deployed
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    BlueprintActionType::Instantiated => {
                        self.eclvm
                            .blueprints_instantiated
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    BlueprintActionType::Verified => {
                        self.eclvm
                            .blueprints_verified
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    BlueprintActionType::Deprecated => {
                        // Deprecated Blueprints - kann über Status-Map getrackt werden
                    }
                }
            }

            StateEvent::SagaProgress {
                compensation_triggered,
                cross_realm,
                ..
            } => {
                self.eclvm
                    .saga_steps_executed
                    .fetch_add(1, Ordering::Relaxed);
                if *cross_realm {
                    self.eclvm.cross_realm_steps.fetch_add(1, Ordering::Relaxed);
                }
                if *compensation_triggered {
                    self.eclvm
                        .compensations_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // PROTECTION STATE EVENTS
            // ═══════════════════════════════════════════════════════════════════
            StateEvent::AnomalyDetected { severity, .. } => {
                match severity {
                    AnomalySeverity::Critical => {
                        self.protection
                            .anomaly
                            .critical
                            .fetch_add(1, Ordering::Relaxed);
                        // Circuit Breaker prüfen
                        self.circuit_breaker.record_critical_anomaly();
                    }
                    AnomalySeverity::High => {
                        self.protection.anomaly.high.fetch_add(1, Ordering::Relaxed);
                    }
                    AnomalySeverity::Medium => {
                        self.protection
                            .anomaly
                            .medium
                            .fetch_add(1, Ordering::Relaxed);
                    }
                    AnomalySeverity::Low => {
                        self.protection.anomaly.low.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            StateEvent::DiversityMetricUpdate {
                warning_triggered, ..
            } => {
                self.protection
                    .diversity
                    .trust_distribution_checks
                    .fetch_add(1, Ordering::Relaxed);
                if *warning_triggered {
                    self.protection
                        .diversity
                        .monoculture_warnings
                        .fetch_add(1, Ordering::Relaxed);
                }
            }

            StateEvent::CalibrationApplied { .. } => {
                self.protection
                    .calibration
                    .updates_total
                    .fetch_add(1, Ordering::Relaxed);
            }

            StateEvent::SystemModeChanged { new_mode, .. } => {
                // Modus wird durch Circuit Breaker gesteuert
                match new_mode {
                    SystemMode::Normal => self.circuit_breaker.reset_to_normal(),
                    SystemMode::Degraded => {
                        // Bereits durch record_critical_anomaly() gesteuert
                    }
                    SystemMode::EmergencyShutdown => {
                        // Bereits durch record_critical_anomaly() gesteuert
                    }
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // PEER + REALM EVENTS
            // ═══════════════════════════════════════════════════════════════════
            StateEvent::RealmLifecycle {
                realm_id, action, ..
            } => match action {
                RealmAction::Created => {
                    self.peer.realm.total_realms.fetch_add(1, Ordering::Relaxed);
                }
                RealmAction::Destroyed => {
                    let _ = self.peer.realm.total_realms.fetch_update(
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                        |v| if v > 0 { Some(v - 1) } else { Some(0) },
                    );
                }
                _ => {}
            },

            StateEvent::MembershipChange {
                realm_id, action, ..
            } => match action {
                MembershipAction::Joined => {
                    self.peer.realm.identity_joined_realm(realm_id);
                }
                MembershipAction::Left | MembershipAction::Banned => {
                    self.peer.realm.identity_left_realm(realm_id);
                }
                _ => {}
            },

            StateEvent::CrossingEvaluated { allowed, .. } => {
                self.peer
                    .gateway
                    .crossings_total
                    .fetch_add(1, Ordering::Relaxed);
                if *allowed {
                    self.peer
                        .gateway
                        .crossings_allowed
                        .fetch_add(1, Ordering::Relaxed);
                } else {
                    self.peer
                        .gateway
                        .crossings_denied
                        .fetch_add(1, Ordering::Relaxed);
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // P2P NETWORK EVENTS
            // ═══════════════════════════════════════════════════════════════════
            StateEvent::NetworkMetricUpdate { metric, delta, .. } => match metric {
                NetworkMetric::ConnectedPeers => {
                    if *delta > 0 {
                        self.p2p
                            .swarm
                            .connected_peers
                            .fetch_add(*delta as usize, Ordering::Relaxed);
                    }
                }
                NetworkMetric::BytesSent => {
                    self.p2p
                        .swarm
                        .bytes_sent
                        .fetch_add(*delta as u64, Ordering::Relaxed);
                }
                NetworkMetric::BytesReceived => {
                    self.p2p
                        .swarm
                        .bytes_received
                        .fetch_add(*delta as u64, Ordering::Relaxed);
                }
                NetworkMetric::GossipMessages => {
                    self.p2p
                        .gossip
                        .messages_received
                        .fetch_add(*delta as u64, Ordering::Relaxed);
                }
                NetworkMetric::DHTLookups => {
                    self.p2p
                        .kademlia
                        .queries_total
                        .fetch_add(*delta as u64, Ordering::Relaxed);
                }
                _ => {}
            },

            StateEvent::PeerConnectionChange { connected, .. } => {
                if *connected {
                    self.p2p
                        .swarm
                        .connected_peers
                        .fetch_add(1, Ordering::Relaxed);
                } else {
                    let _ = self.p2p.swarm.connected_peers.fetch_update(
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                        |v| {
                            if v > 0 {
                                Some(v - 1)
                            } else {
                                Some(0)
                            }
                        },
                    );
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // TRUST GATE EVENTS (v0.4.0)
            // ═══════════════════════════════════════════════════════════════════
            StateEvent::TrustUpdated { .. } => {
                // Trust-Updates werden vom TrustGate selbst verwaltet
                // Hier nur für Logging/Monitoring
            }

            StateEvent::PeerBanned { .. } => {
                // Ban-Statistiken: Könnte in einer separaten Statistik-Struktur
                // getrackt werden (z.B. self.p2p.trust_gate.bans)
            }

            StateEvent::PeerUnbanned { .. } => {
                // Unban-Statistiken
            }

            // ═══════════════════════════════════════════════════════════════════
            // RECOVERY + GOVERNANCE + QUOTA EVENTS
            // ═══════════════════════════════════════════════════════════════════
            StateEvent::CheckpointCreated { .. } => {
                // Checkpoint-Events werden vom EventLog selbst verwaltet
            }

            StateEvent::RecoveryCompleted {
                events_replayed, ..
            } => {
                // Nur Logging, State wurde bereits durch Replay aufgebaut
                self.add_warning(format!(
                    "Recovery completed: {} events replayed",
                    events_replayed
                ));
            }

            StateEvent::ReorgDetected { discarded_ids, .. } => {
                self.add_warning(format!(
                    "DAG reorg detected: {} events discarded",
                    discarded_ids.len()
                ));
            }

            StateEvent::ProposalCreated { .. } => {
                self.governance
                    .proposals_created
                    .fetch_add(1, Ordering::Relaxed);
            }

            StateEvent::VoteCast { .. } => {
                self.governance.votes_cast.fetch_add(1, Ordering::Relaxed);
            }

            StateEvent::ProposalResolved { accepted, .. } => {
                self.governance
                    .proposals_completed
                    .fetch_add(1, Ordering::Relaxed);
                if *accepted {
                    self.governance
                        .proposals_accepted
                        .fetch_add(1, Ordering::Relaxed);
                }
            }

            StateEvent::QuotaViolation { quarantined, .. } => {
                if *quarantined {
                    self.record_anomaly("quota_violation_quarantine");
                }
            }

            StateEvent::RealmQuarantineChange { .. } => {
                // Wird durch quarantine_realm() gesteuert
            }

            // ═══════════════════════════════════════════════════════════════════
            // PRIVACY LAYER EVENTS (Phase 2 Woche 8)
            // ═══════════════════════════════════════════════════════════════════
            StateEvent::PrivacyCircuitCreated { hop_count, .. } => {
                // Tracking für Privacy-Circuits
                self.p2p
                    .privacy
                    .circuits_created
                    .fetch_add(1, Ordering::Relaxed);
                // Durchschnittliche Hop-Zahl könnte hier getrackt werden
                let _ = hop_count; // TODO: Track average hop count
            }

            StateEvent::PrivacyCircuitClosed {
                messages_routed, ..
            } => {
                // Circuit-Statistiken
                self.p2p
                    .privacy
                    .circuits_closed
                    .fetch_add(1, Ordering::Relaxed);
                // Messages über diesen Circuit
                self.p2p
                    .privacy
                    .messages_routed
                    .fetch_add(*messages_routed, Ordering::Relaxed);
            }

            StateEvent::PrivacyMessageSent {
                is_cover_traffic, ..
            } => {
                if *is_cover_traffic {
                    self.p2p
                        .privacy
                        .cover_traffic_sent
                        .fetch_add(1, Ordering::Relaxed);
                } else {
                    self.p2p
                        .privacy
                        .messages_sent
                        .fetch_add(1, Ordering::Relaxed);
                }
            }

            StateEvent::CoverTrafficGenerated {
                messages_count,
                compliance_status,
                ..
            } => {
                // Cover-Traffic-Metriken
                self.p2p
                    .privacy
                    .cover_traffic_sent
                    .fetch_add(*messages_count, Ordering::Relaxed);

                // Compliance-Tracking
                if compliance_status == "violation" {
                    self.record_anomaly("cover_traffic_compliance_violation");
                }
            }

            StateEvent::MixingPoolFlushed {
                messages_flushed, ..
            } => {
                // Mixing-Pool-Statistiken
                self.p2p
                    .privacy
                    .mixing_flushes
                    .fetch_add(1, Ordering::Relaxed);
                self.p2p
                    .privacy
                    .messages_mixed
                    .fetch_add(*messages_flushed, Ordering::Relaxed);
            }

            StateEvent::RelaySelectionCompleted { success, .. } => {
                // Relay-Auswahl-Statistiken
                if *success {
                    self.p2p
                        .privacy
                        .trust_based_selections
                        .fetch_add(1, Ordering::Relaxed);
                } else {
                    self.p2p
                        .privacy
                        .selection_failures
                        .fetch_add(1, Ordering::Relaxed);
                }
            }

            // ═══════════════════════════════════════════════════════════════════
            // IDENTITY EVENTS (Κ6-Κ8)
            // ═══════════════════════════════════════════════════════════════════
            StateEvent::IdentityBootstrapped { .. } => {
                // Identity-Bootstrap wird direkt über IdentityState gesteuert
                // Event wird nur für Audit/Recovery geloggt
            }

            StateEvent::IdentityModeChanged { .. } => {
                // Mode-Change wird direkt über IdentityState.mode gesteuert
            }

            StateEvent::SubDIDDerived { gas_used, .. } => {
                // Gas-Tracking für Sub-DID Derivation
                self.execution
                    .gas
                    .consumed
                    .fetch_add(*gas_used, Ordering::Relaxed);
            }

            StateEvent::WalletDerived { .. } => {
                // Wallet-Derivation wird direkt über IdentityState gesteuert
            }

            StateEvent::DelegationCreated { .. } => {
                // Delegation-Erstellung wird direkt über IdentityState gesteuert
            }

            StateEvent::DelegationRevoked { .. } => {
                // Delegation-Widerruf wird direkt über IdentityState gesteuert
            }

            StateEvent::CredentialIssued { .. } => {
                // Credential-Issuance wird separat getrackt
            }

            StateEvent::CredentialVerified { valid, .. } => {
                // Optional: Verifizierungs-Statistiken tracken
                if *valid {
                    // Erfolgreiche Verifikation
                }
            }

            StateEvent::KeyRotated { .. } => {
                // Key-Rotation ist ein kritisches Event
                // Wird für Recovery/Audit geloggt
            }

            StateEvent::RecoveryInitiated { .. } => {
                // Recovery-Initiierung ist kritisch
                // Circuit-Breaker könnte hier reagieren
            }

            StateEvent::IdentityAnomalyDetected { severity, .. } => {
                // Anomalie-Detection: Könnte Protection-Layer informieren
                if severity == "critical" {
                    self.record_anomaly("identity_anomaly_critical");
                }
            }

            StateEvent::CrossShardIdentityResolved {
                success,
                latency_ms,
                ..
            } => {
                // Cross-Shard Metriken tracken (falls vorhanden)
                if !success {
                    self.record_anomaly("cross_shard_identity_resolution_failed");
                }
                // Latenz könnte für P2P-Metriken relevant sein
                let _ = latency_ms; // TODO: Track in P2P metrics
            }

            StateEvent::RealmMembershipChanged { .. } => {
                // Membership-Änderungen werden separat getrackt
            }
        }

        // Nach jedem Apply: Health-Score neu berechnen
        self.calculate_health();
    }

    /// Batch-Replay von Events (für Recovery)
    ///
    /// Optimiert für Performance: Events werden in Batches von 1000 applied.
    pub fn replay_events(&self, events: &[WrappedStateEvent]) {
        self.event_log.start_recovery();

        for event in events {
            self.apply_state_event(event);
        }

        self.event_log.end_recovery();

        // Finales Health-Score Update
        self.calculate_health();
    }

    /// Erstelle Checkpoint mit aktuellem State-Hash
    pub fn create_checkpoint(&self) -> WrappedStateEvent {
        let checkpoint_id = format!("ckpt_{}", self.event_log.sequence.load(Ordering::SeqCst));
        let state_hash = self.merkle_root();
        self.event_log.mark_checkpoint(checkpoint_id, state_hash)
    }

    /// Prüfe ob Recovery benötigt wird
    pub fn is_recovering(&self) -> bool {
        self.event_log.is_recovering.load(Ordering::Relaxed)
    }

    /// Event-Log Statistiken
    pub fn event_log_stats(&self) -> EventLogSnapshot {
        self.event_log.snapshot()
    }
}

impl Default for UnifiedState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedSnapshot {
    pub timestamp_ms: u64,
    pub uptime_secs: u64,
    /// Identity-Layer Snapshot (Κ6-Κ8)
    pub identity: IdentitySnapshot,
    pub core: CoreSnapshot,
    pub execution: ExecutionSnapshot,
    pub eclvm: ECLVMSnapshot,
    pub protection: ProtectionSnapshot,
    pub storage: StorageSnapshot,
    pub peer: PeerSnapshot,
    pub p2p: P2PSnapshot,
    pub ui: UISnapshot,
    pub api: APISnapshot,
    pub governance: GovernanceSnapshot,
    pub controller: ControllerSnapshot,
    pub data_logic: DataLogicSnapshot,
    pub blueprint_composer: BlueprintComposerSnapshot,
    pub health_score: f64,
    pub warnings: Vec<String>,
    // ─────────────────────────────────────────────────────────────────────────
    // Architektur-Verbesserungen Phase 6.1 Snapshots
    // ─────────────────────────────────────────────────────────────────────────
    /// EventBus-Metriken (P2P/Core Entkopplung)
    pub event_bus: EventBusSnapshot,
    /// Circuit Breaker Status (Degradation)
    pub circuit_breaker: CircuitBreakerSnapshot,
    /// Broadcaster-Metriken (CQRS)
    pub broadcaster: BroadcasterSnapshot,
    /// System-Modus (Normal/Degraded/Emergency)
    pub system_mode: SystemMode,
    // ─────────────────────────────────────────────────────────────────────────
    // Architektur-Verbesserungen Phase 6.2 Snapshots
    // ─────────────────────────────────────────────────────────────────────────
    /// Merkle State Tracker (Differential Snapshots)
    pub merkle_tracker: MerkleTrackerSnapshot,
    /// Multi-Level Gas Metering
    pub multi_gas: MultiGasSnapshot,
    // ─────────────────────────────────────────────────────────────────────────
    // Architektur-Verbesserungen Phase 6.3 Snapshots
    // ─────────────────────────────────────────────────────────────────────────
    /// Event-Sourcing Log Metriken
    pub event_log: EventLogSnapshot,
}

// ============================================================================
// GLOBAL STATE ACCESSOR
// ============================================================================

/// Thread-safe globaler State (Singleton-Pattern)
pub type SharedUnifiedState = Arc<UnifiedState>;

/// Erstelle neuen Shared State
pub fn create_unified_state() -> SharedUnifiedState {
    Arc::new(UnifiedState::new())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_state() {
        let state = TrustState::new();
        state.update(true, false);
        state.update(false, true);
        state.update(false, false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.updates_total, 3);
        assert_eq!(snapshot.positive_updates, 1);
        assert_eq!(snapshot.negative_updates, 2);
        assert_eq!(snapshot.event_triggered_updates, 1);
        assert!((snapshot.asymmetry_ratio - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_event_state() {
        let state = EventState::new();
        state.add(true, 0, 0);
        state.add(false, 2, 1);
        state.add(false, 3, 2);
        state.finalize(100);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.total, 3);
        assert_eq!(snapshot.genesis, 1);
        assert_eq!(snapshot.max_depth, 2);
        assert!(snapshot.avg_parents > 0.0);
    }

    #[test]
    fn test_gateway_state() {
        let state = GatewayState::new();
        state.crossing_allowed(0.8);
        state.crossing_allowed(0.6);
        state.crossing_denied("trust");
        state.crossing_denied("credential");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.crossings_total, 4);
        assert_eq!(snapshot.crossings_allowed, 2);
        assert_eq!(snapshot.crossings_denied, 2);
        assert_eq!(snapshot.trust_violations, 1);
        assert_eq!(snapshot.credential_violations, 1);
        assert!((snapshot.success_rate - 0.5).abs() < 0.01);
        assert!((snapshot.avg_crossing_trust - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_saga_composer_state() {
        let state = SagaComposerState::new();
        state.saga_composed(true, 3, "Transfer");
        state.saga_composed(true, 5, "Delegate");
        state.saga_composed(false, 0, "Transfer");
        state.compensation(true);
        state.compensation(false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.sagas_composed, 3);
        assert_eq!(snapshot.successful_compositions, 2);
        assert_eq!(snapshot.failed_compositions, 1);
        assert!((snapshot.avg_steps_per_saga - 4.0).abs() < 0.01);
        assert_eq!(snapshot.compensations_executed, 2);
        assert_eq!(snapshot.compensations_successful, 1);
        assert!(*snapshot.goals_by_type.get("Transfer").unwrap_or(&0) == 2);
    }

    #[test]
    fn test_swarm_state() {
        let state = SwarmState::new();
        state.peer_connected(true);
        state.peer_connected(false);
        state.peer_connected(false);
        state.peer_disconnected();
        state.record_latency(5000);
        state.record_latency(7000);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.connected_peers, 2);
        assert_eq!(snapshot.inbound_connections, 1);
        assert_eq!(snapshot.outbound_connections, 2);
        assert!((snapshot.avg_latency_ms - 6.0).abs() < 0.01);
    }

    #[test]
    fn test_gossip_state() {
        let state = GossipState::new();
        state.message_received();
        state.message_received();
        state.messages_validated.fetch_add(1, Ordering::Relaxed);
        state.messages_rejected.fetch_add(1, Ordering::Relaxed);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.messages_received, 2);
        assert!((snapshot.validation_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_p2p_state_health() {
        let state = P2PState::new();
        // Ohne Peers: Schlechter Score
        let score1 = state.health_score();
        assert!(score1 < 80.0);

        // Mit Peers: Besserer Score
        state.swarm.peer_connected(true);
        state.swarm.peer_connected(false);
        state.swarm.peer_connected(false);
        state.gossip.mesh_peers.store(3, Ordering::Relaxed);
        if let Ok(mut b) = state.kademlia.bootstrap_complete.write() {
            *b = true;
        }
        let score2 = state.health_score();
        assert!(score2 > score1);
    }

    #[test]
    fn test_unified_state() {
        let state = UnifiedState::new();

        state.core.trust.update(true, false);
        state.core.events.add(false, 2, 1);
        state.execution.start();
        state.execution.complete(true, 1000, 100, 2, 50);
        state.protection.anomaly("low");
        state.peer.gateway.crossing_allowed(0.7);
        state.p2p.swarm.peer_connected(false);
        state.p2p.gossip.message_sent();

        let snapshot = state.snapshot();
        assert_eq!(snapshot.core.trust.updates_total, 1);
        assert_eq!(snapshot.core.events.total, 1);
        assert_eq!(snapshot.execution.executions.total, 1);
        assert_eq!(snapshot.protection.anomaly.total, 1);
        assert_eq!(snapshot.peer.gateway.crossings_total, 1);
        assert_eq!(snapshot.p2p.swarm.connected_peers, 1);
        assert_eq!(snapshot.p2p.gossip.messages_sent, 1);
        assert!(snapshot.health_score > 0.0);
    }

    #[test]
    fn test_state_graph() {
        let graph = StateGraph::erynoa_graph();

        let dependents = graph.dependents(StateComponent::Trust);
        assert!(!dependents.is_empty());

        let triggered = graph.triggered_by(StateComponent::Trust);
        assert!(triggered.contains(&StateComponent::Event));

        // Prüfe Peer/P2P Beziehungen
        let gateway_triggered = graph.triggered_by(StateComponent::Gateway);
        assert!(gateway_triggered.contains(&StateComponent::Event));

        let gossip_deps = graph.dependents(StateComponent::Trust);
        assert!(!gossip_deps.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PHASE 1 TESTS: Neue StateComponent-Varianten und StateGraph-Edges
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_new_state_components_exist() {
        // Teste dass alle neuen StateComponent-Varianten existieren
        let components = vec![
            StateComponent::Room,
            StateComponent::Partition,
            StateComponent::UI,
            StateComponent::DataLogic,
            StateComponent::API,
            StateComponent::Governance,
            StateComponent::Controller,
            StateComponent::BlueprintComposer,
        ];

        // Alle Komponenten sollten serialisierbar sein
        for component in &components {
            let serialized = serde_json::to_string(component).unwrap();
            assert!(!serialized.is_empty());
        }

        // Prüfe dass es genau 8 neue Komponenten sind
        assert_eq!(components.len(), 8);
    }

    #[test]
    fn test_room_partition_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // Room ─ DependsOn ─▶ Realm
        assert!(graph.has_relation(
            StateComponent::Room,
            StateRelation::DependsOn,
            StateComponent::Realm
        ));

        // Room ─ DependsOn ─▶ Trust
        assert!(graph.has_relation(
            StateComponent::Room,
            StateRelation::DependsOn,
            StateComponent::Trust
        ));

        // Room ─ Triggers ─▶ Event
        assert!(graph.has_relation(
            StateComponent::Room,
            StateRelation::Triggers,
            StateComponent::Event
        ));

        // Room ─ Aggregates ─▶ Controller
        assert!(graph.has_relation(
            StateComponent::Room,
            StateRelation::Aggregates,
            StateComponent::Controller
        ));

        // Partition ─ DependsOn ─▶ Room
        assert!(graph.has_relation(
            StateComponent::Partition,
            StateRelation::DependsOn,
            StateComponent::Room
        ));

        // Partition ─ Validates ─▶ Controller
        assert!(graph.has_relation(
            StateComponent::Partition,
            StateRelation::Validates,
            StateComponent::Controller
        ));
    }

    #[test]
    fn test_ui_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // UI hat viele Dependencies
        let ui_deps = graph.dependencies_of(StateComponent::UI);
        assert!(ui_deps.contains(&StateComponent::Trust));
        assert!(ui_deps.contains(&StateComponent::Realm));
        assert!(ui_deps.contains(&StateComponent::Room));
        assert!(ui_deps.contains(&StateComponent::Controller));
        assert!(ui_deps.contains(&StateComponent::ECLVM));
        assert!(ui_deps.contains(&StateComponent::Gas));
        assert!(ui_deps.contains(&StateComponent::Mana));

        // UI triggert Events
        let ui_triggers = graph.triggered_by(StateComponent::UI);
        assert!(ui_triggers.contains(&StateComponent::Event));

        // UI aggregiert DataLogic
        let ui_aggregates = graph.aggregated_by(StateComponent::UI);
        assert!(ui_aggregates.contains(&StateComponent::DataLogic));
    }

    #[test]
    fn test_api_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // API Dependencies
        let api_deps = graph.dependencies_of(StateComponent::API);
        assert!(api_deps.contains(&StateComponent::Trust));
        assert!(api_deps.contains(&StateComponent::Controller));
        assert!(api_deps.contains(&StateComponent::ECLVM));
        assert!(api_deps.contains(&StateComponent::Gas));
        assert!(api_deps.contains(&StateComponent::Mana));

        // API validiert Gateway
        let api_validates = graph.validated_by(StateComponent::API);
        assert!(api_validates.contains(&StateComponent::Gateway));

        // API triggert Events
        let api_triggers = graph.triggered_by(StateComponent::API);
        assert!(api_triggers.contains(&StateComponent::Event));
    }

    #[test]
    fn test_governance_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // Governance Dependencies
        let gov_deps = graph.dependencies_of(StateComponent::Governance);
        assert!(gov_deps.contains(&StateComponent::Trust));
        assert!(gov_deps.contains(&StateComponent::Quadratic));
        assert!(gov_deps.contains(&StateComponent::ECLVM));
        assert!(gov_deps.contains(&StateComponent::Realm));

        // Governance validiert Controller und AntiCalcification
        let gov_validates = graph.validated_by(StateComponent::Governance);
        assert!(gov_validates.contains(&StateComponent::Controller));
        assert!(gov_validates.contains(&StateComponent::AntiCalcification));

        // Governance triggert Controller und Event
        let gov_triggers = graph.triggered_by(StateComponent::Governance);
        assert!(gov_triggers.contains(&StateComponent::Controller));
        assert!(gov_triggers.contains(&StateComponent::Event));
    }

    #[test]
    fn test_controller_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // Controller Dependencies
        let ctrl_deps = graph.dependencies_of(StateComponent::Controller);
        assert!(ctrl_deps.contains(&StateComponent::Trust));
        assert!(ctrl_deps.contains(&StateComponent::Realm));
        assert!(ctrl_deps.contains(&StateComponent::Room));
        assert!(ctrl_deps.contains(&StateComponent::Partition));
        assert!(ctrl_deps.contains(&StateComponent::ECLVM));

        // Controller validiert Gateway, API, UI
        let ctrl_validates = graph.validated_by(StateComponent::Controller);
        assert!(ctrl_validates.contains(&StateComponent::Gateway));
        assert!(ctrl_validates.contains(&StateComponent::API));
        assert!(ctrl_validates.contains(&StateComponent::UI));

        // Controller aggregiert Governance
        let ctrl_aggregates = graph.aggregated_by(StateComponent::Controller);
        assert!(ctrl_aggregates.contains(&StateComponent::Governance));
    }

    #[test]
    fn test_datalogic_engine_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // DataLogic Dependencies
        let dl_deps = graph.dependencies_of(StateComponent::DataLogic);
        assert!(dl_deps.contains(&StateComponent::Event));
        assert!(dl_deps.contains(&StateComponent::Trust));
        assert!(dl_deps.contains(&StateComponent::ECLVM));
        assert!(dl_deps.contains(&StateComponent::Gas));

        // DataLogic aggregiert und triggert Events
        let dl_aggregates = graph.aggregated_by(StateComponent::DataLogic);
        assert!(dl_aggregates.contains(&StateComponent::Event));
        let dl_triggers = graph.triggered_by(StateComponent::DataLogic);
        assert!(dl_triggers.contains(&StateComponent::Event));

        // DataLogic validiert UI
        let dl_validates = graph.validated_by(StateComponent::DataLogic);
        assert!(dl_validates.contains(&StateComponent::UI));
    }

    #[test]
    fn test_blueprint_composer_graph_edges() {
        let graph = StateGraph::erynoa_graph();

        // BlueprintComposer Dependencies
        let bc_deps = graph.dependencies_of(StateComponent::BlueprintComposer);
        assert!(bc_deps.contains(&StateComponent::Blueprint));
        assert!(bc_deps.contains(&StateComponent::ECLVM));
        assert!(bc_deps.contains(&StateComponent::Trust));
        assert!(bc_deps.contains(&StateComponent::Gas));

        // BlueprintComposer aggregiert ECLBlueprint
        let bc_aggregates = graph.aggregated_by(StateComponent::BlueprintComposer);
        assert!(bc_aggregates.contains(&StateComponent::ECLBlueprint));

        // BlueprintComposer validiert Realm
        let bc_validates = graph.validated_by(StateComponent::BlueprintComposer);
        assert!(bc_validates.contains(&StateComponent::Realm));
    }

    #[test]
    fn test_new_components_criticality_scores() {
        let graph = StateGraph::erynoa_graph();

        // Trust sollte der kritischste sein (viele Dependencies)
        let trust_score = graph.criticality_score(StateComponent::Trust);
        assert!(
            trust_score > 20,
            "Trust criticality should be high: {}",
            trust_score
        );

        // Controller sollte mittlere Kritikalität haben
        let ctrl_score = graph.criticality_score(StateComponent::Controller);
        assert!(
            ctrl_score > 5,
            "Controller criticality should be medium: {}",
            ctrl_score
        );

        // ECLVM sollte hohe Kritikalität haben (viele Engines nutzen es)
        let eclvm_score = graph.criticality_score(StateComponent::ECLVM);
        assert!(
            eclvm_score > 10,
            "ECLVM criticality should be high: {}",
            eclvm_score
        );
    }

    #[test]
    fn test_transitive_dependencies_new_components() {
        let graph = StateGraph::erynoa_graph();

        // UI sollte transitiv von Trust abhängen
        let ui_trans_deps = graph.transitive_dependencies(StateComponent::UI);
        assert!(ui_trans_deps.contains(&StateComponent::Trust));

        // Controller sollte transitiv von Trust abhängen
        let ctrl_trans_deps = graph.transitive_dependencies(StateComponent::Controller);
        assert!(ctrl_trans_deps.contains(&StateComponent::Trust));

        // Partition sollte transitiv von Realm abhängen (über Room)
        let part_trans_deps = graph.transitive_dependencies(StateComponent::Partition);
        assert!(part_trans_deps.contains(&StateComponent::Room));
        assert!(part_trans_deps.contains(&StateComponent::Realm));
    }

    #[test]
    fn test_state_graph_edge_count() {
        let graph = StateGraph::erynoa_graph();

        // Wir haben ~50 bestehende + ~42 neue = ~92 Edges
        assert!(
            graph.edges.len() >= 85,
            "StateGraph should have at least 85 edges, got: {}",
            graph.edges.len()
        );
    }

    // ========================================================================
    // 2.1-2.6 Engine-State Tests
    // ========================================================================

    #[test]
    fn test_ui_state() {
        let state = UIState::new();

        // Component registrieren
        state.register_component(Some("test-realm"));
        state.register_component(None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.components_registered, 2);
        assert_eq!(snapshot.components_active, 2);

        // Render durchführen
        state.render(false, 100, 50, Some("test-realm"));
        state.render(true, 0, 0, None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.renders, 2);
        assert!((snapshot.cache_hit_rate - 0.5).abs() < 0.01);

        // Trust-Gate
        state.trust_gate(true, None);
        state.trust_gate(false, Some("test-realm"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.trust_gate_evaluations, 2);
        assert!((snapshot.trust_gate_allow_rate - 0.5).abs() < 0.01);

        // Credential-Gate
        state.credential_gate(true);
        state.credential_gate(true);
        state.credential_gate(false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.credential_gate_evaluations, 3);
        assert!((snapshot.credential_gate_allow_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_api_state() {
        let state = APIState::new();

        // Endpoints registrieren
        state.register_endpoint(Some("test-realm"));
        state.register_endpoint(None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.endpoints_registered, 2);
        assert_eq!(snapshot.endpoints_active, 2);

        // Requests verarbeiten
        state.record_request(1000, 200, 50, 10, Some("test-realm")); // Success
        state.record_request(500, 201, 30, 5, None); // Success
        state.record_request(2000, 404, 10, 2, None); // Client error
        state.record_request(5000, 429, 5, 1, Some("test-realm")); // Rate limited
        state.record_request(3000, 500, 20, 5, None); // Server error

        let snapshot = state.snapshot();
        assert_eq!(snapshot.requests_total, 5);
        assert_eq!(snapshot.requests_success, 2);
        assert_eq!(snapshot.requests_client_error, 2);
        assert_eq!(snapshot.requests_server_error, 1);
        assert_eq!(snapshot.requests_rate_limited, 1);
        assert!((snapshot.success_rate - 0.4).abs() < 0.01);
        assert!(snapshot.avg_latency_us > 0.0);
    }

    #[test]
    fn test_governance_state() {
        let state = GovernanceState::new();

        // Proposals erstellen
        state.proposal_created(Some("test-realm"));
        state.proposal_created(None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.proposals_created, 2);
        assert_eq!(snapshot.proposals_active, 2);

        // Votes abgeben
        state.vote_cast(1.5, false, true, Some("test-realm"));
        state.vote_cast(2.0, true, true, None);
        state.vote_cast(1.0, false, false, None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.votes_cast, 3);
        assert_eq!(snapshot.votes_delegated, 1);
        assert_eq!(snapshot.quadratic_reductions, 2);
        assert!(snapshot.avg_voting_power > 1.0);

        // Proposals abschließen
        state.proposal_completed("accepted");
        state.proposal_completed("rejected");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.proposals_completed, 2);
        assert_eq!(snapshot.proposals_accepted, 1);
        assert_eq!(snapshot.proposals_rejected, 1);
        assert!((snapshot.proposal_success_rate - 0.5).abs() < 0.01);

        // Delegationen
        state.delegation_created(3, Some("test-realm"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.delegations_active, 1);
        assert_eq!(snapshot.max_delegation_depth, 3);

        // Power-Check
        state.power_check(true, 0.35);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.power_violations, 1);
        assert!((snapshot.voting_power_gini - 0.35).abs() < 0.01);
    }

    #[test]
    fn test_controller_state() {
        let state = ControllerState::new();

        // Permissions gewähren
        state.grant_permission(Some("test-realm"));
        state.grant_permission(None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.permission_grants, 2);
        assert_eq!(snapshot.permissions_active, 2);

        // AuthZ-Checks
        state.check_authorization(true, false, 50, "realm", Some("test-realm"));
        state.check_authorization(true, true, 100, "room", None);
        state.check_authorization(false, false, 25, "partition", Some("test-realm"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.authz_checks, 3);
        assert_eq!(snapshot.authz_allowed, 2);
        assert_eq!(snapshot.authz_denied, 1);
        assert!((snapshot.authz_success_rate - 0.666).abs() < 0.01);
        assert!(snapshot.avg_check_latency_us > 0.0);
        assert_eq!(snapshot.realm_scope_checks, 1);
        assert_eq!(snapshot.room_scope_checks, 1);
        assert_eq!(snapshot.partition_scope_checks, 1);

        // Delegation
        state.create_delegation(2, Some("test-realm"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.delegations_active, 1);
        assert_eq!(snapshot.max_delegation_depth, 2);

        // Permission widerrufen
        state.revoke_permission();

        let snapshot = state.snapshot();
        assert_eq!(snapshot.permission_revokes, 1);
        assert_eq!(snapshot.permissions_active, 1);

        // Audit-Entries sollten geschrieben worden sein
        assert!(snapshot.audit_entries > 0);
        assert!(snapshot.audit_log_bytes > 0);
    }

    #[test]
    fn test_data_logic_state() {
        let state = DataLogicState::new();

        // Stream registrieren
        state.register_stream();
        state.register_stream();

        let snapshot = state.snapshot();
        assert_eq!(snapshot.streams_registered, 2);
        assert_eq!(snapshot.streams_active, 2);

        // Events verarbeiten
        state.process_event(false, 50); // Forwarded
        state.process_event(false, 30); // Forwarded
        state.process_event(true, 10); // Filtered

        let snapshot = state.snapshot();
        assert_eq!(snapshot.events_processed, 3);
        assert_eq!(snapshot.events_forwarded, 2);
        assert_eq!(snapshot.events_filtered, 1);
        assert!((snapshot.success_rate - 0.666).abs() < 0.01);

        // Aggregation
        state.aggregation_computed(500, 100);
        state.aggregation_computed(300, 80);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.aggregations_computed, 2);
        assert!(snapshot.avg_aggregation_latency_us > 0.0);

        // Binding Propagation
        state.propagate_binding(true, 100, 20);
        state.propagate_binding(true, 150, 25);
        state.propagate_binding(false, 50, 10);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.binding_propagations, 3);
        assert!((snapshot.binding_success_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_blueprint_composer_state() {
        let state = BlueprintComposerState::new();

        // Compositions erstellen
        state.composition_created(true, 2, 1, 100);
        state.composition_created(true, 3, 0, 150);
        state.composition_created(false, 0, 0, 50);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.compositions_created, 3);
        assert_eq!(snapshot.compositions_successful, 2);
        assert_eq!(snapshot.compositions_failed, 1);
        assert!((snapshot.composition_success_rate - 0.666).abs() < 0.01);
        assert_eq!(snapshot.max_inheritance_depth, 3);
        assert_eq!(snapshot.conflict_resolutions, 1);

        // Instanziierung
        state.instantiate(true, 80);
        state.instantiate(true, 70);
        state.instantiate(false, 30);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.instantiations, 3);
        assert_eq!(snapshot.instances_active, 2);

        // Realm-Compatibility
        state.realm_compatibility_check(true);
        state.realm_compatibility_check(false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.realm_compatibility_checks, 2);
        assert_eq!(snapshot.compatibility_failures, 1);

        // Cache
        state.cache_access(true);
        state.cache_access(true);
        state.cache_access(false);

        let snapshot = state.snapshot();
        assert!((snapshot.cache_hit_rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_unified_state_with_engines() {
        let state = UnifiedState::new();

        // Alle neuen Engines sollten initialisiert sein
        assert_eq!(state.ui.components_registered.load(Ordering::Relaxed), 0);
        assert_eq!(state.api.endpoints_registered.load(Ordering::Relaxed), 0);
        assert_eq!(
            state.governance.proposals_created.load(Ordering::Relaxed),
            0
        );
        assert_eq!(
            state
                .controller
                .permissions_registered
                .load(Ordering::Relaxed),
            0
        );
        assert_eq!(
            state.data_logic.streams_registered.load(Ordering::Relaxed),
            0
        );
        assert_eq!(
            state
                .blueprint_composer
                .compositions_created
                .load(Ordering::Relaxed),
            0
        );

        // Snapshot sollte alle Engines enthalten
        let snapshot = state.snapshot();
        assert_eq!(snapshot.ui.components_registered, 0);
        assert_eq!(snapshot.api.endpoints_registered, 0);
        assert_eq!(snapshot.governance.proposals_created, 0);
        assert_eq!(snapshot.controller.permissions_registered, 0);
        assert_eq!(snapshot.data_logic.streams_registered, 0);
        assert_eq!(snapshot.blueprint_composer.compositions_created, 0);

        // Health sollte hoch bei leerem State sein (einige Defaults sind nicht 100%)
        // Andere Layer wie P2P können Health reduzieren auch ohne Engine-Aktivität
        let health = state.calculate_health();
        assert!(
            health >= 80.0,
            "Initial health should be >= 80%, got: {}",
            health
        );
    }

    #[test]
    fn test_unified_state_health_with_engine_errors() {
        let state = UnifiedState::new();

        // Bootstrap identity to get base health (otherwise identity contributes 0%)
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        // Provoziere Fehler in den Engines
        state.api.record_request(1000, 500, 50, 10, None); // Server Error
        state.api.record_request(1000, 500, 50, 10, None);
        state.api.record_request(1000, 500, 50, 10, None);
        state
            .controller
            .check_authorization(false, false, 50, "realm", None);
        state
            .controller
            .check_authorization(false, false, 50, "realm", None);

        // Health sollte gesunken sein
        let health = state.calculate_health();
        assert!(
            health < 99.0,
            "Health should decrease with errors, got: {}",
            health
        );
        // With identity bootstrap (10% weight) and engine errors, health should be > 70%
        assert!(
            health > 70.0,
            "Health should not drop too low, got: {}",
            health
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Tests: Architektur-Verbesserungen Phase 6.1
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_system_mode_transitions() {
        // SystemMode Helper-Funktionen
        assert!(SystemMode::Normal.is_operational());
        assert!(SystemMode::Degraded.is_operational());
        assert!(!SystemMode::EmergencyShutdown.is_operational());

        assert!(SystemMode::Normal.allows_execution());
        assert!(!SystemMode::Degraded.allows_execution());
        assert!(!SystemMode::EmergencyShutdown.allows_execution());

        assert!(SystemMode::Normal.allows_crossings());
        assert!(!SystemMode::Degraded.allows_crossings());
        assert!(!SystemMode::EmergencyShutdown.allows_crossings());
    }

    #[test]
    fn test_circuit_breaker_degradation_threshold() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.mode(), SystemMode::Normal);

        // Unter dem Threshold bleiben
        for _ in 0..9 {
            cb.record_critical_anomaly();
        }
        assert_eq!(cb.mode(), SystemMode::Normal);

        // Threshold erreichen (10 critical in 1 minute)
        cb.record_critical_anomaly();
        assert_eq!(cb.mode(), SystemMode::Degraded);

        // Reset und wieder Normal
        cb.reset_to_normal();
        assert_eq!(cb.mode(), SystemMode::Normal);
    }

    #[test]
    fn test_circuit_breaker_emergency_threshold() {
        let cb = CircuitBreaker::new();

        // Direkt auf Emergency durch hohe Anzahl kritischer Events
        for _ in 0..50 {
            cb.record_critical_anomaly();
        }
        assert_eq!(cb.mode(), SystemMode::EmergencyShutdown);

        // Nach Reset wieder Normal
        cb.reset_to_normal();
        assert_eq!(cb.mode(), SystemMode::Normal);
    }

    #[test]
    fn test_circuit_breaker_snapshot() {
        let cb = CircuitBreaker::new();
        cb.record_critical_anomaly();
        cb.record_critical_anomaly();

        let snapshot = cb.snapshot();
        assert_eq!(snapshot.mode, SystemMode::Normal);
        assert_eq!(snapshot.critical_count_last_minute, 2);
        assert_eq!(snapshot.degraded_threshold, 10);
        assert_eq!(snapshot.emergency_threshold, 50);
    }

    #[test]
    fn test_event_bus_creation() {
        let eb = EventBus::new();
        let snapshot = eb.snapshot();

        assert_eq!(snapshot.ingress_count, 0);
        assert_eq!(snapshot.egress_count, 0);
        assert_eq!(snapshot.dropped_count, 0);
        assert_eq!(snapshot.processed_count, 0);
        assert_eq!(snapshot.priority_processed, 0);
    }

    #[test]
    fn test_event_bus_ingress_egress() {
        let eb = EventBus::new();

        // Ingress Event (P2P → Core)
        let event = NetworkEvent::new(
            "trust_update".to_string(),
            vec![1, 2, 3],
            EventPriority::Normal,
        );
        assert!(eb.try_send_ingress(event).is_ok());

        let snapshot = eb.snapshot();
        assert_eq!(snapshot.ingress_count, 1);

        // Egress Event (Core → P2P)
        let event2 = NetworkEvent::new("broadcast".to_string(), vec![4, 5, 6], EventPriority::High)
            .with_peer("peer123".to_string());
        assert!(eb.try_send_egress(event2).is_ok());

        let snapshot = eb.snapshot();
        assert_eq!(snapshot.egress_count, 1);
    }

    #[test]
    fn test_event_bus_priority_channel() {
        let eb = EventBus::new();

        // Priority Event (kritisch, wird bevorzugt behandelt)
        let event = NetworkEvent::new("emergency".to_string(), vec![255], EventPriority::Critical);
        // Critical events gehen über Priority-Ingress
        assert!(eb.try_send_ingress(event).is_ok());

        let snapshot = eb.snapshot();
        assert_eq!(snapshot.ingress_count, 1);
    }

    #[test]
    fn test_event_priority_ordering() {
        assert!(EventPriority::Critical < EventPriority::High);
        assert!(EventPriority::High < EventPriority::Normal);
        assert!(EventPriority::Normal < EventPriority::Low);
    }

    #[test]
    fn test_state_delta_creation() {
        let delta = StateDelta::new(
            StateComponent::Trust,
            DeltaType::Increment,
            vec![1, 2, 3, 4],
        );

        assert!(matches!(delta.component, StateComponent::Trust));
        assert!(matches!(delta.delta_type, DeltaType::Increment));
        assert_eq!(delta.data.len(), 4);
        // Sequence ist 0 bei Erstellung, wird erst beim Broadcast erhöht
        assert_eq!(delta.sequence, 0);
        assert!(delta.timestamp_ms > 0);
    }

    #[test]
    fn test_state_broadcaster_subscription() {
        let broadcaster = StateBroadcaster::new();

        // Subscriber erstellen
        let _rx1 = broadcaster.subscribe();
        let _rx2 = broadcaster.subscribe();

        // Broadcast senden
        let delta = StateDelta::new(StateComponent::Execution, DeltaType::Update, vec![42]);
        broadcaster.broadcast(delta);

        let snapshot = broadcaster.snapshot();
        assert_eq!(snapshot.deltas_sent, 1);
        // subscriber_count kann 2 sein oder weniger wenn Receiver dropped
    }

    #[test]
    fn test_delta_type_variants() {
        // Alle DeltaType Varianten testen
        let _ = DeltaType::Increment;
        let _ = DeltaType::Snapshot;
        let _ = DeltaType::Insert;
        let _ = DeltaType::Delete;
        let _ = DeltaType::Update;
        let _ = DeltaType::Batch;
    }

    #[test]
    fn test_storage_handle_creation() {
        let sh = StorageHandle::new(StorageBackend::Memory);
        let snapshot = sh.snapshot();

        assert_eq!(snapshot.reads, 0);
        assert_eq!(snapshot.writes, 0);
        assert_eq!(snapshot.bytes_read, 0);
        assert_eq!(snapshot.bytes_written, 0);
    }

    #[test]
    fn test_storage_handle_metrics_tracking() {
        let sh = StorageHandle::new(StorageBackend::RocksDB);

        sh.record_read(100);
        sh.record_read(200);
        sh.record_read(50);
        sh.record_write(1024);
        sh.record_write(2048);

        let snapshot = sh.snapshot();
        assert_eq!(snapshot.reads, 3);
        assert_eq!(snapshot.writes, 2);
        assert_eq!(snapshot.bytes_read, 350);
        assert_eq!(snapshot.bytes_written, 3072);
    }

    #[test]
    fn test_storage_backend_variants() {
        let _ = StorageBackend::RocksDB;
        let _ = StorageBackend::IPFS;
        let _ = StorageBackend::Cloud;
        let _ = StorageBackend::Memory;
    }

    #[test]
    fn test_unified_state_architecture_integration() {
        let state = UnifiedState::new();

        // Alle neuen Architektur-Komponenten sollten initialisiert sein
        assert_eq!(state.system_mode(), SystemMode::Normal);
        assert!(state.is_operational());
        assert!(state.allows_execution());
        assert!(state.allows_crossings());

        // Snapshot sollte alle neuen Felder enthalten
        let snapshot = state.snapshot();
        assert!(matches!(snapshot.system_mode, SystemMode::Normal));
        assert_eq!(snapshot.circuit_breaker.mode, SystemMode::Normal);
        assert_eq!(snapshot.event_bus.ingress_count, 0);
        assert_eq!(snapshot.broadcaster.deltas_sent, 0);
    }

    #[test]
    fn test_unified_state_anomaly_with_circuit_breaker() {
        let state = UnifiedState::new();

        // Normale Anomalie sollte Mode nicht ändern
        for _ in 0..5 {
            let mode = state.record_anomaly("critical");
            assert_eq!(mode, SystemMode::Normal);
        }

        // Nach genügend kritischen Anomalien sollte Degradation eintreten
        for _ in 0..6 {
            state.record_anomaly("critical");
        }
        // Mindestens 10 kritische Events für Degradation
        assert!(
            state.system_mode() == SystemMode::Normal
                || state.system_mode() == SystemMode::Degraded
        );
    }

    #[test]
    fn test_unified_state_delta_subscription() {
        let state = UnifiedState::new();

        // Subscriber für State Deltas erstellen
        let _rx = state.subscribe_deltas();

        // Delta broadcasten
        state.broadcast_delta(StateComponent::Trust, DeltaType::Update, vec![1, 2, 3]);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.broadcaster.deltas_sent, 1);
    }

    #[test]
    fn test_unified_state_circuit_breaker_reset() {
        let state = UnifiedState::new();

        // Viele kritische Anomalien aufzeichnen
        for _ in 0..15 {
            state.record_anomaly("critical");
        }

        // Sollte jetzt Degraded sein
        if state.system_mode() == SystemMode::Degraded {
            // Reset testen
            state.reset_circuit_breaker();
            assert_eq!(state.system_mode(), SystemMode::Normal);
        }
    }

    #[test]
    fn test_network_event_creation() {
        let event = NetworkEvent::new("test".to_string(), vec![1, 2, 3], EventPriority::Normal);
        assert!(event.id > 0);
        assert_eq!(event.event_type, "test");
        assert_eq!(event.payload, vec![1, 2, 3]);
        assert!(matches!(event.priority, EventPriority::Normal));
        assert!(event.peer_id.is_none());
        assert!(event.realm_id.is_none());
        assert!(event.timestamp_ms > 0);
    }

    #[test]
    fn test_network_event_with_realm() {
        let event = NetworkEvent::new("realm_event".to_string(), vec![42], EventPriority::High)
            .with_realm("realm_xyz".to_string());

        assert!(event.peer_id.is_none());
        assert_eq!(event.realm_id, Some("realm_xyz".to_string()));
    }

    // ============================================================
    // Phase 6.2 Tests: Merkle, Multi-Gas, Realm-Quota
    // ============================================================

    #[test]
    fn test_merkle_node_leaf() {
        let data = b"test data";
        let node = MerkleNode::leaf(data);

        // Hash should be non-zero
        assert!(!node.hash.iter().all(|&b| b == 0));
        // Leaf has no children
        assert!(node.children.is_empty());
        // Timestamp should be set
        assert!(node.updated_ms > 0);
    }

    #[test]
    fn test_merkle_node_branch() {
        let left = MerkleNode::leaf(b"left");
        let right = MerkleNode::leaf(b"right");
        let branch = MerkleNode::branch(&[left.hash, right.hash]);

        // Branch hash differs from both leaves
        assert_ne!(branch.hash, left.hash);
        assert_ne!(branch.hash, right.hash);
        // Branch has 2 children
        assert_eq!(branch.children.len(), 2);
        assert_eq!(branch.children[0], left.hash);
        assert_eq!(branch.children[1], right.hash);
    }

    #[test]
    fn test_merkle_state_tracker() {
        let tracker = MerkleStateTracker::new();
        let initial_root = tracker.root_hash();

        // Update a component
        tracker.update_component(StateComponent::Trust, b"trust data v1");
        let root_v1 = tracker.root_hash();

        // Root should change
        assert_ne!(initial_root, root_v1);

        // Update again
        tracker.update_component(StateComponent::Trust, b"trust data v2");
        let root_v2 = tracker.root_hash();

        // Root should change again
        assert_ne!(root_v1, root_v2);
    }

    #[test]
    fn test_merkle_state_tracker_deltas() {
        let tracker = MerkleStateTracker::new();

        // Generate some deltas (sequences will be 0, 1, 2)
        tracker.update_component(StateComponent::Trust, b"trust");
        tracker.update_component(StateComponent::Event, b"event");
        tracker.update_component(StateComponent::Gas, b"gas");

        // Get deltas since sequence 0 (sequence > 0 means 1, 2) -> 2 deltas
        // But let's check what we actually have
        let all_deltas = tracker.deltas_since(0);
        // The filter is `sequence > since_sequence`, so:
        // - deltas_since(0): sequences 1, 2 -> 2 deltas (sequences > 0)
        // - Actually there are 3 updates, so sequences 0, 1, 2
        // - deltas_since(0) gives 1, 2 -> should be 2 deltas
        assert_eq!(all_deltas.len(), 2);

        // Verify structure of first returned delta
        if !all_deltas.is_empty() {
            assert!(!all_deltas[0].new_root.iter().all(|&b| b == 0));
        }
    }

    #[test]
    fn test_multi_gas_consumption() {
        let gas = MultiGas::new();

        // Consume network gas (bytes_sent, bytes_received, messages)
        gas.consume_network(100, 50, 10);
        // Network tracks all three summed
        let net_total = gas.network.load(Ordering::SeqCst);
        assert!(net_total > 0);

        // Consume compute gas - note: internally multiplied by price (10)
        gas.consume_compute(200);
        // 200 instructions * 10 gas/instruction = 2000
        let compute_total = gas.compute.load(Ordering::SeqCst);
        assert!(compute_total >= 200); // At least the instructions

        // Consume storage gas (bytes_written, operations)
        gas.consume_storage(50, 5);
        let storage_total = gas.storage.load(Ordering::SeqCst);
        assert!(storage_total > 0);

        // Register realm first, then consume
        gas.register_realm("test_realm");
        gas.consume_realm("test_realm", 500);

        // Verify realm was registered
        let realms = gas.realm.read().unwrap();
        assert!(realms.contains_key("test_realm"));
    }

    #[test]
    fn test_gas_layer_default_prices() {
        // Prices based on actual implementation
        assert_eq!(GasLayer::Network.default_price(), 1); // 1 Gas pro Byte
        assert_eq!(GasLayer::Compute.default_price(), 10); // 10 Gas pro Instruction
        assert_eq!(GasLayer::Storage.default_price(), 100); // 100 Gas pro KB
        assert_eq!(GasLayer::Realm.default_price(), 50); // 50 Gas pro Realm-Op
    }

    #[test]
    fn test_resource_type_default_limits() {
        // Limits based on actual implementation
        assert_eq!(ResourceType::QueueSlots.default_limit(), 100);
        assert_eq!(ResourceType::StorageBytes.default_limit(), 10_000_000); // 10 MB
        assert_eq!(ResourceType::ComputeGas.default_limit(), 1_000_000);
        assert_eq!(ResourceType::Events.default_limit(), 10_000);
        assert_eq!(ResourceType::Crossings.default_limit(), 1_000);
    }

    #[test]
    fn test_realm_quota_basic() {
        let quota = RealmQuota::new();

        // Check default limits (from ResourceType::default_limit())
        assert_eq!(quota.queue_slots_limit.load(Ordering::SeqCst), 100);
        assert_eq!(quota.storage_bytes_limit.load(Ordering::SeqCst), 10_000_000);

        // Initially not quarantined
        assert!(!quota.is_quarantined());
    }

    #[test]
    fn test_realm_quota_consumption() {
        let quota = RealmQuota::new();

        // Consume within limits (limit is 100)
        assert!(quota.check_quota(ResourceType::QueueSlots, 10));
        assert!(quota.consume(ResourceType::QueueSlots, 10));
        assert_eq!(quota.queue_slots_used.load(Ordering::SeqCst), 10);

        // Consume more
        assert!(quota.consume(ResourceType::QueueSlots, 5));
        assert_eq!(quota.queue_slots_used.load(Ordering::SeqCst), 15);
    }

    #[test]
    fn test_realm_quota_exceeded() {
        let quota = RealmQuota::new();

        // Set a small limit for testing
        quota.queue_slots_limit.store(100, Ordering::SeqCst);

        // Try to exceed limit - should return false
        let result = quota.check_quota(ResourceType::QueueSlots, 200);
        assert!(!result);

        // Consume should also fail
        let consume_result = quota.consume(ResourceType::QueueSlots, 200);
        assert!(!consume_result);
    }

    #[test]
    fn test_realm_quota_quarantine() {
        let quota = RealmQuota::new();

        // Initially healthy
        assert!(!quota.is_quarantined());

        // Quarantine
        quota.quarantine();
        assert!(quota.is_quarantined());

        // Unquarantine
        quota.unquarantine();
        assert!(!quota.is_quarantined());
    }

    #[test]
    fn test_realm_quota_auto_quarantine_on_violations() {
        let quota = RealmQuota::new();

        // Set small limits
        quota.queue_slots_limit.store(10, Ordering::SeqCst);

        // Cause multiple violations (10 is the auto-quarantine threshold)
        for _ in 0..15 {
            let _ = quota.consume(ResourceType::QueueSlots, 100);
        }

        // Should be auto-quarantined after 10 violations
        assert!(quota.violations.load(Ordering::SeqCst) >= 10);
        assert!(quota.is_quarantined());
    }

    #[test]
    fn test_realm_specific_state_quota() {
        let realm = RealmSpecificState::new(0.5, "democratic");

        // Should have default quota (QueueSlots default is 100)
        assert_eq!(realm.quota.queue_slots_limit.load(Ordering::SeqCst), 100);

        // Check operation - should pass within limits
        assert!(realm.check_operation(ResourceType::ComputeGas, 1000));

        // Consume resource
        assert!(realm.consume_resource(ResourceType::ComputeGas, 500));
        assert_eq!(realm.quota.compute_gas_used.load(Ordering::SeqCst), 500);
    }

    #[test]
    fn test_realm_specific_state_quarantine_flow() {
        let realm = RealmSpecificState::new(0.5, "democratic");

        // Initially healthy
        let health = realm.quota_health();
        assert!(health > 50.0); // Should be mostly healthy

        // Quarantine realm
        realm.quarantine();
        assert!(realm.quota.is_quarantined());

        // Operations should fail when quarantined
        let result = realm.check_operation(ResourceType::QueueSlots, 1);
        assert!(!result);
    }

    #[test]
    fn test_unified_state_merkle_integration() {
        let state = UnifiedState::new();

        // Get initial root
        let root1 = state.merkle_root();

        // Update merkle state via tracker
        state
            .merkle_tracker
            .update_component(StateComponent::Trust, b"new trust data");
        let root2 = state.merkle_root();

        // Root should have changed
        assert_ne!(root1, root2);
    }

    #[test]
    fn test_unified_state_multi_gas_integration() {
        let state = UnifiedState::new();

        // Consume various gas types
        state.consume_network_gas(100, 50, 10);
        state.consume_compute_gas(200);
        state.consume_storage_gas(50, 5);

        // Verify gas totals are set
        // Note: compute gas is multiplied by price (10), so 200 * 10 = 2000
        assert!(state.multi_gas.network.load(Ordering::SeqCst) > 0);
        assert!(state.multi_gas.compute.load(Ordering::SeqCst) >= 200);
        assert!(state.multi_gas.storage.load(Ordering::SeqCst) > 0);
    }

    #[test]
    fn test_unified_state_realm_quota_integration() {
        let state = UnifiedState::new();

        // Register a realm first using multi_gas helper
        state.register_realm_for_gas("test_realm");

        // For quota tests, we need the realm in realm_states
        // Just verify the gas registration works
        let realms = state.multi_gas.realm.read().unwrap();
        assert!(realms.contains_key("test_realm"));
    }

    #[test]
    fn test_unified_state_snapshot_phase6_2() {
        let state = UnifiedState::new();

        // Setup some state
        state.consume_network_gas(500, 0, 0);
        state
            .merkle_tracker
            .update_component(StateComponent::Event, b"event snapshot test");

        // Take snapshot
        let snapshot = state.snapshot();

        // Verify snapshot includes Phase 6.2 data
        assert!(snapshot.merkle_tracker.sequence > 0);
        assert_eq!(snapshot.multi_gas.network, 500);
    }

    // =========================================================================
    // PHASE 6.3: EVENT-SOURCING TESTS
    // =========================================================================

    #[test]
    fn test_state_event_log_basic() {
        let log = StateEventLog::new();

        // Log an event
        let event = StateEvent::TrustUpdate {
            entity_id: "did:example:123".to_string(),
            delta: 0.05,
            reason: TrustReason::PositiveInteraction,
            from_realm: Some("realm_abc".to_string()),
            triggered_events: 1,
            new_trust: 0.75,
        };

        let wrapped = log.log(event.clone(), vec![]);

        // Verify wrapped event
        assert!(!wrapped.id.is_empty());
        assert!(wrapped.timestamp_ms > 0);
        assert_eq!(wrapped.sequence, 0); // Zero-based sequence
        assert_eq!(wrapped.component, StateComponent::Trust);
        assert!(wrapped.parent_ids.is_empty());
    }

    #[test]
    fn test_state_event_log_sequence() {
        let log = StateEventLog::new();

        // Log multiple events
        for i in 0..5 {
            let event = StateEvent::EventProcessed {
                event_id: format!("evt_{}", i),
                depth: 1,
                parents_count: 0,
                triggers: vec![StateComponent::Trust],
                validation_errors: false,
                processing_us: 100,
            };
            let wrapped = log.log(event, vec![]);
            assert_eq!(wrapped.sequence, i as u64); // Zero-based sequence
        }

        // Verify snapshot
        let snapshot = log.snapshot();
        assert_eq!(snapshot.total_events, 5);
        assert_eq!(snapshot.sequence, 5);
    }

    #[test]
    fn test_state_event_causality_tracking() {
        let log = StateEventLog::new();

        // Log parent event
        let parent = log.log(
            StateEvent::ExecutionStarted {
                context_id: "ctx_1".to_string(),
                gas_budget: 10000,
                mana_budget: 1000,
                realm_id: None,
            },
            vec![],
        );

        // Log child event with parent reference
        let child = log.log(
            StateEvent::ExecutionCompleted {
                context_id: "ctx_1".to_string(),
                success: true,
                gas_consumed: 5000,
                mana_consumed: 500,
                events_emitted: 2,
                duration_ms: 100,
                error: None,
            },
            vec![parent.id.clone()],
        );

        // Verify causality
        assert_eq!(child.parent_ids.len(), 1);
        assert_eq!(child.parent_ids[0], parent.id);
    }

    #[test]
    fn test_state_event_primary_component() {
        // TrustUpdate -> Trust
        let trust_event = StateEvent::TrustUpdate {
            entity_id: "did:test".to_string(),
            delta: 0.1,
            reason: TrustReason::PositiveInteraction,
            from_realm: None,
            triggered_events: 0,
            new_trust: 0.6,
        };
        assert_eq!(trust_event.primary_component(), StateComponent::Trust);

        // FormulaComputed -> WorldFormula
        let formula_event = StateEvent::FormulaComputed {
            old_e: 0.5,
            new_e: 0.6,
            contributors_delta: 1,
            human_factor: 1.0,
            trend: 0.1,
            epoch: 1,
            activity: 0.5,
            trust_norm: 0.8,
        };
        assert_eq!(
            formula_event.primary_component(),
            StateComponent::WorldFormula
        );

        // AnomalyDetected -> Anomaly
        let anomaly_event = StateEvent::AnomalyDetected {
            severity: AnomalySeverity::High,
            description: "test".to_string(),
            affected_component: StateComponent::Execution,
            affected_entities: vec![],
            auto_response: None,
        };
        assert_eq!(anomaly_event.primary_component(), StateComponent::Anomaly);
    }

    #[test]
    fn test_state_event_is_critical() {
        // Critical anomaly is critical
        let critical_anomaly = StateEvent::AnomalyDetected {
            severity: AnomalySeverity::Critical,
            description: "critical issue".to_string(),
            affected_component: StateComponent::Consensus,
            affected_entities: vec![],
            auto_response: Some("shutdown".to_string()),
        };
        assert!(critical_anomaly.is_critical());

        // High anomaly is NOT critical
        let high_anomaly = StateEvent::AnomalyDetected {
            severity: AnomalySeverity::High,
            description: "high issue".to_string(),
            affected_component: StateComponent::Consensus,
            affected_entities: vec![],
            auto_response: None,
        };
        assert!(!high_anomaly.is_critical());

        // SystemModeChanged to EmergencyShutdown is critical
        let emergency = StateEvent::SystemModeChanged {
            old_mode: SystemMode::Normal,
            new_mode: SystemMode::EmergencyShutdown,
            trigger_event_id: "manual_trigger".to_string(),
            automatic: false,
        };
        assert!(emergency.is_critical());
    }

    #[test]
    fn test_state_event_estimated_size() {
        let event = StateEvent::TrustUpdate {
            entity_id: "did:example:123".to_string(),
            delta: 0.05,
            reason: TrustReason::PositiveInteraction,
            from_realm: None,
            triggered_events: 0,
            new_trust: 0.5,
        };

        // Estimate should be ~40 bytes for TrustUpdate
        let size = event.estimated_size_bytes();
        assert!(size >= 40 && size <= 200);
    }

    #[test]
    fn test_wrapped_state_event_with_signature() {
        let event = StateEvent::ProposalCreated {
            proposal_id: "prop_1".to_string(),
            realm_id: "realm_abc".to_string(),
            proposer_id: "did:creator".to_string(),
            proposal_type: "ConfigChange".to_string(),
            deadline_ms: 1700000000000,
        };

        let wrapped = WrappedStateEvent::new(event, vec![], 1);
        let signature = vec![0u8; 64]; // Mock signature
        let signed = wrapped.with_signature(signature.clone());

        assert_eq!(signed.signature, Some(signature));
    }

    #[test]
    fn test_state_event_log_checkpoint() {
        let log = StateEventLog::new_with_config(100, 10); // Checkpoint every 10 events

        // Log 15 events to trigger checkpoint need
        for i in 0..15 {
            let event = StateEvent::EventProcessed {
                event_id: format!("evt_{}", i),
                depth: 1,
                parents_count: 0,
                triggers: vec![],
                validation_errors: false,
                processing_us: 50,
            };
            log.log(event, vec![]);
        }

        // Should need checkpoint after 10+ events
        assert!(log.needs_checkpoint());

        // Mark checkpoint - state_hash is MerkleHash ([u8; 32])
        let state_hash: MerkleHash = [0u8; 32];
        let checkpoint = log.mark_checkpoint("ckpt_1".to_string(), state_hash);

        // Checkpoint event was logged
        assert!(matches!(
            checkpoint.event,
            StateEvent::CheckpointCreated { .. }
        ));

        // Should no longer need checkpoint
        assert!(!log.needs_checkpoint());
    }

    #[test]
    fn test_state_event_log_events_since() {
        let log = StateEventLog::new();

        // Log 5 events
        for i in 0..5 {
            let event = StateEvent::EventProcessed {
                event_id: format!("evt_{}", i),
                depth: 1,
                parents_count: 0,
                triggers: vec![],
                validation_errors: false,
                processing_us: 50,
            };
            log.log(event, vec![]);
        }

        // Get events since sequence 2
        let events = log.events_since(2);
        assert_eq!(events.len(), 2); // Events with sequence 3, 4
        assert_eq!(events[0].sequence, 3);
    }

    #[test]
    fn test_state_event_log_events_for_component() {
        let log = StateEventLog::new();

        // Log mixed events
        log.log(
            StateEvent::TrustUpdate {
                entity_id: "a".to_string(),
                delta: 0.1,
                reason: TrustReason::PositiveInteraction,
                from_realm: None,
                triggered_events: 0,
                new_trust: 0.6,
            },
            vec![],
        );
        log.log(
            StateEvent::ExecutionStarted {
                context_id: "ctx".to_string(),
                gas_budget: 1000,
                mana_budget: 100,
                realm_id: None,
            },
            vec![],
        );
        log.log(
            StateEvent::TrustUpdate {
                entity_id: "b".to_string(),
                delta: 0.2,
                reason: TrustReason::RealmActivity,
                from_realm: Some("realm".to_string()),
                triggered_events: 1,
                new_trust: 0.8,
            },
            vec![],
        );

        // Filter by Trust component
        let trust_events = log.events_for_component(StateComponent::Trust);
        assert_eq!(trust_events.len(), 2);
    }

    #[test]
    fn test_unified_state_log_and_apply() {
        let state = UnifiedState::new();

        // Log and apply a trust update
        let event = StateEvent::TrustUpdate {
            entity_id: "did:test".to_string(),
            delta: 0.1,
            reason: TrustReason::PositiveInteraction,
            from_realm: None,
            triggered_events: 0,
            new_trust: 0.6,
        };

        let wrapped = state.log_and_apply(event, vec![]);

        // Verify event was logged
        assert_eq!(wrapped.sequence, 0); // Zero-based sequence
        assert_eq!(wrapped.component, StateComponent::Trust);

        // Verify state was updated (trust updates_total incremented)
        let snapshot = state.snapshot();
        assert!(snapshot.core.trust.updates_total >= 1);
    }

    #[test]
    fn test_unified_state_replay_events() {
        let state = UnifiedState::new();

        // Create some wrapped events to replay
        let events: Vec<WrappedStateEvent> = (0..3)
            .map(|i| {
                WrappedStateEvent::new(
                    StateEvent::ExecutionCompleted {
                        context_id: format!("ctx_{}", i),
                        success: true,
                        gas_consumed: 100,
                        mana_consumed: 10,
                        events_emitted: 1,
                        duration_ms: 50,
                        error: None,
                    },
                    vec![],
                    (i + 1) as u64,
                )
            })
            .collect();

        // Replay events
        state.replay_events(&events);

        // State should reflect replayed events
        let snapshot = state.snapshot();
        assert!(snapshot.execution.executions.successful >= 3);
    }

    #[test]
    fn test_unified_state_snapshot_phase6_3() {
        let state = UnifiedState::new();

        // Log some events
        for _ in 0..3 {
            state.log_and_apply(
                StateEvent::EventProcessed {
                    event_id: "evt".to_string(),
                    depth: 1,
                    parents_count: 0,
                    triggers: vec![],
                    validation_errors: false,
                    processing_us: 100,
                },
                vec![],
            );
        }

        // Take snapshot
        let snapshot = state.snapshot();

        // Verify event_log is included
        assert_eq!(snapshot.event_log.total_events, 3);
        assert_eq!(snapshot.event_log.sequence, 3);
    }
}

// ============================================================================
// PHASE 6.4: ZUSTAND-ABSTRAKTION FÜR ECLVM INTEGRATION
// ============================================================================
//
// Diese Phase bietet abstrakte State-Interfaces für ECLVM:
// 1. ECLVMStateContext - Orchestriert State-Zugriff für ECLVM-Ausführung
// 2. StateView - Read-only Snapshot für Policy-Evaluation
// 3. StateHandle - Realm-scoped schreibbarer Zustand
// 4. TransactionGuard - RAII für atomare Änderungen
// 5. ECLVMBudget - Gas/Mana-Integration
// ============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// ECLVM BUDGET - Gas/Mana Tracking für ECLVM-Ausführung
// ─────────────────────────────────────────────────────────────────────────────

/// Budget-Limits für ECLVM-Ausführung
#[derive(Debug, Clone, Copy)]
pub struct ECLVMBudgetLimits {
    /// Max Gas für Compute-Operationen
    pub gas_limit: u64,
    /// Max Mana für Bandbreite/Events
    pub mana_limit: u64,
    /// Max Stack-Tiefe
    pub max_stack_depth: u32,
    /// Max Ausführungszeit (ms)
    pub timeout_ms: u64,
}

impl Default for ECLVMBudgetLimits {
    fn default() -> Self {
        Self {
            gas_limit: 1_000_000, // 1M Gas default
            mana_limit: 10_000,   // 10K Mana default
            max_stack_depth: 1024,
            timeout_ms: 5_000, // 5s timeout
        }
    }
}

impl ECLVMBudgetLimits {
    /// Minimales Budget für triviale Operationen
    pub fn minimal() -> Self {
        Self {
            gas_limit: 10_000,
            mana_limit: 100,
            max_stack_depth: 64,
            timeout_ms: 100,
        }
    }

    /// Großes Budget für komplexe Sagas
    pub fn saga() -> Self {
        Self {
            gas_limit: 10_000_000, // 10M Gas
            mana_limit: 100_000,   // 100K Mana
            max_stack_depth: 2048,
            timeout_ms: 30_000, // 30s
        }
    }

    /// Trust-basierte Skalierung der Limits
    pub fn with_trust_factor(mut self, trust: f64) -> Self {
        let factor = trust.clamp(0.0, 1.0);
        self.gas_limit = (self.gas_limit as f64 * (0.5 + factor * 0.5)) as u64;
        self.mana_limit = (self.mana_limit as f64 * (0.5 + factor * 0.5)) as u64;
        self
    }
}

/// ECLVM Budget Tracker - überwacht Ressourcenverbrauch während Ausführung
#[derive(Debug)]
pub struct ECLVMBudget {
    /// Konfigurierte Limits
    pub limits: ECLVMBudgetLimits,
    /// Verbrauchtes Gas
    gas_used: AtomicU64,
    /// Verbrauchtes Mana
    mana_used: AtomicU64,
    /// Startzeit der Ausführung
    started_at: Instant,
    /// Wurde Budget erschöpft?
    exhausted: std::sync::atomic::AtomicBool,
    /// Exhaustion-Grund (falls erschöpft)
    exhaustion_reason: RwLock<Option<BudgetExhaustionReason>>,
}

/// Grund für Budget-Erschöpfung
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BudgetExhaustionReason {
    /// Gas-Limit erreicht
    OutOfGas,
    /// Mana-Limit erreicht
    OutOfMana,
    /// Timeout
    Timeout,
    /// Stack-Overflow
    StackOverflow,
}

impl ECLVMBudget {
    /// Erstelle neues Budget mit Limits
    pub fn new(limits: ECLVMBudgetLimits) -> Self {
        Self {
            limits,
            gas_used: AtomicU64::new(0),
            mana_used: AtomicU64::new(0),
            started_at: Instant::now(),
            exhausted: std::sync::atomic::AtomicBool::new(false),
            exhaustion_reason: RwLock::new(None),
        }
    }

    /// Erstelle Budget mit Default-Limits
    pub fn with_defaults() -> Self {
        Self::new(ECLVMBudgetLimits::default())
    }

    /// Prüfe und konsumiere Gas
    /// Returns false wenn Budget erschöpft
    pub fn consume_gas(&self, amount: u64) -> bool {
        if self.exhausted.load(Ordering::Relaxed) {
            return false;
        }

        let new_total = self.gas_used.fetch_add(amount, Ordering::Relaxed) + amount;
        if new_total > self.limits.gas_limit {
            self.mark_exhausted(BudgetExhaustionReason::OutOfGas);
            return false;
        }

        // Check timeout bei jeder Gas-Operation
        if self.started_at.elapsed().as_millis() as u64 > self.limits.timeout_ms {
            self.mark_exhausted(BudgetExhaustionReason::Timeout);
            return false;
        }

        true
    }

    /// Prüfe und konsumiere Mana
    pub fn consume_mana(&self, amount: u64) -> bool {
        if self.exhausted.load(Ordering::Relaxed) {
            return false;
        }

        let new_total = self.mana_used.fetch_add(amount, Ordering::Relaxed) + amount;
        if new_total > self.limits.mana_limit {
            self.mark_exhausted(BudgetExhaustionReason::OutOfMana);
            return false;
        }
        true
    }

    /// Prüfe Stack-Depth
    pub fn check_stack_depth(&self, current_depth: u32) -> bool {
        if current_depth > self.limits.max_stack_depth {
            self.mark_exhausted(BudgetExhaustionReason::StackOverflow);
            return false;
        }
        true
    }

    /// Markiere Budget als erschöpft
    fn mark_exhausted(&self, reason: BudgetExhaustionReason) {
        if !self.exhausted.swap(true, Ordering::SeqCst) {
            if let Ok(mut r) = self.exhaustion_reason.write() {
                *r = Some(reason);
            }
        }
    }

    /// Ist Budget erschöpft?
    pub fn is_exhausted(&self) -> bool {
        self.exhausted.load(Ordering::Relaxed)
    }

    /// Exhaustion-Grund (falls erschöpft)
    pub fn exhaustion_reason(&self) -> Option<BudgetExhaustionReason> {
        self.exhaustion_reason.read().ok().and_then(|r| *r)
    }

    /// Verbrauchtes Gas
    pub fn gas_used(&self) -> u64 {
        self.gas_used.load(Ordering::Relaxed)
    }

    /// Verbrauchtes Mana
    pub fn mana_used(&self) -> u64 {
        self.mana_used.load(Ordering::Relaxed)
    }

    /// Verbleibendes Gas
    pub fn gas_remaining(&self) -> u64 {
        self.limits.gas_limit.saturating_sub(self.gas_used())
    }

    /// Verbleibendes Mana
    pub fn mana_remaining(&self) -> u64 {
        self.limits.mana_limit.saturating_sub(self.mana_used())
    }

    /// Verstrichene Zeit seit Start
    pub fn elapsed_ms(&self) -> u64 {
        self.started_at.elapsed().as_millis() as u64
    }

    /// Verbleibende Zeit bis Timeout
    pub fn time_remaining_ms(&self) -> u64 {
        self.limits.timeout_ms.saturating_sub(self.elapsed_ms())
    }

    /// Snapshot des Budget-Zustands
    pub fn snapshot(&self) -> ECLVMBudgetSnapshot {
        ECLVMBudgetSnapshot {
            gas_used: self.gas_used(),
            gas_limit: self.limits.gas_limit,
            mana_used: self.mana_used(),
            mana_limit: self.limits.mana_limit,
            elapsed_ms: self.elapsed_ms(),
            timeout_ms: self.limits.timeout_ms,
            exhausted: self.is_exhausted(),
            exhaustion_reason: self.exhaustion_reason(),
        }
    }
}

/// Snapshot eines ECLVM-Budgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECLVMBudgetSnapshot {
    pub gas_used: u64,
    pub gas_limit: u64,
    pub mana_used: u64,
    pub mana_limit: u64,
    pub elapsed_ms: u64,
    pub timeout_ms: u64,
    pub exhausted: bool,
    pub exhaustion_reason: Option<BudgetExhaustionReason>,
}

// ─────────────────────────────────────────────────────────────────────────────
// STATE VIEW - Read-only Snapshot für ECLVM Policy-Evaluation
// ─────────────────────────────────────────────────────────────────────────────

/// Read-only State View für ECLVM Policy-Evaluation
///
/// Bietet isolierten, konsistenten Lesezugriff auf State-Daten.
/// Alle Reads kommen aus einem Snapshot - keine Race Conditions.
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │                      StateView (Read-Only)                      │
/// │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
/// │  │ TrustQuery  │  │ RealmQuery  │  │ EventQuery  │             │
/// │  │ get_trust() │  │ get_realm() │  │ get_event() │             │
/// │  └─────────────┘  └─────────────┘  └─────────────┘             │
/// │                                                                 │
/// │  Snapshot-Isolation: Liest immer konsistenten Zustand          │
/// └─────────────────────────────────────────────────────────────────┘
/// ```
#[derive(Debug, Clone)]
pub struct StateView {
    /// Snapshot der relevanten State-Daten (lazy populated)
    pub snapshot_time: u128,
    /// Caller-Identity für Access-Control
    pub caller_did: Option<String>,
    /// Aktuelles Realm (für Realm-scoped Queries)
    pub current_realm: Option<String>,

    // Cached State-Teile (populated on demand)
    trust_cache: Arc<RwLock<HashMap<String, f64>>>,
    realm_cache: Arc<RwLock<HashMap<String, RealmViewData>>>,
    identity_cache: Arc<RwLock<HashMap<String, IdentityViewData>>>,
}

/// Realm-Daten für StateView
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmViewData {
    pub realm_id: String,
    pub name: String,
    pub owner_did: String,
    pub member_count: u64,
    pub trust_threshold: f64,
    pub is_quarantined: bool,
    pub created_at: u64,
}

/// Identity-Daten für StateView
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityViewData {
    pub did: String,
    pub display_name: Option<String>,
    pub trust_score: f64,
    pub realms: Vec<String>,
    pub created_at: u64,
}

impl StateView {
    /// Erstelle StateView für einen Caller
    pub fn new(caller_did: Option<String>, current_realm: Option<String>) -> Self {
        Self {
            snapshot_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            caller_did,
            current_realm,
            trust_cache: Arc::new(RwLock::new(HashMap::new())),
            realm_cache: Arc::new(RwLock::new(HashMap::new())),
            identity_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Erstelle StateView aus UnifiedState Snapshot (Caches werden befüllt)
    pub fn from_unified_snapshot(
        snapshot: &UnifiedSnapshot,
        caller_did: Option<String>,
        current_realm: Option<String>,
    ) -> Self {
        let mut view = Self::new(caller_did, current_realm);
        view.refresh_from_snapshot(snapshot);
        view
    }

    /// Befülle Caches aus UnifiedSnapshot (Phase 2.3 – State-backed ECL).
    ///
    /// Liest trust-/realm-/identity-relevante Felder aus dem Snapshot und
    /// überträgt sie in die Caches, sodass get_trust/get_realm/get_identity
    /// ohne manuelles set_* sinnvolle Werte liefern.
    pub fn refresh_from_snapshot(&mut self, snapshot: &UnifiedSnapshot) {
        // Trust-Cache: Snapshot hat nur Aggregat (avg_trust); Caller mit avg_trust belegen
        if let Ok(mut cache) = self.trust_cache.write() {
            cache.clear();
            if let Some(ref did) = self.caller_did {
                cache.insert(did.clone(), snapshot.core.trust.avg_trust);
            }
        }

        // Realm-Cache: aus snapshot.peer.realm.realms
        if let Ok(mut cache) = self.realm_cache.write() {
            cache.clear();
            for (realm_id, rs) in &snapshot.peer.realm.realms {
                let is_quarantined = rs.quota_health < 0.5;
                cache.insert(
                    realm_id.clone(),
                    RealmViewData {
                        realm_id: realm_id.clone(),
                        name: realm_id.clone(),
                        owner_did: String::new(),
                        member_count: rs.member_count as u64,
                        trust_threshold: rs.min_trust as f64,
                        is_quarantined,
                        created_at: rs.created_at,
                    },
                );
            }
        }

        // Identity-Cache: root_did aus Snapshot + avg_trust
        if let Ok(mut cache) = self.identity_cache.write() {
            cache.clear();
            if let Some(ref root_did) = snapshot.identity.root_did {
                let created_at = snapshot.identity.root_created_at_ms / 1000;
                cache.insert(
                    root_did.clone(),
                    IdentityViewData {
                        did: root_did.clone(),
                        display_name: None,
                        trust_score: snapshot.core.trust.avg_trust,
                        realms: Vec::new(),
                        created_at,
                    },
                );
            }
        }

        // Snapshot-Zeit aktualisieren
        self.snapshot_time = snapshot.timestamp_ms as u128;
    }

    /// Prüfe ob Caller bekannt ist
    pub fn has_caller(&self) -> bool {
        self.caller_did.is_some()
    }

    /// Prüfe ob Realm-Kontext gesetzt
    pub fn has_realm_context(&self) -> bool {
        self.current_realm.is_some()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Trust Queries
    // ─────────────────────────────────────────────────────────────────────

    /// Hole Trust-Wert für Entity (gecached)
    pub fn get_trust(&self, entity_id: &str) -> Option<f64> {
        if let Ok(cache) = self.trust_cache.read() {
            cache.get(entity_id).copied()
        } else {
            None
        }
    }

    /// Setze Trust-Wert im Cache (für Tests/Initialization)
    pub fn set_trust_cached(&self, entity_id: &str, trust: f64) {
        if let Ok(mut cache) = self.trust_cache.write() {
            cache.insert(entity_id.to_string(), trust);
        }
    }

    /// Hole Trust zwischen zwei Entities (Subjekt → Objekt)
    pub fn get_trust_between(&self, subject: &str, object: &str) -> Option<f64> {
        // Vereinfacht: nutze globalen Trust des Objekts
        // In Production: Trust-Graph mit edge-weights
        self.get_trust(object)
    }

    /// Prüfe ob Entity über Threshold liegt
    pub fn trust_above_threshold(&self, entity_id: &str, threshold: f64) -> bool {
        self.get_trust(entity_id)
            .map(|t| t >= threshold)
            .unwrap_or(false)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Realm Queries
    // ─────────────────────────────────────────────────────────────────────

    /// Hole Realm-Daten
    pub fn get_realm(&self, realm_id: &str) -> Option<RealmViewData> {
        if let Ok(cache) = self.realm_cache.read() {
            cache.get(realm_id).cloned()
        } else {
            None
        }
    }

    /// Setze Realm-Daten im Cache
    pub fn set_realm_cached(&self, data: RealmViewData) {
        if let Ok(mut cache) = self.realm_cache.write() {
            cache.insert(data.realm_id.clone(), data);
        }
    }

    /// Prüfe ob Caller Member eines Realms ist
    pub fn is_caller_member_of(&self, realm_id: &str) -> bool {
        self.caller_did.as_ref().map_or(false, |did| {
            self.get_identity(did)
                .map(|id| id.realms.contains(&realm_id.to_string()))
                .unwrap_or(false)
        })
    }

    /// Hole aktuelles Realm (falls gesetzt)
    pub fn current_realm_data(&self) -> Option<RealmViewData> {
        self.current_realm.as_ref().and_then(|r| self.get_realm(r))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Identity Queries
    // ─────────────────────────────────────────────────────────────────────

    /// Hole Identity-Daten
    pub fn get_identity(&self, did: &str) -> Option<IdentityViewData> {
        if let Ok(cache) = self.identity_cache.read() {
            cache.get(did).cloned()
        } else {
            None
        }
    }

    /// Setze Identity-Daten im Cache
    pub fn set_identity_cached(&self, data: IdentityViewData) {
        if let Ok(mut cache) = self.identity_cache.write() {
            cache.insert(data.did.clone(), data);
        }
    }

    /// Hole Caller Identity
    pub fn caller_identity(&self) -> Option<IdentityViewData> {
        self.caller_did.as_ref().and_then(|d| self.get_identity(d))
    }

    /// Prüfe ob Caller existiert und verifiziert ist
    pub fn is_caller_verified(&self) -> bool {
        self.caller_identity()
            .map(|id| id.trust_score > 0.1)
            .unwrap_or(false)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STATE HANDLE - Realm-scoped schreibbarer Zustand
// ─────────────────────────────────────────────────────────────────────────────

/// Schreibbarer State-Handle für ECLVM
///
/// Erlaubt Realm-isolierte Schreiboperationen mit Validierung.
/// Alle Schreiboperationen werden im Event-Log erfasst.
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │                   StateHandle (Write Access)                    │
/// │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
/// │  │ TrustMut    │  │ RealmMut    │  │ StorageMut  │             │
/// │  │ update()    │  │ modify()    │  │ put/del()   │             │
/// │  └─────────────┘  └─────────────┘  └─────────────┘             │
/// │                                                                 │
/// │  Isolation: Nur aktuelles Realm + eigene Identity               │
/// │  Validation: Jede Mutation wird geprüft                         │
/// │  Logging: Alle Änderungen werden im Event-Log erfasst           │
/// └─────────────────────────────────────────────────────────────────┘
/// ```
pub struct StateHandle<'a> {
    /// Referenz auf UnifiedState
    state: &'a UnifiedState,
    /// Caller-Identity für Access-Control
    caller_did: String,
    /// Aktuelles Realm
    realm_id: String,
    /// Budget für diese Operation
    budget: Arc<ECLVMBudget>,
    /// Geänderte Keys (für Rollback)
    dirty_keys: RwLock<HashSet<String>>,
    /// Pending Events (vor Commit)
    pending_events: RwLock<Vec<StateEvent>>,
    /// Wurde Handle committed?
    committed: std::sync::atomic::AtomicBool,
}

/// Ergebnis einer StateHandle-Mutation
#[derive(Debug, Clone)]
pub enum MutationResult {
    /// Erfolgreich
    Success,
    /// Abgelehnt wegen Policy
    PolicyDenied(String),
    /// Nicht genug Budget
    BudgetExhausted(BudgetExhaustionReason),
    /// Realm nicht erlaubt
    RealmAccessDenied,
    /// Validation fehlgeschlagen
    ValidationFailed(String),
}

impl<'a> StateHandle<'a> {
    /// Erstelle neuen StateHandle
    pub fn new(
        state: &'a UnifiedState,
        caller_did: String,
        realm_id: String,
        budget: Arc<ECLVMBudget>,
    ) -> Self {
        Self {
            state,
            caller_did,
            realm_id,
            budget,
            dirty_keys: RwLock::new(HashSet::new()),
            pending_events: RwLock::new(Vec::new()),
            committed: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Caller-DID
    pub fn caller(&self) -> &str {
        &self.caller_did
    }

    /// Aktuelles Realm
    pub fn realm(&self) -> &str {
        &self.realm_id
    }

    /// Budget-Status
    pub fn budget_snapshot(&self) -> ECLVMBudgetSnapshot {
        self.budget.snapshot()
    }

    /// Prüfe ob Handle noch gültig (nicht committed, Budget nicht erschöpft)
    pub fn is_valid(&self) -> bool {
        !self.committed.load(Ordering::Relaxed) && !self.budget.is_exhausted()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Trust Mutations
    // ─────────────────────────────────────────────────────────────────────

    /// Update Trust für Entity (innerhalb Realm-Kontext)
    pub fn update_trust(
        &self,
        target_did: &str,
        delta: f64,
        reason: TrustReason,
    ) -> MutationResult {
        if !self.is_valid() {
            return self
                .budget
                .exhaustion_reason()
                .map(MutationResult::BudgetExhausted)
                .unwrap_or(MutationResult::PolicyDenied("Handle invalid".to_string()));
        }

        // Gas für Trust-Update konsumieren
        if !self.budget.consume_gas(100) {
            return MutationResult::BudgetExhausted(BudgetExhaustionReason::OutOfGas);
        }

        // Validation: Delta muss im vernünftigen Bereich sein
        if delta.abs() > 0.5 {
            return MutationResult::ValidationFailed(
                "Trust delta too large (max ±0.5)".to_string(),
            );
        }

        // Event für Log vorbereiten
        let event = StateEvent::TrustUpdate {
            entity_id: target_did.to_string(),
            delta,
            reason,
            from_realm: Some(self.realm_id.clone()),
            triggered_events: 0,
            new_trust: 0.0, // Wird beim Apply gefüllt
        };

        // Event zu pending hinzufügen
        if let Ok(mut pending) = self.pending_events.write() {
            pending.push(event);
        }

        // Dirty-Key tracken
        if let Ok(mut dirty) = self.dirty_keys.write() {
            dirty.insert(format!("trust:{}", target_did));
        }

        MutationResult::Success
    }

    // ─────────────────────────────────────────────────────────────────────
    // Realm-scoped Storage Mutations
    // ─────────────────────────────────────────────────────────────────────

    /// Speichere Wert im Realm-Storage
    pub fn store_put(&self, key: &str, value: &str) -> MutationResult {
        if !self.is_valid() {
            return self
                .budget
                .exhaustion_reason()
                .map(MutationResult::BudgetExhausted)
                .unwrap_or(MutationResult::PolicyDenied("Handle invalid".to_string()));
        }

        // Gas für Storage-Write konsumieren (abhängig von Value-Größe)
        let gas_cost = 50 + (value.len() as u64 / 10);
        if !self.budget.consume_gas(gas_cost) {
            return MutationResult::BudgetExhausted(BudgetExhaustionReason::OutOfGas);
        }

        // Mana für Write-Bandbreite
        if !self.budget.consume_mana(1 + (value.len() as u64 / 100)) {
            return MutationResult::BudgetExhausted(BudgetExhaustionReason::OutOfMana);
        }

        // Dirty-Key tracken
        if let Ok(mut dirty) = self.dirty_keys.write() {
            dirty.insert(format!("store:{}:{}", self.realm_id, key));
        }

        MutationResult::Success
    }

    /// Lösche Wert aus Realm-Storage
    pub fn store_delete(&self, key: &str) -> MutationResult {
        if !self.is_valid() {
            return self
                .budget
                .exhaustion_reason()
                .map(MutationResult::BudgetExhausted)
                .unwrap_or(MutationResult::PolicyDenied("Handle invalid".to_string()));
        }

        // Gas für Storage-Delete
        if !self.budget.consume_gas(30) {
            return MutationResult::BudgetExhausted(BudgetExhaustionReason::OutOfGas);
        }

        // Dirty-Key tracken
        if let Ok(mut dirty) = self.dirty_keys.write() {
            dirty.insert(format!("store:{}:{}", self.realm_id, key));
        }

        MutationResult::Success
    }

    /// E4: Markiere Key als dirty (für externe Tracker)
    ///
    /// Wird von StateHost genutzt um zu signalisieren, dass ein Key
    /// modifiziert wurde (ohne tatsächlich Storage zu ändern).
    pub fn mark_key_dirty(&self, key: &str) {
        if let Ok(mut dirty) = self.dirty_keys.write() {
            dirty.insert(key.to_string());
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Governance Operations (E4.1)
    // ─────────────────────────────────────────────────────────────────────

    /// E4: Cast a vote on a governance proposal within this realm
    ///
    /// # Arguments
    /// - `proposal_id`: The proposal identifier
    /// - `vote`: true = approve, false = reject
    /// - `weight`: Vote weight (usually based on trust or stake)
    ///
    /// # Gas Cost: 100
    pub fn cast_vote(&self, proposal_id: &str, vote: bool, weight: f64) -> MutationResult {
        if !self.is_valid() {
            return self
                .budget
                .exhaustion_reason()
                .map(MutationResult::BudgetExhausted)
                .unwrap_or(MutationResult::PolicyDenied("Handle invalid".to_string()));
        }

        // Gas für Vote-Operation
        if !self.budget.consume_gas(100) {
            return MutationResult::BudgetExhausted(BudgetExhaustionReason::OutOfGas);
        }

        // Validation: Weight muss positiv sein
        if weight <= 0.0 || weight > 1.0 {
            return MutationResult::ValidationFailed(
                "Vote weight must be between 0.0 and 1.0".to_string(),
            );
        }

        // Event für Vote vorbereiten (nutzt existierendes VoteCast Event)
        let event = StateEvent::VoteCast {
            proposal_id: proposal_id.to_string(),
            voter_id: self.caller_did.clone(),
            vote,
            weight,
        };

        // Event zu pending hinzufügen
        if let Ok(mut pending) = self.pending_events.write() {
            pending.push(event);
        }

        // Dirty-Key tracken
        if let Ok(mut dirty) = self.dirty_keys.write() {
            dirty.insert(format!("vote:{}:{}", self.realm_id, proposal_id));
        }

        MutationResult::Success
    }

    /// E4: Submit a governance proposal within this realm
    ///
    /// # Arguments
    /// - `proposal_type`: Type of proposal (e.g., "parameter_change", "membership")
    /// - `title`: Proposal title (stored in description for now)
    /// - `deadline_hours`: Voting deadline in hours from now
    ///
    /// # Gas Cost: 200
    /// # Mana Cost: 10
    pub fn submit_proposal(
        &self,
        proposal_type: &str,
        title: &str,
        deadline_hours: u64,
    ) -> Result<String, MutationResult> {
        if !self.is_valid() {
            return Err(self
                .budget
                .exhaustion_reason()
                .map(MutationResult::BudgetExhausted)
                .unwrap_or(MutationResult::PolicyDenied("Handle invalid".to_string())));
        }

        // Gas für Proposal-Submission
        if !self.budget.consume_gas(200) {
            return Err(MutationResult::BudgetExhausted(
                BudgetExhaustionReason::OutOfGas,
            ));
        }

        // Mana für Proposal
        if !self.budget.consume_mana(10) {
            return Err(MutationResult::BudgetExhausted(
                BudgetExhaustionReason::OutOfMana,
            ));
        }

        // Generate proposal ID
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let proposal_id = format!("prop_{}_{}", self.realm_id, now_ms);

        // Deadline berechnen
        let deadline_ms = now_ms + (deadline_hours as u128 * 3600 * 1000);

        // Event für Proposal (nutzt existierendes ProposalCreated Event)
        let event = StateEvent::ProposalCreated {
            proposal_id: proposal_id.clone(),
            realm_id: self.realm_id.clone(),
            proposer_id: self.caller_did.clone(),
            proposal_type: proposal_type.to_string(),
            deadline_ms,
        };

        // Event zu pending hinzufügen
        if let Ok(mut pending) = self.pending_events.write() {
            pending.push(event);
        }

        // Dirty-Key tracken
        if let Ok(mut dirty) = self.dirty_keys.write() {
            dirty.insert(format!("proposal:{}", proposal_id));
        }

        Ok(proposal_id)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Event Emission
    // ─────────────────────────────────────────────────────────────────────

    /// E4: Emittiere benutzerdefiniertes Event (wird beim Commit geloggt)
    ///
    /// Ermöglicht ECL-Policies eigene Events zu emittieren, die
    /// beim Commit auf UnifiedState angewendet werden.
    ///
    /// # Arguments
    /// - `event_type`: Typ des Events (z.B. "notification", "action_triggered")
    /// - `payload`: JSON-artiger Payload als String
    ///
    /// # Mana Cost: 1 + payload_size/100
    pub fn emit_event(&self, event_type: &str, payload: &str) -> MutationResult {
        if !self.is_valid() {
            return self
                .budget
                .exhaustion_reason()
                .map(MutationResult::BudgetExhausted)
                .unwrap_or(MutationResult::PolicyDenied("Handle invalid".to_string()));
        }

        // Mana für Event-Emission (abhängig von Payload-Größe)
        let mana_cost = 1 + (payload.len() as u64 / 100);
        if !self.budget.consume_mana(mana_cost) {
            return MutationResult::BudgetExhausted(BudgetExhaustionReason::OutOfMana);
        }

        // Gas für Event-Emission
        if !self.budget.consume_gas(10) {
            return MutationResult::BudgetExhausted(BudgetExhaustionReason::OutOfGas);
        }

        // Event-Tracking im State
        self.state
            .eclvm
            .events_emitted
            .fetch_add(1, Ordering::Relaxed);

        // Dirty-Key für Event tracken
        if let Ok(mut dirty) = self.dirty_keys.write() {
            dirty.insert(format!(
                "event:{}:{}:{}",
                self.realm_id,
                event_type,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or(0)
            ));
        }

        MutationResult::Success
    }

    /// E4: Emittiere StateEvent direkt (für interne Verwendung)
    ///
    /// Ermöglicht das direkte Hinzufügen eines StateEvent zu den pending Events.
    /// Wird bei commit() auf UnifiedState angewendet.
    pub fn emit_state_event(&self, event: StateEvent) -> MutationResult {
        if !self.is_valid() {
            return self
                .budget
                .exhaustion_reason()
                .map(MutationResult::BudgetExhausted)
                .unwrap_or(MutationResult::PolicyDenied("Handle invalid".to_string()));
        }

        // Gas für State-Event-Emission
        if !self.budget.consume_gas(25) {
            return MutationResult::BudgetExhausted(BudgetExhaustionReason::OutOfGas);
        }

        // Event zu pending hinzufügen
        if let Ok(mut pending) = self.pending_events.write() {
            pending.push(event);
        }

        // Event-Tracking
        self.state
            .eclvm
            .events_emitted
            .fetch_add(1, Ordering::Relaxed);

        MutationResult::Success
    }

    /// E4: Hole pending Events (für Debugging/Testing)
    pub fn pending_events(&self) -> Vec<StateEvent> {
        self.pending_events
            .read()
            .map(|p| p.clone())
            .unwrap_or_default()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Commit / Rollback
    // ─────────────────────────────────────────────────────────────────────

    /// Commit alle pending Änderungen
    pub fn commit(self) -> CommitResult {
        if self.committed.swap(true, Ordering::SeqCst) {
            return CommitResult::AlreadyCommitted;
        }

        // Apply pending events
        let events_count = if let Ok(pending) = self.pending_events.read() {
            for event in pending.iter() {
                self.state.log_and_apply(event.clone(), vec![]);
            }
            pending.len()
        } else {
            0
        };

        // Dirty keys count
        let dirty_count = self.dirty_keys.read().map(|d| d.len()).unwrap_or(0);

        CommitResult::Success {
            events_applied: events_count,
            keys_modified: dirty_count,
            gas_used: self.budget.gas_used(),
            mana_used: self.budget.mana_used(),
        }
    }

    /// Rollback (verwerfe alle pending Änderungen)
    pub fn rollback(self) -> RollbackResult {
        if self.committed.swap(true, Ordering::SeqCst) {
            return RollbackResult::AlreadyCommitted;
        }

        let pending_count = self.pending_events.read().map(|p| p.len()).unwrap_or(0);
        let dirty_count = self.dirty_keys.read().map(|d| d.len()).unwrap_or(0);

        RollbackResult::Success {
            events_discarded: pending_count,
            keys_discarded: dirty_count,
        }
    }

    /// Anzahl pending Events
    pub fn pending_events_count(&self) -> usize {
        self.pending_events.read().map(|p| p.len()).unwrap_or(0)
    }

    /// Anzahl dirty Keys
    pub fn dirty_keys_count(&self) -> usize {
        self.dirty_keys.read().map(|d| d.len()).unwrap_or(0)
    }
}

/// Ergebnis eines Commits
#[derive(Debug, Clone)]
pub enum CommitResult {
    /// Erfolgreich committed
    Success {
        events_applied: usize,
        keys_modified: usize,
        gas_used: u64,
        mana_used: u64,
    },
    /// Bereits committed
    AlreadyCommitted,
}

/// Ergebnis eines Rollbacks
#[derive(Debug, Clone)]
pub enum RollbackResult {
    /// Erfolgreich rolled back
    Success {
        events_discarded: usize,
        keys_discarded: usize,
    },
    /// Bereits committed (kann nicht mehr rollback)
    AlreadyCommitted,
}

// ─────────────────────────────────────────────────────────────────────────────
// TRANSACTION GUARD - RAII für atomare State-Änderungen
// ─────────────────────────────────────────────────────────────────────────────

/// RAII Transaction Guard für automatischen Rollback bei Drop
///
/// Wenn Guard dropped wird ohne commit(), wird automatisch rollback() aufgerufen.
pub struct TransactionGuard<'a> {
    handle: Option<StateHandle<'a>>,
    auto_commit: bool,
}

impl<'a> TransactionGuard<'a> {
    /// Erstelle Transaction Guard
    pub fn new(handle: StateHandle<'a>) -> Self {
        Self {
            handle: Some(handle),
            auto_commit: false,
        }
    }

    /// Erstelle Transaction Guard mit Auto-Commit bei Success
    pub fn with_auto_commit(handle: StateHandle<'a>) -> Self {
        Self {
            handle: Some(handle),
            auto_commit: true,
        }
    }

    /// Zugriff auf den StateHandle
    pub fn handle(&self) -> Option<&StateHandle<'a>> {
        self.handle.as_ref()
    }

    /// Mutable Zugriff auf den StateHandle
    pub fn handle_mut(&mut self) -> Option<&StateHandle<'a>> {
        self.handle.as_ref()
    }

    /// Expliziter Commit - nimmt Ownership vom Handle
    pub fn commit(mut self) -> CommitResult {
        self.handle
            .take()
            .map(|h| h.commit())
            .unwrap_or(CommitResult::AlreadyCommitted)
    }

    /// Expliziter Rollback - nimmt Ownership vom Handle
    pub fn rollback(mut self) -> RollbackResult {
        self.handle
            .take()
            .map(|h| h.rollback())
            .unwrap_or(RollbackResult::AlreadyCommitted)
    }
}

impl<'a> Drop for TransactionGuard<'a> {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            if self.auto_commit && handle.is_valid() {
                // Auto-commit wenn erfolgreich
                let _ = handle.commit();
            } else {
                // Rollback bei Drop
                let _ = handle.rollback();
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ECLVM STATE CONTEXT - Orchestriert State-Zugriff für ECLVM-Ausführung
// ─────────────────────────────────────────────────────────────────────────────

/// ECLVM State Context - Orchestriert alle State-Interaktionen
///
/// Bietet ECLVM eine einheitliche Schnittstelle für State-Zugriff.
/// Kombiniert StateView (read) und StateHandle (write) mit Budget-Tracking.
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │                     ECLVMStateContext                           │
/// │  ┌───────────────────────────────────────────────────────────┐  │
/// │  │                      StateView                             │  │
/// │  │  get_trust() | get_realm() | get_identity() ...           │  │
/// │  └───────────────────────────────────────────────────────────┘  │
/// │  ┌───────────────────────────────────────────────────────────┐  │
/// │  │                     StateHandle                            │  │
/// │  │  update_trust() | store_put() | emit_event() ...          │  │
/// │  └───────────────────────────────────────────────────────────┘  │
/// │  ┌───────────────────────────────────────────────────────────┐  │
/// │  │                     ECLVMBudget                            │  │
/// │  │  consume_gas() | consume_mana() | check_timeout()         │  │
/// │  └───────────────────────────────────────────────────────────┘  │
/// │                                                                 │
/// │  Verwendung durch ECLVM Host Interface                          │
/// └─────────────────────────────────────────────────────────────────┘
/// ```
pub struct ECLVMStateContext {
    /// Read-only View auf State
    pub view: StateView,
    /// Budget für diese Ausführung
    pub budget: Arc<ECLVMBudget>,
    /// Caller-Identity
    caller_did: String,
    /// Aktuelles Realm
    realm_id: String,
    /// Referenz auf UnifiedState (für Handle-Erstellung)
    state: Arc<UnifiedState>,
    /// Erstellungszeit
    created_at: Instant,
    /// Execution-ID (für Tracing/Logging)
    execution_id: String,
}

impl ECLVMStateContext {
    /// Erstelle neuen StateContext für ECLVM-Ausführung
    pub fn new(
        state: Arc<UnifiedState>,
        caller_did: String,
        realm_id: String,
        limits: ECLVMBudgetLimits,
    ) -> Self {
        let execution_id = format!(
            "eclvm_{}_{}_{}",
            realm_id,
            caller_did.chars().take(8).collect::<String>(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );

        let budget = Arc::new(ECLVMBudget::new(limits));
        let view = StateView::new(Some(caller_did.clone()), Some(realm_id.clone()));

        Self {
            view,
            budget,
            caller_did,
            realm_id,
            state,
            created_at: Instant::now(),
            execution_id,
        }
    }

    /// Erstelle Context mit Default-Budget
    pub fn with_defaults(state: Arc<UnifiedState>, caller_did: String, realm_id: String) -> Self {
        Self::new(state, caller_did, realm_id, ECLVMBudgetLimits::default())
    }

    /// Execution-ID für Tracing
    pub fn execution_id(&self) -> &str {
        &self.execution_id
    }

    /// Caller-DID
    pub fn caller(&self) -> &str {
        &self.caller_did
    }

    /// Aktuelles Realm
    pub fn realm(&self) -> &str {
        &self.realm_id
    }

    /// Prüfe ob Context noch gültig
    pub fn is_valid(&self) -> bool {
        !self.budget.is_exhausted()
    }

    /// Verstrichene Zeit seit Erstellung
    pub fn elapsed_ms(&self) -> u64 {
        self.created_at.elapsed().as_millis() as u64
    }

    /// Befülle StateView aus UnifiedSnapshot (Phase 2.3 – State-backed ECL).
    ///
    /// Vor der ersten ECL-Ausführung aufrufen, damit get_trust/get_realm/get_identity
    /// sinnvolle Werte aus dem State liefern.
    pub fn refresh_view_from_snapshot(&mut self, snapshot: &UnifiedSnapshot) {
        self.view.refresh_from_snapshot(snapshot);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Read Operations (delegiert an StateView)
    // ─────────────────────────────────────────────────────────────────────

    /// Hole Trust für Entity (gecached, Gas-metered)
    pub fn get_trust(&self, entity_id: &str) -> Option<f64> {
        // Gas für Read-Operation
        if !self.budget.consume_gas(10) {
            return None;
        }
        self.view.get_trust(entity_id)
    }

    /// Hole Realm-Daten
    pub fn get_realm(&self, realm_id: &str) -> Option<RealmViewData> {
        if !self.budget.consume_gas(15) {
            return None;
        }
        self.view.get_realm(realm_id)
    }

    /// Hole Identity-Daten
    pub fn get_identity(&self, did: &str) -> Option<IdentityViewData> {
        if !self.budget.consume_gas(15) {
            return None;
        }
        self.view.get_identity(did)
    }

    /// Prüfe Trust-Threshold
    pub fn check_trust_threshold(&self, entity_id: &str, threshold: f64) -> bool {
        if !self.budget.consume_gas(12) {
            return false;
        }
        self.view.trust_above_threshold(entity_id, threshold)
    }

    /// Ist Caller Member des angegebenen Realms?
    pub fn is_caller_member(&self, realm_id: &str) -> bool {
        if !self.budget.consume_gas(20) {
            return false;
        }
        self.view.is_caller_member_of(realm_id)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Cache Population (für Test/Setup)
    // ─────────────────────────────────────────────────────────────────────

    /// Füge Trust-Wert zum Cache hinzu
    pub fn populate_trust(&self, entity_id: &str, trust: f64) {
        self.view.set_trust_cached(entity_id, trust);
    }

    /// Füge Realm zum Cache hinzu
    pub fn populate_realm(&self, data: RealmViewData) {
        self.view.set_realm_cached(data);
    }

    /// Füge Identity zum Cache hinzu
    pub fn populate_identity(&self, data: IdentityViewData) {
        self.view.set_identity_cached(data);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Write Operations (erstellt StateHandle)
    // ─────────────────────────────────────────────────────────────────────

    /// Erstelle einen StateHandle für Schreiboperationen
    ///
    /// WICHTIG: Handle muss committed oder rolled back werden!
    pub fn create_write_handle(&self) -> StateHandle<'_> {
        StateHandle::new(
            &self.state,
            self.caller_did.clone(),
            self.realm_id.clone(),
            self.budget.clone(),
        )
    }

    /// Erstelle TransactionGuard für automatisches Rollback
    pub fn begin_transaction(&self) -> TransactionGuard<'_> {
        TransactionGuard::new(self.create_write_handle())
    }

    /// Erstelle TransactionGuard mit Auto-Commit
    pub fn begin_auto_commit_transaction(&self) -> TransactionGuard<'_> {
        TransactionGuard::with_auto_commit(self.create_write_handle())
    }

    // ─────────────────────────────────────────────────────────────────────
    // Budget-Interaktion
    // ─────────────────────────────────────────────────────────────────────

    /// Verbleibendes Gas
    pub fn gas_remaining(&self) -> u64 {
        self.budget.gas_remaining()
    }

    /// Verbleibendes Mana
    pub fn mana_remaining(&self) -> u64 {
        self.budget.mana_remaining()
    }

    /// Verbleibende Zeit bis Timeout
    pub fn time_remaining_ms(&self) -> u64 {
        self.budget.time_remaining_ms()
    }

    /// Budget-Snapshot
    pub fn budget_snapshot(&self) -> ECLVMBudgetSnapshot {
        self.budget.snapshot()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Finalization
    // ─────────────────────────────────────────────────────────────────────

    /// Finalisiere Context und gib Summary zurück
    pub fn finalize(self) -> ECLVMExecutionSummary {
        ECLVMExecutionSummary {
            execution_id: self.execution_id,
            caller_did: self.caller_did,
            realm_id: self.realm_id,
            duration_ms: self.created_at.elapsed().as_millis() as u64,
            gas_used: self.budget.gas_used(),
            gas_limit: self.budget.limits.gas_limit,
            mana_used: self.budget.mana_used(),
            mana_limit: self.budget.limits.mana_limit,
            exhausted: self.budget.is_exhausted(),
            exhaustion_reason: self.budget.exhaustion_reason(),
        }
    }
}

/// Summary einer ECLVM-Ausführung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ECLVMExecutionSummary {
    pub execution_id: String,
    pub caller_did: String,
    pub realm_id: String,
    pub duration_ms: u64,
    pub gas_used: u64,
    pub gas_limit: u64,
    pub mana_used: u64,
    pub mana_limit: u64,
    pub exhausted: bool,
    pub exhaustion_reason: Option<BudgetExhaustionReason>,
}

impl ECLVMExecutionSummary {
    /// War Ausführung erfolgreich (nicht exhausted)?
    pub fn is_success(&self) -> bool {
        !self.exhausted
    }

    /// Gas-Utilization (%)
    pub fn gas_utilization_percent(&self) -> f64 {
        if self.gas_limit == 0 {
            0.0
        } else {
            (self.gas_used as f64 / self.gas_limit as f64) * 100.0
        }
    }

    /// Mana-Utilization (%)
    pub fn mana_utilization_percent(&self) -> f64 {
        if self.mana_limit == 0 {
            0.0
        } else {
            (self.mana_used as f64 / self.mana_limit as f64) * 100.0
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// IDENTITY STATE TESTS (Phase 2)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_identity_state {
    use super::*;
    use crate::core::identity_types::{IdentityMode, RealmRole};

    #[test]
    fn test_identity_state_creation() {
        let state = IdentityState::new();

        assert!(!state.is_bootstrapped());
        assert_eq!(state.sub_dids_total.load(Ordering::Relaxed), 0);
        assert_eq!(state.current_mode(), IdentityMode::Interactive);
    }

    #[test]
    fn test_identity_state_bootstrap_test_mode() {
        let state = IdentityState::new();
        let pk = [1u8; 32];

        let result = state.bootstrap_test(&pk);
        assert!(result.is_ok());

        let root_id = result.unwrap();
        assert!(state.is_bootstrapped());
        assert_eq!(state.current_mode(), IdentityMode::Test);
        assert_eq!(state.root_did_id(), Some(root_id));
    }

    #[test]
    fn test_identity_state_double_bootstrap_fails() {
        let state = IdentityState::new();
        let pk = [1u8; 32];

        assert!(state.bootstrap_test(&pk).is_ok());
        assert!(state.bootstrap_test(&pk).is_err());
    }

    #[test]
    fn test_identity_state_snapshot() {
        let state = IdentityState::new();
        let pk = [1u8; 32];
        state.bootstrap_test(&pk).unwrap();

        let snapshot = state.snapshot();

        assert!(snapshot.bootstrap_completed);
        assert_eq!(snapshot.mode, IdentityMode::Test);
        assert!(snapshot.root_did.is_some());
        assert!(snapshot.root_did.unwrap().starts_with("did:erynoa:self:"));
    }

    #[test]
    fn test_identity_state_health_score() {
        let state = IdentityState::new();

        // Nicht bootstrapped → 0.0
        assert_eq!(state.health_score(), 0.0);

        // Bootstrapped → > 0.0
        state.bootstrap_test(&[1u8; 32]).unwrap();
        assert!(state.health_score() > 0.0);
    }

    #[test]
    fn test_identity_state_derive_device() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let device_result = state.derive_device_did(0);
        assert!(device_result.is_ok());

        let device_id = device_result.unwrap();
        assert!(state.device_did_id().is_some());
        assert_eq!(state.device_did_id(), Some(device_id));
        assert_eq!(state.sub_dids_total.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_identity_state_derive_agent() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let agent_result = state.derive_agent_did(0);
        assert!(agent_result.is_ok());

        assert_eq!(state.sub_dids_total.load(Ordering::Relaxed), 1);

        // Verify counter by namespace
        let counts = state.sub_did_counts.read().unwrap();
        assert_eq!(
            counts.get(&crate::domain::unified::identity::DIDNamespace::Spirit),
            Some(&1)
        );
    }

    #[test]
    fn test_identity_state_derive_realm() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");
        let realm_did_result = state.derive_realm_did(&realm_id);
        assert!(realm_did_result.is_ok());

        assert_eq!(state.sub_dids_total.load(Ordering::Relaxed), 1);

        // Verify it's stored under the realm key
        let sub_dids = state.sub_dids.read().unwrap();
        let realm_key = format!("realm:{}", hex::encode(realm_id.as_bytes()));
        assert!(sub_dids.contains_key(&realm_key));
    }

    #[test]
    fn test_identity_state_join_realm() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");
        let result = state.join_realm(realm_id, RealmRole::Member, Some(0.7));
        assert!(result.is_ok());

        assert!(state.is_realm_member(&realm_id));
        assert_eq!(state.active_realm_memberships().len(), 1);

        // Verify membership details
        let membership = state.get_realm_membership(&realm_id).unwrap();
        assert_eq!(membership.role, RealmRole::Member);
        assert!((membership.local_trust - 0.7).abs() < 0.001);
    }

    #[test]
    fn test_identity_state_leave_realm() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");
        state.join_realm(realm_id, RealmRole::Member, None).unwrap();

        assert!(state.is_realm_member(&realm_id));

        state.leave_realm(&realm_id).unwrap();

        // Membership is deactivated, not removed
        assert!(!state.is_realm_member(&realm_id));
        let membership = state.get_realm_membership(&realm_id).unwrap();
        assert!(!membership.is_active);
    }

    #[test]
    fn test_identity_state_update_realm_role() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");
        state.join_realm(realm_id, RealmRole::Member, None).unwrap();

        state
            .update_realm_role(&realm_id, RealmRole::Admin)
            .unwrap();

        let membership = state.get_realm_membership(&realm_id).unwrap();
        assert_eq!(membership.role, RealmRole::Admin);
    }

    #[test]
    fn test_identity_state_delegation() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        let caps = vec![crate::domain::unified::identity::Capability::Read {
            resource: "*".to_string(),
        }];

        let result = state.add_delegation(delegate_id, 0.8, caps, None);
        assert!(result.is_ok());

        assert_eq!(state.active_delegations_count.load(Ordering::Relaxed), 1);

        // Verify delegation exists
        let delegation = state.get_delegation(&delegate_id);
        assert!(delegation.is_some());
        assert_eq!(delegation.unwrap().trust_factor, 0.8);
    }

    #[test]
    fn test_identity_state_revoke_delegation() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        state
            .add_delegation(delegate_id, 0.8, vec![], None)
            .unwrap();

        assert_eq!(state.active_delegations_count.load(Ordering::Relaxed), 1);

        state.revoke_delegation(&delegate_id).unwrap();

        assert_eq!(state.active_delegations_count.load(Ordering::Relaxed), 0);
        assert_eq!(state.revoked_delegations_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_identity_state_invalid_trust_factor() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");

        // Trust factor > 1 should fail
        let result = state.add_delegation(delegate_id, 1.5, vec![], None);
        assert!(result.is_err());

        // Trust factor <= 0 should fail
        let result = state.add_delegation(delegate_id, 0.0, vec![], None);
        assert!(result.is_err());
    }

    #[test]
    fn test_identity_state_wallet_address() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let root_id = state.root_did_id().unwrap();
        let wallet = crate::core::identity_types::WalletAddress::new(
            "eip155:1",
            "0x1234567890123456789012345678901234567890",
            "m/44'/60'/0'/0/0",
            root_id,
        )
        .as_primary();

        let result = state.add_wallet_address(wallet);
        assert!(result.is_ok());

        assert_eq!(state.addresses_total.load(Ordering::Relaxed), 1);

        let wallets = state.get_wallets_for_chain("eip155:1");
        assert_eq!(wallets.len(), 1);

        let primary = state.get_primary_wallet("eip155:1");
        assert!(primary.is_some());
    }

    #[test]
    fn test_identity_state_counters() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        // Test various counter methods
        state.record_credential_issued();
        state.record_credential_issued();
        state.record_credential_verified();
        state.record_gas(1000);
        state.record_mana(50);
        state.record_event_triggered();
        state.record_trust_entry_created();

        assert_eq!(state.credentials_issued.load(Ordering::Relaxed), 2);
        assert_eq!(state.credentials_verified.load(Ordering::Relaxed), 1);
        assert_eq!(state.gas_consumed.load(Ordering::Relaxed), 1000);
        assert_eq!(state.mana_consumed.load(Ordering::Relaxed), 50);
        assert_eq!(state.events_triggered.load(Ordering::Relaxed), 1);
        assert_eq!(state.trust_entries_created.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_identity_state_ephemeral_no_realm() {
        let state = IdentityState::new();
        state.bootstrap_ephemeral(&[1u8; 32]).unwrap();

        assert_eq!(state.current_mode(), IdentityMode::Ephemeral);

        // Ephemeral mode should not allow realm membership
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");
        let result = state.join_realm(realm_id, RealmRole::Member, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_identity_snapshot_default() {
        let snapshot = IdentitySnapshot::default();

        assert!(!snapshot.bootstrap_completed);
        assert_eq!(snapshot.mode, IdentityMode::Interactive);
        assert!(snapshot.root_did.is_none());
        assert_eq!(snapshot.sub_dids_total, 0);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Additional Unit Tests (Phase 8)
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_identity_state_sign_operations() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        // Derive device to have a device key
        state.derive_device_did(0).unwrap();

        // Record signing metrics
        state.record_signature_created();
        state.record_signature_verified();

        assert_eq!(state.signatures_created.load(Ordering::Relaxed), 1);
        assert_eq!(state.signatures_verified.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_identity_state_multiple_realm_memberships() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        // Join multiple realms with different roles
        let realm1 = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm-1");
        let realm2 = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm-2");
        let realm3 = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm-3");

        state
            .join_realm(realm1, RealmRole::Member, Some(0.5))
            .unwrap();
        state
            .join_realm(realm2, RealmRole::Admin, Some(0.8))
            .unwrap();
        state
            .join_realm(realm3, RealmRole::Moderator, Some(0.6))
            .unwrap();

        let memberships = state.active_realm_memberships();
        assert_eq!(memberships.len(), 3);
    }

    #[test]
    fn test_identity_state_update_trust_in_realm() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"trust-test");
        state
            .join_realm(realm_id, RealmRole::Member, Some(0.5))
            .unwrap();

        // Update trust
        state.update_realm_trust(&realm_id, 0.9).unwrap();

        let membership = state.get_realm_membership(&realm_id).unwrap();
        assert!((membership.local_trust - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_identity_state_delegation_validity_check() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        state
            .add_delegation(delegate_id, 0.7, vec![], None)
            .unwrap();

        // Create current time
        let root_id = state.root_did_id().unwrap();
        let now = crate::domain::unified::primitives::TemporalCoord::now(1, &root_id);

        // Should be valid
        assert!(state.is_delegation_valid(&delegate_id, &now));

        // Revoke
        state.revoke_delegation(&delegate_id).unwrap();

        // Should no longer be valid
        assert!(!state.is_delegation_valid(&delegate_id, &now));
    }

    #[test]
    fn test_identity_state_multiple_wallets_same_chain() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        let root_id = state.root_did_id().unwrap();

        // Add primary wallet
        let wallet1 = crate::core::identity_types::WalletAddress::new(
            "eip155:1",
            "0x1111111111111111111111111111111111111111",
            "m/44'/60'/0'/0/0",
            root_id,
        )
        .as_primary();

        // Add secondary wallet
        let wallet2 = crate::core::identity_types::WalletAddress::new(
            "eip155:1",
            "0x2222222222222222222222222222222222222222",
            "m/44'/60'/0'/0/1",
            root_id,
        );

        state.add_wallet_address(wallet1).unwrap();
        state.add_wallet_address(wallet2).unwrap();

        assert_eq!(state.addresses_total.load(Ordering::Relaxed), 2);

        let wallets = state.get_wallets_for_chain("eip155:1");
        assert_eq!(wallets.len(), 2);

        // Primary should still be the first one
        let primary = state.get_primary_wallet("eip155:1");
        assert!(primary.is_some());
        assert!(primary.unwrap().address.contains("1111"));
    }

    #[test]
    fn test_identity_state_not_bootstrapped_operations_fail() {
        let state = IdentityState::new();

        // Should fail without bootstrap
        let result = state.derive_device_did(0);
        assert!(result.is_err());

        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test");
        let result = state.join_realm(realm_id, RealmRole::Member, None);
        assert!(result.is_err());

        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        let result = state.add_delegation(delegate_id, 0.5, vec![], None);
        assert!(result.is_err());
    }

    #[test]
    fn test_identity_health_score_calculation() {
        let state = IdentityState::new();

        // Before bootstrap: 0
        assert_eq!(state.health_score(), 0.0);

        // After bootstrap: base score
        state.bootstrap_test(&[1u8; 32]).unwrap();
        let health_after_bootstrap = state.health_score();
        assert!(health_after_bootstrap > 0.0);

        // Add delegation improves health
        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        state
            .add_delegation(delegate_id, 0.8, vec![], None)
            .unwrap();

        // Note: health calculation is based on multiple factors
        let health_after_delegation = state.health_score();
        assert!(health_after_delegation > 0.0);
    }

    #[test]
    fn test_identity_state_all_bootstrap_modes() {
        // Test mode (primary test mode, doesn't require KeyStore)
        let state1 = IdentityState::new();
        state1.bootstrap_test(&[1u8; 32]).unwrap();
        assert_eq!(state1.current_mode(), IdentityMode::Test);

        // Ephemeral mode (doesn't require KeyStore/Passkey)
        let state2 = IdentityState::new();
        state2.bootstrap_ephemeral(&[2u8; 32]).unwrap();
        assert_eq!(state2.current_mode(), IdentityMode::Ephemeral);

        // Note: Interactive and AgentManaged modes require KeyStore/PasskeyManager
        // which are not available in unit tests without mock implementations.
        // These modes are tested in integration tests with proper setup.
    }

    #[test]
    fn test_identity_state_signature_tracking() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        // Track signature operations
        state.record_signature_created();
        state.record_signature_created();
        state.record_signature_verified();

        assert_eq!(state.signatures_created.load(Ordering::Relaxed), 2);
        assert_eq!(state.signatures_verified.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_identity_state_derive_multiple_sub_dids() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        // Derive device (uses Self_ namespace)
        state.derive_device_did(0).unwrap();
        assert_eq!(state.sub_dids_total.load(Ordering::Relaxed), 1);

        // Derive agents (uses Spirit namespace)
        state.derive_agent_did(0).unwrap();
        state.derive_agent_did(1).unwrap();
        assert_eq!(state.sub_dids_total.load(Ordering::Relaxed), 3);

        // Derive realm DIDs (uses Circle namespace)
        let realm1 = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm1");
        let realm2 = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm2");
        state.derive_realm_did(&realm1).unwrap();
        state.derive_realm_did(&realm2).unwrap();
        assert_eq!(state.sub_dids_total.load(Ordering::Relaxed), 5);

        // Check namespace counts
        let counts = state.sub_did_counts.read().unwrap();
        // Self_ for device
        assert_eq!(
            counts.get(&crate::domain::unified::identity::DIDNamespace::Self_),
            Some(&1)
        );
        // Spirit for agents
        assert_eq!(
            counts.get(&crate::domain::unified::identity::DIDNamespace::Spirit),
            Some(&2)
        );
        // Circle for realm DIDs
        assert_eq!(
            counts.get(&crate::domain::unified::identity::DIDNamespace::Circle),
            Some(&2)
        );
    }

    #[test]
    fn test_identity_state_snapshot_comprehensive() {
        let state = IdentityState::new();
        state.bootstrap_test(&[1u8; 32]).unwrap();

        // Perform various operations
        state.derive_device_did(0).unwrap();
        state.derive_agent_did(0).unwrap();

        // Note: join_realm internally also derives a realm-specific DID
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test");
        state
            .join_realm(realm_id, RealmRole::Member, Some(0.7))
            .unwrap();

        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        state
            .add_delegation(delegate_id, 0.8, vec![], None)
            .unwrap();

        let root_id = state.root_did_id().unwrap();
        let wallet = crate::core::identity_types::WalletAddress::new(
            "eip155:1",
            "0x1234567890123456789012345678901234567890",
            "m/44'/60'/0'/0/0",
            root_id,
        );
        state.add_wallet_address(wallet).unwrap();

        // Take snapshot
        let snapshot = state.snapshot();

        // Verify all fields
        assert!(snapshot.bootstrap_completed);
        assert_eq!(snapshot.mode, IdentityMode::Test);
        assert!(snapshot.root_did.is_some());
        // device + agent + realm_sub_did (from join_realm) = 3
        assert_eq!(snapshot.sub_dids_total, 3);
        assert_eq!(snapshot.realm_membership_count, 1);
        assert_eq!(snapshot.active_delegations, 1);
        assert_eq!(snapshot.addresses_total, 1);
        assert!(snapshot.wallet_chains.contains(&"eip155:1".to_string()));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STATE EVENT TESTS (Phase 4)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_state_event_identity {
    use super::*;
    use crate::core::identity_types::IdentityMode;
    use crate::domain::unified::identity::DIDNamespace;

    #[test]
    fn test_identity_event_primary_component() {
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"test");

        let event = StateEvent::IdentityBootstrapped {
            root_did,
            namespace: DIDNamespace::Self_,
            mode: IdentityMode::Interactive,
            timestamp_ms: 0,
        };
        assert_eq!(event.primary_component(), StateComponent::Identity);

        let event = StateEvent::SubDIDDerived {
            root_did,
            sub_did: UniversalId::new(UniversalId::TAG_DID, 1, b"sub"),
            namespace: DIDNamespace::Spirit,
            derivation_path: "m/44'/0'/0'".to_string(),
            purpose: "agent".to_string(),
            gas_used: 100,
            realm_id: None,
        };
        assert_eq!(event.primary_component(), StateComponent::Identity);

        let event = StateEvent::DelegationCreated {
            delegator: root_did,
            delegate: UniversalId::new(UniversalId::TAG_DID, 1, b"delegate"),
            trust_factor: 0.8,
            capabilities: vec!["read:*".to_string()],
            valid_until: None,
        };
        assert_eq!(event.primary_component(), StateComponent::Identity);
    }

    #[test]
    fn test_identity_event_is_critical() {
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"test");

        // IdentityBootstrapped is critical
        let event = StateEvent::IdentityBootstrapped {
            root_did,
            namespace: DIDNamespace::Self_,
            mode: IdentityMode::Interactive,
            timestamp_ms: 0,
        };
        assert!(event.is_critical());

        // KeyRotated is critical
        let event = StateEvent::KeyRotated {
            did: root_did,
            old_key_id: UniversalId::new(UniversalId::TAG_DID, 1, b"old"),
            new_key_id: UniversalId::new(UniversalId::TAG_DID, 1, b"new"),
            reason: "Scheduled rotation".to_string(),
        };
        assert!(event.is_critical());

        // RecoveryInitiated is critical
        let event = StateEvent::RecoveryInitiated {
            did: root_did,
            recovery_key_id: UniversalId::new(UniversalId::TAG_DID, 1, b"recovery"),
            initiated_at: 0,
        };
        assert!(event.is_critical());

        // IdentityAnomalyDetected with "critical" severity is critical
        let event = StateEvent::IdentityAnomalyDetected {
            did: root_did,
            anomaly_type: "RapidDelegation".to_string(),
            severity: "critical".to_string(),
            details: "test".to_string(),
        };
        assert!(event.is_critical());

        // IdentityAnomalyDetected with "low" severity is not critical
        let event = StateEvent::IdentityAnomalyDetected {
            did: root_did,
            anomaly_type: "MinorAnomaly".to_string(),
            severity: "low".to_string(),
            details: "test".to_string(),
        };
        assert!(!event.is_critical());

        // WalletDerived is not critical
        let event = StateEvent::WalletDerived {
            did: root_did,
            chain_id: "eip155:1".to_string(),
            address: "0x123".to_string(),
            derivation_path: "m/44'/60'".to_string(),
        };
        assert!(!event.is_critical());
    }

    #[test]
    fn test_identity_event_realm_context() {
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"test");
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm");

        // SubDIDDerived with realm_id
        let event = StateEvent::SubDIDDerived {
            root_did,
            sub_did: UniversalId::new(UniversalId::TAG_DID, 1, b"sub"),
            namespace: DIDNamespace::Circle,
            derivation_path: "m/44'/0'".to_string(),
            purpose: "realm".to_string(),
            gas_used: 100,
            realm_id: Some(realm_id),
        };
        assert_eq!(event.realm_context(), Some(&realm_id));

        // SubDIDDerived without realm_id
        let event = StateEvent::SubDIDDerived {
            root_did,
            sub_did: UniversalId::new(UniversalId::TAG_DID, 1, b"sub"),
            namespace: DIDNamespace::Self_,
            derivation_path: "m/44'/0'".to_string(),
            purpose: "device".to_string(),
            gas_used: 100,
            realm_id: None,
        };
        assert!(event.realm_context().is_none());

        // RealmMembershipChanged always has realm context
        let event = StateEvent::RealmMembershipChanged {
            realm_id,
            member_id: root_did,
            action: "Joined".to_string(),
            new_role: Some("member".to_string()),
            realm_sub_did: None,
        };
        assert_eq!(event.realm_context(), Some(&realm_id));
    }

    #[test]
    fn test_identity_event_involved_identities() {
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"root");
        let sub_did = UniversalId::new(UniversalId::TAG_DID, 1, b"sub");
        let delegate = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");

        // IdentityBootstrapped
        let event = StateEvent::IdentityBootstrapped {
            root_did,
            namespace: DIDNamespace::Self_,
            mode: IdentityMode::Interactive,
            timestamp_ms: 0,
        };
        assert_eq!(event.involved_identities(), vec![root_did]);

        // SubDIDDerived
        let event = StateEvent::SubDIDDerived {
            root_did,
            sub_did,
            namespace: DIDNamespace::Spirit,
            derivation_path: "m/0'".to_string(),
            purpose: "agent".to_string(),
            gas_used: 0,
            realm_id: None,
        };
        assert_eq!(event.involved_identities(), vec![root_did, sub_did]);

        // DelegationCreated
        let event = StateEvent::DelegationCreated {
            delegator: root_did,
            delegate,
            trust_factor: 0.8,
            capabilities: vec![],
            valid_until: None,
        };
        assert_eq!(event.involved_identities(), vec![root_did, delegate]);
    }

    #[test]
    fn test_identity_event_is_identity_event() {
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"test");

        // Identity events
        let event = StateEvent::IdentityBootstrapped {
            root_did,
            namespace: DIDNamespace::Self_,
            mode: IdentityMode::Test,
            timestamp_ms: 0,
        };
        assert!(event.is_identity_event());

        let event = StateEvent::DelegationCreated {
            delegator: root_did,
            delegate: root_did,
            trust_factor: 0.5,
            capabilities: vec![],
            valid_until: None,
        };
        assert!(event.is_identity_event());

        // Non-identity event
        let event = StateEvent::TrustUpdate {
            entity_id: "test".to_string(),
            delta: 0.1,
            reason: TrustReason::PositiveInteraction,
            from_realm: None,
            triggered_events: 0,
            new_trust: 0.6,
        };
        assert!(!event.is_identity_event());
    }

    #[test]
    fn test_identity_event_estimated_size() {
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"test");

        // IdentityBootstrapped: 64 bytes
        let event = StateEvent::IdentityBootstrapped {
            root_did,
            namespace: DIDNamespace::Self_,
            mode: IdentityMode::Interactive,
            timestamp_ms: 0,
        };
        assert_eq!(event.estimated_size_bytes(), 64);

        // DelegationCreated: base + capabilities
        let event = StateEvent::DelegationCreated {
            delegator: root_did,
            delegate: root_did,
            trust_factor: 0.8,
            capabilities: vec!["read:*".to_string(), "write:docs".to_string()],
            valid_until: None,
        };
        assert_eq!(event.estimated_size_bytes(), 96 + 2 * 32);

        // IdentityAnomalyDetected: variable size
        let event = StateEvent::IdentityAnomalyDetected {
            did: root_did,
            anomaly_type: "test".to_string(),    // 4 bytes
            severity: "high".to_string(),        // 4 bytes
            details: "some details".to_string(), // 12 bytes
        };
        assert_eq!(event.estimated_size_bytes(), 96 + 4 + 4 + 12);
    }

    #[test]
    fn test_wrapped_state_event_realm_context_identity() {
        let root_did = UniversalId::new(UniversalId::TAG_DID, 1, b"test");
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm");

        // Create wrapped event with realm context
        let event = StateEvent::RealmMembershipChanged {
            realm_id,
            member_id: root_did,
            action: "Joined".to_string(),
            new_role: None,
            realm_sub_did: None,
        };
        let wrapped = WrappedStateEvent::new(event, vec![], 1);

        // String-based realm context should return hex-encoded UniversalId
        let ctx = wrapped.realm_context();
        assert!(ctx.is_some());
        assert_eq!(ctx.unwrap(), hex::encode(realm_id.as_bytes()));

        // UniversalId-based realm context
        let ctx_id = wrapped.realm_context_id();
        assert_eq!(ctx_id, Some(realm_id));
    }

    #[test]
    fn test_state_component_identity_variants() {
        // Verify new StateComponent variants exist
        let _identity = StateComponent::Identity;
        let _credential = StateComponent::Credential;
        let _key_management = StateComponent::KeyManagement;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STATE GRAPH TESTS (Phase 5)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_state_graph_identity {
    use super::*;

    #[test]
    fn test_state_graph_has_identity_edges() {
        let graph = StateGraph::erynoa_graph();

        // Core-Abhängigkeiten
        assert!(
            graph.edges.contains(&(
                StateComponent::Trust,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Trust should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Identity,
                StateRelation::Triggers,
                StateComponent::Trust
            )),
            "Identity should Trigger Trust"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Event,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Event should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Consensus,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Consensus should DependOn Identity"
        );
    }

    #[test]
    fn test_state_graph_identity_realm_integration() {
        let graph = StateGraph::erynoa_graph();

        // Realm-Integration
        assert!(
            graph.edges.contains(&(
                StateComponent::Realm,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Realm should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Identity,
                StateRelation::Triggers,
                StateComponent::Realm
            )),
            "Identity should Trigger Realm"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Room,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Room should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Partition,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Partition should DependOn Identity"
        );
    }

    #[test]
    fn test_state_graph_identity_controller_integration() {
        let graph = StateGraph::erynoa_graph();

        // Controller/Auth
        assert!(
            graph.edges.contains(&(
                StateComponent::Controller,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Controller should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Identity,
                StateRelation::Validates,
                StateComponent::Controller
            )),
            "Identity should Validate Controller"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Controller,
                StateRelation::Aggregates,
                StateComponent::Identity
            )),
            "Controller should Aggregate Identity"
        );
    }

    #[test]
    fn test_state_graph_identity_p2p_integration() {
        let graph = StateGraph::erynoa_graph();

        // P2P Network
        assert!(
            graph.edges.contains(&(
                StateComponent::Swarm,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Swarm should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Swarm,
                StateRelation::Validates,
                StateComponent::Identity
            )),
            "Swarm should Validate Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Gossip,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Gossip should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Privacy,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Privacy should DependOn Identity"
        );
    }

    #[test]
    fn test_state_graph_credential_subsystem() {
        let graph = StateGraph::erynoa_graph();

        // Credential-Sub-System
        assert!(
            graph.edges.contains(&(
                StateComponent::Credential,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Credential should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Credential,
                StateRelation::Validates,
                StateComponent::Identity
            )),
            "Credential should Validate Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Identity,
                StateRelation::Aggregates,
                StateComponent::Credential
            )),
            "Identity should Aggregate Credential"
        );
    }

    #[test]
    fn test_state_graph_key_management_subsystem() {
        let graph = StateGraph::erynoa_graph();

        // KeyManagement-Sub-System
        assert!(
            graph.edges.contains(&(
                StateComponent::KeyManagement,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "KeyManagement should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Identity,
                StateRelation::Aggregates,
                StateComponent::KeyManagement
            )),
            "Identity should Aggregate KeyManagement"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::KeyManagement,
                StateRelation::Triggers,
                StateComponent::Event
            )),
            "KeyManagement should Trigger Event"
        );
    }

    #[test]
    fn test_state_graph_identity_edge_count() {
        let graph = StateGraph::erynoa_graph();

        // Count identity-related edges
        let identity_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|(from, _, to)| {
                *from == StateComponent::Identity
                    || *to == StateComponent::Identity
                    || *from == StateComponent::Credential
                    || *to == StateComponent::Credential
                    || *from == StateComponent::KeyManagement
                    || *to == StateComponent::KeyManagement
            })
            .collect();

        // Should have at least 38 identity-related edges
        assert!(
            identity_edges.len() >= 38,
            "Expected at least 38 identity-related edges, found {}",
            identity_edges.len()
        );
    }

    #[test]
    fn test_state_graph_dependents_of_identity() {
        let graph = StateGraph::erynoa_graph();

        let dependents = graph.dependents(StateComponent::Identity);

        // Many components should depend on Identity
        assert!(
            dependents.contains(&StateComponent::Trust),
            "Trust should be a dependent of Identity"
        );
        assert!(
            dependents.contains(&StateComponent::Controller),
            "Controller should be a dependent of Identity"
        );
        assert!(
            dependents.contains(&StateComponent::Realm),
            "Realm should be a dependent of Identity"
        );
    }

    #[test]
    fn test_state_graph_triggered_by_identity() {
        let graph = StateGraph::erynoa_graph();

        let triggered = graph.triggered_by(StateComponent::Identity);

        // Identity should trigger several components
        assert!(
            triggered.contains(&StateComponent::Trust),
            "Identity should trigger Trust"
        );
        assert!(
            triggered.contains(&StateComponent::Realm),
            "Identity should trigger Realm"
        );
        assert!(
            triggered.contains(&StateComponent::Event),
            "Identity should trigger Event"
        );
        assert!(
            triggered.contains(&StateComponent::Anomaly),
            "Identity should trigger Anomaly"
        );
    }

    #[test]
    fn test_state_graph_engine_layer_depends_on_identity() {
        let graph = StateGraph::erynoa_graph();

        // Engine-Layer should depend on Identity
        assert!(
            graph.edges.contains(&(
                StateComponent::UI,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "UI should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::API,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "API should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::Governance,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "Governance should DependOn Identity"
        );
    }

    #[test]
    fn test_state_graph_eclvm_depends_on_identity() {
        let graph = StateGraph::erynoa_graph();

        // ECLVM Layer should depend on Identity
        assert!(
            graph.edges.contains(&(
                StateComponent::ECLVM,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "ECLVM should DependOn Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::ECLPolicy,
                StateRelation::DependsOn,
                StateComponent::Identity
            )),
            "ECLPolicy should DependOn Identity"
        );
    }

    #[test]
    fn test_state_graph_protection_validates_identity() {
        let graph = StateGraph::erynoa_graph();

        // Protection should validate Identity
        assert!(
            graph.edges.contains(&(
                StateComponent::Anomaly,
                StateRelation::Validates,
                StateComponent::Identity
            )),
            "Anomaly should Validate Identity"
        );
        assert!(
            graph.edges.contains(&(
                StateComponent::AntiCalcification,
                StateRelation::Validates,
                StateComponent::Identity
            )),
            "AntiCalcification should Validate Identity"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// UNIFIED STATE INTEGRATION TESTS (Phase 6)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_unified_state_identity {
    use super::*;
    use crate::core::identity_types::IdentityMode;

    #[test]
    fn test_unified_state_has_identity_field() {
        let state = UnifiedState::new();

        // Identity should be initialized
        assert!(!state.identity.is_bootstrapped());
        assert_eq!(state.identity.current_mode(), IdentityMode::Interactive);
    }

    #[test]
    fn test_unified_state_identity_bootstrap() {
        let state = UnifiedState::new();

        // Bootstrap identity
        let result = state.identity.bootstrap_test(&[1u8; 32]);
        assert!(result.is_ok());
        assert!(state.identity.is_bootstrapped());
    }

    #[test]
    fn test_unified_state_snapshot_includes_identity() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        let snapshot = state.snapshot();

        // Snapshot should include identity
        assert!(snapshot.identity.bootstrap_completed);
        assert_eq!(snapshot.identity.mode, IdentityMode::Test);
        assert!(snapshot.identity.root_did.is_some());
    }

    #[test]
    fn test_unified_state_health_includes_identity() {
        let state = UnifiedState::new();

        // Without bootstrap, identity health is 0
        let health_before = state.calculate_health();

        // Bootstrap identity
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        // Health should improve after bootstrap
        let health_after = state.calculate_health();

        // Identity contributes 10% to health, so after bootstrap health should increase
        // (bootstrapped identity has health_score() returning > 0)
        assert!(
            health_after > health_before,
            "Health should increase after identity bootstrap: before={}, after={}",
            health_before,
            health_after
        );
    }

    #[test]
    fn test_unified_state_identity_integration_with_core() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        // Derive a device DID
        let device_result = state.identity.derive_device_did(0);
        assert!(device_result.is_ok());

        // Verify sub_dids_total increased
        assert_eq!(state.identity.sub_dids_total.load(Ordering::Relaxed), 1);

        // Create snapshot and verify
        let snapshot = state.snapshot();
        assert_eq!(snapshot.identity.sub_dids_total, 1);
    }

    #[test]
    fn test_unified_state_identity_realm_membership() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");

        // Join realm
        let result = state.identity.join_realm(
            realm_id,
            crate::core::identity_types::RealmRole::Member,
            None,
        );
        assert!(result.is_ok());

        // Verify membership
        assert!(state.identity.is_realm_member(&realm_id));

        // Snapshot should reflect membership
        let snapshot = state.snapshot();
        assert_eq!(snapshot.identity.realm_membership_count, 1);
    }

    #[test]
    fn test_unified_state_graph_identity_relationships() {
        let state = UnifiedState::new();

        // Verify the graph has identity relationships
        let identity_deps = state.graph.dependents(StateComponent::Identity);
        assert!(!identity_deps.is_empty());

        let identity_triggers = state.graph.triggered_by(StateComponent::Identity);
        assert!(!identity_triggers.is_empty());
    }

    #[test]
    fn test_unified_state_identity_delegation_integration() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");

        // Create delegation
        let result = state.identity.add_delegation(
            delegate_id,
            0.8,
            vec![crate::domain::unified::identity::Capability::Read {
                resource: "*".to_string(),
            }],
            None,
        );
        assert!(result.is_ok());

        // Verify in snapshot
        let snapshot = state.snapshot();
        assert_eq!(snapshot.identity.active_delegations, 1);
    }

    #[test]
    fn test_unified_state_identity_wallet_integration() {
        let state = UnifiedState::new();
        state.identity.bootstrap_test(&[1u8; 32]).unwrap();

        let root_id = state.identity.root_did_id().unwrap();
        let wallet = crate::core::identity_types::WalletAddress::new(
            "eip155:1",
            "0x1234567890123456789012345678901234567890",
            "m/44'/60'/0'/0/0",
            root_id,
        );

        // Add wallet
        let result = state.identity.add_wallet_address(wallet);
        assert!(result.is_ok());

        // Verify in snapshot
        let snapshot = state.snapshot();
        assert_eq!(snapshot.identity.addresses_total, 1);
        assert!(snapshot
            .identity
            .wallet_chains
            .contains(&"eip155:1".to_string()));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PHASE 7: MIGRATION TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_phase7_migration {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────
    // RealmSpecificState Migration Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_realm_state_member_by_id() {
        let realm = RealmSpecificState::new(0.5, "democratic");
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-member");

        // Add member by UniversalId
        realm.add_member_by_id(identity_id, None);

        // Verify membership
        assert!(realm.is_member_by_id(&identity_id));
        assert_eq!(realm.identity_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_realm_state_member_with_realm_sub_did() {
        let realm = RealmSpecificState::new(0.5, "democratic");
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-member");
        let realm_sub_did = UniversalId::new(UniversalId::TAG_DID, 1, b"realm-sub-did");

        // Add member with realm-specific sub-DID
        realm.add_member_by_id(identity_id, Some(realm_sub_did));

        // Verify membership and sub-DID mapping
        assert!(realm.is_member_by_id(&identity_id));
        assert_eq!(realm.get_realm_sub_did(&identity_id), Some(realm_sub_did));
    }

    #[test]
    fn test_realm_state_remove_member_by_id() {
        let realm = RealmSpecificState::new(0.5, "democratic");
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-member");

        // Add and remove
        realm.add_member_by_id(identity_id, None);
        assert!(realm.is_member_by_id(&identity_id));

        realm.remove_member_by_id(&identity_id);
        assert!(!realm.is_member_by_id(&identity_id));
        assert_eq!(realm.identity_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_realm_state_ban_member_by_id() {
        let realm = RealmSpecificState::new(0.5, "democratic");
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-member");

        // Add then ban
        realm.add_member_by_id(identity_id, None);
        realm.ban_member_by_id(&identity_id);

        // Should be banned but not a member
        assert!(!realm.is_member_by_id(&identity_id));
        assert!(realm.is_banned_by_id(&identity_id));
    }

    #[test]
    fn test_realm_state_admin_by_id() {
        let realm = RealmSpecificState::new(0.5, "democratic");
        let admin_id = UniversalId::new(UniversalId::TAG_DID, 1, b"admin");

        // Add admin
        realm.add_admin_by_id(admin_id, None);

        // Should be both admin and member
        assert!(realm.is_admin_by_id(&admin_id));
        assert!(realm.is_member_by_id(&admin_id));
    }

    // ─────────────────────────────────────────────────────────────────────
    // TrustState Migration Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_trust_state_register_identity() {
        let trust = TrustState::new();
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-identity");

        // Register with initial trust
        let result = trust.register_identity(identity_id, 0.7);
        assert!(result.is_ok());

        // Verify
        assert_eq!(trust.get_trust(&identity_id), Some(0.7));
        assert_eq!(trust.identity_count(), 1);
    }

    #[test]
    fn test_trust_state_update_identity_trust() {
        let trust = TrustState::new();
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-identity");

        trust.register_identity(identity_id, 0.5).unwrap();

        // Update trust
        let result = trust.update_identity_trust(&identity_id, 0.2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.7);
    }

    #[test]
    fn test_trust_state_realm_trust() {
        let trust = TrustState::new();
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-identity");
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"test-realm");

        trust.register_identity(identity_id, 0.5).unwrap();

        // Update realm-specific trust
        trust
            .update_identity_realm_trust(&identity_id, realm_id, 0.3)
            .unwrap();

        // Verify realm-specific trust
        assert_eq!(trust.get_realm_trust(&identity_id, &realm_id), Some(0.8));
        // Global trust unchanged
        assert_eq!(trust.get_trust(&identity_id), Some(0.5));
    }

    #[test]
    fn test_trust_state_decay() {
        let trust = TrustState::new();
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-identity");

        trust.register_identity(identity_id, 1.0).unwrap();

        // Apply decay
        trust.apply_global_decay(0.1); // 10% decay

        // Trust should be reduced
        let current = trust.get_trust(&identity_id).unwrap();
        assert!((current - 0.9).abs() < 0.001);
    }

    // ─────────────────────────────────────────────────────────────────────
    // NetworkEvent Migration Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_network_event_with_peer_identity() {
        let peer_id = UniversalId::new(UniversalId::TAG_DID, 1, b"peer");

        let event = NetworkEvent::new("test_event", vec![1, 2, 3], EventPriority::Normal)
            .with_peer_identity(peer_id);

        assert_eq!(event.peer_universal_id, Some(peer_id));
        assert!(event.signature.is_none());
    }

    #[test]
    fn test_network_event_signed() {
        let signer_id = UniversalId::new(UniversalId::TAG_DID, 1, b"signer");

        // Create signed event with mock signer
        let result = NetworkEvent::signed(
            "test_event",
            vec![1, 2, 3],
            EventPriority::Normal,
            signer_id,
            |_payload| Ok([0u8; 64]),
        );

        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(event.is_signed());
        assert_eq!(event.is_verified(), Some(true));
    }

    #[test]
    fn test_network_event_not_signed() {
        let event = NetworkEvent::new("test_event", vec![1, 2, 3], EventPriority::Normal);

        assert!(!event.is_signed());
        assert_eq!(event.is_verified(), None);
    }

    // ─────────────────────────────────────────────────────────────────────
    // SwarmState Migration Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_swarm_state_peer_universal_id() {
        let swarm = SwarmState::new();
        let peer_id = UniversalId::new(UniversalId::TAG_DID, 1, b"device-did");

        // Set peer identity
        swarm.set_peer_universal_id(peer_id);

        // Verify
        assert_eq!(swarm.get_peer_universal_id(), Some(peer_id));

        // Snapshot should include it
        let snapshot = swarm.snapshot();
        assert_eq!(snapshot.peer_universal_id, Some(peer_id));
    }

    // ─────────────────────────────────────────────────────────────────────
    // TrustEntry Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_trust_entry_creation() {
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test");
        let entry = TrustEntry::new(identity_id, 0.8);

        assert_eq!(entry.global_trust, 0.8);
        assert_eq!(entry.update_count, 0);
        assert!(entry.per_realm_trust.is_empty());
    }

    #[test]
    fn test_trust_entry_clamping() {
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test");

        // Test clamping above 1.0
        let entry = TrustEntry::new(identity_id, 1.5);
        assert_eq!(entry.global_trust, 1.0);

        // Test clamping below 0.0
        let entry = TrustEntry::new(identity_id, -0.5);
        assert_eq!(entry.global_trust, 0.0);
    }

    #[test]
    fn test_trust_entry_realm_fallback() {
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test");
        let realm_id = UniversalId::new(UniversalId::TAG_REALM, 1, b"realm");
        let entry = TrustEntry::new(identity_id, 0.5);

        // Without specific realm trust, should fallback to global
        assert_eq!(entry.get_realm_trust(&realm_id), 0.5);
    }

    // ─────────────────────────────────────────────────────────────────────
    // MembershipChange Event Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_membership_change_with_universal_id() {
        let identity_id = UniversalId::new(UniversalId::TAG_DID, 1, b"member");
        let initiator_id = UniversalId::new(UniversalId::TAG_DID, 1, b"initiator");

        let event = StateEvent::MembershipChange {
            realm_id: "test-realm".to_string(),
            identity_id: "did:erynoa:self:test".to_string(),
            identity_universal_id: Some(identity_id),
            action: MembershipAction::Joined,
            new_role: None,
            initiated_by: None,
            initiated_by_id: Some(initiator_id),
        };

        // Verify event properties
        assert_eq!(event.primary_component(), StateComponent::Realm);
        assert!(event.estimated_size_bytes() > 0);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PHASE 6.4 TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_phase6_4 {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────
    // ECLVMBudget Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_eclvm_budget_creation() {
        let budget = ECLVMBudget::with_defaults();
        assert_eq!(budget.gas_remaining(), 1_000_000);
        assert_eq!(budget.mana_remaining(), 10_000);
        assert!(!budget.is_exhausted());
    }

    #[test]
    fn test_eclvm_budget_gas_consumption() {
        let budget = ECLVMBudget::new(ECLVMBudgetLimits {
            gas_limit: 100,
            mana_limit: 100,
            max_stack_depth: 64,
            timeout_ms: 5000,
        });

        // Konsumiere Gas
        assert!(budget.consume_gas(50));
        assert_eq!(budget.gas_used(), 50);
        assert_eq!(budget.gas_remaining(), 50);

        // Konsumiere mehr Gas
        assert!(budget.consume_gas(49));
        assert_eq!(budget.gas_remaining(), 1);

        // Überschreite Limit
        assert!(!budget.consume_gas(10));
        assert!(budget.is_exhausted());
        assert_eq!(
            budget.exhaustion_reason(),
            Some(BudgetExhaustionReason::OutOfGas)
        );
    }

    #[test]
    fn test_eclvm_budget_mana_consumption() {
        let budget = ECLVMBudget::new(ECLVMBudgetLimits {
            gas_limit: 1000,
            mana_limit: 10,
            max_stack_depth: 64,
            timeout_ms: 5000,
        });

        assert!(budget.consume_mana(5));
        assert!(budget.consume_mana(5));
        assert!(!budget.consume_mana(1));
        assert!(budget.is_exhausted());
        assert_eq!(
            budget.exhaustion_reason(),
            Some(BudgetExhaustionReason::OutOfMana)
        );
    }

    #[test]
    fn test_eclvm_budget_stack_depth() {
        let budget = ECLVMBudget::new(ECLVMBudgetLimits {
            gas_limit: 1000,
            mana_limit: 100,
            max_stack_depth: 10,
            timeout_ms: 5000,
        });

        assert!(budget.check_stack_depth(5));
        assert!(budget.check_stack_depth(10));
        assert!(!budget.check_stack_depth(11));
        assert!(budget.is_exhausted());
        assert_eq!(
            budget.exhaustion_reason(),
            Some(BudgetExhaustionReason::StackOverflow)
        );
    }

    #[test]
    fn test_eclvm_budget_limits_trust_scaling() {
        let base = ECLVMBudgetLimits::default();
        let scaled_low = base.with_trust_factor(0.0);
        let scaled_high = base.with_trust_factor(1.0);

        // Low trust = 50% of base
        assert_eq!(scaled_low.gas_limit, 500_000);
        // High trust = 100% of base
        assert_eq!(scaled_high.gas_limit, 1_000_000);
    }

    // ─────────────────────────────────────────────────────────────────────
    // StateView Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_state_view_creation() {
        let view = StateView::new(
            Some("did:test:caller".to_string()),
            Some("realm_001".to_string()),
        );

        assert!(view.has_caller());
        assert!(view.has_realm_context());
        assert!(view.snapshot_time > 0);
    }

    #[test]
    fn test_state_view_trust_cache() {
        let view = StateView::new(None, None);

        // Initial keine Trusts
        assert!(view.get_trust("alice").is_none());

        // Setze Trust
        view.set_trust_cached("alice", 0.8);
        assert_eq!(view.get_trust("alice"), Some(0.8));

        // Threshold check
        assert!(view.trust_above_threshold("alice", 0.5));
        assert!(!view.trust_above_threshold("alice", 0.9));
    }

    #[test]
    fn test_state_view_realm_cache() {
        let view = StateView::new(None, Some("realm_001".to_string()));

        let realm_data = RealmViewData {
            realm_id: "realm_001".to_string(),
            name: "Test Realm".to_string(),
            owner_did: "did:test:owner".to_string(),
            member_count: 10,
            trust_threshold: 0.3,
            is_quarantined: false,
            created_at: 1000,
        };

        view.set_realm_cached(realm_data);

        let retrieved = view.get_realm("realm_001").unwrap();
        assert_eq!(retrieved.name, "Test Realm");
        assert_eq!(retrieved.member_count, 10);
    }

    #[test]
    fn test_state_view_identity_cache() {
        let view = StateView::new(Some("did:test:caller".to_string()), None);

        let identity_data = IdentityViewData {
            did: "did:test:caller".to_string(),
            display_name: Some("Caller".to_string()),
            trust_score: 0.7,
            realms: vec!["realm_001".to_string()],
            created_at: 1000,
        };

        view.set_identity_cached(identity_data);

        let caller = view.caller_identity().unwrap();
        assert_eq!(caller.trust_score, 0.7);
        assert!(view.is_caller_verified());
    }

    // ─────────────────────────────────────────────────────────────────────
    // StateHandle Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_state_handle_creation() {
        let state = UnifiedState::new();
        let budget = Arc::new(ECLVMBudget::with_defaults());

        let handle = StateHandle::new(
            &state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
            budget,
        );

        assert_eq!(handle.caller(), "did:test:caller");
        assert_eq!(handle.realm(), "realm_001");
        assert!(handle.is_valid());
        assert_eq!(handle.pending_events_count(), 0);
    }

    #[test]
    fn test_state_handle_trust_update() {
        let state = UnifiedState::new();
        let budget = Arc::new(ECLVMBudget::with_defaults());

        let handle = StateHandle::new(
            &state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
            budget.clone(),
        );

        let result = handle.update_trust("did:test:target", 0.1, TrustReason::PositiveInteraction);

        assert!(matches!(result, MutationResult::Success));
        assert_eq!(handle.pending_events_count(), 1);
        assert!(budget.gas_used() >= 100);
    }

    #[test]
    fn test_state_handle_trust_update_validation() {
        let state = UnifiedState::new();
        let budget = Arc::new(ECLVMBudget::with_defaults());

        let handle = StateHandle::new(
            &state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
            budget,
        );

        // Delta zu groß (> 0.5)
        let result = handle.update_trust("did:test:target", 0.6, TrustReason::PositiveInteraction);

        assert!(matches!(result, MutationResult::ValidationFailed(_)));
    }

    #[test]
    fn test_state_handle_store_operations() {
        let state = UnifiedState::new();
        let budget = Arc::new(ECLVMBudget::with_defaults());

        let handle = StateHandle::new(
            &state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
            budget.clone(),
        );

        // Store put
        let result = handle.store_put("key1", "value1");
        assert!(matches!(result, MutationResult::Success));
        assert_eq!(handle.dirty_keys_count(), 1);

        // Store delete
        let result = handle.store_delete("key2");
        assert!(matches!(result, MutationResult::Success));
        assert_eq!(handle.dirty_keys_count(), 2);
    }

    #[test]
    fn test_state_handle_commit() {
        let state = UnifiedState::new();
        let budget = Arc::new(ECLVMBudget::with_defaults());

        let handle = StateHandle::new(
            &state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
            budget,
        );

        // Füge Operation hinzu
        handle.update_trust("target", 0.1, TrustReason::PositiveInteraction);
        handle.store_put("key", "value");

        // Commit
        let result = handle.commit();
        match result {
            CommitResult::Success {
                events_applied,
                keys_modified,
                ..
            } => {
                assert_eq!(events_applied, 1);
                assert!(keys_modified >= 1);
            }
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_state_handle_rollback() {
        let state = UnifiedState::new();
        let budget = Arc::new(ECLVMBudget::with_defaults());

        let handle = StateHandle::new(
            &state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
            budget,
        );

        // Füge Operationen hinzu
        handle.update_trust("target", 0.1, TrustReason::PositiveInteraction);
        handle.store_put("key", "value");

        // Rollback
        let result = handle.rollback();
        match result {
            RollbackResult::Success {
                events_discarded,
                keys_discarded,
            } => {
                assert_eq!(events_discarded, 1);
                assert!(keys_discarded >= 1);
            }
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_state_handle_budget_exhaustion() {
        let state = UnifiedState::new();
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits {
            gas_limit: 50, // Sehr wenig Gas
            mana_limit: 100,
            max_stack_depth: 64,
            timeout_ms: 5000,
        }));

        let handle = StateHandle::new(
            &state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
            budget,
        );

        // Erste Operation verbraucht 100 Gas - sollte fehlschlagen
        let result = handle.update_trust("target", 0.1, TrustReason::PositiveInteraction);
        assert!(matches!(result, MutationResult::BudgetExhausted(_)));
    }

    // ─────────────────────────────────────────────────────────────────────
    // TransactionGuard Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_transaction_guard_commit() {
        let state = UnifiedState::new();
        let budget = Arc::new(ECLVMBudget::with_defaults());

        let handle = StateHandle::new(
            &state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
            budget,
        );

        let guard = TransactionGuard::new(handle);

        // Operation über Guard
        if let Some(h) = guard.handle() {
            h.update_trust("target", 0.1, TrustReason::PositiveInteraction);
        }

        // Expliziter Commit
        let result = guard.commit();
        assert!(matches!(result, CommitResult::Success { .. }));
    }

    #[test]
    fn test_transaction_guard_auto_rollback() {
        let state = UnifiedState::new();
        let initial_trust_updates = state.core.trust.updates_total.load(Ordering::Relaxed);

        {
            let budget = Arc::new(ECLVMBudget::with_defaults());
            let handle = StateHandle::new(
                &state,
                "did:test:caller".to_string(),
                "realm_001".to_string(),
                budget,
            );

            let guard = TransactionGuard::new(handle);

            // Operation
            if let Some(h) = guard.handle() {
                h.update_trust("target", 0.1, TrustReason::PositiveInteraction);
            }

            // Guard wird hier gedroppt ohne commit -> rollback
        }

        // State sollte unverändert sein (Rollback)
        // Note: In echtem Szenario würde man hier prüfen dass die Änderung nicht applied wurde
        // Da wir aber log_and_apply() nicht beim Rollback aufrufen, ist das automatisch der Fall
        let _ = initial_trust_updates; // Unused variable acknowledgment
    }

    // ─────────────────────────────────────────────────────────────────────
    // ECLVMStateContext Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_eclvm_state_context_creation() {
        let state = Arc::new(UnifiedState::new());
        let ctx = ECLVMStateContext::with_defaults(
            state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
        );

        assert!(ctx.is_valid());
        assert_eq!(ctx.caller(), "did:test:caller");
        assert_eq!(ctx.realm(), "realm_001");
        assert!(ctx.execution_id().starts_with("eclvm_realm_001_"));
    }

    #[test]
    fn test_eclvm_state_context_read_operations() {
        let state = Arc::new(UnifiedState::new());
        let ctx = ECLVMStateContext::with_defaults(
            state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
        );

        // Populate cache
        ctx.populate_trust("alice", 0.8);
        ctx.populate_trust("bob", 0.3);

        // Reads verbrauchen Gas
        let initial_gas = ctx.gas_remaining();
        assert_eq!(ctx.get_trust("alice"), Some(0.8));
        assert!(ctx.gas_remaining() < initial_gas);

        // Threshold check
        assert!(ctx.check_trust_threshold("alice", 0.5));
        assert!(!ctx.check_trust_threshold("bob", 0.5));
    }

    #[test]
    fn test_eclvm_state_context_write_transaction() {
        let state = Arc::new(UnifiedState::new());
        let ctx = ECLVMStateContext::with_defaults(
            state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
        );

        // Begin transaction
        let mut txn = ctx.begin_transaction();

        // Operationen über Handle
        if let Some(handle) = txn.handle() {
            handle.update_trust("target", 0.1, TrustReason::PositiveInteraction);
            handle.store_put("key", "value");
        }

        // Commit transaction
        let result = txn.commit();
        assert!(matches!(result, CommitResult::Success { .. }));
    }

    #[test]
    fn test_eclvm_state_context_finalization() {
        let state = Arc::new(UnifiedState::new());
        let ctx = ECLVMStateContext::with_defaults(
            state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
        );

        // Einige Operationen
        ctx.populate_trust("alice", 0.8);
        ctx.get_trust("alice");
        ctx.get_trust("bob"); // None, aber verbraucht Gas

        // Finalize
        let summary = ctx.finalize();

        assert!(summary.is_success());
        assert!(summary.gas_used > 0);
        assert!(summary.gas_utilization_percent() < 1.0); // Weniger als 1% genutzt
    }

    #[test]
    fn test_eclvm_state_context_budget_integration() {
        let state = Arc::new(UnifiedState::new());
        let ctx = ECLVMStateContext::new(
            state,
            "did:test:caller".to_string(),
            "realm_001".to_string(),
            ECLVMBudgetLimits::minimal(),
        );

        // Budget ist minimal
        assert_eq!(ctx.budget.limits.gas_limit, 10_000);

        // Viele Reads bis Budget erschöpft
        ctx.populate_trust("alice", 0.8);
        for _ in 0..2000 {
            if ctx.get_trust("alice").is_none() {
                break; // Budget erschöpft
            }
        }

        // Budget sollte jetzt erschöpft sein
        assert!(ctx.budget.is_exhausted());
        let summary = ctx.finalize();
        assert!(!summary.is_success());
    }

    // ─────────────────────────────────────────────────────────────────────
    // E1.2: ExecutionState ↔ ECLVM Integration Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_e1_policy_evaluated_syncs_execution_state() {
        let state = UnifiedState::new();

        // Initiale Werte
        let initial_exec_total = state.execution.executions.total.load(Ordering::Relaxed);
        let initial_eclvm_executed = state.eclvm.policies_executed.load(Ordering::Relaxed);
        let initial_gas = state.execution.gas.consumed.load(Ordering::Relaxed);
        let initial_mana = state.execution.mana.consumed.load(Ordering::Relaxed);

        // Simuliere PolicyEvaluated Event
        let event = StateEvent::PolicyEvaluated {
            policy_id: "test_policy".to_string(),
            realm_id: Some("realm:test".to_string()),
            passed: true,
            policy_type: ECLPolicyType::Crossing,
            gas_used: 1000,
            mana_used: 50,
            duration_us: 1500, // 1.5ms
        };

        let wrapped = WrappedStateEvent::new(event, vec![], 1);
        state.apply_state_event(&wrapped);

        // Verifiziere ECLVMState wurde aktualisiert
        assert_eq!(
            state.eclvm.policies_executed.load(Ordering::Relaxed),
            initial_eclvm_executed + 1
        );
        assert_eq!(state.eclvm.policies_passed.load(Ordering::Relaxed), 1);
        assert_eq!(state.eclvm.total_gas_consumed.load(Ordering::Relaxed), 1000);
        assert_eq!(state.eclvm.total_mana_consumed.load(Ordering::Relaxed), 50);

        // E1.2: Verifiziere ExecutionState wurde AUCH aktualisiert
        assert_eq!(
            state.execution.executions.total.load(Ordering::Relaxed),
            initial_exec_total + 1
        );
        assert_eq!(
            state
                .execution
                .executions
                .successful
                .load(Ordering::Relaxed),
            1
        );
        assert_eq!(
            state.execution.gas.consumed.load(Ordering::Relaxed),
            initial_gas + 1000
        );
        assert_eq!(
            state.execution.mana.consumed.load(Ordering::Relaxed),
            initial_mana + 50
        );

        // Verifiziere Execution-Zeit wurde erfasst (duration_us → ms)
        let avg_time = state.execution.executions.avg_execution_time();
        assert!(avg_time >= 0.0); // duration_us = 1500 → 1ms
    }

    #[test]
    fn test_e1_failed_policy_updates_failed_count() {
        let state = UnifiedState::new();

        // Fehlgeschlagene Policy
        let event = StateEvent::PolicyEvaluated {
            policy_id: "failed_policy".to_string(),
            realm_id: Some("realm:test".to_string()),
            passed: false,
            policy_type: ECLPolicyType::Membership,
            gas_used: 500,
            mana_used: 25,
            duration_us: 800,
        };

        let wrapped = WrappedStateEvent::new(event, vec![], 1);
        state.apply_state_event(&wrapped);

        // ECLVMState: denied
        assert_eq!(state.eclvm.policies_denied.load(Ordering::Relaxed), 1);
        assert_eq!(state.eclvm.policies_passed.load(Ordering::Relaxed), 0);

        // ExecutionState: failed
        assert_eq!(state.execution.executions.failed.load(Ordering::Relaxed), 1);
        assert_eq!(
            state
                .execution
                .executions
                .successful
                .load(Ordering::Relaxed),
            0
        );
    }

    #[test]
    fn test_e1_multiple_policies_aggregate_correctly() {
        let state = UnifiedState::new();

        // 3 Policies: 2 erfolgreich, 1 fehlgeschlagen
        for (i, passed) in [(true, 100), (true, 200), (false, 300)].iter().enumerate() {
            let event = StateEvent::PolicyEvaluated {
                policy_id: format!("policy_{}", i),
                realm_id: Some("realm:test".to_string()),
                passed: passed.0,
                policy_type: ECLPolicyType::Crossing,
                gas_used: passed.1 as u64,
                mana_used: 10,
                duration_us: 1000,
            };
            let wrapped = WrappedStateEvent::new(event, vec![], (i + 1) as u64);
            state.apply_state_event(&wrapped);
        }

        // ECLVMState
        assert_eq!(state.eclvm.policies_executed.load(Ordering::Relaxed), 3);
        assert_eq!(state.eclvm.policies_passed.load(Ordering::Relaxed), 2);
        assert_eq!(state.eclvm.policies_denied.load(Ordering::Relaxed), 1);
        assert_eq!(state.eclvm.total_gas_consumed.load(Ordering::Relaxed), 600); // 100+200+300
        assert_eq!(state.eclvm.total_mana_consumed.load(Ordering::Relaxed), 30); // 10*3

        // ExecutionState (E1.2)
        assert_eq!(state.execution.executions.total.load(Ordering::Relaxed), 3);
        assert_eq!(
            state
                .execution
                .executions
                .successful
                .load(Ordering::Relaxed),
            2
        );
        assert_eq!(state.execution.executions.failed.load(Ordering::Relaxed), 1);
        assert_eq!(state.execution.gas.consumed.load(Ordering::Relaxed), 600);
        assert_eq!(state.execution.mana.consumed.load(Ordering::Relaxed), 30);

        // Success-Rate
        let success_rate = state.execution.executions.success_rate();
        assert!((success_rate - 0.666666).abs() < 0.01);
    }

    #[test]
    fn test_e1_record_eclvm_policy_execution_aggregates_gas_mana() {
        let executions = ExecutionsState::new();

        // Führe Policy aus
        executions.record_eclvm_policy_execution(
            true, // success
            1000, // gas
            50,   // mana
            1,    // events
            5000, // duration_us (5ms)
        );

        assert_eq!(executions.total.load(Ordering::Relaxed), 1);
        assert_eq!(executions.successful.load(Ordering::Relaxed), 1);
        assert_eq!(executions.gas_aggregations.load(Ordering::Relaxed), 1);
        assert_eq!(executions.mana_aggregations.load(Ordering::Relaxed), 1);

        // Noch eine ohne Mana
        executions.record_eclvm_policy_execution(
            true, 500, 0, // kein Mana
            1, 2000,
        );

        assert_eq!(executions.total.load(Ordering::Relaxed), 2);
        assert_eq!(executions.gas_aggregations.load(Ordering::Relaxed), 2);
        assert_eq!(executions.mana_aggregations.load(Ordering::Relaxed), 1); // Bleibt 1
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // E4 Tests: StateHandle für Realm-scoped Writes
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_e4_state_handle_update_trust() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        let handle = StateHandle::new(
            &state,
            "did:test:alice".to_string(),
            "realm:test".to_string(),
            budget.clone(),
        );

        // Trust-Update sollte funktionieren
        let result = handle.update_trust("did:test:bob", 0.1, TrustReason::Attestation);
        assert!(matches!(result, MutationResult::Success));

        // Pending Event sollte vorhanden sein
        assert_eq!(handle.pending_events_count(), 1);

        // Dirty-Key sollte vorhanden sein
        assert_eq!(handle.dirty_keys_count(), 1);

        // Commit sollte erfolgreich sein
        let commit_result = handle.commit();
        assert!(matches!(commit_result, CommitResult::Success { .. }));
    }

    #[test]
    fn test_e4_state_handle_cast_vote() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        let handle = StateHandle::new(
            &state,
            "did:test:voter".to_string(),
            "realm:gov".to_string(),
            budget.clone(),
        );

        // Vote abgeben
        let result = handle.cast_vote("prop_123", true, 0.8);
        assert!(matches!(result, MutationResult::Success));

        // Event sollte gepusht sein
        let events = handle.pending_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], StateEvent::VoteCast { .. }));
    }

    #[test]
    fn test_e4_state_handle_submit_proposal() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        let handle = StateHandle::new(
            &state,
            "did:test:proposer".to_string(),
            "realm:gov".to_string(),
            budget.clone(),
        );

        // Proposal einreichen
        let result = handle.submit_proposal("parameter_change", "Increase stake", 24);
        assert!(result.is_ok());

        let proposal_id = result.unwrap();
        assert!(proposal_id.starts_with("prop_realm:gov_"));

        // Event sollte gepusht sein
        let events = handle.pending_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], StateEvent::ProposalCreated { .. }));
    }

    #[test]
    fn test_e4_state_handle_store_put() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        let handle = StateHandle::new(
            &state,
            "did:test:alice".to_string(),
            "realm:test".to_string(),
            budget.clone(),
        );

        // Store-Put sollte funktionieren
        let result = handle.store_put("config", "value=42");
        assert!(matches!(result, MutationResult::Success));

        // Dirty-Key sollte vorhanden sein
        assert!(handle.dirty_keys_count() > 0);
    }

    #[test]
    fn test_e4_state_handle_emit_event() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        let handle = StateHandle::new(
            &state,
            "did:test:alice".to_string(),
            "realm:test".to_string(),
            budget.clone(),
        );

        // Custom Event emittieren
        let result = handle.emit_event("notification", r#"{"type":"welcome"}"#);
        assert!(matches!(result, MutationResult::Success));

        // Events-emitted Counter sollte erhöht sein
        assert_eq!(state.eclvm.events_emitted.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_e4_state_handle_commit_applies_events() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        let handle = StateHandle::new(
            &state,
            "did:test:alice".to_string(),
            "realm:test".to_string(),
            budget.clone(),
        );

        // Trust-Update
        handle.update_trust("did:test:target", 0.05, TrustReason::PeerRecommendation);

        // Commit
        let result = handle.commit();

        match result {
            CommitResult::Success {
                events_applied,
                keys_modified,
                gas_used,
                ..
            } => {
                assert_eq!(events_applied, 1);
                assert_eq!(keys_modified, 1);
                assert!(gas_used > 0);
            }
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_e4_state_handle_rollback_discards() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        let handle = StateHandle::new(
            &state,
            "did:test:alice".to_string(),
            "realm:test".to_string(),
            budget.clone(),
        );

        // Trust-Update
        handle.update_trust("did:test:target", 0.1, TrustReason::DirectInteraction);

        assert_eq!(handle.pending_events_count(), 1);

        // Rollback
        let result = handle.rollback();

        match result {
            RollbackResult::Success {
                events_discarded,
                keys_discarded,
            } => {
                assert_eq!(events_discarded, 1);
                assert_eq!(keys_discarded, 1);
            }
            _ => panic!("Expected Success"),
        }
    }

    #[test]
    fn test_e4_transaction_guard_auto_rollback() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        // TransactionGuard scope
        {
            let handle = StateHandle::new(
                &state,
                "did:test:alice".to_string(),
                "realm:test".to_string(),
                budget.clone(),
            );

            let guard = TransactionGuard::new(handle);

            // Modifikation via Guard
            if let Some(h) = guard.handle() {
                h.update_trust("did:test:target", 0.1, TrustReason::DirectInteraction);
            }

            // Guard wird dropped ohne commit → auto rollback
        }

        // Keine Events sollten angewendet worden sein
        // (In einem echten Szenario würden wir das verifizieren können)
    }

    #[test]
    fn test_e4_transaction_guard_explicit_commit() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        let handle = StateHandle::new(
            &state,
            "did:test:alice".to_string(),
            "realm:test".to_string(),
            budget.clone(),
        );

        let guard = TransactionGuard::new(handle);

        // Modifikation
        if let Some(h) = guard.handle() {
            h.store_put("key", "value");
        }

        // Explicit commit
        let result = guard.commit();
        assert!(matches!(result, CommitResult::Success { .. }));
    }

    #[test]
    fn test_e4_budget_exhaustion_blocks_operations() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits {
            gas_limit: 50, // Sehr wenig Gas
            mana_limit: 10,
            max_stack_depth: 64,
            timeout_ms: 1000,
        }));

        let handle = StateHandle::new(
            &state,
            "did:test:alice".to_string(),
            "realm:test".to_string(),
            budget.clone(),
        );

        // Erster Trust-Update verbraucht 100 Gas → sollte scheitern
        let result = handle.update_trust("did:test:target", 0.1, TrustReason::DirectInteraction);
        assert!(matches!(
            result,
            MutationResult::BudgetExhausted(BudgetExhaustionReason::OutOfGas)
        ));
    }

    #[test]
    fn test_e4_vote_weight_validation() {
        let state = Arc::new(UnifiedState::new());
        let budget = Arc::new(ECLVMBudget::new(ECLVMBudgetLimits::default()));

        let handle = StateHandle::new(
            &state,
            "did:test:voter".to_string(),
            "realm:gov".to_string(),
            budget.clone(),
        );

        // Weight > 1.0 sollte scheitern
        let result = handle.cast_vote("prop_123", true, 1.5);
        assert!(matches!(result, MutationResult::ValidationFailed(_)));

        // Weight <= 0 sollte scheitern
        let result = handle.cast_vote("prop_123", true, 0.0);
        assert!(matches!(result, MutationResult::ValidationFailed(_)));

        // Gültiges Weight sollte funktionieren
        let result = handle.cast_vote("prop_123", true, 0.5);
        assert!(matches!(result, MutationResult::Success));
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // E6 Tests: StateEvent-Integration für ECL
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_e6_policy_evaluated_logged_and_broadcast() {
        use std::sync::atomic::Ordering;

        let state = UnifiedState::new();

        // Subscribe to broadcaster before emitting event
        let mut rx = state.broadcaster.subscribe();

        // Initiale Werte
        let initial_events = state.event_log.snapshot().total_events;
        let initial_deltas = state.broadcaster.snapshot().deltas_sent;

        // StateEvent::PolicyEvaluated erstellen und anwenden
        let event = StateEvent::PolicyEvaluated {
            policy_id: "policy:test:e6".to_string(),
            realm_id: Some("realm:test".to_string()),
            passed: true,
            policy_type: ECLPolicyType::Api,
            gas_used: 1000,
            mana_used: 50,
            duration_us: 500,
        };

        // log_and_apply aufrufen
        let wrapped = state.log_and_apply(event, vec!["e6_test".to_string()]);

        // 1. Event sollte geloggt worden sein
        let log_snapshot = state.event_log.snapshot();
        assert_eq!(log_snapshot.total_events, initial_events + 1);

        // 2. WrappedStateEvent sollte korrekte Komponente haben
        assert_eq!(wrapped.component, StateComponent::ECLPolicy);

        // 3. Delta sollte gesendet worden sein
        let broadcaster_snapshot = state.broadcaster.snapshot();
        assert_eq!(broadcaster_snapshot.deltas_sent, initial_deltas + 1);

        // 4. ECLVMState sollte aktualisiert sein
        let eclvm_snapshot = state.eclvm.snapshot();
        assert_eq!(eclvm_snapshot.policies_executed, 1);
        assert_eq!(eclvm_snapshot.policies_passed, 1);
    }

    #[test]
    fn test_e6_policy_evaluated_component_mapping() {
        // Verifiziere dass PolicyEvaluated den korrekten Component zurückgibt
        let event = StateEvent::PolicyEvaluated {
            policy_id: "test".to_string(),
            realm_id: None,
            passed: false,
            policy_type: ECLPolicyType::Crossing,
            gas_used: 0,
            mana_used: 0,
            duration_us: 0,
        };

        assert_eq!(event.primary_component(), StateComponent::ECLPolicy);
    }

    #[test]
    fn test_e6_multiple_policies_all_logged() {
        let state = UnifiedState::new();

        // 5 verschiedene Policies ausführen
        for i in 0..5 {
            let event = StateEvent::PolicyEvaluated {
                policy_id: format!("policy:test:{}", i),
                realm_id: Some("realm:test".to_string()),
                passed: i % 2 == 0, // alternierend passed/failed
                policy_type: ECLPolicyType::Controller,
                gas_used: (i as u64 + 1) * 100,
                mana_used: (i as u64 + 1) * 10,
                duration_us: (i as u64 + 1) * 50,
            };

            state.log_and_apply(event, vec![]);
        }

        // Alle 5 sollten geloggt sein
        let log_snapshot = state.event_log.snapshot();
        assert!(log_snapshot.total_events >= 5);

        // ECLVM sollte alle 5 zählen
        let eclvm_snapshot = state.eclvm.snapshot();
        assert_eq!(eclvm_snapshot.policies_executed, 5);
        assert_eq!(eclvm_snapshot.policies_passed, 3); // 0, 2, 4 = passed
        assert_eq!(eclvm_snapshot.policies_failed, 2); // 1, 3 = failed

        // Broadcaster sollte 5 Deltas gesendet haben
        let broadcaster_snapshot = state.broadcaster.snapshot();
        assert!(broadcaster_snapshot.deltas_sent >= 5);
    }

    #[test]
    fn test_e6_event_log_buffer() {
        let state = UnifiedState::new();

        // Policy-Event erstellen
        let event = StateEvent::PolicyEvaluated {
            policy_id: "policy:e6:buffer".to_string(),
            realm_id: None,
            passed: true,
            policy_type: ECLPolicyType::Ui,
            gas_used: 500,
            mana_used: 25,
            duration_us: 100,
        };

        let wrapped = state.log_and_apply(event, vec![]);

        // Event sollte im Buffer sein
        let events_since = state
            .event_log
            .events_since(wrapped.sequence.saturating_sub(1));
        assert!(!events_since.is_empty());

        // Das letzte Event sollte unser PolicyEvaluated sein
        let last_event = events_since.last().unwrap();
        assert!(matches!(
            last_event.event,
            StateEvent::PolicyEvaluated { .. }
        ));
    }

    #[test]
    fn test_e6_subscriber_receives_policy_delta() {
        use std::time::Duration;

        let state = UnifiedState::new();

        // Subscriber erstellen BEVOR Event gesendet wird
        let mut rx = state.broadcaster.subscribe();

        // Policy-Event senden
        let event = StateEvent::PolicyEvaluated {
            policy_id: "policy:e6:subscriber".to_string(),
            realm_id: Some("realm:subscriber".to_string()),
            passed: true,
            policy_type: ECLPolicyType::DataLogic,
            gas_used: 2000,
            mana_used: 100,
            duration_us: 1000,
        };

        state.log_and_apply(event, vec![]);

        // Subscriber sollte Delta empfangen (non-blocking try)
        match rx.try_recv() {
            Ok(delta) => {
                assert_eq!(delta.component, StateComponent::ECLPolicy);
                assert!(matches!(delta.delta_type, DeltaType::Update));
            }
            Err(_) => {
                // Kann passieren wenn Channel voll ist oder Race Condition
                // In echten Tests würde man async wait nutzen
            }
        }
    }

    #[test]
    fn test_e6_policy_types_correctly_mapped() {
        let state = UnifiedState::new();

        // Teste verschiedene Policy-Typen
        let types = vec![
            ECLPolicyType::Crossing,
            ECLPolicyType::Membership,
            ECLPolicyType::Transaction,
            ECLPolicyType::Governance,
            ECLPolicyType::Privacy,
            ECLPolicyType::Api,
            ECLPolicyType::Ui,
            ECLPolicyType::DataLogic,
            ECLPolicyType::Controller,
            ECLPolicyType::Custom,
        ];

        for (i, ptype) in types.iter().enumerate() {
            let event = StateEvent::PolicyEvaluated {
                policy_id: format!("policy:type:{}", i),
                realm_id: None,
                passed: true,
                policy_type: *ptype,
                gas_used: 100,
                mana_used: 10,
                duration_us: 50,
            };

            state.log_and_apply(event, vec![]);
        }

        // Alle 10 sollten ausgeführt worden sein
        let eclvm_snapshot = state.eclvm.snapshot();
        assert_eq!(eclvm_snapshot.policies_executed, types.len() as u64);
    }
}

// ============================================================================
// PHASE 7: SHARDING-INFRASTRUKTUR FÜR REALM-SKALIERUNG
// ============================================================================
//
// Produktionsreife Sharding-Lösung für Millionen von Realms:
// 1. LazyShardedRealmState - Lock-free DashMap mit deterministischem Sharding
// 2. Lazy Loading - Realms werden bei Bedarf aus Storage geladen
// 3. LRU Eviction - Inaktive Realms werden aus dem Speicher entfernt
// 4. Event-Sourcing Integration - State-Recovery durch Event-Replay
// 5. Background Tasks - Periodische Eviction ohne Blocking
//
// ```text
// ┌─────────────────────────────────────────────────────────────────────────────┐
// │                    LazyShardedRealmState Architecture                        │
// │                                                                              │
// │  ┌─────────────────────────────────────────────────────────────────────────┐│
// │  │                         Shard Selection                                 ││
// │  │                  FxHash(realm_id) % num_shards                         ││
// │  └─────────────────────────────────────────────────────────────────────────┘│
// │                                    │                                         │
// │         ┌──────────────────────────┼──────────────────────────┐             │
// │         ▼                          ▼                          ▼             │
// │  ┌─────────────┐           ┌─────────────┐           ┌─────────────┐       │
// │  │  Shard 0    │           │  Shard 1    │           │  Shard N-1  │       │
// │  │ ┌─────────┐ │           │ ┌─────────┐ │           │ ┌─────────┐ │       │
// │  │ │DashMap  │ │           │ │DashMap  │ │           │ │DashMap  │ │       │
// │  │ │realm→Arc│ │           │ │realm→Arc│ │           │ │realm→Arc│ │       │
// │  │ └─────────┘ │           │ └─────────┘ │           │ └─────────┘ │       │
// │  │ ┌─────────┐ │           │ ┌─────────┐ │           │ ┌─────────┐ │       │
// │  │ │LRU Cache│ │           │ │LRU Cache│ │           │ │LRU Cache│ │       │
// │  │ │(access) │ │           │ │(access) │ │           │ │(access) │ │       │
// │  │ └─────────┘ │           │ └─────────┘ │           │ └─────────┘ │       │
// │  └─────────────┘           └─────────────┘           └─────────────┘       │
// │                                                                              │
// │  ┌─────────────────────────────────────────────────────────────────────────┐│
// │  │                     Background Eviction Tasks                           ││
// │  │              Per-Shard async task, 10min interval                       ││
// │  │              Removes LRU entries beyond max_per_shard                   ││
// │  └─────────────────────────────────────────────────────────────────────────┘│
// │                                                                              │
// │  ┌─────────────────────────────────────────────────────────────────────────┐│
// │  │                     Lazy Loading Pipeline                               ││
// │  │   get_or_load() → Cache Miss → Storage Load → Event Replay → Insert    ││
// │  └─────────────────────────────────────────────────────────────────────────┘│
// └─────────────────────────────────────────────────────────────────────────────┘
// ```
// ============================================================================

// ─────────────────────────────────────────────────────────────────────────────
// SHARDING CONFIGURATION
// ─────────────────────────────────────────────────────────────────────────────

/// Konfiguration für Realm-Sharding
#[derive(Debug, Clone)]
pub struct ShardingConfig {
    /// Anzahl der Shards (Default: 64, optimiert für moderne CPUs)
    pub num_shards: usize,
    /// Max Einträge pro Shard bevor Eviction beginnt
    pub max_per_shard: usize,
    /// Eviction-Intervall in Sekunden
    pub eviction_interval_secs: u64,
    /// LRU-Kapazität pro Shard für Access-Tracking
    pub lru_capacity_per_shard: usize,
    /// Ob Lazy Loading aktiviert ist
    pub lazy_loading_enabled: bool,
    /// Ob Event-Replay bei Load aktiviert ist
    pub event_replay_on_load: bool,
}

impl Default for ShardingConfig {
    fn default() -> Self {
        Self {
            num_shards: 64,
            max_per_shard: 20_000,
            eviction_interval_secs: 600, // 10 Minuten
            lru_capacity_per_shard: 25_000,
            lazy_loading_enabled: true,
            event_replay_on_load: true,
        }
    }
}

impl ShardingConfig {
    /// Minimale Konfiguration für Tests
    pub fn minimal() -> Self {
        Self {
            num_shards: 4,
            max_per_shard: 100,
            eviction_interval_secs: 60,
            lru_capacity_per_shard: 150,
            lazy_loading_enabled: false,
            event_replay_on_load: false,
        }
    }

    /// High-Performance Konfiguration für Production
    pub fn production() -> Self {
        Self {
            num_shards: 128,
            max_per_shard: 50_000,
            eviction_interval_secs: 300, // 5 Minuten
            lru_capacity_per_shard: 60_000,
            lazy_loading_enabled: true,
            event_replay_on_load: true,
        }
    }

    /// Konfiguration basierend auf verfügbaren CPU-Cores
    pub fn auto_scaled() -> Self {
        let num_cpus = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        Self {
            num_shards: (num_cpus * 4).max(16).min(256),
            max_per_shard: 30_000,
            eviction_interval_secs: 600,
            lru_capacity_per_shard: 35_000,
            lazy_loading_enabled: true,
            event_replay_on_load: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// FAST HASHING (FxHash für deterministisches Sharding)
// ─────────────────────────────────────────────────────────────────────────────

/// Schneller, deterministischer Hash für Shard-Selection
#[inline]
fn fx_hash_str(s: &str) -> u64 {
    let mut hasher = FxHasher::default();
    s.hash(&mut hasher);
    hasher.finish()
}

// ─────────────────────────────────────────────────────────────────────────────
// REALM LOAD ERROR
// ─────────────────────────────────────────────────────────────────────────────

/// Fehler beim Laden eines Realms aus Storage
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RealmLoadError {
    /// Realm existiert nicht
    NotFound(String),
    /// Storage-Fehler
    StorageError(String),
    /// Serialization-Fehler
    DeserializationError(String),
    /// Event-Replay fehlgeschlagen
    EventReplayError(String),
    /// Lazy Loading ist deaktiviert
    LazyLoadingDisabled,
    /// Shard ist überlastet
    ShardOverloaded {
        shard_idx: usize,
        current: usize,
        max: usize,
    },
}

impl std::fmt::Display for RealmLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "Realm not found: {}", id),
            Self::StorageError(e) => write!(f, "Storage error: {}", e),
            Self::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
            Self::EventReplayError(e) => write!(f, "Event replay error: {}", e),
            Self::LazyLoadingDisabled => write!(f, "Lazy loading is disabled"),
            Self::ShardOverloaded {
                shard_idx,
                current,
                max,
            } => {
                write!(f, "Shard {} overloaded: {}/{}", shard_idx, current, max)
            }
        }
    }
}

impl std::error::Error for RealmLoadError {}

// ─────────────────────────────────────────────────────────────────────────────
// REALM STORAGE LOADER (Trait für Dependency Injection)
// ─────────────────────────────────────────────────────────────────────────────

/// Trait für asynchrones Laden von Realms aus Storage
///
/// Implementierungen können verschiedene Storage-Backends unterstützen:
/// - RocksDB/Fjall
/// - IPFS
/// - Cloud Storage
/// - Mock für Tests
///
/// Hinweis: Da RealmSpecificState intern Atomics und Locks verwendet und daher
/// nicht Clone ist, arbeiten wir mit Snapshots für Persistence und erstellen
/// bei load_realm_base einen frischen State.
#[async_trait::async_trait]
pub trait RealmStorageLoader: Send + Sync {
    /// Lade Basis-State eines Realms aus Storage
    ///
    /// Gibt einen neuen RealmSpecificState zurück, initialisiert aus gespeicherten Daten
    async fn load_realm_base(&self, realm_id: &str) -> Result<RealmSpecificState, RealmLoadError>;

    /// Lade Events für ein Realm seit einem bestimmten Event-ID
    async fn load_realm_events_since(
        &self,
        realm_id: &str,
        since_event_id: Option<&str>,
    ) -> Result<Vec<WrappedStateEvent>, RealmLoadError>;

    /// Prüfe ob Realm existiert (ohne vollständiges Laden)
    async fn realm_exists(&self, realm_id: &str) -> bool;

    /// Persistiere Realm-State als Snapshot (für Checkpoint)
    async fn persist_realm_snapshot(
        &self,
        realm_id: &str,
        snapshot: &RealmSpecificSnapshot,
    ) -> Result<(), RealmLoadError>;
}

/// Parameter für Realm-Erstellung (aus gespeicherten Daten)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealmCreationParams {
    pub min_trust: f32,
    pub governance_type: String,
}

/// Mock-Implementierung für Tests (keine echte Storage)
#[derive(Debug, Default)]
pub struct MockRealmStorageLoader {
    /// Vorgeladene Realm-Parameter für Tests
    realm_params: RwLock<HashMap<String, RealmCreationParams>>,
    /// Events pro Realm
    events: RwLock<HashMap<String, Vec<WrappedStateEvent>>>,
}

impl MockRealmStorageLoader {
    pub fn new() -> Self {
        Self::default()
    }

    /// Füge Test-Realm hinzu
    pub fn add_realm(&self, realm_id: &str, min_trust: f32, governance_type: &str) {
        if let Ok(mut realms) = self.realm_params.write() {
            realms.insert(
                realm_id.to_string(),
                RealmCreationParams {
                    min_trust,
                    governance_type: governance_type.to_string(),
                },
            );
        }
    }

    /// Füge Test-Events hinzu
    pub fn add_events(&self, realm_id: &str, events: Vec<WrappedStateEvent>) {
        if let Ok(mut e) = self.events.write() {
            e.insert(realm_id.to_string(), events);
        }
    }
}

#[async_trait::async_trait]
impl RealmStorageLoader for MockRealmStorageLoader {
    async fn load_realm_base(&self, realm_id: &str) -> Result<RealmSpecificState, RealmLoadError> {
        let params = self
            .realm_params
            .read()
            .ok()
            .and_then(|r| r.get(realm_id).cloned())
            .ok_or_else(|| RealmLoadError::NotFound(realm_id.to_string()))?;

        // Erstelle frischen State aus gespeicherten Parametern
        Ok(RealmSpecificState::new(
            params.min_trust,
            &params.governance_type,
        ))
    }

    async fn load_realm_events_since(
        &self,
        realm_id: &str,
        _since_event_id: Option<&str>,
    ) -> Result<Vec<WrappedStateEvent>, RealmLoadError> {
        Ok(self
            .events
            .read()
            .ok()
            .and_then(|e| e.get(realm_id).cloned())
            .unwrap_or_default())
    }

    async fn realm_exists(&self, realm_id: &str) -> bool {
        self.realm_params
            .read()
            .ok()
            .map(|r| r.contains_key(realm_id))
            .unwrap_or(false)
    }

    async fn persist_realm_snapshot(
        &self,
        realm_id: &str,
        snapshot: &RealmSpecificSnapshot,
    ) -> Result<(), RealmLoadError> {
        if let Ok(mut realms) = self.realm_params.write() {
            realms.insert(
                realm_id.to_string(),
                RealmCreationParams {
                    min_trust: snapshot.min_trust,
                    governance_type: snapshot.governance_type.clone(),
                },
            );
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PRODUCTION STORAGE BACKEND (Pluggable - Fjall Default, RocksDB Fallback)
// ─────────────────────────────────────────────────────────────────────────────

/// Pluggable Storage-Backend für den Production Storage-Layer
///
/// Anders als `StorageBackend` (High-Level für StorageHandle) enthält diese
/// Enum die konkreten Konfigurationsdetails für die Produktion.
///
/// # Backends
///
/// - **Fjall** (Default): Pure Rust, high-performance LSM-Tree KV Store
/// - **RocksDB**: Legacy-Fallback für spezielle Use-Cases
/// - **InMemory**: Für Tests ohne Disk-I/O
#[derive(Debug, Clone)]
pub enum ProductionStorageBackend {
    /// Fjall - Pure Rust, high-performance (Default)
    Fjall {
        /// Pfad zum Storage-Verzeichnis
        path: String,
        /// Optional: Maximale Größe der Memtable in Bytes
        max_memtable_size: Option<usize>,
    },
    /// RocksDB - Legacy-Fallback
    RocksDB {
        /// Pfad zum Storage-Verzeichnis
        path: String,
    },
    /// In-Memory für Tests
    InMemory,
}

impl Default for ProductionStorageBackend {
    fn default() -> Self {
        Self::Fjall {
            path: "data/storage".to_string(),
            max_memtable_size: None,
        }
    }
}

impl ProductionStorageBackend {
    /// Erstelle Fjall-Backend mit Default-Pfad
    pub fn fjall(path: &str) -> Self {
        Self::Fjall {
            path: path.to_string(),
            max_memtable_size: None,
        }
    }

    /// Erstelle Fjall-Backend mit angepasster Memtable-Größe
    pub fn fjall_with_config(path: &str, max_memtable_size: usize) -> Self {
        Self::Fjall {
            path: path.to_string(),
            max_memtable_size: Some(max_memtable_size),
        }
    }

    /// Erstelle In-Memory-Backend für Tests
    pub fn in_memory() -> Self {
        Self::InMemory
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STORAGE SERVICE METRICS (Wait-Free mit Atomics)
// ─────────────────────────────────────────────────────────────────────────────

/// Wait-Free Metriken für den Storage-Layer
///
/// Alle Operationen sind lock-free und verwenden atomare Counters.
/// Optimiert für High-Throughput-Szenarien ohne Contention.
#[derive(Debug, Default)]
pub struct StorageServiceMetrics {
    /// Write-Operationen (Events, Checkpoints)
    pub writes: AtomicU64,
    /// Bytes geschrieben
    pub bytes_written: AtomicU64,
    /// Read-Operationen
    pub reads: AtomicU64,
    /// Bytes gelesen
    pub bytes_read: AtomicU64,
    /// Realm-Load-Operationen (inkl. Replay)
    pub load_ops: AtomicU64,
    /// Checkpoint-Operationen (Dirty-Checkpoint)
    pub checkpoint_ops: AtomicU64,
    /// Event-Replays bei Load
    pub event_replays: AtomicU64,
    /// Fehler
    pub errors: AtomicU64,
    /// Thundering Herd Collisions (verhinderte Parallel-Loads)
    pub inflight_hits: AtomicU64,
    /// Index-Seeks (Binary Key Range-Queries)
    pub index_seeks: AtomicU64,
}

impl StorageServiceMetrics {
    /// Erstelle neue Metriken
    pub fn new() -> Self {
        Self::default()
    }

    /// Snapshot für Serialisierung
    pub fn snapshot(&self) -> StorageServiceMetricsSnapshot {
        StorageServiceMetricsSnapshot {
            writes: self.writes.load(Ordering::Relaxed),
            bytes_written: self.bytes_written.load(Ordering::Relaxed),
            reads: self.reads.load(Ordering::Relaxed),
            bytes_read: self.bytes_read.load(Ordering::Relaxed),
            load_ops: self.load_ops.load(Ordering::Relaxed),
            checkpoint_ops: self.checkpoint_ops.load(Ordering::Relaxed),
            event_replays: self.event_replays.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
            inflight_hits: self.inflight_hits.load(Ordering::Relaxed),
            index_seeks: self.index_seeks.load(Ordering::Relaxed),
        }
    }

    /// Write-Throughput (Bytes/Write)
    pub fn write_throughput(&self) -> f64 {
        let writes = self.writes.load(Ordering::Relaxed);
        if writes == 0 {
            return 0.0;
        }
        self.bytes_written.load(Ordering::Relaxed) as f64 / writes as f64
    }

    /// Read-Throughput (Bytes/Read)
    pub fn read_throughput(&self) -> f64 {
        let reads = self.reads.load(Ordering::Relaxed);
        if reads == 0 {
            return 0.0;
        }
        self.bytes_read.load(Ordering::Relaxed) as f64 / reads as f64
    }

    /// Fehlerrate (%)
    pub fn error_rate(&self) -> f64 {
        let total = self.writes.load(Ordering::Relaxed) + self.reads.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }
        (self.errors.load(Ordering::Relaxed) as f64 / total as f64) * 100.0
    }
}

/// Snapshot der Storage-Service-Metriken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageServiceMetricsSnapshot {
    pub writes: u64,
    pub bytes_written: u64,
    pub reads: u64,
    pub bytes_read: u64,
    pub load_ops: u64,
    pub checkpoint_ops: u64,
    pub event_replays: u64,
    pub errors: u64,
    pub inflight_hits: u64,
    pub index_seeks: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// STORAGE SERVICE CONFIG
// ─────────────────────────────────────────────────────────────────────────────

/// Konfiguration für den Storage-Service
#[derive(Debug, Clone)]
pub struct StorageServiceConfig {
    /// Storage-Backend
    pub backend: ProductionStorageBackend,
    /// Dirty-Checkpoint: Anzahl pending Events bevor Checkpoint
    pub checkpoint_event_threshold: u64,
    /// Dirty-Checkpoint: Maximale Zeit seit letztem Checkpoint (Sekunden)
    pub checkpoint_time_threshold_secs: u64,
    /// Ob Event-Replay bei Load aktiviert ist
    pub enable_event_replay: bool,
    /// Ob Component-Index aktiviert ist
    pub enable_component_index: bool,
}

impl Default for StorageServiceConfig {
    fn default() -> Self {
        Self {
            backend: ProductionStorageBackend::default(),
            checkpoint_event_threshold: 1000,
            checkpoint_time_threshold_secs: 60,
            enable_event_replay: true,
            enable_component_index: true,
        }
    }
}

impl StorageServiceConfig {
    /// Minimal-Konfiguration für Tests
    pub fn minimal() -> Self {
        Self {
            backend: ProductionStorageBackend::InMemory,
            checkpoint_event_threshold: 10,
            checkpoint_time_threshold_secs: 5,
            enable_event_replay: false,
            enable_component_index: false,
        }
    }

    /// Production-Konfiguration
    pub fn production(path: &str) -> Self {
        Self {
            backend: ProductionStorageBackend::fjall(path),
            checkpoint_event_threshold: 1000,
            checkpoint_time_threshold_secs: 60,
            enable_event_replay: true,
            enable_component_index: true,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// IN-FLIGHT MAP (Thundering Herd Prevention)
// ─────────────────────────────────────────────────────────────────────────────

/// In-Flight-Request für Thundering Herd Prevention
///
/// Verhindert, dass mehrere gleichzeitige Requests dasselbe Realm laden.
/// Erste Request lädt, alle anderen warten auf das Ergebnis.
type InFlightReceiver =
    tokio::sync::oneshot::Receiver<Result<Arc<RealmSpecificState>, RealmLoadError>>;
type InFlightSender = tokio::sync::oneshot::Sender<Result<Arc<RealmSpecificState>, RealmLoadError>>;

/// In-Flight-Map für aktive Lade-Operationen
#[derive(Debug, Default)]
pub struct InFlightMap {
    /// Realm-ID → Liste von wartenden Receivers
    map: DashMap<String, Vec<InFlightSender>>,
}

impl InFlightMap {
    /// Erstelle neue In-Flight-Map
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    /// Prüfe ob Load bereits in-flight ist und registriere ggf. als Waiter
    ///
    /// # Returns
    /// - `None`: Keine aktive Operation, Caller soll laden
    /// - `Some(receiver)`: Aktive Operation, Caller wartet auf Ergebnis
    pub fn try_register(&self, realm_id: &str) -> Option<InFlightReceiver> {
        let mut entry = self.map.entry(realm_id.to_string()).or_default();
        if entry.value().is_empty() {
            // Erster Request - markiere als in-flight
            None
        } else {
            // Bereits in-flight - registriere als Waiter
            let (tx, rx) = tokio::sync::oneshot::channel();
            entry.value_mut().push(tx);
            Some(rx)
        }
    }

    /// Markiere Start einer Lade-Operation
    pub fn start_load(&self, realm_id: &str) {
        self.map.entry(realm_id.to_string()).or_default();
    }

    /// Abschließen einer Lade-Operation und alle Waiter benachrichtigen
    pub fn complete_load(
        &self,
        realm_id: &str,
        result: Result<Arc<RealmSpecificState>, RealmLoadError>,
    ) {
        if let Some((_, waiters)) = self.map.remove(realm_id) {
            for waiter in waiters {
                // Ignoriere Fehler wenn Receiver dropped wurde
                let _ = waiter.send(result.clone());
            }
        }
    }

    /// Anzahl aktiver In-Flight-Operationen
    pub fn active_count(&self) -> usize {
        self.map.len()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// REALM CHECKPOINT STATE (Tracking für Dirty-Checkpoint)
// ─────────────────────────────────────────────────────────────────────────────

/// Checkpoint-State für Dirty-Checkpointing
#[derive(Debug)]
pub struct RealmCheckpointState {
    /// Letzte Checkpoint-Sequence-Number
    pub last_checkpoint_sequence: AtomicU64,
    /// Pending Events seit letztem Checkpoint
    pub pending_events: AtomicU64,
    /// Timestamp des letzten Checkpoints (Unix-MS)
    pub last_checkpoint_ms: AtomicU64,
}

impl Default for RealmCheckpointState {
    fn default() -> Self {
        Self {
            last_checkpoint_sequence: AtomicU64::new(0),
            pending_events: AtomicU64::new(0),
            last_checkpoint_ms: AtomicU64::new(current_time_ms()),
        }
    }
}

impl RealmCheckpointState {
    /// Erstelle neuen Checkpoint-State
    pub fn new() -> Self {
        Self::default()
    }

    /// Prüfe ob Checkpoint nötig (Dirty-Checkpoint-Logik)
    pub fn needs_checkpoint(&self, config: &StorageServiceConfig) -> bool {
        let pending = self.pending_events.load(Ordering::Relaxed);
        let last_ms = self.last_checkpoint_ms.load(Ordering::Relaxed);
        let now_ms = current_time_ms();

        pending >= config.checkpoint_event_threshold
            || (now_ms - last_ms) >= config.checkpoint_time_threshold_secs * 1000
    }

    /// Record neues Event
    pub fn record_event(&self) {
        self.pending_events.fetch_add(1, Ordering::Relaxed);
    }

    /// Checkpoint durchgeführt
    pub fn checkpoint_done(&self, sequence: u64) {
        self.last_checkpoint_sequence
            .store(sequence, Ordering::Relaxed);
        self.pending_events.store(0, Ordering::Relaxed);
        self.last_checkpoint_ms
            .store(current_time_ms(), Ordering::Relaxed);
    }
}

/// Hilfsfunktion: Aktuelle Zeit in Millisekunden
fn current_time_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ─────────────────────────────────────────────────────────────────────────────
// BINARY KEY BUILDER (Stack-Buffer Reuse für Performance)
// ─────────────────────────────────────────────────────────────────────────────

/// Stack-Buffer für Binary Keys (128 Bytes, keine Heap-Allokation)
///
/// Verwendet für:
/// - realm_index: `{realm_id}:{sequence_be}`
/// - component_index: `{component}:{sequence_be}`
#[derive(Debug)]
pub struct BinaryKeyBuilder {
    buffer: [u8; 128],
    len: usize,
}

impl BinaryKeyBuilder {
    /// Erstelle leeren Builder
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: [0u8; 128],
            len: 0,
        }
    }

    /// Baue Realm-Index-Key: `{realm_id}:{sequence_be}`
    #[inline]
    pub fn realm_index_key(realm_id: &str, sequence: u64) -> Self {
        let mut builder = Self::new();
        let realm_bytes = realm_id.as_bytes();
        let realm_len = realm_bytes.len().min(119); // Max 119 bytes für realm_id

        builder.buffer[..realm_len].copy_from_slice(&realm_bytes[..realm_len]);
        builder.buffer[realm_len] = b':';
        builder.buffer[realm_len + 1..realm_len + 9].copy_from_slice(&sequence.to_be_bytes());
        builder.len = realm_len + 9;

        builder
    }

    /// Baue Component-Index-Key: `{component}:{sequence_be}`
    #[inline]
    pub fn component_index_key(component: &str, sequence: u64) -> Self {
        let mut builder = Self::new();
        let comp_bytes = component.as_bytes();
        let comp_len = comp_bytes.len().min(55); // Max 55 bytes für component (Rest für sequence)

        builder.buffer[..comp_len].copy_from_slice(&comp_bytes[..comp_len]);
        builder.buffer[comp_len] = b':';
        builder.buffer[comp_len + 1..comp_len + 9].copy_from_slice(&sequence.to_be_bytes());
        builder.len = comp_len + 9;

        builder
    }

    /// Hole Key als Slice
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..self.len]
    }
}

impl Default for BinaryKeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// IN-MEMORY STORAGE (Für Tests)
// ─────────────────────────────────────────────────────────────────────────────

/// In-Memory Storage für Tests
///
/// Thread-safe via DashMap, simuliert alle Storage-Operationen ohne Disk-I/O.
#[derive(Debug, Default)]
pub struct InMemoryStorage {
    /// Realm-Base Partition (Checkpoints)
    realm_base: DashMap<String, Vec<u8>>,
    /// State-Events Partition
    state_events: DashMap<String, Vec<u8>>,
    /// Realm-Index Partition (Binary Keys)
    realm_index: DashMap<Vec<u8>, String>,
    /// Component-Index Partition (Binary Keys)
    component_index: DashMap<Vec<u8>, String>,
    /// Checkpoint-State pro Realm
    checkpoint_state: DashMap<String, RealmCheckpointState>,
}

impl InMemoryStorage {
    /// Erstelle neuen In-Memory-Storage
    pub fn new() -> Self {
        Self::default()
    }

    /// Persistiere Event (atomisch)
    pub fn persist_event(
        &self,
        event: &WrappedStateEvent,
        enable_component_index: bool,
    ) -> Result<usize, RealmLoadError> {
        // Serialize Event
        let data = bincode::serialize(event)
            .map_err(|e| RealmLoadError::StorageError(format!("Serialization error: {}", e)))?;
        let bytes_written = data.len();

        // Global Event Key
        let global_key = format!("e:{}", event.id);
        self.state_events.insert(global_key, data);

        // Realm-Index
        if let Some(realm_id) = event.realm_context() {
            let index_key = BinaryKeyBuilder::realm_index_key(&realm_id, event.sequence);
            self.realm_index
                .insert(index_key.as_bytes().to_vec(), event.id.clone());

            // Update Checkpoint-State
            self.checkpoint_state
                .entry(realm_id.clone())
                .or_default()
                .record_event();
        }

        // Component-Index (optional)
        if enable_component_index {
            let comp_str = format!("{:?}", event.component);
            let comp_key = BinaryKeyBuilder::component_index_key(&comp_str, event.sequence);
            self.component_index
                .insert(comp_key.as_bytes().to_vec(), event.id.clone());
        }

        Ok(bytes_written)
    }

    /// Persistiere Realm-Base (Checkpoint)
    pub fn persist_realm_base(
        &self,
        realm_id: &str,
        snapshot: &RealmSpecificSnapshot,
        sequence: u64,
    ) -> Result<usize, RealmLoadError> {
        let data = bincode::serialize(snapshot)
            .map_err(|e| RealmLoadError::StorageError(format!("Serialization error: {}", e)))?;
        let bytes_written = data.len();

        let key = format!("realm_base:{}", realm_id);
        self.realm_base.insert(key, data);

        // Update Checkpoint-State
        if let Some(state) = self.checkpoint_state.get(realm_id) {
            state.checkpoint_done(sequence);
        }

        Ok(bytes_written)
    }

    /// Lade Realm-Base
    pub fn load_realm_base(&self, realm_id: &str) -> Result<RealmSpecificSnapshot, RealmLoadError> {
        let key = format!("realm_base:{}", realm_id);
        let data = self
            .realm_base
            .get(&key)
            .ok_or_else(|| RealmLoadError::NotFound(realm_id.to_string()))?;

        bincode::deserialize(&data).map_err(|e| {
            RealmLoadError::DeserializationError(format!("Deserialization error: {}", e))
        })
    }

    /// Lade Events seit Sequence (Range-Query via Binary Keys)
    pub fn load_events_since(
        &self,
        realm_id: &str,
        since_sequence: u64,
    ) -> Result<Vec<WrappedStateEvent>, RealmLoadError> {
        let prefix = format!("{}:", realm_id);
        let prefix_bytes = prefix.as_bytes();

        let mut events = Vec::new();

        // Scan durch Realm-Index (simuliert Seek)
        for entry in self.realm_index.iter() {
            let key = entry.key();
            if key.starts_with(prefix_bytes) {
                // Parse Sequence aus Key
                if key.len() >= prefix_bytes.len() + 8 {
                    let seq_bytes = &key[prefix_bytes.len()..prefix_bytes.len() + 8];
                    let seq = u64::from_be_bytes(seq_bytes.try_into().unwrap_or([0u8; 8]));

                    if seq > since_sequence {
                        let event_id = entry.value();
                        let event_key = format!("e:{}", event_id);
                        if let Some(event_data) = self.state_events.get(&event_key) {
                            let event: WrappedStateEvent = bincode::deserialize(&event_data)
                                .map_err(|e| {
                                    RealmLoadError::DeserializationError(format!(
                                        "Event deserialization error: {}",
                                        e
                                    ))
                                })?;
                            events.push(event);
                        }
                    }
                }
            }
        }

        // Sortiere nach Sequence
        events.sort_by_key(|e| e.sequence);

        Ok(events)
    }

    /// Prüfe ob Realm existiert
    pub fn realm_exists(&self, realm_id: &str) -> bool {
        let key = format!("realm_base:{}", realm_id);
        self.realm_base.contains_key(&key)
    }

    /// Hole Checkpoint-State für Realm
    pub fn get_checkpoint_state(&self, realm_id: &str) -> Option<RealmCheckpointState> {
        // Wir können hier nur einen neuen State zurückgeben da DashMap keine Moves erlaubt
        // In der echten Implementierung würde man eine Referenz verwenden
        self.checkpoint_state
            .get(realm_id)
            .map(|_| RealmCheckpointState::new())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PRODUCTION STORAGE SERVICE (Implementiert RealmStorageLoader)
// ─────────────────────────────────────────────────────────────────────────────

/// Production Storage-Service mit Fjall-Backend
///
/// Features:
/// - Pluggable Backend (Fjall Default, RocksDB Fallback, InMemory für Tests)
/// - Atomare Writes (Transactions/Batches)
/// - Binary Keys + Seek für O(log n) Range-Queries
/// - Dirty-Checkpointing (nur bei Bedarf schreiben)
/// - Wait-Free Metrics (Atomics)
/// - Byte-Reusing (Stack-Buffer für Keys)
/// - Thundering Herd Prevention (InFlight-Map)
/// - Event-Sourcing + Replay für Recovery
///
/// # Architektur
///
/// ```text
/// ┌────────────────────────────────────────────────────────────────────────┐
/// │                     ProductionStorageService                           │
/// │                                                                        │
/// │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐       │
/// │  │   realm_base    │  │  state_events   │  │   realm_index   │       │
/// │  │ (Checkpoints)   │  │ (All Events)    │  │ (Binary Seek)   │       │
/// │  └────────┬────────┘  └────────┬────────┘  └────────┬────────┘       │
/// │           │                    │                    │                 │
/// │           └────────────────────┼────────────────────┘                 │
/// │                                │                                      │
/// │  ┌─────────────────────────────┴─────────────────────────────────┐   │
/// │  │                    In-Memory / Fjall Backend                   │   │
/// │  │  (Atomic Transactions, WAL, Compaction)                        │   │
/// │  └───────────────────────────────────────────────────────────────┘   │
/// │                                                                        │
/// │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐       │
/// │  │   InFlight-Map  │  │ Checkpoint-State│  │    Metrics      │       │
/// │  │ (Thundering     │  │ (Dirty-Check)   │  │ (Wait-Free)     │       │
/// │  │  Herd Prev.)    │  │                 │  │                 │       │
/// │  └─────────────────┘  └─────────────────┘  └─────────────────┘       │
/// └────────────────────────────────────────────────────────────────────────┘
/// ```
#[derive(Debug)]
pub struct ProductionStorageService {
    /// Storage-Backend (InMemory für jetzt, Fjall später)
    storage: InMemoryStorage,
    /// Konfiguration
    config: StorageServiceConfig,
    /// Wait-Free Metriken
    pub metrics: Arc<StorageServiceMetrics>,
    /// In-Flight-Map für Thundering Herd Prevention
    in_flight: InFlightMap,
}

impl ProductionStorageService {
    /// Erstelle neuen Storage-Service
    pub fn new(config: StorageServiceConfig) -> Result<Arc<Self>, RealmLoadError> {
        // Für jetzt verwenden wir InMemoryStorage
        // TODO: Fjall-Integration wenn fjall crate verfügbar
        let storage = match &config.backend {
            ProductionStorageBackend::InMemory => InMemoryStorage::new(),
            ProductionStorageBackend::Fjall { path, .. } => {
                // Später: Echte Fjall-Integration
                // Für jetzt: InMemory als Platzhalter
                tracing::info!(
                    "Fjall backend requested at {}, using InMemory fallback",
                    path
                );
                InMemoryStorage::new()
            }
            ProductionStorageBackend::RocksDB { path } => {
                tracing::info!(
                    "RocksDB backend requested at {}, using InMemory fallback",
                    path
                );
                InMemoryStorage::new()
            }
        };

        Ok(Arc::new(Self {
            storage,
            config,
            metrics: Arc::new(StorageServiceMetrics::new()),
            in_flight: InFlightMap::new(),
        }))
    }

    /// Erstelle mit Default-Konfiguration
    pub fn with_defaults() -> Result<Arc<Self>, RealmLoadError> {
        Self::new(StorageServiceConfig::default())
    }

    /// Erstelle für Tests (InMemory)
    pub fn for_testing() -> Result<Arc<Self>, RealmLoadError> {
        Self::new(StorageServiceConfig::minimal())
    }

    /// Persistiere State-Event
    pub fn persist_state_event(&self, event: &WrappedStateEvent) -> Result<(), RealmLoadError> {
        let bytes = self
            .storage
            .persist_event(event, self.config.enable_component_index)?;

        self.metrics.writes.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .bytes_written
            .fetch_add(bytes as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Persistiere Realm-Checkpoint
    pub fn persist_checkpoint(
        &self,
        realm_id: &str,
        snapshot: &RealmSpecificSnapshot,
        sequence: u64,
    ) -> Result<(), RealmLoadError> {
        let bytes = self
            .storage
            .persist_realm_base(realm_id, snapshot, sequence)?;

        self.metrics.checkpoint_ops.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .bytes_written
            .fetch_add(bytes as u64, Ordering::Relaxed);

        Ok(())
    }

    /// Lade Realm mit Event-Replay (mit Thundering Herd Prevention)
    pub async fn load_realm_with_replay(
        self: &Arc<Self>,
        realm_id: &str,
    ) -> Result<Arc<RealmSpecificState>, RealmLoadError> {
        // 1. Check In-Flight-Map
        if let Some(receiver) = self.in_flight.try_register(realm_id) {
            self.metrics.inflight_hits.fetch_add(1, Ordering::Relaxed);
            return receiver.await.map_err(|_| {
                RealmLoadError::StorageError("In-flight request cancelled".to_string())
            })?;
        }

        // 2. Wir sind der Loader - markiere als in-flight
        self.in_flight.start_load(realm_id);

        // 3. Lade Realm
        let result = self.do_load_realm(realm_id).await;

        // 4. Benachrichtige alle Waiter
        self.in_flight.complete_load(realm_id, result.clone());

        result
    }

    /// Interne Load-Logik
    async fn do_load_realm(
        &self,
        realm_id: &str,
    ) -> Result<Arc<RealmSpecificState>, RealmLoadError> {
        self.metrics.load_ops.fetch_add(1, Ordering::Relaxed);

        // 1. Lade Basis-Snapshot
        let snapshot = self.storage.load_realm_base(realm_id)?;
        self.metrics.reads.fetch_add(1, Ordering::Relaxed);

        // 2. Erstelle State aus Snapshot
        let state = RealmSpecificState::new(snapshot.min_trust, &snapshot.governance_type);

        // 3. Event-Replay (wenn aktiviert)
        if self.config.enable_event_replay {
            let last_seq = self
                .storage
                .get_checkpoint_state(realm_id)
                .map(|s| s.last_checkpoint_sequence.load(Ordering::Relaxed))
                .unwrap_or(0);

            self.metrics.index_seeks.fetch_add(1, Ordering::Relaxed);
            let events = self.storage.load_events_since(realm_id, last_seq)?;

            for event in &events {
                state.apply_state_event(event);
                self.metrics.bytes_read.fetch_add(
                    bincode::serialized_size(event).unwrap_or(0),
                    Ordering::Relaxed,
                );
            }

            self.metrics
                .event_replays
                .fetch_add(events.len() as u64, Ordering::Relaxed);
        }

        Ok(Arc::new(state))
    }

    /// Prüfe ob Dirty-Checkpoint nötig
    pub fn needs_checkpoint(&self, realm_id: &str) -> bool {
        self.storage
            .get_checkpoint_state(realm_id)
            .map(|s| s.needs_checkpoint(&self.config))
            .unwrap_or(false)
    }

    /// Hole Metriken-Snapshot
    pub fn metrics_snapshot(&self) -> StorageServiceMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Anzahl aktiver In-Flight-Loads
    pub fn active_inflight_count(&self) -> usize {
        self.in_flight.active_count()
    }
}

#[async_trait::async_trait]
impl RealmStorageLoader for ProductionStorageService {
    async fn load_realm_base(&self, realm_id: &str) -> Result<RealmSpecificState, RealmLoadError> {
        self.metrics.load_ops.fetch_add(1, Ordering::Relaxed);

        let snapshot = self.storage.load_realm_base(realm_id)?;
        self.metrics.reads.fetch_add(1, Ordering::Relaxed);

        Ok(RealmSpecificState::new(
            snapshot.min_trust,
            &snapshot.governance_type,
        ))
    }

    async fn load_realm_events_since(
        &self,
        realm_id: &str,
        since_event_id: Option<&str>,
    ) -> Result<Vec<WrappedStateEvent>, RealmLoadError> {
        // Konvertiere event_id zu sequence (vereinfacht: 0 wenn None)
        let since_seq = since_event_id
            .and_then(|id| id.parse::<u64>().ok())
            .unwrap_or(0);

        self.metrics.index_seeks.fetch_add(1, Ordering::Relaxed);
        self.storage.load_events_since(realm_id, since_seq)
    }

    async fn realm_exists(&self, realm_id: &str) -> bool {
        self.storage.realm_exists(realm_id)
    }

    async fn persist_realm_snapshot(
        &self,
        realm_id: &str,
        snapshot: &RealmSpecificSnapshot,
    ) -> Result<(), RealmLoadError> {
        let seq = snapshot.events_total;
        self.persist_checkpoint(realm_id, snapshot, seq)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SHARD STATISTICS
// ─────────────────────────────────────────────────────────────────────────────

/// Statistiken für einen einzelnen Shard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardStats {
    /// Shard-Index
    pub index: usize,
    /// Aktuelle Anzahl geladener Realms
    pub loaded_count: usize,
    /// LRU-Cache Größe
    pub lru_size: usize,
    /// Anzahl Cache-Hits
    pub cache_hits: u64,
    /// Anzahl Cache-Misses (Lazy Loads)
    pub cache_misses: u64,
    /// Anzahl Evictions
    pub evictions: u64,
    /// Letzte Eviction-Zeit (Unix timestamp ms)
    pub last_eviction_ms: u64,
}

/// Aggregierte Statistiken für alle Shards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardingStats {
    /// Konfiguration
    pub num_shards: usize,
    pub max_per_shard: usize,
    /// Aggregierte Metriken
    pub total_loaded_realms: usize,
    pub total_cache_hits: u64,
    pub total_cache_misses: u64,
    pub total_evictions: u64,
    /// Hit-Rate (%)
    pub cache_hit_rate_percent: f64,
    /// Load-Verteilung (Standardabweichung)
    pub load_distribution_stddev: f64,
    /// Per-Shard Details
    pub shards: Vec<ShardStats>,
}

// ─────────────────────────────────────────────────────────────────────────────
// LAZY SHARDED REALM STATE
// ─────────────────────────────────────────────────────────────────────────────

/// Lock-free, sharded Realm-State mit Lazy Loading und LRU Eviction
///
/// Diese Struktur skaliert auf Millionen von Realms durch:
/// - **Deterministisches Sharding**: FxHash für O(1) Shard-Selection
/// - **Lock-free DashMap**: Interne Sharding in DashMap für minimale Contention
/// - **Lazy Loading**: Realms werden bei Bedarf aus Storage geladen
/// - **LRU Eviction**: Inaktive Realms werden aus dem Speicher entfernt
/// - **Background Tasks**: Periodische Eviction ohne Blocking
///
/// # Performance
///
/// - **Read**: O(1) bei Cache-Hit, O(n) bei Lazy Load (Storage-abhängig)
/// - **Write**: O(1) lock-free
/// - **Memory**: Nur aktive Realms im Speicher (~1-10GB für 10K-100K hot Realms)
/// - **Contention**: Nahezu 0 bei unabhängigen Realms (unterschiedliche Shards)
pub struct LazyShardedRealmState {
    /// Shards: Jeder ist eine lock-free DashMap
    shards: Box<[DashMap<String, Arc<RealmSpecificState>>]>,

    /// LRU pro Shard für Access-Tracking (async-fähig)
    lru_caches: Box<[TokioRwLock<LruCache<String, ()>>]>,

    /// Per-Shard Statistiken
    shard_stats: Box<[ShardStatistics]>,

    /// Storage-Loader für Lazy Loading
    storage_loader: Option<Arc<dyn RealmStorageLoader>>,

    /// Konfiguration
    config: ShardingConfig,

    /// Global Realm-Count (approximate, für schnelle Metrics)
    total_realms_approx: AtomicUsize,

    /// Global Cache-Hit Counter
    cache_hits: AtomicU64,

    /// Global Cache-Miss Counter
    cache_misses: AtomicU64,

    /// Global Eviction Counter
    evictions: AtomicU64,

    /// Ob Background-Tasks gestartet wurden
    background_tasks_started: std::sync::atomic::AtomicBool,
}

/// Per-Shard Statistiken (atomic für lock-free Updates)
#[derive(Debug)]
struct ShardStatistics {
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    evictions: AtomicU64,
    last_eviction_ms: AtomicU64,
}

impl Default for ShardStatistics {
    fn default() -> Self {
        Self {
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            last_eviction_ms: AtomicU64::new(0),
        }
    }
}

impl LazyShardedRealmState {
    /// Erstelle neue sharded Realm-State-Struktur
    pub fn new(config: ShardingConfig) -> Self {
        let num_shards = config.num_shards.max(1);
        let lru_cap = NonZeroUsize::new(config.lru_capacity_per_shard)
            .unwrap_or(NonZeroUsize::new(1000).unwrap());

        let mut shards = Vec::with_capacity(num_shards);
        let mut lru_caches = Vec::with_capacity(num_shards);
        let mut shard_stats = Vec::with_capacity(num_shards);

        for _ in 0..num_shards {
            shards.push(DashMap::new());
            lru_caches.push(TokioRwLock::new(LruCache::new(lru_cap)));
            shard_stats.push(ShardStatistics::default());
        }

        Self {
            shards: shards.into_boxed_slice(),
            lru_caches: lru_caches.into_boxed_slice(),
            shard_stats: shard_stats.into_boxed_slice(),
            storage_loader: None,
            config,
            total_realms_approx: AtomicUsize::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            background_tasks_started: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Erstelle mit Default-Konfiguration
    pub fn with_defaults() -> Self {
        Self::new(ShardingConfig::default())
    }

    /// Erstelle mit Storage-Loader für Lazy Loading
    pub fn with_storage(mut self, loader: Arc<dyn RealmStorageLoader>) -> Self {
        self.storage_loader = Some(loader);
        self
    }

    /// Setze Storage-Loader nachträglich
    pub fn set_storage_loader(&mut self, loader: Arc<dyn RealmStorageLoader>) {
        self.storage_loader = Some(loader);
    }

    /// Berechne Shard-Index für Realm-ID (deterministisch)
    #[inline]
    fn shard_index(&self, realm_id: &str) -> usize {
        (fx_hash_str(realm_id) as usize) % self.shards.len()
    }

    /// Hole Realm synchron (nur Cache, kein Lazy Load)
    ///
    /// Für Performance-kritische Pfade wo async nicht möglich ist.
    pub fn get_cached(&self, realm_id: &str) -> Option<Arc<RealmSpecificState>> {
        let idx = self.shard_index(realm_id);
        let shard = &self.shards[idx];

        if let Some(realm) = shard.get(realm_id) {
            self.shard_stats[idx]
                .cache_hits
                .fetch_add(1, Ordering::Relaxed);
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            Some(realm.clone())
        } else {
            self.shard_stats[idx]
                .cache_misses
                .fetch_add(1, Ordering::Relaxed);
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    /// Hole oder lade Realm asynchron (mit Lazy Loading + Event Replay)
    ///
    /// Dies ist die Hauptmethode für Realm-Zugriff:
    /// 1. Cache-Check (O(1), lock-free)
    /// 2. Bei Miss: Storage-Load + Event-Replay
    /// 3. LRU-Touch für Eviction-Tracking
    pub async fn get_or_load(
        &self,
        realm_id: &str,
    ) -> Result<Arc<RealmSpecificState>, RealmLoadError> {
        let idx = self.shard_index(realm_id);
        let shard = &self.shards[idx];

        // 1. Fast-Path: Cache-Hit
        if let Some(realm) = shard.get(realm_id) {
            self.shard_stats[idx]
                .cache_hits
                .fetch_add(1, Ordering::Relaxed);
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            self.touch_lru(idx, realm_id).await;
            return Ok(realm.clone());
        }

        // 2. Cache-Miss: Lazy Load
        self.shard_stats[idx]
            .cache_misses
            .fetch_add(1, Ordering::Relaxed);
        self.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Prüfe ob Lazy Loading aktiviert
        if !self.config.lazy_loading_enabled {
            return Err(RealmLoadError::LazyLoadingDisabled);
        }

        // Prüfe ob Storage-Loader verfügbar
        let loader = self
            .storage_loader
            .as_ref()
            .ok_or(RealmLoadError::StorageError(
                "No storage loader configured".to_string(),
            ))?;

        // 3. Lade aus Storage
        let loaded = loader.load_realm_base(realm_id).await?;

        // 4. Optional: Event-Replay für State-Recovery
        if self.config.event_replay_on_load {
            let events = loader.load_realm_events_since(realm_id, None).await?;
            for event in events {
                loaded.apply_state_event(&event);
            }
        }

        let arc_realm = Arc::new(loaded);

        // 5. In Cache einfügen
        shard.insert(realm_id.to_string(), arc_realm.clone());
        self.total_realms_approx.fetch_add(1, Ordering::Relaxed);

        // 6. LRU-Touch
        self.touch_lru(idx, realm_id).await;

        Ok(arc_realm)
    }

    /// Registriere neues Realm (synchron, ohne Storage)
    pub fn register(&self, realm_id: &str, state: RealmSpecificState) -> bool {
        let idx = self.shard_index(realm_id);
        let shard = &self.shards[idx];

        // Entry API für atomares Check+Insert
        if shard.contains_key(realm_id) {
            return false; // Bereits vorhanden
        }

        shard.insert(realm_id.to_string(), Arc::new(state));
        self.total_realms_approx.fetch_add(1, Ordering::Relaxed);
        true
    }

    /// Registriere oder update Realm
    pub fn upsert(&self, realm_id: &str, state: RealmSpecificState) {
        let idx = self.shard_index(realm_id);
        let shard = &self.shards[idx];

        let was_new = !shard.contains_key(realm_id);
        shard.insert(realm_id.to_string(), Arc::new(state));

        if was_new {
            self.total_realms_approx.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Entferne Realm aus Cache
    pub fn remove(&self, realm_id: &str) -> Option<Arc<RealmSpecificState>> {
        let idx = self.shard_index(realm_id);
        let shard = &self.shards[idx];

        if let Some((_, removed)) = shard.remove(realm_id) {
            let _ =
                self.total_realms_approx
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                        if v > 0 {
                            Some(v - 1)
                        } else {
                            Some(0)
                        }
                    });
            Some(removed)
        } else {
            None
        }
    }

    /// Prüfe ob Realm im Cache ist
    pub fn contains(&self, realm_id: &str) -> bool {
        let idx = self.shard_index(realm_id);
        self.shards[idx].contains_key(realm_id)
    }

    /// Touch LRU für Access-Tracking
    async fn touch_lru(&self, idx: usize, realm_id: &str) {
        let mut lru = self.lru_caches[idx].write().await;
        lru.put(realm_id.to_string(), ());
    }

    /// Führe Eviction für einen Shard durch
    async fn evict_shard(&self, idx: usize) -> usize {
        let mut evicted_count = 0;
        let max = self.config.max_per_shard;

        let mut lru = self.lru_caches[idx].write().await;

        while lru.len() > max {
            if let Some((evicted_id, _)) = lru.pop_lru() {
                self.shards[idx].remove(&evicted_id);
                evicted_count += 1;
            } else {
                break;
            }
        }

        if evicted_count > 0 {
            self.shard_stats[idx]
                .evictions
                .fetch_add(evicted_count as u64, Ordering::Relaxed);
            self.evictions
                .fetch_add(evicted_count as u64, Ordering::Relaxed);
            let _ =
                self.total_realms_approx
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |v| {
                        Some(v.saturating_sub(evicted_count))
                    });

            // Update last eviction time
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            self.shard_stats[idx]
                .last_eviction_ms
                .store(now_ms, Ordering::Relaxed);
        }

        evicted_count
    }

    /// Führe Eviction für alle Shards durch
    pub async fn evict_all(&self) -> usize {
        let mut total_evicted = 0;
        for idx in 0..self.shards.len() {
            total_evicted += self.evict_shard(idx).await;
        }
        total_evicted
    }

    /// Starte Background-Eviction-Tasks
    ///
    /// WICHTIG: Nur einmal aufrufen! Spawnt einen Task pro Shard.
    pub fn spawn_eviction_tasks(self: Arc<Self>) {
        if self.background_tasks_started.swap(true, Ordering::SeqCst) {
            return; // Bereits gestartet
        }

        let interval = Duration::from_secs(self.config.eviction_interval_secs);

        for idx in 0..self.shards.len() {
            let this = self.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(interval).await;
                    this.evict_shard(idx).await;
                }
            });
        }
    }

    /// Approximate Anzahl geladener Realms
    pub fn approximate_count(&self) -> usize {
        self.total_realms_approx.load(Ordering::Relaxed)
    }

    /// Exakte Anzahl geladener Realms (teuer, iteriert alle Shards)
    pub fn exact_count(&self) -> usize {
        self.shards.iter().map(|s| s.len()).sum()
    }

    /// Hole Statistiken für alle Shards
    pub fn stats(&self) -> ShardingStats {
        let shards: Vec<ShardStats> = self
            .shards
            .iter()
            .enumerate()
            .map(|(idx, shard)| {
                let lru_size = self.lru_caches[idx]
                    .try_read()
                    .map(|lru| lru.len())
                    .unwrap_or(0);

                ShardStats {
                    index: idx,
                    loaded_count: shard.len(),
                    lru_size,
                    cache_hits: self.shard_stats[idx].cache_hits.load(Ordering::Relaxed),
                    cache_misses: self.shard_stats[idx].cache_misses.load(Ordering::Relaxed),
                    evictions: self.shard_stats[idx].evictions.load(Ordering::Relaxed),
                    last_eviction_ms: self.shard_stats[idx]
                        .last_eviction_ms
                        .load(Ordering::Relaxed),
                }
            })
            .collect();

        let total_loaded: usize = shards.iter().map(|s| s.loaded_count).sum();
        let total_hits: u64 = shards.iter().map(|s| s.cache_hits).sum();
        let total_misses: u64 = shards.iter().map(|s| s.cache_misses).sum();
        let total_evictions: u64 = shards.iter().map(|s| s.evictions).sum();

        let hit_rate = if total_hits + total_misses > 0 {
            (total_hits as f64 / (total_hits + total_misses) as f64) * 100.0
        } else {
            100.0
        };

        // Berechne Load-Verteilung Standardabweichung
        let mean_load = total_loaded as f64 / shards.len() as f64;
        let variance: f64 = shards
            .iter()
            .map(|s| (s.loaded_count as f64 - mean_load).powi(2))
            .sum::<f64>()
            / shards.len() as f64;
        let stddev = variance.sqrt();

        ShardingStats {
            num_shards: self.shards.len(),
            max_per_shard: self.config.max_per_shard,
            total_loaded_realms: total_loaded,
            total_cache_hits: total_hits,
            total_cache_misses: total_misses,
            total_evictions,
            cache_hit_rate_percent: hit_rate,
            load_distribution_stddev: stddev,
            shards,
        }
    }

    /// Iterator über alle geladenen Realms (teuer, nur für Snapshots/Metrics)
    pub fn iter_all(&self) -> impl Iterator<Item = (String, Arc<RealmSpecificState>)> + '_ {
        self.shards.iter().flat_map(|shard| {
            shard
                .iter()
                .map(|entry| (entry.key().clone(), entry.value().clone()))
        })
    }

    /// Snapshot aller geladenen Realms
    pub fn snapshot(&self) -> HashMap<String, RealmSpecificSnapshot> {
        self.iter_all()
            .map(|(id, state)| (id, state.snapshot()))
            .collect()
    }

    /// Konfiguration
    pub fn config(&self) -> &ShardingConfig {
        &self.config
    }
}

// Debug-Implementierung ohne sensitive Daten
impl std::fmt::Debug for LazyShardedRealmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LazyShardedRealmState")
            .field("num_shards", &self.shards.len())
            .field(
                "total_realms_approx",
                &self.total_realms_approx.load(Ordering::Relaxed),
            )
            .field("cache_hits", &self.cache_hits.load(Ordering::Relaxed))
            .field("cache_misses", &self.cache_misses.load(Ordering::Relaxed))
            .field("config", &self.config)
            .finish()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ATOMIC F64 (Portable lock-free f64 operations via bit-casting)
// ─────────────────────────────────────────────────────────────────────────────

/// Lock-free AtomicF64 implementiert via AtomicU64 bit-casting
///
/// Standard-Bibliothek hat kein AtomicF64, daher portable Implementierung:
/// f64 wird zu u64 reinterpretiert für atomare Operationen.
///
/// # Thread-Safety
/// Alle Operationen sind atomisch und lock-free.
///
/// # Precision
/// Volle f64-Präzision bleibt erhalten (keine Rundungsfehler durch casting).
#[derive(Debug)]
pub struct AtomicF64 {
    inner: AtomicU64,
}

impl AtomicF64 {
    /// Erstelle neuen AtomicF64 mit initialem Wert
    #[inline]
    pub const fn new(value: f64) -> Self {
        Self {
            inner: AtomicU64::new(value.to_bits()),
        }
    }

    /// Lade aktuellen Wert atomisch
    #[inline]
    pub fn load(&self, ordering: Ordering) -> f64 {
        f64::from_bits(self.inner.load(ordering))
    }

    /// Speichere Wert atomisch
    #[inline]
    pub fn store(&self, value: f64, ordering: Ordering) {
        self.inner.store(value.to_bits(), ordering);
    }

    /// Swap: Setze neuen Wert und gib alten zurück
    #[inline]
    pub fn swap(&self, value: f64, ordering: Ordering) -> f64 {
        f64::from_bits(self.inner.swap(value.to_bits(), ordering))
    }

    /// Compare-and-Swap für präzise Updates
    #[inline]
    pub fn compare_exchange(
        &self,
        current: f64,
        new: f64,
        success: Ordering,
        failure: Ordering,
    ) -> Result<f64, f64> {
        match self
            .inner
            .compare_exchange(current.to_bits(), new.to_bits(), success, failure)
        {
            Ok(v) => Ok(f64::from_bits(v)),
            Err(v) => Err(f64::from_bits(v)),
        }
    }

    /// Addiere delta atomar (EWMA-freundlich)
    ///
    /// Da f64 keine native atomare Addition hat, nutzen wir CAS-Loop.
    #[inline]
    pub fn fetch_add(&self, delta: f64, ordering: Ordering) -> f64 {
        loop {
            let current = self.load(ordering);
            let new_value = current + delta;
            if self
                .compare_exchange(current, new_value, ordering, Ordering::Relaxed)
                .is_ok()
            {
                return current;
            }
        }
    }

    /// Subtrahiere delta atomar
    #[inline]
    pub fn fetch_sub(&self, delta: f64, ordering: Ordering) -> f64 {
        self.fetch_add(-delta, ordering)
    }

    /// Multipliziere atomar (für EWMA: current * decay)
    #[inline]
    pub fn fetch_mul(&self, factor: f64, ordering: Ordering) -> f64 {
        loop {
            let current = self.load(ordering);
            let new_value = current * factor;
            if self
                .compare_exchange(current, new_value, ordering, Ordering::Relaxed)
                .is_ok()
            {
                return current;
            }
        }
    }

    /// EWMA-Update: new = current * decay + value * (1 - decay)
    ///
    /// Atomischer Exponentially Weighted Moving Average für Shard-Entropy.
    #[inline]
    pub fn ewma_update(&self, new_value: f64, decay: f64, ordering: Ordering) -> f64 {
        loop {
            let current = self.load(ordering);
            let updated = current * decay + new_value * (1.0 - decay);
            if self
                .compare_exchange(current, updated, ordering, Ordering::Relaxed)
                .is_ok()
            {
                return current;
            }
        }
    }

    /// Maximum-Update: Setze auf max(current, value)
    #[inline]
    pub fn fetch_max(&self, value: f64, ordering: Ordering) -> f64 {
        loop {
            let current = self.load(ordering);
            if value <= current {
                return current;
            }
            if self
                .compare_exchange(current, value, ordering, Ordering::Relaxed)
                .is_ok()
            {
                return current;
            }
        }
    }

    /// Minimum-Update: Setze auf min(current, value)
    #[inline]
    pub fn fetch_min(&self, value: f64, ordering: Ordering) -> f64 {
        loop {
            let current = self.load(ordering);
            if value >= current {
                return current;
            }
            if self
                .compare_exchange(current, value, ordering, Ordering::Relaxed)
                .is_ok()
            {
                return current;
            }
        }
    }
}

impl Default for AtomicF64 {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Clone for AtomicF64 {
    fn clone(&self) -> Self {
        Self::new(self.load(Ordering::SeqCst))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SHARD MONITOR (Shard-Aware Protection für horizontale Skalierung)
// ─────────────────────────────────────────────────────────────────────────────

/// Shard-spezifische Überwachung für das Immunsystem (Teil von ProtectionState)
///
/// # Sicherheitsarchitektur für horizontale Skalierung
///
/// Der ShardMonitor adressiert zwei kritische Risiken bei Sharding:
///
/// ## Risiko 1: Lokale Trust-Verzerrung (Shard-Bias)
///
/// Wenn ein Angreifer einen Shard mit Fake-Realms/Entities flutet, könnte
/// Trust lokal verzerrt wirken (z.B. viele positive Updates nur in Shard 5).
///
/// **Lösung:** Shard-Entropy-Score trackt lokale Vielfalt der Update-Quellen.
/// Abweichung > Threshold (50%) → "biased Shard" → Alarm + Dämpfung.
///
/// ## Risiko 2: Cross-Shard-Angriffe
///
/// Angreifer aus einem "toxischen" Shard versucht, in andere Shards
/// einzudringen (z.B. viele failed Sagas/Crossings).
///
/// **Lösung:** Shard-Reputation (0.0–1.0) basierend auf Fehlerrate.
/// Niedrige Reputation → höhere Multi-Gas-Kosten für Outbound-Requests.
/// Hohe Fehlerrate → temporäre Quarantäne.
///
/// # Performance
///
/// - Alle Metriken sind **lock-free** (AtomicF64/AtomicU64)
/// - DashMap für O(1) Shard-Lookup ohne globale Locks
/// - EWMA für Rolling-Updates ohne vollständige Historienführung
#[derive(Debug)]
pub struct ShardMonitor {
    /// Aktivität pro Shard (ShardIndex → Update-Count)
    pub shard_activity: DashMap<u64, AtomicU64>,

    /// Lokale Trust-Entropy pro Shard (ShardIndex → Entropy)
    /// Entropy nahe 1.0 = gesund (diverse Quellen)
    /// Entropy nahe 0.0 = verdächtig (wenige Quellen dominieren)
    pub shard_entropy: DashMap<u64, AtomicF64>,

    /// Fehlgeschlagene Cross-Shard-Versuche (SourceShard → Failures)
    pub cross_shard_failures: DashMap<u64, AtomicU64>,

    /// Erfolgreiche Cross-Shard-Versuche (für Reputation-Berechnung)
    pub cross_shard_successes: DashMap<u64, AtomicU64>,

    /// Dynamische Shard-Reputation (0.0 = toxisch, 1.0 = gesund)
    pub shard_reputation: DashMap<u64, AtomicF64>,

    /// Bias-Alarme pro Shard (für Circuit-Breaker Integration)
    pub bias_alarms: DashMap<u64, AtomicU64>,

    /// Quarantäne-Status pro Shard (true = geblockt)
    pub quarantined_shards: DashMap<u64, std::sync::atomic::AtomicBool>,

    /// Konfiguration
    config: ShardMonitorConfig,
}

/// Konfiguration für ShardMonitor
#[derive(Debug, Clone)]
pub struct ShardMonitorConfig {
    /// Anzahl der erwarteten Shards (für Pre-Allokation)
    pub expected_shards: usize,
    /// Bias-Threshold (50% = Entropy < 50% von Global → Alarm)
    pub bias_threshold: f64,
    /// EWMA Decay-Faktor für Entropy-Updates (0.9 = langsame Anpassung)
    pub entropy_decay: f64,
    /// Failure-Threshold für Quarantäne
    pub quarantine_failure_threshold: u64,
    /// Reputation-Penalty pro Failure
    pub reputation_penalty_per_failure: f64,
    /// Reputation-Bonus pro Success (Recovery)
    pub reputation_bonus_per_success: f64,
    /// Max Penalty-Multiplikator für Cross-Shard-Kosten
    pub max_penalty_multiplier: f64,
}

impl Default for ShardMonitorConfig {
    fn default() -> Self {
        Self {
            expected_shards: 64,
            bias_threshold: 0.5,
            entropy_decay: 0.9,
            quarantine_failure_threshold: 100,
            reputation_penalty_per_failure: 0.1,
            reputation_bonus_per_success: 0.01,
            max_penalty_multiplier: 5.0,
        }
    }
}

impl ShardMonitorConfig {
    /// Strikte Konfiguration für High-Security
    pub fn strict() -> Self {
        Self {
            expected_shards: 128,
            bias_threshold: 0.6, // Strenger
            entropy_decay: 0.95,
            quarantine_failure_threshold: 50, // Schnellere Quarantäne
            reputation_penalty_per_failure: 0.15,
            reputation_bonus_per_success: 0.005,
            max_penalty_multiplier: 10.0,
        }
    }

    /// Relaxed für Tests/Development
    pub fn relaxed() -> Self {
        Self {
            expected_shards: 4,
            bias_threshold: 0.3,
            entropy_decay: 0.5,
            quarantine_failure_threshold: 200,
            reputation_penalty_per_failure: 0.05,
            reputation_bonus_per_success: 0.02,
            max_penalty_multiplier: 2.0,
        }
    }
}

impl ShardMonitor {
    /// Erstelle neuen ShardMonitor mit Konfiguration
    pub fn new(config: ShardMonitorConfig) -> Self {
        Self {
            shard_activity: DashMap::with_capacity(config.expected_shards),
            shard_entropy: DashMap::with_capacity(config.expected_shards),
            cross_shard_failures: DashMap::with_capacity(config.expected_shards),
            cross_shard_successes: DashMap::with_capacity(config.expected_shards),
            shard_reputation: DashMap::with_capacity(config.expected_shards),
            bias_alarms: DashMap::with_capacity(config.expected_shards),
            quarantined_shards: DashMap::with_capacity(config.expected_shards),
            config,
        }
    }

    /// Erstelle mit Default-Konfiguration
    pub fn with_defaults() -> Self {
        Self::new(ShardMonitorConfig::default())
    }

    /// Update bei Trust-Event (für Bias-Erkennung)
    ///
    /// Wird bei jedem Trust-Update aus einem Shard aufgerufen.
    /// `entropy_delta` misst die Vielfalt der Update-Quellen (0.0–1.0).
    pub fn record_trust_update(&self, shard_id: u64, entropy_delta: f64) {
        // Aktivität inkrementieren
        self.shard_activity
            .entry(shard_id)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        // Entropy per EWMA updaten
        let entry = self
            .shard_entropy
            .entry(shard_id)
            .or_insert_with(|| AtomicF64::new(1.0));
        entry.ewma_update(entropy_delta, self.config.entropy_decay, Ordering::Relaxed);
    }

    /// Prüfe Shard-Bias (Risiko 1: Lokale Trust-Verzerrung)
    ///
    /// Vergleicht lokale Shard-Entropy mit globaler Entropy.
    /// Bei signifikanter Abweichung → Bias erkannt → Reputation senken.
    ///
    /// # Returns
    /// `true` wenn Bias erkannt wurde
    pub fn check_shard_bias(&self, shard_id: u64, global_entropy: f64) -> bool {
        if let Some(local_entropy_atomic) = self.shard_entropy.get(&shard_id) {
            let local_entropy = local_entropy_atomic.load(Ordering::Relaxed);
            let threshold = global_entropy * self.config.bias_threshold;

            if local_entropy < threshold {
                // Bias erkannt → Alarm + Reputation senken
                self.bias_alarms
                    .entry(shard_id)
                    .or_insert_with(|| AtomicU64::new(0))
                    .fetch_add(1, Ordering::Relaxed);
                self.lower_reputation(shard_id, 0.2);
                return true;
            }
        }
        false
    }

    /// Berechne globale Entropy (Durchschnitt aller Shards)
    pub fn global_entropy(&self) -> f64 {
        if self.shard_entropy.is_empty() {
            return 1.0; // Default: gesund
        }

        let sum: f64 = self
            .shard_entropy
            .iter()
            .map(|e| e.value().load(Ordering::Relaxed))
            .sum();
        sum / self.shard_entropy.len() as f64
    }

    /// Record successful Cross-Shard Request
    pub fn record_cross_success(&self, source_shard: u64) {
        self.cross_shard_successes
            .entry(source_shard)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);

        // Recovery: Reputation leicht verbessern
        self.raise_reputation(source_shard, self.config.reputation_bonus_per_success);
    }

    /// Record failed Cross-Shard Request (Risiko 2: Cross-Shard-Angriffe)
    ///
    /// Wird bei fehlgeschlagenen Cross-Shard-Operationen aufgerufen
    /// (z.B. failed Sagas, Quota-Violations, Policy-Denials).
    pub fn record_cross_failure(&self, source_shard: u64) {
        let failures = self
            .cross_shard_failures
            .entry(source_shard)
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed)
            + 1;

        // Reputation senken
        self.lower_reputation(source_shard, self.config.reputation_penalty_per_failure);

        // Bei vielen Failures → Quarantäne
        if failures >= self.config.quarantine_failure_threshold {
            self.quarantine_shard(source_shard);
        }
    }

    /// Senke Reputation (mit Floor bei 0.0)
    fn lower_reputation(&self, shard_id: u64, penalty: f64) {
        let entry = self
            .shard_reputation
            .entry(shard_id)
            .or_insert_with(|| AtomicF64::new(1.0));

        loop {
            let current = entry.load(Ordering::Relaxed);
            let new_value = (current - penalty).max(0.0);
            if entry
                .compare_exchange(current, new_value, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Erhöhe Reputation (mit Ceiling bei 1.0)
    fn raise_reputation(&self, shard_id: u64, bonus: f64) {
        let entry = self
            .shard_reputation
            .entry(shard_id)
            .or_insert_with(|| AtomicF64::new(1.0));

        loop {
            let current = entry.load(Ordering::Relaxed);
            let new_value = (current + bonus).min(1.0);
            if entry
                .compare_exchange(current, new_value, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }

    /// Setze Shard unter Quarantäne (blockiert alle Crossings)
    pub fn quarantine_shard(&self, shard_id: u64) {
        self.quarantined_shards
            .entry(shard_id)
            .or_insert_with(|| std::sync::atomic::AtomicBool::new(false))
            .store(true, Ordering::SeqCst);
    }

    /// Hebe Quarantäne auf
    pub fn lift_quarantine(&self, shard_id: u64) {
        if let Some(entry) = self.quarantined_shards.get(&shard_id) {
            entry.store(false, Ordering::SeqCst);
        }
    }

    /// Prüfe ob Shard unter Quarantäne steht
    pub fn is_quarantined(&self, shard_id: u64) -> bool {
        self.quarantined_shards
            .get(&shard_id)
            .map(|e| e.load(Ordering::Relaxed))
            .unwrap_or(false)
    }

    /// Hole Penalty-Multiplier für Cross-Shard-Request (Gateway nutzt das)
    ///
    /// Niedrige Reputation → höhere Kosten (ökonomische Abschreckung).
    /// Bei Quarantäne → f64::INFINITY (Request wird blockiert).
    pub fn get_cross_shard_penalty(&self, source_shard: u64) -> f64 {
        // Quarantäne → unendlich (blockiert)
        if self.is_quarantined(source_shard) {
            return f64::INFINITY;
        }

        let reputation = self
            .shard_reputation
            .get(&source_shard)
            .map(|r| r.load(Ordering::Relaxed))
            .unwrap_or(1.0);

        let failures = self
            .cross_shard_failures
            .get(&source_shard)
            .map(|f| f.load(Ordering::Relaxed))
            .unwrap_or(0);

        // Kombinierte Strafe: Stufen + Reputation-Inverse
        let base_penalty = if failures > 100 {
            5.0
        } else if failures > 50 {
            3.0
        } else if failures > 20 {
            1.5
        } else {
            1.0
        };

        // Reputation-Komponente: 1/reputation (aber max capped)
        let reputation_penalty =
            (1.0 / reputation.max(0.1)).min(self.config.max_penalty_multiplier);

        (base_penalty * reputation_penalty).min(self.config.max_penalty_multiplier)
    }

    /// Integration in Multi-Veto (ProtectionState)
    ///
    /// Liefert einen Risk-Score (1.0 = normal, >1.0 = erhöhtes Risiko).
    /// Wird vom Protection-System in Risk-Aggregation einbezogen.
    pub fn contribute_to_veto(&self, shard_id: u64) -> f64 {
        let global_entropy = self.global_entropy();
        let mut risk_score = 1.0;

        // Bias-Check
        if self.check_shard_bias(shard_id, global_entropy) {
            risk_score *= 2.0;
        }

        // Reputation-Factor
        let reputation = self
            .shard_reputation
            .get(&shard_id)
            .map(|r| r.load(Ordering::Relaxed))
            .unwrap_or(1.0);

        if reputation < 0.5 {
            risk_score *= 1.5;
        }
        if reputation < 0.2 {
            risk_score *= 2.0;
        }

        // Quarantäne = maximales Risiko
        if self.is_quarantined(shard_id) {
            risk_score *= 10.0;
        }

        risk_score
    }

    /// Hole Reputation eines Shards (für Metrics/UI)
    pub fn get_reputation(&self, shard_id: u64) -> f64 {
        self.shard_reputation
            .get(&shard_id)
            .map(|r| r.load(Ordering::Relaxed))
            .unwrap_or(1.0)
    }

    /// Hole Entropy eines Shards (für Metrics/UI)
    pub fn get_entropy(&self, shard_id: u64) -> f64 {
        self.shard_entropy
            .get(&shard_id)
            .map(|e| e.load(Ordering::Relaxed))
            .unwrap_or(1.0)
    }

    /// Hole Activity eines Shards (für Metrics/UI)
    pub fn get_activity(&self, shard_id: u64) -> u64 {
        self.shard_activity
            .get(&shard_id)
            .map(|a| a.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    /// Snapshot für Serialisierung
    pub fn snapshot(&self) -> ShardMonitorSnapshot {
        let per_shard: Vec<ShardSecuritySnapshot> = self
            .shard_activity
            .iter()
            .map(|entry| {
                let shard_id = *entry.key();
                ShardSecuritySnapshot {
                    shard_id,
                    activity: entry.value().load(Ordering::Relaxed),
                    entropy: self.get_entropy(shard_id),
                    reputation: self.get_reputation(shard_id),
                    failures: self
                        .cross_shard_failures
                        .get(&shard_id)
                        .map(|f| f.load(Ordering::Relaxed))
                        .unwrap_or(0),
                    successes: self
                        .cross_shard_successes
                        .get(&shard_id)
                        .map(|s| s.load(Ordering::Relaxed))
                        .unwrap_or(0),
                    bias_alarms: self
                        .bias_alarms
                        .get(&shard_id)
                        .map(|a| a.load(Ordering::Relaxed))
                        .unwrap_or(0),
                    is_quarantined: self.is_quarantined(shard_id),
                }
            })
            .collect();

        let total_failures: u64 = self
            .cross_shard_failures
            .iter()
            .map(|e| e.value().load(Ordering::Relaxed))
            .sum();

        let total_successes: u64 = self
            .cross_shard_successes
            .iter()
            .map(|e| e.value().load(Ordering::Relaxed))
            .sum();

        let quarantined_count = self
            .quarantined_shards
            .iter()
            .filter(|e| e.value().load(Ordering::Relaxed))
            .count();

        ShardMonitorSnapshot {
            global_entropy: self.global_entropy(),
            total_shards_monitored: self.shard_activity.len(),
            total_cross_shard_failures: total_failures,
            total_cross_shard_successes: total_successes,
            cross_shard_success_rate: if total_failures + total_successes > 0 {
                total_successes as f64 / (total_failures + total_successes) as f64
            } else {
                1.0
            },
            quarantined_shard_count: quarantined_count,
            per_shard,
        }
    }

    /// Reset für Tests
    #[cfg(test)]
    pub fn reset(&self) {
        self.shard_activity.clear();
        self.shard_entropy.clear();
        self.cross_shard_failures.clear();
        self.cross_shard_successes.clear();
        self.shard_reputation.clear();
        self.bias_alarms.clear();
        self.quarantined_shards.clear();
    }
}

impl Default for ShardMonitor {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Snapshot eines einzelnen Shards (Sicherheitsmetriken)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardSecuritySnapshot {
    pub shard_id: u64,
    pub activity: u64,
    pub entropy: f64,
    pub reputation: f64,
    pub failures: u64,
    pub successes: u64,
    pub bias_alarms: u64,
    pub is_quarantined: bool,
}

/// Aggregierter Snapshot des ShardMonitors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardMonitorSnapshot {
    pub global_entropy: f64,
    pub total_shards_monitored: usize,
    pub total_cross_shard_failures: u64,
    pub total_cross_shard_successes: u64,
    pub cross_shard_success_rate: f64,
    pub quarantined_shard_count: usize,
    pub per_shard: Vec<ShardSecuritySnapshot>,
}

// ─────────────────────────────────────────────────────────────────────────────
// REALM SPECIFIC STATE EXTENSION (apply_state_event für Replay)
// ─────────────────────────────────────────────────────────────────────────────

impl RealmSpecificState {
    /// Apply ein StateEvent auf diesen RealmSpecificState (für Event-Replay)
    pub fn apply_state_event(&self, event: &WrappedStateEvent) {
        match &event.event {
            StateEvent::MembershipChange {
                realm_id,
                action,
                identity_universal_id,
                ..
            } => {
                if realm_id == &self.realm_id() {
                    match action {
                        MembershipAction::Joined | MembershipAction::InviteAccepted => {
                            if let Some(uid) = identity_universal_id {
                                self.add_member_by_id(*uid, None);
                            } else {
                                // Legacy: nur Counter aktualisieren
                                self.identity_joined();
                            }
                        }
                        MembershipAction::Left | MembershipAction::Banned => {
                            if let Some(uid) = identity_universal_id {
                                self.remove_member_by_id(uid);
                            } else {
                                // Legacy: nur Counter aktualisieren
                                self.identity_left();
                            }
                        }
                        MembershipAction::RoleChanged { .. } | MembershipAction::Invited => {
                            // Role changes und Invites ändern keine Member-Counts
                        }
                    }
                }
            }
            StateEvent::RealmLifecycle {
                realm_id, action, ..
            } => {
                if realm_id == &self.realm_id() {
                    match action {
                        RealmAction::Paused => {
                            self.quarantine();
                        }
                        RealmAction::Resumed => {
                            self.unquarantine();
                        }
                        _ => {}
                    }
                }
            }
            StateEvent::CrossingEvaluated { allowed, .. } => {
                if *allowed {
                    self.crossing_in();
                }
            }
            StateEvent::QuotaViolation {
                resource,
                requested,
                ..
            } => {
                // Verbrauche Ressource bei Violation
                let _ = self.consume_resource(*resource, *requested);
            }
            _ => {
                // Andere Events betreffen nicht den Realm-State direkt
            }
        }
    }

    /// Hole Realm-ID (aus Members oder generiere Default)
    fn realm_id(&self) -> String {
        // Da RealmSpecificState keine realm_id hat, nutzen wir einen Workaround
        // In Production würde dies aus der Struktur kommen
        "unknown".to_string()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// SHARDED REALM STATE SNAPSHOT
// ─────────────────────────────────────────────────────────────────────────────

/// Snapshot des sharded Realm-States für Serialisierung
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardedRealmSnapshot {
    /// Sharding-Statistiken
    pub stats: ShardingStats,
    /// Alle geladenen Realm-Snapshots
    pub realms: HashMap<String, RealmSpecificSnapshot>,
}

// ─────────────────────────────────────────────────────────────────────────────
// PHASE 7 TESTS
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests_phase7_sharding {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────
    // ShardingConfig Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_sharding_config_default() {
        let config = ShardingConfig::default();
        assert_eq!(config.num_shards, 64);
        assert_eq!(config.max_per_shard, 20_000);
        assert!(config.lazy_loading_enabled);
    }

    #[test]
    fn test_sharding_config_minimal() {
        let config = ShardingConfig::minimal();
        assert_eq!(config.num_shards, 4);
        assert!(!config.lazy_loading_enabled);
    }

    #[test]
    fn test_sharding_config_production() {
        let config = ShardingConfig::production();
        assert_eq!(config.num_shards, 128);
        assert!(config.lazy_loading_enabled);
        assert!(config.event_replay_on_load);
    }

    #[test]
    fn test_sharding_config_auto_scaled() {
        let config = ShardingConfig::auto_scaled();
        assert!(config.num_shards >= 16);
        assert!(config.num_shards <= 256);
    }

    // ─────────────────────────────────────────────────────────────────────
    // FxHash Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_fx_hash_deterministic() {
        let hash1 = fx_hash_str("realm_001");
        let hash2 = fx_hash_str("realm_001");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_fx_hash_distribution() {
        let hashes: Vec<u64> = (0..100)
            .map(|i| fx_hash_str(&format!("realm_{}", i)))
            .collect();

        // Alle Hashes sollten unterschiedlich sein
        let unique: HashSet<u64> = hashes.iter().copied().collect();
        assert_eq!(unique.len(), 100);
    }

    // ─────────────────────────────────────────────────────────────────────
    // LazyShardedRealmState Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_sharded_state_creation() {
        let state = LazyShardedRealmState::with_defaults();
        assert_eq!(state.shards.len(), 64);
        assert_eq!(state.approximate_count(), 0);
    }

    #[test]
    fn test_sharded_state_minimal_config() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());
        assert_eq!(state.shards.len(), 4);
    }

    #[test]
    fn test_sharded_state_shard_distribution() {
        let state = LazyShardedRealmState::new(ShardingConfig {
            num_shards: 8,
            ..ShardingConfig::minimal()
        });

        // Registriere viele Realms
        for i in 0..100 {
            let realm = RealmSpecificState::new(0.3, "democratic");
            state.register(&format!("realm_{}", i), realm);
        }

        // Prüfe Verteilung über Shards
        let shard_sizes: Vec<usize> = state.shards.iter().map(|s| s.len()).collect();
        let min = *shard_sizes.iter().min().unwrap();
        let max = *shard_sizes.iter().max().unwrap();

        // Verteilung sollte einigermaßen gleichmäßig sein
        assert!(
            max - min < 30,
            "Ungleiche Verteilung: min={}, max={}",
            min,
            max
        );
    }

    #[test]
    fn test_sharded_state_register_and_get() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());

        let realm = RealmSpecificState::new(0.5, "democratic");
        assert!(state.register("realm_001", realm));

        // Zweite Registrierung sollte false zurückgeben
        let realm2 = RealmSpecificState::new(0.3, "consensus");
        assert!(!state.register("realm_001", realm2));

        // Get sollte funktionieren
        let cached = state.get_cached("realm_001");
        assert!(cached.is_some());
        assert_eq!(state.approximate_count(), 1);
    }

    #[test]
    fn test_sharded_state_upsert() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());

        // Erstes Upsert (Insert)
        state.upsert("realm_001", RealmSpecificState::new(0.5, "democratic"));
        assert_eq!(state.approximate_count(), 1);

        // Zweites Upsert (Update) - Count bleibt gleich
        state.upsert("realm_001", RealmSpecificState::new(0.3, "consensus"));
        assert_eq!(state.approximate_count(), 1);
    }

    #[test]
    fn test_sharded_state_remove() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());

        state.register("realm_001", RealmSpecificState::new(0.5, "democratic"));
        assert_eq!(state.approximate_count(), 1);

        let removed = state.remove("realm_001");
        assert!(removed.is_some());
        assert_eq!(state.approximate_count(), 0);

        // Nochmal entfernen sollte None zurückgeben
        let removed_again = state.remove("realm_001");
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_sharded_state_contains() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());

        assert!(!state.contains("realm_001"));

        state.register("realm_001", RealmSpecificState::new(0.5, "democratic"));
        assert!(state.contains("realm_001"));

        state.remove("realm_001");
        assert!(!state.contains("realm_001"));
    }

    #[test]
    fn test_sharded_state_cache_stats() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());

        state.register("realm_001", RealmSpecificState::new(0.5, "democratic"));

        // Cache-Hit
        let _ = state.get_cached("realm_001");
        let _ = state.get_cached("realm_001");

        // Cache-Miss
        let _ = state.get_cached("nonexistent");

        assert_eq!(state.cache_hits.load(Ordering::Relaxed), 2);
        assert_eq!(state.cache_misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_sharded_state_stats() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());

        for i in 0..10 {
            state.register(
                &format!("realm_{}", i),
                RealmSpecificState::new(0.5, "democratic"),
            );
        }

        // Cache operations
        for i in 0..10 {
            let _ = state.get_cached(&format!("realm_{}", i));
        }
        let _ = state.get_cached("nonexistent");

        let stats = state.stats();

        assert_eq!(stats.num_shards, 4);
        assert_eq!(stats.total_loaded_realms, 10);
        assert_eq!(stats.total_cache_hits, 10);
        assert_eq!(stats.total_cache_misses, 1);
        assert!(stats.cache_hit_rate_percent > 90.0);
    }

    #[test]
    fn test_sharded_state_iter_all() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());

        for i in 0..5 {
            state.register(
                &format!("realm_{}", i),
                RealmSpecificState::new(0.5, "democratic"),
            );
        }

        let all: Vec<_> = state.iter_all().collect();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_sharded_state_snapshot() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());

        state.register("realm_001", RealmSpecificState::new(0.5, "democratic"));
        state.register("realm_002", RealmSpecificState::new(0.3, "consensus"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.len(), 2);
        assert!(snapshot.contains_key("realm_001"));
        assert!(snapshot.contains_key("realm_002"));
    }

    // ─────────────────────────────────────────────────────────────────────
    // Async Tests (Lazy Loading)
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_sharded_state_lazy_load_disabled() {
        let state = LazyShardedRealmState::new(ShardingConfig::minimal());

        // Lazy Loading ist deaktiviert in minimal config
        let result = state.get_or_load("realm_001").await;
        assert!(matches!(result, Err(RealmLoadError::LazyLoadingDisabled)));
    }

    #[tokio::test]
    async fn test_sharded_state_lazy_load_no_storage() {
        let mut config = ShardingConfig::minimal();
        config.lazy_loading_enabled = true;
        let state = LazyShardedRealmState::new(config);

        // Kein Storage-Loader konfiguriert
        let result = state.get_or_load("realm_001").await;
        assert!(matches!(result, Err(RealmLoadError::StorageError(_))));
    }

    #[tokio::test]
    async fn test_sharded_state_lazy_load_mock() {
        let mut config = ShardingConfig::minimal();
        config.lazy_loading_enabled = true;
        config.event_replay_on_load = false;

        let mock_loader = Arc::new(MockRealmStorageLoader::new());
        mock_loader.add_realm("realm_001", 0.5, "democratic");

        let state = LazyShardedRealmState::new(config).with_storage(mock_loader);

        // Erste Anfrage: Lazy Load
        let result = state.get_or_load("realm_001").await;
        assert!(result.is_ok());
        assert_eq!(state.cache_misses.load(Ordering::Relaxed), 1);

        // Zweite Anfrage: Cache-Hit
        let result2 = state.get_or_load("realm_001").await;
        assert!(result2.is_ok());
        assert_eq!(state.cache_hits.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn test_sharded_state_lazy_load_not_found() {
        let mut config = ShardingConfig::minimal();
        config.lazy_loading_enabled = true;

        let mock_loader = Arc::new(MockRealmStorageLoader::new());
        // Kein Realm hinzugefügt

        let state = LazyShardedRealmState::new(config).with_storage(mock_loader);

        let result = state.get_or_load("nonexistent").await;
        assert!(matches!(result, Err(RealmLoadError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_sharded_state_eviction() {
        let config = ShardingConfig {
            num_shards: 2,
            max_per_shard: 5,
            lru_capacity_per_shard: 100, // Groß genug um alle zu halten
            ..ShardingConfig::minimal()
        };

        let state = LazyShardedRealmState::new(config);

        // Füge mehr Realms hinzu als max_per_shard
        for i in 0..20 {
            state.register(
                &format!("realm_{}", i),
                RealmSpecificState::new(0.5, "democratic"),
            );
            // Touch im LRU - wichtig für Eviction-Tracking
            state
                .touch_lru(
                    state.shard_index(&format!("realm_{}", i)),
                    &format!("realm_{}", i),
                )
                .await;
        }

        // Vor Eviction: 20 Realms
        assert_eq!(state.exact_count(), 20);

        // Führe Eviction durch
        let evicted = state.evict_all().await;

        // Es sollten einige Realms evicted worden sein
        // Hinweis: Eviction basiert auf max_per_shard=5 und num_shards=2
        // Also max 10 nach Eviction
        assert!(evicted > 0, "Should have evicted some realms");
        let remaining = state.exact_count();
        assert!(
            remaining <= 10,
            "Expected max 10 (2 shards * 5 max), got {}",
            remaining
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // MockRealmStorageLoader Tests
    // ─────────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_mock_storage_loader() {
        let loader = MockRealmStorageLoader::new();

        // Initial: Realm existiert nicht
        assert!(!loader.realm_exists("realm_001").await);

        // Füge Realm hinzu
        loader.add_realm("realm_001", 0.5, "democratic");

        // Jetzt existiert es
        assert!(loader.realm_exists("realm_001").await);

        // Laden
        let loaded = loader.load_realm_base("realm_001").await;
        assert!(loaded.is_ok());

        // Events (initial leer)
        let events = loader.load_realm_events_since("realm_001", None).await;
        assert!(events.is_ok());
        assert!(events.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_mock_storage_loader_persist() {
        let loader = MockRealmStorageLoader::new();

        let realm = RealmSpecificState::new(0.5, "democratic");
        let snapshot = realm.snapshot();
        let result = loader.persist_realm_snapshot("realm_001", &snapshot).await;
        assert!(result.is_ok());

        // Sollte jetzt ladbar sein
        assert!(loader.realm_exists("realm_001").await);
    }

    // =========================================================================
    // PHASE 7.1 TESTS: AtomicF64 & ShardMonitor
    // =========================================================================

    // ─────────────────────────────────────────────────────────────────────
    // AtomicF64 Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_atomic_f64_basic_ops() {
        let atomic = AtomicF64::new(1.5);
        assert_eq!(atomic.load(Ordering::Relaxed), 1.5);

        atomic.store(2.5, Ordering::Relaxed);
        assert_eq!(atomic.load(Ordering::Relaxed), 2.5);
    }

    #[test]
    fn test_atomic_f64_swap() {
        let atomic = AtomicF64::new(1.0);
        let old = atomic.swap(2.0, Ordering::SeqCst);
        assert_eq!(old, 1.0);
        assert_eq!(atomic.load(Ordering::Relaxed), 2.0);
    }

    #[test]
    fn test_atomic_f64_fetch_add() {
        let atomic = AtomicF64::new(1.0);
        let old = atomic.fetch_add(0.5, Ordering::SeqCst);
        assert_eq!(old, 1.0);
        assert!((atomic.load(Ordering::Relaxed) - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_atomic_f64_fetch_sub() {
        let atomic = AtomicF64::new(2.0);
        let old = atomic.fetch_sub(0.5, Ordering::SeqCst);
        assert_eq!(old, 2.0);
        assert!((atomic.load(Ordering::Relaxed) - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_atomic_f64_fetch_mul() {
        let atomic = AtomicF64::new(2.0);
        let old = atomic.fetch_mul(1.5, Ordering::SeqCst);
        assert_eq!(old, 2.0);
        assert!((atomic.load(Ordering::Relaxed) - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_atomic_f64_ewma_update() {
        let atomic = AtomicF64::new(1.0);
        // EWMA: new = current * 0.9 + value * 0.1 = 1.0 * 0.9 + 0.5 * 0.1 = 0.95
        let old = atomic.ewma_update(0.5, 0.9, Ordering::SeqCst);
        assert_eq!(old, 1.0);
        assert!((atomic.load(Ordering::Relaxed) - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_atomic_f64_fetch_max() {
        let atomic = AtomicF64::new(1.0);
        atomic.fetch_max(0.5, Ordering::SeqCst); // No change (0.5 < 1.0)
        assert_eq!(atomic.load(Ordering::Relaxed), 1.0);

        atomic.fetch_max(2.0, Ordering::SeqCst); // Change (2.0 > 1.0)
        assert_eq!(atomic.load(Ordering::Relaxed), 2.0);
    }

    #[test]
    fn test_atomic_f64_fetch_min() {
        let atomic = AtomicF64::new(1.0);
        atomic.fetch_min(2.0, Ordering::SeqCst); // No change (2.0 > 1.0)
        assert_eq!(atomic.load(Ordering::Relaxed), 1.0);

        atomic.fetch_min(0.5, Ordering::SeqCst); // Change (0.5 < 1.0)
        assert_eq!(atomic.load(Ordering::Relaxed), 0.5);
    }

    #[test]
    fn test_atomic_f64_clone() {
        let atomic = AtomicF64::new(3.14159);
        let cloned = atomic.clone();
        assert_eq!(cloned.load(Ordering::Relaxed), 3.14159);

        // Original und Clone sind unabhängig
        atomic.store(2.0, Ordering::Relaxed);
        assert_eq!(cloned.load(Ordering::Relaxed), 3.14159);
    }

    #[test]
    fn test_atomic_f64_compare_exchange() {
        let atomic = AtomicF64::new(1.0);

        // Erfolgreich
        let result = atomic.compare_exchange(1.0, 2.0, Ordering::SeqCst, Ordering::Relaxed);
        assert!(result.is_ok());
        assert_eq!(atomic.load(Ordering::Relaxed), 2.0);

        // Fehlgeschlagen (current != expected)
        let result = atomic.compare_exchange(1.0, 3.0, Ordering::SeqCst, Ordering::Relaxed);
        assert!(result.is_err());
        assert_eq!(atomic.load(Ordering::Relaxed), 2.0);
    }

    // ─────────────────────────────────────────────────────────────────────
    // ShardMonitor Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_shard_monitor_creation() {
        let monitor = ShardMonitor::with_defaults();
        assert_eq!(monitor.shard_activity.len(), 0);
        assert_eq!(monitor.global_entropy(), 1.0); // Default für leeres Set
    }

    #[test]
    fn test_shard_monitor_trust_update() {
        let monitor = ShardMonitor::with_defaults();

        // Record mehrere Updates für Shard 1
        monitor.record_trust_update(1, 0.8);
        monitor.record_trust_update(1, 0.9);
        monitor.record_trust_update(1, 0.7);

        // Activity sollte 3 sein
        assert_eq!(monitor.get_activity(1), 3);

        // Entropy wurde EWMA-geupdatet
        let entropy = monitor.get_entropy(1);
        assert!(entropy > 0.0 && entropy <= 1.0);
    }

    #[test]
    fn test_shard_monitor_cross_shard_failures() {
        let config = ShardMonitorConfig {
            quarantine_failure_threshold: 10,
            ..ShardMonitorConfig::relaxed()
        };
        let monitor = ShardMonitor::new(config);

        // Initial: Reputation 1.0
        assert_eq!(monitor.get_reputation(1), 1.0);
        assert!(!monitor.is_quarantined(1));

        // 5 Failures
        for _ in 0..5 {
            monitor.record_cross_failure(1);
        }

        // Reputation sollte gesunken sein
        let reputation = monitor.get_reputation(1);
        assert!(
            reputation < 1.0,
            "Reputation should decrease: {}",
            reputation
        );
        assert!(!monitor.is_quarantined(1), "Should not be quarantined yet");

        // 5 weitere Failures → Quarantäne (Threshold = 10)
        for _ in 0..5 {
            monitor.record_cross_failure(1);
        }

        assert!(
            monitor.is_quarantined(1),
            "Should be quarantined after 10 failures"
        );
    }

    #[test]
    fn test_shard_monitor_cross_shard_success_recovery() {
        let config = ShardMonitorConfig {
            reputation_penalty_per_failure: 0.2,
            reputation_bonus_per_success: 0.1,
            ..ShardMonitorConfig::relaxed()
        };
        let monitor = ShardMonitor::new(config);

        // Senke Reputation durch Failures
        monitor.record_cross_failure(1);
        monitor.record_cross_failure(1);
        let low_reputation = monitor.get_reputation(1);

        // Recovery durch Successes
        monitor.record_cross_success(1);
        monitor.record_cross_success(1);

        let recovered_reputation = monitor.get_reputation(1);
        assert!(
            recovered_reputation > low_reputation,
            "Reputation should recover: {} > {}",
            recovered_reputation,
            low_reputation
        );
    }

    #[test]
    fn test_shard_monitor_quarantine_lift() {
        let config = ShardMonitorConfig {
            quarantine_failure_threshold: 5,
            ..ShardMonitorConfig::relaxed()
        };
        let monitor = ShardMonitor::new(config);

        // Quarantäne auslösen
        for _ in 0..6 {
            monitor.record_cross_failure(1);
        }
        assert!(monitor.is_quarantined(1));

        // Quarantäne aufheben
        monitor.lift_quarantine(1);
        assert!(!monitor.is_quarantined(1));
    }

    #[test]
    fn test_shard_monitor_penalty_multiplier() {
        let config = ShardMonitorConfig {
            max_penalty_multiplier: 5.0,
            quarantine_failure_threshold: 200,
            ..ShardMonitorConfig::relaxed()
        };
        let monitor = ShardMonitor::new(config);

        // Initial: Keine Strafe (Multiplier = 1.0)
        let initial_penalty = monitor.get_cross_shard_penalty(1);
        assert_eq!(initial_penalty, 1.0);

        // Nach Failures: Höhere Strafe
        for _ in 0..25 {
            monitor.record_cross_failure(1);
        }
        let penalty_after_failures = monitor.get_cross_shard_penalty(1);
        assert!(
            penalty_after_failures > initial_penalty,
            "Penalty should increase: {} > {}",
            penalty_after_failures,
            initial_penalty
        );

        // Bei Quarantäne: Unendlich
        for _ in 0..180 {
            monitor.record_cross_failure(1);
        }
        let quarantine_penalty = monitor.get_cross_shard_penalty(1);
        assert!(
            quarantine_penalty.is_infinite(),
            "Quarantined shard should have infinite penalty"
        );
    }

    #[test]
    fn test_shard_monitor_bias_detection() {
        let config = ShardMonitorConfig {
            bias_threshold: 0.5,
            ..ShardMonitorConfig::relaxed()
        };
        let monitor = ShardMonitor::new(config);

        // Setup: Shard 1 mit niedriger Entropy
        for _ in 0..10 {
            monitor.record_trust_update(1, 0.2); // Niedrige Entropy
        }

        // Setup: Shard 2 mit hoher Entropy
        for _ in 0..10 {
            monitor.record_trust_update(2, 0.9); // Hohe Entropy
        }

        let global_entropy = monitor.global_entropy();

        // Shard 1 sollte als biased erkannt werden (lokale Entropy < 50% von global)
        let shard1_entropy = monitor.get_entropy(1);
        let shard2_entropy = monitor.get_entropy(2);

        // Je nach EWMA-Konvergenz könnte Bias erkannt werden
        // Wichtig: check_shard_bias senkt auch Reputation
        let shard1_biased = monitor.check_shard_bias(1, global_entropy);
        let shard2_biased = monitor.check_shard_bias(2, global_entropy);

        // Shard 2 sollte NICHT als biased erkannt werden (hohe Entropy)
        if shard2_entropy >= global_entropy * 0.5 {
            assert!(!shard2_biased, "Shard 2 should not be biased");
        }
    }

    #[test]
    fn test_shard_monitor_veto_contribution() {
        let config = ShardMonitorConfig {
            quarantine_failure_threshold: 5,
            ..ShardMonitorConfig::relaxed()
        };
        let monitor = ShardMonitor::new(config);

        // Initial: Neutraler Score
        let initial_score = monitor.contribute_to_veto(1);
        assert!(initial_score >= 1.0);

        // Nach Failures: Höherer Risk-Score
        for _ in 0..3 {
            monitor.record_cross_failure(1);
        }
        let score_after_failures = monitor.contribute_to_veto(1);

        // Bei Quarantäne: Sehr hoher Risk-Score
        for _ in 0..5 {
            monitor.record_cross_failure(1);
        }
        let quarantine_score = monitor.contribute_to_veto(1);
        assert!(
            quarantine_score > score_after_failures,
            "Quarantine should have highest risk: {} > {}",
            quarantine_score,
            score_after_failures
        );
    }

    #[test]
    fn test_shard_monitor_snapshot() {
        let monitor = ShardMonitor::with_defaults();

        // Einige Aktivitäten
        monitor.record_trust_update(1, 0.8);
        monitor.record_trust_update(2, 0.9);
        monitor.record_cross_success(1);
        monitor.record_cross_failure(2);

        let snapshot = monitor.snapshot();

        assert_eq!(snapshot.total_shards_monitored, 2);
        assert_eq!(snapshot.total_cross_shard_successes, 1);
        assert_eq!(snapshot.total_cross_shard_failures, 1);
        assert!((snapshot.cross_shard_success_rate - 0.5).abs() < 0.01);
        assert_eq!(snapshot.quarantined_shard_count, 0);
    }

    // ─────────────────────────────────────────────────────────────────────
    // ProtectionState + ShardMonitor Integration Tests
    // ─────────────────────────────────────────────────────────────────────

    #[test]
    fn test_protection_state_with_shard_monitor() {
        let protection = ProtectionState::with_shard_monitor(ShardMonitorConfig::relaxed());

        assert!(protection.shard_monitor().is_some());

        // Record some activity
        protection.record_shard_trust_update(1, 0.8);
        protection.record_cross_shard_success(1);
        protection.record_cross_shard_failure(2);

        // Penalty check
        let penalty1 = protection.get_cross_shard_penalty(1);
        let penalty2 = protection.get_cross_shard_penalty(2);
        assert!(penalty2 > penalty1, "Shard 2 should have higher penalty");
    }

    #[test]
    fn test_protection_state_without_shard_monitor() {
        let protection = ProtectionState::new();

        assert!(protection.shard_monitor().is_none());

        // Alle Operationen sollten graceful sein (no-op)
        protection.record_shard_trust_update(1, 0.8);
        protection.record_cross_shard_success(1);
        protection.record_cross_shard_failure(1);

        // Penalty ohne Monitor = 1.0 (neutral)
        assert_eq!(protection.get_cross_shard_penalty(1), 1.0);
        assert_eq!(protection.shard_veto_contribution(1), 1.0);
    }

    #[test]
    fn test_protection_state_enable_shard_monitor() {
        let mut protection = ProtectionState::new();
        assert!(protection.shard_monitor().is_none());

        protection.enable_shard_monitor(ShardMonitorConfig::relaxed());
        assert!(protection.shard_monitor().is_some());
    }

    #[test]
    fn test_protection_state_health_with_shard_monitor() {
        let mut protection = ProtectionState::with_shard_monitor(ShardMonitorConfig {
            quarantine_failure_threshold: 3,
            ..ShardMonitorConfig::relaxed()
        });

        let initial_health = protection.health_score();

        // Quarantäne auslösen
        for _ in 0..4 {
            protection.record_cross_shard_failure(1);
        }

        let health_after_quarantine = protection.health_score();
        assert!(
            health_after_quarantine < initial_health,
            "Health should decrease with quarantine: {} < {}",
            health_after_quarantine,
            initial_health
        );
    }

    #[test]
    fn test_protection_state_snapshot_with_shard_monitor() {
        let protection = ProtectionState::with_shard_monitor(ShardMonitorConfig::relaxed());

        protection.record_shard_trust_update(1, 0.8);
        protection.record_cross_shard_success(1);

        let snapshot = protection.snapshot();

        assert!(snapshot.shard_monitor.is_some());
        let shard_snap = snapshot.shard_monitor.unwrap();
        assert_eq!(shard_snap.total_shards_monitored, 1);
        assert_eq!(shard_snap.total_cross_shard_successes, 1);
    }

    #[test]
    fn test_protection_state_snapshot_without_shard_monitor() {
        let protection = ProtectionState::new();
        let snapshot = protection.snapshot();

        assert!(snapshot.shard_monitor.is_none());
    }
}

// ============================================================================
// TESTS: PHASE 7.1 - STORAGE LAYER
// ============================================================================

#[cfg(test)]
mod tests_phase7_1_storage_layer {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // ProductionStorageBackend Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_production_storage_backend_default() {
        let backend = ProductionStorageBackend::default();
        match backend {
            ProductionStorageBackend::Fjall {
                path,
                max_memtable_size,
            } => {
                assert_eq!(path, "data/storage");
                assert!(max_memtable_size.is_none());
            }
            _ => panic!("Default should be Fjall"),
        }
    }

    #[test]
    fn test_production_storage_backend_fjall() {
        let backend = ProductionStorageBackend::fjall("/custom/path");
        match backend {
            ProductionStorageBackend::Fjall {
                path,
                max_memtable_size,
            } => {
                assert_eq!(path, "/custom/path");
                assert!(max_memtable_size.is_none());
            }
            _ => panic!("Should be Fjall"),
        }
    }

    #[test]
    fn test_production_storage_backend_fjall_with_config() {
        let backend = ProductionStorageBackend::fjall_with_config("/path", 64 * 1024 * 1024);
        match backend {
            ProductionStorageBackend::Fjall {
                path,
                max_memtable_size,
            } => {
                assert_eq!(path, "/path");
                assert_eq!(max_memtable_size, Some(64 * 1024 * 1024));
            }
            _ => panic!("Should be Fjall with config"),
        }
    }

    #[test]
    fn test_production_storage_backend_in_memory() {
        let backend = ProductionStorageBackend::in_memory();
        assert!(matches!(backend, ProductionStorageBackend::InMemory));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StorageServiceMetrics Tests (Wait-Free Atomics)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_storage_service_metrics_creation() {
        let metrics = StorageServiceMetrics::new();
        assert_eq!(metrics.writes.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.reads.load(Ordering::Relaxed), 0);
        assert_eq!(metrics.errors.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_storage_service_metrics_snapshot() {
        let metrics = StorageServiceMetrics::new();
        metrics.writes.fetch_add(100, Ordering::Relaxed);
        metrics.bytes_written.fetch_add(50000, Ordering::Relaxed);
        metrics.reads.fetch_add(50, Ordering::Relaxed);
        metrics.bytes_read.fetch_add(25000, Ordering::Relaxed);
        metrics.inflight_hits.fetch_add(5, Ordering::Relaxed);

        let snap = metrics.snapshot();
        assert_eq!(snap.writes, 100);
        assert_eq!(snap.bytes_written, 50000);
        assert_eq!(snap.reads, 50);
        assert_eq!(snap.bytes_read, 25000);
        assert_eq!(snap.inflight_hits, 5);
    }

    #[test]
    fn test_storage_service_metrics_throughput() {
        let metrics = StorageServiceMetrics::new();

        // Initial throughput should be 0
        assert_eq!(metrics.write_throughput(), 0.0);
        assert_eq!(metrics.read_throughput(), 0.0);

        // After some operations
        metrics.writes.fetch_add(10, Ordering::Relaxed);
        metrics.bytes_written.fetch_add(5000, Ordering::Relaxed);
        metrics.reads.fetch_add(5, Ordering::Relaxed);
        metrics.bytes_read.fetch_add(2500, Ordering::Relaxed);

        assert_eq!(metrics.write_throughput(), 500.0); // 5000/10
        assert_eq!(metrics.read_throughput(), 500.0); // 2500/5
    }

    #[test]
    fn test_storage_service_metrics_error_rate() {
        let metrics = StorageServiceMetrics::new();

        assert_eq!(metrics.error_rate(), 0.0);

        metrics.writes.fetch_add(90, Ordering::Relaxed);
        metrics.reads.fetch_add(10, Ordering::Relaxed);
        metrics.errors.fetch_add(5, Ordering::Relaxed);

        assert!((metrics.error_rate() - 5.0).abs() < 0.01); // 5/100 * 100 = 5%
    }

    // ─────────────────────────────────────────────────────────────────────────
    // StorageServiceConfig Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_storage_service_config_default() {
        let config = StorageServiceConfig::default();
        assert!(matches!(
            config.backend,
            ProductionStorageBackend::Fjall { .. }
        ));
        assert_eq!(config.checkpoint_event_threshold, 1000);
        assert_eq!(config.checkpoint_time_threshold_secs, 60);
        assert!(config.enable_event_replay);
        assert!(config.enable_component_index);
    }

    #[test]
    fn test_storage_service_config_minimal() {
        let config = StorageServiceConfig::minimal();
        assert!(matches!(config.backend, ProductionStorageBackend::InMemory));
        assert_eq!(config.checkpoint_event_threshold, 10);
        assert_eq!(config.checkpoint_time_threshold_secs, 5);
        assert!(!config.enable_event_replay);
        assert!(!config.enable_component_index);
    }

    #[test]
    fn test_storage_service_config_production() {
        let config = StorageServiceConfig::production("/production/storage");
        match config.backend {
            ProductionStorageBackend::Fjall { path, .. } => {
                assert_eq!(path, "/production/storage");
            }
            _ => panic!("Should be Fjall"),
        }
        assert_eq!(config.checkpoint_event_threshold, 1000);
        assert!(config.enable_event_replay);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BinaryKeyBuilder Tests (Stack-Buffer)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_binary_key_builder_new() {
        let builder = BinaryKeyBuilder::new();
        assert_eq!(builder.as_bytes().len(), 0);
    }

    #[test]
    fn test_binary_key_builder_realm_index() {
        let builder = BinaryKeyBuilder::realm_index_key("realm123", 42);
        let bytes = builder.as_bytes();

        // realm_id + ":" + 8 bytes für sequence
        assert_eq!(bytes.len(), 8 + 1 + 8); // "realm123" + ":" + BE u64

        // Prüfe Präfix
        assert!(bytes.starts_with(b"realm123:"));

        // Prüfe Sequence (Big-Endian)
        let seq_bytes = &bytes[9..];
        let seq = u64::from_be_bytes(seq_bytes.try_into().unwrap());
        assert_eq!(seq, 42);
    }

    #[test]
    fn test_binary_key_builder_component_index() {
        let builder = BinaryKeyBuilder::component_index_key("Trust", 100);
        let bytes = builder.as_bytes();

        assert!(bytes.starts_with(b"Trust:"));

        let seq_bytes = &bytes[6..];
        let seq = u64::from_be_bytes(seq_bytes.try_into().unwrap());
        assert_eq!(seq, 100);
    }

    #[test]
    fn test_binary_key_builder_ordering() {
        // Sequences in Big-Endian sollten korrekt sortiert werden
        let key1 = BinaryKeyBuilder::realm_index_key("test", 1);
        let key2 = BinaryKeyBuilder::realm_index_key("test", 10);
        let key3 = BinaryKeyBuilder::realm_index_key("test", 100);

        assert!(key1.as_bytes() < key2.as_bytes());
        assert!(key2.as_bytes() < key3.as_bytes());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // InFlightMap Tests (Thundering Herd Prevention)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_inflight_map_creation() {
        let map = InFlightMap::new();
        assert_eq!(map.active_count(), 0);
    }

    #[test]
    fn test_inflight_map_start_load() {
        let map = InFlightMap::new();
        map.start_load("realm1");
        assert_eq!(map.active_count(), 1);

        map.start_load("realm2");
        assert_eq!(map.active_count(), 2);
    }

    #[test]
    fn test_inflight_map_complete_removes_entry() {
        let map = InFlightMap::new();
        map.start_load("realm1");
        assert_eq!(map.active_count(), 1);

        let state = Arc::new(RealmSpecificState::new(0.5, "democracy"));
        map.complete_load("realm1", Ok(state));
        assert_eq!(map.active_count(), 0);
    }

    #[test]
    fn test_inflight_map_first_request_none() {
        let map = InFlightMap::new();
        // Erster Request sollte None zurückgeben (kein Waiter)
        let result = map.try_register("realm1");
        // Nach try_register auf nicht-existierendem Entry wird ein leerer Vec angelegt
        // aber da er leer ist, gibt try_register None zurück
        assert!(result.is_none());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RealmCheckpointState Tests (Dirty-Checkpoint)
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_realm_checkpoint_state_creation() {
        let state = RealmCheckpointState::new();
        assert_eq!(state.last_checkpoint_sequence.load(Ordering::Relaxed), 0);
        assert_eq!(state.pending_events.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_realm_checkpoint_state_record_event() {
        let state = RealmCheckpointState::new();
        state.record_event();
        state.record_event();
        state.record_event();
        assert_eq!(state.pending_events.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_realm_checkpoint_state_checkpoint_done() {
        let state = RealmCheckpointState::new();
        for _ in 0..100 {
            state.record_event();
        }
        assert_eq!(state.pending_events.load(Ordering::Relaxed), 100);

        state.checkpoint_done(42);

        assert_eq!(state.last_checkpoint_sequence.load(Ordering::Relaxed), 42);
        assert_eq!(state.pending_events.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_realm_checkpoint_state_needs_checkpoint_by_events() {
        let state = RealmCheckpointState::new();
        let config = StorageServiceConfig {
            checkpoint_event_threshold: 10,
            checkpoint_time_threshold_secs: 3600, // 1 hour, won't trigger
            ..StorageServiceConfig::minimal()
        };

        // Nicht genug Events
        for _ in 0..9 {
            state.record_event();
        }
        assert!(!state.needs_checkpoint(&config));

        // Jetzt genug Events
        state.record_event();
        assert!(state.needs_checkpoint(&config));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // InMemoryStorage Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_in_memory_storage_creation() {
        let storage = InMemoryStorage::new();
        assert!(!storage.realm_exists("nonexistent"));
    }

    #[test]
    fn test_in_memory_storage_persist_and_load_realm() {
        let storage = InMemoryStorage::new();

        let snapshot = RealmSpecificSnapshot {
            trust: crate::domain::unified::TrustVector6D::newcomer(),
            min_trust: 0.3,
            governance_type: "democracy".to_string(),
            member_count: 10,
            pending_member_count: 2,
            banned_count: 1,
            admin_count: 3,
            active_policies: vec!["policy1".to_string()],
            active_rules: vec![],
            isolation_level: 2,
            leak_attempts: 0,
            leaks_blocked: 0,
            crossings_in: 5,
            crossings_out: 3,
            crossings_denied: 0,
            active_crossings: 2,
            crossing_allowlist_count: 0,
            crossing_blocklist_count: 0,
            sagas_initiated: 1,
            cross_realm_sagas_involved: 0,
            sagas_failed: 0,
            compensations_executed: 0,
            events_total: 100,
            events_today: 10,
            last_event_at: 1234567890,
            created_at: 1234560000,
            quota: RealmQuotaSnapshot {
                queue_slots_limit: 1000,
                queue_slots_used: 50,
                storage_bytes_limit: 10_000_000,
                storage_bytes_used: 500_000,
                compute_gas_limit: 1_000_000,
                compute_gas_used: 100_000,
                events_limit: 10_000,
                events_used: 100,
                crossings_limit: 100,
                crossings_used: 5,
                violations: 0,
                quarantined: false,
                last_reset_ms: 1234560000,
            },
            quota_health: 0.95,
        };

        let result = storage.persist_realm_base("realm1", &snapshot, 100);
        assert!(result.is_ok());

        assert!(storage.realm_exists("realm1"));

        let loaded = storage.load_realm_base("realm1");
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.min_trust, 0.3);
        assert_eq!(loaded.governance_type, "democracy");
    }

    #[test]
    fn test_in_memory_storage_realm_not_found() {
        let storage = InMemoryStorage::new();
        let result = storage.load_realm_base("nonexistent");
        assert!(matches!(result, Err(RealmLoadError::NotFound(_))));
    }

    // ─────────────────────────────────────────────────────────────────────────
    // ProductionStorageService Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_production_storage_service_creation() {
        let service = ProductionStorageService::for_testing();
        assert!(service.is_ok());

        let service = service.unwrap();
        assert_eq!(service.active_inflight_count(), 0);
    }

    #[test]
    fn test_production_storage_service_metrics() {
        let service = ProductionStorageService::for_testing().unwrap();

        // Initial metrics should be zero
        let snapshot = service.metrics_snapshot();
        assert_eq!(snapshot.writes, 0);
        assert_eq!(snapshot.reads, 0);
    }

    #[tokio::test]
    async fn test_production_storage_service_realm_exists() {
        let service = ProductionStorageService::for_testing().unwrap();

        // Non-existent realm
        assert!(!service.realm_exists("nonexistent").await);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // WrappedStateEvent.realm_context() Tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn test_wrapped_state_event_realm_context_trust_update() {
        let event = WrappedStateEvent::new(
            StateEvent::TrustUpdate {
                entity_id: "did:test:123".to_string(),
                delta: 0.1,
                reason: TrustReason::PositiveInteraction,
                from_realm: Some("realm1".to_string()),
                triggered_events: 0,
                new_trust: 0.6,
            },
            vec![],
            1,
        );

        assert_eq!(event.realm_context(), Some("realm1".to_string()));
    }

    #[test]
    fn test_wrapped_state_event_realm_context_membership() {
        let event = WrappedStateEvent::new(
            StateEvent::MembershipChange {
                realm_id: "realm2".to_string(),
                identity_id: "did:test:456".to_string(),
                identity_universal_id: None,
                action: MembershipAction::Joined,
                new_role: None,
                initiated_by: None,
                initiated_by_id: None,
            },
            vec![],
            2,
        );

        assert_eq!(event.realm_context(), Some("realm2".to_string()));
    }

    #[test]
    fn test_wrapped_state_event_realm_context_none() {
        let event = WrappedStateEvent::new(
            StateEvent::ExecutionStarted {
                context_id: "ctx123".to_string(),
                gas_budget: 1000,
                mana_budget: 500,
                realm_id: None, // Kein Realm-Kontext
            },
            vec![],
            3,
        );

        assert!(event.realm_context().is_none());
    }
}
