import adapter from '@sveltejs/adapter-static'

/** @type {import('@sveltejs/kit').Config} */
const config = {
	kit: {
		// Base path for reverse proxy routing
		paths: {
			base: '/docs',
			relative: false, // Absolute paths für @fs node_modules
		},
		// SPA mode with static adapter
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			fallback: 'index.html',
			precompress: false,
			strict: true,
		}),
		alias: {
			$lib: './src/lib',
			$gen: './src/gen',
		},
	},
}

export default config
