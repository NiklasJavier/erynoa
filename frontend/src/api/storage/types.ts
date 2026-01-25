/**
 * Storage Service Types
 * 
 * Protobuf types as primary source with helper functions
 */

// Re-export Protobuf types as primary source
export {
  UploadRequest,
  UploadResponse,
  ListObjectsRequest,
  DeleteObjectRequest,
  HeadObjectRequest,
  HeadObjectResponse,
  GetPresignedUploadUrlRequest,
  GetPresignedUploadUrlResponse,
  GetPresignedDownloadUrlRequest,
  GetPresignedDownloadUrlResponse,
  ListBucketsRequest,
  ListBucketsResponse,
  CreateBucketRequest,
  CreateBucketResponse,
  DeleteBucketRequest,
  ObjectInfo,
} from "../../gen/godstack/v1/storage_pb";

// Export with clearer names for helper functions (avoiding name conflicts with our interface)
export type { 
  ObjectInfo as ProtoObjectInfo, 
  ListObjectsResponse as ProtoListObjectsResponse 
} from "../../gen/godstack/v1/storage_pb";

/**
 * Frontend StorageObject type (compatible with existing code)
 * Converted from Protobuf ObjectInfo
 */
export interface StorageObject {
  key: string;
  size: number;
  content_type?: string;
  last_modified?: string;
  etag?: string;
}

/**
 * Frontend UploadResult type
 */
export interface UploadResult {
  key: string;
  bucket: string;
  url: string;
  etag?: string;
}

/**
 * Frontend PresignedUrl type
 */
export interface PresignedUrl {
  url: string;
  expires_in: number;
}

/**
 * Frontend ListObjectsResponse type
 */
export interface ListObjectsResponse {
  objects: StorageObject[];
  count: number;
}

/**
 * Convert ProtoObjectInfo to StorageObject
 */
export function toStorageObject(proto: import("../../gen/godstack/v1/storage_pb").ObjectInfo): StorageObject {
  return {
    key: proto.key,
    size: Number(proto.size),
    content_type: proto.contentType || undefined,
    last_modified: proto.lastModified
      ? new Date(Number(proto.lastModified.seconds) * 1000).toISOString()
      : undefined,
    // Note: ObjectInfo may not have etag field, check proto definition
    etag: undefined, // etag not available in ObjectInfo
  };
}

/**
 * Convert UploadResponse to UploadResult
 */
export function toUploadResult(response: import("../../gen/godstack/v1/storage_pb").UploadResponse): UploadResult {
  return {
    key: response.key,
    bucket: response.bucket,
    url: response.url,
    etag: response.etag || undefined,
  };
}

/**
 * Convert GetPresignedUrlResponse to PresignedUrl
 */
export function toPresignedUrl(
  response: import("../../gen/godstack/v1/storage_pb").GetPresignedUploadUrlResponse | import("../../gen/godstack/v1/storage_pb").GetPresignedDownloadUrlResponse
): PresignedUrl {
  return {
    url: response.url,
    expires_in: Number(response.expiresInSecs),
  };
}

/**
 * Convert ProtoListObjectsResponse to ListObjectsResponse
 */
export function toListObjectsResponse(
  proto: import("../../gen/godstack/v1/storage_pb").ListObjectsResponse
): ListObjectsResponse {
  return {
    objects: proto.objects.map(toStorageObject),
    count: proto.objects.length,
  };
}
