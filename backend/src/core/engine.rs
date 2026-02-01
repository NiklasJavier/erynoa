//! # Unified Engine Layer
//!
//! ExecutionContext-aware Wrapper für Core-Engines gemäß IPS v1.2.0.
//!
//! ## Architektur
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        ENGINE LAYER (Phase 3)                       │
//! ├─────────────────────────────────────────────────────────────────────┤
//! │  EventProcessor    - Events mit Gas-Accounting (Κ9-Κ12)            │
//! │  TrustUpdater      - Trust-Updates mit History (Κ2-Κ5)             │
//! │  FormulaComputer   - Weltformel mit Cost-Algebra (Κ15)             │
//! │  FinalityTracker   - Consensus mit State-Machine (Κ10)             │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! Diese Wrapper delegieren an die Legacy-Engines, fügen aber:
//! - Gas/Mana-Accounting
//! - Event-Emission über ExecutionContext
//! - Trust-Gate-Checks
//! - Cost-Tracking

use crate::domain::unified::{
    Cost, EventId, EventPayload, FinalityLevel, FinalityState, Hash32, TemporalCoord,
    TrustDimension, TrustRecord, TrustUpdateReason, TrustVector6D, UniversalId,
};
use crate::execution::{gas_costs, mana_costs, ExecutionContext, ExecutionError, ExecutionResult};

// ============================================================================
// Gas-Kosten für Core-Operationen
// ============================================================================

/// Gas-Kosten für Event-Operationen
pub mod event_gas {
    /// Validierung eines Events
    pub const VALIDATE: u64 = 200;
    /// Hinzufügen zum DAG
    pub const ADD_TO_DAG: u64 = 300;
    /// Parent-Lookup pro Parent
    pub const PARENT_LOOKUP: u64 = 50;
    /// Zyklus-Detection
    pub const CYCLE_CHECK: u64 = 100;
    /// Signatur-Verifikation
    pub const SIGNATURE_VERIFY: u64 = 500;
}

/// Gas-Kosten für Trust-Operationen
pub mod trust_gas {
    /// Trust-Lookup
    pub const LOOKUP: u64 = 25;
    /// Trust-Update (eine Dimension)
    pub const UPDATE: u64 = 50;
    /// Kombination (Κ5)
    pub const COMBINE: u64 = 30;
    /// Chain-Trust (Τ1)
    pub const CHAIN_TRUST_BASE: u64 = 40;
    /// Pro Hop in der Kette
    pub const CHAIN_TRUST_PER_HOP: u64 = 20;
    /// History-Eintrag
    pub const HISTORY_ENTRY: u64 = 15;
}

/// Gas-Kosten für Weltformel-Operationen
pub mod formula_gas {
    /// Contribution-Berechnung
    pub const CONTRIBUTION: u64 = 150;
    /// Surprisal-Berechnung
    pub const SURPRISAL: u64 = 80;
    /// Sigmoid-Berechnung
    pub const SIGMOID: u64 = 20;
    /// Globale Aggregation pro Subject
    pub const AGGREGATE_PER_SUBJECT: u64 = 10;
    /// Globale Berechnung (Κ15b)
    pub const GLOBAL_COMPUTE: u64 = 500;
}

// ============================================================================
// EventProcessor - Event-Verarbeitung mit Context
// ============================================================================

/// Event-Verarbeitung mit ExecutionContext
///
/// Wrapped die Legacy-EventEngine mit Gas-Accounting und Context-Integration.
pub struct EventProcessor;

