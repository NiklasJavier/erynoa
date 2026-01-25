//! Storage Module - S3-kompatibler Object Storage (MinIO)
//!
//! Bietet einen hochperformanten S3 Client für:
//! - File Uploads/Downloads
//! - Presigned URLs für direkten Client-Zugriff
//! - Bucket-Management

mod client;

pub use client::StorageClient;
