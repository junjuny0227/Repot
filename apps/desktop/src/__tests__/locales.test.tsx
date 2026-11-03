import { afterEach, expect, test } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { LOCALES, setLocale, type Locale } from "../i18n";
import en from "../locales/en.json";
import { initialState, reducer, type State } from "../state";
import { S, nextSteps } from "../strings";
import type { AvailabilityMap, ExportBundle, ScanResult } from "../types";
import { LanguageSelect } from "../screens/LanguageSelect";
import { StartScreen } from "../screens/StartScreen";
import { ReviewScreen } from "../screens/ReviewScreen";
import { DetailPanel } from "../screens/review/DetailPanel";
import { ExportView } from "../screens/ExportScreen";
import { DoneScreen } from "../screens/DoneScreen";

const noop = () => {};
const fixture = (await import("../__fixtures__/scan.json")).default;
const scan = fixture.scan as unknown as ScanResult;
const avail = fixture.availability.windows as unknown as AvailabilityMap;
const files = import.meta.glob<Record<string, unknown>>("../locales/*.json", { eager: true, import: "default" });
const messagesOf = (code: Locale) => files[`../locales/${code}.json`];

afterEach(() => setLocale("en", false));

function strings(obj: unknown): string[] {
  if (typeof obj === "string") return [obj];
  return Object.values(obj as Record<string, unknown>).flatMap(strings);
}
const escape = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

function renderAll(): string {
  let s: State = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "setTarget", target: "windows" });
  const out = [renderToStaticMarkup(<StartScreen state={s} dispatch={noop} onScan={noop} />)];
  s = reducer(s, { type: "scanFinished", scan, avail });
  out.push(renderToStaticMarkup(<ReviewScreen state={s} dispatch={noop} />));
  for (const item of scan.items) out.push(renderToStaticMarkup(<DetailPanel item={item} state={s} dispatch={noop} />));
  const bundle: ExportBundle = {
    files: [{ path: "restore.ps1", contents: "Write-Host restore", executable: true }],
    warnings: [{ readFailed: { itemId: "git:file:.gitconfig" } }, { unavailable: { itemId: "brew:formula:ripgrep", reason: { differentOs: { source: "macOs" } } } }],
  };
  s = reducer(s, { type: "goExport" });
  out.push(renderToStaticMarkup(<ExportView state={s} dispatch={noop} bundle={bundle} selectedPath="restore.ps1" onSelect={noop} onSave={noop} saving={false} />));
  s = reducer(s, { type: "exported", path: "/tmp/repot-export-2026-11-03" });
  out.push(renderToStaticMarkup(<DoneScreen state={s} dispatch={noop} />));
  return out.join("\n");
}

test("language picker lists every locale in its own language", () => {
  setLocale("ja", false);
  const html = renderToStaticMarkup(<LanguageSelect />);
  for (const { code, name } of LOCALES) expect(html).toContain(`<option value="${code}" lang="${code}"`);
  expect(html).toContain(">English<");
  expect(html).toContain(">한국어<");
  expect(html).toContain(">日本語<");
  expect(html).toContain(">简体中文<");
  expect(html).toMatch(/<option value="ja" lang="ja" selected="">/);
  expect(html).toContain(`<span class="visually-hidden">言語</span>`);
});

test("start and review screens offer the language picker", () => {
  const html = renderAll();
  expect(html.match(/class="popup-field language-field/g)).toHaveLength(2);
});

test("screens render in every locale without English leftovers", () => {
  const english = renderAll();
  for (const { code } of LOCALES.filter((l) => l.code !== "en")) {
    setLocale(code, false);
    const html = renderAll();
    // English phrases on the English screens must not show up on the translated ones
    // (catches copy hard-coded in a component as well as untranslated messages).
    const leftovers = strings(en)
      .filter((s) => s.includes(" ") && !s.includes("{"))
      .filter((s) => english.includes(escape(s)) && html.includes(escape(s)));
    expect(leftovers, code).toEqual([]);
    expect(html).toContain(escape(S.reviewTitle));
    expect(html).toContain(escape(nextSteps("windows", "script")[1]));
  }
});

// Values that may stay as in English: a product name, pure formatting, and terms a
// language's developers write in English (mainland Chinese dev tools say "Shell").
const SAME_EVERYWHERE = new Set(["category.git", "os.linux", "withCount"]);
const SAME_IN: Record<string, Set<string>> = { "zh-Hans": new Set(["category.shell"]) };
function flat(obj: unknown, prefix = ""): [string, string][] {
  if (typeof obj === "string") return [[prefix, obj]];
  return Object.entries(obj as Record<string, unknown>).flatMap(([k, v]) => flat(v, prefix ? `${prefix}.${k}` : k));
}

test("every message is translated", () => {
  const source = new Map(flat(en));
  for (const { code } of LOCALES.filter((l) => l.code !== "en")) {
    const same = flat(messagesOf(code)).filter(([key, value]) => source.get(key) === value && !SAME_EVERYWHERE.has(key) && !SAME_IN[code]?.has(key));
    expect(same.map(([key]) => key), code).toEqual([]);
  }
});

test("commands and product names stay untranslated", () => {
  for (const { code } of LOCALES) {
    setLocale(code, false);
    expect(nextSteps("windows", "script")[1]).toContain("powershell -ExecutionPolicy Bypass -File restore.ps1");
    expect(nextSteps("macOs", "script")[1]).toContain("./restore.sh");
    expect(nextSteps("macOs", "manifest")[0]).toContain("README.md");
    expect(S.appTitle).toBe("Repot");
  }
});
