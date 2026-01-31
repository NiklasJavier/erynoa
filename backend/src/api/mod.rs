//! API Module
//!
//! Feature-basierte API-Struktur mit Versionierung

mod constants;
mod middleware;
mod routes;
mod shared;
pub mod static_files;
mod v1;

pub use constants::API_VERSION;
pub use routes::create_router;
pub use static_files::{create_static_router, StaticConfig};
