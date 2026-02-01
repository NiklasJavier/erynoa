import { svelte } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vitest/config'

export default defineConfig({
	plugins: [svelte({ hot: false })],
	test: {
		include: ['src/**/*.{test,spec}.{js,ts}'],
		environment: 'jsdom',
		globals: true,
		setupFiles: ['./vitest.setup.ts'],
		coverage: {
			provider: 'v8',
			reporter: ['text', 'json', 'html'],
			include: ['src/lib/**/*.ts'],
			exclude: ['src/lib/**/*.d.ts', 'src/lib/**/index.ts', 'src/gen/**'],
		},
		alias: {
			$lib: '/src/lib',
			$app: '/src/app-mocks',
		},
	},
	resolve: {
		alias: {
			$lib: '/src/lib',
			$app: '/src/app-mocks',
		},
	},
})
