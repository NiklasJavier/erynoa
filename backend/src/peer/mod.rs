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
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

pub mod gateway;
pub mod intent_parser;
pub mod saga_composer;

// Re-exports
pub use gateway::GatewayGuard;
pub use intent_parser::IntentParser;
pub use saga_composer::SagaComposer;
