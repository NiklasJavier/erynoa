//! API Routes
//!
//! Haupt-Router der alle Connect-RPC Services zusammenführt

use crate::server::AppState;
use axum::{middleware::{from_fn, from_fn_with_state}, Router};

use super::constants::API_VERSION;
use super::middleware::{build_cors, frontend_origin_middleware, logging_middleware};

#[cfg(feature = "connect")]
use super::v1::connect_routes;

/// Erstellt den Haupt-Router mit allen Connect-RPC Services
/// 
/// REST endpoints wurden entfernt. Alle APIs sind jetzt über Connect-RPC verfügbar.
pub fn create_router(state: AppState) -> Router {
    let cors = build_cors(&state);

    // Connect-RPC routes (gRPC-Web) - Primary API
    #[cfg(feature = "connect")]
    let connect_routes = connect_routes::create_connect_routes(state.clone());
    
    // API Router mit Connect-RPC routes
    #[cfg(feature = "connect")]
    let api = Router::new().nest("/connect", connect_routes);
    
    #[cfg(not(feature = "connect"))]
    let api = Router::new();

    // Haupt-Router mit Middleware und State
    // Note: frontend_origin_middleware needs State, so it must be applied after with_state
    Router::new()
        .nest(API_VERSION, api)
        .layer(cors)
        .layer(from_fn_with_state(state.clone(), frontend_origin_middleware))
        .layer(from_fn(logging_middleware))
        .with_state(state)
}
