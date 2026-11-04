import { expect, test } from 'vitest';
import { version as appVersion } from '../../../desktop/src-tauri/tauri.conf.json';
import {
  assetUrl,
  detectPlatform,
  downloadsFor,
  macArchPreference,
  RELEASE_URL,
  VERSION,
} from '../downloads';

const UA = {
  macSafari:
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.1 Safari/605.1.15',
  macChrome:
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36',
  winEdge:
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0',
  winFirefox: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0',
  linuxFirefox: 'Mozilla/5.0 (X11; Linux x86_64; rv:145.0) Gecko/20100101 Firefox/145.0',
  linuxChrome:
    'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36',
  android:
    'Mozilla/5.0 (Linux; Android 10; K) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Mobile Safari/537.36',
  iphone:
    'Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/26.1 Mobile/15E148 Safari/604.1',
};

test('detects macOS, Windows and Linux from user agents', () => {
  expect(detectPlatform({ userAgent: UA.macSafari, maxTouchPoints: 0 })).toBe('mac');
  expect(detectPlatform({ userAgent: UA.macChrome })).toBe('mac');
  expect(detectPlatform({ userAgent: UA.winEdge })).toBe('windows');
  expect(detectPlatform({ userAgent: UA.winFirefox })).toBe('windows');
  expect(detectPlatform({ userAgent: UA.linuxFirefox })).toBe('linux');
  expect(detectPlatform({ userAgent: UA.linuxChrome })).toBe('linux');
  expect(detectPlatform({ userAgent: 'Mozilla/5.0 (X11; CrOS x86_64 14541.0.0)' })).toBe('unknown');
});

test('prefers userAgentData.platform when present', () => {
  // Chromium freezes the UA string, so the client hint is the better source.
  expect(
    detectPlatform({
      userAgent: UA.macChrome,
      userAgentData: { platform: 'Windows', mobile: false },
    }),
  ).toBe('windows');
  expect(
    detectPlatform({ userAgent: UA.winEdge, userAgentData: { platform: 'macOS', mobile: false } }),
  ).toBe('mac');
  expect(
    detectPlatform({ userAgent: UA.winEdge, userAgentData: { platform: 'Linux', mobile: false } }),
  ).toBe('linux');
  expect(
    detectPlatform({ userAgent: UA.android, userAgentData: { platform: 'Android', mobile: true } }),
  ).toBe('unknown');
  expect(
    detectPlatform({
      userAgent: UA.linuxChrome,
      userAgentData: { platform: 'Chrome OS', mobile: false },
    }),
  ).toBe('unknown');
});

test('iPad and Android are not desktop platforms', () => {
  // iPadOS Safari reports itself as a Mac; touch support gives it away.
  expect(detectPlatform({ userAgent: UA.macSafari, maxTouchPoints: 5 })).toBe('unknown');
  expect(detectPlatform({ userAgent: UA.iphone, maxTouchPoints: 5 })).toBe('unknown');
  expect(detectPlatform({ userAgent: UA.android, maxTouchPoints: 5 })).toBe('unknown');
});

test('macArchPreference defaults to Apple silicon and honors x86', () => {
  expect(macArchPreference()).toBe('mac-arm');
  expect(macArchPreference('arm')).toBe('mac-arm');
  expect(macArchPreference('')).toBe('mac-arm');
  expect(macArchPreference('x86')).toBe('mac-intel');
});

test('asset URLs match the v0.1.0 release files', () => {
  const files = (['mac', 'windows', 'linux'] as const)
    .flatMap((os) => downloadsFor(os, '0.1.0'))
    .map((d) => d.url);
  const base = 'https://github.com/junjuny0227/Repot/releases/download/v0.1.0/';
  expect(files.sort()).toEqual(
    [
      'Repot_0.1.0_aarch64.dmg',
      'Repot_0.1.0_x64.dmg',
      'Repot_0.1.0_x64-setup.exe',
      'Repot_0.1.0_x64_en-US.msi',
      'Repot_0.1.0_amd64.AppImage',
      'Repot_0.1.0_amd64.deb',
      'Repot-0.1.0-1.x86_64.rpm',
    ]
      .map((f) => base + f)
      .sort(),
  );
  expect(assetUrl('Repot_{version}_x64.dmg', '1.2.3')).toBe(
    'https://github.com/junjuny0227/Repot/releases/download/v1.2.3/Repot_1.2.3_x64.dmg',
  );
});

test('VERSION comes from tauri.conf.json', () => {
  expect(VERSION).toBe(appVersion);
  expect(RELEASE_URL).toBe(`https://github.com/junjuny0227/Repot/releases/tag/v${appVersion}`);
  expect(downloadsFor('windows')[0].url).toContain(`/v${appVersion}/Repot_${appVersion}_`);
});

test('the first download is the recommended one', () => {
  expect(downloadsFor('mac')[0].id).toBe('mac-arm');
  expect(downloadsFor('windows')[0].file).toMatch(/-setup\.exe$/);
  expect(downloadsFor('linux')[0].file).toMatch(/\.AppImage$/);
});
