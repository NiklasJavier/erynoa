//! S3 Storage Client
//!
//! Wrapper um AWS SDK für S3-kompatible Operationen mit MinIO

use crate::config::StorageSettings;
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{BehaviorVersion, Builder as S3ConfigBuilder},
    presigning::PresigningConfig,
    primitives::ByteStream,
    Client,
};
use secrecy::ExposeSecret;
use std::time::Duration;

/// Storage Client für S3-kompatible Operationen
#[derive(Clone)]
pub struct StorageClient {
    client: Client,
    default_bucket: String,
    max_upload_size: u64,
}

/// Metadaten eines gespeicherten Objekts
#[derive(Debug, Clone)]
pub struct ObjectMetadata {
    pub key: String,
    pub size: i64,
    pub content_type: Option<String>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub etag: Option<String>,
}

/// Ergebnis einer Upload-Operation
#[derive(Debug, Clone)]
pub struct UploadResult {
    pub key: String,
    pub bucket: String,
    pub etag: Option<String>,
    pub url: String,
}

impl StorageClient {
    /// Erstellt einen neuen Storage Client
    pub async fn connect(settings: &StorageSettings) -> anyhow::Result<Self> {
        let credentials = Credentials::new(
            &settings.access_key_id,
            settings.secret_access_key.expose_secret(),
            None,
            None,
            "godstack",
        );

        let s3_config = S3ConfigBuilder::new()
            .behavior_version(BehaviorVersion::latest())
            .endpoint_url(&settings.endpoint)
            .region(Region::new(settings.region.clone()))
            .credentials_provider(credentials)
            .force_path_style(true) // Wichtig für MinIO
            .build();

        let client = Client::from_conf(s3_config);

        // Verbindung testen
        client
            .list_buckets()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to S3: {}", e))?;

        Ok(Self {
            client,
            default_bucket: settings.default_bucket.clone(),
            max_upload_size: settings.max_upload_size,
        })
    }

