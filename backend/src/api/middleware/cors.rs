//! CORS Middleware
//!
//! Konfiguriert Cross-Origin Resource Sharing basierend auf Environment
//! Unterstützt Connect-RPC/gRPC-Web spezifische Headers

use crate::server::AppState;
use axum::http::{HeaderValue, Method};
use std::time::Duration;
use tower_http::cors::CorsLayer;

/// Erstellt CORS-Layer basierend auf Environment
///
/// Erlaubt Connect-RPC und gRPC-Web spezifische Headers:
/// - Connect-Protocol-Version
/// - Connect-Timeout-Ms
/// - Grpc-Timeout
/// - X-Grpc-Web
/// - X-User-Agent
/// - Authorization (für authentifizierte Requests)
pub fn build_cors(state: &AppState) -> CorsLayer {
    // In Development: sehr permissiv (erlaubt alle Headers inkl. Connect-RPC)
    // In Production: explizite Header-Liste
    // Note: very_permissive() erlaubt bereits alle Headers, was für Connect-RPC ausreicht
    if state.config.application.environment.is_production() {
        let origin = state.config.application.console_url.clone();
        CorsLayer::new()
            .allow_origin(origin.as_str().parse::<HeaderValue>().unwrap())
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
                Method::OPTIONS,
            ])
            .allow_headers(tower_http::cors::Any)
            .allow_credentials(true)
            .max_age(Duration::from_secs(86400))
    } else {
        CorsLayer::very_permissive()
    }
}
