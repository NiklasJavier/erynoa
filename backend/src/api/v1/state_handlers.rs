//! Phase 1: Debug & Observability API Handlers
//!
//! Endpoints für State Snapshots, Health, Invariants, Events, Metrics und Warnings.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{sse::Event, IntoResponse, Sse},
    Json,
};
use base64::Engine;
use serde::Serialize;
use std::collections::HashMap;
use std::convert::Infallible;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use crate::core::{
    CircuitBreakerSnapshot, HealthReport, HealthStatus, InvariantResult, InvariantSeverity,
    StateComponent, StateEvent, SystemMode, UnifiedSnapshot, WrappedStateEvent,
};
use crate::server::AppState;

// ============================================================================
// Query / Path types
// ============================================================================

#[derive(Debug, Default, serde::Deserialize)]
pub struct SnapshotQuery {
    /// Komma-getrennte Komponenten (z.B. "core,eclvm"); leer = alle
    pub components: Option<String>,
    /// Optional: Realm-Filter (für spätere Nutzung)
    pub realm_id: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct InvariantsQuery {
    /// Filter nach Severity: Warning, Error, Critical
    pub severity: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct EventsQuery {
    pub limit: Option<u32>,
    pub since_sequence: Option<u64>,
    /// StateComponent-Name (z.B. "Trust", "ECLVM")
    pub component: Option<String>,
    pub realm_id: Option<String>,
}

// ============================================================================
// Response types
// ============================================================================

#[derive(Serialize)]
pub struct HealthStateResponse {
    pub score: f64,
}

#[derive(Serialize)]
pub struct HealthStateDetailResponse {
    pub overall_score: f64,
    pub status: HealthStatus,
    pub module_scores: HashMap<String, f64>,
    pub invariant_summary: InvariantSummary,
}

#[derive(Serialize)]
pub struct InvariantSummary {
    pub passed: usize,
    pub failed: usize,
    pub by_severity: HashMap<String, usize>,
}

// ============================================================================
// State Snapshots
// ============================================================================

/// GET /api/v1/state/snapshot – Full UnifiedSnapshot, optional filter by components/realm_id
pub async fn state_snapshot_handler(
    State(state): State<AppState>,
    Query(q): Query<SnapshotQuery>,
) -> impl IntoResponse {
    let snapshot = state.unified_state.snapshot();

    if let Some(ref comps) = q.components {
        let filter: Vec<&str> = comps.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if filter.is_empty() {
            return Json(serde_json::to_value(&snapshot).unwrap_or_default()).into_response();
        }
        let filtered = filter_snapshot_by_components(&snapshot, &filter);
        return Json(filtered).into_response();
    }

    Json(snapshot).into_response()
}

/// GET /api/v1/state/:component_name – Single component snapshot (e.g. core, eclvm)
pub async fn state_component_handler(
    State(state): State<AppState>,
    Path(component_name): Path<String>,
) -> impl IntoResponse {
    let snapshot = state.unified_state.snapshot();
    let name = component_name.to_lowercase();

    let value = match name.as_str() {
        "identity" => serde_json::to_value(&snapshot.identity).ok(),
        "core" => serde_json::to_value(&snapshot.core).ok(),
        "execution" => serde_json::to_value(&snapshot.execution).ok(),
        "eclvm" => serde_json::to_value(&snapshot.eclvm).ok(),
        "protection" => serde_json::to_value(&snapshot.protection).ok(),
        "storage" => serde_json::to_value(&snapshot.storage).ok(),
        "peer" => serde_json::to_value(&snapshot.peer).ok(),
        "p2p" => serde_json::to_value(&snapshot.p2p).ok(),
        "ui" => serde_json::to_value(&snapshot.ui).ok(),
        "api" => serde_json::to_value(&snapshot.api).ok(),
        "governance" => serde_json::to_value(&snapshot.governance).ok(),
        "controller" => serde_json::to_value(&snapshot.controller).ok(),
        "data_logic" => serde_json::to_value(&snapshot.data_logic).ok(),
        "blueprint_composer" => serde_json::to_value(&snapshot.blueprint_composer).ok(),
        "event_log" => serde_json::to_value(&snapshot.event_log).ok(),
        "event_bus" => serde_json::to_value(&snapshot.event_bus).ok(),
        "circuit_breaker" => serde_json::to_value(&snapshot.circuit_breaker).ok(),
        "broadcaster" => serde_json::to_value(&snapshot.broadcaster).ok(),
        "merkle_tracker" => serde_json::to_value(&snapshot.merkle_tracker).ok(),
        "multi_gas" => serde_json::to_value(&snapshot.multi_gas).ok(),
        _ => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "unknown component", "component": component_name })),
            )
                .into_response()
        }
    };

    match value {
        Some(v) => Json(v).into_response(),
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "serialization failed" })),
        )
            .into_response(),
    }
}

