import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";
import { nodePolyfills } from "vite-plugin-node-polyfills";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [wasm(), topLevelAwait(), solidPlugin(), nodePolyfills()],
  build: {
    target: "esnext",
  },
  server: {
    port: 3000,
    fs: {
      allow: [".", "../../pkg"],
    },
  },
});
