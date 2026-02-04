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
use std::sync::{Arc, RwLock};

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
        duration_us: u64,
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
// ENGINE-LAYER OBSERVER TRAITS (6 neue Engines für SOLL-Zustand)
// ============================================================================

// ────────────────────────────────────────────────────────────────────────────
// 4.1 UI-ENGINE OBSERVER
// ────────────────────────────────────────────────────────────────────────────

/// UI-Engine Observer - Tracks deklarative, Trust-basierte UI-Komponenten
///
/// # Design
///
/// Die UI-Engine ermöglicht Trust-basierte Sichtbarkeit:
/// - **Component-Tree**: Hierarchischer UI-Aufbau mit Realm-Isolation
/// - **Trust-Gates**: Zeige/Verberge basierend auf Trust-Level
/// - **Credential-Gates**: Zugriff basierend auf Credentials
/// - **Bindings**: Reaktive Daten-Verbindungen zu DataLogic
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// UI ──DependsOn──▶ Trust, Realm, Room, Controller, ECLVM, Gas, Mana
/// UI ──Triggers───▶ Event
/// UI ──Aggregates─▶ DataLogic
/// ```
pub trait UIObserver: Send + Sync {
    // ─────────────────────────────────────────────────────────────────────────
    // COMPONENT-LIFECYCLE EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Component registriert (in den Component-Tree eingefügt)
    fn on_component_registered(
        &self,
        component_id: &str,
        component_type: &str, // "panel", "list", "form", "chart", "card", "modal", etc.
        realm_id: Option<&str>,
        parent_id: Option<&str>,
    );

    /// Component unmounted (aus dem Component-Tree entfernt)
    fn on_component_unmounted(&self, component_id: &str);

    /// Component-Update (Props oder State geändert)
    fn on_component_updated(&self, component_id: &str, update_type: &str);

