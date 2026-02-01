//! # Erynoa Peer Layer
//!
//! Client-Facing Services gemäß Axiome Κ22-Κ24.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                         PEER LAYER                                 │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  intent_parser   - Parst Nutzer-Intents (Κ22)                      │
//! │  saga_composer   - Komponiert Sagas aus Intents (Κ22)              │
//! │  gateway         - Cross-Realm Gateway Guard (Κ23)                 │
//! │  p2p             - libp2p Netzwerk-Schicht (Κ9, Κ10, Κ23)         │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

pub mod gateway;
pub mod intent_parser;
pub mod saga_composer;

// P2P-Modul (feature-gated)
#[cfg(feature = "p2p")]
pub mod p2p;

// Re-exports
pub use gateway::GatewayGuard;
pub use intent_parser::IntentParser;
pub use saga_composer::SagaComposer;

// P2P Re-exports
#[cfg(feature = "p2p")]
pub use p2p::{
    ErynoaBehaviour, P2PConfig, PeerIdentity, RealmTopic, SwarmManager, SyncProtocol, SyncRequest,
    SyncResponse, TopicManager, TrustGate,
};
