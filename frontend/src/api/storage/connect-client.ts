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
   */
  async upload(
    file: File,
    bucket?: string,
    onProgress?: (progress: number) => void
  ): Promise<UploadResult> {
    // Read file as ArrayBuffer
    const arrayBuffer = await file.arrayBuffer();
    const fileBytes = new Uint8Array(arrayBuffer);

    const request = new UploadRequest({
      file: fileBytes,
      filename: file.name,
      contentType: file.type || "application/octet-stream",
      bucket: bucket,
    });

    // TODO: Add progress tracking for Connect-RPC
    // Note: Connect-RPC doesn't natively support upload progress
    // For large files, consider using presigned URLs instead
    if (onProgress) {
      onProgress(50); // Simulate progress
    }

    const response = await this.getClient().upload(request);
    
    if (onProgress) {
      onProgress(100);
    }

    return toUploadResult(response);
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
