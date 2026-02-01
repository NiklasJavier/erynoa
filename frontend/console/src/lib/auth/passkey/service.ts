/**
 * Passkey Service
 *
 * Vollständige WebAuthn/Passkey Implementation für Erynoa.
 * Unterstützt Ed25519-basierte DID-Erstellung und Authentifizierung.
 *
 * @module auth/passkey/service
 */

import { browser } from '$app/environment'
import { startAuthentication, startRegistration } from '@simplewebauthn/browser'
import type {
	AuthenticationResponseJSON,
	PublicKeyCredentialCreationOptionsJSON,
	PublicKeyCredentialRequestOptionsJSON,
	RegistrationResponseJSON,
} from '@simplewebauthn/types'
import {
	ATTESTATION_PREFERENCE,
	AUTHENTICATOR_SELECTION,
	CHALLENGE_TIMEOUT_MS,
	COSE_ALGORITHMS,
	ED25519_ONLY_PUB_KEY_PARAMS,
	ERYNOA_RP_CONFIG,
	PASSKEY_STORAGE_KEYS,
	PasskeyErrorCode,
	SUPPORTED_PUB_KEY_PARAMS,
	mapWebAuthnErrorToCode,
	type ChallengeResponse,
	type PasskeyAuthenticationOptions,
	type PasskeyAuthenticationResult,
	type PasskeyRegistrationOptions,
	type PasskeyRegistrationResult,
	type PasskeySignOptions,
	type PasskeySignatureResult,
	type PasskeySupport,
	type StoredPasskeyCredential,
} from './types'
import {
	arrayBufferToBase64Url,
	base64UrlToUint8Array,
	createPasskeyDid,
	generateUserId,
	uint8ArrayToBase64Url,
	uint8ArrayToHex,
} from './utils'

// ============================================================================
// API CONFIGURATION
// ============================================================================

/**
 * Backend API Base URL
 */
function getApiBaseUrl(): string {
	if (!browser) return ''

	// Verwende die gleiche Origin oder konfigurierte Backend-URL
	const backendUrl = import.meta.env.VITE_BACKEND_URL
	if (backendUrl) return backendUrl

	// Fallback: Gleiche Origin, /api Prefix
	return `${window.location.origin}/api`
}

// ============================================================================
// FEATURE DETECTION
// ============================================================================

/**
 * Prüft WebAuthn/Passkey Feature Support
 *
 * @returns Support Status
 */
export async function checkPasskeySupport(): Promise<PasskeySupport> {
	const support: PasskeySupport = {
		webauthnAvailable: false,
		platformAuthenticatorAvailable: false,
		conditionalUIAvailable: false,
		ed25519Supported: false,
		uvpaAvailable: false,
	}

	if (!browser) return support

	// WebAuthn API Check
	support.webauthnAvailable =
		typeof window !== 'undefined' &&
		typeof window.PublicKeyCredential !== 'undefined' &&
		typeof navigator.credentials !== 'undefined'

	if (!support.webauthnAvailable) return support

	try {
		// Platform Authenticator Check (TouchID, FaceID, Windows Hello)
		support.platformAuthenticatorAvailable =
			await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable()
		support.uvpaAvailable = support.platformAuthenticatorAvailable

		// Conditional UI Check (Autofill)
		if (typeof PublicKeyCredential.isConditionalMediationAvailable === 'function') {
			support.conditionalUIAvailable = await PublicKeyCredential.isConditionalMediationAvailable()
		}

		// Ed25519 Support Check (2026+ Standard)
		// Wir nehmen an, dass es unterstützt wird wenn WebAuthn verfügbar ist
		// Tatsächlicher Test würde eine Registration erfordern
		support.ed25519Supported = true
	} catch (err) {
		console.warn('[PasskeyService] Feature detection error:', err)
	}

	return support
}

/**
 * Prüft ob Passkeys verfügbar sind
 */
