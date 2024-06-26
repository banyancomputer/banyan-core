import { defineConfig } from 'vitest/config';
import react from "@vitejs/plugin-react";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import { resolve } from 'path';
import { rm } from 'node:fs/promises';
import { comlink } from "vite-plugin-comlink";
import path from 'path';

const DEVELOPMENT_PLUGINS = [
	react(),
	wasm(),
	topLevelAwait(),
	{
		name: "Cleaning assets folder",
		async buildStart() {
			await rm(resolve(__dirname, '../dist/assets'), { recursive: true, force: true });
		}
	},
	comlink()
];



export default defineConfig({
    test: {
    environment: 'jsdom'
    // ... Specify options here.
    },
    resolve: {
    alias: {
        "@": path.resolve(__dirname, "./src/"),
        "@app": path.resolve(__dirname, "./src/app/"),
        "@components": path.resolve(__dirname, "./src/app/components/"),
        "@pages": path.resolve(__dirname, "./src/app/pages/"),
        "@layouts": path.resolve(__dirname, "./src/app/layouts/"),
        "@store": path.resolve(__dirname, "./src/app/store/"),
        "@utils": path.resolve(__dirname, "./src/app/utils/"),
        "@static": path.resolve(__dirname, "./src/app/static/"),
    },
},
plugins: DEVELOPMENT_PLUGINS
})