/**
 * Connect-RPC Transport
 *
 * Transport-Layer für Connect-RPC mit Interceptors für:
 * - Logging
 * - Error Handling
 * - Authentication
 */

import type { Interceptor, Transport } from '@connectrpc/connect'
import { createConnectTransport } from '@connectrpc/connect-web'
import { getConnectBaseUrl } from './config'

/**
 * Logging Interceptor (nur Development)
 */
const loggingInterceptor: Interceptor = (next) => async (req) => {
	const service = req.service.typeName
	const method = req.method.name
	const startTime = performance.now()

	if (import.meta.env.DEV) {
		console.log(`[API] → ${service}.${method}`)
	}

	try {
		const response = await next(req)
		const duration = Math.round(performance.now() - startTime)

		if (import.meta.env.DEV) {
			console.log(`[API] ← ${service}.${method} (${duration}ms)`)
		}

		return response
	} catch (error) {
		const duration = Math.round(performance.now() - startTime)
		console.error(`[API] ✗ ${service}.${method} (${duration}ms)`, error)
		throw error
	}
}

/**
 * Error Handling Interceptor
 */
const errorInterceptor: Interceptor = (next) => async (req) => {
	try {
		return await next(req)
	} catch (error: unknown) {
		const { ConnectError } = await import('@connectrpc/connect')

		if (error instanceof ConnectError) {
			// Map Connect-RPC error codes
			const codeMap: Record<string, string> = {
				unauthenticated: 'UNAUTHORIZED',
				permission_denied: 'FORBIDDEN',
				not_found: 'NOT_FOUND',
				already_exists: 'CONFLICT',
				invalid_argument: 'BAD_REQUEST',
				failed_precondition: 'PRECONDITION_FAILED',
				unavailable: 'SERVICE_UNAVAILABLE',
			}

			const mappedCode = codeMap[error.code] || 'UNKNOWN'
			console.error(`[API Error] ${mappedCode}: ${error.message}`)
		}

		throw error
	}
}

/**
 * Auth Interceptor Factory
 */
function createAuthInterceptor(getToken: () => Promise<string | null>): Interceptor {
	return (next) => async (req) => {
		const token = await getToken()

		if (token) {
			req.header.set('Authorization', `Bearer ${token}`)
		}

		return next(req)
	}
}

/**
 * Frontend Identifier Interceptor
 * Setzt einen Custom-Header, damit das Backend das Frontend identifizieren kann
 */
const frontendIdentifierInterceptor: Interceptor = (next) => async (req) => {
	// Setze X-Frontend-Origin Header basierend auf window.location
	if (typeof window !== 'undefined') {
		const pathname = window.location.pathname
		if (pathname.startsWith('/platform')) {
			req.header.set('X-Frontend-Origin', 'platform')
		} else if (pathname.startsWith('/docs')) {
			req.header.set('X-Frontend-Origin', 'docs')
		} else if (pathname.startsWith('/console')) {
			req.header.set('X-Frontend-Origin', 'console')
		}
	}
	return next(req)
}

/**
 * Base Transport (ohne Auth)
 */
export function createBaseTransport(): Transport {
	return createConnectTransport({
		baseUrl: getConnectBaseUrl(),
		interceptors: [frontendIdentifierInterceptor, loggingInterceptor, errorInterceptor],
	})
}

/**
 * Authenticated Transport
 */
export function createAuthenticatedTransport(getToken: () => Promise<string | null>): Transport {
	return createConnectTransport({
		baseUrl: getConnectBaseUrl(),
		interceptors: [
			frontendIdentifierInterceptor,
			loggingInterceptor,
			createAuthInterceptor(getToken),
			errorInterceptor,
		],
	})
}
