import { expect, test } from 'vitest';
import { content } from '../content';
import { bestLocale, LOCALES, suggestion } from '../i18n';

// Every locale file must match en.json so no page shows a missing or raw placeholder.
const files = import.meta.glob<Record<string, unknown>>('../locales/*.json', {
  eager: true,
  import: 'default',
});
const locales = Object.fromEntries(
  Object.entries(files).map(([path, json]) => [path.match(/([\w-]+)\.json$/)![1], json]),
);

function flatten(obj: unknown, prefix = ''): [string, string][] {
  if (typeof obj === 'string') return [[prefix, obj]];
  return Object.entries(obj as Record<string, unknown>).flatMap(([k, v]) =>
    flatten(v, prefix ? `${prefix}.${k}` : k),
  );
}
const placeholders = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();
const en = new Map(flatten(locales.en));

test('every locale file is registered, and every registered locale has a file', () => {
  expect(Object.keys(locales).sort()).toEqual(LOCALES.map((l) => l.code).sort());
});

test('every locale has exactly the keys of en.json', () => {
  for (const [code, json] of Object.entries(locales)) {
    expect(
      flatten(json)
        .map(([k]) => k)
        .sort(),
      code,
    ).toEqual([...en.keys()].sort());
  }
});

test('placeholders match en.json', () => {
  for (const [code, json] of Object.entries(locales)) {
    for (const [key, value] of flatten(json))
      expect(placeholders(value), `${code} ${key}`).toEqual(placeholders(en.get(key)!));
  }
});

test('no empty strings', () => {
  for (const [code, json] of Object.entries(locales)) {
    for (const [key, value] of flatten(json)) expect(value.trim(), `${code} ${key}`).not.toBe('');
  }
});

// Brand and file-type names that read the same in every language.
const SAME = new Set([
  'tools.groups.git',
  'install.files.mac-intel',
  'install.files.linux-appimage',
]);
const SAME_IN: Record<string, Set<string>> = { 'zh-Hans': new Set(['tools.groups.shell']) };

test('every message is translated', () => {
  for (const [code, json] of Object.entries(locales)) {
    if (code === 'en') continue;
    const same = flatten(json).filter(
      ([key, value]) => en.get(key) === value && !SAME.has(key) && !SAME_IN[code]?.has(key),
    );
    expect(
      same.map(([key]) => key),
      code,
    ).toEqual([]);
  }
});

test('bestLocale follows the desktop rules', () => {
  expect(bestLocale(['ko-KR', 'en'])).toBe('ko');
  expect(bestLocale(['ja'])).toBe('ja');
  expect(bestLocale(['zh-CN'])).toBe('zh-Hans');
  expect(bestLocale(['zh-SG'])).toBe('zh-Hans');
  expect(bestLocale(['zh'])).toBe('zh-Hans');
  expect(bestLocale(['zh-TW'])).toBe('en');
  expect(bestLocale(['zh-HK', 'ja'])).toBe('ja');
  expect(bestLocale(['en-GB', 'ko'])).toBe('en');
  expect(bestLocale(['fr', 'de'])).toBe('en');
  expect(bestLocale([])).toBe('en');
});

test('only the English page suggests, once, and only another language', () => {
  expect(suggestion('en', null, ['ko-KR'])).toBe('ko');
  expect(suggestion('en', null, ['ja-JP', 'en'])).toBe('ja');
  expect(suggestion('en', null, ['en-US', 'ko'])).toBeNull();
  expect(suggestion('en', null, ['zh-TW'])).toBeNull();
  // A saved choice or a dismissal stops suggestions.
  expect(suggestion('en', 'en', ['ko-KR'])).toBeNull();
  expect(suggestion('en', 'ko', ['ko-KR'])).toBeNull();
  // Translated pages never suggest; the visitor got there by choice or by link.
  expect(suggestion('ko', null, ['ja'])).toBeNull();
  expect(suggestion('ja', null, ['en'])).toBeNull();
});

// Keys only (not loaded): the guides that exist in the repository.
const guides = import.meta.glob('../../../../crates/cli/README*.md');

test('each locale links the command-line guide written in its language', () => {
  for (const { code } of LOCALES) {
    const url = content(code).install.cliGuideUrl;
    const file = url.replace('https://github.com/junjuny0227/Repot/blob/main/', '');
    expect(file).toBe(code === 'en' ? 'crates/cli/README.md' : `crates/cli/README.${code}.md`);
    expect(Object.keys(guides), file).toContain(`../../../../${file}`);
  }
});