export async function isPasskeyAvailable(): Promise<boolean> {
	const support = await checkPasskeySupport()
	return support.webauthnAvailable
}

// ============================================================================
// CHALLENGE MANAGEMENT
// ============================================================================

/**
 * Holt eine neue Challenge vom Backend
 *
 * @returns Challenge Response
 */
export async function fetchChallenge(): Promise<ChallengeResponse> {
	const baseUrl = getApiBaseUrl()

	try {
		const response = await fetch(`${baseUrl}/v1/auth/challenge`, {
			method: 'GET',
			headers: {
				Accept: 'application/json',
			},
		})

		if (!response.ok) {
			throw new Error(`Challenge fetch failed: ${response.status}`)
		}

		return await response.json()
	} catch (err) {
		console.error('[PasskeyService] Challenge fetch error:', err)
		throw err
	}
}

/**
 * Generiert eine lokale Challenge (für Offline-Nutzung oder Entwicklung)
 *
 * @returns Local Challenge Response
 */
export function generateLocalChallenge(): ChallengeResponse {
	const challengeBytes = crypto.getRandomValues(new Uint8Array(32))
	const challenge = uint8ArrayToBase64Url(challengeBytes)

	return {
		challenge,
		expiresAt: Math.floor(Date.now() / 1000) + 300, // 5 Minuten
	}
}

// ============================================================================
// PASSKEY REGISTRATION
// ============================================================================

/**
 * Registriert einen neuen Passkey und erstellt eine DID
 *
 * @param options - Registrierungsoptionen
 * @returns Registrierungsergebnis mit DID
 */
