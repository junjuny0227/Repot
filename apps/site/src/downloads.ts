// Download links are fixed at build time from the app version, so the page stays
// static and never calls the GitHub API. Links stay pinned to this version (asset
// names contain it), so the page never points at a release it was not built for.
import { version } from '../../desktop/src-tauri/tauri.conf.json';
import assets from './assets.json';

export type Platform = 'mac' | 'windows' | 'linux' | 'unknown';
export type DesktopOs = Exclude<Platform, 'unknown'>;
export type AssetId =
  | 'mac-arm'
  | 'mac-intel'
  | 'windows-setup'
  | 'windows-msi'
  | 'linux-appimage'
  | 'linux-deb'
  | 'linux-rpm';

export interface NavigatorLike {
  userAgent: string;
  maxTouchPoints?: number;
  userAgentData?: { platform?: string; mobile?: boolean };
}

export interface Download {
  id: AssetId;
  os: DesktopOs;
  file: string;
  url: string;
}

const REPO = 'https://github.com/junjuny0227/Repot';

export const VERSION: string = version;
export const RELEASE_URL = `${REPO}/releases/tag/v${VERSION}`;
export const RELEASES_URL = `${REPO}/releases`;

export function detectPlatform(nav: NavigatorLike): Platform {
  const hint = nav.userAgentData;
  if (hint?.platform) {
    if (hint.mobile) return 'unknown';
    if (hint.platform === 'macOS') return 'mac';
    if (hint.platform === 'Windows') return 'windows';
    if (hint.platform === 'Linux') return 'linux';
    return 'unknown';
  }
  const ua = nav.userAgent;
  // Phones and tablets cannot run the desktop app. Android says "Linux" and
  // iPadOS Safari says "Macintosh", so rule them out before the desktop checks.
  if (/Android|iPhone|iPad|iPod|Mobile/.test(ua)) return 'unknown';
  if (/Macintosh|Mac OS X/.test(ua)) return (nav.maxTouchPoints ?? 0) > 1 ? 'unknown' : 'mac';
  if (/Windows/.test(ua)) return 'windows';
  if (/CrOS/.test(ua)) return 'unknown';
  if (/Linux|X11/.test(ua)) return 'linux';
  return 'unknown';
}

// Browsers cannot tell Apple silicon from Intel reliably; Chromium's architecture
// hint is the only signal, and Apple silicon is the safer default for new Macs.
export function macArchPreference(architecture?: string): 'mac-arm' | 'mac-intel' {
  return architecture === 'x86' ? 'mac-intel' : 'mac-arm';
}

export function assetUrl(file: string, v: string = VERSION): string {
  return `${REPO}/releases/download/v${v}/${file.replaceAll('{version}', v)}`;
}

/** Downloads for one OS, recommended file first (assets.json order). */
export function downloadsFor(os: DesktopOs, v: string = VERSION): Download[] {
  return assets
    .filter((a) => a.os === os)
    .map((a) => ({
      id: a.id as AssetId,
      os,
      file: a.file.replaceAll('{version}', v),
      url: assetUrl(a.file, v),
    }));
}
