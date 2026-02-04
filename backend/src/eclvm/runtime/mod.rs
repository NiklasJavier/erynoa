//! # ECLVM Runtime
//!
//! Die Laufzeitumgebung für die ECLVM.
//!
//! ## Module
//!
//! - `gas` - Gas-Metering für DoS-Schutz
//! - `host` - HostInterface Trait und Implementierungen
//! - `runner` - Policy-Ausführung mit Kontext
//! - `vm` - Die ECLVM selbst
//! - `state_host` - E3: StateHost für State-backed ECL

pub mod gas;
pub mod host;
pub mod runner;
pub mod state_host;
pub mod vm;
