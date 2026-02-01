/**
 * Application Configuration
 *
 * Lädt Konfiguration vom Backend und bietet Fallback-Werte
 */

import { browser } from '$app/environment'
import { z } from 'zod'

// Schema
export const ConfigSchema = z.object({
	environment: z.enum(['development', 'staging', 'production', 'local']),
	version: z.string().min(1),
	urls: z.object({
		console: z.string().url(),
		api: z.string().url(),
	}),
	features: z
		.object({
			registration: z.boolean(),
			socialLogin: z.boolean(),
		})
		.optional(),
})

export type Config = z.infer<typeof ConfigSchema>

// Default Konfiguration (Fallback)
export const DEFAULT_CONFIG: Config = {
	environment: 'local',
	version: '0.1.0',
	urls: {
		console: 'http://localhost:3001/console',
		api: 'http://localhost:3000',
	},
	features: {
		registration: true,
		socialLogin: false,
	},
}

// Config Cache
let cachedConfig: Config | null = null

/**
 * Lade Konfiguration vom Backend
 */
export async function fetchConfig(forceReload = false): Promise<Config> {
	if (cachedConfig && !forceReload) return cachedConfig
	if (!browser) return DEFAULT_CONFIG

	try {
		// Clear cache if forcing reload
		if (forceReload) {
			cachedConfig = null
		}

		const { infoClient } = await import('$lib/api/clients')
		const client = infoClient()
		const response = await client.getInfo({})

		const config: Config = {
			environment: (response.environment as Config['environment']) || 'local',
			version: response.version || '0.1.0',
			auth: {
				issuer: response.auth?.issuer || DEFAULT_CONFIG.auth.issuer,
				clientId: response.auth?.clientId || DEFAULT_CONFIG.auth.clientId,
			},
			urls: {
				console: response.urls?.console || DEFAULT_CONFIG.urls.console,
				api: response.urls?.api || DEFAULT_CONFIG.urls.api,
			},
			features: {
				registration: response.features?.registration ?? true,
				socialLogin: response.features?.socialLogin ?? false,
			},
		}

		// Validiere und cache
		cachedConfig = ConfigSchema.parse(config)
		console.log('[Config] Loaded from backend:', {
			environment: cachedConfig.environment,
			clientId: cachedConfig.auth.clientId,
			issuer: cachedConfig.auth.issuer,
		})
		return cachedConfig
	} catch (error) {
		console.error('[Config] Failed to load from backend, using defaults', error)
		// Log detailed error for debugging
		if (error instanceof Error) {
			console.error('[Config] Error details:', error.message, error.stack)
		}
		return DEFAULT_CONFIG
	}
}

/**
 * Hole gecachte Config synchron (oder default)
 */
export function getAppConfig(): Config {
	return cachedConfig || DEFAULT_CONFIG
}

/**
 * Setze Config (für Tests)
 */
export function setConfig(config: Config): void {
	cachedConfig = config
}

/**
 * Leere Config-Cache (z.B. nach Client-ID-Änderungen)
 */
export function clearConfigCache(): void {
	cachedConfig = null
}
