import { format, messages, plural, type Messages } from "./i18n";
import type { Category, ExportWarning, Finding, Format, Origin, Os, ProviderStatus, UnavailableReason } from "./types";

// Screens get every user-facing sentence from here. The words live in ./locales/<locale>.json;
// these functions pick the message for the active locale and fill in the values.

export const TARGET_OPTIONS: Os[] = [
  "macOs",
  "windows",
  { linux: "debian" },
  { linux: "fedora" },
  { linux: "arch" },
];

// Product and distribution names are not translated.
const LINUX_NAMES = { debian: "Debian/Ubuntu", fedora: "Fedora", arch: "Arch" } as const;

export function osLabel(os: Os): string {
  if (os === "macOs") return "macOS";
  if (os === "windows") return "Windows";
  const m = messages().os;
  return format(m.linux, { family: os.linux === "other" ? m.linuxOther : LINUX_NAMES[os.linux] });
}

export const categoryLabel = (c: Category): string => messages().category[c];

export function statusText(s: ProviderStatus): string {
  const m = messages().status;
  if (s === "running") return m.running;
  if (s === "notDetected") return m.notDetected;
  if (s === "timedOut") return m.timedOut;
  if ("ok" in s) return format(m.found, { count: s.ok.count });
  return s.failed.message;
}

function findingBody(f: Finding): string {
  const m = messages().finding;
  const k = f.kind;
  if (k === "unreadable") return m.unreadable;
  if ("secret" in k) return k.secret.name ? format(m.secretNamed, { name: k.secret.name }) : m.secret;
  if ("osSpecificLine" in k) return format(m.osSpecificLine, { key: k.osSpecificLine.key });
  return format(m.duplicate, { provider: k.duplicate.otherProvider });
}

export function findingText(f: Finding): string {
  const text = findingBody(f);
  // Line numbers are identifiers, so they are not grouped like counts (1234, not 1,234).
  return f.line === null ? text : format(messages().finding.atLine, { line: String(f.line), text });
}

export function reasonText(r: UnavailableReason): string {
  const m = messages().reason;
  if (r === "unreadable") return m.unreadable;
  if ("differentOs" in r) return format(m.differentOs, { os: osLabel(r.differentOs.source) });
  if ("shellUnsupported" in r) return format(m.shellUnsupported, { shell: r.shellUnsupported });
  return format(m.noCounterpart, { tool: r.noCounterpart });
}

export function warningText(w: ExportWarning, itemName: string): string {
  const m = messages().warning;
  if ("unavailable" in w) return format(m.unavailable, { item: itemName, reason: reasonText(w.unavailable.reason) });
  if ("invalidValue" in w) return format(m.invalidValue, { item: itemName });
  if ("duplicateTarget" in w) return format(m.duplicateTarget, { item: itemName });
  return format(m.readFailed, { item: itemName });
}

// The commands stay as they are: people type them into a terminal.
export function nextSteps(target: Os, exportFormat: Format): string[] {
  const m = messages().nextSteps;
  if (exportFormat === "manifest") return [m.readme];
  const command = target === "windows" ? "powershell -ExecutionPolicy Bypass -File restore.ps1" : "./restore.sh";
  return [m.copy, format(m.run, { command })];
}

export function originText(o: Origin): string {
  const m = messages().origin;
  return "command" in o ? format(m.command, { command: o.command }) : format(m.path, { path: o.path });
}

export const availableOn = (target: Os): string => format(messages().availableOn, { os: osLabel(target) });
export const withCount = (label: string, n: number): string => format(messages().withCount, { label, count: n });
export const selectedCount = (checked: number, total: number): string => plural(messages().selectedCount, total, { checked, total });
export const selectCategory = (label: string): string => format(messages().selectCategory, { category: label });
export const revealSecretLine = (line: number): string => format(messages().revealSecretLine, { line: String(line) });

type UiText = Readonly<Messages["ui"]> & { readonly appTitle: string };

/** Simple labels for the active locale. Each property is a getter, so it follows locale changes. */
export const S: UiText = Object.defineProperties(
  { appTitle: "Repot" },
  Object.fromEntries(
    Object.keys(messages().ui).map((key) => [key, { enumerable: true, get: () => messages().ui[key as keyof Messages["ui"]] }]),
  ),
) as UiText;
