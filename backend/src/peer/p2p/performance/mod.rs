//! # Performance Module - Phase 5 (Wochen 13-14)
//!
//! Performance-Optimierungen für den P2P Privacy-Layer.
//!
//! ## Module
//!
//! - **batch_crypto**: Batch-Processing für 20× Throughput (RL20)
//! - **circuit_cache**: Pre-Built Circuits für <100ms First-Message-Latenz (RL23)
//! - **hw_accel**: Hardware-Accelerated Crypto mit SIMD (RL26)
//!
//! ## Axiom-Referenzen
//!
//! - **RL20**: Batch-Processing mit Rayon für parallele Crypto-Ops
//! - **RL23**: Circuit-Caching für Latenz-Optimierung
//! - **RL26**: Hardware-Crypto-Acceleration (AVX2/AVX-512/NEON)

pub mod batch_crypto;
pub mod circuit_cache;
pub mod hw_accel;

// Re-exports
pub use batch_crypto::{
    AsyncBatchProcessor, BatchCryptoConfig, BatchCryptoStatsSnapshot, BatchDecryptor,
    BatchEncryptor,
};

pub use circuit_cache::{
    CircuitCache, CircuitCacheConfig, CircuitCacheStatsSnapshot, CircuitHop, PreBuiltCircuit,
};

pub use hw_accel::{
    cpu_capabilities, hw_crypto, CpuCapabilities, HwCryptoEngine, HwCryptoError,
    HwCryptoStatsSnapshot, SimdLevel,
};
