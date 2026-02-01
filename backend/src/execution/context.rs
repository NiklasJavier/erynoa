//! # Execution Context
//!
//! ExecutionContext kapselt alle Seiteneffekte gemäß IPS v1.2.0.
//!
//! ## IPS-Monade ℳ
//!
//! Der Context entspricht der monadischen Komposition:
//! ```text
//! ℳ = State(WorldState) × Writer(Vec<Event>) × Error(ExecutionError)
//! ```
//!
//! In Rust umgesetzt als:
//! - State: `&mut ExecutionContext`
//! - Writer: `emitted_events: Vec<Event>`
//! - Error: `Result<T, ExecutionError>`
//!
//! ## Axiom-Referenz
//!
//! - **Κ11 (Prozess-Korrektheit)**: `{pre} Π {post}` durch execute()
//! - **Κ12 (Event-Erzeugung)**: emit() für Event-Generierung
//! - **Κ15b (Gas-Kosten)**: consume_gas() für Ressourcen-Tracking

use crate::domain::unified::{Cost, TrustVector6D, UniversalId};
use crate::execution::error::{ExecutionError, ExecutionResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

// ============================================================================
// World State
// ============================================================================

/// Globaler Zustand der Welt (State-Komponente der Monade)
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    /// Aktuelle Epoche
    pub epoch: u64,
    /// Lamport-Clock des Kontexts
    pub lamport: u64,
    /// Root-Realm-ID
    pub root_realm: Option<UniversalId>,
    /// Aktive Saga-IDs (für Koordination)
    pub active_sagas: Vec<UniversalId>,
}

impl WorldState {
    /// Erstelle neuen World State
    pub fn new(epoch: u64) -> Self {
        Self {
            epoch,
            lamport: 0,
            root_realm: None,
            active_sagas: Vec::new(),
        }
    }

    /// Inkrementiere Lamport-Clock
    pub fn tick(&mut self) -> u64 {
        self.lamport += 1;
        self.lamport
    }

    /// Synchronisiere mit empfangener Lamport-Clock
    pub fn sync(&mut self, received: u64) {
        self.lamport = self.lamport.max(received) + 1;
    }
}

// ============================================================================
// Trust Context
// ============================================================================

/// Trust-Kontext für die aktuelle Operation (Κ2-Κ5)
#[derive(Debug, Clone)]
pub struct TrustContext {
    /// Identität des Ausführenden
    pub executor_id: UniversalId,
    /// Trust-Vektor des Ausführenden
    pub executor_trust: TrustVector6D,
    /// Delegations-Kette (für Trust-Decay, Κ8)
    pub delegation_chain: Vec<DelegationHop>,
    /// Effektiver Trust nach Delegation
    pub effective_trust: TrustVector6D,
}

/// Ein Hop in der Delegations-Kette
#[derive(Debug, Clone)]
pub struct DelegationHop {
    /// DID des Delegierenden
    pub delegator: UniversalId,
    /// DID des Delegierten
    pub delegate: UniversalId,
    /// Trust-Faktor der Delegation (0, 1]
    pub trust_factor: f32,
}

impl TrustContext {
    /// Erstelle Trust-Kontext für direkte Ausführung (keine Delegation)
    pub fn direct(executor_id: UniversalId, executor_trust: TrustVector6D) -> Self {
        Self {
            executor_id,
            executor_trust,
            delegation_chain: Vec::new(),
            effective_trust: executor_trust,
        }
    }

    /// Erstelle Trust-Kontext mit Delegations-Kette
    pub fn delegated(
        executor_id: UniversalId,
        executor_trust: TrustVector6D,
        chain: Vec<DelegationHop>,
    ) -> Self {
        // Berechne effektiven Trust durch Multiplikation der Faktoren (Κ8)
        let total_factor: f32 = chain.iter().map(|h| h.trust_factor).product();
        let effective_trust = executor_trust.scale(total_factor);

        Self {
            executor_id,
            executor_trust,
            delegation_chain: chain,
            effective_trust,
        }
    }

