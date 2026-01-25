//! User Routes

use axum::{routing::get, Router};

use crate::server::AppState;

use super::handler;

/// Erstellt Router für User Endpoints
pub fn create_users_routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(handler::get_current_user))
        .route("/users", get(handler::list_users))
        .route("/users/:id", get(handler::get_user))
}
