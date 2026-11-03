import type { AvailabilityMap, Category, Format, Os, PackageManager, ProviderReport, ScanResult } from "./types";
import { applyTarget, defaultTarget, initialChecked, isAvailable, revealKey, setCategory } from "./logic";

export type Screen = "start" | "scanning" | "review" | "export" | "done";
export interface State {
  screen: Screen; source: Os | null; target: Os | null;
  /** Destination package manager; reset to the destination's default whenever the destination changes. */
  packages: PackageManager | null;
  providers: ProviderReport[]; scan: ScanResult | null; avail: AvailabilityMap;
  checked: Set<string>; revealed: Set<string>; format: Format;
  exportedPath: string | null; error: string | null;
}
export type Action =
  | { type: "init"; source: Os }
  | { type: "setTarget"; target: Os; avail?: AvailabilityMap }
  | { type: "setPackages"; packages: PackageManager; avail: AvailabilityMap }
  | { type: "scanStarted" }
  | { type: "progress"; report: ProviderReport }
  | { type: "scanFinished"; scan: ScanResult; avail: AvailabilityMap }
  | { type: "toggleItem"; id: string }
  | { type: "setCategory"; category: Category; on: boolean }
  | { type: "toggleReveal"; id: string; line: number }
  | { type: "goExport" } | { type: "backToReview" }
  | { type: "setFormat"; format: Format }
  | { type: "exported"; path: string }
  | { type: "error"; message: string }
  | { type: "restart" };

export const initialState: State = {
  screen: "start", source: null, target: null, packages: null,
  providers: [], scan: null, avail: {},
  checked: new Set(), revealed: new Set(), format: "script",
  exportedPath: null, error: null,
};

export function canExport(state: State): boolean {
  return state.screen === "review" && state.checked.size > 0;
}

function toggled(set: Set<string>, key: string): Set<string> {
  const next = new Set(set);
  if (!next.delete(key)) next.add(key);
  return next;
}

export function reducer(prev: State, action: Action): State {
  if (action.type === "error") return { ...prev, error: action.message };
  const state = prev.error === null ? prev : { ...prev, error: null };
  switch (action.type) {
    case "init":
      return { ...state, source: action.source, target: action.source, packages: defaultTarget(action.source).packages };
    case "setTarget": {
      const packages = defaultTarget(action.target).packages;
      return action.avail
        ? { ...state, target: action.target, packages, avail: action.avail, checked: applyTarget(state.checked, action.avail) }
        : { ...state, target: action.target, packages };
    }
    case "setPackages":
      return { ...state, packages: action.packages, avail: action.avail, checked: applyTarget(state.checked, action.avail) };
    case "scanStarted":
      return { ...state, screen: "scanning", providers: [] };
    case "progress": {
      const exists = state.providers.some((p) => p.id === action.report.id);
      const providers = exists
        ? state.providers.map((p) => (p.id === action.report.id ? action.report : p))
        : [...state.providers, action.report];
      return { ...state, providers };
    }
    case "scanFinished":
      return {
        ...state, screen: "review", scan: action.scan, providers: action.scan.providers, avail: action.avail,
        checked: initialChecked(action.scan.items, action.avail), revealed: new Set(),
      };
    case "toggleItem":
      if (!isAvailable(state.avail[action.id])) return state;
      return { ...state, checked: toggled(state.checked, action.id) };
    case "setCategory":
      return { ...state, checked: setCategory(state.checked, state.scan?.items ?? [], state.avail, action.category, action.on) };
    case "toggleReveal":
      return { ...state, revealed: toggled(state.revealed, revealKey(action.id, action.line)) };
    case "goExport":
      return canExport(state) ? { ...state, screen: "export" } : state;
    case "backToReview":
      return { ...state, screen: "review" };
    case "setFormat":
      return { ...state, format: action.format };
    case "exported":
      return { ...state, screen: "done", exportedPath: action.path };
    case "restart":
      return { ...initialState, checked: new Set(), revealed: new Set(), source: state.source, target: state.target, packages: state.packages };
  }
}
