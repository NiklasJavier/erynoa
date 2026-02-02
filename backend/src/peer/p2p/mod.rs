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
pub mod identity;
#[cfg(feature = "p2p")]
pub mod protocol;
#[cfg(feature = "p2p")]
pub mod swarm;
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

#[cfg(feature = "p2p")]
pub use behaviour::ErynoaBehaviour;
#[cfg(feature = "p2p")]
pub use config::P2PConfig;
#[cfg(feature = "p2p")]
pub use identity::PeerIdentity;
#[cfg(feature = "p2p")]
pub use protocol::{SyncProtocol, SyncRequest, SyncResponse};
#[cfg(feature = "p2p")]
pub use swarm::SwarmManager;
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
