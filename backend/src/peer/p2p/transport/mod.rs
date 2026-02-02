//! # Erynoa Transport-Layer (QUIC + TCP Fallback)
//!
//! Hochperformante Transport-Schicht mit QUIC als primärem Protokoll
//! und TCP als Fallback für NAT-Traversal-Szenarien.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                      TRANSPORT LAYER (V2.6)                             │
//! ├─────────────────────────────────────────────────────────────────────────┤
//! │                                                                         │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                  │
//! │  │    QUIC      │  │     TCP      │  │   HYBRID     │                  │
//! │  │  (Primary)   │  │  (Fallback)  │  │   MANAGER    │                  │
//! │  │   0-RTT      │  │   NAT-safe   │  │  Auto-Switch │                  │
//! │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘                  │
//! │         │                 │                 │                           │
//! │  ┌──────┴─────────────────┴─────────────────┴──────┐                   │
//! │  │              TRANSPORT ABSTRACTION              │                   │
//! │  │  • Unified Connection API                       │                   │
//! │  │  • Multi-Stream Multiplexing                    │                   │
//! │  │  • Automatic Protocol Selection                 │                   │
//! │  └──────────────────────────────────────────────────┘                   │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Performance-Vorteile (QUIC vs TCP)
//!
//! | Feature                | QUIC         | TCP+TLS      | Verbesserung |
//! |------------------------|--------------|--------------|--------------|
//! | Connection Setup       | 0-1 RTT      | 3+ RTT       | 3-6×         |
//! | Head-of-Line Blocking  | Keine        | Ja           | ∞            |
//! | Connection Migration   | Ja           | Nein         | Mobilität    |
//! | Multiplexing           | Native       | Emuliert     | Effizienter  |
//!
//! ## Axiom-Referenzen
//!
//! - **RL24**: QUIC Transport (0-RTT, Multiplexing)
//! - **RL2**: Wissens-Separation über verschlüsselte Streams

#[cfg(feature = "privacy")]
pub mod quic;

#[cfg(feature = "privacy")]
pub mod tcp_fallback;

#[cfg(feature = "privacy")]
pub mod hybrid;

// Re-exports
#[cfg(feature = "privacy")]
pub use quic::{QuicConfig, QuicTransport, QuicError};

#[cfg(feature = "privacy")]
pub use tcp_fallback::{TcpFallbackConfig, TcpFallbackTransport};

#[cfg(feature = "privacy")]
pub use hybrid::{HybridTransport, TransportMode, TransportMetrics};
