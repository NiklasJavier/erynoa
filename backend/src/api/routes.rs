//! API Routes
//!
//! Haupt-Router der alle Connect-RPC Services und REST-Fallbacks zusammenführt

use crate::server::AppState;
use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::get,
    Router,
};
// TODO: Rate Limiting mit tower_governor 0.4 korrekt implementieren
// use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

use super::constants::API_VERSION;
use super::middleware::{build_cors, frontend_origin_middleware, logging_middleware};
use super::v1::rest_handlers;

#[cfg(feature = "connect")]
use super::v1::connect_routes;

/// Erstellt den Haupt-Router mit allen Connect-RPC Services
///
/// REST fallback endpoints für Health-Checks und Info sind unter /api/v1/* verfügbar.
/// Connect-RPC Services sind unter /api/v1/connect/* verfügbar.
pub fn create_router(state: AppState) -> Router {
    let cors = build_cors(&state);

    // ⚡ PRODUCTION HARDENING: Rate Limiting
    // TODO: Rate Limiting mit tower_governor 0.4 korrekt implementieren
    // Verhindert DoS-Attacken und Traffic-Floods
    // - 50 Requests pro Sekunde pro IP (normale Nutzung)
    // - Burst von 100 für kurzzeitige Spitzen erlaubt
    // Temporarily disabled until correct API usage is determined

    // REST fallback routes for health checks and info
    // These are simple endpoints for load balancers, K8s probes, etc.
    let rest_routes = Router::new()
        .route("/health", get(rest_handlers::health_handler))
        .route("/ready", get(rest_handlers::ready_handler))
        .route("/info", get(rest_handlers::info_handler))
        .route("/status", get(rest_handlers::status_handler));

    // Connect-RPC routes (gRPC-Web) - Primary API
    #[cfg(feature = "connect")]
    let connect_routes = connect_routes::create_connect_routes(state.clone());

    // API Router mit REST und Connect-RPC routes
    #[cfg(feature = "connect")]
    let api = Router::new()
        .merge(rest_routes)
        .nest("/connect", connect_routes);

    #[cfg(not(feature = "connect"))]
    let api = rest_routes;

    // Haupt-Router mit Middleware und State
    // Note: frontend_origin_middleware needs State, so it must be applied after with_state
    Router::new()
        .nest(API_VERSION, api)
        .layer(cors)
        // .layer(rate_limit_layer)  // ⚡ Rate Limiting - TODO: Re-enable after fixing API usage
        .layer(from_fn_with_state(
            state.clone(),
            frontend_origin_middleware,
        ))
        .layer(from_fn(logging_middleware))
        .with_state(state)
}
