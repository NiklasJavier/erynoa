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
    pub storage: ServiceStatusJson,
}

#[derive(Serialize)]
pub struct InfoResponse {
    pub version: &'static str,
    pub environment: String,
    pub auth_method: &'static str,
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
    // Storage check (dezentral Fjall)
    let storage_start = Instant::now();
    let storage_healthy = state.storage.ping().await.is_ok();
    let storage_latency = storage_start.elapsed().as_millis() as i64;

    let response = ReadyResponse {
        status: if storage_healthy { "ready" } else { "not_ready" },
        services: ReadyServices {
            storage: ServiceStatusJson {
                healthy: storage_healthy,
                message: if storage_healthy {
                    "decentralized".to_string()
                } else {
                    "unavailable".to_string()
                },
                latency_ms: Some(storage_latency),
            },
        },
    };

    if storage_healthy {
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
        auth_method: "DID-Auth",
    })
}

/// GET /api/v1/status - Service status overview
pub async fn status_handler(State(state): State<AppState>) -> Json<StatusResponse> {
    let storage_ok = state.storage.ping().await.is_ok();

    Json(StatusResponse {
        services: vec![
            ServiceInfo {
                name: "storage",
                status: if storage_ok { "up" } else { "down" },
            },
            ServiceInfo {
                name: "api",
                status: "up",
            },
        ],
    })
}