    /// Ping zum Testen der Verbindung
    pub async fn ping(&self) -> anyhow::Result<()> {
        self.client
            .list_buckets()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("S3 ping failed: {}", e))?;
        Ok(())
    }

    /// Gibt den Standard-Bucket zurück
    pub fn default_bucket(&self) -> &str {
        &self.default_bucket
    }

    /// Maximale Upload-Größe in Bytes
    pub fn max_upload_size(&self) -> u64 {
        self.max_upload_size
    }

    /// Lädt eine Datei in den S3 Storage
    pub async fn upload(
        &self,
        bucket: Option<&str>,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> anyhow::Result<UploadResult> {
        let bucket = bucket.unwrap_or(&self.default_bucket);
        
        if data.len() as u64 > self.max_upload_size {
            anyhow::bail!(
                "File size {} exceeds maximum allowed size {}",
                data.len(),
                self.max_upload_size
            );
        }

        let mut request = self
            .client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(data));

        if let Some(ct) = content_type {
            request = request.content_type(ct);
        }

        let result = request
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Upload failed: {}", e))?;

        Ok(UploadResult {
            key: key.to_string(),
            bucket: bucket.to_string(),
            etag: result.e_tag().map(|s| s.to_string()),
            url: format!("/{}/{}", bucket, key),
        })
    }

    /// Lädt eine Datei aus dem S3 Storage
    pub async fn download(&self, bucket: Option<&str>, key: &str) -> anyhow::Result<Vec<u8>> {
        let bucket = bucket.unwrap_or(&self.default_bucket);

        let result = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Download failed: {}", e))?;

        let data = result
            .body
            .collect()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read body: {}", e))?
            .into_bytes()
            .to_vec();

        Ok(data)
    }

    /// Löscht eine Datei aus dem S3 Storage
    pub async fn delete(&self, bucket: Option<&str>, key: &str) -> anyhow::Result<()> {
        let bucket = bucket.unwrap_or(&self.default_bucket);

        self.client
            .delete_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Delete failed: {}", e))?;

        Ok(())
    }

    /// Prüft ob ein Objekt existiert
    pub async fn exists(&self, bucket: Option<&str>, key: &str) -> anyhow::Result<bool> {
        let bucket = bucket.unwrap_or(&self.default_bucket);

        match self
            .client
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.to_string().contains("NotFound") || e.to_string().contains("404") {
                    Ok(false)
                } else {
                    Err(anyhow::anyhow!("Failed to check object: {}", e))
                }
            }
        }
    }

    /// Holt Metadaten eines Objekts
    pub async fn get_metadata(
        &self,
        bucket: Option<&str>,
        key: &str,
    ) -> anyhow::Result<ObjectMetadata> {
        let bucket = bucket.unwrap_or(&self.default_bucket);

        let result = self
            .client
            .head_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get metadata: {}", e))?;

        Ok(ObjectMetadata {
            key: key.to_string(),
            size: result.content_length().unwrap_or(0),
            content_type: result.content_type().map(|s| s.to_string()),
            last_modified: result.last_modified().and_then(|dt| {
                chrono::DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
            }),
            etag: result.e_tag().map(|s| s.to_string()),
        })
    }

    /// Erstellt eine Presigned URL für Upload (PUT)
    pub async fn presigned_upload_url(
        &self,
        bucket: Option<&str>,
        key: &str,
        expires_in: Duration,
        content_type: Option<&str>,
    ) -> anyhow::Result<String> {
        let bucket = bucket.unwrap_or(&self.default_bucket);
        let presigning_config = PresigningConfig::expires_in(expires_in)
            .map_err(|e| anyhow::anyhow!("Invalid expiry duration: {}", e))?;

        let mut request = self
            .client
            .put_object()
            .bucket(bucket)
            .key(key);

        if let Some(ct) = content_type {
            request = request.content_type(ct);
        }

        let presigned = request
            .presigned(presigning_config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create presigned URL: {}", e))?;

        Ok(presigned.uri().to_string())
    }

    /// Erstellt eine Presigned URL für Download (GET)
    pub async fn presigned_download_url(
        &self,
        bucket: Option<&str>,
        key: &str,
        expires_in: Duration,
    ) -> anyhow::Result<String> {
        let bucket = bucket.unwrap_or(&self.default_bucket);
        let presigning_config = PresigningConfig::expires_in(expires_in)
            .map_err(|e| anyhow::anyhow!("Invalid expiry duration: {}", e))?;

        let presigned = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create presigned URL: {}", e))?;

        Ok(presigned.uri().to_string())
    }

    /// Listet Objekte in einem Bucket mit optionalem Prefix
    pub async fn list_objects(
        &self,
        bucket: Option<&str>,
        prefix: Option<&str>,
        max_keys: Option<i32>,
    ) -> anyhow::Result<Vec<ObjectMetadata>> {
        let bucket = bucket.unwrap_or(&self.default_bucket);

        let mut request = self.client.list_objects_v2().bucket(bucket);

        if let Some(p) = prefix {
            request = request.prefix(p);
        }

        if let Some(max) = max_keys {
            request = request.max_keys(max);
        }

        let result = request
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list objects: {}", e))?;

        let objects = result
            .contents()
            .iter()
            .map(|obj| ObjectMetadata {
                key: obj.key().unwrap_or_default().to_string(),
                size: obj.size().unwrap_or(0),
                content_type: None, // Nicht in list_objects verfügbar
                last_modified: obj.last_modified().and_then(|dt| {
                    chrono::DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
                }),
                etag: obj.e_tag().map(|s| s.to_string()),
            })
            .collect();

        Ok(objects)
    }

    /// Kopiert ein Objekt innerhalb von S3
    pub async fn copy(
        &self,
        source_bucket: Option<&str>,
        source_key: &str,
        dest_bucket: Option<&str>,
        dest_key: &str,
    ) -> anyhow::Result<()> {
        let source_bucket = source_bucket.unwrap_or(&self.default_bucket);
        let dest_bucket = dest_bucket.unwrap_or(&self.default_bucket);

        let copy_source = format!("{}/{}", source_bucket, source_key);

        self.client
            .copy_object()
            .bucket(dest_bucket)
            .key(dest_key)
            .copy_source(copy_source)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Copy failed: {}", e))?;

        Ok(())
    }

    /// Alle Buckets auflisten
    pub async fn list_buckets(&self) -> anyhow::Result<Vec<String>> {
        let resp = self.client.list_buckets().send().await?;

        let buckets = resp
            .buckets()
            .iter()
            .filter_map(|bucket| bucket.name().map(|s| s.to_string()))
            .collect();

        Ok(buckets)
    }

    /// Neuen Bucket erstellen
    pub async fn create_bucket(&self, bucket: &str) -> anyhow::Result<()> {
        self.client
            .create_bucket()
            .bucket(bucket)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create bucket: {}", e))?;

        Ok(())
    }

    /// Bucket löschen
    pub async fn delete_bucket(&self, bucket: &str) -> anyhow::Result<()> {
        self.client
            .delete_bucket()
            .bucket(bucket)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete bucket: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests würden einen laufenden MinIO-Server benötigen
    // Diese sind in tests/storage.rs
}
