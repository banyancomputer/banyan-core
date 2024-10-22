const path = require("path");
const zlib = require("zlib");
import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import viteCompression from "vite-plugin-compression";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

const PRODUCTION_PLUGINS = [
	react(),
	viteCompression({
		algorithm: "brotliCompress",
		ext: ".br",
		compressionOptions: {
			params: {
				[zlib.constants.BROTLI_PARAM_QUALITY]: 11,
			},
		},
		threshold: 10240,
		minRatio: 0.8,
		deleteOriginalAssets: true,
	}),
	wasm(),
	topLevelAwait()
];

const DEVELOPMENT_PLUGINS = [
	react(),
	wasm(),
	topLevelAwait()
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
			outDir: path.resolve(__dirname, "../admin_dist/"),
			cssCodeSplit: false,
		},
		plugins: isProduction ? PRODUCTION_PLUGINS : DEVELOPMENT_PLUGINS,
		resolve: {
			alias: {
				"@": path.resolve(__dirname, "./src/"),
				"@app": path.resolve(__dirname, "./src/app/"),
				"@components": path.resolve(__dirname, "./src/app/components/"),
				"@utils": path.resolve(__dirname, "./src/app/utils/"),
				"@static": path.resolve(__dirname, "./src/app/static/"),
			},
		},
		clean: true,
		minify: isProduction,
	});
};
