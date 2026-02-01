/**
 * Passkey Module - Hauptexport
 *
 * Exportiert alle Passkey-bezogenen Funktionen, Stores und Typen.
 *
 * @module auth/passkey
 */

// Service Functions
export {
	// Authentication
	authenticateWithPasskey,
	// Feature Detection
	checkPasskeySupport,
	clearActiveDid,
	clearAllCredentials,
	deleteCredential,
	// Challenge Management
	fetchChallenge,
	generateLocalChallenge,
	getActiveCredential,
	getActiveDid,
	getCredentialForDid,
	getStoredCredentials,
	isPasskeyAvailable,
	// Backend Integration
	registerCredentialWithBackend,
	// Registration
	registerPasskey,
	// Credential Storage
	saveCredential,
	setActiveDid,
	// Signing
	signWithPasskey,
	verifyAuthenticationWithBackend,
} from './service'

// Store
export {
	activePasskeyCredential,
	activePasskeyDid,
	hasPasskeyRegistered,
	hasPlatformAuthenticator,
	isPasskeyAuthenticated,
	isPasskeyAvailable as isPasskeyAvailableStore,
	isPasskeyInitialized,
	isPasskeyLoading,
	passkeyCount,
	passkeyCredentials,
	passkeyError,
	passkeyErrorCode,
	passkeyStore,
	// Derived Stores
	passkeySupport,
	primaryCredential,
	supportsEd25519,
} from './store'

// Types
export {
	ATTESTATION_PREFERENCE,
	AUTHENTICATOR_SELECTION,
	CHALLENGE_MAX_AGE_SECS,
	CHALLENGE_TIMEOUT_MS,
	COSE_ALGORITHMS,
	ED25519_ONLY_PUB_KEY_PARAMS,
	// Constants
	ERYNOA_RP_CONFIG,
	PASSKEY_STORAGE_KEYS,
	// Enums
	PasskeyErrorCode,
	SUPPORTED_PUB_KEY_PARAMS,
	// Type Guards
	isWebAuthnError,
	mapWebAuthnErrorToCode,
	type ChallengeResponse,
	// Types
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
	type WebAuthnError,
} from './types'

// Utility Functions
export {
	arrayBufferToBase64Url,
	base58ToUint8Array,
	base64UrlToArrayBuffer,
	base64UrlToHex,
	base64UrlToUint8Array,
	concatUint8Arrays,
	createPasskeyDid,
	extractPublicKeyFromAuthData,
	// COSE Key Parsing
	extractPublicKeyFromCose,
	formatDidShort,
	// DID Generation
	generateDidKeyFromEd25519,
	generateErynoaDid,
	generateUserId,
	hexToBase64Url,
	hexToUint8Array,
	isValidDid,
	parseDid,
	// Utilities
	randomBytes,
	uint8ArrayEquals,
	// Base58
	uint8ArrayToBase58,
	// Base64URL
	uint8ArrayToBase64Url,
	// Hex
	uint8ArrayToHex,
} from './utils'
