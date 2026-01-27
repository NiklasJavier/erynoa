/**
 * Auth Module - Hauptexport
 */

// OIDC Functions (nicht die isAuthenticated Funktion exportieren, um Konflikte zu vermeiden)
export {
	initAuth,
	getAuth,
	login,
	logout,
	handleCallback,
	getUser,
	getAccessToken,
	type AuthState,
} from './oidc'

// Svelte Stores
export {
	authStore,
	user,
	isAuthenticated,
	isLoading,
	authError,
} from './store'
