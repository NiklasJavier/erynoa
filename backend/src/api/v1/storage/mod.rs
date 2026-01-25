//! Storage API
//!
//! File upload, download, and object management endpoints

mod handler;
mod models;
mod routes;
#[cfg(feature = "connect")]
mod connect;

pub use routes::create_storage_routes;
#[cfg(feature = "connect")]
pub use connect::{
    upload_handler, list_objects_handler, delete_object_handler, head_object_handler,
    get_presigned_upload_url_handler, get_presigned_download_url_handler,
    list_buckets_handler, create_bucket_handler, delete_bucket_handler,
};
