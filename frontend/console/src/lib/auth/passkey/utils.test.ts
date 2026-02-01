/**
 * Unit Tests für Passkey Utils
 *
 * Tests für Encoding/Decoding-Funktionen und DID-Generierung.
 */

import { describe, expect, it } from 'vitest'
import {
	arrayBufferToBase64Url,
	base58ToUint8Array,
	base64UrlToArrayBuffer,
	base64UrlToHex,
	base64UrlToUint8Array,
	concatUint8Arrays,
	formatDidShort,
	generateErynoaDid,
	generateUserId,
	hexToBase64Url,
	hexToUint8Array,
	isValidDid,
	parseDid,
	randomBytes,
	uint8ArrayEquals,
	uint8ArrayToBase58,
	uint8ArrayToBase64Url,
	uint8ArrayToHex,
} from './utils'

// ============================================================================
// BASE64URL TESTS
// ============================================================================

describe('Base64URL Encoding/Decoding', () => {
	it('should encode Uint8Array to Base64URL', () => {
		const bytes = new Uint8Array([72, 101, 108, 108, 111]) // "Hello"
		const encoded = uint8ArrayToBase64Url(bytes)
		expect(encoded).toBe('SGVsbG8')
	})

	it('should decode Base64URL to Uint8Array', () => {
		const decoded = base64UrlToUint8Array('SGVsbG8')
		expect(Array.from(decoded)).toEqual([72, 101, 108, 108, 111])
	})

	it('should handle URL-unsafe characters', () => {
		// Bytes that produce + and / in standard Base64
		const bytes = new Uint8Array([251, 255, 254])
		const encoded = uint8ArrayToBase64Url(bytes)
		expect(encoded).not.toContain('+')
		expect(encoded).not.toContain('/')
		expect(encoded).not.toContain('=')
	})

	it('should roundtrip correctly', () => {
		const original = new Uint8Array([0, 1, 2, 255, 254, 253])
		const encoded = uint8ArrayToBase64Url(original)
		const decoded = base64UrlToUint8Array(encoded)
		expect(Array.from(decoded)).toEqual(Array.from(original))
	})

	it('should handle empty array', () => {
		const empty = new Uint8Array([])
		const encoded = uint8ArrayToBase64Url(empty)
		expect(encoded).toBe('')
		const decoded = base64UrlToUint8Array('')
		expect(decoded.length).toBe(0)
	})

	it('should convert ArrayBuffer to Base64URL', () => {
		const buffer = new Uint8Array([1, 2, 3]).buffer
		const encoded = arrayBufferToBase64Url(buffer)
		expect(encoded).toBe('AQID')
	})

	it('should convert Base64URL to ArrayBuffer', () => {
		const buffer = base64UrlToArrayBuffer('AQID')
		expect(new Uint8Array(buffer)).toEqual(new Uint8Array([1, 2, 3]))
	})
})

// ============================================================================
// HEX TESTS
// ============================================================================

describe('Hex Encoding/Decoding', () => {
	it('should encode Uint8Array to hex', () => {
		const bytes = new Uint8Array([0, 15, 255])
		const hex = uint8ArrayToHex(bytes)
		expect(hex).toBe('000fff')
	})

	it('should decode hex to Uint8Array', () => {
		const decoded = hexToUint8Array('000fff')
		expect(Array.from(decoded)).toEqual([0, 15, 255])
	})

	it('should handle 0x prefix', () => {
		const decoded = hexToUint8Array('0xdeadbeef')
		expect(uint8ArrayToHex(decoded)).toBe('deadbeef')
	})

	it('should throw on odd-length hex', () => {
		expect(() => hexToUint8Array('abc')).toThrow('Hex string must have even length')
	})

	it('should convert Base64URL to hex', () => {
		const hex = base64UrlToHex('SGVsbG8')
		expect(hex).toBe('48656c6c6f')
	})

	it('should convert hex to Base64URL', () => {
		const base64 = hexToBase64Url('48656c6c6f')
		expect(base64).toBe('SGVsbG8')
	})
})

// ============================================================================
// BASE58 TESTS
// ============================================================================

