import type { Dispatch } from "react";
import { isAvailable, type Filter } from "../../logic";
import type { Action, State } from "../../state";
import { S } from "../../strings";
import type { Item } from "../../types";

interface Props {
  items: Item[];
  state: State;
  dispatch: Dispatch<Action>;
  filter: Filter;
  onFilter: (f: Filter) => void;
  selectedId: string | null;
  onSelect: (id: string) => void;
}

const hasKind = (item: Item, kind: "secret" | "duplicate"): boolean =>
  item.findings.some((f) => typeof f.kind === "object" && kind in f.kind);

export function ItemList({ items, state, dispatch, filter, onFilter, selectedId, onSelect }: Props) {
  return (
    <section className="item-list card">
      <div className="filters">
        <input
          type="search"
          placeholder={S.search}
          aria-label={S.search}
          value={filter.query}
          onChange={(e) => onFilter({ ...filter, query: e.target.value })}
        />
        <label>
          <input type="checkbox" checked={filter.onlyAvailable} onChange={(e) => onFilter({ ...filter, onlyAvailable: e.target.checked })} />
          {S.onlyAvailable}
        </label>
        <label>
          <input type="checkbox" checked={filter.onlyWarnings} onChange={(e) => onFilter({ ...filter, onlyWarnings: e.target.checked })} />
          {S.onlyWarnings}
        </label>
      </div>
      {items.length === 0 ? (
        <p className="muted empty">{S.noItems}</p>
      ) : (
        <ul className="items">
          {items.map((item) => {
            const available = isAvailable(state.avail[item.id]);
            return (
              <li key={item.id} data-id={item.id} className={`item ${item.id === selectedId ? "selected" : ""} ${available ? "" : "unavailable"}`}>
                <input
                  type="checkbox"
                  aria-label={item.name}
                  checked={state.checked.has(item.id)}
                  disabled={!available}
                  onChange={() => dispatch({ type: "toggleItem", id: item.id })}
                />
                <button className="item-body" onClick={() => onSelect(item.id)}>
                  <span className="item-title">
                    <span className="item-name">{item.name}</span>
                    {item.version !== null && <span className="item-version">{item.version}</span>}
                    <span className="badges">
                      {hasKind(item, "secret") && <span title={S.badgeSecret} aria-label={S.badgeSecret}>⚠️</span>}
                      {hasKind(item, "duplicate") && <span title={S.badgeDuplicate} aria-label={S.badgeDuplicate}>🔁</span>}
                      {!available && <span title={S.badgeUnavailable} aria-label={S.badgeUnavailable}>🚫</span>}
                    </span>
                  </span>
                  <span className="item-description muted">{item.description ?? S.noDescription}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
