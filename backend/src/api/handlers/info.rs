//! Info Handler

use axum::{extract::State, Json};
use serde::Serialize;

use crate::server::AppState;

#[derive(Serialize)]
pub struct InfoResponse {
    pub version: &'static str,
    pub environment: String,
    pub auth_issuer: String,
    pub auth_client_id: String,
    pub frontend_url: String,
    pub api_url: String,
}

/// GET /info - Public config for frontend
pub async fn get_info(State(state): State<AppState>) -> Json<InfoResponse> {
    Json(InfoResponse {
        version: env!("CARGO_PKG_VERSION"),
        environment: state.config.application.environment.as_str().to_string(),
        auth_issuer: state.config.auth.issuer.clone(),
        auth_client_id: state.config.auth.frontend_client_id.clone(),
        frontend_url: state.config.application.frontend_url.clone(),
        api_url: state.config.application.api_url.clone(),
    })
}
