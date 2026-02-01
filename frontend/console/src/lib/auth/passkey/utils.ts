/**
 * Passkey Utility Functions
 *
 * Encoding/Decoding und DID-Generierung für WebAuthn-basierte Identitäten.
 * Kompatibel mit Erynoa's Ed25519-basiertem DID-System.
 *
 * @module auth/passkey/utils
 */

import type { ErynoaNamespace, PasskeyDID } from './types'

// ============================================================================
// BASE64URL ENCODING/DECODING
// ============================================================================

/**
 * Konvertiert Uint8Array zu Base64URL String
 *
 * @param bytes - Die zu encodierenden Bytes
 * @returns Base64URL encoded String (ohne Padding)
 */
export function uint8ArrayToBase64Url(bytes: Uint8Array): string {
	// Zu Base64 konvertieren
	let binary = ''
	for (let i = 0; i < bytes.length; i++) {
		binary += String.fromCharCode(bytes[i])
	}
	const base64 = btoa(binary)

	// Base64 zu Base64URL (URL-safe)
	return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')
}

/**
 * Konvertiert Base64URL String zu Uint8Array
 *
 * @param base64url - Base64URL encoded String
 * @returns Decoded Bytes
 */
export function base64UrlToUint8Array(base64url: string): Uint8Array {
	// Base64URL zu Standard Base64
	let base64 = base64url.replace(/-/g, '+').replace(/_/g, '/')

	// Padding hinzufügen falls nötig
	const paddingLength = (4 - (base64.length % 4)) % 4
	base64 += '='.repeat(paddingLength)

	// Dekodieren
	const binary = atob(base64)
	const bytes = new Uint8Array(binary.length)
	for (let i = 0; i < binary.length; i++) {
		bytes[i] = binary.charCodeAt(i)
	}

	return bytes
}

/**
 * Konvertiert ArrayBuffer zu Base64URL String
 *
 * @param buffer - ArrayBuffer
 * @returns Base64URL encoded String
 */
export function arrayBufferToBase64Url(buffer: ArrayBuffer): string {
	return uint8ArrayToBase64Url(new Uint8Array(buffer))
}

/**
 * Konvertiert Base64URL String zu ArrayBuffer
 *
 * @param base64url - Base64URL encoded String
 * @returns ArrayBuffer
 */
export function base64UrlToArrayBuffer(base64url: string): ArrayBuffer {
	return base64UrlToUint8Array(base64url).buffer as ArrayBuffer
}

// ============================================================================
// HEX ENCODING/DECODING
// ============================================================================

/**
 * Konvertiert Uint8Array zu Hex String
 *
 * @param bytes - Die zu encodierenden Bytes
 * @returns Hex String (lowercase)
 */
export function uint8ArrayToHex(bytes: Uint8Array): string {
	return Array.from(bytes)
		.map((b) => b.toString(16).padStart(2, '0'))
		.join('')
}

/**
 * Konvertiert Hex String zu Uint8Array
 *
 * @param hex - Hex String
 * @returns Decoded Bytes
 */
export function hexToUint8Array(hex: string): Uint8Array {
	const cleanHex = hex.replace(/^0x/, '')
	if (cleanHex.length % 2 !== 0) {
		throw new Error('Hex string must have even length')
	}

	const bytes = new Uint8Array(cleanHex.length / 2)
	for (let i = 0; i < bytes.length; i++) {
		bytes[i] = Number.parseInt(cleanHex.substr(i * 2, 2), 16)
	}

	return bytes
}

/**
 * Konvertiert Base64URL zu Hex
 *
 * @param base64url - Base64URL String
 * @returns Hex String
 */
export function base64UrlToHex(base64url: string): string {
	return uint8ArrayToHex(base64UrlToUint8Array(base64url))
}

/**
 * Konvertiert Hex zu Base64URL
 *
 * @param hex - Hex String
 * @returns Base64URL String
 */
export function hexToBase64Url(hex: string): string {
	return uint8ArrayToBase64Url(hexToUint8Array(hex))
}

