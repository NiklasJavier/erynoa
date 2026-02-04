//! API Routes
//!
//! REST-basierte API für Health-Checks, Info und WebAuthn

use crate::server::AppState;
use axum::{
    middleware::from_fn,
    routing::{delete, get, post},
    Router,
};
// TODO: Rate Limiting mit tower_governor 0.4 korrekt implementieren
// use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

use super::constants::API_VERSION;
use super::middleware::{build_cors, logging_middleware};
use super::v1::auth::handlers as auth_handlers;
use super::v1::debug_handlers;
use super::v1::production_handlers;
use super::v1::rest_handlers;
use super::v1::state_handlers;

/// Erstellt den Haupt-Router mit REST-API
///
/// REST endpoints für Health-Checks und Info sind unter /api/v1/* verfügbar.
/// Auth endpoints für Passkey/WebAuthn sind unter /api/v1/auth/* verfügbar.
pub fn create_router(state: AppState) -> Router {
    let cors = build_cors(&state);

    // ⚡ PRODUCTION HARDENING: Rate Limiting
    // TODO: Rate Limiting mit tower_governor 0.4 korrekt implementieren
    // Verhindert DoS-Attacken und Traffic-Floods
    // - 50 Requests pro Sekunde pro IP (normale Nutzung)
    // - Burst von 100 für kurzzeitige Spitzen erlaubt
    // Temporarily disabled until correct API usage is determined

    // REST routes for health checks and info
    // These are simple endpoints for load balancers, K8s probes, etc.
    let rest_routes = Router::new()
        .route("/health", get(rest_handlers::health_handler))
        .route("/ready", get(rest_handlers::ready_handler))
        .route("/info", get(rest_handlers::info_handler))
        .route("/status", get(rest_handlers::status_handler));

    // Auth routes for Passkey/WebAuthn authentication
    // These endpoints handle challenge generation, registration, and verification
    let auth_routes = Router::new()
        .route("/challenge", get(auth_handlers::get_challenge))
        .route("/passkey/register", post(auth_handlers::register_passkey))
        .route("/passkey/verify", post(auth_handlers::verify_passkey));

    // Phase 1 & 4 & 5: State – Snapshots, Metrics, Warnings, Mode, Circuit Breaker, Event, Merkle/Delta/Stream
    let state_routes = Router::new()
        .route("/snapshot", get(state_handlers::state_snapshot_handler))
        .route("/metrics", get(state_handlers::state_metrics_handler))
        .route("/metrics/eclvm", get(state_handlers::state_metrics_eclvm_handler))
        .route("/metrics/health", get(state_handlers::state_metrics_health_handler))
        .route("/warnings", get(state_handlers::state_warnings_list_handler))
        .route("/warnings", delete(state_handlers::state_warnings_clear_all_handler))
        .route("/warnings/:key", delete(state_handlers::state_warnings_clear_by_key_handler))
        .route("/mode/reset", post(state_handlers::state_mode_reset_handler))
        .route("/mode", get(state_handlers::state_mode_handler))
        .route("/mode", post(state_handlers::state_mode_set_handler))
        .route("/circuit_breaker", get(state_handlers::state_circuit_breaker_handler))
        .route("/event", post(state_handlers::state_event_apply_handler))
        // Phase 5: Merkle & Delta Sync, State-Stream
        .route("/merkle/root", get(state_handlers::state_merkle_root_handler))
        .route("/merkle/component/:component", get(state_handlers::state_merkle_component_handler))
        .route("/delta", get(state_handlers::state_delta_handler))
        .route("/proof/:component", get(state_handlers::state_proof_handler))
        .route("/stream", get(state_handlers::state_stream_handler))
        .route("/:component_name", get(state_handlers::state_component_handler));

    let health_routes = Router::new()
        .route("/state", get(state_handlers::health_state_handler))
        .route("/state/detail", get(state_handlers::health_state_detail_handler))
        .route("/aggregate", get(state_handlers::health_aggregate_handler));

    let events_routes = Router::new()
        .route("/log/snapshot", get(state_handlers::events_log_snapshot_handler))
        .route("/checkpoints", get(state_handlers::events_checkpoints_handler))
        .route("/:sequence", get(state_handlers::event_by_sequence_handler))
        .route("/", get(state_handlers::events_list_handler));

    // Phase 4: Debug – Replay, Checkpoint
    let debug_routes = Router::new()
        .route("/replay/checkpoint", post(debug_handlers::debug_replay_checkpoint_handler))
        .route("/replay", post(debug_handlers::debug_replay_handler))
        .route("/checkpoint", post(debug_handlers::debug_checkpoint_handler));

    let invariants_routes = Router::new().route("/", get(state_handlers::invariants_handler));

    // Phase 2: Produktion Kern – Crossing, Trust, Identity, Realm, ECL (stubs)
    let crossing_routes = Router::new()
        .route("/validate", post(production_handlers::crossing_validate_handler))
        .route("/stats", get(production_handlers::crossing_stats_handler));

    let trust_routes = Router::new()
        .route("/update", post(production_handlers::trust_update_handler))
        .route("/:did", get(production_handlers::trust_get_handler));

    let identity_routes = Router::new()
        .route("/root", get(production_handlers::identity_root_handler))
        .route("/:did", get(production_handlers::identity_get_handler));

    let realms_routes = Router::new()
        .route("/:realm_id/ecl", get(production_handlers::realm_ecl_handler))
        .route("/:realm_id/members", post(production_handlers::realm_members_handler))
        .route("/:realm_id", get(production_handlers::realm_get_handler))
        .route("/", get(production_handlers::realms_list_handler))
        .route("/", post(production_handlers::realm_create_handler));

    let ecl_routes = Router::new()
        .route("/run", post(production_handlers::ecl_run_handler))
        .route("/api/:route_id", post(production_handlers::ecl_api_handler))
        .route("/ui/:component_id", post(production_handlers::ecl_ui_handler))
        .route("/controller/:key", post(production_handlers::ecl_controller_handler));

    // Phase 3: Governance, Controller, Intent, Saga
    let governance_routes = Router::new()
        .route("/proposals/:id/vote", post(production_handlers::governance_proposals_vote_handler))
        .route("/proposals", get(production_handlers::governance_proposals_list_handler))
        .route("/proposals", post(production_handlers::governance_proposals_create_handler));

    let controller_routes = Router::new()
        .route("/check", post(production_handlers::controller_check_handler))
        .route("/permissions", get(production_handlers::controller_permissions_handler));

    let intent_routes = Router::new().route("/parse", post(production_handlers::intent_parse_handler));

    let saga_routes = Router::new()
        .route("/compose", post(production_handlers::saga_compose_handler))
        .route("/execute", post(production_handlers::saga_execute_handler))
        .route("/stats", get(production_handlers::saga_stats_handler));

    // API Router mit REST routes, Auth, State, Health, Events, Invariants, Phase 2, Phase 3
    let api = Router::new()
        .merge(rest_routes)
        .nest("/auth", auth_routes)
        .nest("/state", state_routes)
        .nest("/health", health_routes)
        .nest("/events", events_routes)
        .nest("/invariants", invariants_routes)
        .nest("/crossing", crossing_routes)
        .nest("/trust", trust_routes)
        .nest("/identity", identity_routes)
        .nest("/realms", realms_routes)
        .nest("/ecl", ecl_routes)
        .nest("/governance", governance_routes)
        .nest("/controller", controller_routes)
        .nest("/intent", intent_routes)
        .nest("/saga", saga_routes)
        .nest("/debug", debug_routes);

    // Haupt-Router mit Middleware und State
    Router::new()
        .nest(API_VERSION, api)
        .layer(cors)
        // .layer(rate_limit_layer)  // ⚡ Rate Limiting - TODO: Re-enable after fixing API usage
        .layer(from_fn(logging_middleware))
        .with_state(state)
}