export async function registerPasskey(
	options: PasskeyRegistrationOptions = {}
): Promise<PasskeyRegistrationResult> {
	if (!browser) {
		return {
			success: false,
			error: 'Passkey registration only available in browser',
			errorCode: PasskeyErrorCode.NOT_SUPPORTED,
		}
	}

	try {
		// 1. Feature Support prüfen
		const support = await checkPasskeySupport()
		if (!support.webauthnAvailable) {
			return {
				success: false,
				error: 'WebAuthn is not supported in this browser',
				errorCode: PasskeyErrorCode.NOT_SUPPORTED,
			}
		}

		// 2. Challenge vom Backend holen
		let challengeResponse: ChallengeResponse
		try {
			challengeResponse = await fetchChallenge()
		} catch {
			// Fallback zu lokaler Challenge für Entwicklung
			console.warn('[PasskeyService] Using local challenge (backend unavailable)')
			challengeResponse = generateLocalChallenge()
		}

		// 3. WebAuthn Registration Options vorbereiten
		const rpId = window.location.hostname
		const userId = generateUserId()
		const username = options.username || `erynoa-user-${Date.now()}`
		const displayName = options.displayName || 'Erynoa User'

		const pubKeyCredParams = options.forceEd25519
			? ED25519_ONLY_PUB_KEY_PARAMS
			: SUPPORTED_PUB_KEY_PARAMS

		const registrationOptions: PublicKeyCredentialCreationOptionsJSON = {
			rp: {
				name: ERYNOA_RP_CONFIG.name,
				id: rpId,
			},
			user: {
				id: uint8ArrayToBase64Url(userId),
				name: username,
				displayName: displayName,
			},
			challenge: challengeResponse.challenge,
			pubKeyCredParams: pubKeyCredParams.map((p) => ({
				type: p.type as 'public-key',
				alg: p.alg,
			})),
			authenticatorSelection: {
				...AUTHENTICATOR_SELECTION,
				...(options.preferPlatformAuthenticator && {
					authenticatorAttachment: 'platform' as const,
				}),
			},
			attestation: options.attestation || ATTESTATION_PREFERENCE,
			timeout: options.timeout || CHALLENGE_TIMEOUT_MS,
			// Server-seitige Options überschreiben (wenn vorhanden)
			...challengeResponse.options,
		}

		// 4. WebAuthn Registration durchführen
		console.log('[PasskeyService] Starting registration with options:', {
			rpId,
			username,
			algorithms: pubKeyCredParams.map((p) => p.alg),
		})

		const registrationResponse = await startRegistration({ optionsJSON: registrationOptions })

		// 5. Public Key aus Response extrahieren
		const publicKeyBase64 = registrationResponse.response.publicKey
		if (!publicKeyBase64) {
			// Fallback: Public Key aus authenticatorData extrahieren
			console.warn('[PasskeyService] No publicKey in response, extraction required')
			return {
				success: false,
				error: 'Could not extract public key from registration response',
				errorCode: PasskeyErrorCode.UNKNOWN,
			}
		}

		const publicKeyBytes = base64UrlToUint8Array(publicKeyBase64)

		// 6. Algorithmus aus Response ermitteln
		const algorithm = registrationResponse.response.publicKeyAlgorithm || COSE_ALGORITHMS.Ed25519

		// Warnung wenn nicht Ed25519
		if (algorithm !== COSE_ALGORITHMS.Ed25519) {
			console.warn(
				`[PasskeyService] Authenticator used algorithm ${algorithm} instead of Ed25519 (-8). ` +
					'DID compatibility may be limited.'
			)
		}

		// 7. DID generieren
		const namespace = options.namespace || 'self'
		const did = createPasskeyDid(publicKeyBytes, namespace, algorithm)

		// 8. Credential speichern
		const storedCredential: StoredPasskeyCredential = {
			id: registrationResponse.id,
			rawId: registrationResponse.rawId,
			publicKey: publicKeyBase64,
			algorithm,
			did: did.did,
			namespace,
			createdAt: Date.now(),
			transports: registrationResponse.response.transports,
			displayName: options.displayName,
			isPrimary: options.setPrimary,
		}

		// In LocalStorage speichern
		saveCredential(storedCredential)

		if (options.setPrimary) {
			setActiveDid(did.did)
		}

		console.log('[PasskeyService] Registration successful:', {
			did: did.did,
			algorithm,
			credentialId: registrationResponse.id.substring(0, 16) + '...',
		})

		return {
			success: true,
			did,
			credential: storedCredential,
			response: registrationResponse,
		}
	} catch (err) {
		console.error('[PasskeyService] Registration error:', err)

		const errorCode = mapWebAuthnErrorToCode(err)
		const errorMessage =
			err instanceof Error ? err.message : 'Unknown error during passkey registration'

		return {
			success: false,
			error: errorMessage,
			errorCode,
		}
	}
}

// ============================================================================
// PASSKEY AUTHENTICATION
// ============================================================================

/**
 * Authentifiziert mit einem gespeicherten Passkey
 *
 * @param options - Authentifizierungsoptionen
 * @returns Authentifizierungsergebnis
 */
