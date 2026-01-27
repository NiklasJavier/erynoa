/**
 * Connect-RPC Service Clients
 *
 * Typisierte Service-Clients für alle Backend-Services
 */

import { HealthService } from '$gen/erynoa/v1/health_connect'
import { InfoService } from '$gen/erynoa/v1/info_connect'
import { StorageService } from '$gen/erynoa/v1/storage_connect'
import { UserService } from '$gen/erynoa/v1/user_connect'
import { type PromiseClient, createPromiseClient } from '@connectrpc/connect'
import { createAuthenticatedTransport, createBaseTransport } from './transport'

// Client Types
export type UserClient = PromiseClient<typeof UserService>
export type HealthClient = PromiseClient<typeof HealthService>
export type InfoClient = PromiseClient<typeof InfoService>
export type StorageClient = PromiseClient<typeof StorageService>

// Base Transport (Singleton)
let baseTransport: ReturnType<typeof createBaseTransport> | null = null

function getBaseTransport() {
	if (!baseTransport) {
		baseTransport = createBaseTransport()
	}
	return baseTransport
}

// Base Clients (ohne Auth)
export const userClient = (): UserClient => createPromiseClient(UserService, getBaseTransport())

export const healthClient = (): HealthClient =>
	createPromiseClient(HealthService, getBaseTransport())

export const infoClient = (): InfoClient => createPromiseClient(InfoService, getBaseTransport())

export const storageClient = (): StorageClient =>
	createPromiseClient(StorageService, getBaseTransport())

/**
 * Authenticated Clients Factory
 */
export interface AuthenticatedClients {
	users: UserClient
	health: HealthClient
	info: InfoClient
	storage: StorageClient
}

export function createAuthenticatedClients(
	getToken: () => Promise<string | null>
): AuthenticatedClients {
	const transport = createAuthenticatedTransport(getToken)

	return {
		users: createPromiseClient(UserService, transport),
		health: createPromiseClient(HealthService, transport),
		info: createPromiseClient(InfoService, transport),
		storage: createPromiseClient(StorageService, transport),
	}
}
