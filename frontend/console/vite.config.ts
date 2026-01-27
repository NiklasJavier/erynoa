import { sveltekit } from '@sveltejs/kit/vite'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	// base wird von SvelteKit aus svelte.config.js überschrieben
	// base: '/console/',
	server: {
		port: 5173,
		strictPort: true,
		host: true,
		// Erlaube alle Hosts (für Proxy-Zugriff)
		allowedHosts: ['all'],
	},
	preview: {
		port: 5173,
	},
	// Performance-Optimierungen
	optimizeDeps: {
		// Pre-bundle häufig verwendete Dependencies
		include: [
			'@sveltejs/kit',
			'@sveltejs/kit/src/runtime/client',
			'lucide-svelte',
			'@connectrpc/connect',
			'@connectrpc/connect-web',
			'oidc-client-ts',
		],
		// Exclude große Dependencies, die bereits optimiert sind
		exclude: [],
	},
	build: {
		// Besseres Code-Splitting
		rollupOptions: {
			output: {
				manualChunks: {
					// Separate Chunks für große Dependencies
					vendor: ['@sveltejs/kit'],
					connect: ['@connectrpc/connect', '@connectrpc/connect-web'],
					icons: ['lucide-svelte'],
				},
			},
		},
		// Source Maps nur im Dev-Modus (schneller)
		sourcemap: false,
		// Minify für bessere Performance
		minify: 'esbuild',
	},
})
