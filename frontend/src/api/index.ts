/**
 * API Module - Hauptexport
 * 
 * Zentrale Export-Datei für alle API-Clients und Types
 * 
 * Verwendung:
 * ```ts
 * import { api, storage, initApiClient, initStorageClient } from "./api";
 * import type { User, StorageObject } from "./api";
 * ```
 */

// Types
export * from "./types";

// REST Client
export { initRestClient, getRestClient, RestClient } from "./rest/client";
export { api } from "./rest/endpoints";

// Connect-RPC Client
export {
  userClient,
  healthClient,
  infoClient,
  createAuthenticatedClients,
  type UserClient,
  type HealthClient,
  type InfoClient,
} from "./connect/services";

// Storage Client
export {
  StorageClient,
  initStorageClient,
  getStorageClient,
  storage,
} from "./storage/client";

// Legacy exports for backwards compatibility
// TODO: Diese können später entfernt werden, wenn alle Imports aktualisiert sind
import { initRestClient } from "./rest/client";
import { initStorageClient } from "./storage/client";

export function initApiClient(getToken: () => Promise<string | null>) {
  return initRestClient(getToken);
}