    /// Prüfe ob Trust-Anforderung erfüllt ist
    pub fn meets_requirement(&self, required: f32) -> bool {
        self.effective_trust
            .weighted_norm(&TrustVector6D::default_weights())
            >= required
    }
}

// ============================================================================
// Event (Placeholder für Domain-Migration Phase 2)
// ============================================================================

/// Event für Writer-Aspekt der Monade
///
/// TODO: In Phase 2 durch unified/event.rs ersetzen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event-ID
    pub id: UniversalId,
    /// Event-Typ
    pub event_type: String,
    /// Payload (JSON-serialisiert)
    pub payload: Vec<u8>,
    /// Timestamp (Lamport)
    pub lamport: u64,
}

impl Event {
    /// Erstelle neues Event
    pub fn new(id: UniversalId, event_type: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            id,
            event_type: event_type.into(),
            payload,
            lamport: 0,
        }
    }
}

// ============================================================================
// Execution Context
// ============================================================================

/// Execution-Context kapselt alle Seiteneffekte (IPS-Monade ℳ)
///
/// # Design
///
/// Der Context ist die zentrale Struktur für alle Operationen:
/// - **State**: WorldState für globalen Zustand
/// - **Writer**: emitted_events für Event-Erzeugung
/// - **Reader**: trust_context für Berechtigungen
/// - **Resources**: gas_remaining, mana_remaining für Limits
///
/// # Beispiel
///
/// ```rust
/// use erynoa_api::execution::{ExecutionContext, ExecutionError};
///
/// fn process_intent(ctx: &mut ExecutionContext) -> Result<(), ExecutionError> {
///     // Gas verbrauchen
///     ctx.consume_gas(100)?;
///
///     // Operation ausführen
///     let result = expensive_computation();
///
///     // Event emittieren
///     ctx.emit_raw("intent.processed", &result);
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct ExecutionContext {
    // =========================================================================
    // State-Komponente
    // =========================================================================
    /// Globaler Weltzustand
    pub state: WorldState,

    // =========================================================================
    // Resource-Komponente
    // =========================================================================
    /// Verbleibendes Gas für Computation
    pub gas_remaining: u64,

    /// Initiales Gas-Budget
    pub gas_initial: u64,

    /// Verbleibendes Mana für Storage/Network
    pub mana_remaining: u64,

    /// Initiales Mana-Budget
    pub mana_initial: u64,

    // =========================================================================
    // Trust-Komponente (Reader)
    // =========================================================================
    /// Trust-Kontext des Ausführenden
    pub trust_context: TrustContext,

    // =========================================================================
    // Writer-Komponente
    // =========================================================================
    /// Emittierte Events
    pub emitted_events: Vec<Event>,

    // =========================================================================
    // Tracking
    // =========================================================================
    /// Akkumulierte Kosten
    pub accumulated_cost: Cost,

    /// Startzeit der Ausführung
    pub started_at: Instant,

    /// Maximale Ausführungszeit
    pub timeout: Duration,

    /// Zusätzliche Metadaten
    pub metadata: HashMap<String, String>,
}

impl ExecutionContext {
    // =========================================================================
    // Constructor
    // =========================================================================

    /// Erstelle neuen Execution-Context
    pub fn new(
        state: WorldState,
        gas_budget: u64,
        mana_budget: u64,
        trust_context: TrustContext,
    ) -> Self {
        Self {
            state,
            gas_remaining: gas_budget,
            gas_initial: gas_budget,
            mana_remaining: mana_budget,
            mana_initial: mana_budget,
            trust_context,
            emitted_events: Vec::new(),
            accumulated_cost: Cost::ZERO,
            started_at: Instant::now(),
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        }
    }

