//! API v1
//!
//! REST-basierte API für WebAuthn und Health-Checks

// REST fallback handlers for health checks and info
pub mod rest_handlers;

// Phase 1: Debug & Observability (state, health, invariants, events, metrics, warnings)
pub mod state_handlers;

// Phase 2: Produktion Kern (crossing, trust, identity, realm, ecl stubs)
pub mod production_handlers;

// Phase 4: Ops & Recovery (replay, checkpoints)
pub mod debug_handlers;

// Passkey/WebAuthn authentication module
pub mod auth;
pub use auth::StoredPasskeyCredential;
