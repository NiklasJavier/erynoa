//! API Routes
//!
//! REST-basierte API für Health-Checks, Info und WebAuthn

use crate::server::AppState;
use axum::{
    middleware::from_fn,
    routing::{get, post},
    Router,
};
// TODO: Rate Limiting mit tower_governor 0.4 korrekt implementieren
// use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

use super::constants::API_VERSION;
use super::middleware::{build_cors, logging_middleware};
use super::v1::auth::handlers as auth_handlers;
use super::v1::rest_handlers;

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

    // API Router mit REST routes
    let api = Router::new().merge(rest_routes).nest("/auth", auth_routes);

    // Haupt-Router mit Middleware und State
    Router::new()
        .nest(API_VERSION, api)
        .layer(cors)
        // .layer(rate_limit_layer)  // ⚡ Rate Limiting - TODO: Re-enable after fixing API usage
        .layer(from_fn(logging_middleware))
        .with_state(state)
}
