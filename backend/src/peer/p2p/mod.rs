//! # Erynoa P2P-Netzwerk-Schicht (libp2p)
//!
//! Vollständig dezentrale, realm-spezifische P2P-Kommunikation.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                          P2P NETWORK LAYER                              │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
//! │  │   SWARM      │  │   GOSSIPSUB  │  │   KADEMLIA   │                  │
//! │  │   MANAGER    │  │   (PubSub)   │  │   (DHT)      │                  │
//! │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │
//! │         │                 │                 │                           │
//! │  ┌──────┴─────────────────┴─────────────────┴──────┐                   │
//! │  │              ERYNOA BEHAVIOUR                    │                   │
//! │  │  • Realm-Topics (/erynoa/realm/{id}/events/v1)  │                   │
//! │  │  • DID-based PeerID (Ed25519)                   │                   │
//! │  │  • Trust-gated Connections                      │                   │
//! │  │  • Event Sync Protocol                          │                   │
//! │  └──────────────────────────────────────────────────┘                   │
//! │                            │                                            │
//! │  ┌────────────────────────┴────────────────────────┐                   │
//! │  │              PRIVACY LAYER (V2.6) 🆕            │                   │
//! │  │  • Onion-Routing (RL2-RL4)                      │                   │
//! │  │  • Trust-basierte Relay-Auswahl (RL5-RL7)       │                   │
//! │  │  • QUIC Transport mit 0-RTT                     │                   │
//! │  └──────────────────────────────────────────────────┘                   │
//! │                            │                                            │
//! │  ┌────────────────────────┴────────────────────────┐                   │
//! │  │              TRANSPORT LAYER                     │                   │
//! │  │  TCP + Noise (Encryption) + Yamux (Mux)         │                   │
//! │  │  QUIC (Primary) + TCP (Fallback)                │                   │
//! │  └──────────────────────────────────────────────────┘                   │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Kern-Features
//!
//! - **Peer-Discovery**: Bootstrap + Kademlia DHT + mDNS
//! - **Event-Propagation**: Gossipsub mit Realm-Topics
//! - **Trust-Sync**: Attestationen propagieren, lokal berechnen
//! - **Saga-Support**: Cross-Peer-Intents über Request-Response
//! - **Gaming-Resistenz**: Trust-gated, Anomaly-Integration
//! - **Privacy-Layer** (V2.6): Onion-Routing, Multi-Hop-Relays
//! - **QUIC Transport**: 0-RTT Circuit-Setup, Connection-Migration
//!
//! ## Axiom-Referenz
//!
//! - **Κ9 (Kausale Struktur)**: Event-DAG über P2P synchronisiert
//! - **Κ10 (Bezeugung-Finalität)**: Attestationen via Gossipsub
//! - **Κ19 (Anti-Verkalkung)**: Power-Cap bei Peer-Connections
//! - **Κ23 (Gateway)**: Realm-Join via P2P + Policy-Check
//! - **RL2-RL4**: Onion-Routing mit Wissens-Separation
//! - **RL5-RL7**: Trust-basierte Relay-Auswahl
//! - **RL24**: QUIC Transport mit 0-RTT

#[cfg(feature = "p2p")]
pub mod behaviour;
#[cfg(feature = "p2p")]
pub mod config;
#[cfg(feature = "p2p")]
pub mod diagnostics;
#[cfg(feature = "p2p")]
pub mod identity;
#[cfg(feature = "p2p")]
pub mod protocol;
#[cfg(feature = "p2p")]
pub mod swarm;
#[cfg(feature = "p2p")]
pub mod testnet;
#[cfg(feature = "p2p")]
pub mod timing;
#[cfg(feature = "p2p")]
pub mod topics;
#[cfg(feature = "p2p")]
pub mod trust_gate;

// Privacy-Layer (V2.6 Phase 1)
#[cfg(feature = "privacy")]
pub mod privacy;

// Transport-Layer (QUIC + TCP Fallback)
#[cfg(feature = "privacy")]
pub mod transport;

// Performance-Layer (Phase 5)
#[cfg(feature = "privacy")]
pub mod performance;

// Multi-Circuit-Layer (Phase 5c - Conflux-Style)
#[cfg(feature = "privacy")]
pub mod multi_circuit;

// Censorship-Resistance Layer (Phase 6)
#[cfg(feature = "privacy")]
pub mod censorship;

#[cfg(feature = "p2p")]
pub use behaviour::ErynoaBehaviour;
#[cfg(feature = "p2p")]
pub use config::P2PConfig;
#[cfg(feature = "p2p")]
pub use diagnostics::{
    // Helpers
    create_diagnostic_state,
    diagnostic_routes,
    format_bytes,
    format_rate,
    // Dashboard
    generate_dashboard_html,
    // Core types
    ComponentStatus,
    // State (für Real-Time)
    ConnectionType,
    DiagnosticCheck,
    // Events
    DiagnosticEvent,
    // Runner
    DiagnosticRunner,
    DiagnosticState,
    DiagnosticSummary,
    EventBuffer,
    EventSeverity,
    EventType,
    // Stream
    HealthStatus,
    LayerDiagnostic,
    LivePeerInfo,
    // Metrics
    MetricsCollector,
    NatStatus,
    NetworkMetrics,
    P2PDiagnostics,
    PeerInfo,
    StreamSnapshot,
    SwarmSnapshot,
    // SwarmState für echte Laufzeit-Daten (V2.7)
    SwarmState,
};
#[cfg(feature = "p2p")]
pub use identity::PeerIdentity;
#[cfg(feature = "p2p")]
pub use protocol::{SyncProtocol, SyncRequest, SyncResponse};
#[cfg(feature = "p2p")]
pub use swarm::{IncomingSyncRequest, SwarmCommand, SwarmEvent2, SwarmManager};
#[cfg(feature = "p2p")]
pub use testnet::{TestnetBehaviour, TestnetEvent, TestnetSwarm};
#[cfg(feature = "p2p")]
pub use timing::{NetworkConditions, NetworkQuality, SyncTiming, TimingManager, TimingStatus};
#[cfg(feature = "p2p")]
pub use topics::{RealmTopic, TopicManager};
#[cfg(feature = "p2p")]
pub use trust_gate::TrustGate;

// Privacy-Layer Re-exports
#[cfg(feature = "privacy")]
pub use privacy::{
    DecryptedLayer, OnionBuilder, OnionDecryptor, OnionError, RelayCandidate, RelaySelectionError,
    RelaySelector, SensitivityLevel,
};

// Transport-Layer Re-exports
#[cfg(feature = "privacy")]
pub use transport::{HybridTransport, QuicConfig, QuicTransport, TransportMode};

// Performance-Layer Re-exports (Phase 5)
#[cfg(feature = "privacy")]
pub use performance::{
    cpu_capabilities, hw_crypto, BatchCryptoConfig, BatchDecryptor, BatchEncryptor, CircuitCache,
    CircuitCacheConfig, HwCryptoEngine, SimdLevel,
};

// Multi-Circuit Re-exports (Phase 5c - Conflux-Style RL28)
#[cfg(feature = "privacy")]
pub use multi_circuit::{
    ConfluxConfig, ConfluxError, ConfluxManager, ConfluxStats, EgressAggregator,
    EgressAggregatorStats, MultiPathStrategy, SecretSharer,
};

// Censorship-Resistance Re-exports (Phase 6 - RL19)
#[cfg(feature = "privacy")]
pub use censorship::{
    BootstrapConfig, BootstrapError, BootstrapHelper, BridgeInfo, BridgePool, CensorshipLevel,
    RecommendedRelay, TransportManager, TransportType,
};
