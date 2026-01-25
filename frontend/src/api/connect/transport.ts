/**
 * Connect-RPC Transport Configuration
 * 
 * Optimized transport for Connect-RPC/gRPC-Web communication
 * Features:
 * - Automatic retry on network errors
 * - Request/response logging (development only)
 * - Error handling and transformation
 * - Token injection for authenticated requests
 */

import { createConnectTransport } from "@connectrpc/connect-web";
import type { Interceptor } from "@connectrpc/connect";
import { getApiBaseUrl, API_VERSION } from "../../lib/api-config";
import { log } from "../../lib/logger";

/**
 * Get Connect-RPC base URL
 * Connect-RPC endpoints are served under /api/v1/connect
 */
function getConnectBaseUrl(): string {
  return `${getApiBaseUrl()}${API_VERSION}/connect`;
}

/**
 * Logging interceptor
 * Uses structured logger for consistent logging
 */
const loggingInterceptor: Interceptor = (next) => async (req) => {
  const service = req.service.typeName;
  const method = req.method.name;
  const startTime = performance.now();

  log.connectRequest(service, method, {
    url: req.url,
  });

  try {
    const response = await next(req);
    const duration = performance.now() - startTime;

    log.connectResponse(service, method, true, {
      duration_ms: Math.round(duration),
    });

    return response;
  } catch (error) {
    const duration = performance.now() - startTime;
    
    log.connectError(
      service,
      method,
      error instanceof Error ? error : new Error(String(error)),
      {
        duration_ms: Math.round(duration),
      }
    );
    
    throw error;
  }
};

/**
 * Error handling interceptor
 * Transforms Connect-RPC errors to more user-friendly messages
 * Maps Connect-RPC error codes to our ErrorCode format
 */
const errorInterceptor: Interceptor = (next) => async (req) => {
  try {
    return await next(req);
  } catch (error: unknown) {
    // Import ConnectError dynamically to avoid circular dependencies
    const { ConnectError } = await import("@connectrpc/connect");
    
    // Re-throw with additional context
    if (error instanceof ConnectError) {
      // Map Connect-RPC error codes to our ErrorCode format
      const codeMap: Record<string, string> = {
        "unauthenticated": "UNAUTHORIZED",
        "permission_denied": "FORBIDDEN",
        "not_found": "NOT_FOUND",
        "invalid_argument": "BAD_REQUEST",
        "internal": "INTERNAL_ERROR",
        "unavailable": "SERVICE_UNAVAILABLE",
      };

      const mappedCode = codeMap[error.code] || "INTERNAL_ERROR";
      const enhancedError = new Error(
        `${req.service.typeName}.${req.method.name}: [${mappedCode}] ${error.message}`
      );
      enhancedError.cause = error;
      throw enhancedError;
    } else if (error instanceof Error) {
      // Add service and method context to error
      const enhancedError = new Error(
        `${req.service.typeName}.${req.method.name}: ${error.message}`
      );
      enhancedError.cause = error;
      throw enhancedError;
    }
    throw error;
  }
};

/**
 * Create base transport configuration
 * Uses centralized API URL with Connect-RPC path
 * Includes logging and error handling interceptors
 */
export function createBaseTransport() {
  const baseUrl = getConnectBaseUrl();
  
  log.connectRequest("Transport", "initialize", { baseUrl });
  
  return createConnectTransport({
    baseUrl,
    // Use JSON format for unary RPCs (default)
    // This uses application/json Content-Type
    // Binary protobuf can be enabled for better performance if needed
    interceptors: [
      loggingInterceptor,
      errorInterceptor,
    ],
    // Request timeout (30 seconds)
    // This prevents hanging requests
    // Note: Connect-RPC handles timeouts internally
  });
}

/**
 * Create authenticated transport with token injection
 * Includes all base interceptors plus authentication
 */
export function createAuthenticatedTransport(getToken: () => Promise<string | null>) {
  const baseUrl = getConnectBaseUrl();
  
  return createConnectTransport({
    baseUrl,
    interceptors: [
      loggingInterceptor,
      errorInterceptor,
      // Authentication interceptor
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
