//! # State Integration Layer
//!
//! Verbindet UnifiedState mit den echten Engines durch Observer-Pattern.
//! Ermöglicht automatische State-Updates bei Engine-Operationen.
//!
//! ## Architektur
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────────────────┐
//! │                        STATE INTEGRATION                               │
//! │                                                                        │
//! │  ┌─────────────────────────────────────────────────────────────────┐  │
//! │  │                     Observer Traits                              │  │
//! │  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │  │
//! │  │  │TrustObserver │  │EventObserver │  │ExecObserver  │   ...    │  │
//! │  │  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │  │
//! │  │         │                 │                 │                   │  │
//! │  │         └─────────────────┴─────────────────┘                   │  │
//! │  │                           │                                     │  │
//! │  │                    StateIntegrator                              │  │
//! │  │                           │                                     │  │
//! │  │                    ┌──────┴──────┐                              │  │
//! │  │                    │UnifiedState │                              │  │
//! │  │                    └─────────────┘                              │  │
//! │  └─────────────────────────────────────────────────────────────────┘  │
//! │                                                                        │
//! │  ┌─────────────────────────────────────────────────────────────────┐  │
//! │  │                     Event Propagation                           │  │
//! │  │                                                                  │  │
//! │  │  Engine → Observer → StateIntegrator → UnifiedState             │  │
//! │  │                           ↓                                      │  │
//! │  │                    Cross-Module Triggers                         │  │
//! │  │                           ↓                                      │  │
//! │  │                    Dependent State Updates                       │  │
//! │  └─────────────────────────────────────────────────────────────────┘  │
//! │                                                                        │
//! └────────────────────────────────────────────────────────────────────────┘
//! ```

use super::state::{SharedUnifiedState, StateGraph};
use crate::domain::{EventId, UniversalId};
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Type alias für Entity-IDs (DIDs)
pub type EntityId = UniversalId;

// ============================================================================
// OBSERVER TRAITS
// ============================================================================

/// Trust Engine Observer
pub trait TrustObserver: Send + Sync {
    /// Trust-Update verarbeitet
    fn on_trust_update(
        &self,
        from: &EntityId,
        to: &EntityId,
        old_trust: f64,
        new_trust: f64,
        positive: bool,
    );

    /// Entity registriert
    fn on_entity_registered(&self, entity: &EntityId);

    /// Trust-Beziehung erstellt
    fn on_relationship_created(&self, from: &EntityId, to: &EntityId);

    /// Violation erkannt
    fn on_violation_detected(&self, entity: &EntityId, violation_type: &str);
}

/// Event Engine Observer
pub trait EventObserver: Send + Sync {
    /// Event zum DAG hinzugefügt
    fn on_event_added(
        &self,
        event_id: &EventId,
        is_genesis: bool,
        parents_count: usize,
        depth: u64,
    );

    /// Event finalisiert
    fn on_event_finalized(&self, event_id: &EventId, latency_ms: u64);

    /// Event witnessed
    fn on_event_witnessed(&self, event_id: &EventId, witness: &EntityId);

    /// Cycle erkannt
    fn on_cycle_detected(&self, event_id: &EventId);

    /// Validierung fehlgeschlagen
    fn on_validation_error(&self, event_id: &EventId, error: &str);
}

/// Execution Observer
pub trait ExecutionObserver: Send + Sync {
    /// Execution gestartet
    fn on_execution_start(&self, context_id: u64);

    /// Execution abgeschlossen
    fn on_execution_complete(
        &self,
        context_id: u64,
        success: bool,
        gas_used: u64,
        mana_used: u64,
        events_emitted: u64,
        duration_ms: u64,
    );

    /// Gas verbraucht
    fn on_gas_consumed(&self, amount: u64);

    /// Out of Gas
    fn on_out_of_gas(&self, required: u64, available: u64);

    /// Mana verbraucht
    fn on_mana_consumed(&self, amount: u64);

    /// Rate Limited
    fn on_rate_limited(&self, entity: &EntityId);
}

/// Protection Observer
pub trait ProtectionObserver: Send + Sync {
    /// Anomalie erkannt
    fn on_anomaly_detected(&self, severity: &str, description: &str);

    /// Entropy-Update
    fn on_entropy_update(&self, dimension: &str, value: f64);

    /// Monokultur-Warnung
    fn on_monoculture_warning(&self, dimension: &str, concentration: f64);

    /// Anti-Calc Intervention
    fn on_intervention(&self, entity: &EntityId, reason: &str);

    /// Calibration Update
    fn on_calibration_update(&self, param: &str, old_value: f64, new_value: f64);
}

/// World Formula Observer
pub trait FormulaObserver: Send + Sync {
    /// 𝔼 berechnet
    fn on_formula_computed(&self, e: f64, activity: f64, trust_norm: f64, human_factor: f64);

    /// Contributor hinzugefügt
    fn on_contributor_added(&self, entity: &EntityId);

    /// Human Verification
    fn on_human_verified(&self, entity: &EntityId);
}

/// Consensus Observer
pub trait ConsensusObserver: Send + Sync {
    /// Epoch gewechselt
    fn on_epoch_change(&self, old_epoch: u64, new_epoch: u64);

    /// Runde abgeschlossen
    fn on_round_completed(&self, success: bool, duration_ms: u64);

    /// Validator hinzugefügt/entfernt
    fn on_validator_change(&self, added: bool, validator: &EntityId);

    /// Byzantine Verhalten
    fn on_byzantine_detected(&self, validator: &EntityId);

    /// Leader Change
    fn on_leader_change(&self, old_leader: Option<&EntityId>, new_leader: &EntityId);
}

/// Storage Observer
pub trait StorageObserver: Send + Sync {
    /// KV-Operation
    fn on_kv_operation(&self, is_write: bool, key_size: usize, value_size: usize);

    /// Event persistiert
    fn on_event_persisted(&self, event_id: &EventId, size_bytes: usize);

    /// Archiviert
    fn on_archived(&self, epoch: u64, event_count: u64, bytes: u64);

    /// Blueprint Operation
    fn on_blueprint_operation(&self, operation: &str, blueprint_id: &str);
}

// ============================================================================
// PEER LAYER OBSERVER TRAITS (Κ22-Κ24)
// ============================================================================

/// Gateway Observer (Κ23)
pub trait GatewayObserver: Send + Sync {
    /// Crossing erlaubt
    fn on_crossing_allowed(&self, entity: &EntityId, from_realm: &str, to_realm: &str, trust: f64);

    /// Crossing abgelehnt
    fn on_crossing_denied(&self, entity: &EntityId, from_realm: &str, to_realm: &str, reason: &str);

    /// Realm registriert
    fn on_realm_registered(&self, realm_id: &str);

    /// Trust-Dampening angewendet
    fn on_trust_dampened(&self, entity: &EntityId, original: f64, dampened: f64);
}

/// Saga Composer Observer (Κ22, Κ24)
pub trait SagaObserver: Send + Sync {
    /// Saga komponiert
    fn on_saga_composed(&self, saga_id: &str, steps: usize, goal_type: &str, success: bool);

    /// Kompensation ausgeführt
    fn on_compensation_executed(&self, saga_id: &str, step: usize, success: bool);

    /// Budget-Verletzung
    fn on_budget_violation(&self, saga_id: &str, required: u64, available: u64);

    /// Cross-Realm Saga
    fn on_cross_realm_saga(&self, saga_id: &str, realms: &[String]);
}

/// Intent Parser Observer
pub trait IntentObserver: Send + Sync {
    /// Intent geparst
    fn on_intent_parsed(&self, intent_type: &str, success: bool, duration_us: u64);

    /// Validierungsfehler
    fn on_validation_error(&self, intent_id: &str, error: &str);
}

/// Realm Observer (Κ22-Κ24) - Per-Realm Isolation Events
pub trait RealmObserver: Send + Sync {
    /// Realm registriert (mit min_trust und governance_type)
    fn on_realm_registered(&self, realm_id: &str, min_trust: f32, governance_type: &str);

    /// Root-Realm gesetzt
    fn on_root_realm_set(&self, realm_id: &str);

    /// Crossing erfolgreich (von Realm A nach B)
    fn on_crossing_succeeded(&self, from_realm: &str, to_realm: &str);

    /// Crossing fehlgeschlagen
    fn on_crossing_failed(&self, from_realm: &str, to_realm: &str, reason: &str);

