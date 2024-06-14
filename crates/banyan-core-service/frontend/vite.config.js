const path = require("path");
import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import viteCompression from "vite-plugin-compression2";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import { resolve } from 'path';
import { rm } from 'node:fs/promises';
import { comlink } from "vite-plugin-comlink";

const PRODUCTION_PLUGINS = [
	react(),
	viteCompression(),
	wasm(),
	topLevelAwait(),
	comlink()
];

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

export default ({ mode }) => {
	const isProduction = mode === "production";
	const env = loadEnv(mode, process.cwd(), "");

	return defineConfig({
		base: "/",
		define: {
			"process.env": JSON.stringify(env),
		},
		root: path.join(__dirname, "/"),
		server: {
			port: 3000,
		},
		build: {
			minify: isProduction ? "esbuild": false,
			outDir: path.resolve(__dirname, "../dist/"),
			cssCodeSplit: true,
		},
		worker: {
			plugins: [topLevelAwait(), wasm(), comlink()],
			rollupOptions: {
				watch: false,
			}
		},
		plugins: isProduction ? PRODUCTION_PLUGINS : DEVELOPMENT_PLUGINS,
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
		clean: true,
		minify: isProduction,
	});
};
