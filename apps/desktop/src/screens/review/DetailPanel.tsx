import type { Dispatch } from 'react';
import { mapping, revealKey } from '../../logic';
import type { Action, State } from '../../state';
import {
  availableOn,
  categoryLabel,
  findingText,
  itemDescription,
  originText,
  presenceText,
  reasonText,
  revealSecretLine,
  S,
} from '../../strings';
import type { Finding, Item } from '../../types';
import {
  CategoryIcon,
  CheckCircle,
  Duplicate,
  ExclamationCircle,
  InfoCircle,
  NoSign,
  WarningTriangle,
} from '../icons';

function FindingIcon({ finding }: { finding: Finding }) {
  const k = finding.kind;
  if (k === 'unreadable') return <ExclamationCircle className="symbol finding-icon unreadable" />;
  if ('secret' in k) return <WarningTriangle className="symbol finding-icon secret" />;
  if ('duplicate' in k) return <Duplicate className="symbol finding-icon duplicate" />;
  return <InfoCircle className="symbol finding-icon info" />;
}

export function DetailPanel({
  item,
  state,
  dispatch,
}: {
  item: Item | null;
  state: State;
  dispatch: Dispatch<Action>;
}) {
  if (item === null)
    return (
      <aside className="detail inspector empty-state">
        <p className="muted">{S.selectItem}</p>
      </aside>
    );
  const a = state.avail[item.id];
  const reason =
    a !== undefined && typeof a === 'object' && 'unavailable' in a ? a.unavailable : null;
  const mapped = mapping(a);
  const source = 'package' in item.details ? item.details.package.manager : null;
  const checked = state.checked.has(item.id);
  const description = itemDescription(item);
  const hasSecret = item.findings.some((f) => typeof f.kind === 'object' && 'secret' in f.kind);
  return (
    <aside className="detail inspector">
      <header className="inspector-header">
        <CategoryIcon category={item.category} />
        <div className="inspector-title">
          <h2 className="selectable">{item.name}</h2>
          <p className="muted note">
            {categoryLabel(item.category)}
            {item.version !== null && (
              <>
                {' '}
                · <span className="item-version">{item.version}</span>
              </>
            )}
          </p>
        </div>
      </header>
      <section>
        <h3 className="inspector-heading">{S.description}</h3>
        <p className={`selectable ${description === null ? 'muted' : ''}`}>
          {description ?? S.noDescription}
        </p>
      </section>
      <section>
        <h3 className="inspector-heading">{S.origin}</h3>
        <p className="mono origin">{originText(item.origin)}</p>
      </section>
      <section>
        <h3 className="inspector-heading">{S.availability}</h3>
        <p className={`availability ${reason === null ? 'ok' : 'no'}`}>
          {reason === null ? <CheckCircle /> : <NoSign />}
          <span>
            {reason === null
              ? state.target
                ? availableOn(state.target)
                : S.available
              : reasonText(reason)}
          </span>
        </p>
        {mapped !== null && (
          <>
            {/* Tool and package names stay as they are typed (spec §11). */}
            <p className="mapping selectable">
              {source !== null && (
                <>
                  {source} <code>{item.name}</code> →{' '}
                </>
              )}
              {mapped.manager} <code>{mapped.name}</code>
            </p>
            <p className="muted note mapping-note">{S.mappingNote}</p>
          </>
        )}
      </section>
      {state.presence[item.id] !== undefined && (
        <section>
          <h3 className="inspector-heading">{S.thisMachine}</h3>
          <p className={`presence ${state.presence[item.id]}`}>
            {presenceText(state.presence[item.id])}
          </p>
        </section>
      )}
      <section>
        <h3 className="inspector-heading">{S.findings}</h3>
        {item.findings.length === 0 ? (
          <p className="muted">{S.noFindings}</p>
        ) : (
          <ul className="group findings">
            {item.findings.map((f, n) => {
              const line = f.line;
              const secret = typeof f.kind === 'object' && 'secret' in f.kind && line !== null;
              return (
                <li key={n} className="finding">
                  <div className="finding-text">
                    <FindingIcon finding={f} />
                    <p className="selectable">{findingText(f, state.opened !== null)}</p>
                  </div>
                  {secret && state.opened === null && (
                    <label className="reveal">
                      <span>{S.revealSecret}</span>
                      <input
                        type="checkbox"
                        role="switch"
                        className="switch"
                        aria-label={revealSecretLine(line)}
                        checked={state.revealed.has(revealKey(item.id, line))}
                        disabled={!checked}
                        onChange={() => dispatch({ type: 'toggleReveal', id: item.id, line })}
                      />
                    </label>
                  )}
                </li>
              );
            })}
          </ul>
        )}
        {hasSecret && <p className="muted note section-footer">{S.secretLimit}</p>}
      </section>
    </aside>
  );
}
