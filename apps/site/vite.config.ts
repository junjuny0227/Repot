import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

// Served from https://junjuny0227.github.io/Repot/, so every asset URL needs the repo prefix.
export default defineConfig({
  base: "/Repot/",
  plugins: [react()],
  server: { port: 1430, strictPort: true },
  preview: { port: 1430, strictPort: true },
  // tokens.test.ts reads stylesheets as text; Vitest blanks CSS unless asked not to.
  test: { css: true },
});
