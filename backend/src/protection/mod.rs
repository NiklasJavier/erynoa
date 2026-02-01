//! # Erynoa Protection Layer
//!
//! Systemschutz gemäß Axiome Κ19-Κ21, Κ26-Κ28.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │                        PROTECTION LAYER                                     │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │  anti_calcification   - Verhindert Macht-Konzentration (Κ19)                │
//! │  adaptive_calibration - Dynamische Parameteranpassung (Κ19, §IX)            │
//! │  diversity            - Überwacht System-Diversität (Κ20)                   │
//! │  quadratic            - Quadratisches Voting (Κ21)                          │
//! │  anomaly              - Erkennt abnormales Verhalten                        │
//! └─────────────────────────────────────────────────────────────────────────────┘
//! ```

pub mod adaptive_calibration;
pub mod anomaly;
pub mod anti_calcification;
pub mod diversity;
pub mod quadratic;

// Re-exports
pub use adaptive_calibration::{
    CalibratedParameters, CalibrationConfig, CalibrationEngine, CalibrationStats, NetworkMetrics,
    ParameterBounds,
};
pub use anomaly::AnomalyDetector;
pub use anti_calcification::{AntiCalcification, AntiCalcificationConfig};
pub use diversity::DiversityMonitor;
pub use quadratic::QuadraticGovernance;
