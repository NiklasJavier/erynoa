//! EnvironmentService API
//!
//! Connect-RPC Handler für Environment/Realm-Operationen.
//!
//! ## Implementierte Axiome
//!
//! - **Κ19**: Realm-Autonomie
//! - **Κ20**: Reputation-Vererbung bei Crossing
//! - **PR3**: Gateway-Vollständigkeit
//! - **PR6**: Trust-Dämpfung

mod handlers;

pub use handlers::*;
