//! # Execution State Tracking
//!
//! Verbindet ExecutionContext mit UnifiedState für automatisches Tracking.
//!
//! ## Features
//!
//! - Automatische State-Updates bei Execution-Operationen
//! - Gas/Mana-Tracking über Unified State
//! - Event-Emission mit State-Propagation
//!
//! ## Architektur
//!
//! ```text
//! ┌───────────────────────────────────────────────────────────────────┐
//! │                    EXECUTION TRACKING                             │
//! │                                                                   │
//! │   ExecutionContext ──────────► TrackedContext                    │
//! │         │                            │                           │
//! │         │                            ▼                           │
//! │    Operations              StateIntegrator                       │
//! │         │                            │                           │
//! │         │                            ▼                           │
//! │         └──────────────────► UnifiedState                        │
//! │                           (Atomic Updates)                       │
//! │                                                                   │
//! └───────────────────────────────────────────────────────────────────┘
//! ```

use super::context::{Event, ExecutionContext, TrustContext, WorldState};
use super::error::{ExecutionError, ExecutionResult};
use crate::core::state_integration::{ExecutionObserver, StateIntegrator};
use crate::core::SharedUnifiedState;
use crate::domain::unified::UniversalId;
use std::sync::atomic::{AtomicU64, Ordering};

