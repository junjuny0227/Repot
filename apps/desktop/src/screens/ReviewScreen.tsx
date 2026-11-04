import { type Dispatch, useMemo, useRef, useState } from 'react';
import { availabilityFor, presenceFor } from '../api';
import { errorMessage } from '../errors';
import { choosesPackages, defaultTarget, type Filter, filterItems } from '../logic';
import { type Action, canExport, type State, sessionSource } from '../state';
import {
  comparedSummaryParts,
  openedFromParts,
  S,
  selectedCount,
  skippedItemsText,
  statusText,
  withCount,
} from '../strings';
import type { AvailabilityMap, Category, Os, PackageManager, PresenceMap, Target } from '../types';
import { LanguageSelect } from './LanguageSelect';
import { DetailPanel } from './review/DetailPanel';
import { ItemList } from './review/ItemList';
import { Sidebar } from './review/Sidebar';
import { PackageSelect, TargetSelect } from './TargetSelect';

export function ReviewScreen({
  state,
  dispatch,
  onCompare,
}: {
  state: State;
  dispatch: Dispatch<Action>;
  onCompare: () => void;
}) {
  const [filter, setFilter] = useState<Filter>({
    query: '',
    onlyAvailable: false,
    onlyWarnings: false,
    onlyMissing: false,
  });
  const [category, setCategory] = useState<Category | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const request = useRef(0);
  const all = state.scan?.items ?? [];
  const items = useMemo(
    () =>
      filterItems(all, state.avail, filter, state.presence).filter(
        (i) => category === null || i.category === category,
      ),
    [all, state.avail, filter, category, state.presence],
  );
  const selected = all.find((i) => i.id === selectedId) ?? null;
  const notDetected = state.providers.filter((p) => p.status === 'notDetected');
  const failed = state.providers.filter(
    (p) => p.status === 'timedOut' || (typeof p.status === 'object' && 'failed' in p.status),
  );

  // Re-judge against the new destination. Only the latest request may update the state if the user changes it quickly.
  // After a comparison, presence is judged again too (empty for another OS).
  const rejudge = (
    target: Target,
    apply: (avail: AvailabilityMap, presence?: PresenceMap) => Action,
  ) => {
    const id = ++request.current;
    Promise.all([
      availabilityFor(target),
      state.compared ? presenceFor(target) : Promise.resolve(undefined),
    ])
      .then(([avail, presence]) => {
        if (id === request.current) dispatch(apply(avail, presence));
      })
      .catch((e: unknown) => dispatch({ type: 'error', message: errorMessage(e) }));
  };
  // A new destination starts again from its default package manager.
  const changeTarget = (target: Os) =>
    rejudge(defaultTarget(target), (avail, presence) => ({
      type: 'setTarget',
      target,
      avail,
      presence,
    }));
  const changePackages = (packages: PackageManager) => {
    if (state.target !== null)
      rejudge({ os: state.target, packages }, (avail, presence) => ({
        type: 'setPackages',
        packages,
        avail,
        presence,
      }));
  };

  return (
    <section className="screen review">
      <aside className="sidebar-pane">
        <div className="sidebar-titlebar" data-tauri-drag-region="deep" />
        <Sidebar state={state} dispatch={dispatch} category={category} onCategory={setCategory} />
        {state.opened !== null && (
          <footer className="provider-groups opened-from">
            {(() => {
              const { before, date, after } = openedFromParts(state.opened);
              return (
                <p>
                  {before}
                  <span className="nowrap">{date}</span>
                  {after}
                </p>
              );
            })()}
            {state.opened.skippedItems > 0 && (
              <p className="muted">{skippedItemsText(state.opened.skippedItems)}</p>
            )}
            {state.compared && (
              <p className="muted">
                {Object.keys(state.presence).length > 0
                  ? comparedSummaryParts(state.presence).map((part, n, all) => (
                      // The separator ends the previous pair, so a wrapped line never starts with "·".
                      <span key={n}>
                        <span className="nowrap">
                          {part}
                          {n < all.length - 1 && ' ·'}
                        </span>
                        {n < all.length - 1 && ' '}
                      </span>
                    ))
                  : S.compareOtherOs}
              </p>
            )}
            <button className="compare-button" onClick={onCompare}>
              {state.compared ? S.compareAgain : S.compareButton}
            </button>
          </footer>
        )}
        {state.opened === null && (
          <footer className="provider-groups">
            <details>
              <summary>{withCount(S.notDetected, notDetected.length)}</summary>
              <ul>
                {notDetected.map((p) => (
                  <li key={p.id} className="mono">
                    {p.id}
                  </li>
                ))}
              </ul>
            </details>
            <details>
              <summary>{withCount(S.failed, failed.length)}</summary>
              <ul>
                {failed.map((p) => (
                  <li key={p.id}>
                    <span className="mono">{p.id}</span>: {statusText(p.status)}
                    {typeof p.status === 'object' && 'failed' in p.status && (
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
        )}
        <div className="sidebar-language">
          <LanguageSelect />
        </div>
      </aside>
      <div className="review-main">
        <header className="toolbar" data-tauri-drag-region="deep">
          <div className="toolbar-title">
            <h1>{S.reviewTitle}</h1>
            <p className="subtitle">{selectedCount(state.checked.size, all.length)}</p>
          </div>
          <div className="spacer" />
          <TargetSelect value={state.target} onChange={changeTarget} />
          {choosesPackages(sessionSource(state), state.target) && (
            <PackageSelect value={state.packages} onChange={changePackages} />
          )}
          <button
            className="primary"
            disabled={!canExport(state)}
            onClick={() => dispatch({ type: 'goExport' })}
          >
            {S.nextExport}
          </button>
        </header>
        <div className="review-body">
          <ItemList
            items={items}
            state={state}
            dispatch={dispatch}
            filter={filter}
            onFilter={setFilter}
            selectedId={selectedId}
            onSelect={setSelectedId}
          />
          <DetailPanel item={selected} state={state} dispatch={dispatch} />
        </div>
      </div>
    </section>
  );
}