// ============================================================================
// MULTIBASE ENCODING (für did:key Kompatibilität)
// ============================================================================

/**
 * Base58btc Alphabet (Bitcoin-kompatibel)
 */
const BASE58_ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz'

/**
 * Konvertiert Uint8Array zu Base58btc String
 *
 * @param bytes - Die zu encodierenden Bytes
 * @returns Base58btc encoded String
 */
export function uint8ArrayToBase58(bytes: Uint8Array): string {
	// Count leading zeros
	let zeros = 0
	for (let i = 0; i < bytes.length && bytes[i] === 0; i++) {
		zeros++
	}

	// Konvertiere zu Base58
	const result: number[] = []
	let num = BigInt(`0x${uint8ArrayToHex(bytes)}`)

	while (num > 0n) {
		const remainder = Number(num % 58n)
		result.unshift(remainder)
		num = num / 58n
	}

	// Führende Einsen für führende Nullbytes
	const leadingOnes = '1'.repeat(zeros)
	const encoded = result.map((i) => BASE58_ALPHABET[i]).join('')

	return leadingOnes + encoded
}

/**
 * Konvertiert Base58btc String zu Uint8Array
 *
 * @param str - Base58btc encoded String
 * @returns Decoded Bytes
 */
export function base58ToUint8Array(str: string): Uint8Array {
	// Count leading ones (represent leading zeros)
	let zeros = 0
	for (let i = 0; i < str.length && str[i] === '1'; i++) {
		zeros++
	}

	// Konvertiere von Base58 zu BigInt
	let num = 0n
	for (let i = zeros; i < str.length; i++) {
		const index = BASE58_ALPHABET.indexOf(str[i])
		if (index === -1) {
			throw new Error(`Invalid Base58 character: ${str[i]}`)
		}
		num = num * 58n + BigInt(index)
	}

	// BigInt zu Bytes
	let hex = num.toString(16)
	if (hex.length % 2 !== 0) {
		hex = `0${hex}`
	}

	const decoded = hexToUint8Array(hex)

	// Führende Nullbytes hinzufügen
	const result = new Uint8Array(zeros + decoded.length)
	result.set(decoded, zeros)

	return result
}

// ============================================================================
// DID GENERIERUNG
// ============================================================================

/**
 * Multicodec Prefixes
 * @see https://github.com/multiformats/multicodec/blob/master/table.csv
 */
const MULTICODEC = {
	/** Ed25519 Public Key */
	ED25519_PUB: new Uint8Array([0xed, 0x01]),
	/** P-256 (secp256r1) Public Key */
	P256_PUB: new Uint8Array([0x80, 0x24]),
}

/**
 * Generiert eine did:key DID aus einem Ed25519 Public Key
 *
 * Format: did:key:z<multibase-base58btc-encoded-multicodec-prefixed-key>
 *
 * @param publicKey - Ed25519 Public Key (32 bytes)
 * @returns did:key DID String
 */
export function generateDidKeyFromEd25519(publicKey: Uint8Array): string {
	if (publicKey.length !== 32) {
		throw new Error(`Invalid Ed25519 public key length: ${publicKey.length}, expected 32`)
	}

	// Multicodec-Prefix + Public Key
	const prefixedKey = new Uint8Array(MULTICODEC.ED25519_PUB.length + publicKey.length)
	prefixedKey.set(MULTICODEC.ED25519_PUB, 0)
	prefixedKey.set(publicKey, MULTICODEC.ED25519_PUB.length)

	// Base58btc encode mit 'z' Multibase-Prefix
	const encoded = uint8ArrayToBase58(prefixedKey)

	return `did:key:z${encoded}`
}

/**
 * Generiert eine did:erynoa DID aus einem Public Key
 *
 * Format: did:erynoa:<namespace>:<unique-id>
 * Unique ID = erste 16 Zeichen des Hex-encoded Public Keys
 *
 * @param publicKey - Public Key Bytes
 * @param namespace - Erynoa Namespace (default: 'self')
 * @returns did:erynoa DID String
 */
