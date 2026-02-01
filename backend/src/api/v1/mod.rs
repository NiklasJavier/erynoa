//! API v1
//!
//! REST-basierte API für WebAuthn und Health-Checks

// REST fallback handlers for health checks and info
pub mod rest_handlers;

// Passkey/WebAuthn authentication module
pub mod auth;
pub use auth::StoredPasskeyCredential;
