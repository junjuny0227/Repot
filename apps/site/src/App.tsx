import { useEffect, useMemo, useState } from 'react';
import { ContentContext, content } from './content';
import { detectPlatform, macArchPreference, type NavigatorLike, type Platform } from './downloads';
import { type Locale, STORAGE_KEY, suggestion } from './i18n';
import { Footer } from './sections/Footer';
import { Header } from './sections/Header';
import { Hero } from './sections/Hero';
import { HowItWorks } from './sections/HowItWorks';
import { Install } from './sections/Install';
import { LocaleSuggestion } from './sections/LocaleSuggestion';
import { Principles } from './sections/Principles';
import { Tools } from './sections/Tools';

type HintNavigator = NavigatorLike & {
  languages?: readonly string[];
  userAgentData?: NavigatorLike['userAgentData'] & {
    getHighEntropyValues?: (hints: string[]) => Promise<{ architecture?: string }>;
  };
};

function savedLocale(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY);
  } catch {
    return null; // Storage unavailable: behave as if nothing was chosen.
  }
}

export default function App({
  locale = 'en',
  nav = globalThis.navigator as HintNavigator | undefined,
}: {
  locale?: Locale;
  nav?: HintNavigator;
}) {
  const c = useMemo(() => content(locale), [locale]);
  const [platform] = useState<Platform>(() => (nav ? detectPlatform(nav) : 'unknown'));
  const [macArch, setMacArch] = useState<'mac-arm' | 'mac-intel'>('mac-arm');
  // Decided after the first paint: the static HTML never depends on the visitor's browser languages.
  const [suggested, setSuggested] = useState<Locale | null>(null);

  useEffect(() => {
    if (platform !== 'mac') return;
    let live = true;
    nav?.userAgentData
      ?.getHighEntropyValues?.(['architecture'])
      .then((v) => live && setMacArch(macArchPreference(v.architecture)))
      .catch(() => {}); // The hint is optional; keep the Apple silicon default.
    return () => {
      live = false;
    };
  }, [nav, platform]);

  useEffect(() => {
    setSuggested(suggestion(locale, savedLocale(), nav?.languages ?? []));
  }, [locale, nav]);

  return (
    <ContentContext.Provider value={c}>
      <Header />
      {suggested !== null && (
        <LocaleSuggestion locale={suggested} onClose={() => setSuggested(null)} />
      )}
      <main id="main">
        <Hero platform={platform} macArch={macArch} />
        <HowItWorks />
        <Principles />
        <Tools />
        <Install platform={platform} />
      </main>
      <Footer />
    </ContentContext.Provider>
  );
}
