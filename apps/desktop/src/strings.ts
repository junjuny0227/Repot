import { errorMessage } from './errors';
import { format, getLocale, type Messages, messages, plural } from './i18n';
import type {
  Category,
  ExportWarning,
  Finding,
  Format,
  Item,
  JetBrainsIde,
  Origin,
  Os,
  PackageManager,
  Presence,
  PresenceMap,
  ProviderStatus,
  UnavailableReason,
} from './types';

// Screens get every user-facing sentence from here. The words live in ./locales/<locale>.json;
// these functions pick the message for the active locale and fill in the values.

export const TARGET_OPTIONS: Os[] = [
  'macOs',
  'windows',
  { linux: 'debian' },
  { linux: 'fedora' },
  { linux: 'arch' },
];

// Product and distribution names are not translated.
const LINUX_NAMES = { debian: 'Debian/Ubuntu', fedora: 'Fedora', arch: 'Arch' } as const;

export function osLabel(os: Os): string {
  if (os === 'macOs') return 'macOS';
  if (os === 'windows') return 'Windows';
  const m = messages().os;
  return format(m.linux, { family: os.linux === 'other' ? m.linuxOther : LINUX_NAMES[os.linux] });
}

export const categoryLabel = (c: Category): string => messages().category[c];

/** Built-in descriptions are translated by catalog id; tool-provided ones are shown as they are. */
export function itemDescription(item: Item): string | null {
  const catalog: Record<string, string> = messages().catalog;
  if (item.catalogId !== null && Object.hasOwn(catalog, item.catalogId))
    return catalog[item.catalogId];
  return item.description;
}

export function statusText(s: ProviderStatus): string {
  const m = messages().status;
  if (s === 'running') return m.running;
  if (s === 'notDetected') return m.notDetected;
  if (s === 'timedOut') return m.timedOut;
  if ('ok' in s) return format(m.found, { count: s.ok.count });
  return s.failed.message;
}

function findingBody(f: Finding, fromFile: boolean): string {
  const m = messages().finding;
  const k = f.kind;
  if (k === 'unreadable') return m.unreadable;
  // An opened Repot file already stores secrets masked; there is no original to allow.
  if ('secret' in k && fromFile)
    return k.secret.name ? format(m.secretStoredNamed, { name: k.secret.name }) : m.secretStored;
  if ('secret' in k)
    return k.secret.name ? format(m.secretNamed, { name: k.secret.name }) : m.secret;
  if ('osSpecificLine' in k) return format(m.osSpecificLine, { key: k.osSpecificLine.key });
  if ('keyFile' in k) return format(m.keyFile, { path: k.keyFile.path });
  if ('externalInclude' in k) return format(m.externalInclude, { path: k.externalInclude.path });
  return format(m.duplicate, { provider: k.duplicate.otherProvider });
}

export function findingText(f: Finding, fromFile = false): string {
  const text = findingBody(f, fromFile);
  // Line numbers are identifiers, so they are not grouped like counts (1234, not 1,234).
  return f.line === null ? text : format(messages().finding.atLine, { line: String(f.line), text });
}

const MANAGER_LABELS: Record<PackageManager, string> = {
  brew: 'Homebrew',
  winget: 'winget',
  scoop: 'Scoop',
  choco: 'Chocolatey',
  apt: 'apt',
  dnf: 'dnf',
  pacman: 'pacman',
};

/** Tool names stay untranslated (spec §11). */
export function managerLabel(m: PackageManager): string {
  return MANAGER_LABELS[m];
}

const JETBRAINS_LABELS: Record<JetBrainsIde, string> = {
  intelliJIdea: 'IntelliJ IDEA',
  ideaIc: 'IntelliJ IDEA CE',
  webStorm: 'WebStorm',
  pyCharm: 'PyCharm',
  pyCharmCe: 'PyCharm CE',
  goLand: 'GoLand',
  rustRover: 'RustRover',
  cLion: 'CLion',
  phpStorm: 'PhpStorm',
  rubyMine: 'RubyMine',
  dataGrip: 'DataGrip',
  rider: 'Rider',
};

/** The JetBrains IDE a plugin or settings file belongs to, so two IDEs' items can be told apart. */
export function ideLabel(item: Item): string | null {
  const d = item.details;
  if ('idePlugin' in d) return JETBRAINS_LABELS[d.idePlugin.ide];
  if (
    'configFile' in d &&
    typeof d.configFile.target === 'object' &&
    'jetBrains' in d.configFile.target
  )
    return JETBRAINS_LABELS[d.configFile.target.jetBrains.ide];
  return null;
}

