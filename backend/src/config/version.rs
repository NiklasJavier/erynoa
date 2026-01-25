//! Version Information
//!
//! Zentrale Version-Informationen für die API

/// Application Version
/// Wird aus Cargo.toml zur Compile-Zeit eingefügt
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application Name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Application Description
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
