//! # System State Integration
//!
//! Verbindet SystemState mit den tatsächlichen Erynoa-Engines.
//!
//! ## Verwendung
//!
//! ```rust,ignore
//! // Bei Initialisierung
//! let system_state = create_system_state();
//!
//! // In TrustEngine
//! let mut trust_engine = TrustEngine::default()
//!     .with_diagnostics(system_state.clone());
//!
//! // Oder manuell
//! trust_engine.process_event(&event)?;
//! system_state.trust_updated(event.is_negative_trust());
//! ```

use super::SystemState;
use std::sync::Arc;

// ============================================================================
// DIAGNOSTIC OBSERVER TRAIT
// ============================================================================

/// Trait für Engines die diagnostische Daten liefern
pub trait DiagnosticObserver: Send + Sync {
    /// Callback wenn ein Event verarbeitet wurde
    fn on_event_processed(&self, _event_type: &str) {}

    /// Callback wenn eine Operation abgeschlossen wurde
    fn on_operation_completed(&self, _success: bool, _gas: u64) {}
}

/// Null-Implementation für Engines ohne Diagnostics
pub struct NullObserver;

impl DiagnosticObserver for NullObserver {}

impl DiagnosticObserver for Arc<SystemState> {
    fn on_event_processed(&self, event_type: &str) {
        match event_type {
            "trust_update" => {
                self.trust_updates_total
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            "event_added" => {
                self.events_total
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

// ============================================================================
// TRUST ENGINE INTEGRATION
// ============================================================================

/// Extension-Methoden für TrustEngine-Integration
pub trait TrustEngineObserver {
    fn observe_trust_update(&self, positive: bool);
    fn observe_entity_registered(&self);
    fn observe_relationship_added(&self);
    fn observe_violation(&self);
}

impl TrustEngineObserver for Arc<SystemState> {
    fn observe_trust_update(&self, positive: bool) {
        self.trust_updated(positive);
    }

    fn observe_entity_registered(&self) {
        self.trust_entity_registered();
    }

    fn observe_relationship_added(&self) {
        self.trust_relationship_added();
    }

    fn observe_violation(&self) {
        self.trust_violation_detected();
    }
}

// ============================================================================
// EVENT ENGINE INTEGRATION
// ============================================================================

/// Extension-Methoden für EventEngine-Integration
pub trait EventEngineObserver {
    fn observe_event_added(&self, is_genesis: bool);
    fn observe_event_finalized(&self);
    fn observe_event_witnessed(&self);
    fn observe_validation_error(&self);
    fn observe_cycle_detected(&self);
}

impl EventEngineObserver for Arc<SystemState> {
    fn observe_event_added(&self, is_genesis: bool) {
        self.event_added(is_genesis);
    }

    fn observe_event_finalized(&self) {
        self.event_finalized();
    }

    fn observe_event_witnessed(&self) {
        self.event_witnessed();
    }

    fn observe_validation_error(&self) {
        self.event_validation_error();
    }

    fn observe_cycle_detected(&self) {
        self.event_cycle_detected();
    }
}

// ============================================================================
// ECLVM INTEGRATION
// ============================================================================

/// Extension-Methoden für ECLVM-Integration
pub trait EclvmObserver {
    fn observe_program_executed(&self, success: bool, gas_used: u64);
    fn observe_out_of_gas(&self);
    fn observe_vm_started(&self);
    fn observe_vm_stopped(&self);
}

impl EclvmObserver for Arc<SystemState> {
    fn observe_program_executed(&self, success: bool, gas_used: u64) {
        self.eclvm_program_executed(success, gas_used);
    }

    fn observe_out_of_gas(&self) {
        self.eclvm_out_of_gas();
    }

    fn observe_vm_started(&self) {
        self.eclvm_vm_started();
    }

    fn observe_vm_stopped(&self) {
        self.eclvm_vm_stopped();
    }
}

// ============================================================================
// MANA INTEGRATION
// ============================================================================

/// Extension-Methoden für ManaManager-Integration
pub trait ManaObserver {
    fn observe_account_created(&self);
    fn observe_mana_consumed(&self, amount: u64);
    fn observe_mana_regenerated(&self, amount: u64);
    fn observe_rate_limited(&self);
}

impl ManaObserver for Arc<SystemState> {
    fn observe_account_created(&self) {
        self.mana_account_created();
    }

    fn observe_mana_consumed(&self, amount: u64) {
        self.mana_consumed(amount);
    }

    fn observe_mana_regenerated(&self, amount: u64) {
        self.mana_regenerated(amount);
    }

    fn observe_rate_limited(&self) {
        self.mana_rate_limited();
    }
}

// ============================================================================
// PROTECTION INTEGRATION
// ============================================================================

/// Extension-Methoden für Protection-Module
pub trait ProtectionObserver {
    fn observe_anomaly(&self, severity: &str);
    fn observe_diversity_warning(&self);
    fn observe_quadratic_vote_started(&self);
    fn observe_quadratic_vote_completed(&self, participants: u64);
    fn observe_anticac_intervention(&self);
}

impl ProtectionObserver for Arc<SystemState> {
    fn observe_anomaly(&self, severity: &str) {
        self.anomaly_detected(severity);
    }

    fn observe_diversity_warning(&self) {
        self.diversity_warning();
    }

    fn observe_quadratic_vote_started(&self) {
        self.quadratic_vote_started();
    }

    fn observe_quadratic_vote_completed(&self, participants: u64) {
        self.quadratic_vote_completed(participants);
    }

    fn observe_anticac_intervention(&self) {
        self.anticac_intervention();
    }
}

// ============================================================================
// CONVENIENCE MACROS
// ============================================================================

/// Macro um SystemState-Aufrufe optional zu machen
#[macro_export]
macro_rules! observe_if_present {
    ($observer:expr, $method:ident $(, $arg:expr)*) => {
        if let Some(ref obs) = $observer {
            obs.$method($($arg),*);
        }
    };
}

// ============================================================================
// TESTING
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_observer() {
        let system_state = Arc::new(SystemState::new());

        system_state.observe_trust_update(true);
        system_state.observe_trust_update(false);
        system_state.observe_entity_registered();

        let snapshot = system_state.snapshot();
        assert_eq!(snapshot.trust.trust_updates_total, 2);
        assert_eq!(snapshot.trust.positive_updates, 1);
        assert_eq!(snapshot.trust.negative_updates, 1);
        assert_eq!(snapshot.trust.entities_count, 1);
    }

    #[test]
    fn test_event_observer() {
        let system_state = Arc::new(SystemState::new());

        system_state.observe_event_added(true);
        system_state.observe_event_added(false);
        system_state.observe_event_finalized();

        let snapshot = system_state.snapshot();
        assert_eq!(snapshot.events.events_total, 2);
        assert_eq!(snapshot.events.genesis_events, 1);
        assert_eq!(snapshot.events.finalized_events, 1);
    }

    #[test]
    fn test_eclvm_observer() {
        let system_state = Arc::new(SystemState::new());

        system_state.observe_program_executed(true, 1000);
        system_state.observe_program_executed(false, 500);
        system_state.observe_out_of_gas();

        let snapshot = system_state.snapshot();
        assert_eq!(snapshot.eclvm.programs_executed, 2);
        assert_eq!(snapshot.eclvm.successful_executions, 1);
        assert_eq!(snapshot.eclvm.failed_executions, 1);
        assert_eq!(snapshot.eclvm.total_gas_consumed, 1500);
        assert_eq!(snapshot.eclvm.out_of_gas_count, 1);
    }
}
// ============================================================================
// UNIFIED STATE BRIDGE
// ============================================================================

/// Brücke zwischen dem alten SwarmState (Diagnostics) und dem neuen UnifiedState (Core).
///
/// Diese Struktur synchronisiert Daten vom detaillierten Diagnostics-SwarmState
/// zum aggregierten UnifiedState für konsistentes System-weites Monitoring.
///
/// ## Verwendung
///
/// ```rust,ignore
/// use erynoa_api::peer::p2p::diagnostics::{SwarmState, UnifiedStateBridge};
/// use erynoa_api::core::{create_unified_state, StateIntegrator};
///
/// let swarm_state = Arc::new(SwarmState::new("peer-123"));
/// let unified_state = create_unified_state();
/// let integrator = StateIntegrator::new(unified_state.clone());
///
/// let bridge = UnifiedStateBridge::new(swarm_state.clone(), integrator);
///
/// // Events vom Swarm werden automatisch propagiert
/// swarm_state.peer_connected("other-peer", true, false);
/// bridge.sync(); // Synchronisiert zum UnifiedState
/// ```
pub struct UnifiedStateBridge {
    /// Detaillierter Diagnostics-SwarmState
    swarm_state: Arc<super::SwarmState>,

    /// StateIntegrator für Unified State Updates
    integrator: crate::core::StateIntegrator,
}

impl UnifiedStateBridge {
    /// Erstelle neue Bridge
    pub fn new(
        swarm_state: Arc<super::SwarmState>,
        integrator: crate::core::StateIntegrator,
    ) -> Self {
        Self {
            swarm_state,
            integrator,
        }
    }

    /// Synchronisiere aktuellen SwarmState zum UnifiedState
    ///
    /// Sollte periodisch aufgerufen werden (z.B. alle 100ms) oder nach wichtigen Events.
    pub fn sync(&self) {
        use std::sync::atomic::Ordering;

        // Peer count synchronisieren (usize -> usize)
        let connected = self
            .swarm_state
            .connected_peers_count
            .load(Ordering::Relaxed);
        let inbound = self.swarm_state.inbound_connections.load(Ordering::Relaxed) as u64;
        let outbound = self
            .swarm_state
            .outbound_connections
            .load(Ordering::Relaxed) as u64;

        // Swarm metrics zum UnifiedState
        self.integrator
            .state()
            .p2p
            .swarm
            .connected_peers
            .store(connected, Ordering::Relaxed);
        self.integrator
            .state()
            .p2p
            .swarm
            .inbound_connections
            .store(inbound, Ordering::Relaxed);
        self.integrator
            .state()
            .p2p
            .swarm
            .outbound_connections
            .store(outbound, Ordering::Relaxed);

        // Gossip metrics (usize/u64 -> usize/u64)
        let mesh_peers = self.swarm_state.gossip_mesh_size.load(Ordering::Relaxed);
        let topics = self
            .swarm_state
            .gossip_topics_subscribed
            .load(Ordering::Relaxed);
        let msgs_received = self
            .swarm_state
            .gossip_messages_received
            .load(Ordering::Relaxed);
        let msgs_sent = self
            .swarm_state
            .gossip_messages_sent
            .load(Ordering::Relaxed);

        self.integrator
            .state()
            .p2p
            .gossip
            .mesh_peers
            .store(mesh_peers, Ordering::Relaxed);
        self.integrator
            .state()
            .p2p
            .gossip
            .subscribed_topics
            .store(topics, Ordering::Relaxed);
        self.integrator
            .state()
            .p2p
            .gossip
            .messages_received
            .store(msgs_received, Ordering::Relaxed);
        self.integrator
            .state()
            .p2p
            .gossip
            .messages_sent
            .store(msgs_sent, Ordering::Relaxed);

        // Kademlia metrics (usize -> usize/u64)
        let routing_size = self
            .swarm_state
            .kademlia_routing_table_size
            .load(Ordering::Relaxed);
        let bootstrap = self
            .swarm_state
            .kademlia_bootstrap_complete
            .load(Ordering::Relaxed);
        let records = self.swarm_state.dht_records_stored.load(Ordering::Relaxed) as u64;

        self.integrator
            .state()
            .p2p
            .kademlia
            .routing_table_size
            .store(routing_size, Ordering::Relaxed);
        if let Ok(mut b) = self
            .integrator
            .state()
            .p2p
            .kademlia
            .bootstrap_complete
            .write()
        {
            *b = bootstrap;
        }
        self.integrator
            .state()
            .p2p
            .kademlia
            .records_stored
            .store(records, Ordering::Relaxed);

        // Relay metrics (usize -> u64, bool -> RwLock<bool>)
        let has_reservation = self
            .swarm_state
            .has_relay_reservation
            .load(Ordering::Relaxed);
        let circuits = self
            .swarm_state
            .relay_circuits_serving
            .load(Ordering::Relaxed) as u64;
        let dcutr_success = self.swarm_state.dcutr_successes.load(Ordering::Relaxed) as u64;
        let dcutr_fail = self.swarm_state.dcutr_failures.load(Ordering::Relaxed) as u64;

        if let Ok(mut res) = self.integrator.state().p2p.relay.has_reservation.write() {
            *res = has_reservation;
        }
        self.integrator
            .state()
            .p2p
            .relay
            .circuits_served
            .store(circuits, Ordering::Relaxed);
        self.integrator
            .state()
            .p2p
            .relay
            .dcutr_successes
            .store(dcutr_success, Ordering::Relaxed);
        self.integrator
            .state()
            .p2p
            .relay
            .dcutr_failures
            .store(dcutr_fail, Ordering::Relaxed);

        // P2P Warnings prüfen
        self.integrator.check_p2p_health();
    }

    /// Zugriff auf SwarmState
    pub fn swarm_state(&self) -> &Arc<super::SwarmState> {
        &self.swarm_state
    }

    /// Zugriff auf StateIntegrator
    pub fn integrator(&self) -> &crate::core::StateIntegrator {
        &self.integrator
    }
}
