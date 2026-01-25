//! Health Check Routes

use axum::{routing::get, Router};

use crate::server::AppState;

use super::handler;

/// Erstellt Router für Health-Check Endpoints
pub fn create_health_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handler::health_check))
        .route("/ready", get(handler::readiness_check))
}
