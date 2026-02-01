/**
 * Passkey Types & Constants
 *
 * TypeScript-Definitionen für WebAuthn/Passkey-basierte DID-Erstellung.
 * Unterstützt Ed25519 (alg: -8) als primären Algorithmus für Kompatibilität
 * mit dem bestehenden Erynoa DID-System.
 *
 * @module auth/passkey/types
 */

import type {
	AuthenticationResponseJSON,
	AuthenticatorTransportFuture,
	PublicKeyCredentialCreationOptionsJSON,
	RegistrationResponseJSON,
} from '@simplewebauthn/types'

// ============================================================================
// KONSTANTEN
// ============================================================================

/**
 * Erynoa Relying Party Konfiguration
 */
export const ERYNOA_RP_CONFIG = {
	/** Name der Relying Party */
	name: 'Erynoa',
	/** Icon URL (optional) */
	icon: '/favicon.png',
} as const

/**
 * WebAuthn Algorithmus-Konstanten (COSE Algorithm Identifiers)
 * @see https://www.iana.org/assignments/cose/cose.xhtml#algorithms
 */
export const COSE_ALGORITHMS = {
	/** Ed25519 - Primär für Erynoa (best performance, smallest keys) */
	Ed25519: -8,
	/** ES256 (ECDSA w/ SHA-256 on P-256) - Fallback */
	ES256: -7,
	/** RS256 (RSASSA-PKCS1-v1_5 w/ SHA-256) - Legacy Fallback */
	RS256: -257,
} as const

/**
 * Unterstützte Public Key Credential Parameter
 * Ed25519 zuerst für optimale Kompatibilität mit Erynoa's Ed25519-basiertem DID-System
 */
export const SUPPORTED_PUB_KEY_PARAMS: PublicKeyCredentialParameters[] = [
	{ type: 'public-key', alg: COSE_ALGORITHMS.Ed25519 },
	{ type: 'public-key', alg: COSE_ALGORITHMS.ES256 },
]

/**
 * Ed25519-only für strikte Kompatibilität
 */
export const ED25519_ONLY_PUB_KEY_PARAMS: PublicKeyCredentialParameters[] = [
	{ type: 'public-key', alg: COSE_ALGORITHMS.Ed25519 },
]

/**
 * Authenticator Selection Criteria
 */
export const AUTHENTICATOR_SELECTION = {
	/** Resident Key erforderlich für Discoverable Credentials */
	residentKey: 'required' as const,
	/** User Verification bevorzugt (Biometrie/PIN) */
	userVerification: 'preferred' as const,
	/** Authenticator Attachment - plattformübergreifend */
	authenticatorAttachment: undefined, // Beide erlauben: platform (TouchID) und cross-platform (YubiKey)
}

/**
 * Attestation Conveyance Preference
 * 'direct' für volle Attestation, 'none' für Privacy
 */
export const ATTESTATION_PREFERENCE = 'direct' as const

/**
 * Challenge Timeout in Millisekunden (2 Minuten)
 */
export const CHALLENGE_TIMEOUT_MS = 120_000

/**
 * Challenge Gültigkeit in Sekunden (für Backend-Validierung)
 */
export const CHALLENGE_MAX_AGE_SECS = 300

/**
 * LocalStorage Keys für Passkey-Persistenz
 */
export const PASSKEY_STORAGE_KEYS = {
	/** Gespeicherte Credential-IDs */
	CREDENTIALS: 'erynoa_passkey_credentials',
	/** Aktuell ausgewählte Passkey-DID */
	ACTIVE_DID: 'erynoa_passkey_did',
	/** Backup der Public Keys (für Offline-Nutzung) */
	PUBLIC_KEYS: 'erynoa_passkey_pubkeys',
	/** Letzte erfolgreiche Authentifizierung */
	LAST_AUTH: 'erynoa_passkey_last_auth',
} as const

// ============================================================================
// TYPEN
// ============================================================================

/**
 * Erynoa DID Namespace Typ (entspricht Backend DIDNamespace)
 */
export type ErynoaNamespace =
	| 'self'
	| 'guild'
	| 'spirit'
	| 'thing'
	| 'vessel'
	| 'source'
	| 'craft'
	| 'vault'
	| 'pact'
	| 'circle'

/**
 * Gespeicherte Passkey-Credential Informationen
 */
export interface StoredPasskeyCredential {
	/** Credential ID (Base64URL encoded) */
	id: string
	/** Raw Credential ID (für WebAuthn API) */
	rawId: string
	/** Public Key (Base64URL encoded) */
	publicKey: string
	/** Public Key Algorithmus (COSE Algorithm ID) */
	algorithm: number
	/** Generierte DID */
	did: string
	/** DID Namespace */
	namespace: ErynoaNamespace
	/** Erstellungszeitpunkt (Unix Timestamp) */
	createdAt: number
	/** Letzter Authentifizierungszeitpunkt */
	lastUsedAt?: number
	/** Authenticator Transports (usb, nfc, ble, internal) */
	transports?: AuthenticatorTransportFuture[]
	/** Benutzerfreundlicher Name */
	displayName?: string
	/** AAGUID des Authenticators (wenn verfügbar) */
	aaguid?: string
	/** Ist dies die primäre Identität? */
	isPrimary?: boolean
}

