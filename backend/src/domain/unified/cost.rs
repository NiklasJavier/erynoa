//! # Unified Data Model – Kosten-Algebra
//!
//! Implementierung der Kosten-Algebra 𝒦 aus dem IPS-Modell.
//!
//! ## Kosten-Struktur
//!
//! ```text
//! κ = (gas, mana, trust_risk)
//!
//! Sequentiell: κ₁ ⊕ κ₂ = (g₁+g₂, m₁+m₂, 1-(1-t₁)(1-t₂))
//! Parallel:    κ₁ ⊗ κ₂ = (max(g₁,g₂), m₁+m₂, max(t₁,t₂))
//! ```
//!
//! ## Subsystem-Zuordnung
//!
//! - **Gas** → ECLVM (Computation)
//! - **Mana** → Storage + P2P (Resources)
//! - **Trust-Risk** → Risiko bei Erynoa-Interaktion

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, AddAssign};

// ============================================================================
// Cost – Unified Kosten-Struktur
// ============================================================================

/// Unified Cost für alle Operationen
///
/// Implementiert die Kosten-Algebra 𝒦 aus dem IPS-Modell.
///
/// # Beispiel
///
/// ```rust
/// use erynoa_api::domain::unified::Cost;
///
/// let op1 = Cost::new(100, 50, 0.1);
/// let op2 = Cost::new(200, 30, 0.05);
///
/// // Sequentielle Komposition
/// let total = op1.seq(op2);
/// assert_eq!(total.gas, 300);
/// assert_eq!(total.mana, 80);
/// ```
#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct Cost {
    /// Gas für Computation (ECLVM)
    pub gas: u64,
    /// Mana für Storage/Network
    pub mana: u64,
    /// Trust-Risk ∈ [0, 1]
    pub trust_risk: f32,
}

impl Cost {
    /// Keine Kosten
    pub const ZERO: Self = Self {
        gas: 0,
        mana: 0,
        trust_risk: 0.0,
    };

    /// Blockierend (unendliche Kosten)
    pub const BLOCKING: Self = Self {
        gas: u64::MAX,
        mana: u64::MAX,
        trust_risk: 1.0,
    };

    /// Erstelle neue Kosten
    #[inline]
    pub const fn new(gas: u64, mana: u64, trust_risk: f32) -> Self {
        Self {
            gas,
            mana,
            trust_risk,
        }
    }

    /// Nur Gas-Kosten
    #[inline]
    pub const fn gas_only(gas: u64) -> Self {
        Self {
            gas,
            mana: 0,
            trust_risk: 0.0,
        }
    }

    /// Nur Mana-Kosten
    #[inline]
    pub const fn mana_only(mana: u64) -> Self {
        Self {
            gas: 0,
            mana,
            trust_risk: 0.0,
        }
    }

    /// Sequentielle Komposition (⊕)
    ///
    /// ```text
    /// κ₁ ⊕ κ₂ = (g₁+g₂, m₁+m₂, 1-(1-t₁)(1-t₂))
    /// ```
    #[inline]
    pub fn seq(self, other: Self) -> Self {
        Self {
            gas: self.gas.saturating_add(other.gas),
            mana: self.mana.saturating_add(other.mana),
            // Probabilistische Kombination für Trust-Risk
            trust_risk: 1.0 - (1.0 - self.trust_risk) * (1.0 - other.trust_risk),
        }
    }

    /// Parallele Komposition (⊗)
    ///
    /// ```text
    /// κ₁ ⊗ κ₂ = (max(g₁,g₂), m₁+m₂, max(t₁,t₂))
    /// ```
    #[inline]
    pub fn par(self, other: Self) -> Self {
        Self {
            gas: self.gas.max(other.gas),
            mana: self.mana.saturating_add(other.mana),
            trust_risk: self.trust_risk.max(other.trust_risk),
        }
    }

    /// Skalierung mit Faktor
    pub fn scale(self, factor: f32) -> Self {
        Self {
            gas: (self.gas as f64 * factor as f64) as u64,
            mana: (self.mana as f64 * factor as f64) as u64,
            trust_risk: (self.trust_risk * factor).clamp(0.0, 1.0),
        }
    }

    /// Prüfe ob Null-Kosten
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.gas == 0 && self.mana == 0 && self.trust_risk == 0.0
    }

    /// Prüfe ob blockierend
    #[inline]
    pub fn is_blocking(&self) -> bool {
        self.gas == u64::MAX || self.mana == u64::MAX || self.trust_risk >= 1.0
    }

    /// Gesamtkosten als gewichtete Summe (für Vergleiche)
    pub fn total_weighted(&self, gas_weight: f64, mana_weight: f64, risk_weight: f64) -> f64 {
        self.gas as f64 * gas_weight
            + self.mana as f64 * mana_weight
            + self.trust_risk as f64 * risk_weight
    }
}

impl Add for Cost {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        self.seq(other)
    }
}

impl AddAssign for Cost {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = self.seq(other);
    }
}

