import { type Locale, localeInfo, messagesFor, rememberLocale } from '../i18n';

/**
 * Offer on the English page, written in the suggested language so the visitor can read it.
 * Choosing or dismissing is remembered; nothing navigates unless the visitor clicks.
 */
export function LocaleSuggestion({ locale, onClose }: { locale: Locale; onClose: () => void }) {
  const m = messagesFor(locale).suggest;
  return (
    <aside className="locale-suggestion" lang={locale} aria-label={m.label}>
      <div className="bar">
        <p>{m.text}</p>
        <div className="suggestion-actions">
          <a
            className="button primary"
            href={localeInfo(locale).path}
            hrefLang={locale}
            onClick={() => rememberLocale(locale)}
          >
            {m.switch}
          </a>
          <button
            type="button"
            className="button"
            onClick={() => {
              rememberLocale('en');
              onClose();
            }}
          >
            {m.dismiss}
          </button>
        </div>
      </div>
    </aside>
  );
}
