//! Health Check API
//!
//! Endpoints für Liveness und Readiness Probes

mod handler;
mod models;
mod routes;

pub use routes::create_health_routes;
