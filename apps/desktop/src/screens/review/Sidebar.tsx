import type { Dispatch } from 'react';
import { CATEGORIES, categoryCounts, isAvailable } from '../../logic';
import type { Action, State } from '../../state';
import { categoryLabel, S, selectCategory } from '../../strings';
import type { Category } from '../../types';
import { CategoryIcon } from '../icons';

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
    <nav className="sidebar">
      <button
        type="button"
        className={`source-item category-all ${category === null ? 'active' : ''}`}
        aria-current={category === null ? 'true' : undefined}
        onClick={() => onCategory(null)}
      >
        <CategoryIcon category="all" />
        <span className="source-label">{S.allItems}</span>
        <span className="count">
          {state.checked.size}/{items.length}
        </span>
      </button>
      <ul>
        {CATEGORIES.map((c) => {
          const usable = items.filter((i) => i.category === c && isAvailable(state.avail[i.id]));
          const allOn = usable.length > 0 && usable.every((i) => state.checked.has(i.id));
          return (
            <li key={c} className={`source-item category ${category === c ? 'active' : ''}`}>
              <button
                type="button"
                className="source-button"
                aria-current={category === c ? 'true' : undefined}
                onClick={() => onCategory(c)}
              >
                <CategoryIcon category={c} />
                <span className="source-label">{categoryLabel(c)}</span>
              </button>
              <span className="count">
                {counts[c].checked}/{counts[c].total}
              </span>
              <input
                type="checkbox"
                aria-label={selectCategory(categoryLabel(c))}
                checked={allOn}
                disabled={usable.length === 0}
                onChange={() => dispatch({ type: 'setCategory', category: c, on: !allOn })}
              />
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
