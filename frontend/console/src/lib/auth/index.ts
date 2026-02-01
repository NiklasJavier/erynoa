/**
 * Auth Module - Hauptexport
 *
 * Erynoa unterstützt zwei Authentifizierungsmethoden:
 * 1. OIDC - Für Integration mit externen Identity Providern
 * 2. Passkey - Für dezentrale, Passkey-basierte DID-Authentifizierung
 */

// ============================================================================
// OIDC AUTHENTICATION
// ============================================================================

// OIDC Functions (nicht die isAuthenticated Funktion exportieren, um Konflikte zu vermeiden)
export {
	getAccessToken,
	getAuth,
	getUser,
	handleCallback,
	initAuth,
	login,
	logout,
	type AuthState,
} from './oidc'

// Svelte Stores
export {
	authError,
	authStore,
	isAuthenticated,
	isLoading,
	user,
} from './store'

// ============================================================================
// PASSKEY AUTHENTICATION (Dezentrale DID-basierte Auth)
// ============================================================================

// Re-export all passkey functionality
export * from './passkey'

// Explicit re-exports for better IDE support
export {
	// Types
	PasskeyErrorCode,
	activePasskeyCredential,
	activePasskeyDid,
	authenticateWithPasskey,
	// Service Functions
	checkPasskeySupport,
	clearActiveDid,
	createPasskeyDid,
	fetchChallenge,
	formatDidShort,
	// Utilities
	generateErynoaDid,
	generateLocalChallenge,
	getActiveCredential,
	getActiveDid,
	getCredentialForDid,
	getStoredCredentials,
	hasPasskeyRegistered,
	isPasskeyAuthenticated,
	isPasskeyAvailable,
	isPasskeyInitialized,
	isPasskeyLoading,
	isValidDid,
	passkeyCredentials,
	passkeyError,
	// Store
	passkeyStore,
	passkeySupport,
	registerPasskey,
	setActiveDid,
	signWithPasskey,
	type PasskeyAuthenticationOptions,
	type PasskeyAuthenticationResult,
	type PasskeyDID,
	type PasskeyRegistrationOptions,
	type PasskeyRegistrationResult,
	type PasskeySignOptions,
	type PasskeySignatureResult,
	type PasskeySupport,
	type StoredPasskeyCredential,
} from './passkey'