    /// Erstelle Context mit Standard-Werten (für Tests)
    pub fn default_for_testing() -> Self {
        let executor_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test-executor");
        let trust = TrustVector6D::default();
        let trust_context = TrustContext::direct(executor_id, trust);

        Self::new(WorldState::new(1), 1_000_000, 100_000, trust_context)
    }

    /// Erstelle minimalen Context (für Unit-Tests)
    pub fn minimal() -> Self {
        let executor_id = UniversalId::new(UniversalId::TAG_DID, 1, b"minimal");
        let trust_context = TrustContext::direct(executor_id, TrustVector6D::NEWCOMER);

        Self::new(WorldState::new(0), 10_000, 1_000, trust_context)
    }

    // =========================================================================
    // Monadische Operationen
    // =========================================================================

    /// Monadische bind-Operation über Result + Context-Mutation
    ///
    /// Entspricht: `m >>= f` in Haskell
    ///
    /// # Beispiel
    ///
    /// ```rust
    /// ctx.execute(|ctx| {
    ///     ctx.consume_gas(50)?;
    ///     Ok(compute_something())
    /// })?;
    /// ```
    pub fn execute<T, F>(&mut self, op: F) -> ExecutionResult<T>
    where
        F: FnOnce(&mut Self) -> ExecutionResult<T>,
    {
        // Pre-Check: Gas vorhanden?
        if self.gas_remaining == 0 {
            return Err(ExecutionError::GasExhausted {
                required: 1,
                available: 0,
            });
        }

        // Pre-Check: Timeout?
        if self.started_at.elapsed() > self.timeout {
            return Err(ExecutionError::NetworkTimeout {
                timeout_ms: self.timeout.as_millis() as u64,
            });
        }

        // Operation ausführen
        op(self)
    }

    /// Sequentielle Komposition mehrerer Operationen
    ///
    /// Entspricht: `m1 >> m2 >> m3` in Haskell
    pub fn execute_seq<T>(
        &mut self,
        ops: Vec<Box<dyn FnOnce(&mut Self) -> ExecutionResult<T>>>,
    ) -> ExecutionResult<Vec<T>> {
        let mut results = Vec::with_capacity(ops.len());
        for op in ops {
            results.push(self.execute(op)?);
        }
        Ok(results)
    }

    // =========================================================================
    // Gas Management (Κ15b)
    // =========================================================================

    /// Verbrauche Gas
    ///
    /// # Errors
    ///
    /// - `ExecutionError::GasExhausted` wenn nicht genug Gas verfügbar
    pub fn consume_gas(&mut self, amount: u64) -> ExecutionResult<()> {
        if self.gas_remaining < amount {
            return Err(ExecutionError::GasExhausted {
                required: amount,
                available: self.gas_remaining,
            });
        }
        self.gas_remaining -= amount;
        self.accumulated_cost.gas += amount;
        Ok(())
    }

    /// Verbrauche Mana
    ///
    /// # Errors
    ///
    /// - `ExecutionError::ManaExhausted` wenn nicht genug Mana verfügbar
    pub fn consume_mana(&mut self, amount: u64) -> ExecutionResult<()> {
        if self.mana_remaining < amount {
            return Err(ExecutionError::ManaExhausted {
                required: amount,
                available: self.mana_remaining,
            });
        }
        self.mana_remaining -= amount;
        self.accumulated_cost.mana += amount;
        Ok(())
    }

    /// Verbrauche Kosten (Gas + Mana + Trust-Risk)
    pub fn consume_cost(&mut self, cost: Cost) -> ExecutionResult<()> {
        self.consume_gas(cost.gas)?;
        self.consume_mana(cost.mana)?;
        self.accumulated_cost.trust_risk =
            1.0 - (1.0 - self.accumulated_cost.trust_risk) * (1.0 - cost.trust_risk);
        Ok(())
    }

