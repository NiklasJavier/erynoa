//! # Censorship-Resistance Layer (Phase 6)
//!
//! Zensur-Resistenz für Erynoa P2P-Netzwerk.
//!
//! ## Komponenten
//!
//! | Modul                | Beschreibung                              | Axiom-Ref |
//! |----------------------|-------------------------------------------|-----------|
//! | `pluggable_transports` | obfs4, Meek, Snowflake, Domain-Fronting | RL19      |
//! | `bridges`            | Unlisted Entry Points für zensierte Regionen | RL19      |
//! | `bootstrap_helper`   | DHT-Recommended-Lists für Newcomer       | RL5-RL7   |
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                    CENSORSHIP RESISTANCE LAYER                          │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐      │
//! │  │   PLUGGABLE      │  │     BRIDGE       │  │   BOOTSTRAP      │      │
//! │  │   TRANSPORTS     │  │     NETWORK      │  │    HELPERS       │      │
//! │  │                  │  │                  │  │                  │      │
//! │  │  • obfs4         │  │  • MOAT          │  │  • DHT Discovery │      │
//! │  │  • Meek          │  │  • Email         │  │  • Relay Cache   │      │
//! │  │  • Snowflake     │  │  • Social-Graph  │  │  • Auto-Select   │      │
//! │  │  • Domain-Front  │  │  • Physical      │  │  • Diversity     │      │
//! │  └────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘      │
//! │           │                     │                     │                │
//! │           └─────────────────────┴─────────────────────┘                │
//! │                                 │                                      │
//! │                    ┌────────────┴────────────┐                         │
//! │                    │    TRANSPORT MANAGER    │                         │
//! │                    │  • Auto-Selection       │                         │
//! │                    │  • Fallback-Chain       │                         │
//! │                    │  • Blocking-Detection   │                         │
//! │                    └────────────────────────-┘                         │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Censorship-Level Strategie
//!
//! | Level    | Transport           | Bridge        | Multi-Path |
//! |----------|---------------------|---------------|------------|
//! | LOW      | Direct              | -             | -          |
//! | MEDIUM   | obfs4               | Optional      | -          |
//! | HIGH     | Meek/Snowflake      | Required      | 3-of-5     |
//! | CRITICAL | Snowflake + Stego   | Social-Graph  | 5-of-7     |
//!
//! ## Axiom-Referenz
//!
//! - **RL19**: AS-Path Zensur-Resistenz, Pluggable Transports
//! - **Κ20**: Resilience gegen State-Level-Adversaries
//! - **RL5-RL7**: Trust-basierte Relay-Auswahl

pub mod bootstrap_helper;
pub mod bridges;
pub mod pluggable_transports;

// Re-exports: Pluggable Transports
pub use pluggable_transports::{
    CensorshipLevel, MeekConfig, Obfs4Config, SnowflakeConfig, TransportConfig, TransportError,
    TransportManager, TransportManagerConfig, TransportStats, TransportStatus, TransportType,
    TransportWrapper,
};

// Re-exports: Bridges
pub use bridges::{
    BridgeDistributor, BridgeError, BridgeInfo, BridgePool, BridgePoolConfig, BridgePoolStats,
    BridgeStatus, DiscoveryMethod,
};

// Re-exports: Bootstrap Helpers
pub use bootstrap_helper::{
    BootstrapConfig, BootstrapError, BootstrapHelper, BootstrapStats, DiscoveryEvent,
    MockDhtClient, RecommendedRelay, RelayPublisher,
};

// Utility functions
pub use pluggable_transports::{detect_censored_as_path, recommend_transport};