/**
 * Passkey-basierte DID mit zugehörigen Metadaten
 */
export interface PasskeyDID {
	/** Vollständige DID (did:erynoa:self:...) */
	did: string
	/** Namespace */
	namespace: ErynoaNamespace
	/** Unique ID Teil der DID */
	uniqueId: string
	/** Public Key (Hex encoded, für Backend-Kompatibilität) */
	publicKeyHex: string
	/** Public Key (Raw Bytes) */
	publicKeyBytes: Uint8Array
	/** Algorithmus */
	algorithm: number
	/** Erstellungszeitpunkt */
	createdAt: Date
}

/**
 * Challenge Response vom Backend
 */
export interface ChallengeResponse {
	/** Base64URL encoded Challenge */
	challenge: string
	/** Challenge ID für Tracking (optional) */
	challengeId?: string
	/** Ablaufzeit (Unix Timestamp) */
	expiresAt?: number
	/** Zusätzliche Optionen vom Server */
	options?: Partial<PublicKeyCredentialCreationOptionsJSON>
}

/**
 * Registrierungs-Ergebnis
 */
export interface PasskeyRegistrationResult {
	/** Ob die Registrierung erfolgreich war */
	success: boolean
	/** Generierte DID */
	did?: PasskeyDID
	/** Gespeicherte Credential-Informationen */
	credential?: StoredPasskeyCredential
	/** Raw Registration Response (für Backend-Verifizierung) */
	response?: RegistrationResponseJSON
	/** Fehlermeldung bei Fehler */
	error?: string
	/** Fehlercode für programmatische Behandlung */
	errorCode?: PasskeyErrorCode
}

/**
 * Authentifizierungs-Ergebnis
 */
export interface PasskeyAuthenticationResult {
	/** Ob die Authentifizierung erfolgreich war */
	success: boolean
	/** Verwendete DID */
	did?: string
	/** Signatur (Base64URL encoded) */
	signature?: string
	/** Raw Authentication Response */
	response?: AuthenticationResponseJSON
	/** Fehlermeldung bei Fehler */
	error?: string
	/** Fehlercode */
	errorCode?: PasskeyErrorCode
}

/**
 * Signatur-Ergebnis für Intent/Saga Signierung
 */
export interface PasskeySignatureResult {
	/** Ob die Signierung erfolgreich war */
	success: boolean
	/** Signatur (Raw Bytes) */
	signatureBytes?: Uint8Array
	/** Signatur (Hex encoded, für Backend) */
	signatureHex?: string
	/** Signatur (Base64URL encoded) */
	signatureBase64?: string
	/** Verwendete DID */
	did?: string
	/** Challenge die signiert wurde */
	challenge?: string
	/** Fehlermeldung */
	error?: string
	/** Fehlercode */
	errorCode?: PasskeyErrorCode
}

/**
 * Passkey-spezifische Fehlercodes
 */
export enum PasskeyErrorCode {
	/** Kein Fehler */
	NONE = 'none',
	/** Browser unterstützt WebAuthn nicht */
	NOT_SUPPORTED = 'not_supported',
	/** Benutzer hat abgebrochen */
	USER_CANCELLED = 'user_cancelled',
	/** Credential existiert bereits */
	CREDENTIAL_EXISTS = 'credential_exists',
	/** Credential nicht gefunden */
	CREDENTIAL_NOT_FOUND = 'credential_not_found',
	/** Challenge ungültig oder abgelaufen */
	INVALID_CHALLENGE = 'invalid_challenge',
	/** Challenge vom Backend konnte nicht geholt werden */
	CHALLENGE_FETCH_FAILED = 'challenge_fetch_failed',
	/** Signatur-Verifikation fehlgeschlagen */
	VERIFICATION_FAILED = 'verification_failed',
	/** Authenticator nicht verfügbar */
	AUTHENTICATOR_NOT_AVAILABLE = 'authenticator_not_available',
	/** Timeout bei der Operation */
	TIMEOUT = 'timeout',
	/** Algorithmus nicht unterstützt */
	UNSUPPORTED_ALGORITHM = 'unsupported_algorithm',
	/** Ed25519 nicht vom Authenticator unterstützt */
	ED25519_NOT_SUPPORTED = 'ed25519_not_supported',
	/** Netzwerkfehler */
	NETWORK_ERROR = 'network_error',
	/** Speicherfehler (localStorage) */
	STORAGE_ERROR = 'storage_error',
	/** Unbekannter Fehler */
	UNKNOWN = 'unknown',
}

