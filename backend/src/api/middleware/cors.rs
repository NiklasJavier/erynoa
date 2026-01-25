//! CORS Middleware
//!
//! Konfiguriert Cross-Origin Resource Sharing basierend auf Environment

use crate::server::AppState;
use axum::{
    http::{HeaderValue, Method},
    Router,
};
use std::time::Duration;
use tower_http::cors::CorsLayer;

/// Erstellt CORS-Layer basierend auf Environment
pub fn build_cors(state: &AppState) -> CorsLayer {
    if state.config.application.environment.is_production() {
        let origin = state.config.application.frontend_url.clone();
        CorsLayer::new()
            .allow_origin(origin.as_str().parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
            .allow_credentials(true)
            .max_age(Duration::from_secs(86400))
    } else {
        CorsLayer::very_permissive()
    }
}