export function generateErynoaDid(
	publicKey: Uint8Array,
	namespace: ErynoaNamespace = 'self'
): string {
	const publicKeyHex = uint8ArrayToHex(publicKey)
	const uniqueId = publicKeyHex.substring(0, 16)
	return `did:erynoa:${namespace}:${uniqueId}`
}

/**
 * Generiert ein PasskeyDID Objekt aus Public Key Bytes
 *
 * @param publicKey - Public Key Bytes
 * @param namespace - Erynoa Namespace
 * @param algorithm - COSE Algorithm ID
 * @returns PasskeyDID Objekt
 */
export function createPasskeyDid(
	publicKey: Uint8Array,
	namespace: ErynoaNamespace = 'self',
	algorithm = -8 // Ed25519
): PasskeyDID {
	const publicKeyHex = uint8ArrayToHex(publicKey)
	const uniqueId = publicKeyHex.substring(0, 16)

	return {
		did: `did:erynoa:${namespace}:${uniqueId}`,
		namespace,
		uniqueId,
		publicKeyHex,
		publicKeyBytes: publicKey,
		algorithm,
		createdAt: new Date(),
	}
}

/**
 * Parst eine DID und extrahiert die Komponenten
 *
 * @param did - DID String
 * @returns Parsed DID Komponenten oder null bei ungültigem Format
 */
export function parseDid(
	did: string
): { method: string; namespace?: string; uniqueId: string } | null {
	const parts = did.split(':')

	if (parts.length < 3 || parts[0] !== 'did') {
		return null
	}

	const method = parts[1]

	if (method === 'erynoa' && parts.length === 4) {
		return {
			method,
			namespace: parts[2],
			uniqueId: parts[3],
		}
	}

	if (method === 'key' && parts.length === 3) {
		return {
			method,
			uniqueId: parts[2],
		}
	}

	return null
}

// ============================================================================
// COSE KEY PARSING
// ============================================================================

/**
 * Extrahiert den Raw Public Key aus COSE Key Format (CBOR encoded)
 *
 * Für Ed25519 (OKP): Der -2 Schlüssel enthält den 32-byte Public Key
 * Für ES256 (EC2): Die -2 und -3 Schlüssel enthalten x und y Koordinaten
 *
 * @param coseKey - COSE Key in Raw Bytes
 * @returns Public Key Bytes
 */
export function extractPublicKeyFromCose(coseKey: Uint8Array): Uint8Array {
	// Einfache CBOR-Map Parsing für die häufigsten Fälle
	// Vollständiges CBOR-Parsing würde eine Library erfordern

	// Für Ed25519: Der Key ist oft am Ende der COSE-Struktur
	// COSE_Key für Ed25519: {1: 1 (OKP), 3: -8 (EdDSA), -1: 6 (Ed25519), -2: <public_key>}

	// Suche nach dem 32-byte Public Key Block für Ed25519
	if (coseKey.length >= 32) {
		// Heuristik: Der letzte 32-byte Block ist oft der Public Key
		// Dies funktioniert für die meisten Authenticatoren
		const possibleKey = coseKey.slice(-32)

		// Validierung: Ed25519 Public Keys haben bestimmte Eigenschaften
		// (Diese Heuristik ist nicht perfekt, aber funktioniert in der Praxis)
		return possibleKey
	}

	throw new Error('Could not extract public key from COSE format')
}

/**
 * Extrahiert Public Key aus WebAuthn attestation object's authData
 *
 * @param authData - Authenticator Data bytes
 * @returns Public Key Bytes
 */
