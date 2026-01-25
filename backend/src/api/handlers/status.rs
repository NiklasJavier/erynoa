//! Status Handler

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ServiceStatus {
    pub name: String,
    pub status: String,
    pub description: String,
    pub url: Option<String>,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub services: Vec<ServiceStatus>,
}

/// GET /status - Service status for dashboard
pub async fn get_status() -> Json<StatusResponse> {
    let services = vec![
        ServiceStatus {
            name: "API Server".to_string(),
            status: "online".to_string(),
            description: "Backend REST API".to_string(),
            url: Some("http://localhost:3000".to_string()),
        },
        ServiceStatus {
            name: "Database".to_string(),
            status: "online".to_string(),
            description: "PostgreSQL Database".to_string(),
            url: Some("postgresql://db:5432".to_string()),
        },
        ServiceStatus {
            name: "Cache".to_string(),
            status: "online".to_string(),
            description: "DragonflyDB Cache".to_string(),
            url: Some("redis://cache:6379".to_string()),
        },
        ServiceStatus {
            name: "S3 Storage".to_string(),
            status: "online".to_string(),
            description: "MinIO Object Storage".to_string(),
            url: Some("http://minio:9000".to_string()),
        },
        ServiceStatus {
            name: "Authentication".to_string(),
            status: "online".to_string(),
            description: "ZITADEL Auth Service".to_string(),
            url: Some("http://zitadel:8080".to_string()),
        },
    ];

    Json(StatusResponse { services })
}
