/**
 * Passkey Store
 *
 * Svelte Store für reaktives Passkey-State Management.
 * Verwaltet Credentials, aktive DIDs und Feature Support.
 *
 * @module auth/passkey/store
 */

import { browser } from '$app/environment'
import { derived, writable, type Readable } from 'svelte/store'
import {
	authenticateWithPasskey,
	checkPasskeySupport,
	clearActiveDid,
	clearAllCredentials,
	deleteCredential,
	getActiveDid,
	getStoredCredentials,
	registerPasskey,
	setActiveDid,
	signWithPasskey,
} from './service'
import {
	PasskeyErrorCode,
	type PasskeyAuthenticationOptions,
	type PasskeyAuthenticationResult,
	type PasskeyRegistrationOptions,
	type PasskeyRegistrationResult,
	type PasskeySignOptions,
	type PasskeySignatureResult,
	type PasskeyState,
	type PasskeySupport,
	type StoredPasskeyCredential,
} from './types'

// ============================================================================
// INITIAL STATE
// ============================================================================

const initialState: PasskeyState = {
	support: null,
	credentials: [],
	activeDid: null,
	activeCredential: null,
	initialized: false,
	loading: false,
	error: null,
	errorCode: null,
}

// ============================================================================
// STORE CREATION
// ============================================================================

function createPasskeyStore() {
	const { subscribe, set, update } = writable<PasskeyState>(initialState)

	// Hilfsfunktion zum Neuladen der Credentials
	function reloadCredentials() {
		const credentials = getStoredCredentials()
		const activeDid = getActiveDid()
		const activeCredential = credentials.find((c) => c.did === activeDid) || null

		update((s) => ({
			...s,
			credentials,
			activeDid,
			activeCredential,
		}))
	}

	return {
		subscribe,

		/**
		 * Initialisiert den Passkey Store
		 *
		 * Lädt gespeicherte Credentials und prüft Feature Support.
		 */
		async init(): Promise<void> {
			if (!browser) return

			// Vermeide doppelte Initialisierung
			let state: PasskeyState | undefined
			const unsubscribe = passkeyStore.subscribe((s) => {
				state = s
			})
			unsubscribe()

			if (state?.initialized) {
				console.log('[PasskeyStore] Already initialized, skipping')
				return
			}

			update((s) => ({ ...s, loading: true, error: null, errorCode: null }))

			try {
				console.log('[PasskeyStore] Initializing...')

				// Feature Support prüfen
				const support = await checkPasskeySupport()
				console.log('[PasskeyStore] Support check:', support)

				// Gespeicherte Credentials laden
				const credentials = getStoredCredentials()
				const activeDid = getActiveDid()
				const activeCredential = credentials.find((c) => c.did === activeDid) || null

				console.log('[PasskeyStore] Loaded credentials:', credentials.length)
				console.log('[PasskeyStore] Active DID:', activeDid)

				update((s) => ({
					...s,
					support,
					credentials,
					activeDid,
					activeCredential,
					initialized: true,
					loading: false,
				}))

				console.log('[PasskeyStore] Initialized successfully')
			} catch (err) {
				const message = err instanceof Error ? err.message : 'Passkey initialization failed'
				console.error('[PasskeyStore] Init error:', err)

				update((s) => ({
					...s,
					initialized: true,
					loading: false,
					error: message,
					errorCode: PasskeyErrorCode.UNKNOWN,
				}))
			}
		},

		/**
		 * Registriert einen neuen Passkey
		 */
		async register(options: PasskeyRegistrationOptions = {}): Promise<PasskeyRegistrationResult> {
			update((s) => ({ ...s, loading: true, error: null, errorCode: null }))

			try {
				const result = await registerPasskey(options)

				if (result.success) {
					// Store aktualisieren
					reloadCredentials()

					update((s) => ({
						...s,
						loading: false,
					}))
				} else {
					update((s) => ({
						...s,
						loading: false,
						error: result.error || 'Registration failed',
						errorCode: result.errorCode || PasskeyErrorCode.UNKNOWN,
					}))
				}

				return result
			} catch (err) {
				const message = err instanceof Error ? err.message : 'Registration failed'

				update((s) => ({
					...s,
					loading: false,
					error: message,
					errorCode: PasskeyErrorCode.UNKNOWN,
				}))

				return {
					success: false,
					error: message,
					errorCode: PasskeyErrorCode.UNKNOWN,
				}
			}
		},

		/**
		 * Authentifiziert mit Passkey
		 */
		async authenticate(
			options: PasskeyAuthenticationOptions = {}
		): Promise<PasskeyAuthenticationResult> {
			update((s) => ({ ...s, loading: true, error: null, errorCode: null }))

			try {
				const result = await authenticateWithPasskey(options)

				if (result.success && result.did) {
					// Aktive DID setzen
					setActiveDid(result.did)
					reloadCredentials()
				}

				update((s) => ({
					...s,
					loading: false,
					error: result.success ? null : result.error || 'Authentication failed',
					errorCode: result.success ? null : result.errorCode || null,
				}))

				return result
			} catch (err) {
				const message = err instanceof Error ? err.message : 'Authentication failed'

				update((s) => ({
					...s,
					loading: false,
					error: message,
					errorCode: PasskeyErrorCode.UNKNOWN,
				}))

				return {
					success: false,
					error: message,
					errorCode: PasskeyErrorCode.UNKNOWN,
				}
			}
		},

		/**
		 * Signiert eine Nachricht mit Passkey
		 */
		async sign(
			message: Uint8Array | string,
			options: PasskeySignOptions = {}
		): Promise<PasskeySignatureResult> {
			update((s) => ({ ...s, loading: true, error: null, errorCode: null }))

			try {
				const result = await signWithPasskey(message, options)

				update((s) => ({
					...s,
					loading: false,
					error: result.success ? null : result.error || 'Signing failed',
					errorCode: result.success ? null : result.errorCode || null,
				}))

				return result
			} catch (err) {
				const message = err instanceof Error ? err.message : 'Signing failed'

				update((s) => ({
					...s,
					loading: false,
					error: message,
					errorCode: PasskeyErrorCode.UNKNOWN,
				}))

				return {
					success: false,
					error: message,
					errorCode: PasskeyErrorCode.UNKNOWN,
				}
			}
		},

		/**
		 * Setzt die aktive DID
		 */
		setActiveDid(did: string): void {
			setActiveDid(did)
			reloadCredentials()
		},

		/**
		 * Löscht die aktive DID (logout)
		 */
		clearActiveDid(): void {
			clearActiveDid()
			update((s) => ({
				...s,
				activeDid: null,
				activeCredential: null,
			}))
		},

		/**
		 * Löscht ein Credential
		 */
		deleteCredential(credentialId: string): void {
			deleteCredential(credentialId)
			reloadCredentials()
		},

		/**
		 * Löscht alle Credentials (reset)
		 */
		clearAll(): void {
			clearAllCredentials()
			set({
				...initialState,
				initialized: true,
			})
		},

		/**
		 * Setzt Fehler zurück
		 */
		clearError(): void {
			update((s) => ({ ...s, error: null, errorCode: null }))
		},

		/**
		 * Aktualisiert Feature Support
		 */
		async refreshSupport(): Promise<PasskeySupport> {
			const support = await checkPasskeySupport()
			update((s) => ({ ...s, support }))
			return support
		},

		/**
		 * Lädt Credentials neu aus LocalStorage
		 */
		refresh(): void {
			reloadCredentials()
		},
	}
}

