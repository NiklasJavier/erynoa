//! API Module
//!
//! Feature-basierte API-Struktur mit Versionierung

mod constants;
mod middleware;
mod routes;
mod shared;
mod v1;

pub use constants::API_VERSION;
pub use routes::create_router;