describe('Base58 Encoding/Decoding', () => {
	it('should encode Uint8Array to Base58', () => {
		const bytes = new Uint8Array([0, 0, 1])
		const encoded = uint8ArrayToBase58(bytes)
		// Leading zeros become '1's
		expect(encoded.startsWith('11')).toBe(true)
	})

	it('should decode Base58 to Uint8Array', () => {
		// Base58 for "Hello World"
		const decoded = base58ToUint8Array('JxF12TrwUP45BMd')
		expect(decoded.length).toBeGreaterThan(0)
	})

	it('should handle leading zeros', () => {
		const bytes = new Uint8Array([0, 0, 0, 1])
		const encoded = uint8ArrayToBase58(bytes)
		const decoded = base58ToUint8Array(encoded)
		expect(Array.from(decoded)).toEqual([0, 0, 0, 1])
	})

	it('should throw on invalid Base58 character', () => {
		expect(() => base58ToUint8Array('0OIl')).toThrow('Invalid Base58 character')
	})
})

// ============================================================================
// DID TESTS
// ============================================================================

describe('DID Generation and Parsing', () => {
	it('should generate valid Erynoa DID', () => {
		const publicKeyHex = 'a'.repeat(64) // 32 bytes as hex
		const did = generateErynoaDid('self', publicKeyHex)
		expect(did).toMatch(/^did:erynoa:self:[a-z0-9]+$/)
	})

	it('should generate different DIDs for different namespaces', () => {
		const publicKeyHex = 'b'.repeat(64)
		const didSelf = generateErynoaDid('self', publicKeyHex)
		const didGuild = generateErynoaDid('guild', publicKeyHex)
		expect(didSelf).toContain(':self:')
		expect(didGuild).toContain(':guild:')
	})

	it('should parse valid DID', () => {
		const parsed = parseDid('did:erynoa:self:abc123')
		expect(parsed).not.toBeNull()
		expect(parsed?.method).toBe('erynoa')
		expect(parsed?.namespace).toBe('self')
		expect(parsed?.identifier).toBe('abc123')
	})

	it('should parse did:key format', () => {
		const parsed = parseDid('did:key:z6Mk...')
		expect(parsed).not.toBeNull()
		expect(parsed?.method).toBe('key')
	})

	it('should return null for invalid DID', () => {
		expect(parseDid('not-a-did')).toBeNull()
		expect(parseDid('did:')).toBeNull()
		expect(parseDid('')).toBeNull()
	})

	it('should validate DIDs correctly', () => {
		expect(isValidDid('did:erynoa:self:abc123')).toBe(true)
		expect(isValidDid('did:key:z6MkpTHR8VNs...')).toBe(true)
		expect(isValidDid('invalid')).toBe(false)
		expect(isValidDid('')).toBe(false)
	})

	it('should format DID short', () => {
		const did = 'did:erynoa:self:abcdefghijklmnop'
		const short = formatDidShort(did, 6)
		expect(short.length).toBeLessThan(did.length)
		expect(short).toContain('...')
	})

	it('should not truncate short DIDs', () => {
		const did = 'did:erynoa:self:abc'
		const short = formatDidShort(did, 20)
		expect(short).toBe(did)
	})
})

// ============================================================================
// UTILITY TESTS
// ============================================================================

describe('Utility Functions', () => {
	it('should generate random bytes', () => {
		const bytes1 = randomBytes(32)
		const bytes2 = randomBytes(32)
		expect(bytes1.length).toBe(32)
		expect(bytes2.length).toBe(32)
		// Should be different (extremely unlikely to be same)
		expect(uint8ArrayToHex(bytes1)).not.toBe(uint8ArrayToHex(bytes2))
	})

	it('should generate user ID', () => {
		const userId = generateUserId()
		expect(userId.length).toBe(32)
	})

	it('should compare Uint8Arrays correctly', () => {
		const a = new Uint8Array([1, 2, 3])
		const b = new Uint8Array([1, 2, 3])
		const c = new Uint8Array([1, 2, 4])
		const d = new Uint8Array([1, 2])

		expect(uint8ArrayEquals(a, b)).toBe(true)
		expect(uint8ArrayEquals(a, c)).toBe(false)
		expect(uint8ArrayEquals(a, d)).toBe(false)
	})

	it('should concatenate Uint8Arrays', () => {
		const a = new Uint8Array([1, 2])
		const b = new Uint8Array([3, 4])
		const c = new Uint8Array([5])
		const result = concatUint8Arrays(a, b, c)
		expect(Array.from(result)).toEqual([1, 2, 3, 4, 5])
	})

	it('should handle empty concat', () => {
		const result = concatUint8Arrays()
		expect(result.length).toBe(0)
	})
})
