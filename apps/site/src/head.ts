// <head> tags for each locale page, filled in at build time by the plugin in vite.config.ts,
// so titles and descriptions have one source: the locale JSON files.
import { LOCALES, type Locale, localeInfo, messagesFor, SITE_ORIGIN } from './i18n.ts';

const escape = (s: string) =>
  s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
export const pageUrl = (locale: Locale): string => `${SITE_ORIGIN}${localeInfo(locale).path}`;

/** Title, description, canonical URL, and hreflang alternates for every locale plus x-default (English). */
export function headTags(locale: Locale): string {
  const { meta } = messagesFor(locale);
  return [
    `<title>${escape(meta.title)}</title>`,
    `<meta name="description" content="${escape(meta.description)}" />`,
    `<link rel="canonical" href="${pageUrl(locale)}" />`,
    ...LOCALES.map(
      (l) => `<link rel="alternate" hreflang="${l.code}" href="${pageUrl(l.code)}" />`,
    ),
    `<link rel="alternate" hreflang="x-default" href="${pageUrl('en')}" />`,
  ].join('\n    ');
}

/** Fallback for visitors without JavaScript. */
export function noscript(locale: Locale): string {
  const { meta } = messagesFor(locale);
  return `<noscript>
      <p style="font-family: system-ui, sans-serif; padding: 24px">
        ${escape(meta.noscript)}
        <a href="https://github.com/junjuny0227/Repot/releases">${escape(meta.noscriptLink)}</a>
      </p>
    </noscript>`;
}
