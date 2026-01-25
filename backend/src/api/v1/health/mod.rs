//! Health Check API
//!
//! Endpoints für Liveness und Readiness Probes (Connect-RPC only)

mod handlers;

#[cfg(feature = "connect")]
pub use handlers::{health_check_handler, ready_check_handler};