    /// Crossing beendet
    fn on_crossing_completed(&self, from_realm: &str, to_realm: &str);

    /// Cross-Realm-Saga gestartet
    fn on_cross_realm_saga_started(&self, saga_id: &str, realm_ids: &[&str]);

    /// Identity tritt einem Realm bei
    fn on_identity_joined_realm(&self, identity_id: &str, realm_id: &str);

    /// Identity verlässt ein Realm
    fn on_identity_left_realm(&self, identity_id: &str, realm_id: &str);

    /// Realm-Trust aktualisiert
    fn on_realm_trust_updated(&self, realm_id: &str, new_trust: f64);

    /// Rule zu Realm hinzugefügt
    fn on_rule_added_to_realm(&self, realm_id: &str, rule_id: &str);

    /// Rule von Realm entfernt
    fn on_rule_removed_from_realm(&self, realm_id: &str, rule_id: &str);
}

// ============================================================================
// ECLVM OBSERVER TRAIT (ECL/ECLVM Integration)
// ============================================================================

/// ECLVM Observer - Tracks ECL (Erynoa Core Language) policy & blueprint execution
///
/// # ECL-Architektur Integration
///
/// ECL ist die DSL für:
/// - **Policies**: Regeln für Crossing, Membership, Transaction, Governance, Privacy
/// - **Blueprints**: Wiederverwendbare Konfigurationsvorlagen
/// - **Intents**: Hochrangige Benutzerabsichten
/// - **Sagas**: Mehrstufige, kompensierbare Operationen
///
/// # Resource-Modell
///
/// Die ECLVM ist cost-limited mit zwei Ressourcen:
/// - **Gas** (Compute): Für CPU-intensive Operationen
/// - **Mana** (Bandwidth/Events): Für I/O und Event-Emission
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// ECLVM ──DependsOn──▶ Gas
/// ECLVM ──DependsOn──▶ Mana
/// ECLVM ──Triggers───▶ Event
/// ECLVM ──Aggregates─▶ Execution
/// ECLVM ──DependsOn──▶ Trust
///
/// ECLPolicy ──Validates──▶ Gateway
/// ECLPolicy ──Validates──▶ Realm
/// ECLPolicy ──DependsOn──▶ ECLVM
///
/// ECLBlueprint ──DependsOn──▶ ECLVM
/// SagaComposer ──DependsOn──▶ ECLVM
/// Gateway ──DependsOn──▶ ECLPolicy
/// ```
pub trait ECLVMObserver: Send + Sync {
    // ─────────────────────────────────────────────────────────────────────────
    // POLICY ENGINE EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Policy erfolgreich kompiliert
    fn on_policy_compiled(
        &self,
        policy_id: &str,
        policy_type: &str, // "crossing", "membership", "transaction", "governance", "privacy", "custom"
        bytecode_size: usize,
    );

    /// Policy-Kompilierung fehlgeschlagen
    fn on_policy_compilation_failed(&self, policy_id: &str, error: &str);

    /// Policy ausgeführt (Κ23 Integration - Gateway-Validierung)
    fn on_policy_executed(
        &self,
        policy_id: &str,
        policy_type: &str,
        passed: bool,
        gas_used: u64,
        mana_used: u64,
        realm_id: Option<&str>,
    );

    /// Policy gecached (für schnellen Zugriff)
    fn on_policy_cached(&self, policy_id: &str, cache_hit: bool);

    // ─────────────────────────────────────────────────────────────────────────
    // BLUEPRINT ENGINE EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Blueprint im Marketplace publiziert
    fn on_blueprint_published(&self, blueprint_id: &str, version: &str, author: &EntityId);

    /// Blueprint deployed (auf ein Realm angewandt)
    fn on_blueprint_deployed(&self, blueprint_id: &str, realm_id: &str);

    /// Blueprint instanziiert (konkrete Instanz erstellt)
    fn on_blueprint_instantiated(
        &self,
        blueprint_id: &str,
        instance_id: &str,
        gas_used: u64,
        mana_used: u64,
    );

