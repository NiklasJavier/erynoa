//! Health Check Models
//!
//! Request/Response Types für Health Endpoints

use serde::Serialize;

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