export async function authenticateWithPasskey(
	options: PasskeyAuthenticationOptions = {}
): Promise<PasskeyAuthenticationResult> {
	if (!browser) {
		return {
			success: false,
			error: 'Passkey authentication only available in browser',
			errorCode: PasskeyErrorCode.NOT_SUPPORTED,
		}
	}

	try {
		// 1. Challenge holen
		let challengeResponse: ChallengeResponse
		try {
			challengeResponse = await fetchChallenge()
		} catch {
			console.warn('[PasskeyService] Using local challenge for authentication')
			challengeResponse = generateLocalChallenge()
		}

		// 2. Gespeicherte Credentials ermitteln
		const credentials = getStoredCredentials()
		let allowCredentials: { id: string; type: 'public-key'; transports?: string[] }[] = []

		if (options.credentialId) {
			// Spezifisches Credential verwenden
			const cred = credentials.find((c) => c.id === options.credentialId)
			if (cred) {
				allowCredentials = [
					{
						id: cred.id,
						type: 'public-key' as const,
						transports: cred.transports as string[] | undefined,
					},
				]
			}
		} else if (options.did) {
			// Credential für DID finden
			const cred = credentials.find((c) => c.did === options.did)
			if (cred) {
				allowCredentials = [
					{
						id: cred.id,
						type: 'public-key' as const,
						transports: cred.transports as string[] | undefined,
					},
				]
			}
		} else if (credentials.length > 0) {
			// Alle gespeicherten Credentials erlauben
			allowCredentials = credentials.map((c) => ({
				id: c.id,
				type: 'public-key' as const,
				transports: c.transports as string[] | undefined,
			}))
		}

		// 3. WebAuthn Authentication Options
		const authOptions: PublicKeyCredentialRequestOptionsJSON = {
			challenge: challengeResponse.challenge,
			rpId: window.location.hostname,
			timeout: options.timeout || CHALLENGE_TIMEOUT_MS,
			userVerification: options.requireUserVerification ? 'required' : 'preferred',
			allowCredentials:
				allowCredentials.length > 0
					? allowCredentials.map((c) => ({
							...c,
							transports: c.transports as ('ble' | 'hybrid' | 'internal' | 'nfc' | 'usb')[],
						}))
					: undefined,
		}

		// 4. WebAuthn Authentication durchführen
		console.log('[PasskeyService] Starting authentication')

		const authResponse = await startAuthentication({ optionsJSON: authOptions })

		// 5. Verwendetes Credential finden
		const usedCredential = credentials.find((c) => c.id === authResponse.id)

		// 6. Last used timestamp aktualisieren
		if (usedCredential) {
			usedCredential.lastUsedAt = Date.now()
			saveCredential(usedCredential)
		}

		console.log('[PasskeyService] Authentication successful:', {
			credentialId: authResponse.id.substring(0, 16) + '...',
			did: usedCredential?.did,
		})

		return {
			success: true,
			did: usedCredential?.did,
			signature: authResponse.response.signature,
			response: authResponse,
		}
	} catch (err) {
		console.error('[PasskeyService] Authentication error:', err)

		const errorCode = mapWebAuthnErrorToCode(err)
		const errorMessage =
			err instanceof Error ? err.message : 'Unknown error during passkey authentication'

		return {
			success: false,
			error: errorMessage,
			errorCode,
		}
	}
}

// ============================================================================
// MESSAGE SIGNING
// ============================================================================

/**
 * Signiert eine Nachricht mit einem Passkey
 *
 * Die Signatur erfolgt über die WebAuthn Assertion, wobei die Challenge
 * die zu signierende Nachricht enthält.
 *
 * @param message - Nachricht zum Signieren (Bytes oder String)
 * @param options - Signierungsoptionen
 * @returns Signaturergebnis
 */