    /// Tracke Kosten ohne Verbrauch (für Reporting)
    ///
    /// Diese Methode akkumuliert Kosten für das Reporting,
    /// verbraucht aber kein Gas/Mana. Nützlich wenn Gas bereits
    /// separat mit consume_gas() verbraucht wurde.
    pub fn track_cost(&mut self, cost: Cost) {
        self.accumulated_cost = self.accumulated_cost.seq(cost);
    }

    /// Verbleibende Gas-Menge
    #[inline]
    pub fn gas_left(&self) -> u64 {
        self.gas_remaining
    }

    /// Verbrauchte Gas-Menge
    #[inline]
    pub fn gas_used(&self) -> u64 {
        self.gas_initial - self.gas_remaining
    }

    /// Verbleibende Mana-Menge
    #[inline]
    pub fn mana_left(&self) -> u64 {
        self.mana_remaining
    }

    /// Verbrauchte Mana-Menge
    #[inline]
    pub fn mana_used(&self) -> u64 {
        self.mana_initial - self.mana_remaining
    }

    // =========================================================================
    // Event Emission (Κ12)
    // =========================================================================

    /// Emittiere Event
    pub fn emit(&mut self, event: Event) {
        self.emitted_events.push(event);
    }

    /// Emittiere Event mit automatischer ID-Generierung
    pub fn emit_raw(&mut self, event_type: &str, payload: &[u8]) {
        let lamport = self.state.tick();
        let id = UniversalId::new(
            UniversalId::TAG_EVENT,
            1,
            &[event_type.as_bytes(), &lamport.to_le_bytes(), payload].concat(),
        );

        let mut event = Event::new(id, event_type, payload.to_vec());
        event.lamport = lamport;
        self.emitted_events.push(event);
    }

    /// Anzahl emittierter Events
    #[inline]
    pub fn event_count(&self) -> usize {
        self.emitted_events.len()
    }

    /// Events konsumieren (für Persistierung)
    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.emitted_events)
    }

    // =========================================================================
    // Trust Checks (Κ2-Κ5)
    // =========================================================================

    /// Prüfe Trust-Gate
    ///
    /// # Errors
    ///
    /// - `ExecutionError::TrustGateBlocked` wenn Trust nicht ausreicht
    pub fn require_trust(&self, required: f32) -> ExecutionResult<()> {
        let actual = self
            .trust_context
            .effective_trust
            .weighted_norm(&TrustVector6D::default_weights());

        if actual < required {
            return Err(ExecutionError::TrustGateBlocked { required, actual });
        }
        Ok(())
    }

    /// Effektiver Trust-Wert
    pub fn effective_trust(&self) -> f32 {
        self.trust_context
            .effective_trust
            .weighted_norm(&TrustVector6D::default_weights())
    }

    // =========================================================================
    // State Management
    // =========================================================================

    /// Inkrementiere Lamport-Clock
    pub fn tick(&mut self) -> u64 {
        self.state.tick()
    }

    /// Synchronisiere Lamport-Clock
    pub fn sync_lamport(&mut self, received: u64) {
        self.state.sync(received);
    }

    /// Setze Metadata
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    /// Hole Metadata
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(String::as_str)
    }

    // =========================================================================
    // Timeout Management
    // =========================================================================

    /// Setze Timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Prüfe ob Timeout erreicht
    pub fn is_timed_out(&self) -> bool {
        self.started_at.elapsed() > self.timeout
    }

    /// Verbleibende Zeit
    pub fn time_remaining(&self) -> Duration {
        self.timeout.saturating_sub(self.started_at.elapsed())
    }

    // =========================================================================
    // Finalization
    // =========================================================================

    /// Finalisiere Context und gib Summary zurück
    pub fn finalize(self) -> ExecutionSummary {
        ExecutionSummary {
            gas_used: self.gas_used(),
            mana_used: self.mana_used(),
            events_emitted: self.emitted_events.len(),
            duration: self.started_at.elapsed(),
            accumulated_cost: self.accumulated_cost,
            final_lamport: self.state.lamport,
        }
    }
}

