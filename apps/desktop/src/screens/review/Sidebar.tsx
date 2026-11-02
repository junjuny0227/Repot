import type { Dispatch } from "react";
import { CATEGORIES, categoryCounts, isAvailable } from "../../logic";
import type { Action, State } from "../../state";
import { S, categoryLabel, selectCategory } from "../../strings";
import type { Category } from "../../types";

interface Props {
  state: State;
  dispatch: Dispatch<Action>;
  category: Category | null;
  onCategory: (c: Category | null) => void;
}

export function Sidebar({ state, dispatch, category, onCategory }: Props) {
  const items = state.scan?.items ?? [];
  const counts = categoryCounts(items, state.checked);
  return (
    <nav className="sidebar card">
      <button className={`category-all ${category === null ? "active" : ""}`} onClick={() => onCategory(null)}>
        <span>{S.allItems}</span>
        <span className="count">{state.checked.size}/{items.length}</span>
      </button>
      <ul>
        {CATEGORIES.map((c) => {
          const usable = items.filter((i) => i.category === c && isAvailable(state.avail[i.id]));
          const allOn = usable.length > 0 && usable.every((i) => state.checked.has(i.id));
          return (
            <li key={c} className={`category ${category === c ? "active" : ""}`}>
              <input
                type="checkbox"
                aria-label={selectCategory(categoryLabel(c))}
                checked={allOn}
                disabled={usable.length === 0}
                onChange={() => dispatch({ type: "setCategory", category: c, on: !allOn })}
              />
              <button className="link" onClick={() => onCategory(c)}>{categoryLabel(c)}</button>
              <span className="count">{counts[c].checked}/{counts[c].total}</span>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
