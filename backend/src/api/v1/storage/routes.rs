//! Storage Routes

use axum::{
    routing::{delete, get, head, post},
    Router,
};

use crate::server::AppState;

use super::handler;

/// Erstellt Router für Storage Endpoints
pub fn create_storage_routes() -> Router<AppState> {
    Router::new()
        // File Operations
        .route("/storage/upload", post(handler::upload_file))
        .route("/storage/upload/:bucket", post(handler::upload_file_to_bucket))
        .route("/storage/list", get(handler::list_objects))
        .route("/storage/:key", delete(handler::delete_object))
        .route("/storage/:key", head(handler::head_object))
        // Presigned URLs
        .route(
            "/storage/presigned/upload/:key",
            get(handler::get_presigned_upload_url),
        )
        .route(
            "/storage/presigned/download/:key",
            get(handler::get_presigned_download_url),
        )
        // Bucket Management
        .route("/storage/buckets", get(handler::list_buckets))
        .route("/storage/buckets", post(handler::create_bucket))
        .route("/storage/buckets/:bucket", delete(handler::delete_bucket))
}
