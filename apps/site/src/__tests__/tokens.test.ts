import { expect, test } from "vitest";
import desktopTokens from "../../../desktop/src/styles/tokens.css?raw";

// An undefined var(--x) silently resolves to nothing, so a renamed desktop token
// would drop colors from the site without any build error.
const siteCss = import.meta.glob<string>("../styles/*.css", { query: "?raw", import: "default", eager: true });

const defined = (css: string) => new Set([...css.matchAll(/(--[\w-]+)\s*:/g)].map((m) => m[1]));
const used = (css: string) => new Set([...css.matchAll(/var\((--[\w-]+)/g)].map((m) => m[1]));

test("every CSS variable the site uses is defined", () => {
  const all = [desktopTokens, ...Object.values(siteCss)];
  const names = new Set(all.flatMap((css) => [...defined(css)]));
  const missing = Object.values(siteCss).flatMap((css) => [...used(css)]).filter((n) => !names.has(n));
  expect(Object.keys(siteCss).length).toBeGreaterThan(0);
  expect(missing).toEqual([]);
});

test("the site imports the desktop tokens instead of copying them", () => {
  expect(Object.values(siteCss).join("\n")).toContain('@import "../../../desktop/src/styles/tokens.css"');
});
