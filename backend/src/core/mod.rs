//! # Erynoa Core Logic Layer
//!
//! Implementiert die Business-Logik gemäß V4.1 Axiomen.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        CORE LOGIC LAYER                            │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  event_engine     - Event-Verarbeitung (Κ9-Κ12)                    │
//! │  trust_engine     - Trust-Berechnung (Κ2-Κ5)                       │
//! │  surprisal        - Surprisal-Berechnung (Κ15a)                    │
//! │  world_formula    - Weltformel-Engine (Κ15b-d)                     │
//! │  consensus        - Konsensus-Mechanismus (Κ18)                    │
//! │  engine           - ExecutionContext-aware Wrapper (Phase 3)       │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```

pub mod consensus;
pub mod engine;
pub mod event_engine;
pub mod surprisal;
pub mod trust_engine;
pub mod world_formula;

// Re-exports - Legacy engines
pub use consensus::ConsensusEngine;
pub use event_engine::EventEngine;
pub use surprisal::SurprisalCalculator;
pub use trust_engine::TrustEngine;
pub use world_formula::WorldFormulaEngine;

// Re-exports - Unified engine layer (Phase 3)
pub use engine::{
    event_gas, formula_gas, trust_gas, EventProcessor, FinalityTracker, FormulaComputer,
    TrustUpdater,
};
