import type { Availability, AvailabilityMap, Category, ExportWarning, Item, Os, OutputFile, PackageManager, Selection, Target } from "./types";
import { TARGET_OPTIONS, itemDescription } from "./strings";

export const CATEGORIES: Category[] = ["packageManager", "runtime", "globalPackage", "editor", "git", "shell"];

/** The package manager a destination uses unless the user picks another (spec §9). */
export function defaultTarget(os: Os): Target {
  const packages = os === "macOs" ? "brew" : os === "windows" ? "winget"
    : ({ debian: "apt", fedora: "dnf", arch: "pacman", other: null } as const)[os.linux];
  return { os, packages };
}

export const WINDOWS_PACKAGE_MANAGERS: PackageManager[] = ["winget", "scoop", "choco"];

/** Only a Windows destination offers a choice, and only from another OS: same-OS moves keep the original managers. */
export const choosesPackages = (source: Os | null, target: Os | null): boolean =>
  target === "windows" && source !== null && source !== "windows";

/** The destination sent to core for judging, previewing and exporting. */
export function currentTarget(s: { target: Os | null; packages: PackageManager | null }): Target | null {
  return s.target === null ? null : { os: s.target, packages: s.packages };
}

export function isAvailable(a: Availability | undefined): boolean {
  return a === "available" || (typeof a === "object" && "mapped" in a);
}

/** The destination manager and name when core mapped a package to another OS's manager. */
export function mapping(a: Availability | undefined): { manager: PackageManager; name: string } | null {
  return typeof a === "object" && "mapped" in a ? a.mapped : null;
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
    // Search what the user sees: the description in the current language.
    (!q || i.name.toLowerCase().includes(q) || (itemDescription(i) ?? "").toLowerCase().includes(q)) &&
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

export interface FileTree { files: OutputFile[]; folders: { name: string; files: OutputFile[] }[] }

/** Top-level files stay as they are; anything under a folder (e.g. `files/`) is grouped so it can be folded. */
export function fileTree(files: OutputFile[]): FileTree {
  const tree: FileTree = { files: [], folders: [] };
  for (const file of files) {
    const at = file.path.indexOf("/");
    if (at < 0) { tree.files.push(file); continue; }
    const name = file.path.slice(0, at);
    const folder = tree.folders.find((f) => f.name === name);
    if (folder) folder.files.push(file);
    else tree.folders.push({ name, files: [file] });
  }
  return tree;
}

export function warningItemId(w: ExportWarning): string {
  if ("unavailable" in w) return w.unavailable.itemId;
  if ("invalidValue" in w) return w.invalidValue.itemId;
  if ("duplicateTarget" in w) return w.duplicateTarget.itemId;
  return w.readFailed.itemId;
}
