import { sveltekit } from '@sveltejs/kit/vite'
import tailwindcss from '@tailwindcss/vite'
import { defineConfig } from 'vite'

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	base: '/docs/',
	server: {
		port: 5175,
		strictPort: true,
		host: true,
	},
	preview: {
		port: 5175,
	},
})
