/**
 * Connect-RPC Client Setup
 * Uses binary protobuf for efficient communication
 */

import { createConnectTransport } from "@connectrpc/connect-web";
import { createPromiseClient, type PromiseClient } from "@connectrpc/connect";
import { UserService } from "../gen/godstack/v1/user_connect";
import { HealthService } from "../gen/godstack/v1/health_connect";
import { InfoService } from "../gen/godstack/v1/info_connect";

// Transport configuration
const transport = createConnectTransport({
  baseUrl: import.meta.env.VITE_API_URL || "",
  // Use binary protobuf for better performance
  // Falls back to JSON if needed
});

// Create typed service clients
export const userClient = createPromiseClient(UserService, transport);
export const healthClient = createPromiseClient(HealthService, transport);
export const infoClient = createPromiseClient(InfoService, transport);

// Export client types
export type UserClient = PromiseClient<typeof UserService>;
export type HealthClient = PromiseClient<typeof HealthService>;
export type InfoClient = PromiseClient<typeof InfoService>;

/**
 * Create authenticated transport with token injection
 */
export function createAuthenticatedTransport(getToken: () => Promise<string | null>) {
  return createConnectTransport({
    baseUrl: import.meta.env.VITE_API_URL || "",
    interceptors: [
      (next) => async (req) => {
        const token = await getToken();
        if (token) {
          req.header.set("Authorization", `Bearer ${token}`);
        }
        return next(req);
      },
    ],
  });
}

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
