/**
 * API Configuration
 *
 * Zentralisierte Konfiguration für API-Verbindungen
 */

import { browser } from '$app/environment'

/**
 * API Version Konstante
 */
export const API_VERSION = '/api/v1'

/**
 * API Base URL aus Environment oder Default
 */
export function getApiBaseUrl(): string {
	if (!browser) return 'http://localhost:3000'
	return import.meta.env.VITE_API_URL || 'http://localhost:3000'
}

/**
 * Connect-RPC Base URL
 */
export function getConnectBaseUrl(): string {
	return `${getApiBaseUrl()}${API_VERSION}/connect`
}

/**
 * Vollständige API URL
 */
export function getApiUrl(): string {
	return `${getApiBaseUrl()}${API_VERSION}`
}

/**
 * API Konfiguration Interface
 */
export interface ApiConfig {
	baseUrl: string
	version: string
	fullUrl: string
	connectUrl: string
	timeout: number
}

/**
 * Hole vollständige API Konfiguration
 */
export function getApiConfig(): ApiConfig {
	const baseUrl = getApiBaseUrl()
	return {
		baseUrl,
		version: API_VERSION,
		fullUrl: `${baseUrl}${API_VERSION}`,
		connectUrl: `${baseUrl}${API_VERSION}/connect`,
		timeout: 30000,
	}
}
