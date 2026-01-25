//! Connect-RPC Storage Service Implementation

use axum::extract::State;
use std::time::Duration;

use crate::server::AppState;
use crate::gen::godstack::v1::{
    UploadRequest, UploadResponse, ListObjectsRequest, ListObjectsResponse,
    DeleteObjectRequest, DeleteObjectResponse, HeadObjectRequest, HeadObjectResponse,
    GetPresignedUploadUrlRequest, GetPresignedUploadUrlResponse,
    GetPresignedDownloadUrlRequest, GetPresignedDownloadUrlResponse,
    ListBucketsRequest, ListBucketsResponse, CreateBucketRequest, CreateBucketResponse,
    DeleteBucketRequest, DeleteBucketResponse, ObjectInfo,
};
use axum_connect::pbjson_types::Timestamp;

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
pub async fn upload_handler(
    state: State<AppState>,
    request: UploadRequest,
) -> UploadResponse {
    let storage = match state.storage.as_ref() {
        Some(s) => s,
        None => {
            return UploadResponse {
                key: String::new(),
                bucket: String::new(),
                url: String::new(),
                etag: String::new(),
            };
        }
    };

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

    let result = match storage
        .upload(
            request.bucket.as_deref(),
            &key,
            request.file.clone(),
            Some(&request.content_type),
        )
        .await
    {
        Ok(r) => r,
        Err(_) => {
            return UploadResponse {
                key: String::new(),
                bucket: String::new(),
                url: String::new(),
                etag: String::new(),
            };
        }
    };

    UploadResponse {
        key: result.key,
        bucket: result.bucket,
        url: result.url,
        etag: result.etag.unwrap_or_default(),
    }
}

/// List Objects Handler
pub async fn list_objects_handler(
    state: State<AppState>,
    request: ListObjectsRequest,
) -> ListObjectsResponse {
    let storage = match state.storage.as_ref() {
        Some(s) => s,
        None => {
            return ListObjectsResponse {
                objects: vec![],
                count: 0,
            };
        }
    };

    let max_keys = if request.max_keys == 0 { 100 } else { request.max_keys };
    let objects = match storage
        .list_objects(
            request.bucket.as_deref(),
            request.prefix.as_deref(),
            Some(max_keys),
        )
        .await
    {
        Ok(objs) => objs
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
            .collect(),
        Err(_) => vec![],
    };

    ListObjectsResponse {
        count: objects.len() as i32,
        objects,
    }
}

/// Delete Object Handler
pub async fn delete_object_handler(
    state: State<AppState>,
    request: DeleteObjectRequest,
) -> DeleteObjectResponse {
    let storage = match state.storage.as_ref() {
        Some(s) => s,
        None => {
            return DeleteObjectResponse {};
        }
    };

    let _ = storage
        .delete(request.bucket.as_deref(), &request.key)
        .await;

    DeleteObjectResponse {}
}

/// Head Object Handler (check if exists)
pub async fn head_object_handler(
    state: State<AppState>,
    request: HeadObjectRequest,
) -> HeadObjectResponse {
    let storage = match state.storage.as_ref() {
        Some(s) => s,
        None => {
            return HeadObjectResponse {
                exists: false,
            };
        }
    };

    match storage.exists(request.bucket.as_deref(), &request.key).await {
        Ok(true) => HeadObjectResponse {
            exists: true,
        },
        _ => HeadObjectResponse {
            exists: false,
        },
    }
}

/// Get Presigned Upload URL Handler
pub async fn get_presigned_upload_url_handler(
    state: State<AppState>,
    request: GetPresignedUploadUrlRequest,
) -> GetPresignedUploadUrlResponse {
    let storage = match state.storage.as_ref() {
        Some(s) => s,
        None => {
            return GetPresignedUploadUrlResponse {
                url: String::new(),
                expires_in_secs: 0,
                method: "PUT".to_string(),
            };
        }
    };

    let expires_in_secs = if request.expires_in == 0 { 3600 } else { request.expires_in };
    let expires_in = Duration::from_secs(expires_in_secs as u64);
    let url = match storage
        .presigned_upload_url(
            request.bucket.as_deref(),
            &request.key,
            expires_in,
            request.content_type.as_deref(),
        )
        .await
    {
        Ok(u) => u,
        Err(_) => String::new(),
    };

    GetPresignedUploadUrlResponse {
        url,
        expires_in_secs: expires_in_secs as i64,
        method: "PUT".to_string(),
    }
}

/// Get Presigned Download URL Handler
pub async fn get_presigned_download_url_handler(
    state: State<AppState>,
    request: GetPresignedDownloadUrlRequest,
) -> GetPresignedDownloadUrlResponse {
    let storage = match state.storage.as_ref() {
        Some(s) => s,
        None => {
            return GetPresignedDownloadUrlResponse {
                url: String::new(),
                expires_in_secs: 0,
                method: "GET".to_string(),
            };
        }
    };

    let expires_in_secs = if request.expires_in == 0 { 3600 } else { request.expires_in };
    let expires_in = Duration::from_secs(expires_in_secs as u64);
    let url = match storage
        .presigned_download_url(request.bucket.as_deref(), &request.key, expires_in)
        .await
    {
        Ok(u) => u,
        Err(_) => String::new(),
    };

    GetPresignedDownloadUrlResponse {
        url,
        expires_in_secs: expires_in_secs as i64,
        method: "GET".to_string(),
    }
}

/// List Buckets Handler
pub async fn list_buckets_handler(
    state: State<AppState>,
    _request: ListBucketsRequest,
) -> ListBucketsResponse {
    let storage = match state.storage.as_ref() {
        Some(s) => s,
        None => {
            return ListBucketsResponse { buckets: vec![] };
        }
    };

    let buckets = match storage.list_buckets().await {
        Ok(b) => b,
        Err(_) => vec![],
    };

    ListBucketsResponse { buckets }
}

/// Create Bucket Handler
pub async fn create_bucket_handler(
    state: State<AppState>,
    request: CreateBucketRequest,
) -> CreateBucketResponse {
    let storage = match state.storage.as_ref() {
        Some(s) => s,
        None => {
            return CreateBucketResponse {
                name: request.name.clone(),
                message: "Storage service not available".to_string(),
            };
        }
    };

    match storage.create_bucket(&request.name).await {
        Ok(_) => CreateBucketResponse {
            name: request.name.clone(),
            message: "Bucket created successfully".to_string(),
        },
        Err(e) => CreateBucketResponse {
            name: request.name.clone(),
            message: format!("Failed to create bucket: {}", e),
        },
    }
}

/// Delete Bucket Handler
pub async fn delete_bucket_handler(
    state: State<AppState>,
    request: DeleteBucketRequest,
) -> DeleteBucketResponse {
    let storage = match state.storage.as_ref() {
        Some(s) => s,
        None => {
            return DeleteBucketResponse {};
        }
    };

    let _ = storage
        .delete_bucket(&request.name)
        .await;

    DeleteBucketResponse {}
}
