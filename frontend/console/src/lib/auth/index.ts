/**
 * Auth Module - Hauptexport
 *
 * Erynoa verwendet Passkey-basierte DID-Authentifizierung.
 * Alle Auth-Funktionen werden aus dem passkey-Modul re-exportiert.
 */

// Re-export all passkey functionality
export * from './passkey'

// Explicit re-exports for better IDE support
export {
	// Types
	PasskeyErrorCode,
	// Derived Stores
	activePasskeyCredential,
	activePasskeyDid,
	// Service Functions
	authenticateWithPasskey,
	checkPasskeySupport,
	clearActiveDid,
	clearAllCredentials,
	deleteCredential,
	fetchChallenge,
	generateLocalChallenge,
	getActiveCredential,
	getActiveDid,
	getCredentialForDid,
	getStoredCredentials,
	hasPasskeyRegistered,
	hasPlatformAuthenticator,
	isPasskeyAuthenticated,
	isPasskeyAvailable,
	isPasskeyAvailable as isPasskeyAvailableStore,
	isPasskeyInitialized,
	isPasskeyLoading,
	passkeyCount,
	passkeyCredentials,
	passkeyError,
	passkeyErrorCode,
	// Store
	passkeyStore,
	passkeySupport,
	primaryCredential,
	registerCredentialWithBackend,
	registerPasskey,
	saveCredential,
	setActiveDid,
	signWithPasskey,
	supportsEd25519,
	verifyAuthenticationWithBackend,
	type ChallengeResponse,
	type ErynoaNamespace,
	type PasskeyAuthenticationOptions,
	type PasskeyAuthenticationResult,
	type PasskeyDID,
	type PasskeyRegistrationOptions,
	type PasskeyRegistrationResult,
	type PasskeySignOptions,
	type PasskeySignatureResult,
	type PasskeyState,
	type PasskeySupport,
	type StoredPasskeyCredential,
} from './passkey'
