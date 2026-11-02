import { expect, test } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { initialState, reducer, type State } from "../state";
import { S, osLabel, statusText } from "../strings";
import { StartScreen } from "../screens/StartScreen";
import { ScanScreen } from "../screens/ScanScreen";

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