fn filter_snapshot_by_components(snapshot: &UnifiedSnapshot, components: &[&str]) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    let comps: std::collections::HashSet<&str> = components.iter().copied().collect();

    if comps.contains("identity") {
        out.insert("identity".into(), serde_json::to_value(&snapshot.identity).unwrap_or_default());
    }
    if comps.contains("core") {
        out.insert("core".into(), serde_json::to_value(&snapshot.core).unwrap_or_default());
    }
    if comps.contains("execution") {
        out.insert("execution".into(), serde_json::to_value(&snapshot.execution).unwrap_or_default());
    }
    if comps.contains("eclvm") {
        out.insert("eclvm".into(), serde_json::to_value(&snapshot.eclvm).unwrap_or_default());
    }
    if comps.contains("protection") {
        out.insert("protection".into(), serde_json::to_value(&snapshot.protection).unwrap_or_default());
    }
    if comps.contains("storage") {
        out.insert("storage".into(), serde_json::to_value(&snapshot.storage).unwrap_or_default());
    }
    if comps.contains("peer") {
        out.insert("peer".into(), serde_json::to_value(&snapshot.peer).unwrap_or_default());
    }
    if comps.contains("p2p") {
        out.insert("p2p".into(), serde_json::to_value(&snapshot.p2p).unwrap_or_default());
    }
    if comps.contains("ui") {
        out.insert("ui".into(), serde_json::to_value(&snapshot.ui).unwrap_or_default());
    }
    if comps.contains("api") {
        out.insert("api".into(), serde_json::to_value(&snapshot.api).unwrap_or_default());
    }
    if comps.contains("governance") {
        out.insert("governance".into(), serde_json::to_value(&snapshot.governance).unwrap_or_default());
    }
    if comps.contains("controller") {
        out.insert("controller".into(), serde_json::to_value(&snapshot.controller).unwrap_or_default());
    }
    if comps.contains("data_logic") {
        out.insert("data_logic".into(), serde_json::to_value(&snapshot.data_logic).unwrap_or_default());
    }
    if comps.contains("blueprint_composer") {
        out.insert("blueprint_composer".into(), serde_json::to_value(&snapshot.blueprint_composer).unwrap_or_default());
    }
    if comps.contains("event_log") {
        out.insert("event_log".into(), serde_json::to_value(&snapshot.event_log).unwrap_or_default());
    }
    if comps.contains("event_bus") {
        out.insert("event_bus".into(), serde_json::to_value(&snapshot.event_bus).unwrap_or_default());
    }
    if comps.contains("circuit_breaker") {
        out.insert("circuit_breaker".into(), serde_json::to_value(&snapshot.circuit_breaker).unwrap_or_default());
    }
    if comps.contains("broadcaster") {
        out.insert("broadcaster".into(), serde_json::to_value(&snapshot.broadcaster).unwrap_or_default());
    }
    if comps.contains("merkle_tracker") {
        out.insert("merkle_tracker".into(), serde_json::to_value(&snapshot.merkle_tracker).unwrap_or_default());
    }
    if comps.contains("multi_gas") {
        out.insert("multi_gas".into(), serde_json::to_value(&snapshot.multi_gas).unwrap_or_default());
    }

    out.insert("timestamp_ms".into(), serde_json::json!(snapshot.timestamp_ms));
    out.insert("health_score".into(), serde_json::json!(snapshot.health_score));
    serde_json::Value::Object(out)
}

// ============================================================================
// Health & Invariants
// ============================================================================

/// GET /api/v1/health/state – Aggregated health score (state.calculate_health)
pub async fn health_state_handler(State(state): State<AppState>) -> Json<HealthStateResponse> {
    let score = state.unified_state.snapshot().health_score;
    Json(HealthStateResponse { score })
}

