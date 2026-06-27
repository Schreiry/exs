import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Tauri expects a fixed dev port and leaves the terminal output intact so the
// Rust side can detect when the dev server is ready.
export default defineConfig({
	plugins: [sveltekit()],
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true
	},
	// Expose TAURI_* env vars (e.g. platform) and our own VITE_* to the client.
	envPrefix: ['VITE_', 'TAURI_'],
	build: {
		target: 'esnext'
	}
});
