import '@testing-library/jest-dom/vitest'

// Mock SvelteKit's $app modules
vi.mock('$app/environment', () => ({
	browser: true,
	dev: true,
	building: false,
	version: 'test',
}))

vi.mock('$app/navigation', () => ({
	goto: vi.fn(),
	invalidate: vi.fn(),
	invalidateAll: vi.fn(),
	prefetch: vi.fn(),
	prefetchRoutes: vi.fn(),
	beforeNavigate: vi.fn(),
	afterNavigate: vi.fn(),
}))

vi.mock('$app/stores', () => ({
	page: {
		subscribe: vi.fn(),
	},
	navigating: {
		subscribe: vi.fn(),
	},
	updated: {
		subscribe: vi.fn(),
		check: vi.fn(),
	},
}))

// Mock localStorage
const localStorageMock = {
	getItem: vi.fn(),
	setItem: vi.fn(),
	removeItem: vi.fn(),
	clear: vi.fn(),
	length: 0,
	key: vi.fn(),
}
Object.defineProperty(globalThis, 'localStorage', { value: localStorageMock })

// Mock crypto.randomUUID
if (!globalThis.crypto) {
	Object.defineProperty(globalThis, 'crypto', {
		value: {
			randomUUID: () => 'test-uuid-1234-5678-9abc-def012345678',
			getRandomValues: (arr: Uint8Array) => {
				for (let i = 0; i < arr.length; i++) {
					arr[i] = Math.floor(Math.random() * 256)
				}
				return arr
			},
		},
	})
}

// Mock window.PublicKeyCredential for WebAuthn feature detection
Object.defineProperty(globalThis, 'PublicKeyCredential', {
	value: {
		isUserVerifyingPlatformAuthenticatorAvailable: vi.fn().mockResolvedValue(true),
		isConditionalMediationAvailable: vi.fn().mockResolvedValue(false),
	},
	writable: true,
})
