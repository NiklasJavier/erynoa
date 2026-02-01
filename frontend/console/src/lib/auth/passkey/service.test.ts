/**
 * Unit Tests für Passkey Service
 *
 * Tests für WebAuthn-Operationen mit gemockten Browser-APIs.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
	checkPasskeySupport,
	clearActiveDid,
	clearAllCredentials,
	deleteCredential,
	getActiveCredential,
	getActiveDid,
	getStoredCredentials,
	isPasskeyAvailable,
	saveCredential,
	setActiveDid,
} from './service'
import { PASSKEY_STORAGE_KEYS, type StoredPasskeyCredential } from './types'

// ============================================================================
// TEST FIXTURES
// ============================================================================

const mockCredential: StoredPasskeyCredential = {
	credential_id: 'test-credential-id-base64url',
	public_key_base64: 'dGVzdC1wdWJsaWMta2V5LWJhc2U2NA',
	public_key_hex: 'a'.repeat(64),
	algorithm: -8, // Ed25519
	did: 'did:erynoa:self:testuser123',
	namespace: 'self',
	display_name: 'Test User',
	username: 'testuser',
	transports: ['internal'],
	created_at: Date.now(),
	isPrimary: true,
}

const mockCredential2: StoredPasskeyCredential = {
	credential_id: 'test-credential-id-2',
	public_key_base64: 'c2Vjb25kLXB1YmxpYy1rZXk',
	public_key_hex: 'b'.repeat(64),
	algorithm: -7, // ES256
	did: 'did:erynoa:guild:guild123',
	namespace: 'guild',
	display_name: 'Guild Key',
	username: 'guildadmin',
	transports: ['usb', 'nfc'],
	created_at: Date.now() - 86400000,
	isPrimary: false,
}

// ============================================================================
// SETUP / TEARDOWN
// ============================================================================

beforeEach(() => {
	// Clear localStorage mock
	vi.mocked(localStorage.getItem).mockReturnValue(null)
	vi.mocked(localStorage.setItem).mockClear()
	vi.mocked(localStorage.removeItem).mockClear()
})

afterEach(() => {
	vi.clearAllMocks()
})

// ============================================================================
// FEATURE DETECTION TESTS
// ============================================================================

describe('Feature Detection', () => {
	it('should detect WebAuthn support', async () => {
		const support = await checkPasskeySupport()

		expect(support.webauthnAvailable).toBe(true)
		expect(support).toHaveProperty('platformAuthenticatorAvailable')
		expect(support).toHaveProperty('conditionalUIAvailable')
		expect(support).toHaveProperty('ed25519Supported')
	})

	it('should detect platform authenticator availability', async () => {
		vi.mocked(PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable).mockResolvedValue(
			true
		)

		const support = await checkPasskeySupport()
		expect(support.platformAuthenticatorAvailable).toBe(true)
	})

	it('should handle missing PublicKeyCredential', async () => {
		const originalPKC = globalThis.PublicKeyCredential
		// @ts-expect-error - Testing missing API
		delete globalThis.PublicKeyCredential

		const available = await isPasskeyAvailable()
		expect(available).toBe(false)

		// Restore
		globalThis.PublicKeyCredential = originalPKC
	})

	it('should return quick availability check', async () => {
		const available = await isPasskeyAvailable()
		expect(typeof available).toBe('boolean')
	})
})

// ============================================================================
// CREDENTIAL STORAGE TESTS
// ============================================================================

describe('Credential Storage', () => {
	describe('saveCredential', () => {
		it('should save credential to localStorage', () => {
			vi.mocked(localStorage.getItem).mockReturnValue(null)

			saveCredential(mockCredential)

			expect(localStorage.setItem).toHaveBeenCalledWith(
				PASSKEY_STORAGE_KEYS.CREDENTIALS,
				expect.any(String)
			)
		})

		it('should append to existing credentials', () => {
			vi.mocked(localStorage.getItem).mockReturnValue(JSON.stringify([mockCredential]))

			saveCredential(mockCredential2)

			const setItemCall = vi
				.mocked(localStorage.setItem)
				.mock.calls.find((call) => call[0] === PASSKEY_STORAGE_KEYS.CREDENTIALS)
			expect(setItemCall).toBeDefined()

			const savedCredentials = JSON.parse(setItemCall![1])
			expect(savedCredentials).toHaveLength(2)
		})

		it('should update existing credential with same ID', () => {
			const existingCredential = { ...mockCredential, display_name: 'Old Name' }
			vi.mocked(localStorage.getItem).mockReturnValue(JSON.stringify([existingCredential]))

			const updatedCredential = { ...mockCredential, display_name: 'New Name' }
			saveCredential(updatedCredential)

			const setItemCall = vi
				.mocked(localStorage.setItem)
				.mock.calls.find((call) => call[0] === PASSKEY_STORAGE_KEYS.CREDENTIALS)
			const savedCredentials = JSON.parse(setItemCall![1])

			expect(savedCredentials).toHaveLength(1)
			expect(savedCredentials[0].display_name).toBe('New Name')
		})
	})

	describe('getStoredCredentials', () => {
		it('should return empty array when no credentials', () => {
			vi.mocked(localStorage.getItem).mockReturnValue(null)

			const credentials = getStoredCredentials()
			expect(credentials).toEqual([])
		})

		it('should return stored credentials', () => {
			vi.mocked(localStorage.getItem).mockReturnValue(
				JSON.stringify([mockCredential, mockCredential2])
			)

			const credentials = getStoredCredentials()
			expect(credentials).toHaveLength(2)
			expect(credentials[0].did).toBe(mockCredential.did)
		})

		it('should handle corrupted JSON gracefully', () => {
			vi.mocked(localStorage.getItem).mockReturnValue('not-valid-json{')

			const credentials = getStoredCredentials()
			expect(credentials).toEqual([])
		})
	})

	describe('deleteCredential', () => {
		it('should remove credential by ID', () => {
			vi.mocked(localStorage.getItem).mockReturnValue(
				JSON.stringify([mockCredential, mockCredential2])
			)

			deleteCredential(mockCredential.credential_id)

			const setItemCall = vi
				.mocked(localStorage.setItem)
				.mock.calls.find((call) => call[0] === PASSKEY_STORAGE_KEYS.CREDENTIALS)
			const remainingCredentials = JSON.parse(setItemCall![1])

			expect(remainingCredentials).toHaveLength(1)
			expect(remainingCredentials[0].credential_id).toBe(mockCredential2.credential_id)
		})

		it('should handle non-existent credential ID', () => {
			vi.mocked(localStorage.getItem).mockReturnValue(JSON.stringify([mockCredential]))

			// Should not throw
			expect(() => deleteCredential('non-existent-id')).not.toThrow()
		})
	})

	describe('clearAllCredentials', () => {
		it('should remove all credentials', () => {
			clearAllCredentials()

			expect(localStorage.removeItem).toHaveBeenCalledWith(PASSKEY_STORAGE_KEYS.CREDENTIALS)
			expect(localStorage.removeItem).toHaveBeenCalledWith(PASSKEY_STORAGE_KEYS.ACTIVE_DID)
		})
	})
})

// ============================================================================
// ACTIVE DID TESTS
// ============================================================================

describe('Active DID Management', () => {
	describe('setActiveDid', () => {
		it('should store active DID', () => {
			setActiveDid('did:erynoa:self:test123')

			expect(localStorage.setItem).toHaveBeenCalledWith(
				PASSKEY_STORAGE_KEYS.ACTIVE_DID,
				'did:erynoa:self:test123'
			)
		})
	})

	describe('getActiveDid', () => {
		it('should return null when no active DID', () => {
			vi.mocked(localStorage.getItem).mockReturnValue(null)

			const did = getActiveDid()
			expect(did).toBeNull()
		})

		it('should return stored active DID', () => {
			vi.mocked(localStorage.getItem).mockReturnValue('did:erynoa:self:test123')

			const did = getActiveDid()
			expect(did).toBe('did:erynoa:self:test123')
		})
	})

	describe('clearActiveDid', () => {
		it('should remove active DID', () => {
			clearActiveDid()

			expect(localStorage.removeItem).toHaveBeenCalledWith(PASSKEY_STORAGE_KEYS.ACTIVE_DID)
		})
	})

	describe('getActiveCredential', () => {
		it('should return null when no active DID', () => {
			vi.mocked(localStorage.getItem).mockReturnValue(null)

			const credential = getActiveCredential()
			expect(credential).toBeNull()
		})

		it('should return credential matching active DID', () => {
			vi.mocked(localStorage.getItem).mockImplementation((key) => {
				if (key === PASSKEY_STORAGE_KEYS.ACTIVE_DID) {
					return mockCredential.did
				}
				if (key === PASSKEY_STORAGE_KEYS.CREDENTIALS) {
					return JSON.stringify([mockCredential, mockCredential2])
				}
				return null
			})

			const credential = getActiveCredential()
			expect(credential).not.toBeNull()
			expect(credential?.did).toBe(mockCredential.did)
		})
	})
})

// ============================================================================
// WEBAUTHN OPERATIONS TESTS (Mocked)
// ============================================================================

describe('WebAuthn Operations', () => {
	// Note: These tests would require more complex mocking of
	// @simplewebauthn/browser. For now, we test the support functions.

	it('should have registerPasskey function exported', async () => {
		const { registerPasskey } = await import('./service')
		expect(typeof registerPasskey).toBe('function')
	})

	it('should have authenticateWithPasskey function exported', async () => {
		const { authenticateWithPasskey } = await import('./service')
		expect(typeof authenticateWithPasskey).toBe('function')
	})

	it('should have signWithPasskey function exported', async () => {
		const { signWithPasskey } = await import('./service')
		expect(typeof signWithPasskey).toBe('function')
	})
})
