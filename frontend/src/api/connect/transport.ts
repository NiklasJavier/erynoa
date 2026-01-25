/**
 * Connect-RPC Transport Configuration
 * 
 * Binary protobuf transport for efficient communication
 */

import { createConnectTransport } from "@connectrpc/connect-web";
import { getApiBaseUrl } from "../../lib/api-config";

/**
 * Create base transport configuration
 * Uses centralized API URL from config
 */

export function createBaseTransport() {
  return createConnectTransport({
    baseUrl: getApiBaseUrl(),
    // Use binary protobuf for better performance
    // Falls back to JSON if needed
  });
}

/**
 * Create authenticated transport with token injection
 */
export function createAuthenticatedTransport(getToken: () => Promise<string | null>) {
  return createConnectTransport({
    baseUrl: getApiBaseUrl(),
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
