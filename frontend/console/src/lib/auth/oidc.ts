/**
 * OIDC Authentication für Svelte 5
 *
 * Verwendet oidc-client-ts für PKCE Flow (kein Client Secret für SPAs)
 * Kompatibel mit verschiedenen Identity Providern (Keycloak, Auth0, ZITADEL, etc.)
 */

import { browser } from '$app/environment'
import { base } from '$app/paths'
import { type User, UserManager, WebStorageStateStore } from 'oidc-client-ts'

// Auth State Types
export interface AuthState {
	user: User | null
	isAuthenticated: boolean
	isLoading: boolean
	error: string | null
}

// Singleton UserManager
let userManager: UserManager | null = null
let currentClientId: string | null = null

/**
 * Initialisiere den OIDC UserManager
 * Wird neu initialisiert, wenn sich clientId ändert
 *
 * @param issuer - OIDC Issuer URL
 * @param clientId - OIDC Client ID
 * @param consoleUrl - Vollständige Console URL (z.B. http://localhost:3001/console) für Redirect-URIs
 */
export function initAuth(issuer: string, clientId: string, consoleUrl?: string): UserManager {
	if (!browser) {
		throw new Error('Auth can only be initialized in the browser')
	}

	// Re-initialize if clientId changed
	if (userManager && currentClientId === clientId) {
		return userManager
	}

	// Reset if clientId changed
	if (userManager && currentClientId !== clientId) {
		console.log('[Auth] Client ID changed, re-initializing UserManager', {
			old: currentClientId,
			new: clientId,
		})
		userManager = null
	}

	currentClientId = clientId

	// Verwende Console-URL aus Config für exakte Übereinstimmung mit IDP
	// Falls nicht angegeben, Fallback auf dynamische Generierung
	// Normalisiere URL: entferne trailing slash, füge /callback hinzu
	let redirectUri: string
	let postLogoutRedirectUri: string

	if (consoleUrl) {
		// Entferne trailing slash falls vorhanden
		const normalizedUrl = consoleUrl.replace(/\/+$/, '')
		redirectUri = `${normalizedUrl}/callback`
		postLogoutRedirectUri = normalizedUrl
	} else {
		// Fallback auf dynamische Generierung
		redirectUri = `${window.location.origin}${base}/callback`
		postLogoutRedirectUri = `${window.location.origin}${base}`
	}

	console.log('[Auth] Initializing with:', {
		issuer,
		clientId,
		redirectUri,
		postLogoutRedirectUri,
		consoleUrl,
		windowLocation: window.location.href,
		base,
	})

	userManager = new UserManager({
		authority: issuer,
		client_id: clientId,
		redirect_uri: redirectUri,
		post_logout_redirect_uri: postLogoutRedirectUri,
		response_type: 'code',
		scope: 'openid profile email',
		automaticSilentRenew: true,
		userStore: new WebStorageStateStore({ store: window.localStorage }),
		loadUserInfo: true,
		// OIDC endpoints (configured for the identity provider)
		metadata: {
			issuer: issuer,
			authorization_endpoint: `${issuer}/oauth/v2/authorize`,
			token_endpoint: `${issuer}/oauth/v2/token`,
			userinfo_endpoint: `${issuer}/oidc/v1/userinfo`,
			end_session_endpoint: `${issuer}/oidc/v1/end_session`,
			jwks_uri: `${issuer}/.well-known/jwks.json`,
		},
	})

	// Event Handlers
	userManager.events.addSilentRenewError((error) => {
		console.error('[Auth] Silent renew error:', error)
	})

	userManager.events.addUserLoaded((user) => {
		console.log('[Auth] User loaded:', user?.profile?.preferred_username)
	})

	userManager.events.addUserUnloaded(() => {
		console.log('[Auth] User unloaded')
	})

	userManager.events.addAccessTokenExpiring(() => {
		console.log('[Auth] Token expiring, will renew')
	})

	return userManager
}

/**
 * Hole UserManager (muss bereits initialisiert sein)
 */
export function getAuth(): UserManager | null {
	return userManager
}

/**
 * Login starten
 */
export async function login(): Promise<void> {
	if (!userManager) {
		throw new Error('Auth not initialized')
	}
	await userManager.signinRedirect()
}

/**
 * Logout
 */
export async function logout(): Promise<void> {
	if (!userManager) {
		throw new Error('Auth not initialized')
	}
	await userManager.signoutRedirect()
}

/**
 * Callback verarbeiten (nach OIDC Redirect)
 */
export async function handleCallback(): Promise<User> {
	if (!userManager) {
		throw new Error('Auth not initialized')
	}
	return await userManager.signinRedirectCallback()
}

/**
 * Aktuellen User holen
 */
export async function getUser(): Promise<User | null> {
	if (!userManager) return null
	return await userManager.getUser()
}

/**
 * Access Token holen
 */
export async function getAccessToken(): Promise<string | null> {
	const user = await getUser()
	return user?.access_token ?? null
}

/**
 * Prüfe ob User authentifiziert ist
 */
export async function isAuthenticated(): Promise<boolean> {
	const user = await getUser()
	return !!user && !user.expired
}
