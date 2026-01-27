import { sveltekit } from '@sveltejs/kit/vite'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	base: '/console/',
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
	optimizeDeps: {
		// lucide-svelte scheint Probleme mit dem Dep-Optimizer zu haben
		// und erzeugt fehlende .vite/deps/*-Chunks -> 404 + MIME-Type-Fehler.
		// Durch das Excluden lädt Vite die Module direkt aus node_modules.
		exclude: ['lucide-svelte'],
	},
})
