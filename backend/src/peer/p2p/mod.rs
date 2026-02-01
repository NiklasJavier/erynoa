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
//! │  │              TRANSPORT LAYER                     │                   │
//! │  │  TCP + Noise (Encryption) + Yamux (Mux)         │                   │
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
//!
//! ## Axiom-Referenz
//!
//! - **Κ9 (Kausale Struktur)**: Event-DAG über P2P synchronisiert
//! - **Κ10 (Bezeugung-Finalität)**: Attestationen via Gossipsub
//! - **Κ19 (Anti-Verkalkung)**: Power-Cap bei Peer-Connections
//! - **Κ23 (Gateway)**: Realm-Join via P2P + Policy-Check

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
pub mod topics;
#[cfg(feature = "p2p")]
pub mod trust_gate;

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
pub use topics::{RealmTopic, TopicManager};
#[cfg(feature = "p2p")]
pub use trust_gate::TrustGate;
