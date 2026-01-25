//! Storage Handlers

use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::time::Duration;

use crate::{error::ApiError, server::AppState, Result};

use super::models::{
    BucketsResponse, CreateBucketRequest, ListQuery, ObjectInfo, ObjectListResponse,
    PresignedQuery, PresignedUrlResponse, UploadResponse,
};

/// POST /storage/upload - Datei hochladen
pub async fn upload_file(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Invalid multipart data: {}", e)))?
        .ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;

    let file_name = field
        .file_name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let content_type = field.content_type().map(|s| s.to_string());

    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {}", e)))?
        .to_vec();

    let key = format!(
        "{}/{}-{}",
        chrono::Utc::now().format("%Y/%m/%d"),
        uuid::Uuid::new_v4(),
        sanitize_filename(&file_name)
    );

    let result = storage
        .upload(None, &key, data, content_type.as_deref())
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(Json(UploadResponse {
        key: result.key,
        bucket: result.bucket,
        url: result.url,
        etag: result.etag,
    }))
}

/// POST /storage/upload/:bucket - Datei in spezifischen Bucket hochladen
pub async fn upload_file_to_bucket(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Invalid multipart data: {}", e)))?
        .ok_or_else(|| ApiError::BadRequest("No file provided".to_string()))?;

    let file_name = field
        .file_name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let content_type = field.content_type().map(|s| s.to_string());

    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Failed to read file: {}", e)))?
        .to_vec();

    let key = format!(
        "{}/{}-{}",
        chrono::Utc::now().format("%Y/%m/%d"),
        uuid::Uuid::new_v4(),
        sanitize_filename(&file_name)
    );

    let result = storage
        .upload(Some(&bucket), &key, data, content_type.as_deref())
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(Json(UploadResponse {
        key: result.key,
        bucket: result.bucket,
        url: result.url,
        etag: result.etag,
    }))
}

/// GET /storage/presigned/upload/:key - Presigned URL für Upload generieren
pub async fn get_presigned_upload_url(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Query(params): Query<PresignedQuery>,
) -> Result<Json<PresignedUrlResponse>> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    let url = storage
        .presigned_upload_url(
            None,
            &key,
            Duration::from_secs(params.expires_in),
            params.content_type.as_deref(),
        )
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(Json(PresignedUrlResponse {
        url,
        expires_in_secs: params.expires_in,
        method: "PUT",
    }))
}

/// GET /storage/presigned/download/:key - Presigned URL für Download generieren
pub async fn get_presigned_download_url(
    State(state): State<AppState>,
    Path(key): Path<String>,
    Query(params): Query<PresignedQuery>,
) -> Result<Json<PresignedUrlResponse>> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    let url = storage
        .presigned_download_url(None, &key, Duration::from_secs(params.expires_in))
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(Json(PresignedUrlResponse {
        url,
        expires_in_secs: params.expires_in,
        method: "GET",
    }))
}

/// GET /storage/list - Objekte auflisten
pub async fn list_objects(
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<ObjectListResponse>> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    let objects = storage
        .list_objects(None, params.prefix.as_deref(), Some(params.max_keys))
        .await
        .map_err(|e| ApiError::Internal(e))?;

    let response: Vec<ObjectInfo> = objects
        .into_iter()
        .map(|o| ObjectInfo {
            key: o.key,
            size: o.size,
            content_type: o.content_type,
            last_modified: o.last_modified.map(|dt| dt.to_rfc3339()),
        })
        .collect();

    let count = response.len();

    Ok(Json(ObjectListResponse {
        objects: response,
        count,
    }))
}

/// DELETE /storage/:key - Objekt löschen
pub async fn delete_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<StatusCode> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    storage
        .delete(None, &key)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(StatusCode::NO_CONTENT)
}

/// HEAD /storage/:key - Prüfen ob Objekt existiert
pub async fn head_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<StatusCode> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    if storage.exists(None, &key).await.map_err(|e| ApiError::Internal(e))? {
        Ok(StatusCode::OK)
    } else {
        Ok(StatusCode::NOT_FOUND)
    }
}

/// GET /storage/buckets - Alle Buckets auflisten
pub async fn list_buckets(State(state): State<AppState>) -> Result<Json<BucketsResponse>> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    let buckets = storage
        .list_buckets()
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(Json(BucketsResponse { buckets }))
}

/// POST /storage/buckets - Neuen Bucket erstellen
pub async fn create_bucket(
    State(state): State<AppState>,
    Json(req): Json<CreateBucketRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    let bucket_name = req.name.trim();
    if bucket_name.is_empty() {
        return Err(ApiError::BadRequest("Bucket name cannot be empty".to_string()));
    }

    storage
        .create_bucket(bucket_name)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "name": bucket_name,
            "message": "Bucket created successfully"
        })),
    ))
}

/// DELETE /storage/buckets/:bucket - Bucket löschen
pub async fn delete_bucket(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> Result<StatusCode> {
    let storage = state.storage.as_ref().ok_or_else(|| {
        ApiError::ServiceUnavailable("Storage service not available".to_string())
    })?;

    storage
        .delete_bucket(&bucket)
        .await
        .map_err(|e| ApiError::Internal(e))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Sanitize filename für sichere Speicherung
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>()
        .chars()
        .take(255) // Max filename length
        .collect()
}
