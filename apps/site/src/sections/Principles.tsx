import { useContent } from "../content";
import { SymbolTile, type PrincipleKey } from "../icons";

export function Principles() {
  const C = useContent();
  return (
    <section className="section" id="principles" aria-labelledby="principles-title">
      <div className="section-head">
        <h2 id="principles-title">{C.principles.title}</h2>
      </div>
      <ul className="principles">
        {C.principles.items.map((p) => (
          <li key={p.key} className="group principle">
            <SymbolTile name={p.key as PrincipleKey} />
            <h3>{p.title}</h3>
            <p>{p.text}</p>
          </li>
        ))}
      </ul>
    </section>
  );
}
