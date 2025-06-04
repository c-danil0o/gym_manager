import { paraglideVitePlugin } from '@inlang/paraglide-js'
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import tailwindcss from '@tailwindcss/vite';
import path from 'path';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	plugins: [paraglideVitePlugin({ project: './project.inlang', outdir: './src/lib/paraglide' }),tailwindcss(), sveltekit()],
	clearScreen: false,
	server: {
		port: 1420,
		strictPort: true,
		host: host || false,
		hmr: host
			? {
					protocol: 'ws',
					host,
					port: 1421
				}
			: undefined,
		watch: {
			ignored: ['**/src-tauri/**']
		}
	},
	resolve: {
		alias: {
			$lib: path.resolve('./src/lib')
		}
	},
	build: {
		sourcemap: false
	},
	css: {
		devSourcemap: false
	}
});
