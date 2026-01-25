/**
 * Shared API Types
 * 
 * Zentrale Type-Definitionen für alle API-Clients
 */

// Common Types
// Re-export error types
export * from "./errors";

// Legacy ApiError removed - use ApiErrorResponse from errors.ts instead
// If you need backward compatibility, use toLegacyError() helper from errors.ts

// Health Check Types
export interface HealthResponse {
  status: string;
  version: string;
}

export interface ServiceStatus {
  status: string;
  latency_ms?: number;
  error?: string;
}

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

// Info Types
export interface InfoResponse {
  version: string;
  environment: string;
  auth_issuer: string;
  auth_client_id: string;
  frontend_url: string;
  api_url: string;
}

// User Types
export interface User {
  id: string;
  email: string | null;
  name: string | null;
  roles: string[];
}

// Storage Types
export interface StorageObject {
  key: string;
  size: number;
  content_type?: string;
  last_modified?: string;
  etag?: string;
}

export interface UploadResult {
  key: string;
  bucket: string;
  url: string;
  etag?: string;
}

export interface PresignedUrl {
  url: string;
  expires_in: number;
}

export interface ListObjectsResponse {
  objects: StorageObject[];
  count: number;
}
