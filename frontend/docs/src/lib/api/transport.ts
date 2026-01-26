/**
 * Connect-RPC Transport
 * 
 * Transport-Layer für Connect-RPC mit Interceptors für:
 * - Logging
 * - Error Handling
 * - Authentication
 */

import { createConnectTransport } from '@connectrpc/connect-web';
import type { Interceptor, Transport } from '@connectrpc/connect';
import { getConnectBaseUrl } from './config';

/**
 * Logging Interceptor (nur Development)
 */
const loggingInterceptor: Interceptor = (next) => async (req) => {
  const service = req.service.typeName;
  const method = req.method.name;
  const startTime = performance.now();

  if (import.meta.env.DEV) {
    console.log(`[API] → ${service}.${method}`);
  }

  try {
    const response = await next(req);
    const duration = Math.round(performance.now() - startTime);

    if (import.meta.env.DEV) {
      console.log(`[API] ← ${service}.${method} (${duration}ms)`);
    }

    return response;
  } catch (error) {
    const duration = Math.round(performance.now() - startTime);
    console.error(`[API] ✗ ${service}.${method} (${duration}ms)`, error);
    throw error;
  }
};

/**
 * Error Handling Interceptor
 */
const errorInterceptor: Interceptor = (next) => async (req) => {
  try {
    return await next(req);
  } catch (error: unknown) {
    const { ConnectError } = await import('@connectrpc/connect');
    
    if (error instanceof ConnectError) {
      // Map Connect-RPC error codes
      const codeMap: Record<string, string> = {
        'unauthenticated': 'UNAUTHORIZED',
        'permission_denied': 'FORBIDDEN',
        'not_found': 'NOT_FOUND',
        'already_exists': 'CONFLICT',
        'invalid_argument': 'BAD_REQUEST',
        'failed_precondition': 'PRECONDITION_FAILED',
        'unavailable': 'SERVICE_UNAVAILABLE',
      };
      
      const mappedCode = codeMap[error.code] || 'UNKNOWN';
      console.error(`[API Error] ${mappedCode}: ${error.message}`);
    }
    
    throw error;
  }
};

/**
 * Auth Interceptor Factory
 */
function createAuthInterceptor(getToken: () => Promise<string | null>): Interceptor {
  return (next) => async (req) => {
    const token = await getToken();
    
    if (token) {
      req.header.set('Authorization', `Bearer ${token}`);
    }
    
    return next(req);
  };
}

/**
 * Base Transport (ohne Auth)
 */
export function createBaseTransport(): Transport {
  return createConnectTransport({
    baseUrl: getConnectBaseUrl(),
    interceptors: [loggingInterceptor, errorInterceptor],
  });
}

/**
 * Authenticated Transport
 */
export function createAuthenticatedTransport(
  getToken: () => Promise<string | null>
): Transport {
  return createConnectTransport({
    baseUrl: getConnectBaseUrl(),
    interceptors: [
      loggingInterceptor,
      createAuthInterceptor(getToken),
      errorInterceptor,
    ],
  });
}