export async function signWithPasskey(
	message: Uint8Array | string,
	options: PasskeySignOptions = {}
): Promise<PasskeySignatureResult> {
	if (!browser) {
		return {
			success: false,
			error: 'Passkey signing only available in browser',
			errorCode: PasskeyErrorCode.NOT_SUPPORTED,
		}
	}

	try {
		// Nachricht zu Bytes konvertieren
		const messageBytes = typeof message === 'string' ? new TextEncoder().encode(message) : message

		// Challenge verwenden oder vom Backend holen
		let challenge: string
		if (options.challenge) {
			challenge = uint8ArrayToBase64Url(options.challenge)
		} else {
			// Challenge = Hash der Nachricht + Timestamp für Replay-Schutz
			const timestamp = Date.now().toString()
			const dataToHash = new Uint8Array([...messageBytes, ...new TextEncoder().encode(timestamp)])
			const hashBuffer = await crypto.subtle.digest('SHA-256', dataToHash)
			challenge = arrayBufferToBase64Url(hashBuffer)
		}

		// Credentials ermitteln
		const credentials = getStoredCredentials()
		let allowCredentials:
			| { id: string; type: 'public-key'; transports?: AuthenticatorTransport[] }[]
			| undefined

		if (options.credentialId) {
			const cred = credentials.find((c) => c.id === options.credentialId)
			if (cred) {
				allowCredentials = [
					{
						id: cred.id,
						type: 'public-key' as const,
						transports: cred.transports as AuthenticatorTransport[] | undefined,
					},
				]
			}
		} else if (options.did) {
			const cred = credentials.find((c) => c.did === options.did)
			if (cred) {
				allowCredentials = [
					{
						id: cred.id,
						type: 'public-key' as const,
						transports: cred.transports as AuthenticatorTransport[] | undefined,
					},
				]
			}
		}

		// WebAuthn Authentication (zum Signieren)
		const authOptions: PublicKeyCredentialRequestOptionsJSON = {
			challenge,
			rpId: window.location.hostname,
			timeout: CHALLENGE_TIMEOUT_MS,
			userVerification: options.requireUserVerification ? 'required' : 'preferred',
			allowCredentials: allowCredentials?.map((c) => ({
				...c,
				transports: c.transports as ('ble' | 'hybrid' | 'internal' | 'nfc' | 'usb')[],
			})),
		}

		const authResponse = await startAuthentication({ optionsJSON: authOptions })

		// Signatur extrahieren
		const signatureBase64 = authResponse.response.signature
		const signatureBytes = base64UrlToUint8Array(signatureBase64)

		// Verwendetes Credential finden
		const usedCredential = credentials.find((c) => c.id === authResponse.id)

		return {
			success: true,
			signatureBytes,
			signatureHex: uint8ArrayToHex(signatureBytes),
			signatureBase64,
			did: usedCredential?.did,
			challenge,
		}
	} catch (err) {
		console.error('[PasskeyService] Signing error:', err)

		const errorCode = mapWebAuthnErrorToCode(err)
		const errorMessage = err instanceof Error ? err.message : 'Unknown error during passkey signing'

		return {
			success: false,
			error: errorMessage,
			errorCode,
		}
	}
}

// ============================================================================
// CREDENTIAL STORAGE
// ============================================================================

/**
 * Speichert ein Credential in LocalStorage
 */
export function saveCredential(credential: StoredPasskeyCredential): void {
	if (!browser) return

	try {
		const credentials = getStoredCredentials()
		const existingIndex = credentials.findIndex((c) => c.id === credential.id)

		if (existingIndex >= 0) {
			credentials[existingIndex] = credential
		} else {
			credentials.push(credential)
		}

		localStorage.setItem(PASSKEY_STORAGE_KEYS.CREDENTIALS, JSON.stringify(credentials))
	} catch (err) {
		console.error('[PasskeyService] Failed to save credential:', err)
	}
}

/**
 * Lädt alle gespeicherten Credentials
 */
export function getStoredCredentials(): StoredPasskeyCredential[] {
	if (!browser) return []

	try {
		const stored = localStorage.getItem(PASSKEY_STORAGE_KEYS.CREDENTIALS)
		return stored ? JSON.parse(stored) : []
	} catch {
		return []
	}
}

/**
 * Löscht ein gespeichertes Credential
 */
export function deleteCredential(credentialId: string): void {
	if (!browser) return

	try {
		const credentials = getStoredCredentials().filter((c) => c.id !== credentialId)
		localStorage.setItem(PASSKEY_STORAGE_KEYS.CREDENTIALS, JSON.stringify(credentials))

		// Wenn aktive DID betroffen, zurücksetzen
		const activeDid = getActiveDid()
		const deleted = getStoredCredentials().find((c) => c.id === credentialId)
		if (deleted && deleted.did === activeDid) {
			clearActiveDid()
		}
	} catch (err) {
		console.error('[PasskeyService] Failed to delete credential:', err)
	}
}

/**
 * Löscht alle gespeicherten Credentials
 */
