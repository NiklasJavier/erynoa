//! API Routes
//!
//! Haupt-Router der alle Feature-Router zusammenführt

use crate::server::AppState;
use axum::{middleware::from_fn, Router};

use super::constants::API_VERSION;
use super::middleware::{build_cors, logging_middleware};
use super::v1::{health, info, storage, users};

#[cfg(feature = "connect")]
use super::v1::connect_routes;

/// Erstellt den Haupt-Router mit allen API-Features
pub fn create_router(state: AppState) -> Router {
    let cors = build_cors(&state);

    // Public routes (keine Auth erforderlich)
    let public_routes = Router::new()
        .merge(health::create_health_routes())
        .merge(info::create_info_routes());

    // Protected routes (Auth erforderlich)
    // Note: Auth-Middleware wird aktuell nicht als Layer verwendet,
    // da Claims direkt als Extractor in Handlern verwendet werden.
    // Die auth_middleware kann später für automatische Token-Validierung
    // auf bestimmten Routen verwendet werden.
    let protected_routes = Router::new()
        .merge(users::create_users_routes())
        .merge(storage::create_storage_routes());

    // Connect-RPC routes (gRPC-Web)
    #[cfg(feature = "connect")]
    let connect_routes = connect_routes::create_connect_routes(state.clone());
    
    // Kombiniere alle Routen
    let api = Router::new()
        .merge(public_routes)
        .merge(protected_routes);
    
    // Add Connect-RPC routes if enabled
    #[cfg(feature = "connect")]
    let api = api.nest("/connect", connect_routes);

    // Haupt-Router mit Middleware und State
    Router::new()
        .nest(API_VERSION, api)
        .layer(cors)
        .layer(from_fn(logging_middleware))
        .with_state(state)
}