/// GET /api/v1/health/state/detail – Detailed breakdown per layer
pub async fn health_state_detail_handler(State(state): State<AppState>) -> Json<HealthStateDetailResponse> {
    let report = state.coordinator.aggregate_health();
    let mut by_severity: HashMap<String, usize> = HashMap::new();
    for r in &report.invariant_results {
        let key = format!("{:?}", r.invariant.severity());
        *by_severity.entry(key).or_default() += 1;
    }
    let (passed, failed) = report
        .invariant_results
        .iter()
        .fold((0, 0), |(p, f), r| if r.passed { (p + 1, f) } else { (p, f + 1) });

    Json(HealthStateDetailResponse {
        overall_score: report.overall_score,
        status: report.status,
        module_scores: report.module_scores.clone(),
        invariant_summary: InvariantSummary {
            passed,
            failed,
            by_severity,
        },
    })
}

/// GET /api/v1/health/aggregate – Full HealthReport
pub async fn health_aggregate_handler(State(state): State<AppState>) -> Json<HealthReport> {
    Json(state.coordinator.aggregate_health())
}

/// GET /api/v1/invariants – List InvariantResult, optional severity filter
pub async fn invariants_handler(
    State(state): State<AppState>,
    Query(q): Query<InvariantsQuery>,
) -> Json<Vec<InvariantResult>> {
    let mut results = state.coordinator.check_invariants();

    if let Some(ref sev) = q.severity {
        let filter_sev = match sev.to_lowercase().as_str() {
            "warning" => InvariantSeverity::Warning,
            "error" => InvariantSeverity::Error,
            "critical" => InvariantSeverity::Critical,
            _ => {
                return Json(results);
            }
        };
        results.retain(|r| r.invariant.severity() == filter_sev);
    }

    Json(results)
}

// ============================================================================
// Event Log (read-only)
// ============================================================================

/// GET /api/v1/events – List WrappedStateEvent with limit, since_sequence, component, realm_id
pub async fn events_list_handler(
    State(state): State<AppState>,
    Query(q): Query<EventsQuery>,
) -> Json<Vec<WrappedStateEvent>> {
    let log = &state.unified_state.event_log;
    let limit = q.limit.unwrap_or(100).min(1000);

    let events = if let Some(since) = q.since_sequence {
        log.events_since(since)
    } else if let Some(ref comp) = q.component {
        let comp_parsed = parse_component(comp);
        log.events_for_component(comp_parsed)
    } else {
        let snap = log.snapshot();
        let since = snap.sequence.saturating_sub(limit as u64);
        log.events_since(since)
    };

    let out: Vec<WrappedStateEvent> = events.into_iter().take(limit as usize).collect();
    Json(out)
}

/// GET /api/v1/events/:sequence – Single WrappedStateEvent by sequence
pub async fn event_by_sequence_handler(
    State(state): State<AppState>,
    Path(sequence): Path<u64>,
) -> impl IntoResponse {
    let log = &state.unified_state.event_log;
    let snap = log.snapshot();
    if sequence >= snap.sequence {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "sequence not found", "sequence": sequence })),
        )
            .into_response();
    }
    let events = log.events_since(0);
    match events.into_iter().find(|e| e.sequence == sequence) {
        Some(ev) => Json(ev).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "event not in buffer", "sequence": sequence })),
        )
            .into_response(),
    }
}

/// GET /api/v1/events/log/snapshot – EventLogSnapshot
pub async fn events_log_snapshot_handler(State(state): State<AppState>) -> impl IntoResponse {
    let snapshot = state.unified_state.event_log_stats();
    Json(snapshot)
}

/// GET /api/v1/events/checkpoints – Last checkpoint (sequence; id/state_hash from event log if available)
#[derive(serde::Serialize)]
pub struct EventsCheckpointResponse {
    pub last_checkpoint_sequence: u64,
    pub current_sequence: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_hash_hex: Option<String>,
}

