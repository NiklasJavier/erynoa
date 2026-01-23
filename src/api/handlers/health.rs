//! Health Check Handlers

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

use crate::server::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub database: &'static str,
    pub cache: &'static str,
}

/// GET /health - Liveness probe
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// GET /ready - Readiness probe
pub async fn readiness_check(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    let db_status = match state.db.ping().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let cache_status = match state.cache.ping().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let healthy = db_status == "connected" && cache_status == "connected";

    (
        if healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE },
        Json(ReadinessResponse {
            status: if healthy { "ready" } else { "not_ready" },
            database: db_status,
            cache: cache_status,
        }),
    )
}
