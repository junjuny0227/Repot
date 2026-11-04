import { expect, test } from 'vitest';
import {
  categoryLabel,
  findingText,
  ideLabel,
  nextSteps,
  osLabel,
  reasonText,
  revealSecretLine,
  S,
  selectedCount,
  statusText,
  TARGET_OPTIONS,
  warningText,
} from '../strings';
import type { Item } from '../types';

test('labels all targets and structured reports', () => {
  expect(TARGET_OPTIONS.map(osLabel)).toEqual([
    'macOS',
    'Windows',
    'Linux (Debian/Ubuntu)',
    'Linux (Fedora)',
    'Linux (Arch)',
  ]);
  expect(categoryLabel('globalPackage')).toBe('Global packages');
  expect(statusText({ ok: { count: 3 } })).toContain('3');
  expect(statusText({ failed: { message: 'failed', detail: 'raw detail' } })).toBe('failed');
  expect(findingText({ line: 2, kind: { secret: { name: 'API_TOKEN' } } })).toContain('API_TOKEN');
  expect(reasonText({ differentOs: { source: 'macOs' } })).toContain('macOS');
  expect(warningText({ readFailed: { itemId: 'git:file' } }, 'Git config')).toContain('Git config');
});
test('next steps use destination format and OS', () => {
  expect(nextSteps('macOs', 'script')).toEqual([
    'Copy this folder to the new machine.',
    'Run: ./restore.sh',
  ]);
  expect(nextSteps('windows', 'script')).toContain(
    'Run: powershell -ExecutionPolicy Bypass -File restore.ps1',
  );
  expect(nextSteps('windows', 'manifest')).toEqual([
    'Open README.md in this folder and follow the steps.',
  ]);
});

test('scan timeout copy matches the 60-second provider limit', () => {
  expect(S.scanIntro).toContain('60 seconds');
  expect(statusText('timedOut')).toBe('Timed out (60 s)');
});

test('counts use the plural form for the total', () => {
  expect(selectedCount(1, 1)).toBe('1 of 1 item selected');
  expect(selectedCount(2, 12)).toBe('2 of 12 items selected');
  expect(revealSecretLine(4)).toBe('Export the original value of line 4');
});

test('open errors map to messages; unknown errors pass through', async () => {
  const { openErrorText, openedFromParts, skippedItemsText } = await import('../strings');
  expect(openErrorText('newerVersion:2')).toBe(
    'This file was made by a newer version of Repot. Update Repot to open it.',
  );
  expect(openErrorText('notRepotFile')).toContain('repot.json');
  expect(openErrorText('readFailed')).toBe('The file couldn’t be read.');
  expect(openErrorText(new Error('boom'))).toBe('boom');
  expect(openErrorText('hasOwnProperty')).toBe('hasOwnProperty');
  expect(openedFromParts({ source: 'macOs', createdAt: '2026-11-04T10:00Z' })).toEqual({
    before: 'From a Repot file made on macOS, ',
    date: 'Nov 4, 2026',
    after: '',
  });
  expect(skippedItemsText(1)).toBe('1 item this version of Repot can’t read was skipped.');
});

test('secrets from an opened Repot file are described as already masked', async () => {
  const { findingText } = await import('../strings');
  const f = { line: 2, kind: { secret: { name: 'API_TOKEN' } } };
  expect(findingText(f, true)).toBe(
    'Line 2: Masked in the Repot file (API_TOKEN). Fill in the value after restoring.',
  );
  expect(findingText(f)).toContain('unless you allow the original value');
});

test('comparison summary counts each presence', async () => {
  const { comparedSummary, presenceText } = await import('../strings');
  expect(comparedSummary({ a: 'missing', b: 'present', c: 'present', d: 'differs' })).toBe(
    'Missing: 1 · Already here: 2 · Different: 1',
  );
  expect(presenceText('present')).toContain('skip');
});

test("every locale places the opened file's date once, so the screen can keep it on one line", async () => {
  for (const code of ['en', 'ko', 'ja', 'zh-Hans']) {
    const messages = (await import(`../locales/${code}.json`)).default as {
      openedFrom: string;
      comparedSummary: string;
    };
    expect(messages.openedFrom.split('{date}'), code).toHaveLength(2);
    expect(messages.comparedSummary.split(' · '), code).toHaveLength(3);
  }
});

test('names the JetBrains IDE of plugins and settings files only', () => {
  const base: Item = {
    id: 'x',
    provider: 'jetbrains',
    category: 'editor',
    name: 'com.github.copilot',
    version: null,
    description: null,
    catalogId: null,
    origin: { path: '/x' },
    portability: 'any',
    details: { idePlugin: { ide: 'dataGrip', folder: 'github-copilot-intellij' } },
    findings: [],
  };
  expect(ideLabel(base)).toBe('DataGrip');
  const target = { jetBrains: { ide: 'cLion' as const, version: '2026.1', rel: 'keymaps/K.xml' } };
  expect(ideLabel({ ...base, details: { configFile: { source: '/x', target } } })).toBe('CLion');
  expect(
    ideLabel({ ...base, details: { configFile: { source: '/x', target: { home: '.zshrc' } } } }),
  ).toBe(null);
  expect(ideLabel({ ...base, details: { extension: { editor: 'cursor' } } })).toBe(null);
});