// ============================================================================
// Execution Summary
// ============================================================================

/// Zusammenfassung einer Execution
#[derive(Debug, Clone)]
pub struct ExecutionSummary {
    /// Verbrauchtes Gas
    pub gas_used: u64,
    /// Verbrauchtes Mana
    pub mana_used: u64,
    /// Anzahl emittierter Events
    pub events_emitted: usize,
    /// Ausführungsdauer
    pub duration: Duration,
    /// Akkumulierte Kosten
    pub accumulated_cost: Cost,
    /// Finale Lamport-Clock
    pub final_lamport: u64,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = ExecutionContext::default_for_testing();
        assert_eq!(ctx.gas_remaining, 1_000_000);
        assert_eq!(ctx.mana_remaining, 100_000);
        assert_eq!(ctx.emitted_events.len(), 0);
    }

    #[test]
    fn test_gas_consumption() {
        let mut ctx = ExecutionContext::minimal();
        assert_eq!(ctx.gas_remaining, 10_000);

        ctx.consume_gas(1000).unwrap();
        assert_eq!(ctx.gas_remaining, 9_000);
        assert_eq!(ctx.gas_used(), 1_000);

        ctx.consume_gas(9000).unwrap();
        assert_eq!(ctx.gas_remaining, 0);

        // Sollte fehlschlagen
        let result = ctx.consume_gas(1);
        assert!(matches!(result, Err(ExecutionError::GasExhausted { .. })));
    }

    #[test]
    fn test_mana_consumption() {
        let mut ctx = ExecutionContext::minimal();
        assert_eq!(ctx.mana_remaining, 1_000);

        ctx.consume_mana(500).unwrap();
        assert_eq!(ctx.mana_remaining, 500);

        // Sollte fehlschlagen
        let result = ctx.consume_mana(600);
        assert!(matches!(result, Err(ExecutionError::ManaExhausted { .. })));
    }

    #[test]
    fn test_cost_consumption() {
        let mut ctx = ExecutionContext::minimal();
        let cost = Cost::new(100, 50, 0.1);

        ctx.consume_cost(cost).unwrap();
        assert_eq!(ctx.gas_used(), 100);
        assert_eq!(ctx.mana_used(), 50);
        assert!((ctx.accumulated_cost.trust_risk - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_event_emission() {
        let mut ctx = ExecutionContext::minimal();

        ctx.emit_raw("test.event", b"payload");
        assert_eq!(ctx.event_count(), 1);
        assert_eq!(ctx.state.lamport, 1);

        ctx.emit_raw("test.event2", b"payload2");
        assert_eq!(ctx.event_count(), 2);
        assert_eq!(ctx.state.lamport, 2);

        let events = ctx.drain_events();
        assert_eq!(events.len(), 2);
        assert_eq!(ctx.event_count(), 0);
    }

    #[test]
    fn test_execute_monad() {
        let mut ctx = ExecutionContext::minimal();

        let result = ctx.execute(|ctx| {
            ctx.consume_gas(100)?;
            Ok(42)
        });

        assert_eq!(result.unwrap(), 42);
        assert_eq!(ctx.gas_used(), 100);
    }

    #[test]
    fn test_execute_nested() {
        let mut ctx = ExecutionContext::minimal();

        let result = ctx.execute(|ctx| {
            ctx.consume_gas(100)?;
            ctx.execute(|ctx| {
                ctx.consume_gas(200)?;
                Ok(10)
            })?;
            Ok(42)
        });

        assert_eq!(result.unwrap(), 42);
        assert_eq!(ctx.gas_used(), 300);
    }

    #[test]
    fn test_execute_failure_rollback() {
        let executor_id = UniversalId::new(UniversalId::TAG_DID, 1, b"test");
        let trust_context = TrustContext::direct(executor_id, TrustVector6D::NEWCOMER);

        // Erstelle Context mit genau 150 Gas
        let mut ctx = ExecutionContext::new(WorldState::new(0), 150, 1_000, trust_context);
        assert_eq!(ctx.gas_remaining, 150);
        assert_eq!(ctx.gas_initial, 150);

        let result: ExecutionResult<i32> = ctx.execute(|ctx| {
            ctx.consume_gas(100)?;
            ctx.consume_gas(100)?; // Sollte fehlschlagen (150 - 100 = 50 verbleibend, aber 100 benötigt)
            Ok(42)
        });

        assert!(result.is_err());
        // Gas wurde bis zum Fehler verbraucht (100 von 150)
        assert_eq!(ctx.gas_used(), 100);
        assert_eq!(ctx.gas_remaining, 50);
    }

    #[test]
    fn test_trust_gate() {
        let executor_id = UniversalId::new(UniversalId::TAG_DID, 1, b"executor");
        let trust = TrustVector6D::new(0.8, 0.8, 0.8, 0.8, 0.8, 0.8);
        let trust_context = TrustContext::direct(executor_id, trust);

        let ctx = ExecutionContext::new(WorldState::new(1), 1000, 100, trust_context);

        // Sollte bestehen
        assert!(ctx.require_trust(0.7).is_ok());

        // Sollte fehlschlagen
        assert!(ctx.require_trust(0.9).is_err());
    }

    #[test]
    fn test_delegation_trust_decay() {
        let delegator_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegator");
        let delegate_id = UniversalId::new(UniversalId::TAG_DID, 1, b"delegate");
        let executor_id = UniversalId::new(UniversalId::TAG_DID, 1, b"executor");

        let original_trust = TrustVector6D::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0);

        let chain = vec![
            DelegationHop {
                delegator: delegator_id,
                delegate: delegate_id,
                trust_factor: 0.8,
            },
            DelegationHop {
                delegator: delegate_id,
                delegate: executor_id,
                trust_factor: 0.9,
            },
        ];

        let trust_context = TrustContext::delegated(executor_id, original_trust, chain);

        // Effektiver Trust sollte 0.8 * 0.9 = 0.72 des Originals sein
        let expected_factor = 0.8 * 0.9;
        let effective = trust_context.effective_trust.r;
        assert!((effective - expected_factor).abs() < 0.001);
    }

    #[test]
    fn test_lamport_clock() {
        let mut ctx = ExecutionContext::minimal();
        assert_eq!(ctx.state.lamport, 0);

        ctx.tick();
        assert_eq!(ctx.state.lamport, 1);

        ctx.sync_lamport(10);
        assert_eq!(ctx.state.lamport, 11);

        ctx.sync_lamport(5); // Niedriger als aktuell
        assert_eq!(ctx.state.lamport, 12); // max(11, 5) + 1
    }

    #[test]
    fn test_metadata() {
        let mut ctx = ExecutionContext::minimal();

        ctx.set_metadata("request_id", "abc-123");
        ctx.set_metadata("user_agent", "test/1.0");

        assert_eq!(ctx.get_metadata("request_id"), Some("abc-123"));
        assert_eq!(ctx.get_metadata("user_agent"), Some("test/1.0"));
        assert_eq!(ctx.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_finalize() {
        let mut ctx = ExecutionContext::minimal();

        ctx.consume_gas(500).unwrap();
        ctx.consume_mana(100).unwrap();
        ctx.emit_raw("test", b"data");

        let summary = ctx.finalize();

        assert_eq!(summary.gas_used, 500);
        assert_eq!(summary.mana_used, 100);
        assert_eq!(summary.events_emitted, 1);
        assert_eq!(summary.final_lamport, 1);
    }

    #[test]
    fn test_world_state() {
        let mut state = WorldState::new(42);
        assert_eq!(state.epoch, 42);
        assert_eq!(state.lamport, 0);

        state.tick();
        assert_eq!(state.lamport, 1);

        state.sync(100);
        assert_eq!(state.lamport, 101);
    }
}
