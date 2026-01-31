//! Connect-RPC Health Service Implementation

use axum::extract::State;
use std::time::Instant;

use crate::gen::erynoa::v1::{
    check_response, CheckRequest, CheckResponse, ReadyRequest, ReadyResponse, ServiceStatus,
};
use crate::server::AppState;

/// Health Check Handler
///
/// This function signature matches what axum-connect expects for automatic
/// RpcHandlerUnary implementation: (State<S>, Request) -> Response
pub async fn health_check_handler(
    _state: State<AppState>,
    _request: CheckRequest,
) -> CheckResponse {
    CheckResponse {
        status: check_response::ServingStatus::Serving as i32,
    }
}

/// Ready Check Handler
///
/// Prüft ob der dezentrale Storage (Fjall) bereit ist.
pub async fn ready_check_handler(state: State<AppState>, _request: ReadyRequest) -> ReadyResponse {
    // Dezentraler Storage check
    let storage_start = Instant::now();
    let storage_healthy = state.storage.ping().await.is_ok();
    let storage_latency = storage_start.elapsed().as_millis() as i64;

    ReadyResponse {
        ready: storage_healthy,
        database: Some(ServiceStatus {
            healthy: true,
            message: "using decentralized storage".to_string(),
            latency_ms: 0,
        }),
        cache: Some(ServiceStatus {
            healthy: true,
            message: "not needed (embedded storage)".to_string(),
            latency_ms: 0,
        }),
        auth: Some(ServiceStatus {
            healthy: true,
            message: "using DID-Auth".to_string(),
            latency_ms: 0,
        }),
        storage: Some(ServiceStatus {
            healthy: storage_healthy,
            message: if storage_healthy {
                "decentralized storage ready".to_string()
            } else {
                "storage unavailable".to_string()
            },
            latency_ms: storage_latency,
        }),
    }
}