export function reasonText(r: UnavailableReason): string {
  const m = messages().reason;
  if (r === 'unreadable') return m.unreadable;
  if (r === 'noPackageManager') return m.noPackageManager;
  if ('notInPackageMap' in r)
    return format(m.notInPackageMap, { manager: managerLabel(r.notInPackageMap.manager) });
  if ('noPackageInManager' in r)
    return format(m.noPackageInManager, { manager: managerLabel(r.noPackageInManager.manager) });
  if ('differentOs' in r) return format(m.differentOs, { os: osLabel(r.differentOs.source) });
  if ('shellUnsupported' in r) return format(m.shellUnsupported, { shell: r.shellUnsupported });
  return format(m.noCounterpart, { tool: r.noCounterpart });
}

export function warningText(w: ExportWarning, itemName: string): string {
  const m = messages().warning;
  if ('unavailable' in w)
    return format(m.unavailable, { item: itemName, reason: reasonText(w.unavailable.reason) });
  if ('invalidValue' in w) return format(m.invalidValue, { item: itemName });
  if ('duplicateTarget' in w) return format(m.duplicateTarget, { item: itemName });
  return format(m.readFailed, { item: itemName });
}

// The commands stay as they are: people type them into a terminal.
export function nextSteps(target: Os, exportFormat: Format): string[] {
  const m = messages().nextSteps;
  if (exportFormat === 'manifest') return [m.readme];
  const command =
    target === 'windows' ? 'powershell -ExecutionPolicy Bypass -File restore.ps1' : './restore.sh';
  return [m.copy, format(m.run, { command })];
}

export function originText(o: Origin): string {
  const m = messages().origin;
  return 'command' in o
    ? format(m.command, { command: o.command })
    : format(m.path, { path: o.path });
}

/** Tooltip for the mapped name in the item list. */
export const mappedTo = (manager: PackageManager, name: string): string =>
  format(messages().mappedTo, { manager: managerLabel(manager), name });
export const availableOn = (target: Os): string =>
  format(messages().availableOn, { os: osLabel(target) });
export const withCount = (label: string, n: number): string =>
  format(messages().withCount, { label, count: n });
/** "From a Repot file made on macOS, Nov 4, 2026" for an opened file. */
/**
 * "From a Repot file made on macOS, {date}" split around the date, so the screen can keep
 * the date on one line with `white-space: nowrap`. Non-breaking spaces are not enough:
 * WebKit's `text-wrap: pretty` (the macOS app's engine) still breaks at them.
 */
export function openedFromParts(opened: { source: Os; createdAt: string }): {
  before: string;
  date: string;
  after: string;
} {
  const date = new Date(opened.createdAt);
  const shown = Number.isNaN(date.getTime())
    ? opened.createdAt
    : new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium' }).format(date);
  const [before, after = ''] = format(messages().openedFrom, { os: osLabel(opened.source) }).split(
    '{date}',
  );
  return { before, date: shown, after };
}
export const presenceText = (p: Presence): string => messages().presence[p];
/** The summary's "label count" pairs, so each can stay on one line in the narrow sidebar. */
export const comparedSummaryParts = (presence: PresenceMap): string[] =>
  comparedSummary(presence).split(' · ');
/** "Missing: 1 · Already here: 180 · Different: 2" after comparing with this machine. */
export function comparedSummary(presence: PresenceMap): string {
  const values = Object.values(presence);
  return format(messages().comparedSummary, {
    present: values.filter((p) => p === 'present').length,
    differs: values.filter((p) => p === 'differs').length,
    missing: values.filter((p) => p === 'missing').length,
  });
}
export const skippedItemsText = (count: number): string => plural(messages().skippedItems, count);
/** `open_snapshot` rejects with an error kind; anything else is shown as it is. */
export function openErrorText(e: unknown): string {
  const raw = errorMessage(e);
  const m: Record<string, string> = messages().openError;
  const kind = raw.startsWith('newerVersion:') ? 'newerVersion' : raw;
  return Object.hasOwn(m, kind) ? m[kind] : raw;
}
export const selectedCount = (checked: number, total: number): string =>
  plural(messages().selectedCount, total, { checked, total });
export const selectCategory = (label: string): string =>
  format(messages().selectCategory, { category: label });
export const revealSecretLine = (line: number): string =>
  format(messages().revealSecretLine, { line: String(line) });

type UiText = Readonly<Messages['ui']> & { readonly appTitle: string };

/** Simple labels for the active locale. Each property is a getter, so it follows locale changes. */
export const S: UiText = Object.defineProperties(
  { appTitle: 'Repot' },
  Object.fromEntries(
    Object.keys(messages().ui).map((key) => [
      key,
      { enumerable: true, get: () => messages().ui[key as keyof Messages['ui']] },
    ]),
  ),
) as UiText;
