import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

export default defineConfig({
  plugins: [wasm(), topLevelAwait(), solidPlugin()],
  server: {
    port: 3000,
    fs: {
      allow: [".", "../../pkg"],
    },
  },
});
