import { expect, test } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { initialState, reducer, type State } from "../state";
import { S, categoryLabel, findingText, originText, osLabel, reasonText, statusText, withCount } from "../strings";
import type { AvailabilityMap, ScanResult } from "../types";
import { StartScreen } from "../screens/StartScreen";
import { ScanScreen } from "../screens/ScanScreen";
import { ReviewScreen } from "../screens/ReviewScreen";
import { Sidebar } from "../screens/review/Sidebar";
import { DetailPanel } from "../screens/review/DetailPanel";

const noop = () => {};

test("start screen shows the detected OS and defaults the destination to it", () => {
  const s = reducer(initialState, { type: "init", source: { linux: "fedora" } });
  const html = renderToStaticMarkup(<StartScreen state={s} dispatch={noop} onScan={noop} />);
  expect(html).toContain(osLabel({ linux: "fedora" }));
  expect(html).toMatch(/<option[^>]*selected=""[^>]*>Linux \(Fedora\)<\/option>/);
  expect(html).toMatch(new RegExp(`<button[^>]*>${S.scanButton}</button>`));
  expect(html).not.toMatch(/<button[^>]*disabled/);
});

test("start screen disables scanning until the OS is detected", () => {
  const html = renderToStaticMarkup(<StartScreen state={initialState} dispatch={noop} onScan={noop} />);
  expect(html).toContain(S.detectingOs);
  expect(html).toMatch(/<button[^>]*disabled=""[^>]*>/);
});

test("scan screen lists provider statuses and offers no way to leave", () => {
  let s: State = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "scanStarted" });
  s = reducer(s, { type: "progress", report: { id: "brew", category: "packageManager", status: "running" } });
  s = reducer(s, { type: "progress", report: { id: "npm", category: "globalPackage", status: { ok: { count: 4 } } } });
  const html = renderToStaticMarkup(<ScanScreen state={s} />);
  expect(html).toContain("brew");
  expect(html).toContain(statusText("running"));
  expect(html).toContain(statusText({ ok: { count: 4 } }));
  expect(html).not.toContain("<button");
  expect(html).not.toContain("<select");
});

const fixture = (await import("../__fixtures__/scan.json")).default;
const scan = fixture.scan as unknown as ScanResult;
const availMac = fixture.availability.macOs as unknown as AvailabilityMap;
const availWin = fixture.availability.windows as unknown as AvailabilityMap;

function reviewState(target: "macOs" | "windows"): State {
  const s = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "setTarget", target });
  return reducer(s, { type: "scanFinished", scan, avail: target === "macOs" ? availMac : availWin });
}
const row = (html: string, id: string): string => html.match(new RegExp(`<li[^>]*data-id="${id}"[^>]*>(.*?)</li>`))?.[1] ?? "";
const nextButton = (html: string): string => html.match(new RegExp(`<button[^>]*>${S.nextExport}</button>`))?.[0] ?? "";

test("review marks unavailable and secret items and keeps them unchecked on another OS", () => {
  const html = renderToStaticMarkup(<ReviewScreen state={reviewState("windows")} dispatch={noop} />);
  const brew = row(html, "brew:formula:ripgrep");
  expect(brew).toContain("🚫");
  expect(brew).toMatch(/<input[^>]*type="checkbox"[^>]*disabled=""/);
  expect(brew).not.toMatch(/checked=""/);
  expect(row(html, "shell:file:.zshrc")).toContain("⚠️");
  expect(row(html, "npm:typescript")).toMatch(/checked=""/);
  expect(row(html, "npm:typescript")).toContain("Typed JavaScript tooling");
  expect(nextButton(html)).not.toContain("disabled");
});

test("items without a description say so", () => {
  const bare = { ...scan, items: scan.items.map((i) => ({ ...i, description: null })) };
  const s = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "scanFinished", scan: bare, avail: availMac });
  expect(row(renderToStaticMarkup(<ReviewScreen state={s} dispatch={noop} />), "npm:typescript")).toContain(S.noDescription);
});

test("review disables Next when nothing is selected", () => {
  let s = reviewState("macOs");
  for (const id of [...s.checked]) s = reducer(s, { type: "toggleItem", id });
  expect(nextButton(renderToStaticMarkup(<ReviewScreen state={s} dispatch={noop} />))).toContain("disabled");
});

test("sidebar shows checked/total per category", () => {
  const html = renderToStaticMarkup(<Sidebar state={reviewState("windows")} dispatch={noop} category={null} onCategory={noop} />);
  expect(html).toContain(categoryLabel("packageManager"));
  expect(html).toMatch(/0\/1/);
  expect(html).toMatch(/1\/1/);
});

test("detail panel enables the reveal toggle only for checked items", () => {
  const zshrc = scan.items.find((i) => i.id === "shell:file:.zshrc")!;
  let s = reviewState("macOs");
  let html = renderToStaticMarkup(<DetailPanel item={zshrc} state={s} dispatch={noop} />);
  expect(html).toContain(findingText(zshrc.findings[0]));
  expect(html).toContain(originText(zshrc.origin));
  expect(html).toContain(S.revealSecret);
  expect(html).not.toMatch(/<input[^>]*disabled=""/);
  s = reducer(s, { type: "toggleItem", id: zshrc.id });
  html = renderToStaticMarkup(<DetailPanel item={zshrc} state={s} dispatch={noop} />);
  expect(html).toMatch(/<input[^>]*disabled=""/);
  html = renderToStaticMarkup(<DetailPanel item={zshrc} state={reviewState("windows")} dispatch={noop} />);
  expect(html).toContain(reasonText({ shellUnsupported: "zsh" }));
});

test("not detected and failed providers are collapsed, with raw failure detail", () => {
  const failed = { ...scan, providers: [...scan.providers, { id: "vscode2", category: "editor" as const, status: { failed: { message: "code failed", detail: "raw stderr" } } }] };
  const s = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "scanFinished", scan: failed, avail: availMac });
  const html = renderToStaticMarkup(<ReviewScreen state={s} dispatch={noop} />);
  expect(html).not.toMatch(/<details[^>]*open/);
  expect(html).toContain(withCount(S.notDetected, 2));
  expect(html).toContain(withCount(S.failed, 1));
  expect(html).toContain("<pre>raw stderr</pre>");
});
