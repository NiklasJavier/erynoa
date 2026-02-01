//! API v1
//!
//! Version 1 der API mit Connect-RPC Services

#[cfg(feature = "connect")]
pub mod environment;
#[cfg(feature = "connect")]
pub mod health;
#[cfg(feature = "connect")]
pub mod info;
#[cfg(feature = "connect")]
pub mod intent;
#[cfg(feature = "connect")]
pub mod peer;
#[cfg(feature = "connect")]
pub mod saga;

#[cfg(feature = "connect")]
pub mod connect_routes;

// REST fallback handlers for health checks and info
pub mod rest_handlers;

// Passkey/WebAuthn authentication module
pub mod auth;
pub use auth::StoredPasskeyCredential;
