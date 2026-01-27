//! Connect-RPC Storage Service Implementation

#[cfg(feature = "connect")]
use axum::extract::State;
#[cfg(feature = "connect")]
use std::time::Duration;
#[cfg(feature = "connect")]
use axum_connect::error::RpcError;

#[cfg(feature = "connect")]
use crate::server::AppState;
#[cfg(feature = "connect")]
use crate::error::ApiError;
#[cfg(feature = "connect")]
use crate::error::ApiErrorToRpc;
#[cfg(feature = "connect")]
use crate::gen::erynoa::v1::{
    UploadRequest, UploadResponse, ListObjectsRequest, ListObjectsResponse,
    DeleteObjectRequest, DeleteObjectResponse, HeadObjectRequest, HeadObjectResponse,
    GetPresignedUploadUrlRequest, GetPresignedUploadUrlResponse,
    GetPresignedDownloadUrlRequest, GetPresignedDownloadUrlResponse,
    ListBucketsRequest, ListBucketsResponse, CreateBucketRequest, CreateBucketResponse,
    DeleteBucketRequest, DeleteBucketResponse, ObjectInfo,
};
#[cfg(feature = "connect")]
use axum_connect::pbjson_types::Timestamp;

#[cfg(feature = "connect")]
// Helper function to sanitize filename
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>()
        .chars()
        .take(255)
        .collect()
}

/// Upload Handler
/// 
/// Uploads a file to S3-compatible storage (MinIO).
/// 
/// # Authentication
/// Requires authentication via JWT token.
/// 
/// # Behavior
/// - Generates a unique key based on date and UUID
/// - Sanitizes filename to prevent path traversal
/// - Returns upload result with key, bucket, URL, and ETag
/// 
/// # Example Request
/// ```json
/// {
///   "file": [/* binary data */],
///   "filename": "document.pdf",
///   "content_type": "application/pdf",
///   "bucket": "erynoa"
/// }
/// ```
/// 
/// # Example Response
/// ```json
/// {
///   "key": "2024/01/25/123e4567-e89b-12d3-a456-426614174000-document.pdf",
///   "bucket": "erynoa",
///   "url": "/erynoa/2024/01/25/123e4567-e89b-12d3-a456-426614174000-document.pdf",
///   "etag": "\"abc123def456\""
/// }
/// ```
/// 
/// # Errors
/// - Returns `RpcError::Unavailable` if storage service is not configured
/// - Returns `RpcError::Internal` on upload failures (errors are logged)
pub async fn upload_handler(
    state: State<AppState>,
    request: UploadRequest,
) -> Result<UploadResponse, RpcError> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
            .to_rpc_error()
    })?;

    let file_name = if request.filename.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        request.filename.clone()
    };

    let key = format!(
        "{}/{}-{}",
        chrono::Utc::now().format("%Y/%m/%d"),
        uuid::Uuid::new_v4(),
        sanitize_filename(&file_name)
    );

    let result = storage
        .upload(
            request.bucket.as_deref(),
            &key,
            request.file.clone(),
            Some(&request.content_type),
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Storage upload failed");
            ApiError::Internal(anyhow::anyhow!("Upload failed: {}", e))
                .to_rpc_error()
        })?;

    Ok(UploadResponse {
        key: result.key,
        bucket: result.bucket,
        url: result.url,
        etag: result.etag.unwrap_or_default(),
    })
}

/// List Objects Handler
pub async fn list_objects_handler(
    state: State<AppState>,
    request: ListObjectsRequest,
) -> Result<ListObjectsResponse, RpcError> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
            .to_rpc_error()
    })?;

    let max_keys = if request.max_keys == 0 { 100 } else { request.max_keys };
    let objects: Vec<ObjectInfo> = storage
        .list_objects(
            request.bucket.as_deref(),
            request.prefix.as_deref(),
            Some(max_keys),
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Storage list objects failed");
            ApiError::Internal(anyhow::anyhow!("Failed to list objects: {}", e))
                .to_rpc_error()
        })?
        .into_iter()
        .map(|o| ObjectInfo {
            key: o.key,
            size: o.size,
            content_type: o.content_type,
            last_modified: o.last_modified.map(|dt| {
                Timestamp {
                    seconds: dt.timestamp(),
                    nanos: dt.timestamp_subsec_nanos() as i32,
                }
            }),
        })
        .collect();

    Ok(ListObjectsResponse {
        count: objects.len() as i32,
        objects,
    })
}

/// Delete Object Handler
pub async fn delete_object_handler(
    state: State<AppState>,
    request: DeleteObjectRequest,
) -> Result<DeleteObjectResponse, RpcError> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
            .to_rpc_error()
    })?;

    storage
        .delete(request.bucket.as_deref(), &request.key)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, key = %request.key, "Storage delete failed");
            ApiError::Internal(anyhow::anyhow!("Failed to delete object: {}", e))
                .to_rpc_error()
        })?;

    Ok(DeleteObjectResponse {})
}

