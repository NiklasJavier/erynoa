import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	base: '/console/',
	server: {
		port: 5173,
		strictPort: true,
		host: true,
	},
	preview: {
		port: 5173,
	},
});
