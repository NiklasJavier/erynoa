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
})
