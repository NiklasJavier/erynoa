//! # Erynoa Protection Layer
//!
//! Systemschutz gemäß Axiome Κ19-Κ21, Κ26-Κ28.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                      PROTECTION LAYER                              │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  anti_calcification  - Verhindert Macht-Konzentration (Κ19)        │
//! │  diversity           - Überwacht System-Diversität (Κ20)           │
//! │  quadratic           - Quadratisches Voting (Κ21)                  │
//! │  anomaly             - Erkennt abnormales Verhalten                │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

pub mod anti_calcification;
pub mod anomaly;
pub mod diversity;
pub mod quadratic;

// Re-exports
pub use anti_calcification::AntiCalcification;
pub use anomaly::AnomalyDetector;
pub use diversity::DiversityMonitor;
pub use quadratic::QuadraticGovernance;
