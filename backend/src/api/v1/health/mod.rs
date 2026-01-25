//! Health Check API
//!
//! Endpoints für Liveness und Readiness Probes

mod handler;
mod models;
mod routes;
#[cfg(feature = "connect")]
mod connect;

pub use routes::create_health_routes;
#[cfg(feature = "connect")]
pub use connect::{health_check_handler, ready_check_handler};
