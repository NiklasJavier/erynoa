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
		include: ['lucide-svelte', '@connectrpc/connect', '@connectrpc/connect-web', 'oidc-client-ts'],
	},
	build: {
		// Source Maps nur im Dev-Modus (schneller)
		sourcemap: false,
		// Minify für bessere Performance
		minify: 'esbuild',
	},
})
