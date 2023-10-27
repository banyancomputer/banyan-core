const path = require("path");
const zlib = require("zlib");
import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import viteCompression from "vite-plugin-compression";

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
    deleteOriginalAssets: false,
  })
];

const DEVELOPMENT_PLUGINS = [react()];

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
      minify: "esbuild",
      outDir: path.resolve(__dirname, "dist/"),
      cssCodeSplit: false,
    },
    plugins: isProduction ? PRODUCTION_PLUGINS : DEVELOPMENT_PLUGINS,
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src/"),
        "@app": path.resolve(__dirname, "./src/app/"),
        "@components": path.resolve(__dirname, "./src/app/components/"),
        "@static": path.resolve(__dirname, "./src/app/static/"),
      },
    },
    clean: true,
    minify: true,
  });
};