impl EventProcessor {
    /// Validiere Event-Struktur (Κ9)
    ///
    /// Gas: VALIDATE + PARENT_LOOKUP × num_parents
    pub fn validate(
        ctx: &mut ExecutionContext,
        event_id: &EventId,
        parents: &[EventId],
        payload: &EventPayload,
    ) -> ExecutionResult<()> {
        // Gas für Basis-Validierung
        ctx.consume_gas(event_gas::VALIDATE)?;

        // Gas für Parent-Lookups
        let parent_cost = event_gas::PARENT_LOOKUP * parents.len() as u64;
        ctx.consume_gas(parent_cost)?;

        // Gas für Zyklus-Check (nur wenn Parents vorhanden)
        if !parents.is_empty() {
            ctx.consume_gas(event_gas::CYCLE_CHECK)?;
        }

        // Kosten tracken
        let total_gas = event_gas::VALIDATE + parent_cost + event_gas::CYCLE_CHECK;
        ctx.track_cost(Cost::new(total_gas, 0, 0.0));

        // Payload-spezifische Validierung
        Self::validate_payload(ctx, payload)?;

        Ok(())
    }

    /// Validiere Event-Payload
    fn validate_payload(ctx: &mut ExecutionContext, payload: &EventPayload) -> ExecutionResult<()> {
        match payload {
            EventPayload::Transfer { amount, .. } => {
                if *amount == 0 {
                    return Err(ExecutionError::InvalidInput(
                        "Transfer amount cannot be zero".into(),
                    ));
                }
            }
            EventPayload::Delegate { trust_factor, .. } => {
                if *trust_factor <= 0.0 || *trust_factor > 1.0 {
                    return Err(ExecutionError::TrustDecayViolation {
                        factor: *trust_factor,
                    });
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Verarbeite Signatur-Verifikation
    ///
    /// Gas: SIGNATURE_VERIFY
    pub fn verify_signature(
        ctx: &mut ExecutionContext,
        _signature: &[u8],
        _public_key: &[u8],
        _message: &[u8],
    ) -> ExecutionResult<bool> {
        ctx.consume_gas(event_gas::SIGNATURE_VERIFY)?;
        ctx.track_cost(Cost::new(event_gas::SIGNATURE_VERIFY, 0, 0.0));

        // TODO: Echte Signatur-Verifikation
        Ok(true)
    }

    /// Emittiere Event über Context
    ///
    /// Gas: EVENT_EMIT + Storage-Kosten
    pub fn emit_event(
        ctx: &mut ExecutionContext,
        event_type: &str,
        payload: &[u8],
    ) -> ExecutionResult<UniversalId> {
        ctx.consume_gas(gas_costs::EVENT_EMIT)?;

        // Storage-Kosten für Payload
        let storage_cost = gas_costs::STORAGE_PER_BYTE * payload.len() as u64;
        ctx.consume_gas(storage_cost)?;

        // Mana für persistenten Storage
        ctx.consume_mana(mana_costs::STORAGE_WRITE)?;

        // Event-ID generieren
        let event_id = UniversalId::new(UniversalId::TAG_EVENT, ctx.state.epoch as u16, payload);

        // Event emittieren
        ctx.emit_raw(event_type, payload);

        // Kosten tracken
        let total_gas = gas_costs::EVENT_EMIT + storage_cost;
        ctx.track_cost(Cost::new(total_gas, mana_costs::STORAGE_WRITE, 0.0));

        Ok(event_id)
    }

    /// Update Finality-Level (Κ10)
    ///
    /// Prüft dass Finality nie sinken kann (Permanenz).
    pub fn update_finality(
        ctx: &mut ExecutionContext,
        current: FinalityLevel,
        new: FinalityLevel,
    ) -> ExecutionResult<FinalityLevel> {
        // Κ10: Finality kann nur steigen
        if new < current {
            return Err(ExecutionError::FinalityRegression {
                event_id: "unknown".into(),
                old_level: current as u8,
                new_level: new as u8,
            });
        }

        // Gas für State-Transition
        ctx.consume_gas(50)?;

        Ok(new)
    }
}

// ============================================================================
// TrustUpdater - Trust-Updates mit Context
// ============================================================================

/// Trust-Updates mit ExecutionContext und TrustRecord
pub struct TrustUpdater;

impl TrustUpdater {
    /// Lookup Trust für Subject
    ///
    /// Gas: LOOKUP
    pub fn lookup(
        ctx: &mut ExecutionContext,
        trust_store: &std::collections::HashMap<UniversalId, TrustRecord>,
        subject: &UniversalId,
    ) -> ExecutionResult<Option<TrustRecord>> {
        ctx.consume_gas(trust_gas::LOOKUP)?;
        ctx.track_cost(Cost::new(trust_gas::LOOKUP, 0, 0.0));

        Ok(trust_store.get(subject).cloned())
    }

    /// Κ4: Asymmetrisches Trust-Update
    ///
    /// Gas: UPDATE + HISTORY_ENTRY
    pub fn update(
        ctx: &mut ExecutionContext,
        record: &mut TrustRecord,
        dimension: TrustDimension,
        delta: f32,
        reason: TrustUpdateReason,
    ) -> ExecutionResult<()> {
        ctx.consume_gas(trust_gas::UPDATE)?;
        ctx.consume_gas(trust_gas::HISTORY_ENTRY)?;

        // Update über TrustRecord (nutzt Asymmetrie aus TrustDimension)
        let coord = crate::domain::unified::TemporalCoord::new(
            ctx.state.epoch * 1000,
            ctx.state.lamport as u32,
            1,
        );
        record.update(dimension, delta, reason, coord);

        // Kosten tracken
        ctx.track_cost(Cost::new(
            trust_gas::UPDATE + trust_gas::HISTORY_ENTRY,
            0,
            0.0,
        ));

        // Lamport-Clock inkrementieren
        ctx.state.tick();

        Ok(())
    }

    /// Κ5: Kombiniere Trust aus mehreren Quellen
    ///
    /// Gas: COMBINE × num_sources
    pub fn combine(
        ctx: &mut ExecutionContext,
        sources: &[TrustVector6D],
    ) -> ExecutionResult<TrustVector6D> {
        let gas = trust_gas::COMBINE * sources.len() as u64;
        ctx.consume_gas(gas)?;
        ctx.track_cost(Cost::new(gas, 0, 0.0));

        if sources.is_empty() {
            return Ok(TrustVector6D::NEWCOMER);
        }

        // Probabilistische Kombination
        let mut result = sources[0];
        for other in &sources[1..] {
            result = result.combine(other);
        }

        Ok(result)
    }

    /// Τ1: Berechne Chain-Trust über mehrere Hops
    ///
    /// Gas: CHAIN_TRUST_BASE + CHAIN_TRUST_PER_HOP × chain_length
    pub fn chain_trust(ctx: &mut ExecutionContext, chain: &[f32]) -> ExecutionResult<f32> {
        let gas = trust_gas::CHAIN_TRUST_BASE + trust_gas::CHAIN_TRUST_PER_HOP * chain.len() as u64;
        ctx.consume_gas(gas)?;
        ctx.track_cost(Cost::new(gas, 0, 0.0));

        if chain.is_empty() {
            return Ok(1.0);
        }

        // Chain-Trust mit √n Dämpfung
        let n = chain.len() as f32;
        let log_sum: f32 = chain.iter().map(|t| t.max(1e-10).ln()).sum();
        let result = (log_sum / n.sqrt()).exp();

        Ok(result)
    }

    /// Trust-Gate prüfen
    ///
    /// Prüft ob der Executor die erforderliche Trust-Schwelle erfüllt.
    pub fn check_gate(ctx: &ExecutionContext, required_trust: f32) -> ExecutionResult<()> {
        let effective = ctx
            .trust_context
            .effective_trust
            .weighted_norm(&TrustVector6D::default_weights());

        if effective < required_trust {
            return Err(ExecutionError::TrustGateBlocked {
                required: required_trust,
                actual: effective,
            });
        }

        Ok(())
    }
}

// ============================================================================
// FormulaComputer - Weltformel-Berechnung mit Context
// ============================================================================

/// Weltformel-Berechnung mit ExecutionContext und Cost-Algebra
pub struct FormulaComputer;

impl FormulaComputer {
    /// Berechne Activity 𝔸(s)
    ///
    /// Gas: Minimal (nur Arithmetik)
    pub fn compute_activity(
        ctx: &mut ExecutionContext,
        recent_events: u64,
        kappa: u64,
    ) -> ExecutionResult<f64> {
        ctx.consume_gas(10)?;

        let n = recent_events as f64;
        let k = kappa as f64;
        let activity = n / (n + k);

        Ok(activity)
    }

    /// Berechne Trust-gedämpfte Surprisal 𝒮(s) = ‖𝕎‖² · ℐ (Κ15a)
    ///
    /// Gas: SURPRISAL
    pub fn compute_surprisal(
        ctx: &mut ExecutionContext,
        raw_surprisal: f64,
        trust_norm: f32,
    ) -> ExecutionResult<f64> {
        ctx.consume_gas(formula_gas::SURPRISAL)?;
        ctx.track_cost(Cost::new(formula_gas::SURPRISAL, 0, 0.0));

        // 𝒮 = ‖𝕎‖² × ℐ
        let dampened = (trust_norm as f64).powi(2) * raw_surprisal;

        Ok(dampened)
    }

    /// Berechne Sigmoid σ⃗(x) (Κ15c)
    ///
    /// Gas: SIGMOID
    pub fn sigmoid(ctx: &mut ExecutionContext, x: f64) -> ExecutionResult<f64> {
        ctx.consume_gas(formula_gas::SIGMOID)?;

        let result = 1.0 / (1.0 + (-x).exp());

        Ok(result)
    }

    /// Berechne einzelne Contribution (Κ15b)
    ///
    /// `contribution(s) = 𝔸(s) · σ⃗( ‖𝕎‖_w · ln|ℂ| · 𝒮 ) · Ĥ(s) · w(s,t)`
    ///
    /// Gas: CONTRIBUTION
    /// Returns: (contribution_value, computation_cost)
    pub fn compute_contribution(
        ctx: &mut ExecutionContext,
        activity: f64,
        trust_norm: f32,
        causal_connectivity: u64,
        surprisal: f64,
        human_factor: f64,
        temporal_weight: f64,
    ) -> ExecutionResult<(f64, Cost)> {
        ctx.consume_gas(formula_gas::CONTRIBUTION)?;

        // Inner term: ‖𝕎‖_w · ln|ℂ| · 𝒮
        let ln_connectivity = (causal_connectivity.max(1) as f64).ln();
        let inner = (trust_norm as f64) * ln_connectivity * surprisal;

        // Sigmoid
        let sigmoid = 1.0 / (1.0 + (-inner).exp());

        // Final contribution
        let contribution = activity * sigmoid * human_factor * temporal_weight;

        let cost = Cost::new(formula_gas::CONTRIBUTION, 0, 0.0);
        ctx.track_cost(cost);

        Ok((contribution, cost))
    }

    /// Berechne globale Weltformel 𝔼 (Κ15b)
    ///
    /// `𝔼 = Σ contribution(s)`
    ///
    /// Gas: AGGREGATE_PER_SUBJECT × num_subjects
    pub fn compute_global(
        ctx: &mut ExecutionContext,
        contributions: &[f64],
    ) -> ExecutionResult<(f64, Cost)> {
        let gas = formula_gas::AGGREGATE_PER_SUBJECT * contributions.len() as u64;
        ctx.consume_gas(gas)?;

        let total: f64 = contributions.iter().sum();
        let cost = Cost::new(gas, 0, 0.0);
        ctx.track_cost(cost);

        Ok((total, cost))
    }
}

// ============================================================================
// FinalityTracker - Finality-State-Machine mit Context
// ============================================================================

/// Finality-Tracking mit ExecutionContext (Κ10)
pub struct FinalityTracker;

impl FinalityTracker {
    /// Erstelle initialen FinalityState
    pub fn initial(coord: TemporalCoord) -> FinalityState {
        FinalityState::nascent(coord)
    }

    /// Transition zu Validated
    ///
    /// Erfordert: Signatur-Verifikation erfolgreich
    pub fn to_validated(
        ctx: &mut ExecutionContext,
        state: &mut FinalityState,
    ) -> ExecutionResult<()> {
        ctx.consume_gas(50)?;

        let coord = TemporalCoord::new(
            ctx.state.epoch * 1000 + ctx.state.lamport,
            ctx.state.lamport as u32,
            1,
        );

        state
            .validate(coord)
            .map_err(|_e| ExecutionError::FinalityRegression {
                event_id: "unknown".into(),
                old_level: state.level as u8,
                new_level: FinalityLevel::Validated as u8,
            })?;

        Ok(())
    }

    /// Transition zu Witnessed
    ///
    /// Erfordert: min_witnesses Zeugen mit Trust ≥ threshold
    pub fn to_witnessed(
        ctx: &mut ExecutionContext,
        state: &mut FinalityState,
        witness_count: u32,
        min_witnesses: u32,
        min_trust: f32,
    ) -> ExecutionResult<()> {
        ctx.consume_gas(75)?;

        if witness_count < min_witnesses {
            return Err(ExecutionError::InvalidInput(format!(
                "Need {} witnesses, have {}",
                min_witnesses, witness_count
            )));
        }

        let coord = TemporalCoord::new(
            ctx.state.epoch * 1000 + ctx.state.lamport,
            ctx.state.lamport as u32,
            1,
        );

        state
            .witness(witness_count, min_trust, coord)
            .map_err(|_e| ExecutionError::FinalityRegression {
                event_id: "unknown".into(),
                old_level: state.level as u8,
                new_level: FinalityLevel::Witnessed as u8,
            })?;

        Ok(())
    }

    /// Transition zu Anchored (L1-Verankerung)
    pub fn to_anchored(
        ctx: &mut ExecutionContext,
        state: &mut FinalityState,
        anchor_hash: Hash32,
        anchor_system: &str,
    ) -> ExecutionResult<()> {
        ctx.consume_gas(100)?;
        ctx.consume_mana(mana_costs::P2P_BROADCAST)?;

        let coord = TemporalCoord::new(
            ctx.state.epoch * 1000 + ctx.state.lamport,
            ctx.state.lamport as u32,
            1,
        );

        state
            .anchor(anchor_hash, anchor_system, coord)
            .map_err(|_e| ExecutionError::FinalityRegression {
                event_id: "unknown".into(),
                old_level: state.level as u8,
                new_level: FinalityLevel::Anchored as u8,
            })?;

        Ok(())
    }

    /// Revert-Wahrscheinlichkeit (sinkt mit Confirmations)
    pub fn revert_probability(
        ctx: &mut ExecutionContext,
        state: &FinalityState,
        confirmations: u32,
    ) -> ExecutionResult<f64> {
        ctx.consume_gas(20)?;

        // P(revert) ≈ 2^(-confirmations) für PoS
        let prob = 0.5_f64.powi(confirmations as i32);

        Ok(prob)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::unified::TemporalCoord;

    fn test_ctx() -> ExecutionContext {
        ExecutionContext::default_for_testing()
    }

    #[test]
    fn test_event_processor_validate() {
        let mut ctx = test_ctx();
        let event_id = UniversalId::new(UniversalId::TAG_EVENT, 1, b"test");
        let parents = vec![];
        let payload = EventPayload::Custom {
            event_type: "test".into(),
            data: vec![],
        };

        let result = EventProcessor::validate(&mut ctx, &event_id, &parents, &payload);
        assert!(result.is_ok());
        assert!(ctx.gas_remaining < ctx.gas_initial);
    }

    #[test]
    fn test_event_processor_emit() {
        let mut ctx = test_ctx();

        let result = EventProcessor::emit_event(&mut ctx, "test.event", b"payload");
        assert!(result.is_ok());

        let event_id = result.unwrap();
        assert_eq!(event_id.type_tag(), UniversalId::TAG_EVENT);
    }

    #[test]
    fn test_trust_updater_combine() {
        let mut ctx = test_ctx();

        let t1 = TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5);
        let t2 = TrustVector6D::new(0.5, 0.5, 0.5, 0.5, 0.5, 0.5);

        let result = TrustUpdater::combine(&mut ctx, &[t1, t2]);
        assert!(result.is_ok());

        let combined = result.unwrap();
        // 1 - (1-0.5)(1-0.5) = 0.75
        assert!((combined.r - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_trust_updater_chain() {
        let mut ctx = test_ctx();

        let chain = vec![0.8, 0.8, 0.8];
        let result = TrustUpdater::chain_trust(&mut ctx, &chain);
        assert!(result.is_ok());

        let chain_trust = result.unwrap();
        assert!(chain_trust > 0.5 && chain_trust < 1.0);
    }

    #[test]
    fn test_trust_gate_blocked() {
        let mut ctx = ExecutionContext::minimal(); // Newcomer trust

        let result = TrustUpdater::check_gate(&ctx, 0.5);
        assert!(matches!(
            result,
            Err(ExecutionError::TrustGateBlocked { .. })
        ));
    }

    #[test]
    fn test_formula_computer_activity() {
        let mut ctx = test_ctx();

        let result = FormulaComputer::compute_activity(&mut ctx, 50, 10);
        assert!(result.is_ok());

        let activity = result.unwrap();
        // 50 / (50 + 10) = 0.833
        assert!((activity - 0.833).abs() < 0.01);
    }

    #[test]
    fn test_formula_computer_surprisal() {
        let mut ctx = test_ctx();

        let result = FormulaComputer::compute_surprisal(&mut ctx, 4.0, 0.5);
        assert!(result.is_ok());

        let surprisal = result.unwrap();
        // 0.5² × 4.0 = 0.25 × 4.0 = 1.0
        assert!((surprisal - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_formula_computer_sigmoid() {
        let mut ctx = test_ctx();

        let result = FormulaComputer::sigmoid(&mut ctx, 0.0);
        assert!(result.is_ok());
        assert!((result.unwrap() - 0.5).abs() < 0.001);

        let result2 = FormulaComputer::sigmoid(&mut ctx, 10.0);
        assert!(result2.unwrap() > 0.999);
    }

    #[test]
    fn test_finality_tracker_transitions() {
        let mut ctx = test_ctx();
        let coord = TemporalCoord::new(1000, 1, 1);
        let mut state = FinalityTracker::initial(coord);

        // Initial → Validated
        let result = FinalityTracker::to_validated(&mut ctx, &mut state);
        assert!(result.is_ok());
        assert_eq!(state.level, FinalityLevel::Validated);

        // Validated → Witnessed
        let result = FinalityTracker::to_witnessed(&mut ctx, &mut state, 3, 3, 0.5);
        assert!(result.is_ok());
        assert_eq!(state.level, FinalityLevel::Witnessed);
    }

    #[test]
    fn test_finality_regression_blocked() {
        let mut ctx = test_ctx();

        let current = FinalityLevel::Witnessed;
        let new = FinalityLevel::Validated; // Regression!

        let result = EventProcessor::update_finality(&mut ctx, current, new);
        assert!(matches!(
            result,
            Err(ExecutionError::FinalityRegression { .. })
        ));
    }

    #[test]
    fn test_gas_consumption_tracking() {
        let mut ctx = test_ctx();
        let initial_gas = ctx.gas_remaining;

        // Mehrere Operationen
        FormulaComputer::compute_activity(&mut ctx, 10, 10).unwrap();
        FormulaComputer::compute_surprisal(&mut ctx, 2.0, 0.5).unwrap();
        FormulaComputer::sigmoid(&mut ctx, 0.0).unwrap();

        // Gas sollte gesunken sein
        assert!(ctx.gas_remaining < initial_gas);

        // Cost sollte getrackt sein
        assert!(ctx.accumulated_cost.gas > 0);
    }
}
