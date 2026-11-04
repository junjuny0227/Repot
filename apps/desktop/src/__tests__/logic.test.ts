import { expect, test } from 'vitest';
import type { AvailabilityMap, ScanResult } from '../types';

const fixture = (await import('../__fixtures__/scan.json')).default;
const scan = fixture.scan as unknown as ScanResult;
const availMac = fixture.availability.macOs as unknown as AvailabilityMap;
const availWin = fixture.availability.windows as unknown as AvailabilityMap;

import {
  applyTarget,
  buildSelection,
  categoryCounts,
  choosesPackages,
  currentTarget,
  defaultTarget,
  fileTree,
  filterItems,
  initialChecked,
  isAvailable,
  osKey,
  revealKey,
  secretLines,
  setCategory,
  targetOptions,
  WINDOWS_PACKAGE_MANAGERS,
  warningItemId,
} from '../logic';
import { TARGET_OPTIONS } from '../strings';

test('initialChecked selects only available items', () => {
  const checked = initialChecked(scan.items, availWin);
  for (const item of scan.items) expect(checked.has(item.id)).toBe(isAvailable(availWin[item.id]));
});

test('applyTarget unchecks newly unavailable items', () => {
  const mac = initialChecked(scan.items, availMac);
  const after = applyTarget(mac, availWin);
  expect([...after].every((id) => isAvailable(availWin[id]))).toBe(true);
  expect(after.size).toBeLessThan(mac.size);
});

test('selection_drops_reveals_of_unchecked_items', () => {
  // Review Focus 2
  const zshrc = scan.items.find((i) => i.id === 'shell:file:.zshrc')!;
  const line = secretLines(zshrc)[0];
  const revealed = new Set([revealKey(zshrc.id, line)]);
  expect(buildSelection(new Set([zshrc.id]), revealed).revealSecretLines).toEqual([
    [zshrc.id, line],
  ]);
  expect(buildSelection(new Set(), revealed)).toEqual({ items: [], revealSecretLines: [] });
});

test('filterItems by query and warnings', () => {
  const all = { query: '', onlyAvailable: false, onlyWarnings: false, onlyMissing: false };
  expect(
    filterItems(scan.items, availMac, { ...all, onlyWarnings: true }).map((i) => i.id),
  ).toContain('shell:file:.zshrc');
  expect(filterItems(scan.items, availMac, { ...all, query: 'ZSHRC' }).length).toBeGreaterThan(0);
});

test('setCategory toggles only available items of that category', () => {
  const none = setCategory(
    initialChecked(scan.items, availWin),
    scan.items,
    availWin,
    'packageManager',
    false,
  );
  expect(
    scan.items.filter((i) => i.category === 'packageManager').some((i) => none.has(i.id)),
  ).toBe(false);
});

test('setCategory on never checks unavailable items and counts follow', () => {
  const on = setCategory(new Set(), scan.items, availWin, 'shell', true);
  expect(on.has('shell:file:.zshrc')).toBe(false);
  const counts = categoryCounts(scan.items, initialChecked(scan.items, availMac));
  expect(counts.shell).toEqual({ total: 1, checked: 1 });
  expect(counts.runtime).toEqual({ total: 0, checked: 0 });
});

test('osKey and targetOptions keep the current target selectable', () => {
  expect(osKey({ linux: 'arch' })).toBe(osKey({ linux: 'arch' }));
  expect(osKey('macOs')).not.toBe(osKey('windows'));
  expect(targetOptions('macOs')).toEqual(TARGET_OPTIONS);
  expect(targetOptions(null)).toEqual(TARGET_OPTIONS);
  expect(targetOptions({ linux: 'other' })).toEqual([...TARGET_OPTIONS, { linux: 'other' }]);
});

test('fileTree keeps top-level files and folds nested ones into folders', () => {
  const f = (path: string) => ({ path, contents: '', executable: false });
  const tree = fileTree([
    f('restore.sh'),
    f('files/.zshrc'),
    f('files/vscode/settings.json'),
    f('README.md'),
  ]);
  expect(tree.files.map((x) => x.path)).toEqual(['restore.sh', 'README.md']);
  expect(tree.folders).toEqual([
    { name: 'files', files: [f('files/.zshrc'), f('files/vscode/settings.json')] },
  ]);
});

test('warningItemId reads the item of every warning kind', () => {
  expect(warningItemId({ unavailable: { itemId: 'a', reason: 'unreadable' } })).toBe('a');
  expect(warningItemId({ invalidValue: { itemId: 'b' } })).toBe('b');
  expect(warningItemId({ duplicateTarget: { itemId: 'c' } })).toBe('c');
  expect(warningItemId({ readFailed: { itemId: 'd' } })).toBe('d');
});

test("defaultTarget picks the destination's package manager", () => {
  expect(defaultTarget('macOs')).toEqual({ os: 'macOs', packages: 'brew' });
  expect(defaultTarget('windows')).toEqual({ os: 'windows', packages: 'winget' });
  expect(defaultTarget({ linux: 'debian' })).toEqual({ os: { linux: 'debian' }, packages: 'apt' });
  expect(defaultTarget({ linux: 'arch' })).toEqual({ os: { linux: 'arch' }, packages: 'pacman' });
  expect(defaultTarget({ linux: 'other' })).toEqual({ os: { linux: 'other' }, packages: null });
});

test('the package manager is chosen only for Windows from another OS', () => {
  expect(WINDOWS_PACKAGE_MANAGERS).toEqual(['winget', 'scoop', 'choco']);
  expect(choosesPackages('macOs', 'windows')).toBe(true);
  expect(choosesPackages({ linux: 'fedora' }, 'windows')).toBe(true);
  expect(choosesPackages('windows', 'windows')).toBe(false);
  expect(choosesPackages('macOs', 'macOs')).toBe(false);
  expect(choosesPackages('macOs', { linux: 'debian' })).toBe(false);
  expect(choosesPackages(null, 'windows')).toBe(false);
});

test('currentTarget pairs the destination with the chosen manager', () => {
  expect(currentTarget({ target: null, packages: null })).toBeNull();
  expect(currentTarget({ target: 'windows', packages: 'scoop' })).toEqual({
    os: 'windows',
    packages: 'scoop',
  });
});

test('missing-only selection and filter', async () => {
  const { missingChecked } = await import('../logic');
  const [a, b, c] = scan.items.map((i) => i.id).filter((id) => isAvailable(availMac[id]));
  const presence = { [a]: 'missing', [b]: 'present', [c]: 'differs' } as const;
  expect([...missingChecked(scan.items, availMac, presence)]).toEqual([a]);
  const all = { query: '', onlyAvailable: false, onlyWarnings: false, onlyMissing: true };
  expect(filterItems(scan.items, availMac, all, presence).map((i) => i.id)).toEqual([a]);
});