/// Globaler Context-ID-Counter
static CONTEXT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_context_id() -> u64 {
    CONTEXT_ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Tracked Execution Context - ExecutionContext mit State-Integration
///
/// Wraps ExecutionContext und propagiert alle Operationen zum UnifiedState.
pub struct TrackedContext {
    /// Inner ExecutionContext
    inner: ExecutionContext,

    /// State Integrator für Updates
    integrator: StateIntegrator,

    /// Unique Context ID für Tracking
    context_id: u64,

    /// Gas zu Beginn
    initial_gas: u64,

    /// Mana zu Beginn
    initial_mana: u64,

    /// Events emittiert während dieser Execution
    events_count: u64,
}

impl TrackedContext {
    /// Erstelle neuen Tracked Context
    pub fn new(
        state: WorldState,
        gas_budget: u64,
        mana_budget: u64,
        trust_context: TrustContext,
        integrator: StateIntegrator,
    ) -> Self {
        let context_id = next_context_id();
        let inner = ExecutionContext::new(state, gas_budget, mana_budget, trust_context);

        // Notify State: Execution gestartet
        integrator.on_execution_start(context_id);

        // Update Lamport Clock
        integrator
            .state()
            .execution
            .current_lamport
            .store(inner.state.lamport, Ordering::Relaxed);

        Self {
            inner,
            integrator,
            context_id,
            initial_gas: gas_budget,
            initial_mana: mana_budget,
            events_count: 0,
        }
    }

    /// Erstelle aus SharedUnifiedState
    pub fn with_unified_state(
        state: WorldState,
        gas_budget: u64,
        mana_budget: u64,
        trust_context: TrustContext,
        unified_state: SharedUnifiedState,
    ) -> Self {
        let integrator = StateIntegrator::new(unified_state);
        Self::new(state, gas_budget, mana_budget, trust_context, integrator)
    }

    /// Zugriff auf inner Context
    pub fn inner(&self) -> &ExecutionContext {
        &self.inner
    }

    /// Mutabler Zugriff auf inner Context
    pub fn inner_mut(&mut self) -> &mut ExecutionContext {
        &mut self.inner
    }

    /// Context ID
    pub fn id(&self) -> u64 {
        self.context_id
    }

    // =========================================================================
    // Tracked Gas Operations
    // =========================================================================

    /// Verbrauche Gas mit State-Tracking
    pub fn consume_gas(&mut self, amount: u64) -> ExecutionResult<()> {
        let result = self.inner.consume_gas(amount);

        match &result {
            Ok(_) => {
                self.integrator.on_gas_consumed(amount);
            }
            Err(ExecutionError::GasExhausted {
                required,
                available,
            }) => {
                self.integrator.on_out_of_gas(*required, *available);
            }
            _ => {}
        }

        result
    }

    /// Refund Gas - erhöht Gas-Budget wieder
    pub fn refund_gas(&mut self, amount: u64) {
        let before = self.inner.gas_remaining;
        self.inner.gas_remaining = (self.inner.gas_remaining + amount).min(self.inner.gas_initial);
        let refunded = self.inner.gas_remaining - before;

        if refunded > 0 {
            self.integrator
                .state()
                .execution
                .gas_refunded
                .fetch_add(refunded, Ordering::Relaxed);
        }
    }

    /// Verbleibendes Gas
    pub fn gas_remaining(&self) -> u64 {
        self.inner.gas_remaining
    }

    /// Verbrauchtes Gas
    pub fn gas_consumed(&self) -> u64 {
        self.initial_gas.saturating_sub(self.inner.gas_remaining)
    }

    // =========================================================================
    // Tracked Mana Operations
    // =========================================================================

    /// Verbrauche Mana mit State-Tracking
    pub fn consume_mana(&mut self, amount: u64) -> ExecutionResult<()> {
        let result = self.inner.consume_mana(amount);

        if result.is_ok() {
            self.integrator.on_mana_consumed(amount);
        }

        result
    }

    /// Regeneriere Mana
    pub fn regenerate_mana(&mut self, amount: u64) {
        let before = self.inner.mana_remaining;
        self.inner.mana_remaining =
            (self.inner.mana_remaining + amount).min(self.inner.mana_initial);
        let regenerated = self.inner.mana_remaining - before;

        if regenerated > 0 {
            self.integrator
                .state()
                .execution
                .mana_regenerated
                .fetch_add(regenerated, Ordering::Relaxed);
        }
    }

    /// Verbleibendes Mana
    pub fn mana_remaining(&self) -> u64 {
        self.inner.mana_remaining
    }

    /// Verbrauchtes Mana
    pub fn mana_consumed(&self) -> u64 {
        self.initial_mana.saturating_sub(self.inner.mana_remaining)
    }

    // =========================================================================
    // Tracked Event Operations
    // =========================================================================

    /// Emit Event mit State-Tracking
    pub fn emit(&mut self, event: Event) {
        self.inner.emit(event);
        self.events_count += 1;
        // Events werden separat über EventObserver getrackt
    }

    /// Emit rohe Event-Daten
    pub fn emit_raw(&mut self, event_type: &str, payload: &[u8]) {
        self.inner.emit_raw(event_type, payload);
        self.events_count += 1;
    }

    /// Anzahl emittierter Events
    pub fn events_emitted(&self) -> u64 {
        self.events_count
    }

    // =========================================================================
    // Tracked Execution
    // =========================================================================

    /// Führe Operation aus mit automatischem Tracking
    pub fn execute<T, F>(&mut self, op: F) -> ExecutionResult<T>
    where
        F: FnOnce(&mut ExecutionContext) -> ExecutionResult<T>,
    {
        self.inner.execute(op)
    }

    /// Finalisiere Context und propagiere zum State
    pub fn finalize(self, success: bool) -> ExecutionSummary {
        let duration_ms = self.inner.started_at.elapsed().as_millis() as u64;
        let gas_used = self.gas_consumed();
        let mana_used = self.mana_consumed();

        // Notify State: Execution abgeschlossen
        self.integrator.on_execution_complete(
            self.context_id,
            success,
            gas_used,
            mana_used,
            self.events_count,
            duration_ms,
        );

        // Update Epoch/Lamport falls nötig
        self.integrator
            .state()
            .execution
            .current_epoch
            .store(self.inner.state.epoch, Ordering::Relaxed);
        self.integrator
            .state()
            .execution
            .current_lamport
            .store(self.inner.state.lamport, Ordering::Relaxed);

        ExecutionSummary {
            context_id: self.context_id,
            success,
            gas_used,
            gas_refunded: self
                .initial_gas
                .saturating_sub(gas_used + self.inner.gas_remaining),
            mana_used,
            events_emitted: self.events_count,
            duration_ms,
            epoch: self.inner.state.epoch,
            lamport: self.inner.state.lamport,
        }
    }
}

impl std::ops::Deref for TrackedContext {
    type Target = ExecutionContext;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for TrackedContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Zusammenfassung einer Execution
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    pub context_id: u64,
    pub success: bool,
    pub gas_used: u64,
    pub gas_refunded: u64,
    pub mana_used: u64,
    pub events_emitted: u64,
    pub duration_ms: u64,
    pub epoch: u64,
    pub lamport: u64,
}

// ============================================================================
// TRACKED EXECUTION BUILDER
// ============================================================================

/// Builder für TrackedContext
pub struct TrackedContextBuilder {
    state: Option<WorldState>,
    gas_budget: u64,
    mana_budget: u64,
    trust_context: Option<TrustContext>,
    unified_state: Option<SharedUnifiedState>,
}

impl TrackedContextBuilder {
    pub fn new() -> Self {
        Self {
            state: None,
            gas_budget: 1_000_000,
            mana_budget: 100_000,
            trust_context: None,
            unified_state: None,
        }
    }

    pub fn with_world_state(mut self, state: WorldState) -> Self {
        self.state = Some(state);
        self
    }

    pub fn with_gas_budget(mut self, gas: u64) -> Self {
        self.gas_budget = gas;
        self
    }

    pub fn with_mana_budget(mut self, mana: u64) -> Self {
        self.mana_budget = mana;
        self
    }

    pub fn with_trust_context(mut self, trust: TrustContext) -> Self {
        self.trust_context = Some(trust);
        self
    }

    pub fn with_unified_state(mut self, state: SharedUnifiedState) -> Self {
        self.unified_state = Some(state);
        self
    }

    pub fn build(self) -> TrackedContext {
        let state = self.state.unwrap_or_else(|| WorldState::new(1));
        let trust_context = self.trust_context.unwrap_or_else(|| {
            let id = UniversalId::new(UniversalId::TAG_DID, 1, b"default");
            TrustContext::direct(id, crate::domain::unified::TrustVector6D::NEWCOMER)
        });

        let unified_state = self
            .unified_state
            .unwrap_or_else(crate::core::create_unified_state);

        TrackedContext::with_unified_state(
            state,
            self.gas_budget,
            self.mana_budget,
            trust_context,
            unified_state,
        )
    }
}

impl Default for TrackedContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::create_unified_state;
    use crate::domain::unified::TrustVector6D;

    fn test_trust_context() -> TrustContext {
        let id = UniversalId::new(UniversalId::TAG_DID, 1, b"test");
        TrustContext::direct(id, TrustVector6D::default())
    }

    #[test]
    fn test_tracked_gas() {
        let state = create_unified_state();
        let mut ctx = TrackedContext::with_unified_state(
            WorldState::new(1),
            10_000,
            1_000,
            test_trust_context(),
            state.clone(),
        );

        ctx.consume_gas(500).unwrap();
        ctx.consume_gas(300).unwrap();

        assert_eq!(ctx.gas_consumed(), 800);
        assert_eq!(ctx.gas_remaining(), 9200);

        let summary = ctx.finalize(true);
        assert!(summary.success);
        assert_eq!(summary.gas_used, 800);

        // State sollte aktualisiert sein
        let snapshot = state.snapshot();
        assert!(snapshot.execution.gas_consumed >= 800);
    }

    #[test]
    fn test_tracked_mana() {
        let state = create_unified_state();
        let mut ctx = TrackedContext::with_unified_state(
            WorldState::new(1),
            10_000,
            1_000,
            test_trust_context(),
            state.clone(),
        );

        ctx.consume_mana(100).unwrap();
        assert_eq!(ctx.mana_consumed(), 100);

        ctx.regenerate_mana(50);
        assert_eq!(ctx.mana_remaining(), 950);

        let snapshot = state.snapshot();
        assert!(snapshot.execution.mana_consumed >= 100);
        assert!(snapshot.execution.mana_regenerated >= 50);
    }

    #[test]
    fn test_builder() {
        let state = create_unified_state();

        let ctx = TrackedContextBuilder::new()
            .with_gas_budget(5_000)
            .with_mana_budget(500)
            .with_unified_state(state.clone())
            .build();

        assert_eq!(ctx.inner().gas_initial, 5_000);
        assert_eq!(ctx.inner().mana_initial, 500);
    }

    #[test]
    fn test_events_tracking() {
        let state = create_unified_state();
        let mut ctx = TrackedContext::with_unified_state(
            WorldState::new(1),
            10_000,
            1_000,
            test_trust_context(),
            state.clone(),
        );

        ctx.emit_raw("test.event", b"payload");
        ctx.emit_raw("test.event2", b"payload2");

        assert_eq!(ctx.events_emitted(), 2);

        let summary = ctx.finalize(true);
        assert_eq!(summary.events_emitted, 2);
    }
}
