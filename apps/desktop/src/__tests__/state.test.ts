import { expect, test } from "vitest";
import type { AvailabilityMap, ScanResult } from "../types";
const fixture = (await import("../__fixtures__/scan.json")).default;
const scan = fixture.scan as unknown as ScanResult;
const availMac = fixture.availability.macOs as unknown as AvailabilityMap;
const availWin = fixture.availability.windows as unknown as AvailabilityMap;
import { canExport, initialState, reducer } from "../state";
import { isAvailable } from "../logic";

test("scan_finished_uses_current_target", () => {   // Review Focus 1
  let s = reducer(initialState, { type: "init", source: "macOs" });
  s = reducer(s, { type: "scanStarted" });
  s = reducer(s, { type: "setTarget", target: "windows" });
  s = reducer(s, { type: "scanFinished", scan, avail: availWin });
  expect(s.screen).toBe("review");
  expect(s.target).toBe("windows");
  expect([...s.checked].every((id) => isAvailable(availWin[id]))).toBe(true);
});

test("progress replaces reports by id", () => {
  let s = reducer(initialState, { type: "progress", report: { id: "brew", category: "packageManager", status: "running" } });
  s = reducer(s, { type: "progress", report: { id: "brew", category: "packageManager", status: { ok: { count: 3 } } } });
  expect(s.providers).toEqual([{ id: "brew", category: "packageManager", status: { ok: { count: 3 } } }]);
});

test("can_export_requires_selection", () => {   // Review Focus 3
  let s = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "scanFinished", scan, avail: availMac });
  expect(canExport(s)).toBe(true);
  for (const id of [...s.checked]) s = reducer(s, { type: "toggleItem", id });
  expect(canExport(s)).toBe(false);
});

test("export_error_keeps_export_screen", () => {   // Review Focus 5
  let s = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "scanFinished", scan, avail: availMac });
  s = reducer(s, { type: "goExport" });
  s = reducer(s, { type: "error", message: "permission denied" });
  expect(s.screen).toBe("export");
  expect(s.error).toBe("permission denied");
  s = reducer(s, { type: "setFormat", format: "manifest" });
  expect(s.error).toBeNull();
});

test("setTarget after scan unchecks, toggleItem ignores unavailable, restart keeps OSes", () => {
  let s = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "scanFinished", scan, avail: availMac });
  s = reducer(s, { type: "toggleReveal", id: "shell:file:.zshrc", line: 1 });
  s = reducer(s, { type: "setTarget", target: "windows", avail: availWin });
  expect(s.checked.has("brew:cask:rectangle")).toBe(false);
  s = reducer(s, { type: "toggleItem", id: "brew:cask:rectangle" });
  expect(s.checked.has("brew:cask:rectangle")).toBe(false);
  // A package mapped to a Windows manager stays selectable.
  expect(s.checked.has("brew:formula:ripgrep")).toBe(true);
  s = reducer(s, { type: "restart" });
  expect(s).toEqual({ ...initialState, source: "macOs", target: "windows", packages: "winget" });
});

test("init and setTarget use the destination's default package manager", () => {
  let s = reducer(initialState, { type: "init", source: "macOs" });
  expect(s.packages).toBe("brew");
  s = reducer(s, { type: "setTarget", target: { linux: "fedora" } });
  expect(s.packages).toBe("dnf");
  s = reducer(s, { type: "setTarget", target: { linux: "other" } });
  expect(s.packages).toBeNull();
});

test("changing the destination resets the Windows package manager", () => {   // Review Focus 5
  let s = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "setTarget", target: "windows" });
  s = reducer(s, { type: "scanFinished", scan, avail: availWin });
  s = reducer(s, { type: "setPackages", packages: "scoop", avail: availWin });
  expect(s.packages).toBe("scoop");
  s = reducer(s, { type: "setTarget", target: "macOs", avail: availMac });
  s = reducer(s, { type: "setTarget", target: "windows", avail: availWin });
  expect(s.packages).toBe("winget");
});

test("changing the package manager re-judges and unchecks newly unavailable items", () => {   // Review Focus 5
  let s = reducer(reducer(initialState, { type: "init", source: "macOs" }), { type: "setTarget", target: "windows" });
  s = reducer(s, { type: "scanFinished", scan, avail: availWin });
  expect(s.checked.has("brew:formula:ripgrep")).toBe(true);
  const choco: AvailabilityMap = { ...availWin, "brew:formula:ripgrep": { unavailable: { notInPackageMap: { manager: "choco" } } } };
  s = reducer(s, { type: "setPackages", packages: "choco", avail: choco });
  expect(s.packages).toBe("choco");
  expect(s.avail).toBe(choco);
  expect(s.checked.has("brew:formula:ripgrep")).toBe(false);
  s = reducer(s, { type: "toggleItem", id: "brew:formula:ripgrep" });
  expect(s.checked.has("brew:formula:ripgrep")).toBe(false);
});
