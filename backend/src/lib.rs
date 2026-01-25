//! # God-Stack Backend
//!
//! High-Performance Rust Backend mit:
//! - **Axum** HTTP/2 + REST API
//! - **SQLx** PostgreSQL/OrioleDB
//! - **fred** DragonflyDB Cache
//! - **ZITADEL** JWT Auth

pub mod api;
pub mod auth;
pub mod cache;
pub mod config;
pub mod db;
pub mod error;
pub mod server;
pub mod storage;
pub mod telemetry;

pub use error::{ApiError, Result};
pub use server::AppState;

// Re-export version from config module (centralized)
pub use config::version::{VERSION, NAME, DESCRIPTION};
