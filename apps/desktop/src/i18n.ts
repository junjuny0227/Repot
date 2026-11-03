// Dependency-free localization: one JSON file per locale in ./locales, typed from en.json.
// Placeholders are {name}; plurals are objects keyed by Intl.PluralRules categories.
import { useSyncExternalStore } from "react";
import en from "./locales/en.json";
import ja from "./locales/ja.json";
import ko from "./locales/ko.json";
import zhHans from "./locales/zh-Hans.json";

/** Plural message: one entry per CLDR category the locale uses; `other` is always required. */
export type PluralMessage = { [K in Intl.LDMLPluralRule]?: string } & { other: string };
type Widen<T> = T extends string ? string : T extends { other: string } ? PluralMessage : { [K in keyof T]: Widen<T[K]> };
/** Shape of every locale file. Missing keys in a locale file are a type error. */
export type Messages = Widen<typeof en>;

// Adding a language: add its JSON file, import it here, and list it in LOCALES.
const MESSAGES = { en, ko, ja, "zh-Hans": zhHans } satisfies Record<string, Messages>;
export type Locale = keyof typeof MESSAGES;

/** Each language is named in its own language so people can find theirs. */
export const LOCALES: readonly { code: Locale; name: string }[] = [
  { code: "en", name: "English" },
  { code: "ko", name: "한국어" },
  { code: "ja", name: "日本語" },
  { code: "zh-Hans", name: "简体中文" },
];

export const STORAGE_KEY = "repot.locale";
const isLocale = (value: unknown): value is Locale => typeof value === "string" && value in MESSAGES;

/** Supported locale a BCP 47 tag asks for, or null. Chinese maps by script: only Simplified is supported. */
function match(tag: string, supported: readonly string[]): string | null {
  let locale: Intl.Locale;
  try {
    locale = new Intl.Locale(tag);
  } catch {
    return null; // Not a valid language tag.
  }
  const code = locale.language === "zh" ? (locale.maximize().script === "Hans" ? "zh-Hans" : null) : locale.language;
  return code !== null && supported.includes(code) ? code : null;
}

/** Saved choice, then the first browser language we support, then English. */
export function resolveLocale(saved: string | null, languages: readonly string[], supported: readonly string[] = Object.keys(MESSAGES)): string {
  if (saved !== null && supported.includes(saved)) return saved;
  for (const tag of languages) {
    const code = match(tag, supported);
    if (code !== null) return code;
  }
  return "en";
}

let active: Locale = "en";
const listeners = new Set<() => void>();

export const getLocale = (): Locale => active;
export const messages = (): Messages => MESSAGES[active];

export function setLocale(locale: Locale, persist = true): void {
  active = locale;
  if (typeof document !== "undefined") document.documentElement.lang = locale;
  if (persist) {
    try {
      localStorage.setItem(STORAGE_KEY, locale);
    } catch {
      // Storage can be unavailable; the choice then lasts for this session only.
    }
  }
  for (const listener of listeners) listener();
}

/** Locale to start with: the saved choice, else the browser's languages. */
export function initialLocale(): Locale {
  let saved: string | null = null;
  try {
    saved = localStorage.getItem(STORAGE_KEY);
  } catch {
    // Unavailable storage behaves like no saved choice.
  }
  const code = resolveLocale(saved, typeof navigator === "undefined" ? [] : navigator.languages);
  return isLocale(code) ? code : "en";
}

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Re-renders the caller when the locale changes. */
export const useLocale = (): Locale => useSyncExternalStore(subscribe, getLocale, getLocale);

type Vars = Record<string, string | number>;

/** Fills {name} placeholders. Numbers are formatted for the active locale; unknown names stay as they are. */
export function format(msg: string, vars: Vars = {}): string {
  return msg.replace(/\{(\w+)\}/g, (whole, name: string) => {
    const value = vars[name];
    if (value === undefined) return whole;
    return typeof value === "number" ? new Intl.NumberFormat(active).format(value) : value;
  });
}

/** Picks the plural form for `count` with Intl.PluralRules, then fills placeholders ({count} included). */
export function plural(msg: PluralMessage, count: number, vars: Vars = {}): string {
  const category = new Intl.PluralRules(active).select(count);
  return format(msg[category] ?? msg.other, { count, ...vars });
}
