/**
 * API Module - Hauptexport
 * 
 * Zentrale Export-Datei für alle API-Clients und Types
 * 
 * Verwendung:
 * ```ts
 * import { storage, createAuthenticatedClients, initStorageClient } from "./api";
 * import type { User, StorageObject } from "./api";
 * ```
 * 
 * Connect-RPC is the primary communication method.
 */

// Types - Feature-based exports (primary)
// Export types with explicit names to avoid conflicts
export type {
  ProtoServiceStatus,
  HealthCheckRequest,
  HealthCheckResponse,
  ReadyRequest,
  ReadyResponse,
} from "./health";
export type {
  GetInfoRequest,
  GetInfoResponse,
} from "./info";
export type {
  User,
  ProtoUser,
  ListUsersRequest,
  ListUsersResponse,
  GetUserRequest,
  GetUserResponse,
} from "./users";
export type {
  StorageObject,
  UploadResult,
  PresignedUrl,
  ListObjectsResponse,
} from "./storage";

// Connect-RPC Client (Primary)
export {
  userClient,
  healthClient,
  infoClient,
  storageClient,
  createAuthenticatedClients,
  type UserClient,
  type HealthClient,
  type InfoClient,
  type StorageClient as ConnectStorageClientType,
} from "./connect/services";

// Storage Client (uses Connect-RPC internally)
export {
  StorageClient,
  initStorageClient,
  getStorageClient,
  storage,
} from "./storage/index";

