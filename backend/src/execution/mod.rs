//! # Execution Module
//!
//! Ausführungskontext und Fehlerbehandlung gemäß IPS v1.2.0.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        EXECUTION MODULE                            │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  context    - ExecutionContext (IPS-Monade ℳ)                      │
//! │  error      - ExecutionError (ℳ_VM + ℳ_S + ℳ_P)                    │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## IPS v1.2.0 Mapping
//!
//! Die Monade ℳ aus dem IPS-Modell wird durch zwei Komponenten realisiert:
//!
//! 1. **ExecutionContext**: Kapselt State, Writer, Reader
//! 2. **ExecutionError**: Unifizierte Fehler-Hierarchie
//!
//! Zusammen ergeben sie das Pattern:
//! ```text
//! fn operation(ctx: &mut ExecutionContext) -> Result<T, ExecutionError>
//! ```
//!
//! ## Monadische Gesetze
//!
//! Die Rust-Implementierung erfüllt die monadischen Gesetze:
//!
//! - **Left Identity**: `Ok(a).and_then(f) ≡ f(a)`
//! - **Right Identity**: `m.and_then(Ok) ≡ m`
//! - **Associativity**: `m.and_then(f).and_then(g) ≡ m.and_then(|x| f(x).and_then(g))`
//!
//! ## Axiom-Referenz
//!
//! | Axiom | Implementation |
//! |-------|----------------|
//! | Κ4    | TrustGateBlocked mit Trust-Werten |
//! | Κ8    | TrustDecayViolation |
//! | Κ9    | CausalOrderViolation |
//! | Κ10   | FinalityRegression |
//! | Κ11   | execute() mit Pre/Post-Conditions |
//! | Κ12   | emit() für Event-Erzeugung |
//! | Κ15b  | consume_gas() für Ressourcen-Tracking |
//!
//! ## Beispiel
//!
//! ```rust
//! use erynoa_api::execution::{ExecutionContext, ExecutionError, ExecutionResult};
//!
//! fn process_intent(ctx: &mut ExecutionContext) -> ExecutionResult<u64> {
//!     // Trust-Gate prüfen
//!     ctx.require_trust(0.5)?;
//!
//!     // Gas verbrauchen
//!     ctx.consume_gas(100)?;
//!
//!     // Monadische Ausführung
//!     ctx.execute(|ctx| {
//!         ctx.consume_gas(50)?;
//!
//!         // Event emittieren
//!         ctx.emit_raw("intent.processed", b"data");
//!
//!         Ok(42)
//!     })
//! }
//! ```

pub mod context;
pub mod error;

// Re-exports
pub use context::{
    DelegationHop, Event, ExecutionContext, ExecutionSummary, TrustContext, WorldState,
};
pub use error::{ErrorCategory, ExecutionError, ExecutionResult};

// ============================================================================
// Prelude für häufige Imports
// ============================================================================

/// Prelude für Execution-bezogene Imports
pub mod prelude {
    pub use super::context::{ExecutionContext, ExecutionSummary, TrustContext, WorldState};
    pub use super::error::{ExecutionError, ExecutionResult};
}

// ============================================================================
// Gas Constants
// ============================================================================

/// Standard-Gas-Kosten für verschiedene Operationen
pub mod gas_costs {
    /// Basis-Kosten für Event-Emission
    pub const EVENT_EMIT: u64 = 100;

    /// Basis-Kosten für Storage-Read
    pub const STORAGE_READ: u64 = 50;

    /// Basis-Kosten für Storage-Write
    pub const STORAGE_WRITE: u64 = 200;

    /// Basis-Kosten für P2P-Message
    pub const P2P_MESSAGE: u64 = 150;

    /// Basis-Kosten für Trust-Lookup
    pub const TRUST_LOOKUP: u64 = 25;

    /// Basis-Kosten für Signature-Verification
    pub const SIGNATURE_VERIFY: u64 = 500;

    /// Basis-Kosten für Hash-Computation
    pub const HASH_COMPUTE: u64 = 10;

    /// Kosten pro Byte bei Storage-Write
    pub const STORAGE_PER_BYTE: u64 = 1;

    /// Kosten pro Byte bei P2P-Message
    pub const P2P_PER_BYTE: u64 = 2;
}

/// Standard-Mana-Kosten für verschiedene Operationen
pub mod mana_costs {
    /// Basis-Kosten für Storage-Write
    pub const STORAGE_WRITE: u64 = 10;

    /// Basis-Kosten für P2P-Broadcast
    pub const P2P_BROADCAST: u64 = 50;

    /// Basis-Kosten für DHT-Lookup
    pub const DHT_LOOKUP: u64 = 5;

    /// Mana pro KB bei Storage
    pub const STORAGE_PER_KB: u64 = 1;

    /// Mana pro KB bei P2P
    pub const P2P_PER_KB: u64 = 2;
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Prüfe dass alle Re-Exports funktionieren
        let _: ExecutionContext = ExecutionContext::minimal();
        let _: ExecutionError = ExecutionError::GasExhausted {
            required: 100,
            available: 0,
        };
    }

    #[test]
    fn test_prelude() {
        use super::prelude::*;

        let _ctx: ExecutionContext = ExecutionContext::minimal();
        let _err: ExecutionError = ExecutionError::Internal("test".into());
    }

    #[test]
    fn test_gas_costs_constants() {
        use gas_costs::*;

        assert!(EVENT_EMIT > 0);
        assert!(STORAGE_WRITE > STORAGE_READ);
        assert!(SIGNATURE_VERIFY > HASH_COMPUTE);
    }

    #[test]
    fn test_mana_costs_constants() {
        use mana_costs::*;

        assert!(STORAGE_WRITE > 0);
        assert!(P2P_BROADCAST > DHT_LOOKUP);
    }
}
