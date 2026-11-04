import { LOCALES, type Locale, setLocale, useLocale } from '../i18n';
import { S } from '../strings';
import { UpdateSettings } from '../UpdateSettings';
import { Chevrons, Globe } from './icons';

/** Pop-up button for the UI language. Each language is named in itself, in its own font. */
export function LanguageSelect({ className = '' }: { className?: string }) {
  const locale = useLocale();
  return (
    <div className="app-preferences">
      <label className={`popup-field language-field ${className}`}>
        <Globe />
        <span className="visually-hidden">{S.language}</span>
        <span className="popup">
          <select value={locale} onChange={(e) => setLocale(e.target.value as Locale)}>
            {LOCALES.map((l) => (
              <option key={l.code} value={l.code} lang={l.code}>
                {l.name}
              </option>
            ))}
          </select>
          <Chevrons />
        </span>
      </label>
      <UpdateSettings />
    </div>
  );
}
