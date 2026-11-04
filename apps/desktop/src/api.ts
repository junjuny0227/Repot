import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { Target, Os, ProviderReport, ScanResult, AvailabilityMap, Selection, Format, ExportBundle, OpenedFile, PresenceMap } from "./types";

export const detectOs = (): Promise<Os> => invoke("detect_os");
export async function runScan(onProgress: (r: ProviderReport) => void): Promise<ScanResult> {
  const unlisten = await listen<ProviderReport>("scan://progress", (event) => onProgress(event.payload));
  try { return await invoke<ScanResult>("scan"); } finally { unlisten(); }
}
/** Rejects with an error kind: tooLarge, notRepotFile, newerVersion:<n>, invalid or readFailed. */
export const openSnapshot = (path: string): Promise<OpenedFile> => invoke("open_snapshot", { path });
/** Scan this machine to compare the opened file with it; progress uses the scan event. */
export async function scanMachine(onProgress: (r: ProviderReport) => void): Promise<void> {
  const unlisten = await listen<ProviderReport>("scan://progress", (event) => onProgress(event.payload));
  try { await invoke("scan_machine"); } finally { unlisten(); }
}
/** Empty unless this machine was scanned for comparison and the target is this machine's OS. */
export const presenceFor = (target: Target): Promise<PresenceMap> => invoke("presence_for", { target });
export const availabilityFor = (target: Target): Promise<AvailabilityMap> => invoke("availability_for", { target });
export const previewExport = (selection: Selection, target: Target, format: Format): Promise<ExportBundle> => invoke("preview_export", { selection, target, format });
export const exportTo = (selection: Selection, target: Target, format: Format, parent: string): Promise<string> => invoke("export", { selection, target, format, parent });
export const openExportFolder = (): Promise<void> => invoke("open_export_folder");
export async function chooseRepotFile(): Promise<string | null> {
  const result = await open({ multiple: false, directory: false, filters: [{ name: "Repot", extensions: ["json"] }] });
  return typeof result === "string" ? result : null;
}
export async function chooseFolder(): Promise<string | null> {
  const result = await open({ directory: true, multiple: false });
  return typeof result === "string" ? result : null;
}
