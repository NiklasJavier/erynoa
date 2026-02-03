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
//! - **Fjall** Embedded Key-Value Store (dezentral)
//! - **Ed25519** DID-basierte Authentifizierung
//! - **CAS** Content Addressable Storage
//!
//! ## Axiom-Referenz
//!
//! Dieses Backend implementiert die 28 Kern-Axiome (Κ1-Κ28) des Erynoa-Protokolls.
//! Siehe `domain` Modul für die grundlegenden Typen.

// === Domain Layer (Kern-Typen gemäß V4.1) ===
pub mod domain;

// === Execution Layer (IPS v1.2.0 Monade ℳ) ===
pub mod execution;

// === Core Logic Layer (Κ2-Κ18) ===
pub mod core;

// === Protection Layer (Κ19-Κ21, Κ26-Κ28) ===
pub mod protection;

// === Peer Layer (Κ22-Κ24) ===
pub mod peer;

// === Decentralized Storage Layer ===
pub mod local;

// === ECLVM - Erynoa Configuration Language VM ===
pub mod eclvm;

// === API & Server ===
pub mod api;
pub mod config;
pub mod error;
pub mod server;
pub mod telemetry;

// === Egui Debugger (optional, feature "debug") ===
#[cfg(feature = "debug")]
pub mod debug;

pub use error::{ApiError, Result};
pub use server::AppState;

// Re-export version from config module (centralized)
pub use config::version::{DESCRIPTION, NAME, VERSION};
