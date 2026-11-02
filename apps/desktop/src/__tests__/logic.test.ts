import { expect, test } from "vitest";
import type { AvailabilityMap, ScanResult } from "../types";
const fixture = (await import("../__fixtures__/scan.json")).default;
const scan = fixture.scan as unknown as ScanResult;
const availMac = fixture.availability.macOs as unknown as AvailabilityMap;
const availWin = fixture.availability.windows as unknown as AvailabilityMap;
import { applyTarget, buildSelection, categoryCounts, filterItems, initialChecked, isAvailable, revealKey, secretLines, setCategory } from "../logic";

test("initialChecked selects only available items", () => {
  const checked = initialChecked(scan.items, availWin);
  for (const item of scan.items) expect(checked.has(item.id)).toBe(isAvailable(availWin[item.id]));
});

test("applyTarget unchecks newly unavailable items", () => {
  const mac = initialChecked(scan.items, availMac);
  const after = applyTarget(mac, availWin);
  expect([...after].every((id) => isAvailable(availWin[id]))).toBe(true);
  expect(after.size).toBeLessThan(mac.size);
});

test("selection_drops_reveals_of_unchecked_items", () => {   // Review Focus 2
  const zshrc = scan.items.find((i) => i.id === "shell:file:.zshrc")!;
  const line = secretLines(zshrc)[0];
  const revealed = new Set([revealKey(zshrc.id, line)]);
  expect(buildSelection(new Set([zshrc.id]), revealed).revealSecretLines).toEqual([[zshrc.id, line]]);
  expect(buildSelection(new Set(), revealed)).toEqual({ items: [], revealSecretLines: [] });
});

test("filterItems by query and warnings", () => {
  const all = { query: "", onlyAvailable: false, onlyWarnings: false };
  expect(filterItems(scan.items, availMac, { ...all, onlyWarnings: true }).map((i) => i.id)).toContain("shell:file:.zshrc");
  expect(filterItems(scan.items, availMac, { ...all, query: "ZSHRC" }).length).toBeGreaterThan(0);
});

test("setCategory toggles only available items of that category", () => {
  const none = setCategory(initialChecked(scan.items, availWin), scan.items, availWin, "packageManager", false);
  expect(scan.items.filter((i) => i.category === "packageManager").some((i) => none.has(i.id))).toBe(false);
});

test("setCategory on never checks unavailable items and counts follow", () => {
  const on = setCategory(new Set(), scan.items, availWin, "shell", true);
  expect(on.has("shell:file:.zshrc")).toBe(false);
  const counts = categoryCounts(scan.items, initialChecked(scan.items, availMac));
  expect(counts.shell).toEqual({ total: 1, checked: 1 });
  expect(counts.runtime).toEqual({ total: 0, checked: 0 });
});
