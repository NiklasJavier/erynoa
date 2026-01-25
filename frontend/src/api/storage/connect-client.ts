/**
 * Connect-RPC Storage Client
 * 
 * Storage operations using Connect-RPC/gRPC-Web
 */

import { createAuthenticatedClients } from "../connect/services";
import { 
  UploadRequest, 
  ListObjectsRequest,
  DeleteObjectRequest,
  HeadObjectRequest,
  GetPresignedUploadUrlRequest,
  GetPresignedDownloadUrlRequest,
  ListBucketsRequest,
  CreateBucketRequest,
  DeleteBucketRequest,
} from "../../gen/godstack/v1/storage_pb";
import type {
  UploadResult,
  PresignedUrl,
  ListObjectsResponse as StorageListResponse,
} from "./types";
import {
  toUploadResult,
  toPresignedUrl,
  toListObjectsResponse,
} from "./types";

/**
 * Connect-RPC Storage Client
 */
export class ConnectStorageClient {
  private getToken: () => Promise<string | null>;

  constructor(getToken: () => Promise<string | null>) {
    this.getToken = getToken;
  }

  private getClient() {
    const clients = createAuthenticatedClients(this.getToken);
    return clients.storage;
  }

  /**
   * Upload a file
   * 
   * For large files (>5MB), automatically uses presigned URLs for progress tracking.
   * For smaller files, uses direct Connect-RPC upload.
   */
  async upload(
    file: File,
    bucket?: string,
    onProgress?: (progress: number) => void
  ): Promise<UploadResult> {
    const LARGE_FILE_THRESHOLD = 5 * 1024 * 1024; // 5MB

    // For large files, use presigned URLs for better progress tracking
    if (file.size > LARGE_FILE_THRESHOLD && onProgress) {
      return this.uploadWithPresignedUrl(file, bucket, onProgress);
    }

    // For smaller files, use direct Connect-RPC upload
    // Read file as ArrayBuffer
    const arrayBuffer = await file.arrayBuffer();
    const fileBytes = new Uint8Array(arrayBuffer);

    const request = new UploadRequest({
      file: fileBytes,
      filename: file.name,
      contentType: file.type || "application/octet-stream",
      bucket: bucket,
    });

    // Simulate progress for small files (Connect-RPC doesn't support native progress)
    if (onProgress) {
      onProgress(10); // Start
    }

    try {
      const response = await this.getClient().upload(request);
      
      if (onProgress) {
        onProgress(100);
      }

      return toUploadResult(response);
    } catch (error) {
      if (onProgress) {
        onProgress(0); // Reset on error
      }
      throw error;
    }
  }

  /**
   * Upload using presigned URL with progress tracking
   * This provides real progress updates for large files
   */
  private async uploadWithPresignedUrl(
    file: File,
    bucket: string | undefined,
    onProgress: (progress: number) => void
  ): Promise<UploadResult> {
    // Generate a key for the file
    const timestamp = new Date().toISOString().split('T')[0].replace(/-/g, '/');
    const uuid = crypto.randomUUID();
    const sanitizedFilename = file.name.replace(/[^a-zA-Z0-9._-]/g, '_').substring(0, 255);
    const key = `${timestamp}/${uuid}-${sanitizedFilename}`;

    // Get presigned upload URL
    onProgress(5);
    const { url } = await this.getPresignedUploadUrl(key, bucket, 3600);
    
    onProgress(10);

    // Upload directly to S3 using XMLHttpRequest for progress tracking
    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();

      xhr.upload.addEventListener("progress", (event) => {
        if (event.lengthComputable) {
          // Progress from 10% to 90% (reserve 10% for completion)
          const progress = 10 + Math.round((event.loaded / event.total) * 80);
          onProgress(progress);
        }
      });

      xhr.addEventListener("load", () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          onProgress(100);
          // Return result similar to direct upload
          resolve({
            key,
            bucket: bucket || "godstack",
            url: url.split('?')[0], // Remove query params for display URL
            etag: xhr.getResponseHeader("ETag")?.replace(/"/g, "") || "",
          });
        } else {
          onProgress(0);
          reject(new Error(`Upload failed: ${xhr.statusText}`));
        }
      });

      xhr.addEventListener("error", () => {
        onProgress(0);
        reject(new Error("Upload failed: Network error"));
      });

      xhr.addEventListener("abort", () => {
        onProgress(0);
        reject(new Error("Upload aborted"));
      });

      xhr.open("PUT", url);
      xhr.setRequestHeader("Content-Type", file.type || "application/octet-stream");
      xhr.send(file);
    });
  }

  /**
   * List objects
   */
  async list(prefix?: string, bucket?: string): Promise<StorageListResponse> {
    const request = new ListObjectsRequest({
      prefix: prefix,
      bucket: bucket,
      maxKeys: 100,
    });

    const response = await this.getClient().list(request);

    return toListObjectsResponse(response);
  }

  /**
   * Delete an object
   */
  async delete(key: string, bucket?: string): Promise<void> {
    const request = new DeleteObjectRequest({
      key,
      bucket: bucket,
    });

    await this.getClient().delete(request);
  }

  /**
   * Check if object exists
   */
  async exists(key: string, bucket?: string): Promise<boolean> {
    const request = new HeadObjectRequest({
      key,
      bucket: bucket,
    });

    const response = await this.getClient().head(request);
    return response.exists;
  }

  /**
   * Get presigned upload URL
   */
  async getPresignedUploadUrl(
    key: string,
    bucket?: string,
    expiresIn: number = 3600
  ): Promise<PresignedUrl> {
    const request = new GetPresignedUploadUrlRequest({
      key,
      bucket: bucket,
      expiresIn: BigInt(expiresIn),
    });

    const response = await this.getClient().getPresignedUploadUrl(request);

    return toPresignedUrl(response);
  }

  /**
   * Get presigned download URL
   */
  async getPresignedDownloadUrl(
    key: string,
    bucket?: string,
    expiresIn: number = 3600
  ): Promise<PresignedUrl> {
    const request = new GetPresignedDownloadUrlRequest({
      key,
      bucket: bucket,
      expiresIn: BigInt(expiresIn),
    });

    const response = await this.getClient().getPresignedDownloadUrl(request);

    return toPresignedUrl(response);
  }

  /**
   * Get download URL (convenience method)
   */
  async getDownloadUrl(key: string, bucket?: string): Promise<string> {
    const { url } = await this.getPresignedDownloadUrl(key, bucket);
    return url;
  }

  /**
   * List buckets
   */
  async listBuckets(): Promise<string[]> {
    const request = new ListBucketsRequest({});
    const response = await this.getClient().listBuckets(request);
    return response.buckets;
  }

  /**
   * Create bucket
   */
  async createBucket(name: string): Promise<void> {
    const request = new CreateBucketRequest({
      name: name.trim(),
    });

    await this.getClient().createBucket(request);
  }

  /**
   * Delete bucket
   */
  async deleteBucket(name: string): Promise<void> {
    const request = new DeleteBucketRequest({
      name,
    });

    await this.getClient().deleteBucket(request);
  }
}
