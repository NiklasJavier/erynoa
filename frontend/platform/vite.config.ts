import { sveltekit } from '@sveltejs/kit/vite'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	base: '/platform/',
	server: {
		port: 5174,
		strictPort: true,
		host: true,
		// Erlaube alle Hosts (für Proxy-Zugriff)
		allowedHosts: ['all'],
	},
	preview: {
		port: 5174,
	},
})
