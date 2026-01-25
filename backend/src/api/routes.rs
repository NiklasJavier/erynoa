//! API Routes

use crate::server::AppState;
use axum::{
    http::{HeaderValue, Method},
    routing::get,
    Router,
};
use std::time::Duration;
use tower_http::cors::CorsLayer;

use super::handlers;

pub fn create_router(state: AppState) -> Router {
    let cors = build_cors(&state);

    // Build API routes - explicitly typed as Router<AppState>
    let api: Router<AppState> = Router::new()
        // Public routes (no auth)
        .route("/health", get(handlers::health::health_check))
        .route("/ready", get(handlers::health::readiness_check))
        .route("/info", get(handlers::info::get_info))
        .route("/status", get(handlers::status::get_status))
        // Protected routes
        .route("/me", get(handlers::users::get_current_user))
        .route("/users", get(handlers::users::list_users))
        .route("/users/:id", get(handlers::users::get_user));

    // Wrap with layers and provide state
    Router::new()
        .nest("/api/v1", api)
        .layer(cors)
        .with_state(state)
}

fn build_cors(state: &AppState) -> CorsLayer {
    if state.config.application.environment.is_production() {
        let origin = state.config.application.frontend_url.clone();
        CorsLayer::new()
            .allow_origin(origin.as_str().parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_credentials(true)
            .max_age(Duration::from_secs(86400))
    } else {
        CorsLayer::very_permissive()
    }
}
