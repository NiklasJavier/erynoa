/**
 * Connect-RPC Service Clients
 * 
 * Typed service clients for Connect-RPC endpoints
 */

import { createPromiseClient, type PromiseClient } from "@connectrpc/connect";
import { UserService } from "../../gen/godstack/v1/user_connect";
import { HealthService } from "../../gen/godstack/v1/health_connect";
import { InfoService } from "../../gen/godstack/v1/info_connect";
import { createBaseTransport, createAuthenticatedTransport } from "./transport";

// Base transport (no auth)
const baseTransport = createBaseTransport();

// Create typed service clients (base, no auth)
export const userClient = createPromiseClient(UserService, baseTransport);
export const healthClient = createPromiseClient(HealthService, baseTransport);
export const infoClient = createPromiseClient(InfoService, baseTransport);

// Export client types
export type UserClient = PromiseClient<typeof UserService>;
export type HealthClient = PromiseClient<typeof HealthService>;
export type InfoClient = PromiseClient<typeof InfoService>;

/**
 * Create authenticated service clients
 */
export function createAuthenticatedClients(getToken: () => Promise<string | null>) {
  const transport = createAuthenticatedTransport(getToken);
  
  return {
    users: createPromiseClient(UserService, transport),
    health: createPromiseClient(HealthService, transport),
    info: createPromiseClient(InfoService, transport),
  };
}
