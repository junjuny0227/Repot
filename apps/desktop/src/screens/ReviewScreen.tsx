import { useMemo, useRef, useState, type Dispatch } from "react";
import { availabilityFor } from "../api";
import { errorMessage } from "../errors";
import { filterItems, type Filter } from "../logic";
import { canExport, type Action, type State } from "../state";
import { S, selectedCount, statusText, withCount } from "../strings";
import type { Category, Os } from "../types";
import { TargetSelect } from "./TargetSelect";
import { Sidebar } from "./review/Sidebar";
import { ItemList } from "./review/ItemList";
import { DetailPanel } from "./review/DetailPanel";

export function ReviewScreen({ state, dispatch }: { state: State; dispatch: Dispatch<Action> }) {
  const [filter, setFilter] = useState<Filter>({ query: "", onlyAvailable: false, onlyWarnings: false });
  const [category, setCategory] = useState<Category | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const request = useRef(0);
  const all = state.scan?.items ?? [];
  const items = useMemo(
    () => filterItems(all, state.avail, filter).filter((i) => category === null || i.category === category),
    [all, state.avail, filter, category],
  );
  const selected = all.find((i) => i.id === selectedId) ?? null;
  const notDetected = state.providers.filter((p) => p.status === "notDetected");
  const failed = state.providers.filter((p) => p.status === "timedOut" || (typeof p.status === "object" && "failed" in p.status));

  const changeTarget = (target: Os) => {
    // Only the latest request may update the state if the user changes the target quickly.
    const id = ++request.current;
    availabilityFor(target)
      .then((avail) => { if (id === request.current) dispatch({ type: "setTarget", target, avail }); })
      .catch((e: unknown) => dispatch({ type: "error", message: errorMessage(e) }));
  };

  return (
    <section className="screen review">
      <header className="toolbar">
        <h1>{S.reviewTitle}</h1>
        <TargetSelect value={state.target} onChange={changeTarget} />
        <span className="muted">{selectedCount(state.checked.size, all.length)}</span>
        <button className="primary" disabled={!canExport(state)} onClick={() => dispatch({ type: "goExport" })}>{S.nextExport}</button>
      </header>
      <div className="review-body">
        <Sidebar state={state} dispatch={dispatch} category={category} onCategory={setCategory} />
        <ItemList items={items} state={state} dispatch={dispatch} filter={filter} onFilter={setFilter} selectedId={selectedId} onSelect={setSelectedId} />
        <DetailPanel item={selected} state={state} dispatch={dispatch} />
      </div>
      <footer className="provider-groups">
        <details>
          <summary>{withCount(S.notDetected, notDetected.length)}</summary>
          <ul>{notDetected.map((p) => <li key={p.id} className="mono">{p.id}</li>)}</ul>
        </details>
        <details>
          <summary>{withCount(S.failed, failed.length)}</summary>
          <ul>
            {failed.map((p) => (
              <li key={p.id}>
                <span className="mono">{p.id}</span>: {statusText(p.status)}
                {typeof p.status === "object" && "failed" in p.status && (
                  <details>
                    <summary>{S.failedDetail}</summary>
                    <pre>{p.status.failed.detail}</pre>
                  </details>
                )}
              </li>
            ))}
          </ul>
        </details>
      </footer>
    </section>
  );
}
