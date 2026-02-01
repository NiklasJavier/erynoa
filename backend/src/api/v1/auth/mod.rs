//! Passkey/WebAuthn Authentication Module
//!
//! Handles Challenge generation, Passkey registration, and verification.
//! Supports Ed25519-based DIDs for compatibility with Erynoa's identity system.

pub mod handlers;
pub mod types;

pub use handlers::*;
pub use types::*;
