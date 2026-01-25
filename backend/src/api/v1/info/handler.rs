//! Info Handler

use axum::Json;

use crate::config::version::VERSION;
use crate::config::constants::service_urls;
use crate::server::AppState;
use axum::extract::State;

use super::models::{InfoResponse, ServiceStatus, StatusResponse};

/// GET /info - Public config for frontend
pub async fn get_info(State(state): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        version: VERSION,
        environment: state.config.application.environment.as_str().to_string(),
        auth_issuer: state.config.auth.issuer.clone(),
        auth_client_id: state.config.auth.frontend_client_id.clone(),
        frontend_url: state.config.application.frontend_url.clone(),
        api_url: state.config.application.api_url.clone(),
    })
}

/// GET /status - Service status for dashboard
pub async fn get_status() -> Json<StatusResponse> {
    let services = vec![
        ServiceStatus {
            name: "API Server".to_string(),
            status: "online".to_string(),
            description: "Backend REST API".to_string(),
            url: Some(service_urls::API.to_string()),
        },
        ServiceStatus {
            name: "Database".to_string(),
            status: "online".to_string(),
            description: "PostgreSQL Database".to_string(),
            url: Some("postgresql://db:5432".to_string()), // Docker internal URL
        },
        ServiceStatus {
            name: "Cache".to_string(),
            status: "online".to_string(),
            description: "DragonflyDB Cache".to_string(),
            url: Some("redis://cache:6379".to_string()), // Docker internal URL
        },
        ServiceStatus {
            name: "S3 Storage".to_string(),
            status: "online".to_string(),
            description: "MinIO Object Storage".to_string(),
            url: Some("http://minio:9000".to_string()), // Docker internal URL
        },
        ServiceStatus {
            name: "Authentication".to_string(),
            status: "online".to_string(),
            description: "ZITADEL Auth Service".to_string(),
            url: Some("http://zitadel:8080".to_string()), // Docker internal URL
        },
    ];

    Json(StatusResponse { services })
}
