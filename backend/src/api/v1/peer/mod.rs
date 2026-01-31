//! PeerService API
//!
//! Connect-RPC Handler für Peer-Operationen.
//!
//! ## Implementierte Axiome
//!
//! - **PR1**: Intent-Auflösung (Composer)
//! - **PR3**: Gateway-Vollständigkeit
//! - **PR5**: Schlüssel-Isolation (Key Vault)
//! - **PR6**: Trust-Dämpfung bei Realm-Crossing

mod handlers;

pub use handlers::*;
