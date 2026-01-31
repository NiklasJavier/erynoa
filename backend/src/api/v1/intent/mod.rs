//! IntentService API
//!
//! Connect-RPC Handler für Intent-Operationen.
//!
//! ## Implementierte Axiome
//!
//! - **PR1**: Intent-Auflösung (IntentParser → SagaComposer)
//! - **PR2**: Saga-Ausführung
//! - **PR4**: HTLC-Atomarität (bei Cross-Chain)
//! - **Κ22**: Saga-Komposition
//! - **Κ23**: Gateway-Kohärenz
//! - **Κ24**: HTLC-Garantie

mod handlers;

pub use handlers::*;