impl fmt::Debug for Cost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cost(gas={}, mana={}, risk={:.2}%)",
            self.gas,
            self.mana,
            self.trust_risk * 100.0
        )
    }
}

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}g/{}m/{:.1}%r", self.gas, self.mana, self.trust_risk * 100.0)
    }
}

// ============================================================================
// Budget – Kosten-Limit mit Tracking
// ============================================================================

/// Budget für Intent/Saga-Ausführung
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Budget {
    /// Maximale Kosten
    pub max_cost: Cost,
    /// Bereits verbraucht
    pub spent: Cost,
    /// Asset für Bezahlung
    pub payment_asset: String,
    /// Reservierter Betrag
    pub reserved_amount: u128,
}

impl Budget {
    /// Erstelle neues Budget
    pub fn new(max_gas: u64, max_mana: u64, max_risk: f32) -> Self {
        Self {
            max_cost: Cost::new(max_gas, max_mana, max_risk),
            spent: Cost::ZERO,
            payment_asset: "ERY".to_string(),
            reserved_amount: 0,
        }
    }

    /// Mit spezifischem Asset
    pub fn with_asset(mut self, asset: impl Into<String>, amount: u128) -> Self {
        self.payment_asset = asset.into();
        self.reserved_amount = amount;
        self
    }

    /// Prüfe ob Operation bezahlbar
    pub fn can_afford(&self, cost: &Cost) -> bool {
        let remaining = self.remaining();
        cost.gas <= remaining.gas
            && cost.mana <= remaining.mana
            && cost.trust_risk <= remaining.trust_risk
    }

    /// Verbleibende Kosten
    pub fn remaining(&self) -> Cost {
        Cost {
            gas: self.max_cost.gas.saturating_sub(self.spent.gas),
            mana: self.max_cost.mana.saturating_sub(self.spent.mana),
            trust_risk: (self.max_cost.trust_risk - self.spent.trust_risk).max(0.0),
        }
    }

    /// Verbrauche Kosten
    pub fn consume(&mut self, cost: Cost) -> Result<(), BudgetExhausted> {
        if !self.can_afford(&cost) {
            return Err(BudgetExhausted {
                required: cost,
                remaining: self.remaining(),
            });
        }
        self.spent = self.spent.seq(cost);
        Ok(())
    }

    /// Verbrauchter Prozentsatz (Gas-basiert)
    pub fn usage_percentage(&self) -> f64 {
        if self.max_cost.gas == 0 {
            return 0.0;
        }
        (self.spent.gas as f64 / self.max_cost.gas as f64) * 100.0
    }
}

impl Default for Budget {
    fn default() -> Self {
        Self::new(1_000_000, 100_000, 0.5)
    }
}

/// Budget erschöpft
#[derive(Debug, Clone)]
pub struct BudgetExhausted {
    pub required: Cost,
    pub remaining: Cost,
}

impl fmt::Display for BudgetExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Budget exhausted: required {}, remaining {}",
            self.required, self.remaining
        )
    }
}

impl std::error::Error for BudgetExhausted {}

// ============================================================================
// CostTable – Kosten-Tabelle pro Operation
// ============================================================================

/// Kosten-Tabelle für alle Operationen
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostTable {
    // === ECLVM OpCodes ===
    pub vm_push_const: Cost,
    pub vm_add: Cost,
    pub vm_mul: Cost,
    pub vm_div: Cost,
    pub vm_call_base: Cost,
    pub vm_call_per_arg: Cost,
    pub vm_host_call: Cost,
    pub vm_branch: Cost,
    pub vm_load: Cost,
    pub vm_store: Cost,

    // === Storage ===
    pub storage_get: Cost,
    pub storage_put_base: Cost,
    pub storage_put_per_kb: Cost,
    pub storage_delete: Cost,
    pub storage_query_base: Cost,
    pub storage_query_per_result: Cost,

    // === P2P ===
    pub p2p_publish: Cost,
    pub p2p_sync_request: Cost,
    pub p2p_connect: Cost,
    pub p2p_dht_put: Cost,
    pub p2p_dht_get: Cost,

    // === Blueprint ===
    pub blueprint_upload_base: Cost,
    pub blueprint_upload_per_kb: Cost,
    pub blueprint_deploy: Cost,
    pub blueprint_rate: Cost,

    // === Events ===
    pub event_create: Cost,
    pub event_witness: Cost,
    pub event_anchor: Cost,

    // === Trust ===
    pub trust_update: Cost,
    pub trust_query: Cost,
    pub trust_delegate: Cost,
}

