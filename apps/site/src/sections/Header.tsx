import appIcon from '../../../desktop/app-icon-macos.svg';
import { useContent } from '../content';
import { LanguageMenu } from './LanguageMenu';

export function Header() {
  const C = useContent();
  return (
    <header className="site-header">
      <a className="skip-link" href="#main">
        {C.skip}
      </a>
      <div className="bar">
        <a className="brand" href="#top" aria-label={C.backToTop}>
          <img src={appIcon} alt="" width={28} height={28} />
          <span>{C.name}</span>
        </a>
        <div className="header-end">
          <nav aria-label={C.sectionsNav}>
            <ul>
              {C.nav.map((n) => (
                <li key={n.href} className="section-link">
                  <a href={n.href}>{n.label}</a>
                </li>
              ))}
              <li>
                <a href="https://github.com/junjuny0227/Repot">{C.github}</a>
              </li>
            </ul>
          </nav>
          <LanguageMenu />
        </div>
      </div>
    </header>
  );
}