export function extractPublicKeyFromAuthData(authData: Uint8Array): Uint8Array {
	// AuthData Format:
	// - rpIdHash (32 bytes)
	// - flags (1 byte)
	// - signCount (4 bytes)
	// - attestedCredentialData (variable, wenn AT flag gesetzt):
	//   - aaguid (16 bytes)
	//   - credentialIdLength (2 bytes, big-endian)
	//   - credentialId (credentialIdLength bytes)
	//   - credentialPublicKey (COSE_Key, CBOR encoded, rest of data)

	const rpIdHashLength = 32
	const flagsLength = 1
	const signCountLength = 4
	const aaguidLength = 16
	const credIdLengthSize = 2

	const headerLength = rpIdHashLength + flagsLength + signCountLength + aaguidLength

	if (authData.length < headerLength + credIdLengthSize) {
		throw new Error('AuthData too short to contain credential data')
	}

	// Credential ID Länge (Big-Endian)
	const credIdLength = (authData[headerLength] << 8) | authData[headerLength + 1]

	const publicKeyStart = headerLength + credIdLengthSize + credIdLength
	const coseKey = authData.slice(publicKeyStart)

	return extractPublicKeyFromCose(coseKey)
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/**
 * Generiert zufällige Bytes
 *
 * @param length - Anzahl der Bytes
 * @returns Zufällige Bytes
 */
export function randomBytes(length: number): Uint8Array {
	return crypto.getRandomValues(new Uint8Array(length))
}

/**
 * Generiert eine zufällige User ID für WebAuthn
 *
 * @returns 16 zufällige Bytes
 */
export function generateUserId(): Uint8Array {
	return randomBytes(16)
}

/**
 * Vergleicht zwei Uint8Arrays auf Gleichheit
 *
 * @param a - Erstes Array
 * @param b - Zweites Array
 * @returns true wenn gleich
 */
export function uint8ArrayEquals(a: Uint8Array, b: Uint8Array): boolean {
	if (a.length !== b.length) return false
	for (let i = 0; i < a.length; i++) {
		if (a[i] !== b[i]) return false
	}
	return true
}

/**
 * Konkateniert mehrere Uint8Arrays
 *
 * @param arrays - Arrays zum Konkatenieren
 * @returns Konkateniertes Array
 */
export function concatUint8Arrays(...arrays: Uint8Array[]): Uint8Array {
	const totalLength = arrays.reduce((sum, arr) => sum + arr.length, 0)
	const result = new Uint8Array(totalLength)
	let offset = 0
	for (const arr of arrays) {
		result.set(arr, offset)
		offset += arr.length
	}
	return result
}

/**
 * Formatiert eine DID für die Anzeige (gekürzt)
 *
 * @param did - DID String
 * @param maxLength - Maximale Länge (default: 24)
 * @returns Gekürzte DID
 */
export function formatDidShort(did: string, maxLength = 24): string {
	if (did.length <= maxLength) return did

	const parts = did.split(':')
	if (parts.length < 3) return `${did.slice(0, maxLength - 3)}...`

	const method = parts[1]
	const lastPart = parts[parts.length - 1]

	if (method === 'erynoa' && parts.length === 4) {
		const namespace = parts[2]
		const shortId = `${lastPart.slice(0, 4)}...${lastPart.slice(-4)}`
		return `did:${method}:${namespace}:${shortId}`
	}

	if (method === 'key') {
		const shortKey = `${lastPart.slice(0, 8)}...${lastPart.slice(-4)}`
		return `did:key:${shortKey}`
	}

	return `${did.slice(0, maxLength - 3)}...`
}

/**
 * Validiert eine DID
 *
 * @param did - DID String
 * @returns true wenn valide
 */
export function isValidDid(did: string): boolean {
	const parsed = parseDid(did)
	if (!parsed) return false

	if (parsed.method === 'erynoa') {
		const validNamespaces = [
			'self',
			'guild',
			'spirit',
			'thing',
			'vessel',
			'source',
			'craft',
			'vault',
			'pact',
			'circle',
		]
		return Boolean(parsed.namespace && validNamespaces.includes(parsed.namespace))
	}

	if (parsed.method === 'key') {
		// did:key muss mit 'z' beginnen (Multibase Base58btc)
		return parsed.uniqueId.startsWith('z')
	}

	return false
}
