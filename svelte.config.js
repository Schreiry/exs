import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	compilerOptions: {
		// Svelte 5 runes mode everywhere in our own source.
		runes: true
	},
	kit: {
		// Static export — Tauri serves the built `build/` dir as the webview.
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			fallback: 'index.html',
			strict: false
		}),
		alias: {
			$lib: 'src/lib'
		}
	}
};

export default config;
