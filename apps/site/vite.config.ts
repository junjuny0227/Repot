import { defineConfig, type Plugin } from "vitest/config";
import react from "@vitejs/plugin-react";
import { headTags, noscript } from "./src/head.ts";
import { LOCALES, isLocale } from "./src/i18n.ts";

// One static page per locale. Each HTML file only sets <html lang>; the plugin fills in the
// translated <title>, description, canonical and hreflang links from the locale JSON files.
function localePages(): Plugin {
  return {
    name: "repot-locale-pages",
    transformIndexHtml: {
      order: "pre",
      handler(html) {
        const lang = html.match(/<html lang="([^"]+)"/)?.[1];
        if (!isLocale(lang)) throw new Error(`Unknown page language: ${lang}`);
        return html.replace("<!-- repot:head -->", headTags(lang)).replace("<!-- repot:noscript -->", noscript(lang));
      },
    },
  };
}

// Served from https://junjuny0227.github.io/Repot/, so every asset URL needs the repo prefix.
export default defineConfig({
  base: "/Repot/",
  plugins: [react(), localePages()],
  build: {
    rollupOptions: {
      // Relative to the site root (apps/site): index.html, ko/index.html, …
      input: Object.fromEntries(LOCALES.map((l) => [l.code, l.code === "en" ? "index.html" : `${l.code}/index.html`])),
    },
  },
  server: { port: 1430, strictPort: true },
  preview: { port: 1430, strictPort: true },
  // tokens.test.ts reads stylesheets as text; Vitest blanks CSS unless asked not to.
  test: { css: true },
});
