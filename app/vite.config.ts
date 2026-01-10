import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
    plugins: [svelte()],
    build: {
        target: 'esnext',
    },
    optimizeDeps: {
        exclude: ['ic10_emulator'],
    },
    server: {
        fs: {
            allow: ['..'],
        },
    },
});
