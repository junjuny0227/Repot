import { beforeEach, expect, test, vi } from "vitest";

const invoke = vi.fn(async () => ({}));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
const { availabilityFor, exportTo, previewExport } = await import("../api");

beforeEach(() => invoke.mockClear());

test("availability, preview and export send the same Target", async () => {
  const target = { os: "windows" as const, packages: "scoop" as const };
  const selection = { items: ["brew:formula:ripgrep"], revealSecretLines: [] };
  await availabilityFor(target);
  await previewExport(selection, target, "script");
  await exportTo(selection, target, "manifest", "/tmp/out");
  expect(invoke.mock.calls).toEqual([
    ["availability_for", { target }],
    ["preview_export", { selection, target, format: "script" }],
    ["export", { selection, target, format: "manifest", parent: "/tmp/out" }],
  ]);
});

test("openSnapshot sends the chosen path", async () => {
  const { openSnapshot } = await import("../api");
  await openSnapshot("/tmp/repot-export-2026-11-04/repot.json");
  expect(invoke.mock.calls).toEqual([["open_snapshot", { path: "/tmp/repot-export-2026-11-04/repot.json" }]]);
});

test("presenceFor sends the Target", async () => {
  const { presenceFor } = await import("../api");
  const target = { os: "macOs" as const, packages: "brew" as const };
  await presenceFor(target);
  expect(invoke.mock.calls).toEqual([["presence_for", { target }]]);
});
