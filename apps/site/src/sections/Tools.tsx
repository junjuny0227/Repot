import { C } from "../content";
import { SymbolTile, type ToolKey } from "../icons";

export function Tools() {
  return (
    <section className="section" id="tools" aria-labelledby="tools-title">
      <div className="section-head">
        <h2 id="tools-title">{C.tools.title}</h2>
      </div>
      <dl className="group tools">
        {C.tools.groups.map((g) => (
          <div key={g.key} className="tool-row">
            <dt><SymbolTile name={g.key as ToolKey} />{g.name}</dt>
            <dd>{g.tools}</dd>
          </div>
        ))}
      </dl>
      <p className="footnote">{C.tools.note}</p>
    </section>
  );
}
