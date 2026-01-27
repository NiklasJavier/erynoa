//! Connect-RPC Health Service Implementation

use axum::extract::State;
use std::time::Instant;

use crate::server::AppState;
use crate::gen::erynoa::v1::{
    CheckRequest, CheckResponse, ReadyRequest, ReadyResponse,
    ServiceStatus,
};

/// Health Check Handler
/// 
/// This function signature matches what axum-connect expects for automatic
/// RpcHandlerUnary implementation: (State<S>, Request) -> Response
pub async fn health_check_handler(
    _state: State<AppState>,
    _request: CheckRequest,
) -> CheckResponse {
    CheckResponse {
        status: CheckResponse::ServingStatus::Serving as i32,
    }
}

/// Ready Check Handler
pub async fn ready_check_handler(
    state: State<AppState>,
    _request: ReadyRequest,
) -> ReadyResponse {
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
    
    let ready = db_healthy && cache_healthy && (auth_healthy || state.jwt_validator.is_none()) 
        && (storage_healthy || state.storage.is_none());
    
    ReadyResponse {
        ready,
        database: Some(ServiceStatus {
            healthy: db_healthy,
            message: if db_healthy { "connected".to_string() } else { "disconnected".to_string() },
            latency_ms: db_latency,
        }),
        cache: Some(ServiceStatus {
            healthy: cache_healthy,
            message: if cache_healthy { "connected".to_string() } else { "disconnected".to_string() },
            latency_ms: cache_latency,
        }),
        auth: Some(ServiceStatus {
            healthy: auth_healthy || state.jwt_validator.is_none(),
            message: if state.jwt_validator.is_some() {
                if auth_healthy { "connected".to_string() } else { "unhealthy".to_string() }
            } else {
                "disabled".to_string()
            },
            latency_ms: 0,
        }),
        storage: Some(ServiceStatus {
            healthy: storage_healthy || state.storage.is_none(),
            message: if state.storage.is_some() {
                if storage_healthy { "connected".to_string() } else { "disconnected".to_string() }
            } else {
                "disabled".to_string()
            },
            latency_ms: storage_latency,
        }),
    }
}
