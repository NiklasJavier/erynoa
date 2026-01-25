//! Storage API
//!
//! File upload, download, and object management endpoints (Connect-RPC only)

mod handlers;

#[cfg(feature = "connect")]
pub use handlers::{
    upload_handler, list_objects_handler, delete_object_handler, head_object_handler,
    get_presigned_upload_url_handler, get_presigned_download_url_handler,
    list_buckets_handler, create_bucket_handler, delete_bucket_handler,
};
