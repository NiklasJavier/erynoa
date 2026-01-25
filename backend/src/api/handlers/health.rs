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
pub struct ServiceStatus {
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub services: ServicesStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_secs: Option<u64>,
}

#[derive(Serialize)]
pub struct ServicesStatus {
    pub database: ServiceStatus,
    pub cache: ServiceStatus,
    pub storage: ServiceStatus,
    pub auth: ServiceStatus,
}

/// GET /health - Liveness probe
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// GET /ready - Readiness probe with detailed status
pub async fn readiness_check(State(state): State<AppState>) -> (StatusCode, Json<ReadinessResponse>) {
    use std::time::Instant;
    
    // Database check with latency
    let db_start = Instant::now();
    let db_status = match state.db.ping().await {
        Ok(_) => ServiceStatus {
            status: "connected",
            latency_ms: Some(db_start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => ServiceStatus {
            status: "disconnected",
            latency_ms: None,
            error: Some(e.to_string()),
        },
    };

    // Cache check with latency
    let cache_start = Instant::now();
    let cache_status = match state.cache.ping().await {
        Ok(_) => ServiceStatus {
            status: "connected",
            latency_ms: Some(cache_start.elapsed().as_millis() as u64),
            error: None,
        },
        Err(e) => ServiceStatus {
            status: "disconnected",
            latency_ms: None,
            error: Some(e.to_string()),
        },
    };

    // Auth/ZITADEL check
    let auth_status = if let Some(ref validator) = state.jwt_validator {
        if validator.is_healthy().await {
            ServiceStatus {
                status: "connected",
                latency_ms: None,
                error: None,
            }
        } else {
            ServiceStatus {
                status: "unhealthy",
                latency_ms: None,
                error: Some("JWKS fetch failed".to_string()),
            }
        }
    } else {
        ServiceStatus {
            status: "disabled",
            latency_ms: None,
            error: None,
        }
    };

    // Storage/MinIO check
    let storage_start = std::time::Instant::now();
    let storage_status = if let Some(ref storage) = state.storage {
        match storage.ping().await {
            Ok(_) => ServiceStatus {
                status: "connected",
                latency_ms: Some(storage_start.elapsed().as_millis() as u64),
                error: None,
            },
            Err(e) => ServiceStatus {
                status: "disconnected",
                latency_ms: None,
                error: Some(e.to_string()),
            },
        }
    } else {
        ServiceStatus {
            status: "disabled",
            latency_ms: None,
            error: None,
        }
    };

    let healthy = db_status.status == "connected" 
        && cache_status.status == "connected"
        && (auth_status.status == "connected" || auth_status.status == "disabled")
        && (storage_status.status == "connected" || storage_status.status == "disabled");

    let uptime = state.started_at.map(|t| t.elapsed().as_secs());

    (
        if healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE },
        Json(ReadinessResponse {
            status: if healthy { "ready" } else { "not_ready" },
            services: ServicesStatus {
                database: db_status,
                cache: cache_status,
                storage: storage_status,
                auth: auth_status,
            },
            uptime_secs: uptime,
        }),
    )
}