/// Head Object Handler (check if exists)
pub async fn head_object_handler(
    state: State<AppState>,
    request: HeadObjectRequest,
) -> Result<HeadObjectResponse, RpcError> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
            .to_rpc_error()
    })?;

    let exists = storage
        .exists(request.bucket.as_deref(), &request.key)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, key = %request.key, "Storage exists check failed");
            ApiError::Internal(anyhow::anyhow!("Failed to check object existence: {}", e))
                .to_rpc_error()
        })?;

    Ok(HeadObjectResponse { exists })
}

/// Get Presigned Upload URL Handler
pub async fn get_presigned_upload_url_handler(
    state: State<AppState>,
    request: GetPresignedUploadUrlRequest,
) -> Result<GetPresignedUploadUrlResponse, RpcError> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
            .to_rpc_error()
    })?;

    let expires_in_secs = if request.expires_in == 0 { 3600 } else { request.expires_in };
    let expires_in = Duration::from_secs(expires_in_secs as u64);
    let url = storage
        .presigned_upload_url(
            request.bucket.as_deref(),
            &request.key,
            expires_in,
            request.content_type.as_deref(),
        )
        .await
        .map_err(|e| {
            tracing::error!(error = %e, key = %request.key, "Storage presigned upload URL failed");
            ApiError::Internal(anyhow::anyhow!("Failed to create presigned upload URL: {}", e))
                .to_rpc_error()
        })?;

    Ok(GetPresignedUploadUrlResponse {
        url,
        expires_in_secs: expires_in_secs as i64,
        method: "PUT".to_string(),
    })
}

/// Get Presigned Download URL Handler
pub async fn get_presigned_download_url_handler(
    state: State<AppState>,
    request: GetPresignedDownloadUrlRequest,
) -> Result<GetPresignedDownloadUrlResponse, RpcError> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
            .to_rpc_error()
    })?;

    let expires_in_secs = if request.expires_in == 0 { 3600 } else { request.expires_in };
    let expires_in = Duration::from_secs(expires_in_secs as u64);
    let url = storage
        .presigned_download_url(request.bucket.as_deref(), &request.key, expires_in)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, key = %request.key, "Storage presigned download URL failed");
            ApiError::Internal(anyhow::anyhow!("Failed to create presigned download URL: {}", e))
                .to_rpc_error()
        })?;

    Ok(GetPresignedDownloadUrlResponse {
        url,
        expires_in_secs: expires_in_secs as i64,
        method: "GET".to_string(),
    })
}

/// List Buckets Handler
pub async fn list_buckets_handler(
    state: State<AppState>,
    _request: ListBucketsRequest,
) -> Result<ListBucketsResponse, RpcError> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
            .to_rpc_error()
    })?;

    let buckets = storage
        .list_buckets()
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Storage list buckets failed");
            ApiError::Internal(anyhow::anyhow!("Failed to list buckets: {}", e))
                .to_rpc_error()
        })?;

    Ok(ListBucketsResponse { buckets })
}

/// Create Bucket Handler
pub async fn create_bucket_handler(
    state: State<AppState>,
    request: CreateBucketRequest,
) -> Result<CreateBucketResponse, RpcError> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
            .to_rpc_error()
    })?;

    storage
        .create_bucket(&request.name)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, bucket = %request.name, "Storage create bucket failed");
            // Check if bucket already exists
            let error_msg = e.to_string();
            if error_msg.contains("already exists") || error_msg.contains("BucketAlreadyExists") {
                ApiError::Conflict(format!("Bucket '{}' already exists", request.name))
                    .to_rpc_error()
            } else {
                ApiError::Internal(anyhow::anyhow!("Failed to create bucket: {}", e))
                    .to_rpc_error()
            }
        })?;

    Ok(CreateBucketResponse {
        name: request.name.clone(),
        message: "Bucket created successfully".to_string(),
    })
}

/// Delete Bucket Handler
pub async fn delete_bucket_handler(
    state: State<AppState>,
    request: DeleteBucketRequest,
) -> Result<DeleteBucketResponse, RpcError> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
            .to_rpc_error()
    })?;

    storage
        .delete_bucket(&request.name)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, bucket = %request.name, "Storage delete bucket failed");
            // Check if bucket not found
            let error_msg = e.to_string();
            if error_msg.contains("not found") || error_msg.contains("NotFound") {
                ApiError::NotFound(format!("Bucket '{}' not found", request.name))
                    .to_rpc_error()
            } else {
                ApiError::Internal(anyhow::anyhow!("Failed to delete bucket: {}", e))
                    .to_rpc_error()
            }
        })?;

    Ok(DeleteBucketResponse {})
}
