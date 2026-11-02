import type { Category, ExportWarning, Finding, Format, Origin, Os, ProviderStatus, UnavailableReason } from "./types";

// Every user-facing sentence lives here so the UI can be translated by replacing this file.

export const TARGET_OPTIONS: Os[] = [
  "macOs",
  "windows",
  { linux: "debian" },
  { linux: "fedora" },
  { linux: "arch" },
];

const LINUX_LABELS = { debian: "Debian/Ubuntu", fedora: "Fedora", arch: "Arch", other: "other" } as const;

export function osLabel(os: Os): string {
  if (os === "macOs") return "macOS";
  if (os === "windows") return "Windows";
  return `Linux (${LINUX_LABELS[os.linux]})`;
}

const CATEGORY_LABELS: Record<Category, string> = {
  packageManager: "Packages",
  runtime: "Runtimes",
  globalPackage: "Global packages",
  editor: "Editors",
  git: "Git",
  shell: "Shell",
};
export const categoryLabel = (c: Category): string => CATEGORY_LABELS[c];

export function statusText(s: ProviderStatus): string {
  if (s === "running") return "Scanning…";
  if (s === "notDetected") return "Not detected";
  if (s === "timedOut") return "Timed out (30 s)";
  if ("ok" in s) return `Found ${s.ok.count}`;
  return s.failed.message;
}

export function findingText(f: Finding): string {
  const at = f.line === null ? "" : `Line ${f.line}: `;
  const k = f.kind;
  if (k === "unreadable") return `${at}This file could not be read, so it will not be exported.`;
  if ("secret" in k) return `${at}Looks like a secret${k.secret.name ? ` (${k.secret.name})` : ""}. It will be masked unless you allow the original value.`;
  if ("osSpecificLine" in k) return `${at}${k.osSpecificLine.key} is specific to this OS and will be commented out on a different OS.`;
  return `${at}Also detected by ${k.duplicate.otherProvider}.`;
}

export function reasonText(r: UnavailableReason): string {
  if (r === "unreadable") return "Unavailable: the file could not be read.";
  if ("differentOs" in r) return `Unavailable: only works on ${osLabel(r.differentOs.source)}.`;
  if ("shellUnsupported" in r) return `Unavailable: ${r.shellUnsupported} is not available on the destination OS.`;
  return `Unavailable: ${r.noCounterpart} has no counterpart on the destination OS.`;
}

export function warningText(w: ExportWarning, itemName: string): string {
  if ("unavailable" in w) return `${itemName} was skipped. ${reasonText(w.unavailable.reason)}`;
  if ("invalidValue" in w) return `${itemName} was skipped because it contains characters that are unsafe in a script.`;
  if ("duplicateTarget" in w) return `${itemName} was skipped because another item writes to the same file.`;
  return `${itemName} was skipped because the file could not be read.`;
}

export function nextSteps(target: Os, format: Format): string[] {
  if (format === "manifest") return ["Open README.md in this folder and follow the steps."];
  if (target === "windows") return ["Copy this folder to the new machine.", "Run: powershell -ExecutionPolicy Bypass -File restore.ps1"];
  return ["Copy this folder to the new machine.", "Run: ./restore.sh"];
}

export function originText(o: Origin): string {
  return "command" in o ? `Detected with: ${o.command}` : `File: ${o.path}`;
}

export const availableOn = (target: Os): string => `Available on ${osLabel(target)}.`;
export const withCount = (label: string, n: number): string => `${label} (${n})`;
export const selectedCount = (checked: number, total: number): string => `${checked} of ${total} items selected`;
export const selectCategory = (label: string): string => `Select all in ${label}`;
export const revealLine = (line: number): string => `Line ${line}`;

export const S = {
  appTitle: "Repot",
  currentOs: "This machine",
  targetOs: "Destination OS",
  scanButton: "Scan this machine",
  detectingOs: "Detecting…",
  startIntro: "Repot reads your development setup and turns it into a restore script or package lists for a new machine. Nothing on this machine is changed.",
  scanning: "Scanning this machine…",
  scanIntro: "Each tool is checked separately. This can take up to 30 seconds.",
  errorTitle: "Something went wrong",
  available: "Available",
  reviewTitle: "Review what to move",
  allItems: "All items",
  noDescription: "No description",
  noItems: "No items match the current filters.",
  selectItem: "Select an item to see its details.",
  description: "Description",
  origin: "Source",
  availability: "On the destination",
  findings: "Findings",
  noFindings: "Nothing to warn about.",
  badgeSecret: "Contains a secret",
  badgeDuplicate: "Also found by another tool",
  badgeUnavailable: "Unavailable on the destination OS",
  failedDetail: "Details",
  search: "Search",
  onlyAvailable: "Available only",
  onlyWarnings: "With warnings only",
  revealSecret: "Export original value",
  notDetected: "Not detected",
  failed: "Failed",
  nextExport: "Next: Export",
  exportTitle: "Export",
  format: "Format",
  formatScript: "Restore script",
  formatManifest: "Native manifests",
  back: "Back",
  save: "Choose folder and save",
  saving: "Saving…",
  previewLoading: "Preparing preview…",
  previewFailed: "The preview could not be created. Change the format or go back and try again.",
  previewEmpty: "No files will be created.",
  warningsTitle: "Some items will be skipped",
  savedTo: "Saved to",
  nextStepsTitle: "Next steps",
  done: "Export complete",
  openFolder: "Open folder",
  startOver: "Start over",
  secretLimit: "Secret detection is pattern-based and may miss some secrets. Review files before sharing.",
} as const;
