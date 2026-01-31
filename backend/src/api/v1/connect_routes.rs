//! Connect-RPC Routes
//!
//! Routes für Connect-RPC/gRPC-Web Services
//!
//! Aktuell verfügbare Services:
//! - Health: Health checks und Readiness probes
//! - Info: System-Informationen

use crate::server::AppState;
use axum::Router;

// Import generated service types
use crate::gen::erynoa::v1::{HealthService, InfoService};

// Import handler functions
use crate::api::v1::health::{health_check_handler, ready_check_handler};
use crate::api::v1::info::get_info_handler;

/// Erstellt Router für Connect-RPC Services
///
/// Connect-RPC Services werden unter /api/v1/connect/ bereitgestellt
/// z.B. /api/v1/connect/erynoa.v1.HealthService/Check
pub fn create_connect_routes(_state: AppState) -> Router<AppState> {
    // Start with empty router
    let router = Router::new();

    // Apply Health Service handlers
    let router = HealthService::check(health_check_handler)(router);
    let router = HealthService::ready(ready_check_handler)(router);

    // Apply Info Service handlers
    let router = InfoService::get_info(get_info_handler)(router);

    router
}