impl Default for CostTable {
    fn default() -> Self {
        Self {
            // ECLVM
            vm_push_const: Cost::new(1, 0, 0.0),
            vm_add: Cost::new(2, 0, 0.0),
            vm_mul: Cost::new(3, 0, 0.0),
            vm_div: Cost::new(5, 0, 0.0),
            vm_call_base: Cost::new(10, 0, 0.0),
            vm_call_per_arg: Cost::new(2, 0, 0.0),
            vm_host_call: Cost::new(50, 10, 0.1),
            vm_branch: Cost::new(3, 0, 0.0),
            vm_load: Cost::new(5, 0, 0.0),
            vm_store: Cost::new(10, 0, 0.0),

            // Storage
            storage_get: Cost::new(0, 5, 0.0),
            storage_put_base: Cost::new(0, 10, 0.0),
            storage_put_per_kb: Cost::new(0, 10, 0.0),
            storage_delete: Cost::new(0, 5, 0.0),
            storage_query_base: Cost::new(0, 20, 0.0),
            storage_query_per_result: Cost::new(0, 5, 0.0),

            // P2P
            p2p_publish: Cost::new(0, 10, 0.01),
            p2p_sync_request: Cost::new(0, 5, 0.05),
            p2p_connect: Cost::new(0, 20, 0.1),
            p2p_dht_put: Cost::new(0, 20, 0.02),
            p2p_dht_get: Cost::new(0, 10, 0.01),

            // Blueprint
            blueprint_upload_base: Cost::new(0, 500, 0.05),
            blueprint_upload_per_kb: Cost::new(0, 20, 0.0),
            blueprint_deploy: Cost::new(0, 50, 0.02),
            blueprint_rate: Cost::new(0, 10, 0.02),

            // Events
            event_create: Cost::new(0, 20, 0.01),
            event_witness: Cost::new(0, 5, 0.02),
            event_anchor: Cost::new(0, 100, 0.05),

            // Trust
            trust_update: Cost::new(0, 15, 0.05),
            trust_query: Cost::new(0, 5, 0.0),
            trust_delegate: Cost::new(0, 30, 0.1),
        }
    }
}

impl CostTable {
    /// ECLVM-Operation Kosten berechnen
    pub fn vm_op_cost(&self, op: &str, args: usize) -> Cost {
        match op {
            "push" | "const" => self.vm_push_const,
            "add" | "sub" => self.vm_add,
            "mul" => self.vm_mul,
            "div" | "mod" => self.vm_div,
            "call" => self.vm_call_base.seq(self.vm_call_per_arg.scale(args as f32)),
            "host" => self.vm_host_call,
            "br" | "br_if" => self.vm_branch,
            "load" | "get" => self.vm_load,
            "store" | "set" => self.vm_store,
            _ => Cost::new(5, 0, 0.0), // Default für unbekannte Ops
        }
    }

    /// Storage-Operation Kosten berechnen
    pub fn storage_op_cost(&self, op: &str, size_kb: usize) -> Cost {
        match op {
            "get" => self.storage_get,
            "put" => self
                .storage_put_base
                .seq(self.storage_put_per_kb.scale(size_kb as f32)),
            "delete" => self.storage_delete,
            "query" => self.storage_query_base,
            _ => Cost::new(0, 10, 0.0),
        }
    }
}

// ============================================================================
// Compile-Time Assertions
// ============================================================================

// Note: Cost ist 24 Bytes wegen Padding (8 + 8 + 4 + 4 padding)
// Das ist OK für Cache-Effizienz

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_sequential() {
        let c1 = Cost::new(100, 50, 0.1);
        let c2 = Cost::new(200, 30, 0.1);

        let total = c1.seq(c2);

        assert_eq!(total.gas, 300);
        assert_eq!(total.mana, 80);
        // 1 - (1-0.1)(1-0.1) = 1 - 0.81 = 0.19
        assert!((total.trust_risk - 0.19).abs() < 0.001);
    }

    #[test]
    fn test_cost_parallel() {
        let c1 = Cost::new(100, 50, 0.1);
        let c2 = Cost::new(200, 30, 0.05);

        let total = c1.par(c2);

        assert_eq!(total.gas, 200); // max
        assert_eq!(total.mana, 80); // sum
        assert_eq!(total.trust_risk, 0.1); // max
    }

    #[test]
    fn test_budget_consume() {
        let mut budget = Budget::new(1000, 500, 0.5);

        assert!(budget.can_afford(&Cost::new(100, 50, 0.1)));
        budget.consume(Cost::new(100, 50, 0.1)).unwrap();

        assert_eq!(budget.spent.gas, 100);
        assert_eq!(budget.remaining().gas, 900);
    }

    #[test]
    fn test_budget_exhausted() {
        let mut budget = Budget::new(100, 50, 0.5);

        let result = budget.consume(Cost::new(200, 100, 0.1));

        assert!(result.is_err());
    }

    #[test]
    fn test_cost_scale() {
        let c = Cost::new(100, 50, 0.2);
        let scaled = c.scale(2.0);

        assert_eq!(scaled.gas, 200);
        assert_eq!(scaled.mana, 100);
        assert!((scaled.trust_risk - 0.4).abs() < 0.001);
    }

    #[test]
    fn test_cost_add_operator() {
        let c1 = Cost::new(10, 5, 0.05);
        let c2 = Cost::new(20, 10, 0.05);

        let total = c1 + c2;

        assert_eq!(total.gas, 30);
        assert_eq!(total.mana, 15);
    }
}