pub async fn events_checkpoints_handler(State(state): State<AppState>) -> Json<EventsCheckpointResponse> {
    let snapshot = state.unified_state.event_log_stats();
    Json(EventsCheckpointResponse {
        last_checkpoint_sequence: snapshot.last_checkpoint_sequence,
        current_sequence: snapshot.sequence,
        id: None,
        state_hash_hex: None,
    })
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct DeltaQuery {
    /// Merkle-Root (hex) – Deltas ab dem State mit diesem Root
    pub since_root: Option<String>,
    /// Alternativ: Deltas ab dieser Sequenz
    pub since_sequence: Option<u64>,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct StreamQuery {
    /// Komma-getrennte Komponenten (z.B. "trust,eclvm"); leer = alle
    pub components: Option<String>,
}

fn parse_component(s: &str) -> StateComponent {
    match s.to_lowercase().as_str() {
        "identity" => StateComponent::Identity,
        "trust" => StateComponent::Trust,
        "event" => StateComponent::Event,
        "worldformula" | "world_formula" => StateComponent::WorldFormula,
        "consensus" => StateComponent::Consensus,
        "gas" => StateComponent::Gas,
        "mana" => StateComponent::Mana,
        "execution" => StateComponent::Execution,
        "eclvm" => StateComponent::ECLVM,
        "eclpolicy" | "ecl_policy" => StateComponent::ECLPolicy,
        "eclblueprint" | "ecl_blueprint" => StateComponent::ECLBlueprint,
        "anomaly" => StateComponent::Anomaly,
        "diversity" => StateComponent::Diversity,
        "gateway" => StateComponent::Gateway,
        "swarm" => StateComponent::Swarm,
        "realm" => StateComponent::Realm,
        "core" => StateComponent::Trust,
        "peer" => StateComponent::Gateway,
        "protection" => StateComponent::Anomaly,
        "storage" | "kvstore" => StateComponent::KvStore,
        "eventstore" => StateComponent::EventStore,
        "p2p" => StateComponent::Swarm,
        "ui" => StateComponent::UI,
        "api" => StateComponent::API,
        "governance" => StateComponent::Governance,
        "controller" => StateComponent::Controller,
        "datalogic" | "data_logic" => StateComponent::DataLogic,
        "blueprintcomposer" | "blueprint_composer" => StateComponent::BlueprintComposer,
        _ => StateComponent::Event,
    }
}

fn merkle_hash_to_hex(h: &[u8; 32]) -> String {
    hex::encode(h)
}

fn hex_to_merkle_hash(s: &str) -> Option<[u8; 32]> {
    let bytes = hex::decode(s).ok()?;
    if bytes.len() != 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Some(out)
}

// ============================================================================
// Phase 5: Merkle & Delta Sync (Light-Clients)
// ============================================================================

/// GET /api/v1/state/merkle/root – Merkle-Root des aktuellen State
pub async fn state_merkle_root_handler(State(state): State<AppState>) -> impl IntoResponse {
    let root = state.unified_state.merkle_root();
    Json(serde_json::json!({ "root": merkle_hash_to_hex(&root) }))
}

/// GET /api/v1/state/merkle/component/:component – Merkle-Hash einer Komponente
pub async fn state_merkle_component_handler(
    State(state): State<AppState>,
    Path(component_name): Path<String>,
) -> impl IntoResponse {
    let comp = parse_component(&component_name);
    match state.unified_state.merkle_component_hash(comp) {
        Some(hash) => Json(
            serde_json::json!({ "component": component_name, "hash": merkle_hash_to_hex(&hash) }),
        )
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "component hash not found",
                "component": component_name
            })),
        )
            .into_response(),
    }
}

/// GET /api/v1/state/delta?since_root=<hex>&since_sequence=<n> – Delta seit Root/Sequenz
pub async fn state_delta_handler(
    State(state): State<AppState>,
    Query(q): Query<DeltaQuery>,
) -> impl IntoResponse {
    let since_seq = if let Some(ref hex_root) = q.since_root {
        hex_to_merkle_hash(hex_root.trim())
            .and_then(|root| state.unified_state.merkle_sequence_for_root(&root))
            .unwrap_or(0)
    } else {
        q.since_sequence.unwrap_or(0)
    };
    let deltas = state.unified_state.deltas_since(since_seq);
    let current_root = state.unified_state.merkle_root();
    let deltas_json: Vec<serde_json::Value> = deltas
        .iter()
        .map(|d| {
            serde_json::json!({
                "old_root": merkle_hash_to_hex(&d.old_root),
                "new_root": merkle_hash_to_hex(&d.new_root),
                "component": format!("{:?}", d.component),
                "proof_path": d.proof_path.iter().map(merkle_hash_to_hex).collect::<Vec<_>>(),
                "data_base64": base64::engine::general_purpose::STANDARD.encode(&d.data),
                "timestamp_ms": d.timestamp_ms,
                "sequence": d.sequence,
            })
        })
        .collect();
    Json(serde_json::json!({
        "root": merkle_hash_to_hex(&current_root),
        "deltas": deltas_json,
    }))
}