/**
 * Passkey-Feature Support Status
 */
export interface PasskeySupport {
	/** WebAuthn API verfügbar */
	webauthnAvailable: boolean
	/** Platform Authenticator verfügbar (TouchID, FaceID, Windows Hello) */
	platformAuthenticatorAvailable: boolean
	/** Conditional UI verfügbar (Autofill) */
	conditionalUIAvailable: boolean
	/** Ed25519 unterstützt */
	ed25519Supported: boolean
	/** User Verifying Platform Authenticator verfügbar */
	uvpaAvailable: boolean
}

/**
 * Optionen für Passkey-Registrierung
 */
export interface PasskeyRegistrationOptions {
	/** DID Namespace (default: 'self') */
	namespace?: ErynoaNamespace
	/** Benutzerfreundlicher Name für die Identität */
	displayName?: string
	/** Username/Identifier für den Authenticator */
	username?: string
	/** Ed25519 erzwingen (keine Fallback-Algorithmen) */
	forceEd25519?: boolean
	/** Platform Authenticator bevorzugen (TouchID, FaceID) */
	preferPlatformAuthenticator?: boolean
	/** Als primäre Identität setzen */
	setPrimary?: boolean
	/** Attestation-Typ */
	attestation?: AttestationConveyancePreference
	/** Custom Timeout (ms) */
	timeout?: number
}

/**
 * Optionen für Passkey-Authentifizierung
 */
export interface PasskeyAuthenticationOptions {
	/** Spezifische Credential-ID verwenden */
	credentialId?: string
	/** Spezifische DID verwenden */
	did?: string
	/** User Verification erzwingen */
	requireUserVerification?: boolean
	/** Custom Timeout (ms) */
	timeout?: number
}

/**
 * Optionen für Message-Signierung
 */
export interface PasskeySignOptions {
	/** Spezifische Credential-ID verwenden */
	credentialId?: string
	/** Spezifische DID verwenden */
	did?: string
	/** Challenge (wenn nicht vom Backend geholt werden soll) */
	challenge?: Uint8Array
	/** User Verification erzwingen */
	requireUserVerification?: boolean
}

/**
 * Public Key Credential Parameter (für WebAuthn API)
 */
export interface PublicKeyCredentialParameters {
	type: 'public-key'
	alg: number
}

/**
 * Passkey-Manager State für Svelte Store
 */
export interface PasskeyState {
	/** Feature-Support Status */
	support: PasskeySupport | null
	/** Alle gespeicherten Credentials */
	credentials: StoredPasskeyCredential[]
	/** Aktuell aktive DID */
	activeDid: string | null
	/** Aktuell aktives Credential */
	activeCredential: StoredPasskeyCredential | null
	/** Initialisiert */
	initialized: boolean
	/** Lädt gerade */
	loading: boolean
	/** Letzte Fehlermeldung */
	error: string | null
	/** Letzter Fehlercode */
	errorCode: PasskeyErrorCode | null
}

/**
 * WebAuthn Error Event Interface
 */
export interface WebAuthnError extends Error {
	name:
		| 'AbortError'
		| 'ConstraintError'
		| 'InvalidStateError'
		| 'NotAllowedError'
		| 'NotSupportedError'
		| 'SecurityError'
		| 'TypeError'
		| 'UnknownError'
}

/**
 * Type Guard für WebAuthn Errors
 */
export function isWebAuthnError(error: unknown): error is WebAuthnError {
	return (
		error instanceof Error &&
		[
			'AbortError',
			'ConstraintError',
			'InvalidStateError',
			'NotAllowedError',
			'NotSupportedError',
			'SecurityError',
			'TypeError',
			'UnknownError',
		].includes(error.name)
	)
}

/**
 * Mappt WebAuthn Error auf PasskeyErrorCode
 */
export function mapWebAuthnErrorToCode(error: unknown): PasskeyErrorCode {
	if (!isWebAuthnError(error)) {
		return PasskeyErrorCode.UNKNOWN
	}

	switch (error.name) {
		case 'AbortError':
			return PasskeyErrorCode.TIMEOUT
		case 'NotAllowedError':
			return PasskeyErrorCode.USER_CANCELLED
		case 'InvalidStateError':
			return PasskeyErrorCode.CREDENTIAL_EXISTS
		case 'NotSupportedError':
			return PasskeyErrorCode.NOT_SUPPORTED
		case 'SecurityError':
			return PasskeyErrorCode.VERIFICATION_FAILED
		case 'ConstraintError':
			return PasskeyErrorCode.UNSUPPORTED_ALGORITHM
		default:
			return PasskeyErrorCode.UNKNOWN
	}
}