// ============================================================================
// EXPORTS
// ============================================================================

/**
 * Singleton Passkey Store
 */
export const passkeyStore = createPasskeyStore()

// ============================================================================
// DERIVED STORES
// ============================================================================

/**
 * Feature Support Status
 */
export const passkeySupport: Readable<PasskeySupport | null> = derived(
	passkeyStore,
	($s) => $s.support
)

/**
 * Ob Passkeys verfügbar sind
 */
export const isPasskeyAvailable: Readable<boolean> = derived(
	passkeyStore,
	($s) => $s.support?.webauthnAvailable ?? false
)

/**
 * Ob Platform Authenticator verfügbar ist (TouchID, FaceID, Windows Hello)
 */
export const hasPlatformAuthenticator: Readable<boolean> = derived(
	passkeyStore,
	($s) => $s.support?.platformAuthenticatorAvailable ?? false
)

/**
 * Alle gespeicherten Credentials
 */
export const passkeyCredentials: Readable<StoredPasskeyCredential[]> = derived(
	passkeyStore,
	($s) => $s.credentials
)

/**
 * Aktuell aktive DID
 */
export const activePasskeyDid: Readable<string | null> = derived(passkeyStore, ($s) => $s.activeDid)

/**
 * Aktuell aktives Credential
 */
export const activePasskeyCredential: Readable<StoredPasskeyCredential | null> = derived(
	passkeyStore,
	($s) => $s.activeCredential
)

/**
 * Ob Passkey-Authentifizierung aktiv ist
 */
export const isPasskeyAuthenticated: Readable<boolean> = derived(
	passkeyStore,
	($s) => $s.activeDid !== null && $s.activeCredential !== null
)

/**
 * Ob mindestens ein Passkey registriert ist
 */
export const hasPasskeyRegistered: Readable<boolean> = derived(
	passkeyStore,
	($s) => $s.credentials.length > 0
)

/**
 * Anzahl registrierter Passkeys
 */
export const passkeyCount: Readable<number> = derived(passkeyStore, ($s) => $s.credentials.length)

/**
 * Ob Store initialisiert ist
 */
export const isPasskeyInitialized: Readable<boolean> = derived(passkeyStore, ($s) => $s.initialized)

/**
 * Ob gerade geladen wird
 */
export const isPasskeyLoading: Readable<boolean> = derived(passkeyStore, ($s) => $s.loading)

/**
 * Aktueller Fehler
 */
export const passkeyError: Readable<string | null> = derived(passkeyStore, ($s) => $s.error)

/**
 * Aktueller Fehlercode
 */
export const passkeyErrorCode: Readable<PasskeyErrorCode | null> = derived(
	passkeyStore,
	($s) => $s.errorCode
)

/**
 * Primäres Credential (falls markiert)
 */
export const primaryCredential: Readable<StoredPasskeyCredential | null> = derived(
	passkeyStore,
	($s) => $s.credentials.find((c) => c.isPrimary) || null
)

/**
 * Ob Ed25519 Passkeys unterstützt werden
 */
export const supportsEd25519: Readable<boolean> = derived(
	passkeyStore,
	($s) => $s.support?.ed25519Supported ?? false
)