/// GET /api/v1/state/proof/:component – State-Proof für eine Komponente (gegen Root verifizierbar)
pub async fn state_proof_handler(
    State(state): State<AppState>,
    Path(component_name): Path<String>,
) -> impl IntoResponse {
    let comp = parse_component(&component_name);
    let root = state.unified_state.merkle_root();
    match state.unified_state.merkle_component_proof(comp) {
        Some((hash, proof_path)) => Json(serde_json::json!({
            "root": merkle_hash_to_hex(&root),
            "component": component_name,
            "component_hash": merkle_hash_to_hex(&hash),
            "proof_path": proof_path.iter().map(merkle_hash_to_hex).collect::<Vec<_>>(),
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "component proof not found",
                "component": component_name
            })),
        )
            .into_response(),
    }
}

// ============================================================================
// Phase 5: State-Delta-Stream (CQRS / SSE)
// ============================================================================

/// GET /api/v1/state/stream – State-Delta-Subscription (SSE); optional ?components=trust,eclvm
pub async fn state_stream_handler(
    State(state): State<AppState>,
    Query(q): Query<StreamQuery>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.unified_state.subscribe_deltas();
    let stream = BroadcastStream::new(rx);
    let comp_filter: std::sync::Arc<Option<std::collections::HashSet<StateComponent>>> =
        std::sync::Arc::new(
            q.components
                .as_ref()
                .map(|s| s.split(',').map(|c| parse_component(c.trim())).collect()),
        );
    let comp_filter = comp_filter.clone();
    let stream = stream.filter_map(move |r| {
        let delta = r.ok()?;
        if let Some(ref set) = *comp_filter {
            if !set.contains(&delta.component) {
                return None;
            }
        }
        let data = serde_json::to_string(&delta).unwrap_or_default();
        Some(Ok::<_, Infallible>(Event::default().data(data)))
    });
    Sse::new(stream)
}

// ============================================================================
// Metrics
// ============================================================================

/// GET /api/v1/state/metrics – Key-value metrics for scraping
pub async fn state_metrics_handler(State(state): State<AppState>) -> String {
    let snapshot = state.unified_state.snapshot();
    let mut out = String::new();
    out.push_str(&format!("erynoa_health_score {}", snapshot.health_score));
    out.push('\n');
    out.push_str(&format!("erynoa_timestamp_ms {}", snapshot.timestamp_ms));
    out.push('\n');
    out.push_str(&format!("erynoa_uptime_secs {}", snapshot.uptime_secs));
    out.push('\n');
    out.push_str(&format!("erynoa_events_sequence {}", snapshot.event_log.sequence));
    out.push('\n');
    out.push_str(&format!("erynoa_events_buffer_size {}", snapshot.event_log.buffer_size));
    out.push('\n');
    out
}

/// GET /api/v1/state/metrics/eclvm – ECLVM-specific metrics
pub async fn state_metrics_eclvm_handler(State(state): State<AppState>) -> String {
    let snapshot = state.unified_state.snapshot();
    let e = &snapshot.eclvm;
    let mut out = String::new();
    out.push_str(&format!("erynoa_eclvm_policies_compiled {}", e.policies_compiled));
    out.push('\n');
    out.push_str(&format!("erynoa_eclvm_policies_executed {}", e.policies_executed));
    out.push('\n');
    out.push_str(&format!("erynoa_eclvm_policy_success_rate {}", e.policy_success_rate));
    out.push('\n');
    out.push_str(&format!("erynoa_eclvm_total_gas_consumed {}", e.total_gas_consumed));
    out.push('\n');
    out.push_str(&format!("erynoa_eclvm_total_mana_consumed {}", e.total_mana_consumed));
    out.push('\n');
    out.push_str(&format!("erynoa_eclvm_out_of_gas_aborts {}", e.out_of_gas_aborts));
    out.push('\n');
    out
}

