import type { Dispatch } from "react";
import { revealKey } from "../../logic";
import type { Action, State } from "../../state";
import { S, availableOn, findingText, originText, reasonText, revealLine } from "../../strings";
import type { Item } from "../../types";

export function DetailPanel({ item, state, dispatch }: { item: Item | null; state: State; dispatch: Dispatch<Action> }) {
  if (item === null) return <aside className="detail card muted">{S.selectItem}</aside>;
  const a = state.avail[item.id];
  const checked = state.checked.has(item.id);
  const hasSecret = item.findings.some((f) => typeof f.kind === "object" && "secret" in f.kind);
  return (
    <aside className="detail card">
      <h2>{item.name} {item.version !== null && <span className="item-version">{item.version}</span>}</h2>
      <section>
        <h3>{S.description}</h3>
        <p className={item.description === null ? "muted" : ""}>{item.description ?? S.noDescription}</p>
      </section>
      <section>
        <h3>{S.origin}</h3>
        <p className="mono">{originText(item.origin)}</p>
      </section>
      <section>
        <h3>{S.availability}</h3>
        <p>{a === undefined || a === "available" ? (state.target ? availableOn(state.target) : S.available) : reasonText(a.unavailable)}</p>
      </section>
      <section>
        <h3>{S.findings}</h3>
        {item.findings.length === 0 ? (
          <p className="muted">{S.noFindings}</p>
        ) : (
          <ul className="findings">
            {item.findings.map((f, n) => {
              const line = f.line;
              const secret = typeof f.kind === "object" && "secret" in f.kind && line !== null;
              return (
                <li key={n}>
                  <p>{findingText(f)}</p>
                  {secret && (
                    <label className="reveal">
                      <input
                        type="checkbox"
                        aria-label={`${S.revealSecret} ${revealLine(line)}`}
                        checked={state.revealed.has(revealKey(item.id, line))}
                        disabled={!checked}
                        onChange={() => dispatch({ type: "toggleReveal", id: item.id, line })}
                      />
                      {S.revealSecret}
                    </label>
                  )}
                </li>
              );
            })}
          </ul>
        )}
        {hasSecret && <p className="muted note">{S.secretLimit}</p>}
      </section>
    </aside>
  );
}