    /// Blueprint-Status geändert
    fn on_blueprint_status_changed(
        &self,
        blueprint_id: &str,
        old_status: &str, // "draft", "published", "verified", "deprecated"
        new_status: &str,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // INTENT/SAGA ORCHESTRATION EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Intent geparst und validiert
    fn on_intent_processed(
        &self,
        intent_id: &str,
        intent_type: &str,
        success: bool,
        validation_errors: usize,
    );

    /// Saga-Schritt ausgeführt (Κ22/Κ24 Integration)
    fn on_saga_step_executed(
        &self,
        saga_id: &str,
        step_index: usize,
        total_steps: usize,
        success: bool,
        gas_used: u64,
        mana_used: u64,
        is_cross_realm: bool,
    );

    /// Saga-Kompensation ausgeführt (Rollback)
    fn on_saga_compensation(&self, saga_id: &str, step_index: usize, reason: &str);

    /// Cross-Realm Saga koordiniert
    fn on_cross_realm_saga(
        &self,
        saga_id: &str,
        source_realm: &str,
        target_realms: &[String],
        total_gas_budget: u64,
        total_mana_budget: u64,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // RESOURCE TRACKING EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Gas verbraucht durch ECL-Ausführung
    fn on_ecl_gas_consumed(
        &self,
        policy_or_blueprint_id: &str,
        amount: u64,
        realm_id: Option<&str>,
    );

    /// Mana verbraucht durch ECL-Ausführung
    fn on_ecl_mana_consumed(
        &self,
        policy_or_blueprint_id: &str,
        amount: u64,
        realm_id: Option<&str>,
    );

    /// Out-of-Gas während ECL-Ausführung
    fn on_ecl_out_of_gas(&self, context: &str, required: u64, available: u64);

    /// Out-of-Mana während ECL-Ausführung
    fn on_ecl_out_of_mana(&self, context: &str, required: u64, available: u64);

    // ─────────────────────────────────────────────────────────────────────────
    // CROSSING-POLICY EVENTS (Κ23 Gateway-Integration)
    // ─────────────────────────────────────────────────────────────────────────

    /// Crossing-Policy evaluiert
    fn on_crossing_policy_evaluated(
        &self,
        from_realm: &str,
        to_realm: &str,
        entity: &EntityId,
        allowed: bool,
        trust_score: f64,
        policy_id: Option<&str>,
    );

    /// Membership-Check durchgeführt
    fn on_membership_check(
        &self,
        realm_id: &str,
        entity: &EntityId,
        is_member: bool,
        required_trust: f64,
        actual_trust: f64,
    );

    /// Governance-Regel angewendet
    fn on_governance_rule_applied(
        &self,
        realm_id: &str,
        rule_id: &str,
        action: &str, // "vote", "propose", "veto", "delegate"
        success: bool,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // REALM-SPECIFIC ECL EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Realm-spezifische ECL-Policy aktiviert
    fn on_realm_ecl_policy_activated(&self, realm_id: &str, policy_id: &str, policy_type: &str);

    /// Realm-spezifische ECL-Policy deaktiviert
    fn on_realm_ecl_policy_deactivated(&self, realm_id: &str, policy_id: &str);

    /// Realm ECL-State aggregiert (periodisch)
    fn on_realm_ecl_metrics(
        &self,
        realm_id: &str,
        policies_executed: u64,
        gas_consumed: u64,
        mana_consumed: u64,
        active_policies: usize,
    );
}

/// Shared ECLVM Observer (thread-safe Arc)
pub type SharedECLVMObserver = Arc<dyn ECLVMObserver>;

// ============================================================================
// P2P NETWORK OBSERVER TRAITS
// ============================================================================

/// Swarm Observer
pub trait SwarmObserver: Send + Sync {
    /// Peer verbunden
    fn on_peer_connected(&self, peer_id: &str, inbound: bool, is_relayed: bool);

    /// Peer getrennt
    fn on_peer_disconnected(&self, peer_id: &str);

    /// Verbindungsfehler
    fn on_connection_error(&self, error: &str);

    /// Bytes übertragen
    fn on_bytes_transferred(&self, sent: u64, received: u64);

    /// Latenz gemessen
    fn on_latency_measured(&self, peer_id: &str, latency_us: u64);

    /// NAT-Status geändert
    fn on_nat_status_changed(&self, status: &str);

    /// Externe Adresse erkannt
    fn on_external_address(&self, address: &str);
}

/// Gossipsub Observer
pub trait GossipObserver: Send + Sync {
    /// Message empfangen
    fn on_message_received(&self, topic: &str, from_peer: &str);

    /// Message gesendet
    fn on_message_sent(&self, topic: &str);

    /// Message validiert
    fn on_message_validated(&self, accepted: bool);

    /// Peer zum Mesh hinzugefügt (Graft)
    fn on_peer_grafted(&self, peer_id: &str, topic: &str);

    /// Peer aus Mesh entfernt (Prune)
    fn on_peer_pruned(&self, peer_id: &str, topic: &str, reason: &str);

    /// Topic subscribed
    fn on_topic_subscribed(&self, topic: &str);

    /// Duplicate message
    fn on_duplicate_message(&self, topic: &str);
}

/// Kademlia DHT Observer
pub trait KademliaObserver: Send + Sync {
    /// Bootstrap abgeschlossen
    fn on_bootstrap_complete(&self, routing_table_size: usize);

    /// Routing Table Update
    fn on_routing_table_update(&self, size: usize);

    /// Record gespeichert
    fn on_record_stored(&self, key: &str);

    /// Query durchgeführt
    fn on_query(&self, query_type: &str, success: bool);

    /// Provider registriert
    fn on_provider_registered(&self, key: &str);
}

/// Relay Observer
pub trait RelayObserver: Send + Sync {
    /// Relay-Reservation erhalten
    fn on_reservation_accepted(&self, relay_peer: &str);

    /// Circuit geöffnet
    fn on_circuit_opened(&self);

    /// Circuit geschlossen
    fn on_circuit_closed(&self);

    /// DCUTR (Hole-Punching)
    fn on_dcutr_attempt(&self, success: bool);

    /// Bytes über Relay
    fn on_relay_bytes(&self, bytes: u64);
}

/// Privacy Layer Observer
pub trait PrivacyObserver: Send + Sync {
    /// Circuit erstellt
    fn on_circuit_created(&self, hops: usize);

    /// Circuit geschlossen
    fn on_circuit_closed(&self);

    /// Private message gesendet
    fn on_private_message(&self);

    /// Cover-Traffic gesendet
    fn on_cover_traffic(&self);

    /// Relay rotiert
    fn on_relay_rotation(&self);

    /// Trust-basierte Auswahl
    fn on_trust_based_selection(&self, selected_peer: &str, trust: f64);
}

// ============================================================================
// STATE INTEGRATOR
// ============================================================================

/// State Integrator - Verbindet Observer mit UnifiedState
///
/// Implementiert alle Observer-Traits und propagiert Updates zum State.
#[derive(Clone)]
pub struct StateIntegrator {
    state: SharedUnifiedState,
}

impl StateIntegrator {
    /// Erstelle neuen StateIntegrator
    pub fn new(state: SharedUnifiedState) -> Self {
        Self { state }
    }

    /// Zugriff auf State
    pub fn state(&self) -> &SharedUnifiedState {
        &self.state
    }

    /// Öffentliche Methode zum Prüfen der P2P-Gesundheit
    ///
    /// Kann von externen Brücken aufgerufen werden, um Warnings zu synchronisieren.
    pub fn check_p2p_health(&self) {
        self.check_p2p_warnings();
    }

    /// Propagiere State-Updates basierend auf Beziehungen
    fn propagate_update(&self, from: super::state::StateComponent) {
        let graph = StateGraph::erynoa_graph();

        // Finde abhängige Komponenten
        for component in graph.triggered_by(from) {
            match component {
                super::state::StateComponent::Trust => {
                    // Trust durch Event getriggert
                    self.state
                        .core
                        .trust
                        .event_triggered_updates
                        .fetch_add(1, Ordering::Relaxed);
                }
                super::state::StateComponent::Event => {
                    // Event durch Trust getriggert
                    self.state
                        .core
                        .events
                        .trust_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }
                _ => {}
            }
        }
    }

    /// Check und setze Warnings basierend auf State
    fn check_warnings(&self) {
        // Trust Asymmetry Check
        let asymmetry = self.state.core.trust.asymmetry_ratio();
        if asymmetry < 1.5 || asymmetry > 3.0 {
            self.state.add_warning(format!(
                "Trust asymmetry ratio {} outside expected range [1.5, 3.0]",
                asymmetry
            ));
        } else {
            self.state.clear_warning("Trust asymmetry ratio");
        }

        // Consensus Success Rate Check
        let success_rate = self.state.core.consensus.success_rate();
        if success_rate < 0.9 {
            self.state.add_warning(format!(
                "Consensus success rate {} below threshold 0.9",
                success_rate
            ));
        } else {
            self.state.clear_warning("Consensus success rate");
        }

        // Event Validation Error Check
        let errors = self
            .state
            .core
            .events
            .validation_errors
            .load(Ordering::Relaxed);
        let total = self.state.core.events.total.load(Ordering::Relaxed);
        if total > 100 && errors as f64 / total as f64 > 0.01 {
            self.state
                .add_warning("Event validation error rate > 1%".into());
        } else {
            self.state.clear_warning("Event validation error rate");
        }
    }

    /// Check und setze P2P-spezifische Warnings
    fn check_p2p_warnings(&self) {
        // Mindestens 3 Peers für stabiles Netzwerk
        let peers = self.state.p2p.swarm.connected_peers.load(Ordering::Relaxed);
        if peers < 3 {
            self.state
                .add_warning(format!("Low peer count: {} (minimum 3 recommended)", peers));
        } else {
            self.state.clear_warning("Low peer count");
        }

        // Kademlia Bootstrap Check
        let bootstrap = self
            .state
            .p2p
            .kademlia
            .bootstrap_complete
            .read()
            .map(|b| *b)
            .unwrap_or(false);
        if !bootstrap {
            self.state
                .add_warning("Kademlia bootstrap not complete".into());
        } else {
            self.state.clear_warning("Kademlia bootstrap");
        }

        // Gossip Mesh Health
        let mesh = self.state.p2p.gossip.mesh_peers.load(Ordering::Relaxed);
        if peers > 0 && mesh == 0 {
            self.state.add_warning("No peers in gossipsub mesh".into());
        } else {
            self.state.clear_warning("No peers in gossipsub mesh");
        }
    }
}

// ============================================================================
// TRUST OBSERVER IMPLEMENTATION
// ============================================================================

impl TrustObserver for StateIntegrator {
    fn on_trust_update(
        &self,
        _from: &EntityId,
        _to: &EntityId,
        _old_trust: f64,
        _new_trust: f64,
        positive: bool,
    ) {
        self.state.core.trust.update(positive, false);
        self.propagate_update(super::state::StateComponent::Trust);
    }

    fn on_entity_registered(&self, _entity: &EntityId) {
        self.state
            .core
            .trust
            .entities
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_relationship_created(&self, _from: &EntityId, _to: &EntityId) {
        self.state
            .core
            .trust
            .relationships
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_violation_detected(&self, _entity: &EntityId, _violation_type: &str) {
        self.state
            .core
            .trust
            .violations
            .fetch_add(1, Ordering::Relaxed);
        self.check_warnings();
    }
}

// ============================================================================
// EVENT OBSERVER IMPLEMENTATION
// ============================================================================

impl EventObserver for StateIntegrator {
    fn on_event_added(
        &self,
        _event_id: &EventId,
        is_genesis: bool,
        parents_count: usize,
        depth: u64,
    ) {
        self.state.core.events.add(is_genesis, parents_count, depth);
    }

    fn on_event_finalized(&self, _event_id: &EventId, latency_ms: u64) {
        self.state.core.events.finalize(latency_ms);
    }

    fn on_event_witnessed(&self, _event_id: &EventId, _witness: &EntityId) {
        self.state
            .core
            .events
            .witnessed
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_cycle_detected(&self, _event_id: &EventId) {
        self.state
            .core
            .events
            .cycles_detected
            .fetch_add(1, Ordering::Relaxed);
        self.state.add_warning("Cycle detected in event DAG".into());
    }

    fn on_validation_error(&self, _event_id: &EventId, error: &str) {
        self.state
            .core
            .events
            .validation_errors
            .fetch_add(1, Ordering::Relaxed);
        tracing::warn!("Event validation error: {}", error);
        self.check_warnings();
    }
}

// ============================================================================
// EXECUTION OBSERVER IMPLEMENTATION
// ============================================================================

impl ExecutionObserver for StateIntegrator {
    fn on_execution_start(&self, _context_id: u64) {
        self.state.execution.start();
    }

    fn on_execution_complete(
        &self,
        _context_id: u64,
        success: bool,
        gas_used: u64,
        mana_used: u64,
        events_emitted: u64,
        duration_ms: u64,
    ) {
        self.state
            .execution
            .complete(success, gas_used, mana_used, events_emitted, duration_ms);
    }

    fn on_gas_consumed(&self, amount: u64) {
        self.state
            .execution
            .gas_consumed
            .fetch_add(amount, Ordering::Relaxed);
    }

    fn on_out_of_gas(&self, _required: u64, _available: u64) {
        self.state
            .execution
            .out_of_gas
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_mana_consumed(&self, amount: u64) {
        self.state
            .execution
            .mana_consumed
            .fetch_add(amount, Ordering::Relaxed);
    }

    fn on_rate_limited(&self, _entity: &EntityId) {
        self.state
            .execution
            .rate_limited
            .fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// PROTECTION OBSERVER IMPLEMENTATION
// ============================================================================

impl ProtectionObserver for StateIntegrator {
    fn on_anomaly_detected(&self, severity: &str, description: &str) {
        self.state.protection.anomaly(severity);
        if severity == "critical" || severity == "high" {
            self.state
                .add_warning(format!("Anomaly detected: {}", description));
        }
    }

    fn on_entropy_update(&self, dimension: &str, value: f64) {
        self.state.protection.set_entropy(dimension, value);
    }

    fn on_monoculture_warning(&self, dimension: &str, concentration: f64) {
        self.state
            .protection
            .monoculture_warnings
            .fetch_add(1, Ordering::Relaxed);
        self.state.add_warning(format!(
            "Monoculture warning: {} concentration {:.2}%",
            dimension,
            concentration * 100.0
        ));
    }

    fn on_intervention(&self, _entity: &EntityId, reason: &str) {
        self.state
            .protection
            .interventions
            .fetch_add(1, Ordering::Relaxed);
        tracing::info!("Anti-calcification intervention: {}", reason);
    }

    fn on_calibration_update(&self, param: &str, _old_value: f64, new_value: f64) {
        self.state
            .protection
            .calibration_updates
            .fetch_add(1, Ordering::Relaxed);
        if let Ok(mut params) = self.state.protection.calibrated_params.write() {
            params.insert(param.to_string(), new_value);
        }
    }
}

// ============================================================================
// FORMULA OBSERVER IMPLEMENTATION
// ============================================================================

impl FormulaObserver for StateIntegrator {
    fn on_formula_computed(&self, e: f64, activity: f64, trust_norm: f64, human_factor: f64) {
        self.state
            .core
            .formula
            .update(e, activity, trust_norm, human_factor);
    }

    fn on_contributor_added(&self, _entity: &EntityId) {
        self.state
            .core
            .formula
            .contributors
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_human_verified(&self, _entity: &EntityId) {
        self.state
            .core
            .formula
            .human_verified
            .fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// CONSENSUS OBSERVER IMPLEMENTATION
// ============================================================================

impl ConsensusObserver for StateIntegrator {
    fn on_epoch_change(&self, _old_epoch: u64, new_epoch: u64) {
        self.state
            .core
            .consensus
            .epoch
            .store(new_epoch, Ordering::Relaxed);
    }

    fn on_round_completed(&self, success: bool, duration_ms: u64) {
        self.state
            .core
            .consensus
            .round_completed(success, duration_ms);
        if !success {
            self.check_warnings();
        }
    }

    fn on_validator_change(&self, added: bool, _validator: &EntityId) {
        if added {
            self.state
                .core
                .consensus
                .validators
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.state
                .core
                .consensus
                .validators
                .fetch_sub(1, Ordering::Relaxed);
        }
    }

    fn on_byzantine_detected(&self, _validator: &EntityId) {
        self.state
            .core
            .consensus
            .byzantine_detected
            .fetch_add(1, Ordering::Relaxed);
        self.state.add_warning("Byzantine behavior detected".into());
    }

    fn on_leader_change(&self, _old_leader: Option<&EntityId>, _new_leader: &EntityId) {
        self.state
            .core
            .consensus
            .leader_changes
            .fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// STORAGE OBSERVER IMPLEMENTATION
// ============================================================================

impl StorageObserver for StateIntegrator {
    fn on_kv_operation(&self, is_write: bool, _key_size: usize, value_size: usize) {
        if is_write {
            self.state.storage.kv_writes.fetch_add(1, Ordering::Relaxed);
            self.state
                .storage
                .kv_bytes
                .fetch_add(value_size as u64, Ordering::Relaxed);
        } else {
            self.state.storage.kv_reads.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn on_event_persisted(&self, _event_id: &EventId, size_bytes: usize) {
        self.state
            .storage
            .event_store_count
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .storage
            .event_store_bytes
            .fetch_add(size_bytes as u64, Ordering::Relaxed);
    }

    fn on_archived(&self, _epoch: u64, event_count: u64, bytes: u64) {
        self.state
            .storage
            .archived_epochs
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .storage
            .archived_events
            .fetch_add(event_count, Ordering::Relaxed);
        self.state
            .storage
            .archive_bytes
            .fetch_add(bytes, Ordering::Relaxed);
    }

    fn on_blueprint_operation(&self, operation: &str, _blueprint_id: &str) {
        match operation {
            "publish" => self
                .state
                .storage
                .blueprints_published
                .fetch_add(1, Ordering::Relaxed),
            "deploy" => self
                .state
                .storage
                .blueprints_deployed
                .fetch_add(1, Ordering::Relaxed),
            "download" => self
                .state
                .storage
                .blueprints_downloaded
                .fetch_add(1, Ordering::Relaxed),
            _ => 0,
        };
    }
}

// ============================================================================
// GATEWAY OBSERVER IMPLEMENTATION (Κ23)
// ============================================================================

impl GatewayObserver for StateIntegrator {
    fn on_crossing_allowed(
        &self,
        _entity: &EntityId,
        _from_realm: &str,
        _to_realm: &str,
        trust: f64,
    ) {
        self.state.peer.gateway.crossing_allowed(trust);
        self.propagate_update(super::state::StateComponent::Gateway);
    }

    fn on_crossing_denied(
        &self,
        _entity: &EntityId,
        _from_realm: &str,
        to_realm: &str,
        reason: &str,
    ) {
        self.state.peer.gateway.crossing_denied(reason);
        tracing::debug!("Gateway crossing denied to realm {}: {}", to_realm, reason);
    }

    fn on_realm_registered(&self, _realm_id: &str) {
        self.state
            .peer
            .gateway
            .registered_realms
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_trust_dampened(&self, _entity: &EntityId, _original: f64, _dampened: f64) {
        self.state
            .peer
            .gateway
            .dampening_applied
            .fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// SAGA OBSERVER IMPLEMENTATION (Κ22, Κ24)
// ============================================================================

impl SagaObserver for StateIntegrator {
    fn on_saga_composed(&self, _saga_id: &str, steps: usize, goal_type: &str, success: bool) {
        self.state
            .peer
            .saga
            .saga_composed(success, steps, goal_type);
    }

    fn on_compensation_executed(&self, _saga_id: &str, _step: usize, success: bool) {
        self.state.peer.saga.compensation(success);
    }

    fn on_budget_violation(&self, _saga_id: &str, _required: u64, _available: u64) {
        self.state
            .peer
            .saga
            .budget_violations
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_cross_realm_saga(&self, _saga_id: &str, _realms: &[String]) {
        self.state
            .peer
            .saga
            .cross_realm_sagas
            .fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// INTENT OBSERVER IMPLEMENTATION
// ============================================================================

impl IntentObserver for StateIntegrator {
    fn on_intent_parsed(&self, intent_type: &str, success: bool, duration_us: u64) {
        self.state
            .peer
            .intent
            .parsed(success, intent_type, duration_us);
    }

    fn on_validation_error(&self, _intent_id: &str, _error: &str) {
        self.state
            .peer
            .intent
            .validation_errors
            .fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// REALM OBSERVER IMPLEMENTATION (Κ22-Κ24)
// ============================================================================

impl RealmObserver for StateIntegrator {
    fn on_realm_registered(&self, realm_id: &str, min_trust: f32, governance_type: &str) {
        self.state
            .peer
            .realm
            .register_realm(realm_id, min_trust, governance_type);
        self.propagate_update(super::state::StateComponent::Realm);
        tracing::info!(
            "Realm registered: {} (min_trust: {}, governance: {})",
            realm_id,
            min_trust,
            governance_type
        );
    }

    fn on_root_realm_set(&self, realm_id: &str) {
        self.state.peer.realm.set_root_realm(realm_id);
        tracing::info!("Root realm set: {}", realm_id);
    }

    fn on_crossing_succeeded(&self, from_realm: &str, to_realm: &str) {
        self.state
            .peer
            .realm
            .crossing_succeeded(from_realm, to_realm);
        tracing::debug!("Realm crossing succeeded: {} -> {}", from_realm, to_realm);
    }

    fn on_crossing_failed(&self, from_realm: &str, to_realm: &str, reason: &str) {
        self.state.peer.realm.crossing_failed();
        tracing::warn!(
            "Realm crossing failed: {} -> {} (reason: {})",
            from_realm,
            to_realm,
            reason
        );
    }

    fn on_crossing_completed(&self, _from_realm: &str, _to_realm: &str) {
        self.state.peer.realm.crossing_completed();
    }

    fn on_cross_realm_saga_started(&self, saga_id: &str, realm_ids: &[&str]) {
        self.state.peer.realm.cross_realm_saga_started(realm_ids);
        tracing::info!(
            "Cross-realm saga started: {} (realms: {:?})",
            saga_id,
            realm_ids
        );
    }

    fn on_identity_joined_realm(&self, identity_id: &str, realm_id: &str) {
        self.state.peer.realm.identity_joined_realm(realm_id);
        tracing::debug!("Identity {} joined realm {}", identity_id, realm_id);
    }

    fn on_identity_left_realm(&self, identity_id: &str, realm_id: &str) {
        self.state.peer.realm.identity_left_realm(realm_id);
        tracing::debug!("Identity {} left realm {}", identity_id, realm_id);
    }

    fn on_realm_trust_updated(&self, realm_id: &str, new_trust: f64) {
        // Erstelle TrustVector6D aus dem aggregierten Trust-Wert
        let trust = crate::domain::unified::TrustVector6D::new(
            new_trust as f32,
            new_trust as f32,
            new_trust as f32,
            new_trust as f32,
            new_trust as f32,
            new_trust as f32,
        );
        self.state.peer.realm.update_realm_trust(realm_id, trust);
    }

    fn on_rule_added_to_realm(&self, realm_id: &str, rule_id: &str) {
        self.state.peer.realm.add_rule_to_realm(realm_id, rule_id);
        tracing::debug!("Rule {} added to realm {}", rule_id, realm_id);
    }

    fn on_rule_removed_from_realm(&self, realm_id: &str, rule_id: &str) {
        // Realm-spezifisches Rule-Removal via direct access
        if let Ok(realms) = self.state.peer.realm.realms.read() {
            if let Some(realm) = realms.get(realm_id) {
                realm.remove_rule(rule_id);
            }
        }
        tracing::debug!("Rule {} removed from realm {}", rule_id, realm_id);
    }
}

// ============================================================================
// SWARM OBSERVER IMPLEMENTATION
// ============================================================================

impl SwarmObserver for StateIntegrator {
    fn on_peer_connected(&self, _peer_id: &str, inbound: bool, _is_relayed: bool) {
        self.state.p2p.swarm.peer_connected(inbound);
        self.check_p2p_warnings();
    }

    fn on_peer_disconnected(&self, _peer_id: &str) {
        self.state.p2p.swarm.peer_disconnected();
        self.check_p2p_warnings();
    }

    fn on_connection_error(&self, error: &str) {
        self.state
            .p2p
            .swarm
            .connection_errors
            .fetch_add(1, Ordering::Relaxed);
        tracing::warn!("P2P connection error: {}", error);
    }

    fn on_bytes_transferred(&self, sent: u64, received: u64) {
        self.state
            .p2p
            .swarm
            .bytes_sent
            .fetch_add(sent, Ordering::Relaxed);
        self.state
            .p2p
            .swarm
            .bytes_received
            .fetch_add(received, Ordering::Relaxed);
    }

    fn on_latency_measured(&self, _peer_id: &str, latency_us: u64) {
        self.state.p2p.swarm.record_latency(latency_us);
    }

    fn on_nat_status_changed(&self, status: &str) {
        let nat_status = match status {
            "public" | "Public" => super::state::NatStatus::Public,
            "private" | "Private" => super::state::NatStatus::Private,
            _ => super::state::NatStatus::Unknown,
        };
        if let Ok(mut s) = self.state.p2p.swarm.nat_status.write() {
            *s = nat_status;
        }
    }

    fn on_external_address(&self, address: &str) {
        if let Ok(mut addrs) = self.state.p2p.swarm.external_addresses.write() {
            if !addrs.contains(&address.to_string()) {
                addrs.push(address.to_string());
            }
        }
    }
}

// ============================================================================
// GOSSIP OBSERVER IMPLEMENTATION
// ============================================================================

impl GossipObserver for StateIntegrator {
    fn on_message_received(&self, _topic: &str, _from_peer: &str) {
        self.state.p2p.gossip.message_received();
    }

    fn on_message_sent(&self, _topic: &str) {
        self.state.p2p.gossip.message_sent();
    }

    fn on_message_validated(&self, accepted: bool) {
        if accepted {
            self.state
                .p2p
                .gossip
                .messages_validated
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.state
                .p2p
                .gossip
                .messages_rejected
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    fn on_peer_grafted(&self, _peer_id: &str, _topic: &str) {
        self.state
            .p2p
            .gossip
            .peers_grafted
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .p2p
            .gossip
            .mesh_peers
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_peer_pruned(&self, _peer_id: &str, _topic: &str, _reason: &str) {
        self.state
            .p2p
            .gossip
            .peers_pruned
            .fetch_add(1, Ordering::Relaxed);
        let _ = self.state.p2p.gossip.mesh_peers.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            },
        );
    }

    fn on_topic_subscribed(&self, _topic: &str) {
        self.state
            .p2p
            .gossip
            .subscribed_topics
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_duplicate_message(&self, _topic: &str) {
        self.state
            .p2p
            .gossip
            .duplicate_messages
            .fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// KADEMLIA OBSERVER IMPLEMENTATION
// ============================================================================

impl KademliaObserver for StateIntegrator {
    fn on_bootstrap_complete(&self, routing_table_size: usize) {
        if let Ok(mut b) = self.state.p2p.kademlia.bootstrap_complete.write() {
            *b = true;
        }
        self.state
            .p2p
            .kademlia
            .routing_table_size
            .store(routing_table_size, Ordering::Relaxed);
        self.check_p2p_warnings();
    }

    fn on_routing_table_update(&self, size: usize) {
        self.state
            .p2p
            .kademlia
            .routing_table_size
            .store(size, Ordering::Relaxed);
    }

    fn on_record_stored(&self, _key: &str) {
        self.state
            .p2p
            .kademlia
            .records_stored
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_query(&self, _query_type: &str, success: bool) {
        self.state
            .p2p
            .kademlia
            .queries_total
            .fetch_add(1, Ordering::Relaxed);
        if success {
            self.state
                .p2p
                .kademlia
                .queries_successful
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    fn on_provider_registered(&self, _key: &str) {
        self.state
            .p2p
            .kademlia
            .provider_registrations
            .fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// RELAY OBSERVER IMPLEMENTATION
// ============================================================================

impl RelayObserver for StateIntegrator {
    fn on_reservation_accepted(&self, relay_peer: &str) {
        if let Ok(mut has) = self.state.p2p.relay.has_reservation.write() {
            *has = true;
        }
        if let Ok(mut peer) = self.state.p2p.relay.relay_peer.write() {
            *peer = Some(relay_peer.to_string());
        }
    }

    fn on_circuit_opened(&self) {
        self.state
            .p2p
            .relay
            .circuits_served
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .p2p
            .relay
            .circuits_active
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_circuit_closed(&self) {
        let _ = self.state.p2p.relay.circuits_active.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            },
        );
    }

    fn on_dcutr_attempt(&self, success: bool) {
        if success {
            self.state
                .p2p
                .relay
                .dcutr_successes
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.state
                .p2p
                .relay
                .dcutr_failures
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    fn on_relay_bytes(&self, bytes: u64) {
        self.state
            .p2p
            .relay
            .relay_bytes
            .fetch_add(bytes, Ordering::Relaxed);
    }
}

// ============================================================================
// PRIVACY OBSERVER IMPLEMENTATION
// ============================================================================

impl PrivacyObserver for StateIntegrator {
    fn on_circuit_created(&self, hops: usize) {
        self.state
            .p2p
            .privacy
            .circuits_created
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .p2p
            .privacy
            .circuits_active
            .fetch_add(1, Ordering::Relaxed);
        // Update avg hops
        if let Ok(mut avg) = self.state.p2p.privacy.avg_hops.write() {
            let total = self
                .state
                .p2p
                .privacy
                .circuits_created
                .load(Ordering::Relaxed) as f64;
            *avg = (*avg * (total - 1.0) + hops as f64) / total;
        }
    }

    fn on_circuit_closed(&self) {
        let _ = self.state.p2p.privacy.circuits_active.fetch_update(
            Ordering::SeqCst,
            Ordering::SeqCst,
            |v| {
                if v > 0 {
                    Some(v - 1)
                } else {
                    Some(0)
                }
            },
        );
    }

    fn on_private_message(&self) {
        self.state
            .p2p
            .privacy
            .private_messages
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_cover_traffic(&self) {
        self.state
            .p2p
            .privacy
            .cover_traffic
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_relay_rotation(&self) {
        self.state
            .p2p
            .privacy
            .relay_rotations
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_trust_based_selection(&self, _selected_peer: &str, _trust: f64) {
        self.state
            .p2p
            .privacy
            .trust_based_selections
            .fetch_add(1, Ordering::Relaxed);
    }
}

// ============================================================================
// ECLVM OBSERVER IMPLEMENTATION
// ============================================================================

impl ECLVMObserver for StateIntegrator {
    // ─────────────────────────────────────────────────────────────────────────
    // POLICY ENGINE EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    fn on_policy_compiled(&self, _policy_id: &str, policy_type: &str, bytecode_size: usize) {
        // Parse policy type from string
        let ptype = match policy_type {
            "crossing" => super::state::ECLPolicyType::Crossing,
            "membership" => super::state::ECLPolicyType::Membership,
            "transaction" => super::state::ECLPolicyType::Transaction,
            "governance" => super::state::ECLPolicyType::Governance,
            "privacy" => super::state::ECLPolicyType::Privacy,
            _ => super::state::ECLPolicyType::Custom,
        };
        self.state.eclvm.policy_compiled(true, ptype);
        self.propagate_update(super::state::StateComponent::ECLPolicy);
        tracing::debug!(
            "ECL Policy compiled: {} (type: {:?}, size: {} bytes)",
            _policy_id,
            ptype,
            bytecode_size
        );
    }

    fn on_policy_compilation_failed(&self, policy_id: &str, error: &str) {
        self.state
            .eclvm
            .policy_compiled(false, super::state::ECLPolicyType::Custom);
        tracing::warn!("ECL Policy compilation failed: {} - {}", policy_id, error);
    }

    fn on_policy_executed(
        &self,
        policy_id: &str,
        policy_type: &str,
        passed: bool,
        gas_used: u64,
        mana_used: u64,
        realm_id: Option<&str>,
    ) {
        let ptype = match policy_type {
            "crossing" => super::state::ECLPolicyType::Crossing,
            "membership" => super::state::ECLPolicyType::Membership,
            "transaction" => super::state::ECLPolicyType::Transaction,
            "governance" => super::state::ECLPolicyType::Governance,
            "privacy" => super::state::ECLPolicyType::Privacy,
            _ => super::state::ECLPolicyType::Custom,
        };
        // Use 0 for duration_us as we don't have it in this context
        self.state
            .eclvm
            .policy_executed(passed, ptype, gas_used, mana_used, 0, realm_id);
        self.propagate_update(super::state::StateComponent::ECLPolicy);
        tracing::trace!(
            "ECL Policy executed: {} (passed: {}, gas: {}, mana: {}, realm: {:?})",
            policy_id,
            passed,
            gas_used,
            mana_used,
            realm_id
        );
    }

    fn on_policy_cached(&self, policy_id: &str, cache_hit: bool) {
        // Cache hits are already tracked via policies_cached in policy_compiled
        tracing::trace!("ECL Policy cache: {} (hit: {})", policy_id, cache_hit);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BLUEPRINT ENGINE EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    fn on_blueprint_published(&self, blueprint_id: &str, version: &str, _author: &EntityId) {
        self.state.eclvm.blueprint_published();
        self.propagate_update(super::state::StateComponent::ECLBlueprint);
        tracing::info!("Blueprint published: {} v{}", blueprint_id, version);
    }

    fn on_blueprint_deployed(&self, blueprint_id: &str, realm_id: &str) {
        self.state.eclvm.blueprint_deployed();
        tracing::debug!("Blueprint deployed: {} to realm {}", blueprint_id, realm_id);
    }

    fn on_blueprint_instantiated(
        &self,
        blueprint_id: &str,
        instance_id: &str,
        _gas_used: u64,
        _mana_used: u64,
    ) {
        // blueprint_instantiated only takes realm_id
        self.state.eclvm.blueprint_instantiated(blueprint_id);
        self.propagate_update(super::state::StateComponent::ECLBlueprint);
        tracing::debug!(
            "Blueprint instantiated: {} -> {}",
            blueprint_id,
            instance_id,
        );
    }

    fn on_blueprint_status_changed(&self, blueprint_id: &str, old_status: &str, new_status: &str) {
        tracing::info!(
            "Blueprint {} status changed: {} -> {}",
            blueprint_id,
            old_status,
            new_status
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // INTENT/SAGA ORCHESTRATION EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    fn on_intent_processed(
        &self,
        intent_id: &str,
        intent_type: &str,
        success: bool,
        validation_errors: usize,
    ) {
        self.state.eclvm.intent_processed(success);
        tracing::debug!(
            "Intent processed: {} (type: {}, success: {}, errors: {})",
            intent_id,
            intent_type,
            success,
            validation_errors
        );
    }

    fn on_saga_step_executed(
        &self,
        saga_id: &str,
        step_index: usize,
        total_steps: usize,
        _success: bool,
        gas_used: u64,
        mana_used: u64,
        is_cross_realm: bool,
    ) {
        // saga_step_executed takes (cross_realm, gas, mana)
        self.state
            .eclvm
            .saga_step_executed(is_cross_realm, gas_used, mana_used);

        // Also notify SagaState for compatibility
        if step_index + 1 == total_steps {
            self.state
                .peer
                .saga
                .sagas_composed
                .fetch_add(1, Ordering::Relaxed);
            self.state
                .peer
                .saga
                .successful_compositions
                .fetch_add(1, Ordering::Relaxed);
        }

        tracing::trace!(
            "Saga step executed: {} [{}/{}] (cross_realm: {})",
            saga_id,
            step_index + 1,
            total_steps,
            is_cross_realm
        );
    }

    fn on_saga_compensation(&self, saga_id: &str, step_index: usize, reason: &str) {
        // Use compensation_triggered instead
        self.state.eclvm.compensation_triggered();

        // Also update SagaState
        self.state
            .peer
            .saga
            .compensations_executed
            .fetch_add(1, Ordering::Relaxed);

        tracing::warn!(
            "Saga compensation: {} step {} (reason: {})",
            saga_id,
            step_index,
            reason
        );
    }

    fn on_cross_realm_saga(
        &self,
        saga_id: &str,
        source_realm: &str,
        target_realms: &[String],
        total_gas_budget: u64,
        total_mana_budget: u64,
    ) {
        // Update SagaState
        self.state
            .peer
            .saga
            .cross_realm_sagas
            .fetch_add(1, Ordering::Relaxed);

        tracing::info!(
            "Cross-realm saga: {} from {} to {:?} (gas: {}, mana: {})",
            saga_id,
            source_realm,
            target_realms,
            total_gas_budget,
            total_mana_budget
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // RESOURCE TRACKING EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    fn on_ecl_gas_consumed(
        &self,
        _policy_or_blueprint_id: &str,
        amount: u64,
        realm_id: Option<&str>,
    ) {
        self.state
            .eclvm
            .total_gas_consumed
            .fetch_add(amount, Ordering::Relaxed);

        // Update per-realm tracking if realm specified
        if let Some(realm) = realm_id {
            let realm_ecl = self.state.eclvm.get_or_create_realm_ecl(realm);
            realm_ecl.gas_consumed.fetch_add(amount, Ordering::Relaxed);
        }

        // Also update global ExecutionState
        self.state
            .execution
            .gas_consumed
            .fetch_add(amount, Ordering::Relaxed);
    }

    fn on_ecl_mana_consumed(
        &self,
        _policy_or_blueprint_id: &str,
        amount: u64,
        realm_id: Option<&str>,
    ) {
        self.state
            .eclvm
            .total_mana_consumed
            .fetch_add(amount, Ordering::Relaxed);

        // Update per-realm tracking if realm specified
        if let Some(realm) = realm_id {
            let realm_ecl = self.state.eclvm.get_or_create_realm_ecl(realm);
            realm_ecl.mana_consumed.fetch_add(amount, Ordering::Relaxed);
        }

        // Also update global ExecutionState
        self.state
            .execution
            .mana_consumed
            .fetch_add(amount, Ordering::Relaxed);
    }

    fn on_ecl_out_of_gas(&self, context: &str, required: u64, available: u64) {
        self.state.eclvm.out_of_gas();

        tracing::warn!(
            "ECL Out-of-Gas: {} (required: {}, available: {})",
            context,
            required,
            available
        );
    }

    fn on_ecl_out_of_mana(&self, context: &str, required: u64, available: u64) {
        self.state.eclvm.rate_limited();

        tracing::warn!(
            "ECL Out-of-Mana: {} (required: {}, available: {})",
            context,
            required,
            available
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // CROSSING-POLICY EVENTS (Κ23 Gateway-Integration)
    // ─────────────────────────────────────────────────────────────────────────

    fn on_crossing_policy_evaluated(
        &self,
        from_realm: &str,
        to_realm: &str,
        entity: &EntityId,
        allowed: bool,
        trust_score: f64,
        policy_id: Option<&str>,
    ) {
        self.state
            .eclvm
            .crossing_policy_evaluated(allowed, from_realm, to_realm);
        self.propagate_update(super::state::StateComponent::ECLPolicy);

        // Also update RealmState for the realms involved
        if allowed {
            self.state
                .peer
                .realm
                .crossing_succeeded(from_realm, to_realm);
        } else {
            self.state.peer.realm.crossing_failed();
        }

        tracing::debug!(
            "Crossing policy evaluated: {} -> {} for {:?} (allowed: {}, trust: {:.2}, policy: {:?})",
            from_realm,
            to_realm,
            entity,
            allowed,
            trust_score,
            policy_id
        );
    }

    fn on_membership_check(
        &self,
        realm_id: &str,
        entity: &EntityId,
        is_member: bool,
        required_trust: f64,
        actual_trust: f64,
    ) {
        // Track membership check via policy execution
        let ptype = super::state::ECLPolicyType::Membership;
        self.state
            .eclvm
            .policy_executed(is_member, ptype, 100, 10, 0, Some(realm_id));

        tracing::debug!(
            "Membership check: realm {} entity {:?} (member: {}, required: {:.2}, actual: {:.2})",
            realm_id,
            entity,
            is_member,
            required_trust,
            actual_trust
        );
    }

    fn on_governance_rule_applied(
        &self,
        realm_id: &str,
        rule_id: &str,
        action: &str,
        success: bool,
    ) {
        let ptype = super::state::ECLPolicyType::Governance;
        self.state
            .eclvm
            .policy_executed(success, ptype, 200, 50, 0, Some(realm_id));

        tracing::info!(
            "Governance rule applied: {} in realm {} (action: {}, success: {})",
            rule_id,
            realm_id,
            action,
            success
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // REALM-SPECIFIC ECL EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    fn on_realm_ecl_policy_activated(&self, realm_id: &str, policy_id: &str, policy_type: &str) {
        let realm_ecl = self.state.eclvm.get_or_create_realm_ecl(realm_id);
        // active_policies is a RwLock<Vec<String>>, so we need to write-lock it
        if let Ok(mut policies) = realm_ecl.active_policies.write() {
            if !policies.contains(&policy_id.to_string()) {
                policies.push(policy_id.to_string());
            }
        }

        tracing::debug!(
            "ECL Policy activated in realm {}: {} (type: {})",
            realm_id,
            policy_id,
            policy_type
        );
    }

    fn on_realm_ecl_policy_deactivated(&self, realm_id: &str, policy_id: &str) {
        let realm_ecl = self.state.eclvm.get_or_create_realm_ecl(realm_id);
        // active_policies is a RwLock<Vec<String>>, so we need to write-lock it
        if let Ok(mut policies) = realm_ecl.active_policies.write() {
            policies.retain(|p| p != policy_id);
        }

        tracing::debug!(
            "ECL Policy deactivated in realm {}: {}",
            realm_id,
            policy_id
        );
    }

    fn on_realm_ecl_metrics(
        &self,
        realm_id: &str,
        policies_executed: u64,
        gas_consumed: u64,
        mana_consumed: u64,
        active_policies: usize,
    ) {
        tracing::trace!(
            "Realm ECL metrics: {} (executed: {}, gas: {}, mana: {}, active: {})",
            realm_id,
            policies_executed,
            gas_consumed,
            mana_consumed,
            active_policies
        );
    }
}

// ============================================================================
// SHARED OBSERVER TYPES
// ============================================================================

/// Shared Trust Observer
pub type SharedTrustObserver = Arc<dyn TrustObserver>;

/// Shared Event Observer
pub type SharedEventObserver = Arc<dyn EventObserver>;

/// Shared Execution Observer
pub type SharedExecutionObserver = Arc<dyn ExecutionObserver>;

/// Shared Protection Observer
pub type SharedProtectionObserver = Arc<dyn ProtectionObserver>;

/// Shared Formula Observer
pub type SharedFormulaObserver = Arc<dyn FormulaObserver>;

/// Shared Consensus Observer
pub type SharedConsensusObserver = Arc<dyn ConsensusObserver>;

/// Shared Storage Observer
pub type SharedStorageObserver = Arc<dyn StorageObserver>;

/// Shared Gateway Observer
pub type SharedGatewayObserver = Arc<dyn GatewayObserver>;

/// Shared Saga Observer
pub type SharedSagaObserver = Arc<dyn SagaObserver>;

/// Shared Intent Observer
pub type SharedIntentObserver = Arc<dyn IntentObserver>;

/// Shared Realm Observer (Κ22-Κ24)
pub type SharedRealmObserver = Arc<dyn RealmObserver>;

/// Shared Swarm Observer
pub type SharedSwarmObserver = Arc<dyn SwarmObserver>;

/// Shared Gossip Observer
pub type SharedGossipObserver = Arc<dyn GossipObserver>;

/// Shared Kademlia Observer
pub type SharedKademliaObserver = Arc<dyn KademliaObserver>;

/// Shared Relay Observer
pub type SharedRelayObserver = Arc<dyn RelayObserver>;

/// Shared Privacy Observer
pub type SharedPrivacyObserver = Arc<dyn PrivacyObserver>;

// ============================================================================
// COMPOSITE OBSERVER
// ============================================================================

/// Composite Observer - Leitet an mehrere Observer weiter
pub struct CompositeObserver<O> {
    observers: Vec<Arc<O>>,
}

impl<O> CompositeObserver<O> {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn add(&mut self, observer: Arc<O>) {
        self.observers.push(observer);
    }

    pub fn observers(&self) -> &[Arc<O>] {
        &self.observers
    }
}

impl<O> Default for CompositeObserver<O> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ENGINE EXTENSION TRAITS
// ============================================================================

/// Trait für Engines die Observer unterstützen
pub trait ObservableEngine<O> {
    /// Observer hinzufügen
    fn add_observer(&mut self, observer: Arc<O>);

    /// Observer entfernen (falls möglich)
    fn remove_all_observers(&mut self);
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::create_unified_state;
    use crate::domain::UniversalId;

    #[test]
    fn test_state_integrator() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Trust Updates
        let entity = UniversalId::new(UniversalId::TAG_DID, 1, b"test-entity");
        integrator.on_entity_registered(&entity);
        integrator.on_trust_update(&entity, &entity, 0.5, 0.6, true);
        integrator.on_trust_update(&entity, &entity, 0.6, 0.5, false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.core.trust.entities, 1);
        assert_eq!(snapshot.core.trust.updates_total, 2);
        assert_eq!(snapshot.core.trust.positive_updates, 1);
        assert_eq!(snapshot.core.trust.negative_updates, 1);
    }

    #[test]
    fn test_event_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        let event_id = UniversalId::new(UniversalId::TAG_EVENT, 1, b"test-event");
        integrator.on_event_added(&event_id, true, 0, 0);
        integrator.on_event_added(&event_id, false, 2, 1);
        integrator.on_event_finalized(&event_id, 100);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.core.events.total, 2);
        assert_eq!(snapshot.core.events.genesis, 1);
        assert_eq!(snapshot.core.events.finalized, 1);
    }

    #[test]
    fn test_execution_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        integrator.on_execution_start(1);
        integrator.on_gas_consumed(1000);
        integrator.on_mana_consumed(100);
        integrator.on_execution_complete(1, true, 1000, 100, 5, 50);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.execution.total_executions, 1);
        assert_eq!(snapshot.execution.successful, 1);
        assert_eq!(snapshot.execution.gas_consumed, 2000); // 1000 direct + 1000 in complete
        assert_eq!(snapshot.execution.mana_consumed, 200);
    }

    #[test]
    fn test_gateway_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        let entity = UniversalId::new(UniversalId::TAG_DID, 1, b"test-entity");
        integrator.on_crossing_allowed(&entity, "realm-a", "realm-b", 0.8);
        integrator.on_crossing_denied(&entity, "realm-a", "realm-c", "trust");
        // Explicitly call GatewayObserver's method to avoid ambiguity with RealmObserver
        GatewayObserver::on_realm_registered(&integrator, "realm-d");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.peer.gateway.crossings_total, 2);
        assert_eq!(snapshot.peer.gateway.crossings_allowed, 1);
        assert_eq!(snapshot.peer.gateway.crossings_denied, 1);
        assert_eq!(snapshot.peer.gateway.registered_realms, 1);
    }

    #[test]
    fn test_realm_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Test realm registration via RealmObserver
        RealmObserver::on_realm_registered(&integrator, "test-realm", 0.5, "quadratic");
        RealmObserver::on_realm_registered(&integrator, "finance-realm", 0.8, "token");
        integrator.on_root_realm_set("test-realm");

        // Test crossings
        integrator.on_crossing_succeeded("test-realm", "finance-realm");
        integrator.on_crossing_failed("test-realm", "finance-realm", "trust");

        // Test cross-realm saga
        integrator.on_cross_realm_saga_started("saga-1", &["test-realm", "finance-realm"]);

        // Test identity management
        integrator.on_identity_joined_realm("identity-1", "test-realm");
        integrator.on_identity_joined_realm("identity-2", "test-realm");
        integrator.on_identity_left_realm("identity-1", "test-realm");

        // Test rule management
        integrator.on_rule_added_to_realm("test-realm", "rule-1");
        integrator.on_rule_added_to_realm("test-realm", "rule-2");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.peer.realm.total_realms, 2);
        assert_eq!(snapshot.peer.realm.root_realm_id, Some("test-realm".into()));
        assert_eq!(snapshot.peer.realm.active_crossings, 1);
        assert_eq!(snapshot.peer.realm.crossing_failures, 1);
        assert_eq!(snapshot.peer.realm.total_cross_realm_sagas, 1);

        // Test realm-specific state
        let test_realm = snapshot.peer.realm.realms.get("test-realm").unwrap();
        assert_eq!(test_realm.member_count, 1); // 2 joined, 1 left
        assert_eq!(test_realm.crossings_out, 1);
        assert_eq!(test_realm.active_rules.len(), 2);
        assert!(test_realm.active_rules.contains(&"rule-1".to_string()));

        let finance_realm = snapshot.peer.realm.realms.get("finance-realm").unwrap();
        assert_eq!(finance_realm.crossings_in, 1);
        assert_eq!(finance_realm.min_trust, 0.8);
        assert_eq!(finance_realm.governance_type, "token");
    }

    #[test]
    fn test_saga_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        integrator.on_saga_composed("saga-1", 3, "Transfer", true);
        integrator.on_saga_composed("saga-2", 5, "Delegate", true);
        integrator.on_compensation_executed("saga-1", 2, true);
        // Use SagaObserver version
        SagaObserver::on_cross_realm_saga(
            &integrator,
            "saga-3",
            &["realm-a".into(), "realm-b".into()],
        );

        let snapshot = state.snapshot();
        assert_eq!(snapshot.peer.saga.sagas_composed, 2);
        assert_eq!(snapshot.peer.saga.successful_compositions, 2);
        assert_eq!(snapshot.peer.saga.compensations_executed, 1);
        assert_eq!(snapshot.peer.saga.cross_realm_sagas, 1);
    }

    #[test]
    fn test_swarm_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        integrator.on_peer_connected("peer-1", true, false);
        integrator.on_peer_connected("peer-2", false, false);
        integrator.on_latency_measured("peer-1", 5000);
        integrator.on_bytes_transferred(1000, 2000);
        integrator.on_nat_status_changed("public");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.p2p.swarm.connected_peers, 2);
        assert_eq!(snapshot.p2p.swarm.inbound_connections, 1);
        assert_eq!(snapshot.p2p.swarm.outbound_connections, 1);
        assert_eq!(snapshot.p2p.swarm.bytes_sent, 1000);
        assert_eq!(snapshot.p2p.swarm.bytes_received, 2000);
        assert_eq!(
            snapshot.p2p.swarm.nat_status,
            crate::core::state::NatStatus::Public
        );
    }

    #[test]
    fn test_gossip_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        integrator.on_message_received("events/v1", "peer-1");
        integrator.on_message_sent("events/v1");
        integrator.on_message_validated(true);
        integrator.on_peer_grafted("peer-2", "events/v1");
        integrator.on_topic_subscribed("events/v1");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.p2p.gossip.messages_received, 1);
        assert_eq!(snapshot.p2p.gossip.messages_sent, 1);
        assert_eq!(snapshot.p2p.gossip.messages_validated, 1);
        assert_eq!(snapshot.p2p.gossip.mesh_peers, 1);
        assert_eq!(snapshot.p2p.gossip.subscribed_topics, 1);
    }

    #[test]
    fn test_relay_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        integrator.on_reservation_accepted("relay-peer");
        integrator.on_circuit_opened();
        integrator.on_dcutr_attempt(true);
        integrator.on_dcutr_attempt(false);
        integrator.on_relay_bytes(5000);

        let snapshot = state.snapshot();
        assert!(snapshot.p2p.relay.has_reservation);
        assert_eq!(
            snapshot.p2p.relay.relay_peer,
            Some("relay-peer".to_string())
        );
        assert_eq!(snapshot.p2p.relay.circuits_active, 1);
        assert_eq!(snapshot.p2p.relay.dcutr_successes, 1);
        assert_eq!(snapshot.p2p.relay.dcutr_failures, 1);
    }

    #[test]
    fn test_privacy_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        integrator.on_circuit_created(3);
        integrator.on_private_message();
        integrator.on_cover_traffic();
        integrator.on_trust_based_selection("peer-1", 0.8);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.p2p.privacy.circuits_created, 1);
        assert_eq!(snapshot.p2p.privacy.circuits_active, 1);
        assert_eq!(snapshot.p2p.privacy.private_messages, 1);
        assert_eq!(snapshot.p2p.privacy.cover_traffic, 1);
        assert_eq!(snapshot.p2p.privacy.trust_based_selections, 1);
    }

    #[test]
    fn test_warning_propagation() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Byzantine detected should add warning
        let validator = UniversalId::new(UniversalId::TAG_DID, 1, b"bad-validator");
        integrator.on_byzantine_detected(&validator);

        let snapshot = state.snapshot();
        assert!(snapshot.warnings.iter().any(|w| w.contains("Byzantine")));
    }

    #[test]
    fn test_p2p_warnings() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Connect one peer - should trigger low peer warning
        integrator.on_peer_connected("peer-1", false, false);

        let snapshot = state.snapshot();
        assert!(snapshot
            .warnings
            .iter()
            .any(|w| w.contains("Low peer count")));

        // Connect more peers - warning should clear
        integrator.on_peer_connected("peer-2", false, false);
        integrator.on_peer_connected("peer-3", false, false);

        let snapshot2 = state.snapshot();
        assert!(!snapshot2
            .warnings
            .iter()
            .any(|w| w.contains("Low peer count")));
    }
}
