//! SagaService API
//!
//! Connect-RPC Handler für Saga-Operationen.
//!
//! ## Implementierte Axiome
//!
//! - **Κ22**: Saga-Komposition
//! - **Κ24**: Atomare Kompensation (fail(Sᵢ) → compensate(S₁..Sᵢ₋₁))
//! - **PR2**: Saga-Ausführung
//! - **PR4**: HTLC-Atomarität

mod handlers;

pub use handlers::*;