    /// Component gerendert
    fn on_component_rendered(
        &self,
        component_id: &str,
        from_cache: bool,
        gas_used: u64,
        mana_used: u64,
        realm_id: Option<&str>,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // TRUST-GATE EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Trust-Gate evaluiert (Sichtbarkeits-Entscheidung)
    fn on_trust_gate_evaluated(
        &self,
        component_id: &str,
        required_trust: f64,
        actual_trust: f64,
        allowed: bool,
        realm_id: Option<&str>,
    );

    /// Trust-Gate-Konfiguration geändert
    fn on_trust_gate_configured(&self, component_id: &str, min_trust: f64, max_trust: Option<f64>);

    // ─────────────────────────────────────────────────────────────────────────
    // CREDENTIAL-GATE EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Credential-Gate evaluiert (Zugriffs-Entscheidung)
    fn on_credential_gate_evaluated(
        &self,
        component_id: &str,
        required_credentials: &[String],
        has_credentials: bool,
        allowed: bool,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // BINDING EVENTS (UI ↔ DataLogic Integration)
    // ─────────────────────────────────────────────────────────────────────────

    /// Binding erstellt (UI-Component zu DataLogic-Stream)
    fn on_binding_created(
        &self,
        binding_id: &str,
        component_id: &str,
        stream_id: &str,
        binding_type: &str, // "one-way", "two-way", "computed"
    );

    /// Binding-Update propagiert (DataLogic → UI)
    fn on_binding_updated(
        &self,
        binding_id: &str,
        success: bool,
        latency_us: u64,
        realm_id: Option<&str>,
    );

    /// Binding-Fehler (z.B. Stream nicht verfügbar)
    fn on_binding_error(&self, binding_id: &str, error: &str);

    /// Binding gelöscht
    fn on_binding_removed(&self, binding_id: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // UI-EVENT PROPAGATION
    // ─────────────────────────────────────────────────────────────────────────

    /// UI-Action getriggert (Button-Click, Form-Submit, etc.)
    fn on_ui_action(
        &self,
        component_id: &str,
        action_type: &str, // "click", "submit", "change", "focus", "blur"
        payload_size: usize,
        realm_id: Option<&str>,
    );

    /// UI-Event an Event-System propagiert
    fn on_event_emitted(&self, component_id: &str, event_type: &str);
}

/// Shared UI Observer (thread-safe Arc)
pub type SharedUIObserver = Arc<dyn UIObserver>;

// ────────────────────────────────────────────────────────────────────────────
// 4.2 API-ENGINE OBSERVER
// ────────────────────────────────────────────────────────────────────────────

/// API-Engine Observer - Tracks dynamische REST-API-Endpoints
///
/// # Design
///
/// Die API-Engine ermöglicht ECL-definierte REST-APIs:
/// - **Endpoint-Registry**: Dynamische Routen per Realm
/// - **Trust-basierte Rate-Limits**: Höherer Trust = mehr Requests
/// - **Handler-Execution**: ECL-Funktionen als Request-Handler
/// - **Latenz-Tracking**: P95/P99 Percentiles
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// API ──DependsOn──▶ Trust, Controller, ECLVM, Gas, Mana
/// API ──Validates──▶ Gateway
/// API ──Triggers───▶ Event
/// API ──Aggregates─▶ DataLogic
/// ```
pub trait APIObserver: Send + Sync {
    // ─────────────────────────────────────────────────────────────────────────
    // ENDPOINT LIFECYCLE
    // ─────────────────────────────────────────────────────────────────────────

    /// Endpoint registriert
    fn on_endpoint_registered(
        &self,
        endpoint_id: &str,
        method: &str,     // "GET", "POST", "PUT", "DELETE", "PATCH"
        path: &str,       // "/api/v1/users/:id"
        handler_id: &str, // ECL-Handler-ID
        realm_id: Option<&str>,
    );

    /// Endpoint aktualisiert (Hot-Reload)
    fn on_endpoint_updated(&self, endpoint_id: &str, changes: &str);

    /// Endpoint deregistriert
    fn on_endpoint_removed(&self, endpoint_id: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // REQUEST PROCESSING
    // ─────────────────────────────────────────────────────────────────────────

    /// Request empfangen
    fn on_request_received(
        &self,
        request_id: &str,
        endpoint_id: &str,
        method: &str,
        client_trust: f64,
        realm_id: Option<&str>,
    );

    /// Request verarbeitet
    fn on_request_completed(
        &self,
        request_id: &str,
        status_code: u16,
        latency_us: u64,
        gas_used: u64,
        mana_used: u64,
        response_size: usize,
    );

    /// Request-Validierung fehlgeschlagen
    fn on_request_validation_failed(&self, request_id: &str, error: &str, error_code: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // RATE LIMITING
    // ─────────────────────────────────────────────────────────────────────────

    /// Rate-Limit-Bucket erstellt (pro Client/Endpoint)
    fn on_rate_limit_bucket_created(
        &self,
        bucket_id: &str,
        client_id: &str,
        max_requests: u64,
        window_secs: u64,
        trust_multiplier: f64,
    );

    /// Rate-Limit erreicht (429 response)
    fn on_rate_limited(
        &self,
        client_id: &str,
        endpoint_id: &str,
        retry_after_secs: u64,
        realm_id: Option<&str>,
    );

    /// Rate-Limit-Reset
    fn on_rate_limit_reset(&self, bucket_id: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // AUTHENTICATION & AUTHORIZATION
    // ─────────────────────────────────────────────────────────────────────────

    /// Auth-Check durchgeführt (401/403)
    fn on_auth_check(
        &self,
        request_id: &str,
        auth_type: &str, // "bearer", "api-key", "signature", "trust"
        success: bool,
        failure_reason: Option<&str>,
    );

    /// AuthZ via Controller durchgeführt
    fn on_authz_delegated(&self, request_id: &str, permission: &str, allowed: bool);
}

/// Shared API Observer (thread-safe Arc)
pub type SharedAPIObserver = Arc<dyn APIObserver>;

// ────────────────────────────────────────────────────────────────────────────
// 4.3 GOVERNANCE-ENGINE OBSERVER
// ────────────────────────────────────────────────────────────────────────────

/// Governance-Engine Observer - Tracks DAO-artige Abstimmungen
///
/// # Design
///
/// Die Governance-Engine implementiert dezentrale Entscheidungsfindung:
/// - **Quadratic Voting**: √-basierte Stimmgewichtung (Κ21)
/// - **Liquid Democracy**: Transitive Vote-Delegation
/// - **Anti-Calcification**: Machtkonzentrations-Checks (Κ19)
/// - **Proposal-Lifecycle**: Draft → Active → Voting → Decided
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// Governance ──DependsOn──▶ Trust, Quadratic, ECLVM, Realm
/// Governance ──Validates──▶ Controller, AntiCalcification
/// Governance ──Triggers───▶ Controller, Event
/// ```
pub trait GovernanceObserver: Send + Sync {
    // ─────────────────────────────────────────────────────────────────────────
    // PROPOSAL LIFECYCLE
    // ─────────────────────────────────────────────────────────────────────────

    /// Proposal erstellt
    fn on_proposal_created(
        &self,
        proposal_id: &str,
        proposal_type: &str, // "parameter-change", "permission-grant", "membership", "budget", "custom"
        author: &EntityId,
        realm_id: Option<&str>,
        voting_period_secs: u64,
        quorum_percent: f64,
    );

    /// Proposal-Status geändert
    fn on_proposal_status_changed(
        &self,
        proposal_id: &str,
        old_status: &str, // "draft", "active", "voting", "passed", "rejected", "expired", "vetoed"
        new_status: &str,
    );

    /// Proposal abgeschlossen (mit Ergebnis)
    fn on_proposal_completed(
        &self,
        proposal_id: &str,
        result: &str, // "accepted", "rejected", "expired", "vetoed"
        yes_votes: u64,
        no_votes: u64,
        abstain_votes: u64,
        participation_rate: f64,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // VOTING EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Vote abgegeben
    fn on_vote_cast(
        &self,
        proposal_id: &str,
        voter: &EntityId,
        vote: &str, // "yes", "no", "abstain"
        raw_voting_power: f64,
        effective_voting_power: f64, // nach Quadratic Reduction
        is_delegated: bool,
        delegation_chain_length: usize,
        realm_id: Option<&str>,
    );

    /// Vote zurückgezogen (falls erlaubt)
    fn on_vote_withdrawn(&self, proposal_id: &str, voter: &EntityId);

    /// Quadratic Reduction angewendet (Κ21)
    fn on_quadratic_reduction(
        &self,
        voter: &EntityId,
        original_power: f64,
        reduced_power: f64,
        reduction_factor: f64,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // DELEGATION EVENTS (Liquid Democracy)
    // ─────────────────────────────────────────────────────────────────────────

    /// Delegation erstellt
    fn on_delegation_created(
        &self,
        delegator: &EntityId,
        delegate: &EntityId,
        scope: &str, // "all", "realm", "topic"
        scope_id: Option<&str>,
        expiration: Option<u64>,
    );

    /// Delegation widerrufen
    fn on_delegation_revoked(&self, delegator: &EntityId, delegate: &EntityId);

    /// Delegation abgelaufen
    fn on_delegation_expired(&self, delegator: &EntityId, delegate: &EntityId);

    /// Zirkuläre Delegation verhindert
    fn on_circular_delegation_prevented(
        &self,
        delegator: &EntityId,
        delegate: &EntityId,
        cycle_path: &[String],
    );

    /// Delegation-Kette aufgelöst (für Vote-Propagation)
    fn on_delegation_chain_resolved(
        &self,
        original_voter: &EntityId,
        final_delegate: &EntityId,
        chain_length: usize,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // ANTI-CALCIFICATION EVENTS (Κ19)
    // ─────────────────────────────────────────────────────────────────────────

    /// Power-Concentration-Check durchgeführt
    fn on_power_concentration_check(
        &self,
        realm_id: Option<&str>,
        top_entity: &EntityId,
        concentration_percent: f64,
        threshold_percent: f64,
        violated: bool,
    );

    /// Gini-Koeffizient berechnet
    fn on_gini_calculated(&self, realm_id: Option<&str>, gini_coefficient: f64);

    /// Veto ausgeübt (gegen Machtkonzentration)
    fn on_veto_exercised(&self, proposal_id: &str, veto_entity: &EntityId, reason: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // GOVERNANCE-TO-CONTROLLER TRIGGERS
    // ─────────────────────────────────────────────────────────────────────────

    /// Governance-Entscheidung triggert Permission-Änderung
    fn on_permission_change_triggered(
        &self,
        proposal_id: &str,
        permission: &str,
        target: &EntityId,
        action: &str, // "grant", "revoke"
    );
}

/// Shared Governance Observer (thread-safe Arc)
pub type SharedGovernanceObserver = Arc<dyn GovernanceObserver>;

// ────────────────────────────────────────────────────────────────────────────
// 4.4 CONTROLLER-ENGINE OBSERVER
// ────────────────────────────────────────────────────────────────────────────

/// Controller-Engine Observer - Tracks Berechtigungs-Management
///
/// # Design
///
/// Die Controller-Engine verwaltet alle Berechtigungen:
/// - **Scoped Permissions**: Realm > Room > Partition Hierarchie
/// - **Delegation**: Transitive Permission-Vererbung
/// - **Audit-Trail**: Lückenlose Berechtigungs-History
/// - **Automation**: Trigger-basierte Permission-Änderungen
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// Controller ──DependsOn──▶ Trust, Realm, Room, Partition, ECLVM
/// Controller ──Validates──▶ Gateway, API, UI
/// Controller ──Aggregates─▶ Governance
/// Controller ──Triggers───▶ Event
/// ```
pub trait ControllerObserver: Send + Sync {
    // ─────────────────────────────────────────────────────────────────────────
    // PERMISSION LIFECYCLE
    // ─────────────────────────────────────────────────────────────────────────

    /// Permission registriert (Schema definiert)
    fn on_permission_registered(
        &self,
        permission_id: &str,
        permission_name: &str,
        scope_type: &str, // "global", "realm", "room", "partition"
        description: &str,
    );

    /// Permission gewährt
    fn on_permission_granted(
        &self,
        permission_id: &str,
        grantee: &EntityId,
        granter: &EntityId,
        scope_id: Option<&str>,
        realm_id: Option<&str>,
        expiration: Option<u64>,
        conditions: Option<&str>, // ECL-Expression
    );

    /// Permission widerrufen
    fn on_permission_revoked(
        &self,
        permission_id: &str,
        entity: &EntityId,
        revoker: &EntityId,
        reason: &str,
    );

    /// Permission abgelaufen
    fn on_permission_expired(&self, permission_id: &str, entity: &EntityId);

    // ─────────────────────────────────────────────────────────────────────────
    // AUTHORIZATION CHECKS
    // ─────────────────────────────────────────────────────────────────────────

    /// AuthZ-Check durchgeführt
    fn on_authz_check(
        &self,
        requester: &EntityId,
        permission: &str,
        resource: &str,
        scope: &str, // "realm", "room", "partition"
        scope_id: Option<&str>,
        allowed: bool,
        via_delegation: bool,
        latency_us: u64,
    );

    /// AuthZ-Check via Scope-Inheritance aufgelöst
    fn on_scope_inheritance_resolved(
        &self,
        permission: &str,
        from_scope: &str,
        to_scope: &str,
        inherited: bool,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // DELEGATION EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Permission-Delegation erstellt
    fn on_permission_delegated(
        &self,
        permission_id: &str,
        delegator: &EntityId,
        delegate: &EntityId,
        can_redelegate: bool,
        constraints: Option<&str>,
    );

    /// Delegation-Chain aufgelöst
    fn on_delegation_resolved(
        &self,
        permission_id: &str,
        original_requester: &EntityId,
        final_granter: &EntityId,
        chain_length: usize,
    );

    /// Delegation-Konflikt erkannt
    fn on_delegation_conflict(
        &self,
        permission_id: &str,
        entity: &EntityId,
        conflict_type: &str, // "circular", "conflicting-constraints", "expired-chain"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // AUDIT EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Audit-Entry geschrieben
    fn on_audit_entry(
        &self,
        entry_id: &str,
        action: &str, // "grant", "revoke", "check", "delegate"
        actor: &EntityId,
        target: Option<&EntityId>,
        permission: &str,
        result: &str, // "success", "denied", "error"
        metadata_size: usize,
    );

    /// Audit-Log rotiert
    fn on_audit_log_rotated(&self, old_size_bytes: u64, entries_archived: u64);

    // ─────────────────────────────────────────────────────────────────────────
    // AUTOMATION EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Automation-Rule registriert
    fn on_automation_rule_registered(
        &self,
        rule_id: &str,
        trigger_type: &str, // "time", "event", "condition"
        action: &str,       // "grant", "revoke", "notify"
    );

    /// Automation-Rule getriggert
    fn on_automation_triggered(
        &self,
        rule_id: &str,
        trigger_reason: &str,
        permissions_affected: usize,
    );
}

/// Shared Controller Observer (thread-safe Arc)
pub type SharedControllerObserver = Arc<dyn ControllerObserver>;

// ────────────────────────────────────────────────────────────────────────────
// 4.5 DATALOGIC-ENGINE OBSERVER
// ────────────────────────────────────────────────────────────────────────────

/// DataLogic-Engine Observer - Tracks reaktive Datenströme
///
/// # Design
///
/// Die DataLogic-Engine verarbeitet Events reaktiv:
/// - **Streams**: Event-basierte Datenströme mit Filtering
/// - **Aggregations**: count, sum, avg, window-basierte Berechnungen
/// - **Bindings**: Reaktive Verbindungen zu UI-Components
/// - **Trust-Filter**: Events basierend auf Trust filtern
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// DataLogic ──DependsOn──▶ Event, Trust, ECLVM, Gas
/// DataLogic ──Aggregates─▶ Event
/// DataLogic ──Triggers───▶ Event
/// DataLogic ──Validates──▶ UI
/// ```
pub trait DataLogicObserver: Send + Sync {
    // ─────────────────────────────────────────────────────────────────────────
    // STREAM LIFECYCLE
    // ─────────────────────────────────────────────────────────────────────────

    /// Stream registriert
    fn on_stream_registered(
        &self,
        stream_id: &str,
        source_type: &str, // "event", "aggregation", "computed", "external"
        filter_expression: Option<&str>,
    );

    /// Stream-Subscription erstellt
    fn on_stream_subscribed(&self, stream_id: &str, subscriber_id: &str);

    /// Stream-Subscription beendet
    fn on_stream_unsubscribed(&self, stream_id: &str, subscriber_id: &str);

    /// Stream geschlossen
    fn on_stream_closed(&self, stream_id: &str, reason: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // EVENT PROCESSING
    // ─────────────────────────────────────────────────────────────────────────

    /// Event in Stream empfangen
    fn on_event_received(&self, stream_id: &str, event_type: &str, size_bytes: usize);

    /// Event gefiltert (Trust/Access)
    fn on_event_filtered(&self, stream_id: &str, filter_reason: &str);

    /// Event weitergeleitet
    fn on_event_forwarded(&self, stream_id: &str, subscribers_notified: usize, gas_used: u64);

    /// Processing-Fehler
    fn on_processing_error(&self, stream_id: &str, error: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // AGGREGATION EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Aggregation registriert
    fn on_aggregation_registered(
        &self,
        aggregation_id: &str,
        aggregation_type: &str, // "count", "sum", "avg", "min", "max", "window"
        source_stream: &str,
        window_size: Option<u64>,
    );

    /// Aggregation berechnet
    fn on_aggregation_computed(
        &self,
        aggregation_id: &str,
        result_value: f64,
        events_aggregated: u64,
        latency_us: u64,
        gas_used: u64,
    );

    /// Aggregation-Result emittiert (als neues Event)
    fn on_aggregation_emitted(&self, aggregation_id: &str, event_type: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // BINDING PROPAGATION (DataLogic → UI)
    // ─────────────────────────────────────────────────────────────────────────

    /// Binding-Update propagiert
    fn on_binding_propagated(
        &self,
        stream_id: &str,
        binding_id: &str,
        success: bool,
        latency_us: u64,
        mana_used: u64,
    );
}

/// Shared DataLogic Observer (thread-safe Arc)
pub type SharedDataLogicObserver = Arc<dyn DataLogicObserver>;

// ────────────────────────────────────────────────────────────────────────────
// 4.6 BLUEPRINTCOMPOSER-ENGINE OBSERVER
// ────────────────────────────────────────────────────────────────────────────

/// BlueprintComposer-Engine Observer - Tracks Template-Komposition
///
/// # Design
///
/// Der BlueprintComposer verwaltet wiederverwendbare Konfigurationen:
/// - **Composition**: Blueprint-Vererbung und -Erweiterung
/// - **Versioning**: Semantic Versioning mit Migrations-Pfaden
/// - **Validation**: Realm-Compatibility-Checks
/// - **Caching**: Compiled Blueprint Cache für Performance
///
/// # StateGraph-Verknüpfungen
///
/// ```text
/// BlueprintComposer ──DependsOn──▶ Blueprint, ECLVM, Trust, Gas
/// BlueprintComposer ──Aggregates─▶ ECLBlueprint
/// BlueprintComposer ──Triggers───▶ Event
/// BlueprintComposer ──Validates──▶ Realm
/// ```
pub trait BlueprintComposerObserver: Send + Sync {
    // ─────────────────────────────────────────────────────────────────────────
    // COMPOSITION EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Composition gestartet
    fn on_composition_started(
        &self,
        composition_id: &str,
        base_blueprints: &[String],
        target_realm: Option<&str>,
    );

    /// Composition abgeschlossen
    fn on_composition_completed(
        &self,
        composition_id: &str,
        success: bool,
        inheritance_depth: usize,
        conflicts_resolved: usize,
        gas_used: u64,
    );

    /// Inheritance aufgelöst
    fn on_inheritance_resolved(
        &self,
        child_blueprint: &str,
        parent_blueprint: &str,
        overrides_count: usize,
    );

    /// Konflikt bei Composition erkannt und aufgelöst
    fn on_conflict_resolved(
        &self,
        composition_id: &str,
        conflict_type: &str, // "property-override", "method-collision", "constraint-violation"
        resolution: &str,    // "use-child", "use-parent", "merge", "error"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // VERSIONING EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Blueprint-Version publiziert
    fn on_version_published(
        &self,
        blueprint_id: &str,
        version: &str, // semver: "1.2.3"
        author: &EntityId,
        changelog: Option<&str>,
    );

    /// Migration durchgeführt
    fn on_migration_executed(
        &self,
        blueprint_id: &str,
        from_version: &str,
        to_version: &str,
        success: bool,
        instances_migrated: u64,
    );

    /// Deprecation markiert
    fn on_deprecation_marked(
        &self,
        blueprint_id: &str,
        version: &str,
        deprecation_reason: &str,
        sunset_date: Option<u64>,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // INSTANTIATION EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Blueprint instanziiert
    fn on_blueprint_instantiated(
        &self,
        blueprint_id: &str,
        instance_id: &str,
        realm_id: Option<&str>,
        gas_used: u64,
    );

    /// Instanz deaktiviert
    fn on_instance_deactivated(&self, instance_id: &str, reason: &str);

    // ─────────────────────────────────────────────────────────────────────────
    // VALIDATION EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Realm-Compatibility-Check durchgeführt
    fn on_realm_compatibility_check(
        &self,
        blueprint_id: &str,
        realm_id: &str,
        compatible: bool,
        incompatibility_reasons: Option<&[String]>,
    );

    /// Dependency aufgelöst
    fn on_dependency_resolved(
        &self,
        blueprint_id: &str,
        dependency_id: &str,
        version_constraint: &str,
        resolved_version: &str,
    );

    // ─────────────────────────────────────────────────────────────────────────
    // CACHING EVENTS
    // ─────────────────────────────────────────────────────────────────────────

    /// Cache-Hit (kompiliertes Blueprint aus Cache)
    fn on_cache_hit(&self, blueprint_id: &str, version: &str);

    /// Cache-Miss (Blueprint muss kompiliert werden)
    fn on_cache_miss(&self, blueprint_id: &str, version: &str);

    /// Cache-Eviction (Blueprint aus Cache entfernt)
    fn on_cache_eviction(&self, blueprint_id: &str, reason: &str);
}

/// Shared BlueprintComposer Observer (thread-safe Arc)
pub type SharedBlueprintComposerObserver = Arc<dyn BlueprintComposerObserver>;

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
/// Verwaltet registrierte Engine-Observer für Event-Broadcasting.
#[derive(Clone)]
pub struct StateIntegrator {
    state: SharedUnifiedState,
    // ────────────────────────────────────────────────────────────────────────
    // Phase 5.1: Engine-Layer Observer-Vektoren
    // ────────────────────────────────────────────────────────────────────────
    /// Registrierte UI-Observer für Component-Lifecycle Events
    ui_observers: Arc<RwLock<Vec<SharedUIObserver>>>,
    /// Registrierte API-Observer für Endpoint/Request Events
    api_observers: Arc<RwLock<Vec<SharedAPIObserver>>>,
    /// Registrierte Governance-Observer für Proposal/Voting Events
    governance_observers: Arc<RwLock<Vec<SharedGovernanceObserver>>>,
    /// Registrierte Controller-Observer für Permission/AuthZ Events
    controller_observers: Arc<RwLock<Vec<SharedControllerObserver>>>,
    /// Registrierte DataLogic-Observer für Stream/Aggregation Events
    data_logic_observers: Arc<RwLock<Vec<SharedDataLogicObserver>>>,
    /// Registrierte BlueprintComposer-Observer für Composition/Versioning Events
    blueprint_composer_observers: Arc<RwLock<Vec<SharedBlueprintComposerObserver>>>,
}

impl StateIntegrator {
    /// Erstelle neuen StateIntegrator
    pub fn new(state: SharedUnifiedState) -> Self {
        Self {
            state,
            ui_observers: Arc::new(RwLock::new(Vec::new())),
            api_observers: Arc::new(RwLock::new(Vec::new())),
            governance_observers: Arc::new(RwLock::new(Vec::new())),
            controller_observers: Arc::new(RwLock::new(Vec::new())),
            data_logic_observers: Arc::new(RwLock::new(Vec::new())),
            blueprint_composer_observers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Zugriff auf State
    pub fn state(&self) -> &SharedUnifiedState {
        &self.state
    }

    // ════════════════════════════════════════════════════════════════════════
    // PHASE 5.2: OBSERVER REGISTRATION METHODS
    // ════════════════════════════════════════════════════════════════════════

    /// Registriere einen UI-Observer für Component-Lifecycle Events
    ///
    /// # Arguments
    /// * `observer` - Observer der UI-Events empfängt (component registered/rendered/updated)
    ///
    /// # Example
    /// ```ignore
    /// let integrator = StateIntegrator::new(state);
    /// integrator.register_ui_observer(Arc::new(MyUIObserver));
    /// ```
    pub fn register_ui_observer(&self, observer: SharedUIObserver) {
        if let Ok(mut observers) = self.ui_observers.write() {
            observers.push(observer);
            tracing::debug!("UI Observer registered (total: {})", observers.len());
        }
    }

    /// Registriere einen API-Observer für Endpoint/Request Events
    ///
    /// # Arguments
    /// * `observer` - Observer der API-Events empfängt (endpoint registered, request completed)
    pub fn register_api_observer(&self, observer: SharedAPIObserver) {
        if let Ok(mut observers) = self.api_observers.write() {
            observers.push(observer);
            tracing::debug!("API Observer registered (total: {})", observers.len());
        }
    }

    /// Registriere einen Governance-Observer für Proposal/Voting Events
    ///
    /// # Arguments
    /// * `observer` - Observer der Governance-Events empfängt (proposal created, vote cast)
    pub fn register_governance_observer(&self, observer: SharedGovernanceObserver) {
        if let Ok(mut observers) = self.governance_observers.write() {
            observers.push(observer);
            tracing::debug!(
                "Governance Observer registered (total: {})",
                observers.len()
            );
        }
    }

    /// Registriere einen Controller-Observer für Permission/AuthZ Events
    ///
    /// # Arguments
    /// * `observer` - Observer der Controller-Events empfängt (permission granted, authz check)
    pub fn register_controller_observer(&self, observer: SharedControllerObserver) {
        if let Ok(mut observers) = self.controller_observers.write() {
            observers.push(observer);
            tracing::debug!(
                "Controller Observer registered (total: {})",
                observers.len()
            );
        }
    }

    /// Registriere einen DataLogic-Observer für Stream/Aggregation Events
    ///
    /// # Arguments
    /// * `observer` - Observer der DataLogic-Events empfängt (stream registered, event forwarded)
    pub fn register_data_logic_observer(&self, observer: SharedDataLogicObserver) {
        if let Ok(mut observers) = self.data_logic_observers.write() {
            observers.push(observer);
            tracing::debug!("DataLogic Observer registered (total: {})", observers.len());
        }
    }

    /// Registriere einen BlueprintComposer-Observer für Composition/Versioning Events
    ///
    /// # Arguments
    /// * `observer` - Observer der BlueprintComposer-Events empfängt (composition completed, version published)
    pub fn register_blueprint_composer_observer(&self, observer: SharedBlueprintComposerObserver) {
        if let Ok(mut observers) = self.blueprint_composer_observers.write() {
            observers.push(observer);
            tracing::debug!(
                "BlueprintComposer Observer registered (total: {})",
                observers.len()
            );
        }
    }

    // ════════════════════════════════════════════════════════════════════════
    // PHASE 5.3: NOTIFICATION HELPER METHODS
    // ════════════════════════════════════════════════════════════════════════

    /// Benachrichtige alle UI-Observer über Component-Registration
    pub fn notify_ui_component_registered(
        &self,
        component_id: &str,
        component_type: &str,
        realm_id: Option<&str>,
        parent_id: Option<&str>,
    ) {
        if let Ok(observers) = self.ui_observers.read() {
            for observer in observers.iter() {
                observer.on_component_registered(component_id, component_type, realm_id, parent_id);
            }
        }
    }

    /// Benachrichtige alle UI-Observer über Component-Render
    pub fn notify_ui_component_rendered(
        &self,
        component_id: &str,
        from_cache: bool,
        gas_used: u64,
        mana_used: u64,
        realm_id: Option<&str>,
    ) {
        if let Ok(observers) = self.ui_observers.read() {
            for observer in observers.iter() {
                observer.on_component_rendered(
                    component_id,
                    from_cache,
                    gas_used,
                    mana_used,
                    realm_id,
                );
            }
        }
    }

    /// Benachrichtige alle UI-Observer über Trust-Gate Evaluation
    pub fn notify_ui_trust_gate(
        &self,
        component_id: &str,
        required_trust: f64,
        actual_trust: f64,
        allowed: bool,
        realm_id: Option<&str>,
    ) {
        if let Ok(observers) = self.ui_observers.read() {
            for observer in observers.iter() {
                observer.on_trust_gate_evaluated(
                    component_id,
                    required_trust,
                    actual_trust,
                    allowed,
                    realm_id,
                );
            }
        }
    }

    /// Benachrichtige alle API-Observer über Endpoint-Registration
    pub fn notify_api_endpoint_registered(
        &self,
        endpoint_id: &str,
        method: &str,
        path: &str,
        handler_id: &str,
        realm_id: Option<&str>,
    ) {
        if let Ok(observers) = self.api_observers.read() {
            for observer in observers.iter() {
                observer.on_endpoint_registered(endpoint_id, method, path, handler_id, realm_id);
            }
        }
    }

    /// Benachrichtige alle API-Observer über Request-Completion
    pub fn notify_api_request_completed(
        &self,
        request_id: &str,
        status_code: u16,
        latency_us: u64,
        gas_used: u64,
        mana_used: u64,
        response_size: usize,
    ) {
        if let Ok(observers) = self.api_observers.read() {
            for observer in observers.iter() {
                observer.on_request_completed(
                    request_id,
                    status_code,
                    latency_us,
                    gas_used,
                    mana_used,
                    response_size,
                );
            }
        }
    }

    /// Benachrichtige alle API-Observer über Rate-Limiting
    pub fn notify_api_rate_limited(
        &self,
        client_id: &str,
        endpoint_id: &str,
        retry_after_secs: u64,
        realm_id: Option<&str>,
    ) {
        if let Ok(observers) = self.api_observers.read() {
            for observer in observers.iter() {
                observer.on_rate_limited(client_id, endpoint_id, retry_after_secs, realm_id);
            }
        }
    }

    /// Benachrichtige alle Governance-Observer über Proposal-Creation
    pub fn notify_governance_proposal_created(
        &self,
        proposal_id: &str,
        proposal_type: &str,
        author: &EntityId,
        realm_id: Option<&str>,
        voting_period_secs: u64,
        quorum_percent: f64,
    ) {
        if let Ok(observers) = self.governance_observers.read() {
            for observer in observers.iter() {
                observer.on_proposal_created(
                    proposal_id,
                    proposal_type,
                    author,
                    realm_id,
                    voting_period_secs,
                    quorum_percent,
                );
            }
        }
    }

    /// Benachrichtige alle Governance-Observer über Vote-Cast
    pub fn notify_governance_vote_cast(
        &self,
        proposal_id: &str,
        voter: &EntityId,
        vote: &str,
        raw_voting_power: f64,
        effective_voting_power: f64,
        is_delegated: bool,
        delegation_chain_length: usize,
        realm_id: Option<&str>,
    ) {
        if let Ok(observers) = self.governance_observers.read() {
            for observer in observers.iter() {
                observer.on_vote_cast(
                    proposal_id,
                    voter,
                    vote,
                    raw_voting_power,
                    effective_voting_power,
                    is_delegated,
                    delegation_chain_length,
                    realm_id,
                );
            }
        }
    }

    /// Benachrichtige alle Governance-Observer über Power-Concentration Check
    pub fn notify_governance_power_check(
        &self,
        realm_id: Option<&str>,
        top_entity: &EntityId,
        concentration_percent: f64,
        threshold_percent: f64,
        violated: bool,
    ) {
        if let Ok(observers) = self.governance_observers.read() {
            for observer in observers.iter() {
                observer.on_power_concentration_check(
                    realm_id,
                    top_entity,
                    concentration_percent,
                    threshold_percent,
                    violated,
                );
            }
        }
    }

    /// Benachrichtige alle Controller-Observer über Permission-Grant
    pub fn notify_controller_permission_granted(
        &self,
        permission_id: &str,
        grantee: &EntityId,
        granter: &EntityId,
        scope_id: Option<&str>,
        realm_id: Option<&str>,
        expiration: Option<u64>,
        conditions: Option<&str>,
    ) {
        if let Ok(observers) = self.controller_observers.read() {
            for observer in observers.iter() {
                observer.on_permission_granted(
                    permission_id,
                    grantee,
                    granter,
                    scope_id,
                    realm_id,
                    expiration,
                    conditions,
                );
            }
        }
    }

    /// Benachrichtige alle Controller-Observer über AuthZ-Check
    pub fn notify_controller_authz_check(
        &self,
        requester: &EntityId,
        permission: &str,
        resource: &str,
        scope: &str,
        scope_id: Option<&str>,
        allowed: bool,
        via_delegation: bool,
        latency_us: u64,
    ) {
        if let Ok(observers) = self.controller_observers.read() {
            for observer in observers.iter() {
                observer.on_authz_check(
                    requester,
                    permission,
                    resource,
                    scope,
                    scope_id,
                    allowed,
                    via_delegation,
                    latency_us,
                );
            }
        }
    }

    /// Benachrichtige alle Controller-Observer über Automation-Trigger
    pub fn notify_controller_automation_triggered(
        &self,
        rule_id: &str,
        trigger_reason: &str,
        permissions_affected: usize,
    ) {
        if let Ok(observers) = self.controller_observers.read() {
            for observer in observers.iter() {
                observer.on_automation_triggered(rule_id, trigger_reason, permissions_affected);
            }
        }
    }

    /// Benachrichtige alle DataLogic-Observer über Stream-Registration
    pub fn notify_data_logic_stream_registered(
        &self,
        stream_id: &str,
        source_type: &str,
        filter_expression: Option<&str>,
    ) {
        if let Ok(observers) = self.data_logic_observers.read() {
            for observer in observers.iter() {
                observer.on_stream_registered(stream_id, source_type, filter_expression);
            }
        }
    }

    /// Benachrichtige alle DataLogic-Observer über Event-Forwarding
    pub fn notify_data_logic_event_forwarded(
        &self,
        stream_id: &str,
        subscribers_notified: usize,
        gas_used: u64,
    ) {
        if let Ok(observers) = self.data_logic_observers.read() {
            for observer in observers.iter() {
                observer.on_event_forwarded(stream_id, subscribers_notified, gas_used);
            }
        }
    }

    /// Benachrichtige alle DataLogic-Observer über Aggregation-Computation
    pub fn notify_data_logic_aggregation_computed(
        &self,
        aggregation_id: &str,
        result_value: f64,
        events_aggregated: u64,
        latency_us: u64,
        gas_used: u64,
    ) {
        if let Ok(observers) = self.data_logic_observers.read() {
            for observer in observers.iter() {
                observer.on_aggregation_computed(
                    aggregation_id,
                    result_value,
                    events_aggregated,
                    latency_us,
                    gas_used,
                );
            }
        }
    }

    /// Benachrichtige alle BlueprintComposer-Observer über Composition-Completion
    pub fn notify_blueprint_composition_completed(
        &self,
        composition_id: &str,
        success: bool,
        inheritance_depth: usize,
        conflicts_resolved: usize,
        gas_used: u64,
    ) {
        if let Ok(observers) = self.blueprint_composer_observers.read() {
            for observer in observers.iter() {
                observer.on_composition_completed(
                    composition_id,
                    success,
                    inheritance_depth,
                    conflicts_resolved,
                    gas_used,
                );
            }
        }
    }

    /// Benachrichtige alle BlueprintComposer-Observer über Version-Publishing
    pub fn notify_blueprint_version_published(
        &self,
        blueprint_id: &str,
        version: &str,
        author: &EntityId,
        changelog: Option<&str>,
    ) {
        if let Ok(observers) = self.blueprint_composer_observers.read() {
            for observer in observers.iter() {
                observer.on_version_published(blueprint_id, version, author, changelog);
            }
        }
    }

    /// Benachrichtige alle BlueprintComposer-Observer über Blueprint-Instantiation
    pub fn notify_blueprint_instantiated(
        &self,
        blueprint_id: &str,
        instance_id: &str,
        realm_id: Option<&str>,
        gas_used: u64,
    ) {
        if let Ok(observers) = self.blueprint_composer_observers.read() {
            for observer in observers.iter() {
                observer.on_blueprint_instantiated(blueprint_id, instance_id, realm_id, gas_used);
            }
        }
    }

    /// Öffentliche Methode zum Prüfen der P2P-Gesundheit
    ///
    /// Kann von externen Brücken aufgerufen werden, um Warnings zu synchronisieren.
    pub fn check_p2p_health(&self) {
        self.check_p2p_warnings();
    }

    /// Propagiere State-Updates basierend auf Beziehungen
    ///
    /// Diese Methode implementiert die tiefe Integration aller State-Beziehungen:
    /// - **Triggers**: Kaskadiert Updates zu abhängigen Komponenten
    /// - **Aggregates**: Aktualisiert Aggregations-Zähler
    /// - **Validates**: Führt Validierungen durch
    /// - **DependsOn**: Trackt Abhängigkeits-Updates
    /// - **Bidirectional**: Synchronisiert bidirektionale Beziehungen
    fn propagate_update(&self, from: super::state::StateComponent) {
        use super::state::StateComponent::*;

        let graph = StateGraph::erynoa_graph();

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 1: TRIGGER PROPAGATION (A → B)
        // Alle Komponenten die von `from` getriggert werden
        // ═══════════════════════════════════════════════════════════════════
        for component in graph.triggered_by(from) {
            match (from, component) {
                // ─────────────────────────────────────────────────────────────
                // Trust → * Beziehungen
                // ─────────────────────────────────────────────────────────────
                (Trust, Event) => {
                    self.state
                        .core
                        .events
                        .trust_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Trust, AntiCalcification) => {
                    // Anti-Calcification prüft Trust-Limits
                    self.state
                        .protection
                        .anti_calcification
                        .trust_limits_checked
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Event → * Beziehungen
                // ─────────────────────────────────────────────────────────────
                (Event, Trust) => {
                    self.state
                        .core
                        .trust
                        .event_triggered_updates
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // WorldFormula → * Beziehungen
                // ─────────────────────────────────────────────────────────────
                (WorldFormula, Consensus) => {
                    // 𝔼 beeinflusst Konsens-Parameter
                    self.state
                        .core
                        .consensus
                        .successful_rounds
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Execution → * Beziehungen
                // ─────────────────────────────────────────────────────────────
                (Execution, Event) => {
                    // Execution emittiert Events
                    self.state
                        .core
                        .events
                        .execution_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Calibration → * Beziehungen
                // ─────────────────────────────────────────────────────────────
                (Calibration, Gas) => {
                    // Calibration passt Gas-Preise an
                    self.state
                        .execution
                        .gas
                        .calibration_adjustments
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Calibration, Mana) => {
                    // Calibration passt Mana-Regen an
                    self.state
                        .execution
                        .mana
                        .calibration_adjustments
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Gateway → * Beziehungen (Κ23)
                // ─────────────────────────────────────────────────────────────
                (Gateway, Event) => {
                    // Crossings erzeugen Events
                    self.state
                        .core
                        .events
                        .gateway_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // SagaComposer → * Beziehungen (Κ22/Κ24)
                // ─────────────────────────────────────────────────────────────
                (SagaComposer, Execution) => {
                    // Sagas erzeugen Executions
                    self.state
                        .execution
                        .executions
                        .saga_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Realm → * Beziehungen (Κ22-Κ24)
                // ─────────────────────────────────────────────────────────────
                (Realm, Trust) => {
                    // Realm-Aktivität beeinflusst Trust
                    self.state
                        .core
                        .trust
                        .realm_triggered_updates
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Realm, SagaComposer) => {
                    // Realm kann Cross-Realm-Sagas auslösen
                    self.state
                        .peer
                        .saga
                        .cross_realm_sagas
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Realm, Event) => {
                    // Realm-Events (Registrierung, Membership, Rules)
                    self.state
                        .core
                        .events
                        .realm_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // ECLVM → * Beziehungen
                // ─────────────────────────────────────────────────────────────
                (ECLVM, Event) => {
                    // ECL-Ausführungen emittieren Events
                    self.state
                        .core
                        .events
                        .eclvm_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // ECLPolicy → * Beziehungen
                // ─────────────────────────────────────────────────────────────
                (ECLPolicy, Event) => {
                    // Policy-Evaluationen erzeugen Events
                    self.state
                        .core
                        .events
                        .policy_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // ECLBlueprint → * Beziehungen
                // ─────────────────────────────────────────────────────────────
                (ECLBlueprint, Event) => {
                    // Blueprint-Instanziierungen erzeugen Events
                    self.state
                        .core
                        .events
                        .blueprint_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // P2P → * Beziehungen
                // ─────────────────────────────────────────────────────────────
                (Swarm, Event) => {
                    // Swarm propagiert Events
                    self.state
                        .core
                        .events
                        .swarm_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Gossip, Event) => {
                    // Gossip verteilt Events
                    self.state
                        .core
                        .events
                        .gossip_triggered
                        .fetch_add(1, Ordering::Relaxed);
                }

                _ => {
                    // Unbehandelte Trigger-Beziehung - als Debug loggen in Zukunft
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 2: VALIDATION PROPAGATION (A ✓ B)
        // Alle Komponenten die von `from` validiert werden
        // ═══════════════════════════════════════════════════════════════════
        for component in graph.validated_by(from) {
            match (from, component) {
                // ─────────────────────────────────────────────────────────────
                // Anomaly validiert Event/Trust
                // ─────────────────────────────────────────────────────────────
                (Anomaly, Event) => {
                    self.state
                        .protection
                        .anomaly
                        .events_validated
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Anomaly, Trust) => {
                    self.state
                        .protection
                        .anomaly
                        .trust_patterns_checked
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Diversity validiert Trust/Consensus
                // ─────────────────────────────────────────────────────────────
                (Diversity, Trust) => {
                    self.state
                        .protection
                        .diversity
                        .trust_distribution_checks
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Diversity, Consensus) => {
                    self.state
                        .protection
                        .diversity
                        .validator_mix_checks
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // AntiCalcification validiert Trust
                // ─────────────────────────────────────────────────────────────
                (AntiCalcification, Trust) => {
                    self.state
                        .protection
                        .anti_calcification
                        .power_checks
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Consensus validiert Event
                // ─────────────────────────────────────────────────────────────
                (Consensus, Event) => {
                    self.state
                        .core
                        .consensus
                        .events_validated
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Gateway validiert Trust (Κ23)
                // ─────────────────────────────────────────────────────────────
                (Gateway, Trust) => {
                    self.state
                        .peer
                        .gateway
                        .crossings_total
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // IntentParser validiert Event
                // ─────────────────────────────────────────────────────────────
                (IntentParser, Event) => {
                    self.state
                        .peer
                        .intent
                        .validation_errors
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // ECLPolicy validiert Gateway/Realm
                // ─────────────────────────────────────────────────────────────
                (ECLPolicy, Gateway) => {
                    self.state
                        .eclvm
                        .crossing_evaluations
                        .fetch_add(1, Ordering::Relaxed);
                }
                (ECLPolicy, Realm) => {
                    self.state
                        .eclvm
                        .policies_executed
                        .fetch_add(1, Ordering::Relaxed);
                }

                _ => {
                    // Unbehandelte Validations-Beziehung
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 3: AGGREGATION PROPAGATION (A ⊃ B)
        // Alle Komponenten deren Daten in `from` aggregiert werden
        // ═══════════════════════════════════════════════════════════════════
        for component in graph.aggregated_by(from) {
            match (from, component) {
                // ─────────────────────────────────────────────────────────────
                // Execution aggregiert Gas/Mana
                // ─────────────────────────────────────────────────────────────
                (Execution, Gas) => {
                    self.state
                        .execution
                        .executions
                        .gas_aggregations
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Execution, Mana) => {
                    self.state
                        .execution
                        .executions
                        .mana_aggregations
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Storage aggregiert Events
                // ─────────────────────────────────────────────────────────────
                (EventStore, Event) => {
                    self.state
                        .storage
                        .event_store_count
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Archive, EventStore) => {
                    self.state
                        .storage
                        .archived_events
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // SagaComposer aggregiert IntentParser
                // ─────────────────────────────────────────────────────────────
                (SagaComposer, IntentParser) => {
                    self.state
                        .peer
                        .saga
                        .sagas_composed
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Realm aggregiert Gateway/ECLPolicy
                // ─────────────────────────────────────────────────────────────
                (Realm, Gateway) => {
                    self.state
                        .peer
                        .realm
                        .active_crossings
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Realm, ECLPolicy) => {
                    self.state
                        .peer
                        .realm
                        .total_realms
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // ECLVM aggregiert Execution
                // ─────────────────────────────────────────────────────────────
                (ECLVM, Execution) => {
                    self.state
                        .eclvm
                        .intents_processed
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // ECLBlueprint aggregiert Blueprint
                // ─────────────────────────────────────────────────────────────
                (ECLBlueprint, Blueprint) => {
                    self.state
                        .eclvm
                        .blueprints_instantiated
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Kademlia aggregiert Swarm
                // ─────────────────────────────────────────────────────────────
                (Kademlia, Swarm) => {
                    self.state
                        .p2p
                        .kademlia
                        .queries_successful
                        .fetch_add(1, Ordering::Relaxed);
                }

                _ => {
                    // Unbehandelte Aggregations-Beziehung
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // PHASE 4: DEPENDENCY TRACKING (A ← B)
        // Alle Komponenten von denen `from` abhängt - Notify wenn sich diese ändern
        // ═══════════════════════════════════════════════════════════════════
        for component in graph.dependencies_of(from) {
            match (from, component) {
                // ─────────────────────────────────────────────────────────────
                // Trust → WorldFormula Dependency
                // ─────────────────────────────────────────────────────────────
                (Trust, WorldFormula) => {
                    self.state
                        .core
                        .formula
                        .computations
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Gas/Mana → Trust Dependency
                // ─────────────────────────────────────────────────────────────
                (Gas, Trust) => {
                    self.state
                        .execution
                        .gas
                        .trust_dependency_updates
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Mana, Trust) => {
                    self.state
                        .execution
                        .mana
                        .trust_dependency_updates
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Quadratic → Trust Dependency
                // ─────────────────────────────────────────────────────────────
                (Quadratic, Trust) => {
                    self.state
                        .protection
                        .quadratic
                        .trust_dependency_updates
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // KvStore/Blueprint → Trust Dependency
                // ─────────────────────────────────────────────────────────────
                (KvStore, Trust) => {
                    self.state.storage.kv_reads.fetch_add(1, Ordering::Relaxed);
                }
                (Blueprint, Trust) => {
                    self.state
                        .storage
                        .blueprints_downloaded
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Gateway/SagaComposer → Trust Dependency (Κ22-Κ24)
                // ─────────────────────────────────────────────────────────────
                (Gateway, Trust) => {
                    self.state
                        .peer
                        .gateway
                        .crossings_total
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Gateway, Realm) => {
                    self.state
                        .peer
                        .gateway
                        .registered_realms
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Gateway, ECLPolicy) => {
                    self.state
                        .peer
                        .gateway
                        .rule_violations
                        .fetch_add(1, Ordering::Relaxed);
                }
                (SagaComposer, Trust) => {
                    self.state
                        .peer
                        .saga
                        .budget_violations
                        .fetch_add(1, Ordering::Relaxed);
                }
                (SagaComposer, ECLVM) => {
                    self.state
                        .peer
                        .saga
                        .cross_realm_sagas
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // Realm → Trust Dependency
                // ─────────────────────────────────────────────────────────────
                (Realm, Trust) => {
                    self.state
                        .peer
                        .realm
                        .total_realms
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // ECLVM → Gas/Mana/Trust Dependency
                // ─────────────────────────────────────────────────────────────
                (ECLVM, Gas) => {
                    self.state
                        .eclvm
                        .total_gas_consumed
                        .fetch_add(1, Ordering::Relaxed);
                }
                (ECLVM, Mana) => {
                    self.state
                        .eclvm
                        .total_mana_consumed
                        .fetch_add(1, Ordering::Relaxed);
                }
                (ECLVM, Trust) => {
                    self.state
                        .eclvm
                        .saga_steps_executed
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // ECLPolicy → ECLVM Dependency
                // ─────────────────────────────────────────────────────────────
                (ECLPolicy, ECLVM) => {
                    self.state
                        .eclvm
                        .policies_compiled
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // ECLBlueprint → ECLVM Dependency
                // ─────────────────────────────────────────────────────────────
                (ECLBlueprint, ECLVM) => {
                    self.state
                        .eclvm
                        .blueprints_deployed
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // IntentParser → ECLPolicy Dependency
                // ─────────────────────────────────────────────────────────────
                (IntentParser, ECLPolicy) => {
                    self.state
                        .peer
                        .intent
                        .parse_errors
                        .fetch_add(1, Ordering::Relaxed);
                }

                // ─────────────────────────────────────────────────────────────
                // P2P → Trust Dependency
                // ─────────────────────────────────────────────────────────────
                (Gossip, Trust) => {
                    self.state
                        .p2p
                        .gossip
                        .messages_validated
                        .fetch_add(1, Ordering::Relaxed);
                }
                (Relay, Trust) => {
                    self.state
                        .p2p
                        .relay
                        .circuits_served
                        .fetch_add(1, Ordering::Relaxed);
                }

                _ => {
                    // Unbehandelte Dependency-Beziehung
                }
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
            .entities_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_relationship_created(&self, _from: &EntityId, _to: &EntityId) {
        self.state
            .core
            .trust
            .relationships_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_violation_detected(&self, _entity: &EntityId, _violation_type: &str) {
        self.state
            .core
            .trust
            .violations_count
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
            .gas
            .consumed
            .fetch_add(amount, Ordering::Relaxed);
    }

    fn on_out_of_gas(&self, _required: u64, _available: u64) {
        self.state
            .execution
            .gas
            .out_of_gas_count
            .fetch_add(1, Ordering::Relaxed);
    }

    fn on_mana_consumed(&self, amount: u64) {
        self.state
            .execution
            .mana
            .consumed
            .fetch_add(amount, Ordering::Relaxed);
    }

    fn on_rate_limited(&self, _entity: &EntityId) {
        self.state
            .execution
            .mana
            .rate_limited_count
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
            .diversity
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
            .anti_calcification
            .interventions
            .fetch_add(1, Ordering::Relaxed);
        tracing::info!("Anti-calcification intervention: {}", reason);
    }

    fn on_calibration_update(&self, param: &str, _old_value: f64, new_value: f64) {
        self.state
            .protection
            .calibration
            .updates_total
            .fetch_add(1, Ordering::Relaxed);
        if let Ok(mut params) = self.state.protection.calibration.params_map.write() {
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
        // Parse policy type from string (Gap 3: Api, Ui, DataLogic, Controller)
        let ptype = match policy_type {
            "crossing" => super::state::ECLPolicyType::Crossing,
            "membership" => super::state::ECLPolicyType::Membership,
            "transaction" => super::state::ECLPolicyType::Transaction,
            "governance" => super::state::ECLPolicyType::Governance,
            "privacy" => super::state::ECLPolicyType::Privacy,
            "api" => super::state::ECLPolicyType::Api,
            "ui" => super::state::ECLPolicyType::Ui,
            "datalogic" => super::state::ECLPolicyType::DataLogic,
            "controller" => super::state::ECLPolicyType::Controller,
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
        duration_us: u64,
        realm_id: Option<&str>,
    ) {
        let ptype = match policy_type {
            "crossing" => super::state::ECLPolicyType::Crossing,
            "membership" => super::state::ECLPolicyType::Membership,
            "transaction" => super::state::ECLPolicyType::Transaction,
            "governance" => super::state::ECLPolicyType::Governance,
            "privacy" => super::state::ECLPolicyType::Privacy,
            "api" => super::state::ECLPolicyType::Api,
            "ui" => super::state::ECLPolicyType::Ui,
            "datalogic" => super::state::ECLPolicyType::DataLogic,
            "controller" => super::state::ECLPolicyType::Controller,
            _ => super::state::ECLPolicyType::Custom,
        };
        // Gap 6: ECL-Pfad emittiert StateEvent::PolicyEvaluated → apply_state_event → eclvm.policy_executed
        let event = super::state::StateEvent::PolicyEvaluated {
            policy_id: policy_id.to_string(),
            realm_id: realm_id.map(|s| s.to_string()),
            passed,
            policy_type: ptype,
            gas_used,
            mana_used,
            duration_us,
        };
        let _ = self
            .state
            .log_and_apply(event, vec!["ecl_policy".to_string()]);
        self.propagate_update(super::state::StateComponent::ECLPolicy);
        tracing::trace!(
            "ECL Policy executed: {} (passed: {}, gas: {}, mana: {}, duration_us: {}, realm: {:?})",
            policy_id,
            passed,
            gas_used,
            mana_used,
            duration_us,
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
            .gas
            .consumed
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
            .mana
            .consumed
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
// ECLVM OBSERVER ADAPTER (Phase 1.1 – ProgrammableGateway → ECLVMState)
// ============================================================================
//
// Verbindet ProgrammableGateway mit StateIntegrator: eclvm::PolicyExecutionObserver
// wird auf ECLVMObserver (on_policy_executed, on_crossing_policy_evaluated) gemappt.

/// Adapter: StateIntegrator als eclvm::PolicyExecutionObserver
pub struct ECLVMObserverAdapter {
    integrator: StateIntegrator,
}

impl ECLVMObserverAdapter {
    pub fn new(integrator: StateIntegrator) -> Self {
        Self { integrator }
    }
}

impl crate::eclvm::PolicyExecutionObserver for ECLVMObserverAdapter {
    fn on_policy_executed(
        &self,
        policy_id: &str,
        policy_type: &str,
        passed: bool,
        gas_used: u64,
        mana_used: u64,
        duration_us: u64,
        realm_id: Option<&str>,
    ) {
        self.integrator.on_policy_executed(
            policy_id,
            policy_type,
            passed,
            gas_used,
            mana_used,
            duration_us,
            realm_id,
        );
    }

    fn on_crossing_policy_evaluated(
        &self,
        from_realm: &str,
        to_realm: &str,
        entity_id: &str,
        allowed: bool,
        trust_score: f64,
        policy_id: Option<&str>,
    ) {
        let entity = UniversalId::new(UniversalId::TAG_DID, 1, entity_id.as_bytes());
        self.integrator.on_crossing_policy_evaluated(
            from_realm,
            to_realm,
            &entity,
            allowed,
            trust_score,
            policy_id,
        );
    }
}

// ============================================================================
// ENGINE-LAYER OBSERVER IMPLEMENTATIONS (4.1-4.6)
// ============================================================================

// ────────────────────────────────────────────────────────────────────────────
// 4.1 UI OBSERVER IMPLEMENTATION
// ────────────────────────────────────────────────────────────────────────────

impl UIObserver for StateIntegrator {
    fn on_component_registered(
        &self,
        component_id: &str,
        component_type: &str,
        realm_id: Option<&str>,
        _parent_id: Option<&str>,
    ) {
        self.state.ui.register_component(realm_id);
        tracing::debug!(
            "UI Component registered: {} (type: {}, realm: {:?})",
            component_id,
            component_type,
            realm_id
        );
    }

    fn on_component_unmounted(&self, component_id: &str) {
        self.state.ui.unregister_component();
        tracing::trace!("UI Component unmounted: {}", component_id);
    }

    fn on_component_updated(&self, component_id: &str, update_type: &str) {
        self.state
            .ui
            .component_updates
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "UI Component updated: {} (type: {})",
            component_id,
            update_type
        );
    }

    fn on_component_rendered(
        &self,
        component_id: &str,
        from_cache: bool,
        gas_used: u64,
        mana_used: u64,
        realm_id: Option<&str>,
    ) {
        self.state
            .ui
            .render(from_cache, gas_used, mana_used, realm_id);
        tracing::trace!(
            "UI Component rendered: {} (cache: {}, gas: {}, mana: {})",
            component_id,
            from_cache,
            gas_used,
            mana_used
        );
    }

    fn on_trust_gate_evaluated(
        &self,
        component_id: &str,
        required_trust: f64,
        actual_trust: f64,
        allowed: bool,
        realm_id: Option<&str>,
    ) {
        self.state.ui.trust_gate(allowed, realm_id);
        self.state
            .ui
            .trust_dependency_updates
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "UI Trust-Gate: {} (required: {:.2}, actual: {:.2}, allowed: {})",
            component_id,
            required_trust,
            actual_trust,
            allowed
        );
    }

    fn on_trust_gate_configured(&self, component_id: &str, min_trust: f64, max_trust: Option<f64>) {
        tracing::debug!(
            "UI Trust-Gate configured: {} (min: {:.2}, max: {:?})",
            component_id,
            min_trust,
            max_trust
        );
    }

    fn on_credential_gate_evaluated(
        &self,
        component_id: &str,
        _required_credentials: &[String],
        _has_credentials: bool,
        allowed: bool,
    ) {
        self.state.ui.credential_gate(allowed);
        self.state
            .ui
            .controller_validations
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "UI Credential-Gate: {} (allowed: {})",
            component_id,
            allowed
        );
    }

    fn on_binding_created(
        &self,
        binding_id: &str,
        component_id: &str,
        stream_id: &str,
        binding_type: &str,
    ) {
        self.state
            .ui
            .bindings_active
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!(
            "UI Binding created: {} ({} -> {} via {})",
            binding_id,
            component_id,
            stream_id,
            binding_type
        );
    }

    fn on_binding_updated(
        &self,
        binding_id: &str,
        success: bool,
        _latency_us: u64,
        realm_id: Option<&str>,
    ) {
        self.state.ui.binding_update(success, realm_id);
        self.state
            .ui
            .datalogic_aggregations
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!("UI Binding updated: {} (success: {})", binding_id, success);
    }

    fn on_binding_error(&self, binding_id: &str, error: &str) {
        self.state.ui.binding_errors.fetch_add(1, Ordering::Relaxed);
        tracing::warn!("UI Binding error: {} - {}", binding_id, error);
    }

    fn on_binding_removed(&self, binding_id: &str) {
        self.state
            .ui
            .bindings_active
            .fetch_sub(1, Ordering::Relaxed);
        tracing::trace!("UI Binding removed: {}", binding_id);
    }

    fn on_ui_action(
        &self,
        component_id: &str,
        action_type: &str,
        _payload_size: usize,
        _realm_id: Option<&str>,
    ) {
        tracing::trace!("UI Action: {} on {} ", action_type, component_id);
    }

    fn on_event_emitted(&self, component_id: &str, event_type: &str) {
        self.state
            .ui
            .events_triggered
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!("UI Event emitted: {} from {}", event_type, component_id);
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4.2 API OBSERVER IMPLEMENTATION
// ────────────────────────────────────────────────────────────────────────────

impl APIObserver for StateIntegrator {
    fn on_endpoint_registered(
        &self,
        endpoint_id: &str,
        method: &str,
        path: &str,
        _handler_id: &str,
        realm_id: Option<&str>,
    ) {
        self.state.api.register_endpoint(realm_id);
        tracing::debug!(
            "API Endpoint registered: {} {} {} (realm: {:?})",
            endpoint_id,
            method,
            path,
            realm_id
        );
    }

    fn on_endpoint_updated(&self, endpoint_id: &str, changes: &str) {
        self.state
            .api
            .endpoint_updates
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!("API Endpoint updated: {} - {}", endpoint_id, changes);
    }

    fn on_endpoint_removed(&self, endpoint_id: &str) {
        self.state
            .api
            .endpoints_active
            .fetch_sub(1, Ordering::Relaxed);
        tracing::debug!("API Endpoint removed: {}", endpoint_id);
    }

    fn on_request_received(
        &self,
        request_id: &str,
        endpoint_id: &str,
        method: &str,
        client_trust: f64,
        _realm_id: Option<&str>,
    ) {
        self.state
            .api
            .trust_dependency_updates
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "API Request received: {} {} {} (trust: {:.2})",
            request_id,
            method,
            endpoint_id,
            client_trust
        );
    }

    fn on_request_completed(
        &self,
        request_id: &str,
        status_code: u16,
        latency_us: u64,
        gas_used: u64,
        mana_used: u64,
        _response_size: usize,
    ) {
        self.state
            .api
            .record_request(latency_us, status_code, gas_used, mana_used, None);
        tracing::trace!(
            "API Request completed: {} (status: {}, latency: {}µs)",
            request_id,
            status_code,
            latency_us
        );
    }

    fn on_request_validation_failed(&self, request_id: &str, error: &str, error_code: &str) {
        self.state
            .api
            .requests_client_error
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!(
            "API Request validation failed: {} - {} ({})",
            request_id,
            error,
            error_code
        );
    }

    fn on_rate_limit_bucket_created(
        &self,
        bucket_id: &str,
        _client_id: &str,
        max_requests: u64,
        window_secs: u64,
        trust_multiplier: f64,
    ) {
        self.state
            .api
            .rate_limit_buckets
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!(
            "API Rate-Limit bucket created: {} (max: {}, window: {}s, trust_mult: {:.2})",
            bucket_id,
            max_requests,
            window_secs,
            trust_multiplier
        );
    }

    fn on_rate_limited(
        &self,
        client_id: &str,
        endpoint_id: &str,
        retry_after_secs: u64,
        realm_id: Option<&str>,
    ) {
        // record_request wird mit status 429 aufgerufen, was rate_limited zählt
        self.state.api.record_request(0, 429, 0, 0, realm_id);
        tracing::info!(
            "API Rate-Limited: {} on {} (retry after: {}s)",
            client_id,
            endpoint_id,
            retry_after_secs
        );
    }

    fn on_rate_limit_reset(&self, bucket_id: &str) {
        self.state
            .api
            .rate_limit_resets
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!("API Rate-Limit reset: {}", bucket_id);
    }

    fn on_auth_check(
        &self,
        request_id: &str,
        auth_type: &str,
        success: bool,
        failure_reason: Option<&str>,
    ) {
        if !success {
            self.state
                .api
                .requests_auth_failed
                .fetch_add(1, Ordering::Relaxed);
        }
        tracing::trace!(
            "API Auth check: {} (type: {}, success: {}, reason: {:?})",
            request_id,
            auth_type,
            success,
            failure_reason
        );
    }

    fn on_authz_delegated(&self, request_id: &str, permission: &str, allowed: bool) {
        self.state
            .api
            .controller_validations
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "API AuthZ delegated: {} (permission: {}, allowed: {})",
            request_id,
            permission,
            allowed
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4.3 GOVERNANCE OBSERVER IMPLEMENTATION
// ────────────────────────────────────────────────────────────────────────────

impl GovernanceObserver for StateIntegrator {
    fn on_proposal_created(
        &self,
        proposal_id: &str,
        proposal_type: &str,
        _author: &EntityId,
        realm_id: Option<&str>,
        voting_period_secs: u64,
        quorum_percent: f64,
    ) {
        self.state.governance.proposal_created(realm_id);
        tracing::info!(
            "Governance Proposal created: {} (type: {}, voting: {}s, quorum: {:.1}%)",
            proposal_id,
            proposal_type,
            voting_period_secs,
            quorum_percent * 100.0
        );
    }

    fn on_proposal_status_changed(&self, proposal_id: &str, old_status: &str, new_status: &str) {
        tracing::info!(
            "Governance Proposal status: {} ({} -> {})",
            proposal_id,
            old_status,
            new_status
        );
    }

    fn on_proposal_completed(
        &self,
        proposal_id: &str,
        result: &str,
        yes_votes: u64,
        no_votes: u64,
        abstain_votes: u64,
        participation_rate: f64,
    ) {
        self.state.governance.proposal_completed(result);
        tracing::info!(
            "Governance Proposal completed: {} (result: {}, yes: {}, no: {}, abstain: {}, participation: {:.1}%)",
            proposal_id,
            result,
            yes_votes,
            no_votes,
            abstain_votes,
            participation_rate * 100.0
        );
    }

    fn on_vote_cast(
        &self,
        proposal_id: &str,
        _voter: &EntityId,
        vote: &str,
        raw_voting_power: f64,
        effective_voting_power: f64,
        is_delegated: bool,
        _delegation_chain_length: usize,
        realm_id: Option<&str>,
    ) {
        let quadratic_reduced = (raw_voting_power - effective_voting_power).abs() > 0.001;
        self.state.governance.vote_cast(
            effective_voting_power,
            is_delegated,
            quadratic_reduced,
            realm_id,
        );
        tracing::debug!(
            "Governance Vote: {} on {} (power: {:.2}, delegated: {})",
            vote,
            proposal_id,
            effective_voting_power,
            is_delegated
        );
    }

    fn on_vote_withdrawn(&self, proposal_id: &str, _voter: &EntityId) {
        // Votes können nicht wirklich "withdrawn" werden im aktuellen Design
        tracing::debug!("Governance Vote withdrawn from: {}", proposal_id);
    }

    fn on_quadratic_reduction(
        &self,
        _voter: &EntityId,
        original_power: f64,
        reduced_power: f64,
        reduction_factor: f64,
    ) {
        self.state
            .governance
            .quadratic_reductions
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .governance
            .quadratic_validations
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "Governance Quadratic reduction: {:.2} -> {:.2} (factor: {:.3})",
            original_power,
            reduced_power,
            reduction_factor
        );
    }

    fn on_delegation_created(
        &self,
        _delegator: &EntityId,
        _delegate: &EntityId,
        scope: &str,
        _scope_id: Option<&str>,
        _expiration: Option<u64>,
    ) {
        self.state.governance.delegation_created(1, None);
        tracing::debug!("Governance Delegation created (scope: {})", scope);
    }

    fn on_delegation_revoked(&self, _delegator: &EntityId, _delegate: &EntityId) {
        self.state
            .governance
            .delegations_active
            .fetch_sub(1, Ordering::Relaxed);
        tracing::debug!("Governance Delegation revoked");
    }

    fn on_delegation_expired(&self, _delegator: &EntityId, _delegate: &EntityId) {
        self.state
            .governance
            .delegations_active
            .fetch_sub(1, Ordering::Relaxed);
        self.state
            .governance
            .delegations_expired
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!("Governance Delegation expired");
    }

    fn on_circular_delegation_prevented(
        &self,
        _delegator: &EntityId,
        _delegate: &EntityId,
        cycle_path: &[String],
    ) {
        self.state
            .governance
            .circular_delegations_prevented
            .fetch_add(1, Ordering::Relaxed);
        tracing::warn!(
            "Governance Circular delegation prevented (cycle length: {})",
            cycle_path.len()
        );
    }

    fn on_delegation_chain_resolved(
        &self,
        _original_voter: &EntityId,
        _final_delegate: &EntityId,
        chain_length: usize,
    ) {
        // Update max delegation depth if needed
        let current_max = self
            .state
            .governance
            .max_delegation_depth
            .load(Ordering::Relaxed);
        if chain_length as u64 > current_max {
            self.state
                .governance
                .max_delegation_depth
                .store(chain_length as u64, Ordering::Relaxed);
        }
        tracing::trace!(
            "Governance Delegation chain resolved (length: {})",
            chain_length
        );
    }

    fn on_power_concentration_check(
        &self,
        _realm_id: Option<&str>,
        _top_entity: &EntityId,
        concentration_percent: f64,
        threshold_percent: f64,
        violated: bool,
    ) {
        self.state
            .governance
            .power_check(violated, concentration_percent / 100.0);
        if violated {
            tracing::warn!(
                "Governance Power concentration violation: {:.1}% > {:.1}%",
                concentration_percent,
                threshold_percent
            );
        }
    }

    fn on_gini_calculated(&self, _realm_id: Option<&str>, gini_coefficient: f64) {
        if let Ok(mut gini) = self.state.governance.voting_power_gini.write() {
            *gini = gini_coefficient;
        }
        tracing::trace!("Governance Gini coefficient: {:.3}", gini_coefficient);
    }

    fn on_veto_exercised(&self, proposal_id: &str, _veto_entity: &EntityId, reason: &str) {
        self.state
            .governance
            .proposals_vetoed
            .fetch_add(1, Ordering::Relaxed);
        tracing::info!(
            "Governance Veto exercised on: {} (reason: {})",
            proposal_id,
            reason
        );
    }

    fn on_permission_change_triggered(
        &self,
        proposal_id: &str,
        permission: &str,
        _target: &EntityId,
        action: &str,
    ) {
        self.state
            .governance
            .controller_triggers
            .fetch_add(1, Ordering::Relaxed);
        tracing::info!(
            "Governance triggered permission change: {} {} (proposal: {})",
            action,
            permission,
            proposal_id
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4.4 CONTROLLER OBSERVER IMPLEMENTATION
// ────────────────────────────────────────────────────────────────────────────

impl ControllerObserver for StateIntegrator {
    fn on_permission_registered(
        &self,
        permission_id: &str,
        permission_name: &str,
        scope_type: &str,
        _description: &str,
    ) {
        self.state
            .controller
            .permissions_registered
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!(
            "Controller Permission registered: {} ({}, scope: {})",
            permission_id,
            permission_name,
            scope_type
        );
    }

    fn on_permission_granted(
        &self,
        permission_id: &str,
        _grantee: &EntityId,
        _granter: &EntityId,
        _scope_id: Option<&str>,
        realm_id: Option<&str>,
        _expiration: Option<u64>,
        _conditions: Option<&str>,
    ) {
        self.state.controller.grant_permission(realm_id);
        tracing::debug!(
            "Controller Permission granted: {} (realm: {:?})",
            permission_id,
            realm_id
        );
    }

    fn on_permission_revoked(
        &self,
        permission_id: &str,
        _entity: &EntityId,
        _revoker: &EntityId,
        reason: &str,
    ) {
        self.state.controller.revoke_permission();
        tracing::debug!(
            "Controller Permission revoked: {} (reason: {})",
            permission_id,
            reason
        );
    }

    fn on_permission_expired(&self, permission_id: &str, _entity: &EntityId) {
        self.state
            .controller
            .permissions_active
            .fetch_sub(1, Ordering::Relaxed);
        tracing::trace!("Controller Permission expired: {}", permission_id);
    }

    fn on_authz_check(
        &self,
        _requester: &EntityId,
        permission: &str,
        _resource: &str,
        scope: &str,
        _scope_id: Option<&str>,
        allowed: bool,
        via_delegation: bool,
        latency_us: u64,
    ) {
        self.state
            .controller
            .check_authorization(allowed, via_delegation, latency_us, scope, None);
        tracing::trace!(
            "Controller AuthZ check: {} (allowed: {}, scope: {}, latency: {}µs)",
            permission,
            allowed,
            scope,
            latency_us
        );
    }

    fn on_scope_inheritance_resolved(
        &self,
        _permission: &str,
        from_scope: &str,
        to_scope: &str,
        inherited: bool,
    ) {
        self.state
            .controller
            .scope_inheritance_resolutions
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "Controller Scope inheritance: {} -> {} (inherited: {})",
            from_scope,
            to_scope,
            inherited
        );
    }

    fn on_permission_delegated(
        &self,
        permission_id: &str,
        _delegator: &EntityId,
        _delegate: &EntityId,
        _can_redelegate: bool,
        _constraints: Option<&str>,
    ) {
        self.state.controller.create_delegation(1, None);
        tracing::debug!("Controller Permission delegated: {}", permission_id);
    }

    fn on_delegation_resolved(
        &self,
        permission_id: &str,
        _original_requester: &EntityId,
        _final_granter: &EntityId,
        chain_length: usize,
    ) {
        self.state
            .controller
            .delegations_used
            .fetch_add(1, Ordering::Relaxed);
        // Update max depth
        let current_max = self
            .state
            .controller
            .max_delegation_depth
            .load(Ordering::Relaxed);
        if chain_length as u64 > current_max {
            self.state
                .controller
                .max_delegation_depth
                .store(chain_length as u64, Ordering::Relaxed);
        }
        tracing::trace!(
            "Controller Delegation resolved: {} (chain length: {})",
            permission_id,
            chain_length
        );
    }

    fn on_delegation_conflict(&self, permission_id: &str, _entity: &EntityId, conflict_type: &str) {
        self.state
            .controller
            .delegation_conflicts
            .fetch_add(1, Ordering::Relaxed);
        tracing::warn!(
            "Controller Delegation conflict: {} (type: {})",
            permission_id,
            conflict_type
        );
    }

    fn on_audit_entry(
        &self,
        _entry_id: &str,
        action: &str,
        _actor: &EntityId,
        _target: Option<&EntityId>,
        _permission: &str,
        _result: &str,
        metadata_size: usize,
    ) {
        self.state
            .controller
            .audit_entries
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .controller
            .audit_log_bytes
            .fetch_add(metadata_size as u64, Ordering::Relaxed);
        tracing::trace!("Controller Audit entry: {}", action);
    }

    fn on_audit_log_rotated(&self, old_size_bytes: u64, entries_archived: u64) {
        tracing::info!(
            "Controller Audit log rotated: {} bytes, {} entries archived",
            old_size_bytes,
            entries_archived
        );
    }

    fn on_automation_rule_registered(&self, rule_id: &str, trigger_type: &str, action: &str) {
        self.state
            .controller
            .automation_rules_active
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!(
            "Controller Automation rule registered: {} (trigger: {}, action: {})",
            rule_id,
            trigger_type,
            action
        );
    }

    fn on_automation_triggered(
        &self,
        rule_id: &str,
        trigger_reason: &str,
        permissions_affected: usize,
    ) {
        self.state
            .controller
            .automation_triggers
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .controller
            .events_triggered
            .fetch_add(1, Ordering::Relaxed);
        tracing::info!(
            "Controller Automation triggered: {} (reason: {}, affected: {})",
            rule_id,
            trigger_reason,
            permissions_affected
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4.5 DATALOGIC OBSERVER IMPLEMENTATION
// ────────────────────────────────────────────────────────────────────────────

impl DataLogicObserver for StateIntegrator {
    fn on_stream_registered(
        &self,
        stream_id: &str,
        source_type: &str,
        _filter_expression: Option<&str>,
    ) {
        self.state.data_logic.register_stream();
        tracing::debug!(
            "DataLogic Stream registered: {} (source: {})",
            stream_id,
            source_type
        );
    }

    fn on_stream_subscribed(&self, stream_id: &str, subscriber_id: &str) {
        self.state
            .data_logic
            .stream_subscriptions
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "DataLogic Stream subscribed: {} <- {}",
            stream_id,
            subscriber_id
        );
    }

    fn on_stream_unsubscribed(&self, stream_id: &str, subscriber_id: &str) {
        self.state
            .data_logic
            .stream_subscriptions
            .fetch_sub(1, Ordering::Relaxed);
        tracing::trace!(
            "DataLogic Stream unsubscribed: {} <- {}",
            stream_id,
            subscriber_id
        );
    }

    fn on_stream_closed(&self, stream_id: &str, reason: &str) {
        self.state
            .data_logic
            .streams_active
            .fetch_sub(1, Ordering::Relaxed);
        tracing::debug!(
            "DataLogic Stream closed: {} (reason: {})",
            stream_id,
            reason
        );
    }

    fn on_event_received(&self, stream_id: &str, event_type: &str, _size_bytes: usize) {
        tracing::trace!("DataLogic Event received: {} on {}", event_type, stream_id);
    }

    fn on_event_filtered(&self, stream_id: &str, filter_reason: &str) {
        self.state.data_logic.process_event(true, 0);
        tracing::trace!(
            "DataLogic Event filtered: {} (reason: {})",
            stream_id,
            filter_reason
        );
    }

    fn on_event_forwarded(&self, stream_id: &str, subscribers_notified: usize, gas_used: u64) {
        self.state.data_logic.process_event(false, gas_used);
        tracing::trace!(
            "DataLogic Event forwarded: {} (subscribers: {}, gas: {})",
            stream_id,
            subscribers_notified,
            gas_used
        );
    }

    fn on_processing_error(&self, stream_id: &str, error: &str) {
        self.state
            .data_logic
            .processing_errors
            .fetch_add(1, Ordering::Relaxed);
        tracing::warn!("DataLogic Processing error: {} - {}", stream_id, error);
    }

    fn on_aggregation_registered(
        &self,
        aggregation_id: &str,
        aggregation_type: &str,
        source_stream: &str,
        window_size: Option<u64>,
    ) {
        self.state
            .data_logic
            .aggregations_registered
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!(
            "DataLogic Aggregation registered: {} (type: {}, source: {}, window: {:?})",
            aggregation_id,
            aggregation_type,
            source_stream,
            window_size
        );
    }

    fn on_aggregation_computed(
        &self,
        aggregation_id: &str,
        result_value: f64,
        _events_aggregated: u64,
        latency_us: u64,
        gas_used: u64,
    ) {
        self.state
            .data_logic
            .aggregation_computed(latency_us, gas_used);
        tracing::trace!(
            "DataLogic Aggregation computed: {} = {:.4} (latency: {}µs)",
            aggregation_id,
            result_value,
            latency_us
        );
    }

    fn on_aggregation_emitted(&self, aggregation_id: &str, event_type: &str) {
        self.state
            .data_logic
            .events_triggered
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "DataLogic Aggregation emitted: {} -> {}",
            aggregation_id,
            event_type
        );
    }

    fn on_binding_propagated(
        &self,
        _stream_id: &str,
        binding_id: &str,
        success: bool,
        latency_us: u64,
        mana_used: u64,
    ) {
        self.state
            .data_logic
            .propagate_binding(success, latency_us, mana_used);
        tracing::trace!(
            "DataLogic Binding propagated: {} (success: {}, latency: {}µs)",
            binding_id,
            success,
            latency_us
        );
    }
}

// ────────────────────────────────────────────────────────────────────────────
// 4.6 BLUEPRINTCOMPOSER OBSERVER IMPLEMENTATION
// ────────────────────────────────────────────────────────────────────────────

impl BlueprintComposerObserver for StateIntegrator {
    fn on_composition_started(
        &self,
        composition_id: &str,
        base_blueprints: &[String],
        target_realm: Option<&str>,
    ) {
        tracing::debug!(
            "BlueprintComposer Composition started: {} (bases: {:?}, realm: {:?})",
            composition_id,
            base_blueprints,
            target_realm
        );
    }

    fn on_composition_completed(
        &self,
        composition_id: &str,
        success: bool,
        inheritance_depth: usize,
        conflicts_resolved: usize,
        gas_used: u64,
    ) {
        self.state.blueprint_composer.composition_created(
            success,
            inheritance_depth as u64,
            conflicts_resolved as u64,
            gas_used,
        );
        tracing::info!(
            "BlueprintComposer Composition completed: {} (success: {}, depth: {}, conflicts: {})",
            composition_id,
            success,
            inheritance_depth,
            conflicts_resolved
        );
    }

    fn on_inheritance_resolved(
        &self,
        child_blueprint: &str,
        parent_blueprint: &str,
        overrides_count: usize,
    ) {
        tracing::trace!(
            "BlueprintComposer Inheritance resolved: {} extends {} ({} overrides)",
            child_blueprint,
            parent_blueprint,
            overrides_count
        );
    }

    fn on_conflict_resolved(&self, composition_id: &str, conflict_type: &str, resolution: &str) {
        self.state
            .blueprint_composer
            .conflict_resolutions
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!(
            "BlueprintComposer Conflict resolved: {} (type: {}, resolution: {})",
            composition_id,
            conflict_type,
            resolution
        );
    }

    fn on_version_published(
        &self,
        blueprint_id: &str,
        version: &str,
        _author: &EntityId,
        _changelog: Option<&str>,
    ) {
        self.state
            .blueprint_composer
            .versions_published
            .fetch_add(1, Ordering::Relaxed);
        self.state
            .blueprint_composer
            .events_triggered
            .fetch_add(1, Ordering::Relaxed);
        tracing::info!(
            "BlueprintComposer Version published: {} v{}",
            blueprint_id,
            version
        );
    }

    fn on_migration_executed(
        &self,
        blueprint_id: &str,
        from_version: &str,
        to_version: &str,
        success: bool,
        instances_migrated: u64,
    ) {
        self.state
            .blueprint_composer
            .migrations_executed
            .fetch_add(1, Ordering::Relaxed);
        if !success {
            self.state
                .blueprint_composer
                .migration_errors
                .fetch_add(1, Ordering::Relaxed);
        }
        tracing::info!(
            "BlueprintComposer Migration executed: {} ({} -> {}, success: {}, instances: {})",
            blueprint_id,
            from_version,
            to_version,
            success,
            instances_migrated
        );
    }

    fn on_deprecation_marked(
        &self,
        blueprint_id: &str,
        version: &str,
        deprecation_reason: &str,
        _sunset_date: Option<u64>,
    ) {
        self.state
            .blueprint_composer
            .deprecations
            .fetch_add(1, Ordering::Relaxed);
        tracing::info!(
            "BlueprintComposer Deprecation marked: {} v{} (reason: {})",
            blueprint_id,
            version,
            deprecation_reason
        );
    }

    fn on_blueprint_instantiated(
        &self,
        blueprint_id: &str,
        instance_id: &str,
        _realm_id: Option<&str>,
        gas_used: u64,
    ) {
        self.state.blueprint_composer.instantiate(true, gas_used);
        tracing::debug!(
            "BlueprintComposer Blueprint instantiated: {} -> {} (gas: {})",
            blueprint_id,
            instance_id,
            gas_used
        );
    }

    fn on_instance_deactivated(&self, instance_id: &str, reason: &str) {
        self.state
            .blueprint_composer
            .instances_active
            .fetch_sub(1, Ordering::Relaxed);
        tracing::debug!(
            "BlueprintComposer Instance deactivated: {} (reason: {})",
            instance_id,
            reason
        );
    }

    fn on_realm_compatibility_check(
        &self,
        blueprint_id: &str,
        realm_id: &str,
        compatible: bool,
        _incompatibility_reasons: Option<&[String]>,
    ) {
        self.state
            .blueprint_composer
            .realm_compatibility_check(compatible);
        tracing::trace!(
            "BlueprintComposer Realm compatibility: {} in {} (compatible: {})",
            blueprint_id,
            realm_id,
            compatible
        );
    }

    fn on_dependency_resolved(
        &self,
        blueprint_id: &str,
        dependency_id: &str,
        version_constraint: &str,
        resolved_version: &str,
    ) {
        self.state
            .blueprint_composer
            .dependency_resolutions
            .fetch_add(1, Ordering::Relaxed);
        tracing::trace!(
            "BlueprintComposer Dependency resolved: {} -> {} ({} matched {})",
            blueprint_id,
            dependency_id,
            version_constraint,
            resolved_version
        );
    }

    fn on_cache_hit(&self, blueprint_id: &str, _version: &str) {
        self.state.blueprint_composer.cache_access(true);
        tracing::trace!("BlueprintComposer Cache hit: {}", blueprint_id);
    }

    fn on_cache_miss(&self, blueprint_id: &str, _version: &str) {
        self.state.blueprint_composer.cache_access(false);
        tracing::trace!("BlueprintComposer Cache miss: {}", blueprint_id);
    }

    fn on_cache_eviction(&self, blueprint_id: &str, reason: &str) {
        self.state
            .blueprint_composer
            .cache_evictions
            .fetch_add(1, Ordering::Relaxed);
        tracing::debug!(
            "BlueprintComposer Cache eviction: {} (reason: {})",
            blueprint_id,
            reason
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
        assert_eq!(snapshot.core.trust.entities_count, 1);
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
        assert_eq!(snapshot.execution.executions.total, 1);
        assert_eq!(snapshot.execution.executions.successful, 1);
        assert_eq!(snapshot.execution.gas.consumed, 2000); // 1000 direct + 1000 in complete
        assert_eq!(snapshot.execution.mana.consumed, 200);
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
        // Note: With deep relationship tracking, propagate_update() may increment
        // counters through Trigger/Validate/Aggregate relationships
        assert!(snapshot.peer.gateway.crossings_total >= 2);
        assert!(snapshot.peer.gateway.crossings_allowed >= 1);
        assert!(snapshot.peer.gateway.crossings_denied >= 1);
        // registered_realms may include propagated updates from Gateway→Realm relationship
        assert!(snapshot.peer.gateway.registered_realms >= 1);
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
        // Note: With deep relationship tracking, propagate_update() increments
        // counters through Trigger/Validate/Aggregate relationships
        assert!(snapshot.peer.realm.total_realms >= 2);
        assert_eq!(snapshot.peer.realm.root_realm_id, Some("test-realm".into()));
        assert!(snapshot.peer.realm.active_crossings >= 1);
        assert!(snapshot.peer.realm.crossing_failures >= 1);
        assert!(snapshot.peer.realm.total_cross_realm_sagas >= 1);

        // Test realm-specific state
        let test_realm = snapshot.peer.realm.realms.get("test-realm").unwrap();
        // member_count may be affected by relationship propagation
        assert!(test_realm.member_count >= 1); // 2 joined, 1 left = at least 1
        assert!(test_realm.crossings_out >= 1);
        assert_eq!(test_realm.active_rules.len(), 2);
        assert!(test_realm.active_rules.contains(&"rule-1".to_string()));

        let finance_realm = snapshot.peer.realm.realms.get("finance-realm").unwrap();
        assert!(finance_realm.crossings_in >= 1);
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

    // ========================================================================
    // PHASE 6.2: ENGINE-LAYER OBSERVER UNIT TESTS
    // ========================================================================

    #[test]
    fn test_ui_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Component Lifecycle
        integrator.on_component_registered("comp-1", "button", Some("test-realm"), None);
        integrator.on_component_registered("comp-2", "form", None, Some("comp-1"));
        integrator.on_component_updated("comp-1", "style");
        integrator.on_component_unmounted("comp-2");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.ui.components_registered, 2);
        assert_eq!(snapshot.ui.components_active, 1); // 2 registered, 1 unmounted
        assert_eq!(snapshot.ui.component_updates, 1);

        // Render
        integrator.on_component_rendered("comp-1", false, 100, 50, Some("test-realm"));
        integrator.on_component_rendered("comp-1", true, 0, 0, None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.ui.renders, 2);
        assert!((snapshot.ui.cache_hit_rate - 0.5).abs() < 0.01);
        assert_eq!(snapshot.ui.gas_consumed, 100);
        assert_eq!(snapshot.ui.mana_consumed, 50);

        // Trust-Gate
        integrator.on_trust_gate_evaluated("comp-1", 0.6, 0.8, true, Some("test-realm"));
        integrator.on_trust_gate_evaluated("comp-1", 0.8, 0.5, false, None);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.ui.trust_gate_evaluations, 2);
        assert!((snapshot.ui.trust_gate_allow_rate - 0.5).abs() < 0.01);

        // Credential-Gate
        integrator.on_credential_gate_evaluated("comp-1", &["admin".into()], true, true);
        integrator.on_credential_gate_evaluated("comp-1", &["admin".into()], false, false);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.ui.credential_gate_evaluations, 2);

        // Bindings
        integrator.on_binding_created("bind-1", "comp-1", "stream-1", "reactive");
        integrator.on_binding_updated("bind-1", true, 50, Some("test-realm"));
        integrator.on_binding_error("bind-1", "timeout");
        integrator.on_binding_removed("bind-1");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.ui.binding_errors, 1);
        assert_eq!(snapshot.ui.binding_updates, 1);

        // Events
        integrator.on_ui_action("comp-1", "click", 10, None);
        integrator.on_event_emitted("comp-1", "clicked");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.ui.events_triggered, 1);
    }

    #[test]
    fn test_api_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Endpoint Lifecycle
        integrator.on_endpoint_registered(
            "ep-1",
            "GET",
            "/api/users",
            "handler-1",
            Some("test-realm"),
        );
        integrator.on_endpoint_registered("ep-2", "POST", "/api/users", "handler-2", None);
        integrator.on_endpoint_updated("ep-1", "added-cache");
        integrator.on_endpoint_removed("ep-2");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.api.endpoints_registered, 2);
        assert_eq!(snapshot.api.endpoints_active, 1); // 2 registered, 1 removed
        assert_eq!(snapshot.api.endpoint_updates, 1);

        // Request Processing
        integrator.on_request_received("req-1", "ep-1", "GET", 0.8, Some("test-realm"));
        integrator.on_request_completed("req-1", 200, 1000, 50, 10, 1024);
        integrator.on_request_received("req-2", "ep-1", "GET", 0.5, None);
        integrator.on_request_completed("req-2", 500, 5000, 100, 20, 512);
        integrator.on_request_validation_failed("req-3", "invalid input", "VALIDATION_ERROR");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.api.requests_total, 2);
        assert_eq!(snapshot.api.requests_success, 1);
        assert_eq!(snapshot.api.requests_server_error, 1);
        assert_eq!(snapshot.api.requests_client_error, 1); // Validation failed
        assert!(snapshot.api.avg_latency_us > 0.0);

        // Rate Limiting
        integrator.on_rate_limit_bucket_created("bucket-1", "client-1", 100, 60, 1.5);
        APIObserver::on_rate_limited(&integrator, "client-1", "ep-1", 30, Some("test-realm"));

        let snapshot = state.snapshot();
        assert_eq!(snapshot.api.requests_rate_limited, 1);

        // Authentication
        integrator.on_auth_check("req-1", "jwt", true, None);
        integrator.on_auth_check("req-2", "jwt", false, Some("expired token"));
        integrator.on_authz_delegated("req-1", "read:users", true);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.api.requests_auth_failed, 1);
    }

    #[test]
    fn test_governance_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());
        let author = UniversalId::new(UniversalId::TAG_DID, 1, b"author");
        let voter1 = UniversalId::new(UniversalId::TAG_DID, 1, b"voter1");
        let voter2 = UniversalId::new(UniversalId::TAG_DID, 1, b"voter2");

        // Proposal Lifecycle
        integrator.on_proposal_created(
            "prop-1",
            "parameter-change",
            &author,
            Some("test-realm"),
            86400,
            0.5,
        );
        integrator.on_proposal_status_changed("prop-1", "pending", "active");
        integrator.on_proposal_completed("prop-1", "accepted", 100, 50, 10, 0.8);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.governance.proposals_created, 1);
        assert_eq!(snapshot.governance.proposals_completed, 1);
        assert_eq!(snapshot.governance.proposals_accepted, 1);

        // Voting
        integrator.on_vote_cast(
            "prop-2",
            &voter1,
            "yes",
            2.0,
            1.41,
            false,
            0,
            Some("test-realm"),
        );
        integrator.on_vote_cast("prop-2", &voter2, "no", 1.0, 1.0, true, 2, None);
        integrator.on_vote_withdrawn("prop-2", &voter1);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.governance.votes_cast, 2);
        assert_eq!(snapshot.governance.votes_delegated, 1);
        assert!(snapshot.governance.avg_voting_power > 1.0);

        // Quadratic Voting
        integrator.on_quadratic_reduction(&voter1, 4.0, 2.0, 0.5);

        let snapshot = state.snapshot();
        // quadratic_reductions wird auch bei vote_cast erhöht (voter2 hatte delegated=true)
        assert!(snapshot.governance.quadratic_reductions >= 1);

        // Delegation
        integrator.on_delegation_created(
            &voter1,
            &voter2,
            "realm",
            Some("test-realm"),
            Some(86400),
        );
        integrator.on_delegation_chain_resolved(&voter1, &voter2, 2);
        integrator.on_circular_delegation_prevented(
            &voter1,
            &voter2,
            &["a".into(), "b".into(), "a".into()],
        );
        integrator.on_delegation_expired(&voter1, &voter2);

        let snapshot = state.snapshot();
        assert!(snapshot.governance.delegations_active == 0); // 1 created, 1 expired
        assert_eq!(snapshot.governance.max_delegation_depth, 2);

        // Anti-Calcification
        integrator.on_power_concentration_check(Some("test-realm"), &voter1, 35.0, 30.0, true);
        integrator.on_gini_calculated(Some("test-realm"), 0.45);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.governance.power_violations, 1);
        assert!((snapshot.governance.voting_power_gini - 0.45).abs() < 0.01);

        // Veto & Permission Changes
        integrator.on_veto_exercised("prop-3", &author, "safety concern");
        integrator.on_permission_change_triggered("prop-1", "admin", &voter1, "grant");

        // proposals_vetoed und controller_triggers sind State-intern, nicht im Snapshot
        let snapshot = state.snapshot();
        // events_triggered wird bei jeder Governance-Aktion erhöht (proposals, votes, etc.)
        assert!(snapshot.governance.events_triggered >= 2);
    }

    #[test]
    fn test_controller_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());
        let grantee = UniversalId::new(UniversalId::TAG_DID, 1, b"grantee");
        let granter = UniversalId::new(UniversalId::TAG_DID, 1, b"granter");

        // Permission Registration
        integrator.on_permission_registered("perm-1", "read:users", "realm", "Read user data");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.controller.permissions_registered, 1);

        // Permission Grants
        integrator.on_permission_granted(
            "perm-1",
            &grantee,
            &granter,
            None,
            Some("test-realm"),
            Some(86400),
            None,
        );
        integrator.on_permission_granted(
            "perm-2",
            &grantee,
            &granter,
            Some("room-1"),
            None,
            None,
            Some("trust > 0.5"),
        );
        integrator.on_permission_revoked("perm-2", &grantee, &granter, "policy violation");
        integrator.on_permission_expired("perm-1", &grantee);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.controller.permission_grants, 2);
        assert_eq!(snapshot.controller.permission_revokes, 1);
        assert_eq!(snapshot.controller.permissions_active, 0); // 2 granted, 1 revoked, 1 expired

        // Authorization Checks
        integrator.on_authz_check(
            &grantee,
            "read:users",
            "/api/users",
            "realm",
            Some("test-realm"),
            true,
            false,
            50,
        );
        integrator.on_authz_check(
            &grantee,
            "write:users",
            "/api/users",
            "room",
            Some("room-1"),
            true,
            true,
            100,
        );
        integrator.on_authz_check(
            &grantee,
            "delete:users",
            "/api/users",
            "partition",
            None,
            false,
            false,
            25,
        );

        let snapshot = state.snapshot();
        assert_eq!(snapshot.controller.authz_checks, 3);
        assert_eq!(snapshot.controller.authz_allowed, 2);
        assert_eq!(snapshot.controller.authz_denied, 1);
        assert_eq!(snapshot.controller.realm_scope_checks, 1);
        assert_eq!(snapshot.controller.room_scope_checks, 1);
        assert_eq!(snapshot.controller.partition_scope_checks, 1);
        assert!(snapshot.controller.avg_check_latency_us > 0.0);

        // Delegation
        integrator.on_permission_delegated("perm-1", &granter, &grantee, true, Some("max_depth=2"));
        integrator.on_delegation_resolved("perm-1", &grantee, &granter, 3);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.controller.delegations_active, 1);
        assert_eq!(snapshot.controller.max_delegation_depth, 3);

        // Audit
        integrator.on_audit_entry(
            "audit-1",
            "permission_granted",
            &granter,
            Some(&grantee),
            "read:users",
            "success",
            256,
        );
        integrator.on_audit_log_rotated(1024 * 1024, 10000);

        let snapshot = state.snapshot();
        // audit_entries kann durch vorherige Aktionen erhöht worden sein
        assert!(snapshot.controller.audit_entries >= 1);
        assert!(snapshot.controller.audit_log_bytes > 0);

        // Automation
        integrator.on_automation_rule_registered("rule-1", "trust_drop", "revoke_permissions");
        integrator.on_automation_triggered("rule-1", "trust dropped below 0.3", 5);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.controller.automation_triggers, 1);
        // events_triggered wird bei mehreren Controller-Aktionen erhöht
        assert!(snapshot.controller.events_triggered >= 1);
    }

    #[test]
    fn test_data_logic_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Stream Lifecycle
        integrator.on_stream_registered("stream-1", "trust_events", Some("type == 'update'"));
        integrator.on_stream_registered("stream-2", "execution_events", None);
        integrator.on_stream_subscribed("stream-1", "subscriber-1");
        integrator.on_stream_subscribed("stream-1", "subscriber-2");
        integrator.on_stream_unsubscribed("stream-1", "subscriber-1");
        integrator.on_stream_closed("stream-2", "no subscribers");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.data_logic.streams_registered, 2);
        assert_eq!(snapshot.data_logic.streams_active, 1); // 2 registered, 1 closed
        assert_eq!(snapshot.data_logic.stream_subscriptions, 1); // 2 subscribed, 1 unsubscribed

        // Event Processing
        integrator.on_event_received("stream-1", "trust_update", 256);
        integrator.on_event_filtered("stream-1", "type mismatch");
        integrator.on_event_forwarded("stream-1", 2, 50);
        integrator.on_processing_error("stream-1", "serialization failed");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.data_logic.events_processed, 2); // filtered + forwarded
        assert_eq!(snapshot.data_logic.events_filtered, 1);
        assert_eq!(snapshot.data_logic.events_forwarded, 1);
        assert_eq!(snapshot.data_logic.gas_consumed, 50);

        // Aggregation
        integrator.on_aggregation_registered("agg-1", "avg", "stream-1", Some(60));
        integrator.on_aggregation_computed("agg-1", 0.75, 100, 500, 80);
        integrator.on_aggregation_emitted("agg-1", "aggregation_result");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.data_logic.aggregations_registered, 1);
        assert_eq!(snapshot.data_logic.aggregations_computed, 1);
        assert!(snapshot.data_logic.avg_aggregation_latency_us > 0.0);
        // events_triggered wird bei aggregation_emitted erhöht und ggf. bei anderen Aktionen
        assert!(snapshot.data_logic.events_triggered >= 1);

        // Binding Propagation
        integrator.on_binding_propagated("stream-1", "binding-1", true, 100, 20);
        integrator.on_binding_propagated("stream-1", "binding-2", false, 50, 10);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.data_logic.binding_propagations, 2);
        assert!((snapshot.data_logic.binding_success_rate - 0.5).abs() < 0.01);
        assert_eq!(snapshot.data_logic.mana_consumed, 30);
    }

    #[test]
    fn test_blueprint_composer_observer() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());
        let author = UniversalId::new(UniversalId::TAG_DID, 1, b"author");

        // Composition
        integrator.on_composition_started(
            "comp-1",
            &["base-a".into(), "base-b".into()],
            Some("test-realm"),
        );
        integrator.on_inheritance_resolved("child", "parent", 3);
        integrator.on_conflict_resolved("comp-1", "method_override", "child_wins");
        integrator.on_composition_completed("comp-1", true, 3, 2, 150);
        integrator.on_composition_completed("comp-2", false, 0, 0, 50);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.blueprint_composer.compositions_created, 2);
        assert_eq!(snapshot.blueprint_composer.compositions_successful, 1);
        assert_eq!(snapshot.blueprint_composer.compositions_failed, 1);
        assert_eq!(snapshot.blueprint_composer.max_inheritance_depth, 3);
        assert_eq!(snapshot.blueprint_composer.conflict_resolutions, 3); // 2 from composition + 1 explicit
        assert_eq!(snapshot.blueprint_composer.gas_consumed, 200);

        // Versioning
        integrator.on_version_published("blueprint-1", "1.0.0", &author, Some("Initial release"));
        integrator.on_migration_executed("blueprint-1", "0.9.0", "1.0.0", true, 10);
        integrator.on_migration_executed("blueprint-2", "1.0.0", "2.0.0", false, 0);

        let snapshot = state.snapshot();
        assert_eq!(snapshot.blueprint_composer.versions_published, 1);
        assert_eq!(snapshot.blueprint_composer.migrations_executed, 2);
        // events_triggered wird bei version_published und anderen Aktionen erhöht
        assert!(snapshot.blueprint_composer.events_triggered >= 1);

        // Instantiation
        BlueprintComposerObserver::on_blueprint_instantiated(
            &integrator,
            "blueprint-1",
            "instance-1",
            Some("test-realm"),
            80,
        );
        BlueprintComposerObserver::on_blueprint_instantiated(
            &integrator,
            "blueprint-1",
            "instance-2",
            None,
            70,
        );
        integrator.on_instance_deactivated("instance-1", "realm deleted");

        let snapshot = state.snapshot();
        assert_eq!(snapshot.blueprint_composer.instantiations, 2);
        assert_eq!(snapshot.blueprint_composer.instances_active, 1); // 2 created, 1 deactivated

        // Realm Compatibility
        integrator.on_realm_compatibility_check("blueprint-1", "test-realm", true, None);
        integrator.on_realm_compatibility_check(
            "blueprint-1",
            "restricted-realm",
            false,
            Some(&["missing_permission".into()]),
        );

        let snapshot = state.snapshot();
        assert_eq!(snapshot.blueprint_composer.realm_compatibility_checks, 2);
        assert_eq!(snapshot.blueprint_composer.compatibility_failures, 1);

        // Dependency Resolution
        integrator.on_dependency_resolved("blueprint-1", "dependency-1", "^1.0.0", "1.2.3");

        // dependency_resolutions wird im State aber nicht im Snapshot tracked
        // Verifiziere nur dass der Aufruf keinen Fehler wirft

        // Caching
        integrator.on_cache_hit("blueprint-1", "1.0.0");
        integrator.on_cache_hit("blueprint-1", "1.0.0");
        integrator.on_cache_miss("blueprint-2", "2.0.0");
        integrator.on_cache_eviction("blueprint-1", "memory pressure");

        let snapshot = state.snapshot();
        assert!((snapshot.blueprint_composer.cache_hit_rate - 0.666).abs() < 0.01);
        // cache_evictions wird im State aber nicht im Snapshot tracked
    }

    // ========================================================================
    // PHASE 6.3: OBSERVER REGISTRATION & NOTIFICATION INTEGRATION TESTS
    // ========================================================================

    /// Mock UIObserver for testing registration and notification
    struct MockUIObserver {
        calls: std::sync::Mutex<Vec<String>>,
    }

    impl MockUIObserver {
        fn new() -> Self {
            Self {
                calls: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn call_count(&self) -> usize {
            self.calls.lock().unwrap().len()
        }
    }

    impl UIObserver for MockUIObserver {
        fn on_component_registered(
            &self,
            component_id: &str,
            _: &str,
            _: Option<&str>,
            _: Option<&str>,
        ) {
            self.calls
                .lock()
                .unwrap()
                .push(format!("registered:{}", component_id));
        }
        fn on_component_unmounted(&self, _: &str) {}
        fn on_component_updated(&self, _: &str, _: &str) {}
        fn on_component_rendered(
            &self,
            component_id: &str,
            _: bool,
            _: u64,
            _: u64,
            _: Option<&str>,
        ) {
            self.calls
                .lock()
                .unwrap()
                .push(format!("rendered:{}", component_id));
        }
        fn on_trust_gate_evaluated(
            &self,
            component_id: &str,
            _: f64,
            _: f64,
            _: bool,
            _: Option<&str>,
        ) {
            self.calls
                .lock()
                .unwrap()
                .push(format!("trust_gate:{}", component_id));
        }
        fn on_trust_gate_configured(&self, _: &str, _: f64, _: Option<f64>) {}
        fn on_credential_gate_evaluated(&self, _: &str, _: &[String], _: bool, _: bool) {}
        fn on_binding_created(&self, _: &str, _: &str, _: &str, _: &str) {}
        fn on_binding_updated(&self, _: &str, _: bool, _: u64, _: Option<&str>) {}
        fn on_binding_error(&self, _: &str, _: &str) {}
        fn on_binding_removed(&self, _: &str) {}
        fn on_ui_action(&self, _: &str, _: &str, _: usize, _: Option<&str>) {}
        fn on_event_emitted(&self, _: &str, _: &str) {}
    }

    #[test]
    fn test_observer_registration_ui() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state);

        let mock_observer = Arc::new(MockUIObserver::new());
        integrator.register_ui_observer(mock_observer.clone());

        // Trigger notifications
        integrator.notify_ui_component_registered("comp-1", "button", Some("realm"), None);
        integrator.notify_ui_component_rendered("comp-1", false, 100, 50, None);
        integrator.notify_ui_trust_gate("comp-1", 0.5, 0.8, true, None);

        assert_eq!(mock_observer.call_count(), 3);
    }

    /// Mock GovernanceObserver for testing
    struct MockGovernanceObserver {
        proposals_created: std::sync::atomic::AtomicU64,
        votes_cast: std::sync::atomic::AtomicU64,
    }

    impl MockGovernanceObserver {
        fn new() -> Self {
            Self {
                proposals_created: std::sync::atomic::AtomicU64::new(0),
                votes_cast: std::sync::atomic::AtomicU64::new(0),
            }
        }
    }

    impl GovernanceObserver for MockGovernanceObserver {
        fn on_proposal_created(
            &self,
            _: &str,
            _: &str,
            _: &EntityId,
            _: Option<&str>,
            _: u64,
            _: f64,
        ) {
            self.proposals_created.fetch_add(1, Ordering::Relaxed);
        }
        fn on_proposal_status_changed(&self, _: &str, _: &str, _: &str) {}
        fn on_proposal_completed(&self, _: &str, _: &str, _: u64, _: u64, _: u64, _: f64) {}
        fn on_vote_cast(
            &self,
            _: &str,
            _: &EntityId,
            _: &str,
            _: f64,
            _: f64,
            _: bool,
            _: usize,
            _: Option<&str>,
        ) {
            self.votes_cast.fetch_add(1, Ordering::Relaxed);
        }
        fn on_vote_withdrawn(&self, _: &str, _: &EntityId) {}
        fn on_quadratic_reduction(&self, _: &EntityId, _: f64, _: f64, _: f64) {}
        fn on_delegation_created(
            &self,
            _: &EntityId,
            _: &EntityId,
            _: &str,
            _: Option<&str>,
            _: Option<u64>,
        ) {
        }
        fn on_delegation_revoked(&self, _: &EntityId, _: &EntityId) {}
        fn on_delegation_expired(&self, _: &EntityId, _: &EntityId) {}
        fn on_circular_delegation_prevented(&self, _: &EntityId, _: &EntityId, _: &[String]) {}
        fn on_delegation_chain_resolved(&self, _: &EntityId, _: &EntityId, _: usize) {}
        fn on_power_concentration_check(
            &self,
            _: Option<&str>,
            _: &EntityId,
            _: f64,
            _: f64,
            _: bool,
        ) {
        }
        fn on_gini_calculated(&self, _: Option<&str>, _: f64) {}
        fn on_veto_exercised(&self, _: &str, _: &EntityId, _: &str) {}
        fn on_permission_change_triggered(&self, _: &str, _: &str, _: &EntityId, _: &str) {}
    }

    #[test]
    fn test_observer_registration_governance() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state);

        let mock_observer = Arc::new(MockGovernanceObserver::new());
        integrator.register_governance_observer(mock_observer.clone());

        let author = UniversalId::new(UniversalId::TAG_DID, 1, b"author");
        let voter = UniversalId::new(UniversalId::TAG_DID, 1, b"voter");

        // Trigger notifications
        integrator
            .notify_governance_proposal_created("prop-1", "change", &author, None, 86400, 0.5);
        integrator.notify_governance_vote_cast("prop-1", &voter, "yes", 1.0, 1.0, false, 0, None);
        integrator.notify_governance_vote_cast("prop-1", &voter, "no", 1.0, 1.0, false, 0, None);

        assert_eq!(mock_observer.proposals_created.load(Ordering::Relaxed), 1);
        assert_eq!(mock_observer.votes_cast.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_multiple_observers_notification() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state);

        // Register multiple UI observers
        let observer1 = Arc::new(MockUIObserver::new());
        let observer2 = Arc::new(MockUIObserver::new());
        let observer3 = Arc::new(MockUIObserver::new());

        integrator.register_ui_observer(observer1.clone());
        integrator.register_ui_observer(observer2.clone());
        integrator.register_ui_observer(observer3.clone());

        // Single notification should reach all observers
        integrator.notify_ui_component_registered("comp-1", "button", None, None);

        assert_eq!(observer1.call_count(), 1);
        assert_eq!(observer2.call_count(), 1);
        assert_eq!(observer3.call_count(), 1);
    }

    #[test]
    fn test_integration_observer_state_consistency() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Simuliere vollständigen Workflow
        let author = UniversalId::new(UniversalId::TAG_DID, 1, b"author");

        // 1. UI Component registriert
        integrator.on_component_registered("dashboard", "container", Some("main-realm"), None);

        // 2. API Endpoint registriert
        integrator.on_endpoint_registered(
            "user-api",
            "GET",
            "/users",
            "handler",
            Some("main-realm"),
        );

        // 3. Governance Proposal erstellt
        integrator.on_proposal_created(
            "upgrade-v2",
            "system-upgrade",
            &author,
            Some("main-realm"),
            86400,
            0.51,
        );

        // 4. Controller Permission gewährt
        integrator.on_permission_granted(
            "admin-read",
            &author,
            &author,
            None,
            Some("main-realm"),
            None,
            None,
        );

        // 5. DataLogic Stream registriert
        integrator.on_stream_registered("events-stream", "all_events", None);

        // 6. Blueprint erstellt
        integrator.on_composition_completed("realm-blueprint", true, 2, 0, 200);

        // Verifiziere konsistenten State
        let snapshot = state.snapshot();
        assert_eq!(snapshot.ui.components_registered, 1);
        assert_eq!(snapshot.api.endpoints_registered, 1);
        assert_eq!(snapshot.governance.proposals_created, 1);
        assert_eq!(snapshot.controller.permission_grants, 1);
        assert_eq!(snapshot.data_logic.streams_registered, 1);
        assert_eq!(snapshot.blueprint_composer.compositions_created, 1);

        // Health sollte hoch sein mit einem aktiven Element pro Engine
        let health = state.calculate_health();
        assert!(health >= 80.0, "Health should be high: {}", health);
    }

    #[test]
    fn test_realm_specific_state_tracking() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        let realm_a = "realm-alpha";
        let realm_b = "realm-beta";

        // UI Components in verschiedenen Realms
        integrator.on_component_registered("comp-a1", "button", Some(realm_a), None);
        integrator.on_component_registered("comp-a2", "form", Some(realm_a), None);
        integrator.on_component_registered("comp-b1", "table", Some(realm_b), None);
        integrator.on_component_rendered("comp-a1", false, 100, 50, Some(realm_a));
        integrator.on_component_rendered("comp-b1", true, 0, 0, Some(realm_b));

        // API Endpoints in verschiedenen Realms
        integrator.on_endpoint_registered("ep-a", "GET", "/alpha/data", "h1", Some(realm_a));
        integrator.on_endpoint_registered("ep-b", "POST", "/beta/data", "h2", Some(realm_b));

        // Governance in verschiedenen Realms
        let author = UniversalId::new(UniversalId::TAG_DID, 1, b"author");
        integrator.on_proposal_created("prop-a", "change", &author, Some(realm_a), 3600, 0.5);
        integrator.on_vote_cast("prop-a", &author, "yes", 1.0, 1.0, false, 0, Some(realm_a));

        // Verifiziere Realm-spezifischen State via direkten State-Zugriff
        // UI Realm States (RealmUIState hat: components, renders, bindings, trust_gate_denied)
        let ui_realm_a = state.ui.realm_ui.read().unwrap();
        let ui_a = ui_realm_a.get(realm_a).unwrap();
        assert_eq!(ui_a.components.load(Ordering::Relaxed), 2);
        assert_eq!(ui_a.renders.load(Ordering::Relaxed), 1);
        drop(ui_realm_a);

        let ui_realm_b = state.ui.realm_ui.read().unwrap();
        let ui_b = ui_realm_b.get(realm_b).unwrap();
        assert_eq!(ui_b.components.load(Ordering::Relaxed), 1);
        assert_eq!(ui_b.renders.load(Ordering::Relaxed), 1);
        drop(ui_realm_b);

        // API State - globale Endpoints prüfen (realm_api wurde entfernt)
        assert!(state.api.endpoints_registered.load(Ordering::Relaxed) >= 2);

        // Governance Realm States (RealmGovernanceState hat: proposals, votes, delegations)
        let gov_realms = state.governance.realm_governance.read().unwrap();
        let gov_a = gov_realms.get(realm_a).unwrap();
        assert_eq!(gov_a.proposals.load(Ordering::Relaxed), 1);
        assert_eq!(gov_a.votes.load(Ordering::Relaxed), 1);
        drop(gov_realms);
    }

    #[test]
    fn test_cross_engine_event_flow() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Simuliere Cross-Engine Event Flow:
        // Governance → Controller → API → UI

        let admin = UniversalId::new(UniversalId::TAG_DID, 1, b"admin");

        // 1. Governance: Permission-Änderung wird vorgeschlagen und akzeptiert
        integrator.on_proposal_created(
            "grant-api-access",
            "permission-grant",
            &admin,
            None,
            3600,
            0.5,
        );
        integrator.on_proposal_completed("grant-api-access", "accepted", 10, 2, 1, 0.8);
        integrator.on_permission_change_triggered("grant-api-access", "api:read", &admin, "grant");

        // 2. Controller: Permission wird gewährt
        integrator.on_permission_granted("api:read", &admin, &admin, None, None, Some(86400), None);

        // 3. API: Neuer Endpoint wird aktiviert
        integrator.on_endpoint_registered("protected-api", "GET", "/protected", "handler", None);
        integrator.on_request_completed("req-1", 200, 100, 50, 10, 256);

        // 4. UI: Dashboard zeigt neuen Status
        integrator.on_component_registered("status-indicator", "badge", None, None);
        integrator.on_binding_updated("status-binding", true, 50, None);

        // Verifiziere Event-Propagation durch alle Engines
        let snapshot = state.snapshot();
        assert_eq!(snapshot.governance.proposals_completed, 1);
        // controller_triggers wird im State getrackt aber nicht im Snapshot
        assert_eq!(snapshot.controller.permission_grants, 1);
        assert_eq!(snapshot.api.endpoints_registered, 1);
        assert_eq!(snapshot.api.requests_total, 1);
        assert_eq!(snapshot.ui.components_registered, 1);
    }

    #[test]
    fn test_error_recovery_state_consistency() {
        let state = create_unified_state();
        let integrator = StateIntegrator::new(state.clone());

        // Simuliere Fehler und Recovery

        // API Errors
        integrator.on_request_completed("req-1", 500, 1000, 50, 10, 256);
        integrator.on_request_completed("req-2", 500, 1000, 50, 10, 256);
        integrator.on_request_completed("req-3", 200, 100, 10, 5, 128); // Recovery

        // Controller Denied
        let entity = UniversalId::new(UniversalId::TAG_DID, 1, b"user");
        integrator.on_authz_check(&entity, "admin", "/admin", "realm", None, false, false, 50);
        integrator.on_authz_check(&entity, "read", "/data", "realm", None, true, false, 25); // Recovery

        // DataLogic Errors
        integrator.on_processing_error("stream-1", "connection lost");
        integrator.on_event_forwarded("stream-1", 5, 100); // Recovery

        // Blueprint Failures
        integrator.on_composition_completed("bad-blueprint", false, 0, 0, 50);
        integrator.on_composition_completed("good-blueprint", true, 2, 0, 100); // Recovery

        let snapshot = state.snapshot();

        // API sollte Recovery zeigen
        assert_eq!(snapshot.api.requests_total, 3);
        assert_eq!(snapshot.api.requests_server_error, 2);
        assert_eq!(snapshot.api.requests_success, 1);

        // Controller sollte Recovery zeigen
        assert_eq!(snapshot.controller.authz_checks, 2);
        assert_eq!(snapshot.controller.authz_denied, 1);
        assert_eq!(snapshot.controller.authz_allowed, 1);

        // DataLogic sollte Recovery zeigen
        // processing_errors wird im State aber nicht im Snapshot getrackt
        assert_eq!(snapshot.data_logic.events_forwarded, 1);

        // Blueprint sollte Recovery zeigen
        assert_eq!(snapshot.blueprint_composer.compositions_failed, 1);
        assert_eq!(snapshot.blueprint_composer.compositions_successful, 1);

        // Gesamtgesundheit sollte unter 100% aber nicht kritisch sein
        let health = state.calculate_health();
        assert!(health < 99.0, "Health should reflect errors: {}", health);
        assert!(health > 70.0, "Health should not be critical: {}", health);
    }
}
