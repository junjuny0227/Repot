import { renderToStaticMarkup } from 'react-dom/server';
import { expect, test } from 'vitest';
import App from '../App';
import { type NavigatorLike, VERSION } from '../downloads';
import { LOCALES, type Locale } from '../i18n';
import { LocaleSuggestion } from '../sections/LocaleSuggestion';

const render = (nav: NavigatorLike, locale: Locale = 'en') =>
  renderToStaticMarkup(<App nav={nav} locale={locale} />);
const codes = LOCALES.map((l) => l.code);
const mac = {
  userAgent:
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.1 Safari/605.1.15',
  maxTouchPoints: 0,
};
const windows = {
  userAgent: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0',
};
const iphone = {
  userAgent:
    'Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148',
};

test('one h1, and every section is titled by an h2, in every language', () => {
  for (const code of codes) {
    const html = render(mac, code);
    expect(html.match(/<h1[\s>]/g), code).toHaveLength(1);
    const sections = html.match(/<section[^>]*aria-labelledby="([^"]+)"/g) ?? [];
    expect(sections.length, code).toBe(5);
    for (const s of sections) {
      const id = s.match(/aria-labelledby="([^"]+)"/)![1];
      expect(html).toMatch(new RegExp(`<h[12] id="${id}"`));
    }
  }
});

test('every image has alt text', () => {
  for (const code of codes) {
    const imgs = render(mac, code).match(/<img[^>]*>/g) ?? [];
    expect(imgs.length).toBeGreaterThan(0);
    for (const img of imgs) expect(img).toMatch(/ alt="/);
  }
});

test("download links all point at the current version's release assets, in every language", () => {
  for (const code of codes) {
    for (const nav of [mac, windows, iphone]) {
      const hrefs = [...render(nav, code).matchAll(/href="([^"]*releases\/download[^"]*)"/g)].map(
        (m) => m[1],
      );
      expect(hrefs.length).toBeGreaterThan(0);
      for (const h of hrefs)
        expect(
          h.startsWith(`https://github.com/junjuny0227/Repot/releases/download/v${VERSION}/`),
          `${code} ${h}`,
        ).toBe(true);
    }
  }
});

test("the hero offers the detected platform's installer", () => {
  for (const code of codes) {
    expect(render(mac, code)).toContain(`v${VERSION}/Repot_${VERSION}_aarch64.dmg`);
    const win = render(windows, code);
    const hero = win.slice(0, win.indexOf('id="how"'));
    expect(hero).toContain('-setup.exe');
    expect(hero).not.toContain('.dmg');
    // Phones get pointed at the download list instead of a file.
    const phone = render(iphone, code);
    expect(phone.slice(0, phone.indexOf('id="how"'))).not.toContain('releases/download');
  }
});

test('the hero shows the CLI install command for the detected platform, with a copy button', () => {
  const hero = (html: string) => html.slice(0, html.indexOf('id="how"'));
  for (const code of codes) {
    const onMac = hero(render(mac, code));
    expect(onMac).toContain('brew install junjuny0227/tap/repot');
    expect(onMac).toMatch(/<pre><code>brew install[^<]*<\/code><\/pre><button type="button">/);
    const onWindows = hero(render(windows, code));
    expect(onWindows).toContain('scoop install repot/repot');
    expect(onWindows).not.toContain('brew install');
    // No package manager to suggest on a phone.
    expect(hero(render(iphone, code))).not.toContain('class="command"');
  }
});

test('every command on the page has a copy button', () => {
  const html = render(mac);
  const commands = html.match(/<pre>/g)?.length ?? 0;
  expect(commands).toBeGreaterThan(0);
  expect(
    html.match(/<div class="command"><pre>[\s\S]*?<\/pre><button type="button">/g),
  ).toHaveLength(commands);
});

test('every page links to every language, marks its own, and asks for translation help', () => {
  for (const { code } of LOCALES) {
    const html = render(mac, code);
    for (const other of LOCALES)
      expect(html).toContain(`href="${other.path}" hrefLang="${other.code}" lang="${other.code}"`);
    expect(
      html.match(new RegExp(`hrefLang="${code}" lang="${code}" aria-current="page"`, 'g')),
    ).toHaveLength(2); // header menu and footer
    expect(html).toContain(
      'href="https://github.com/junjuny0227/Repot/issues/new?title=Translation%3A%20"',
    );
  }
});

test('pages render their own language and keep commands untranslated', () => {
  expect(render(mac, 'ko')).toContain('설치와 첫 실행');
  expect(render(mac, 'ja')).toContain('インストールと初回起動');
  expect(render(mac, 'zh-Hans')).toContain('安装与首次启动');
  for (const code of codes)
    expect(render(mac, code)).toContain('xattr -dr com.apple.quarantine /Applications/Repot.app');
  expect(render(mac, 'ja')).toContain('Homebrew、winget、Scoop');
});

test('no suggestion is part of the static page; the banner speaks the suggested language', () => {
  // The suggestion depends on the browser, so it appears only after the first client render.
  expect(render(mac, 'en')).not.toContain('locale-suggestion');
  const banner = renderToStaticMarkup(<LocaleSuggestion locale="ko" onClose={() => {}} />);
  expect(banner).toContain('lang="ko"');
  expect(banner).toContain('이 페이지는 한국어로도 볼 수 있습니다.');
  expect(banner).toContain('href="/Repot/ko/"');
  expect(banner).toContain('<button');
});
