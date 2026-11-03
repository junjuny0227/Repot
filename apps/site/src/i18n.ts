// Dependency-free localization for the site: one JSON file per locale in ./locales,
// typed from en.json. Each locale is a static page; the URL, not the browser, picks it.
// Import attributes and extensions let vite.config.ts load this file natively too.
import en from "./locales/en.json" with { type: "json" };
import ja from "./locales/ja.json" with { type: "json" };
import ko from "./locales/ko.json" with { type: "json" };
import zhHans from "./locales/zh-Hans.json" with { type: "json" };

type Widen<T> = T extends string ? string : T extends readonly (infer U)[] ? Widen<U>[] : { [K in keyof T]: Widen<T[K]> };
/** Shape of every locale file. Missing keys in a locale file are a type error. */
export type Messages = Widen<typeof en>;

// Adding a language: add its JSON file and its page (<code>/index.html), import it here, and list it in LOCALES.
const MESSAGES = { en, ko, ja, "zh-Hans": zhHans } satisfies Record<string, Messages>;
export type Locale = keyof typeof MESSAGES;

export const SITE_ORIGIN = "https://junjuny0227.github.io";
export const BASE = "/Repot/";
export const STORAGE_KEY = "repot.site.locale";

/** Each language is named in itself. English is the default page at the site root. */
export const LOCALES: readonly { code: Locale; name: string; path: string }[] = [
  { code: "en", name: "English", path: BASE },
  { code: "ko", name: "한국어", path: `${BASE}ko/` },
  { code: "ja", name: "日本語", path: `${BASE}ja/` },
  { code: "zh-Hans", name: "简体中文", path: `${BASE}zh-Hans/` },
];

export const isLocale = (value: unknown): value is Locale => typeof value === "string" && Object.hasOwn(MESSAGES, value);
export const messagesFor = (locale: Locale): Messages => MESSAGES[locale];
export const localeInfo = (locale: Locale) => LOCALES.find((l) => l.code === locale)!;

/** Supported locale a BCP 47 tag asks for, or null. Chinese maps by script: only Simplified is supported. */
function match(tag: string): Locale | null {
  let locale: Intl.Locale;
  try {
    locale = new Intl.Locale(tag);
  } catch {
    return null; // Not a valid language tag.
  }
  const code = locale.language === "zh" ? (locale.maximize().script === "Hans" ? "zh-Hans" : null) : locale.language;
  return isLocale(code) ? code : null;
}

/** The first browser language the site has, else English. Same rule as the desktop app. */
export function bestLocale(languages: readonly string[]): Locale {
  for (const tag of languages) {
    const code = match(tag);
    if (code !== null) return code;
  }
  return "en";
}

/**
 * Locale to offer on this page, or null. Only the English root page suggests, only before the
 * visitor has chosen or dismissed (anything saved), and never redirects by itself.
 */
export function suggestion(page: Locale, saved: string | null, languages: readonly string[]): Locale | null {
  if (page !== "en" || saved !== null) return null;
  const best = bestLocale(languages);
  return best === "en" ? null : best;
}

/** Remembers the visitor's choice; storage can be unavailable, then the choice lasts for this visit. */
export function rememberLocale(locale: Locale): void {
  try {
    localStorage.setItem(STORAGE_KEY, locale);
  } catch {
    // Private browsing or blocked storage: nothing to remember.
  }
}

/** Fills {name} placeholders. */
export function format(msg: string, vars: Record<string, string>): string {
  return msg.replace(/\{(\w+)\}/g, (whole, name: string) => vars[name] ?? whole);
}

/** "A, B, and C" in the locale's own list style ("A、B、C" in Japanese). */
export const formatList = (locale: Locale, items: string[]): string => new Intl.ListFormat(locale, { type: "conjunction" }).format(items);
