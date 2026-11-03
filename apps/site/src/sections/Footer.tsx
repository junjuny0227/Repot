import appIcon from "../../../desktop/app-icon-macos.svg";
import { useContent } from "../content";
import { Globe } from "../icons";
import { LanguageLinks } from "./LanguageMenu";

export function Footer() {
  const C = useContent();
  return (
    <footer className="site-footer">
      <div className="bar">
        <div className="footer-brand">
          <img src={appIcon} alt="" width={32} height={32} />
          <div>
            <p className="footer-name">{C.name}</p>
            <p>{C.footer.summary}</p>
          </div>
        </div>
        <nav aria-label={C.projectNav}>
          <ul>
            {C.footer.links.map((l) => <li key={l.href}><a href={l.href}>{l.label}</a></li>)}
          </ul>
        </nav>
      </div>
      <div className="bar footer-languages">
        <nav aria-label={C.footer.languages}>
          <Globe />
          <LanguageLinks />
        </nav>
        <a href={C.footer.translate.href}>{C.footer.translate.label}</a>
      </div>
    </footer>
  );
}
