export type LinuxFamily = "debian" | "fedora" | "arch" | "other";
export type Os = "macOs" | "windows" | { linux: LinuxFamily };
export type Shell = "zsh" | "bash" | "fish" | "powerShell";
export type Category = "packageManager" | "runtime" | "globalPackage" | "editor" | "git" | "shell";
export type RuntimeTool = "nvm" | "nvmWindows" | "fnm" | "volta" | "pyenv" | "pyenvWin" | "rustup" | "sdkman";
export type Editor = "vsCode" | "cursor";
export type TargetPath = { home: string } | { editorUser: { editor: Editor; rel: string } } | "powerShellProfile";
export type Origin = { command: string } | { path: string };
export type Portability = "any" | "sameOs" | { sameShell: Shell } | { mapped: RuntimeTool };
export type ItemDetails =
  | { package: { manager: string; kind: "formula" | "cask" | "tap" | "package" } }
  | { runtime: { tool: RuntimeTool; runtime: string; isDefault: boolean } }
  | { globalPackage: { manager: string } }
  | { extension: { editor: Editor } }
  | { configFile: { source: string; target: TargetPath } }
  | { shellPlugin: { url: string; target: TargetPath } };
export type FindingKind =
  | { secret: { name: string | null } } | { osSpecificLine: { key: string } }
  | { duplicate: { otherProvider: string } } | "unreadable";
export interface Finding { line: number | null; kind: FindingKind }
export interface Item {
  id: string; provider: string; category: Category; name: string;
  version: string | null; description: string | null; catalogId: string | null; origin: Origin;
  portability: Portability; details: ItemDetails; findings: Finding[];
}
export type ProviderStatus = "running" | "notDetected" | "timedOut" | { ok: { count: number } } | { failed: { message: string; detail: string } };
export interface ProviderReport { id: string; category: Category; status: ProviderStatus }
export interface ScanResult { items: Item[]; providers: ProviderReport[] }
export type UnavailableReason = { differentOs: { source: Os } } | { shellUnsupported: Shell } | { noCounterpart: RuntimeTool } | "unreadable";
export type Availability = "available" | { unavailable: UnavailableReason };
export type AvailabilityMap = Record<string, Availability>;
export interface Selection { items: string[]; revealSecretLines: [string, number][] }
export type Format = "script" | "manifest";
export interface OutputFile { path: string; contents: string; executable: boolean }
export type ExportWarning =
  | { unavailable: { itemId: string; reason: UnavailableReason } } | { invalidValue: { itemId: string } }
  | { duplicateTarget: { itemId: string } } | { readFailed: { itemId: string } };
export interface ExportBundle { files: OutputFile[]; warnings: ExportWarning[] }
