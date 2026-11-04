import {
  applyTarget,
  defaultTarget,
  initialChecked,
  isAvailable,
  missingChecked,
  revealKey,
  setCategory,
} from './logic';
import type {
  AvailabilityMap,
  Category,
  Format,
  OpenedFile,
  Os,
  PackageManager,
  PresenceMap,
  ProviderReport,
  ScanResult,
} from './types';

export type Screen = 'start' | 'scanning' | 'review' | 'export' | 'done';
/** Where the items came from when a Repot file was opened instead of scanning. */
export type OpenedInfo = Omit<OpenedFile, 'scan'>;
export interface State {
  /** `source` is this machine; an opened file brings its own source OS in `opened`. */
  screen: Screen;
  source: Os | null;
  target: Os | null;
  opened: OpenedInfo | null;
  /** Destination package manager; reset to the destination's default whenever the destination changes. */
  packages: PackageManager | null;
  providers: ProviderReport[];
  scan: ScanResult | null;
  avail: AvailabilityMap;
  /** Set after comparing an opened file with this machine; empty for other destinations. */
  compared: boolean;
  presence: PresenceMap;
  checked: Set<string>;
  revealed: Set<string>;
  format: Format;
  exportedPath: string | null;
  error: string | null;
}
export type Action =
  | { type: 'init'; source: Os }
  | { type: 'setTarget'; target: Os; avail?: AvailabilityMap; presence?: PresenceMap }
  | {
      type: 'setPackages';
      packages: PackageManager;
      avail: AvailabilityMap;
      presence?: PresenceMap;
    }
  | { type: 'compareStarted' }
  | { type: 'compareFailed' }
  | {
      type: 'compareFinished';
      target: Os;
      packages: PackageManager | null;
      avail: AvailabilityMap;
      presence: PresenceMap;
    }
  | { type: 'scanStarted' }
  | { type: 'progress'; report: ProviderReport }
  | { type: 'scanFinished'; scan: ScanResult; avail: AvailabilityMap }
  | { type: 'fileOpened'; file: OpenedFile; avail: AvailabilityMap }
  | { type: 'toggleItem'; id: string }
  | { type: 'setCategory'; category: Category; on: boolean }
  | { type: 'toggleReveal'; id: string; line: number }
  | { type: 'goExport' }
  | { type: 'backToReview' }
  | { type: 'setFormat'; format: Format }
  | { type: 'exported'; path: string }
  | { type: 'error'; message: string }
  | { type: 'restart' };

export const initialState: State = {
  screen: 'start',
  source: null,
  target: null,
  opened: null,
  packages: null,
  providers: [],
  scan: null,
  avail: {},
  compared: false,
  presence: {},
  checked: new Set(),
  revealed: new Set(),
  format: 'script',
  exportedPath: null,
  error: null,
};

/** The OS the reviewed items come from: the opened file's, or this machine's. */
export const sessionSource = (state: State): Os | null => state.opened?.source ?? state.source;

export function canExport(state: State): boolean {
  return state.screen === 'review' && state.checked.size > 0;
}

function toggled(set: Set<string>, key: string): Set<string> {
  const next = new Set(set);
  if (!next.delete(key)) next.add(key);
  return next;
}

export function reducer(prev: State, action: Action): State {
  if (action.type === 'error') return { ...prev, error: action.message };
  const state = prev.error === null ? prev : { ...prev, error: null };
  switch (action.type) {
    case 'init':
      return {
        ...state,
        source: action.source,
        target: action.source,
        packages: defaultTarget(action.source).packages,
      };
    case 'setTarget': {
      const packages = defaultTarget(action.target).packages;
      const presence = action.presence ?? state.presence;
      return action.avail
        ? {
            ...state,
            target: action.target,
            packages,
            avail: action.avail,
            presence,
            checked: applyTarget(state.checked, action.avail),
          }
        : { ...state, target: action.target, packages, presence };
    }
    case 'setPackages':
      return {
        ...state,
        packages: action.packages,
        avail: action.avail,
        presence: action.presence ?? state.presence,
        checked: applyTarget(state.checked, action.avail),
      };
    case 'scanStarted':
      return {
        ...state,
        screen: 'scanning',
        providers: [],
        opened: null,
        compared: false,
        presence: {},
      };
    case 'compareStarted':
      return { ...state, screen: 'scanning', providers: [] };
    case 'compareFailed':
      return { ...state, screen: 'review' };
    case 'compareFinished':
      return {
        ...state,
        screen: 'review',
        compared: true,
        target: action.target,
        packages: action.packages,
        avail: action.avail,
        presence: action.presence,
        checked: missingChecked(state.scan?.items ?? [], action.avail, action.presence),
      };
    case 'progress': {
      const exists = state.providers.some((p) => p.id === action.report.id);
      const providers = exists
        ? state.providers.map((p) => (p.id === action.report.id ? action.report : p))
        : [...state.providers, action.report];
      return { ...state, providers };
    }
    case 'scanFinished':
      return {
        ...state,
        screen: 'review',
        scan: action.scan,
        providers: action.scan.providers,
        avail: action.avail,
        checked: initialChecked(action.scan.items, action.avail),
        revealed: new Set(),
      };
    case 'fileOpened': {
      const { scan, ...opened } = action.file;
      return {
        ...state,
        screen: 'review',
        scan,
        providers: [],
        avail: action.avail,
        opened,
        compared: false,
        presence: {},
        checked: initialChecked(scan.items, action.avail),
        revealed: new Set(),
      };
    }
    case 'toggleItem':
      if (!isAvailable(state.avail[action.id])) return state;
      return { ...state, checked: toggled(state.checked, action.id) };
    case 'setCategory':
      return {
        ...state,
        checked: setCategory(
          state.checked,
          state.scan?.items ?? [],
          state.avail,
          action.category,
          action.on,
        ),
      };
    case 'toggleReveal':
      // An opened file stores secrets masked; there is no original value to export.
      if (state.opened !== null) return state;
      return { ...state, revealed: toggled(state.revealed, revealKey(action.id, action.line)) };
    case 'goExport':
      return canExport(state) ? { ...state, screen: 'export' } : state;
    case 'backToReview':
      return { ...state, screen: 'review' };
    case 'setFormat':
      return { ...state, format: action.format };
    case 'exported':
      return { ...state, screen: 'done', exportedPath: action.path };
    case 'restart':
      return {
        ...initialState,
        checked: new Set(),
        revealed: new Set(),
        source: state.source,
        target: state.target,
        packages: state.packages,
      };
  }
}
