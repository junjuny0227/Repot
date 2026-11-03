import appIcon from "../../../desktop/app-icon-macos.svg";
import { C } from "../content";

export function Footer() {
  return (
    <footer className="site-footer">
      <div className="bar">
        <div className="footer-brand">
          <img src={appIcon} alt="" width={32} height={32} />
          <div>
            <p className="footer-name">{C.name}</p>
            <p>{C.footer.summary} {C.footer.license}.</p>
          </div>
        </div>
        <nav aria-label={C.projectNav}>
          <ul>
            {C.footer.links.map((l) => <li key={l.href}><a href={l.href}>{l.label}</a></li>)}
          </ul>
        </nav>
      </div>
    </footer>
  );
}
