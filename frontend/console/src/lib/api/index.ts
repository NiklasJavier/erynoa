/**
 * API Module - Hauptexport
 *
 * Zentrale Export-Datei für alle API-Clients und Types
 *
 * @example
 * ```ts
 * import { api, createAuthenticatedClients } from '$lib/api';
 *
 * // Info abrufen
 * const info = await api.info.getInfo({});
 *
 * // Mit Auth
 * const authClients = createAuthenticatedClients(() => getToken());
 * const users = await authClients.users.list({});
 * ```
 */

// Re-export clients
export {
	userClient,
	healthClient,
	infoClient,
	storageClient,
	createAuthenticatedClients,
	type UserClient,
	type HealthClient,
	type InfoClient,
	type StorageClient,
	type AuthenticatedClients,
} from './clients'

// Re-export config
export {
	getApiBaseUrl,
	getApiUrl,
	getConnectBaseUrl,
	getApiConfig,
	API_VERSION,
	type ApiConfig,
} from './config'

// Re-export transport
export {
	createBaseTransport,
	createAuthenticatedTransport,
} from './transport'

// Convenience object for base clients
export const api = {
	get users() {
		return import('./clients').then((m) => m.userClient())
	},
	get health() {
		return import('./clients').then((m) => m.healthClient())
	},
	get info() {
		return import('./clients').then((m) => m.infoClient())
	},
	get storage() {
		return import('./clients').then((m) => m.storageClient())
	},
}

// Re-export generated types
export * from '$gen/erynoa/v1/user_pb'
export * from '$gen/erynoa/v1/health_pb'
export * from '$gen/erynoa/v1/info_pb'
export * from '$gen/erynoa/v1/storage_pb'
