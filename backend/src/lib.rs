//! # Erynoa Backend
//!
//! High-Performance Rust Backend mit:
//! - **Axum** HTTP/2 + REST API
//! - **Connect-RPC/gRPC-Web** für type-safe API (optional)
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

// Generated Connect-RPC code (available after build with --features connect)
#[cfg(feature = "connect")]
pub mod gen {
    pub mod erynoa {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/erynoa.v1.rs"));
        }
    }
}

pub use error::{ApiError, Result};
pub use server::AppState;

// Re-export version from config module (centralized)
pub use config::version::{VERSION, NAME, DESCRIPTION};
