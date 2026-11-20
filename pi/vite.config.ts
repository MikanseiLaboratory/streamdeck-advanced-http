import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const root = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
  plugins: [react()],
  base: "./",
  build: {
    outDir: "../plugin/dev.flowingspdg.advancedhttp.sdPlugin/propertyinspector",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(root, "index.html"),
      },
    },
  },
});
