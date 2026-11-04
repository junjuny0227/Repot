// Everything the site shows, for one locale. Sentences come from ./locales/<locale>.json;
// links, product and tool names, and terminal commands stay here because they are not translated.
// Facts come from README.md and the MVP spec only.
import { createContext, useContext } from 'react';
import type { AssetId, DesktopOs } from './downloads';
import { format, formatList, type Locale, messagesFor } from './i18n';
import type { PrincipleKey, ToolKey } from './icons';

const REPO = 'https://github.com/junjuny0227/Repot';

export function content(locale: Locale) {
  const m = messagesFor(locale);
  const principles: PrincipleKey[] = ['readonly', 'secrets', 'network', 'review'];
  const tools: [ToolKey, string][] = [
    [
      'packageManager',
      formatList(locale, ['Homebrew', 'winget', 'Scoop', 'Chocolatey', 'apt', 'dnf', 'pacman']),
    ],
    [
      'runtime',
      formatList(locale, [
        'nvm',
        'nvm-windows',
        'fnm',
        'Volta',
        m.tools.pyenv,
        'rustup',
        'SDKMAN!',
      ]),
    ],
    ['globalPackage', formatList(locale, ['npm', 'pnpm', 'pipx', 'Cargo'])],
    ['editor', m.tools.editor],
    ['git', m.tools.git],
    ['shell', m.tools.shell],
  ];
  return {
    locale,
    // Space between sentences: Japanese and Chinese run sentences together after 。
    gap: locale === 'ja' || locale === 'zh-Hans' ? '' : ' ',
    name: 'Repot',
    tagline: m.tagline,
    skip: m.skip,
    nav: [
      { href: '#how', label: m.nav.how },
      { href: '#principles', label: m.nav.principles },
      { href: '#tools', label: m.nav.tools },
      { href: '#install', label: m.nav.install },
    ],
    github: 'GitHub',
    sectionsNav: m.sectionsNav,
    projectNav: m.projectNav,
    backToTop: m.backToTop,
    language: m.language,

    hero: {
      ...m.hero,
      downloadFor: m.hero.downloadFor as Record<DesktopOs, string>,
      fileHint: m.hero.fileHint as Record<'windows' | 'linux', string>,
      macArm: m.install.files['mac-arm'],
      macIntel: m.install.files['mac-intel'],
      version: (version: string) => format(m.hero.version, { version }),
    },

    how: {
      title: m.how.title,
      lead: m.how.lead,
      steps: [m.how.steps.scan, m.how.steps.select, m.how.steps.export],
      screenshotAlt: m.how.screenshotAlt,
    },

    principles: {
      title: m.principles.title,
      items: principles.map((key) => ({ key, ...m.principles.items[key] })),
    },

    tools: {
      title: m.tools.title,
      groups: tools.map(([key, names]) => ({ key, name: m.tools.groups[key], tools: names })),
      note: m.tools.note,
    },

    install: {
      ...m.install,
      os: { mac: 'macOS', windows: 'Windows', linux: 'Linux' } as Record<DesktopOs, string>,
      downloadFile: (file: string) => format(m.install.downloadFile, { file }),
      files: m.install.files as Record<AssetId, string>,
      steps: m.install.steps as Record<DesktopOs, string[]>,
      macCommand: 'xattr -dr com.apple.quarantine /Applications/Repot.app',
      linuxCommand: 'chmod +x Repot_*.AppImage',
      cliGuideUrl: `${REPO}/blob/main/crates/cli/README${locale === 'en' ? '' : `.${locale}`}.md`,
      cliCommand: {
        mac: 'brew install junjuny0227/tap/repot',
        linux: 'brew install junjuny0227/tap/repot',
        windows:
          'scoop bucket add repot https://github.com/junjuny0227/scoop-bucket\nscoop install repot/repot',
      } as Record<DesktopOs, string>,
    },

    footer: {
      summary: m.footer.summary,
      links: [
        { href: REPO, label: 'GitHub' },
        { href: `${REPO}/releases`, label: m.footer.links.releases },
        { href: `${REPO}/blob/main/LICENSE`, label: m.footer.links.license },
      ],
      languages: m.footer.languages,
      translate: {
        href: `${REPO}/issues/new?title=${encodeURIComponent('Translation: ')}`,
        label: m.footer.translate,
      },
    },
  };
}

export type Content = ReturnType<typeof content>;
export const ContentContext = createContext<Content>(content('en'));
export const useContent = (): Content => useContext(ContentContext);
