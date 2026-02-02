//! # Multi-Circuit Module - Phase 5c (Woche 14)
//!
//! Conflux-Style Multi-Circuit Multiplexing für 4× Throughput.
//!
//! ## Module
//!
//! - **parallel_paths**: Multi-Circuit-Routing mit Secret-Sharing (RL28)
//!
//! ## Axiom-Referenzen
//!
//! - **RL28**: Multi-Circuit-Multiplexing (Conflux-Style)
//! - **RL6**: Relay-Diversität zwischen Circuits
//! - **RL5**: Trust-basierte Pfad-Auswahl

pub mod parallel_paths;

// Re-exports
pub use parallel_paths::{
    ActiveCircuit, CircuitStats, ConfluxConfig, ConfluxError, ConfluxManager, ConfluxStats,
    EgressAggregator, MultiPathResult, MultiPathStrategy, SecretSharer, MAX_PARALLEL_CIRCUITS,
    MIN_AS_DISTANCE,
};
