import { afterEach, expect, test } from 'vitest';
import {
  format,
  getLocale,
  LOCALES,
  type PluralMessage,
  plural,
  resolveLocale,
  setLocale,
} from '../i18n';

// Every locale file must match en.json so no screen ever shows a key or a raw placeholder.
const files = import.meta.glob<Record<string, unknown>>('../locales/*.json', {
  eager: true,
  import: 'default',
});
const locales = Object.fromEntries(
  Object.entries(files).map(([path, json]) => [path.match(/([\w-]+)\.json$/)![1], json]),
);
const CATEGORIES = new Set(['zero', 'one', 'two', 'few', 'many', 'other']);

const isPlural = (v: unknown): v is PluralMessage =>
  typeof v === 'object' &&
  v !== null &&
  'other' in v &&
  Object.keys(v).every((k) => CATEGORIES.has(k));

/** Flattens a locale file to "a.b.c" keys; plural objects stay whole. */
function flatten(obj: Record<string, unknown>, prefix = ''): Map<string, string | PluralMessage> {
  const out = new Map<string, string | PluralMessage>();
  for (const [key, value] of Object.entries(obj)) {
    const path = prefix + key;
    if (typeof value === 'string' || isPlural(value)) out.set(path, value);
    else for (const [k, v] of flatten(value as Record<string, unknown>, `${path}.`)) out.set(k, v);
  }
  return out;
}

const placeholders = (s: string) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort();
const en = flatten(locales.en);

afterEach(() => setLocale('en', false));

test('every locale file is registered, and every registered locale has a file', () => {
  expect(Object.keys(locales).sort()).toEqual(LOCALES.map((l) => l.code).sort());
});

test('every locale has exactly the keys of en.json', () => {
  for (const [code, json] of Object.entries(locales)) {
    expect([...flatten(json).keys()].sort(), code).toEqual([...en.keys()].sort());
  }
});

test('placeholders match en.json', () => {
  for (const [code, json] of Object.entries(locales)) {
    for (const [key, value] of flatten(json)) {
      const source = en.get(key)!;
      const expected = placeholders(typeof source === 'string' ? source : source.other);
      if (typeof value === 'string') {
        expect(placeholders(value), `${code} ${key}`).toEqual(expected);
        continue;
      }
      for (const [category, form] of Object.entries(value)) {
        // A singular form may spell the number out ("one item"), so only {count} is optional there.
        const want =
          category === 'other'
            ? expected
            : expected.filter((p) => p !== 'count' || placeholders(form!).includes('count'));
        expect(placeholders(form!), `${code} ${key}.${category}`).toEqual(want);
      }
    }
  }
});

test('plural messages have the categories Intl uses for the locale', () => {
  for (const [code, json] of Object.entries(locales)) {
    const needed = new Intl.PluralRules(code).resolvedOptions().pluralCategories.slice().sort();
    for (const [key, value] of flatten(json)) {
      if (typeof value === 'string') continue;
      expect(Object.keys(value).sort(), `${code} ${key}`).toEqual(needed);
    }
  }
});

test('no empty strings', () => {
  for (const [code, json] of Object.entries(locales)) {
    for (const [key, value] of flatten(json)) {
      for (const text of typeof value === 'string' ? [value] : Object.values(value)) {
        expect(text!.trim(), `${code} ${key}`).not.toBe('');
      }
    }
  }
});

test('resolveLocale prefers the saved choice, then the browser, then English', () => {
  const all = ['en', 'ko', 'ja', 'zh-Hans'];
  const cases: [string | null, string[], string][] = [
    [null, ['ko-KR'], 'ko'],
    [null, ['ko'], 'ko'],
    [null, ['ja-JP'], 'ja'],
    [null, ['zh-CN'], 'zh-Hans'],
    [null, ['zh'], 'zh-Hans'],
    [null, ['zh-SG'], 'zh-Hans'],
    [null, ['zh-Hans-HK'], 'zh-Hans'],
    // Traditional Chinese is not available yet; never show Simplified instead.
    [null, ['zh-TW'], 'en'],
    [null, ['zh-HK'], 'en'],
    [null, ['zh-Hant'], 'en'],
    [null, ['zh-TW', 'ja'], 'ja'],
    [null, ['en-US', 'ko'], 'en'],
    [null, ['fr-FR', 'de'], 'en'],
    [null, ['not a tag', 'ko'], 'ko'],
    [null, [], 'en'],
    ['ja', ['ko-KR'], 'ja'],
    ['xx', ['ko-KR'], 'ko'],
  ];
  for (const [saved, languages, expected] of cases) {
    expect(resolveLocale(saved, languages, all), `${saved} ${languages.join(',')}`).toBe(expected);
  }
});

test('resolveLocale only returns supported locales', () => {
  expect(resolveLocale('ko', ['ko-KR', 'ja'], ['en'])).toBe('en');
});

test('format fills placeholders and formats numbers', () => {
  expect(format('{a} and {b}', { a: 'x', b: 2 })).toBe('x and 2');
  expect(format('{count} items', { count: 12345 })).toBe('12,345 items');
  expect(format('{missing} stays', {})).toBe('{missing} stays');
  expect(format('{line}', { line: String(12345) })).toBe('12345');
});

test('plural picks the CLDR category', () => {
  const msg = { one: '{count} item', other: '{count} items' };
  expect(getLocale()).toBe('en');
  expect(plural(msg, 1)).toBe('1 item');
  expect(plural(msg, 0)).toBe('0 items');
  expect(plural(msg, 2)).toBe('2 items');
  expect(plural({ other: '{count} things' }, 1)).toBe('1 things');
});
