import { defineConfig } from "vite";
import { dirname, resolve } from "node:path";
//import { VitePWA } from "vite-plugin-pwa";
//import pkg from "./package.json" with { type: "json" };

export default defineConfig({
  build: {
    outDir: "./dist",
    emptyOutDir: true,
    lib: {
      entry: resolve(import.meta.dirname, "src-js/main.js"),
      name: "MdEditor",
      fileName: "md-editor",
    },
  },
});
