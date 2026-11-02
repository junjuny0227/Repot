import type { Availability, AvailabilityMap, Category, Item, Os, Selection } from "./types";
import { TARGET_OPTIONS } from "./strings";

export const CATEGORIES: Category[] = ["packageManager", "runtime", "globalPackage", "editor", "git", "shell"];

export function isAvailable(a: Availability | undefined): boolean {
  return a === "available";
}

export function initialChecked(items: Item[], avail: AvailabilityMap): Set<string> {
  return new Set(items.filter((i) => isAvailable(avail[i.id])).map((i) => i.id));
}

export function applyTarget(checked: Set<string>, avail: AvailabilityMap): Set<string> {
  return new Set([...checked].filter((id) => isAvailable(avail[id])));
}

export function setCategory(checked: Set<string>, items: Item[], avail: AvailabilityMap, category: Category, on: boolean): Set<string> {
  const next = new Set(checked);
  for (const item of items) {
    if (item.category !== category) continue;
    if (!on) next.delete(item.id);
    else if (isAvailable(avail[item.id])) next.add(item.id);
  }
  return next;
}

export interface Filter { query: string; onlyAvailable: boolean; onlyWarnings: boolean }

export function filterItems(items: Item[], avail: AvailabilityMap, f: Filter): Item[] {
  const q = f.query.trim().toLowerCase();
  return items.filter((i) =>
    (!q || i.name.toLowerCase().includes(q) || (i.description ?? "").toLowerCase().includes(q)) &&
    (!f.onlyAvailable || isAvailable(avail[i.id])) &&
    (!f.onlyWarnings || i.findings.length > 0));
}

export function secretLines(item: Item): number[] {
  return item.findings.flatMap((f) => (typeof f.kind === "object" && "secret" in f.kind && f.line !== null ? [f.line] : []));
}

export const revealKey = (id: string, line: number): string => `${id}#${line}`;

export function buildSelection(checked: Set<string>, revealed: Set<string>): Selection {
  const revealSecretLines: [string, number][] = [];
  for (const key of revealed) {
    const at = key.lastIndexOf("#");
    const id = key.slice(0, at);
    const line = Number(key.slice(at + 1));
    if (at > 0 && checked.has(id) && Number.isInteger(line)) revealSecretLines.push([id, line]);
  }
  return { items: [...checked], revealSecretLines };
}

export function categoryCounts(items: Item[], checked: Set<string>): Record<Category, { total: number; checked: number }> {
  const counts = Object.fromEntries(CATEGORIES.map((c) => [c, { total: 0, checked: 0 }])) as Record<Category, { total: number; checked: number }>;
  for (const item of items) {
    counts[item.category].total += 1;
    if (checked.has(item.id)) counts[item.category].checked += 1;
  }
  return counts;
}

/** Stable string key for an OS, usable as a `<select>` option value. */
export const osKey = (os: Os): string => JSON.stringify(os);

/** Destination choices, plus the current target if it is not one of them (e.g. Linux "other"). */
export function targetOptions(current: Os | null): Os[] {
  if (current === null || TARGET_OPTIONS.some((o) => osKey(o) === osKey(current))) return TARGET_OPTIONS;
  return [...TARGET_OPTIONS, current];
}
