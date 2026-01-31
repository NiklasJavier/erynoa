//! # Erynoa Backend
//!
//! High-Performance Rust Backend für das Erynoa-Protokoll V4.1.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │  Client/Peer Layer    (IntentParser, SagaComposer, GatewayGuard)   │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  Core Logic Layer     (EventEngine, TrustEngine, ConsensusEngine)  │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  Storage/Realm Layer  (EventStore, IdentityStore, RealmHierarchy)  │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  Protection Layer     (AntiCalcification, DiversityMonitor)        │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Kern-Features
//!
//! - **Axum** HTTP/2 + REST API
//! - **Connect-RPC/gRPC-Web** für type-safe API (optional)
//! - **Sled** Embedded Key-Value Store (dezentral)
//! - **Ed25519** DID-basierte Authentifizierung
//! - **CAS** Content Addressable Storage
//!
//! ## Legacy Features (optional)
//!
//! - **SQLx** PostgreSQL/OrioleDB (`legacy-sql`)
//! - **fred** DragonflyDB Cache (`legacy-cache`)
//! - **ZITADEL** JWT Auth (`legacy-oidc`)
//!
//! ## Axiom-Referenz
//!
//! Dieses Backend implementiert die 28 Kern-Axiome (Κ1-Κ28) des Erynoa-Protokolls.
//! Siehe `domain` Modul für die grundlegenden Typen.

// === Domain Layer (Kern-Typen gemäß V4.1) ===
pub mod domain;

// === Core Logic Layer (Κ2-Κ18) ===
pub mod core;

// === Protection Layer (Κ19-Κ21, Κ26-Κ28) ===
pub mod protection;

// === Peer Layer (Κ22-Κ24) ===
pub mod peer;

// === Decentralized Storage Layer ===
pub mod local;

// === Existing Modules ===
pub mod api;
#[cfg(feature = "legacy-oidc")]
pub mod auth;
#[cfg(feature = "legacy-cache")]
pub mod cache;
pub mod config;
#[cfg(feature = "legacy-sql")]
pub mod db;
pub mod error;
pub mod server;
#[cfg(feature = "legacy-s3")]
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
pub use config::version::{DESCRIPTION, NAME, VERSION};
