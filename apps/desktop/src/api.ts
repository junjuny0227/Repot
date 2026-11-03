import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { Target, Os, ProviderReport, ScanResult, AvailabilityMap, Selection, Format, ExportBundle } from "./types";

export const detectOs = (): Promise<Os> => invoke("detect_os");
export async function runScan(onProgress: (r: ProviderReport) => void): Promise<ScanResult> {
  const unlisten = await listen<ProviderReport>("scan://progress", (event) => onProgress(event.payload));
  try { return await invoke<ScanResult>("scan"); } finally { unlisten(); }
}
export const availabilityFor = (target: Target): Promise<AvailabilityMap> => invoke("availability_for", { target });
export const previewExport = (selection: Selection, target: Target, format: Format): Promise<ExportBundle> => invoke("preview_export", { selection, target, format });
export const exportTo = (selection: Selection, target: Target, format: Format, parent: string): Promise<string> => invoke("export", { selection, target, format, parent });
export const openExportFolder = (): Promise<void> => invoke("open_export_folder");
export async function chooseFolder(): Promise<string | null> {
  const result = await open({ directory: true, multiple: false });
  return typeof result === "string" ? result : null;
}
