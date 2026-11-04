import { useEffect, useRef } from 'react';
import { useContent } from '../content';
import { LOCALES, localeInfo, rememberLocale } from '../i18n';
import { Globe } from '../icons';

/** Links to every language version, each named in itself. Plain links: they work without JS and crawlers follow them. */
export function LanguageLinks() {
  const C = useContent();
  return (
    <ul className="language-links">
      {LOCALES.map((l) => (
        <li key={l.code}>
          <a
            href={l.path}
            hrefLang={l.code}
            lang={l.code}
            aria-current={l.code === C.locale ? 'page' : undefined}
            onClick={() => rememberLocale(l.code)}
          >
            {l.name}
          </a>
        </li>
      ))}
    </ul>
  );
}

/** Header pop-up: a disclosure (works without JS) that closes on Escape or an outside click. */
export function LanguageMenu() {
  const C = useContent();
  const ref = useRef<HTMLDetailsElement>(null);
  useEffect(() => {
    const close = (e: Event) => {
      const menu = ref.current;
      if (!menu?.open) return;
      if (e instanceof KeyboardEvent ? e.key === 'Escape' : !menu.contains(e.target as Node))
        menu.open = false;
    };
    document.addEventListener('click', close);
    document.addEventListener('keydown', close);
    return () => {
      document.removeEventListener('click', close);
      document.removeEventListener('keydown', close);
    };
  }, []);
  return (
    <details className="language-menu" ref={ref}>
      <summary aria-label={`${C.language}: ${localeInfo(C.locale).name}`}>
        <Globe />
        <span lang={C.locale}>{localeInfo(C.locale).name}</span>
      </summary>
      <div className="language-popover">
        <LanguageLinks />
      </div>
    </details>
  );
}
