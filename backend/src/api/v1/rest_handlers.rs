//! REST Fallback Handlers
//!
//! Simple REST endpoints for health checks and info.
//! These provide compatibility for load balancers, Kubernetes probes,
//! and simple HTTP clients that don't support Connect-RPC.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::time::Instant;

use crate::config::version::VERSION;
use crate::server::AppState;

// ============================================================================
// Response Types
// ============================================================================

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
}

#[derive(Serialize)]
pub struct ServiceStatusJson {
    pub healthy: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<i64>,
}

#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    pub services: ReadyServices,
}

#[derive(Serialize)]
pub struct ReadyServices {
    pub database: ServiceStatusJson,
    pub cache: ServiceStatusJson,
    pub auth: ServiceStatusJson,
    pub storage: ServiceStatusJson,
}

#[derive(Serialize)]
pub struct InfoResponse {
    pub version: &'static str,
    pub environment: String,
    pub auth_issuer: String,
    pub auth_client_id: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub services: Vec<ServiceInfo>,
}

#[derive(Serialize)]
pub struct ServiceInfo {
    pub name: &'static str,
    pub status: &'static str,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/v1/health - Liveness probe
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: VERSION,
    })
}

/// GET /api/v1/ready - Readiness probe
pub async fn ready_handler(State(state): State<AppState>) -> impl IntoResponse {
    // Database check
    let db_start = Instant::now();
    let db_healthy = state.db.ping().await.is_ok();
    let db_latency = db_start.elapsed().as_millis() as i64;

    // Cache check
    let cache_start = Instant::now();
    let cache_healthy = state.cache.ping().await.is_ok();
    let cache_latency = cache_start.elapsed().as_millis() as i64;

    // Auth check
    let auth_healthy = if let Some(ref validator) = state.jwt_validator {
        validator.is_healthy().await
    } else {
        true // disabled is considered healthy
    };

    // Storage check
    let storage_start = Instant::now();
    let storage_healthy = if let Some(ref storage) = state.storage {
        storage.ping().await.is_ok()
    } else {
        true // disabled is considered healthy
    };
    let storage_latency = storage_start.elapsed().as_millis() as i64;

    let all_healthy = db_healthy && cache_healthy && auth_healthy && storage_healthy;

    let response = ReadyResponse {
        status: if all_healthy { "ready" } else { "not_ready" },
        services: ReadyServices {
            database: ServiceStatusJson {
                healthy: db_healthy,
                message: if db_healthy {
                    "connected".to_string()
                } else {
                    "disconnected".to_string()
                },
                latency_ms: Some(db_latency),
            },
            cache: ServiceStatusJson {
                healthy: cache_healthy,
                message: if cache_healthy {
                    "connected".to_string()
                } else {
                    "disconnected".to_string()
                },
                latency_ms: Some(cache_latency),
            },
            auth: ServiceStatusJson {
                healthy: auth_healthy || state.jwt_validator.is_none(),
                message: if state.jwt_validator.is_some() {
                    if auth_healthy {
                        "connected".to_string()
                    } else {
                        "unhealthy".to_string()
                    }
                } else {
                    "disabled".to_string()
                },
                latency_ms: None,
            },
            storage: ServiceStatusJson {
                healthy: storage_healthy || state.storage.is_none(),
                message: if state.storage.is_some() {
                    if storage_healthy {
                        "connected".to_string()
                    } else {
                        "disconnected".to_string()
                    }
                } else {
                    "disabled".to_string()
                },
                latency_ms: Some(storage_latency),
            },
        },
    };

    if all_healthy {
        (StatusCode::OK, Json(response))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(response))
    }
}

/// GET /api/v1/info - Application info
pub async fn info_handler(State(state): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        version: VERSION,
        environment: state.config.application.environment.as_str().to_string(),
        auth_issuer: state.config.auth.issuer.clone(),
        auth_client_id: state.config.auth.console_client_id.clone(),
    })
}

/// GET /api/v1/status - Service status overview
pub async fn status_handler(State(state): State<AppState>) -> Json<StatusResponse> {
    let db_ok = state.db.ping().await.is_ok();
    let cache_ok = state.cache.ping().await.is_ok();

    Json(StatusResponse {
        services: vec![
            ServiceInfo {
                name: "database",
                status: if db_ok { "up" } else { "down" },
            },
            ServiceInfo {
                name: "cache",
                status: if cache_ok { "up" } else { "down" },
            },
            ServiceInfo {
                name: "api",
                status: "up",
            },
        ],
    })
}
