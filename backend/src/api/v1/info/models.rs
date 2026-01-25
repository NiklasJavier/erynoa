//! Info Models

use serde::Serialize;

#[derive(Serialize)]
pub struct InfoResponse {
    pub version: &'static str,
    pub environment: String,
    pub auth_issuer: String,
    pub auth_client_id: String,
    pub frontend_url: String,
    pub api_url: String,
}

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
