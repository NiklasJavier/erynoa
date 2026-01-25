//! Connect-RPC Routes
//!
//! Routes für Connect-RPC/gRPC-Web Services

use crate::server::AppState;
use axum::Router;

// Import generated service types
use crate::gen::godstack::v1::{HealthService, InfoService, UserService, StorageService};

// Import handler functions (via public re-exports from handlers.rs)
#[cfg(feature = "connect")]
use crate::api::v1::health::{health_check_handler, ready_check_handler};
#[cfg(feature = "connect")]
use crate::api::v1::info::get_info_handler;
#[cfg(feature = "connect")]
use crate::api::v1::users::{list_users_handler, get_user_handler, get_current_user_handler};
#[cfg(feature = "connect")]
use crate::api::v1::storage::{
    upload_handler, list_objects_handler, delete_object_handler, head_object_handler,
    get_presigned_upload_url_handler, get_presigned_download_url_handler,
    list_buckets_handler, create_bucket_handler, delete_bucket_handler,
};

/// Erstellt Router für Connect-RPC Services
/// 
/// Connect-RPC Services werden unter /api/v1/connect/ bereitgestellt
/// z.B. /api/v1/connect/godstack.v1.HealthService/Check
pub fn create_connect_routes(_state: AppState) -> Router<AppState> {
    // Start with empty router
    let router = Router::new();
    
    // Apply Health Service handlers
    // Note: Handler functions are automatically converted to RpcHandlerUnary by axum-connect
    // The signature (State<S>, Request) -> Response matches the trait requirements
    let router = HealthService::check(health_check_handler)(router);
    let router = HealthService::ready(ready_check_handler)(router);
    
    // Apply Info Service handlers
    let router = InfoService::get_info(get_info_handler)(router);
    
    // Apply User Service handlers
    let router = UserService::list(list_users_handler)(router);
    let router = UserService::get(get_user_handler)(router);
    let router = UserService::get_current(get_current_user_handler)(router);
    
    // Apply Storage Service handlers
    let router = StorageService::upload(upload_handler)(router);
    let router = StorageService::list(list_objects_handler)(router);
    let router = StorageService::delete(delete_object_handler)(router);
    let router = StorageService::head(head_object_handler)(router);
    let router = StorageService::get_presigned_upload_url(get_presigned_upload_url_handler)(router);
    let router = StorageService::get_presigned_download_url(get_presigned_download_url_handler)(router);
    let router = StorageService::list_buckets(list_buckets_handler)(router);
    let router = StorageService::create_bucket(create_bucket_handler)(router);
    let router = StorageService::delete_bucket(delete_bucket_handler)(router);
    
    router
}
