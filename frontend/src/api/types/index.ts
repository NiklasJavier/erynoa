/**
 * Shared API Types
 * 
 * @deprecated Use feature-based type exports instead:
 * - import { User, toUser } from "./api/users"
 * - import { StorageObject, toStorageObject } from "./api/storage"
 * - import { HealthResponse } from "./api/health"
 * - import { GetInfoResponse } from "./api/info"
 * 
 * This file is kept for backwards compatibility only.
 */

// Common Types
// Re-export error types (still used)
export * from "./errors";

// Legacy Types - Deprecated, use Protobuf types from feature modules instead
/**
 * @deprecated Use types from "./api/health" instead
 */
export interface HealthResponse {
  status: string;
  version: string;
}

/**
 * @deprecated Use types from "./api/health" instead
 */
export interface ServiceStatus {
  status: string;
  latency_ms?: number;
  error?: string;
}

/**
 * @deprecated Use types from "./api/health" instead
 */
export interface ReadinessResponse {
  status: string;
  services: {
    database: ServiceStatus;
    cache: ServiceStatus;
    storage: ServiceStatus;
    auth: ServiceStatus;
  };
  uptime_secs?: number;
}

/**
 * @deprecated Use types from "./api/info" instead
 */
export interface InfoResponse {
  version: string;
  environment: string;
  auth_issuer: string;
  auth_client_id: string;
  frontend_url: string;
  api_url: string;
}

/**
 * @deprecated Use types from "./api/users" instead
 */
export interface User {
  id: string;
  email: string | null;
  name: string | null;
  roles: string[];
}

/**
 * @deprecated Use types from "./api/storage" instead
 */
export interface StorageObject {
  key: string;
  size: number;
  content_type?: string;
  last_modified?: string;
  etag?: string;
}

/**
 * @deprecated Use types from "./api/storage" instead
 */
export interface UploadResult {
  key: string;
  bucket: string;
  url: string;
  etag?: string;
}

/**
 * @deprecated Use types from "./api/storage" instead
 */
export interface PresignedUrl {
  url: string;
  expires_in: number;
}

/**
 * @deprecated Use types from "./api/storage" instead
 */
export interface ListObjectsResponse {
  objects: StorageObject[];
  count: number;
}
