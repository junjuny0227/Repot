import { expect, test } from 'vitest';
import { headTags, noscript } from '../head';
import { LOCALES, messagesFor } from '../i18n';

// The locale pages that Vite builds (index.html, ko/index.html, …).
const pages = import.meta.glob<string>(
  ['../../index.html', '../../*/index.html', '!../../{dist,node_modules,public}/**'],
  { query: '?raw', import: 'default', eager: true },
);

const SITE = 'https://junjuny0227.github.io/Repot/';

test('there is one page per locale, with its lang and the head and noscript slots', () => {
  const byLang = Object.entries(pages).map(([path, html]) => [
    path,
    html.match(/<html lang="([^"]+)"/)?.[1],
  ]);
  expect(byLang.map(([, lang]) => lang).sort()).toEqual(LOCALES.map((l) => l.code).sort());
  for (const [path, lang] of byLang) {
    expect(path, String(lang)).toBe(
      lang === 'en' ? '../../index.html' : `../../${lang}/index.html`,
    );
    expect(pages[path!]).toContain('<!-- repot:head -->');
    expect(pages[path!]).toContain('<!-- repot:noscript -->');
  }
});

test('each page has its translated title and description, its canonical URL and all alternates', () => {
  for (const { code, path } of LOCALES) {
    const head = headTags(code);
    const { meta } = messagesFor(code);
    expect(head).toContain(`<title>${meta.title}</title>`);
    expect(head).toContain(`<meta name="description" content="${meta.description}" />`);
    expect(head).toContain(`<link rel="canonical" href="https://junjuny0227.github.io${path}" />`);
    for (const other of LOCALES)
      expect(head).toContain(
        `<link rel="alternate" hreflang="${other.code}" href="https://junjuny0227.github.io${other.path}" />`,
      );
    expect(noscript(code)).toContain(meta.noscriptLink);
  }
});

test('x-default points at the English page and the URLs are the published ones', () => {
  expect(headTags('ko')).toContain(`<link rel="alternate" hreflang="x-default" href="${SITE}" />`);
  expect(LOCALES.map((l) => l.path)).toEqual([
    '/Repot/',
    '/Repot/ko/',
    '/Repot/ja/',
    '/Repot/zh-Hans/',
  ]);
});

test('head text is escaped', () => {
  expect(headTags('en')).not.toMatch(/content="[^"]*<[^"]*"/);
});