/// GET /api/v1/state/metrics/health – Health-related metrics
pub async fn state_metrics_health_handler(State(state): State<AppState>) -> String {
    let report = state.coordinator.aggregate_health();
    let mut out = String::new();
    out.push_str(&format!("erynoa_health_overall_score {}", report.overall_score));
    out.push('\n');
    out.push_str(&format!(
        "erynoa_health_status \"{}\"",
        match report.status {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Warning => "warning",
            HealthStatus::Critical => "critical",
        }
    ));
    out.push('\n');
    out.push_str(&format!("erynoa_health_invariant_score {}", report.invariant_score));
    out.push('\n');
    out.push_str(&format!("erynoa_health_timestamp_ms {}", report.timestamp_ms));
    out.push('\n');
    for (k, v) in &report.module_scores {
        let key = k.replace('-', "_");
        out.push_str(&format!("erynoa_health_module_{} {}", key, v));
        out.push('\n');
    }
    out
}

// ============================================================================
// Warnings
// ============================================================================

/// GET /api/v1/state/warnings – List active warnings
pub async fn state_warnings_list_handler(State(state): State<AppState>) -> Json<Vec<String>> {
    let warnings = state.unified_state.snapshot().warnings.clone();
    Json(warnings)
}

/// DELETE /api/v1/state/warnings – Clear all warnings (prefix "" matches all)
pub async fn state_warnings_clear_all_handler(State(state): State<AppState>) -> impl IntoResponse {
    // clear_warning("") retains nothing because every string starts with ""
    state.unified_state.clear_warning("");
    (StatusCode::NO_CONTENT, ()).into_response()
}

/// DELETE /api/v1/state/warnings/:key – Clear warnings by prefix
pub async fn state_warnings_clear_by_key_handler(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    state.unified_state.clear_warning(&key);
    (StatusCode::NO_CONTENT, ()).into_response()
}

// ============================================================================
// Phase 4: Circuit Breaker & System Mode
// ============================================================================

#[derive(serde::Serialize)]
pub struct StateModeResponse {
    pub mode: SystemMode,
    pub description: String,
}

/// GET /api/v1/state/mode – Current SystemMode + description
pub async fn state_mode_handler(State(state): State<AppState>) -> Json<StateModeResponse> {
    let mode = state.unified_state.system_mode();
    Json(StateModeResponse {
        mode,
        description: mode.description().to_string(),
    })
}

#[derive(serde::Deserialize)]
pub struct StateModeSetBody {
    pub mode: String,
}

/// POST /api/v1/state/mode – Set SystemMode (Ops/Notfall)
pub async fn state_mode_set_handler(
    State(state): State<AppState>,
    Json(body): Json<StateModeSetBody>,
) -> impl IntoResponse {
    let new_mode = match body.mode.to_lowercase().as_str() {
        "normal" => SystemMode::Normal,
        "degraded" => SystemMode::Degraded,
        "emergencyshutdown" | "emergency_shutdown" => SystemMode::EmergencyShutdown,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "invalid_mode",
                    "message": "mode must be Normal, Degraded, or EmergencyShutdown"
                })),
            )
                .into_response();
        }
    };
    state.unified_state.circuit_breaker.set_mode(new_mode);
    Json(StateModeResponse {
        mode: new_mode,
        description: new_mode.description().to_string(),
    })
    .into_response()
}

/// POST /api/v1/state/mode/reset – Reset to Normal
pub async fn state_mode_reset_handler(State(state): State<AppState>) -> Json<StateModeResponse> {
    state.unified_state.reset_circuit_breaker();
    let mode = state.unified_state.system_mode();
    Json(StateModeResponse {
        mode,
        description: mode.description().to_string(),
    })
}

/// GET /api/v1/state/circuit_breaker – CircuitBreakerSnapshot
pub async fn state_circuit_breaker_handler(State(state): State<AppState>) -> Json<CircuitBreakerSnapshot> {
    let snapshot = state.unified_state.circuit_breaker.snapshot();
    Json(snapshot)
}

// ============================================================================
// Phase 4: State Event (Mutation)
// ============================================================================

#[derive(serde::Deserialize)]
pub struct StateEventApplyBody {
    pub event: StateEvent,
    #[serde(default)]
    pub parent_ids: Option<Vec<String>>,
}

/// POST /api/v1/state/event – Apply StateEvent (log_and_apply). Admin/Debug.
pub async fn state_event_apply_handler(
    State(state): State<AppState>,
    Json(body): Json<StateEventApplyBody>,
) -> Json<WrappedStateEvent> {
    let parent_ids = body.parent_ids.unwrap_or_default();
    let wrapped = state.unified_state.log_and_apply(body.event, parent_ids);
    Json(wrapped)
}