export function clearAllCredentials(): void {
	if (!browser) return

	try {
		localStorage.removeItem(PASSKEY_STORAGE_KEYS.CREDENTIALS)
		localStorage.removeItem(PASSKEY_STORAGE_KEYS.ACTIVE_DID)
		localStorage.removeItem(PASSKEY_STORAGE_KEYS.PUBLIC_KEYS)
		localStorage.removeItem(PASSKEY_STORAGE_KEYS.LAST_AUTH)
	} catch (err) {
		console.error('[PasskeyService] Failed to clear credentials:', err)
	}
}

/**
 * Setzt die aktive DID
 */
export function setActiveDid(did: string): void {
	if (!browser) return

	try {
		localStorage.setItem(PASSKEY_STORAGE_KEYS.ACTIVE_DID, did)
	} catch (err) {
		console.error('[PasskeyService] Failed to set active DID:', err)
	}
}

/**
 * Holt die aktive DID
 */
export function getActiveDid(): string | null {
	if (!browser) return null

	try {
		return localStorage.getItem(PASSKEY_STORAGE_KEYS.ACTIVE_DID)
	} catch {
		return null
	}
}

/**
 * Löscht die aktive DID
 */
export function clearActiveDid(): void {
	if (!browser) return

	try {
		localStorage.removeItem(PASSKEY_STORAGE_KEYS.ACTIVE_DID)
	} catch (err) {
		console.error('[PasskeyService] Failed to clear active DID:', err)
	}
}

/**
 * Holt Credential für aktive DID
 */
export function getActiveCredential(): StoredPasskeyCredential | null {
	const activeDid = getActiveDid()
	if (!activeDid) return null

	const credentials = getStoredCredentials()
	return credentials.find((c) => c.did === activeDid) || null
}

/**
 * Holt Credential für eine spezifische DID
 */
export function getCredentialForDid(did: string): StoredPasskeyCredential | null {
	const credentials = getStoredCredentials()
	return credentials.find((c) => c.did === did) || null
}

// ============================================================================
// BACKEND REGISTRATION (optional)
// ============================================================================

/**
 * Registriert das Credential beim Backend
 *
 * Optional: Das Backend speichert den Public Key zur Signatur-Verifizierung.
 * Bei Ed25519 kann das bestehende Erynoa Verification-System verwendet werden.
 *
 * @param credential - Gespeichertes Credential
 * @param response - Registration Response
 * @returns Backend Registration Erfolg
 */
export async function registerCredentialWithBackend(
	credential: StoredPasskeyCredential,
	_response: RegistrationResponseJSON
): Promise<boolean> {
	const baseUrl = getApiBaseUrl()

	try {
		const res = await fetch(`${baseUrl}/v1/auth/passkey/register`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({
				credentialId: credential.id,
				publicKey: credential.publicKey,
				algorithm: credential.algorithm,
				did: credential.did,
				namespace: credential.namespace,
			}),
		})

		if (!res.ok) {
			console.warn('[PasskeyService] Backend registration failed:', res.status)
			return false
		}

		return true
	} catch (err) {
		console.warn('[PasskeyService] Backend registration error:', err)
		return false
	}
}

/**
 * Verifiziert eine Authentifizierung beim Backend
 *
 * @param response - Authentication Response
 * @returns Verifikationsergebnis
 */
export async function verifyAuthenticationWithBackend(
	response: AuthenticationResponseJSON
): Promise<boolean> {
	const baseUrl = getApiBaseUrl()

	try {
		const res = await fetch(`${baseUrl}/v1/auth/passkey/verify`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({
				credentialId: response.id,
				signature: response.response.signature,
				authenticatorData: response.response.authenticatorData,
				clientDataJSON: response.response.clientDataJSON,
			}),
		})

		return res.ok
	} catch (err) {
		console.warn('[PasskeyService] Backend verification error:', err)
		return false
	}
}
