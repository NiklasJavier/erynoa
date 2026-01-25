//! Info Routes

use axum::{routing::get, Router};

use crate::server::AppState;

use super::handler;

/// Erstellt Router für Info Endpoints
pub fn create_info_routes() -> Router<AppState> {
    Router::new()
        .route("/info", get(handler::get_info))
        .route("/status", get(handler::get_status))
}
