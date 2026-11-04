import type { Dispatch } from 'react';
import { type Filter, isAvailable, mapping } from '../../logic';
import type { Action, State } from '../../state';
import { itemDescription, mappedTo, S } from '../../strings';
import type { Item } from '../../types';
import { Duplicate, Magnifier, NoSign, WarningTriangle } from '../icons';

interface Props {
  items: Item[];
  state: State;
  dispatch: Dispatch<Action>;
  filter: Filter;
  onFilter: (f: Filter) => void;
  selectedId: string | null;
  onSelect: (id: string) => void;
}

const hasKind = (item: Item, kind: 'secret' | 'duplicate'): boolean =>
  item.findings.some((f) => typeof f.kind === 'object' && kind in f.kind);

export function ItemList({
  items,
  state,
  dispatch,
  filter,
  onFilter,
  selectedId,
  onSelect,
}: Props) {
  return (
    <section className="item-list">
      <div className="filters">
        <label className="search-field">
          <Magnifier />
          <input
            type="search"
            placeholder={S.search}
            aria-label={S.search}
            value={filter.query}
            onChange={(e) => onFilter({ ...filter, query: e.target.value })}
          />
        </label>
        <label className="check-label">
          <input
            type="checkbox"
            checked={filter.onlyAvailable}
            onChange={(e) => onFilter({ ...filter, onlyAvailable: e.target.checked })}
          />
          {S.onlyAvailable}
        </label>
        <label className="check-label">
          <input
            type="checkbox"
            checked={filter.onlyWarnings}
            onChange={(e) => onFilter({ ...filter, onlyWarnings: e.target.checked })}
          />
          {S.onlyWarnings}
        </label>
        {Object.keys(state.presence).length > 0 && (
          <label className="check-label">
            <input
              type="checkbox"
              checked={filter.onlyMissing}
              onChange={(e) => onFilter({ ...filter, onlyMissing: e.target.checked })}
            />
            {S.onlyMissing}
          </label>
        )}
      </div>
      <div className="item-scroll">
        {items.length === 0 ? (
          <p className="muted empty">{S.noItems}</p>
        ) : (
          <ul className="group items">
            {items.map((item) => {
              const available = isAvailable(state.avail[item.id]);
              const mapped = mapping(state.avail[item.id]);
              return (
                <li
                  key={item.id}
                  data-id={item.id}
                  className={`item ${item.id === selectedId ? 'selected' : ''} ${available ? '' : 'unavailable'}`}
                >
                  <input
                    type="checkbox"
                    aria-label={item.name}
                    checked={state.checked.has(item.id)}
                    disabled={!available}
                    onChange={() => dispatch({ type: 'toggleItem', id: item.id })}
                  />
                  <button
                    className="item-body"
                    aria-pressed={item.id === selectedId}
                    onClick={() => onSelect(item.id)}
                  >
                    <span className="item-title">
                      <span className="item-name">{item.name}</span>
                      {item.version !== null && (
                        <span className="item-version">{item.version}</span>
                      )}
                      {mapped !== null && (
                        <span className="item-mapped" title={mappedTo(mapped.manager, mapped.name)}>
                          → {mapped.name}
                        </span>
                      )}
                      {state.presence[item.id] === 'present' && (
                        <span className="item-presence present">{S.badgePresent}</span>
                      )}
                      {state.presence[item.id] === 'differs' && (
                        <span className="item-presence differs">{S.badgeDiffers}</span>
                      )}
                      <span className="badges">
                        {hasKind(item, 'secret') && (
                          <span
                            className="badge"
                            role="img"
                            title={S.badgeSecret}
                            aria-label={S.badgeSecret}
                          >
                            <WarningTriangle className="symbol secret" />
                          </span>
                        )}
                        {hasKind(item, 'duplicate') && (
                          <span
                            className="badge"
                            role="img"
                            title={S.badgeDuplicate}
                            aria-label={S.badgeDuplicate}
                          >
                            <Duplicate className="symbol duplicate" />
                          </span>
                        )}
                        {!available && (
                          <span
                            className="badge"
                            role="img"
                            title={S.badgeUnavailable}
                            aria-label={S.badgeUnavailable}
                          >
                            <NoSign className="symbol unavailable" />
                          </span>
                        )}
                      </span>
                    </span>
                    <span className="item-description">
                      {itemDescription(item) ?? S.noDescription}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </section>
  );
}
