//! Storage API
//!
//! File upload, download, and object management endpoints

mod handler;
mod models;
mod routes;

pub use routes::create_storage_routes;
