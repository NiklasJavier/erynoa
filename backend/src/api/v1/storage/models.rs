//! Storage Models

use serde::{Deserialize, Serialize};

/// Response für Upload-Operationen
#[derive(Serialize)]
pub struct UploadResponse {
    pub key: String,
    pub bucket: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

/// Response für Presigned URL
#[derive(Serialize)]
pub struct PresignedUrlResponse {
    pub url: String,
    pub expires_in_secs: u64,
    pub method: &'static str,
}

/// Response für Object-Liste
#[derive(Serialize)]
pub struct ObjectListResponse {
    pub objects: Vec<ObjectInfo>,
    pub count: usize,
}

/// Info über ein einzelnes Objekt
#[derive(Serialize)]
pub struct ObjectInfo {
    pub key: String,
    pub size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
}

/// Query Parameter für Presigned URLs
#[derive(Deserialize)]
pub struct PresignedQuery {
    /// Gültigkeit in Sekunden (default: 3600)
    #[serde(default = "default_expires")]
    pub expires_in: u64,
    /// Content-Type für Uploads
    #[serde(default)]
    pub content_type: Option<String>,
}

fn default_expires() -> u64 {
    3600 // 1 Stunde
}

/// Query Parameter für List-Operationen
#[derive(Deserialize)]
pub struct ListQuery {
    /// Prefix-Filter
    #[serde(default)]
    pub prefix: Option<String>,
    /// Max Anzahl
    #[serde(default = "default_max_keys")]
    pub max_keys: i32,
}

fn default_max_keys() -> i32 {
    100
}

/// Response für Bucket-Liste
#[derive(Serialize)]
pub struct BucketsResponse {
    pub buckets: Vec<String>,
}

/// Request für neuen Bucket
#[derive(Deserialize)]
pub struct CreateBucketRequest {
    pub name: String,
}
