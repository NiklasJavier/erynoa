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
 * @deprecated REST Client exports - Use Connect-RPC instead
 * Connect-RPC is now the primary communication method.
 * REST endpoints are kept for backwards compatibility only.
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

// Legacy types export (deprecated, kept for backwards compatibility)
export * from "./types";

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

// REST Client (Deprecated - kept for backwards compatibility)
/**
 * @deprecated Use Connect-RPC clients instead (createAuthenticatedClients, infoClient, healthClient, etc.)
 * REST endpoints will be removed in a future version.
 */
export { initRestClient, getRestClient, RestClient } from "./rest/client";
/**
 * @deprecated Use Connect-RPC clients instead
 */
export { api } from "./rest/endpoints";

// Legacy exports for backwards compatibility
/**
 * @deprecated Use createAuthenticatedClients() from "./connect/services" instead
 */
import { initRestClient } from "./rest/client";

export function initApiClient(getToken: () => Promise<string | null>) {
  // Dynamic import to avoid circular dependencies
  import("../lib/logger").then(({ logger }) => {
    logger.warn("initApiClient is deprecated. Use createAuthenticatedClients() from './connect/services' instead.", {
      deprecated: true,
      alternative: "createAuthenticatedClients",
    });
  });
  return initRestClient(getToken);
}
